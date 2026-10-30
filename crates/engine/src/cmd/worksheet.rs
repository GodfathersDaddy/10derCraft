//! A project calculation document sharing the CAD parameter graph, not a second evaluator.
//! Native `.scmath` files and embedded project definitions use the same named expressions.
use super::CommandSpec;
use crate::params::{bad, str_};
use crate::{Result, Session};
use serde_json::{Value, json};

const KEY: &str = "org.solidcraft.worksheet";
const FORMAT: &str = "solidcraft-math/1";
const MAX_BYTES: usize = 4 * 1024 * 1024;

pub(super) const PRESENTATION: &str = "org.solidcraft.engineering-presentation";
#[derive(serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct PresentationParams {
    workspace: String,
    #[serde(default)]
    layout: Option<Value>,
}
#[derive(serde::Serialize, schemars::JsonSchema)]
struct PresentationOutput {
    workspace: String,
    layout: Value,
}
const PRESENTATION_CONTRACT: crate::schema::TypedContract<PresentationParams> = crate::schema::TypedContract::new();
fn presentation(s: &mut Session, p: &Value) -> Result<Value> {
    let workspace = str_(p, "workspace").unwrap_or("");
    if !["mks", "worksheet"].contains(&workspace) {
        return Err(bad("worksheet.presentation", "workspace must be mks or worksheet"));
    }
    let mut all = s.doc.metadata.get(PRESENTATION).cloned().unwrap_or_else(|| json!({}));
    if !all.is_object() {
        return Err(bad("worksheet.presentation", "invalid saved presentation metadata"));
    }
    if let Some(layout) = p.get("layout") {
        if !layout.is_object() || serde_json::to_vec(layout).map_or(true, |v| v.len() > 128 * 1024) {
            return Err(bad("worksheet.presentation", "layout must be an object of at most 128 KiB"));
        }
        if let Some(positions) = layout.get("positions") {
            let valid = positions.as_object().is_some_and(|m| {
                m.len() <= 2048
                    && m.iter().all(|(key, v)| {
                        key.len() <= 256
                            && v.as_array().is_some_and(|a| {
                                a.len() == 2 && a.iter().all(|v| v.as_f64().is_some_and(|v| v.is_finite() && (0.0..=20000.0).contains(&v)))
                            })
                    })
            });
            if !valid {
                return Err(bad("worksheet.presentation", "positions require at most 2048 finite coordinate pairs in 0..20000"));
            }
        }
        for (key, limit) in [("notes", 100), ("charts", 20)] {
            if let Some(values) = layout.get(key)
                && !values.as_array().is_some_and(|a| a.len() <= limit && a.iter().all(|v| v.as_str().is_some_and(|s| s.len() <= 10000)))
            {
                return Err(bad("worksheet.presentation", "invalid worksheet regions"));
            }
        }
        for key in ["sensors", "unscoped_sensors"] {
            if let Some(values) = layout.get(key)
                && !values.as_array().is_some_and(|a| a.len() <= 128 && a.iter().all(|v| v.as_u64().is_some()))
            {
                return Err(bad("worksheet.presentation", "invalid sensor selection"));
            }
        }
        all[workspace] = layout.clone();
        s.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, PRESENTATION, all.clone())?;
    }
    Ok(json!({"workspace":workspace,"layout":all[workspace]}))
}

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("worksheet.presentation", "Arrange Engineering Workspace", crate::checked!("worksheet.presentation", presentation)).icon("settings")
        .contract(&PRESENTATION_CONTRACT).output(crate::schema::output_schema_for::<PresentationOutput>).doc_only(),
    CommandSpec::new("worksheet.calcpad.edit", "Edit Local Calculation", super::worksheet_calcpad::edit).icon("params").doc_only(),
    CommandSpec::new("worksheet.calcpad.calculate", "Calculate Locally", super::worksheet_calcpad::calculate).icon("params").doc_only(),
    CommandSpec::new("worksheet.variant.save", "Save Calculation Variant", super::worksheet_variants::save).icon("params").doc_only(),
    CommandSpec::new("worksheet.variant.select", "Select Calculation Variant", super::worksheet_variants::select).icon("params"),
    CommandSpec::new("worksheet.reference", "Publish Calculation Reference", super::worksheet_references::reference).icon("params")
        .params("source: result name, parameter?: CAD alias (default ref_<source>), enabled?: bool; captures the initial value; subsequent changes need worksheet.apply"),
    CommandSpec::new("worksheet.references", "Review Calculation Changes", super::worksheet_references::read).icon("params").noundo().doc_only(),
    CommandSpec::new("worksheet.apply", "Update Construction", super::worksheet_references::apply).icon("params")
        .params("Atomically apply all changed published calculation results to their CAD parameters; refuse invalid results or broken features"),
    CommandSpec::new("worksheet.open", "Open Calculation Document", open).icon("params").noundo().doc_only(),
    CommandSpec::new("worksheet.read", "Read Calculation Document", read).icon("params").noundo().doc_only(),
    CommandSpec::new("worksheet.set", "Define Calculation", set).icon("params")
        .params("name, expression, unit?, comment?; defines or changes a shared user parameter; CAD and sketch expressions follow atomically"),
    CommandSpec::new("worksheet.describe", "Describe Calculation Document", describe).icon("params").doc_only()
        .params("title?, notes?; saved inside the current project"),
    CommandSpec::new("worksheet.save", "Save Calculation File", save).icon("params").noundo().doc_only()
        .params("path?: .scmath JSON; returns the file document too; no external scripts are executed"),
    CommandSpec::new("worksheet.load", "Load Calculation File", load).icon("params")
        .params("path | document: solidcraft-math/1; merges named user definitions and recomputes CAD, all or nothing; model parameters cannot be overwritten"),
];

