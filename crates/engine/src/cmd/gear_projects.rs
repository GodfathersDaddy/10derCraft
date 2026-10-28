//! Saved sizing alternatives, explicit updates and CAD consistency checks.
use super::gear::{KEY, evaluate, native_inputs};
use crate::params::bad;
use crate::{Result, Session};
use serde_json::{Value, json};

pub(super) fn name(value: Option<&Value>, index: usize) -> Result<String> {
    match value {
        None => Ok(format!("Entwurf {}", index + 1)),
        Some(Value::String(s)) if !s.trim().is_empty() && s.trim().len() <= 100 => Ok(s.trim().into()),
        _ => Err(bad("gear.design", "name must contain 1..100 bytes")),
    }
}
fn index(p: &Value, key: &str) -> Result<usize> {
    p.get(key)
        .and_then(Value::as_u64)
        .filter(|i| *i < 100)
        .map(|i| i as usize)
        .ok_or_else(|| bad("gear.review", format!("{key}: saved design index in 0..99 required")))
}
fn item(s: &Session, i: usize) -> Result<Value> {
    s.doc
        .metadata
        .get(KEY)
        .and_then(|v| v["items"].as_array())
        .and_then(|a| a.get(i))
        .cloned()
        .ok_or_else(|| bad("gear.review", "saved design not found"))
}
fn different(a: f64, b: f64) -> bool {
    (a - b).abs() > 1e-10 * a.abs().max(b.abs()).max(1.)
}
fn differences(a: &Value, b: &Value) -> Vec<Value> {
    let ar = a["rows"].as_array().cloned().unwrap_or_default();
    let br = b["rows"].as_array().cloned().unwrap_or_default();
    let mut symbols: Vec<String> = ar.iter().chain(&br).filter_map(|r| r["symbol"].as_str().map(str::to_owned)).collect();
    symbols.sort();
    symbols.dedup();
    symbols
        .into_iter()
        .filter_map(|symbol| {
            let a = ar.iter().find(|r| r["symbol"] == symbol);
            let b = br.iter().find(|r| r["symbol"] == symbol);
            let av = a.and_then(|r| r["value"].as_f64());
            let bv = b.and_then(|r| r["value"].as_f64());
            let same_unit = a.zip(b).is_some_and(|(a, b)| a["unit"] == b["unit"]);
            if same_unit && av.zip(bv).is_some_and(|(a, b)| !different(a, b)) {
                return None;
            }
            Some(json!({"symbol":symbol,"before":av,"after":bv,"before_unit":a.map(|v|&v["unit"]),"after_unit":b.map(|v|&v["unit"]),
            "delta":if same_unit {av.zip(bv).map(|(a,b)|b-a)}else{None}}))
        })
        .collect()
}
fn one(s: &mut Session, i: usize) -> Result<Value> {
    let saved = item(s, i)?;
    let name = saved["name"].as_str().map(str::to_owned).unwrap_or(format!("Entwurf {}", i + 1));
    let current = match evaluate(s, &saved["inputs"]) {
        Ok(v) => v,
        Err(e) => return Ok(json!({"design":i,"name":name,"status":"invalid","error":e.to_string(),"saved":saved})),
    };
    let changes = differences(&saved["snapshot"], &current);
    let id = saved["feature"]["feature"].as_u64();
    let geometry = if let Some(id) = id {
        if let Some(f) = s.doc.feature(id) {
            if current["native_geometry"] != true {
                "unsupported_family"
            } else {
                let mut expected = super::involute::definition(s, &native_inputs(&saved["inputs"]))?;
                if s.doc.units.trim() != "mm" {
                    for (_, e, kind) in expected.inputs_mut() {
                        *e = s.doc.with_design_unit(e, kind).into();
                    }
                }
                if f.suppressed {
                    "suppressed"
                } else if s.model.result(id).is_some_and(|r| r.error.is_some()) {
                    "failed"
                } else if expected == f.kind {
                    "current"
                } else {
                    "different_definition"
                }
            }
        } else {
            "missing"
        }
    } else {
        "unlinked"
    };
    Ok(json!({"design":i,"name":name,"status":if changes.is_empty() {"current"}else{"outdated"},"changes":changes,
        "feature_status":geometry,"feature":id,"current":current,"saved":saved}))
}
pub(super) fn review(s: &mut Session, p: &Value) -> Result<Value> {
    if p.get("design").is_some() {
        return one(s, index(p, "design")?);
    }
    let count = s.doc.metadata.get(KEY).and_then(|v| v["items"].as_array()).map_or(0, Vec::len);
    if count > 100 {
        return Err(bad("gear.review", "too many saved designs"));
    }
    let items = (0..count).map(|i| one(s, i)).collect::<Result<Vec<_>>>()?;
    Ok(json!({"items":items}))
}
pub(super) fn update(s: &mut Session, p: &Value) -> Result<Value> {
    let i = index(p, "design")?;
    let mut saved = item(s, i)?;
    let mut inputs = saved["inputs"].clone();
    if let Some(patch) = p.get("inputs") {
        let object = patch.as_object().ok_or_else(|| bad("gear.update", "inputs must be an object"))?;
        let dest = inputs.as_object_mut().ok_or_else(|| bad("gear.update", "saved inputs are invalid"))?;
        if !object.contains_key("center")
            && ["center_x", "center_y", "center_z"].iter().any(|key| object.contains_key(*key))
            && let Some(center) = dest.remove("center").and_then(|v| v.as_array().cloned())
        {
            for (i, key) in ["center_x", "center_y", "center_z"].iter().enumerate() {
                if let Some(v) = center.get(i) {
                    dest.insert((*key).into(), v.clone());
                }
            }
        }
        for (key, v) in object {
            dest.insert(key.clone(), v.clone());
        }
    }
    let label = if p.get("name").is_some() {
        name(p.get("name"), i)?
    } else {
        saved["name"].as_str().map(str::to_owned).unwrap_or(format!("Entwurf {}", i + 1))
    };
    let current = evaluate(s, &inputs)?;
    let create = match p.get("create") {
        None => false,
        Some(Value::Bool(v)) => *v,
        _ => return Err(bad("gear.update", "create must be boolean")),
    };
    let mut next = s.scratch();
    if create {
        if current["native_geometry"] != true {
            return Err(bad("gear.update", "native flank generator is not implemented for this family"));
        }
        let native = native_inputs(&inputs);
        if let Some(id) = saved["feature"]["feature"].as_u64() {
            let f = next.doc.feature(id).ok_or_else(|| bad("gear.update", "linked feature was deleted; no replacement created"))?;
            if !matches!(f.kind, solidcraft_doc::FeatureKind::InvoluteGear { .. }) {
                return Err(bad("gear.update", "linked feature is not an involute gear"));
            }
            next.execute("timeline.redefine", &json!({"feature":id,"command":"solid.involute_gear","params":native}))?;
        } else {
            saved["feature"] = next.execute("solid.involute_gear", &native)?;
        }
    }
    inputs["name"] = json!(label);
    saved["name"] = json!(label);
    saved["inputs"] = inputs;
    saved["snapshot"] = current;
    let mut state = next.doc.metadata.get(KEY).cloned().ok_or_else(|| bad("gear.update", "saved designs missing"))?;
    let slot =
        state.get_mut("items").and_then(Value::as_array_mut).and_then(|a| a.get_mut(i)).ok_or_else(|| bad("gear.update", "saved design missing"))?;
    *slot = saved;
    next.doc_mut().metadata.set(KEY, state)?;
    super::worksheet::commit(s, next)?;
    // Read the committed document against its freshly evaluated state.
    s.refresh();
    one(s, i)
}
pub(super) fn compare(s: &mut Session, p: &Value) -> Result<Value> {
    let left = index(p, "left")?;
    let right = index(p, "right")?;
    let a = item(s, left)?;
    let b = item(s, right)?;
    let basis = p.get("basis").and_then(Value::as_str).unwrap_or("snapshot");
    let (ar, br) = match basis {
        "snapshot" => (a["snapshot"].clone(), b["snapshot"].clone()),
        "current" => (evaluate(s, &a["inputs"])?, evaluate(s, &b["inputs"])?),
        _ => return Err(bad("gear.compare", "basis must be snapshot or current")),
    };
    Ok(json!({"left":left,"right":right,"basis":basis,"changes":differences(&ar,&br),"left_result":ar,"right_result":br}))
}
