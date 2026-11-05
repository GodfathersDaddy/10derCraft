//! CAD bodies, native joints, prescribed-motion signals and force-sensor scopes.
//! Commands own model reduction and integration; this module only edits job inputs.
use crate::{SolveApp, scenario, theme::Tokens};
use egui::{Align2, FontId, Rect, Sense, Stroke, pos2, vec2};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
struct Draft {
    settings: Value,
    last_settings: Value,
    selected: Option<u64>,
    sensors: BTreeSet<u64>,
    unscoped: BTreeSet<u64>,
    component: usize,
    preview: bool,
    playing: bool,
    sample: usize,
    playhead: f64,
    clock: f64,
    error: String,
    positions: BTreeMap<String, [f32; 2]>,
    layout_dirty: bool,
    library: String,
    scope_visible: bool,
    inspector: bool,
    selected_body: Option<String>,
    last_layout: Value,
    joint_tab: usize,
    chart_kind: usize,
    pending_target: Option<String>,
    quick_insert: bool,
    diagram: crate::mks_diagram_ui::Diagram,
    shared_control: bool,
    control_open: bool,
    outputs: BTreeMap<u64, BTreeSet<usize>>,
    insert: Option<crate::mks_library_ui::Insert>,
    scene: Rect,
    running: Option<std::sync::Arc<crate::mks_run_ui::Running>>,
}

