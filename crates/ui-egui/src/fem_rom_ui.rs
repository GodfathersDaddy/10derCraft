//! Generic component FEM/CMS setup. Computation is off the window thread;
//! the engine accepts a result only against unchanged CAD and FEM inputs.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
#[derive(Clone)]
struct Draft {
    id: String,
    block: String,
    body: String,
    interfaces: Vec<String>,
    rigid_attachments: Vec<Value>,
    joint_interfaces: std::collections::BTreeMap<u64, String>,
    young: f64,
    poisson: f64,
    density: f64,
    divisions: [u32; 3],
    modes: u32,
    force: [f64; 3],
    angle: f64,
    duration: f64,
    error: String,
    mesh_file: String,
    stored_mesh: Option<Value>,
    element_order: u32,
    moment: [f64; 3],
}
impl Default for Draft {
    fn default() -> Self {
        Self {
            id: "Component_1".into(),
            block: String::new(),
            body: String::new(),
            interfaces: Vec::new(),
            rigid_attachments: Vec::new(),
            joint_interfaces: Default::default(),
            young: 210000.,
            poisson: 0.3,
            density: 7850.,
            divisions: [4, 2, 2],
            modes: 6,
            force: [0., 0., -100.],
            angle: 15.,
            duration: 1.,
            error: String::new(),
            mesh_file: String::new(),
            stored_mesh: None,
            element_order: 2,
            moment: [0.; 3],
        }
    }
}
struct Running {
    done: Mutex<Option<Result<Value, String>>>,
}
fn key(app: &SolveApp) -> egui::Id {
    egui::Id::new(("component_fem", app.docs.active, app.session.document_id.clone()))
}
pub fn selected(app: &SolveApp, ctx: &egui::Context) -> Option<String> {
    let selected = ctx.data(|d| d.get_temp::<String>(key(app).with("selected")));
    if selected.as_deref() == Some("__beam__") {
        return None;
    }
    selected.or_else(|| {
        app.session.doc.fem_models.as_ref()?.get("models")?.as_object()?.iter().find(|(_, m)| m["result"].is_object()).map(|(id, _)| id.clone())
    })
}
pub fn select_beam(app: &SolveApp, ctx: &egui::Context) {
    ctx.data_mut(|d| {
        d.insert_temp(key(app).with("selected"), "__beam__".to_owned());
    });
}
pub fn select_model(app: &SolveApp, ctx: &egui::Context, id: &str) {
    ctx.data_mut(|data| {
        data.insert_temp(key(app).with("selected"), id.to_owned());
        data.insert_temp(egui::Id::new(("fem_results_visible", app.docs.active)), true);
    });
}
pub fn prepare_block(app: &mut SolveApp, ctx: &egui::Context, block: &str) {
    let draft = Draft { id: format!("ROM_{block}"), block: block.into(), ..Draft::default() };
    ctx.data_mut(|data| data.insert_temp(key(app), draft));
    app.ui.project_page = "fem".into();
    app.ui.tab = "FEM / ROM".into();
    app.ui.engineering_dialog = Some("fem_rom".into());
}
fn model_draft(app: &mut SolveApp, id: &str) -> Result<Draft, String> {
    let state = app.session.execute("fem.model.read", &json!({"id":id,"detail":true})).map_err(|e| e.to_string())?;
    let record = &state["data"];
    let job = &record["job"];
    let mut draft = Draft {
        id: id.into(),
        block: record["block"].as_str().unwrap_or_default().into(),
        body: record["body"].as_str().unwrap_or_default().into(),
        interfaces: job["interfaces"].as_array().into_iter().flatten().filter_map(|p| p["id"].as_str().map(str::to_owned)).collect(),
        rigid_attachments: record["rigid_attachments"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|a| json!({"body":a["owner"]["source"],"path":a["owner"]["path"],"interface":a["interface"]}))
            .collect(),
        joint_interfaces: record["joint_interfaces"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|j| Some((j["joint"].as_u64()?, j["interface"].as_str()?.to_owned())))
            .collect(),
        ..Draft::default()
    };
    draft.young = job["material"]["young_mpa"].as_f64().unwrap_or(draft.young);
    draft.poisson = job["material"]["poisson_ratio"].as_f64().unwrap_or(draft.poisson);
    draft.density = job["material"]["density_kg_m3"].as_f64().unwrap_or(draft.density);
    draft.modes = job["modes"].as_u64().unwrap_or(6) as u32;
    draft.element_order = job["mesh"]["element_order"].as_u64().unwrap_or(2) as u32;
    if job["mesh"]["nodes_mm"].is_array() {
        draft.stored_mesh = Some(json!({"nodes_mm":job["mesh"]["nodes_mm"],"tetrahedra":job["mesh"]["tetrahedra"]}));
    }
    for (i, value) in draft.divisions.iter_mut().enumerate() {
        *value = job["mesh"]["divisions"][i].as_u64().unwrap_or(u64::from(*value)) as u32;
    }
    for i in 0..3 {
        draft.force[i] = job["loads"][0]["wrench_world_n_nm"][i].as_f64().unwrap_or(draft.force[i]);
        draft.moment[i] = job["loads"][0]["wrench_world_n_nm"][i + 3].as_f64().unwrap_or(0.);
    }
    Ok(draft)
}
pub fn edit_model(app: &mut SolveApp, ctx: &egui::Context, id: &str) -> Result<(), String> {
    let draft = model_draft(app, id)?;
    ctx.data_mut(|data| data.insert_temp(key(app), draft));
    app.ui.project_page = "fem".into();
    app.ui.tab = "FEM / ROM".into();
    app.ui.engineering_dialog = Some("fem_rom".into());
    Ok(())
}
/// Automation status for a desktop calculation without exposing the dense ROM.
pub fn inspect(app: &SolveApp, ctx: &egui::Context) -> Value {
    let k = key(app);
    let running = ctx.data(|d| d.get_temp::<Arc<Running>>(k.with("running")));
    let last = ctx.data(|d| d.get_temp::<egui::Id>(egui::Id::new("component_fem_last_started")));
    json!({"running":running.is_some(),"ready":running.as_ref().is_some_and(|r|r.done.lock().is_ok_and(|v|v.is_some())),"error":ctx.data(|d|d.get_temp::<String>(k.with("error"))),"pending_in_other_document":last.is_some_and(|v|v!=k&&ctx.data(|d|d.get_temp::<Arc<Running>>(v.with("running"))).is_some())})
}
pub fn poll(app: &mut SolveApp, ctx: &egui::Context) {
    let k = key(app);
    let running = ctx.data(|d| d.get_temp::<Arc<Running>>(k.with("running")));
    if let Some(running) = running {
        let done = running.done.lock().ok().and_then(|mut v| v.take());
        if let Some(done) = done {
            ctx.data_mut(|d| d.remove::<Arc<Running>>(k.with("running")));
            let outcome = done.and_then(|p| {
                let id = p["id"].as_str().unwrap_or_default().to_owned();
                app.run("fem.result.accept", p).map_err(|e| e.to_string())?;
                Ok(id)
            });
            match outcome {
                Ok(id) => {
                    ctx.data_mut(|d| {
                        d.insert_temp(k.with("selected"), id);
                        d.insert_temp(egui::Id::new(("fem_results_visible", app.docs.active)), true);
                    });
                    app.session.studio.interface_preview = false;
                    app.ui.engineering_dialog = None;
                }
                Err(e) => {
                    ctx.data_mut(|d| {
                        d.insert_temp(k.with("error"), e);
                    });
                }
            }
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}
fn start(app: &SolveApp, ctx: &egui::Context, inputs: Value) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let k = key(app);
        let mut scratch = app.session.scratch();
        let running = Arc::new(Running { done: Mutex::new(None) });
        let worker = running.clone();
        let ctx_worker = ctx.clone();
        std::thread::Builder::new()
            .name("component-fem".into())
            .spawn(move || {
                let r = scratch.execute("fem.rom.reduce", &inputs).map_err(|e| e.to_string());
                if let Ok(mut done) = worker.done.lock() {
                    *done = Some(r);
                }
                ctx_worker.request_repaint();
            })
            .map_err(|e| e.to_string())?;
        ctx.data_mut(|d| {
            d.insert_temp(k.with("running"), running);
            d.insert_temp(egui::Id::new("component_fem_last_started"), k);
        });
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (app, ctx, inputs);
        Err("Local FEM requires the desktop application".into())
    }
}
pub fn show(app: &mut SolveApp, ui: &mut egui::Ui) {
    // Bound the inner form to the shared card's actual content region. A
    // world-height scroll area can otherwise keep controls below the card's
    // clip without considering them scrollable.
    let height = (ui.ctx().content_rect().height() * 0.3).clamp(Tokens::get().control_height * 4., Tokens::get().control_height * 10.);
    egui::ScrollArea::vertical().id_salt("component_fem_form").max_height(height).auto_shrink([false, true]).show(ui, |ui| form(app, ui));
}
fn form(app: &mut SolveApp, ui: &mut egui::Ui) {
    let k = key(app);
    let mut d = ui.ctx().data(|v| v.get_temp::<Draft>(k)).unwrap_or_default();
    let cache_key = k.with("state");
    let cached = ui.ctx().data(|v| v.get_temp::<(u64, Value)>(cache_key));
    let state = match cached {
        Some((rev, state)) if rev == app.session.revision => state,
        _ => {
            let state = app.session.execute("fem.model.read", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}));
            ui.ctx().data_mut(|v| v.insert_temp(cache_key, (app.session.revision, state.clone())));
            state
        }
    };
    ui.heading(app.tr("ui.fem_rom.open"));
    ui.add(egui::Label::new(app.tr("ui.fem_rom.workflow")).wrap());
    if let Some(e) = state["error"].as_str() {
        ui.colored_label(Tokens::get().warning, e);
        return;
    }
    let scopes = state["scopes"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    if d.block.is_empty()
        && let Some(b) = scopes.iter().find(|b| b["interfaces"].as_array().is_some_and(|a| !a.is_empty())).or_else(|| scopes.first())
    {
        d.block = b["id"].as_str().unwrap_or_default().into();
    }
    let label = |scope: &Value| {
        let names = scope["bodies"].as_array().into_iter().flatten().filter_map(|b| b["source"].as_str()).collect::<Vec<_>>().join(" + ");
        if names.is_empty() { scope["id"].as_str().unwrap_or_default().to_owned() } else { names }
    };
    egui::ComboBox::from_id_salt("fem_block").selected_text(scopes.iter().find(|b| b["id"] == d.block).map(label).unwrap_or_default()).show_ui(
        ui,
        |ui| {
            for scope in scopes {
                if let Some(id) = scope["id"].as_str() {
                    let row = ui.selectable_value(&mut d.block, id.into(), label(scope));
                    scenario::publish_handle(&format!("fem:rom:block:{id}"), row.rect.center());
                    if row.changed() {
                        d.body.clear();
                        d.interfaces.clear();
                        d.rigid_attachments.clear();
                        d.joint_interfaces.clear();
                        d.stored_mesh = None;
                        d.mesh_file.clear();
                    }
                }
            }
        },
    );
    let scope = scopes.iter().find(|b| b["id"] == d.block);
    if let Some(scope) = scope {
        egui::ComboBox::from_id_salt("fem_part").selected_text(if d.body.is_empty() { app.tr("ui.fem_rom.whole") } else { &d.body }).show_ui(
            ui,
            |ui| {
                if ui.selectable_value(&mut d.body, String::new(), app.tr("ui.fem_rom.whole")).changed() {
                    d.stored_mesh = None;
                    d.mesh_file.clear();
                    d.interfaces.clear();
                    d.rigid_attachments.clear();
                    d.joint_interfaces.clear();
                }
                for b in scope["bodies"].as_array().into_iter().flatten() {
                    if let Some(name) = b["source"].as_str() {
                        if ui.selectable_value(&mut d.body, name.into(), name).changed() {
                            d.stored_mesh = None;
                            d.mesh_file.clear();
                            d.interfaces.clear();
                            d.rigid_attachments.clear();
                            d.joint_interfaces.clear();
                        }
                    }
                }
            },
        );
        ui.label(app.tr("ui.fem_rom.port_order"));
        ui.horizontal_wrapped(|ui| {
            for id in scope["interfaces"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                if !d.body.is_empty() && !scope["surface_owners"].as_array().into_iter().flatten().any(|p| p["id"] == id && p["body"] == d.body) {
                    continue;
                }
                let mut on = d.interfaces.iter().any(|v| v == id);
                let check = ui.checkbox(&mut on, id);
                scenario::publish_handle(&format!("fem:rom:interface:{id}"), check.rect.center());
                if check.changed() {
                    if on {
                        d.interfaces.push(id.into());
                    } else {
                        d.interfaces.retain(|v| v != id);
                    }
                }
            }
        });
        for (i, id) in d.interfaces.iter().enumerate() {
            ui.label(format!("{}: {}{}", i + 1, id, if i == 0 { format!(" · {}", app.tr("ui.fem_rom.reference")) } else { String::new() }));
        }
        if !d.body.is_empty() {
            ui.collapsing(app.tr("ui.fem_rom.attachments"), |ui| {
                ui.add(egui::Label::new(app.tr("ui.fem_rom.attachments_help")).wrap());
                let mut attachments = Vec::new();
                for body in scope["bodies"].as_array().into_iter().flatten().filter(|b| b["source"] != d.body) {
                    let name = body["source"].as_str().unwrap_or_default();
                    let mut port = d
                        .rigid_attachments
                        .iter()
                        .find(|a| a["body"] == name && a["path"] == body["path"])
                        .and_then(|a| a["interface"].as_str())
                        .filter(|id| d.interfaces.iter().any(|p| p == id))
                        .unwrap_or_default()
                        .to_owned();
                    ui.horizontal(|ui| {
                        ui.label(name);
                        let row = egui::ComboBox::from_id_salt(("rom_attachment", name, body["path"].to_string()))
                            .selected_text(if port.is_empty() { app.tr("ui.fem_rom.unassigned") } else { &port })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut port, String::new(), app.tr("ui.fem_rom.unassigned"));
                                for id in &d.interfaces {
                                    ui.selectable_value(&mut port, id.clone(), id);
                                }
                            });
                        scenario::publish_handle(&format!("fem:rom:attachment:{name}"), row.response.rect.center());
                    });
                    if !port.is_empty() {
                        attachments.push(json!({"body":name,"path":body["path"],"interface":port}));
                    }
                }
                d.rigid_attachments = attachments;
            });
        }
        ui.collapsing(app.tr("ui.fem_rom.joint_interfaces"), |ui| {
            ui.add(egui::Label::new(app.tr("ui.fem_rom.joint_interfaces_help")).wrap());
            for joint in scope["joints"].as_array().into_iter().flatten() {
                if let Some(id) = joint["id"].as_u64() {
                    let port = d.joint_interfaces.entry(id).or_default();
                    if !d.interfaces.contains(port) {
                        port.clear();
                    }
                    ui.horizontal(|ui| {
                        ui.label(joint["name"].as_str().unwrap_or_default());
                        let row = egui::ComboBox::from_id_salt(("rom_joint_interface", id))
                            .selected_text(if port.is_empty() { app.tr("ui.fem_rom.auto_surface") } else { port.as_str() })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(port, String::new(), app.tr("ui.fem_rom.auto_surface"));
                                for id in &d.interfaces {
                                    ui.selectable_value(port, id.clone(), id);
                                }
                            });
                        scenario::publish_handle(&format!("fem:rom:joint_interface:{id}"), row.response.rect.center());
                    });
                }
            }
        });
    }
    ui.horizontal(|ui| {
        ui.label(app.tr("ui.fem_rom.id"));
        let r = ui.text_edit_singleline(&mut d.id);
        scenario::publish_handle("fem:rom:id", r.rect.center());
    });
    ui.collapsing(app.tr("ui.fem_rom.material_mesh"), |ui| {
        for (label, value) in [("E [MPa]", &mut d.young), ("ν", &mut d.poisson), ("ρ [kg/m³]", &mut d.density)] {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.add(egui::DragValue::new(value));
            });
        }
        ui.horizontal(|ui| {
            ui.label(app.tr("ui.fem_rom.divisions"));
            for n in &mut d.divisions {
                ui.add(egui::DragValue::new(n).range(1..=8));
            }
        });
        ui.horizontal(|ui| {
            ui.label(app.tr("ui.fem_rom.modes"));
            ui.add(egui::DragValue::new(&mut d.modes).range(1..=24));
        });
        ui.label(app.tr("ui.fem_rom.mesh_note"));
        if d.stored_mesh.is_some() {
            ui.label(app.tr("ui.fem_rom.stored_mesh"));
        }
        ui.horizontal(|ui| {
            ui.label(app.tr("ui.fem_rom.mesh_file"));
            ui.text_edit_singleline(&mut d.mesh_file);
        });
    });
    ui.horizontal(|ui| {
        ui.label(app.tr("ui.fem_rom.force"));
        for f in &mut d.force {
            ui.add(egui::DragValue::new(f));
        }
    });
    ui.horizontal(|ui| {
        ui.label(app.tr("ui.fem_rom.moment"));
        for moment in &mut d.moment {
            ui.add(egui::DragValue::new(moment));
        }
    });
    ui.horizontal(|ui| {
        ui.label(app.tr("ui.fem_rom.motion"));
        ui.add(egui::DragValue::new(&mut d.angle).suffix("°").range(-180. ..=180.));
        ui.add(egui::DragValue::new(&mut d.duration).suffix(" s").range(0.01..=5.));
    });
    let running = ui.ctx().data(|v| v.get_temp::<Arc<Running>>(k.with("running"))).is_some();
    ui.horizontal(|ui|{for (flexible,label,handle) in [(false,"ui.fem_rom.reduce","fem:rom:reduce"),(true,"ui.fem_rom.flexible","fem:rom:flexible")]{
        let b=ui.add_enabled(!running&&!d.interfaces.is_empty(),egui::Button::new(app.tr(label)));scenario::publish_handle(handle,b.rect.center());
        if b.clicked(){let outcome=(||{
            let mut inputs=json!({"id":d.id,"block":d.block,"interfaces":d.interfaces,"material":{"young_mpa":d.young,"poisson_ratio":d.poisson,"density_kg_m3":d.density},"divisions":d.divisions,"modes":d.modes,"element_order":d.element_order});
            if !d.body.is_empty(){inputs["body"]=json!(d.body);}
            inputs["rigid_attachments"]=json!(d.rigid_attachments);
            inputs["joint_interfaces"]=json!(d.joint_interfaces.iter().filter(|(_,p)|!p.is_empty()).map(|(id,p)|json!({"joint":id,"interface":p})).collect::<Vec<_>>());
            if !d.mesh_file.is_empty(){inputs["mesh_path"]=json!(d.mesh_file);} else if let Some(mesh)=&d.stored_mesh {inputs["mesh"]=mesh.clone();}
            app.run("fem.model.create",inputs).map_err(|e|e.to_string())?;
            start(app,ui.ctx(),json!({"id":d.id,"flexible":flexible,"force_n":d.force,"moment_nm":d.moment,"motion":{"duration_s":d.duration,"steps":1000,"samples":101,"angle_deg":d.angle,"frequency_hz":1.,"ramp_s":d.duration.min(0.2)}}))
        })();d.error=outcome.err().unwrap_or_default();}
    }});
    if running {
        ui.spinner();
        ui.label(app.tr("ui.fem_rom.running"));
    }
    if let Some(e) = ui.ctx().data(|v| v.get_temp::<String>(k.with("error"))) {
        ui.colored_label(Tokens::get().warning, e);
    }
    if !d.error.is_empty() {
        ui.colored_label(Tokens::get().warning, &d.error);
    }
    for m in state["models"].as_array().into_iter().flatten() {
        ui.separator();
        let id = m["id"].as_str().unwrap_or_default();
        ui.horizontal(|ui| {
            ui.strong(id);
            let edit = ui.add_enabled(!running, egui::Button::new(app.tr("ui.fem_rom.edit")));
            scenario::publish_handle(&format!("fem:rom:edit:{id}"), edit.rect.center());
            if edit.clicked() {
                match model_draft(app, id) {
                    Ok(draft) => d = draft,
                    Err(error) => d.error = error,
                }
            }
            ui.label(app.tr(if m["current"] == true { "ui.fem_rom.current" } else { "ui.fem_rom.stale" }));
            let mut enabled = m["enabled"] == true;
            let b = ui.add_enabled(
                enabled || m["has_result"] == true && m["current"] == true,
                egui::Checkbox::new(&mut enabled, app.tr("ui.fem_rom.bind")),
            );
            scenario::publish_handle(&format!("fem:rom:bind:{id}"), b.rect.center());
            if b.changed()
                && let Err(e) = app.run("fem.mks.bind", json!({"id":id,"enabled":enabled}))
            {
                d.error = e.to_string();
            }
            if ui.add_enabled(m["has_result"] == true && m["current"] == true, egui::Button::new(app.tr("ui.analysis.results"))).clicked() {
                ui.ctx().data_mut(|v| {
                    v.insert_temp(k.with("selected"), id.to_owned());
                    v.insert_temp(egui::Id::new(("fem_results_visible", app.docs.active)), true);
                });
                app.session.studio.interface_preview = false;
                app.ui.engineering_dialog = None;
            }
        });
        ui.label(format!(
            "{} kg · {} → {} DOF · {} mm · {} MPa",
            m["mass_kg"], m["full_dofs"], m["reduced_dofs"], m["max_displacement_mm"], m["max_stress_mpa"]
        ));
        ui.label(format!("f [Hz]: {}", m["frequencies_hz"]));
    }
    ui.ctx().data_mut(|v| v.insert_temp(k, d));
}
