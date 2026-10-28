//! Engineering graph drafts and independent, explicitly scoped reference studies.

use super::CommandSpec;
use crate::params::bad;
use crate::{Result, Session};
use serde_json::{Value, json};

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("studio.demo", "Load Engineering Prototype", demo)
        .icon("component")
        .params("Build the original robot in an empty design; no simulation model is created."),
    CommandSpec::new("studio.state", "Inspect Engineering Prototype", state).icon("list").noundo().doc_only(),
    CommandSpec::new("studio.view", "Engineering Workspace", view)
        .icon("ws_design")
        .noundo()
        .doc_only()
        .params("page: parameters|assembly|variants|system|fem|interfaces|loads|studies|cad"),
    CommandSpec::new("studio.tool", "Preview End Effector", tool).icon("component").noundo().params("tool: gripper|welder|payload; preview only"),
    CommandSpec::new("studio.select", "Select Engineering Module", select)
        .icon("select")
        .noundo()
        .params("module: base|shoulder|upper|forearm|wrist|flange|tool"),
    CommandSpec::new("studio.representation", "Preview Model Representation", representation)
        .icon("graph")
        .noundo()
        .doc_only()
        .params("representation: rigid|flexible|rom; no reduced model is generated"),
    CommandSpec::new("studio.variant", "Preview Engineering Variant", variant)
        .icon("library")
        .noundo()
        .doc_only()
        .params("variant: baseline|lightweight|reach; illustrative configuration only"),
    CommandSpec::new("studio.interface.add", "Create Mechanical Interface", interface_add)
        .icon("new").at("INTERFACES", "CREATE").noundo().doc_only()
        .params("owner?, label?; create an unbound session-only mechanical port draft"),
    CommandSpec::new("studio.interface.select", "Select Mechanical Interface", interface_select)
        .icon("select").noundo().doc_only().params("id: logical interface id"),
    CommandSpec::new("studio.interface.update", "Configure Mechanical Interface", interface_update)
        .icon("params").at("INTERFACES", "MODIFY").noundo().doc_only()
        .params("id?, support?: point|surface|surface_group, transfer?: point_force|frame_wrench|rigid_remote|distributed_remote|pressure, origin_mm?: [x,y,z], quaternion_wxyz?: [w,x,y,z]; preview only"),
    CommandSpec::new("studio.interface.capture", "Bind Interface CAD Selection", interface_capture)
        .icon("select").at("INTERFACES", "MODIFY").noundo()
        .params("id?; capture current CAD picks as provisional geometry locators, not a qualified FEM coupling"),
    CommandSpec::new("studio.connection.arm", "Connect Mechanical Ports", connection_arm)
        .icon("graph").at("MBS", "ASSEMBLE").noundo().doc_only()
        .params("id?: source interface, cancel?: bool; then select the target port; drafts only"),
    CommandSpec::new("studio.connection.add", "Create Mechanical Connection", connection_add)
        .icon("graph").noundo().doc_only().params("a, b, kind?: fixed|revolute|prismatic; no solver constraint is created"),
    CommandSpec::new("studio.connection.remove", "Remove Mechanical Connection", connection_remove)
        .icon("delete").noundo().doc_only().params("id: draft connection id"),
    CommandSpec::new("studio.beam.evaluate", "Evaluate Cantilever Reference", beam_evaluate)
        .icon("params").noundo().doc_only()
        .params("length_mm?, thickness_mm?, height_mm?, young_mpa?, density_kg_m3?, force_n?, deflection_limit_mm?; parameter_bindings?: {length field: CAD length parameter}; independent rectangular Euler-Bernoulli cantilever, not CAD/FEM"),
    CommandSpec::new("studio.job.create", "Prepare Reference Solver Job", job_create)
        .icon("graph").noundo().doc_only()
        .params("backend: scipy_beam_size|exudyn_beam_reference; same optional beam parameters as studio.beam.evaluate; returns a JSON job for an optional external process, does not launch software"),
    CommandSpec::new("studio.job.accept", "Import Reference Solver Result", job_accept)
        .icon("list").noundo().doc_only()
        .params("result: JSON from the external reference worker; requires the exact current prepared job; retains provenance and never changes CAD"),
    CommandSpec::new("studio.system.open", "Open Multibody Model", system_tool)
        .icon("graph").at("MBS", "CREATE").noundo().doc_only(),
    CommandSpec::new("studio.fem.open", "Open FEM and ROM Tools", fem_tool)
        .icon("graph").at("FEM / ROM", "CREATE").noundo().doc_only(),
    CommandSpec::new("studio.worksheet.open", "Open Engineering Parameters", worksheet_tool)
        .icon("params").at("WORKSHEET", "MODIFY").noundo().doc_only(),
    CommandSpec::new("studio.variants.open", "Open Engineering Variants", variants_tool)
        .icon("library").at("VARIANTS", "MODIFY").noundo().doc_only(),
    CommandSpec::new("studio.studies.open", "Open Reference Studies", studies_tool)
        .icon("params").at("STUDIES", "CREATE").noundo().doc_only(),
    CommandSpec::new("studio.loads.open", "Inspect Mechanical Load Paths", loads_tool)
        .icon("graph").at("LOAD PATHS", "INSPECT").noundo().doc_only(),
];

