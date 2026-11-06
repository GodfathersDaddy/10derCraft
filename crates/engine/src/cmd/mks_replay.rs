//! Validated, immutable replay data. Geometry and trajectory parsing happen once,
//! not once per displayed frame. Interpolation is presentation-only.
use crate::schema::{TypedContract, output_schema_for};
use crate::{Result, Session, params::bad};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use solidcraft_doc::{IDENTITY, Mat, apply_point, apply_vector, mat_mul};
use solidcraft_geom::{Aabb3, Mesh, Vec3};
use solidcraft_render::{Quat, Rgb};
use std::sync::Arc;
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PoseParams {
    time_s: f64,
}
#[derive(Serialize, JsonSchema)]
struct PoseTransform {
    source: String,
    world: Mat,
}
#[derive(Serialize, JsonSchema)]
struct PoseReply {
    time_s: f64,
    transforms: Vec<PoseTransform>,
    interpolation: String,
    cad_changed: bool,
}
const POSE: TypedContract<PoseParams> = TypedContract::new();
pub static COMMANDS: &[super::CommandSpec] =
    &[super::CommandSpec::new("mks.result.pose", "Read Interpolated Dynamics Pose", crate::checked!("mks.result.pose", pose))
        .icon("move")
        .contract(&POSE)
        .output(output_schema_for::<PoseReply>)
        .noundo()];
fn pose(s: &mut Session, p: &Value) -> Result<Value> {
    let p: PoseParams = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    let replay = prepare(s)?;
    if !p.time_s.is_finite() || p.time_s < replay.times[0] || p.time_s > *replay.times.last().ok_or_else(|| error("times missing"))? {
        return Err(error("time must lie inside the stored trajectory"));
    }
    Ok(
        json!({"time_s":p.time_s,"transforms":replay.meshes.iter().zip(replay.worlds(p.time_s)).map(|(mesh,world)|json!({"source":mesh.source,"world":world})).collect::<Vec<_>>(),
        "interpolation":if replay.elastic.is_empty(){"joint_relative_slerp_anchor_preserving"}else{"reference_slerp_modal_linear"},"cad_changed":false}),
    )
}

pub struct ReplayMesh {
    pub source: String,
    pub mesh: Arc<Mesh>,
    pub color: Rgb,
    pub block: usize,
    pub world: Mat,
    attachment: Option<String>,
}
struct Pose {
    position: Vec3,
    rotation: Quat,
}
struct Block {
    com: Vec3,
    poses: Vec<Pose>,
}
struct Link {
    parent: usize,
    child: usize,
    pivot: Vec3,
    slider: bool,
}
pub struct ReplayPort {
    pub block: usize,
    pub origin: Vec3,
    pub axes: [Vec3; 3],
}
pub struct AssemblyReplay {
    elastic: std::collections::BTreeMap<usize, Elastic>,
    times: Vec<f64>,
    pub meshes: Vec<ReplayMesh>,
    pub ports: Vec<ReplayPort>,
    pub bounds: Aabb3,
    blocks: Vec<Block>,
    links: Vec<Link>,
}
struct Elastic {
    source: String,
    basis: Vec<Vec<f64>>,
    states: Vec<Vec<f64>>,
    attachments: std::collections::BTreeMap<String, Block>,
}
fn error(message: impl Into<String>) -> crate::EngineError {
    bad("mks.replay", message)
}
fn vector(v: &Value) -> Result<Vec3> {
    serde_json::from_value(v.clone()).map_err(|e| error(e.to_string()))
}
fn poses(value: &Value, count: usize) -> Result<Vec<Pose>> {
    let rows = value.as_array().filter(|v| v.len() == count).ok_or_else(|| error("trajectory missing"))?;
    rows.iter()
        .map(|row| {
            let row: Vec<f64> = serde_json::from_value(row.clone()).map_err(|e| error(e.to_string()))?;
            if row.len() != 12 || row.iter().any(|v| !v.is_finite()) {
                return Err(error("invalid pose"));
            }
            Ok(Pose {
                position: Vec3::new(row[0], row[1], row[2]) * 1000.,
                rotation: Quat::from_basis(Vec3::new(row[3], row[6], row[9]), Vec3::new(row[4], row[7], row[10]), Vec3::new(row[5], row[8], row[11])),
            })
        })
        .collect()
}
pub(super) const MAX_PREVIEW_TRIANGLES: usize = 250_000;
pub(super) const MAX_PREVIEW_VERTICES: usize = 500_000;

