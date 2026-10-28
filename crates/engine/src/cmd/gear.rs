//! Gear-family sizing. Equations are independent of the available CAD constructors.
use super::CommandSpec;
use crate::params::{bad, req_expr};
use crate::{Result, Session};
use serde_json::{Value, json};
use solidcraft_doc::expr::Kind;
use std::f64::consts::PI;

pub(super) const KEY: &str = "org.solidcraft.gear-designs";
const SOURCE: &str = "https://khkgears.net/new/gear_knowledge/gear_technical_reference/calculation_gear_dimensions.html";
pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("gear.design", "Gear Design", design).at("SOLID", "CREATE").icon("cylinder")
        .params("family, module, teeth, mate_teeth, width, pressure_angle, helix_angle, starts, diameter_factor, shaft_angle, groove, torque_nm, speed_rpm: expressions; create?: bool; radius?, center?, phase?, backlash?; saves sizing and optionally creates an exact native spur/internal gear"),
    CommandSpec::new("gear.evaluate", "Evaluate Gear Design", evaluate).icon("params").noundo().doc_only()
        .params("Same inputs as gear.design; read-only sizing, explicit units and geometry capability; no strength or manufacturing certification"),
    CommandSpec::new("gear.catalog", "Gear Families", catalog).icon("params").noundo().doc_only(),
    CommandSpec::new("gear.designs", "Saved Gear Designs", read).icon("params").noundo().doc_only(),
    CommandSpec::new("gear.review", "Review Gear Changes", super::gear_projects::review).icon("params").noundo()
        .params("design?: zero-based saved design index; read-only comparison of saved results, current parameters and linked CAD feature"),
    CommandSpec::new("gear.update", "Update Saved Gear Design", super::gear_projects::update).icon("params")
        .params("design: index, inputs?: expression overrides, name?: text; create?: bool; explicitly updates existing native feature in place, or only refreshes calculation"),
    CommandSpec::new("gear.compare", "Compare Gear Designs", super::gear_projects::compare).icon("params").noundo().doc_only()
        .params("left, right: saved design indices; basis?: snapshot|current (default snapshot); unit-checked result differences, no document changes"),
];
const FAMILIES: &[(&str, &str, bool)] = &[
    ("spur", "Stirnrad", true),
    ("internal", "Hohlrad", true),
    ("helical", "Schraegverzahntes Stirnrad", false),
    ("internal_helical", "Schraegverzahntes Hohlrad", false),
    ("herringbone", "Pfeilverzahnung ohne Mittelnut", false),
    ("double_helical", "Doppelschraegverzahnung mit Mittelnut", false),
    ("worm", "Schnecke / Schneckenrad", false),
    ("bevel", "Geradverzahntes Kegelrad", false),
    ("spiral_bevel", "Spiralkegelrad", false),
    ("rack", "Zahnstange", false),
];
fn catalog(_: &mut Session, _: &Value) -> Result<Value> {
    Ok(
        json!({"families":FAMILIES.iter().map(|(id,label,cad)|json!({"id":id,"label":label,"native_geometry":cad})).collect::<Vec<_>>(),"sources":[SOURCE]}),
    )
}
fn read(s: &mut Session, _: &Value) -> Result<Value> {
    Ok(s.doc.metadata.get(KEY).cloned().unwrap_or_else(|| json!({"items":[]})))
}
fn scalar(s: &Session, p: &Value, key: &str, default: &str, kind: Kind) -> Result<f64> {
    let p = if p.get(key).is_some() { p.clone() } else { json!({key:default}) };
    let e = req_expr("gear.evaluate", &p, key)?;
    let v = s.doc.eval(&e, kind).map_err(|e| bad("gear.evaluate", format!("{key}: {e}")))?;
    if !v.is_finite() || v.abs() > 1e9 {
        return Err(bad("gear.evaluate", format!("{key}: finite bounded input required")));
    }
    Ok(v)
}
fn positive(v: f64, name: &str) -> Result<f64> {
    if v <= 0. {
        return Err(bad("gear.evaluate", format!("{name} must be positive")));
    }
    Ok(v)
}
fn teeth(v: f64, name: &str) -> Result<f64> {
    if !(1. ..=10_000.).contains(&v) || (v - v.round()).abs() > 1e-9 {
        return Err(bad("gear.evaluate", format!("{name}: integer in 1..10000 required")));
    }
    Ok(v)
}
pub(super) fn evaluate(s: &mut Session, p: &Value) -> Result<Value> {
    if !p.is_object() {
        return Err(bad("gear.evaluate", "inputs must be an object"));
    }
    let family = p.get("family").and_then(Value::as_str).unwrap_or("spur");
    let (_, _, native) = FAMILIES.iter().find(|(id, _, _)| *id == family).ok_or_else(|| bad("gear.evaluate", "unknown gear family"))?;
    let val = |k, d, kind| scalar(s, p, k, d, kind);
    let m = positive(val("module", "2 mm", Kind::Length)?, "module")?;
    let z = teeth(val("teeth", "30", Kind::Unitless)?, "teeth")?;
    let z2 = teeth(val("mate_teeth", "60", Kind::Unitless)?, "mate_teeth")?;
    if family != "worm" && z < 3. {
        return Err(bad("gear.evaluate", "at least three teeth required for positive standard root dimensions"));
    }
    let b = positive(val("width", "20 mm", Kind::Length)?, "width")?;
    let alpha = val("pressure_angle", "20 deg", Kind::Angle)?;
    if !(5_f64.to_radians()..=45_f64.to_radians()).contains(&alpha) {
        return Err(bad("gear.evaluate", "pressure_angle must be in 5..45 deg"));
    }
    let torque = val("torque_nm", "100", Kind::Unitless)?;
    let speed = val("speed_rpm", "1500", Kind::Unitless)?;
    let mut rows = vec![json!({"symbol":"P_in","value":torque*speed*2.*PI/60.,"unit":"W","equation":"T_in * n_in * 2*pi/60"})];
    let mut add =
        |symbol: &str, value: f64, unit: &str, equation: &str| rows.push(json!({"symbol":symbol,"value":value,"unit":unit,"equation":equation}));
    add("b", b, "mm", "Zahnbreite (Eingabe)");
    add("T_in", torque, "Nm", "Eingangsdrehmoment (Eingabe)");
    add("n_in", speed, "rpm", "Eingangsdrehzahl (Eingabe)");
    let mut warnings = vec!["Vorentwurf ohne Profilverschiebung und ohne Tragfaehigkeits-, Lager-, Toleranz- oder Fertigungsnachweis.".to_owned()];
    if family == "worm" {
        let starts = teeth(val("starts", "2", Kind::Unitless)?, "starts")?;
        let q = positive(val("diameter_factor", "10", Kind::Unitless)?, "diameter_factor")?;
        let gamma = (starts / q).atan();
        let d1 = m * q;
        let d2 = m * z2;
        add("gamma", gamma.to_degrees(), "deg", "atan(starts/q)");
        add("d_worm", d1, "mm", "m_axial*q");
        add("d_wheel", d2, "mm", "m_axial*z_wheel");
        add("lead", PI * m * starts, "mm", "pi*m_axial*starts");
        add("a", (d1 + d2) / 2., "mm", "(d_worm+d_wheel)/2");
        add("i", z2 / starts, "1", "z_wheel/starts");
        add("n_out", speed * starts / z2, "rpm", "n_in/i (Betrag; Richtung durch Einbau)");
        add("Ft_worm", 2000. * torque / d1, "N", "2*T_in/d_worm");
        warnings.push("Modul hier AXIALMODUL der Schnecke. 90-Grad-Achsen; nur Bezugsabmessungen. Radflanke benoetigt Werkzeug-Huellflaeche; keine Selbsthemmungszusage.".into());
    } else if matches!(family, "bevel" | "spiral_bevel") {
        let sigma = val("shaft_angle", "90 deg", Kind::Angle)?;
        if !(10_f64.to_radians()..=170_f64.to_radians()).contains(&sigma) {
            return Err(bad("gear.evaluate", "shaft_angle must be in 10..170 deg"));
        }
        let delta = sigma.sin().atan2(z2 / z + sigma.cos());
        let delta2 = sigma - delta;
        if delta <= 0. || delta2 <= 0. || delta >= PI / 2. || delta2 >= PI / 2. {
            return Err(bad("gear.evaluate", "pitch cones must both be in 0..90 deg; crown/internal bevels are not covered"));
        }
        let d = m * z;
        let re = d / (2. * delta.sin());
        if b >= re {
            return Err(bad("gear.evaluate", "width must be smaller than outer cone distance"));
        }
        add("d_outer", d, "mm", "m_outer_transverse*z");
        add("d_mate_outer", m * z2, "mm", "m_outer_transverse*z_mate");
        add("delta", delta.to_degrees(), "deg", "atan2(sin(Sigma), z_mate/z+cos(Sigma))");
        add("delta_mate", delta2.to_degrees(), "deg", "Sigma-delta");
        add("R_outer", re, "mm", "d_outer/(2*sin(delta))");
        add("d_mean", d * (1. - b / (2. * re)), "mm", "d_outer*(1-b/(2*R_outer))");
        add("i", z2 / z, "1", "z_mate/z");
        add("n_out", speed * z / z2, "rpm", "n_in/i (Betrag)");
        add("Ft_mean", 2000. * torque / (d * (1. - b / (2. * re))), "N", "2*T_in/d_mean");
        if b > re / 3. {
            warnings.push("Zahnbreite groesser als R_outer/3: Vorentwurf pruefen.".into());
        }
        warnings.push("Modul am AUSSENENDE im Stirnschnitt. Nur Teilkegel-/Kinematikberechnung; Flankensystem, Spiralwinkel und Herstellverfahren sind noch nicht ausgelegt.".into());
    } else {
        let internal = matches!(family, "internal" | "internal_helical");
        let helical = matches!(family, "helical" | "internal_helical" | "herringbone" | "double_helical");
        let beta = if helical { val("helix_angle", "20 deg", Kind::Angle)? } else { 0. };
        if beta.abs() >= 60_f64.to_radians() {
            return Err(bad("gear.evaluate", "helix_angle magnitude must be below 60 deg"));
        }
        let mt = m / beta.cos();
        let at = (alpha.tan() / beta.cos()).atan();
        let d = mt * z;
        let d2 = mt * z2;
        if internal && z2 >= z {
            return Err(bad("gear.evaluate", "internal gear teeth must exceed mate_teeth"));
        }
        let gap = if family == "double_helical" { val("groove", "2 mm", Kind::Length)? } else { 0. };
        if gap < 0. || gap >= b || (family == "double_helical" && gap == 0.) {
            return Err(bad("gear.evaluate", "double-helical groove must be positive and smaller than width"));
        }
        let half = matches!(family, "herringbone" | "double_helical");
        let effective = if half { (b - gap) / 2. } else { b };
        add("m_transverse", mt, "mm", "m_normal/cos(beta)");
        add("alpha_transverse", at.to_degrees(), "deg", "atan(tan(alpha_normal)/cos(beta))");
        add("d", d, "mm", "m_transverse*z");
        add("d_mate", d2, "mm", "m_transverse*z_mate");
        add("d_base", d * at.cos(), "mm", "d*cos(alpha_transverse)");
        add("d_tip", d + if internal { -2. * m } else { 2. * m }, "mm", "d +/- 2*m_normal");
        add("d_root", d + if internal { 2.5 * m } else { -2.5 * m }, "mm", "d -/+ 2.5*m_normal");
        add("pitch_normal", PI * m, "mm", "pi*m_normal");
        add("z_virtual", z / beta.cos().powi(3), "1", "z/cos(beta)^3");
        add("Ft", 2000. * torque / d, "N", "2*T_in/d");
        add("Fr", (2000. * torque / d) * at.tan(), "N", "Ft*tan(alpha_transverse)");
        if family == "rack" {
            add("travel_per_rev", PI * d, "mm/rev", "pi*d");
            add("rack_speed", PI * d * speed / 60., "mm/s", "pi*d*n_in/60");
        } else {
            add("a", if internal { (d - d2) / 2. } else { (d + d2) / 2. }, "mm", "(d +/- d_mate)/2");
            add("i", z2 / z, "1", "z_mate/z");
            add("n_out", speed * z / z2 * if internal { 1. } else { -1. }, "rpm", "n_in*z/z_mate; aussen gegenlaeufig, innen gleichlaeufig");
        }
        if helical {
            add("twist_per_half", effective * beta.tan() * 2. / d * 180. / PI, "deg", "b_effective*tan(beta)/(d/2)");
            add("epsilon_beta_half", effective * beta.sin().abs() / (PI * m), "1", "b_effective*abs(sin(beta))/(pi*m_normal)");
            add(
                "Fa_net_ideal",
                if half { 0. } else { (2000. * torque / d) * beta.tan() },
                "N",
                "Ft*tan(beta); symmetrische Pfeilhaelften ideal aufgehoben",
            );
            if half {
                warnings.push("Axialkraft null nur bei ideal symmetrischer Lastverteilung. Mittelnut reduziert die wirksame Breite.".into());
            }
            warnings.push("Normalmodulsystem; Parallelachsenpaar: Aussenraeder entgegengesetzte Hand, Hohlrad/Paar gleiche Hand.".into());
        }
        if z / beta.cos().powi(3) < 2. / alpha.sin().powi(2) {
            warnings.push("Unterschnitt bei Standardzahnstange ohne Profilverschiebung moeglich; native Geometrie kann dies ablehnen.".into());
        }
        warnings.push("Kein Eingriffs-/Interferenznachweis allein aus diesen Bezugsabmessungen.".into());
    }
    if rows.iter().any(|r| r["value"].as_f64().is_none_or(|x| !x.is_finite())) {
        return Err(bad("gear.evaluate", "non-finite result"));
    }
    Ok(json!({"family":family,"native_geometry":native,"rows":rows,"warnings":warnings,"sources":[SOURCE],"inputs":p,
        "geometry_status":if *native {"Exact involute flanks; circular root transition, not a generated rack trochoid. Native constructor has separate validity limits."}else{"Sizing only; no native CAD flank generator yet. No surrogate spur geometry will be created."}}))
}
fn design(s: &mut Session, p: &Value) -> Result<Value> {
    let result = evaluate(s, p)?;
    let create = p.get("create").and_then(Value::as_bool).unwrap_or(false);
    if create && result["native_geometry"] != true {
        return Err(bad("gear.design", "native flank generator for this family is not implemented; use create:false to save sizing"));
    }
    let mut next = s.scratch();
    let feature = if create {
        let inputs = native_inputs(p);
        Some(next.execute("solid.involute_gear", &inputs)?)
    } else {
        None
    };
    let mut saved = next.doc.metadata.get(KEY).cloned().unwrap_or_else(|| json!({"items":[]}));
    let items = saved.get_mut("items").and_then(Value::as_array_mut).ok_or_else(|| bad("gear.design", "invalid gear design metadata"))?;
    if items.len() >= 100 {
        return Err(bad("gear.design", "maximum 100 saved sizing records"));
    }
    let name = super::gear_projects::name(p.get("name"), items.len())?;
    items.push(json!({"name":name,"inputs":p,"snapshot":result,"feature":feature,"snapshot_note":"Sizing snapshot: gear.review detects changes; gear.update explicitly refreshes calculation and optionally CAD."}));
    next.doc_mut().metadata.set(KEY, saved)?;
    super::worksheet::commit(s, next)?;
    Ok(json!({"calculation":result,"feature":feature}))
}
pub(super) fn native_inputs(p: &Value) -> Value {
    let mut inputs = p.clone();
    inputs["internal"] = json!(p.get("family").and_then(Value::as_str) == Some("internal"));
    for (key, default) in [("module", "2 mm"), ("teeth", "30"), ("width", "20 mm"), ("radius", "5 mm")] {
        if inputs.get(key).is_none() {
            inputs[key] = json!(default);
        }
    }
    inputs
}