// Programmatic calls describe the tool; only ui.start opens interactive windows.
fn system_tool(_: &mut Session, _: &Value) -> Result<Value> {
    Ok(json!({"tool":"system","dialog_opened":false}))
}
fn fem_tool(_: &mut Session, _: &Value) -> Result<Value> {
    Ok(json!({"tool":"fem","dialog_opened":false}))
}
fn worksheet_tool(_: &mut Session, _: &Value) -> Result<Value> {
    Ok(json!({"tool":"parameters","dialog_opened":false}))
}
fn variants_tool(_: &mut Session, _: &Value) -> Result<Value> {
    Ok(json!({"tool":"variants","dialog_opened":false}))
}
fn studies_tool(_: &mut Session, _: &Value) -> Result<Value> {
    Ok(json!({"tool":"studies","dialog_opened":false}))
}
fn loads_tool(_: &mut Session, _: &Value) -> Result<Value> {
    Ok(json!({"tool":"loads","dialog_opened":false}))
}

fn state(s: &mut Session, _: &Value) -> Result<Value> {
    let current = s.studio.beam_result.as_ref().is_none_or(|report| crate::studio::reference_parameters_current(&s.doc, report));
    Ok(
        json!({"prototype":s.studio,"simulation_implemented":false,"reference_studies":["analytic_rectangular_cantilever"],"beam_parameter_snapshot_current":current,"persistence":"session_only"}),
    )
}

fn beam_inputs(s: &Session, p: &Value, cmd: &str) -> Result<crate::studio_beam::Beam> {
    if !p.is_object() {
        return Err(bad(cmd, "parameters must be an object"));
    }
    let mut resolved = p.clone();
    if let Some(bindings) = p.get("parameter_bindings") {
        let bindings = bindings.as_object().ok_or_else(|| bad(cmd, "parameter_bindings must be an object"))?;
        if bindings.len() > 4 {
            return Err(bad(cmd, "at most four length parameter bindings"));
        }
        let (values, errors) = s.doc.param_values();
        for (field, name) in bindings {
            if !["length_mm", "thickness_mm", "height_mm", "deflection_limit_mm"].contains(&field.as_str()) {
                return Err(bad(cmd, "only length fields can bind to CAD parameters; force/material units are explicit numeric inputs"));
            }
            if p.get(field).is_some() {
                return Err(bad(cmd, format!("{field}: choose a numeric value or a parameter binding")));
            }
            let name = name.as_str().ok_or_else(|| bad(cmd, "parameter binding must name a CAD parameter"))?;
            if let Some(error) = errors.get(name) {
                return Err(bad(cmd, error));
            }
            let number = values.get(name).ok_or_else(|| bad(cmd, format!("unknown CAD parameter {name}")))?;
            if number.kind() != Some(solidcraft_doc::expr::Kind::Length) {
                return Err(bad(cmd, format!("{name} must have a length unit")));
            }
            resolved[field] = json!(number.v);
        }
    }
    let mut beam = crate::studio_beam::Beam::default();
    for (key, target) in [
        ("length_mm", &mut beam.length_mm),
        ("thickness_mm", &mut beam.thickness_mm),
        ("height_mm", &mut beam.height_mm),
        ("young_mpa", &mut beam.young_mpa),
        ("density_kg_m3", &mut beam.density_kg_m3),
        ("force_n", &mut beam.force_n),
        ("deflection_limit_mm", &mut beam.deflection_limit_mm),
    ] {
        if let Some(input) = resolved.get(key) {
            *target = input.as_f64().filter(|v| v.is_finite()).ok_or_else(|| bad(cmd, format!("{key} must be a finite number")))?;
        }
    }
    beam.evaluate().map_err(|message| bad(cmd, message))?;
    Ok(beam)
}

