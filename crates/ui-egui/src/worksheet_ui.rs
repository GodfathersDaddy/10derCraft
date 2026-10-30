//! Native calculation document. Every editable definition is a shared CAD parameter.
#[path = "worksheet_canvas_ui.rs"]
mod canvas;
use crate::{SolveApp, scenario, theme::Tokens};
use egui::{RichText, vec2};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, Default)]
struct Draft {
    expressions: BTreeMap<String, String>,
    name: String,
    expression: String,
    unit: String,
    comment: String,
    path: String,
    error: String,
    source: Option<String>,
    variant_name: String,
    inputs: BTreeMap<String, String>,
}

fn run(app: &mut SolveApp, draft: &mut Draft, command: &str, params: Value) -> bool {
    let result = if matches!(command, "worksheet.apply" | "worksheet.set" | "worksheet.variant.select") {
        app.run_heavy(command, params)
    } else {
        app.run(command, params)
    };
    match result {
        Ok(_) => {
            draft.error.clear();
            true
        }
        Err(error) => {
            draft.error = error;
            false
        }
    }
}

pub fn show(app: &mut SolveApp, ui: &mut egui::Ui) {
    if canvas::show(app, ui) {
        return;
    }
    let id = egui::Id::new(("calculation_document", app.docs.active, app.session.path.clone()));
    let mut draft = ui.data_mut(|d| d.get_temp::<Draft>(id)).unwrap_or_default();
    if draft.path.is_empty() {
        draft.path = app
            .session
            .path
            .as_ref()
            .map(|p| std::path::Path::new(p).with_extension("scmath").to_string_lossy().to_string())
            .unwrap_or_else(|| "Berechnung.scmath".into());
    }
    let snapshot = app.session.execute("worksheet.read", &json!({})).unwrap_or_default();
    let title = snapshot["document"]["title"].as_str().unwrap_or("Berechnung");
    crate::ui_kit::FileView::page(ui, |ui| {
        egui::ScrollArea::vertical().id_salt("worksheet_page_scroll").auto_shrink([false, false]).show(ui, |ui| {
        ui.heading(title);
        variants(app, ui, &snapshot, &mut draft);
        if !snapshot["planetary"].is_null() { planetary_inputs(app,ui,&mut draft); }
        let snapshot = app.session.execute("worksheet.read", &json!({})).unwrap_or_default();
        local_calcpad(app, ui, &snapshot, &mut draft);
        ui.label("Berechnungsergebnisse als benannte Referenzen in Skizzen und CAD verwenden.");
        ui.add_space(10.);
        ui.horizontal_wrapped(|ui| {
            ui.label("Mathedatei");
            let field = ui.add(egui::TextEdit::singleline(&mut draft.path).desired_width(390.));
            scenario::publish_handle("worksheet:path", field.rect.center());
            if ui.button("Speichern (.scmath)").clicked() {
                let path = draft.path.clone();
                run(app, &mut draft, "worksheet.save", json!({"path":path}));
            }
            if ui.button("Berechnung laden").clicked() {
                let path = draft.path.clone();
                if run(app, &mut draft, "worksheet.load", json!({"path":path})) { draft.expressions.clear(); draft.source = None; }
            }
        });
        ui.label(RichText::new("Die Berechnung wird zusätzlich mit dem CAD-Projekt gespeichert. Änderungen mit Enter oder beim Verlassen eines Formelfeldes übernehmen.").small());
        if !draft.error.is_empty() { ui.colored_label(Tokens::get().text, format!("Nicht übernommen: {}", draft.error)); }
        ui.separator();
        egui::ScrollArea::both().id_salt("worksheet_reference_scroll").max_height(320.).auto_shrink([false, true]).show(ui, |ui| {
            ui.collapsing("Native CAD-Formeln", |ui| {
            egui::Grid::new("calculation_rows").num_columns(6).spacing(vec2(18., 12.)).striped(true).show(ui, |ui| {
                for heading in ["Name", "Definition :=", "Ergebnis", "Einheit", "Beschreibung"] { ui.strong(heading); }
                ui.strong("CAD-Referenz"); ui.end_row();
                if let Some(rows) = snapshot["rows"].as_array() {
                    for row in rows {
                        let name = row["name"].as_str().unwrap_or("");
                        let expression = row["expression"].as_str().unwrap_or("");
                        ui.monospace(name);
                        let value = draft.expressions.entry(name.into()).or_insert_with(|| expression.into());
                        let response = ui.add_sized([310., 23.], egui::TextEdit::singleline(value).id_salt(("definition", name)).desired_width(310.));
                        scenario::publish_handle(&format!("worksheet:expr:{name}"), response.rect.center());
                        let edited = value.clone();
                        let commit = (response.lost_focus() || response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) && edited != expression;
                        if commit && run(app, &mut draft, "worksheet.set", json!({"name":name,"expression":edited})) {
                            draft.expressions.remove(name);
                        }
                        ui.monospace(row["value"].as_f64().map(|v| format!("{v:.6}")).unwrap_or_else(|| "Fehler".into()));
                        ui.label(row["unit"].as_str().unwrap_or(""));
                        ui.label(row["comment"].as_str().unwrap_or(""));
                        reference(app, ui, &snapshot, &mut draft, name);
                        ui.end_row();
                    }
                }
            });
            ui.add_space(20.);
            ui.strong("Neue Berechnung");
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::TextEdit::singleline(&mut draft.name).hint_text("Name, z. B. Breite").desired_width(155.));
                ui.label(":=");
                ui.add(egui::TextEdit::singleline(&mut draft.expression).hint_text("Laenge / 2").desired_width(255.));
                ui.add(egui::TextEdit::singleline(&mut draft.unit).hint_text("mm / deg / leer").desired_width(120.));
                ui.add(egui::TextEdit::singleline(&mut draft.comment).hint_text("Beschreibung").desired_width(220.));
                let add = ui.button("Definieren");
                scenario::publish_handle("worksheet:add", add.rect.center());
                if add.clicked() {
                    let mut params = json!({"name":draft.name,"expression":draft.expression,"unit":draft.unit,"comment":draft.comment});
                    if draft.unit.trim().is_empty() && let Some(object) = params.as_object_mut() {
                        object.remove("unit");
                    }
                    if run(app, &mut draft, "worksheet.set", params) {
                        draft.name.clear(); draft.expression.clear(); draft.comment.clear();
                    }
                }
            });
            ui.add_space(20.);
            ui.label("Verwendung im CAD: den Namen direkt als Maß eingeben, z. B. Breite oder 2 * Breite + 5 mm.");
            ui.label("Unterstützt: Skalare, Längen, Winkel, Funktionen und benannte Abhängigkeiten. Physikalische Verbundeinheiten und Gleichungssysteme sind noch nicht enthalten.");
            });
            review(app, ui, &snapshot["cad_references"], &mut draft);
        });
        });
    });
    ui.data_mut(|d| d.insert_temp(id, draft));
}

