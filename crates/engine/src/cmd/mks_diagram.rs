//! Persistent independent motion sources and typed fan-out into native joint inputs.
use super::CommandSpec;
use crate::{
    Result, Session,
    params::bad,
    schema::{TypedContract, output_schema_for},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub(super) const KEY: &str = "org.solidcraft.mks-diagram";

#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Source {
    name: String,
    #[serde(default = "motion_kind")]
    kind: String,
    #[serde(default = "real_type")]
    data_type: String,
    #[serde(default)]
    upstream: Option<u64>,
    #[serde(default)]
    link: String,
    #[serde(default)]
    sensor_joint: Option<u64>,
    #[serde(default)]
    value: Option<Value>,
    dimension: String,
    displacement: String,
    profile: String,
    ramp_s: f64,
    frequency_hz: f64,
}
fn motion_kind() -> String {
    "motion".into()
}
fn real_type() -> String {
    "real".into()
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Route {
    source: u64,
    target: u64,
    #[serde(default)]
    replace: bool,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Sample {
    id: u64,
    time_s: f64,
}
#[derive(Serialize, JsonSchema)]
struct SignalSample {
    id: u64,
    time_s: f64,
    data_type: String,
    dimension: String,
    value: Value,
}
const ROUTE: TypedContract<Route> = TypedContract::new();
const SAMPLE: TypedContract<Sample> = TypedContract::new();
#[derive(Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
struct Graph {
    sources: BTreeMap<u64, Source>,
    // None deliberately masks a former/legacy driver after disconnection.
    inputs: BTreeMap<u64, Option<u64>>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SourceParams {
    id: Option<u64>,
    source: Source,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Connect {
    source: u64,
    joint: u64,
    #[serde(default)]
    replace: bool,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Joint {
    joint: u64,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Id {
    id: u64,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Serialize, JsonSchema)]
struct Reply {
    graph: Graph,
    source: Option<u64>,
}
const SOURCE: TypedContract<SourceParams> = TypedContract::new();
const CONNECT: TypedContract<Connect> = TypedContract::new();
const JOINT: TypedContract<Joint> = TypedContract::new();
const ID: TypedContract<Id> = TypedContract::new();
const EMPTY: TypedContract<Empty> = TypedContract::new();
pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("mks.diagram.route", "Connect Typed Signal Ports", crate::checked!("mks.diagram.route", route))
        .icon("joint")
        .contract(&ROUTE)
        .output(output_schema_for::<Reply>)
        .doc_only(),
    CommandSpec::new("mks.diagram.sample", "Read Typed Signal Value", crate::checked!("mks.diagram.sample", sample_signal))
        .icon("graph")
        .contract(&SAMPLE)
        .output(output_schema_for::<SignalSample>)
        .noundo()
        .doc_only(),
    CommandSpec::new("mks.diagram.read", "Read MKS Connections", crate::checked!("mks.diagram.read", read))
        .icon("graph")
        .contract(&EMPTY)
        .output(output_schema_for::<Reply>)
        .noundo()
        .doc_only(),
    CommandSpec::new("mks.diagram.source", "Edit Independent Motion Source", crate::checked!("mks.diagram.source", source))
        .icon("drive")
        .contract(&SOURCE)
        .output(output_schema_for::<Reply>)
        .doc_only(),
    CommandSpec::new("mks.diagram.connect", "Connect Motion Source", crate::checked!("mks.diagram.connect", connect))
        .icon("joint")
        .contract(&CONNECT)
        .output(output_schema_for::<Reply>)
        .doc_only(),
    CommandSpec::new("mks.diagram.disconnect", "Disconnect Motion Input", crate::checked!("mks.diagram.disconnect", disconnect))
        .icon("delete")
        .contract(&JOINT)
        .output(output_schema_for::<Reply>)
        .doc_only(),
    CommandSpec::new("mks.diagram.release", "Use Local Motion Input", crate::checked!("mks.diagram.release", release))
        .icon("joint")
        .contract(&JOINT)
        .output(output_schema_for::<Reply>)
        .doc_only(),
    CommandSpec::new("mks.diagram.remove", "Remove Motion Source", crate::checked!("mks.diagram.remove", remove))
        .icon("delete")
        .contract(&ID)
        .output(output_schema_for::<Reply>)
        .doc_only(),
];
fn error(message: impl Into<String>) -> crate::EngineError {
    bad("mks.diagram", message)
}
fn parse<T: serde::de::DeserializeOwned>(p: &Value) -> Result<T> {
    serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))
}
fn load(s: &Session) -> Result<Graph> {
    let graph: Graph = match s.doc.metadata.get(KEY) {
        Some(v) => parse(v)?,
        None => Graph::default(),
    };
    if graph.sources.len() > 32 || graph.inputs.len() > 128 {
        return Err(error("at most 32 sources and 128 joint inputs"));
    }
    for src in graph.sources.values() {
        validate_source(src)?;
    }
    for id in graph.inputs.values().flatten() {
        if !graph.sources.contains_key(id) {
            return Err(error("connection source no longer exists"));
        }
    }
    validate_routes(&graph)?;
    Ok(graph)
}
fn validate_source(src: &Source) -> Result<()> {
    if src.name.trim().is_empty()
        || src.name.len() > 100
        || src.displacement.len() > 256
        || !super::mks_signals::valid(&src.profile)
        || !src.ramp_s.is_finite()
        || !(0.000001..=10.).contains(&src.ramp_s)
        || !src.frequency_hz.is_finite()
        || !(0.01..=20.).contains(&src.frequency_hz)
    {
        return Err(error("invalid source name, profile, ramp or frequency"));
    }
    if !["motion", "input", "output", "link"].contains(&src.kind.as_str())
        || !["real", "integer", "boolean", "vector3"].contains(&src.data_type.as_str())
        || src.link.len() > 64
    {
        return Err(error("invalid signal port kind, type or link name"));
    }
    if src.kind == "motion" && (src.data_type != "real" || !["angle", "length"].contains(&src.dimension.as_str())) {
        return Err(error("motion sources require real angle or length"));
    }
    if let Some(value) = &src.value {
        let number = |v: &Value| v.as_f64().is_some_and(|v| v.is_finite() && v.abs() <= 1e12);
        let valid = match src.data_type.as_str() {
            "boolean" => value.is_boolean(),
            "integer" => value.as_i64().is_some(),
            "vector3" => value.as_array().is_some_and(|v| v.len() == 3 && v.iter().all(number)),
            _ => number(value),
        };
        if !valid || src.upstream.is_some() || src.sensor_joint.is_some() {
            return Err(error("constant signal type mismatch or multiple signal drivers"));
        }
    }
    if src.upstream.is_some() && src.sensor_joint.is_some() {
        return Err(error("signal input has two drivers"));
    }
    if !["angle", "length", "force", "torque", "scalar", "velocity", "angular_velocity", "linear_velocity"].contains(&src.dimension.as_str()) {
        return Err(error("source dimension must be angle or length"));
    }
    Ok(())
}
fn joint_dimension(s: &Session, id: u64) -> Result<&'static str> {
    use solidcraft_doc::joints::JointKind;
    let joint = s.doc.assembly.joints.iter().find(|j| j.id == id && !j.suppressed).ok_or_else(|| error("active native joint missing"))?;
    match joint.kind {
        JointKind::Revolute => Ok("angle"),
        JointKind::Slider => Ok("length"),
        _ => Err(error("motion input requires a revolute or slider joint")),
    }
}
fn write(s: &mut Session, g: Graph, id: Option<u64>) -> Result<Value> {
    s.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY, serde_json::to_value(&g).map_err(|e| error(e.to_string()))?)?;
    Ok(json!({"graph":g,"source":id}))
}
fn read(s: &mut Session, _: &Value) -> Result<Value> {
    Ok(json!({"graph":load(s)?,"source":null}))
}
fn source(s: &mut Session, p: &Value) -> Result<Value> {
    let p: SourceParams = parse(p)?;
    validate_source(&p.source)?;
    if p.source.kind == "motion" {
        s.doc.eval(
            &p.source.displacement,
            if p.source.dimension == "angle" { solidcraft_doc::expr::Kind::Angle } else { solidcraft_doc::expr::Kind::Length },
        )?;
    }
    if p.source.kind == "motion" && (p.source.upstream.is_some() || p.source.sensor_joint.is_some() || p.source.value.is_some()) {
        return Err(error("motion source cannot have additional drivers"));
    }
    if let Some(joint) = p.source.sensor_joint {
        let dimension = joint_dimension(s, joint)?;
        if !["real", "vector3"].contains(&p.source.data_type.as_str())
            || (p.source.data_type == "vector3" && !["force", "torque"].contains(&p.source.dimension.as_str()))
        {
            return Err(error("sensor signal requires a real scalar or a force/torque vector"));
        }
        if ["angle", "length"].contains(&p.source.dimension.as_str()) && p.source.dimension != dimension {
            return Err(error("sensor coordinate dimension mismatch"));
        }
        if p.source.dimension == "angular_velocity" && dimension != "angle" || p.source.dimension == "linear_velocity" && dimension != "length" {
            return Err(error("sensor velocity dimension mismatch"));
        }
    }
    let mut g = load(s)?;
    let id = if let Some(id) = p.id {
        if !g.sources.contains_key(&id) {
            return Err(error("source missing"));
        }
        id
    } else {
        if g.sources.len() >= 32 {
            return Err(error("source limit reached"));
        }
        g.sources.keys().copied().max().unwrap_or(0).checked_add(1).ok_or_else(|| error("source id exhausted"))?
    };
    for (joint, linked) in &g.inputs {
        if *linked == Some(id) && joint_dimension(s, *joint)? != p.source.dimension {
            return Err(error("disconnect incompatible joint inputs before changing source dimension"));
        }
    }
    g.sources.insert(id, p.source);
    validate_routes(&g)?;
    write(s, g, Some(id))
}
fn connect(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Connect = parse(p)?;
    let mut g = load(s)?;
    let src = resolve(&g, p.source)?;
    if src.kind != "motion" || src.data_type != "real" || joint_dimension(s, p.joint)? != src.dimension {
        return Err(error("source and joint input have incompatible units"));
    }
    if g.inputs.len() >= 128 && !g.inputs.contains_key(&p.joint) {
        return Err(error("input limit reached"));
    }
    let already = g.inputs.get(&p.joint).copied().flatten();
    let legacy = s
        .doc
        .metadata
        .get(super::worksheet::PRESENTATION)
        .and_then(|v| v.get("mks"))
        .and_then(|v| v.get("settings"))
        .or_else(|| s.doc.metadata.get("org.solidcraft.assembly-simulation").and_then(|v| v.get("job")).and_then(|v| v.get("settings")))
        .and_then(|v| v.get("drivers"))
        .and_then(Value::as_array)
        .is_some_and(|v| v.iter().any(|d| d["joint"] == p.joint));
    if !p.replace && (already.is_some_and(|id| id != p.source) || (!g.inputs.contains_key(&p.joint) && legacy)) {
        return Err(error("joint already driven; explicitly replace its input"));
    }
    g.inputs.insert(p.joint, Some(p.source));
    write(s, g, Some(p.source))
}
fn disconnect(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Joint = parse(p)?;
    joint_dimension(s, p.joint)?;
    let mut g = load(s)?;
    if g.inputs.len() >= 128 && !g.inputs.contains_key(&p.joint) {
        return Err(error("input limit reached"));
    }
    g.inputs.insert(p.joint, None);
    write(s, g, None)
}
fn remove(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Id = parse(p)?;
    let mut g = load(s)?;
    if g.sources.remove(&p.id).is_none() {
        return Err(error("source missing"));
    }
    for source in g.sources.values_mut() {
        if source.upstream == Some(p.id) {
            source.upstream = None;
        }
    }
    for linked in g.inputs.values_mut() {
        if *linked == Some(p.id) {
            *linked = None;
        }
    }
    write(s, g, None)
}
fn release(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Joint = parse(p)?;
    joint_dimension(s, p.joint)?;
    let mut g = load(s)?;
    g.inputs.remove(&p.joint);
    write(s, g, None)
}
fn parent(g: &Graph, source: &Source) -> Result<Option<u64>> {
    if source.kind == "link" {
        let matches: Vec<_> = g.sources.iter().filter(|(_, v)| v.kind == "output" && !source.link.is_empty() && v.link == source.link).collect();
        if matches.len() > 1 {
            return Err(error("ambiguous named signal export"));
        }
        return Ok(matches.first().map(|(id, _)| **id));
    }
    Ok(source.upstream)
}
fn validate_routes(g: &Graph) -> Result<()> {
    let mut exports = std::collections::BTreeSet::new();
    for source in g.sources.values() {
        if source.kind == "output" && !source.link.is_empty() && !exports.insert(&source.link) {
            return Err(error("named exports must be unique"));
        }
    }
    for id in g.sources.keys() {
        let mut visited = std::collections::BTreeSet::new();
        let mut current = Some(*id);
        while let Some(id) = current {
            if !visited.insert(id) {
                return Err(error("cyclic signal connection"));
            }
            let source = g.sources.get(&id).ok_or_else(|| error("signal upstream missing"))?;
            current = parent(g, source)?;
            if let Some(parent) = current {
                let upstream = g.sources.get(&parent).ok_or_else(|| error("signal upstream missing"))?;
                if upstream.dimension != source.dimension || upstream.data_type != source.data_type {
                    return Err(error("signal data type or physical dimension mismatch"));
                }
            }
        }
    }
    Ok(())
}
fn resolve(g: &Graph, id: u64) -> Result<&Source> {
    let mut current = id;
    for _ in 0..=g.sources.len() {
        let source = g.sources.get(&current).ok_or_else(|| error("signal source missing"))?;
        if source.kind == "motion" || source.sensor_joint.is_some() || source.value.is_some() {
            return Ok(source);
        }
        current = parent(g, source)?.ok_or_else(|| error("unconnected signal input or missing named export"))?;
    }
    Err(error("cyclic signal graph"))
}
fn route(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Route = parse(p)?;
    let mut graph = load(s)?;
    let source = graph.sources.get(&p.source).ok_or_else(|| error("source missing"))?;
    let target = graph.sources.get(&p.target).ok_or_else(|| error("target missing"))?;
    if !["input", "output"].contains(&target.kind.as_str()) || source.dimension != target.dimension || source.data_type != target.data_type {
        return Err(error("incompatible input/output signal ports"));
    }
    if target.upstream.is_some() && !p.replace {
        return Err(error("signal input already connected"));
    }
    let target = graph.sources.get_mut(&p.target).ok_or_else(|| error("target missing"))?;
    target.upstream = Some(p.source);
    target.sensor_joint = None;
    target.value = None;
    validate_routes(&graph)?;
    write(s, graph, Some(p.target))
}
fn sample_signal(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Sample = parse(p)?;
    if !p.time_s.is_finite() || !(0. ..=10.).contains(&p.time_s) {
        return Err(error("signal time outside 0..10 seconds"));
    }
    let graph = load(s)?;
    validate_routes(&graph)?;
    let node = graph.sources.get(&p.id).ok_or_else(|| error("port missing"))?;
    let source = resolve(&graph, p.id)?;
    let value = if let Some(value) = &source.value {
        value.clone()
    } else if let Some(joint) = source.sensor_joint {
        let state = s.execute("mks.job.state", &json!({}))?;
        if state["result_current"] != true {
            return Err(error("current simulation result required for sensor signal"));
        }
        let result = &state["data"]["result"];
        let times: Vec<f64> = serde_json::from_value(result["times_s"].clone()).map_err(|e| error(e.to_string()))?;
        if times.len() < 2
            || times.iter().any(|t| !t.is_finite())
            || times.windows(2).any(|w| w[1] <= w[0])
            || p.time_s < times[0]
            || p.time_s > *times.last().ok_or_else(|| error("empty sensor history"))?
        {
            return Err(error("sensor time outside result"));
        }
        if p.time_s == 0. && ["force", "torque"].contains(&source.dimension.as_str()) && result["numerics"]["force_initial_sample_valid"] == false {
            return Err(error("initial force sample is not valid"));
        }
        let a = times.partition_point(|t| *t <= p.time_s).saturating_sub(1).min(times.len() - 2);
        let b = a + 1;
        let u = (p.time_s - times[a]) / (times[b] - times[a]);
        let key = match source.dimension.as_str() {
            "force" => "joint_forces_local_n",
            "torque" => "joint_torques_local_nm",
            _ => "joint_coordinates_si",
        };
        let rows = &result[key][joint.to_string()];
        let indexes: Vec<usize> = if source.data_type == "vector3" {
            vec![0, 1, 2]
        } else {
            vec![if ["velocity", "angular_velocity", "linear_velocity"].contains(&source.dimension.as_str()) { 1 } else { 0 }]
        };
        let mut values = Vec::new();
        for index in indexes {
            let va = rows[a][index].as_f64().ok_or_else(|| error("sensor channel missing"))?;
            let vb = rows[b][index].as_f64().ok_or_else(|| error("sensor channel missing"))?;
            values.push(va * (1. - u) + vb * u);
        }
        if source.data_type == "vector3" { json!(values) } else { json!(values[0]) }
    } else {
        let sampled = super::mks_signals::value_at(&source.profile, p.time_s, source.ramp_s, source.frequency_hz);
        let amplitude = s.doc.eval(
            &source.displacement,
            if source.dimension == "angle" { solidcraft_doc::expr::Kind::Angle } else { solidcraft_doc::expr::Kind::Length },
        )?;
        json!(sampled * if source.dimension == "length" { amplitude * 0.001 } else { amplitude })
    };
    serde_json::to_value(SignalSample { id: p.id, time_s: p.time_s, data_type: node.data_type.clone(), dimension: node.dimension.clone(), value })
        .map_err(|e| error(e.to_string()))
}