fn beam_report(beam: crate::studio_beam::Beam) -> Result<Value> {
    let r = beam.evaluate().map_err(|message| bad("studio.beam.evaluate", message))?;
    let mut warnings = vec![
        "Independent uniform rectangular beam; not derived from the current CAD or port graph. No shear deformation, self-weight, contact, yielding or buckling.",
    ];
    if r.slenderness < 10. {
        warnings.push("Length/height < 10: check a shear-deformable beam or FEM before engineering use.");
    }
    if r.deflection_ratio > 0.02 {
        warnings.push("Deflection/length > 0.02: small-displacement assumptions need review.");
    }
    Ok(json!({
        "model":"euler_bernoulli_rectangular_cantilever/v1", "cad_linked":false,
        "inputs":{"length_mm":beam.length_mm,"thickness_mm":beam.thickness_mm,"height_mm":beam.height_mm,
            "young_mpa":beam.young_mpa,"density_kg_m3":beam.density_kg_m3,"force_n":beam.force_n,"deflection_limit_mm":beam.deflection_limit_mm},
        "results":{"mass_kg":r.mass_kg,"second_moment_mm4":r.second_moment_mm4,"tip_deflection_mm":r.tip_deflection_mm,
            "root_bending_stress_mpa":r.root_stress_mpa,"tip_stiffness_n_m":r.stiffness_n_m,"strain_energy_j":r.strain_energy_j,
            "deflection_constraint_satisfied":r.feasible,"length_height_ratio":r.slenderness},
        "frame":"beam x along span; height along z; tip force along signed z; wrench order Fx,Fy,Fz,Mx,My,Mz",
        "root_reaction_wrench_n_nm":[0.,0.,-beam.force_n,0.,beam.force_n*beam.length_mm*0.001,0.],
        "qualification":"analytic_reference_only", "warnings":warnings,
        "derivation":{"second_moment":"t*h^3/12","deflection":"F*L^3/(3*E*I)","stress":"abs(F)*L*h/(2*I)","mass":"rho*L*t*h*1e-9"}
    }))
}

fn beam_evaluate(s: &mut Session, p: &Value) -> Result<Value> {
    let mut report = beam_report(beam_inputs(s, p, "studio.beam.evaluate")?)?;
    report["parameter_bindings"] = p.get("parameter_bindings").cloned().unwrap_or(json!({}));
    s.studio.beam_result = Some(report.clone());
    s.studio.backend_job = None;
    s.studio.backend_result = None;
    s.studio.notice = "Analytischer Kragträger berechnet. Unabhängiger Referenzfall, keine CAD-/FEM-Auswertung.".into();
    Ok(report)
}

fn job_create(s: &mut Session, p: &Value) -> Result<Value> {
    let backend = value(p, "backend", &["scipy_beam_size", "exudyn_beam_reference"], "studio.job.create")?;
    let mut report = beam_report(beam_inputs(s, p, "studio.job.create")?)?;
    report["parameter_bindings"] = p.get("parameter_bindings").cloned().unwrap_or(json!({}));
    let job = json!({
        "schema":"solidcraft.reference-job/1", "backend":backend, "reference":report,
        "context":{"design_name":s.doc.name,"variant":s.studio.variant,"tool":s.studio.tool,"cad_linked":false},
        "settings":if backend=="scipy_beam_size" {
            json!({"thickness_bounds_mm":[2.,20.],"height_bounds_mm":[10.,200.],"max_iterations":100})
        } else {
            json!({"tip_mass_kg":5.,"damping_ratio":0.02,"duration_s":0.2,"steps":4000})
        },
        "status":"prepared", "launch_policy":"external_optional_worker",
        "scope":"Independent reference model; no robot assembly, CAD face coupling or FEM-derived ROM."
    });
    s.studio.backend_job = Some(job.clone());
    s.studio.backend_result = None;
    s.studio.notice = "Solverauftrag als JSON vorbereitet. Noch kein externer Prozess gestartet.".into();
    Ok(job)
}

fn job_accept(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "studio.job.accept";
    let result = p.get("result").ok_or_else(|| bad(cmd, "result required"))?;
    let job = s.studio.backend_job.as_ref().ok_or_else(|| bad(cmd, "prepare a reference job first"))?;
    if !job.get("reference").is_some_and(|r| crate::studio::reference_parameters_current(&s.doc, r)) {
        return Err(bad(cmd, "bound CAD parameters changed; prepare a new job before importing results"));
    }
    if result.get("schema").and_then(Value::as_str) != Some("solidcraft.reference-result/1") || result.get("job") != Some(job) {
        return Err(bad(cmd, "result schema or job snapshot does not match the currently prepared job"));
    }
    let status = result.get("status").and_then(Value::as_str).ok_or_else(|| bad(cmd, "result status required"))?;
    if !["success", "failed", "infeasible"].contains(&status) {
        return Err(bad(cmd, "unsupported result status"));
    }
    let summary = result.get("summary").and_then(Value::as_object).ok_or_else(|| bad(cmd, "numeric result summary required"))?;
    if summary.is_empty() || summary.len() > 32 || !summary.values().all(|v| v.as_f64().is_some_and(f64::is_finite)) {
        return Err(bad(cmd, "summary must contain 1..32 finite numbers"));
    }
    if !result.get("backend_version").and_then(Value::as_str).is_some_and(|v| !v.is_empty() && v.len() <= 128) {
        return Err(bad(cmd, "backend version required"));
    }
    let provenance = result.get("provenance").and_then(Value::as_object).ok_or_else(|| bad(cmd, "provenance object required"))?;
    for field in ["job_sha256", "worker_sha256"] {
        if !provenance.get(field).and_then(Value::as_str).is_some_and(|v| v.len() == 64 && v.bytes().all(|b| b.is_ascii_hexdigit())) {
            return Err(bad(cmd, format!("{field}: SHA-256 text required")));
        }
    }
    let bytes = serde_json::to_vec(result).map_err(|error| bad(cmd, error.to_string()))?;
    if bytes.len() > 2_000_000 {
        return Err(bad(cmd, "reference result exceeds 2 MB"));
    }
    s.studio.backend_result = Some(result.clone());
    s.studio.notice = format!("Externes Referenzergebnis importiert: {status}. Keine CAD-Änderung ausgeführt.");
    Ok(json!({"status":status,"cad_changed":false}))
}

