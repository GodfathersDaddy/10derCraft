//! Free worksheet layout over the existing native/Calcpad calculation engines.
use super::{Draft, review, run, variants};
use crate::{SolveApp, scenario, theme::Tokens};
use egui::{Align2, FontId, Rect, Sense, Stroke, pos2, vec2};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, Default)]
struct Sheet {
    initialized: bool,
    library_open: bool,
    family_open: bool,
    symbols_open: bool,
    family: FamilyDraft,
    library_search: String,
    library_tab: usize,
    inline: crate::worksheet_inline_ui::State,
    scroll_to: Option<String>,
    positions: BTreeMap<String, [f32; 2]>,
    notes: Vec<String>,
    charts: Vec<String>,
    new_formula: bool,
    dirty: bool,
    source_mode: bool,
    last_layout: Value,
    draft: Draft,
}

pub(super) fn show(app: &mut SolveApp, ui: &mut egui::Ui) -> bool {
    let key = egui::Id::new(("worksheet_canvas", app.docs.active, app.session.path.clone(), app.session.doc.name.clone()));
    let snapshot = app.session.execute("worksheet.read", &json!({})).unwrap_or_default();
    let layout = &snapshot["document"]["presentation"];
    let mut sheet = ui.ctx().data(|d| d.get_temp::<Sheet>(key)).unwrap_or_else(|| Sheet {
        positions: serde_json::from_value(layout["positions"].clone()).unwrap_or_default(),
        notes: serde_json::from_value(layout["notes"].clone()).unwrap_or_default(),
        charts: serde_json::from_value(layout["charts"].clone()).unwrap_or_default(),
        last_layout: layout.clone(),
        ..Sheet::default()
    });
    if *layout != sheet.last_layout && !sheet.dirty {
        sheet.positions = serde_json::from_value(layout["positions"].clone()).unwrap_or_default();
        sheet.notes = serde_json::from_value(layout["notes"].clone()).unwrap_or_default();
        sheet.charts = serde_json::from_value(layout["charts"].clone()).unwrap_or_default();
        sheet.last_layout = layout.clone();
    }
    sheet.inline.begin(&snapshot);
    sheet.inline.digits = layout["significant_digits"].as_u64().filter(|v| (2..=16).contains(v)).unwrap_or(sheet.inline.digits as u64) as usize;
    let host = crate::ui_kit::Container::new(ui.ctx(), crate::ui_kit::Surface::Chrome);
    host.edge("worksheet_ribbon", crate::ui_kit::Edge::Top).frame(host.edge_frame(Tokens::get().toolbar).inner_margin(8.)).show(ui, |ui| {
        ui.set_min_height(58.);
        ui.style_mut().always_scroll_the_only_direction = true;
        egui::ScrollArea::horizontal()
            .id_salt("worksheet_ribbon_scroll")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(app.tr("ui.sheet.title"));
                    let symbols = ui.selectable_label(sheet.symbols_open, app.tr("ui.sheet_library.symbols"));
                    scenario::publish_handle("worksheet:symbols", symbols.rect.center());
                    if symbols.clicked() {
                        sheet.symbols_open = !sheet.symbols_open;
                    }

                    let library = crate::toolbar::engineering_action(
                        ui,
                        if sheet.library_open { "book_open" } else { "book_closed" },
                        app.tr("ui.sheet_library.title"),
                        sheet.library_open,
                    );
                    scenario::publish_handle("worksheet:library", library.rect.center());
                    if library.clicked() {
                        sheet.inline.commit(app);
                        sheet.library_open = !sheet.library_open;
                    }

                    let family = crate::toolbar::engineering_action(ui, "params", app.tr("ui.sheet_family.title"), sheet.family_open);
                    scenario::publish_handle("worksheet:families", family.rect.center());
                    if family.clicked() {
                        sheet.inline.commit(app);
                        sheet.family_open = !sheet.family_open;
                        sheet.library_open = false;
                    }
                    let b = crate::toolbar::engineering_action(ui, "params", app.tr("ui.sheet.formula"), false);
                    scenario::publish_handle("worksheet:sheet:add", b.rect.center());
                    if b.clicked() {
                        sheet.new_formula = true;
                    }
                    let pick = crate::toolbar::engineering_action(ui, "select", app.tr("ui.sheet_inline.pick_reference"), sheet.inline.pick);
                    scenario::publish_handle("worksheet:pick_reference", pick.rect.center());
                    if pick.clicked() {
                        sheet.inline.commit(app);
                        sheet.inline.pick = !sheet.inline.pick;
                        sheet.inline.arrange = false;
                    }
                    let arrange = crate::toolbar::engineering_action(ui, "move", app.tr("ui.sheet_inline.arrange"), sheet.inline.arrange);
                    if arrange.clicked() {
                        sheet.inline.commit(app);
                        sheet.inline.arrange = !sheet.inline.arrange;
                        sheet.inline.pick = false;
                    }
                    ui.label(app.tr("ui.sheet_inline.digits"));
                    let digits = ui.add(egui::DragValue::new(&mut sheet.inline.digits).range(2..=16));
                    scenario::publish_handle("worksheet:precision", digits.rect.center());
                    if digits.changed() {
                        sheet.dirty = true;
                    }
                    if crate::toolbar::engineering_action(ui, "text", app.tr("ui.sheet.text"), false).clicked() && sheet.notes.len() < 100 {
                        sheet.notes.push(app.tr("ui.sheet.note").to_string());
                        sheet.dirty = true;
                    }
                    if crate::toolbar::engineering_action(ui, "graph", app.tr("ui.sheet.chart"), false).clicked() && sheet.charts.len() < 20 {
                        sheet.charts.push(String::new());
                        sheet.dirty = true;
                    }
                    ui.separator();
                    let calculate = crate::toolbar::engineering_action(ui, "params", app.tr("ui.sheet.calculate"), false);
                    scenario::publish_handle("worksheet:calculate", calculate.rect.center());
                    if calculate.clicked() && !snapshot["calcpad"].is_null() {
                        sheet.inline.commit(app);
                        run(app, &mut sheet.draft, "worksheet.calcpad.calculate", json!({}));
                    }
                    let apply = crate::toolbar::engineering_action(ui, "drive", app.tr("ui.sheet.apply"), false);
                    scenario::publish_handle("worksheet:apply:top", apply.rect.center());
                    if apply.clicked() {
                        sheet.inline.commit(app);
                        run(app, &mut sheet.draft, "worksheet.apply", json!({}));
                    }
                    ui.separator();
                    if crate::toolbar::engineering_action(ui, "text", app.tr("ui.sheet.source"), sheet.source_mode).clicked() {
                        sheet.source_mode = !sheet.source_mode;
                    }
                    if sheet.symbols_open {
                        ui.separator();
                        for symbol in ["α", "β", "γ", "δ", "ε", "η", "θ", "λ", "μ", "ν", "π", "ρ", "σ", "τ", "φ", "ω", "Δ", "Σ", "Ω"]
                        {
                            let button = ui.button(symbol);
                            scenario::publish_handle(&format!("worksheet:symbol:{symbol}"), button.rect.center());
                            if button.clicked() {
                                sheet.inline.symbol(ui.ctx(), symbol);
                            }
                        }
                    }
                });
            });
        scenario::publish_handle("worksheet:ribbon", ui.min_rect().center());
    });

    if sheet.library_open {
        host.edge("worksheet_library", crate::ui_kit::Edge::Top)
            .default_size(340.)
            .resizable(true)
            .frame(host.edge_frame(Tokens::get().toolbar).inner_margin(10.))
            .show(ui, |ui| {
                ui.set_min_height(310.);
                ui.horizontal(|ui| {
                    for (i, label) in
                        ["ui.library.materials", "ui.library.fasteners", "ui.library.constants", "ui.library.families"].iter().enumerate()
                    {
                        let tab = ui.selectable_label(sheet.library_tab == i, app.tr(label));
                        scenario::publish_handle(&format!("worksheet:library:tab:{i}"), tab.rect.center());
                        if tab.clicked() {
                            sheet.library_tab = i;
                        }
                    }
                });
                let search = ui.add(egui::TextEdit::singleline(&mut sheet.library_search).hint_text(app.tr("ui.sheet_library.search")));
                scenario::publish_handle("worksheet:library:search", search.rect.center());
                egui::ScrollArea::vertical().id_salt("worksheet_library_scroll").min_scrolled_height(240.).max_height(260.).show(ui, |ui| {
                    if sheet.library_tab == 3 {
                        family_table(app, ui, &snapshot, &mut sheet.family, &mut sheet.draft);
                    } else {
                        material_tables(app, ui, &snapshot, &mut sheet.draft, &sheet.library_search, sheet.library_tab);
                    }
                });
            });
    }
    if sheet.family_open {
        host.edge("worksheet_family", crate::ui_kit::Edge::Top)
            .default_size(300.)
            .resizable(true)
            .frame(host.edge_frame(Tokens::get().toolbar).inner_margin(8.))
            .show(ui, |ui| {
                ui.set_min_height(230.);
                family_table(app, ui, &snapshot, &mut sheet.family, &mut sheet.draft);
            });
    }
    let graphical = !sheet.source_mode;
    if graphical {
        let panel = crate::ui_kit::Container::new(ui.ctx(), crate::ui_kit::Surface::Panel);
        panel
            .edge("worksheet_project", crate::ui_kit::Edge::Left)
            .default_size(245.)
            .resizable(true)
            .frame(panel.edge_frame(Tokens::get().toolbar).inner_margin(10.))
            .show(ui, |ui| {
                let project_height = (ui.available_height() - 210.).max(100.);
                egui::ScrollArea::vertical().id_salt("worksheet_project_top").max_height(project_height).min_scrolled_height(project_height).show(
                    ui,
                    |ui| {
                        ui.heading(app.tr("ui.sheet.project"));

                        variants(app, ui, &snapshot, &mut sheet.draft);
                        ui.separator();
                        ui.small(app.tr(if sheet.inline.pick { "ui.sheet_inline.pick_hint" } else { "ui.sheet_inline.edit_hint" }));
                        if snapshot["calcpad"]["dirty"] == true {
                            ui.colored_label(Tokens::get().warning, app.tr("ui.sheet.stale"));
                        }
                        ui.collapsing(app.tr("ui.sheet_inline.cad_changes"), |ui| review(app, ui, &snapshot["cad_references"], &mut sheet.draft));
                        if !sheet.inline.error.is_empty() {
                            ui.colored_label(Tokens::get().warning, &sheet.inline.error);
                        }
                        if !sheet.draft.error.is_empty() {
                            ui.colored_label(Tokens::get().warning, &sheet.draft.error);
                        }
                    },
                );
                ui.add_space((ui.available_height() - 195.).max(0.));
                if let Some(name) = sheet.inline.reference_list(ui, &rows(&snapshot), app.tr("ui.sheet_inline.references")) {
                    sheet.scroll_to = Some(name);
                }
            });
        crate::ui_kit::FileView::page(ui, |ui| {
            let mut scroll = egui::ScrollArea::both().id_salt(("worksheet_paper_scroll", key)).auto_shrink([false, false]);
            if !sheet.initialized {
                scroll = scroll.scroll_offset(egui::Vec2::ZERO);
                sheet.initialized = true;
            }
            scroll.show(ui, |ui| {
                let rows = rows(&snapshot);
                if let Some(name) = sheet.inline.scroll_to.take() {
                    sheet.scroll_to = Some(name);
                }
                let extent = sheet.positions.values().fold(vec2(1080., 1480.), |a, p| vec2(a.x.max(p[0] + 500.), a.y.max(p[1] + 240.)));
                let height = extent.y.max(180. + rows.len() as f32 * 132. + (sheet.notes.len() + sheet.charts.len()) as f32 * 200.);
                let (paper, drop_target) = ui.allocate_exact_size(vec2(extent.x, height), Sense::hover());

                if let Some(payload) = drop_target.dnd_release_payload::<LibraryValue>() {
                    sheet.inline.commit(app);
                    match app.run("worksheet.formula", json!({"name":payload.name,"expression":payload.expression})) {
                        Ok(_reply) => {
                            if let Some(point) = ui.input(|i| i.pointer.hover_pos()) {
                                sheet.positions.insert(
                                    format!("formula:calcpad:{}", payload.name),
                                    [(point.x - paper.left()).max(0.), (point.y - paper.top()).max(0.)],
                                );
                                sheet.dirty = true;
                            }
                        }
                        Err(error) => sheet.draft.error = error,
                    }
                }
                let t = Tokens::get();
                let painter = ui.painter_at(paper);
                painter.rect_filled(paper, 0., t.field);
                for x in (0..(paper.width() / 24.).min(900.) as usize).map(|i| paper.left() + i as f32 * 24.) {
                    painter.line_segment([pos2(x, paper.top()), pos2(x, paper.bottom())], Stroke::new(0.25, t.border));
                }
                for y in (0..(paper.height() / 24.).min(2000.) as usize).map(|i| paper.top() + i as f32 * 24.) {
                    painter.line_segment([pos2(paper.left(), y), pos2(paper.right(), y)], Stroke::new(0.25, t.border));
                }
                painter.text(
                    paper.min + vec2(52., 30.),
                    Align2::LEFT_TOP,
                    snapshot["document"]["title"].as_str().unwrap_or(""),
                    FontId::proportional(26.),
                    t.text,
                );
                for (index, row) in rows.iter().take(1000).enumerate() {
                    let id = row["id"].as_str().unwrap_or("");
                    let position_id =
                        format!("formula:{}:{}", if id.starts_with("native:") { "native" } else { "calcpad" }, row["name"].as_str().unwrap_or(""));
                    if let Some(old) = sheet.positions.remove(id) {
                        sheet.positions.entry(position_id.clone()).or_insert(old);
                        sheet.dirty = true;
                    }

                    let rect = region(
                        &sheet,
                        &position_id,
                        paper.min,
                        vec2(52. + (index % 2) as f32 * 500., 110. + (index / 2) as f32 * 132.),
                        vec2(460., 110.),
                    );
                    if sheet.scroll_to.as_deref() == row["name"].as_str() {
                        ui.scroll_to_rect(rect, Some(egui::Align::Center));
                        sheet.scroll_to = None;
                    }
                    if !rect.intersects(ui.clip_rect().expand(160.)) {
                        continue;
                    }
                    if let Some(delta) = sheet.inline.row(app, ui, row, rect, key) {
                        let position = rect.min - paper.min + delta;
                        sheet.positions.insert(position_id, [position.x.clamp(0., 20000.), position.y.clamp(0., 20000.)]);
                        sheet.dirty = true;
                    }
                }
                let base = 132. + rows.len().div_ceil(2) as f32 * 132.;
                for index in 0..sheet.notes.len() {
                    let id = format!("note:{index}");
                    let rect = region(&sheet, &id, paper.min, vec2(52., base + index as f32 * 130.), vec2(460., 110.));
                    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                        if ui.add(egui::TextEdit::multiline(&mut sheet.notes[index]).frame(egui::Frame::NONE).desired_width(rect.width())).changed() {
                            sheet.dirty = true;
                        }
                    });
                }
                for index in 0..sheet.charts.len() {
                    let id = format!("chart:{index}");
                    let rect =
                        region(&sheet, &id, paper.min, vec2(52., base + sheet.notes.len() as f32 * 130. + index as f32 * 260.), vec2(700., 240.));
                    let response = ui.interact(rect, ui.id().with(&id), Sense::click_and_drag());
                    if sheet.inline.arrange || ui.input(|i| i.modifiers.alt) {
                        drag(&mut sheet, &id, rect, paper.min, &response);
                    }
                    chart(app, &painter, rect, &rows, &sheet.charts[index]);
                    let input = Rect::from_min_size(rect.left_bottom() - vec2(0., 28.), vec2(rect.width(), 28.));
                    ui.scope_builder(egui::UiBuilder::new().max_rect(input), |ui| {
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut sheet.charts[index])
                                    .hint_text(app.tr("ui.sheet.chart_hint"))
                                    .desired_width(rect.width()),
                            )
                            .changed()
                        {
                            sheet.dirty = true;
                        }
                    });
                }
                scenario::publish_handle("worksheet:paper", paper.min + vec2(500., 300.));
            });
        });
        sheet.inline.finish(app, ui, key);
        if sheet.new_formula {
            sheet.inline.add(app, ui.ctx(), key, &snapshot);
            sheet.new_formula = false;
        }
        if sheet.dirty && !ui.input(|i| i.pointer.any_down()) {
            let mut layout = app
                .session
                .doc
                .metadata
                .get("org.solidcraft.engineering-presentation")
                .and_then(|v| v.get("worksheet"))
                .filter(|v| v.is_object())
                .cloned()
                .unwrap_or_else(|| json!({}));
            layout["positions"] = json!(sheet.positions);
            layout["notes"] = json!(sheet.notes);
            layout["charts"] = json!(sheet.charts);
            layout["significant_digits"] = json!(sheet.inline.digits);
            let data = json!({"workspace":"worksheet","layout":layout});
            run(app, &mut sheet.draft, "worksheet.presentation", data);
            sheet.dirty = false;
        }
    }
    ui.ctx().data_mut(|d| d.insert_temp(key, sheet));
    graphical
}