/// The Start page shows actual project variants when a calculation/design exists.
pub fn project_variants(app: &mut SolveApp, ui: &mut egui::Ui) -> bool {
    let snapshot = app.session.execute("worksheet.read", &json!({})).unwrap_or_default();
    let has_variants = snapshot["variants"]["items"].as_array().is_some_and(|a| !a.is_empty());
    let has_gears = app.session.doc.metadata.get("org.solidcraft.gear-designs").and_then(|v| v["items"].as_array()).is_some_and(|a| !a.is_empty());
    if !has_variants && !has_gears {
        return false;
    }
    let id = egui::Id::new(("calculation_document", app.docs.active, app.session.path.clone()));
    let mut draft = ui.data_mut(|d| d.get_temp::<Draft>(id)).unwrap_or_default();
    ui.heading("Projektvarianten");
    variants(app, ui, &snapshot, &mut draft);
    let state = app.session.execute("worksheet.references", &json!({})).unwrap_or_default();
    review(app, ui, &state, &mut draft);
    if !draft.error.is_empty() {
        ui.label(&draft.error);
    }
    ui.data_mut(|d| d.insert_temp(id, draft));
    if has_gears {
        ui.separator();
        ui.strong("Gespeicherte Zahnradentwürfe");
        let cache = ui.id().with("gear_review_cache");
        let cached = ui.data_mut(|d| d.get_temp::<(u64, Value)>(cache));
        let report = if let Some((revision, report)) = cached.filter(|(revision, _)| *revision == app.session.revision) {
            let _ = revision;
            report
        } else {
            let report = app.session.execute("gear.review", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}));
            ui.data_mut(|d| d.insert_temp(cache, (app.session.revision, report.clone())));
            report
        };
        if let Some(items) = report["items"].as_array() {
            for item in items {
                let status = match item["status"].as_str().unwrap_or("") {
                    "current" => "Berechnung aktuell",
                    "outdated" => "Berechnung geändert",
                    _ => "Berechnung ungültig",
                };
                let cad = match item["feature_status"].as_str().unwrap_or("") {
                    "current" => "CAD aktuell",
                    "unlinked" => "ohne CAD-Körper",
                    "different_definition" => "CAD-Definition abweichend",
                    "missing" => "CAD-Verknüpfung fehlt",
                    _ => "CAD prüfen",
                };
                ui.label(format!("{} — {status}; {cad}", item["name"].as_str().unwrap_or("Entwurf")));
            }
        } else if let Some(error) = report["error"].as_str() {
            ui.label(error);
        }
        let open = ui.button("Zahnradentwürfe öffnen / vergleichen");
        scenario::publish_handle("suite:gear-designs", open.rect.center());
        if open.clicked() {
            app.ui.engineering_dialog = None;
            app.ui.project_page = "cad".into();
            app.start("gear.design");
        }
        ui.small("Zahnradentwürfe vergleichen gespeicherte Auslegungen. Projektvarianten wechseln das lokale Berechnungsblatt; CAD wird erst nach Übernahme geändert.");
    }
    true
}
fn variants(app: &mut SolveApp, ui: &mut egui::Ui, snapshot: &Value, draft: &mut Draft) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Variante");
        let active = snapshot["variants"]["active"].as_str().unwrap_or("");
        let mut selected = active.to_owned();
        let combo = egui::ComboBox::from_id_salt("worksheet_variant")
            .selected_text(if active.is_empty() { "Aktuelle Berechnung" } else { active })
            .show_ui(ui, |ui| {
                if let Some(items) = snapshot["variants"]["items"].as_array() {
                    for item in items {
                        if let Some(name) = item["name"].as_str() {
                            let option = ui.selectable_value(&mut selected, name.to_owned(), name);
                            scenario::publish_handle(&format!("worksheet:variant:option:{name}"), option.rect.center());
                        }
                    }
                }
            });
        scenario::publish_handle("worksheet:variant", combo.response.rect.center());
        if selected != active && run(app, draft, "worksheet.variant.select", json!({"name":selected})) {
            draft.expressions.clear();
            draft.source = None;
            draft.inputs.clear();
        }
        ui.add(egui::TextEdit::singleline(&mut draft.variant_name).hint_text("Name einer Variante").desired_width(170.));
        let save = ui.button("Als Variante speichern");
        scenario::publish_handle("worksheet:variant:save", save.rect.center());
        if save.clicked() {
            let name = draft.variant_name.clone();
            run(app, draft, "worksheet.variant.save", json!({"name":name}));
        }
    });
    ui.label(RichText::new("Auswahl lädt und berechnet die Variante. CAD-Referenzen werden mit „Konstruktion aktualisieren“ übernommen. Änderungen zuerst als Variante speichern.").small());
}