fn demo(s: &mut Session, _: &Value) -> Result<Value> {
    if !s.doc.features.is_empty() || !s.doc.params.is_empty() || !s.doc.components.is_empty() {
        return Err(bad("studio.demo", "open an empty design first; existing work is never replaced"));
    }
    s.run_script(&crate::studio::robot_script())?;
    s.doc_mut().name = "10derCraft · Modular Robot / Geometrievorschau".into();
    s.studio = crate::studio::State {
        enabled: true,
        interfaces: crate::studio::demo_interfaces(),
        connections: crate::studio::demo_connections(),
        selected_interface: "upper.DH_A".into(),
        module_frames: crate::studio::demo_module_frames(),
        ..Default::default()
    };
    crate::studio::apply_tool_visibility(s);
    state(s, &json!({}))
}

fn value<'a>(p: &'a Value, field: &str, choices: &[&str], cmd: &str) -> Result<&'a str> {
    p.get(field)
        .and_then(Value::as_str)
        .filter(|v| choices.contains(v))
        .ok_or_else(|| bad(cmd, format!("`{field}` must be one of {}", choices.join(", "))))
}

fn view(s: &mut Session, p: &Value) -> Result<Value> {
    let page = value(p, "page", &["parameters", "assembly", "variants", "system", "fem", "interfaces", "loads", "studies", "cad"], "studio.view")?;
    s.studio.page = page.into();
    s.studio.enabled = true;
    state(s, p)
}

fn tool(s: &mut Session, p: &Value) -> Result<Value> {
    let tool = value(p, "tool", &["gripper", "welder", "payload"], "studio.tool")?;
    s.studio.tool = tool.into();
    for interface in s.studio.interfaces.iter_mut().filter(|i| i.owner == "tool") {
        interface.cad_selection.clear();
    }
    s.studio.selected = "tool".into();
    s.studio.notice = "Werkzeugvorschau gewechselt. Montage, Lasten und Kompatibilität sind noch nicht geprüft.".into();
    s.selection.clear();
    crate::studio::apply_tool_visibility(s);
    state(s, p)
}

fn select(s: &mut Session, p: &Value) -> Result<Value> {
    let module = p
        .get("module")
        .and_then(Value::as_str)
        .filter(|id| {
            id.len() <= 128
                && (["base", "axis1", "shoulder", "upper", "forearm", "wrist4", "wrist5", "wrist", "flange", "tool"].contains(id)
                    || s.studio.interfaces.iter().any(|i| i.owner == *id))
        })
        .ok_or_else(|| bad("studio.select", "unknown module"))?;
    s.studio.selected = module.into();
    if let Some(port) = s.studio.interfaces.iter().find(|i| i.owner == module) {
        s.studio.selected_interface = port.id.clone();
    }
    s.selection = s
        .model
        .state()
        .bodies
        .iter()
        .filter(|b| crate::studio::belongs_to_module(module, &s.studio.tool, &b.name))
        .map(|b| crate::Sel::Body { name: b.name.clone() })
        .collect();
    state(s, p)
}

fn representation(s: &mut Session, p: &Value) -> Result<Value> {
    let rep = value(p, "representation", &["rigid", "flexible", "rom"], "studio.representation")?;
    let modes = match p.get("mode_count") {
        Some(v) => v
            .as_u64()
            .filter(|n| [6, 20].contains(n))
            .ok_or_else(|| bad("studio.representation", "mode_count must be 6 or 20 for this demonstration"))? as usize,
        None => s.studio.mode_count,
    };
    s.studio.representation = rep.into();
    s.studio.mode_count = modes;
    s.studio.notice = "Geplante Modellrepräsentation ausgewählt. FEM und ROM-Erzeugung sind noch nicht implementiert.".into();
    state(s, p)
}

