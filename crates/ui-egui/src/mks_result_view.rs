//! Read-only 3D result view using native display meshes and engine-provided poses.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::json;
use solidcraft_engine::{
    geom::Aabb3,
    render::{Camera, Rgb, Scene},
};
use std::sync::Arc;

fn fit_preview(camera: &mut Camera, bounds: &Aabb3, width: f32, height: f32) {
    camera.fit(bounds);
    // Camera::fit frames the vertical extent. A narrow simulation panel must
    // leave the same margin horizontally, while retaining the shared camera.
    camera.distance *= f64::from((height.max(1.) / width.max(1.)).max(1.));
}

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

/// Embedded playback for the MKS graph; explicit playback never opens a second window.
fn inline_cpu(app: &mut SolveApp, ui: &mut egui::Ui, sample: usize) {
    type Image = (u64, usize, Camera, egui::TextureHandle, Aabb3);
    let key = egui::Id::new(("mks_inline", app.docs.active, app.session.path.clone()));
    let mut cached = ui.ctx().data(|d| d.get_temp::<Image>(key));
    let width = ui.available_width().max(120.);
    let height = ui.available_height().max(160.);
    let scale = (1000. / width).min(800. / height).min(1.5);
    let pixels = [(width * scale).round().max(1.) as usize, (height * scale).round().max(1.) as usize];
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click_and_drag());
    crate::navigation::handle(app, ui, rect, &response, crate::navigation::NavCaps { orbit: true, pick: false });
    if response.double_clicked_by(egui::PointerButton::Middle)
        && let Some((_, _, _, _, bounds)) = &cached
    {
        fit_preview(&mut app.cam, bounds, width, height);
    }
    if !cached.as_ref().is_some_and(|v| v.0 == app.session.revision && v.1 == sample && v.2 == app.cam && v.3.size() == pixels) {
        match scene(app, sample) {
            Ok((scene, bounds)) => {
                if cached.is_none() {
                    fit_preview(&mut app.cam, &bounds, width, height);
                }
                let image = solidcraft_engine::render::render(&scene, &app.cam, pixels[0], pixels[1]);
                let pixels = egui::ColorImage::from_rgba_unmultiplied([image.w, image.h], &image.rgba);
                let texture = ui.ctx().load_texture("mks-inline", pixels, egui::TextureOptions::LINEAR);
                cached = Some((app.session.revision, sample, app.cam, texture, bounds));
                if let Some(v) = &cached {
                    ui.ctx().data_mut(|d| d.insert_temp(key, v.clone()));
                }
            }
            Err(e) => {
                ui.colored_label(Tokens::get().warning, e);
                return;
            }
        }
    }
    if let Some((_, _, _, texture, _)) = cached {
        ui.painter().image(texture.id(), rect, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)), egui::Color32::WHITE);
        scenario::publish_handle("mks:inline:canvas", rect.center());
        scenario::publish_handle(&format!("mks:inline:frame:{sample}"), rect.center());
    }
}