fn reference(app: &mut SolveApp, ui: &mut egui::Ui, snapshot: &Value, draft: &mut Draft, name: &str) {
    let binding = snapshot["cad_references"]["references"].as_array().and_then(|r| r.iter().find(|r| r["source"] == name));
    let mut enabled = binding.is_some();
    let label = binding.and_then(|b| b["parameter"].as_str()).unwrap_or("Als Referenz");
    let check = ui.checkbox(&mut enabled, label);
    scenario::publish_handle(&format!("worksheet:reference:{name}"), check.rect.center());
    if check.changed() {
        run(app, draft, "worksheet.reference", json!({"source":name,"enabled":enabled}));
    }
}

fn local_calcpad(app: &mut SolveApp, ui: &mut egui::Ui, snapshot: &Value, draft: &mut Draft) {
    if snapshot["calcpad"].is_null() {
        if ui.button("Lokales Calcpad-Berechnungsblatt anlegen").clicked() {
            run(app, draft, "worksheet.calcpad.edit", json!({"source":"'Bauteilberechnung\nL = 120mm\nB = L/3\nh = 5mm\nV = L*B*h"}));
        }
        return;
    }
    ui.separator();
    ui.strong("Calcpad · lokal");
    let source = draft.source.get_or_insert_with(|| snapshot["calcpad"]["source"].as_str().unwrap_or("").to_owned());
    let editor = egui::ScrollArea::vertical()
        .id_salt("calcpad_source_scroll")
        .max_height(240.)
        .show(ui, |ui| ui.add(egui::TextEdit::multiline(source).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY).desired_rows(9)))
        .inner;
    scenario::publish_handle("worksheet:calcpad:source", editor.rect.center());
    let edited = source.clone();
    if editor.changed() {
        draft.inputs.clear();
        run(app, draft, "worksheet.calcpad.edit", json!({"source":edited}));
    }
    let calculate = ui.button("Berechnen");
    scenario::publish_handle("worksheet:calculate", calculate.rect.center());
    if calculate.clicked() {
        run(app, draft, "worksheet.calcpad.calculate", json!({"source":edited}));
    }
    if snapshot["cad_references"]["pending"].as_u64().unwrap_or(0) > 0 {
        let apply = ui.button("Konstruktion aktualisieren");
        scenario::publish_handle("worksheet:apply:top", apply.rect.center());
        if apply.clicked() {
            run(app, draft, "worksheet.apply", json!({}));
        }
    }
    if snapshot["calcpad"]["dirty"] == true {
        ui.label("Berechnung geändert – Ergebnisse erst neu berechnen.");
    }
    egui::ScrollArea::vertical().id_salt("calcpad_result_scroll").max_height(220.).show(ui, |ui| {
        egui::Grid::new("calcpad_results").num_columns(4).striped(true).show(ui, |ui| {
            for title in ["Ergebnis", "Wert", "Einheit", "CAD-Referenz"] {
                ui.strong(title);
            }
            ui.end_row();
            if let Some(results) = snapshot["calcpad"]["results"].as_object() {
                for (name, result) in results {
                    ui.monospace(name);
                    ui.monospace(result["value"].as_f64().map(|v| format!("{v:.8}")).unwrap_or_default());
                    let unit = result["unit"].as_str().unwrap_or("");
                    ui.label(unit);
                    if solidcraft_engine::doc::expr::unit_info(unit).is_some() {
                        reference(app, ui, snapshot, draft, name);
                    } else {
                        ui.label("Berechnungsergebnis");
                    }
                    ui.end_row();
                }
            }
        });
    });
    ui.label(RichText::new("Im Skizzenmaß den Referenznamen einsetzen, z. B. ref_B oder 2 * ref_B + 5 mm. Calcpad-Gleichungen und Einheiten laufen lokal; Bedingte Varianten sind eingebunden; Dateiimporte und Solver-Makros fehlen noch.").small());
}

