//! Reusable library factories. Bodies and joints are ordinary editable native CAD elements.
use super::CommandSpec;
use crate::{
    Result, Session,
    params::bad,
    schema::{TypedContract, output_schema_for},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
const KEY: &str = "org.solidcraft.mks-library";
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BodyParams {
    name: String,
    mass_kg: f64,
    size_mm: [f64; 3],
    #[serde(default)]
    position_mm: [f64; 3],
    #[serde(default)]
    port: Option<String>,
    #[serde(default)]
    grounded: bool,
}
#[derive(Serialize, JsonSchema)]
struct BodyOutput {
    component: u64,
    occurrence: u64,
    body: String,
    mass_kg: f64,
    attachment_port: Option<String>,
    joint: Option<u64>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct JointParams {
    name: String,
    kind: String,
    a: u64,
    b: u64,
    point_mm: [f64; 3],
    axis: [f64; 3],
}
#[derive(Serialize, JsonSchema)]
struct JointOutput {
    joint: u64,
}
const BODY: TypedContract<BodyParams> = TypedContract::new();
const JOINT: TypedContract<JointParams> = TypedContract::new();
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct MassParams {
    occurrence: u64,
    mass_kg: f64,
}
const MASS: TypedContract<MassParams> = TypedContract::new();
pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("mks.library.body", "Insert MKS Body", crate::checked!("mks.library.body", body))
        .icon("component")
        .contract(&BODY)
        .output(output_schema_for::<BodyOutput>),
    CommandSpec::new("mks.library.joint", "Insert MKS Joint", crate::checked!("mks.library.joint", joint))
        .icon("joint")
        .contract(&JOINT)
        .output(output_schema_for::<JointOutput>),
    CommandSpec::new("mks.library.mass", "Set MKS Payload Mass", crate::checked!("mks.library.mass", mass))
        .icon("center_of_mass")
        .contract(&MASS)
        .output(output_schema_for::<BodyOutput>),
];
fn mass(s: &mut Session, p: &Value) -> Result<Value> {
    let p: MassParams = serde_json::from_value(p.clone()).map_err(|e| bad("mks.library.mass", e.to_string()))?;
    if !p.mass_kg.is_finite() || !(0.000001..=100000.).contains(&p.mass_kg) {
        return Err(bad("mks.library.mass", "mass must be positive and bounded"));
    }
    let mut entries = s.doc.metadata.get(KEY).and_then(|v| v.get("items")).and_then(Value::as_array).cloned().unwrap_or_default();
    let entry = entries.iter_mut().find(|v| v["occurrence"] == p.occurrence).ok_or_else(|| bad("mks.library.mass", "library body not found"))?;
    let physical = super::mks_mass::calculate(s, &json!({}))?;
    let body = physical["blocks"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|b| b["bodies"].as_array().into_iter().flatten())
        .find(|b| b["source"] == entry["body"] && b["component"] == entry["component"])
        .ok_or_else(|| bad("mks.library.mass", "native mass properties unavailable"))?;
    let old = body["mass_kg"].as_f64().filter(|v| *v > 0.).ok_or_else(|| bad("mks.library.mass", "native body has no positive mass"))?;
    let density = body["density_kg_m3"].as_f64().ok_or_else(|| bad("mks.library.mass", "density missing"))? / 1000. * p.mass_kg / old;
    let mut next = s.scratch();
    next.execute("material.edit", &json!({"material":body["material"],"density_g_cm3":density}))?;
    entry["mass_kg"] = json!(p.mass_kg);
    let result = entry.clone();
    next.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY, json!({"items":entries}))?;
    super::worksheet::commit(s, next)?;
    Ok(result)
}
fn body(s: &mut Session, p: &Value) -> Result<Value> {
    let p: BodyParams = serde_json::from_value(p.clone()).map_err(|e| bad("mks.library.body", e.to_string()))?;
    if p.name.trim().is_empty()
        || p.name.len() > 100
        || !p.mass_kg.is_finite()
        || !(0.000001..=100000.).contains(&p.mass_kg)
        || p.size_mm.iter().any(|v| !v.is_finite() || !(0.1..=10000.).contains(v))
        || p.position_mm.iter().any(|v| !v.is_finite() || v.abs() > 1e7)
    {
        return Err(bad("mks.library.body", "bounded name, mass, size and position required"));
    }
    let density = p.mass_kg * 1e6 / p.size_mm.iter().product::<f64>();
    if density > 1000. {
        return Err(bad("mks.library.body", "mass too large for the chosen cuboid; increase its size"));
    }
    if p.grounded && p.port.is_some() {
        return Err(bad("mks.library.body", "a payload attached to a moving port cannot be grounded"));
    }
    let mut position = p.position_mm;
    let mut attach = None;
    if let Some(id) = &p.port {
        let graph = super::mks::model(s, &json!({}))?;
        let block = graph["blocks"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|b| b["ports"].as_array().into_iter().flatten().any(|v| v["port"]["id"] == *id))
            .ok_or_else(|| bad("mks.library.body", "attachment port does not resolve to a CAD body"))?;
        let port = block["ports"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|v| v["port"]["id"] == *id)
            .ok_or_else(|| bad("mks.library.body", "port missing"))?;
        position = serde_json::from_value(port["frame"]["origin"].clone()).map_err(|e| bad("mks.library.body", e.to_string()))?;
        let occurrence = port["port"]["occurrence"].as_array().and_then(|a| a.last()).and_then(Value::as_u64).unwrap_or(0);
        attach = Some(occurrence);
    }
    let mut next = s.scratch();
    next.execute("component.activate", &json!({"component":0}))?;
    let component = next.execute("component.create", &json!({"name":p.name,"parent":0,"grounded":p.grounded}))?;
    let cid = component["component"].as_u64().ok_or_else(|| bad("mks.library.body", "new component id missing"))?;
    let oid = component["occurrence"].as_u64().ok_or_else(|| bad("mks.library.body", "new occurrence id missing"))?;
    let names: Vec<_> = (0..3).map(|i| format!("mbs_{oid}_{}", ["L", "B", "H"][i])).collect();
    for (i, name) in names.iter().enumerate() {
        next.execute("worksheet.set", &json!({"name":name,"expression":format!("{} mm",p.size_mm[i]),"unit":"mm"}))?;
    }
    let body_name = format!("{}-{oid}", p.name);
    next.execute("solid.box",&json!({"length":names[0],"width":names[1],"height":names[2],"corner":[position[0]-p.size_mm[0]/2.,position[1]-p.size_mm[1]/2.,position[2]-p.size_mm[2]/2.],"body_name":body_name}))?;
    let material = format!("MKS Ersatzmasse {oid}");
    next.execute("material.create", &json!({"name":material,"density_g_cm3":density}))?;
    next.execute("material.assign", &json!({"bodies":[body_name],"material":material}))?;
    next.execute("component.activate", &json!({"component":0}))?;
    let joint = if let Some(a) = attach {
        Some(next.execute("joint.create",&json!({"name":format!("{} - rigid",p.name),"type":"rigid","a":{"occurrence":a,"point":position,"z":[0,0,1]},"b":{"occurrence":oid,"point":position,"z":[0,0,1]},"flip":true}))?["joint"].as_u64().ok_or_else(||bad("mks.library.body","attachment joint missing"))?)
    } else {
        None
    };
    let result = json!({"component":cid,"occurrence":oid,"body":body_name,"mass_kg":p.mass_kg,"attachment_port":p.port,"joint":joint});
    let mut entries = next.doc.metadata.get(KEY).and_then(|v| v.get("items")).and_then(Value::as_array).cloned().unwrap_or_default();
    if entries.len() >= 128 {
        return Err(bad("mks.library.body", "library body limit 128"));
    }
    entries.push(result.clone());
    next.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY, json!({"items":entries}))?;
    super::worksheet::commit(s, next)?;
    Ok(result)
}
fn joint(s: &mut Session, p: &Value) -> Result<Value> {
    let p: JointParams = serde_json::from_value(p.clone()).map_err(|e| bad("mks.library.joint", e.to_string()))?;
    if !["rigid", "revolute", "slider"].contains(&p.kind.as_str())
        || p.a == p.b
        || p.name.len() > 100
        || p.point_mm.iter().any(|v| !v.is_finite() || v.abs() > 1e7)
        || p.axis.iter().any(|v| !v.is_finite() || v.abs() > 1e6)
        || p.axis.iter().map(|v| v * v).sum::<f64>() < 1e-16
    {
        return Err(bad("mks.library.joint", "rigid/revolute/slider, distinct occurrences and finite point/axis required"));
    }
    let mut next = s.scratch();
    let result=next.execute("joint.create",&json!({"name":p.name,"type":p.kind,"a":{"occurrence":p.a,"point":p.point_mm,"z":p.axis},"b":{"occurrence":p.b,"point":p.point_mm,"z":p.axis},"flip":true}))?;
    super::worksheet::commit(s, next)?;
    Ok(json!({"joint":result["joint"]}))
}