fn variant(s: &mut Session, p: &Value) -> Result<Value> {
    let variant = value(p, "variant", &["baseline", "lightweight", "reach"], "studio.variant")?;
    s.studio.variant = variant.into();
    s.studio.notice = "Variantenentwurf ausgewählt. Parameterableitung und persistente Varianten sind noch nicht implementiert.".into();
    state(s, p)
}

fn interface_id<'a>(s: &'a Session, p: &'a Value) -> &'a str {
    p.get("id").and_then(Value::as_str).unwrap_or(&s.studio.selected_interface)
}

fn interface_index(s: &Session, p: &Value, cmd: &str) -> Result<usize> {
    let id = interface_id(s, p);
    s.studio.interfaces.iter().position(|i| i.id == id).ok_or_else(|| bad(cmd, "select an existing mechanical interface first"))
}

fn interface_add(s: &mut Session, p: &Value) -> Result<Value> {
    if s.studio.interfaces.len() >= 256 {
        return Err(bad("studio.interface.add", "at most 256 draft ports"));
    }
    let body = s.selection.first().and_then(|pick| match pick {
        crate::Sel::Body { name } => Some(name.as_str()),
        crate::Sel::Face { body, .. } | crate::Sel::Edge { body, .. } => Some(body.as_str()),
        _ => None,
    });
    let inferred = body
        .map(|body| {
            s.studio
                .module_frames
                .keys()
                .find(|owner| crate::studio::belongs_to_module(owner, &s.studio.tool, body))
                .cloned()
                .unwrap_or_else(|| body.into())
        })
        .unwrap_or_else(|| if s.studio.module_frames.is_empty() { "custom".into() } else { s.studio.selected.clone() });
    let owner = p.get("owner").and_then(Value::as_str).unwrap_or(&inferred);
    if owner.is_empty() || owner.len() > 128 {
        return Err(bad("studio.interface.add", "owner must contain 1..128 bytes"));
    }
    let label = p.get("label").and_then(Value::as_str).unwrap_or("Anschluss");
    if label.is_empty() || label.len() > 128 {
        return Err(bad("studio.interface.add", "label must contain 1..128 bytes"));
    }
    let mut number = 1;
    while s.studio.interfaces.iter().any(|i| i.id == format!("{owner}.P{number}")) {
        number += 1;
    }
    let id = format!("{owner}.P{number}");
    s.studio.module_frames.entry(owner.into()).or_insert(crate::studio::InterfaceFrame { origin_mm: [0.; 3], quaternion_wxyz: [1., 0., 0., 0.] });
    s.studio.interfaces.push(crate::studio::MechanicalInterface {
        id: id.clone(),
        owner: owner.into(),
        label: label.into(),
        origin_mm: [0.; 3],
        quaternion_wxyz: [1., 0., 0., 0.],
        support: "point".into(),
        transfer: "frame_wrench".into(),
        cad_selection: Vec::new(),
    });
    s.studio.selected_interface = id;
    s.studio.notice = "Anschlussentwurf angelegt. Referenzrahmen und CAD-Anbindung sind noch zu definieren.".into();
    state(s, p)
}

fn interface_select(s: &mut Session, p: &Value) -> Result<Value> {
    let index = interface_index(s, p, "studio.interface.select")?;
    let target = s.studio.interfaces.get(index).map(|i| i.id.clone()).ok_or_else(|| bad("studio.interface.select", "missing port"))?;
    if let Some(source) = s.studio.pending_connection.clone()
        && source != target
    {
        add_connection(s, &source, &target, "fixed")?;
        s.studio.pending_connection = None;
    }
    s.studio.selected_interface = target;
    if let Some(port) = s.studio.interfaces.get(index) {
        s.studio.selected = port.owner.clone();
    }
    state(s, p)
}

fn numbers<const N: usize>(p: &Value, key: &str, current: [f64; N]) -> Result<[f64; N]> {
    let Some(raw) = p.get(key) else { return Ok(current) };
    let values = raw.as_array().filter(|v| v.len() == N).ok_or_else(|| bad("studio.interface.update", format!("{key} needs {N} numbers")))?;
    let mut result = current;
    for (slot, value) in result.iter_mut().zip(values) {
        *slot = value
            .as_f64()
            .filter(|n| n.is_finite() && n.abs() <= 1e9)
            .ok_or_else(|| bad("studio.interface.update", "frame values must be finite and bounded"))?;
    }
    Ok(result)
}

