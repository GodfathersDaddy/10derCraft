//! Explicit calculation exports. CAD reads committed parameters, not pending results.
use crate::params::{bad, str_};
use crate::{Result, Session};
use serde_json::{Value, json};
use solidcraft_doc::expr::{self, Kind};

pub const KEY: &str = "org.solidcraft.calculation-references";
pub const CALCPAD: &str = "org.solidcraft.calcpad";

pub fn bindings(s: &Session) -> Vec<Value> {
    s.doc.metadata.get(KEY).and_then(|m| m["items"].as_array()).cloned().unwrap_or_default()
}

fn source(s: &Session, name: &str) -> Result<(expr::Value, String)> {
    if let Some(v) = s.doc.metadata.get(CALCPAD).and_then(|m| m["results"].get(name)) {
        if s.doc.metadata.get(CALCPAD).is_some_and(|m| m["dirty"] == true) {
            return Err(bad("worksheet.reference", "calculation changed; calculate locally before updating CAD"));
        }
        let unit = v["unit"].as_str().unwrap_or("");
        let value = v["value"].as_f64().filter(|v| v.is_finite()).ok_or_else(|| bad("worksheet.reference", "result is not a finite scalar"))?;
        let (kind, scale) =
            expr::unit_info(unit).ok_or_else(|| bad("worksheet.reference", "CAD references require length, angle or dimensionless results"))?;
        let value = match kind {
            Kind::Length => expr::Value::length(value * scale),
            Kind::Angle => expr::Value::angle(value * scale),
            Kind::Unitless => expr::Value::num(value),
        };
        return Ok((value, unit.into()));
    }
    let (values, _) = s.doc.param_values();
    let value = values.get(name).copied().ok_or_else(|| bad("worksheet.reference", format!("no valid result `{name}`")))?;
    let unit = s.doc.param(name).map(|p| p.unit.clone()).ok_or_else(|| bad("worksheet.reference", "select a named calculation result"))?;
    Ok((value, unit))
}

fn literal(value: expr::Value) -> Result<String> {
    let unit = match value.kind() {
        Some(Kind::Length) => " mm",
        Some(Kind::Angle) => " rad",
        Some(Kind::Unitless) => "",
        None => return Err(bad("worksheet.reference", "unsupported result dimension")),
    };
    Ok(format!("{}{unit}", value.v))
}

pub fn state(s: &Session) -> Value {
    let (values, _) = s.doc.param_values();
    let rows: Vec<Value> = bindings(s)
        .into_iter()
        .map(|binding| {
            let name = binding["source"].as_str().unwrap_or("");
            let cad = binding["parameter"].as_str().unwrap_or("");
            let old = values.get(cad).copied();
            let unit = s.doc.param(cad).map(|p| p.unit.as_str()).unwrap_or("");
            let result = source(s, name).and_then(|(new, _)| {
                if old.is_none_or(|old| old.kind() != new.kind()) {
                    return Err(bad("worksheet.references", "CAD reference missing or result dimension changed"));
                }
                Ok(new)
            });
            let new = result.as_ref().ok().copied();
            let changed = match (old, new) {
                (Some(a), Some(b)) => (a.v - b.v).abs() > 1e-10 * a.v.abs().max(b.v.abs()).max(1.),
                _ => true,
            };
            json!({"source":name,"parameter":cad,"old_value":old.map(|v| expr::value_in_unit(v,unit)),
            "new_value":new.map(|v| expr::value_in_unit(v,unit)),"unit":unit,"changed":changed,
            "error":result.err().map(|e| e.to_string()),"users":s.doc.param_users(cad)})
        })
        .collect();
    let pending = rows.iter().filter(|r| r["changed"] == true).count();
    json!({"references":rows,"pending":pending})
}

pub fn read(s: &mut Session, _: &Value) -> Result<Value> {
    Ok(state(s))
}

pub fn reference(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_(p, "source").ok_or_else(|| bad("worksheet.reference", "source required"))?;
    let mut rows = bindings(s);
    let existing = rows.iter().find(|r| r["source"] == name).cloned();
    let enabled = p["enabled"].as_bool().unwrap_or(true);
    let mut next = s.scratch();
    if !enabled {
        if let Some(row) = existing {
            let cad = row["parameter"].as_str().ok_or_else(|| bad("worksheet.reference", "invalid binding"))?;
            next.execute("parameters.delete", &json!({"name":cad}))?;
            rows.retain(|r| r["source"] != name);
        }
    } else if existing.is_none() {
        if rows.len() >= 128 {
            return Err(bad("worksheet.reference", "at most 128 CAD references"));
        }
        let (value, unit) = source(s, name)?;
        let default = format!("ref_{name}");
        let cad = str_(p, "parameter").unwrap_or(&default).trim();
        if s.doc.param(cad).is_some() || rows.iter().any(|r| r["parameter"] == cad) {
            return Err(bad("worksheet.reference", "CAD reference name is already in use"));
        }
        next.execute("parameters.add",&json!({"name":cad,"expression":literal(value)?,"unit":unit,"comment":format!("Calculation reference: {name}; committed by worksheet.apply")}))?;
        rows.push(json!({"source":name,"parameter":cad}));
    }
    next.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY, json!({"items":rows}))?;
    super::worksheet::commit(s, next)?;
    Ok(state(s))
}

pub fn apply(s: &mut Session, _: &Value) -> Result<Value> {
    let report = state(s);
    let mut changes = serde_json::Map::new();
    if let Some(rows) = report["references"].as_array() {
        for row in rows.iter().filter(|r| r["changed"] == true) {
            if row["error"].is_string() {
                return Err(bad("worksheet.apply", row["error"].as_str().unwrap_or("invalid reference")));
            }
            let name = row["source"].as_str().ok_or_else(|| bad("worksheet.apply", "invalid source"))?;
            let cad = row["parameter"].as_str().ok_or_else(|| bad("worksheet.apply", "invalid parameter"))?;
            changes.insert(cad.into(), json!(literal(source(s, name)?.0)?));
        }
    }
    if !changes.is_empty() {
        let mut next = s.scratch();
        next.execute("parameters.change", &json!({"changes":changes}))?;
        super::planetary::sync(&mut next)?;
        super::worksheet::commit(s, next)?;
    }
    Ok(json!({"applied":changes.keys().collect::<Vec<_>>(),"before":report,"after":state(s)}))
}