fn comparable_settings(settings: &Value, diagram: &crate::mks_diagram_ui::Diagram) -> Value {
    let mut value = settings.clone();
    if let Some(drivers) = value["drivers"].as_array_mut() {
        drivers.retain(|d| !d["joint"].as_u64().is_some_and(|id| diagram.managed(id)));
        drivers.sort_by_key(|d| d["joint"].as_u64().unwrap_or(0));
    }
    display_numbers(&mut value);
    value
}
pub fn show(app: &mut SolveApp, ui: &mut egui::Ui, model: &Value) {
    ui.painter().rect_filled(ui.available_rect_before_wrap(), 0., Tokens::get().toolbar);
    let key = egui::Id::new(("mks_signal_graph", app.docs.active, app.session.path.clone()));
    let cache_key = key.with("state");
    let cache = ui.ctx().data(|d| d.get_temp::<(u64, std::sync::Arc<Value>)>(cache_key));
    let state = match cache {
        Some((revision, value)) if revision == app.session.revision => value,
        _ => {
            let value = std::sync::Arc::new(app.session.execute("mks.job.state", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()})));
            ui.ctx().data_mut(|d| d.insert_temp(cache_key, (app.session.revision, value.clone())));
            value
        }
    };
    let blocks = model["blocks"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let joints = model["joints"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let layout = app.session.doc.metadata.get("org.solidcraft.engineering-presentation").and_then(|v| v.get("mks")).cloned().unwrap_or(Value::Null);
    let mut draft = ui.ctx().data(|d|d.get_temp::<Draft>(key)).unwrap_or_else(|| {
        let stored = &state["data"]["job"]["settings"];
        Draft {running:None,last_settings:stored.clone(),settings:if layout["settings"].is_object(){layout["settings"].clone()}else if stored.is_object() {stored.clone()} else {json!({"duration_s":2.,"steps":1000,"samples":201,"gravity_m_s2":[0.,0.,-9.81],"allow_assumed_density":false,"port_loads":[],"drivers":[]})},
            selected:None,sensors:serde_json::from_value(layout["sensors"].clone()).unwrap_or_else(|_|joints.first().and_then(|j|j["id"].as_u64()).into_iter().collect()),unscoped:serde_json::from_value(layout["unscoped_sensors"].clone()).unwrap_or_default(),component:3,preview:false,playing:false,sample:0,playhead:0.,clock:-1.,error:String::new(),positions:serde_json::from_value(layout["positions"].clone()).unwrap_or_default(),layout_dirty:false,library:String::new(),scope_visible:false,inspector:false,selected_body:None,last_layout:layout.clone(),joint_tab:0,chart_kind:0,pending_target:None,quick_insert:false,diagram:crate::mks_diagram_ui::Diagram::default(),shared_control:layout["shared_control"]==true,control_open:false,outputs:serde_json::from_value(layout["outputs"].clone()).unwrap_or_default(),insert:None,scene:Rect::ZERO}
    });
    if layout != draft.last_layout && !draft.layout_dirty {
        draft.positions = serde_json::from_value(layout["positions"].clone()).unwrap_or_default();
        if let Ok(sensors) = serde_json::from_value(layout["sensors"].clone()) {
            draft.sensors = sensors;
        }
        draft.unscoped = serde_json::from_value(layout["unscoped_sensors"].clone()).unwrap_or_default();
        draft.shared_control = layout["shared_control"] == true;
        draft.outputs = serde_json::from_value(layout["outputs"].clone()).unwrap_or_default();
        draft.last_layout = layout.clone();
    }
    let stored = &state["data"]["job"]["settings"];
    if stored.is_object() && *stored != draft.last_settings {
        if draft.settings == draft.last_settings || draft.settings == *stored {
            draft.settings = stored.clone();
        }
        draft.last_settings = stored.clone();
    }
    draft.diagram.sync(app);
    if let Some(running) = &draft.running {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));
        if let Some(outcome) = running.take() {
            let same_inputs = running.inputs == draft.settings;
            draft.running = None;
            let outcome = if same_inputs { outcome } else { Err(app.tr("ui.mks_speed.changed").into()) };
            match outcome.and_then(|result| app.run("mks.job.commit", json!({"result":result}))) {
                Ok(_) => {
                    if let Ok(current) = app.session.execute("mks.job.state", &json!({})) {
                        draft.settings = current["data"]["job"]["settings"].clone();
                        draft.last_settings = draft.settings.clone();
                    }
                    draft.error.clear();
                    draft.selected = None;
                    draft.preview = true;
                    draft.playing = true;
                    draft.sample = 0;
                    draft.playhead = 0.;
                    draft.clock = -1.;
                    ui.ctx().request_repaint();
                }
                Err(e) => draft.error = e,
            }
        }
    }
    let before_settings = draft.settings.clone();
    let host = crate::ui_kit::Container::new(ui.ctx(), crate::ui_kit::Surface::Chrome);
    host.edge("mks_ribbon", crate::ui_kit::Edge::Top).frame(host.edge_frame(Tokens::get().toolbar).inner_margin(Tokens::get().space_8)).show(
        ui,
        |ui| {
            ui.set_min_height(58.);
            ui.style_mut().always_scroll_the_only_direction = true;
            egui::ScrollArea::horizontal()
                .id_salt("mks_ribbon_scroll")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("MKS");
                        ui.label(app.tr("ui.mks_graph.duration"));
                        let mut duration = draft.settings["duration_s"].as_f64().unwrap_or(2.);
                        if ui.add(egui::DragValue::new(&mut duration).range(0.001..=10.).speed(0.1)).changed() {
                            draft.settings["duration_s"] = json!(duration);
                        }
                        let busy = draft.running.is_some();
                        let run =
                            ui.add_enabled_ui(!busy, |ui| crate::toolbar::engineering_action(ui, "drive", app.tr("ui.mks_graph.run"), false)).inner;
                        scenario::publish_handle("mks:graph:run", run.rect.center());
                        let recompute = ui
                            .add_enabled_ui(!busy, |ui| crate::toolbar::engineering_action(ui, "update", app.tr("ui.mks_speed.recompute"), false))
                            .inner;
                        scenario::publish_handle("mks:graph:recompute", recompute.rect.center());
                        if run.clicked() || recompute.clicked() {
                            draft.playing = false;
                            match crate::mks_run_ui::start(app, ui.ctx(), draft.settings.clone(), recompute.clicked()) {
                                Ok(running) => {
                                    draft.running = Some(running);
                                    draft.error.clear();
                                    ui.ctx().request_repaint();
                                }
                                Err(e) => draft.error = e,
                            }
                        }
                        if busy {
                            ui.spinner();
                            ui.label(app.tr("ui.mks_speed.computing"));
                            scenario::publish_handle("mks:run:busy", ui.min_rect().center());
                            let discard = ui.button(app.tr("ui.mks_speed.discard"));
                            scenario::publish_handle("mks:run:discard", discard.rect.center());
                            if discard.clicked() {
                                draft.running = None;
                            }
                        }
                        let gravity = draft.settings["gravity_m_s2"].clone();
                        let fine = ui.button(app.tr("ui.mks_quality.fine"));
                        scenario::publish_handle("mks:quality:fine", fine.rect.center());
                        if fine.clicked() && !busy {
                            let steps = draft.settings["steps"].as_u64().unwrap_or(1000).clamp(10, 10000);
                            draft.settings["steps"] = json!(steps.div_ceil(1000) * 1000);
                            draft.settings["samples"] = json!(1001);
                        }
                        ui.small(format!("{} {}", draft.settings["samples"], app.tr("ui.mks_quality.samples")));
                        ui.label(format!("g = {gravity} m/s²"));
                        let arrange = crate::toolbar::engineering_action(ui, "fit", app.tr("ui.workspace.arrange"), false);
                        scenario::publish_handle("mks:graph:arrange", arrange.rect.center());
                        if arrange.clicked() {
                            draft.positions.clear();
                            draft.scene = Rect::ZERO;
                            draft.layout_dirty = true;
                        }
                        let scope = crate::toolbar::engineering_action(ui, "graph", "Scope", draft.scope_visible);
                        if scope.clicked() {
                            draft.scope_visible = !draft.scope_visible;
                        }
                        scenario::publish_handle("mks:scope:toggle", scope.rect.center());
                        crate::mks_video_ui::show(
                            app,
                            ui,
                            state["result_current"] == true
                                && comparable_settings(&draft.settings, &draft.diagram)
                                    == comparable_settings(&state["data"]["job"]["settings"], &draft.diagram),
                        );
                        if crate::toolbar::engineering_action(ui, "settings", app.tr("ui.workspace.properties"), draft.inspector).clicked() {
                            draft.inspector = !draft.inspector;
                        }
                        let cad = crate::toolbar::engineering_action(ui, "joint_origin", app.tr("ui.mks_cad.title"), false);
                        scenario::publish_handle("mks:cad:open", cad.rect.center());
                        if cad.clicked() {
                            app.ui.project_page = "cad".into();
                            app.ui.engineering_dialog = Some("cad_mks".into());
                        }
                        crate::mks_diagram_ui::ribbon(app, ui, &mut draft.diagram);
                        if comparable_settings(&draft.settings, &draft.diagram)
                            == comparable_settings(&state["data"]["job"]["settings"], &draft.diagram)
                        {
                            playback(app, ui, &state, &mut draft);
                        } else if state["result_current"] == true {
                            draft.playing = false;
                            ui.label(app.tr("ui.mks_quality.changed"));
                        }
                    });
                });
            scenario::publish_handle("mks:ribbon", ui.min_rect().center());
        },
    );
    if !draft.error.is_empty() {
        ui.colored_label(Tokens::get().warning, &draft.error);
    }
    if blocks.len() > 64 || joints.len() > 128 {
        ui.label(app.tr("ui.mks_graph.limit"));
        return;
    }
    let mut visible_state = state.clone();
    // Diagram inputs replace (or explicitly mask) local drivers in the engine.
    // A saved draft may omit these drivers while the canonical job includes them.
    // Their source/connection validity is already checked by result_current.
    if comparable_settings(&draft.settings, &draft.diagram) != comparable_settings(&state["data"]["job"]["settings"], &draft.diagram) {
        visible_state = std::sync::Arc::new(json!({"result_current":false}));
    }
    let panel = crate::ui_kit::Container::new(ui.ctx(), crate::ui_kit::Surface::Panel);
    panel
        .edge("mks_library", crate::ui_kit::Edge::Left)
        .default_size(218.)
        .resizable(true)
        .frame(panel.edge_frame(Tokens::get().toolbar).inner_margin(10.))
        .show(ui, |ui| library(app, ui, blocks, joints, &mut draft));
    if draft.preview && visible_state["result_current"] == true {
        panel
            .edge("mks_playback", crate::ui_kit::Edge::Right)
            .default_size(ui.available_width() * 0.5)
            .resizable(true)
            .frame(panel.edge_frame(Tokens::get().field).inner_margin(8.))
            .show(ui, |ui| {
                ui.strong(app.tr("ui.workspace.simulation"));
                ui.small(app.tr("ui.workspace.orbit"));
                if crate::mks_result_view::inline(app, ui, draft.sample, draft.playhead) && draft.playing {
                    // Start the wall clock after preparation, without skipping the
                    // opening motion while meshes/shaders are first made ready.
                    draft.clock = -1.;
                }
            });
    }
    if draft.scope_visible {
        panel
            .edge("mks_scope", crate::ui_kit::Edge::Bottom)
            .default_size(340.)
            .resizable(true)
            .frame(panel.edge_frame(Tokens::get().toolbar).inner_margin(8.))
            .show(ui, |ui| scope(app, ui, joints, &visible_state, &mut draft));
    }
    egui::Frame::NONE.fill(Tokens::get().toolbar).show(ui, |ui| crate::mks_diagram_ui::hint(app, ui, &mut draft.diagram));
    crate::ui_kit::FileView::page(ui, |ui| graph(app, ui, blocks, joints, &mut draft));
    if let Some(id) = draft.selected
        && let Some(joint) = joints.iter().find(|j| j["id"] == id)
    {
        let closed = crate::ui_kit::Window::new(key.with("joint_settings"), "settings", app.tr("ui.workspace.joint_settings"))
            .cards(2.5)
            .show_body(ui.ctx(), |ui| edit_joint(app, ui, joint, blocks, &mut draft));
        if closed {
            draft.selected = None;
        }
    }
    if draft.control_open {
        let closed = crate::ui_kit::Window::new(key.with("controller"), "drive", app.tr("ui.mks_lib.controller"))
            .cards(3.)
            .show_body(ui.ctx(), |ui| control_editor(app, ui, joints, &mut draft));
        if closed {
            draft.control_open = false;
        }
    }
    crate::mks_library_ui::show(app, ui.ctx(), model, &mut draft.insert);
    if let Some(kind) = draft.pending_target.clone() {
        let mut selected = None;
        let closed =
            crate::ui_kit::Window::new(key.with("target"), "joint", app.tr("ui.mks_ext.choose_target")).cards(2.).show_body(ui.ctx(), |ui| {
                for joint in joints {
                    let id = joint["id"].as_u64().unwrap_or(0);
                    let r = ui.button(joint["name"].as_str().unwrap_or(""));
                    scenario::publish_handle(&format!("mks:target:joint:{id}"), r.rect.center());
                    if r.clicked() {
                        selected = Some((id, joint["kind"] == "slider"));
                    }
                }
            });
        if let Some((id, slider)) = selected {
            attach(&mut draft, id, &kind, slider);
            draft.pending_target = None;
        }
        if closed {
            draft.pending_target = None;
        }
    }
    if draft.quick_insert {
        let closed = crate::ui_kit::Window::new(key.with("quick_insert"), "component", app.tr("ui.mks_ext.quick_insert"))
            .cards(2.)
            .show_body(ui.ctx(), |ui| library(app, ui, blocks, joints, &mut draft));
        if closed {
            draft.quick_insert = false;
        }
    }
    if draft.inspector {
        let closed =
            crate::ui_kit::Window::new(key.with("advanced"), "settings", app.tr("ui.workspace.properties")).cards(3.).show_body(ui.ctx(), |ui| {
                egui::ScrollArea::vertical().max_height(600.).show(ui, |ui| {
                    crate::robot_ui::motion(app, ui);
                    crate::mks_dynamics_ui::show(app, ui, model);
                    crate::mechanics_ui::show(app, ui, false);
                });
            });
        if closed {
            draft.inspector = false;
        }
    }
    if let Some(id) = draft.selected_body.clone()
        && let Some(block) = blocks.iter().find(|b| b["id"] == id)
    {
        let closed = crate::ui_kit::Window::new(key.with("body"), "settings", &id).cards(2.).show_body(ui.ctx(), |ui| {
            ui.label(app.tr("ui.workspace.cad_owned"));
            for m in block["members"].as_array().into_iter().flatten() {
                ui.label(m["name"].as_str().unwrap_or(""));
            }
            ui.label(format!("Ground: {}", block["grounded"]));
            if block["grounded"] != true {
                let create = ui.button(app.tr("ui.mks_rom.prepare_component"));
                scenario::publish_handle(&format!("mks:rom:prepare:{id}"), create.rect.center());
                if create.clicked() {
                    crate::fem_rom_ui::prepare_block(app, ui.ctx(), &id);
                }
            }
            let rom_key = key.with("rom_summary");
            let cached = ui.ctx().data(|data| data.get_temp::<(u64, Value)>(rom_key));
            let state = match cached {
                Some((revision, state)) if revision == app.session.revision => Ok(state),
                _ => app.session.execute("fem.model.read", &json!({})),
            };
            if let Ok(state) = state {
                ui.ctx().data_mut(|data| data.insert_temp(rom_key, (app.session.revision, state.clone())));
                for model in state["models"].as_array().into_iter().flatten().filter(|m| m["block"] == id) {
                    let Some(model_id) = model["id"].as_str() else { continue };
                    ui.separator();
                    ui.strong(format!("ROM: {model_id}"));
                    ui.label(app.tr(if model["current"] == true { "ui.fem_rom.current" } else { "ui.fem_rom.stale" }));
                    ui.label(format!("{} → {} DOF", model["full_dofs"], model["reduced_dofs"]));
                    let mut enabled = model["enabled"] == true;
                    let toggle = ui.add_enabled(
                        enabled || model["current"] == true && model["has_result"] == true,
                        egui::Checkbox::new(&mut enabled, app.tr("ui.fem_rom.bind")),
                    );
                    scenario::publish_handle(&format!("mks:rom:bind:{model_id}"), toggle.rect.center());
                    if toggle.changed()
                        && let Err(error) = app.run("fem.mks.bind", json!({"id":model_id,"enabled":enabled}))
                    {
                        draft.error = error.to_string();
                    }
                    let results = ui.add_enabled(
                        model["has_result"] == true && model["current"] == true,
                        egui::Button::new(app.tr("ui.mks_rom.component_results")),
                    );
                    scenario::publish_handle(&format!("mks:rom:results:{model_id}"), results.rect.center());
                    if results.clicked() {
                        crate::fem_rom_ui::select_model(app, ui.ctx(), model_id);
                        app.ui.project_page = "fem".into();
                        app.ui.tab = "FEM / ROM".into();
                    }
                }
            }
            if block["grounded"] == true {
                ui.label(app.tr("ui.workspace.gravity"));
                let mut gravity: [f64; 3] = serde_json::from_value(draft.settings["gravity_m_s2"].clone()).unwrap_or([0., 0., -9.81]);
                ui.horizontal(|ui| {
                    for value in &mut gravity {
                        ui.add(egui::DragValue::new(value).speed(0.1).range(-1000000.0..=1000000.0));
                    }
                });
                draft.settings["gravity_m_s2"] = json!(gravity);
            } else {
                for port in block["ports"].as_array().into_iter().flatten() {
                    let Some(name) = port["port"]["id"].as_str() else { continue };
                    if port["port"]["transfer"] == "pressure" {
                        continue;
                    }
                    let mut loads = draft.settings["port_loads"].as_array().cloned().unwrap_or_default();
                    let mut enabled = loads.iter().any(|l| l["port"] == name);
                    let mut load = loads
                        .iter()
                        .find(|l| l["port"] == name)
                        .cloned()
                        .unwrap_or_else(|| json!({"port":name,"force_n":[0.,0.,-10.],"moment_nm":[0.,0.,0.]}));
                    let toggle = ui.checkbox(&mut enabled, format!("{}: {name}", app.tr("ui.workspace.force")));
                    scenario::publish_handle(&format!("mks:force:{name}"), toggle.rect.center());
                    if enabled {
                        ui.small(app.tr("ui.workspace.force_frame"));
                        let mut force: [f64; 3] = serde_json::from_value(load["force_n"].clone()).unwrap_or([0.; 3]);
                        ui.horizontal(|ui| {
                            for value in &mut force {
                                ui.add(egui::DragValue::new(value).speed(1.).range(-1e8..=1e8));
                            }
                        });
                        load["force_n"] = json!(force);
                    }
                    loads.retain(|l| l["port"] != name);
                    if enabled {
                        loads.push(load);
                    }
                    draft.settings["port_loads"] = json!(loads);
                }
            }
        });
        if closed {
            draft.selected_body = None;
        }
    }
    crate::mks_diagram_ui::show(app, ui, &mut draft.diagram, draft.settings["duration_s"].as_f64().unwrap_or(2.));
    if let Some(id) = draft.diagram.remove_measure.take() {
        draft.unscoped.insert(id);
        draft.layout_dirty = true;
    }
    if before_settings != draft.settings {
        draft.layout_dirty = true;
    }
    if draft.layout_dirty && !ui.input(|i| i.pointer.any_down()) {
        match app.run(
            "worksheet.presentation",
            json!({"workspace":"mks","layout":{"positions":draft.positions,"sensors":draft.sensors,"unscoped_sensors":draft.unscoped,"settings":draft.settings,"shared_control":draft.shared_control,"outputs":draft.outputs}}),
        ) {
            Ok(_) => draft.layout_dirty = false,
            Err(e) => {
                draft.error = e;
                draft.layout_dirty = false;
            }
        }
    }
    ui.ctx().data_mut(|d| d.insert_temp(key, draft));
}