/// Only effective motion inputs affect a dynamics result. Observer ports do not.
pub(super) fn physical_inputs(s: &Session) -> Result<Value> {
    let graph = load(s)?;
    let mut inputs = serde_json::Map::new();
    for (joint, source) in &graph.inputs {
        let value = if let Some(source) = source {
            let source = resolve(&graph, *source)?;
            json!({"dimension":source.dimension,"data_type":source.data_type,"displacement":source.displacement,"value":source.value,"profile":source.profile,"ramp_s":source.ramp_s,"frequency_hz":source.frequency_hz})
        } else {
            Value::Null
        };
        inputs.insert(joint.to_string(), value);
    }
    Ok(Value::Object(inputs))
}

/// Lower the graph into the existing solver command, with no UI-only physics path.
pub(super) fn settings(s: &Session, p: &Value) -> Result<Value> {
    if s.doc.metadata.get(KEY).is_none() {
        return Ok(p.clone());
    }
    let g = load(s)?;
    let mut p = p.clone();
    let obj = p.as_object_mut().ok_or_else(|| error("job settings must be an object"))?;
    let mut drivers = match obj.get("drivers") {
        None => Vec::new(),
        Some(Value::Array(v)) => v.clone(),
        _ => return Err(error("drivers must be an array")),
    };
    drivers.retain(|d| !d["joint"].as_u64().is_some_and(|id| g.inputs.contains_key(&id)));
    for (joint, linked) in &g.inputs {
        let Some(id) = linked else { continue };
        let src = resolve(&g, *id)?;
        if src.kind != "motion" || src.data_type != "real" || joint_dimension(s, *joint)? != src.dimension {
            return Err(error("source and joint units changed; repair connection"));
        }
        let displacement = src.displacement.clone();
        drivers.push(json!({"joint":joint,"displacement":displacement,"profile":src.profile,"ramp_s":src.ramp_s,"frequency_hz":src.frequency_hz}));
    }
    drivers.sort_by_key(|d| d["joint"].as_u64().unwrap_or(0));
    obj.insert("drivers".into(), json!(drivers));
    Ok(p)
}
