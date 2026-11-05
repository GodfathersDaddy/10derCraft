//! Engineering tabs inside the original SolidCraft shell; calculations live in the engine.
use crate::{SolveApp, scenario, theme::Tokens};
use egui::{Align2, FontId, RichText, Sense, Stroke, StrokeKind, pos2, vec2};
use serde_json::json;
use solidcraft_engine::studio::{MODULES, TOOLS, support_label, tool_label, transfer_label};
use std::collections::BTreeMap;

pub const TABS: &[&str] = &["MBS", "FEM / ROM", "INTERFACES", "WORKSHEET", "VARIANTS", "STUDIES", "LOAD PATHS"];

pub fn page_for_tab(tab: &str) -> Option<&'static str> {
    match tab {
        "MBS" => Some("system"),
        "FEM / ROM" => Some("fem"),
        "INTERFACES" => Some("interfaces"),
        "WORKSHEET" => Some("parameters"),
        "VARIANTS" => Some("variants"),
        "STUDIES" => Some("studies"),
        "LOAD PATHS" => Some("loads"),
        _ => None,
    }
}

pub fn tab_for_page(page: &str) -> &'static str {
    match page {
        "system" => "MBS",
        "fem" | "fem_rom" => "FEM / ROM",
        "interfaces" => "INTERFACES",
        "parameters" => "WORKSHEET",
        "variants" => "VARIANTS",
        "studies" => "STUDIES",
        "loads" => "LOAD PATHS",
        _ => "MODEL",
    }
}

/// Tab changes only change the native ribbon. Tools explicitly open transient windows.
pub fn start(app: &mut SolveApp, id: &str) -> bool {
    if id == "worksheet.open" || id == "studio.worksheet.open" {
        crate::dialogs::cancel(app);
        app.tool = None;
        app.params = None;
        app.home.open = false;
        app.ui.engineering_dialog = None;
        app.ui.project_page = "parameters".into();
        return true;
    }
    if matches!(id, "studio.system.open" | "studio.fem.open" | "mks.model") {
        app.home.open = false;
        app.ui.engineering_dialog = None;
        app.ui.project_page = if id == "studio.fem.open" { "fem" } else { "mks" }.into();
        app.ui.tab = if id == "studio.fem.open" { "FEM / ROM" } else { "MBS" }.into();
        return true;
    }
    let page = match id {
        "mks.interface.open" => "cad_mks",
        "studio.system.open" => "system",
        "mks.model" => "system",
        "studio.fem.open" => "fem",
        "studio.variants.open" => "variants",
        "studio.studies.open" => "studies",
        "studio.loads.open" => "loads",
        "mechanics.port" | "mechanics.connect" | "mechanics.case" => "interfaces",
        "mechanics.evaluate" | "mechanics.solve" => "loads",
        "studio.interface.update" => "interfaces",
        "studio.interface.add" => {
            if app.run(id, json!({})).is_err() {
                return true;
            }
            "interfaces"
        }
        _ => return false,
    };
    app.home.open = false;
    app.ui.project_page = "cad".into();
    app.ui.engineering_dialog = Some(page.into());
    true
}

fn run(app: &mut SolveApp, id: &str, params: serde_json::Value) {
    if let Err(error) = app.run(id, params) {
        app.set_status(error, true);
    }
}

fn button(ui: &mut egui::Ui, handle: &str, label: &str) -> bool {
    let response = ui.button(label);
    scenario::publish_handle(handle, response.rect.center());
    response.clicked()
}

fn note(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(11.).color(Tokens::get().text_dim));
}

