//! Calculation-driven planetary assembly; native analytic gears and native joints.
use super::CommandSpec;
use crate::params::bad;
use crate::{Result, Session};
use serde_json::{Value, json};
use solidcraft_doc::{IDENTITY, joints::Snap};
use solidcraft_geom::Vec3;
use std::f64::consts::{PI, TAU};
pub const KEY: &str = "org.solidcraft.planetary-stage";
const SOURCE: &str = include_str!("../calculations/kue-stage.cpd");
const REFS: &[&str] = &[
    "Variante",
    "mode_1",
    "mode_2",
    "z_s",
    "z_p",
    "z_r",
    "modul",
    "b",
    "a",
    "bore_s",
    "bore_p",
    "pin_r",
    "ring_outer",
    "Platte",
    "hub_r",
    "Spiel_axial",
    "Zahnflankenspiel",
    "alpha_grad",
    "R_1",
    "S_Zahn",
    "S_Bolzen",
    "S_Durchbiegung",
    "S_Neigung",
    "Wandreserve",
    "Drehzahlabweichung",
    "z_s2",
    "z_p2",
    "z_r2",
    "n_eingang_rpm",
    "T_eingang_Nm",
];
pub static COMMANDS:&[CommandSpec]=&[
    CommandSpec::new("planetary.create","Create Calculated Planetary Stage",create).icon("component")
        .params("Empty design required; installs local Calcpad KUE variants 1..4, constructs only stage 1 with analytic involutes, native components and moving joints"),
    CommandSpec::new("planetary.inspect","Inspect Planetary Design",inspect).icon("params").noundo().doc_only(),
    CommandSpec::new("planetary.inputs","Edit Planetary Inputs",inputs).icon("params").doc_only().params("Read editable source expressions, or supply name and expression to edit one of R_Ziel, T_in, z_s, modul, alpha_grad, breitenfaktor in the embedded Calcpad sheet"),
];
fn value(s: &Session, name: &str) -> Result<f64> {
    let (vals, _) = s.doc.param_values();
    vals.get(&format!("ref_{name}")).map(|v| v.v).filter(|v| v.is_finite()).ok_or_else(|| bad("planetary", format!("missing reference: {name}")))
}
pub fn validation(s: &Session) -> Result<Value> {
    let get = |n| value(s, n);
    let (zs, zp, zr, m, alpha) = (get("z_s")?, get("z_p")?, get("z_r")?, get("modul")?, get("alpha_grad")?.to_radians());
    for z in [zs, zp, zr, get("z_s2")?, get("z_p2")?, get("z_r2")?] {
        if !(18. ..=240.).contains(&z) || (z - z.round()).abs() > 1e-8 {
            return Err(bad("planetary", "all tooth counts must be integers in 18..240"));
        }
    }
    if (zr - zs - 2. * zp).abs() > 1e-8 || (((zs + zr) / 3.).round() - (zs + zr) / 3.).abs() > 1e-8 {
        return Err(bad("planetary", "tooth counts violate coaxiality or the three-planet assembly condition"));
    }
    let (rs, rp, rr) = (m * zs / 2., m * zp / 2., m * zr / 2.);
    let a = rs + rp;
    let (bs, bp, br) = (rs * alpha.cos(), rp * alpha.cos(), rr * alpha.cos());
    let (ss, sp, sr) = (((rs + m).powi(2) - bs * bs).sqrt(), ((rp + m).powi(2) - bp * bp).sqrt(), ((rr - m).powi(2) - br * br).sqrt());
    let eps_external = (ss + sp - a * alpha.sin()) / (PI * m * alpha.cos());
    let eps_internal = (sp - sr + a * alpha.sin()) / (PI * m * alpha.cos());
    let min_s = (a * alpha.sin() - sp) / bs;
    let min_p_external = (a * alpha.sin() - ss) / bp;
    let min_p_internal = (sr - a * alpha.sin()) / bp;
    let start = |r: f64| {
        let rf = r - 1.25 * m;
        let rb = r * alpha.cos();
        ((rf / rb).powi(2) - 1.).max(0.).sqrt()
    };
    if !(eps_external > 1.05
        && eps_internal > 1.05
        && min_s >= start(rs) - 1e-8
        && min_p_external >= start(rp) - 1e-8
        && min_p_internal >= start(rp) - 1e-8)
    {
        return Err(bad(
            "planetary",
            "working contact reaches a non-involute root or has insufficient contact ratio; change tooth counts/pressure angle",
        ));
    }
    let gap = 3f64.sqrt() * a - 2. * (rp + m);
    if !(gap > 0. && (get("a")? - a).abs() < 1e-7 * m && m > 0.) {
        return Err(bad("planetary", "planet spacing or center-distance reference is invalid"));
    }
    for n in ["S_Zahn", "S_Bolzen", "S_Durchbiegung", "S_Neigung"] {
        if get(n)? < 1. - 1e-8 {
            return Err(bad("planetary", format!("design criterion {n} is below 1; increase dimensions or revise inputs")));
        }
    }
    if get("Wandreserve")? < 2. {
        return Err(bad("planetary", "planet root-to-bearing wall must be at least 2 modules"));
    }
    if get("Drehzahlabweichung")?.abs() > 0.05 {
        return Err(bad("planetary", "full train generator speed differs by more than 5%; revise stage ratio/tooth counts"));
    }
    let mode = get("mode_1")?;
    if mode != 1. && mode != 2. {
        return Err(bad("planetary", "mode_1 must be 1 (ring fixed) or 2 (carrier fixed)"));
    }
    let ratio = if mode == 1. { 1. + zr / zs } else { -zr / zs };
    Ok(json!({"teeth":{"sun":zs,"planet":zp,"ring":zr},"module_mm":m,"width_mm":get("b")?,"center_mm":a,
        "mode":mode,"fixed":if mode==1. {"Ring"} else {"Carrier"},"input":if mode==1. {"Carrier"} else {"Ring"},
        "sun_per_input":ratio,"planet_relative_per_input":if mode==1. {-zr/zp} else {zr/zp},
        "contact_ratio_external":eps_external,"contact_ratio_internal":eps_internal,"planet_tip_gap_mm":gap,"input_rpm":get("n_eingang_rpm")?,"input_torque_Nm":get("T_eingang_Nm")?,
        "minimum_working_involute_parameters":[min_s,min_p_external,min_p_internal],"assembly_index":(zs+zr)/3.}))
}

