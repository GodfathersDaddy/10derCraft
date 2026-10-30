//! Native assembly block view. All reduction remains in the registered engine command.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::{Value, json};

pub fn show(app: &mut SolveApp, ui: &mut egui::Ui) {
    let key = egui::Id::new(("native_mks_model", app.docs.active, app.session.path.clone()));
    let cached = ui.data_mut(|d| d.get_temp::<(u64, Value)>(key));
    let model = if let Some((_, v)) = cached.filter(|(r, _)| *r == app.session.revision) {
        v
    } else {
        let v = app.session.execute("mks.model", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}));
        ui.data_mut(|d| d.insert_temp(key, (app.session.revision, v.clone())));
        v
    };
    ui.heading("MKS · Baugruppenmodell");
    ui.label("Starre Teile bilden einen Block. Bewegliche Gelenke und Kraftschnittstellen stammen aus dem aktuellen CAD-Projekt.");
    ui.small("Die Blockstruktur folgt dem CAD-Stand. Dynamikläufe werden separat berechnet und mit ihrem Eingabestand gespeichert.");
    if let Some(error) = model["error"].as_str() {
        ui.colored_label(Tokens::get().warning, error);
        return;
    }
    let blocks = model["blocks"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let joints = model["joints"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    ui.label(format!(
        "{} starre Blöcke · {} bewegliche Gelenke / Relationen · {} unterdrückte Gelenke",
        blocks.len(),
        joints.len(),
        model["suppressed_joints"].as_array().map_or(0, Vec::len)
    ));
    scenario::publish_handle("mks:model", ui.min_rect().center());
    physical_properties(app, ui, key.with("mass"));
    ui.collapsing(app.tr("ui.analysis.dynamics"), |ui| crate::mks_dynamics_ui::show(app, ui, &model));
    crate::mks_graph_ui::show(app, ui, &model);
    egui::ScrollArea::vertical().id_salt("native_mks_details").max_height(230.).show(ui, |ui| {
        for issue in model["issues"].as_array().into_iter().flatten() {
            ui.colored_label(Tokens::get().warning, format!("{}: {}", issue["kind"].as_str().unwrap_or(""), issue["message"].as_str().unwrap_or("")));
        }
        for p in model["unbound_ports"].as_array().into_iter().flatten() {
            ui.colored_label(Tokens::get().warning, format!("Schnittstelle {} ungültig: {}", p["port"]["id"], p["error"]));
        }
        ui.collapsing("Blöcke, Einbaupfade und Kraftschnittstellen", |ui| {
            for b in blocks {
                ui.strong(b["id"].as_str().unwrap_or(""));
                for m in b["members"].as_array().into_iter().flatten() {
                    ui.label(format!("{} · Pfad {}", m["name"].as_str().unwrap_or(""), m["path"]));
                }
                for p in b["ports"].as_array().into_iter().flatten() {
                    ui.label(format!("Port {} · {} / {}", p["port"]["id"], p["port"]["support"], p["port"]["transfer"]));
                }
            }
        });
        ui.collapsing("Gelenke und lokale Freiheitsgrade", |ui| {
            for j in joints {
                ui.label(format!(
                    "{}: {} ↔ {} · {} · {}{}",
                    j["name"].as_str().unwrap_or(""),
                    j["a"],
                    j["b"],
                    j["kind"],
                    j["local_dofs"],
                    if j["closes_graph_cycle"] == true { " · geschlossene Kette" } else { "" }
                ));
            }
            ui.small(
                "Lokale Gelenkfreiheiten ergeben durch Addition keine globale Beweglichkeit. Geschlossene Ketten benötigen eine gesonderte Prüfung.",
            );
        });
    });
    ui.collapsing("Kraftschnittstellen und statische Lastfälle bearbeiten", |ui| crate::mechanics_ui::show(app, ui, false));
}

fn physical_properties(app: &mut SolveApp, ui: &mut egui::Ui, key: egui::Id) {
    let r = ui.button("Masse und Trägheit aus CAD berechnen");
    scenario::publish_handle("mks:mass:calculate", r.rect.center());
    if r.clicked() {
        let result = app.session.execute("mks.mass_properties", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}));
        ui.data_mut(|d| d.insert_temp(key, (app.session.revision, result)));
    }
    let report = ui.data_mut(|d| d.get_temp::<(u64, Value)>(key));
    if let Some((revision, report)) = report {
        if revision != app.session.revision {
            let r =
                ui.colored_label(Tokens::get().warning, "Masseneigenschaften veraltet – nach CAD-, Material- oder Positionsänderung neu berechnen.");
            scenario::publish_handle("mks:mass:stale", r.rect.center());
            return;
        }
        if let Some(error) = report["error"].as_str() {
            ui.colored_label(Tokens::get().warning, error);
            return;
        }
        scenario::publish_handle("mks:mass:current", ui.min_rect().center());
        let details =
            ui.collapsing("Physikalische Eigenschaften · aktueller CAD-Stand", |ui| {
                egui::ScrollArea::vertical().id_salt("mks_mass_details").max_height(190.).show(ui,|ui|{
            for block in report["blocks"].as_array().into_iter().flatten() {
                let id = block["id"].as_str().unwrap_or("");
                if block["complete"] == false {
                    ui.colored_label(Tokens::get().warning, format!("{id}: unvollständig; keine Blockmasse freigegeben"));
                    for error in block["errors"].as_array().into_iter().flatten() {
                        ui.label(format!("{}: {}", error["source"], error["error"]));
                    }
                    continue;
                }
                if block["status"] == "empty" {
                    ui.label(format!("{id}: ohne Volumenkörper"));
                    continue;
                }
                ui.strong(format!("{id} · {:.6} kg", block["mass_kg"].as_f64().unwrap_or(0.)));
                    let coordinate=|i|block["com_world_m"].get(i).and_then(Value::as_f64).unwrap_or(0.);
                    ui.label(format!("Schwerpunkt Welt [m]: [{:.6}, {:.6}, {:.6}]",coordinate(0),coordinate(1),coordinate(2)));
                let assumed = block["bodies"].as_array().into_iter().flatten().filter(|b| b["density_assumed"] == true).count();
                if assumed > 0 {
                    ui.colored_label(Tokens::get().warning, format!("{assumed} Körper verwenden angenommene CAD-Standarddichte; Material zuweisen."));
                }
                    let tensor=ui.collapsing(format!("{id}: Trägheitstensor [kg·m²]"), |ui| {
                    ui.label("Um den Blockschwerpunkt, Achsen parallel zum Weltkoordinatensystem.");
                    for row in block["inertia_com_world_kg_m2"].as_array().into_iter().flatten() {
                            let cell=|i|row.get(i).and_then(Value::as_f64).unwrap_or(0.);
                            ui.monospace(format!("[{:12.5e}  {:12.5e}  {:12.5e}]",cell(0),cell(1),cell(2)));
                    }
                });
                    scenario::publish_handle(&format!("mks:mass:tensor:{id}"),tensor.header_response.rect.center());
            }
            ui.small("Numerische Netzintegration mit zwei Auflösungen. Körper werden addiert; Überschneidungen werden nicht automatisch abgezogen.");
            });
            });
        scenario::publish_handle("mks:mass:details", details.header_response.rect.center());
    }
}