fn open(_: &mut Session, _: &Value) -> Result<Value> {
    Ok(json!({"tool":"worksheet","dialog_opened":false}))
}

fn description(s: &Session) -> Value {
    s.doc.metadata.get(KEY).cloned().unwrap_or_else(|| json!({"title":"Berechnung","notes":""}))
}

pub(super) fn document(s: &Session) -> Value {
    let aliases = super::worksheet_references::bindings(s);
    let definitions: Vec<Value> = s
        .doc
        .params
        .iter()
        .filter(|p| !p.model && !aliases.iter().any(|r| r["parameter"] == p.name))
        .map(|p| json!({"name":p.name,"expression":p.expr,"unit":p.unit,"comment":p.comment}))
        .collect();
    let meta = description(s);
    json!({"families":s.doc.metadata.get(super::worksheet_families::KEY),"format":FORMAT,"title":meta["title"],"notes":meta["notes"],"definitions":definitions,"calcpad":s.doc.metadata.get(super::worksheet_references::CALCPAD),"variants":super::worksheet_variants::state(s),"presentation":s.doc.metadata.get(PRESENTATION).and_then(|v|v.get("worksheet"))})
}

fn read(s: &mut Session, _: &Value) -> Result<Value> {
    let raw = s.doc.metadata.get(super::worksheet_references::CALCPAD);
    let display_current = raw.is_some_and(|v| {
        v["dirty"] != true
            && super::worksheet_inline::fingerprint(&v["results"]).is_ok_and(|hash| super::worksheet_inline::layout(s)["unit_values_source"] == hash)
    });
    let references = super::worksheet_references::state(s);
    let aliases: Vec<_> = super::worksheet_references::bindings(s).into_iter().filter_map(|v| v["parameter"].as_str().map(str::to_owned)).collect();
    let rows: Vec<_> = s.doc.param_rows().into_iter().filter(|r| r.source == "user" && !aliases.contains(&r.name)).collect();
    Ok(json!({"document":document(s),"rows":rows,"shared_cad_graph":true,
        "cad_references":references,"calcpad":s.doc.metadata.get(super::worksheet_references::CALCPAD),"display_units_current":display_current,"variants":super::worksheet_variants::state(s),
        "planetary":s.doc.metadata.get(super::planetary::KEY),
        "families":s.doc.metadata.get(super::worksheet_families::KEY),
        "material_tables":serde_json::from_str::<Value>(include_str!("../../../../assets/data/worksheet-materials.json")).unwrap_or(Value::Null),
        "units":"native CAD references: length, angle and dimensionless; local Calcpad: physical units; output-unit factors preserve dimensions"}))
}