/// Update grounding, anchors and Willis links together with changed CAD parameters.
pub fn sync(s: &mut Session) -> Result<()> {
    if s.doc.metadata.get(KEY).is_none() {
        return Ok(());
    }
    let report = validation(s)?;
    let mode = report["mode"].as_f64().unwrap_or(1.);
    let a = value(s, "a")?;
    let doc = s.doc_mut();
    let occurrence = |name: &str| {
        doc.occurrences.iter().find(|o| o.name == format!("{name}:1")).map(|o| o.id).ok_or_else(|| bad("planetary", "assembly occurrence missing"))
    };
    let ring = occurrence("Ring")?;
    let carrier = occurrence("Carrier")?;
    let sun = occurrence("Sun")?;
    let owned = [ring, carrier, sun, occurrence("Planet1")?, occurrence("Planet2")?, occurrence("Planet3")?];
    for o in &mut doc.occurrences {
        if owned.contains(&o.id) {
            o.transform = IDENTITY;
            o.grounded = o.id == if mode == 1. { ring } else { carrier };
        }
    }
    let fixed = if mode == 1. { ring } else { carrier };
    let input = if mode == 1. { carrier } else { ring };
    for j in &mut doc.assembly.joints {
        if !matches!(j.name.as_str(), "Input" | "SunOutput" | "PlanetSpin1" | "PlanetSpin2" | "PlanetSpin3") {
            continue;
        }
        j.values.fill(0.);
        j.offset = 0.;
        j.angle = 0.;
        if j.name == "Input" {
            j.a.occurrence = fixed;
            j.b.occurrence = input;
        }
        if j.name == "SunOutput" {
            j.a.occurrence = fixed;
            j.b.occurrence = sun;
        }
        if let Some(k) = j.name.strip_prefix("PlanetSpin").and_then(|n| n.parse::<usize>().ok()).filter(|n| (1..=3).contains(n)) {
            let angle = TAU * (k - 1) as f64 / 3.;
            let origin = Vec3::new(a * angle.cos(), a * angle.sin(), 0.);
            j.a.snap = Snap::Frame { origin, z: Vec3::Z, x: Some(Vec3::X) };
            j.b.snap = j.a.snap.clone();
        }
    }
    let joints = doc.assembly.joints.clone();
    for link in &mut doc.assembly.links {
        if !joints.iter().any(|j| j.id == link.a && j.name == "Input") {
            continue;
        }
        let name = joints.iter().find(|j| j.id == link.b).map(|j| j.name.as_str()).unwrap_or("");
        if !matches!(name, "SunOutput" | "PlanetSpin1" | "PlanetSpin2" | "PlanetSpin3") {
            continue;
        }
        link.ratio = if name == "SunOutput" {
            report["sun_per_input"].as_f64().unwrap_or(1.)
        } else {
            report["planet_relative_per_input"].as_f64().unwrap_or(1.)
        };
        link.offset = 0.;
    }
    doc.metadata.set(KEY, json!({"type":"kue-first-stage/1","report":report}))?;
    Ok(())
}