#[derive(Clone)]
struct BlockPicture {
    revision: u64,
    image: Option<egui::ColorImage>,
    texture: Option<egui::TextureHandle>,
}

/// Native CAD geometry, exact placement path from mks.model; bounded background rendering.
pub(super) fn block_picture(app: &SolveApp, ui: &mut egui::Ui, block: &Value, rect: egui::Rect) {
    if cfg!(target_arch = "wasm32") {
        return;
    }
    use solidcraft_engine::{
        geom::Aabb3,
        render::{Camera, Scene, SceneMesh, StandardView},
    };
    use std::sync::Arc;
    let id = block["id"].as_str().unwrap_or("");
    let key = egui::Id::new(("mks_block_picture", app.docs.active, app.session.path.clone(), id));
    let busy = egui::Id::new("mks_block_picture_worker");
    let revision = app.session.revision;
    let old = ui.ctx().data(|d| d.get_temp::<BlockPicture>(key));
    if let Some(mut cached) = old.filter(|c| c.revision == revision) {
        if cached.texture.is_none()
            && let Some(image) = cached.image.take()
        {
            cached.texture = Some(ui.ctx().load_texture(format!("mks-{id}"), image, egui::TextureOptions::LINEAR));
            ui.ctx().data_mut(|d| d.insert_temp(key, cached.clone()));
        }
        if let Some(texture) = cached.texture {
            ui.painter().image(texture.id(), rect, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)), egui::Color32::WHITE);
            scenario::publish_handle(&format!("mks:block:picture:{id}"), rect.center());
        }
        return;
    }
    if ui.ctx().data(|d| d.get_temp::<bool>(busy)).unwrap_or(false) {
        return;
    }
    // Oversized assemblies keep the named card; never tessellate an unbounded picture.
    if block["bodies"].as_array().is_some_and(|b| b.len() > 100) {
        return;
    }
    let state = app.session.model.state();
    let mut scene = Scene::default();
    let mut bounds = Aabb3::EMPTY;
    let mut triangles = 0;
    for placed in block["bodies"].as_array().into_iter().flatten() {
        let Some(body) = state.bodies.iter().find(|b| {
            Some(b.name.as_str()) == placed["source"].as_str() && Some(app.session.doc.model_body_component(b)) == placed["component"].as_u64()
        }) else {
            continue;
        };
        let Ok(world) = serde_json::from_value::<solidcraft_engine::doc::Mat>(placed["world"].clone()) else { continue };
        let source = body.mesh();
        triangles += source.triangles.len();
        if triangles > 100_000 || source.positions.len() > 500_000 {
            return;
        }
        let mut mesh = (*source).clone();
        for p in &mut mesh.positions {
            *p = solidcraft_engine::doc::apply_point(&world, *p);
            bounds.add(*p);
        }
        for n in &mut mesh.normals {
            *n = solidcraft_engine::doc::apply_vector(&world, *n);
        }
        // The miniature contains surfaces only; no untransformed source edges.
        mesh.edges.clear();
        let color = app
            .session
            .doc
            .body_color(body)
            .map(|c| solidcraft_engine::render::Rgb((c[0] * 255.).round() as u8, (c[1] * 255.).round() as u8, (c[2] * 255.).round() as u8))
            .unwrap_or(solidcraft_engine::view::colors::BODY);
        scene.meshes.push(SceneMesh::solid(Arc::new(mesh), color));
    }
    ui.ctx().data_mut(|d| d.insert_temp(key, BlockPicture { revision, image: None, texture: None }));
    if bounds.is_empty() {
        return;
    }
    let mut camera = Camera::default();
    camera.set_view(StandardView::Iso);
    camera.fit(&bounds);
    scene.radius = bounds.diagonal().max(1.);
    let ctx = ui.ctx().clone();
    ctx.data_mut(|d| d.insert_temp(busy, true));
    let worker_ctx = ctx.clone();
    let spawn = std::thread::Builder::new().name("mks-block-picture".into()).spawn(move || {
        let picture = solidcraft_engine::render::render(&scene, &camera, 254, 102);
        let image = egui::ColorImage::from_rgba_unmultiplied([picture.w, picture.h], &picture.rgba);
        worker_ctx.data_mut(|d| {
            d.insert_temp(key, BlockPicture { revision, image: Some(image), texture: None });
            d.insert_temp(busy, false);
        });
        worker_ctx.request_repaint();
    });
    if spawn.is_err() {
        ctx.data_mut(|d| d.insert_temp(busy, false));
    }
}