fn edit_joint(app: &mut SolveApp, ui: &mut egui::Ui, joint: &Value, blocks: &[Value], draft: &mut Draft) {
    ui.strong(format!("{} — {}", joint["name"].as_str().unwrap_or(""), joint["kind"].as_str().unwrap_or("")));
    ui.horizontal_wrapped(|ui| {
        for (i, key) in ["connections_tab", "actuation_tab", "sensing_tab"].iter().enumerate() {
            let r = ui.selectable_value(&mut draft.joint_tab, i, app.tr(&format!("ui.mks_lib.{key}")));
            scenario::publish_handle(&format!("mks:joint:tab:{i}"), r.rect.center());
        }
    });
    ui.separator();
    match draft.joint_tab {
        0 => {
            for key in ["a", "b"] {
                let names = blocks
                    .iter()
                    .find(|b| b["id"] == joint[key])
                    .into_iter()
                    .flat_map(|b| b["members"].as_array().into_iter().flatten())
                    .filter_map(|m| m["name"].as_str())
                    .collect::<Vec<_>>()
                    .join(" + ");
                ui.add(egui::Label::new(format!("{}: {names}", if key == "a" { "B" } else { "F" })).wrap());
            }
            ui.add(egui::Label::new(app.tr("ui.mks_lib.native_joint")).wrap());
        }
        1 => {
            ui.add(egui::Label::new(app.tr("ui.mks_ext.relative")).wrap());
            let id = joint["id"].as_u64().unwrap_or(0);
            if joint["kind"] == "revolute" || joint["kind"] == "slider" {
                let new_source = ui.button(app.tr("ui.mks_workflow.new_source"));
                scenario::publish_handle("mks:joint:new_source", new_source.rect.center());
                if new_source.clicked() {
                    draft.diagram.new_for_joint(id, joint["kind"] == "slider");
                    draft.selected = None;
                }
                egui::CollapsingHeader::new(app.tr("ui.mks_workflow.existing_source")).show(ui, |ui| {
                    let dimension = if joint["kind"] == "slider" { "length" } else { "angle" };
                    let choices: Vec<_> = draft.diagram.graph["sources"]
                        .as_object()
                        .into_iter()
                        .flatten()
                        .filter(|(_, source)| source["dimension"] == dimension)
                        .filter_map(|(key, source)| Some((key.parse::<u64>().ok()?, source["name"].as_str()?.to_owned())))
                        .collect();
                    for (source, name) in choices {
                        let button = ui.button(name);
                        scenario::publish_handle(&format!("mks:joint:connect_source:{source}"), button.rect.center());
                        if button.clicked() {
                            draft.diagram.target = Some((source, id));
                            draft.selected = None;
                        }
                    }
                });
                ui.separator();
            }
            if draft.diagram.managed(id) {
                if let Some(source) = draft.diagram.linked(id) {
                    ui.label(format!(
                        "{}: {}",
                        app.tr("ui.mks_wire.connected"),
                        draft.diagram.graph["sources"][source.to_string()]["name"].as_str().unwrap_or("")
                    ));
                    let b = ui.button(app.tr("ui.mks_wire.edit_source"));
                    scenario::publish_handle("mks:wire:edit_source", b.rect.center());
                    if b.clicked() {
                        draft.diagram.open(source);
                        draft.selected = None;
                    }
                    let b = ui.button(app.tr("ui.mks_wire.disconnect"));
                    scenario::publish_handle("mks:wire:disconnect", b.rect.center());
                    if b.clicked() {
                        draft.diagram.disconnect(id);
                    }
                } else {
                    ui.label(app.tr("ui.mks_wire.disconnected"));
                    let b = ui.button(app.tr("ui.mks_wire.local"));
                    scenario::publish_handle("mks:wire:local", b.rect.center());
                    if b.clicked() {
                        draft.diagram.request = Some(("mks.diagram.release".into(), json!({"joint":id})));
                    }
                }
            } else {
                joint_motion(app, ui, joint, draft);
            }
        }
        _ => {
            ui.add(egui::Label::new(app.tr("ui.mks_ext.measurement_note")).wrap());
            joint_sensing(app, ui, joint, draft);
        }
    }
    ui.separator();
    let close = ui.button(app.tr("ui.mks_graph.close"));
    scenario::publish_handle("mks:graph:close", close.rect.center());
    if close.clicked() {
        draft.selected = None;
    }
}
fn joint_motion(app: &mut SolveApp, ui: &mut egui::Ui, joint: &Value, draft: &mut Draft) {
    let id = joint["id"].as_u64().unwrap_or(0);
    let old = draft.settings["drivers"].as_array().cloned().unwrap_or_default();
    let existing = old.iter().find(|d| d["joint"] == id);
    let mut enabled = existing.is_some();
    let toggle = ui.checkbox(&mut enabled, app.tr("ui.mks_graph.motion"));
    scenario::publish_handle("mks:graph:motion:enable", toggle.rect.center());
    let mut drive =
        existing.cloned().unwrap_or_else(|| json!({"joint":id,"displacement":if joint["kind"]=="slider" {"50 mm"}else{"30 deg"},"ramp_s":0.5}));
    if enabled {
        ui.horizontal(|ui| {
            ui.label(app.tr("ui.mks_graph.increment"));
            let mut expression = drive["displacement"].as_str().unwrap_or("0 deg").to_string();
            let field = ui.text_edit_singleline(&mut expression);
            scenario::publish_handle("mks:graph:motion:expression", field.rect.center());
            drive["displacement"] = json!(expression);
            ui.label(app.tr("ui.mks_graph.ramp"));
            let mut ramp = drive["ramp_s"].as_f64().unwrap_or(0.5);
            ui.add(egui::DragValue::new(&mut ramp).range(0.001..=10.).speed(0.1));
            drive["ramp_s"] = json!(ramp);
        });
    }
    if enabled {
        crate::mks_source_ui::fields(app, ui, &mut drive, id, true);
        crate::mks_source_ui::preview(app, ui, &drive, draft.settings["duration_s"].as_f64().unwrap_or(2.));
    }
    let mut drivers: Vec<_> = old.into_iter().filter(|d| d["joint"] != id).collect();
    if enabled {
        drivers.push(drive);
    }
    drivers.sort_by_key(|d| d["joint"].as_u64().unwrap_or(0));
    draft.settings["drivers"] = json!(drivers);
}
fn joint_sensing(app: &SolveApp, ui: &mut egui::Ui, joint: &Value, draft: &mut Draft) {
    let id = joint["id"].as_u64().unwrap_or(0);
    let mut sensor = draft.sensors.contains(&id);
    let r = ui.checkbox(&mut sensor, app.tr("ui.mks_ext.sensor_enable"));
    scenario::publish_handle("mks:graph:sensor:enable", r.rect.center());
    if r.changed() {
        draft.layout_dirty = true;
    }
    if sensor {
        draft.sensors.insert(id);
    } else {
        draft.sensors.remove(&id);
    }
    let selected = draft.outputs.entry(id).or_insert_with(|| BTreeSet::from([3]));
    ui.horizontal_wrapped(|ui| {
        let angular = joint["kind"] == "revolute";
        for (i, label) in
            ["Fx [N]", "Fy [N]", "Fz [N]", "|F| [N]", if angular { "q [deg]" } else { "q [mm]" }, if angular { "ω [deg/s]" } else { "v [mm/s]" }]
                .iter()
                .enumerate()
        {
            let mut enabled = selected.contains(&i);
            let r = ui.checkbox(&mut enabled, *label);
            scenario::publish_handle(&format!("mks:joint:output:{id}:{i}"), r.rect.center());
            if r.changed() {
                if enabled {
                    selected.insert(i);
                } else {
                    selected.remove(&i);
                }
                draft.layout_dirty = true;
            }
        }
    });
    if ui.button(app.tr("ui.mks_lib.connect_scope")).clicked() {
        draft.unscoped.remove(&id);
        draft.sensors.insert(id);
        draft.scope_visible = true;
        draft.layout_dirty = true;
    }
}

