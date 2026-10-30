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
    dimension: String,
    displacement: String,
    profile: String,
    ramp_s: f64,
    frequency_hz: f64,
}
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
    if !["angle", "length"].contains(&src.dimension.as_str()) {
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
    s.doc.eval(
        &p.source.displacement,
        if p.source.dimension == "angle" { solidcraft_doc::expr::Kind::Angle } else { solidcraft_doc::expr::Kind::Length },
    )?;
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
    write(s, g, Some(id))
}
fn connect(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Connect = parse(p)?;
    let mut g = load(s)?;
    let src = g.sources.get(&p.source).ok_or_else(|| error("source missing"))?;
    if joint_dimension(s, p.joint)? != src.dimension {
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
        let src = g.sources.get(id).ok_or_else(|| error("source missing"))?;
        if joint_dimension(s, *joint)? != src.dimension {
            return Err(error("source and joint units changed; repair connection"));
        }
        drivers
            .push(json!({"joint":joint,"displacement":src.displacement,"profile":src.profile,"ramp_s":src.ramp_s,"frequency_hz":src.frequency_hz}));
    }
    drivers.sort_by_key(|d| d["joint"].as_u64().unwrap_or(0));
    obj.insert("drivers".into(), json!(drivers));
    Ok(p)
}
