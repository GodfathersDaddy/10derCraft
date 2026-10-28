//! Thin controls for the native shared robot workflow; transient tool windows only.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::{Value, json};
#[derive(Clone)]
struct Cached{revision:u64,value:Value}
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
        egui::Grid::new("robot_fem_results").striped(true).show(ui,|ui|{
            ui.label("CAD-Körper");ui.label("Radius [mm]");ui.label("Länge [mm]");ui.label("Normalspannung [MPa]");ui.end_row();
            if let Some(beams)=report["beams"].as_array(){for b in beams{ui.label(b["body"].as_str().unwrap_or(""));for field in ["radius_mm","length_mm","max_normal_stress_mpa"]{ui.label(format!("{:.4}",b[field].as_f64().unwrap_or(0.)));}ui.end_row();}}
        });
        ui.collapsing("Diskretisierung und Lastannahmen",|ui|{ui.monospace(serde_json::to_string_pretty(&report["load_case"]).unwrap_or_default());if let Some(warnings)=report["warnings"].as_array(){for w in warnings{ui.label(w.as_str().unwrap_or(""));}}});
        ui.small("Geometrie und MKS-Ergebnis bleiben unverändert. Das Ergebnis mit Eingabestand wird im Projekt gespeichert. Keine Volumen-FEM oder flexible Rückkopplung in die Bewegung.");
    });
    true
}