fn interface_form(app: &mut SolveApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Mechanischer Anschluss");
        note(ui, "Referenzrahmen und Lasteinleitung");
        ui.separator();
        let interfaces = app.session.studio.interfaces.clone();
        let selected = app.session.studio.selected_interface.clone();
        let mut choice = None;
        egui::ComboBox::from_id_salt("engineering_port_picker")
            .width(ui.available_width() - 12.)
            .selected_text(if selected.is_empty() { "Anschluss auswählen" } else { &selected })
            .show_ui(ui, |ui| {
                for port in &interfaces {
                    if ui.selectable_label(port.id == selected, &port.id).clicked() {
                        choice = Some(port.id.clone());
                    }
                }
            });
        if let Some(id) = choice {
            run(app, "studio.interface.select", json!({"id":id}));
        }
        let Some(port) = interfaces.iter().find(|p| p.id == selected).cloned() else {
            note(ui, "Im CAD einen Anschlussentwurf anlegen. Ein Körper kann mehrere Schnittstellen besitzen.");
            if button(ui, "studio:interface:add", "Anschluss anlegen") {
                run(app, "studio.interface.add", json!({}));
            }
            return;
        };
        ui.add_space(8.);
        let mut label = port.label.clone();
        if ui.text_edit_singleline(&mut label).changed() && !label.is_empty() {
            run(app, "studio.interface.update", json!({"label":label}));
        }
        note(ui, &format!("Bezugsmodul: {}", port.owner));
        if let Some(frame) = app.session.studio.module_frames.get(&port.owner) {
            note(ui, &format!("Modulrahmen im CAD: {:?} mm", frame.origin_mm));
        } else {
            note(ui, "Modulrahmen noch nicht an CAD angebunden.");
        }
        ui.add_space(8.);
        ui.label("Geometrische Anbindung");
        ui.horizontal_wrapped(|ui| {
            for (id, label) in [("point", "Punkt"), ("surface", "Fläche"), ("surface_group", "Flächengruppe")] {
                let response = ui.selectable_label(port.support == id, label);
                scenario::publish_handle(&format!("studio:support:{id}"), response.rect.center());
                if response.clicked() {
                    run(app, "studio.interface.update", json!({"support":id}));
                }
            }
        });
        ui.add_space(8.);
        ui.label("Übertragungsmodell");
        let choices: &[&str] =
            if port.support == "point" { &["frame_wrench", "point_force"] } else { &["distributed_remote", "rigid_remote", "pressure"] };
        let mut transfer = None;
        egui::ComboBox::from_id_salt("engineering_port_transfer")
            .width(ui.available_width() - 12.)
            .selected_text(transfer_label(&port.transfer))
            .show_ui(ui, |ui| {
                for id in choices {
                    if ui.selectable_label(port.transfer == *id, transfer_label(id)).clicked() {
                        transfer = Some(*id);
                    }
                }
            });
        if let Some(id) = transfer {
            run(app, "studio.interface.update", json!({"transfer":id}));
        }
        note(
            ui,
            match port.transfer.as_str() {
                "point_force" => "Drei Kraftkomponenten am Punkt. Kein unabhängiges Einzelmoment.",
                "frame_wrench" => "Resultierende F und M am Frame; noch keine FEM-Lasteinleitung definiert.",
                "rigid_remote" => "Bindet den Bereich starr an den Frame; kann die Anschlusszone versteifen.",
                "pressure" => "Verteilte Normalbelastung. Flächenorientierung und Druckfeld sind noch zu definieren.",
                _ => "Verteilte Anbindung einer Fläche an den Referenzrahmen. Gewichte und FEM-Abbildung sind noch offen.",
            },
        );
        ui.separator();
        ui.label("Portursprung im Modulrahmen [mm]");
        let mut origin = port.origin_mm;
        let mut changed = false;
        egui::Grid::new("engineering_port_origin").num_columns(2).show(ui, |ui| {
            for (axis, value) in ["x", "y", "z"].into_iter().zip(origin.iter_mut()) {
                ui.label(axis);
                changed |= ui.add(egui::DragValue::new(value).speed(0.5).range(-1e9..=1e9)).changed();
                ui.end_row();
            }
        });
        if changed {
            run(app, "studio.interface.update", json!({"origin_mm":origin}));
        }
        ui.collapsing("Orientierung: Quaternion w, x, y, z", |ui| {
            let key = ui.id().with((&port.id, "orientation"));
            let mut q = ui.ctx().data(|d| d.get_temp::<[f64; 4]>(key)).unwrap_or(port.quaternion_wxyz);
            let mut edited = false;
            ui.horizontal_wrapped(|ui| {
                for value in &mut q {
                    edited |= ui.add(egui::DragValue::new(value).speed(0.01).range(-1.0..=1.0)).changed();
                }
            });
            if edited {
                ui.ctx().data_mut(|d| d.insert_temp(key, q));
            }
            if ui.button("Normiert übernehmen").clicked() {
                let norm = q.iter().map(|v| v * v).sum::<f64>().sqrt();
                if norm.is_finite() && norm > 1e-12 {
                    run(app, "studio.interface.update", json!({"quaternion_wxyz":q.map(|v|v/norm)}));
                } else {
                    app.set_status("Orientierung benötigt ein von null verschiedenes Quaternion", true);
                }
            }
        });
        ui.separator();
        ui.label(format!("CAD-Referenzen: {}", port.cad_selection.len()));
        for pick in &port.cad_selection {
            let label = match pick {
                solidcraft_engine::Sel::Face { body, index, .. } => format!("{body} / Fläche {index}"),
                solidcraft_engine::Sel::Edge { body, index, .. } => format!("{body} / Kante {index}"),
                _ => "CAD-Pick".into(),
            };
            note(ui, &label);
        }
        if button(ui, "studio:interface:capture", "Aktuelle CAD-Auswahl zuordnen") {
            run(app, "studio.interface.capture", json!({}));
        }
        note(ui, "Im Reiter Schnittstellen direkt Flächen wählen. Für eine Flächengruppe mehrere Flächen desselben Körpers auswählen.");
        note(ui, "Der Portursprung wird nicht automatisch auf einen Flächenschwerpunkt verschoben.");
        ui.separator();
        if app.session.studio.pending_connection.is_some() {
            if button(ui, "studio:connection:cancel", "Verbinden abbrechen") {
                run(app, "studio.connection.arm", json!({"cancel":true}));
            }
            note(ui, "Zielport im MKS-Blockmodell anklicken.");
        } else if button(ui, "studio:connection:arm", "Mit anderem Port verbinden") {
            run(app, "studio.connection.arm", json!({}));
        }
        let connections = app.session.studio.connections.clone();
        for edge in connections.iter().filter(|c| c.a == port.id || c.b == port.id) {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("{} · {}", edge.id, edge.kind));
                if ui.small_button("Entfernen").clicked() {
                    run(app, "studio.connection.remove", json!({"id":edge.id}));
                }
            });
            note(ui, if edge.a == port.id { &edge.b } else { &edge.a });
        }
        ui.separator();
        note(ui, "ENTWURF · nur diese Sitzung");
        note(ui, "Keine berechnete FEM-Kopplung. CAD-Topologiereferenzen und physikalische Kompatibilität sind ungeprüft.");
        ui.add_space(6.);
        note(ui, &app.session.studio.notice);
    });
}

