//! Direct formula/unit editing, shared by every calculation row on the paper.
use crate::{SolveApp, scenario, worksheet_math_ui as math};
use egui::{Rect, pos2, vec2};
use serde_json::{Value, json};
use std::collections::BTreeSet;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Formula,
    Unit,
}
#[derive(Clone)]
struct Active {
    row: Value,
    part: Part,
    field: String,
    widget: egui::Id,
    editor: math::Editor,
    original: String,
}
#[derive(Clone)]
pub struct State {
    pub scroll_to: Option<String>,
    pending_focus: bool,
    restore_focus_frames: u8,
    active: Option<Active>,
    catalog: Vec<Completion>,
    completion_index: usize,
    frame: Option<(math::MathBox, egui::Id)>,
    pub references: BTreeSet<String>,
    pub pick: bool,
    pub arrange: bool,
    pub digits: usize,
    pub error: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            scroll_to: None,
            pending_focus: false,
            restore_focus_frames: 0,
            active: None,
            catalog: Vec::new(),
            completion_index: 0,
            frame: None,
            references: BTreeSet::new(),
            pick: false,
            arrange: false,
            digits: 6,
            error: String::new(),
        }
    }
}
fn definition(row: &Value) -> String {
    format!("{} := {}", row["name"].as_str().unwrap_or(""), row["expression"].as_str().unwrap_or(""))
}
impl State {
    pub fn symbol(&mut self, ctx: &egui::Context, text: &str) {
        if let Some(active) = &mut self.active {
            active.editor.insert_text(text);
            self.restore_focus_frames = 3;
            ctx.memory_mut(|m| m.request_focus(active.widget));
        }
    }
    pub fn begin(&mut self, snapshot: &Value) {
        self.frame = None;
        self.catalog.clear();
        for entry in snapshot["material_tables"]["entries"].as_array().into_iter().flatten() {
            let class = entry["id"].as_str().unwrap_or("");
            for value in entry["values"].as_array().into_iter().flatten() {
                let property = value["name"].as_str().unwrap_or("");
                let variable = format!("{class}_{property}");
                let number = value["value"].to_string().replace("e+", "*10^").replace('e', "*10^");
                self.catalog.push(Completion {
                    label: format!("{class}.{property}"),
                    variable,
                    expression: format!("{number}*({})", value["unit"].as_str().unwrap_or("")),
                    source: entry["source"].as_str().unwrap_or("").into(),
                });
            }
        }
        for (class, table) in snapshot["families"]["tables"].as_object().into_iter().flatten() {
            let selected = table["active"].as_str().unwrap_or("");
            for property in table["definition"]["columns"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                self.catalog.push(Completion {
                    label: format!("{class}.{property}"),
                    variable: format!("{class}_{property}"),
                    expression: table["definition"]["rows"][selected][property].as_str().unwrap_or("").into(),
                    source: String::new(),
                });
            }
        }

        self.references = serde_json::from_value(snapshot["document"]["presentation"]["pinned_references"].clone()).unwrap_or_default();
        self.references.extend(
            snapshot["cad_references"]["references"].as_array().into_iter().flatten().filter_map(|r| r["source"].as_str().map(str::to_owned)),
        );
    }
    pub fn add(&mut self, app: &mut SolveApp, ctx: &egui::Context, key: egui::Id, snapshot: &Value) {
        self.commit(app);
        let mut index = 1;
        while snapshot["calcpad"]["results"].get(format!("x_{index}")).is_some() {
            index += 1;
        }
        match app.run("worksheet.formula", json!({"name":format!("x_{index}"),"expression":"0"})) {
            Ok(reply) => {
                let id = reply["id"].as_str().unwrap_or("");
                let row =
                    json!({"id":id,"name":reply["name"],"expression":"0","line":id.strip_prefix("calcpad:").and_then(|s|s.parse::<usize>().ok())});
                let field = format!("{}:{id}:formula", key.value());
                let widget = egui::Id::new(("worksheet_math", &field));
                let text = definition(&row);
                let mut editor = math::Editor::new(text.clone(), text.len());
                editor.anchor = text.len().saturating_sub(1);
                self.scroll_to = row["name"].as_str().map(str::to_owned);
                self.pending_focus = true;
                self.active = Some(Active { row, part: Part::Formula, field, widget, editor, original: text });
                ctx.memory_mut(|m| m.request_focus(widget));
            }
            Err(e) => self.error = e,
        }
    }
    pub fn commit(&mut self, app: &mut SolveApp) -> Option<Value> {
        let active = self.active.clone()?;
        if active.editor.text == active.original {
            self.active = None;
            return Some(active.row);
        }
        let result = if active.part == Part::Unit {
            app.run("worksheet.unit", json!({"source":active.row["name"],"unit":active.editor.text.trim()}))
        } else {
            let text = active.editor.text.trim();
            let pair = text.split_once(":=").or_else(|| text.split_once('≔')).or_else(|| text.split_once('='));
            let (name, expression) = pair.map(|(n, e)| (n.trim(), e.trim())).unwrap_or((active.row["name"].as_str().unwrap_or(""), text));
            let native = active.row["id"].as_str().is_some_and(|id| id.starts_with("native:"));
            let params = json!({"name":name,"expression":expression,"old_name":active.row["name"],"line":active.row.get("line"),"native":native});
            if native { app.run_heavy("worksheet.formula", params) } else { app.run("worksheet.formula", params) }
        };
        match result {
            Ok(reply) => {
                if let Some(error) = reply["error"].as_str() {
                    self.error = error.into();
                    return None;
                }
                self.error.clear();
                let mut row = active.row;
                if reply["id"].is_string() {
                    row["id"] = reply["id"].clone();
                    row["name"] = reply["name"].clone();
                    row["line"] = row["id"]
                        .as_str()
                        .and_then(|id| id.strip_prefix("calcpad:"))
                        .and_then(|s| s.parse::<usize>().ok())
                        .map_or(Value::Null, |v| json!(v));
                }
                self.active = None;
                Some(row)
            }
            Err(error) => {
                self.error = error;
                None
            }
        }
    }
    fn select(
        &mut self,
        app: &mut SolveApp,
        ui: &egui::Ui,
        row: &Value,
        part: Part,
        field: String,
        text: String,
        response: &egui::Response,
        math: &math::MathBox,
        at: egui::Pos2,
    ) {
        let same = self.active.as_ref().is_some_and(|a| a.field == field);
        if !same {
            self.commit(app);
            self.active = Some(Active {
                row: row.clone(),
                part,
                field,
                widget: response.id,
                editor: math::Editor::new(text.clone(), text.len()),
                original: text,
            });
        }
        if let Some(pointer) = response.interact_pointer_pos()
            && let Some(active) = &mut self.active
        {
            active.editor.mouse(
                math,
                pos2(pointer.x - at.x, pointer.y - at.y),
                ui.input(|i| i.modifiers.shift) || response.dragged(),
                response.double_clicked(),
            );
        }
        response.request_focus();
    }
    pub fn row(&mut self, app: &mut SolveApp, ui: &mut egui::Ui, row: &Value, rect: Rect, key: egui::Id) -> Option<egui::Vec2> {
        let id = row["id"].as_str().unwrap_or("");
        let name = row["name"].as_str().unwrap_or("");
        let at = rect.min + vec2(8., 8.);
        let field = format!("{}:{id}:formula", key.value());
        if self.pending_focus
            && let Some(active) = &self.active
            && active.field == field
        {
            ui.memory_mut(|m| m.request_focus(active.widget));
            self.pending_focus = false;
        }
        let active = self.active.as_ref().filter(|a| a.field == field);
        let text = active.map(|a| a.editor.text.clone()).unwrap_or_else(|| definition(row));
        let highlight = if self.references.contains(name) && text.starts_with(name) { Some(0..name.len()) } else { None };
        let (expression, response) = math::field(ui, &field, &text, at, 22., active.map(|a| &a.editor), highlight, false);
        scenario::publish_handle(&format!("worksheet:region:{id}"), response.rect.center());
        scenario::publish_handle(&format!("worksheet:formula:{name}"), response.rect.center());
        let editing_field = self.active.as_ref().is_some_and(|a| a.field == field);
        let moving = !self.pick && (self.arrange || ui.input(|i| i.modifiers.alt) || !editing_field);
        let gesture =
            response.clicked() || response.drag_started_by(egui::PointerButton::Primary) || response.dragged_by(egui::PointerButton::Primary);
        if gesture && (!moving || response.double_clicked()) {
            if self.pick {
                if response.clicked()
                    && let Some(pointer) = response.interact_pointer_pos()
                {
                    let offset = expression.nearest(pos2(pointer.x - at.x, pointer.y - at.y));
                    let range = math::identifier_range(&text, offset);
                    match app.run("worksheet.pin", json!({"source":&text[range]})) {
                        Ok(_) => self.error.clear(),
                        Err(e) => self.error = e,
                    }
                }
            } else if response.double_clicked() || editing_field {
                self.select(app, ui, row, Part::Formula, field.clone(), text.clone(), &response, &expression, at);
            }
        }
        if self.active.as_ref().is_some_and(|a| a.field == field) {
            self.frame = Some((expression.clone(), response.id));
        }
        let value_at = at + vec2(0., expression.size.y + 12.);
        let editing = self.active.as_ref().is_some_and(|a| a.row["id"] == id && a.editor.text != a.original);
        let value = if editing { "…".into() } else { row["value"].as_f64().map(|v| math::number(v, self.digits)).unwrap_or_else(|| "…".into()) };
        let result = math::layout(ui, &format!("= {value}"), 18.);

        scenario::publish_handle(&format!("worksheet:result:{name}"), value_at + result.size * 0.5);
        let unit_at = value_at + vec2(result.size.x + 4., 0.);
        let unit_field = format!("{}:{id}:unit", key.value());
        let active = self.active.as_ref().filter(|a| a.field == unit_field);
        let unit = active.map(|a| a.editor.text.clone()).unwrap_or_else(|| row["unit"].as_str().unwrap_or("").into());
        let unit = unit.replace('∕', "/").replace('·', "*");
        let unit_layout = math::layout(ui, &unit, 18.);
        let align = unit_layout.baseline.max(result.baseline);
        let value_at = value_at + vec2(0., align - result.baseline);
        let unit_at = unit_at + vec2(0., align - unit_layout.baseline);
        result.paint(ui, value_at, None, None, true);
        let (unit_math, unit_response) = math::field(ui, &unit_field, &unit, unit_at, 18., active.map(|a| &a.editor), None, true);
        scenario::publish_handle(&format!("worksheet:unit:{name}"), unit_response.rect.center());
        if !self.pick
            && (unit_response.double_clicked()
                || (active.is_some() && (unit_response.clicked() || unit_response.dragged_by(egui::PointerButton::Primary))))
        {
            self.select(app, ui, row, Part::Unit, unit_field.clone(), unit, &unit_response, &unit_math, unit_at);
        }
        if self.active.as_ref().is_some_and(|a| a.field == unit_field) {
            self.frame = Some((unit_math, unit_response.id));
        }

        if editing_field && !self.pick {
            let suggestions = self.suggestions();
            if !suggestions.is_empty() {
                let area = Rect::from_min_size(at + vec2(0., expression.size.y + 70.), vec2(440., 240.));
                ui.scope_builder(egui::UiBuilder::new().max_rect(area), |ui| {
                    ui.group(|ui| {
                        for (index, item) in
                            suggestions.iter().enumerate().skip((self.completion_index % suggestions.len()).saturating_sub(7)).take(8)
                        {
                            let choice = ui.selectable_label(index == self.completion_index % suggestions.len(), &item.label);
                            scenario::publish_handle(&format!("worksheet:completion:{}", item.label), choice.rect.center());
                            if choice.clicked() {
                                self.accept_completion(app, ui.ctx(), item);
                            }
                        }
                    });
                });
            }
        }
        if moving && response.dragged_by(egui::PointerButton::Primary) { Some(response.drag_delta()) } else { None }
    }
    pub fn finish(&mut self, app: &mut SolveApp, ui: &mut egui::Ui, key: egui::Id) {
        let Some((math, widget)) = self.frame.take() else { return };
        let suggestions = self.suggestions();
        if !suggestions.is_empty() && ui.memory(|m| m.has_focus(widget)) {
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
                self.completion_index = (self.completion_index + 1) % suggestions.len();
            }
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
                self.completion_index = (self.completion_index + suggestions.len() - 1) % suggestions.len();
            }
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)) {
                self.accept_completion(app, ui.ctx(), &suggestions[self.completion_index % suggestions.len()]);
                return;
            }
        }

        let Some(active) = &mut self.active else { return };
        if active.widget != widget {
            return;
        }
        if self.restore_focus_frames > 0 {
            ui.memory_mut(|m| m.request_focus(widget));
            self.restore_focus_frames -= 1;
            ui.ctx().request_repaint();
        }
        let input = active.editor.input(ui, widget, &math);
        if input.cancel {
            self.active = None;
            ui.memory_mut(|m| m.surrender_focus(widget));
            return;
        }
        if input.changed {
            self.error.clear();
        }
        if input.evaluate || input.finish {
            let part = active.part;
            if let Some(row) = self.commit(app) {
                self.active = None;
                if input.evaluate && part == Part::Formula {
                    let now = app.session.execute("worksheet.read", &json!({})).unwrap_or_default();
                    let name = row["name"].as_str().unwrap_or("");
                    let layout = &now["document"]["presentation"];
                    let unit = layout["unit_values"][name]["unit"]
                        .as_str()
                        .or_else(|| now["calcpad"]["results"][name]["unit"].as_str())
                        .or_else(|| row["unit"].as_str())
                        .unwrap_or("")
                        .to_owned();
                    let field = format!("{}:{}:unit", key.value(), row["id"].as_str().unwrap_or(""));
                    let widget = egui::Id::new(("worksheet_math", &field));
                    let mut editor = math::Editor::new(unit.clone(), unit.len());
                    editor.select_all();
                    self.active = Some(Active { row, part: Part::Unit, field, widget, editor, original: unit });
                    ui.memory_mut(|m| m.request_focus(widget));
                } else {
                    ui.memory_mut(|m| m.surrender_focus(widget));
                }
            }
            ui.ctx().request_repaint();
        }
    }
    pub fn reference_list(&self, ui: &mut egui::Ui, rows: &[Value], title: &str) -> Option<String> {
        ui.separator();
        ui.strong(title);
        let mut selected = None;
        egui::ScrollArea::vertical().id_salt("worksheet_pinned_references").max_height(175.).show(ui, |ui| {
            for name in &self.references {
                let row = rows.iter().find(|r| r["name"] == *name);
                let value = row.and_then(|r| r["value"].as_f64()).map(|v| math::number(v, self.digits)).unwrap_or_else(|| "…".into());
                let unit = row.and_then(|r| r["unit"].as_str()).unwrap_or("");
                let label = math::layout(ui, &format!("{name} = {value} {unit}"), 16.);
                let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), label.size.y + 8.), egui::Sense::click());
                label.paint(ui, rect.min, None, None, true);
                scenario::publish_handle(&format!("worksheet:pinned:{name}"), rect.center());
                if response.clicked() {
                    selected = Some(name.clone());
                }
            }
        });
        selected
    }
}