fn rows(snapshot: &Value) -> Vec<Value> {
    let mut rows = Vec::new();
    for row in snapshot["rows"].as_array().into_iter().flatten() {
        let mut r = row.clone();
        r["id"] = json!(format!("native:{}", row["name"].as_str().unwrap_or("")));
        rows.push(r);
    }
    for (index, line) in snapshot["calcpad"]["source"].as_str().unwrap_or("").lines().take(2000).enumerate() {
        let Some((name, expression)) = line.split_once('=') else { continue };
        let name = name.trim();
        if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') || name.starts_with(char::is_numeric) {
            continue;
        }
        let result = &snapshot["calcpad"]["results"][name];
        rows.push(json!({"id":format!("calcpad:{index}"),"line":index,"name":name,"expression":expression.trim(),"value":if snapshot["calcpad"]["dirty"]==true {Value::Null}else{result["value"].clone()},"unit":result["unit"]}));
    }
    for row in &mut rows {
        let name = row["name"].as_str().unwrap_or("");
        let display = &snapshot["document"]["presentation"]["unit_values"][name];
        let current = if row["line"].is_number() {
            snapshot["display_units_current"] == true
        } else {
            display["raw_value"] == row["value"] && display["raw_unit"] == row["unit"]
        };
        if current && display["value"].is_number() {
            row["value"] = display["value"].clone();
            row["unit"] = display["unit"].clone();
        }
    }
    rows
}
fn region(sheet: &Sheet, id: &str, origin: egui::Pos2, default: egui::Vec2, size: egui::Vec2) -> Rect {
    let p =
        sheet.positions.get(id).copied().filter(|p| p.iter().all(|v| v.is_finite() && *v >= 0. && *v <= 20000.)).unwrap_or([default.x, default.y]);
    Rect::from_min_size(origin + vec2(p[0], p[1]), size)
}
fn drag(sheet: &mut Sheet, id: &str, rect: Rect, origin: egui::Pos2, response: &egui::Response) {
    if response.dragged() {
        let p = rect.min - origin + response.drag_delta();
        sheet.positions.insert(id.into(), [p.x.clamp(0., 20000.), p.y.clamp(0., 20000.)]);
        sheet.dirty = true;
    }
}

