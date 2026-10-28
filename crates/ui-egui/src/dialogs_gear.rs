//! Thin gear-tool dialog: all equations and capability decisions live in engine commands.
use crate::SolveApp;
use crate::dialogs::{Kind, check, combo, field, row_label};
use crate::selection::SelInput;
use serde_json::{Value, json};

const IDS: &[&str] = &["spur", "internal", "helical", "internal_helical", "herringbone", "double_helical", "worm", "bevel", "spiral_bevel", "rack"];
const LABELS: &[&str] = &[
    "Stirnrad",
    "Hohlrad",
    "Schrägverzahntes Stirnrad",
    "Schrägverzahntes Hohlrad",
    "Pfeilverzahnung",
    "Doppelschräg mit Mittelnut",
    "Schnecke / Schneckenrad",
    "Gerades Kegelrad",
    "Spiralkegelrad",
    "Zahnstange",
];
#[derive(Clone, Debug)]
pub struct Gear {
    family: usize,
    create: bool,
    view: usize,
    fields: Vec<(&'static str, &'static str, String)>,
    result: Option<Result<Value, String>>,
    selected: usize,
    name: String,
    base: Value,
    review: Option<Value>,
    revision: u64,
    other: usize,
    comparison: Option<Result<Value, String>>,
}
fn blank() -> Gear {
    Gear {
        family: 0,
        create: true,
        view: 0,
        result: None,
        selected: 0,
        name: "Entwurf 1".into(),
        base: json!({}),
        review: None,
        revision: 0,
        other: 0,
        comparison: None,
        fields: vec![
            ("module", "Modul", "2 mm".into()),
            ("teeth", "Zähnezahl", "30".into()),
            ("mate_teeth", "Zähnezahl Gegenrad", "60".into()),
            ("width", "Zahnbreite", "20 mm".into()),
            ("pressure_angle", "Eingriffswinkel", "20 deg".into()),
            ("helix_angle", "Schrägungswinkel", "20 deg".into()),
            ("groove", "Mittelnut", "2 mm".into()),
            ("starts", "Schneckengänge", "2".into()),
            ("diameter_factor", "Durchmesserfaktor q", "10".into()),
            ("shaft_angle", "Achswinkel", "90 deg".into()),
            ("torque_nm", "Eingangsdrehmoment [Nm]", "100".into()),
            ("speed_rpm", "Eingangsdrehzahl [rpm]", "1500".into()),
            ("radius", "Bohrungsradius / Außenradius", "5 mm".into()),
            ("backlash", "Zahnflankenspiel", "0 mm".into()),
            ("phase", "Drehwinkel", "0 deg".into()),
            ("center_x", "Position X", "0 mm".into()),
            ("center_y", "Position Y", "0 mm".into()),
            ("center_z", "Position Z", "0 mm".into()),
        ],
    }
}
fn load(gear: &mut Gear, item: &Value, index: usize) {
    *gear = blank();
    gear.selected = index + 1;
    gear.name = item["name"].as_str().map(str::to_owned).unwrap_or(format!("Entwurf {}", index + 1));
    let inputs = &item["inputs"];
    gear.base = if inputs.is_object() { inputs.clone() } else { json!({}) };
    if let Some(object) = gear.base.as_object_mut() {
        object.remove("center");
    }
    gear.family = IDS.iter().position(|id| inputs["family"] == *id).unwrap_or(0);
    gear.create = item["feature"]["feature"].is_u64() && gear.family < 2;
    for (key, _, value) in &mut gear.fields {
        let coordinate = match *key {
            "center_x" => Some(0),
            "center_y" => Some(1),
            "center_z" => Some(2),
            _ => None,
        };
        if let Some(v) = coordinate.and_then(|i| inputs["center"].get(i)).or_else(|| inputs.get(*key)) {
            if let Some(text) = v.as_str() {
                *value = text.into();
            } else if v.is_number() {
                *value = v.to_string();
            }
        }
    }
}
pub fn start(app: &SolveApp, id: &str) -> Option<(Kind, Vec<SelInput>)> {
    if id != "gear.design" {
        return None;
    }
    let mut gear = blank();
    if let Some(items) = app.session.doc.metadata.get("org.solidcraft.gear-designs").and_then(|v| v["items"].as_array())
        && let Some(item) = items.last()
    {
        load(&mut gear, item, items.len() - 1);
    }
    Some((Kind::Gear(gear), vec![]))
}
fn params(k: &Gear) -> Value {
    let mut p = k.base.clone();
    p["family"] = json!(IDS.get(k.family).copied().unwrap_or("spur"));
    p["create"] = json!(k.create && k.family < 2);
    p["name"] = json!(k.name);
    for (key, _, value) in &k.fields {
        p[*key] = json!(value);
    }
    p
}
pub fn commands(k: &Gear) -> Result<Vec<(String, Value)>, String> {
    if k.selected == 0 {
        Ok(vec![("gear.design".into(), params(k))])
    } else {
        Ok(vec![("gear.update".into(), json!({"design":k.selected-1,"name":k.name,"inputs":params(k),"create":k.create && k.family<2}))])
    }
}
pub fn rows(app: &mut SolveApp, ui: &mut egui::Ui, k: &mut Gear) -> bool {
    let items = app.session.doc.metadata.get("org.solidcraft.gear-designs").and_then(|v| v["items"].as_array()).cloned().unwrap_or_default();
    let mut labels = vec!["Neuer Entwurf / Kopie".to_owned()];
    labels.extend(
        items
            .iter()
            .enumerate()
            .map(|(i, v)| format!("{}: {}", i + 1, v["name"].as_str().map(str::to_owned).unwrap_or(format!("Entwurf {}", i + 1)))),
    );
    row_label(ui, "Entwurf");
    let previous = k.selected;
    combo(ui, "gear_saved", &labels.iter().map(String::as_str).collect::<Vec<_>>(), &mut k.selected);
    ui.end_row();
    if previous != k.selected {
        if let Some(item) = k.selected.checked_sub(1).and_then(|i| items.get(i)) {
            load(k, item, k.selected - 1);
        } else {
            k.name = format!("Entwurf {}", items.len() + 1);
            k.create = false;
            k.review = None;
            k.comparison = None;
            if let Some(o) = k.base.as_object_mut() {
                o.remove("body_name");
            }
        }
    }
    row_label(ui, "Name");
    field(ui, &mut k.name);
    ui.end_row();
    if k.selected > 0 && (k.review.is_none() || k.revision != app.session.revision) {
        k.review = Some(app.session.execute("gear.review", &json!({"design":k.selected-1})).unwrap_or_else(|e| json!({"error":e.to_string()})));
        k.revision = app.session.revision;
    }
    if let Some(review) = &k.review {
        row_label(ui, "Gespeicherter Stand");
        let message = if review.get("error").is_some() {
            review["error"].as_str().unwrap_or("Prüfung fehlgeschlagen")
        } else if review["status"] == "invalid" {
            "Referenzen ungültig. Berechnung prüfen."
        } else if review["status"] == "outdated" {
            "Berechnungswerte geändert. Erneut berechnen und mit OK übernehmen."
        } else {
            "Berechnung aktuell"
        };
        ui.label(message);
        ui.end_row();
        row_label(ui, "CAD-Verbindung");
        ui.label(match review["feature_status"].as_str().unwrap_or("") {
            "current" => "Konstruktion entspricht den gespeicherten Eingaben",
            "unlinked" => "Berechnung ohne verknüpftes Zahnrad",
            "different_definition" => "CAD und Entwurf haben unterschiedliche Eingaben. Aktualisieren ersetzt die CAD-Definition.",
            "missing" => "Verknüpftes Zahnrad wurde gelöscht",
            "suppressed" => "Verknüpftes Zahnrad ist unterdrückt",
            "unsupported_family" => "Für diese Familie fehlt der CAD-Generator",
            _ => "CAD-Verbindung prüfen",
        });
        ui.end_row();
    }
    let before = params(k);
    row_label(ui, "Zahnradart");
    combo(ui, "gear_family", LABELS, &mut k.family);
    ui.end_row();
    if before["family"] != params(k)["family"] {
        k.result = None;
        if k.family == 1 {
            for (key, _, value) in &mut k.fields {
                match *key {
                    "teeth" => *value = "60".into(),
                    "mate_teeth" => *value = "30".into(),
                    "radius" => *value = "80 mm".into(),
                    _ => {}
                }
            }
        }
        k.create = k.family < 2;
    }
    row_label(ui, "Modulsystem");
    ui.label(if k.family == 6 {
        "Axialmodul der Schnecke"
    } else if matches!(k.family, 7 | 8) {
        "Außenmodul, Stirnschnitt"
    } else {
        "Normalmodul (bei beta=0: Stirnmodul)"
    });
    ui.end_row();
    row_label(ui, "CAD-Geometrie");
    if k.family < 2 {
        ui.horizontal(|ui| {
            check(ui, &mut k.create);
            ui.label(if k.selected > 0 { "Zahnrad erstellen / aktualisieren" } else { "Zahnrad erstellen" });
        });
    } else {
        ui.label("Nur Berechnung; nativer Flankengenerator noch offen.");
    }
    ui.end_row();
    row_label(ui, "Ansicht");
    combo(ui, "gear_view", &["Eingaben", "Ergebnisse", "Vergleich"], &mut k.view);
    ui.end_row();
    row_label(ui, "Berechnung");
    let r = ui.button("Berechnen");
    crate::scenario::publish_handle("dialog:Berechnen", r.rect.center());
    let calculate = r.clicked();
    ui.end_row();
    if k.view == 2 && k.selected > 0 {
        row_label(ui, "Vergleich mit");
        let previous_other = k.other;
        combo(ui, "gear_other", &labels.iter().skip(1).map(String::as_str).collect::<Vec<_>>(), &mut k.other);
        ui.end_row();
        if previous_other != k.other {
            k.comparison = None;
        }
        row_label(ui, "Gespeicherte Ergebnisse");
        let r = ui.button("Vergleichen");
        crate::scenario::publish_handle("dialog:Vergleichen", r.rect.center());
        if r.clicked() {
            k.comparison = Some(app.session.execute("gear.compare", &json!({"left":k.selected-1,"right":k.other})).map_err(|e| e.to_string()));
        }
        ui.end_row();
        if let Some(result) = &k.comparison {
            match result {
                Ok(v) => {
                    if let Some(changes) = v["changes"].as_array() {
                        for c in changes {
                            row_label(ui, c["symbol"].as_str().unwrap_or(""));
                            ui.label(format!(
                                "{} {} → {} {}",
                                c["before"],
                                c["before_unit"].as_str().unwrap_or(""),
                                c["after"],
                                c["after_unit"].as_str().unwrap_or("")
                            ));
                            ui.end_row();
                        }
                        if changes.is_empty() {
                            row_label(ui, "Vergleich");
                            ui.label("Keine Ergebnisunterschiede im gespeicherten Stand");
                            ui.end_row();
                        }
                    }
                }
                Err(e) => {
                    row_label(ui, "Vergleich");
                    ui.label(e);
                    ui.end_row();
                }
            }
        }
    }
    if k.view == 0 {
        for (key, label, value) in &mut k.fields {
            let visible = match *key {
                "helix_angle" => matches!(k.family, 2..=5),
                "groove" => k.family == 5,
                "starts" | "diameter_factor" => k.family == 6,
                "shaft_angle" => matches!(k.family, 7 | 8),
                "teeth" => k.family != 6,
                "mate_teeth" => k.family != 9,
                "radius" | "backlash" | "phase" | "center_x" | "center_y" | "center_z" => k.family < 2,
                _ => true,
            };
            if visible {
                row_label(ui, label);
                field(ui, value);
                ui.end_row();
            }
        }
        if before != params(k) {
            k.result = None;
        }
    }
    if calculate {
        k.result = Some(app.session.execute("gear.evaluate", &params(k)).map_err(|e| e.to_string()));
        k.view = 1;
    }
    if before != params(k) {
        k.comparison = None;
    }
    if k.view == 1 {
        if let Some(result) = &k.result {
            match result {
                Ok(v) => {
                    if let Some(rows) = v["rows"].as_array() {
                        for row in rows {
                            row_label(ui, row["symbol"].as_str().unwrap_or(""));
                            ui.label(format!("{:.6} {}", row["value"].as_f64().unwrap_or(0.), row["unit"].as_str().unwrap_or("")))
                                .on_hover_text(row["equation"].as_str().unwrap_or(""));
                            ui.end_row();
                        }
                    }
                    row_label(ui, "Geltungsbereich");
                    ui.vertical(|ui| {
                        if let Some(warnings) = v["warnings"].as_array() {
                            for w in warnings {
                                ui.label(w.as_str().unwrap_or(""));
                            }
                        }
                        ui.hyperlink_to(
                            "KHK: Berechnungsgrundlagen",
                            "https://khkgears.net/new/gear_knowledge/gear_technical_reference/calculation_gear_dimensions.html",
                        );
                    });
                    ui.end_row();
                }
                Err(e) => {
                    row_label(ui, "Fehler");
                    ui.label(e);
                    ui.end_row();
                }
            }
        } else {
            row_label(ui, "Ergebnisse");
            ui.label("Eingaben geändert oder noch nicht berechnet. Berechnen drücken.");
            ui.end_row();
        }
    }
    row_label(ui, "Projekt");
    ui.label(if k.selected == 0 {
        "OK speichert einen neuen Entwurf. ref_... kann in jedem Zahlenfeld stehen."
    } else {
        "OK aktualisiert diesen Entwurf. Mit aktivierter CAD-Geometrie wird das vorhandene Zahnrad ersetzt; seine Feature-ID bleibt erhalten."
    });
    ui.end_row();
    false
}
