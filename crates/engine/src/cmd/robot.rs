//! A shared native robot project across calculation, CAD, loads, MKS and beam FEM.
use super::CommandSpec;
use crate::{Result, Session, params::bad};
use serde_json::{Value, json};
use solidcraft_doc::{FeatureKind, expr::Kind};
pub(super) const KEY: &str = "org.solidcraft.robot-workflow";
pub(super) const FEM_KEY: &str = "org.solidcraft.robot-fem";
const SOURCE: &str = include_str!("../calculations/robot.cpd");
pub static COMMANDS:&[CommandSpec]=&[
    CommandSpec::new("robot.prepare","Prepare Robot Across Domains",prepare).icon("component").params("Requires the native six-axis robot assembly; atomically adds local Calcpad references, assumed material cards, ports and two section variants. Existing calculation document is refused."),
    CommandSpec::new("robot.inspect","Inspect Robot Domains",inspect).icon("list").noundo(),
    CommandSpec::new("robot.motion.run","Run Six-Axis Robot Motion",motion).icon("graph").params("Uses ref_Hub_A1..6 as prescribed cosine increments, .5 s, 400 steps, 11 samples; keeps CAD pose unchanged, checks native travel limits"),
    CommandSpec::new("robot.loads.solve","Solve Locked Robot Load Path",loads).icon("graph").params("TCP force from ref_F_TCP_N, negative port Z; locked-actuator static resultant path to foundation, no gravity or actuator torque solution"),
    CommandSpec::new("robot.fem.solve","Solve Robot Beam FEM",super::robot_fem::solve).icon("graph").params("elements?:1..32 (default 8); locked actuators, native solid-cylinder arm idealization, TCP external force only. Linear Euler-Bernoulli bending FE plus axial/torsional compliance; no solid mesh or flexible MKS coupling"),
];
pub(super) fn number(s: &Session, name: &str, kind: Kind) -> Result<f64> {
    s.doc.eval(&format!("ref_{name}"), kind).map_err(Into::into)
}
pub(super) fn config(s: &Session) -> Result<Value> {
    s.doc.metadata.get(KEY).cloned().ok_or_else(|| bad("robot", "run robot.prepare first"))
}
fn prepare(s: &mut Session, _: &Value) -> Result<Value> {
    if s.doc.metadata.get(KEY).is_some() || s.doc.metadata.get("org.solidcraft.calcpad").is_some() || !s.doc.params.is_empty() {
        return Err(bad("robot.prepare", "requires the original robot without an existing calculation document; save your design first"));
    }
    let mut next = s.scratch();
    let listing = next.execute("component.list", &json!({}))?;
    let components = listing["components"].as_array().ok_or_else(|| bad("robot", "component list missing"))?;
    let mut occurrences = serde_json::Map::new();
    for name in ["base", "axis1", "upper", "forearm", "wrist4", "wrist5", "flange", "tool"] {
        let c = components.iter().find(|c| c["name"] == name).ok_or_else(|| bad("robot", format!("missing native component {name}")))?;
        let occ = c["occurrences"]
            .as_array()
            .filter(|v| v.len() == 1)
            .and_then(|v| v.first())
            .and_then(|v| v["id"].as_u64())
            .ok_or_else(|| bad("robot", "requires unique module occurrences"))?;
        occurrences.insert(name.into(), json!(occ));
    }
    let mut dimensions = Vec::new();
    for name in ["Oberarm", "Unterarm"] {
        let f = next
            .doc
            .features
            .iter()
            .find(|f| f.name == name && !f.suppressed && matches!(f.kind, FeatureKind::Cylinder { .. }))
            .ok_or_else(|| bad("robot", format!("missing native cylinder {name}")))?;
        let info = next.execute("timeline.inspect", &json!({"feature":f.id}))?;
        let p = info["inputs"]
            .as_array()
            .and_then(|v| v.iter().find(|i| i["label"] == "Radius"))
            .and_then(|i| i["param"].as_str())
            .ok_or_else(|| bad("robot", "native radius dimension missing"))?;
        dimensions.push((name, p.to_owned()));
    }
    // Alternative end effectors in the old visual scene are simultaneous solids,
    // not physical variants. Delete their authoring features and dependent transfers.
    let alternative = |name: &str| name.starts_with("Schweiss-") || name.starts_with("Nutzlast-");
    if next.doc.features.iter().any(
        |f| matches!(&f.kind,FeatureKind::BodyTransfer{bodies,..} if bodies.iter().any(|b|alternative(b))&&!bodies.iter().all(|b|alternative(b))),
    ) {
        return Err(bad("robot.prepare", "mixed alternative-tool body transfer requires manual separation"));
    }
    let unused: Vec<_> = next
        .doc
        .features
        .iter()
        .filter(|f| {
            alternative(&f.name) || matches!(&f.kind,FeatureKind::BodyTransfer{bodies,..} if !bodies.is_empty()&&bodies.iter().all(|b|alternative(b)))
        })
        .map(|f| f.id)
        .collect();
    if !unused.is_empty() {
        next.execute("timeline.delete", &json!({"features":unused}))?;
    }
    next.execute("worksheet.describe",&json!({"title":"Roboterarm – Berechnung und Domänen","notes":"Native CAD-Baugruppe. Kreisvollquerschnitte sind eine Strukturidealisierung. Dichten und E-Modul sind ausdrücklich angenommene Forschungswerte."}))?;
    next.execute("worksheet.calcpad.calculate", &json!({"source":SOURCE}))?;
    for name in ["R_Oberarm", "R_Unterarm", "E_MPa", "nu", "F_TCP_N", "Grenze_TCP", "Hub_A1", "Hub_A2", "Hub_A3", "Hub_A4", "Hub_A5", "Hub_A6"] {
        next.execute("worksheet.reference", &json!({"source":name}))?;
    }
    for (feature, dimension) in dimensions {
        next.execute("parameters.change", &json!({"name":dimension,"expression":format!("ref_R_{feature}")}))?;
    }
    next.execute("worksheet.variant.save", &json!({"name":"Roboter Basis"}))?;
    let light = SOURCE.replace("R_Oberarm = 77mm", "R_Oberarm = 65mm").replace("R_Unterarm = 65mm", "R_Unterarm = 55mm");
    next.execute("worksheet.calcpad.calculate", &json!({"source":light}))?;
    next.execute("worksheet.variant.save", &json!({"name":"Roboter Leichtbau"}))?;
    next.execute("worksheet.variant.select", &json!({"name":"Roboter Basis"}))?;
    next.execute("worksheet.apply", &json!({}))?;
    next.execute("material.create", &json!({"name":"Robot Assumed Aluminium","density_g_cm3":2.7}))?;
    next.execute("material.create", &json!({"name":"Robot Assumed Steel","density_g_cm3":7.85}))?;
    let names: Vec<_> = next.model.state().bodies.iter().map(|b| b.name.clone()).collect();
    let (steel, aluminium): (Vec<_>, Vec<_>) =
        names.into_iter().partition(|n| n.contains("Schraube") || n.contains("Lager") || n.contains("Flansch") || n.starts_with("Fundament"));
    for (bodies, material) in [(steel, "Robot Assumed Steel"), (aluminium, "Robot Assumed Aluminium")] {
        if !bodies.is_empty() {
            next.execute("material.assign", &json!({"bodies":bodies,"material":material}))?;
        }
    }
    let mut joints = Vec::new();
    for i in 1..=6 {
        let j = next.doc.assembly.joints.iter().find(|j| j.name == format!("J{i}")).ok_or_else(|| bad("robot", "six native joints required"))?;
        joints.push(j.id);
    }
    for (id, module, body, point) in [
        ("robot-ground", "base", "Fundament", json!([0, 0, 0])),
        ("robot-upper-root", "upper", "Oberarm", json!([0, 0, 405])),
        ("robot-upper-tip", "upper", "Oberarm", json!([300, 0, 975])),
        ("robot-forearm-root", "forearm", "Unterarm", json!([320, 0, 972])),
        ("robot-forearm-tip", "forearm", "Unterarm", json!([820, 0, 900])),
        ("robot-tcp", "tool", "Greifer-Gehaeuse", json!([1191, 0, 925])),
    ] {
        next.execute("mechanics.port", &json!({"id":id,"body":body,"occurrence":[occurrences[module]],"origin":point,"transfer":"frame_wrench"}))?;
    }
    for (parent, child) in [
        ("robot-ground", "robot-upper-root"),
        ("robot-upper-root", "robot-upper-tip"),
        ("robot-upper-tip", "robot-forearm-root"),
        ("robot-forearm-root", "robot-forearm-tip"),
        ("robot-forearm-tip", "robot-tcp"),
    ] {
        next.execute("mechanics.connect", &json!({"parent":parent,"child":child}))?;
    }
    next.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY,json!({"version":1,"occurrences":occurrences,"joints":joints,"assumptions":"Illustrative additive CAD solids; assumed densities and elastic data; original geometry retained. Only gripper active. Locked-actuator static port path is distinct from revolute MKS joints."}))?;
    super::worksheet::commit(s, next)?;
    inspect(s, &json!({}))
}
fn inspect(s: &mut Session, _: &Value) -> Result<Value> {
    let config = config(s)?;
    let result = s.doc.metadata.get(FEM_KEY).cloned().unwrap_or(Value::Null);
    let current = result.is_object() && super::mks_simulation::snapshot(s).is_ok_and(|v| v == result["source"]);
    Ok(
        json!({"configuration":config,"fem":result,"fem_current":current,"limits":"Beam FEM, no automatic solid mesh, dynamic stress recovery, controller or hardware execution"}),
    )
}
fn motion(s: &mut Session, _: &Value) -> Result<Value> {
    let c = config(s)?;
    let joints = c["joints"].as_array().filter(|v| v.len() == 6).ok_or_else(|| bad("robot", "six joints required"))?;
    let drivers: Vec<_> = joints.iter().enumerate().map(|(i, j)| json!({"joint":j,"displacement":format!("ref_Hub_A{}",i+1),"ramp_s":0.5})).collect();
    let load = number(s, "F_TCP_N", Kind::Unitless)?;
    s.execute(
        "mks.job.run",
        &json!({"duration_s":0.5,"steps":400,"samples":11,"drivers":drivers,"port_loads":[{"port":"robot-tcp","force_n":[0.,0.,-load]}]}),
    )
}
fn loads(s: &mut Session, _: &Value) -> Result<Value> {
    config(s)?;
    let force = number(s, "F_TCP_N", Kind::Unitless)?;
    s.execute("mechanics.case", &json!({"id":"robot-tcp-force","root":"robot-ground","loads":[{"port":"robot-tcp","force_n":[0.,0.,-force]}]}))?;
    s.execute("mechanics.evaluate", &json!({"case":"robot-tcp-force"}))
}