pub fn prepare(s: &mut Session) -> Result<AssemblyReplay> {
    let state = s.execute("mks.job.state", &json!({}))?;
    if state["result_current"] != true {
        return Err(error("current dynamics result required"));
    }
    let graph = s.execute("mks.model", &json!({}))?;
    let data = &state["data"];
    let times: Vec<f64> = serde_json::from_value(data["result"]["times_s"].clone()).map_err(|e| error(e.to_string()))?;
    if !(2..=1001).contains(&times.len()) || times.iter().any(|v| !v.is_finite()) || times.windows(2).any(|p| p[1] <= p[0]) {
        return Err(error("invalid replay times"));
    }
    let native = s.model.state();
    let mut out = AssemblyReplay {
        elastic: Default::default(),
        times,
        meshes: Vec::new(),
        ports: Vec::new(),
        blocks: Vec::new(),
        links: Vec::new(),
        bounds: Aabb3::EMPTY,
    };
    let mut triangle_count = 0;
    let mut vertex_count = 0;
    for (index, block) in graph["blocks"].as_array().into_iter().flatten().enumerate() {
        let mut item = Block { com: Vec3::ZERO, poses: Vec::new() };
        if block["grounded"] != true {
            let id = block["id"].as_str().ok_or_else(|| error("block id missing"))?;
            let body = data["job"]["model"]["bodies"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|b| b["id"] == id)
                .ok_or_else(|| error("body definition missing"))?;
            item.com = vector(&body["com_world_m"])? * 1000.;
            let rows =
                data["result"]["trajectories"][id].as_array().filter(|v| v.len() == out.times.len()).ok_or_else(|| error("trajectory missing"))?;
            for row in rows {
                let row: Vec<f64> = serde_json::from_value(row.clone()).map_err(|e| error(e.to_string()))?;
                if row.len() != 12 || row.iter().any(|v| !v.is_finite()) {
                    return Err(error("invalid pose"));
                }
                item.poses.push(Pose {
                    position: Vec3::new(row[0], row[1], row[2]) * 1000.,
                    rotation: Quat::from_basis(
                        Vec3::new(row[3], row[6], row[9]),
                        Vec3::new(row[4], row[7], row[10]),
                        Vec3::new(row[5], row[8], row[11]),
                    ),
                });
            }
        }
        out.blocks.push(item);
        let block_id = block["id"].as_str().ok_or_else(|| error("block id missing"))?;
        let elastic = &data["result"]["flexible_states"][block_id];
        if let Some(id) = elastic["component_id"].as_str() {
            let record = s.doc.fem_models.as_ref().and_then(|v| v["models"].get(id)).ok_or_else(|| error("ROM missing for flexible replay"))?;
            let r = &record["result"];
            let positions: Vec<Vec3> = serde_json::from_value(r["nodes_mm"].clone()).map_err(|e| error(e.to_string()))?;
            let triangles: Vec<[u32; 3]> = serde_json::from_value(r["surface_triangles"].clone()).map_err(|e| error(e.to_string()))?;
            let basis: Vec<Vec<f64>> = serde_json::from_value(r["rom"]["basis"].clone()).map_err(|e| error(e.to_string()))?;
            let states: Vec<Vec<f64>> = serde_json::from_value(elastic["modal_coordinates"].clone()).map_err(|e| error(e.to_string()))?;
            if positions.len() > 1500
                || basis.len() != 3 * positions.len()
                || states.len() != out.times.len()
                || triangles.iter().flatten().any(|i| *i as usize >= positions.len())
                || states.iter().any(|q| q.is_empty() || q.len() > 72 || q.iter().any(|v| !v.is_finite()))
                || basis.iter().any(|v| v.len() != states[0].len() || v.iter().any(|v| !v.is_finite()))
            {
                return Err(error("invalid flexible replay matrices"));
            }
            triangle_count += triangles.len();
            vertex_count += positions.len();
            if triangle_count > MAX_PREVIEW_TRIANGLES || vertex_count > MAX_PREVIEW_VERTICES {
                return Err(error("replay geometry exceeds 250000 triangles / 500000 total vertices"));
            }
            let mesh = Mesh { positions, triangles, ..Default::default() };
            let mut attachments = std::collections::BTreeMap::new();
            for (port, state) in elastic["attachment_states"].as_object().into_iter().flatten() {
                attachments
                    .insert(port.clone(), Block { com: vector(&state["com_world_m"])? * 1000., poses: poses(&state["poses"], out.times.len())? });
            }
            out.elastic.insert(index, Elastic { source: id.into(), basis, states, attachments });
            out.meshes.push(ReplayMesh {
                source: id.into(),
                mesh: Arc::new(mesh),
                color: crate::view::Palette::DEFAULT.body,
                block: index,
                world: IDENTITY,
                attachment: None,
            });
        }
        {
            for placed in block["bodies"].as_array().into_iter().flatten() {
                let mut attachment = None;
                if let Some(elastic) = out.elastic.get(&index) {
                    let record = s.doc.fem_models.as_ref().and_then(|v| v["models"].get(&elastic.source)).ok_or_else(|| error("ROM missing"))?;
                    if record["owners"].as_array().into_iter().flatten().any(|owner| owner == placed) {
                        continue;
                    }
                    let binding = record["rigid_attachments"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .find(|a| a["owner"] == *placed)
                        .and_then(|a| a["interface"].as_str())
                        .filter(|id| elastic.attachments.contains_key(*id))
                        .ok_or_else(|| error("rigid attachment replay missing"))?;
                    attachment = Some(binding.to_owned());
                }
                let body = native
                    .bodies
                    .iter()
                    .find(|b| {
                        Some(b.name.as_str()) == placed["source"].as_str() && Some(s.doc.model_body_component(b)) == placed["component"].as_u64()
                    })
                    .ok_or_else(|| error("CAD source missing"))?;
                let mesh = body.mesh();
                triangle_count += mesh.triangles.len();
                vertex_count += mesh.positions.len();
                if triangle_count > MAX_PREVIEW_TRIANGLES || vertex_count > MAX_PREVIEW_VERTICES {
                    return Err(error("replay geometry exceeds 250000 triangles / 500000 total vertices"));
                }
                let color = s
                    .doc
                    .body_color(body)
                    .map(|c| Rgb((c[0] * 255.).round() as u8, (c[1] * 255.).round() as u8, (c[2] * 255.).round() as u8))
                    .unwrap_or(crate::view::Palette::DEFAULT.body);
                out.meshes.push(ReplayMesh {
                    source: body.name.clone(),
                    mesh,
                    color,
                    block: index,
                    world: serde_json::from_value(placed["world"].clone()).map_err(|e| error(e.to_string()))?,
                    attachment,
                });
            }
        }
        for p in block["ports"].as_array().into_iter().flatten().filter(|_| !out.elastic.contains_key(&index)) {
            out.ports.push(ReplayPort {
                block: index,
                origin: vector(&p["frame"]["origin"])?,
                axes: [vector(&p["frame"]["axes"][0])?, vector(&p["frame"]["axes"][1])?, vector(&p["frame"]["axes"][2])?],
            });
        }
    }
    // Orient an arbitrary CAD joint order from ground towards the leaves.
    let ids: std::collections::BTreeMap<_, _> =
        graph["blocks"].as_array().into_iter().flatten().enumerate().filter_map(|(i, b)| b["id"].as_str().map(|id| (id.to_owned(), i))).collect();
    let mut reached: std::collections::BTreeSet<_> = out.blocks.iter().enumerate().filter(|(_, b)| b.poses.is_empty()).map(|(i, _)| i).collect();
    let mut pending = data["job"]["model"]["joints"].as_array().cloned().ok_or_else(|| error("joints missing"))?;
    for _ in 0..out.blocks.len() {
        let mut next = Vec::new();
        for joint in pending {
            let a = joint["a"].as_str().and_then(|id| ids.get(id)).copied().ok_or_else(|| error("parent body missing"))?;
            let b = joint["b"].as_str().and_then(|id| ids.get(id)).copied().ok_or_else(|| error("follower body missing"))?;
            let pair = if reached.contains(&a) && !reached.contains(&b) {
                Some((a, b))
            } else if reached.contains(&b) && !reached.contains(&a) {
                Some((b, a))
            } else {
                None
            };
            if let Some((parent, child)) = pair {
                out.links.push(Link { parent, child, pivot: vector(&joint["pivot_world_m"])? * 1000., slider: joint["kind"] == "slider" });
                reached.insert(child);
            } else {
                next.push(joint);
            }
        }
        pending = next;
        if pending.is_empty() {
            break;
        }
    }
    if !pending.is_empty() || reached.len() != out.blocks.len() {
        return Err(error("replay requires a rooted open tree"));
    }
    // Sweep local mesh boxes, not millions of transformed triangle vertices.
    let boxes: Vec<_> = out.meshes.iter().map(|m| m.mesh.bounds()).collect();
    for &time in &out.times {
        let transforms = out.worlds(time);
        for (bbox, world) in boxes.iter().zip(transforms) {
            for x in [bbox.min.x, bbox.max.x] {
                for y in [bbox.min.y, bbox.max.y] {
                    for z in [bbox.min.z, bbox.max.z] {
                        out.bounds.add(apply_point(&world, Vec3::new(x, y, z)));
                    }
                }
            }
        }
    }
    if out.bounds.is_empty() {
        return Err(error("empty replay geometry"));
    }
    Ok(out)
}
impl AssemblyReplay {
    pub fn times(&self) -> &[f64] {
        &self.times
    }
    /// SLERP of joint-relative rotations; translations preserve joint anchors.
    /// Stored solver samples and measured forces are never modified.
    pub fn deltas(&self, time: f64) -> Vec<Mat> {
        let time = if time.is_finite() { time } else { self.times[0] };
        let a = self.times.partition_point(|t| *t <= time).saturating_sub(1).min(self.times.len() - 1);
        let b = (a + 1).min(self.times.len() - 1);
        let u = if a == b { 0. } else { ((time - self.times[a]) / (self.times[b] - self.times[a])).clamp(0., 1.) };
        let mut matrices: Vec<Mat> = self
            .blocks
            .iter()
            .map(|block| {
                let (Some(p), Some(q)) = (block.poses.get(a), block.poses.get(b)) else { return IDENTITY };
                let (x, y, z) = p.rotation.slerp(q.rotation, u).basis();
                let mut matrix = IDENTITY;
                for (i, v) in [x, y, z].iter().enumerate() {
                    matrix[i] = [v.x, v.y, v.z, 0.];
                }
                let translation = p.position * (1. - u) + q.position * u - apply_vector(&matrix, block.com);
                matrix[3] = [translation.x, translation.y, translation.z, 1.];
                matrix
            })
            .collect();
        if u > 0. && u < 1. && self.elastic.is_empty() {
            let first = self.raw_deltas(a);
            let last = self.raw_deltas(b);
            for link in &self.links {
                let parent = matrices[link.parent];
                let relative =
                    relative_rotation(&first[link.parent], &first[link.child]).slerp(relative_rotation(&last[link.parent], &last[link.child]), u);
                let (x, y, z) = relative.basis();
                let mut child = IDENTITY;
                for (i, axis) in [x, y, z].iter().enumerate() {
                    let v = apply_vector(&parent, *axis);
                    child[i] = [v.x, v.y, v.z, 0.];
                }
                let offset = if link.slider {
                    let local = |p: &Mat, c: &Mat| {
                        let v = apply_point(c, link.pivot) - apply_point(p, link.pivot);
                        Vec3::new(
                            Vec3::new(p[0][0], p[0][1], p[0][2]).dot(v),
                            Vec3::new(p[1][0], p[1][1], p[1][2]).dot(v),
                            Vec3::new(p[2][0], p[2][1], p[2][2]).dot(v),
                        )
                    };
                    apply_vector(
                        &parent,
                        local(&first[link.parent], &first[link.child]) * (1. - u) + local(&last[link.parent], &last[link.child]) * u,
                    )
                } else {
                    Vec3::ZERO
                };
                let t = apply_point(&parent, link.pivot) + offset - apply_vector(&child, link.pivot);
                child[3] = [t.x, t.y, t.z, 1.];
                matrices[link.child] = child;
            }
        }
        matrices
    }
    fn raw_deltas(&self, index: usize) -> Vec<Mat> {
        self.blocks
            .iter()
            .map(|b| {
                let Some(pose) = b.poses.get(index) else { return IDENTITY };
                let (x, y, z) = pose.rotation.basis();
                let mut m = IDENTITY;
                for (i, v) in [x, y, z].iter().enumerate() {
                    m[i] = [v.x, v.y, v.z, 0.];
                }
                let t = pose.position - apply_vector(&m, b.com);
                m[3] = [t.x, t.y, t.z, 1.];
                m
            })
            .collect()
    }
    pub fn worlds(&self, time: f64) -> Vec<Mat> {
        let deltas = self.deltas(time);
        let time = if time.is_finite() { time } else { self.times[0] };
        let a = self.times.partition_point(|t| *t <= time).saturating_sub(1).min(self.times.len() - 1);
        let b = (a + 1).min(self.times.len() - 1);
        let u = if a == b { 0. } else { ((time - self.times[a]) / (self.times[b] - self.times[a])).clamp(0., 1.) };
        self.meshes
            .iter()
            .map(|m| {
                if let Some(attachment) = m.attachment.as_ref().and_then(|id| self.elastic.get(&m.block)?.attachments.get(id)) {
                    let p = &attachment.poses[a];
                    let q = &attachment.poses[b];
                    let (x, y, z) = p.rotation.slerp(q.rotation, u).basis();
                    let mut delta = IDENTITY;
                    for (i, v) in [x, y, z].iter().enumerate() {
                        delta[i] = [v.x, v.y, v.z, 0.];
                    }
                    let translation = p.position * (1. - u) + q.position * u - apply_vector(&delta, attachment.com);
                    delta[3] = [translation.x, translation.y, translation.z, 1.];
                    mat_mul(&delta, &m.world)
                } else {
                    mat_mul(deltas.get(m.block).unwrap_or(&IDENTITY), &m.world)
                }
            })
            .collect()
    }
    /// Bounded CPU reconstruction of the FE surface; rigid CAD meshes remain
    /// uploaded once. Interpolation changes the view, never solver histories.
    pub fn elastic_mesh(&self, index: usize, time: f64) -> Option<Mesh> {
        let source = self.meshes.get(index)?;
        let elastic = self.elastic.get(&source.block)?;
        if source.attachment.is_some() || source.source != elastic.source {
            return None;
        }
        let a = self.times.partition_point(|t| *t <= time).saturating_sub(1).min(self.times.len() - 1);
        let b = (a + 1).min(self.times.len() - 1);
        let u = if a == b { 0. } else { ((time - self.times[a]) / (self.times[b] - self.times[a])).clamp(0., 1.) };
        let q: Vec<_> = elastic.states[a].iter().zip(&elastic.states[b]).map(|(a, b)| a * (1. - u) + b * u).collect();
        let values: Vec<_> = elastic.basis.iter().map(|row| row.iter().zip(&q).map(|(v, q)| v * q).sum::<f64>() * 1000.).collect();
        let mut mesh = (*source.mesh).clone();
        for (p, v) in mesh.positions.iter_mut().zip(values.as_chunks::<3>().0) {
            *p += Vec3::new(v[0], v[1], v[2]);
        }
        mesh.normals = vec![Vec3::ZERO; mesh.positions.len()];
        for t in &mesh.triangles {
            let [a, b, c] = t.map(|i| mesh.positions[i as usize]);
            let n = (b - a).cross(c - a);
            for i in t {
                mesh.normals[*i as usize] += n;
            }
        }
        for n in &mut mesh.normals {
            *n = n.normalized().unwrap_or(Vec3::Z);
        }
        Some(mesh)
    }
}
fn relative_rotation(parent: &Mat, child: &Mat) -> Quat {
    let axes = parent[..3].iter().map(|v| Vec3::new(v[0], v[1], v[2])).collect::<Vec<_>>();
    let local = |i: usize| {
        let v = Vec3::new(child[i][0], child[i][1], child[i][2]);
        Vec3::new(axes[0].dot(v), axes[1].dot(v), axes[2].dot(v))
    };
    Quat::from_basis(local(0), local(1), local(2))
}