fn graph(app: &SolveApp, ui: &mut egui::Ui, blocks: &[Value], joints: &[Value], draft: &mut Draft) {
    let indices: BTreeMap<_, _> = blocks.iter().enumerate().filter_map(|(i, b)| Some((b["id"].as_str()?, i))).collect();
    let mut depth = vec![usize::MAX; blocks.len()];
    for (i, b) in blocks.iter().enumerate() {
        if b["grounded"] == true {
            depth[i] = 0;
        }
    }
    if !depth.contains(&0)
        && let Some(d) = depth.first_mut()
    {
        *d = 0;
    }
    for _ in 0..blocks.len() {
        for j in joints {
            if let (Some(&a), Some(&b)) = (j["a"].as_str().and_then(|s| indices.get(s)), j["b"].as_str().and_then(|s| indices.get(s))) {
                if depth[a] != usize::MAX {
                    depth[b] = depth[b].min(depth[a].saturating_add(1));
                }
                if depth[b] != usize::MAX {
                    depth[a] = depth[a].min(depth[b].saturating_add(1));
                }
            }
        }
    }
    for d in &mut depth {
        if *d == usize::MAX {
            *d = 0;
        }
    }
    let mut children = vec![Vec::<usize>::new(); blocks.len()];
    for j in joints {
        if let (Some(&a), Some(&b)) = (j["a"].as_str().and_then(|s| indices.get(s)), j["b"].as_str().and_then(|s| indices.get(s))) {
            if depth[a] < depth[b] {
                children[a].push(b);
            } else if depth[b] < depth[a] {
                children[b].push(a);
            }
        }
    }
    let mut rows = vec![0.; blocks.len()];
    let mut leaf = 0.;
    for (i, c) in children.iter().enumerate() {
        if c.is_empty() {
            rows[i] = leaf;
            leaf += 1.;
        }
    }
    let max_depth = depth.iter().copied().max().unwrap_or(0);
    for level in (0..=max_depth).rev() {
        for (i, c) in children.iter().enumerate() {
            if depth[i] == level && !c.is_empty() {
                rows[i] = c.iter().map(|&v| rows[v]).sum::<f32>() / c.len() as f32;
            }
        }
    }
    let sources = draft.diagram.graph["sources"].as_object().cloned().unwrap_or_default();
    let body_top = 300.0_f32.max(sources.len().div_ceil(2) as f32 * 145. + 80.);
    let mut scene = draft.scene;
    egui::Scene::new()
        .zoom_range(0.15..=1.5)
        .max_inner_size(vec2(21000., 21000.))
        .drag_pan_buttons(egui::DragPanButtons::MIDDLE | egui::DragPanButtons::SECONDARY)
        .show(ui, &mut scene, |ui| {
            let extent = draft.positions.values().fold(vec2(0., 0.), |a, p| vec2(a.x.max(p[0] + 700.), a.y.max(p[1] + 240.)));
            let size = vec2(
                ((max_depth.min(2) + 1) as f32 * 360. + 220.).max(1500. + joints.len().saturating_sub(8) as f32 * 16.).max(extent.x),
                ((max_depth / 3 + 1) as f32 * 330. + body_top + 120. + leaf.max(1.) * 30.).max(extent.y),
            );
            let (area, drop) = ui.allocate_exact_size(size, Sense::click());
            // Scene's clip is expressed in world coordinates, including negative
            // coordinates after panning. Paint the visible world, not the node extent.
            let painter = ui.painter().clone();
            let visible = ui.clip_rect();
            let t = Tokens::get();
            painter.rect_filled(visible, 0., t.field);
            if visible.is_finite() {
                for n in 0..((visible.width() / 24.).ceil() as usize + 2).min(4096) {
                    let x = (visible.left() / 24.).floor() * 24. + n as f32 * 24.;
                    painter.line_segment([pos2(x, visible.top()), pos2(x, visible.bottom())], Stroke::new(0.35, t.border));
                }
                for n in 0..((visible.height() / 24.).ceil() as usize + 2).min(4096) {
                    let y = (visible.top() / 24.).floor() * 24. + n as f32 * 24.;
                    painter.line_segment([pos2(visible.left(), y), pos2(visible.right(), y)], Stroke::new(0.35, t.border));
                }
            }
            node_handle(ui, "mks:canvas", area.center());
            let cards: Vec<_> = depth
                .iter()
                .zip(&rows)
                .enumerate()
                .map(|(i, (d, r))| {
                    let id = format!("body:{}", blocks[i]["id"].as_str().unwrap_or(""));
                    {
                        let column = if (*d / 3) % 2 == 0 { *d % 3 } else { 2 - *d % 3 };
                        let ports = blocks[i]["ports"].as_array().map_or(0, Vec::len).min(4);
                        placed(
                            draft,
                            &id,
                            area.min,
                            vec2(column as f32 * 360. + 20., (*d / 3) as f32 * 330. + r * 30. + body_top),
                            vec2(180., 144. + ports as f32 * 18.),
                        )
                    }
                })
                .collect();
            let wire = |a: egui::Pos2, b: egui::Pos2, color: egui::Color32| {
                let middle = (a.x + b.x) * 0.5;
                for pair in [[a, pos2(middle, a.y)], [pos2(middle, a.y), pos2(middle, b.y)], [pos2(middle, b.y), b]] {
                    painter.line_segment(pair, Stroke::new(1.5, color));
                }
            };
            let control_x = area.left() + 8.;

            let mut explicit_routes = Vec::new();
            let mut controls = Vec::new();
            let mut measurements = Vec::new();
            let path = |points: &[egui::Pos2], color: egui::Color32, width: f32| {
                for pair in points.windows(2) {
                    if let [a, b] = pair {
                        painter.line_segment([*a, *b], Stroke::new(width, color));
                    }
                }
            };
            let controller = placed(draft, "controller", area.min, vec2(20., 20.), vec2(260., 190.));
            if draft.shared_control {
                painter.rect_filled(controller, t.radius_md, t.accent_soft);
                painter.rect_stroke(controller, t.radius_md, Stroke::new(1.5, t.accent), egui::StrokeKind::Inside);
                painter.text(controller.min + vec2(12., 12.), Align2::LEFT_TOP, app.tr("ui.mks_lib.controller"), FontId::proportional(16.), t.text);
                for (i, drive) in draft.settings["drivers"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|d| !draft.diagram.managed(d["joint"].as_u64().unwrap_or(0)))
                    .take(6)
                    .enumerate()
                {
                    let name = joints.iter().find(|j| j["id"] == drive["joint"]).and_then(|j| j["name"].as_str()).unwrap_or("");
                    painter.text(
                        controller.min + vec2(12., 48. + i as f32 * 22.),
                        Align2::LEFT_TOP,
                        format!("{}: {}", name, drive["displacement"].as_str().unwrap_or("")),
                        FontId::proportional(12.),
                        t.text,
                    );
                }
                let response = ui.interact(controller, ui.id().with("controller"), Sense::click_and_drag());
                move_node(draft, "controller", controller, area.min, &response);
                if response.double_clicked() {
                    draft.control_open = true;
                }
                node_handle(ui, "mks:controller", controller.center());
            }
            let mut source_ports = BTreeMap::new();
            let mut source_obstacles = Vec::new();
            let mut extra_obstacles = Vec::new();
            let mut measurement_ports = BTreeMap::new();
            for (i, (key, source)) in sources.iter().enumerate() {
                let Ok(id) = key.parse::<u64>() else { continue };
                let node = format!("source:{id}");
                let rect = placed(draft, &node, area.min, vec2(330. + (i % 2) as f32 * 300., 20. + (i / 2) as f32 * 145.), vec2(225., 110.));
                source_obstacles.push(rect);
                painter.rect_filled(rect, t.radius_md, t.accent_soft);
                painter.rect_stroke(rect, t.radius_md, Stroke::new(1.5, t.accent), egui::StrokeKind::Inside);
                painter.text(rect.min + vec2(10., 8.), Align2::LEFT_TOP, source["name"].as_str().unwrap_or(""), FontId::proportional(15.), t.text);
                painter.text(
                    rect.min + vec2(10., 36.),
                    Align2::LEFT_TOP,
                    format!(
                        "{}\n{}",
                        app.tr(&format!("ui.mks_ext.{}", source["profile"].as_str().unwrap_or("quintic"))),
                        source["displacement"].as_str().unwrap_or("")
                    ),
                    FontId::proportional(12.),
                    t.text,
                );
                let r = ui.interact(rect, ui.id().with(("source", id)), Sense::click_and_drag());
                move_node(draft, &node, rect, area.min, &r);
                node_handle(ui, &format!("mks:independent:{id}"), rect.center());
                if r.double_clicked() {
                    draft.diagram.open(id);
                }
                let port = rect.right_center();
                source_ports.insert(id, port);
                painter.circle_filled(port, 5., t.accent);
                let pin = ui.interact(Rect::from_center_size(port, vec2(16., 16.)), ui.id().with(("source_pin", id)), Sense::click_and_drag());
                node_handle(ui, &format!("mks:independent:out:{id}"), port);
                if pin.clicked() || pin.drag_started_by(egui::PointerButton::Primary) {
                    draft.diagram.dragging = pin.drag_started_by(egui::PointerButton::Primary);
                    draft.diagram.wiring = true;
                    draft.diagram.wire = Some(id);
                    draft.diagram.measure = None;
                }
            }
            let mut joint_obstacles = Vec::new();
            for j in joints {
                let (Some(&a), Some(&b)) = (j["a"].as_str().and_then(|s| indices.get(s)), j["b"].as_str().and_then(|s| indices.get(s))) else {
                    continue;
                };
                let (Some(ra), Some(rb)) = (cards.get(a), cards.get(b)) else { continue };
                let id = j["id"].as_u64().unwrap_or(0);
                let middle = ra.center().lerp(rb.center(), 0.5);
                let node = format!("joint:{id}");
                let rect = placed(draft, &node, area.min, middle - area.min - vec2(55., 36.), vec2(110., 72.));
                joint_obstacles.push(rect);
                let vertical = (ra.center().x - rb.center().x).abs() < 1.;
                let reverse = ra.center().x > rb.center().x;
                let (base, follower) = if vertical {
                    (rect.center_top(), rect.center_bottom())
                } else if reverse {
                    (rect.right_center(), rect.left_center())
                } else {
                    (rect.left_center(), rect.right_center())
                };
                let (from, to) = if vertical {
                    (ra.center_bottom(), rb.center_top())
                } else if reverse {
                    (ra.left_center(), rb.right_center())
                } else {
                    (ra.right_center(), rb.left_center())
                };
                wire(from, base, t.text_dim);
                wire(follower, to, t.text_dim);
                painter.rect_filled(rect, t.radius_md, t.field);
                painter.rect_stroke(
                    rect,
                    t.radius_md,
                    Stroke::new(1.5, if draft.selected == Some(id) { t.accent } else { t.border }),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    rect.center() - vec2(0., 12.),
                    Align2::CENTER_CENTER,
                    j["name"].as_str().unwrap_or(""),
                    FontId::proportional(13.),
                    t.text,
                );
                painter.text(
                    rect.center() + vec2(0., 14.),
                    Align2::CENTER_CENTER,
                    j["kind"].as_str().unwrap_or(""),
                    FontId::proportional(12.),
                    t.text,
                );
                painter.text(
                    base + if vertical {
                        vec2(-8., 8.)
                    } else if reverse {
                        vec2(-8., -14.)
                    } else {
                        vec2(8., -14.)
                    },
                    Align2::CENTER_CENTER,
                    "B",
                    FontId::proportional(10.),
                    t.text,
                );
                painter.text(
                    follower
                        + if vertical {
                            vec2(8., -8.)
                        } else if reverse {
                            vec2(8., 14.)
                        } else {
                            vec2(-8., 14.)
                        },
                    Align2::CENTER_CENTER,
                    "F",
                    FontId::proportional(10.),
                    t.text,
                );
                node_handle(ui, &format!("mks:joint:{id}"), rect.center());
                let response = ui.interact(rect, ui.id().with(("joint", id)), Sense::click_and_drag());
                move_node(draft, &node, rect, area.min, &response);
                if response.double_clicked() {
                    draft.selected = Some(id);
                }
                if let Some(kind) = response.dnd_release_payload::<String>() {
                    attach(draft, id, &kind, j["kind"] == "slider");
                }
                for (port, label) in [(base, "B"), (follower, "F")] {
                    let pin = Rect::from_center_size(port, vec2(10., 10.));
                    painter.rect_filled(pin, 1., t.field);
                    painter.rect_stroke(pin, 1., Stroke::new(1.5, t.text_dim), egui::StrokeKind::Inside);
                    let response = ui.interact(pin.expand(2.), ui.id().with(("frame_port", id, label)), Sense::click());
                    if response.clicked() {
                        draft.selected = Some(id);
                        draft.joint_tab = 0;
                    }
                    node_handle(ui, &format!("mks:joint:port:{id}:{label}"), port);
                }
                {
                    let port = if vertical { rect.left_center() } else { rect.center_top() };
                    painter.circle_filled(port, 4., t.accent);
                    let response = ui.interact(Rect::from_center_size(port, vec2(12., 12.)), ui.id().with(("motion_port", id)), Sense::click());
                    if response.clicked() || (draft.diagram.dragging && released_over(ui, response.rect.expand(12.))) {
                        if draft.diagram.wiring {
                            draft.diagram.input(id);
                        } else {
                            draft.selected = Some(id);
                            draft.joint_tab = 1;
                        }
                    }
                    if let Some(source) = draft.diagram.linked(id)
                        && let Some(from) = source_ports.get(&source)
                    {
                        explicit_routes.push((*from, port, id));
                    }
                    node_handle(ui, &format!("mks:joint:port:{id}:q"), port);
                }
                if !draft.shared_control
                    && !draft.diagram.managed(id)
                    && let Some(drive) = draft.settings["drivers"].as_array().into_iter().flatten().find(|d| d["joint"] == id)
                {
                    let motion_key = format!("motion:{id}");
                    let motion = placed(draft, &motion_key, area.min, rect.min - area.min - vec2(0., 110.), vec2(170., 56.));
                    extra_obstacles.push(motion);
                    painter.rect_filled(motion, t.radius_md, t.accent_soft);
                    painter.text(
                        motion.center(),
                        Align2::CENTER_CENTER,
                        format!("{}\n{}", app.tr("ui.mks_graph.motion_node"), drive["displacement"].as_str().unwrap_or("")),
                        FontId::proportional(12.),
                        t.text,
                    );
                    wire(motion.center_bottom(), rect.center_top(), t.accent);
                    node_handle(ui, &format!("mks:motion:{id}"), motion.center());
                    let response = ui.interact(motion, ui.id().with(("motion", id)), Sense::click_and_drag());
                    move_node(draft, &motion_key, motion, area.min, &response);
                    if response.double_clicked() {
                        draft.selected = Some(id);
                        draft.joint_tab = 1;
                    }
                }
                if draft.shared_control
                    && !draft.diagram.managed(id)
                    && draft.settings["drivers"].as_array().into_iter().flatten().any(|d| d["joint"] == id)
                {
                    let q = if vertical { rect.left_center() } else { rect.center_top() };
                    let lane = if vertical { q.y } else { ra.top().min(rb.top()) - 50. };
                    controls.push((q, lane, id));
                }
                if draft.diagram.wiring && !draft.sensors.contains(&id) {
                    let port = if vertical { rect.right_center() } else { rect.center_bottom() };
                    measurement_ports.insert(id, port);
                    painter.circle_stroke(port, 5., Stroke::new(1.5, t.axis_y));
                    let pin = ui.interact(Rect::from_center_size(port, vec2(16., 16.)), ui.id().with(("new_measure", id)), Sense::click_and_drag());
                    node_handle(ui, &format!("mks:joint:port:{id}:out"), port);
                    if pin.clicked() || pin.drag_started_by(egui::PointerButton::Primary) {
                        draft.diagram.dragging = pin.drag_started_by(egui::PointerButton::Primary);
                        draft.diagram.measure = Some(id);
                        draft.diagram.wire = None;
                    }
                }
                if draft.sensors.contains(&id) {
                    let sensor_key = format!("sensor:{id}");
                    let offset = if (ra.center().x - rb.center().x).abs() < 1. { vec2(110., -19.) } else { vec2(-60., 90.) };
                    let sensor = placed(draft, &sensor_key, area.min, rect.center() - area.min + offset, vec2(120., 38.));
                    extra_obstacles.push(sensor);
                    painter.rect_filled(sensor, t.radius_md, t.accent_soft);
                    painter.with_clip_rect(sensor.shrink(2.)).text(
                        sensor.center(),
                        Align2::CENTER_CENTER,
                        format!("{}\n{}", j["name"].as_str().unwrap_or(""), app.tr("ui.mks_ext.measurement_block")),
                        FontId::proportional(10.),
                        t.text,
                    );
                    let response = ui.interact(sensor, ui.id().with(("sensor", id)), Sense::click_and_drag());
                    move_node(draft, &sensor_key, sensor, area.min, &response);
                    if response.double_clicked() {
                        draft.selected = Some(id);
                        draft.joint_tab = 2;
                    }
                    let source = if vertical { rect.right_center() } else { rect.center_bottom() };
                    measurement_ports.insert(id, source);
                    let target = if vertical { sensor.left_center() } else { sensor.center_top() };
                    painter.circle_filled(source, 4., t.axis_y);
                    let response =
                        ui.interact(Rect::from_center_size(source, vec2(12., 12.)), ui.id().with(("sensor_port", id)), Sense::click_and_drag());
                    if response.clicked() || response.drag_started_by(egui::PointerButton::Primary) {
                        if draft.diagram.wiring || response.drag_started_by(egui::PointerButton::Primary) {
                            draft.diagram.wiring = true;
                            draft.diagram.dragging = response.drag_started_by(egui::PointerButton::Primary);
                            draft.diagram.measure = Some(id);
                            draft.diagram.wire = None;
                        } else {
                            draft.selected = Some(id);
                            draft.joint_tab = 2;
                        }
                    }
                    node_handle(ui, &format!("mks:joint:port:{id}:out"), source);

                    wire(source, target, t.axis_y);
                    let outputs = draft.outputs.get(&id).cloned().unwrap_or_else(|| BTreeSet::from([3]));
                    let labels = outputs.iter().filter_map(|i| ["Fx", "Fy", "Fz", "|F|", "q", "v"].get(*i).copied()).collect::<Vec<_>>().join(", ");
                    painter.text(rect.center_bottom() + vec2(0., -8.), Align2::CENTER_BOTTOM, labels, FontId::proportional(10.), t.accent);
                    let lane = if vertical { sensor.center().y } else { ra.bottom().max(rb.bottom()) + 12. };
                    let _ = lane;
                    let out = sensor.right_center();
                    let pin = ui.interact(Rect::from_center_size(out, vec2(16., 16.)), ui.id().with(("sensor_output", id)), Sense::click_and_drag());
                    painter.circle_filled(out, 4., t.axis_y);
                    node_handle(ui, &format!("mks:sensor:out:{id}"), out);
                    if pin.clicked() || pin.drag_started_by(egui::PointerButton::Primary) {
                        draft.diagram.wiring = true;
                        draft.diagram.dragging = pin.drag_started_by(egui::PointerButton::Primary);
                        draft.diagram.measure = Some(id);
                        draft.diagram.wire = None;
                    }
                    if !draft.unscoped.contains(&id) {
                        measurements.push((out, id));
                    }
                    node_handle(ui, &format!("mks:sensor:{id}"), sensor.center());
                }
            }
            let right_edge = cards
                .iter()
                .chain(source_obstacles.iter())
                .chain(extra_obstacles.iter())
                .chain(joint_obstacles.iter())
                .map(Rect::right)
                .fold(area.left() + 680., f32::max)
                .max(if draft.shared_control { controller.right() } else { area.left() });
            let scope_margin = (64. + measurements.len() as f32 * 16.).max(140.);
            let shared_scope = placed(
                draft,
                "scope",
                area.min,
                vec2(right_edge - area.left() + scope_margin, 50.),
                vec2(180., (draft.sensors.len() as f32 * 14. + 24.).max(90.)),
            );
            painter.rect_filled(shared_scope, t.radius_md, t.field);
            painter.rect_stroke(shared_scope, t.radius_md, Stroke::new(1.5, t.accent), egui::StrokeKind::Inside);
            painter.text(shared_scope.center(), Align2::CENTER_CENTER, app.tr("ui.mks_ext.scope_block"), FontId::proportional(14.), t.text);
            let response = ui.interact(shared_scope, ui.id().with("common_scope"), Sense::click_and_drag());
            move_node(draft, "scope", shared_scope, area.min, &response);
            if response.double_clicked() {
                draft.scope_visible = true;
            }
            node_handle(ui, "mks:common_scope", shared_scope.center());
            let scope_pin = shared_scope.left_center();
            painter.circle_filled(scope_pin, 5., t.axis_y);
            let pin = ui.interact(Rect::from_center_size(scope_pin, vec2(16., 16.)), ui.id().with("scope_input"), Sense::click());
            node_handle(ui, "mks:scope:input", scope_pin);
            if (pin.clicked() || (draft.diagram.dragging && released_over(ui, pin.rect.expand(12.))))
                && let Some(id) = draft.diagram.measure.take()
            {
                draft.sensors.insert(id);
                draft.unscoped.remove(&id);
                draft.outputs.entry(id).or_insert_with(|| BTreeSet::from([3]));
                draft.scope_visible = true;
                draft.layout_dirty = true;
            }
            if let Some(max) = controls.iter().map(|(_, y, _)| *y).reduce(f32::max) {
                let origin = controller.center_bottom();
                let first = origin.y + 20.;
                path(&[origin, pos2(origin.x, first), pos2(control_x, first), pos2(control_x, max)], t.accent, 2.5);
                for (port, lane, id) in &controls {
                    path(&[pos2(control_x, *lane), pos2(port.x, *lane), *port], t.accent, 1.5);
                    painter.circle_filled(pos2(control_x, *lane), 2.5, t.accent);
                    painter.text(*port + vec2(-8., -8.), Align2::RIGHT_BOTTOM, format!("q{id}"), FontId::proportional(10.), t.accent);
                }
            }
            for (block, rect) in blocks.iter().zip(&cards) {
                let id = block["id"].as_str().unwrap_or("");
                let response = ui.interact(*rect, ui.id().with(("body", id)), Sense::click_and_drag());
                move_node(draft, &format!("body:{id}"), *rect, area.min, &response);
                node_handle(ui, &format!("mks:body:{id}"), rect.center());
                if response.double_clicked() {
                    draft.selected_body = Some(id.to_owned());
                }
                if let Some(kind) = response.dnd_release_payload::<String>() {
                    if *kind == "force" || *kind == "rom" {
                        draft.selected_body = Some(id.to_owned());
                    } else if *kind == "payload" {
                        let mut insert = crate::mks_library_ui::Insert::new("payload");
                        insert.port =
                            block["ports"].as_array().and_then(|p| p.last()).and_then(|p| p["port"]["id"].as_str()).unwrap_or("").to_owned();
                        draft.insert = Some(insert);
                    }
                }
                painter.rect_filled(*rect, t.radius_md, t.field);
                painter.rect_stroke(*rect, t.radius_md, Stroke::new(1., t.border), egui::StrokeKind::Inside);
                crate::mks_ui::block_picture(app, ui, block, Rect::from_min_size(rect.min + vec2(8., 5.), vec2(164., 66.)));
                let names = block["members"].as_array().into_iter().flatten().filter_map(|m| m["name"].as_str()).collect::<Vec<_>>().join(" + ");
                let clipped = painter.with_clip_rect(*rect);
                let rom = app
                    .session
                    .doc
                    .fem_models
                    .as_ref()
                    .and_then(|v| v["models"].as_object())
                    .into_iter()
                    .flatten()
                    .find(|(_, m)| m["enabled"] == true && m["block"] == block["id"])
                    .map(|(id, _)| id.as_str());
                let title = if block["grounded"] == true {
                    "Ground".to_owned()
                } else if let Some(rom) = rom {
                    crate::i18n::Locale::for_language(&app.preferences.language).format("ui.fem_rom.block_badge", &[("name", rom)])
                } else {
                    block["id"].as_str().unwrap_or_default().to_owned()
                };
                clipped.text(rect.min + vec2(8., 85.), Align2::LEFT_CENTER, title, FontId::proportional(13.), t.text);
                clipped.text(rect.min + vec2(8., 108.), Align2::LEFT_CENTER, names, FontId::proportional(12.), t.text);
                clipped.text(
                    rect.min + vec2(8., 130.),
                    Align2::LEFT_CENTER,
                    crate::i18n::Locale::for_language(&app.preferences.language)
                        .format("ui.mks_graph.body_count", &[("count", &block["bodies"].as_array().map_or(0, Vec::len).to_string())]),
                    FontId::proportional(11.),
                    t.text,
                );
                for (i, port) in block["ports"].as_array().into_iter().flatten().take(4).enumerate() {
                    let p = &port["port"];
                    let label = p["label"].as_str().filter(|v| !v.is_empty()).or_else(|| p["id"].as_str()).unwrap_or("?");
                    let role = p["role"].as_str().unwrap_or("bidirectional");
                    let arrow = match role {
                        "load_in" => "→",
                        "load_out" => "←",
                        _ => "↔",
                    };
                    let pin = rect.min + vec2(0., 150. + i as f32 * 18.);
                    painter.circle_filled(pin, 3., if port["valid"] == true { t.accent } else { t.warning });
                    clipped.text(pin + vec2(8., 0.), Align2::LEFT_CENTER, format!("{arrow} {label}"), FontId::proportional(11.), t.text);
                    let response = ui.interact(
                        Rect::from_min_size(pin - vec2(0., 8.), vec2(180., 16.)),
                        ui.id().with(("cad_port_label", id, i)),
                        Sense::hover(),
                    );
                    response.on_hover_text(format!(
                        "{}\n{}\n{}\nA = {} mm²\n{}",
                        p["id"].as_str().unwrap_or(""),
                        app.tr(&format!("ui.mks_face.{role}")),
                        p["body"].as_str().unwrap_or(""),
                        p["area_mm2"],
                        p["area_face"].as_str().unwrap_or("")
                    ));
                    node_handle(ui, &format!("mks:interface:{}", p["id"].as_str().unwrap_or("")), pin);
                }
                for entry in app
                    .session
                    .doc
                    .metadata
                    .get("org.solidcraft.mks-library")
                    .and_then(|v| v.get("items"))
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    if block["members"].as_array().into_iter().flatten().any(|m| m["component"] == entry["component"])
                        && entry["attachment_port"].is_string()
                    {
                        let node = format!("payload:{}", entry["occurrence"]);
                        let payload = placed(draft, &node, area.min, rect.min - area.min + vec2(0., 210.), vec2(180., 64.));
                        painter.rect_filled(payload, t.radius_md, t.accent_soft);
                        painter.text(
                            payload.center(),
                            Align2::CENTER_CENTER,
                            format!("{}\n{} kg", app.tr("ui.mks_lib.payload"), entry["mass_kg"]),
                            FontId::proportional(12.),
                            t.text,
                        );
                        wire(rect.center_bottom(), payload.center_top(), t.accent);
                        let response = ui.interact(payload, ui.id().with(&node), Sense::click_and_drag());
                        move_node(draft, &node, payload, area.min, &response);
                        node_handle(ui, &format!("mks:payload:{}", entry["occurrence"]), payload.center());
                        if response.double_clicked()
                            && let Some(oid) = entry["occurrence"].as_u64()
                        {
                            draft.insert = Some(crate::mks_library_ui::Insert::edit_mass(oid, entry["mass_kg"].as_f64().unwrap_or(1.)));
                        }
                    }
                }
                for (index, load) in draft.settings["port_loads"].as_array().cloned().unwrap_or_default().iter().enumerate() {
                    if !block["ports"].as_array().into_iter().flatten().any(|p| p["port"]["id"] == load["port"]) {
                        continue;
                    }
                    let node = format!("force:{}", load["port"].as_str().unwrap_or(""));
                    let force = placed(draft, &node, area.min, rect.min - area.min + vec2(20., 270. + index as f32 * 80.), vec2(230., 60.));
                    painter.rect_filled(force, t.radius_md, t.accent_soft);
                    painter.text(
                        force.center(),
                        Align2::CENTER_CENTER,
                        format!("{}\n{}: {}", app.tr("ui.workspace.force"), load["port"].as_str().unwrap_or(""), load["force_n"]),
                        FontId::proportional(12.),
                        t.text,
                    );
                    wire(force.center_top(), rect.center_bottom(), t.accent);
                    let response = ui.interact(force, ui.id().with(&node), Sense::click_and_drag());
                    move_node(draft, &node, force, area.min, &response);
                    if response.double_clicked() {
                        draft.selected_body = Some(id.to_owned());
                    }
                }
            }
            let mut obstacles = cards.clone();
            obstacles.extend(source_obstacles);
            obstacles.extend(joint_obstacles);
            obstacles.extend(extra_obstacles);
            obstacles.push(shared_scope);
            if draft.shared_control {
                obstacles.push(controller);
            }
            for (index, (from, id)) in measurements.iter().enumerate() {
                node_handle(ui, &format!("mks:wire:lane:{id}"), pos2(right_edge + 28. + index as f32 * 14., from.y));
                let to = pos2(shared_scope.left(), shared_scope.top() + 14. + index as f32 * 14.);
                painter.circle_filled(to, 3., t.axis_y);
                let points = crate::mks_wire_ui::measurement(
                    ui,
                    &format!("sensor:{id}"),
                    *from,
                    to,
                    &obstacles,
                    area,
                    (index + 1, right_edge + 28. + index as f32 * 14.),
                );
                path(&points, t.axis_y, 1.7);
                if let Some(p) = points.get(points.len().saturating_sub(2)) {
                    let d = (to - *p).normalized();
                    let back = to - d * 8.;
                    painter.add(egui::Shape::convex_polygon(
                        vec![to, back + vec2(-d.y, d.x) * 3., back - vec2(-d.y, d.x) * 3.],
                        t.axis_y,
                        Stroke::NONE,
                    ));
                }
                if let Some(pair) = points.windows(2).max_by(|a, b| (a[0] - a[1]).length_sq().total_cmp(&(b[0] - b[1]).length_sq())) {
                    node_handle(ui, &format!("mks:wire:sensor:{id}"), pair[0].lerp(pair[1], 0.5));
                    for (segment, pair) in points.windows(2).enumerate() {
                        let length = (pair[1] - pair[0]).length();
                        if length < 28. {
                            continue;
                        }
                        let a = if pair[0] == *from { pair[0].lerp(pair[1], 14. / length) } else { pair[0] };
                        let b = if pair[1] == to { pair[1].lerp(pair[0], 14. / length) } else { pair[1] };
                        let r = ui.interact(Rect::from_two_pos(a, b).expand(4.), ui.id().with(("measurement_wire", id, segment)), Sense::click());
                        if r.clicked() {
                            draft.diagram.selected_measure = Some(*id);
                            draft.diagram.selected_wire = None;
                        }
                    }
                } else {
                    painter.text(to - vec2(10., 0.), Align2::RIGHT_CENTER, app.tr("ui.mks_wire.route_blocked"), FontId::proportional(11.), t.warning);
                }
            }
            for (index, (from, to, id)) in explicit_routes.into_iter().enumerate() {
                let points = crate::mks_wire_ui::route_channel(ui, &format!("motion:{id}"), from, to, &obstacles, area, index + 1);
                path(&points, t.accent, if draft.diagram.selected_wire == Some(id) { 3.5 } else { 2. });
                if let Some(p) = points.get(points.len().saturating_sub(2)) {
                    let d = (to - *p).normalized();
                    let back = to - d * 8.;
                    painter.add(egui::Shape::convex_polygon(
                        vec![to, back + vec2(-d.y, d.x) * 3., back - vec2(-d.y, d.x) * 3.],
                        t.accent,
                        Stroke::NONE,
                    ));
                }
                if let Some(pair) = points.windows(2).max_by(|a, b| (a[0] - a[1]).length_sq().total_cmp(&(b[0] - b[1]).length_sq())) {
                    node_handle(ui, &format!("mks:wire:line:{id}"), pair[0].lerp(pair[1], 0.5));
                    for (segment, pair) in points.windows(2).enumerate() {
                        let length = (pair[1] - pair[0]).length();
                        if length < 28. {
                            continue;
                        }
                        let start = if pair[0] == from { pair[0].lerp(pair[1], 14. / length) } else { pair[0] };
                        let end = if pair[1] == to { pair[1].lerp(pair[0], 14. / length) } else { pair[1] };
                        let r = ui.interact(Rect::from_two_pos(start, end).expand(4.), ui.id().with(("wire_segment", id, segment)), Sense::click());
                        if r.clicked() {
                            draft.diagram.selected_wire = Some(id);
                            draft.diagram.selected_measure = None;
                        }
                    }
                }
                if points.is_empty() {
                    painter.text(
                        to - vec2(0., 20.),
                        Align2::CENTER_BOTTOM,
                        app.tr("ui.mks_wire.route_blocked"),
                        FontId::proportional(11.),
                        t.warning,
                    );
                }
            }
            if draft.diagram.wiring
                && let Some(pointer) = ui.input(|i| i.pointer.hover_pos())
            {
                let pointer = ui.ctx().layer_transform_to_global(ui.layer_id()).map_or(pointer, |t| t.inverse() * pointer);
                if let Some(id) = draft.diagram.wire
                    && let Some(from) = source_ports.get(&id)
                {
                    wire(*from, pointer, t.accent);
                }
                if let Some(id) = draft.diagram.measure
                    && let Some(port) = measurement_ports.get(&id)
                {
                    wire(*port, pointer, t.axis_y);
                }
            }
            if draft.diagram.dragging && ui.input(|i| i.pointer.any_released()) {
                draft.diagram.dragging = false;
                draft.diagram.wire = None;
                draft.diagram.measure = None;
            }
            if drop.double_clicked() {
                draft.quick_insert = true;
            }
            if let Some(kind) = drop.dnd_release_payload::<String>() {
                insert_kind(draft, &kind, blocks);
            }
        });
    draft.scene = scene;
}

