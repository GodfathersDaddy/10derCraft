//! Thin controls for the native shared robot workflow; transient tool windows only.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::{Value, json};
#[derive(Clone)]
struct Cached {
    revision: u64,
    value: Value,
}
fn enabled(app: &SolveApp) -> bool {
    app.session.doc.metadata.get("org.solidcraft.robot-workflow").is_some()
}
pub fn motion(app: &mut SolveApp, ui: &mut egui::Ui) {
    if !enabled(app) {
        return;
    }
    ui.label("Roboterarm · dieselbe Berechnung, CAD-Baugruppe und Kraftschnittstellen");
    let b = ui.button("Sechsachsige Bewegung berechnen");
    scenario::publish_handle("robot:motion:run", b.rect.center());
    if b.clicked() {
        let _ = app.run("robot.motion.run", json!({}));
    }
    ui.small("Ideale Antriebe aus ref_Hub_A1…6; native Achsgrenzen werden geprüft. Danach 3D-Ergebnisansicht öffnen.");
    ui.separator();
}
pub fn fem(app: &mut SolveApp, ui: &mut egui::Ui) -> bool {
    if !enabled(app) {
        return false;
    }
    egui::ScrollArea::vertical().id_salt("robot_fem_scroll").max_height((ui.ctx().content_rect().height()-180.).max(300.)).show(ui,|ui|{
        ui.heading("Roboterarm – Strukturrechnung");
        ui.label("Oberarm und Unterarm aus nativen CAD-Zylindern. Aktuatoren gesperrt; übrige Module starr.");
        ui.small("Balken-FEM mit Hermite-Elementen; axiale und Torsionsnachgiebigkeit. Last: F_TCP_N entlang TCP −Z. Eigengewicht und dynamische Spannungen sind hier noch nicht enthalten.");
        let key=ui.id().with("elements");let mut n=ui.ctx().data(|d|d.get_temp::<u32>(key)).unwrap_or(8);
        ui.horizontal(|ui|{ui.label("Elemente pro Armglied");let r=ui.add(egui::DragValue::new(&mut n).range(1..=32));scenario::publish_handle("robot:fem:elements",r.rect.center());
            let b=ui.button("Struktur berechnen");scenario::publish_handle("robot:fem:run",b.rect.center());if b.clicked(){let _=app.run("robot.fem.solve",json!({"elements":n}));}
        });ui.ctx().data_mut(|d|d.insert_temp(key,n));
        let cache_key=egui::Id::new(("robot_fem_state",app.docs.active,app.session.path.clone()));
        let cached=ui.ctx().data(|d|d.get_temp::<Cached>(cache_key));
        let state=match cached{Some(c) if c.revision==app.session.revision=>c.value,_=>{
            let value=app.session.execute("robot.inspect",&json!({})).unwrap_or_else(|e|json!({"error":e.to_string()}));ui.ctx().data_mut(|d|d.insert_temp(cache_key,Cached{revision:app.session.revision,value:value.clone()}));value
        }};
        if let Some(e)=state["error"].as_str(){ui.colored_label(Tokens::get().warning,e);return;}
        if !state["fem"].is_object(){ui.label("Noch kein Strukturergebnis. Berechnung unter Parameter bearbeiten und Referenzen übernehmen.");return;}
        let current=state["fem_current"]==true;
        let r=ui.colored_label(if current{Tokens::get().text}else{Tokens::get().warning},if current{"Ergebnis aktuell"}else{"Ergebnis historisch – Eingaben, CAD oder Gelenkstellung geändert. Neu berechnen."});
        scenario::publish_handle(if current{"robot:fem:current"}else{"robot:fem:stale"},r.rect.center());
        let report=&state["fem"];
        ui.label(format!("TCP-Verschiebung: {:.6} mm · Grenze {:.3} mm · {}",report["tcp_deflection_mm"].as_f64().unwrap_or(0.),report["limit_mm"].as_f64().unwrap_or(0.),if report["feasible"]==true{"eingehalten"}else{"überschritten"}));
        egui::Grid::new("robot_fem_results").min_col_width(110.).striped(true).show(ui,|ui|{
            ui.label("CAD-Körper");ui.label("Radius [mm]");ui.label("Länge [mm]");ui.label("Normalsp. [MPa]").on_hover_text("Maximale nominelle Normalspannung des Balkenmodells");ui.end_row();
            if let Some(beams)=report["beams"].as_array(){for b in beams{ui.label(b["body"].as_str().unwrap_or(""));for field in ["radius_mm","length_mm","max_normal_stress_mpa"]{ui.label(format!("{:.4}",b[field].as_f64().unwrap_or(0.)));}ui.end_row();}}
        });
        ui.collapsing("Diskretisierung und Lastannahmen",|ui|{ui.monospace(serde_json::to_string_pretty(&report["load_case"]).unwrap_or_default());if let Some(warnings)=report["warnings"].as_array(){for w in warnings{ui.label(w.as_str().unwrap_or(""));}}});
        ui.small("Geometrie und MKS-Ergebnis bleiben unverändert. Das Ergebnis mit Eingabestand wird im Projekt gespeichert. Keine Volumen-FEM oder flexible Rückkopplung in die Bewegung.");
    });
    true
}

