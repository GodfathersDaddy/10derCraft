//! CAD-picked interfaces reuse associative native joint origins, not duplicate MKS geometry.
use super::CommandSpec;
use crate::{
    Result, Sel, Session,
    params::bad,
    schema::{TypedContract, output_schema_for},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Capture {
    name: String,
    occurrence: u64,
    transfer: String,
    #[serde(default)]
    surface: bool,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    role: Option<String>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Connect {
    name: String,
    a: String,
    b: String,
    kind: String,
    #[serde(default)]
    flip: bool,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Description {
    name: String,
    label: String,
    role: String,
}
#[derive(Serialize, JsonSchema)]
struct Reply {
    interfaces: Vec<Value>,
    selection: Vec<Value>,
    occurrences: Vec<Value>,
}
#[derive(Serialize, JsonSchema)]
struct Connected {
    joint: u64,
}
const CAPTURE: TypedContract<Capture> = TypedContract::new();
const CONNECT: TypedContract<Connect> = TypedContract::new();
const EMPTY: TypedContract<Empty> = TypedContract::new();
const DESCRIPTION: TypedContract<Description> = TypedContract::new();
pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("mks.interface.open", "CAD Simulation Interfaces", crate::checked!("mks.interface.open", read))
        .contract(&EMPTY)
        .output(output_schema_for::<Reply>)
        .noundo()
        .doc_only()
        .icon("joint_origin")
        .at("SOLID", "ASSEMBLE"),
    CommandSpec::new("mks.interface.read", "Read CAD Simulation Interfaces", crate::checked!("mks.interface.read", read))
        .icon("joint_origin")
        .contract(&EMPTY)
        .output(output_schema_for::<Reply>)
        .noundo()
        .doc_only(),
    CommandSpec::new("mks.interface.capture", "Capture CAD Simulation Interface", crate::checked!("mks.interface.capture", capture))
        .icon("joint_origin")
        .contract(&CAPTURE)
        .output(output_schema_for::<Reply>),
    CommandSpec::new("mks.interface.connect", "Join CAD Simulation Interfaces", crate::checked!("mks.interface.connect", connect))
        .icon("joint")
        .contract(&CONNECT)
        .output(output_schema_for::<Connected>),
    CommandSpec::new("mks.interface.describe", "Label CAD Simulation Interface", crate::checked!("mks.interface.describe", describe))
        .icon("settings")
        .contract(&DESCRIPTION)
        .output(output_schema_for::<Reply>),
];
fn error(message: impl Into<String>) -> crate::EngineError {
    bad("mks.interface", message)
}
pub(super) fn frame(s: &Session, name: &str) -> Result<solidcraft_doc::Mat> {
    let doc = s.placed_document();
    let origin = doc.assembly.origins.iter().find(|o| o.name == name).ok_or_else(|| error("CAD interface no longer exists"))?;
    solidcraft_doc::joints::resolve_origin(&doc, &s.model.state(), &origin.origin).map_err(|e| error(e.to_string()))
}
fn read(s: &mut Session, _: &Value) -> Result<Value> {
    let ports = super::mechanics::model_ports(s)?;
    let origins: Vec<_> = s
        .doc
        .assembly
        .origins
        .iter()
        .take(256)
        .map(|o| match frame(s, &o.name) {
            Ok(m) => {
                let port = ports.iter().find(|p| p["port"]["joint_origin"] == o.name);
                json!({"name":o.name,"occurrence":o.origin.occurrence,"reference":o.origin.snap,"valid":port.is_none_or(|p|p["valid"]==true),"frame":m,"mechanical":port})
            },
            Err(e) => json!({"name":o.name,"occurrence":o.origin.occurrence,"valid":false,"error":e.to_string()}),
        })
        .collect();
    let occurrences: Vec<_> = std::iter::once(json!({"id":0,"name":"World / Root","component":0}))
        .chain(s.doc.occurrences.iter().filter(|o| o.parent == 0).take(1024).map(|o| json!({"id":o.id,"name":o.name,"component":o.component})))
        .collect();
    let selection: Vec<_> = s
        .selection
        .iter()
        .filter_map(|p| {
            if let Sel::Face { body, index, point } = p {
                Some(json!({"body":body,"index":index,"point":point,"component":s.model.state().body(body).map(|b|s.doc.model_body_component(b))}))
            } else {
                None
            }
        })
        .collect();
    Ok(json!({"interfaces":origins,"selection":selection,"occurrences":occurrences}))
}
fn capture(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Capture = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    if p.name.is_empty()
        || p.name.len() > 64
        || !p.name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        || !["point_force", "frame_wrench"].contains(&p.transfer.as_str())
    {
        return Err(error("use a unique interface ID (letters, digits, _ or -), and point_force/frame_wrench"));
    }
    if s.doc.assembly.origins.len() >= 256 {
        return Err(error("at most 256 named interfaces"));
    }
    let [Sel::Face { body, index, .. }] = s.selection.as_slice() else {
        return Err(error("select exactly one planar CAD face first"));
    };
    let st = s.model.state();
    let body_model = st.body(body).ok_or_else(|| error("selected native body missing"))?;
    let component = s.doc.model_body_component(body_model);
    let occurrence_component = if p.occurrence == 0 {
        0
    } else {
        s.doc
            .occurrences
            .iter()
            .find(|o| o.id == p.occurrence && o.parent == 0)
            .ok_or_else(|| error("select a top-level occurrence; nested placements require the native Joint Origin tool"))?
            .component
    };
    if component != occurrence_component {
        return Err(error("selected face belongs to a different component"));
    }
    let face = solidcraft_doc::naming::face_names(body_model).get(*index).cloned().ok_or_else(|| error("selected CAD face no longer exists"))?;
    let area = if p.surface { Some(super::mechanics::cad_face_area(s, body, &face)?) } else { None };
    // Named native face snaps are re-resolved after dimension and placement edits.
    let origin = super::joints::origin_param(s, Some(&json!({"occurrence":p.occurrence,"face":face,"body":body})), "mks.interface.capture")?;
    if !matches!(origin.snap, solidcraft_doc::joints::Snap::FaceCenter { face: Some(_), .. }) {
        return Err(error("face has no persistent native name"));
    }
    let mut next = s.scratch();
    next.execute("joint.origin", &json!({"name":p.name,"occurrence":p.occurrence,"face":face,"body":body}))?;
    // MKS consumes the resultant wrench at this associative face frame. Distributed
    // pressure/FE load interpolation is deliberately a different contract.
    let mut port = json!({"id":p.name,"body":body,"occurrence":if p.occurrence==0{Vec::<u64>::new()}else{vec![p.occurrence]},"joint_origin":p.name,
        "support":if p.surface{"surface"}else{"point"},"transfer":p.transfer,"label":p.label,"role":p.role});
    if let Some(area) = area {
        port["area_mm2"] = json!(area);
        port["area_face"] = json!(face);
    }
    next.execute("mechanics.port", &port)?;
    super::worksheet::commit(s, next)?;
    read(s, &json!({}))
}
fn connect(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Connect = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    if p.a == p.b || p.name.trim().is_empty() || p.name.len() > 100 || !["rigid", "revolute", "slider"].contains(&p.kind.as_str()) {
        return Err(error("two distinct interfaces, bounded name and rigid/revolute/slider required"));
    }
    for name in [&p.a, &p.b] {
        frame(s, name)?;
        if s.doc
            .metadata
            .get("org.solidcraft.mechanical-project")
            .and_then(|v| v.get("ports"))
            .and_then(|v| v.get(name))
            .is_some_and(|p| p["transfer"] == "point_force")
        {
            return Err(error("a point-force port cannot define a full mechanical joint; capture a force/moment frame"));
        }
    }
    let mut next = s.scratch();
    let reply = next.execute("joint.create", &json!({"name":p.name,"a":p.a,"b":p.b,"type":p.kind,"flip":p.flip}))?;
    super::worksheet::commit(s, next)?;
    Ok(json!({"joint":reply["joint"]}))
}
fn describe(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Description = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    let ports = super::mechanics::model_ports(s)?;
    let mut port =
        ports.iter().find(|v| v["port"]["joint_origin"] == p.name).ok_or_else(|| error("CAD-bound mechanical interface missing"))?["port"].clone();
    port["label"] = json!(p.label);
    port["role"] = json!(p.role);
    s.execute("mechanics.port", &port)?;
    read(s, &json!({}))
}