#[derive(Clone)]
struct Completion {
    label: String,
    variable: String,
    expression: String,
    source: String,
}
impl State {
    fn completion_token(&self) -> Option<(usize, String)> {
        let a = self.active.as_ref()?;
        if a.part != Part::Formula {
            return None;
        }
        let before = &a.editor.text[..a.editor.cursor];
        let start = before.char_indices().rev().find(|(_, c)| !c.is_alphanumeric() && *c != '_' && *c != '.').map_or(0, |(i, c)| i + c.len_utf8());
        let token = &before[start..];
        if !token.contains('.') {
            return None;
        }
        Some((start, token.into()))
    }
    fn suggestions(&self) -> Vec<Completion> {
        let Some((_, token)) = self.completion_token() else { return Vec::new() };
        let query = token.to_lowercase();
        let (class, property) = query.split_once('.').unwrap_or(("", ""));
        self.catalog
            .iter()
            .filter(|c| c.label.split_once('.').is_some_and(|(a, b)| a.to_lowercase().starts_with(class) && b.to_lowercase().starts_with(property)))
            .take(128)
            .cloned()
            .collect()
    }
    fn accept_completion(&mut self, app: &mut SolveApp, ctx: &egui::Context, item: &Completion) {
        let Some((start, _)) = self.completion_token() else { return };
        if item.expression.is_empty() {
            self.error = app.tr("ui.library.choose_row").into();
            return;
        }
        let Some(mut active) = self.active.clone() else { return };
        let state = app.session.execute("worksheet.read", &json!({})).unwrap_or_default();
        let source = state["calcpad"]["source"].as_str().unwrap_or("");
        if !source.lines().any(|l| l.split_once('=').is_some_and(|(name, _)| name.trim() == item.variable)) {
            if active.row["id"].as_str().is_some_and(|id| id.starts_with("native:")) {
                self.error = app.tr("ui.library.local_formula").into();
                return;
            }
            let source = format!("'{} | {}\n{} = {}\n{source}", item.label, item.source, item.variable, item.expression);
            if let Err(e) = app.run("worksheet.calcpad.edit", json!({"source":source})) {
                self.error = e;
                return;
            }
            if let Err(e) = app.run("worksheet.calcpad.calculate", json!({})) {
                self.error = e;
                return;
            }
            if let Some(line) =
                source.lines().position(|l| l.split_once('=').is_some_and(|(n, _)| n.trim() == active.row["name"].as_str().unwrap_or("")))
            {
                let old = active.row["id"].as_str().unwrap_or("").to_owned();
                let id = format!("calcpad:{line}");
                active.field = active.field.replace(&format!(":{old}:"), &format!(":{id}:"));
                active.widget = egui::Id::new(("worksheet_math", &active.field));
                active.row["id"] = json!(id);
                active.row["line"] = json!(line);
            }
        }
        active.editor.anchor = start;
        active.editor.insert_text(&item.variable);
        ctx.memory_mut(|m| m.request_focus(active.widget));
        self.active = Some(active);
        self.pending_focus = true;
        self.restore_focus_frames = 3;
        self.error.clear();
        ctx.request_repaint();
    }
}