pub(super) fn commit(s: &mut Session, next: Session) -> Result<Value> {
    let previous: std::collections::BTreeSet<_> = s.model.results.iter().filter_map(|r| r.error.as_ref().map(|e| (r.id, e.clone()))).collect();
    if let Some(r) = next.model.results.iter().find(|r| r.error.as_ref().is_some_and(|e| !previous.contains(&(r.id, e.clone())))) {
        return Err(bad("worksheet.set", format!("CAD update refused: {}: {}", r.name, r.error.as_deref().unwrap_or("invalid feature"))));
    }
    s.doc = next.doc;
    read(s, &json!({}))
}

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_(p, "name").ok_or_else(|| bad("worksheet.set", "name is required"))?;
    if model_name(s, name) || super::worksheet_references::bindings(s).iter().any(|r| r["parameter"] == name) {
        return Err(bad("worksheet.set", "a calculation cannot overwrite a model dimension; reference it by name instead"));
    }
    let mut next = s.scratch();
    let command = if next.doc.param(name).is_some() { "parameters.change" } else { "parameters.add" };
    next.execute(command, p)?;
    commit(s, next)
}

fn model_name(s: &Session, name: &str) -> bool {
    s.doc.param(name).is_some_and(|q| q.model) || s.doc.features.iter().any(|f| f.param_names.iter().any(|n| n == name))
}

fn describe(s: &mut Session, p: &Value) -> Result<Value> {
    let old = description(s);
    let title = str_(p, "title").or_else(|| old["title"].as_str()).unwrap_or("Berechnung");
    let notes = str_(p, "notes").or_else(|| old["notes"].as_str()).unwrap_or("");
    if title.len() > 256 || notes.len() > 16 * 1024 {
        return Err(bad("worksheet.describe", "title or notes too long"));
    }
    s.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, KEY, json!({"title":title,"notes":notes}))?;
    read(s, &json!({}))
}

fn save(s: &mut Session, p: &Value) -> Result<Value> {
    let file = document(s);
    if let Some(path) = str_(p, "path") {
        let text = serde_json::to_vec_pretty(&file).map_err(|e| bad("worksheet.save", e.to_string()))?;
        solidcraft_io::vfs::write(path, &text).map_err(|e| bad("worksheet.save", e.to_string()))?;
    }
    Ok(json!({"document":file,"path":p.get("path")}))
}