fn interface_update(s: &mut Session, p: &Value) -> Result<Value> {
    let index = interface_index(s, p, "studio.interface.update")?;
    let mut next = s.studio.interfaces.get(index).cloned().ok_or_else(|| bad("studio.interface.update", "missing port"))?;
    if let Some(raw) = p.get("label") {
        next.label = raw
            .as_str()
            .filter(|v| !v.is_empty() && v.len() <= 128)
            .ok_or_else(|| bad("studio.interface.update", "label needs 1..128 bytes"))?
            .into();
    }
    if p.get("support").is_some() {
        next.support = value(p, "support", &["point", "surface", "surface_group"], "studio.interface.update")?.into();
        next.transfer = if next.support == "point" { "frame_wrench" } else { "distributed_remote" }.into();
        next.cad_selection.clear();
    }
    if p.get("transfer").is_some() {
        next.transfer =
            value(p, "transfer", &["point_force", "frame_wrench", "rigid_remote", "distributed_remote", "pressure"], "studio.interface.update")?
                .into();
    }
    let valid = match next.transfer.as_str() {
        "point_force" | "frame_wrench" => next.support == "point",
        _ => next.support != "point",
    };
    if !valid {
        return Err(bad("studio.interface.update", "point loads/frames and area coupling require matching support kinds"));
    }
    if ["point_force", "pressure"].contains(&next.transfer.as_str()) && s.studio.connections.iter().any(|c| c.a == next.id || c.b == next.id) {
        return Err(bad(
            "studio.interface.update",
            "a connected joint port needs a full mechanical frame; create a separate point-force/pressure load port or remove its connections first",
        ));
    }
    next.origin_mm = numbers(p, "origin_mm", next.origin_mm)?;
    next.quaternion_wxyz = numbers(p, "quaternion_wxyz", next.quaternion_wxyz)?;
    let norm = next.quaternion_wxyz.iter().map(|v| v * v).sum::<f64>();
    if (norm - 1.).abs() > 1e-6 {
        return Err(bad("studio.interface.update", "orientation must be a unit quaternion, w,x,y,z"));
    }
    if let Some(slot) = s.studio.interfaces.get_mut(index) {
        *slot = next;
    }
    s.studio.notice = "Schnittstellenentwurf geändert. FEM-Kopplung und physikalische Gültigkeit sind noch nicht berechnet.".into();
    state(s, p)
}

fn interface_capture(s: &mut Session, p: &Value) -> Result<Value> {
    let index = interface_index(s, p, "studio.interface.capture")?;
    let interface = s.studio.interfaces.get(index).ok_or_else(|| bad("studio.interface.capture", "missing port"))?;
    let selected = &s.selection;
    let count = selected.len();
    let valid = match interface.support.as_str() {
        "point" => count == 1 && selected.iter().all(|pick| matches!(pick, crate::Sel::Face { .. } | crate::Sel::Edge { .. })),
        "surface" => count == 1 && selected.iter().all(|pick| matches!(pick, crate::Sel::Face { .. })),
        _ => (2..=128).contains(&count) && selected.iter().all(|pick| matches!(pick, crate::Sel::Face { .. })),
    };
    if !valid {
        return Err(bad(
            "studio.interface.capture",
            "select one face/edge pick for a point, one face for a surface, or 2..128 faces for a surface group",
        ));
    }
    let body = selected.first().and_then(|pick| match pick {
        crate::Sel::Face { body, .. } | crate::Sel::Edge { body, .. } => Some(body),
        _ => None,
    });
    if selected.iter().any(|pick| match pick {
        crate::Sel::Face { body: other, .. } | crate::Sel::Edge { body: other, .. } => Some(other) != body,
        _ => true,
    }) {
        return Err(bad("studio.interface.capture", "this prototype captures geometry from one CAD body per interface"));
    }
    if let Some(slot) = s.studio.interfaces.get_mut(index) {
        slot.cad_selection = selected.clone();
    }
    s.studio.notice =
        "CAD-Auswahl erfasst. Portursprung bleibt unabhängig; Topologiestabilität, Netzabbildung und Lastverteilung sind ungeprüft.".into();
    state(s, p)
}

fn connection_arm(s: &mut Session, p: &Value) -> Result<Value> {
    if p.get("cancel").and_then(Value::as_bool) == Some(true) {
        s.studio.pending_connection = None;
    } else {
        let _ = interface_index(s, p, "studio.connection.arm")?;
        s.studio.pending_connection = Some(interface_id(s, p).to_string());
        s.studio.notice = "Zielport im Blockmodell wählen. Es entsteht ein Entwurf einer festen Verbindung; kein Solver wird gestartet.".into();
    }
    state(s, p)
}

