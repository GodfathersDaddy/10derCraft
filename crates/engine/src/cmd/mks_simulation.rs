//! Persistent, input-bound jobs for an explicitly scoped optional external solver.
use super::CommandSpec;
use crate::{Result, Session, params::bad};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use solidcraft_doc::{IDENTITY, Mat, apply_point, apply_vector, mat_mul};
use solidcraft_geom::Vec3;
use std::collections::{BTreeMap, BTreeSet};
const KEY: &str = "org.solidcraft.assembly-simulation";
pub static COMMANDS:&[CommandSpec]=&[
    CommandSpec::new("mks.result.frame","Read Dynamics Frame",frame).noundo().params("sample: saved result sample index; simulated world transforms for CAD members/bodies and ports without changing CAD; refuses historical results"),
    CommandSpec::new("mks.job.create","Prepare CAD Dynamics Job",create).params("duration_s?:2, steps?:1000, samples?:21, gravity_m_s2?:[0,0,-9.81], allow_assumed_density?:false, port_loads?:[{port,force_n?:[x,y,z],moment_nm?:[x,y,z],pressure_mpa?:number}], drivers?:[{joint:id,displacement:angle/length expression,ramp_s:number}]; smooth cosine increment from current pose; open rigid revolute/slider tree only"),
    CommandSpec::new("mks.job.run","Run Local CAD Dynamics",run).params("Same settings as mks.job.create; starts optional installed external Exudyn adapter, bounded timeout, saves input-bound result; no CAD motion or network server"),
    CommandSpec::new("mks.job.accept","Import CAD Dynamics Result",accept).params("result: assembly_worker output; exact current prepared job and CAD source required; no CAD changes"),
    CommandSpec::new("mks.job.state","Inspect CAD Dynamics Result",state).noundo().params("Persistent job and compact trajectories, with validity against current CAD, material, parameters and pending poses"),
];
fn error(msg: impl Into<String>) -> crate::EngineError {
    bad("mks.simulation", msg)
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Settings {
    duration_s: f64,
    steps: u32,
    samples: u32,
    gravity_m_s2: [f64; 3],
    allow_assumed_density: bool,
    port_loads: Vec<Load>,
    drivers: Vec<Drive>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            duration_s: 2.,
            steps: 1000,
            samples: 21,
            gravity_m_s2: [0., 0., -9.81],
            allow_assumed_density: false,
            port_loads: Vec::new(),
            drivers: Vec::new(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Load {
    port: String,
    #[serde(default)]
    force_n: [f64; 3],
    #[serde(default)]
    moment_nm: [f64; 3],
    pressure_mpa: Option<f64>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Drive {
    joint: u64,
    displacement: String,
    ramp_s: f64,
}
pub(super) fn snapshot(s: &Session) -> Result<Value> {
    if !s.model.param_errors.is_empty() || s.model.results.iter().any(|r| r.error.is_some() || r.stale) {
        return Err(error("repair CAD feature/parameter errors before dynamics"));
    }
    let mut source = serde_json::to_value(&*s.placed_document()).map_err(|e| error(e.to_string()))?;
    // Saving an untitled design assigns its filename as title; this is not a physical edit.
    if let Some(map) = source.as_object_mut() {
        map.remove("name");
    }
    if let Some(m) = source.get_mut("metadata").and_then(Value::as_object_mut) {
        m.remove(KEY);
        m.remove(super::robot::FEM_KEY);
    }
    if source.get("metadata").and_then(Value::as_object).is_some_and(|m| m.is_empty())
        && let Some(map) = source.as_object_mut()
    {
        map.remove("metadata");
    }
    // Native source includes imported STEP text/meshes. Evaluator cache keys are
    // session-local and must not be mistaken for persistent geometry identities.
    let bytes = serde_json::to_vec(&source).map_err(|e| error(e.to_string()))?;
    if bytes.len() > 20_000 {
        use sha2::{Digest, Sha256};
        Ok(
            json!({"document_sha256":format!("{:x}",Sha256::digest(&bytes)),"canonical_json_bytes":bytes.len(),"normalization":"source-v2; excludes title and simulation result keys"}),
        )
    } else {
        Ok(json!({"document":source}))
    }
}
fn array(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(|| error("model id missing"))
}
fn vector(v: &Value) -> Result<Vec3> {
    serde_json::from_value(v.clone()).map_err(|e| error(e.to_string()))
}
fn build(s: &mut Session, p: &Value) -> Result<Value> {
    let settings: Settings = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    if !settings.duration_s.is_finite()
        || !(0.001..=10.).contains(&settings.duration_s)
        || !(10..=10000).contains(&settings.steps)
        || !(2..=101).contains(&settings.samples)
        || !settings.steps.is_multiple_of(settings.samples - 1)
        || settings.gravity_m_s2.iter().any(|v| !v.is_finite() || v.abs() > 100.)
        || settings.port_loads.len() > 8
    {
        return Err(error("duration .001..10 s, steps 10..10000 divisible by samples-1, samples 2..101, |gravity|<=100, at most 8 loads"));
    }
    let source = snapshot(s)?;
    let graph = super::mks::model(s, &json!({}))?;
    if !array(&graph["issues"]).is_empty() || !array(&graph["unbound_ports"]).is_empty() || !array(&graph["floating_blocks"]).is_empty() {
        return Err(error(
            "assembly contains unresolved/unsupported constraints, contacts, motion links, ports or floating blocks; inspect mks.model",
        ));
    }
    let mass = super::mks_mass::calculate(s, &json!({}))?;
    if mass["complete"] != true {
        return Err(error("one or more rigid blocks have incomplete mass properties"));
    }
    let mut bodies = Vec::new();
    let mut ground = None;
    let mut ports = BTreeMap::new();
    for b in array(&graph["blocks"]) {
        let id = text(&b["id"])?;
        if b["grounded"] == true {
            ground = Some(id.to_owned());
        } else {
            let physical = array(&mass["blocks"]).iter().find(|m| m["id"] == id).ok_or_else(|| error("block mass missing"))?;
            if physical["status"] != "evaluated" {
                return Err(error(format!("{id}: moving block has no massive body")));
            }
            if !settings.allow_assumed_density && array(&physical["bodies"]).iter().any(|b| b["density_assumed"] == true) {
                return Err(error(format!("{id}: assign material or explicitly allow assumed CAD density")));
            }
            bodies.push(json!({"id":id,"mass_kg":physical["mass_kg"],"com_world_m":physical["com_world_m"],"inertia_com_world_kg_m2":physical["inertia_com_world_kg_m2"]}));
        }
        for port in array(&b["ports"]) {
            ports.insert(text(&port["port"]["id"])?.to_owned(), (id.to_owned(), port.clone()));
        }
    }
    if bodies.is_empty() || bodies.len() > 8 || bodies.len() * (settings.samples as usize - 1) > 100 {
        return Err(error("adapter supports 1..8 moving blocks and at most 100 body/sample intervals; reduce samples for larger chains"));
    }
    let ground = ground.ok_or_else(|| error("ground missing"))?;
    let mut joints = Vec::new();
    let mut reached = BTreeSet::from([ground.clone()]);
    for j in array(&graph["joints"]) {
        let kind = text(&j["kind"])?;
        if !["revolute", "slider"].contains(&kind) || j["closes_graph_cycle"] == true {
            return Err(error("only open revolute/slider trees are supported"));
        }
        let a: Mat = serde_json::from_value(j["frame_a_world"].clone()).map_err(|e| error(e.to_string()))?;
        let b: Mat = serde_json::from_value(j["frame_b_world"].clone()).map_err(|e| error(e.to_string()))?;
        let za = apply_vector(&a, Vec3::Z);
        let zb = apply_vector(&b, Vec3::Z);
        let delta = apply_point(&b, Vec3::ZERO) - apply_point(&a, Vec3::ZERO);
        if za.cross(zb).len() > 1e-7 || delta.cross(za).len() > 1e-4 {
            return Err(error("native joint axes/anchors do not agree with current CAD pose"));
        }
        let rotation: Vec<Vec<f64>> = (0..3).map(|r| (0..3).map(|c| a.get(c).and_then(|v| v.get(r)).copied().unwrap_or(0.)).collect()).collect();
        joints.push(json!({"id":j["id"],"name":j["name"],"kind":kind,"a":j["a"],"b":j["b"],
            "pivot_world_m":apply_point(&a,Vec3::ZERO)*0.001,"axis_world":za,"rotation_world":rotation,"native_values_at_start":j["values"]}));
    }
    if joints.len() != bodies.len() {
        return Err(error("every moving block must belong to an open tree connected to ground"));
    }
    for _ in 0..=joints.len() {
        for j in &joints {
            let a = text(&j["a"])?;
            let b = text(&j["b"])?;
            if reached.contains(a) {
                reached.insert(b.into());
            } else if reached.contains(b) {
                reached.insert(a.into());
            }
        }
    }
    if bodies.iter().any(|b| !reached.contains(b["id"].as_str().unwrap_or(""))) {
        return Err(error("disconnected dynamic block"));
    }
    let mut loads = Vec::new();
    for load in &settings.port_loads {
        if load.force_n.iter().chain(load.moment_nm.iter()).any(|v| !v.is_finite() || v.abs() > 1e8) {
            return Err(error("load components must be finite and within +/-1e8"));
        }
        let (block, p) = ports.get(&load.port).ok_or_else(|| error(format!("unknown port {}", load.port)))?;
        if block == &ground {
            return Err(error("a dynamic load must act on a moving block"));
        }
        let transfer = text(&p["port"]["transfer"])?;
        if transfer == "point_force" && load.moment_nm.iter().any(|v| v.abs() > 0.) {
            return Err(error("point-force port cannot transmit a moment"));
        }
        let mut force = Vec3::new(load.force_n[0], load.force_n[1], load.force_n[2]);
        if let Some(pressure) = load.pressure_mpa {
            if transfer != "pressure" || !pressure.is_finite() || pressure.abs() > 1e6 || force.len() > 0. || load.moment_nm.iter().any(|v| *v != 0.)
            {
                return Err(error("pressure needs a pressure surface port and no additional wrench"));
            }
            force = Vec3::Z * (pressure * p["port"]["area_mm2"].as_f64().ok_or_else(|| error("pressure area missing"))?);
        } else if transfer == "pressure" {
            return Err(error("pressure port requires pressure_mpa"));
        }
        if force.len() > 1e8 {
            return Err(error("pressure resultant exceeds adapter force limit (1e8 N)"));
        }
        let rotate = |v: Vec3| -> Result<Vec3> {
            Ok(vector(&p["frame"]["axes"][0])? * v.x + vector(&p["frame"]["axes"][1])? * v.y + vector(&p["frame"]["axes"][2])? * v.z)
        };
        loads.push(json!({"port":load.port,"block":block,"position_world_m":vector(&p["frame"]["origin"])?*0.001,
            "force_initial_world_n":rotate(force)?,"moment_initial_world_nm":rotate(Vec3::new(load.moment_nm[0],load.moment_nm[1],load.moment_nm[2]))?}));
    }
    if settings.drivers.len() > joints.len() {
        return Err(error("more drives than joints"));
    }
    let mut drivers = Vec::new();
    let mut driven = BTreeSet::new();
    for drive in &settings.drivers {
        let joint = joints.iter().find(|j| j["id"] == drive.joint).ok_or_else(|| error("drive joint does not exist in supported model"))?;
        if !driven.insert(drive.joint)
            || !drive.ramp_s.is_finite()
            || drive.ramp_s < settings.duration_s / settings.steps as f64 * 10.
            || drive.ramp_s > settings.duration_s
        {
            return Err(error("one driver per joint; ramp_s must span at least 10 integration steps and fit within duration"));
        }
        if drive.displacement.len() > 256 {
            return Err(error("drive expression exceeds 256 bytes"));
        }
        let angular = joint["kind"] == "revolute";
        let amplitude =
            s.doc.eval(&drive.displacement, if angular { solidcraft_doc::expr::Kind::Angle } else { solidcraft_doc::expr::Kind::Length })?
                * if angular { 1. } else { 0.001 };
        if !amplitude.is_finite() || amplitude.abs() > if angular { 3.0 } else { 10. } {
            return Err(error("drive increment exceeds adapter range (3 rad or 10 m)"));
        }
        drivers.push(json!({"joint":drive.joint,"amplitude_si":amplitude,"ramp_s":drive.ramp_s,"profile":"half_cosine_displacement_then_hold"}));
    }
    // No stop/contact law: bounded joints are admitted only when a prescribed,
    // monotone ramp stays wholly inside their native travel range.
    for j in array(&graph["joints"]) {
        for (index, limit) in array(&j["native"]["limits"]).iter().enumerate() {
            if limit.is_null() {
                continue;
            }
            if index != 0 {
                return Err(error("unsupported multi-coordinate joint limit"));
            }
            let drive = drivers
                .iter()
                .find(|d| d["joint"] == j["id"])
                .ok_or_else(|| error("limited joint requires a prescribed drive; free-motion stops are not implemented"))?;
            let q = j["values"].get(0).and_then(Value::as_f64).ok_or_else(|| error("limited joint coordinate missing"))?;
            let mut increment = drive["amplitude_si"].as_f64().ok_or_else(|| error("drive amplitude missing"))?;
            if j["kind"] == "slider" {
                increment *= 1000.;
            }
            let lo = limit.get(0).and_then(Value::as_f64).ok_or_else(|| error("invalid joint minimum"))?;
            let hi = limit.get(1).and_then(Value::as_f64).ok_or_else(|| error("invalid joint maximum"))?;
            if !lo.is_finite() || !hi.is_finite() || lo > hi || q.min(q + increment) < lo - 1e-10 || q.max(q + increment) > hi + 1e-10 {
                return Err(error("prescribed ramp exceeds native joint limits"));
            }
        }
    }
    Ok(
        json!({"schema":"solidcraft.assembly-job/1","backend":"exudyn_rigid_open_chain","settings":settings,"source":source,"model":{"ground":ground,"bodies":bodies,"joints":joints,"loads":loads,"drivers":drivers}}),
    )
}
fn save(s: &mut Session, job: Value, result: Option<Value>) -> Result<Value> {
    let v = json!({"job":job,"result":result});
    s.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY, v.clone())?;
    Ok(v)
}
fn create(s: &mut Session, p: &Value) -> Result<Value> {
    let job = build(s, p)?;
    save(s, job.clone(), None)?;
    Ok(job)
}
fn validate(job: &Value, result: &Value) -> Result<Value> {
    if result["schema"] != "solidcraft.assembly-result/1" || result["status"] != "success" || result.get("job") != Some(job) {
        return Err(error("solver result does not match the exact prepared job"));
    }
    let times = array(&result["times_s"]);
    let n = job["settings"]["samples"].as_u64().unwrap_or(0) as usize;
    let end = job["settings"]["duration_s"].as_f64().unwrap_or(0.);
    if times.len() != n
        || times.iter().enumerate().any(|(i, t)| !t.as_f64().is_some_and(|t| t.is_finite() && (t - end * i as f64 / (n - 1) as f64).abs() < 1e-7))
    {
        return Err(error("incomplete/invalid sample time grid"));
    }
    let finite_rows = |v: &Value, cols: usize| -> bool {
        array(v).len() == n
            && array(v).iter().all(|r| array(r).len() == cols && array(r).iter().all(|x| x.as_f64().is_some_and(|v| v.is_finite() && v.abs() < 1e12)))
    };
    let trajectories = result["trajectories"].as_object().ok_or_else(|| error("trajectories required"))?;
    if trajectories.len() != array(&job["model"]["bodies"]).len() || !finite_rows(&result["energy_j"], 3) {
        return Err(error("incomplete energy/trajectory output"));
    }
    for b in array(&job["model"]["bodies"]) {
        let id = text(&b["id"])?;
        let poses = trajectories.get(id).ok_or_else(|| error("missing body trajectory"))?;
        if !finite_rows(poses, 12) {
            return Err(error("invalid body poses"));
        }
        for row in array(poses) {
            let at = |r: usize, c: usize| row.get(3 + r * 3 + c).and_then(Value::as_f64).unwrap_or(f64::NAN);
            let rows: Vec<Vec<f64>> = (0..3).map(|r| vec![at(r, 0), at(r, 1), at(r, 2), 0.]).collect();
            super::component::matrix::rigid(&json!(rows), 1e-5, "mks.simulation")?;
        }
    }
    let reactions = result["joint_forces_local_n"].as_object().ok_or_else(|| error("joint force histories required"))?;
    if reactions.len() != array(&job["model"]["joints"]).len() {
        return Err(error("incomplete joint force histories"));
    }
    for j in array(&job["model"]["joints"]) {
        if !finite_rows(&reactions.get(&j["id"].to_string()).cloned().unwrap_or(Value::Null), 3) {
            return Err(error("invalid joint forces"));
        }
    }
    for (key, limit) in [("max_position_constraint_m", 1e-5), ("max_axis_alignment_error", 1e-5), ("max_energy_change_j", 1e12)] {
        if !result["summary"][key].as_f64().is_some_and(|v| v.is_finite() && v >= 0. && v <= limit) {
            return Err(error("invalid solver residual summary"));
        }
    }
    if !result["backend_version"].as_str().is_some_and(|s| !s.is_empty() && s.len() < 128) {
        return Err(error("backend version missing"));
    }
    for key in ["worker_sha256", "job_sha256"] {
        if !result["provenance"][key].as_str().is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())) {
            return Err(error("solver provenance missing"));
        }
    }
    let mut compact = result.clone();
    if let Some(map) = compact.as_object_mut() {
        map.remove("job");
    }
    Ok(compact)
}
fn accept(s: &mut Session, p: &Value) -> Result<Value> {
    let job = s.doc.metadata.get(KEY).and_then(|v| v.get("job")).cloned().ok_or_else(|| error("prepare a dynamics job first"))?;
    if job["source"] != snapshot(s)? {
        return Err(error("CAD, material, parameters or pose changed; result is historical"));
    }
    let result = validate(&job, p.get("result").ok_or_else(|| error("result required"))?)?;
    save(s, job, Some(result))?;
    Ok(json!({"accepted":true,"cad_changed":false}))
}
fn run(s: &mut Session, p: &Value) -> Result<Value> {
    let job = build(s, p)?;
    let result = super::mks_local::execute(&job)?;
    let compact = validate(&job, &result)?;
    save(s, job, Some(compact.clone()))?;
    Ok(compact)
}
fn state(s: &mut Session, _: &Value) -> Result<Value> {
    let data = s.doc.metadata.get(KEY).cloned().unwrap_or_else(|| json!({}));
    let current = data.get("job").is_some() && snapshot(s).is_ok_and(|v| v == data["job"]["source"]);
    Ok(json!({"data":data,"job_current":current,"result_current":current && data["result"].is_object(),"persistence":"project_metadata"}))
}

pub(super) fn frame(s: &mut Session, p: &Value) -> Result<Value> {
    let state = state(s, &json!({}))?;
    if state["result_current"] != true {
        return Err(error("a current dynamics result is required for replay"));
    }
    let index = p.get("sample").and_then(Value::as_u64).filter(|i| *i < 101).ok_or_else(|| error("sample index 0..100 required"))? as usize;
    let data = &state["data"];
    let result = &data["result"];
    let job = &data["job"];
    let time = result["times_s"].get(index).ok_or_else(|| error("sample outside result"))?;
    let graph = super::mks::model(s, &json!({}))?;
    let mut bodies = Vec::new();
    let mut occurrences = Vec::new();
    let mut ports = Vec::new();
    for block in array(&graph["blocks"]) {
        let id = text(&block["id"])?;
        let mut delta = IDENTITY;
        if block["grounded"] != true {
            let original = array(&job["model"]["bodies"]).iter().find(|b| b["id"] == id).ok_or_else(|| error("missing body definition"))?;
            let com = vector(&original["com_world_m"])? * 1000.;
            let row = result["trajectories"][id].get(index).ok_or_else(|| error("missing result pose"))?;
            for (c, col) in delta.iter_mut().enumerate().take(3) {
                for (r, cell) in col.iter_mut().enumerate().take(3) {
                    *cell = row.get(3 + r * 3 + c).and_then(Value::as_f64).ok_or_else(|| error("invalid rotation"))?;
                }
            }
            let translation = Vec3::new(row[0].as_f64().unwrap_or(0.), row[1].as_f64().unwrap_or(0.), row[2].as_f64().unwrap_or(0.)) * 1000.
                - apply_vector(&delta, com);
            if let Some(t) = delta.get_mut(3) {
                *t = [translation.x, translation.y, translation.z, 1.];
            }
        }
        for (field, out) in [("bodies", &mut bodies), ("members", &mut occurrences)] {
            for item in array(&block[field]) {
                let world: Mat = serde_json::from_value(item["world"].clone()).map_err(|e| error(e.to_string()))?;
                let mut pose = item.clone();
                pose["world"] = json!(mat_mul(&delta, &world));
                pose["block"] = json!(id);
                out.push(pose);
            }
        }
        for port in array(&block["ports"]) {
            ports.push(json!({"id":port["port"]["id"],"block":id,"origin_mm":apply_point(&delta,vector(&port["frame"]["origin"])?),
                "axes":array(&port["frame"]["axes"]).iter().map(|v|vector(v).map(|v|apply_vector(&delta,v))).collect::<Result<Vec<_>>>()?}));
        }
    }
    Ok(
        json!({"sample":index,"time_s":time,"bodies":bodies,"occurrences":occurrences,"ports":ports,"units":{"position":"mm","matrix":"column_major"},"cad_changed":false}),
    )
}
