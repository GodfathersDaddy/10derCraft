//! Project mechanics uses the same registered commands as MCP; no solver in the UI.
use crate::{SolveApp, scenario, theme::Tokens};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone)]
struct Draft {
    port: String,
    body: String,
    occurrence: String,
    origin: [String; 3],
    quaternion: [f64; 4],
    transfer: usize,
    area: f64,
    parent: String,
    child: String,
    case: String,
    root: String,
    load_port: String,
    force: [f64; 3],
    moment: [f64; 3],
    pressure: f64,
    pressure_load: bool,
    error: String,
    report: Option<Value>,
}
impl Default for Draft {
    fn default() -> Self {
        Self {
            port: "tip".into(),
            body: String::new(),
            occurrence: String::new(),
            origin: ["0 mm".into(), "0 mm".into(), "0 mm".into()],
            quaternion: [1., 0., 0., 0.],
            transfer: 0,
            area: 100.,
            parent: String::new(),
            child: String::new(),
            case: "loadcase".into(),
            root: String::new(),
            load_port: String::new(),
            force: [0., 0., -100.],
            moment: [0.; 3],
            pressure: 1.,
            pressure_load: false,
            error: String::new(),
            report: None,
        }
    }
}
fn run(app: &mut SolveApp, d: &mut Draft, id: &str, p: Value) -> bool {
    match app.run(id, p) {
        Ok(v) => {
            d.error.clear();
            if id == "mechanics.solve" {
                d.report = Some(v);
            }
            true
        }
        Err(e) => {
            d.error = e;
            false
        }
    }
}
fn button(ui: &mut egui::Ui, id: &str, label: &str) -> bool {
    let r = ui.button(label);
    scenario::publish_handle(id, r.rect.center());
    r.clicked()
}
fn text(ui: &mut egui::Ui, id: &str, label: &str, value: &mut String) {
    ui.vertical(|ui| {
        ui.label(label);
        let r = ui.add(egui::TextEdit::singleline(value).desired_width(ui.available_width()));
        scenario::publish_handle(id, r.rect.center());
    });
}
fn load(d: &mut Draft, p: &Value) {
    d.port = p["id"].as_str().unwrap_or("").into();
    d.body = p["body"].as_str().unwrap_or("").into();
    d.occurrence = p["occurrence"].as_array().map(|a| a.iter().map(Value::to_string).collect::<Vec<_>>().join(",")).unwrap_or_default();
    for (i, v) in d.origin.iter_mut().enumerate() {
        *v = p["origin"].get(i).and_then(Value::as_str).unwrap_or("0 mm").into();
    }
    for (i, v) in d.quaternion.iter_mut().enumerate() {
        *v = p["quaternion"].get(i).and_then(Value::as_f64).unwrap_or(if i == 0 { 1. } else { 0. });
    }
    d.transfer = match (p["support"].as_str(), p["transfer"].as_str()) {
        (_, Some("point_force")) => 0,
        (Some("surface"), Some("pressure")) => 3,
        (Some("surface"), _) => 2,
        _ => 1,
    };
    d.area = p["area_mm2"].as_f64().unwrap_or(100.);
}
pub fn has_project(app: &SolveApp) -> bool {
    app.session.doc.metadata.get("org.solidcraft.mechanical-project").and_then(|v| v["ports"].as_object()).is_some_and(|p| !p.is_empty())
}
pub fn show(app: &mut SolveApp, ui: &mut egui::Ui, graph: bool) {
    let key = egui::Id::new(("project_mechanics_draft", app.docs.active, app.session.path.clone()));
    let mut d = ui.data_mut(|data| data.get_temp::<Draft>(key)).unwrap_or_default();
    if d.body.is_empty() {
        d.body = app.session.model.state().bodies.first().map(|b| b.name.clone()).unwrap_or_default();
    }
    let cache = key.with("state");
    let cached = ui.data_mut(|data| data.get_temp::<(u64, Value)>(cache));
    let state = if let Some((_, v)) = cached.filter(|(r, _)| *r == app.session.revision) {
        v
    } else {
        let v = app.session.execute("mechanics.state", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}));
        ui.data_mut(|data| data.insert_temp(cache, (app.session.revision, v.clone())));
        v
    };
    egui::ScrollArea::vertical().id_salt("mechanical_project_scroll").max_height(650.).show(ui,|ui| {
        ui.heading("Mechanische Projektschnittstellen");
        ui.label("Im Projekt gespeichert. Starre Lastpfade und statische Resultanten; keine dynamische MKS oder FEM-Kopplung.");
        if let Some(error)=state["error"].as_str(){ui.label(error);}
        let ports=state["network"]["ports"].as_object();
        if graph {diagram(ui,&state,&mut d);}
        if let Some(ports)=ports {ui.horizontal_wrapped(|ui|{for (id,p) in ports {if button(ui,&format!("mechanics:select:{id}"),id){load(&mut d,p);}}});}
        for p in state["resolved_ports"].as_array().into_iter().flatten().filter(|p|p["valid"]==false){ui.label(format!("{}: {}",p["id"],p["error"]));}
        ui.collapsing("Schnittstelle definieren / ändern",|ui|{
            text(ui,"mechanics:port:id","ID",&mut d.port);text(ui,"mechanics:port:body","CAD-Körper (Quellname)",&mut d.body);
            text(ui,"mechanics:port:occurrence","Occurrence-Pfad, z. B. 1,3 (leer bei Root-Körper)",&mut d.occurrence);
            for (i,label) in ["Position X [mm / Ausdruck]","Position Y [mm / Ausdruck]","Position Z [mm / Ausdruck]"].iter().enumerate(){if let Some(value)=d.origin.get_mut(i){text(ui,&format!("mechanics:origin:{i}"),label,value);}}
            egui::ComboBox::from_id_salt("mechanical_transfer").selected_text(["Punktkraft (3)","Rahmen: Kraft/Moment (6)","Fläche: Kraft/Moment (6)","Fläche: gleichmäßiger Normaldruck"].get(d.transfer).copied().unwrap_or("")).show_ui(ui,|ui|{
                for (i,label) in ["Punktkraft (3)","Rahmen: Kraft/Moment (6)","Fläche: Kraft/Moment (6)","Fläche: gleichmäßiger Normaldruck"].iter().enumerate(){ui.selectable_value(&mut d.transfer,i,*label);}
            });
            if d.transfer>=2{ui.horizontal(|ui|{ui.label("Deklarierte Fläche [mm²], nicht aus CAD gemessen");ui.add(egui::DragValue::new(&mut d.area));});}
            ui.horizontal_wrapped(|ui|{ui.label("Orientierung Quaternion w,x,y,z");for q in &mut d.quaternion {ui.add(egui::DragValue::new(q).speed(0.01));}});
            if button(ui,"mechanics:port:save","Schnittstelle speichern") {
                let path=d.occurrence.split(',').map(str::trim).filter(|s|!s.is_empty()).map(str::parse::<u64>).collect::<Result<Vec<_>,_>>();
                match path {Ok(path)=>{let mut p=json!({"id":d.port,"body":d.body,"occurrence":path,"origin":d.origin,"quaternion":d.quaternion,
                    "support":if d.transfer>=2 {"surface"}else{"point"},"transfer":match d.transfer{0=>"point_force",3=>"pressure",_=>"frame_wrench"}});
                    if d.transfer>=2 {p["area_mm2"]=json!(d.area);}run(app,&mut d,"mechanics.port",p);},Err(e)=>d.error=e.to_string()}
            }
        });
        ui.collapsing("Starren Lastpfad verbinden",|ui|{
            text(ui,"mechanics:parent","Richtung Fundament / Eltern-Port",&mut d.parent);text(ui,"mechanics:child","Richtung Spitze / Kind-Port",&mut d.child);
            ui.small("Expliziter starrer Lastpfad. Ein Eltern-Port je Kind, keine Schleifen oder automatische Ableitung aus CAD-Gelenken.");
            if button(ui,"mechanics:connect","Lastpfad speichern"){let p=json!({"parent":d.parent,"child":d.child});run(app,&mut d,"mechanics.connect",p);}
            if button(ui,"mechanics:disconnect","Eltern-Verbindung des Kind-Ports lösen"){let p=json!({"child":d.child});run(app,&mut d,"mechanics.disconnect",p);}
        });
        ui.collapsing("Einzellastfall anlegen",|ui|{
            text(ui,"mechanics:case:id","Neue Lastfall-ID",&mut d.case);text(ui,"mechanics:case:root","Fundament-Port",&mut d.root);text(ui,"mechanics:case:load","Last-Port",&mut d.load_port);
            ui.checkbox(&mut d.pressure_load,"Normaldruck statt Kraft/Moment");
            if d.pressure_load {ui.horizontal(|ui|{ui.label("Druck [MPa = N/mm²]");ui.add(egui::DragValue::new(&mut d.pressure));});}
            else {for (label,values) in [("Kraft lokal [N]",&mut d.force),("Moment lokal [Nm]",&mut d.moment)] {ui.horizontal(|ui|{ui.label(label);for v in values {ui.add(egui::DragValue::new(v));}});}}
            if button(ui,"mechanics:case:save","Einzellastfall speichern") {
                let existing=state["network"]["cases"].get(&d.case);
                if existing.and_then(|v|v["loads"].as_array()).is_some_and(|a|a.len()>1){d.error="Mehrlastfall über mechanics.case bearbeiten; der Einzellasteditor überschreibt ihn nicht.".into();}
                else {let l=if d.pressure_load {json!({"port":d.load_port,"pressure_mpa":d.pressure})}else{json!({"port":d.load_port,"force_n":d.force,"moment_nm":d.moment})};
                    let p=json!({"id":d.case,"root":d.root,"loads":[l]});run(app,&mut d,"mechanics.case",p);}
            }
        });
        ui.separator();ui.strong("Lastfälle und gespeicherte Ergebnisse");
        for (id,_) in state["network"]["cases"].as_object().into_iter().flatten(){ui.horizontal(|ui|{
            ui.label(id);let status=state["results"].as_array().and_then(|a|a.iter().find(|r|r["case"]==*id));
            ui.label(match status {Some(v) if v["current"]==true=>"Ergebnis aktuell",Some(_)=>"Ergebnis veraltet / ungültig",None=>"Noch nicht berechnet"});
            if button(ui,&format!("mechanics:solve:{id}"),"Statisch berechnen"){run(app,&mut d,"mechanics.solve",json!({"case":id}));}
        });}
        if let Some(report)=d.report.as_ref().or_else(||state["network"]["results"].as_object().and_then(|r|r.values().next())){
            ui.strong(format!("Fundamentreaktion, Lastfall {}",report["case"].as_str().unwrap_or("")));
            ui.label(format!("Kraft Welt [N]: {}",report["root_reaction_world"]["force_n"]));
            ui.label(format!("Moment Welt [Nm]: {}",report["root_reaction_world"]["moment_nm"]));
            ui.small("Gespeicherter Ergebnisstand; Gültigkeitsanzeige je Lastfall beachten. M = r × F berücksichtigt mm → m.");
        }
        if !d.error.is_empty(){ui.colored_label(Tokens::get().warning,&d.error);}
    });
    ui.data_mut(|data| data.insert_temp(key, d));
}
fn diagram(ui: &mut egui::Ui, state: &Value, d: &mut Draft) {
    let t = Tokens::get();
    let mut groups: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for p in state["network"]["ports"].as_object().into_iter().flat_map(|m| m.values()) {
        groups.entry(format!("{} @ {}", p["body"].as_str().unwrap_or(""), p["occurrence"])).or_default().push(p);
    }
    if groups.is_empty() {
        ui.label("Zuerst Projektschnittstellen anlegen.");
        return;
    }
    egui::ScrollArea::horizontal().id_salt("mechanical_graph").show(ui, |ui| {
        let width = (groups.len().min(3) as f32 * 220.).max(440.);
        let height = groups.len().div_ceil(3) as f32 * 160. + 30.;
        let (area, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
        let painter = ui.painter_at(area);
        let mut anchors = BTreeMap::new();
        for (i, (name, ports)) in groups.iter().enumerate() {
            let rect =
                egui::Rect::from_min_size(area.min + egui::vec2((i % 3) as f32 * 220. + 12., (i / 3) as f32 * 160. + 20.), egui::vec2(190., 120.));
            painter.rect_filled(rect, 4., t.field);
            painter.rect_stroke(rect, 4., egui::Stroke::new(1., t.border), egui::StrokeKind::Inside);
            painter.text(rect.center_top() + egui::vec2(0., 15.), egui::Align2::CENTER_CENTER, name, egui::FontId::proportional(11.), t.text);
            for (j, p) in ports.iter().take(4).enumerate() {
                let point = rect.left_top() + egui::vec2(12., 38. + j as f32 * 18.);
                let id = p["id"].as_str().unwrap_or("");
                painter.circle_filled(point, 3., t.accent);
                painter.text(point + egui::vec2(9., 0.), egui::Align2::LEFT_CENTER, id, egui::FontId::proportional(11.), t.text);
                anchors.insert(id, point);
                let hit = egui::Rect::from_center_size(point + egui::vec2(60., 0.), egui::vec2(130., 17.));
                let r = ui.interact(hit, ui.id().with(("mechanical_port", id)), egui::Sense::click());
                scenario::publish_handle(&format!("mechanics:node:{id}"), hit.center());
                if r.clicked() {
                    load(d, p);
                }
            }
            if ports.len() > 4 {
                painter.text(
                    rect.center_bottom() - egui::vec2(0., 5.),
                    egui::Align2::CENTER_BOTTOM,
                    format!("+ {} Ports", ports.len() - 4),
                    egui::FontId::proportional(10.),
                    t.text_dim,
                );
            }
        }
        for (child, parent) in state["network"]["parents"].as_object().into_iter().flatten() {
            if let Some((a, b)) = parent.as_str().and_then(|p| anchors.get(p)).zip(anchors.get(child.as_str())) {
                painter.line_segment([*a, *b], egui::Stroke::new(1.2, t.accent));
            }
        }
    });
    ui.small("Blöcke gruppieren Ports desselben CAD-Körpers und Occurrence-Pfads. Linien sind deklarierte starre Lastpfade, keine automatisch erzeugten MKS-Gelenke.");
}
