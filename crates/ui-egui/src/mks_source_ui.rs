//! Source configuration and a cached preview obtained from the registered engine command.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::{Value, json};
pub fn fields(app: &SolveApp, ui: &mut egui::Ui, drive: &mut Value, id: u64, frequency: bool) {
    let old = drive["profile"].as_str().unwrap_or("cosine").to_owned();
    let mut profile = old.clone();
    let combo =
        egui::ComboBox::from_id_salt(("mks_profile", id)).width(130.).selected_text(app.tr(&format!("ui.mks_ext.{profile}"))).show_ui(ui, |ui| {
            for kind in ["cosine", "quintic", "sine"] {
                let r = ui.selectable_value(&mut profile, kind.to_owned(), app.tr(&format!("ui.mks_ext.{kind}")));
                scenario::publish_handle(&format!("mks:source:profile:{id}:{kind}"), r.rect.center());
            }
        });
    scenario::publish_handle(&format!("mks:source:profile:{id}"), combo.response.rect.center());
    if profile != old || drive.get("profile").is_some() {
        drive["profile"] = json!(profile);
    }
    if frequency && profile == "sine" {
        ui.horizontal(|ui| {
            ui.label(app.tr("ui.mks_ext.frequency"));
            let mut f = drive["frequency_hz"].as_f64().unwrap_or(1.);
            let r = ui.add(egui::DragValue::new(&mut f).range(0.01..=20.).speed(0.1));
            scenario::publish_handle(&format!("mks:source:frequency:{id}"), r.rect.center());
            if r.changed() {
                drive["frequency_hz"] = json!(f);
            }
        });
    }
}
pub fn preview(app: &mut SolveApp, ui: &mut egui::Ui, drive: &Value, duration: f64) {
    let profile = drive["profile"].as_str().unwrap_or("cosine");
    let f = drive["frequency_hz"].as_f64().unwrap_or(1.);
    let params = json!({"profile":profile,"duration_s":duration,"ramp_s":drive["ramp_s"].as_f64().unwrap_or(0.5),"frequency_hz":f,"samples":((duration*f*40.).ceil() as usize).clamp(121,1001)});
    let key = egui::Id::new(("mks_source_preview", app.docs.active));
    let old = ui.ctx().data(|d| d.get_temp::<(Value, Value)>(key));
    let result = if let Some((_, value)) = old.filter(|(p, _)| p == &params) {
        value
    } else {
        let value = app.session.execute("mks.signal.sample", &params).unwrap_or_else(|e| json!({"error":e.to_string()}));
        ui.ctx().data_mut(|d| d.insert_temp(key, (params, value.clone())));
        value
    };
    if let Some(error) = result["error"].as_str() {
        ui.colored_label(Tokens::get().warning, error);
        return;
    }
    let t = Tokens::get();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 110.), egui::Sense::hover());
    let plot = rect.shrink2(egui::vec2(28., 16.));
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, t.radius_md, t.field);
    let ymin = if profile == "sine" { -1. } else { 0. };
    let span = 1. - ymin;
    let map =
        |x: f64, y: f64| egui::pos2(plot.left() + plot.width() * (x / duration) as f32, plot.bottom() - plot.height() * ((y - ymin) / span) as f32);
    painter.line_segment([map(0., 0.), map(duration, 0.)], egui::Stroke::new(1., t.border));
    let points: Vec<_> =
        result["points"].as_array().into_iter().flatten().filter_map(|p| Some(map(p.get(0)?.as_f64()?, p.get(1)?.as_f64()?))).collect();
    for pair in points.windows(2) {
        if let [a, b] = pair {
            painter.line_segment([*a, *b], egui::Stroke::new(2., t.accent));
        }
    }
    painter.text(rect.left_top(), egui::Align2::LEFT_TOP, "q/A", egui::FontId::proportional(12.), t.text);
    painter.text(rect.right_bottom(), egui::Align2::RIGHT_BOTTOM, "t [s]", egui::FontId::proportional(12.), t.text);
    scenario::publish_handle("mks:source:preview", rect.center());
}