fn add_connection(s: &mut Session, a: &str, b: &str, kind: &str) -> Result<()> {
    if a == b || !s.studio.interfaces.iter().any(|i| i.id == a) || !s.studio.interfaces.iter().any(|i| i.id == b) {
        return Err(bad("studio.connection.add", "two distinct existing interface ids are required"));
    }
    if s.studio.interfaces.iter().filter(|i| i.id == a || i.id == b).any(|i| ["point_force", "pressure"].contains(&i.transfer.as_str())) {
        return Err(bad("studio.connection.add", "point-force and pressure loads are not full mechanical joint ports"));
    }
    if s.studio.connections.len() >= 512 {
        return Err(bad("studio.connection.add", "at most 512 connection drafts"));
    }
    if s.studio.connections.iter().any(|c| (c.a == a && c.b == b) || (c.a == b && c.b == a)) {
        return Err(bad("studio.connection.add", "connection already exists"));
    }
    let mut number = 1;
    while s.studio.connections.iter().any(|c| c.id == format!("C{number}")) {
        number += 1;
    }
    s.studio.connections.push(crate::studio::MechanicalConnection {
        id: format!("C{number}"),
        a: a.into(),
        b: b.into(),
        kind: kind.into(),
        axis_in_a: if kind == "fixed" { None } else { Some([0., 0., 1.]) },
    });
    s.studio.notice = "Mechanische Verbindung als Entwurf angelegt. Freiheitsgrade und physikalische Kompatibilität sind noch ungeprüft.".into();
    Ok(())
}

fn connection_add(s: &mut Session, p: &Value) -> Result<Value> {
    let a = p.get("a").and_then(Value::as_str).ok_or_else(|| bad("studio.connection.add", "a required"))?;
    let b = p.get("b").and_then(Value::as_str).ok_or_else(|| bad("studio.connection.add", "b required"))?;
    let kind = if p.get("kind").is_some() { value(p, "kind", &["fixed", "revolute", "prismatic"], "studio.connection.add")? } else { "fixed" };
    add_connection(s, a, b, kind)?;
    state(s, p)
}

