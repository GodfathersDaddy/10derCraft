//! User-facing CAD selection → native interface → native joint → derived MKS workflow.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::json;
#[derive(Clone)]
struct Draft {
    tab: usize,
    name: String,
    occurrence: u64,
    transfer: String,
    surface: bool,
    label: String,
    role: String,
    a: String,
    b: String,
    kind: String,
    joint_name: String,
    flip: bool,
    error: String,
}
impl Default for Draft {
    fn default() -> Self {
        Self {
            tab: 0,
            name: "IF_1".into(),
            occurrence: 0,
            transfer: "frame_wrench".into(),
            surface: true,
            label: String::new(),
            role: "bidirectional".into(),
            a: String::new(),
            b: String::new(),
            kind: "revolute".into(),
            joint_name: "Joint".into(),
            flip: false,
            error: String::new(),
        }
    }
}
pub fn show(app: &mut SolveApp, ui: &mut egui::Ui) {
    let key = ui.id().with(("cad_mks_interfaces", app.docs.active));
    let mut d = ui.ctx().data(|data| data.get_temp::<Draft>(key)).unwrap_or_default();
    let state = app.session.execute("mks.interface.read", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}));
    ui.horizontal(|ui| {
        ui.heading(app.tr("ui.mks_cad.title"));
        let b = ui.button(app.tr("ui.mks_cad.to_mks"));
        scenario::publish_handle("mks:cad:to_mks", b.rect.center());
        if b.clicked() {
            app.ui.engineering_dialog = None;
            app.ui.project_page = "mks".into();
            app.ui.tab = "MBS".into();
        }
    });
    ui.add(egui::Label::new(app.tr("ui.mks_cad.workflow")).wrap());
    let mut preview = app.session.studio.interface_preview;
    let toggle = ui.checkbox(&mut preview, app.tr("ui.fem_rom.interfaces"));
    scenario::publish_handle("mks:cad:highlight", toggle.rect.center());
    if toggle.changed() {
        let _ = app.run("mks.interface.display", json!({"enabled":preview}));
    }
    ui.separator();
    ui.horizontal(|ui| {
        for (tab, label) in ["ui.mks_cad.tab_capture", "ui.mks_cad.tab_joint", "ui.mks_cad.tab_overview"].iter().enumerate() {
            let b = ui.selectable_value(&mut d.tab, tab, app.tr(label));
            scenario::publish_handle(&format!("mks:cad:tab:{tab}"), b.rect.center());
        }
    });
    if d.tab == 0 {
        ui.strong(app.tr("ui.mks_cad.capture"));
        let selections = state["selection"].as_array().cloned().unwrap_or_default();
        if let Some(face) = selections.first() {
            ui.label(format!("{} · Face {}", face["body"].as_str().unwrap_or(""), face["index"]));
        } else {
            ui.colored_label(Tokens::get().warning, app.tr("ui.mks_cad.pick_hint"));
        }
        let selected_component = selections.first().and_then(|p| p["component"].as_u64());
        let occurrences: Vec<_> = state["occurrences"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|o| Some(o["component"].as_u64().unwrap_or(0)) == selected_component)
            .collect();
        if occurrences.len() == 1 {
            d.occurrence = occurrences[0]["id"].as_u64().unwrap_or(0);
        }
        ui.horizontal(|ui| {
            ui.label(app.tr("ui.mks_wire.name"));
            let r = ui.text_edit_singleline(&mut d.name);
            scenario::publish_handle("mks:cad:name", r.rect.center());
        });
        let combo = egui::ComboBox::from_id_salt("cad_interface_occurrence")
            .selected_text(occurrences.iter().find(|o| o["id"] == d.occurrence).and_then(|o| o["name"].as_str()).unwrap_or("—"))
            .show_ui(ui, |ui| {
                for o in &occurrences {
                    let id = o["id"].as_u64().unwrap_or(0);
                    let b = ui.selectable_value(&mut d.occurrence, id, o["name"].as_str().unwrap_or(""));
                    scenario::publish_handle(&format!("mks:cad:occurrence:{id}"), b.rect.center());
                }
            });
        scenario::publish_handle("mks:cad:occurrence", combo.response.rect.center());
        ui.horizontal_wrapped(|ui| {
            for (kind, label) in [("frame_wrench", "ui.mks_cad.frame"), ("point_force", "ui.mks_cad.point")] {
                let b = ui.selectable_value(&mut d.transfer, kind.into(), app.tr(label));
                scenario::publish_handle(&format!("mks:cad:transfer:{kind}"), b.rect.center());
            }
        });
        if d.transfer == "point_force" {
            d.surface = false;
        }
        ui.add_enabled_ui(d.transfer != "point_force", |ui| {
            let r = ui.checkbox(&mut d.surface, app.tr("ui.mks_face.surface"));
            scenario::publish_handle("mks:cad:surface", r.rect.center());
        });
        let save = ui.add_enabled(selections.len() == 1 && !occurrences.is_empty(), egui::Button::new(app.tr("ui.mks_cad.save")));
        scenario::publish_handle("mks:cad:capture", save.rect.center());
        if save.clicked() {
            match app.run(
                "mks.interface.capture",
                json!({"name":d.name,"occurrence":d.occurrence,"transfer":d.transfer,"surface":d.surface,"label":d.label,"role":d.role}),
            ) {
                Ok(_) => {
                    d.a = if d.a.is_empty() { d.name.clone() } else { d.a };
                    d.b = d.name.clone();
                    d.error.clear();
                }
                Err(e) => d.error = e,
            }
        }
        description(app, ui, &mut d);
        ui.collapsing(app.tr("ui.mks_face.details"), |ui| {
            ui.small(app.tr("ui.mks_cad.resultant_note"));
        });
        ui.separator();
    }
    if d.tab == 1 {
        ui.strong(app.tr("ui.mks_cad.connect"));
        for (port, slot) in [("B", &mut d.a), ("F", &mut d.b)] {
            let combo = egui::ComboBox::from_id_salt(("cad_interface_side", port)).selected_text(format!("{port}: {slot}")).show_ui(ui, |ui| {
                for origin in state["interfaces"].as_array().into_iter().flatten().filter(|o| o["valid"] == true) {
                    let name = origin["name"].as_str().unwrap_or("");
                    let b = ui.selectable_value(slot, name.into(), name);
                    scenario::publish_handle(&format!("mks:cad:{port}:{name}"), b.rect.center());
                }
            });
            scenario::publish_handle(&format!("mks:cad:{port}"), combo.response.rect.center());
        }
        ui.horizontal(|ui| {
            ui.label(app.tr("ui.mks_wire.name"));
            let b = ui.text_edit_singleline(&mut d.joint_name);
            scenario::publish_handle("mks:cad:joint_name", b.rect.center());
        });
        ui.horizontal(|ui| {
            for kind in ["rigid", "revolute", "slider"] {
                let b = ui.selectable_value(&mut d.kind, kind.into(), app.tr(&format!("ui.mks_ext.block.{kind}")));
                scenario::publish_handle(&format!("mks:cad:kind:{kind}"), b.rect.center());
            }
        });
        ui.checkbox(&mut d.flip, app.tr("ui.mks_cad.flip"));
        ui.small(app.tr("ui.mks_cad.align_note"));
        let b = ui.add_enabled(!d.a.is_empty() && !d.b.is_empty() && d.a != d.b, egui::Button::new(app.tr("ui.mks_cad.create_joint")));
        scenario::publish_handle("mks:cad:joint", b.rect.center());
        if b.clicked() {
            match app.run("mks.interface.connect", json!({"name":d.joint_name,"a":d.a,"b":d.b,"kind":d.kind,"flip":d.flip})) {
                Ok(_) => d.error.clear(),
                Err(e) => d.error = e,
            }
        }
        ui.separator();
    }
    if d.tab == 2 {
        egui::ScrollArea::vertical().max_height(200.).show(ui, |ui| {
            for origin in state["interfaces"].as_array().into_iter().flatten() {
                let name = origin["name"].as_str().unwrap_or("");
                let b = ui.selectable_label(
                    d.name == name,
                    format!(
                        "{} · {}",
                        origin["name"].as_str().unwrap_or(""),
                        if origin["valid"] == true { app.tr("ui.mks_cad.bound") } else { origin["error"].as_str().unwrap_or("?") }
                    ),
                );
                if b.clicked() {
                    d.name = name.into();
                    d.label = origin["mechanical"]["port"]["label"].as_str().unwrap_or("").into();
                    d.role = origin["mechanical"]["port"]["role"].as_str().unwrap_or("bidirectional").into();
                }
                if let Some(area) = origin["mechanical"]["port"]["area_mm2"].as_f64() {
                    ui.small(format!("A = {area:.2} mm² · {}", origin["mechanical"]["port"]["area_face"].as_str().unwrap_or("")));
                }
            }
        });
        ui.label(&d.name);
        description(app, ui, &mut d);
        let rename = ui.button(app.tr("ui.mks_face.rename"));
        scenario::publish_handle("mks:cad:describe", rename.rect.center());
        if rename.clicked() {
            match app.run("mks.interface.describe", json!({"name":d.name,"label":d.label,"role":d.role})) {
                Ok(_) => d.error.clear(),
                Err(e) => d.error = e,
            }
        }
    }
    ui.separator();
    if let Some(error) = state["error"].as_str() {
        ui.colored_label(Tokens::get().warning, error);
    }
    if !d.error.is_empty() {
        ui.colored_label(Tokens::get().warning, &d.error);
    }
    ui.ctx().data_mut(|data| data.insert_temp(key, d));
}
fn description(app: &SolveApp, ui: &mut egui::Ui, d: &mut Draft) {
    ui.horizontal(|ui| {
        ui.label(app.tr("ui.mks_face.label"));
        let r = ui.add(egui::TextEdit::singleline(&mut d.label).desired_width(210.));
        scenario::publish_handle("mks:cad:label", r.rect.center());
        let role = egui::ComboBox::from_id_salt("cad_force_role").selected_text(app.tr(&format!("ui.mks_face.{}", d.role))).show_ui(ui, |ui| {
            for role in ["load_in", "load_out", "bidirectional"] {
                let r = ui.selectable_value(&mut d.role, role.into(), app.tr(&format!("ui.mks_face.{role}")));
                scenario::publish_handle(&format!("mks:cad:role:{role}"), r.rect.center());
            }
        });
        scenario::publish_handle("mks:cad:role", role.response.rect.center());
    });
}