fn chart(app: &SolveApp, p: &egui::Painter, rect: Rect, rows: &[Value], names: &str) {
    let t = Tokens::get();
    p.rect_filled(rect, t.radius_md, t.toolbar);
    let names: Vec<_> = names.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    let values: Vec<_> = rows
        .iter()
        .filter(|r| names.contains(&r["name"].as_str().unwrap_or("")))
        .filter_map(|r| Some((r["name"].as_str()?, r["value"].as_f64()?, r["unit"].as_str().unwrap_or(""))))
        .collect();
    if values.is_empty() {
        p.text(rect.center(), Align2::CENTER_CENTER, app.tr("ui.sheet.chart_hint"), FontId::proportional(16.), t.text);
        return;
    }
    let unit = values[0].2;
    if values.iter().any(|v| v.2 != unit) {
        p.text(rect.center(), Align2::CENTER_CENTER, app.tr("ui.sheet.chart_units"), FontId::proportional(16.), t.warning);
        return;
    }
    let low = values.iter().map(|v| v.1).fold(0., f64::min);
    let high = values.iter().map(|v| v.1).fold(0., f64::max);
    let span = (high - low).max(1e-12);
    p.text(rect.min + vec2(12., 8.), Align2::LEFT_TOP, format!("{} [{unit}]", app.tr("ui.sheet.chart")), FontId::proportional(16.), t.text);
    let plot = rect.shrink2(vec2(35., 40.));
    let zero = plot.bottom() - ((-low) / span) as f32 * plot.height();
    p.line_segment([pos2(plot.left(), zero), pos2(plot.right(), zero)], Stroke::new(1., t.border));
    let width = plot.width() / values.len().max(1) as f32;
    for (i, (name, value, _)) in values.iter().enumerate() {
        let x = plot.left() + i as f32 * width;
        let y = plot.bottom() - ((*value - low) / span) as f32 * plot.height();
        let height = plot.bottom() - y;
        p.rect_filled(Rect::from_min_max(pos2(x + 8., y.min(zero)), pos2(x + width - 8., y.max(zero))), 2., t.accent);
        p.text(pos2(x + width / 2., plot.bottom() + 5.), Align2::CENTER_TOP, *name, FontId::proportional(12.), t.text);
        p.text(pos2(x + width / 2., plot.bottom() - height - 3.), Align2::CENTER_BOTTOM, format!("{value:.4}"), FontId::proportional(12.), t.text);
    }
}