#[derive(Clone)]
struct DeformationScene {
    grid: bool,
    revision: u64,
    scale: f64,
    scene: std::sync::Arc<solidcraft_engine::render::Scene>,
    bounds: solidcraft_engine::geom::Aabb3,
    maximum: f64,
}
#[derive(Clone)]
struct DeformationImage {
    grid: bool,
    size: [usize; 2],
    revision: u64,
    scale: f64,
    camera: solidcraft_engine::render::Camera,
    image: Option<egui::ColorImage>,
    texture: Option<egui::TextureHandle>,
}

pub fn deformation_workspace(app: &mut SolveApp, ui: &mut egui::Ui) {
    if cfg!(target_arch = "wasm32") {
        ui.label(app.tr("ui.analysis.desktop_preview"));
        return;
    }
    crate::ui_kit::FileView::page(ui, |ui| {
        ui.heading(app.tr("ui.analysis.deformation"));
        ui.label(app.tr("ui.analysis.beam_caption"));
        let key = egui::Id::new(("fem_deformation", app.docs.active, app.session.path.clone()));
        let mut scale = ui.ctx().data(|d| d.get_temp::<f64>(key.with("scale"))).unwrap_or(1000.);
        ui.horizontal(|ui| {
            ui.label(app.tr("ui.analysis.scale"));
            let control = ui.add(egui::DragValue::new(&mut scale).range(0. ..=10000.).speed(10.));
            scenario::publish_handle("fem:deformation:scale", control.rect.center());
        });
        ui.ctx().data_mut(|d| d.insert_temp(key.with("scale"), scale));
        let mut grid = ui.ctx().data(|d| d.get_temp::<bool>(key.with("grid"))).unwrap_or(false);
        let toggle = ui.checkbox(&mut grid, app.tr("ui.analysis.display_grid"));
        scenario::publish_handle("fem:deformation:grid", toggle.rect.center());
        ui.ctx().data_mut(|d| d.insert_temp(key.with("grid"), grid));
        ui.small(app.tr("ui.analysis.grid_note"));
        let cached = ui.ctx().data(|d| d.get_temp::<DeformationScene>(key));
        let scene = match cached {
            Some(v) if v.revision == app.session.revision && v.scale == scale && v.grid == grid => v,
            previous => {
                let palette = solidcraft_engine::view::Palette::DEFAULT;
                match solidcraft_engine::cmd::robot_fem_result_scene(&mut app.session, scale, palette) {
                    Ok((mut scene, bounds, maximum)) => {
                        if grid {
                            let mut lines = Vec::new();
                            for surface in scene.meshes.iter().filter(|m| !m.face_colors.is_empty()) {
                                let mut edges = std::collections::BTreeSet::new();
                                for t in &surface.mesh.triangles {
                                    for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                                        edges.insert((a.min(b), a.max(b)));
                                    }
                                }
                                for (a, b) in edges {
                                    if let (Some(a), Some(b)) = (surface.mesh.positions.get(a as usize), surface.mesh.positions.get(b as usize)) {
                                        lines.push(solidcraft_engine::render::SceneLine {
                                            points: vec![*a, *b],
                                            color: solidcraft_engine::view::colors::EDGE,
                                            alpha: 1.,
                                            width: 0.7,
                                            on_top: false,
                                        });
                                    }
                                }
                            }
                            scene.lines.extend(lines);
                        }
                        if previous.is_none() {
                            app.cam.set_view(solidcraft_engine::render::StandardView::Iso);
                            app.cam.fit(&bounds);
                        }
                        let cache =
                            DeformationScene { grid, revision: app.session.revision, scale, scene: std::sync::Arc::new(scene), bounds, maximum };
                        ui.ctx().data_mut(|d| d.insert_temp(key, cache.clone()));
                        cache
                    }
                    Err(e) => {
                        ui.colored_label(Tokens::get().warning, e.to_string());
                        return;
                    }
                }
            }
        };
        let locale = crate::i18n::Locale::for_language(&app.preferences.language);
        ui.label(locale.format("ui.analysis.field_range", &[("maximum", &format!("{:.6}", scene.maximum))]));
        if let Some(field) = scene.scene.meshes.iter().find(|m| !m.face_colors.is_empty()) {
            let (bar, _) = ui.allocate_exact_size(egui::vec2(320., 12.), egui::Sense::hover());
            let count = field.face_colors.len().max(1);
            for (i, (_, color)) in field.face_colors.iter().enumerate() {
                let segment = egui::Rect::from_min_max(
                    egui::pos2(bar.left() + bar.width() * i as f32 / count as f32, bar.top()),
                    egui::pos2(bar.left() + bar.width() * (i + 1) as f32 / count as f32, bar.bottom()),
                );
                ui.painter().rect_filled(segment, 0., egui::Color32::from_rgb(color.0, color.1, color.2));
            }
        }
        let fit = ui.button(app.tr("ui.analysis.fit"));
        if fit.clicked() {
            app.cam.fit(&scene.bounds);
        }
        scenario::publish_handle("fem:deformation:fit", fit.rect.center());
        ui.small(app.tr("ui.analysis.navigation_hint"));
        if let Ok(state) = app.session.execute("robot.inspect", &json!({})) {
            ui.label(locale.format(
                "ui.analysis.tcp_summary",
                &[("tcp", &format!("{:.6}", state["fem"]["tcp_deflection_mm"].as_f64().unwrap_or(0.))), ("scale", &format!("{scale:.1}"))],
            ));
        }
        let size = egui::vec2(ui.available_width().max(100.), ui.available_height().max(100.));
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        crate::navigation::handle(app, ui, rect, &response, crate::navigation::NavCaps { orbit: true, pick: false });
        let pixels = rect.size() * ui.ctx().pixels_per_point();
        let resolution_scale =
            (2048. / pixels.x.max(1.)).min(1400. / pixels.y.max(1.)).min((2_000_000. / (pixels.x * pixels.y).max(1.)).sqrt()).min(1.);
        let render_size = [(pixels.x * resolution_scale).round().max(64.) as usize, (pixels.y * resolution_scale).round().max(64.) as usize];
        let image_key = key.with("image");
        let busy_key = key.with("busy");
        let mut image = ui.ctx().data(|d| d.get_temp::<DeformationImage>(image_key));
        let current = image.as_ref().is_some_and(|v| {
            v.revision == app.session.revision && v.scale == scale && v.grid == grid && v.camera == app.cam && v.size == render_size
        });
        // Keep the last camera frame visible during navigation; never show another model revision.
        if let Some(cache) = &mut image
            && cache.revision == app.session.revision
            && cache.scale == scale
            && cache.grid == grid
        {
            if cache.texture.is_none()
                && let Some(pixels) = cache.image.take()
            {
                cache.texture = Some(ui.ctx().load_texture("fem-deformation", pixels, egui::TextureOptions::LINEAR));
                ui.ctx().data_mut(|d| d.insert_temp(image_key, cache.clone()));
            }
            if let Some(texture) = &cache.texture {
                let dimensions = egui::vec2(cache.size[0] as f32, cache.size[1] as f32);
                let fit_size = dimensions * (rect.width() / dimensions.x).min(rect.height() / dimensions.y);
                let target = egui::Rect::from_center_size(rect.center(), fit_size);
                ui.painter().image(texture.id(), target, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)), egui::Color32::WHITE);
                if current {
                    scenario::publish_handle("fem:deformation:ready", target.center());
                    scenario::publish_handle(&format!("fem:deformation:pixels:{}x{}", cache.size[0], cache.size[1]), target.center());
                }
            }
        }
        if !current && !ui.ctx().data(|d| d.get_temp::<bool>(busy_key)).unwrap_or(false) {
            let ctx = ui.ctx().clone();
            let worker = ctx.clone();
            let camera = app.cam;
            let revision = app.session.revision;
            ctx.data_mut(|d| d.insert_temp(busy_key, true));
            let spawn = std::thread::Builder::new().name("fem-deformation".into()).spawn(move || {
                let rgba = solidcraft_engine::capture::rgba(&scene.scene, &camera, render_size[0] as u32, render_size[1] as u32)
                    .map(|picture| picture.rgba)
                    .unwrap_or_else(|_| solidcraft_engine::render::render(&scene.scene, &camera, render_size[0], render_size[1]).rgba);
                let image = egui::ColorImage::from_rgba_unmultiplied(render_size, &rgba);
                worker.data_mut(|d| {
                    d.insert_temp(
                        image_key,
                        DeformationImage { grid, size: render_size, revision, scale, camera, image: Some(image), texture: None },
                    );
                    d.insert_temp(busy_key, false);
                });
                worker.request_repaint();
            });
            if spawn.is_err() {
                ctx.data_mut(|d| d.insert_temp(busy_key, false));
            }
        }
        scenario::publish_handle("fem:deformation:canvas", rect.center());
    });
}