fn create(s: &mut Session, _: &Value) -> Result<Value> {
    if !s.doc.features.is_empty() || !s.doc.components.is_empty() {
        return Err(bad("planetary.create", "create the planetary stage in a new empty project"));
    }
    let mut next = s.scratch();
    next.execute("worksheet.describe",&json!({"title":"KUE - Planetenstufe und Gesamtgetriebeberechnung","notes":"Variante 1..4 waehlt die Angabe und Aufbauart. Erste Stufe: analytische Evolvente, native Baugruppe. Zahnfuss-Vorauslegung und Bolzenmodell, keine ISO-6336-/Lagerfreigabe."}))?;
    for n in 1..=4 {
        let source = SOURCE.replacen("Variante = 1", &format!("Variante = {n}"), 1)
            .replacen("Aufbauvariante = 1", &format!("Aufbauvariante = {n}"), 1);
        next.execute("worksheet.calcpad.calculate", &json!({"source":source}))?;
        next.execute("worksheet.variant.save", &json!({"name":format!("KUE Variante {n}")}))?;
    }
    next.execute("worksheet.variant.select", &json!({"name":"KUE Variante 1"}))?;
    for name in REFS {
        next.execute("worksheet.reference", &json!({"source":name}))?;
    }
    validation(&next)?;
    let component = |s: &mut Session, name: &str| s.execute("component.create", &json!({"name":name,"parent":0,"activate":true}));
    let gear = |s: &mut Session, name: &str, teeth: &str, center: Value, phase: String, radius: &str, internal: bool| {
        s.execute("solid.involute_gear",&json!({"name":name,"body_name":name,"module":"ref_modul","teeth":teeth,"width":"ref_b","pressure_angle":"ref_alpha_grad * 1 deg","radius":radius,"center":center,"phase":phase,"backlash":"ref_Zahnflankenspiel","internal":internal}))
    };
    component(&mut next, "Ring")?;
    gear(&mut next, "Ring gear", "ref_z_r", json!([0, 0, 0]), "(ref_z_p-1)*pi/ref_z_r * 1 rad".into(), "ref_ring_outer", true)?;
    next.execute("appearance.assign", &json!({"bodies":["Ring gear"],"color":"#535e6b"}))?;
    component(&mut next, "Sun")?;
    gear(&mut next, "Sun gear", "ref_z_s", json!([0, 0, 0]), "0 deg".into(), "ref_bore_s", false)?;
    next.execute("appearance.assign", &json!({"bodies":["Sun gear"],"color":"#ed862e"}))?;
    component(&mut next, "Carrier")?;
    let cyl = |s: &mut Session, name: &str, r: &str, h: &str, base: Value| {
        round_extrusion(s, name, r, h, &base, None)
    };
    cyl(&mut next, "Carrier rear", "ref_a+ref_pin_r+ref_modul", "ref_Platte", json!([0, 0, "-ref_Spiel_axial-ref_Platte"]))?;
    cyl(&mut next, "Carrier front", "ref_a+ref_pin_r+ref_modul", "ref_Platte", json!([0, 0, "ref_b+ref_Spiel_axial"]))?;
    cyl(&mut next, "Carrier hub", "ref_hub_r", "4*ref_modul", json!([0, 0, "-ref_Spiel_axial-ref_Platte-4*ref_modul"]))?;
    next.execute("appearance.assign", &json!({"bodies":["Carrier rear","Carrier front","Carrier hub"],"color":"#3e99a5"}))?;
    let cut = |s: &mut Session, name: &str, r: &str, base: Value, targets: Value| {
        round_extrusion(s, name, r, "ref_b+2*ref_Spiel_axial+2*ref_Platte+2 mm", &base, Some(targets))
    };
    cut(
        &mut next,
        "Sun shaft clearance",
        "ref_bore_s+0.2*ref_modul",
        json!([0, 0, "-ref_Spiel_axial-ref_Platte-1 mm"]),
        json!(["Carrier rear", "Carrier front"]),
    )?;
    // Keep the front support visible without hiding the gear engagement.
    next.execute("appearance.assign", &json!({"bodies":["Carrier front"],"color":"#3e99a5","opacity":0.2}))?;
    for k in 0..3 {
        let t = TAU * k as f64 / 3.;
        let x = format!("ref_a*{}", t.cos());
        let y = format!("ref_a*{}", t.sin());
        next.execute("component.activate", &json!({"component":"Carrier"}))?;
        cut(
            &mut next,
            &format!("Pin seat {}", k + 1),
            "ref_pin_r+0.02 mm",
            json!([x, y, "-ref_Spiel_axial-ref_Platte-1 mm"]),
            json!(["Carrier rear", "Carrier front"]),
        )?;
        cyl(
            &mut next,
            &format!("Planet pin {}", k + 1),
            "ref_pin_r",
            "ref_b+2*ref_Spiel_axial+2*ref_Platte",
            json!([x, y, "-ref_Spiel_axial-ref_Platte"]),
        )?;
        component(&mut next, &format!("Planet{}", k + 1))?;
        let phase = format!("((ref_z_s+ref_z_p)*{t}+ref_z_p*pi-pi)/ref_z_p * 1 rad");
        gear(&mut next, &format!("Planet gear {}", k + 1), "ref_z_p", json!([x, y, 0]), phase, "ref_bore_p", false)?;
        next.execute("appearance.assign", &json!({"bodies":[format!("Planet gear {}",k+1)],"color":"#bac6d1"}))?;
        let bush = format!("Planet bushing {}", k + 1);
        cyl(&mut next, &bush, "ref_bore_p", "ref_b", json!([x, y, 0]))?;
        cut(&mut next, &format!("Bushing bore {}", k + 1), "ref_pin_r+0.05 mm", json!([x, y, "-ref_Spiel_axial-ref_Platte-1 mm"]), json!([bush]))?;
        next.execute("appearance.assign", &json!({"bodies":[bush],"color":"#b49346"}))?;
    }
    next.execute("component.activate", &json!({"component":0}))?;
    next.execute("occurrence.ground", &json!({"occurrence":"Ring:1","grounded":true}))?;
    for (name, b) in [("Input", "Carrier:1"), ("SunOutput", "Sun:1")] {
        next.execute(
            "joint.as_built",
            &json!({"name":name,"type":"revolute","a":"Ring:1","b":b,"at":{"occurrence":"Ring:1","point":[0,0,0],"z":[0,0,1]}}),
        )?;
    }
    for k in 0..3 {
        let t = TAU * k as f64 / 3.;
        let a = value(&next, "a")?;
        next.execute("joint.as_built",&json!({"name":format!("PlanetSpin{}",k+1),"type":"revolute","a":"Carrier:1","b":format!("Planet{}:1",k+1),"at":{"occurrence":"Carrier:1","point":[a*t.cos(),a*t.sin(),0],"z":[0,0,1]}}))?;
    }
    let zs = value(&next, "z_s")?;
    let zr = value(&next, "z_r")?;
    let zp = value(&next, "z_p")?;
    next.execute("joint.motion_link", &json!({"a":"Input","b":"SunOutput","ratio":1.+zr/zs}))?;
    for k in 1..=3 {
        next.execute("joint.motion_link", &json!({"a":"Input","b":format!("PlanetSpin{k}"),"ratio":-zr/zp}))?;
    }
    next.doc_mut().metadata.set(KEY, json!({"type":"kue-first-stage/1"}))?;
    sync(&mut next)?;
    super::worksheet::commit(s, next)
}
fn inspect(s: &mut Session, _: &Value) -> Result<Value> {
    validation(s)
}

