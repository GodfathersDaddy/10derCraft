//! Independent sources and explicit wiring are backed by registered engine commands.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::{Value, json};
#[derive(Clone, Default)]
pub struct Diagram {
    pub graph: Value,
    pub wiring: bool,
    pub dragging: bool,
    pub wire: Option<u64>,
    pub measure: Option<u64>,
    pub selected_wire: Option<u64>,
    pub selected_measure: Option<u64>,
    pub remove_measure: Option<u64>,
    pub edit: Option<(Option<u64>, Value)>,
    pub target: Option<(u64, u64)>,
    pub request: Option<(String, Value)>,
    pub error: String,
    pub playback_speed: f64,
    pub repeat: bool,
    new_target: Option<u64>,
    revision: Option<u64>,
}
impl Diagram {
    pub fn sync(&mut self, app: &mut SolveApp) {
        if self.revision != Some(app.session.revision) {
            match app.session.execute("mks.diagram.read", &json!({})) {
                Ok(reply) => {
                    self.graph = reply["graph"].clone();
                    self.revision = Some(app.session.revision);
                }
                Err(e) => self.error = e.to_string(),
            }
        }
    }
    pub fn new_source(&mut self) {
        self.new_target = None;
        self.edit =
            Some((None, json!({"name":"Motion","dimension":"angle","displacement":"30 deg","profile":"quintic","ramp_s":0.5,"frequency_hz":1.})));
        self.error.clear();
    }
    pub fn new_for_joint(&mut self, joint: u64, slider: bool) {
        self.new_source();
        self.new_target = Some(joint);
        if let Some((_, source)) = &mut self.edit {
            source["dimension"] = json!(if slider { "length" } else { "angle" });
            source["displacement"] = json!(if slider { "50 mm" } else { "30 deg" });
        }
    }
    pub fn linked(&self, id: u64) -> Option<u64> {
        self.graph["inputs"][id.to_string()].as_u64()
    }
    pub fn managed(&self, id: u64) -> bool {
        self.graph["inputs"].get(id.to_string()).is_some()
    }
    pub fn open(&mut self, id: u64) {
        self.new_target = None;
        if let Some(v) = self.graph["sources"].get(id.to_string()) {
            self.edit = Some((Some(id), v.clone()));
            self.error.clear();
        }
    }
    pub fn input(&mut self, joint: u64) {
        if let Some(source) = self.wire.take() {
            self.target = Some((source, joint));
        }
    }
    pub fn disconnect(&mut self, joint: u64) {
        self.request = Some(("mks.diagram.disconnect".into(), json!({"joint":joint})));
    }
}
pub fn ribbon(app: &SolveApp, ui: &mut egui::Ui, d: &mut Diagram) {
    let b = crate::toolbar::engineering_action(ui, "joint", app.tr("ui.mks_wire.mode"), d.wiring);
    scenario::publish_handle("mks:wire:mode", b.rect.center());
    if b.clicked() {
        d.wiring = !d.wiring;
        d.wire = None;
        d.measure = None;
    }
    if let Some(id) = d.selected_wire {
        let b = ui.button(format!("{} · Joint {id}", app.tr("ui.mks_wire.disconnect")));
        scenario::publish_handle("mks:wire:delete", b.rect.center());
        if b.clicked() {
            d.disconnect(id);
            d.selected_wire = None;
        }
    }
    if let Some(id) = d.selected_measure {
        let b = ui.button(app.tr("ui.mks_cad.disconnect_measure"));
        scenario::publish_handle("mks:wire:delete_measure", b.rect.center());
        if b.clicked() {
            d.remove_measure = Some(id);
            d.selected_measure = None;
        }
    }
}
/// Fixed-height guidance keeps graph ports stationary during a drag, including
/// narrow windows where conditional controls would wrap the ribbon.
pub fn hint(app: &SolveApp, ui: &mut egui::Ui, d: &mut Diagram) {
    ui.horizontal(|ui| {
        let text = app.tr(if d.wire.is_some() {
            "ui.mks_wire.target_hint"
        } else if d.measure.is_some() {
            "ui.mks_wire.scope_hint"
        } else {
            "ui.mks_wire.source_hint"
        });
        ui.add_sized([(ui.available_width() - 100.).max(0.), Tokens::get().control_height], egui::Label::new(text).truncate());
        let b = ui.add_enabled(d.wire.is_some() || d.measure.is_some(), egui::Button::new(app.tr("ui.mks_wire.cancel")));
        scenario::publish_handle("mks:wire:cancel", b.rect.center());
        if b.clicked() {
            d.wire = None;
            d.measure = None;
            d.dragging = false;
        }
    });
}
pub fn show(app: &mut SolveApp, ui: &mut egui::Ui, d: &mut Diagram, duration: f64) {
    if d.edit.is_none()
        && d.target.is_none()
        && !ui.ctx().egui_wants_keyboard_input()
        && ui.input(|i| i.key_pressed(egui::Key::Delete))
        && let Some(id) = d.selected_wire.take()
    {
        d.disconnect(id);
    }
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        d.dragging = false;
        d.wire = None;
        d.measure = None;
        d.target = None;
    }
    if let Some((source, joint)) = d.target {
        let closed = crate::ui_kit::Window::new(ui.id().with("connect_source"), "joint", app.tr("ui.mks_wire.connect_title")).cards(2.).show_body(
            ui.ctx(),
            |ui| {
                ui.label(format!("{} → Joint {joint}", d.graph["sources"][source.to_string()]["name"].as_str().unwrap_or("")));
                ui.add(egui::Label::new(app.tr("ui.mks_wire.replace_note")).wrap());
                let b = ui.button(app.tr("ui.mks_wire.connect"));
                scenario::publish_handle("mks:wire:confirm", b.rect.center());
                if b.clicked() {
                    d.request = Some(("mks.diagram.connect".into(), json!({"source":source,"joint":joint,"replace":true})));
                }
                if !d.error.is_empty() {
                    ui.colored_label(Tokens::get().warning, &d.error);
                }
            },
        );
        if closed {
            d.target = None;
            d.error.clear();
        }
    }
    if let Some((id, mut source)) = d.edit.clone() {
        let mut close = false;
        let closed = crate::ui_kit::Window::new(ui.id().with("independent_source"), "drive", app.tr("ui.mks_wire.source_title"))
            .cards(2.5)
            .show_body(ui.ctx(), |ui| {
                for (key, label) in [("name", "ui.mks_wire.name"), ("displacement", "ui.mks_wire.amplitude")] {
                    ui.horizontal(|ui| {
                        ui.label(app.tr(label));
                        let mut text = source[key].as_str().unwrap_or("").to_owned();
                        let r = ui.add(egui::TextEdit::singleline(&mut text).desired_width(300.));
                        scenario::publish_handle(&format!("mks:independent:{key}"), r.rect.center());
                        source[key] = json!(text);
                    });
                }
                ui.horizontal(|ui| {
                    ui.label(app.tr("ui.mks_wire.dimension"));
                    for (value, label) in [("angle", "ui.mks_wire.angle"), ("length", "ui.mks_wire.length")] {
                        let r = ui.selectable_label(source["dimension"] == value, app.tr(label));
                        scenario::publish_handle(&format!("mks:independent:dimension:{value}"), r.rect.center());
                        if r.clicked() {
                            source["dimension"] = json!(value);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(app.tr("ui.mks_graph.ramp"));
                    let mut ramp = source["ramp_s"].as_f64().unwrap_or(0.5);
                    ui.add(egui::DragValue::new(&mut ramp).range(0.001..=10.).speed(0.05));
                    source["ramp_s"] = json!(ramp);
                });
                crate::mks_source_ui::fields(app, ui, &mut source, id.unwrap_or(u64::MAX), true);
                crate::mks_source_ui::preview(app, ui, &source, duration);
                ui.add(egui::Label::new(app.tr("ui.mks_wire.independent_note")).wrap());
                ui.horizontal(|ui| {
                    let b = ui.button(app.tr("ui.mks_wire.save"));
                    scenario::publish_handle("mks:independent:save", b.rect.center());
                    if b.clicked() {
                        d.request = Some(("mks.diagram.source".into(), json!({"id":id,"source":source})));
                    }
                    if let Some(id) = id {
                        let b = ui.button(app.tr("ui.mks_wire.remove"));
                        scenario::publish_handle("mks:independent:remove", b.rect.center());
                        if b.clicked() {
                            d.request = Some(("mks.diagram.remove".into(), json!({"id":id})));
                        }
                    }
                    if ui.button(app.tr("ui.mks_wire.cancel")).clicked() {
                        close = true;
                    }
                });
                if !d.error.is_empty() {
                    ui.colored_label(Tokens::get().warning, &d.error);
                }
            });
        if closed || close {
            d.new_target = None;
        }
        d.edit = if closed || close { None } else { Some((id, source)) };
    }
    if let Some((cmd, p)) = d.request.take() {
        match app.run(&cmd, p) {
            Ok(reply) => {
                d.graph = reply["graph"].clone();
                d.revision = Some(app.session.revision);
                d.edit = None;
                d.target = None;
                if cmd == "mks.diagram.source"
                    && let Some(joint) = d.new_target.take()
                    && let Some(source) = reply["source"].as_u64()
                {
                    d.target = Some((source, joint));
                }
                d.error.clear();
            }
            Err(e) => d.error = e,
        }
    }
    if !d.error.is_empty() && d.edit.is_none() && d.target.is_none() {
        ui.colored_label(Tokens::get().warning, &d.error);
    }
}
