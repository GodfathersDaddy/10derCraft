//! Thin local dynamics controls and a sampled trajectory view, without changing CAD poses.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::{Value, json};
#[derive(Clone)]
struct Draft {
    duration: f64,
    steps: u32,
    samples: u32,
    gravity: [f64; 3],
    assumed: bool,
    use_load: bool,
    port: String,
    force: [f64; 3],
    moment: [f64; 3],
    error: String,
    sample: usize,
    playing: bool,
    clock: f64,
    use_driver: bool,
    driver_joint: u64,
    driver_displacement: String,
    ramp: f64,
    preview: bool,
    speed: f64,
}
impl Default for Draft {
    fn default() -> Self {
        Self {
            duration: 2.,
            steps: 1000,
            samples: 21,
            gravity: [0., 0., -9.81],
            assumed: false,
            use_load: false,
            port: String::new(),
            force: [0., 0., 0.],
            moment: [0.; 3],
            error: String::new(),
            sample: 0,
            playing: false,
            clock: 0.,
            use_driver: false,
            driver_joint: 0,
            driver_displacement: "30 deg".into(),
            ramp: 1.,
            preview: false,
            speed: 1.,
        }
    }
}
pub fn show(app: &mut SolveApp, ui: &mut egui::Ui, model: &Value) {
    let key = egui::Id::new(("mks_dynamics", app.docs.active, app.session.path.clone()));
    let stored = app
        .session
        .doc
        .metadata
        .get("org.solidcraft.assembly-simulation")
        .and_then(|v| v.get("job"))
        .and_then(|v| v.get("settings"))
        .cloned()
        .unwrap_or_else(|| json!({}));
    let complex = stored["port_loads"].as_array().is_some_and(|a| a.len() > 1 || a.iter().any(|v| v["pressure_mpa"].is_number()))
        || stored["drivers"].as_array().is_some_and(|a| a.len() > 1);
    let mut draft = ui.data_mut(|d| d.get_temp::<Draft>(key)).unwrap_or_else(|| {
        let mut d = Draft::default();
        if model["blocks"].as_array().into_iter().flatten().filter(|b| b["grounded"] != true).count() > 5 {
            d.samples = 11;
        }
        if stored.is_object() && stored.get("duration_s").is_some() {
            d.duration = stored["duration_s"].as_f64().unwrap_or(d.duration);
            d.steps = stored["steps"].as_u64().and_then(|v| u32::try_from(v).ok()).unwrap_or(d.steps);
            d.samples = stored["samples"].as_u64().and_then(|v| u32::try_from(v).ok()).unwrap_or(d.samples);
            d.assumed = stored["allow_assumed_density"].as_bool().unwrap_or(false);
            for (i, g) in d.gravity.iter_mut().enumerate() {
                *g = stored["gravity_m_s2"].get(i).and_then(Value::as_f64).unwrap_or(*g);
            }
            if let Some(drive) = stored["drivers"].get(0) {
                d.use_driver = true;
                d.driver_joint = drive["joint"].as_u64().unwrap_or(0);
                d.driver_displacement = drive["displacement"].as_str().unwrap_or("0 deg").into();
                d.ramp = drive["ramp_s"].as_f64().unwrap_or(1.);
            }
            if let Some(load) = stored["port_loads"].get(0)
                && !load["pressure_mpa"].is_number()
            {
                d.use_load = true;
                d.port = load["port"].as_str().unwrap_or("").into();
                for (i, v) in d.force.iter_mut().enumerate() {
                    *v = load["force_n"].get(i).and_then(Value::as_f64).unwrap_or(0.);
                }
                for (i, v) in d.moment.iter_mut().enumerate() {
                    *v = load["moment_nm"].get(i).and_then(Value::as_f64).unwrap_or(0.);
                }
            }
        }
        d
    });
    let cache = key.with("state");
    let cached = ui.data_mut(|d| d.get_temp::<(u64, Value)>(cache));
    let state = if let Some((_, v)) = cached.filter(|(r, _)| *r == app.session.revision) {
        v
    } else {
        let v = app.session.execute("mks.job.state", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}));
        ui.data_mut(|d| d.insert_temp(cache, (app.session.revision, v.clone())));
        v
    };
    let controls=ui.collapsing("Dynamik · lokaler Exudyn-Lauf",|ui|{
        ui.small("Offene Ketten mit Dreh-/Schubgelenken, Start aus Ruhe in aktueller CAD-Pose. Keine Zahnradkopplung, Anschläge oder Kontakt.");
        ui.horizontal(|ui|{ui.label("Dauer [s]");ui.add(egui::DragValue::new(&mut draft.duration).speed(0.1));ui.label("Schritte");ui.add(egui::DragValue::new(&mut draft.steps));ui.label("Ausgabepunkte");ui.add(egui::DragValue::new(&mut draft.samples));});
        ui.horizontal(|ui|{ui.label("Gravitation Welt [m/s²]");for v in &mut draft.gravity{ui.add(egui::DragValue::new(v).speed(0.1));}});
        ui.checkbox(&mut draft.assumed,"Angenommene CAD-Standarddichte ausdrücklich zulassen");
        let drive=ui.checkbox(&mut draft.use_driver,"Gelenkbewegung vorgeben: weiche Kosinusrampe, danach halten");
        scenario::publish_handle("mks:dynamics:driver:enable",drive.rect.center());
        if draft.use_driver{
            scenario::publish_handle("mks:dynamics:driver:enabled",drive.rect.center());
            let joints:Vec<_>=app.session.doc.assembly.joints.iter().filter(|j|!j.suppressed && matches!(j.kind,solidcraft_engine::doc::joints::JointKind::Revolute|solidcraft_engine::doc::joints::JointKind::Slider)).collect();
            if draft.driver_joint==0 && let Some(j)=joints.first(){draft.driver_joint=j.id;if j.kind==solidcraft_engine::doc::joints::JointKind::Slider{draft.driver_displacement="50 mm".into();}}
            egui::ComboBox::from_id_salt("mks_drive_joint").selected_text(joints.iter().find(|j|j.id==draft.driver_joint).map(|j|j.name.as_str()).unwrap_or("Gelenk wählen")).show_ui(ui,|ui|{
                for j in joints{if ui.selectable_value(&mut draft.driver_joint,j.id,&j.name).clicked(){draft.driver_displacement=if j.kind==solidcraft_engine::doc::joints::JointKind::Slider{"50 mm"}else{"30 deg"}.into();}}
            });
            ui.horizontal(|ui|{ui.label("Inkrement / Parameterausdruck");ui.text_edit_singleline(&mut draft.driver_displacement);ui.label("Rampenzeit [s]");ui.add(egui::DragValue::new(&mut draft.ramp).speed(0.1));});
        }
        ui.checkbox(&mut draft.use_load,"Konstante Kraft/Moment an einem Projektport (mitbewegtes Portsystem)");
        if draft.use_load{
            ui.horizontal(|ui|{ui.label("Port-ID");ui.text_edit_singleline(&mut draft.port);});
            for (label,values) in [("Kraft [N]",&mut draft.force),("Moment [Nm]",&mut draft.moment)]{ui.horizontal(|ui|{ui.label(label);for v in values{ui.add(egui::DragValue::new(v));}});}
            ui.small("Druck und mehrere Lasten sind über mks.job.create/run verfügbar. Dieser Dialog legt einen neuen Lauf mit höchstens einer Kraft-/Momentenlast an.");
        }
        if complex{ui.colored_label(Tokens::get().warning,"Gespeicherter Lauf enthält mehrere Antriebe/Lasten oder Druck. Über mks.job.run bearbeiten; der einfache Dialog überschreibt ihn nicht.");}
        let r=ui.add_enabled(!complex,egui::Button::new("Dynamik lokal berechnen"));scenario::publish_handle("mks:dynamics:run",r.rect.center());
        if r.clicked(){
            let loads=if draft.use_load{json!([{"port":draft.port,"force_n":draft.force,"moment_nm":draft.moment}])}else{json!([])};
            let drivers=if draft.use_driver{json!([{"joint":draft.driver_joint,"displacement":draft.driver_displacement,"ramp_s":draft.ramp}])}else{json!([])};
            match app.run("mks.job.run",json!({"duration_s":draft.duration,"steps":draft.steps,"samples":draft.samples,"gravity_m_s2":draft.gravity,"allow_assumed_density":draft.assumed,"port_loads":loads,"drivers":drivers})){
                Ok(_)=>{draft.error.clear();draft.sample=0;draft.playing=false;},Err(e)=>draft.error=e,
            }
        }
        if !draft.error.is_empty(){ui.colored_label(Tokens::get().warning,&draft.error);}
    });
    scenario::publish_handle("mks:dynamics:controls", controls.header_response.rect.center());
    let has_result = state["data"]["result"].is_object();
    if has_result {
        let current = state["result_current"] == true;
        let r =
            ui.label(if current { "Dynamikergebnis aktuell" } else { "Dynamikergebnis historisch – CAD/Material/Parameter/Position geändert" });
        scenario::publish_handle(if current { "mks:dynamics:current" } else { "mks:dynamics:stale" }, r.rect.center());
        if current {
            let result = &state["data"]["result"];
            let times = result["times_s"].as_array().map(Vec::as_slice).unwrap_or(&[]);
            if !times.is_empty() {
                draft.sample = draft.sample.min(times.len() - 1);
                ui.horizontal_wrapped(|ui| {
                    let play = ui.button(if draft.playing { "Pause" } else { "Verlauf abspielen" });
                    scenario::publish_handle("mks:dynamics:play", play.rect.center());
                    if play.clicked() {
                        draft.playing = !draft.playing;
                        if draft.playing && draft.sample == times.len() - 1 {
                            draft.sample = 0;
                        }
                        draft.clock = ui.input(|i| i.time);
                    }
                    let previous = ui.button("Zurück");
                    scenario::publish_handle("mks:dynamics:previous", previous.rect.center());
                    if previous.clicked() {
                        draft.sample = draft.sample.saturating_sub(1);
                        draft.playing = false;
                    }
                    let next = ui.button("Weiter");
                    scenario::publish_handle("mks:dynamics:next", next.rect.center());
                    if next.clicked() {
                        draft.sample = (draft.sample + 1).min(times.len() - 1);
                        draft.playing = false;
                    }
                    let sample = ui.add(egui::Slider::new(&mut draft.sample, 0..=times.len() - 1).text("Zeitschritt"));
                    scenario::publish_handle("mks:dynamics:sample", sample.rect.center());
                    if sample.changed() {
                        draft.playing = false;
                    }
                    ui.label(format!("t = {:.4} s", times.get(draft.sample).and_then(Value::as_f64).unwrap_or(0.)));
                    ui.label("Tempo");
                    ui.add(egui::DragValue::new(&mut draft.speed).range(0.05..=10.).speed(0.05).suffix("×"));
                });
                if draft.playing {
                    let interval = (times.get(1).and_then(Value::as_f64).unwrap_or(0.001) - times.first().and_then(Value::as_f64).unwrap_or(0.))
                        .max(1e-6)
                        / draft.speed;
                    let elapsed = ui.input(|i| i.time) - draft.clock;
                    let advance = (elapsed / interval).floor().max(0.) as usize;
                    if advance > 0 {
                        draft.sample = draft.sample.saturating_add(advance).min(times.len() - 1);
                        draft.clock += advance as f64 * interval;
                    }
                    if draft.sample == times.len() - 1 {
                        draft.playing = false;
                    }
                }
                if draft.playing {
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
                }
                trajectory(ui, result, draft.sample);
                let preview = ui.checkbox(&mut draft.preview, "CAD-Körper im 3D-Ergebnisfenster anzeigen");
                scenario::publish_handle("mks:3d:enable", preview.rect.center());
                if draft.preview {
                    crate::mks_result_view::show(app, ui.ctx(), draft.sample, &mut draft.preview);
                }
                ui.small(format!(
                    "Exudyn {} · max. Gelenkfehler {:.3e} m · Energieänderung {:.3e} J",
                    result["backend_version"].as_str().unwrap_or(""),
                    result["summary"]["max_position_constraint_m"].as_f64().unwrap_or(0.),
                    result["summary"]["max_energy_change_j"].as_f64().unwrap_or(0.)
                ));
                ui.small("Projektion der Blockschwerpunkte in X/Z. Energieänderung enthält die Arbeit äußerer Lasten und Antriebe. Die Wiedergabe verändert die CAD-Konstruktion nicht.");
            }
        }
    }
    ui.data_mut(|d| d.insert_temp(key, draft));
}
fn trajectory(ui: &mut egui::Ui, result: &Value, sample: usize) {
    let mut paths = Vec::new();
    let mut min = egui::vec2(f32::INFINITY, f32::INFINITY);
    let mut max = egui::vec2(f32::NEG_INFINITY, f32::NEG_INFINITY);
    for (id, rows) in result["trajectories"].as_object().into_iter().flatten() {
        let points: Vec<_> = rows
            .as_array()
            .into_iter()
            .flatten()
            .map(|r| egui::vec2(r.get(0).and_then(Value::as_f64).unwrap_or(0.) as f32, -r.get(2).and_then(Value::as_f64).unwrap_or(0.) as f32))
            .collect();
        for p in &points {
            min = min.min(*p);
            max = max.max(*p);
        }
        paths.push((id, points));
    }
    if paths.is_empty() {
        return;
    }
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width().min(700.), 150.), egui::Sense::hover());
    let t = Tokens::get();
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 3., t.field);
    painter.text(rect.left_top() + egui::vec2(8., 8.), egui::Align2::LEFT_TOP, "Schwerpunktbahn X/Z [m]", egui::FontId::proportional(10.), t.text);
    painter.text(
        rect.left_bottom() - egui::vec2(-8., 4.),
        egui::Align2::LEFT_BOTTOM,
        format!("X [{:.3}, {:.3}] m · Z [{:.3}, {:.3}] m", min.x, max.x, -max.y, -min.y),
        egui::FontId::proportional(10.),
        t.text,
    );
    let span = max - min;
    let scale = ((rect.width() - 40.) / span.x.max(0.001)).min((rect.height() - 30.) / span.y.max(0.001));
    let map = |p: egui::Vec2| rect.center() + (p - (min + max) * 0.5) * scale;
    for (id, points) in paths {
        for w in points.windows(2) {
            if let [a, b] = w {
                painter.line_segment([map(*a), map(*b)], egui::Stroke::new(1., t.border));
            }
        }
        if let Some(p) = points.get(sample) {
            let pos = map(*p);
            painter.circle_filled(pos, 4., t.accent);
            let right = pos.x > rect.right() - 70.;
            painter.text(
                pos + egui::vec2(if right { -8. } else { 8. }, 0.),
                if right { egui::Align2::RIGHT_CENTER } else { egui::Align2::LEFT_CENTER },
                id,
                egui::FontId::proportional(11.),
                t.text,
            );
        }
    }
    scenario::publish_handle("mks:dynamics:trajectory", rect.center());
}