fn scope(app: &SolveApp, ui: &mut egui::Ui, joints: &[Value], state: &Value, draft: &mut Draft) {
    ui.separator();
    ui.heading(app.tr("ui.mks_ext.scope_title"));
    ui.horizontal_wrapped(|ui| {
        for (key, quantity, component) in [("all_forces", 0, 3), ("all_angles", 1, 4), ("all_positions", 3, 4)] {
            let button = ui.button(app.tr(&format!("ui.mks_workflow.{key}")));
            scenario::publish_handle(&format!("mks:scope:{key}"), button.rect.center());
            if button.clicked() {
                for joint in joints {
                    if quantity == 1 && joint["kind"] != "revolute" || quantity == 3 && joint["kind"] != "slider" {
                        continue;
                    }
                    if let Some(id) = joint["id"].as_u64() {
                        draft.sensors.insert(id);
                        draft.unscoped.remove(&id);
                        draft.outputs.entry(id).or_default().insert(component);
                    }
                }
                draft.chart_kind = quantity;
                draft.layout_dirty = true;
            }
        }
        let clear = ui.button(app.tr("ui.mks_workflow.clear_scope"));
        scenario::publish_handle("mks:scope:clear", clear.rect.center());
        if clear.clicked() {
            draft.sensors.clear();
            draft.layout_dirty = true;
        }
    });
    if !draft.unscoped.is_empty() {
        ui.small(app.tr("ui.mks_cad.scope_unconnected"));
    }
    ui.small(app.tr(if draft.chart_kind == 0 { "ui.mks_graph.force_note" } else { "ui.mks_ext.kinematic_note" }));
    ui.horizontal_wrapped(|ui| {
        for j in joints {
            if let Some(id) = j["id"].as_u64() {
                let mut on = draft.sensors.contains(&id);
                let checkbox = ui.checkbox(&mut on, j["name"].as_str().unwrap_or(""));
                scenario::publish_handle(&format!("mks:scope:joint:{id}"), checkbox.rect.center());
                if checkbox.changed() {
                    draft.layout_dirty = true;
                    if on {
                        draft.sensors.insert(id);
                        draft.unscoped.remove(&id);
                    } else {
                        draft.sensors.remove(&id);
                    }
                }
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        for (index, key) in ["force", "angle", "angular_speed", "position", "speed"].iter().enumerate() {
            let r = ui.selectable_value(&mut draft.chart_kind, index, app.tr(&format!("ui.mks_ext.plot.{key}")));
            scenario::publish_handle(&format!("mks:scope:quantity:{index}"), r.rect.center());
        }
    });
    if draft.chart_kind == 0 {
        egui::ComboBox::from_id_salt("joint_force_component")
            .selected_text(["Fx [N]", "Fy [N]", "Fz [N]", "|F| [N]"].get(draft.component).copied().unwrap_or(""))
            .show_ui(ui, |ui| {
                for (i, label) in ["Fx [N]", "Fy [N]", "Fz [N]", "|F| [N]"].iter().enumerate() {
                    ui.selectable_value(&mut draft.component, i, *label);
                }
            });
    }
    if state["result_current"] != true {
        ui.label(app.tr("ui.mks_graph.no_result"));
        return;
    }
    let result = &state["data"]["result"];
    if draft.chart_kind > 0 && !result["joint_coordinates_si"].is_object() {
        ui.label(app.tr("ui.mks_ext.recalculate"));
        return;
    }

    if state["data"]["job"]["settings"]["drivers"].as_array().is_some_and(|v| !v.is_empty())
        && !result["numerics"]["generalized_alpha_spectral_radius"].as_f64().is_some_and(|v| v > 0. && v < 1.)
    {
        ui.label(app.tr("ui.mks_graph.recalculate_forces"));
        return;
    }
    let times = result["times_s"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let t = Tokens::get();
    let colors = [t.axis_x, t.axis_y, t.axis_z, t.accent, t.warning, t.sketch_accent];
    let mut series = Vec::new();
    let mut low = 0f64;
    let mut high = 0f64;
    for id in draft.sensors.iter().filter(|id| !draft.unscoped.contains(id)).take(8) {
        let outputs = draft.outputs.get(id).cloned().unwrap_or_else(|| BTreeSet::from([draft.component]));
        let angular = joints.iter().find(|j| j["id"] == *id).is_some_and(|j| j["kind"] == "revolute");
        for component in outputs.iter().copied().filter(|i| *i < 6) {
            let family = if component < 4 {
                0
            } else if angular {
                if component == 4 { 1 } else { 2 }
            } else if component == 4 {
                3
            } else {
                4
            };
            if family != draft.chart_kind {
                continue;
            }
            let rows = if component < 4 { &result["joint_forces_local_n"][id.to_string()] } else { &result["joint_coordinates_si"][id.to_string()] };
            let rows = rows.as_array().map(Vec::as_slice).unwrap_or(&[]);
            let points: Vec<_> = times
                .iter()
                .zip(rows)
                .skip(if component < 4 { 1 } else { 0 })
                .filter_map(|(time, row)| {
                    let value = if component >= 4 {
                        row.get(component - 4)?.as_f64()? * if angular { 180. / std::f64::consts::PI } else { 1000. }
                    } else if component == 3 {
                        (0..3).filter_map(|i| row.get(i).and_then(Value::as_f64)).map(|v| v * v).sum::<f64>().sqrt()
                    } else {
                        row.get(component)?.as_f64()?
                    };
                    if !value.is_finite() {
                        return None;
                    }
                    low = low.min(value);
                    high = high.max(value);
                    Some((time.as_f64()?, value))
                })
                .collect();
            if !points.is_empty() {
                series.push((*id, component, points));
            }
        }
    }
    if series.is_empty() {
        ui.label(app.tr("ui.mks_ext.no_channels"));
        return;
    }
    ui.label(["N", "deg", "deg/s", "mm", "mm/s"].get(draft.chart_kind).copied().unwrap_or(""));
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width().min(1200.), 260.), Sense::click());
    let plot = rect.shrink2(vec2(65., 28.));
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, t.radius_md, t.field);
    let end = times.last().and_then(Value::as_f64).unwrap_or(1.).max(1e-9);
    let span = (high - low).max(if high.abs() < 1e-12 { 1. } else { high.abs() * 0.02 });
    let map = |x: f64, y: f64| pos2(plot.left() + plot.width() * (x / end) as f32, plot.bottom() - plot.height() * ((y - low) / span) as f32);
    for i in 0..=5 {
        let f = i as f64 / 5.;
        let y = low + span * f;
        let x = end * f;
        painter.line_segment([map(0., y), map(end, y)], Stroke::new(0.5, t.border));
        painter.line_segment([map(x, low), map(x, low + span)], Stroke::new(0.5, t.border));
        let label = if y != 0. && (y.abs() < 0.01 || y.abs() > 1e5) { format!("{y:.2e}") } else { format!("{y:.2}") };
        painter.text(pos2(plot.left() - 6., map(0., y).y), Align2::RIGHT_CENTER, label, FontId::proportional(11.), t.text);
        painter.text(pos2(map(x, low).x, plot.bottom() + 6.), Align2::CENTER_TOP, format!("{x:.2}"), FontId::proportional(11.), t.text);
    }
    painter.text(rect.right_bottom(), Align2::RIGHT_BOTTOM, "t [s]", FontId::proportional(12.), t.text);
    for (i, (id, component, points)) in series.iter().enumerate() {
        let color = colors[i % colors.len()];
        for pair in points.windows(2) {
            if let [a, b] = pair {
                painter.line_segment([map(a.0, a.1), map(b.0, b.1)], Stroke::new(2., color));
            }
        }
        let name = joints.iter().find(|j| j["id"] == *id).and_then(|j| j["name"].as_str()).unwrap_or("");
        painter.text(
            rect.min + vec2(70. + (i % 4) as f32 * 180., 4. + (i / 4) as f32 * 16.),
            Align2::LEFT_TOP,
            format!("{name} · {}", ["Fx", "Fy", "Fz", "|F|", "q", "dq/dt"][*component]),
            FontId::proportional(11.),
            color,
        );
        scenario::publish_handle(&format!("mks:scope:series:{id}"), rect.min + vec2(70. + i as f32 * 150., 8.));
    }
    let cursor = draft.playhead;
    painter.line_segment([map(cursor, low), map(cursor, low + span)], Stroke::new(1., t.accent));
    if let Some(pos) = response.hover_pos() {
        let time = ((pos.x - plot.left()) / plot.width()).clamp(0., 1.) as f64 * end;
        painter.line_segment([map(time, low), map(time, low + span)], Stroke::new(1., t.text));
        if response.clicked() {
            draft.playhead = time;
            draft.sample = times.partition_point(|t| t.as_f64().unwrap_or(0.) <= time).saturating_sub(1);
            draft.playing = false;
            draft.preview = true;
            draft.clock = -1.;
        }
        let mut tooltip = format!("t = {time:.4} s");
        for (id, component, points) in &series {
            if let Some((_, value)) = points.iter().min_by(|a, b| (a.0 - time).abs().total_cmp(&(b.0 - time).abs())) {
                let name = joints.iter().find(|j| j["id"] == *id).and_then(|j| j["name"].as_str()).unwrap_or("");
                tooltip.push_str(&format!("\n{name}: {} = {value:.4}", ["Fx [N]", "Fy [N]", "Fz [N]", "|F| [N]", "q", "dq/dt"][*component]));
            }
        }
        response.on_hover_text(tooltip);
    }
    scenario::publish_handle("mks:scope:plot", plot.center());
    scenario::publish_handle("mks:scope:forces", rect.center());
}

