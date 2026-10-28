//! Native assembly block view. All reduction remains in the registered engine command.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn show(app: &mut SolveApp, ui: &mut egui::Ui) {
    ui.set_max_width(740.0_f32.min(ui.available_width()));
    let key = egui::Id::new(("native_mks_model", app.docs.active, app.session.path.clone()));
    let cached = ui.data_mut(|d| d.get_temp::<(u64, Value)>(key));
    let model = if let Some((_, v)) = cached.filter(|(r, _)| *r == app.session.revision) {
        v
    } else {
        let v = app.session.execute("mks.model", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}));
        ui.data_mut(|d| d.insert_temp(key, (app.session.revision, v.clone())));
        v
    };
    ui.heading("MKS · Baugruppenmodell");
    ui.label("Starre Teile bilden einen Block. Bewegliche Gelenke und Kraftschnittstellen stammen aus dem aktuellen CAD-Projekt.");
    ui.small("Die Blockstruktur folgt dem CAD-Stand. Dynamikläufe werden separat berechnet und mit ihrem Eingabestand gespeichert.");
    if let Some(error) = model["error"].as_str() {
        ui.colored_label(Tokens::get().warning, error);
        return;
    }
    let blocks = model["blocks"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let joints = model["joints"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    ui.label(format!(
        "{} starre Blöcke · {} bewegliche Gelenke / Relationen · {} unterdrückte Gelenke",
        blocks.len(),
        joints.len(),
        model["suppressed_joints"].as_array().map_or(0, Vec::len)
    ));
    scenario::publish_handle("mks:model", ui.min_rect().center());
    physical_properties(app, ui, key.with("mass"));
    crate::mks_dynamics_ui::show(app, ui, &model);
    egui::ScrollArea::both().id_salt("native_mks_canvas").max_height(330.).show(ui, |ui| {
        let t = Tokens::get();
        let columns = ((ui.available_width() / 260.).floor() as usize).clamp(1, 3).min(blocks.len().max(1));
        let size = egui::vec2(columns as f32 * 260., blocks.len().div_ceil(columns) as f32 * 150.);
        let (area, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter_at(area);
        let mut anchors = BTreeMap::new();
        for (i, b) in blocks.iter().enumerate() {
            let rect = egui::Rect::from_min_size(
                area.min + egui::vec2((i % columns) as f32 * 260. + 10., (i / columns) as f32 * 150. + 20.),
                egui::vec2(220., 108.),
            );
            anchors.insert(b["id"].as_str().unwrap_or(""), (rect.left_center(), rect.right_center()));
        }
        for j in joints {
            let a = j["a"].as_str().and_then(|s| anchors.get(s));
            let b = j["b"].as_str().and_then(|s| anchors.get(s));
            if let (Some(a), Some(b)) = (a, b) {
                let color = if j["connector_supported"] == true { t.accent } else { t.warning };
                painter.line_segment([a.1, b.0], egui::Stroke::new(1.5, color));
                painter.circle_filled(a.1, 4., color);
                painter.circle_filled(b.0, 4., color);
                painter.text(
                    a.1.lerp(b.0, 0.5) + egui::vec2(0., -10.),
                    egui::Align2::CENTER_CENTER,
                    j["kind"].as_str().unwrap_or(""),
                    egui::FontId::proportional(10.),
                    color,
                );
            }
        }
        for b in blocks {
            let id = b["id"].as_str().unwrap_or("");
            if let Some((left, _)) = anchors.get(id) {
                let rect = egui::Rect::from_min_size(*left - egui::vec2(0., 54.), egui::vec2(220., 108.));
                painter.rect_filled(rect, 4., t.field);
                painter.rect_stroke(
                    rect,
                    4.,
                    egui::Stroke::new(1., if b["grounded"] == true { t.accent } else { t.border }),
                    egui::StrokeKind::Inside,
                );
                let names = b["members"].as_array().into_iter().flatten().take(3).filter_map(|m| m["name"].as_str()).collect::<Vec<_>>().join(" + ");
                let status = if b["grounded"] == true {
                    "Fundament"
                } else if b["connected_to_ground"] == false {
                    "Frei / unverbunden"
                } else {
                    "Beweglich verbunden"
                };
                for (row, text) in [
                    format!("{id} · {status}"),
                    names,
                    format!(
                        "{} Körper · {} Kraftschnittstellen",
                        b["bodies"].as_array().map_or(0, Vec::len),
                        b["ports"].as_array().map_or(0, Vec::len)
                    ),
                ]
                .iter()
                .enumerate()
                {
                    painter.text(
                        rect.min + egui::vec2(10., 18. + row as f32 * 25.),
                        egui::Align2::LEFT_CENTER,
                        text,
                        egui::FontId::proportional(11.),
                        t.text,
                    );
                }
                for (i, p) in b["ports"].as_array().into_iter().flatten().take(3).enumerate() {
                    let point = rect.min + egui::vec2(10. + i as f32 * 70., 92.);
                    let port = p["port"]["id"].as_str().unwrap_or("");
                    painter.circle_filled(point, 3., t.accent);
                    painter.text(
                        point + egui::vec2(7., 0.),
                        egui::Align2::LEFT_CENTER,
                        port.chars().take(9).collect::<String>(),
                        egui::FontId::proportional(10.),
                        t.text,
                    );
                    let hit = egui::Rect::from_center_size(point + egui::vec2(28., 0.), egui::vec2(65., 18.));
                    scenario::publish_handle(&format!("mks:port:{port}"), hit.center());
                    ui.interact(hit, ui.id().with(("mks_port", port)), egui::Sense::hover()).on_hover_text(format!(
                        "{} · {} / {}\nPosition Welt [mm]: {}",
                        port, p["port"]["support"], p["port"]["transfer"], p["frame"]["origin"]
                    ));
                }
            }
        }
    });
    egui::ScrollArea::vertical().id_salt("native_mks_details").max_height(230.).show(ui, |ui| {
        for issue in model["issues"].as_array().into_iter().flatten() {
            ui.colored_label(Tokens::get().warning, format!("{}: {}", issue["kind"].as_str().unwrap_or(""), issue["message"].as_str().unwrap_or("")));
        }
        for p in model["unbound_ports"].as_array().into_iter().flatten() {
            ui.colored_label(Tokens::get().warning, format!("Schnittstelle {} ungültig: {}", p["port"]["id"], p["error"]));
        }
        ui.collapsing("Blöcke, Einbaupfade und Kraftschnittstellen", |ui| {
            for b in blocks {
                ui.strong(b["id"].as_str().unwrap_or(""));
                for m in b["members"].as_array().into_iter().flatten() {
                    ui.label(format!("{} · Pfad {}", m["name"].as_str().unwrap_or(""), m["path"]));
                }
                for p in b["ports"].as_array().into_iter().flatten() {
                    ui.label(format!("Port {} · {} / {}", p["port"]["id"], p["port"]["support"], p["port"]["transfer"]));
                }
            }
        });
        ui.collapsing("Gelenke und lokale Freiheitsgrade", |ui| {
            for j in joints {
                ui.label(format!(
                    "{}: {} ↔ {} · {} · {}{}",
                    j["name"].as_str().unwrap_or(""),
                    j["a"],
                    j["b"],
                    j["kind"],
                    j["local_dofs"],
                    if j["closes_graph_cycle"] == true { " · geschlossene Kette" } else { "" }
                ));
            }
            ui.small(
                "Lokale Gelenkfreiheiten ergeben durch Addition keine globale Beweglichkeit. Geschlossene Ketten benötigen eine gesonderte Prüfung.",
            );
        });
    });
    ui.collapsing("Kraftschnittstellen und statische Lastfälle bearbeiten", |ui| crate::mechanics_ui::show(app, ui, false));
}

fn physical_properties(app: &mut SolveApp, ui: &mut egui::Ui, key: egui::Id) {
    let r = ui.button("Masse und Trägheit aus CAD berechnen");
    scenario::publish_handle("mks:mass:calculate", r.rect.center());
    if r.clicked() {
        let result = app.session.execute("mks.mass_properties", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}));
        ui.data_mut(|d| d.insert_temp(key, (app.session.revision, result)));
    }
    let report = ui.data_mut(|d| d.get_temp::<(u64, Value)>(key));
    if let Some((revision, report)) = report {
        if revision != app.session.revision {
            let r =
                ui.colored_label(Tokens::get().warning, "Masseneigenschaften veraltet – nach CAD-, Material- oder Positionsänderung neu berechnen.");
            scenario::publish_handle("mks:mass:stale", r.rect.center());
            return;
        }
        if let Some(error) = report["error"].as_str() {
            ui.colored_label(Tokens::get().warning, error);
            return;
        }
        scenario::publish_handle("mks:mass:current", ui.min_rect().center());
        let details =
            ui.collapsing("Physikalische Eigenschaften · aktueller CAD-Stand", |ui| {
                egui::ScrollArea::vertical().id_salt("mks_mass_details").max_height(190.).show(ui,|ui|{
            for block in report["blocks"].as_array().into_iter().flatten() {
                let id = block["id"].as_str().unwrap_or("");
                if block["complete"] == false {
                    ui.colored_label(Tokens::get().warning, format!("{id}: unvollständig; keine Blockmasse freigegeben"));
                    for error in block["errors"].as_array().into_iter().flatten() {
                        ui.label(format!("{}: {}", error["source"], error["error"]));
                    }
                    continue;
                }
                if block["status"] == "empty" {
                    ui.label(format!("{id}: ohne Volumenkörper"));
                    continue;
                }
                ui.strong(format!("{id} · {:.6} kg", block["mass_kg"].as_f64().unwrap_or(0.)));
                    let coordinate=|i|block["com_world_m"].get(i).and_then(Value::as_f64).unwrap_or(0.);
                    ui.label(format!("Schwerpunkt Welt [m]: [{:.6}, {:.6}, {:.6}]",coordinate(0),coordinate(1),coordinate(2)));
                let assumed = block["bodies"].as_array().into_iter().flatten().filter(|b| b["density_assumed"] == true).count();
                if assumed > 0 {
                    ui.colored_label(Tokens::get().warning, format!("{assumed} Körper verwenden angenommene CAD-Standarddichte; Material zuweisen."));
                }
                    let tensor=ui.collapsing(format!("{id}: Trägheitstensor [kg·m²]"), |ui| {
                    ui.label("Um den Blockschwerpunkt, Achsen parallel zum Weltkoordinatensystem.");
                    for row in block["inertia_com_world_kg_m2"].as_array().into_iter().flatten() {
                            let cell=|i|row.get(i).and_then(Value::as_f64).unwrap_or(0.);
                            ui.monospace(format!("[{:12.5e}  {:12.5e}  {:12.5e}]",cell(0),cell(1),cell(2)));
                    }
                });
                    scenario::publish_handle(&format!("mks:mass:tensor:{id}"),tensor.header_response.rect.center());
            }
            ui.small("Numerische Netzintegration mit zwei Auflösungen. Körper werden addiert; Überschneidungen werden nicht automatisch abgezogen.");
            });
            });
        scenario::publish_handle("mks:mass:details", details.header_response.rect.center());
    }
}