pub fn show_dialogs(app: &mut SolveApp, ctx: &egui::Context) {
    let Some(page) = app.ui.engineering_dialog.clone() else { return };
    let title = match page.as_str() {
        "cad_mks" => "CAD → MKS",
        "system" => "Mehrkörpermodell",
        "fem" | "fem_rom" => "FEM / ROM",
        "interfaces" => "Mechanischer Anschluss",
        "variants" => "Varianten",
        "studies" => "Referenzstudien",
        "loads" => "Mechanische Lastpfade",
        _ => "Engineering",
    };
    let open = true;
    let closed = crate::ui_kit::Window::new(egui::Id::new(("engineering_tool_window", &page)), "graph", title)
        .cards(if page == "interfaces" { 1.5 } else { 3. })
        .page_rows(if page == "cad_mks" || page == "fem_rom" { 32. } else { 24. })
        .show_body(ctx, |ui| {
            scenario::publish_handle("studio:dialog:open", ui.min_rect().center());
            match page.as_str() {
                "cad_mks" => crate::mks_interfaces_ui::show(app, ui),
                "system" => system(app, ui),
                "fem" => fem(app, ui),
                "fem_rom" => crate::fem_rom_ui::show(app, ui),
                "interfaces" => {
                    crate::mechanics_ui::show(app, ui, false);
                    if app.session.studio.enabled {
                        ui.collapsing("Alte Sitzungsentwürfe / Roboterdemo", |ui| interface_form(app, ui));
                    }
                }
                "variants" => variants(app, ui),
                "studies" => studies(app, ui),
                "loads" => {
                    crate::mechanics_ui::show(app, ui, true);
                }
                _ => {}
            }
            ui.separator();
            if button(ui, "studio:dialog:close", "Schließen") {
                app.ui.engineering_dialog = None;
            }
        });
    if closed || !open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.ui.engineering_dialog = None;
    }
}

fn representation(app: &mut SolveApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Oberarm / Unterarm:");
        for (id, modes, label) in [("rigid", 6, "Starr"), ("flexible", 6, "Flexibel · 6 Moden"), ("flexible", 20, "Flexibel · 20 Moden")] {
            let active = app.session.studio.representation == id && (id == "rigid" || app.session.studio.mode_count == modes);
            let response = ui.selectable_label(active, label);
            scenario::publish_handle(&format!("studio:representation:{id}:{modes}"), response.rect.center());
            if response.clicked() {
                run(app, "studio.representation", json!({"representation":id,"mode_count":modes}));
            }
        }
    });
    note(ui, "Varianten besitzen dieselben äußeren Ports. Flexible Matrizen sind noch nicht geladen.");
}