fn material_tables(app: &mut SolveApp, ui: &mut egui::Ui, snapshot: &Value, draft: &mut Draft, search: &str, tab: usize) {
    if tab == 1 {
        let exists = snapshot["families"]["tables"]["Fastener"].is_object();
        let button = ui.add_enabled(!exists, egui::Button::new(app.tr("ui.mks_quality.fastener_family")));
        scenario::publish_handle("worksheet:library:fastener_family", button.rect.center());
        if button.clicked() {
            let columns = ["thread_diameter", "tap_drill_diameter", "clearance_hole_diameter"];
            let mut rows = serde_json::Map::new();
            for entry in snapshot["material_tables"]["entries"].as_array().into_iter().flatten().filter(|e| e["category"] == "Schrauben") {
                let Some(name) = entry["id"].as_str() else { continue };
                let mut values = serde_json::Map::new();
                for column in columns {
                    if let Some(value) = entry["values"].as_array().into_iter().flatten().find(|v| v["name"] == column) {
                        values.insert(column.into(), json!(format!("{} {}", value["value"], value["unit"].as_str().unwrap_or("mm"))));
                    }
                }
                if values.len() == columns.len() {
                    rows.insert(name.into(), Value::Object(values));
                }
            }
            if run(app, draft, "worksheet.family.set", json!({"table":{"name":"Fastener","columns":columns,"rows":rows}})) {
                run(app, draft, "worksheet.family.select", json!({"table":"Fastener","row":"M6"}));
            }
        }
        ui.small(app.tr("ui.mks_quality.family_note"));
    }
    ui.vertical(|ui| {
        for entry in snapshot["material_tables"]["entries"].as_array().into_iter().flatten() {
            if (tab == 0 && entry["category"] != "Werkstoffe")
                || (tab == 1 && entry["category"] != "Schrauben")
                || (tab == 2 && (entry["category"] == "Werkstoffe" || entry["category"] == "Schrauben"))
            {
                continue;
            }
            if !entry.to_string().to_lowercase().contains(&search.trim().to_lowercase()) {
                continue;
            }
            let id = entry["id"].as_str().unwrap_or("");
            let row =
                ui.collapsing(format!("{}: {} / {}", entry["category"].as_str().unwrap_or(""), entry["name"].as_str().unwrap_or(""), id), |ui| {
                    ui.small(entry["condition"].as_str().unwrap_or(""));
                    egui::Grid::new(("library_values", id)).striped(true).min_col_width(130.).show(ui, |ui| {
                        ui.label(app.tr("ui.library.property"));
                        ui.label(app.tr("ui.library.value"));
                        ui.label(app.tr("ui.library.unit"));
                        ui.end_row();
                        for value in entry["values"].as_array().into_iter().flatten() {
                            let name = format!("{id}_{}", value["name"].as_str().unwrap_or(""));
                            let number = value["value"].to_string().replace("e+", "*10^").replace('e', "*10^");
                            let label = ui.add(egui::Label::new(value["name"].as_str().unwrap_or("")).sense(egui::Sense::click_and_drag()));
                            label.dnd_set_drag_payload(LibraryValue {
                                name,
                                expression: format!("{number}*({})", value["unit"].as_str().unwrap_or("")),
                            });
                            scenario::publish_handle(
                                &format!("worksheet:library:value:{id}:{}", value["name"].as_str().unwrap_or("")),
                                label.rect.center(),
                            );
                            ui.label(value["value"].to_string());
                            ui.label(value["unit"].as_str().unwrap_or(""));
                            ui.end_row();
                        }
                    });
                    ui.hyperlink_to(app.tr("ui.sheet_materials.source"), entry["source"].as_str().unwrap_or(""));
                    let values: Vec<_> = entry["values"].as_array().into_iter().flatten().collect();
                    let names: Vec<_> = values.iter().map(|v| format!("{id}_{}", v["name"].as_str().unwrap_or(""))).collect();
                    let source = snapshot["calcpad"]["source"].as_str().unwrap_or("");
                    let exists = names.iter().any(|n| {
                        snapshot["calcpad"]["results"].get(n).is_some()
                            || snapshot["rows"].as_array().into_iter().flatten().any(|r| r["name"] == *n)
                            || source.lines().any(|l| l.split_once('=').is_some_and(|(lhs, _)| lhs.trim() == n))
                    });
                    let button = ui.add_enabled(!exists, egui::Button::new(app.tr("ui.sheet_materials.insert")));
                    scenario::publish_handle(&format!("worksheet:material:{id}"), button.rect.center());
                    if button.clicked() {
                        let mut next = format!(
                            "{source}\n'{} | {} | {}\n",
                            entry["name"].as_str().unwrap_or(""),
                            entry["condition"].as_str().unwrap_or(""),
                            entry["source"].as_str().unwrap_or("")
                        );
                        for (name, value) in names.iter().zip(values) {
                            let number = value["value"].to_string().replace("e+", "*10^").replace('e', "*10^");
                            next.push_str(&format!("{name} = {number}*({})\n", value["unit"].as_str().unwrap_or("")));
                        }
                        run(app, draft, "worksheet.calcpad.edit", json!({"source":next}));
                        if draft.error.is_empty() {
                            run(app, draft, "worksheet.calcpad.calculate", json!({}));
                        }
                    }
                });
            scenario::publish_handle(&format!("worksheet:material:open:{id}"), row.header_response.rect.center());
        }
    });
}

