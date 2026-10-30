//! Named family rows select ordinary calculation definitions; CAD aliases stay stable.
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
pub const KEY: &str = "org.solidcraft.calculation-families";
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Table {
    name: String,
    columns: Vec<String>,
    rows: BTreeMap<String, BTreeMap<String, String>>,
}
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Set {
    table: Table,
}
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Select {
    table: String,
    row: String,
}
#[derive(Serialize, JsonSchema)]
struct Reply {
    name: String,
    active: Option<String>,
}
const SET: TypedContract<Set> = TypedContract::new();
const SELECT: TypedContract<Select> = TypedContract::new();
pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("worksheet.family.set", "Save Calculation Family", crate::checked!("worksheet.family.set", set))
        .icon("params")
        .contract(&SET)
        .output(output_schema_for::<Reply>),
    CommandSpec::new("worksheet.family.select", "Select Calculation Family Row", crate::checked!("worksheet.family.select", select))
        .icon("params")
        .contract(&SELECT)
        .output(output_schema_for::<Reply>),
];
fn err(s: impl Into<String>) -> crate::EngineError {
    bad("worksheet.family", s)
}
fn identifier(n: &str) -> bool {
    !n.is_empty() && n.len() <= 60 && n.starts_with(|c: char| c.is_alphabetic()) && n.chars().all(|c| c.is_alphanumeric() || c == '_')
}
fn set(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Set = serde_json::from_value(p.clone()).map_err(|e| err(e.to_string()))?;
    let t = p.table;
    if !identifier(&t.name) || t.columns.is_empty() || t.columns.len() > 32 || t.rows.is_empty() || t.rows.len() > 128 {
        return Err(err("family requires a name, 1..32 columns and 1..128 rows"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for c in &t.columns {
        if !identifier(c) || !seen.insert(c) {
            return Err(err("column names must be unique identifiers"));
        }
    }
    for (name, row) in &t.rows {
        if name.is_empty() || name.len() > 100 || row.len() != t.columns.len() {
            return Err(err("invalid family row"));
        }
        for c in &t.columns {
            let e = row.get(c).ok_or_else(|| err("missing column"))?;
            if e.is_empty() || e.len() > 512 || e.chars().any(char::is_control) {
                return Err(err("invalid cell expression"));
            }
        }
    }
    let mut state = s.doc.metadata.get(KEY).cloned().unwrap_or_else(|| json!({"tables":{}}));
    if state["tables"].as_object().map_or(0, |m| m.len()) >= 32 && state["tables"].get(&t.name).is_none() {
        return Err(err("at most 32 families"));
    }
    let active = state["tables"][&t.name]["active"].as_str().filter(|n| t.rows.contains_key(*n)).map(str::to_owned);
    state["tables"][&t.name] = json!({"definition":t,"active":active});
    s.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY, state)?;
    Ok(json!({"name":t.name,"active":active}))
}
fn select(s: &mut Session, p: &Value) -> Result<Value> {
    let p: Select = serde_json::from_value(p.clone()).map_err(|e| err(e.to_string()))?;
    let mut state = s.doc.metadata.get(KEY).cloned().ok_or_else(|| err("no families"))?;
    let t: Table = serde_json::from_value(state["tables"][&p.table]["definition"].clone()).map_err(|e| err(e.to_string()))?;
    let row = t.rows.get(&p.row).ok_or_else(|| err("unknown row"))?;
    let mut next = s.scratch();
    next.execute("worksheet.family.set", &json!({"table":t}))?;
    let source = next.doc.metadata.get(super::worksheet_references::CALCPAD).and_then(|c| c["source"].as_str()).unwrap_or("");
    let names: Vec<_> = t.columns.iter().map(|c| format!("{}_{}", t.name, c)).collect();
    let mut lines: Vec<_> = source
        .lines()
        .filter(|line| !line.split_once('=').is_some_and(|(name, _)| names.iter().any(|n| n == name.trim())))
        .map(str::to_owned)
        .collect();
    // Insert before dependent expressions, preserving all unrelated definitions.
    let mut definitions = Vec::new();
    for (name, c) in names.iter().zip(&t.columns) {
        definitions.push(format!("{name} = {}", row[c]));
    }
    definitions.append(&mut lines);
    next.execute("worksheet.calcpad.edit", &json!({"source":definitions.join("\n")}))?;
    next.execute("worksheet.calcpad.calculate", &json!({}))?;
    let results =
        next.doc.metadata.get(super::worksheet_references::CALCPAD).and_then(|c| c.get("results")).ok_or_else(|| err("no scalar results"))?;
    if names.iter().any(|n| !results[n]["value"].as_f64().is_some_and(f64::is_finite)) {
        return Err(err("every family column must evaluate to a finite real scalar"));
    }

    state["tables"][&p.table]["active"] = json!(p.row);
    next.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY, state)?;
    super::worksheet::commit(s, next)?;
    Ok(json!({"name":p.table,"active":p.row}))
}