fn connection_remove(s: &mut Session, p: &Value) -> Result<Value> {
    let id = p.get("id").and_then(Value::as_str).ok_or_else(|| bad("studio.connection.remove", "id required"))?;
    if !s.studio.connections.iter().any(|c| c.id == id) {
        return Err(bad("studio.connection.remove", "unknown connection"));
    }
    s.studio.connections.retain(|c| c.id != id);
    state(s, p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beam_reference_is_atomic_and_rejects_results_from_other_jobs() {
        let mut s = Session::default();
        let report = s.execute("studio.beam.evaluate", &json!({})).unwrap();
        assert!((report["results"]["mass_kg"].as_f64().unwrap() - 7.85).abs() < 1e-12);
        assert!(s.execute("studio.beam.evaluate", &json!({"height_mm":0})).is_err());
        assert_eq!(s.studio.beam_result.as_ref(), Some(&report));
        let job = s.execute("studio.job.create", &json!({"backend":"scipy_beam_size"})).unwrap();
        let mut result = json!({"schema":"solidcraft.reference-result/1","job":job,"status":"success",
            "summary":{"mass_kg":7.47},"backend_version":"fixture",
            "provenance":{"job_sha256":"0".repeat(64),"worker_sha256":"1".repeat(64)}});
        s.execute("studio.job.accept", &json!({"result":result})).unwrap();
        result["job"]["reference"]["inputs"]["height_mm"] = json!(200.);
        let previous = s.studio.backend_result.clone();
        assert!(s.execute("studio.job.accept", &json!({"result":result})).is_err());
        assert_eq!(s.studio.backend_result, previous);
        s.execute("studio.beam.evaluate", &json!({"height_mm":90})).unwrap();
        assert!(s.studio.backend_job.is_none());
        assert!(s.studio.backend_result.is_none());
        assert!(!s.is_dirty());
    }

    #[test]
    fn cad_length_bindings_preserve_units_and_invalidate_old_reference_jobs() {
        let mut s = Session::default();
        s.execute("parameters.add", &json!({"name":"span","expression":"1 m","unit":"mm"})).unwrap();
        let request = json!({"parameter_bindings":{"length_mm":"span"}});
        let report = s.execute("studio.beam.evaluate", &request).unwrap();
        assert_eq!(report["inputs"]["length_mm"], 1000.);
        assert!(crate::studio::reference_parameters_current(&s.doc, &report));
        assert!(s.execute("studio.beam.evaluate", &json!({"length_mm":20,"parameter_bindings":{"length_mm":"span"}})).is_err());
        let mut job_request = request.clone();
        job_request["backend"] = json!("scipy_beam_size");
        let job = s.execute("studio.job.create", &job_request).unwrap();
        s.execute("parameters.change", &json!({"name":"span","expression":"1100 mm"})).unwrap();
        assert!(!crate::studio::reference_parameters_current(&s.doc, &report));
        let stale = json!({"schema":"solidcraft.reference-result/1","job":job,"status":"success",
            "summary":{"mass_kg":7.47},"backend_version":"fixture",
            "provenance":{"job_sha256":"0".repeat(64),"worker_sha256":"1".repeat(64)}});
        assert!(s.execute("studio.job.accept", &json!({"result":stale})).is_err());
        s.execute("parameters.add", &json!({"name":"joint_angle","expression":"30 deg","unit":"deg"})).unwrap();
        assert!(s.execute("studio.beam.evaluate", &json!({"parameter_bindings":{"length_mm":"joint_angle"}})).is_err());
    }

    #[test]
    fn invalid_choices_do_not_change_preview_and_no_results_are_claimed() {
        let mut s = Session::default();
        assert!(s.execute("studio.tool", &json!({"tool":"unknown"})).is_err());
        assert_eq!(s.studio.tool, "gripper");
        s.execute("studio.view", &json!({"page":"system"})).unwrap();
        s.execute("studio.tool", &json!({"tool":"welder"})).unwrap();
        let reply = s.execute("studio.state", &json!({})).unwrap();
        assert_eq!(reply["simulation_implemented"], false);
        assert_eq!(reply["prototype"]["tool"], "welder");
        assert_eq!(reply["persistence"], "session_only");
        assert!(!s.is_dirty());
    }

    #[test]
    fn demo_builds_cad_geometry_switches_tools_and_protects_existing_designs() {
        let mut s = Session::default();
        s.execute("studio.demo", &json!({})).unwrap();
        assert!(s.studio.enabled);
        assert!(s.model.state().bodies.iter().any(|b| b.name == "Werkzeugflansch"));
        assert_eq!(s.doc.components.len(), 8);
        assert_eq!(s.doc.assembly.joints.len(), 7);
        let joints = s.execute("joint.list", &json!({})).unwrap();
        assert_eq!(joints["floating"], json!([]));
        assert_eq!(joints["conflicts"], json!([]));
        for axis in 1..=6 {
            let moved = s.execute("joint.drive", &json!({"joint":format!("J{axis}"),"value":12})).unwrap();
            assert_eq!(moved["moved"], true);
            s.execute("joint.drive", &json!({"joint":format!("J{axis}"),"value":0})).unwrap();
        }
        assert!(!s.visibility.hidden_bodies.iter().any(|n| n == "Greifer-Gehaeuse"));
        assert!(s.visibility.hidden_bodies.iter().any(|n| n == "Nutzlast-Block"));
        s.execute("studio.tool", &json!({"tool":"payload"})).unwrap();
        assert!(s.visibility.hidden_bodies.iter().any(|n| n == "Greifer-Gehaeuse"));
        assert!(!s.visibility.hidden_bodies.iter().any(|n| n == "Nutzlast-Block"));
        let count = s.doc.features.len();
        assert!(s.execute("studio.demo", &json!({})).is_err());
        assert_eq!(s.doc.features.len(), count);
    }

    #[test]
    fn interface_contract_survives_representation_changes_and_rejects_wrong_load_types() {
        let mut s = Session::default();
        s.studio.interfaces = crate::studio::demo_interfaces();
        s.studio.connections = crate::studio::demo_connections();
        s.studio.module_frames = crate::studio::demo_module_frames();
        s.studio.selected_interface = "upper.DH_A".into();
        let original = serde_json::to_value(&s.studio.interfaces).unwrap();
        s.execute("studio.representation", &json!({"representation":"flexible","mode_count":20})).unwrap();
        assert_eq!(serde_json::to_value(&s.studio.interfaces).unwrap(), original);
        assert!(s.execute("studio.interface.update", &json!({"support":"point","transfer":"pressure"})).is_err());
        assert_eq!(serde_json::to_value(&s.studio.interfaces).unwrap(), original);
        assert!(s.execute("studio.interface.update", &json!({"support":"point","transfer":"point_force"})).is_err());
        assert!(s.execute("studio.interface.update", &json!({"quaternion_wxyz":[0,0,0,0]})).is_err());
        assert_eq!(serde_json::to_value(&s.studio.interfaces).unwrap(), original);
        for edge in &s.studio.connections {
            assert_eq!(s.studio.interface_origin_model_mm(&edge.a), s.studio.interface_origin_model_mm(&edge.b));
        }
        s.execute("studio.interface.add", &json!({"owner":"upper","label":"Lastpunkt"})).unwrap();
        s.execute("studio.interface.update", &json!({"transfer":"point_force"})).unwrap();
        let id = s.studio.selected_interface.clone();
        assert!(s.execute("studio.connection.add", &json!({"a":id,"b":"upper.DH_A"})).is_err());
        assert!(!s.is_dirty());
    }
}