#[derive(Clone)]
struct LibraryValue {
    name: String,
    expression: String,
}

#[derive(Clone)]
struct FamilyDraft {
    name: String,
    columns: Vec<String>,
    rows: Vec<(String, Vec<String>)>,
}
impl Default for FamilyDraft {
    fn default() -> Self {
        Self {
            name: "FlangeBolt".into(),
            columns: ["thread_diameter", "thread_pitch", "tap_drill_diameter", "clearance_hole_diameter"].iter().map(|s| (*s).into()).collect(),
            rows: vec![("Custom".into(), vec!["0 mm".into(); 4])],
        }
    }
}
fn family_table(app: &mut SolveApp, ui: &mut egui::Ui, snapshot: &Value, f: &mut FamilyDraft, draft: &mut Draft) {
    ui.horizontal_wrapped(|ui| {
        ui.label(app.tr("ui.sheet_family.name"));
        let name = ui.text_edit_singleline(&mut f.name);
        scenario::publish_handle("worksheet:family:name", name.rect.center());
        if ui.button(app.tr("ui.sheet_family.add_row")).clicked() && f.rows.len() < 128 {
            f.rows.push((format!("Row{}", f.rows.len() + 1), vec!["0 mm".into(); f.columns.len()]));
        }
        if ui.button(app.tr("ui.sheet_family.add_column")).clicked() && f.columns.len() < 32 {
            f.columns.push(format!("value_{}", f.columns.len() + 1));
            for (_, r) in &mut f.rows {
                r.push("0 mm".into());
            }
        }
        for (name, table) in snapshot["families"]["tables"].as_object().into_iter().flatten() {
            let load = ui.button(name);
            scenario::publish_handle(&format!("worksheet:family:load:{name}"), load.rect.center());
            if load.clicked() {
                let t = &table["definition"];
                f.name = name.clone();
                f.columns = t["columns"].as_array().into_iter().flatten().filter_map(|c| c.as_str().map(str::to_owned)).collect();
                f.rows = t["rows"]
                    .as_object()
                    .into_iter()
                    .flatten()
                    .map(|(n, r)| (n.clone(), f.columns.iter().map(|c| r[c].as_str().unwrap_or("").into()).collect()))
                    .collect();
            }
        }
    });
    egui::ScrollArea::both().id_salt("family_grid_scroll").max_height(210.).show(ui, |ui| {
        egui::Grid::new("family_grid").min_col_width(220.).striped(true).show(ui, |ui| {
            ui.label(app.tr("ui.sheet_family.row"));
            for c in &mut f.columns {
                ui.add_sized(vec2(210., ui.spacing().interact_size.y), egui::TextEdit::singleline(c));
            }
            ui.end_row();
            let mut select = None;
            for (i, (name, values)) in f.rows.iter_mut().enumerate() {
                ui.add(egui::TextEdit::singleline(name).desired_width(100.));
                for (j, value) in values.iter_mut().enumerate() {
                    let field = ui.add_sized(vec2(210., ui.spacing().interact_size.y), egui::TextEdit::singleline(value));
                    scenario::publish_handle(&format!("worksheet:family:cell:{i}:{j}"), field.rect.center());
                }
                let choose = ui.button(app.tr("ui.sheet_family.select"));
                scenario::publish_handle(&format!("worksheet:family:select:{i}"), choose.rect.center());
                if choose.clicked() {
                    select = Some(name.clone());
                }
                ui.end_row();
            }
            if let Some(row) = select {
                let rows: BTreeMap<_, _> = f
                    .rows
                    .iter()
                    .map(|(name, values)| (name.clone(), f.columns.iter().cloned().zip(values.iter().cloned()).collect::<BTreeMap<_, _>>()))
                    .collect();
                if rows.len() != f.rows.len() {
                    draft.error = app.tr("ui.sheet_family.duplicate").into();
                    return;
                }
                if run(app, draft, "worksheet.family.set", json!({"table":{"name":f.name,"columns":f.columns,"rows":rows}})) {
                    run(app, draft, "worksheet.family.select", json!({"table":f.name,"row":row}));
                }
            }
        });
    });
    if !draft.error.is_empty() {
        ui.colored_label(Tokens::get().warning, &draft.error);
    }
}