fn load(s: &mut Session, p: &Value) -> Result<Value> {
    let file = if let Some(path) = str_(p, "path") {
        let text = solidcraft_io::vfs::read_to_string(path, MAX_BYTES).map_err(|e| bad("worksheet.load", e.to_string()))?;
        serde_json::from_str::<Value>(&text).map_err(|e| bad("worksheet.load", e.to_string()))?
    } else {
        p.get("document").cloned().ok_or_else(|| bad("worksheet.load", "path or document required"))?
    };
    if file["format"] != FORMAT {
        return Err(bad("worksheet.load", "unsupported math document format"));
    }
    let rows = file["definitions"]
        .as_array()
        .filter(|r| r.len() <= 1000)
        .ok_or_else(|| bad("worksheet.load", "definitions must list at most 1000 calculations"))?;
    if rows
        .iter()
        .any(|r| r["name"].as_str().is_some_and(|n| model_name(s, n) || super::worksheet_references::bindings(s).iter().any(|r| r["parameter"] == n)))
    {
        return Err(bad("worksheet.load", "cannot overwrite a model dimension"));
    }
    let text = serde_json::to_string(rows).map_err(|e| bad("worksheet.load", e.to_string()))?;
    let mut next = s.scratch();
    next.execute("parameters.import", &json!({"text":text}))?;
    let (_, errors) = next.doc.param_values();
    if !errors.is_empty() {
        return Err(bad("worksheet.load", format!("invalid calculations: {errors:?}")));
    }
    describe(&mut next, &file)?;
    if let Some(source) = file["calcpad"]["source"].as_str() {
        super::worksheet_calcpad::edit(&mut next, &json!({"source":source}))?;
    }
    if file["variants"].is_object() {
        let variants = &file["variants"];
        if variants["items"].as_array().is_none_or(|r| r.len() > 100) {
            return Err(bad("worksheet.load", "invalid variants"));
        }
        next.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, super::worksheet_variants::KEY, variants.clone())?;
    }
    if let Some(tables) = file["families"]["tables"].as_object() {
        if tables.len() > 32 {
            return Err(bad("worksheet.load", "at most 32 families"));
        }
        for (name, table) in tables {
            if table["definition"]["name"] != *name {
                return Err(bad("worksheet.load", "family name mismatch"));
            }
            next.execute("worksheet.family.set", &json!({"table":table["definition"]}))?;
            if let Some(active) = table["active"].as_str() {
                if table["definition"]["rows"].get(active).is_none() {
                    return Err(bad("worksheet.load", "unknown active family row"));
                }
                let mut families = next.doc.metadata.get(super::worksheet_families::KEY).cloned().unwrap_or_else(|| json!({}));
                families["tables"][name]["active"] = json!(active);
                next.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, super::worksheet_families::KEY, families)?;
            }
        }
    }
    if file["presentation"].is_object() {
        presentation(&mut next, &json!({"workspace":"worksheet","layout":file["presentation"]}))?;
    }
    commit(s, next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_results_wait_for_explicit_apply_and_variant_switches_are_reviewable() {
        let mut s = Session::default();
        s.execute("worksheet.set", &json!({"name":"width","expression":"20 mm","unit":"mm"})).unwrap();
        s.execute("worksheet.set", &json!({"name":"depth","expression":"width / 2","unit":"mm"})).unwrap();
        s.execute("worksheet.reference", &json!({"source":"depth"})).unwrap();
        s.execute("solid.box", &json!({"length":"ref_depth","width":"5 mm","height":"3 mm","body_name":"VariantPlate"})).unwrap();
        s.execute("worksheet.variant.save", &json!({"name":"Small"})).unwrap();
        s.execute("worksheet.set", &json!({"name":"width","expression":"40 mm"})).unwrap();
        s.execute("worksheet.variant.save", &json!({"name":"Large"})).unwrap();
        assert_eq!(s.doc.param_values().0["ref_depth"].v, 10.);
        assert_eq!(s.execute("worksheet.references", &json!({})).unwrap()["pending"], 1);
        assert!(s.execute("worksheet.set", &json!({"name":"ref_depth","expression":"90 mm"})).is_err());
        s.execute("worksheet.apply", &json!({})).unwrap();
        assert_eq!(s.doc.param_values().0["ref_depth"].v, 20.);
        s.execute("worksheet.variant.select", &json!({"name":"Small"})).unwrap();
        assert_eq!(s.doc.param_values().0["depth"].v, 10.);
        assert_eq!(s.doc.param_values().0["ref_depth"].v, 20.);
        assert_eq!(s.execute("worksheet.references", &json!({})).unwrap()["pending"], 1);
        let reopened = solidcraft_doc::Document::from_json(&s.doc.to_json()).unwrap();
        assert_eq!(reopened.param_values().0["ref_depth"].v, 20.);
        assert_eq!(reopened.metadata.get(super::super::worksheet_variants::KEY).unwrap()["active"], "Small");
        s.execute("worksheet.apply", &json!({})).unwrap();
        let bounds = s.model.state().body("VariantPlate").unwrap().mesh().bounds();
        assert!((bounds.max.x - bounds.min.x - 10.).abs() < 1e-6);
        assert!(s.execute("worksheet.reference", &json!({"source":"depth","enabled":false})).is_err());
        let exported = s.execute("worksheet.save", &json!({})).unwrap();
        assert!(!exported["document"]["definitions"].as_array().unwrap().iter().any(|r| r["name"] == "ref_depth"));
    }

    #[test]
    fn changed_calcpad_source_blocks_stale_reference_updates() {
        let mut s = Session::default();
        s.doc_mut()
            .metadata
            .set(
                solidcraft_doc::metadata::Owner::Design,
                super::super::worksheet_references::CALCPAD,
                json!({"results":{"b":{"value":2.,"unit":"cm"}},"dirty":false}),
            )
            .unwrap();
        s.execute("worksheet.reference", &json!({"source":"b"})).unwrap();
        assert_eq!(s.doc.param_values().0["ref_b"].v, 20.);
        s.execute("worksheet.calcpad.edit", &json!({"source":"b = 3cm"})).unwrap();
        assert!(s.execute("worksheet.apply", &json!({})).is_err());
        assert_eq!(s.doc.param_values().0["ref_b"].v, 20.);
    }

    #[test]
    fn calculations_drive_unrelated_cad_and_roundtrip_without_a_second_graph() {
        let mut s = Session::default();
        s.execute("worksheet.set", &json!({"name":"span","expression":"30 mm","unit":"mm"})).unwrap();
        s.execute("worksheet.set", &json!({"name":"half_span","expression":"span / 2","unit":"mm"})).unwrap();
        s.execute("solid.box", &json!({"length":"span","width":"half_span","height":"5 mm","body_name":"Plate"})).unwrap();
        let before = s.doc.clone();
        s.execute("worksheet.set", &json!({"name":"span","expression":"50 mm"})).unwrap();
        assert_eq!(s.doc.param_values().0["half_span"].v, 25.);
        let bounds = s.model.state().body("Plate").unwrap().mesh().bounds();
        assert!((bounds.max.x - bounds.min.x - 50.).abs() < 1e-6);
        assert!((bounds.max.y - bounds.min.y - 25.).abs() < 1e-6);
        assert!(!std::sync::Arc::ptr_eq(&before, &s.doc));
        let good = s.doc.clone();
        assert!(s.execute("worksheet.set", &json!({"name":"span","expression":"half_span"})).is_err());
        assert!(std::sync::Arc::ptr_eq(&good, &s.doc));
        let exported = s.execute("worksheet.save", &json!({})).unwrap();
        let mut other = Session::default();
        other.execute("worksheet.load", &json!({"document":exported["document"]})).unwrap();
        assert_eq!(other.doc.param_values().0["half_span"].v, 25.);
        let reopened = solidcraft_doc::Document::from_json(&s.doc.to_json()).unwrap();
        assert_eq!(reopened.param_values().0["half_span"].v, 25.);
    }

    #[test]
    fn calculation_names_drive_sketch_coordinates_and_reject_broken_geometry_atomically() {
        let mut s = Session::default();
        s.execute(
            "worksheet.load",
            &json!({"document":{"format":FORMAT,"title":"Plate calculation","definitions":[
                {"name":"depth","expression":"width / 2","unit":"mm"},
                {"name":"width","expression":"20 mm","unit":"mm"}
            ]}}),
        )
        .unwrap();
        s.execute("sketch.create", &json!({"plane":"XY"})).unwrap();
        s.execute("sketch.line", &json!({"points":[[0,0],["width",0],["width","depth"],[0,"depth"]],"closed":true})).unwrap();
        s.execute("sketch.finish", &json!({})).unwrap();
        s.execute("solid.extrude", &json!({"distance":"5 mm","body_name":"SketchPlate"})).unwrap();
        s.execute("worksheet.set", &json!({"name":"width","expression":"40 mm"})).unwrap();
        let bounds = s.model.state().body("SketchPlate").unwrap().mesh().bounds();
        assert!((bounds.max.x - bounds.min.x - 40.).abs() < 1e-6);
        assert!((bounds.max.y - bounds.min.y - 20.).abs() < 1e-6);
        let good = s.doc.clone();
        assert!(s.execute("worksheet.set", &json!({"name":"width","expression":"-10 mm"})).is_ok());
        s.doc = good;
        s.refresh();
        assert!(s.execute("worksheet.set", &json!({"name":"width","expression":"0 mm"})).is_err());
        assert_eq!(s.doc.param_values().0["width"].v, 40.);
    }
}