fn system(app: &mut SolveApp, ui: &mut egui::Ui) {
    crate::robot_ui::motion(app, ui);
    if !app.session.doc.occurrences.is_empty() {
        egui::ScrollArea::vertical()
            .id_salt("mks_workspace_scroll")
            .max_height((ui.ctx().content_rect().height() - 160.).max(320.))
            .show(ui, |ui| crate::mks_ui::show(app, ui));
        return;
    }
    if crate::mechanics_ui::has_project(app) {
        crate::mechanics_ui::show(app, ui, true);
        return;
    }
    representation(app, ui);
    ui.separator();
    note(ui, "Mechanik: F/M und v/ω, bidirektional. Gestrichelt: gerichtete Signale. Gelenke sind eigene Kopplungselemente.");
    let t = Tokens::get();
    let ports = app.session.studio.interfaces.clone();
    let connections = app.session.studio.connections.clone();
    let selected = app.session.studio.selected_interface.clone();
    if ports.is_empty() {
        ui.heading("Mehrkörpermodell aus CAD-Schnittstellen");
        note(
            ui,
            "Zuerst im CAD einen Körper oder eine Fläche auswählen und unter Schnittstellen einen Port anlegen. Jede Komponente kann mehrere Ports erhalten.",
        );
        if ui.button("Schnittstellen definieren").clicked() {
            app.start("studio.interface.update");
        }
        return;
    }
    let mut modules: Vec<(String, String, String)> = MODULES
        .iter()
        .filter(|(owner, _, _)| ports.iter().any(|p| p.owner == *owner))
        .map(|(id, name, kind)| ((*id).into(), (*name).into(), (*kind).into()))
        .collect();
    for port in &ports {
        if !modules.iter().any(|(id, _, _)| id == &port.owner) {
            modules.push((port.owner.clone(), port.owner.clone(), "CAD-Körper / Entwurf".into()));
        }
    }
    let columns = if ui.available_width() > 930. {
        4
    } else if ui.available_width() > 650. {
        3
    } else {
        2
    };
    let step_x = 205.;
    let step_y = 170.;
    let rows = modules.len().div_ceil(columns);
    egui::ScrollArea::both().show(ui, |ui| {
        let (area, _) =
            ui.allocate_exact_size(vec2((columns as f32 * step_x).max(ui.available_width()).max(720.), rows as f32 * step_y + 310.), Sense::hover());
        let painter = ui.painter_at(area);
        let mut rects = BTreeMap::new();
        let mut anchors = BTreeMap::new();
        for (index, (id, _, _)) in modules.iter().enumerate() {
            let row = index / columns;
            let col = if row % 2 == 0 { index % columns } else { columns - 1 - index % columns };
            let rect = egui::Rect::from_min_size(area.min + vec2(24. + col as f32 * step_x, 45. + row as f32 * step_y), vec2(150., 100.));
            rects.insert(id.clone(), rect);
            let owned: Vec<_> = ports.iter().filter(|p| p.owner == *id).collect();
            for (i, port) in owned.iter().enumerate() {
                let pos = if i == 0 {
                    if (*id == "base") == (row % 2 == 0) { rect.right_center() } else { rect.left_center() }
                } else if i == 1 {
                    if row % 2 == 0 { rect.right_center() } else { rect.left_center() }
                } else {
                    rect.center_bottom() + vec2(0., (i - 2) as f32 * 19.)
                };
                anchors.insert(port.id.clone(), pos);
            }
        }
        for edge in &connections {
            if let (Some(a), Some(b)) = (anchors.get(&edge.a), anchors.get(&edge.b)) {
                let middle = pos2((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
                let middle_x = (a.x + b.x) * 0.5;
                let path = [*a, pos2(middle_x, a.y), pos2(middle_x, b.y), *b];
                for pair in path.windows(2) {
                    if let [from, to] = pair {
                        painter.line_segment([*from, *to], Stroke::new(1.8, t.text_dim));
                    }
                }
                let joint = egui::Rect::from_center_size(middle, vec2(40., 24.));
                painter.rect(joint, 2., t.field, Stroke::new(1., t.border), StrokeKind::Inside);
                painter.text(joint.center(), Align2::CENTER_CENTER, &edge.id, FontId::proportional(10.), t.text);
                if ui.interact(joint, ui.id().with((&edge.id, "joint")), Sense::click()).clicked() {
                    run(app, "studio.interface.select", json!({"id":edge.a}));
                }
            }
        }
        for (id, name, kind) in &modules {
            let Some(rect) = rects.get(id).copied() else { continue };
            let active = app.session.studio.selected.as_str() == id.as_str();
            painter.rect(
                rect,
                3.,
                if active { t.accent_soft } else { t.panel },
                Stroke::new(1.2, if active { t.accent } else { t.border }),
                StrokeKind::Inside,
            );
            let title = if id == "tool" { tool_label(&app.session.studio.tool) } else { name.as_str() };
            painter.text(rect.center_top() + vec2(0., 23.), Align2::CENTER_CENTER, title, FontId::proportional(12.), t.text);
            let subtitle = if ["upper", "forearm"].contains(&id.as_str()) && app.session.studio.representation != "rigid" {
                format!("Flexibel · {} Moden (Entwurf)", app.session.studio.mode_count)
            } else {
                kind.clone()
            };
            painter.text(rect.center_top() + vec2(0., 46.), Align2::CENTER_CENTER, subtitle, FontId::proportional(9.5), t.text_dim);
            painter.text(rect.center_top() + vec2(0., 72.), Align2::CENTER_CENTER, "mechanischer Körper", FontId::proportional(9.), t.text_dim);
            let response = ui.interact(rect.shrink(12.), ui.id().with((id, "module")), Sense::click());
            scenario::publish_handle(&format!("studio:block:{id}"), rect.center());
            if response.clicked() {
                run(app, "studio.select", json!({"module":id}));
            }
        }
        for port in &ports {
            let Some(pos) = anchors.get(&port.id).copied() else { continue };
            let color = if selected == port.id { t.accent } else { t.text_dim };
            if port.support == "point" {
                painter.circle_filled(pos, 5., color);
            } else {
                painter.rect_filled(egui::Rect::from_center_size(pos, vec2(10., 10.)), 1., color);
            }
            let response = ui.interact(egui::Rect::from_center_size(pos, vec2(20., 20.)), ui.id().with((&port.id, "port")), Sense::click());
            scenario::publish_handle(&format!("studio:port:{}", port.id), pos);
            if response.clicked() {
                run(app, "studio.interface.select", json!({"id":port.id}));
            }
            response.on_hover_text(format!(
                "{}\n{} · {}\nLokaler Frame: {:?} mm",
                port.id,
                support_label(&port.support),
                transfer_label(&port.transfer),
                port.origin_mm
            ));
            painter.text(pos + vec2(0., -15.), Align2::CENTER_BOTTOM, &port.label, FontId::proportional(9.), color);
        }
        let y = area.top() + rows as f32 * step_y + 35.;
        let mut last: Option<egui::Pos2> = None;
        for (name, x) in [("Bewegungsprofil", 24.), ("Antrieb / Regler", 235.), ("Sensor / Auswertung", 446.)] {
            let rect = egui::Rect::from_min_size(pos2(area.left() + x, y), vec2(165., 55.));
            painter.rect(rect, 3., t.field, Stroke::new(1., t.border), StrokeKind::Inside);
            painter.text(rect.center_top() + vec2(0., 17.), Align2::CENTER_CENTER, name, FontId::proportional(11.), t.text);
            painter.text(
                rect.center_bottom() - vec2(0., 10.),
                Align2::CENTER_BOTTOM,
                "Signalbaustein · geplant",
                FontId::proportional(9.),
                t.text_dim,
            );
            if let Some(from) = last {
                let to = rect.left_center();
                let mut x = from.x;
                while x < to.x - 8. {
                    painter.line_segment([pos2(x, from.y), pos2((x + 6.).min(to.x - 8.), from.y)], Stroke::new(1., t.accent));
                    x += 11.;
                }
                painter.arrow(to - vec2(8., 0.), vec2(8., 0.), Stroke::new(1., t.accent));
            }
            last = Some(rect.right_center());
        }
        painter.text(
            pos2(area.left() + 24., y + 80.),
            Align2::LEFT_TOP,
            "Blockentwurf: keine Bewegungsgleichungen assembliert, kein Simulationslauf gestartet.",
            FontId::proportional(10.),
            t.text_dim,
        );
        let artifact_y = y + 120.;
        let mut previous: Option<egui::Pos2> = None;
        for (label, x) in [("CAD / Netz", 24.), ("FEM / Moden", 190.), ("ROM: M, K, C", 356.), ("MKS-Repräsentation", 522.)] {
            let r = egui::Rect::from_min_size(pos2(area.left() + x, artifact_y), vec2(140., 55.));
            painter.rect(r, 3., t.panel, Stroke::new(1., t.accent), StrokeKind::Inside);
            painter.text(r.center_top() + vec2(0., 18.), Align2::CENTER_CENTER, label, FontId::proportional(10.5), t.text);
            painter.text(r.center_bottom() - vec2(0., 9.), Align2::CENTER_BOTTOM, "Ableitung · geplant", FontId::proportional(9.), t.text_dim);
            if let Some(from) = previous {
                painter.arrow(from, r.left_center() - from, Stroke::new(1., t.accent));
            }
            previous = Some(r.right_center());
        }
        painter.text(
            pos2(area.left() + 24., artifact_y + 70.),
            Align2::LEFT_TOP,
            "Artefaktfluss; dieselben mechanischen Schnittstellen bleiben an CAD, FEM, ROM und MKS referenziert.",
            FontId::proportional(10.),
            t.text_dim,
        );
    });
}

fn fem(app: &mut SolveApp, ui: &mut egui::Ui) {
    if crate::robot_ui::fem(app, ui) {
        return;
    }
    note(ui, "FEM / ROM verwendet dasselbe Blockmodell und dieselben Ports wie die Mehrkoerpersimulation.");
    system(app, ui);
}

fn variants(app: &mut SolveApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        if crate::worksheet_ui::project_variants(app, ui) {
            return;
        }
        ui.heading("Varianten mit unveränderten äußeren Anschlüssen");
        representation(app, ui);
        ui.separator();
        ui.label("Modul am Werkzeugflansch IF-06");
        ui.horizontal_wrapped(|ui| {
            for (id, label, _) in TOOLS {
                let r = ui.selectable_label(app.session.studio.tool == id, label);
                scenario::publish_handle(&format!("studio:tool:{id}"), r.rect.center());
                if r.clicked() {
                    run(app, "studio.tool", json!({"tool":id}));
                }
            }
        });
        ui.add_space(12.);
        ui.label("Konstruktionsvariante");
        ui.horizontal_wrapped(|ui| {
            for (id, label) in [("baseline", "Basis"), ("lightweight", "Leichtbau"), ("reach", "Reichweite")] {
                let r = ui.selectable_label(app.session.studio.variant == id, label);
                scenario::publish_handle(&format!("studio:variant:{id}"), r.rect.center());
                if r.clicked() {
                    run(app, "studio.variant", json!({"variant":id}));
                }
            }
        });
        ui.add_space(12.);
        note(ui, "Werkzeuge wechseln die sichtbare CAD-Geometrie. Strukturoptimierung und flexible Modelle werden hier nur als Entwurf ausgewählt.");
        note(ui, "Bei Werkzeugwechsel werden erfasste CAD-Referenzen des Werkzeugs verworfen; die logischen Port-IDs bleiben bestehen.");
    });
}

