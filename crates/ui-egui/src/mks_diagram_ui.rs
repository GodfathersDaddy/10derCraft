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
    pub fn new_port(&mut self, kind: &str) {
        self.new_source();
        if let Some((_, source)) = &mut self.edit {
            source["kind"] = json!(kind);
            source["name"] = json!(kind);
            source["data_type"] = json!("real");
            source["link"] = json!("");
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
        let port = source["kind"].as_str().is_some_and(|kind| kind != "motion");
        let closed = crate::ui_kit::Window::new(
            ui.id().with("independent_source"),
            "drive",
            app.tr(if port { "ui.signal.settings" } else { "ui.mks_wire.source_title" }),
        )
        .cards(2.5)
        .show_body(ui.ctx(), |ui| {
            if port {
                let mut kind = source["kind"].as_str().unwrap_or("input").to_owned();
                egui::ComboBox::from_id_salt("signal_kind").selected_text(app.tr(&format!("ui.signal.{kind}"))).show_ui(ui, |ui| {
                    for value in ["input", "output", "link"] {
                        ui.selectable_value(&mut kind, value.to_owned(), app.tr(&format!("ui.signal.{value}")));
                    }
                });
                source["kind"] = json!(kind);
                let mut dtype = source["data_type"].as_str().unwrap_or("real").to_owned();
                egui::ComboBox::from_id_salt("signal_type").selected_text(&dtype).show_ui(ui, |ui| {
                    for value in ["real", "integer", "boolean", "vector3"] {
                        if ui.selectable_value(&mut dtype, value.to_owned(), value).changed() {
                            source["value"] = Value::Null;
                            source["upstream"] = Value::Null;
                            source["sensor_joint"] = Value::Null;
                        }
                    }
                });
                source["data_type"] = json!(dtype);
                ui.horizontal_wrapped(|ui| {
                    ui.label(app.tr("ui.signal.link_name"));
                    let mut link = source["link"].as_str().unwrap_or("").to_owned();
                    ui.text_edit_singleline(&mut link);
                    source["link"] = json!(link);
                });
                if source["kind"] != "link" {
                    let mut upstream = source["upstream"].as_u64();
                    egui::ComboBox::from_id_salt("signal_upstream")
                        .selected_text(upstream.map(|id| format!("#{id}")).unwrap_or_else(|| app.tr("ui.signal.unconnected").to_owned()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut upstream, None, app.tr("ui.signal.unconnected"));
                            for (key, node) in d.graph["sources"].as_object().into_iter().flatten() {
                                let candidate = key.parse::<u64>().ok();
                                if candidate != id
                                    && node["dimension"] == source["dimension"]
                                    && node["data_type"].as_str().unwrap_or("real") == source["data_type"].as_str().unwrap_or("real")
                                {
                                    ui.selectable_value(&mut upstream, candidate, node["name"].as_str().unwrap_or(key));
                                }
                            }
                        });
                    if upstream != source["upstream"].as_u64() {
                        source["upstream"] = json!(upstream);
                        source["value"] = Value::Null;
                        source["sensor_joint"] = Value::Null;
                    }
                    let mut constant = !source["value"].is_null();
                    if ui.checkbox(&mut constant, app.tr("ui.signal.constant")).changed() {
                        source["value"] = if constant {
                            match source["data_type"].as_str() {
                                Some("boolean") => json!(false),
                                Some("vector3") => json!([0., 0., 0.]),
                                _ => json!(0),
                            }
                        } else {
                            Value::Null
                        };
                        source["upstream"] = Value::Null;
                        source["sensor_joint"] = Value::Null;
                    }
                    if constant {
                        match source["data_type"].as_str() {
                            Some("boolean") => {
                                let mut v = source["value"].as_bool().unwrap_or(false);
                                ui.checkbox(&mut v, app.tr("ui.signal.value"));
                                source["value"] = json!(v);
                            }
                            Some("integer") => {
                                let mut v = source["value"].as_i64().unwrap_or(0);
                                ui.add(egui::DragValue::new(&mut v));
                                source["value"] = json!(v);
                            }
                            Some("vector3") => {
                                let mut v: [f64; 3] = serde_json::from_value(source["value"].clone()).unwrap_or([0.; 3]);
                                ui.horizontal_wrapped(|ui| {
                                    for value in &mut v {
                                        ui.add(egui::DragValue::new(value));
                                    }
                                });
                                source["value"] = json!(v);
                            }
                            _ => {
                                let mut v = source["value"].as_f64().unwrap_or(0.);
                                ui.add(egui::DragValue::new(&mut v));
                                source["value"] = json!(v);
                            }
                        }
                        ui.small(app.tr("ui.signal.si"));
                    }
                    let mut sensor = source["sensor_joint"].as_u64();
                    egui::ComboBox::from_id_salt("signal_sensor")
                        .selected_text(sensor.map(|id| format!("Joint {id}")).unwrap_or_else(|| app.tr("ui.signal.sensor").to_owned()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut sensor, None, app.tr("ui.signal.unconnected"));
                            for joint in app.session.doc.assembly.joints.iter().filter(|j| {
                                !j.suppressed
                                    && matches!(
                                        j.kind,
                                        solidcraft_engine::doc::joints::JointKind::Revolute | solidcraft_engine::doc::joints::JointKind::Slider
                                    )
                            }) {
                                ui.selectable_value(&mut sensor, Some(joint.id), &joint.name);
                            }
                        });
                    if sensor != source["sensor_joint"].as_u64() {
                        source["sensor_joint"] = json!(sensor);
                        source["value"] = Value::Null;
                        source["upstream"] = Value::Null;
                    }
                }
            }
            for (key, label) in [("name", "ui.mks_wire.name"), ("displacement", "ui.mks_wire.amplitude")] {
                if port && key == "displacement" {
                    continue;
                }

                ui.horizontal_wrapped(|ui| {
                    ui.label(app.tr(label));
                    let mut text = source[key].as_str().unwrap_or("").to_owned();
                    let r = ui.add(egui::TextEdit::singleline(&mut text).desired_width(300.));
                    scenario::publish_handle(&format!("mks:independent:{key}"), r.rect.center());
                    source[key] = json!(text);
                });
            }
            ui.horizontal_wrapped(|ui| {
                ui.label(app.tr("ui.mks_wire.dimension"));
                for (value, label) in [
                    ("angle", "ui.mks_wire.angle"),
                    ("length", "ui.mks_wire.length"),
                    ("force", "ui.signal.force"),
                    ("torque", "ui.signal.torque"),
                    ("scalar", "ui.signal.scalar"),
                    ("angular_velocity", "ui.signal.angular_velocity"),
                    ("linear_velocity", "ui.signal.linear_velocity"),
                ] {
                    if !port && !["angle", "length"].contains(&value) {
                        continue;
                    }
                    let r = ui.selectable_label(source["dimension"] == value, app.tr(label));
                    scenario::publish_handle(&format!("mks:independent:dimension:{value}"), r.rect.center());
                    if r.clicked() {
                        source["dimension"] = json!(value);
                    }
                }
            });
            if !port {
                ui.horizontal_wrapped(|ui| {
                    ui.label(app.tr("ui.mks_graph.ramp"));
                    let mut ramp = source["ramp_s"].as_f64().unwrap_or(0.5);
                    ui.add(egui::DragValue::new(&mut ramp).range(0.001..=10.).speed(0.05));
                    source["ramp_s"] = json!(ramp);
                });
                crate::mks_source_ui::fields(app, ui, &mut source, id.unwrap_or(u64::MAX), true);
                crate::mks_source_ui::preview(app, ui, &source, duration);
            }
            ui.add(egui::Label::new(app.tr(if port { "ui.signal.note" } else { "ui.mks_wire.independent_note" })).wrap());
            if port {
                if let Some(id) = id {
                    let read_key = ui.id().with(("signal_read", id));
                    let mut time = ui.ctx().data(|d| d.get_temp::<f64>(read_key.with("time"))).unwrap_or(duration.min(0.5));
                    ui.horizontal_wrapped(|ui| {
                        ui.add(egui::DragValue::new(&mut time).range(0. ..=duration).suffix(" s"));
                        if ui.button(app.tr("ui.signal.read")).clicked() {
                            let text = match app.session.execute("mks.diagram.sample", &json!({"id":id,"time_s":time})) {
                                Ok(value) => format!("{}", value["value"]),
                                Err(error) => error.to_string(),
                            };
                            ui.ctx().data_mut(|d| d.insert_temp(read_key, text));
                        }
                    });
                    ui.ctx().data_mut(|d| d.insert_temp(read_key.with("time"), time));
                    if let Some(text) = ui.ctx().data(|d| d.get_temp::<String>(read_key)) {
                        ui.label(text);
                    }
                }
            }
            ui.horizontal_wrapped(|ui| {
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
