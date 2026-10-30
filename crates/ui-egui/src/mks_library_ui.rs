//! Library insert forms dispatch ordinary, reusable engine factories.
use crate::{SolveApp, scenario};
use serde_json::{Value, json};
#[derive(Clone)]
pub(super) struct Insert {
    pub kind: String,
    pub port: String,
    name: String,
    mass: f64,
    size: [f64; 3],
    point: [f64; 3],
    axis: [f64; 3],
    a: u64,
    b: u64,
    error: String,
    occurrence: Option<u64>,
}
impl Insert {
    pub fn new(kind: &str) -> Self {
        Self {
            kind: kind.into(),
            port: String::new(),
            name: kind.into(),
            mass: 5.,
            size: [80.; 3],
            point: [0.; 3],
            axis: [0., 0., 1.],
            a: 0,
            b: 0,
            error: String::new(),
            occurrence: None,
        }
    }
    pub fn edit_mass(occurrence: u64, mass: f64) -> Self {
        let mut d = Self::new("payload_edit");
        d.occurrence = Some(occurrence);
        d.mass = mass;
        d
    }
}
pub(super) fn show(app: &mut SolveApp, ctx: &egui::Context, model: &Value, insert: &mut Option<Insert>) {
    let Some(d) = insert.as_mut() else { return };
    let mut done = false;
    let title = app.tr(if d.occurrence.is_some() { "ui.mks_lib.edit_payload" } else { "ui.mks_lib.insert" }).to_owned();
    let closed=crate::ui_kit::Window::new(egui::Id::new("mks_library_insert"),"component",&title).cards(2.5).show_body(ctx,|ui|{
        ui.strong(app.tr(&format!("ui.mks_lib.{}",if d.kind=="payload_edit"{"payload"}else{&d.kind})));
        if d.occurrence.is_none(){crate::ui_kit::text_row(ui,app.tr("ui.mks_lib.name"),&mut d.name,"");}
        let body=d.kind=="body" || d.kind=="payload" || d.kind=="payload_edit";
        if body {
            crate::ui_kit::row(ui,app.tr("ui.mks_lib.mass"),|ui|{let r=ui.add(egui::DragValue::new(&mut d.mass).range(0.000001..=100000.).speed(0.1));scenario::publish_handle("mks:insert:mass",r.rect.center());});
            if d.kind!="payload_edit"{ui.label(app.tr("ui.mks_lib.size"));vector(ui,&mut d.size,"size");}
            if d.kind=="payload" {
                ui.label(app.tr("ui.mks_lib.attachment"));
                let combo=egui::ComboBox::from_id_salt("payload_port").selected_text(&d.port).show_ui(ui,|ui|{
                    for b in model["blocks"].as_array().into_iter().flatten().filter(|b|b["grounded"]!=true){
                        for p in b["ports"].as_array().into_iter().flatten(){if let Some(id)=p["port"]["id"].as_str(){let r=ui.selectable_value(&mut d.port,id.to_owned(),id);scenario::publish_handle(&format!("mks:insert:port:{id}"),r.rect.center());}}
                    }
                });
                scenario::publish_handle("mks:insert:port",combo.response.rect.center());
                ui.small(app.tr("ui.mks_lib.native_mass"));
            } else if d.kind!="payload_edit" {ui.label(app.tr("ui.mks_lib.position"));vector(ui,&mut d.point,"position");}
        } else {
            let members:Vec<_>=model["blocks"].as_array().into_iter().flatten().flat_map(|b|b["members"].as_array().into_iter().flatten()).collect();
            for (key,slot) in [("B",&mut d.a),("F",&mut d.b)] {
                egui::ComboBox::from_id_salt(("joint_occ",key)).selected_text(format!("{key}: {}",members.iter().find(|m|m["path"].as_array().and_then(|p|p.last()).and_then(Value::as_u64).unwrap_or(0)==*slot).and_then(|m|m["name"].as_str()).unwrap_or("World"))).show_ui(ui,|ui|{
                    for m in &members {let id=m["path"].as_array().and_then(|p|p.last()).and_then(Value::as_u64).unwrap_or(0);ui.selectable_value(slot,id,m["name"].as_str().unwrap_or("World"));}
                });
            }
            ui.label(app.tr("ui.mks_lib.position"));vector(ui,&mut d.point,"position");
            ui.label(app.tr("ui.mks_lib.axis"));vector(ui,&mut d.axis,"axis");
        }
        let create=ui.button(app.tr(if d.occurrence.is_some(){"ui.mks_lib.apply_mass"}else{"ui.mks_lib.create"}));scenario::publish_handle("mks:insert:create",create.rect.center());
        if create.clicked(){
            let (cmd,params)=if let Some(oid)=d.occurrence {("mks.library.mass",json!({"occurrence":oid,"mass_kg":d.mass}))}else if body {("mks.library.body",json!({"name":d.name,"mass_kg":d.mass,"size_mm":d.size,"position_mm":d.point,"port":if d.kind=="payload"{Some(&d.port)}else{None}}))}
                else{("mks.library.joint",json!({"name":d.name,"kind":d.kind,"a":d.a,"b":d.b,"point_mm":d.point,"axis":d.axis}))};
            match app.run(cmd,params){Ok(_)=>done=true,Err(e)=>d.error=e}
        }
        if !d.error.is_empty(){ui.colored_label(crate::theme::Tokens::get().warning,&d.error);}
    });
    if closed || done {
        *insert = None;
    }
}
fn vector(ui: &mut egui::Ui, values: &mut [f64; 3], key: &str) {
    ui.horizontal(|ui| {
        for (i, v) in values.iter_mut().enumerate() {
            let r = ui.add(egui::DragValue::new(v).speed(1.));
            scenario::publish_handle(&format!("mks:insert:{key}:{i}"), r.rect.center());
        }
    });
}