fn studies(app: &mut SolveApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Simulationen und Auswertungen");
        ui.label(RichText::new("Referenzstudie: parametrischer Kragträger").strong());
        note(ui, "Rechteckquerschnitt, linear-elastische Biegung. Dieser Referenzfall ist unabhängig von der aktuellen CAD-Baugruppe.");
        let id = ui.id().with("reference_beam_inputs");
        let default = solidcraft_engine::studio_beam::Beam::default();
        let mut inputs = ui.ctx().data_mut(|data| data.get_temp::<[f64; 7]>(id)).unwrap_or([
            default.length_mm, default.thickness_mm, default.height_mm, default.young_mpa,
            default.density_kg_m3, default.force_n, default.deflection_limit_mm,
        ]);
        let labels = ["Länge [mm]", "Dicke [mm]", "Höhe [mm]", "E-Modul [MPa]", "Dichte [kg/m³]", "Endkraft z [N]", "Max. Durchbiegung [mm]"];
        let names = ["length_mm", "thickness_mm", "height_mm", "young_mpa", "density_kg_m3", "force_n", "deflection_limit_mm"];
        egui::Grid::new("reference_beam_inputs").striped(true).show(ui, |ui| {
            for (label, value) in labels.into_iter().zip(inputs.iter_mut()) {
                ui.label(label);
                ui.add(egui::DragValue::new(value).speed(0.1));
                ui.end_row();
            }
        });
        ui.ctx().data_mut(|data| data.insert_temp(id, inputs));
        let mut params = serde_json::Map::new();
        for (key, value) in names.into_iter().zip(inputs) { params.insert(key.into(), json!(value)); }
        ui.horizontal_wrapped(|ui| {
            if button(ui, "studio:beam:evaluate", "Referenz berechnen") {
                run(app, "studio.beam.evaluate", serde_json::Value::Object(params.clone()));
            }
            for (backend, label) in [("scipy_beam_size", "SciPy-Auftrag vorbereiten"), ("exudyn_beam_reference", "Exudyn-Auftrag vorbereiten")] {
                if button(ui, &format!("studio:job:{backend}"), label) {
                    let mut p = params.clone();
                    p.insert("backend".into(), json!(backend));
                    run(app, "studio.job.create", serde_json::Value::Object(p));
                }
            }
        });
        if let Some(report) = app.session.studio.beam_result.clone() {
            ui.group(|ui| {
                ui.label(RichText::new("Letztes berechnetes Referenzergebnis").strong());
                let evaluated = report.get("inputs");
                if evaluated != Some(&serde_json::Value::Object(params.clone()))
                    || !solidcraft_engine::studio::reference_parameters_current(&app.session.doc, &report) {
                    ui.colored_label(Tokens::get().accent, "Eingaben geändert – angezeigtes Ergebnis gehört zum vorherigen Parametersatz.");
                }
                for (key, label, unit) in [("mass_kg", "Masse", "kg"), ("tip_deflection_mm", "Durchbiegung", "mm"),
                    ("root_bending_stress_mpa", "Biegespannung", "MPa"), ("tip_stiffness_n_m", "Endsteifigkeit", "N/m")] {
                    if let Some(value) = report.get("results").and_then(|r|r.get(key)).and_then(serde_json::Value::as_f64) {
                        ui.label(format!("{label}: {value:.5} {unit}"));
                    }
                }
                let feasible = report.get("results").and_then(|r|r.get("deflection_constraint_satisfied")).and_then(serde_json::Value::as_bool) == Some(true);
                ui.label(if feasible { "Durchbiegungsgrenze eingehalten." } else { "Durchbiegungsgrenze überschritten." });
                note(ui, "Keine Spannungsfreigabe, keine Berücksichtigung von Eigengewicht, Schub, Beulen oder Plastizität.");
                if let Some(warnings) = report.get("warnings").and_then(serde_json::Value::as_array) {
                    for warning in warnings.iter().skip(1).filter_map(serde_json::Value::as_str) { note(ui, warning); }
                }
            });
        }
        if let Some(job) = app.session.studio.backend_job.clone() {
            ui.collapsing("Vorbereiteter externer Solverauftrag", |ui| {
                if job.get("reference").is_some_and(|r| !solidcraft_engine::studio::reference_parameters_current(&app.session.doc, r)) {
                    ui.colored_label(Tokens::get().accent, "Gebundene CAD-Parameter geändert: Auftrag ist veraltet.");
                }
                note(ui, "JSON für den optionalen Worker. Vorbereitung startet keine Simulation. Ergebnisse können über studio.job.accept importiert werden.");
                if let Ok(text) = serde_json::to_string_pretty(&job) {
                    if ui.button("Auftrags-JSON kopieren").clicked() { ui.ctx().copy_text(text.clone()); }
                    ui.monospace(text);
                }
            });
        }
        if let Some(result) = app.session.studio.backend_result.clone() {
            ui.collapsing("Importiertes externes Referenzergebnis", |ui| {
                if let Ok(text) = serde_json::to_string_pretty(&result.get("summary")) { ui.monospace(text); }
                ui.label(format!("Status: {}", result.get("status").and_then(serde_json::Value::as_str).unwrap_or("unknown")));
            });
        }
        ui.separator();
        ui.label(RichText::new("Weitere Baugruppenstudien – geplant").strong());
        for (name, description) in [
            ("Starre Kinematik / inverse Lasten", "Bewegungsvorgabe, Gelenkgrenzen, Reaktionskräfte und Momente."),
            ("Flexible Mehrkörpersimulation", "Kraft- oder Bewegungsvorgabe konsistent am selben Freiheitsgrad; Sensoren an erhaltenen Frames."),
            ("Varianten- und Bewegungsprofile", "Gleiche äußere Ports; Versuchsnummer, Anregung, ROM-Stand und Parameter gemeinsam protokollieren."),
            ("Entwurfsoptimierung", "Minimale Masse bei begrenzter Durchbiegung; Ergebnis führt zu einer neuen CAD-Variante."),
        ] {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(name).strong());
                note(ui, description);
                ui.add_enabled(false, egui::Button::new("Simulationslauf – noch nicht implementiert"));
            });
            ui.add_space(10.);
        }
        note(
            ui,
            "Rohsignale und gefilterte Auswertung künftig getrennt speichern. Numerische Fehler werden nicht durch Filter als behoben bewertet.",
        );
    });
}