#[derive(Clone)]
struct LiveReplay {
    revision: u64,
    data: Arc<solidcraft_engine::cmd::AssemblyReplay>,
    geometry: Arc<crate::gpu::GpuScene>,
    colors: Arc<Vec<crate::gpu::PaletteIndex>>,
    slot: crate::gpu::SceneSlot,
    empty: crate::gpu::SceneSlot,
    generation: u64,
    tick: u64,
    last_time: f64,
    frames: u64,
}
/// The CAD GPU pipeline keeps each mesh uploaded. A playback tick uploads only
/// rigid placements and the few interface axes, never re-tessellates the CAD.
pub fn inline(app: &mut SolveApp, ui: &mut egui::Ui, sample: usize, time: f64) -> bool {
    if app.viewport.gpu.is_none() {
        inline_cpu(app, ui, sample);
        return false;
    }
    let key = egui::Id::new(("mks_live_replay", app.docs.active, app.session.path.clone()));
    let mut cache = ui.ctx().data(|d| d.get_temp::<LiveReplay>(key));
    let fresh = !cache.as_ref().is_some_and(|c| c.revision == app.session.revision);
    if fresh {
        match solidcraft_engine::cmd::prepare_assembly_replay(&mut app.session) {
            Ok(data) => {
                let generation = egui::Id::new((key, ui.ctx().cumulative_frame_nr(), app.session.revision)).value();
                let mut geometry = crate::gpu::GpuScene::default();
                let mut colors = Vec::new();
                for (i, mesh) in data.meshes.iter().enumerate() {
                    colors.push(geometry.color([mesh.color.0, mesh.color.1, mesh.color.2, 255]));
                    geometry.instanced.push(crate::gpu::InstancedMesh {
                        key: egui::Id::new((generation, i)).value(),
                        tris: Arc::new(crate::gpu::SharedTris::new(&mesh.mesh)),
                        instances: Vec::new(),
                        look: crate::gpu::InstanceLook::Opaque,
                        levels: Vec::new(),
                    });
                }
                if cache.is_none() {
                    fit_preview(&mut app.cam, &data.bounds, ui.available_width().max(120.), ui.available_height().max(160.));
                }
                cache = Some(LiveReplay {
                    revision: app.session.revision,
                    data: Arc::new(data),
                    geometry: Arc::new(geometry),
                    colors: Arc::new(colors),
                    slot: Arc::new(std::sync::Mutex::new(None)),
                    empty: Arc::new(std::sync::Mutex::new(Some(crate::gpu::GpuScene::default()))),
                    generation,
                    tick: 0,
                    last_time: f64::NEG_INFINITY,
                    frames: 0,
                });
            }
            Err(e) => {
                ui.colored_label(Tokens::get().warning, e.to_string());
                return false;
            }
        }
    }
    let Some(mut cache) = cache else { return false };
    if time != cache.last_time {
        let mut geometry = (*cache.geometry).clone();
        for ((mesh, world), color) in geometry.instanced.iter_mut().zip(cache.data.worlds(time)).zip(cache.colors.iter()) {
            mesh.place(std::array::from_fn(|r| std::array::from_fn(|c| world[c][r] as f32)), *color);
        }
        let deltas = cache.data.deltas(time);
        let length = (cache.data.bounds.diagonal() * 0.025).max(5.);
        for port in &cache.data.ports {
            if let Some(delta) = deltas.get(port.block) {
                let origin = solidcraft_engine::doc::apply_point(delta, port.origin);
                let color = geometry.color(Tokens::get().accent.to_array());
                for axis in port.axes {
                    let end = origin + solidcraft_engine::doc::apply_vector(delta, axis) * length;
                    geometry.line(origin.to_f32(), end.to_f32(), color, 1.5, true);
                }
            }
        }
        if let Ok(mut slot) = cache.slot.lock() {
            *slot = Some(geometry);
        }
        cache.last_time = time;
        cache.tick = cache.tick.wrapping_add(1);
        cache.frames += 1;
    }
    let size = egui::vec2(ui.available_width().max(120.), ui.available_height().max(160.));
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    crate::navigation::handle(app, ui, rect, &response, crate::navigation::NavCaps { orbit: true, pick: false });
    if response.double_clicked_by(egui::PointerButton::Middle) {
        fit_preview(&mut app.cam, &cache.data.bounds, rect.width(), rect.height());
    }
    ui.painter().rect_filled(rect, 0., Tokens::get().field);
    let camera = app.cam;
    let radius = cache.data.bounds.diagonal().max(10.) + cache.data.bounds.center().dist(camera.target);
    let eye = camera.eye();
    let ppp = ui.ctx().pixels_per_point();
    let empty_key = cache.generation.wrapping_add(1);
    let callback = crate::gpu::ViewportCallback(crate::gpu::Frame {
        product_lights: None,
        product: true,
        upload_error: None,
        key: egui::Id::new((cache.generation, cache.tick)).value(),
        slot: cache.slot.clone(),
        sketch_key: empty_key,
        sketch_slot: cache.empty.clone(),
        hl_key: empty_key,
        hl_slot: cache.empty.clone(),
        pv_key: empty_key,
        pv_slot: cache.empty.clone(),
        pv_hide: false,
        view_proj: camera.view_proj(f64::from(rect.width() / rect.height().max(1.)), radius).to_f32(),
        back: camera.back().to_f32(),
        size_px: [rect.width() * ppp, rect.height() * ppp],
        clip: [0.; 4],
        cap: [0.; 4],
        images: Vec::new(),
        points: Vec::new(),
        analysis: [0.; 8],
        analysis_colors: [0.; 36],
        access: None,
        outline: true,
        eye: [eye.x as f32, eye.y as f32, eye.z as f32, if camera.fov > 0. { 1. } else { 0. }],
        ao_radius: 0.,
        ao_samples: solidcraft_gpu::view::AO_SAMPLES,
        ssaa: 1.,
        hover_tint: [0.; 3],
        selection_tint: [0.; 3],
    });
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(rect, callback));
    scenario::publish_handle("mks:inline:canvas", rect.center());
    scenario::publish_handle(&format!("mks:inline:frame:{sample}"), rect.center());
    scenario::publish_handle(&format!("mks:inline:time:{}", (time * 1000.).round() as u64), rect.center());
    scenario::publish_handle(&format!("mks:inline:render:{}", cache.frames), rect.center());
    ui.ctx().data_mut(|d| d.insert_temp(key, cache));
    fresh
}
