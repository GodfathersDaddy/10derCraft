//! Named calculation alternatives embedded in the same CAD project.
use crate::params::{bad, str_};
use crate::{Result, Session};
use serde_json::{Value, json};
pub const KEY: &str = "org.solidcraft.calculation-variants";

pub fn state(s: &Session) -> Value {
    s.doc.metadata.get(KEY).cloned().unwrap_or_else(|| json!({"active":"","items":[]}))
}

pub fn save(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_(p, "name")
        .map(str::trim)
        .filter(|n| !n.is_empty() && n.len() <= 100)
        .ok_or_else(|| bad("worksheet.variant.save", "name required (at most 100 bytes)"))?;
    let mut variants = state(s);
    let mut items = variants["items"].as_array().cloned().unwrap_or_default();
    let sheet = super::worksheet::document(s);
    let calcpad = sheet["calcpad"]["source"].as_str().map(|source| json!({"source":source}));
    let item = json!({"name":name,"definitions":sheet["definitions"],"calcpad":calcpad});
    if let Some(old) = items.iter_mut().find(|v| v["name"] == name) {
        *old = item;
    } else {
        if items.len() >= 100 {
            return Err(bad("worksheet.variant.save", "at most 100 variants"));
        }
        items.push(item);
    }
    variants = json!({"active":name,"items":items});
    s.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY, variants.clone())?;
    Ok(variants)
}

pub fn select(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_(p, "name").ok_or_else(|| bad("worksheet.variant.select", "name required"))?;
    let mut variants = state(s);
    let item = variants["items"]
        .as_array()
        .and_then(|r| r.iter().find(|v| v["name"] == name))
        .ok_or_else(|| bad("worksheet.variant.select", "unknown variant"))?
        .clone();
    let mut next = s.scratch();
    next.execute("worksheet.load", &json!({"document":{"format":"solidcraft-math/1","definitions":item["definitions"]}}))?;
    if let Some(source) = item["calcpad"]["source"].as_str() {
        super::worksheet_calcpad::calculate(&mut next, &json!({"source":source}))?;
    }
    variants["active"] = json!(name);
    next.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY, variants)?;
    super::worksheet::commit(s, next)
}