fn planetary_inputs(app: &mut SolveApp, ui: &mut egui::Ui, draft: &mut Draft) {
    let data = app.session.execute("planetary.inputs", &json!({})).unwrap_or_default();
    let pairs = if ui.available_width() >= 900. { 2 } else { 1 };
    egui::Grid::new("planetary_input_grid").num_columns(pairs * 2).min_col_width(160.).spacing(vec2(14., 8.)).show(ui, |ui| {
        if let Some(rows) = data["rows"].as_array() {
            for (index, row) in rows.iter().enumerate() {
                let name = row["name"].as_str().unwrap_or("");
                let current = row["expression"].as_str().unwrap_or("");
                let label = match name {
                    "Aufbauvariante" => "Aufbauvariante (1–4)",
                    "R_Ziel" => "Schnellgangfaktor Ziel",
                    "T_in" => "Eingangsdrehmoment",
                    "n_in" => "Eingangsdrehzahl",
                    "z_s" => "Zähne Sonne",
                    "modul" => "Modul (Formel oder Wert)",
                    "alpha_grad" => "Eingriffswinkel [°]",
                    "breitenfaktor" => "b / Modul",
                    _ => name,
                };
                let value = draft.inputs.entry(name.into()).or_insert_with(|| current.into());
                ui.label(label);
                let field = ui.add_sized([245., 23.], egui::TextEdit::singleline(value).id_salt(("planetary_input", name)).desired_width(245.));
                scenario::publish_handle(&format!("planetary:input:{name}"), field.rect.center());
                let expression = value.clone();
                if (field.lost_focus() || field.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    && expression != current
                    && run(app, draft, "planetary.inputs", json!({"name":name,"expression":expression}))
                {
                    draft.inputs.remove(name);
                    draft.source = None;
                }
                if (index + 1) % pairs == 0 {
                    ui.end_row();
                }
            }
        }
    });
    ui.small("Der Schnellgangfaktor ist n_ab / n_an; die Berechnung zeigt die mit ganzzahligen Zähnen erreichte Übersetzung. Berechnen, Änderungen prüfen, Konstruktion aktualisieren.");
}

fn review(app: &mut SolveApp, ui: &mut egui::Ui, state: &Value, draft: &mut Draft) {
    let pending = state["pending"].as_u64().unwrap_or(0);
    if pending == 0 {
        return;
    }
    ui.separator();
    ui.strong(format!("{pending} geänderte CAD-Referenzen"));
    if let Some(rows) = state["references"].as_array() {
        for row in rows.iter().filter(|r| r["changed"] == true) {
            let name = row["parameter"].as_str().unwrap_or("");
            if let Some(error) = row["error"].as_str() {
                ui.label(format!("{name}: {error}"));
            } else {
                ui.label(format!("{name}: {} → {} {}", row["old_value"], row["new_value"], row["unit"].as_str().unwrap_or("")));
            }
            let users: std::collections::BTreeSet<_> = row["users"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|user| {
                    let name = user["name"].as_str()?;
                    Some(user["input"].as_str().map(|input| format!("{name} ({input})")).unwrap_or_else(|| name.to_owned()))
                })
                .collect();
            ui.small(format!(
                "Verwendet in: {}",
                if users.is_empty() { "noch nicht verwendet".into() } else { users.into_iter().collect::<Vec<_>>().join(", ") }
            ));
        }
    }
    let apply = ui.button("Konstruktion aktualisieren");
    scenario::publish_handle("worksheet:apply", apply.rect.center());
    if apply.clicked() {
        run(app, draft, "worksheet.apply", json!({}));
    }
}

/// Appears in CAD only when calculation references differ; no permanent editor.
pub fn cad_notice(app: &mut SolveApp, ui: &mut egui::Ui) {
    let state = app.session.execute("worksheet.references", &json!({})).unwrap_or_default();
    if state["pending"].as_u64().unwrap_or(0) == 0 {
        return;
    }
    let host = crate::ui_kit::Container::new(ui.ctx(), crate::ui_kit::Surface::Chrome);
    host.edge("calculation_changes", crate::ui_kit::Edge::Top).show(ui, |ui| {
        let mut draft = Draft::default();
        ui.collapsing(format!("Berechnung geändert: {} CAD-Referenzen – Änderungen prüfen", state["pending"]), |ui| {
            review(app, ui, &state, &mut draft);
        });
        if !draft.error.is_empty() {
            ui.label(draft.error);
        }
    });
}
