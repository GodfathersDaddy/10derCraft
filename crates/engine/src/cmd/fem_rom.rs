//! Per-part/component solid FEM with associative load surfaces and bounded CMS artifacts.
use super::CommandSpec;
use crate::{
    Result, Session,
    params::bad,
    schema::{TypedContract, output_schema_for},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use solidcraft_doc::{FeatureKind, apply_point, apply_vector, expr::Kind};
use solidcraft_geom::{Aabb3, Mesh, Vec3};
use std::sync::Arc;

fn error(v: impl Into<String>) -> crate::EngineError {
    bad("fem.rom", v)
}
#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Material {
    pub young_mpa: f64,
    pub poisson_ratio: f64,
    pub density_kg_m3: f64,
}
#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ImportedMesh {
    nodes_mm: Vec<[f64; 3]>,
    tetrahedra: Vec<[u32; 4]>,
}
fn divisions() -> [u32; 3] {
    [4, 2, 2]
}
fn modes() -> usize {
    6
}
fn order() -> u32 {
    2
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Build {
    id: String,
    block: String,
    #[serde(default)]
    body: Option<String>,
    interfaces: Vec<String>,
    material: Material,
    #[serde(default = "divisions")]
    divisions: [u32; 3],
    #[serde(default = "modes")]
    modes: usize,
    #[serde(default = "order")]
    element_order: u32,
    #[serde(default)]
    mesh: Option<ImportedMesh>,
    #[serde(default)]
    mesh_path: Option<String>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Query {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    detail: bool,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Run {
    id: String,
    #[serde(default)]
    flexible: bool,
    #[serde(default)]
    force_n: Option<[f64; 3]>,
    #[serde(default)]
    moment_nm: Option<[f64; 3]>,
    #[serde(default)]
    motion: Option<Value>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Accept {
    id: String,
    job: Value,
    result: Value,
}
#[derive(Serialize, JsonSchema)]
struct Reply {
    models: Vec<Value>,
    scopes: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}
#[derive(Serialize, JsonSchema)]
struct Built {
    id: String,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Binding {
    id: String,
    enabled: bool,
}
const BINDING: TypedContract<Binding> = TypedContract::new();
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Export {
    id: String,
    path: String,
    #[serde(default)]
    overwrite: bool,
}
#[derive(Serialize, JsonSchema)]
struct Exported {
    path: String,
    bytes: usize,
}
const EXPORT: TypedContract<Export> = TypedContract::new();
const BUILD: TypedContract<Build> = TypedContract::new();
const QUERY: TypedContract<Query> = TypedContract::new();
const RUN: TypedContract<Run> = TypedContract::new();
const ACCEPT: TypedContract<Accept> = TypedContract::new();
pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("fem.mks.bind", "Use Component ROM in MBS", crate::checked!("fem.mks.bind", bind))
        .icon("joint")
        .contract(&BINDING)
        .output(output_schema_for::<Built>)
        .doc_only(),
    CommandSpec::new("fem.model.create", "Create Component FEM", crate::checked!("fem.model.create", create))
        .icon("graph")
        .contract(&BUILD)
        .output(output_schema_for::<Built>)
        .doc_only(),
    CommandSpec::new("fem.model.read", "Read Component FEM", crate::checked!("fem.model.read", read))
        .icon("list")
        .contract(&QUERY)
        .output(output_schema_for::<Reply>)
        .noundo()
        .doc_only(),
    CommandSpec::new("fem.rom.reduce", "Compute Component FEM and ROM", crate::checked!("fem.rom.reduce", run))
        .icon("graph")
        .contract(&RUN)
        .output(output_schema_for::<Accept>)
        .doc_only(),
    CommandSpec::new("fem.result.accept", "Accept Component FEM Result", crate::checked!("fem.result.accept", accept))
        .icon("graph")
        .contract(&ACCEPT)
        .output(output_schema_for::<Built>)
        .doc_only(),
    CommandSpec::new("fem.rom.export", "Export Component ROM", crate::checked!("fem.rom.export", export))
        .icon("save")
        .contract(&EXPORT)
        .output(output_schema_for::<Exported>)
        .noundo()
        .doc_only(),
];
pub(super) fn bindings(s: &Session) -> Vec<Value> {
    s.doc
        .fem_models
        .as_ref()
        .and_then(|v| v["models"].as_object())
        .into_iter()
        .flatten()
        .filter(|(_, m)| m["enabled"] == true)
        .map(|(id, m)| json!({"id":id,"block":m["block"],"signature":m["signature"]}))
        .collect()
}
pub(super) fn binding(s: &Session, block: &Value) -> Result<Option<String>> {
    let bound = bindings(s);
    let selected: Vec<_> = bound.iter().filter(|v| v["block"] == block["id"]).collect();
    if selected.len() > 1 {
        return Err(error("multiple active ROMs for one CAD/MKS block"));
    }
    let Some(binding) = selected.first() else { return Ok(None) };
    let id = binding["id"].as_str().ok_or_else(|| error("invalid ROM binding"))?;
    let record = get(s, id)?;
    if record["job"]["source"] != source(s)? {
        return Err(error(format!("{id}: ROM is stale; rebuild mesh and reduction")));
    }
    validate(&record["job"], &record["result"])?;
    if block["bodies"] != record["owners"] {
        return Err(error("ROM must cover the entire native rigid block; attached parts cannot be omitted"));
    }
    Ok(Some(id.into()))
}
fn bind(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Binding = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    let record = get(s, &p.id)?;
    let block_id = record["block"].clone();
    if p.enabled {
        if record["job"]["source"] != source(s)? {
            return Err(error("rebuild stale ROM before MKS binding"));
        }
        validate(&record["job"], &record["result"])?;
        let owners = record["owners"].clone();
        let graph = s.execute("mks.model", &json!({}))?;
        let block = graph["blocks"].as_array().into_iter().flatten().find(|b| b["id"] == block_id).ok_or_else(|| error("CAD/MKS block missing"))?;
        if block["grounded"] == true || block["bodies"] != owners {
            return Err(error("bind a complete moving CAD/MKS block; a part-only ROM cannot silently replace its rigid attachments"));
        }
    }
    let mut all = models(s);
    if p.enabled {
        for (_, m) in all["models"].as_object_mut().into_iter().flatten() {
            if m["block"] == block_id {
                m["enabled"] = json!(false);
            }
        }
    }
    all["models"][&p.id]["enabled"] = json!(p.enabled);
    save(s, all)?;
    Ok(json!({"id":p.id}))
}
pub(super) fn worker_job(s: &Session, job: &Value) -> Result<Value> {
    if job["backend"] != "exudyn_ffrf_open_chain" {
        return Ok(job.clone());
    }
    let mut components = serde_json::Map::new();
    for b in job["model"]["bodies"].as_array().into_iter().flatten() {
        if let Some(id) = b["flexible_component"].as_str() {
            let m = get(s, id)?;
            validate(&m["job"], &m["result"])?;
            components.insert(id.into(), m.clone());
        }
    }
    if components.is_empty() {
        return Err(error("active ROM binding does not belong to a moving block"));
    }
    Ok(json!({"schema":"solidcraft.flex-assembly-job/1","job":job,"components":components}))
}
fn export(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Export = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    if !p.path.to_ascii_lowercase().ends_with(".json") || p.path.len() > 4096 {
        return Err(error("ROM export requires a .json path"));
    }
    let record = get(s, &p.id)?;
    if record["job"]["source"] != source(s)? {
        return Err(error("rebuild stale ROM before export"));
    }
    validate(&record["job"], &record["result"])?;
    solidcraft_io::vfs::check(&p.path).map_err(|e| error(e.to_string()))?;
    if !p.overwrite && solidcraft_io::vfs::exists(&p.path) {
        return Err(error("destination exists; select another path"));
    }
    let bytes = serde_json::to_vec(record).map_err(|e| error(e.to_string()))?;
    solidcraft_io::vfs::write(&p.path, &bytes).map_err(|e| error(e.to_string()))?;
    Ok(json!({"path":p.path,"bytes":bytes.len()}))
}
fn models(s: &Session) -> Value {
    s.doc.fem_models.as_ref().map(|v| (**v).clone()).unwrap_or_else(|| json!({"schema":"solidcraft.fem-components/1","models":{}}))
}
fn save(s: &mut Session, value: Value) -> Result<()> {
    if serde_json::to_vec(&value).map_err(|e| error(e.to_string()))?.len() > 16 * 1024 * 1024 {
        return Err(error("component FEM storage exceeds 16 MiB"));
    }
    s.doc_mut().fem_models = Some(Arc::new(value));
    Ok(())
}
fn source(s: &Session) -> Result<Value> {
    super::mks_simulation::physical_snapshot(s)
}
fn get<'a>(s: &'a Session, id: &str) -> Result<&'a Value> {
    s.doc.fem_models.as_ref().and_then(|v| v["models"].get(id)).filter(|m| m.is_object()).ok_or_else(|| error("FEM component not found"))
}
fn automatic_shape(s: &Session, body: &solidcraft_doc::ModelBody, world: solidcraft_doc::Mat) -> Result<Value> {
    let feature = s
        .doc
        .features
        .iter()
        .find(|f| f.id == body.feature && !f.suppressed)
        .ok_or_else(|| error("native primitive feature not found; import a conforming TET4 mesh"))?;
    let coord = |p: &[solidcraft_doc::Expr; 3]| -> Result<Vec3> {
        Ok(Vec3::new(s.doc.eval(&p[0], Kind::Length)?, s.doc.eval(&p[1], Kind::Length)?, s.doc.eval(&p[2], Kind::Length)?))
    };
    let mesh = body.mesh();
    match &feature.kind {
        FeatureKind::Box { corner, length, width, height, .. } => {
            let c = coord(corner)?;
            let size = Vec3::new(s.doc.eval(length, Kind::Length)?, s.doc.eval(width, Kind::Length)?, s.doc.eval(height, Kind::Length)?);
            if size.x <= 0. || size.y <= 0. || size.z <= 0. {
                return Err(error("positive box dimensions required"));
            }
            let boundary = |p: Vec3| {
                let d = p - c;
                let e = size.len().max(1.) * 1e-7;
                d.x >= -e
                    && d.y >= -e
                    && d.z >= -e
                    && d.x <= size.x + e
                    && d.y <= size.y + e
                    && d.z <= size.z + e
                    && (d.x.abs() < e
                        || d.y.abs() < e
                        || d.z.abs() < e
                        || (d.x - size.x).abs() < e
                        || (d.y - size.y).abs() < e
                        || (d.z - size.z).abs() < e)
            };
            if mesh.positions.iter().any(|p| !boundary(*p))
                || mesh
                    .triangles
                    .iter()
                    .any(|t| !boundary((mesh.positions[t[0] as usize] + mesh.positions[t[1] as usize] + mesh.positions[t[2] as usize]) / 3.))
            {
                return Err(error("modified box needs an imported volume mesh; bounding-box replacement is refused"));
            }
            let area: f64 = mesh
                .triangles
                .iter()
                .map(|t| {
                    let [a, b, c] = t.map(|i| mesh.positions[i as usize]);
                    (b - a).cross(c - a).len() / 2.
                })
                .sum();
            if (area - 2. * (size.x * size.y + size.y * size.z + size.z * size.x)).abs() > area * 1e-6 {
                return Err(error("CAD boundary differs from a complete box"));
            }
            Ok(json!({"kind":"box","corner_mm":c,"size_mm":size,"world":world}))
        }
        FeatureKind::Cylinder { base, axis, radius, height, .. } => {
            let base = coord(base)?;
            let axis = axis.0;
            let r = s.doc.eval(radius, Kind::Length)?;
            let l = s.doc.eval(height, Kind::Length)?;
            if r <= 0. || l <= 0. || solidcraft_doc::naming::face_names(body).len() != 3 {
                return Err(error("only complete unmodified cylinders are automatically meshed; import other geometry"));
            }
            let tol = r.max(l).max(1.) * 1e-6;
            if mesh.positions.iter().any(|p| {
                let v = *p - base;
                let x = v.dot(axis);
                let radial = (v - axis * x).len();
                x < -tol || x > l + tol || radial > r + tol || (x.abs() > tol && (x - l).abs() > tol && (radial - r).abs() > tol)
            }) {
                return Err(error("CAD cylinder boundary differs from its primitive; import a conforming mesh"));
            }
            Ok(json!({"kind":"cylinder","base_mm":base,"axis":axis,"radius_mm":r,"length_mm":l,"world":world}))
        }
        _ => Err(error("automatic mesh supports unmodified boxes and cylinders; use a conforming imported TET4 mesh for other parts/assemblies")),
    }
}
fn create(s: &mut Session, p: &Value) -> Result<Value> {
    let mut p: Build = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    if let Some(path) = &p.mesh_path {
        if p.mesh.is_some() {
            return Err(error("choose mesh or mesh_path, not both"));
        }
        let bytes = solidcraft_io::vfs::read(path, 6 * 1024 * 1024).map_err(|e| error(e.to_string()))?;
        p.mesh = Some(serde_json::from_slice(&bytes).map_err(|e| error(e.to_string()))?);
    }
    if p.id.is_empty()
        || p.id.len() > 64
        || !p.id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        || !(1..=24).contains(&p.modes)
        || p.divisions.iter().any(|n| !(1..=16).contains(n))
        || ![1, 2].contains(&p.element_order)
        || !(1..=8).contains(&p.interfaces.len())
    {
        return Err(error("bounded ID, one to eight surface interfaces, mesh divisions 1..16 and modes 1..24 required"));
    }
    if !p.material.young_mpa.is_finite()
        || !p.material.poisson_ratio.is_finite()
        || !p.material.density_kg_m3.is_finite()
        || !(0.001..=1e7).contains(&p.material.young_mpa)
        || !(-0.99..0.499).contains(&p.material.poisson_ratio)
        || !(1. ..=1e6).contains(&p.material.density_kg_m3)
    {
        return Err(error("invalid elastic material"));
    }
    let fingerprint = source(s)?;
    let graph = s.execute("mks.model", &json!({}))?;
    let block =
        graph["blocks"].as_array().and_then(|a| a.iter().find(|v| v["id"] == p.block)).ok_or_else(|| error("select a native CAD/MKS block"))?;
    let owned: Vec<_> =
        block["bodies"].as_array().into_iter().flatten().filter(|b| p.body.as_ref().is_none_or(|name| b["source"] == *name)).collect();
    if owned.is_empty() || owned.len() > 256 || p.body.is_some() && owned.len() != 1 {
        return Err(error("ambiguous/empty body selection; select the complete block or a uniquely named part"));
    }
    let native = s.model.state();
    let mut shape = None;
    if p.mesh.is_none() {
        if owned.len() != 1 {
            return Err(error("multi-part assemblies require an imported connected volume mesh including their bonded connections"));
        }
        let item = owned[0];
        let body = native
            .bodies
            .iter()
            .find(|b| Some(b.name.as_str()) == item["source"].as_str() && Some(s.doc.model_body_component(b)) == item["component"].as_u64())
            .ok_or_else(|| error("CAD part missing"))?;
        let world = serde_json::from_value(item["world"].clone()).map_err(|e| error(e.to_string()))?;
        shape = Some(automatic_shape(s, body, world)?);
    }
    let ports = super::mechanics::model_ports(s)?;
    let mut interfaces = Vec::new();
    for id in &p.interfaces {
        if interfaces.iter().any(|v: &Value| v["id"] == *id) {
            return Err(error("surface interfaces must be distinct"));
        }
        let port = ports
            .iter()
            .find(|v| v["port"]["id"] == *id && v["valid"] == true && v["port"]["support"] == "surface")
            .ok_or_else(|| error(format!("{id}: a valid CAD-bound surface interface is required")))?;
        let placed = owned
            .iter()
            .find(|b| b["source"] == port["port"]["body"] && b["path"] == port["port"]["occurrence"])
            .ok_or_else(|| error("interface does not belong to the selected component/part"))?;
        let body = native
            .bodies
            .iter()
            .find(|b| Some(b.name.as_str()) == placed["source"].as_str() && Some(s.doc.model_body_component(b)) == placed["component"].as_u64())
            .ok_or_else(|| error("interface CAD part missing"))?;
        let face = solidcraft_doc::naming::face_names(body)
            .iter()
            .position(|n| Some(n.as_str()) == port["port"]["area_face"].as_str())
            .ok_or_else(|| error("interface named face is stale"))?;
        let world: solidcraft_doc::Mat = serde_json::from_value(placed["world"].clone()).map_err(|e| error(e.to_string()))?;
        let mesh = body.mesh();
        let triangles: Vec<_> = mesh
            .triangles
            .iter()
            .enumerate()
            .filter(|(i, _)| mesh.tri_face.get(*i) == Some(&(face as u32)))
            .map(|(_, t)| t.map(|i| apply_point(&world, mesh.positions[i as usize])))
            .collect();
        if triangles.is_empty() || triangles.len() > 2048 {
            return Err(error("interface surface tessellation is missing or exceeds 2048 triangles"));
        }
        let frame = super::mks_interfaces::frame(s, id)?;
        let origin = apply_point(&frame, Vec3::ZERO);
        let mut spec = json!({"id":id,"origin_mm":origin,"triangles_mm":triangles,"area_mm2":port["port"]["area_mm2"]});
        if let Some(sh) = &shape
            && sh["kind"] == "cylinder"
        {
            let base: Vec3 = serde_json::from_value(sh["base_mm"].clone()).map_err(|e| error(e.to_string()))?;
            let axis: Vec3 = serde_json::from_value(sh["axis"].clone()).map_err(|e| error(e.to_string()))?;
            let radius = sh["radius_mm"].as_f64().ok_or_else(|| error("cylinder radius missing"))?;
            let length = sh["length_mm"].as_f64().ok_or_else(|| error("cylinder length missing"))?;
            let start = apply_point(&world, base);
            let direction = apply_vector(&world, axis);
            if (origin - start).len() < 1e-4 || (origin - start - direction * length).len() < 1e-4 {
                spec["disk"] = json!({"center_mm":origin,"normal":direction,"radius_mm":radius});
            }
        }
        interfaces.push(spec);
    }
    let mesh = if let Some(m) = p.mesh {
        if m.nodes_mm.len() > 512 || m.tetrahedra.len() > 3000 || m.nodes_mm.iter().flatten().any(|v| !v.is_finite()) {
            return Err(error("imported mesh exceeds bounded size or has non-finite nodes"));
        }
        json!({"nodes_mm":m.nodes_mm,"tetrahedra":m.tetrahedra,"element_order":p.element_order})
    } else {
        json!({"shape":shape,"divisions":p.divisions,"element_order":p.element_order})
    };
    let job = json!({"schema":"solidcraft.fem-job/1","action":"reduce","generation":super::inspect::new_document_id(),"source":fingerprint,"mesh":mesh,"material":p.material,"interfaces":interfaces,"modes":p.modes,"loads":[{"interface":p.interfaces.last(),"wrench_world_n_nm":[0,0,-100,0,0,0]}]});
    let mut all = models(s);
    if all["models"].as_object().is_none_or(|m| m.len() >= 8 && !m.contains_key(&p.id)) {
        return Err(error("at most eight FEM component models"));
    }
    let enabled = all["models"][&p.id]["enabled"] == true;
    all["models"][&p.id] = json!({"id":p.id,"block":p.block,"body":p.body,"owners":owned,"job":job,"result":null,"enabled":enabled});
    save(s, all)?;
    Ok(json!({"id":p.id}))
}
fn read(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Query = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    let src = source(s)?;
    let entries = s.doc.fem_models.as_ref().and_then(|v| v["models"].as_object());
    let list:Vec<_>=entries.into_iter().flatten().map(|(id,m)|json!({"id":id,"block":m["block"],"body":m["body"],"current":m["job"]["source"]==src,"has_result":m["result"].is_object(),"has_flexible":m["result"]["flexible"].is_object(),"enabled":m["enabled"]==true,"interfaces":m["job"]["interfaces"].as_array().map(|a|a.iter().map(|v|v["id"].clone()).collect::<Vec<_>>()),"mass_kg":m["result"]["mass_kg"],"full_dofs":m["result"]["rom"]["full_dofs"],"reduced_dofs":m["result"]["rom"]["reduced_dofs"],"frequencies_hz":m["result"]["rom"]["fixed_interface_frequencies_hz"],"max_displacement_mm":m["result"]["static"]["max_displacement_mm"],"max_stress_mpa":m["result"]["static"]["max_von_mises_mpa"],"static_rom_error":m["result"]["static"]["full_vs_rom_relative_displacement_error"]})).collect();
    let graph = s.execute("mks.model", &json!({}))?;
    let ports = super::mechanics::model_ports(s)?;
    let scopes:Vec<_>=graph["blocks"].as_array().into_iter().flatten().map(|b|json!({"id":b["id"],"name":b["id"],"bodies":b["bodies"],"interfaces":ports.iter().filter(|port|port["valid"]==true&&port["port"]["support"]=="surface"&&b["bodies"].as_array().is_some_and(|bodies|bodies.iter().any(|v|v["source"]==port["port"]["body"]&&v["path"]==port["port"]["occurrence"]))).map(|v|v["port"]["id"].clone()).collect::<Vec<_>>()})).collect();
    let data = if p.detail { p.id.as_deref().map(|id| get(s, id).cloned()).transpose()? } else { None };
    Ok(json!({"models":list,"scopes":scopes,"data":data}))
}
fn validate(job: &Value, result: &Value) -> Result<()> {
    if result["schema"] != "solidcraft.fem-rom/1" || result["status"] != "success" || result["source"] != job["source"] {
        return Err(error("FEM result schema or source mismatch"));
    }
    let nodes = result["nodes_mm"].as_array().filter(|a| (4..=1500).contains(&a.len())).ok_or_else(|| error("invalid result mesh size"))?;
    for p in nodes {
        let p: Vec3 = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
        if ![p.x, p.y, p.z].iter().all(|x| x.is_finite() && x.abs() <= 1e7) {
            return Err(error("invalid FEM result coordinates"));
        }
    }
    let topology = |key: &str, width: usize, max: usize| -> Result<()> {
        let rows = result[key].as_array().filter(|a| !a.is_empty() && a.len() <= max).ok_or_else(|| error("invalid FEM connectivity size"))?;
        for row in rows {
            let indices = row.as_array().filter(|r| r.len() == width).ok_or_else(|| error("invalid FEM connectivity row"))?;
            if indices.iter().any(|i| i.as_u64().is_none_or(|i| i as usize >= nodes.len())) {
                return Err(error("FEM connectivity index out of range"));
            }
        }
        Ok(())
    };
    topology("surface_triangles", 3, 24000)?;
    topology("tetrahedra", if result["element_type"] == "TET10" { 10 } else { 4 }, 3000)?;
    let n = result["rom"]["reduced_dofs"].as_u64().filter(|n| (1..=72).contains(n)).ok_or_else(|| error("invalid reduced dimension"))? as usize;
    for (key, rows) in [("basis", nodes.len() * 3), ("mass_reduced", n), ("stiffness_reduced", n)] {
        let a = result["rom"][key].as_array().filter(|a| a.len() == rows).ok_or_else(|| error("invalid ROM matrix rows"))?;
        let limit = if key == "basis" { 1e12 } else { 1e30 };
        if a.iter()
            .any(|r| r.as_array().is_none_or(|v| v.len() != n || v.iter().any(|v| v.as_f64().is_none_or(|x| !x.is_finite() || x.abs() > limit))))
        {
            return Err(error("invalid ROM matrix columns/values"));
        }
    }
    if result["static"]["full_vs_rom_relative_displacement_error"].as_f64().is_none_or(|v| !v.is_finite() || v > 1e-6) {
        return Err(error("ROM static verification failed"));
    }
    let rows = |v: &Value, count: usize, width: usize| {
        v.as_array().is_some_and(|a| {
            a.len() == count
                && a.iter().all(|r| {
                    r.as_array().is_some_and(|r| r.len() == width && r.iter().all(|v| v.as_f64().is_some_and(|x| x.is_finite() && x.abs() <= 1e12)))
                })
        })
    };
    if !rows(&result["static"]["node_displacements_mm"], nodes.len(), 3)
        || result["mass_kg"].as_f64().is_none_or(|x| !x.is_finite() || x <= 0.)
        || result["rom"]["modal_residual"].as_f64().is_none_or(|x| !x.is_finite() || !(0. ..=1e-6).contains(&x))
    {
        return Err(error("invalid FEM physical output"));
    }
    let frequencies = result["rom"]["fixed_interface_frequencies_hz"]
        .as_array()
        .filter(|a| a.len() == job["modes"].as_u64().unwrap_or(0) as usize)
        .ok_or_else(|| error("fixed-interface frequencies missing"))?;
    if frequencies.iter().any(|v| v.as_f64().is_none_or(|v| !v.is_finite() || v <= 0.)) {
        return Err(error("invalid FEM frequency"));
    }
    if let Some(f) = result.get("flexible") {
        let times = f["times_s"].as_array().filter(|a| (2..=201).contains(&a.len())).ok_or_else(|| error("invalid flexible sample count"))?;
        let mut previous = -1.;
        for t in times {
            let t = t.as_f64().filter(|x| x.is_finite() && *x >= 0. && *x > previous).ok_or_else(|| error("invalid flexible times"))?;
            previous = t;
        }
        if !rows(&f["modal_coordinates"], times.len(), n)
            || !rows(&f["reference_poses"], times.len(), 12)
            || !rows(&f["joint_forces_local_n"], times.len(), 3)
        {
            return Err(error("invalid flexible states"));
        }
    }
    Ok(())
}
fn run(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Run = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    let stored = get(s, &p.id)?;
    if stored["job"]["source"] != source(s)? {
        return Err(error("CAD/parameters/interfaces changed; rebuild the FEM model before reduction"));
    }
    let mut job = stored["job"].clone();
    job["action"] = json!(if p.flexible { "flexible" } else { "reduce" });
    if p.force_n.is_some() || p.moment_nm.is_some() {
        let f = p.force_n.unwrap_or([0., 0., -100.]);
        let m = p.moment_nm.unwrap_or([0.; 3]);
        if f.iter().chain(m.iter()).any(|v| !v.is_finite() || v.abs() > 1e7) {
            return Err(error("finite bounded wrench required"));
        }
        job["loads"][0]["wrench_world_n_nm"] = json!([f[0], f[1], f[2], m[0], m[1], m[2]]);
    }
    if let Some(motion) = p.motion {
        job["motion"] = motion;
    }
    let result = super::mks_local::execute(&job).map_err(|e| error(e.to_string()))?;
    validate(&job, &result)?;
    let reply = json!({"id":p.id,"job":job,"result":result});
    accept(s, &reply)?;
    Ok(reply)
}
fn accept(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Accept = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    let stored = get(s, &p.id)?;
    if p.job["source"] != source(s)? || stored["job"]["source"] != p.job["source"] {
        return Err(error("FEM inputs changed while solving; computed result discarded"));
    }
    for key in ["mesh", "material", "interfaces", "modes", "generation"] {
        if stored["job"][key] != p.job[key] {
            return Err(error("FEM model changed while solving; computed result discarded"));
        }
    }
    validate(&p.job, &p.result)?;
    use sha2::{Digest, Sha256};
    let signature = format!("{:x}", Sha256::digest(serde_json::to_vec(&p.result).map_err(|e| error(e.to_string()))?));
    let mut all = models(s);
    all["models"][&p.id]["job"] = p.job;
    all["models"][&p.id]["result"] = p.result;
    all["models"][&p.id]["signature"] = json!(signature);
    save(s, all)?;
    Ok(json!({"id":p.id}))
}

/// Actual FE skin; interpolation is only for viewing, never for force results.
fn mks_history(s: &mut Session, id: &str) -> Result<Option<Value>> {
    let state = s.execute("mks.job.state", &json!({}))?;
    if state["result_current"] != true {
        return Ok(None);
    }
    let result = &state["data"]["result"];
    let Some(history) = result["flexible_states"].as_object().and_then(|states| states.values().find(|v| v["component_id"] == id)) else {
        return Ok(None);
    };
    Ok(Some(json!({"times_s":result["times_s"],"modal_coordinates":history["modal_coordinates"],"reference_poses":history["reference_poses"]})))
}
pub fn mks_duration(s: &mut Session, id: &str) -> Result<Option<f64>> {
    Ok(mks_history(s, id)?.and_then(|history| history["times_s"].as_array()?.last()?.as_f64()))
}
pub fn scene(
    s: &mut Session,
    id: &str,
    scale: f64,
    time_s: Option<f64>,
    palette: crate::view::Palette,
) -> Result<(solidcraft_render::Scene, Aabb3, f64)> {
    if !scale.is_finite() || !(0. ..=10000.).contains(&scale) {
        return Err(error("display scale must be 0..10000"));
    }
    let assembly_history = if time_s.is_some() { mks_history(s, id)? } else { None };
    let record = get(s, id)?;
    if record["job"]["source"] != source(s)? {
        return Err(error("FEM result is historical; rebuild after CAD/interface changes"));
    }
    let r = &record["result"];
    validate(&record["job"], r)?;
    let positions: Vec<Vec3> = serde_json::from_value(r["nodes_mm"].clone()).map_err(|e| error(e.to_string()))?;
    let triangles: Vec<[u32; 3]> = serde_json::from_value(r["surface_triangles"].clone()).map_err(|e| error(e.to_string()))?;
    let mut displacement: Vec<Vec3> = serde_json::from_value(r["static"]["node_displacements_mm"].clone()).map_err(|e| error(e.to_string()))?;
    let mut rotation = solidcraft_doc::IDENTITY;
    let mut origin = Vec3::ZERO;
    let mut moving = false;
    if let Some(t) = time_s {
        let f = assembly_history.as_ref().unwrap_or(&r["flexible"]);
        let times: Vec<f64> = serde_json::from_value(f["times_s"].clone()).map_err(|e| error(e.to_string()))?;
        if !t.is_finite()
            || !(2..=1001).contains(&times.len())
            || times.iter().any(|v| !v.is_finite())
            || times.windows(2).any(|pair| pair[1] <= pair[0])
            || t < times[0]
            || t > *times.last().ok_or_else(|| error("flexible times missing"))?
        {
            return Err(error("time outside flexible result"));
        }
        let a = times.partition_point(|v| *v <= t).saturating_sub(1).min(times.len() - 2);
        let b = a + 1;
        let u = (t - times[a]) / (times[b] - times[a]);
        let qa: Vec<f64> = serde_json::from_value(f["modal_coordinates"][a].clone()).map_err(|e| error(e.to_string()))?;
        let qb: Vec<f64> = serde_json::from_value(f["modal_coordinates"][b].clone()).map_err(|e| error(e.to_string()))?;
        let modes = r["rom"]["basis"].get(0).and_then(Value::as_array).map(Vec::len).ok_or_else(|| error("ROM basis missing"))?;
        if qa.len() != modes || qb.len() != modes || qa.iter().chain(&qb).any(|value| !value.is_finite()) {
            return Err(error("dynamic states differ from the component ROM basis"));
        }
        let q: Vec<_> = qa.iter().zip(qb).map(|(a, b)| a * (1. - u) + b * u).collect();
        let basis = r["rom"]["basis"].as_array().ok_or_else(|| error("ROM basis missing"))?;
        let mut values = Vec::new();
        for row in basis {
            let row: Vec<f64> = serde_json::from_value(row.clone()).map_err(|e| error(e.to_string()))?;
            values.push(row.iter().zip(&q).map(|(a, b)| a * b).sum::<f64>() * 1000.);
        }
        displacement = values.as_chunks::<3>().0.iter().map(|p| Vec3::new(p[0], p[1], p[2])).collect();
        let pa: Vec<f64> = serde_json::from_value(f["reference_poses"][a].clone()).map_err(|e| error(e.to_string()))?;
        let pb: Vec<f64> = serde_json::from_value(f["reference_poses"][b].clone()).map_err(|e| error(e.to_string()))?;
        if pa.len() != 12 || pb.len() != 12 || pa.iter().chain(&pb).any(|value| !value.is_finite()) {
            return Err(error("invalid flexible reference pose"));
        }
        let quaternion =
            |p: &[f64]| solidcraft_render::Quat::from_basis(Vec3::new(p[3], p[6], p[9]), Vec3::new(p[4], p[7], p[10]), Vec3::new(p[5], p[8], p[11]));
        let (x, y, z) = quaternion(&pa).slerp(quaternion(&pb), u).basis();
        rotation[0] = [x.x, x.y, x.z, 0.];
        rotation[1] = [y.x, y.y, y.z, 0.];
        rotation[2] = [z.x, z.y, z.z, 0.];
        rotation[3] = [(pa[0] * (1. - u) + pb[0] * u) * 1000., (pa[1] * (1. - u) + pb[1] * u) * 1000., (pa[2] * (1. - u) + pb[2] * u) * 1000., 1.];
        origin = serde_json::from_value(r["interfaces"][0]["origin_mm"].clone()).map_err(|e| error(e.to_string()))?;
        moving = true;
    }
    if displacement.len() != positions.len() || triangles.iter().flatten().any(|i| *i as usize >= positions.len()) {
        return Err(error("invalid FEM display connectivity"));
    }
    let maximum = displacement.iter().map(|v| v.len()).fold(0., f64::max);
    let mut mesh = Mesh::default();
    let mut bounds = Aabb3::EMPTY;
    mesh.positions = positions
        .iter()
        .zip(&displacement)
        .map(|(p, u)| {
            let p = *p + *u * scale;
            let p = if moving { apply_point(&rotation, p - origin) } else { p };
            bounds.add(p);
            p
        })
        .collect();
    mesh.triangles = triangles;
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
    mesh.tri_face = mesh
        .triangles
        .iter()
        .map(|t| {
            let value = t.iter().map(|i| displacement[*i as usize].len()).sum::<f64>() / 3.;
            (value / maximum.max(1e-30) * 31.).round().clamp(0., 31.) as u32
        })
        .collect();
    let face_colors = (0..32)
        .map(|i| {
            let f = f64::from(i) / 31.;
            let a = palette.axes[2];
            let b = palette.axes[0];
            let mix = |x: u8, y: u8| (f64::from(x) * (1. - f) + f64::from(y) * f).round() as u8;
            (i, solidcraft_render::Rgb(mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2)))
        })
        .collect();
    let sc = solidcraft_render::Scene {
        background: Some(palette.background),
        radius: bounds.diagonal().max(1.),
        meshes: vec![solidcraft_render::SceneMesh { mesh: Arc::new(mesh), color: palette.body, face_colors, opacity: 1. }],
        ..Default::default()
    };
    Ok((sc, bounds, maximum))
}