/// Author ordinary sketches, driving radius dimensions and extrusion features.
fn round_extrusion(s: &mut Session, name: &str, radius: &str, height: &str, base: &Value, targets: Option<Value>) -> Result<Value> {
    let xyz = base.as_array().filter(|v| v.len() == 3).ok_or_else(|| bad("planetary.create", "circle placement needs x, y, z"))?;
    let x = xyz.first().ok_or_else(|| bad("planetary.create", "missing circle x"))?;
    let y = xyz.get(1).ok_or_else(|| bad("planetary.create", "missing circle y"))?;
    let z = xyz.get(2).ok_or_else(|| bad("planetary.create", "missing sketch offset"))?;
    s.execute("sketch.create", &json!({"name":format!("Kontur - {name}"),"plane":"XY","offset":z}))?;
    let seed = s.doc.eval(radius, solidcraft_doc::expr::Kind::Length)?;
    let drawn = s.execute("sketch.circle.center", &json!({"center":[x,y],"radius":seed}))?;
    let circle = drawn["curves"].as_array().and_then(|c| c.first()).cloned().ok_or_else(|| bad("planetary.create", "circle did not return a curve"))?;
    s.execute("sketch.dimension", &json!({"entities":[circle],"type":"radius","value":radius}))?;
    s.execute("sketch.finish", &json!({}))?;
    let mut params = json!({"name":name,"body_name":name,"distance":height,"operation":if targets.is_some() {"cut"} else {"new"}});
    if let Some(targets) = targets { params["targets"] = targets; }
    s.execute("solid.extrude", &params)
}