fn playback(app: &mut SolveApp, ui: &mut egui::Ui, state: &Value, draft: &mut Draft) {
    if state["result_current"] != true {
        return;
    }
    let times = state["data"]["result"]["times_s"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    if times.is_empty() {
        return;
    }
    draft.sample = draft.sample.min(times.len() - 1);
    ui.horizontal(|ui| {
        if draft.diagram.playback_speed <= 0. {
            draft.diagram.playback_speed = 1.;
        }
        ui.label(app.tr("ui.mks_workflow.playback_speed"));
        let speed = ui.add(egui::DragValue::new(&mut draft.diagram.playback_speed).range(0.1..=4.).speed(0.1).suffix("×"));
        scenario::publish_handle("mks:playback:speed", speed.rect.center());
        let repeat = ui.checkbox(&mut draft.diagram.repeat, app.tr("ui.mks_workflow.repeat"));
        scenario::publish_handle("mks:playback:repeat", repeat.rect.center());
        let preview = ui.checkbox(&mut draft.preview, app.tr("ui.mks_graph.preview"));
        scenario::publish_handle("mks:graph:preview", preview.rect.center());
        let play = crate::toolbar::engineering_action(ui, "drive", app.tr("ui.mks_graph.play"), draft.playing);
        scenario::publish_handle("mks:graph:play", play.rect.center());
        if play.clicked() {
            draft.sample = 0;
            draft.playhead = 0.;
            draft.playing = true;
            draft.clock = -1.;
            draft.preview = true;
        }
        let pause = crate::toolbar::engineering_action(ui, "settings", app.tr("ui.mks_graph.pause"), !draft.playing);
        scenario::publish_handle("mks:graph:pause", pause.rect.center());
        if pause.clicked() {
            draft.playing = false;
        }
        let sample = ui.add(egui::DragValue::new(&mut draft.sample).range(0..=times.len() - 1));
        if sample.changed() {
            draft.playhead = times.get(draft.sample).and_then(Value::as_f64).unwrap_or(0.);
            draft.clock = -1.;
        }
        scenario::publish_handle("mks:graph:sample", sample.rect.center());
        ui.label(format!("t = {:.4} s", draft.playhead));
    });
    if draft.playing {
        let now = ui.input(|i| i.time);
        if draft.clock < 0. {
            draft.clock = now;
        }
        let end = times.last().and_then(Value::as_f64).unwrap_or(0.);
        let next = draft.playhead + (now - draft.clock).max(0.) * draft.diagram.playback_speed;
        draft.playhead = if draft.diagram.repeat && end > 0. { next.rem_euclid(end) } else { next.min(end) };
        draft.clock = now;
        draft.sample = times.partition_point(|t| t.as_f64().unwrap_or(0.) <= draft.playhead).saturating_sub(1);
        if draft.playhead >= end && !draft.diagram.repeat {
            draft.playing = false;
        }
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
    }
}

fn placed(draft: &Draft, id: &str, origin: egui::Pos2, default: egui::Vec2, size: egui::Vec2) -> Rect {
    let p =
        draft.positions.get(id).copied().filter(|p| p.iter().all(|v| v.is_finite() && *v >= 0. && *v <= 20000.)).unwrap_or([default.x, default.y]);
    Rect::from_min_size(origin + vec2(p[0], p[1]), size)
}
// A JSON draft can contain `8` where the typed engine settings return `8.0`.
// Normalise only exactly representable integers; large IDs remain lossless.
fn display_numbers(value: &mut Value) {
    match value {
        Value::Number(n) => {
            let exact = n.is_f64()
                || n.as_u64().is_some_and(|v| v <= 9_007_199_254_740_992)
                || n.as_i64().is_some_and(|v| v.unsigned_abs() <= 9_007_199_254_740_992);
            if exact && let Some(v) = n.as_f64() {
                *value = json!(v);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(display_numbers),
        Value::Object(items) => items.values_mut().for_each(display_numbers),
        _ => {}
    }
}
fn move_node(draft: &mut Draft, id: &str, rect: Rect, origin: egui::Pos2, response: &egui::Response) {
    if response.dragged_by(egui::PointerButton::Primary) {
        let p = rect.min - origin + response.drag_delta();
        draft.positions.insert(id.to_owned(), [p.x.clamp(0., 20000.), p.y.clamp(0., 20000.)]);
        draft.layout_dirty = true;
    }
}
fn attach(draft: &mut Draft, id: u64, kind: &str, slider: bool) {
    if ["independent", "motion", "cosine", "quintic", "sine"].contains(&kind) {
        draft.diagram.new_for_joint(id, slider);
        if ["cosine", "quintic", "sine"].contains(&kind)
            && let Some((_, source)) = &mut draft.diagram.edit
        {
            source["profile"] = json!(kind);
        }
        draft.selected = None;
        return;
    }
    if ["motion", "cosine", "quintic", "sine"].contains(&kind) {
        let mut drivers = draft.settings["drivers"].as_array().cloned().unwrap_or_default();
        if !drivers.iter().any(|d| d["joint"] == id) {
            drivers.push(json!({"joint":id,"displacement":if slider{"50 mm"}else{"30 deg"},"ramp_s":0.5}));
        }
        if kind != "motion"
            && let Some(drive) = drivers.iter_mut().find(|d| d["joint"] == id)
        {
            drive["profile"] = json!(kind);
        }
        draft.settings["drivers"] = json!(drivers);
        draft.joint_tab = 1;
    } else if ["sensor", "scope", "position_sensor", "velocity_sensor"].contains(&kind) {
        if kind == "scope" {
            draft.unscoped.remove(&id);
        } else {
            draft.unscoped.insert(id);
        }
        draft.sensors.insert(id);
        draft.layout_dirty = true;
        draft.scope_visible = true;
        draft.joint_tab = 2;
        if kind == "position_sensor" || kind == "velocity_sensor" {
            let component = if kind == "position_sensor" { 4 } else { 5 };
            draft.outputs.entry(id).or_default().insert(component);
            draft.chart_kind = if slider {
                if component == 4 { 3 } else { 4 }
            } else if component == 4 {
                1
            } else {
                2
            };
        }
    }
    draft.selected = Some(id);
}
fn library(app: &SolveApp, ui: &mut egui::Ui, blocks: &[Value], joints: &[Value], draft: &mut Draft) {
    ui.heading(app.tr("ui.workspace.library"));
    scenario::publish_handle("mks:library", ui.min_rect().center());
    let search_field = ui.add(egui::TextEdit::singleline(&mut draft.library).hint_text(app.tr("ui.workspace.search")));
    scenario::publish_handle("mks:library:search", search_field.rect.center());
    ui.small(app.tr("ui.workspace.drop_hint"));
    let search = draft.library.to_lowercase();
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (category, entries) in [
            ("bodies", vec!["world", "body", "payload", "rom"]),
            ("joints", vec!["rigid", "revolute", "slider"]),
            ("sources", vec!["independent", "controller", "motion", "cosine", "quintic", "sine"]),
            ("measurements", vec!["sensor", "position_sensor", "velocity_sensor"]),
            ("outputs", vec!["scope", "force"]),
        ] {
            let entries: Vec<_> = entries
                .into_iter()
                .filter(|kind| {
                    search.is_empty()
                        || kind.contains(&search)
                        || app.tr(&format!("ui.mks_ext.block.{kind}")).to_lowercase().contains(&search)
                        || app.tr(&format!("ui.mks_ext.category.{category}")).to_lowercase().contains(&search)
                })
                .collect();
            if entries.is_empty() {
                continue;
            }
            egui::CollapsingHeader::new(app.tr(&format!("ui.mks_ext.category.{category}")))
                .id_salt(category)
                .default_open(true)
                .open(if search.is_empty() { None } else { Some(true) })
                .show(ui, |ui| {
                    for kind in entries {
                        let label = app.tr(&format!("ui.mks_ext.block.{kind}")).to_owned();
                        ui.horizontal(|ui| {
                            let response = ui
                                .dnd_drag_source(ui.id().with(kind), kind.to_owned(), |ui| {
                                    ui.group(|ui| {
                                        ui.set_min_width(150.);
                                        ui.horizontal(|ui| {
                                            let icon = match kind {
                                                "world" => "render_ground",
                                                "body" | "payload" | "rom" => "component",
                                                "rigid" => "rigid_group",
                                                "revolute" => "joint",
                                                "slider" => "move",
                                                "scope" | "sensor" | "position_sensor" | "velocity_sensor" => "graph",
                                                _ => "drive",
                                            };
                                            let (rect, _) = ui.allocate_exact_size(vec2(24., 24.), Sense::hover());
                                            crate::icons::paint_icon(ui.painter(), rect, icon, 1.);
                                            ui.strong(label);
                                        });
                                    });
                                })
                                .response;
                            scenario::publish_handle(&format!("mks:library:{kind}"), response.rect.center());
                            let add = ui.small_button("+");
                            scenario::publish_handle(&format!("mks:library:add:{kind}"), add.rect.center());
                            if add.clicked() {
                                insert_kind(draft, kind, blocks);
                            }
                        });
                    }
                });
        }
        egui::CollapsingHeader::new(app.tr("ui.workspace.cad_bodies")).default_open(true).show(ui, |ui| {
            for b in blocks {
                let name = b["members"].as_array().into_iter().flatten().filter_map(|m| m["name"].as_str()).collect::<Vec<_>>().join(" + ");
                if search.is_empty() || name.to_lowercase().contains(&search) {
                    let id = b["id"].as_str().unwrap_or("");
                    let title = if b["grounded"] == true { format!("World / Ground — {name}") } else { format!("Body — {name}") };
                    if ui.button(title).clicked() {
                        draft.selected_body = Some(id.to_owned());
                    }
                }
            }
        });
        egui::CollapsingHeader::new(app.tr("ui.workspace.joints")).default_open(true).show(ui, |ui| {
            for j in joints {
                if ui.button(j["name"].as_str().unwrap_or("")).clicked() {
                    draft.selected = j["id"].as_u64();
                }
            }
            ui.small(app.tr("ui.workspace.cad_owned"));
        });
        ui.separator();
        ui.small(app.tr("ui.workspace.backend"));
    });
}

fn node_handle(ui: &egui::Ui, name: &str, point: egui::Pos2) {
    let point = ui.ctx().layer_transform_to_global(ui.layer_id()).map_or(point, |t| t * point);
    scenario::publish_handle(name, point);
}
/// Drops do not own pointer focus: the dragged source still owns it. Test the
/// released point in graph coordinates rather than the target's hover response.
fn released_over(ui: &egui::Ui, rect: Rect) -> bool {
    let (released, pointer) = ui.ctx().input(|i| (i.pointer.any_released(), i.pointer.interact_pos()));
    if !released {
        return false;
    }
    let Some(point) = pointer else {
        return false;
    };
    let local = ui.ctx().layer_transform_to_global(ui.layer_id()).map_or(point, |t| t.inverse() * point);
    rect.contains(local)
}
fn insert_kind(draft: &mut Draft, kind: &str, blocks: &[Value]) {
    draft.quick_insert = false;
    match kind {
        "rom" => {
            draft.selected_body = blocks.iter().find(|b| b["grounded"] != true).and_then(|b| b["id"].as_str()).map(str::to_owned);
        }
        "independent" => draft.diagram.new_source(),
        "motion" | "sensor" | "position_sensor" | "velocity_sensor" | "cosine" | "quintic" | "sine" => draft.pending_target = Some(kind.to_owned()),
        "controller" => {
            draft.shared_control = true;
            draft.control_open = true;
            draft.layout_dirty = true;
        }
        "world" => {
            draft.selected_body = blocks.iter().find(|b| b["grounded"] == true).and_then(|b| b["id"].as_str()).map(str::to_owned);
        }
        "body" | "payload" | "rigid" | "revolute" | "slider" => draft.insert = Some(crate::mks_library_ui::Insert::new(kind)),
        "scope" => {
            draft.scope_visible = true;
        }
        _ => {}
    }
}
fn control_editor(app: &SolveApp, ui: &mut egui::Ui, joints: &[Value], draft: &mut Draft) {
    ui.add(egui::Label::new(app.tr("ui.mks_ext.control_note")).wrap());
    if draft.diagram.graph["inputs"].as_object().is_some_and(|v| !v.is_empty()) {
        ui.small(app.tr("ui.mks_wire.control_note"));
    }
    let mut drivers = draft.settings["drivers"].as_array().cloned().unwrap_or_default();
    egui::Grid::new("controller_channels").num_columns(5).min_col_width(65.).spacing(vec2(16., 8.)).striped(true).show(ui, |ui| {
        for title in ["Joint", "q / Δq", "Ramp [s]", app.tr("ui.mks_ext.profile"), "Scope"] {
            ui.strong(title);
        }
        ui.end_row();
        for j in joints {
            let Some(id) = j["id"].as_u64() else { continue };
            if draft.diagram.managed(id) {
                continue;
            }
            let existing = drivers.iter().find(|d| d["joint"] == id).cloned();
            let mut on = existing.is_some();
            ui.checkbox(&mut on, j["name"].as_str().unwrap_or(""));
            let mut drive = existing.unwrap_or_else(|| json!({"joint":id,"displacement":if j["kind"]=="slider"{"0 mm"}else{"0 deg"},"ramp_s":0.5}));
            let mut expression = drive["displacement"].as_str().unwrap_or("").to_owned();
            let field = ui.add_sized([240., 26.], egui::TextEdit::singleline(&mut expression).desired_width(240.));
            scenario::publish_handle(&format!("mks:control:expression:{id}"), field.rect.center());
            drive["displacement"] = json!(expression);
            let mut ramp = drive["ramp_s"].as_f64().unwrap_or(0.5);
            ui.add(egui::DragValue::new(&mut ramp).range(0.001..=10.).speed(0.05));
            drive["ramp_s"] = json!(ramp);
            crate::mks_source_ui::fields(app, ui, &mut drive, id, false);
            let mut sensor = draft.sensors.contains(&id);
            if ui.checkbox(&mut sensor, "F").changed() {
                if sensor {
                    draft.sensors.insert(id);
                } else {
                    draft.sensors.remove(&id);
                }
                draft.layout_dirty = true;
            }
            ui.end_row();
            drivers.retain(|d| d["joint"] != id);
            if on {
                drivers.push(drive);
            }
        }
    });
    drivers.sort_by_key(|d| d["joint"].as_u64().unwrap_or(0));
    draft.settings["drivers"] = json!(drivers);
    if ui.button(app.tr("ui.mks_lib.all_forces")).clicked() {
        draft.unscoped.clear();
        draft.sensors = joints.iter().filter_map(|j| j["id"].as_u64()).collect();
        draft.layout_dirty = true;
        draft.scope_visible = true;
    }
    let close = ui.button(app.tr("ui.mks_graph.close"));
    scenario::publish_handle("mks:control:close", close.rect.center());
    if close.clicked() {
        draft.control_open = false;
    }
}
