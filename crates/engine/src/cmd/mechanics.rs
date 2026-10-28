//! Persistent project ports and explicit rigid-tree static resultant transport.
//! This is not a joint reaction, deformation or dynamic solver.
use super::CommandSpec;
use crate::params::{bad, req_expr};
use crate::{Result, Session};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use solidcraft_doc::{Expr, IDENTITY, apply_point, apply_vector, expr::Kind};
use solidcraft_geom::Vec3;
use std::collections::{BTreeMap, BTreeSet};

const KEY: &str = "org.solidcraft.mechanical-project";
pub static COMMANDS:&[CommandSpec]=&[
    CommandSpec::new("mechanics.port","Define Project Mechanical Port",port).params("id, body, occurrence?: [ids], origin?: [length expressions], quaternion?: [w,x,y,z], support?: point|surface, transfer?: point_force|frame_wrench|pressure, area_mm2?: declared surface area; saved in project"),
    CommandSpec::new("mechanics.connect","Connect Rigid Load Path",connect).params("parent, child: port ids; explicit fixed resultant path, one parent per port, cycles rejected; not a native joint"),
    CommandSpec::new("mechanics.disconnect","Disconnect Rigid Load Path",disconnect).params("child: port id; removes its declared parent link; existing reports become historical"),
    CommandSpec::new("mechanics.case","Define Mechanical Load Case",case).params("id, root: port id, loads: [{port, force_n?:[x,y,z], moment_nm?:[x,y,z], pressure_mpa?:number}]; vectors in local port frame, pressure only on declared surface"),
    CommandSpec::new("mechanics.evaluate","Evaluate Static Load Path",evaluate).noundo().params("case: id; read-only rigid-tree resultant transport, positions in mm, forces N, moments Nm; no deformation/joint reactions"),
    CommandSpec::new("mechanics.solve","Save Static Load Path Result",solve).params("case: id; stores report and exact input snapshot in project, no CAD mutation"),
    CommandSpec::new("mechanics.state","Inspect Project Mechanics",state).noundo().params("Saved ports, fixed paths, load cases, resolved frames, and result validity against current CAD/parameters/occurrence pose"),
];
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Network {
    #[serde(default)]
    ports: BTreeMap<String, Port>,
    #[serde(default)]
    parents: BTreeMap<String, String>,
    #[serde(default)]
    cases: BTreeMap<String, Case>,
    #[serde(default)]
    results: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Port {
    id: String,
    body: String,
    #[serde(default)]
    occurrence: Vec<u64>,
    origin: [Expr; 3],
    quaternion: [f64; 4],
    support: String,
    transfer: String,
    #[serde(default)]
    area_mm2: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    root: String,
    loads: Vec<Load>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Load {
    port: String,
    #[serde(default)]
    force_n: Option<[f64; 3]>,
    #[serde(default)]
    moment_nm: Option<[f64; 3]>,
    #[serde(default)]
    pressure_mpa: Option<f64>,
}
#[derive(Clone, Debug, Serialize)]
struct Frame {
    origin: Vec3,
    axes: [Vec3; 3],
}
impl Frame {
    fn rotate(&self, v: Vec3) -> Vec3 {
        self.axes[0] * v.x + self.axes[1] * v.y + self.axes[2] * v.z
    }
    fn local(&self, v: Vec3) -> Vec3 {
        Vec3::new(self.axes[0].dot(v), self.axes[1].dot(v), self.axes[2].dot(v))
    }
}
fn error(message: impl Into<String>) -> crate::EngineError {
    bad("mechanics", message)
}
fn id(p: &Value, key: &str) -> Result<String> {
    let text = p.get(key).and_then(Value::as_str).ok_or_else(|| error(format!("{key}: id required")))?;
    if text.is_empty() || text.len() > 64 || !text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
        return Err(error(format!("{key}: use 1..64 ASCII letters, digits, _ or -")));
    }
    Ok(text.into())
}
fn read(s: &Session) -> Result<Network> {
    let n: Network = serde_json::from_value(s.doc.metadata.get(KEY).cloned().unwrap_or_else(|| json!({}))).map_err(|e| error(e.to_string()))?;
    if n.ports.len() > 128 || n.parents.len() > 128 || n.cases.len() > 32 || n.results.len() > 32 {
        return Err(error("mechanical project exceeds size limits"));
    }
    for (key, p) in &n.ports {
        if key != &p.id {
            return Err(error("port identity mismatch"));
        }
        validate_port(p)?;
    }
    for c in n.cases.values() {
        validate_case(&n, c)?;
    }
    validate_graph(&n)?;
    Ok(n)
}
fn save(s: &mut Session, n: &Network) -> Result<Value> {
    let value = serde_json::to_value(n).map_err(|e| error(e.to_string()))?;
    s.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY, value.clone())?;
    Ok(value)
}
fn valid_numbers(v: &[f64]) -> bool {
    v.iter().all(|x| x.is_finite() && x.abs() <= 1e9)
}
fn validate_port(p: &Port) -> Result<()> {
    id(&json!({"id":p.id}), "id")?;
    if p.body.is_empty() || p.body.len() > 200 || p.occurrence.len() > 64 || p.origin.iter().any(|e| e.len() > 4096) {
        return Err(error("invalid body, occurrence or position expression"));
    }
    if !valid_numbers(&p.quaternion) || (p.quaternion.iter().map(|x| x * x).sum::<f64>() - 1.).abs() > 1e-8 {
        return Err(error("port quaternion must be a finite unit quaternion"));
    }
    if !["point", "surface"].contains(&p.support.as_str()) || !["point_force", "frame_wrench", "pressure"].contains(&p.transfer.as_str()) {
        return Err(error("unknown support or transfer type"));
    }
    if p.support == "point" && (p.transfer == "pressure" || p.area_mm2.is_some()) {
        return Err(error("point support cannot have pressure or area"));
    }
    if p.support == "surface" && p.transfer == "point_force" {
        return Err(error("surface support requires frame_wrench or pressure; use point support for a three-component point force"));
    }
    if p.support == "surface" && !p.area_mm2.is_some_and(|a| a.is_finite() && a > 0. && a <= 1e12) {
        return Err(error("surface requires declared positive area_mm2, up to 1e12"));
    }
    Ok(())
}
fn resolve(s: &Session, p: &Port) -> Result<Frame> {
    validate_port(p)?;
    let native = s.model.state();
    let body = native.body(&p.body).ok_or_else(|| error(format!("{}: CAD body is missing", p.id)))?;
    let placement = if p.occurrence.is_empty() {
        if s.doc.model_body_component(body) != 0 {
            return Err(error(format!("{}: component body requires an exact occurrence path", p.id)));
        }
        IDENTITY
    } else {
        let pose = s.occurrence_placement(&p.occurrence)?;
        if !pose.bodies.iter().any(|b| b.source == p.body) {
            return Err(error(format!("{}: body does not belong to this occurrence", p.id)));
        }
        pose.world
    };
    let v = p.origin.iter().map(|e| s.doc.eval(e, Kind::Length).map_err(|e| error(format!("{}: {e}", p.id)))).collect::<Result<Vec<_>>>()?;
    let local = Vec3::new(
        *v.first().ok_or_else(|| error("missing x"))?,
        *v.get(1).ok_or_else(|| error("missing y"))?,
        *v.get(2).ok_or_else(|| error("missing z"))?,
    );
    let [w, x, y, z] = p.quaternion;
    let q = Vec3::new(x, y, z);
    let rotate = |v: Vec3| v + q.cross(v) * (2. * w) + q.cross(q.cross(v)) * 2.;
    let axes = [Vec3::new(1., 0., 0.), Vec3::new(0., 1., 0.), Vec3::new(0., 0., 1.)].map(|v| apply_vector(&placement, rotate(v)));
    if axes.iter().any(|v| !v.is_finite() || (v.dot(*v) - 1.).abs() > 1e-7)
        || axes[0].dot(axes[1]).abs() > 1e-7
        || axes[0].cross(axes[1]).dot(axes[2]) < 1. - 1e-7
    {
        return Err(error("occurrence frame must be a proper rigid transform"));
    }
    let origin = apply_point(&placement, local);
    if !origin.is_finite() || origin.x.abs().max(origin.y.abs()).max(origin.z.abs()) > 1e9 {
        return Err(error("invalid resolved origin"));
    }
    Ok(Frame { origin, axes })
}
fn port(s: &mut Session, p: &Value) -> Result<Value> {
    let mut n = read(s)?;
    let key = id(p, "id")?;
    let mut value = p.clone();
    if !value.is_object() {
        return Err(error("port object required"));
    }
    for (k, v) in [
        ("origin", json!(["0 mm", "0 mm", "0 mm"])),
        ("quaternion", json!([1., 0., 0., 0.])),
        ("support", json!("point")),
        ("transfer", json!("frame_wrench")),
    ] {
        if value.get(k).is_none() {
            value[k] = v;
        }
    }
    let coordinates = value["origin"].as_array().filter(|a| a.len() == 3).ok_or_else(|| error("origin needs three length expressions"))?;
    let converted = coordinates.iter().map(|v| req_expr("mechanics.port", &json!({"v":v}), "v").map(|e| json!(e))).collect::<Result<Vec<_>>>()?;
    value["origin"] = json!(converted);
    let mut port: Port = serde_json::from_value(value).map_err(|e| error(e.to_string()))?;
    if valid_numbers(&port.quaternion) {
        let norm = port.quaternion.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-12 {
            for x in &mut port.quaternion {
                *x /= norm;
            }
        }
    }
    validate_port(&port)?;
    resolve(s, &port)?;
    if !n.ports.contains_key(&key) && n.ports.len() >= 128 {
        return Err(error("at most 128 ports"));
    }
    n.ports.insert(key, port);
    save(s, &n)
}
fn validate_graph(n: &Network) -> Result<()> {
    for (child, parent) in &n.parents {
        if !n.ports.contains_key(child) || !n.ports.contains_key(parent) {
            return Err(error("load path references missing port"));
        }
        let mut cursor = child.as_str();
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert(cursor) {
                return Err(error("load path cycle; redundant constraints are not supported"));
            }
            let Some(next) = n.parents.get(cursor) else { break };
            cursor = next;
        }
    }
    Ok(())
}
fn connect(s: &mut Session, p: &Value) -> Result<Value> {
    let mut n = read(s)?;
    let parent = id(p, "parent")?;
    let child = id(p, "child")?;
    if n.parents.get(&child).is_some_and(|v| *v != parent) {
        return Err(error("child already has another parent; parallel load paths are not supported"));
    }
    n.parents.insert(child, parent);
    validate_graph(&n)?;
    save(s, &n)
}
fn disconnect(s: &mut Session, p: &Value) -> Result<Value> {
    let mut n = read(s)?;
    let child = id(p, "child")?;
    if n.parents.remove(&child).is_none() {
        return Err(error("port has no parent link"));
    }
    save(s, &n)
}
fn validate_case(n: &Network, c: &Case) -> Result<()> {
    id(&json!({"id":c.id}), "id")?;
    if !n.ports.contains_key(&c.root) || c.loads.is_empty() || c.loads.len() > 128 {
        return Err(error("load case requires an existing root and 1..128 loads"));
    }
    for l in &c.loads {
        if !n.ports.contains_key(&l.port)
            || l.force_n.is_some_and(|v| !valid_numbers(&v))
            || l.moment_nm.is_some_and(|v| !valid_numbers(&v))
            || l.pressure_mpa.is_some_and(|v| !v.is_finite() || v.abs() > 1e9)
        {
            return Err(error("load references missing port or non-finite/unbounded values"));
        }
        if l.pressure_mpa.is_some() && (l.force_n.is_some() || l.moment_nm.is_some()) {
            return Err(error("choose pressure or force/moment components"));
        }
        if l.pressure_mpa.is_none() && l.force_n.is_none() && l.moment_nm.is_none() {
            return Err(error("empty load"));
        }
    }
    Ok(())
}
fn case(s: &mut Session, p: &Value) -> Result<Value> {
    let mut n = read(s)?;
    let c: Case = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    validate_case(&n, &c)?;
    if !n.cases.contains_key(&c.id) && n.cases.len() >= 32 {
        return Err(error("at most 32 load cases"));
    }
    let key = c.id.clone();
    n.cases.insert(key.clone(), c);
    calculation(s, &n, &key)?;
    save(s, &n)
}
fn calculation(s: &Session, n: &Network, key: &str) -> Result<Value> {
    let c = n.cases.get(key).ok_or_else(|| error("load case not found"))?;
    validate_case(n, c)?;
    if n.parents.contains_key(&c.root) {
        return Err(error("load case root must have no parent"));
    }
    let mut frames = BTreeMap::new();
    let mut total: BTreeMap<String, (Vec3, Vec3)> = BTreeMap::new();
    let mut paths = Vec::new();
    let get_frame = |key: &str| -> Result<Frame> { resolve(s, n.ports.get(key).ok_or_else(|| error("port missing"))?) };
    let root = get_frame(&c.root)?;
    let mut applied_force = Vec3::ZERO;
    let mut applied_moment = Vec3::ZERO;
    for l in &c.loads {
        let p = n.ports.get(&l.port).ok_or_else(|| error("port missing"))?;
        let source = get_frame(&l.port)?;
        let f = if let Some(pressure) = l.pressure_mpa {
            if p.transfer != "pressure" {
                return Err(error("pressure load requires a pressure surface port"));
            }
            Vec3::new(0., 0., pressure * p.area_mm2.ok_or_else(|| error("area missing"))?)
        } else {
            if p.transfer == "pressure" {
                return Err(error("pressure port requires a uniform normal pressure load"));
            }
            Vec3::from(l.force_n.unwrap_or([0.; 3]))
        };
        let m = Vec3::from(l.moment_nm.unwrap_or([0.; 3]));
        if p.transfer == "point_force" && m.dot(m) > 1e-18 {
            return Err(error("point force port cannot accept a moment"));
        }
        let force = source.rotate(f);
        let moment = source.rotate(m);
        applied_force += force;
        applied_moment = applied_moment + moment + (source.origin - root.origin).cross(force) * 0.001;
        let mut cursor = l.port.as_str();
        let mut path = Vec::new();
        loop {
            if path.len() >= 128 {
                return Err(error("load path too long"));
            }
            let frame = get_frame(cursor)?;
            let shifted = moment + (source.origin - frame.origin).cross(force) * 0.001;
            let entry = total.entry(cursor.into()).or_insert((Vec3::ZERO, Vec3::ZERO));
            entry.0 += force;
            entry.1 += shifted;
            frames.insert(cursor.to_owned(), frame);
            path.push(cursor.to_owned());
            if cursor == c.root {
                break;
            }
            cursor = n.parents.get(cursor).ok_or_else(|| error(format!("{} has no path to root {}", l.port, c.root)))?;
        }
        paths.push(path);
    }
    let mut rows = Vec::new();
    for (key, (f, m)) in &total {
        let p = n.ports.get(key).ok_or_else(|| error("port missing"))?;
        if p.transfer == "point_force" && m.dot(*m).sqrt() > 1e-9 * f.dot(*f).sqrt().max(1.) {
            return Err(error(format!("{key}: point-force interface cannot carry the required resultant moment")));
        }
        if p.transfer == "pressure" && (key == &c.root || n.parents.values().any(|parent| parent == key)) {
            return Err(error("pressure port must be a loaded leaf, not a relay or root"));
        }
        if !f.is_finite() || !m.is_finite() {
            return Err(error("non-finite static resultant"));
        }
        rows.push(json!({"port":key,"force_world_n":f,"moment_world_nm":m,"origin_world_mm":frames.get(key).map(|f|f.origin)}));
    }
    let (force, moment) = total.get(&c.root).copied().ok_or_else(|| error("root has no load"))?;
    let snapshot = json!({"ports":n.ports,"parents":n.parents,"case":c,"resolved_frames":frames,
        "cad_features":s.doc.features,"cad_parameters":s.doc.params,"cad_units":s.doc.units});
    if serde_json::to_vec(&snapshot).map_err(|e| error(e.to_string()))?.len() > 2 * 1024 * 1024 {
        return Err(error("input snapshot exceeds 2 MiB; reduce reference model size"));
    }
    Ok(json!({"model":"rigid_tree_static_resultants/1","case":key,"paths":paths,"resultants":rows,"snapshot":snapshot,
        "root_reaction_world":{"force_n":-force,"moment_nm":-moment},"root_reaction_local":{"force_n":root.local(-force),"moment_nm":root.local(-moment)},
        "equilibrium_residual":{"force_n":applied_force-force,"moment_nm":applied_moment-moment},
        "units":{"position":"mm","force":"N","moment":"Nm","pressure":"MPa = N/mm2"},
        "assumptions":["Manually declared fixed load-path tree, including branch superposition; no native joint reaction solution or automatic CAD constraint inference.",
            "No deformation, dynamics, gravity, contact or FEM nodal distribution. Surface area is authored, not measured from CAD.",
            "Port origin is authored in component CAD coordinates; exact occurrence placement and local frame orientation are applied. Surface pressure resultant assumes its declared centroid is the port origin."],
        "derivation":"M_at_target = R*M_local + ((origin_source_mm-origin_target_mm)/1000) cross (R*F_local); root reaction = negative total external wrench"}))
}
fn evaluate(s: &mut Session, p: &Value) -> Result<Value> {
    let n = read(s)?;
    calculation(s, &n, &id(p, "case")?)
}
fn solve(s: &mut Session, p: &Value) -> Result<Value> {
    let mut n = read(s)?;
    let key = id(p, "case")?;
    let report = calculation(s, &n, &key)?;
    n.results.insert(key, report.clone());
    save(s, &n)?;
    Ok(report)
}
pub(super) fn model_ports(s: &Session) -> Result<Vec<Value>> {
    Ok(read(s)?
        .ports
        .values()
        .map(|p| match resolve(s, p) {
            Ok(frame) => json!({"port":p,"valid":true,"frame":frame}),
            Err(e) => json!({"port":p,"valid":false,"error":e.to_string()}),
        })
        .collect())
}
fn state(s: &mut Session, _: &Value) -> Result<Value> {
    let n = read(s)?;
    let frames = n
        .ports
        .iter()
        .map(|(key, p)| match resolve(s, p) {
            Ok(f) => json!({"id":key,"valid":true,"frame":f}),
            Err(e) => json!({"id":key,"valid":false,"error":e.to_string()}),
        })
        .collect::<Vec<_>>();
    let statuses = n
        .results
        .iter()
        .map(|(key, old)| match calculation(s, &n, key) {
            Ok(now) => json!({"case":key,"current":old["snapshot"]==now["snapshot"]}),
            Err(e) => json!({"case":key,"current":false,"error":e.to_string()}),
        })
        .collect::<Vec<_>>();
    Ok(json!({"network":n,"resolved_ports":frames,"results":statuses,"persistence":"project_metadata","dynamic_simulation":false}))
}