fn inputs(s: &mut Session, p: &Value) -> Result<Value> {
    const NAMES: &[&str] = &["Aufbauvariante", "R_Ziel", "T_in", "n_in", "z_s", "modul", "alpha_grad", "breitenfaktor"];
    let source = s.doc.metadata.get(super::worksheet_references::CALCPAD).and_then(|m| m["source"].as_str()).unwrap_or("").to_owned();
    if let Some(name) = p["name"].as_str() {
        let expression = p["expression"]
            .as_str()
            .filter(|e| !e.trim().is_empty() && e.len() <= 400 && !e.contains(['\n', '\r', '#', '$']))
            .ok_or_else(|| bad("planetary.inputs", "a single calculation expression is required"))?;
        if !NAMES.contains(&name) {
            return Err(bad("planetary.inputs", "unknown input"));
        }
        let mut count = 0;
        let lines: Vec<_> = source
            .lines()
            .map(|line| {
                if line.split_once('=').is_some_and(|(n, _)| n.trim() == name) {
                    count += 1;
                    format!("{name} = {expression}")
                } else {
                    line.to_owned()
                }
            })
            .collect();
        if count != 1 {
            return Err(bad("planetary.inputs", "input must have exactly one assignment in the worksheet"));
        }
        return super::worksheet_calcpad::edit(s, &json!({"source":lines.join("\n")}));
    }
    let rows: Vec<_> = source
        .lines()
        .filter_map(|line| {
            let (name, expression) = line.split_once('=')?;
            NAMES.contains(&name.trim()).then(|| json!({"name":name.trim(),"expression":expression.trim()}))
        })
        .collect();
    Ok(json!({"rows":rows}))
}
