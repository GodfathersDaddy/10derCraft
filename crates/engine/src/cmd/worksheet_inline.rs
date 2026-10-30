//! Inline worksheet edits and presentation units. Full precision results and CAD
//! reference values remain separate from rounded or converted display values.
use super::CommandSpec;
use crate::{
    Result, Session,
    params::bad,
    schema::{TypedContract, output_schema_for},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
const PRESENTATION: &str = super::worksheet::PRESENTATION;
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Formula {
    name: String,
    expression: String,
    #[serde(default)]
    line: Option<usize>,
    #[serde(default)]
    old_name: Option<String>,
    #[serde(default)]
    native: bool,
}
#[derive(Serialize, JsonSchema)]
struct FormulaReply {
    id: String,
    name: String,
    calculated: bool,
    error: Option<String>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct UnitRequest {
    source: String,
    unit: String,
}
#[derive(Serialize, JsonSchema)]
struct UnitReply {
    source: String,
    value: f64,
    unit: String,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Pin {
    source: String,
}
#[derive(Serialize, JsonSchema)]
struct PinReply {
    source: String,
    cad_parameter: Option<String>,
}
const FORMULA: TypedContract<Formula> = TypedContract::new();
const UNIT: TypedContract<UnitRequest> = TypedContract::new();
const PIN: TypedContract<Pin> = TypedContract::new();
pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("worksheet.formula", "Edit Worksheet Formula", crate::checked!("worksheet.formula", formula))
        .icon("params")
        .contract(&FORMULA)
        .output(output_schema_for::<FormulaReply>),
    CommandSpec::new("worksheet.unit", "Set Worksheet Display Unit", crate::checked!("worksheet.unit", unit))
        .icon("params")
        .contract(&UNIT)
        .output(output_schema_for::<UnitReply>)
        .doc_only(),
    CommandSpec::new("worksheet.pin", "Select Worksheet Reference", crate::checked!("worksheet.pin", pin))
        .icon("status_pin")
        .contract(&PIN)
        .output(output_schema_for::<PinReply>),
];
fn error(message: impl Into<String>) -> crate::EngineError {
    bad("worksheet.inline", message)
}
fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 160 && !name.starts_with(char::is_numeric) && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}
pub(super) fn layout(s: &Session) -> Value {
    s.doc.metadata.get(PRESENTATION).and_then(|v| v.get("worksheet")).filter(|v| v.is_object()).cloned().unwrap_or_else(|| json!({}))
}
fn store(s: &mut Session, value: Value) -> Result<()> {
    s.execute("worksheet.presentation", &json!({"workspace":"worksheet","layout":value}))?;
    Ok(())
}
pub(super) fn requests(s: &Session) -> Value {
    let value = layout(s);
    let mut units = serde_json::Map::new();
    for (name, unit) in value["unit_overrides"].as_object().into_iter().flatten().take(128) {
        if valid_name(name) && unit.as_str().is_some_and(|u| u.len() <= 160) {
            units.insert(name.clone(), unit.clone());
        }
    }
    json!(units)
}
pub(super) fn fingerprint(values: &Value) -> Result<String> {
    let bytes = serde_json::to_vec(values).map_err(|e| error(e.to_string()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
pub(super) fn display_results(s: &mut Session, result: &Value) -> Result<()> {
    let mut value = layout(s);
    value["unit_values"] = result["display_results"].as_object().map_or_else(|| json!({}), |v| json!(v));
    value["unit_errors"] = result["display_errors"].as_object().map_or_else(|| json!({}), |v| json!(v));
    value["unit_values_source"] = json!(fingerprint(&result["results"])?);
    store(s, value)
}
fn rename_tokens(source: &str, old: &str, new: &str) -> String {
    let mut out = String::new();
    for line in source.lines() {
        if line.trim_start().starts_with('\'') {
            out.push_str(line);
        } else {
            let mut token = String::new();
            for c in line.chars().chain(std::iter::once('\0')) {
                if c.is_alphanumeric() || c == '_' {
                    token.push(c);
                } else {
                    out.push_str(if token == old { new } else { &token });
                    token.clear();
                    if c != '\0' {
                        out.push(c);
                    }
                }
            }
        }
        out.push('\n');
    }
    out
}
fn rename_references(s: &mut Session, old: &str, new: &str) -> Result<()> {
    if old == new {
        return Ok(());
    }
    let key = super::worksheet_references::KEY;
    if let Some(mut value) = s.doc.metadata.get(key).cloned() {
        if let Some(items) = value["items"].as_array_mut() {
            for item in items {
                if item["source"] == old {
                    item["source"] = json!(new);
                }
            }
        }
        s.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, key, value)?;
    }
    let mut value = layout(s);
    for field in ["unit_overrides", "unit_values"] {
        if let Some(map) = value.get_mut(field).and_then(Value::as_object_mut)
            && let Some(entry) = map.remove(old)
        {
            map.insert(new.into(), entry);
        }
    }
    if let Some(pins) = value.get_mut("pinned_references").and_then(Value::as_array_mut) {
        for pin in pins {
            if pin == old {
                *pin = json!(new);
            }
        }
    }
    if let Some(positions) = value.get_mut("positions").and_then(Value::as_object_mut)
        && let Some(position) = positions.remove(&format!("native:{old}"))
    {
        positions.insert(format!("native:{new}"), position);
    }
    if let Some(positions) = value.get_mut("positions").and_then(Value::as_object_mut) {
        for prefix in ["formula:native:", "formula:calcpad:"] {
            if let Some(position) = positions.remove(&format!("{prefix}{old}")) {
                positions.insert(format!("{prefix}{new}"), position);
            }
        }
    }
    if let Some(map) = value.get_mut("suspended_references").and_then(Value::as_object_mut)
        && let Some(mut entry) = map.remove(old)
    {
        entry["source"] = json!(new);
        map.insert(new.into(), entry);
    }
    store(s, value)
}
fn formula(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Formula = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    if !valid_name(&p.name) || p.expression.len() > 4096 || p.expression.contains(['\n', '\r']) {
        return Err(error("bounded variable name and one expression required"));
    }
    let old = p.old_name.as_deref().unwrap_or(&p.name);
    if !valid_name(old) {
        return Err(error("invalid previous variable name"));
    }
    let mut next = s.scratch();
    rename_references(&mut next, old, &p.name)?;
    let mut calculation_error = None;
    let id = if p.native {
        if old != p.name {
            next.execute("parameters.rename", &json!({"name":old,"new_name":p.name}))?;
        }
        next.execute("worksheet.set", &json!({"name":p.name,"expression":p.expression}))?;
        if let Some(preferred) = requests(&next).get(&p.name).and_then(Value::as_str).map(str::to_owned) {
            unit(&mut next, &json!({"source":p.name,"unit":preferred}))?;
        }
        format!("native:{}", p.name)
    } else {
        let original = next.doc.metadata.get(super::worksheet_references::CALCPAD).and_then(|v| v["source"].as_str()).unwrap_or("").to_owned();
        let source = if old != p.name { rename_tokens(&original, old, &p.name) } else { original };
        let mut lines: Vec<_> = source.lines().map(str::to_owned).collect();
        let index = if let Some(line) = p.line {
            if lines.get(line).and_then(|l| l.split_once('=').map(|(name, _)| name.trim())).is_none_or(|n| n != p.name) {
                return Err(error("worksheet line changed; select the formula again"));
            }
            line
        } else {
            if lines.iter().any(|l| l.split_once('=').is_some_and(|(n, _)| n.trim() == p.name)) {
                return Err(error("variable already defined"));
            }
            lines.push(String::new());
            lines.len() - 1
        };
        if let Some(line) = lines.get_mut(index) {
            *line = format!("{} = {}", p.name, p.expression);
        }
        next.execute("worksheet.calcpad.edit", &json!({"source":lines.join("\n")}))?;
        if let Err(e) = next.execute("worksheet.calcpad.calculate", &json!({})) {
            calculation_error = Some(e.to_string());
        }
        format!("calcpad:{index}")
    };
    super::worksheet::commit(s, next)?;
    Ok(json!({"id":id,"name":p.name,"calculated":calculation_error.is_none(),"error":calculation_error}))
}
fn unit(s: &mut Session, p: &Value) -> Result<Value> {
    let p: UnitRequest = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    if !valid_name(&p.source) || p.unit.len() > 160 || p.unit.chars().any(char::is_control) {
        return Err(error("invalid display unit"));
    }
    let mut next = s.scratch();
    let mut value = layout(&next);
    if !value["unit_overrides"].is_object() {
        value["unit_overrides"] = json!({});
    }
    if p.unit.trim().is_empty() {
        if let Some(map) = value["unit_overrides"].as_object_mut() {
            map.remove(&p.source);
        }
    } else {
        value["unit_overrides"][&p.source] = json!(p.unit.trim());
    }
    store(&mut next, value)?;
    let raw = next.doc.metadata.get(super::worksheet_references::CALCPAD).cloned().unwrap_or(Value::Null);
    let result = if raw["results"].get(&p.source).is_some() {
        next.execute("worksheet.calcpad.calculate", &json!({}))?;
        let display = layout(&next);
        if let Some(message) = display["unit_errors"][&p.source].as_str() {
            return Err(error(message));
        }
        display["unit_values"].get(&p.source).cloned().unwrap_or_else(|| raw["results"][&p.source].clone())
    } else {
        let rows = next.execute("worksheet.read", &json!({}))?;
        let row = rows["rows"].as_array().into_iter().flatten().find(|r| r["name"] == p.source).ok_or_else(|| error("variable missing"))?;
        let source_unit = row["unit"].as_str().unwrap_or("");
        let target = if p.unit.is_empty() { source_unit } else { p.unit.as_str() };
        let number = row["value"].as_f64().ok_or_else(|| error("scalar result required"))?;
        let outcome = super::worksheet_calcpad::local(&format!("{} = {}{}", p.source, number, source_unit), &json!({p.source.clone():target}))?;
        if outcome["ok"] != true {
            return Err(error(outcome["error"].as_str().unwrap_or("local calculation failed")));
        }
        if let Some(message) = outcome["display_errors"][&p.source].as_str() {
            return Err(error(message));
        }
        let converted = outcome["display_results"].get(&p.source).cloned().unwrap_or_else(|| json!({"value":number,"unit":source_unit}));
        let result = json!({"value":converted["value"],"unit":converted["unit"],"raw_value":row["value"],"raw_unit":row["unit"]});
        let mut layout = layout(&next);
        layout["unit_values"][&p.source] = result.clone();
        store(&mut next, layout)?;
        result
    };
    let number = result["value"].as_f64().filter(|v| v.is_finite()).ok_or_else(|| error("finite scalar result required"))?;
    super::worksheet::commit(s, next)?;
    Ok(json!({"source":p.source,"value":number,"unit":result["unit"]}))
}
fn pin(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Pin = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    if !valid_name(&p.source) {
        return Err(error("invalid reference name"));
    }
    let mut next = s.scratch();
    if next.doc.metadata.get(super::worksheet_references::CALCPAD).is_some_and(|v| v["dirty"] == true) {
        next.execute("worksheet.calcpad.calculate", &json!({}))?;
    }
    let mut presentation = layout(&next);
    let mut selected: Vec<String> = serde_json::from_value(presentation["pinned_references"].clone()).unwrap_or_default();
    let mut bindings = super::worksheet_references::bindings(&next);
    if selected.contains(&p.source) || bindings.iter().any(|b| b["source"] == p.source) {
        if let Some(binding) = bindings.iter().find(|b| b["source"] == p.source) {
            if !presentation["suspended_references"].is_object() {
                presentation["suspended_references"] = json!({});
            }
            presentation["suspended_references"][&p.source] = binding.clone();
        }
        selected.retain(|n| n != &p.source);
        bindings.retain(|b| b["source"] != p.source);
        presentation["pinned_references"] = json!(selected);
        let mut references = next.doc.metadata.get(super::worksheet_references::KEY).cloned().unwrap_or_else(|| json!({}));
        references["items"] = json!(bindings);
        next.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, super::worksheet_references::KEY, references)?;
        store(&mut next, presentation)?;
        super::worksheet::commit(s, next)?;
        return Ok(json!({"source":p.source,"cad_parameter":null}));
    }
    let sheet = next.execute("worksheet.read", &json!({}))?;
    let row = sheet["calcpad"]["results"]
        .get(&p.source)
        .cloned()
        .or_else(|| sheet["rows"].as_array().into_iter().flatten().find(|r| r["name"] == p.source).cloned())
        .ok_or_else(|| error("select a calculated scalar variable"))?;
    if !row["value"].as_f64().is_some_and(|v| v.is_finite()) {
        return Err(error("calculated scalar result required"));
    }
    let mut cad = None;
    if solidcraft_doc::expr::unit_info(row["unit"].as_str().unwrap_or("")).is_some() {
        if let Some(binding) = layout(&next)["suspended_references"].get(&p.source).cloned()
            && binding["parameter"].as_str().is_some_and(|n| next.doc.param(n).is_some())
        {
            let mut rows = super::worksheet_references::bindings(&next);
            rows.push(binding);
            next.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, super::worksheet_references::KEY, json!({"items":rows}))?;
            let mut presentation = layout(&next);
            if let Some(map) = presentation["suspended_references"].as_object_mut() {
                map.remove(&p.source);
            }
            store(&mut next, presentation)?;
        }
        let result = next.execute("worksheet.reference", &json!({"source":p.source}))?;
        cad = result["references"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|v| v["source"] == p.source)
            .and_then(|v| v["parameter"].as_str())
            .map(str::to_owned);
    }
    let mut value = layout(&next);
    let mut pins: Vec<String> = serde_json::from_value(value["pinned_references"].clone()).unwrap_or_default();
    if !pins.contains(&p.source) {
        if pins.len() >= 128 {
            return Err(error("at most 128 references"));
        }
        pins.push(p.source.clone());
    }
    value["pinned_references"] = json!(pins);
    store(&mut next, value)?;
    super::worksheet::commit(s, next)?;
    Ok(json!({"source":p.source,"cad_parameter":cad}))
}