/// Project workspace, not a floating tool window.
pub fn mks_workspace(app: &mut SolveApp, ui: &mut egui::Ui) {
    let key = egui::Id::new(("mks_workspace_model", app.docs.active, app.session.path.clone()));
    let cached = ui.ctx().data(|d| d.get_temp::<(u64, serde_json::Value)>(key));
    let model = if let Some((_, v)) = cached.filter(|(r, _)| *r == app.session.revision) {
        v
    } else {
        let v = app.session.execute("mks.model", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}));
        ui.ctx().data_mut(|d| d.insert_temp(key, (app.session.revision, v.clone())));
        v
    };
    crate::mks_graph_ui::show(app, ui, &model);
    show_dialogs(app, ui.ctx());
}

/// Compact FEM controls leave the regular, selectable CAD viewport below.
pub fn fem_ribbon(app: &mut SolveApp, ui: &mut egui::Ui) {
    let host = crate::ui_kit::Container::new(ui.ctx(), crate::ui_kit::Surface::Chrome);
    host.edge("fem_workspace_ribbon", crate::ui_kit::Edge::Top)
        .frame(host.edge_frame(Tokens::get().toolbar).inner_margin(Tokens::get().space_8))
        .show(ui, |ui| {
            let t = Tokens::get();
            ui.set_min_height(t.form_button_height + t.space_16);
            ui.spacing_mut().interact_size.y = t.form_button_height + t.space_8;
            ui.horizontal_wrapped(|ui| {
                ui.heading("FEM");
                let b = ui.button(app.tr("ui.fem_rom.open"));
                scenario::publish_handle("fem:rom:open", b.rect.center());
                if b.clicked() {
                    app.ui.engineering_dialog = Some("fem_rom".into());
                }
                let mut preview = app.session.studio.interface_preview;
                let r = ui.checkbox(&mut preview, app.tr("ui.fem_rom.interfaces"));
                scenario::publish_handle("fem:interfaces:toggle", r.rect.center());
                if r.changed() {
                    let _ = app.run("mks.interface.display", json!({"enabled":preview}));
                }
                if crate::ui_kit::button(ui, app.tr("ui.analysis.interfaces"), crate::ui_kit::Button::Normal).clicked() {
                    app.ui.engineering_dialog = Some("cad_mks".into());
                }
                let run = ui.add_enabled(
                    app.session.doc.metadata.get("org.solidcraft.robot-workflow").is_some(),
                    egui::Button::new(app.tr("ui.fem_rom.beam")),
                );
                scenario::publish_handle("fem:workspace:solve", run.rect.center());
                if run.clicked() {
                    crate::fem_rom_ui::select_beam(app, ui.ctx());
                    let _ = app.run("robot.fem.solve", json!({"elements":16}));
                }
                let results = crate::ui_kit::button(ui, app.tr("ui.analysis.results"), crate::ui_kit::Button::Normal);
                scenario::publish_handle("fem:workspace:results", results.rect.center());
                if results.clicked() {
                    let key = egui::Id::new(("fem_results_visible", app.docs.active));
                    ui.ctx().data_mut(|d| d.insert_temp(key, true));
                }
                let setup = crate::ui_kit::button(ui, app.tr("ui.analysis.setup"), crate::ui_kit::Button::Normal);
                scenario::publish_handle("fem:workspace:setup", setup.rect.center());
                if setup.clicked() {
                    let key = egui::Id::new(("fem_results_visible", app.docs.active));
                    ui.ctx().data_mut(|d| d.insert_temp(key, false));
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use crate::scenario::Harness;
    use serde_json::json;

    #[test]
    fn engineering_inputs_require_an_explicit_interactive_command() {
        let mut h = Harness::new();
        assert!(h.app.ui.engineering_dialog.is_none());
        for tab in super::TABS {
            h.app.ui.tab = (*tab).into();
            h.frames(2);
            assert!(h.app.ui.engineering_dialog.is_none());
            assert!(h.widget("Referenz berechnen").is_none());
        }
        h.app.run("studio.studies.open", json!({})).unwrap();
        assert!(h.app.ui.engineering_dialog.is_none(), "engine calls must not open input windows");
        h.app.start("studio.studies.open");
        h.frames(2);
        assert_eq!(h.app.ui.engineering_dialog.as_deref(), Some("studies"));
        assert!(h.widget("Referenz berechnen").is_some());
        h.app.ui.engineering_dialog = None;
        h.frames(2);
        assert!(h.widget("Referenz berechnen").is_none());
    }
}
