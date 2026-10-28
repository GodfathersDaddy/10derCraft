//! Read-only 3D result view using native display meshes and engine-provided poses.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::json;
use solidcraft_engine::{
    geom::Aabb3,
    render::{Camera, Rgb, Scene},
};
use std::sync::Arc;

#[derive(Clone)]
struct Cached {
    revision: u64,
    sample: usize,
    scene: Arc<Scene>,
    bounds: Aabb3,
    texture: Option<egui::TextureHandle>,
    camera: Option<Camera>,
    error: String,
    export_path: String,
    message: String,
}
fn scene(app: &mut SolveApp, sample: usize) -> Result<(Scene, Aabb3), String> {
    let t = Tokens::get();
    let palette = solidcraft_engine::view::Palette {
        background: (Rgb(t.field.r(), t.field.g(), t.field.b()), Rgb(t.field.r(), t.field.g(), t.field.b())),
        body: Rgb(t.body.r(), t.body.g(), t.body.b()),
        edge: Rgb(t.body_edge.r(), t.body_edge.g(), t.body_edge.b()),
        axes: [Rgb(t.accent.r(), t.accent.g(), t.accent.b()); 3],
        ..solidcraft_engine::view::Palette::DEFAULT
    };
    solidcraft_engine::cmd::assembly_result_scene(&mut app.session, sample, palette).map_err(|e| e.to_string())
}
pub fn show(app: &mut SolveApp, ctx: &egui::Context, sample: usize, open: &mut bool) {
    let key = egui::Id::new(("mks_3d_result", app.docs.active, app.session.path.clone()));
    let old = ctx.data_mut(|d| d.get_temp::<Cached>(key));
    let export_path = old.as_ref().map(|c| c.export_path.clone()).unwrap_or_else(|| {
        app.session
            .path
            .as_ref()
            .map(|p| std::path::Path::new(p).with_extension("").to_string_lossy().to_string() + "-Ergebnis")
            .unwrap_or_else(|| "Dynamik-Ergebnis".into())
    });
    let message = old.as_ref().map(|c| c.message.clone()).unwrap_or_default();
    let mut cache = match old {
        Some(v) if v.revision == app.session.revision && v.sample == sample => v,
        old => match scene(app, sample) {
            Ok((scene, bounds)) => {
                if old.is_none() {
                    app.cam.fit(&bounds);
                }
                Cached {
                    revision: app.session.revision,
                    sample,
                    scene: Arc::new(scene),
                    bounds,
                    texture: old.and_then(|v| v.texture),
                    camera: None,
                    error: String::new(),
                    export_path: export_path.clone(),
                    message: message.clone(),
                }
            }
            Err(e) => Cached {
                revision: app.session.revision,
                sample,
                scene: Arc::new(Scene::default()),
                bounds: Aabb3::EMPTY,
                texture: None,
                camera: None,
                error: e,
                export_path,
                message,
            },
        },
    };
    let closed = crate::ui_kit::Window::new(key.with("window"), "graph", "3D-Dynamikergebnis")
        .cards(2.5)
        .placed(egui::Align2::LEFT_TOP, ctx.content_rect().min + egui::vec2(30., 170.))
        .show_body(ctx, |ui| {
            ui.label("Berechnete CAD-Körper und Kraftschnittstellen · reine Ergebnisansicht");
            ui.small(format!("Ausgabepunkt {sample}. Navigation wie im CAD; die Konstruktion und ihre Gelenke bleiben unverändert."));
            if !cache.error.is_empty() {
                ui.colored_label(Tokens::get().warning, &cache.error);
                return;
            }
            ui.horizontal(|ui| {
                let fit = ui.button("Ergebnis einpassen");
                scenario::publish_handle("mks:3d:fit", fit.rect.center());
                if fit.clicked() {
                    app.cam.fit(&cache.bounds);
                }
                if ui.button("Isometrisch").clicked() {
                    app.cam.set_view(solidcraft_engine::render::StandardView::Iso);
                    app.cam.fit(&cache.bounds);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Exportpfad");
                let field = ui.text_edit_singleline(&mut cache.export_path);
                scenario::publish_handle("mks:3d:export:path", field.rect.center());
                for (extension, label, command) in
                    [("png", "PNG exportieren", "mks.result.snapshot"), ("gif", "GIF exportieren", "mks.result.animation")]
                {
                    let b = ui.button(label);
                    scenario::publish_handle(&format!("mks:3d:export:{extension}"), b.rect.center());
                    if b.clicked() {
                        let mut path = std::path::PathBuf::from(&cache.export_path);
                        if path.extension().is_some_and(|v| v == "png" || v == "gif") {
                            path.set_extension(extension);
                        } else {
                            path = std::path::PathBuf::from(cache.export_path.clone() + "." + extension);
                        }
                        match app.run(command, json!({"sample":sample,"path":path.to_string_lossy(),"camera":app.cam})) {
                            Ok(_) => cache.message = format!("Exportiert: {}", path.display()),
                            Err(e) => cache.message = e,
                        }
                    }
                }
            });
            if !cache.message.is_empty() {
                ui.label(&cache.message);
            }
            let size = egui::vec2(ui.available_width().clamp(240., 640.), 320.);
            let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
            crate::navigation::handle(app, ui, rect, &response, crate::navigation::NavCaps::RENDER);
            if response.double_clicked_by(egui::PointerButton::Middle) {
                app.cam.fit(&cache.bounds);
            }
            if cache.camera != Some(app.cam) || cache.texture.is_none() {
                let image = solidcraft_engine::render::render(&cache.scene, &app.cam, 640, 320);
                let pixels = egui::ColorImage::from_rgba_unmultiplied([image.w, image.h], &image.rgba);
                if let Some(texture) = &mut cache.texture {
                    texture.set(pixels, egui::TextureOptions::LINEAR);
                } else {
                    cache.texture = Some(ctx.load_texture("mks_3d_result", pixels, egui::TextureOptions::LINEAR));
                }
                cache.camera = Some(app.cam);
            }
            if let Some(texture) = &cache.texture {
                ui.painter().image(texture.id(), rect, egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)), egui::Color32::WHITE);
            }
            scenario::publish_handle("mks:3d:canvas", rect.center());
            scenario::publish_handle(&format!("mks:3d:frame:{sample}"), rect.center());
            ui.small(
                "Portachsen sind hervorgehoben. Die Anzeige verwendet CAD-Darstellungsnetze; keine neue oder veränderte Konstruktionsgeometrie.",
            );
        });
    if closed {
        *open = false;
    }
    ctx.data_mut(|d| d.insert_temp(key, cache));
}
