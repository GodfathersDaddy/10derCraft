//! Export a frozen result on a worker, keeping the CAD/MKS window responsive.
use crate::{SolveApp, scenario};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
#[derive(Clone)]
struct Export {
    open: bool,
    path: String,
    fps: u32,
    width: u32,
    height: u32,
    job: Option<Job>,
    message: String,
}
pub fn show(app: &SolveApp, ui: &mut egui::Ui, current: bool) {
    let key = egui::Id::new(("mks_video_export", app.docs.active, app.session.path.clone()));
    let mut state = ui.ctx().data(|d| d.get_temp::<Export>(key)).unwrap_or_else(|| Export {
        open: false,
        path: app
            .session
            .path
            .as_ref()
            .map(|p| std::path::Path::new(p).with_extension("avi").to_string_lossy().into_owned())
            .unwrap_or_else(|| "MKS-Simulation.avi".into()),
        fps: 30,
        width: 640,
        height: 360,
        job: None,
        message: String::new(),
    });
    let b = ui.add_enabled_ui(current, |ui| crate::toolbar::engineering_action(ui, "save", app.tr("ui.mks_video.title"), state.open)).inner;
    scenario::publish_handle("mks:video:open", b.rect.center());
    if b.clicked() {
        state.open = true;
    }
    if let Some(job) = &state.job {
        let done = job.lock().ok().and_then(|mut v| v.take());
        if let Some(done) = done {
            state.message = match done {
                Ok(v) => format!("{}: {} ({} frames, {} fps)", app.tr("ui.mks_video.saved"), v["path"].as_str().unwrap_or(""), v["frames"], v["fps"]),
                Err(e) => e,
            };
            state.job = None;
        } else {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
    if state.open {
        let closed = crate::ui_kit::Window::new(key.with("window"), "save", app.tr("ui.mks_video.title")).cards(2.).show_body(ui.ctx(), |ui| {
            ui.add(egui::Label::new(app.tr("ui.mks_video.note")).wrap());
            ui.add_enabled_ui(state.job.is_none(), |ui| {
                let path = ui.add(egui::TextEdit::singleline(&mut state.path).desired_width(ui.available_width()));
                scenario::publish_handle("mks:video:path", path.rect.center());
                ui.horizontal(|ui| {
                    ui.label("FPS");
                    ui.add(egui::DragValue::new(&mut state.fps).range(1..=60));
                    ui.label(app.tr("ui.mks_video.size"));
                    ui.add(egui::DragValue::new(&mut state.width).range(64..=1024));
                    ui.label("×");
                    ui.add(egui::DragValue::new(&mut state.height).range(64..=1024));
                });
                let save = ui.add_enabled(current, egui::Button::new(app.tr("ui.mks_video.save")));
                scenario::publish_handle("mks:video:save", save.rect.center());
                if save.clicked() {
                    state.message.clear();
                    match start(app, ui.ctx(), json!({"path":state.path,"fps":state.fps,"width":state.width,"height":state.height})) {
                        Ok(job) => state.job = Some(job),
                        Err(e) => state.message = e,
                    }
                }
            });
            if state.job.is_some() {
                ui.spinner();
                ui.label(app.tr("ui.mks_video.running"));
            }
            if !state.message.is_empty() {
                ui.add(egui::Label::new(&state.message).wrap());
            }
        });
        if closed {
            state.open = false;
        }
    }
    ui.ctx().data_mut(|d| d.insert_temp(key, state));
}
type Job = Arc<Mutex<Option<Result<Value, String>>>>;
fn start(app: &SolveApp, ctx: &egui::Context, params: Value) -> Result<Job, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut scratch = app.session.scratch();
        let job: Job = Arc::new(Mutex::new(None));
        let worker = job.clone();
        let ctx = ctx.clone();
        std::thread::Builder::new()
            .name("mks-video".into())
            .spawn(move || {
                let result = scratch.execute("mks.result.video", &params).map_err(|e| e.to_string());
                if let Ok(mut done) = worker.lock() {
                    *done = Some(result);
                }
                ctx.request_repaint();
            })
            .map_err(|e| e.to_string())?;
        Ok(job)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (app, ctx, params);
        Err("Video export is available on the desktop".into())
    }
}
