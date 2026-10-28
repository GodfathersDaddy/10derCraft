//! Workspace-shipped project catalogue; no solver, server or document edits.
use super::CommandSpec;
use crate::{Result, Session, params::bad};
use serde::Deserialize;
use serde_json::{Value, json};
pub static COMMANDS: &[CommandSpec] = &[CommandSpec::new("suite.examples", "List 10derCraft Examples", list)
    .icon("library")
    .noundo()
    .doc_only()
    .params("Discover the shipped 10derCraft.examples.json beside this workspace; returns existing project paths for the native Start page")];
#[derive(Deserialize)]
struct Catalogue {
    schema: String,
    examples: Vec<Example>,
}
#[derive(Deserialize)]
struct Example {
    id: String,
    title: String,
    path: String,
    description: String,
}
#[cfg(target_arch = "wasm32")]
fn list(_: &mut Session, _: &Value) -> Result<Value> {
    Ok(json!({"examples":[],"desktop_catalogue":true}))
}
#[cfg(not(target_arch = "wasm32"))]
fn list(_: &mut Session, _: &Value) -> Result<Value> {
    use std::path::{Component, Path};
    let binary = std::env::current_exe().map_err(|e| bad("suite.examples", e.to_string()))?;
    let Some(catalogue) = binary.ancestors().map(|p| p.join("10derCraft.examples.json")).find(|p| p.is_file()) else {
        return Ok(json!({"examples":[]}));
    };
    let root = catalogue.parent().ok_or_else(|| bad("suite.examples", "catalogue root missing"))?;
    let source = solidcraft_io::vfs::read_to_string(&catalogue.to_string_lossy(), 64 * 1024).map_err(|e| bad("suite.examples", e.to_string()))?;
    let c: Catalogue = serde_json::from_str(&source).map_err(|e| bad("suite.examples", e.to_string()))?;
    if c.schema != "10dercraft.examples/1" || c.examples.len() > 32 {
        return Err(bad("suite.examples", "invalid catalogue schema/size"));
    }
    let root = root.canonicalize().map_err(|e| bad("suite.examples", e.to_string()))?;
    let mut examples = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for e in c.examples {
        if e.id.is_empty()
            || e.id.len() > 64
            || !e.id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || !ids.insert(e.id.clone())
            || e.title.len() > 200
            || e.description.len() > 1000
            || e.path.len() > 1024
        {
            return Err(bad("suite.examples", "invalid example entry"));
        }
        let relative = Path::new(&e.path);
        if relative.components().any(|c| !matches!(c, Component::Normal(_))) || !relative.extension().is_some_and(|e| e == "solidcraft" || e == "sc")
        {
            return Err(bad("suite.examples", "example must name a relative native project path"));
        }
        let path = root.join(relative);
        let present = match path.canonicalize() {
            Ok(p) => p.starts_with(&root) && p.is_file(),
            Err(_) => false,
        };
        examples.push(json!({"id":e.id,"title":e.title,"description":e.description,"path":path.to_string_lossy(),"present":present}));
    }
    Ok(json!({"examples":examples,"catalogue":catalogue.to_string_lossy()}))
}
