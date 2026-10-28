//! Transient state and an original robot scene for the 10derCraft interaction prototype.
//! The CAD/MKS graph remains a draft; independent beam reference studies are separate.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub enabled: bool,
    pub page: String,
    pub tool: String,
    pub selected: String,
    pub representation: String,
    pub variant: String,
    pub notice: String,
    pub interfaces: Vec<MechanicalInterface>,
    pub connections: Vec<MechanicalConnection>,
    pub selected_interface: String,
    pub pending_connection: Option<String>,
    pub mode_count: usize,
    pub module_frames: std::collections::BTreeMap<String, InterfaceFrame>,
    #[serde(default)]
    pub beam_result: Option<Value>,
    #[serde(default)]
    pub backend_job: Option<Value>,
    #[serde(default)]
    pub backend_result: Option<Value>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            enabled: false,
            page: "cad".into(),
            tool: "gripper".into(),
            selected: "flange".into(),
            representation: "rigid".into(),
            variant: "baseline".into(),
            notice: "Geometrievorschau. Simulation und Kompatibilitätsprüfung sind noch nicht implementiert.".into(),
            interfaces: Vec::new(),
            connections: Vec::new(),
            selected_interface: String::new(),
            pending_connection: None,
            mode_count: 6,
            module_frames: Default::default(),
            beam_result: None,
            backend_job: None,
            backend_result: None,
        }
    }
}

/// A reference becomes historical once a bound CAD length changes or vanishes.
pub fn reference_parameters_current(doc: &crate::doc::Document, report: &Value) -> bool {
    let Some(bindings) = report.get("parameter_bindings").and_then(Value::as_object) else { return true };
    let (values, _) = doc.param_values();
    bindings.iter().all(|(field, name)| {
        let Some(value) = name.as_str().and_then(|name| values.get(name)) else { return false };
        value.kind() == Some(crate::doc::expr::Kind::Length)
            && report.get("inputs").and_then(|v| v.get(field)).and_then(Value::as_f64) == Some(value.v)
    })
}

/// A logical mechanical port. Its frame and its geometric load support are separate.
/// CAD picks are provisional locators, not persistent topology or qualified FEM couplings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MechanicalInterface {
    pub id: String,
    pub owner: String,
    pub label: String,
    pub origin_mm: [f64; 3],
    pub quaternion_wxyz: [f64; 4],
    pub support: String,
    pub transfer: String,
    pub cad_selection: Vec<crate::Sel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MechanicalConnection {
    pub id: String,
    pub a: String,
    pub b: String,
    pub kind: String,
    pub axis_in_a: Option<[f64; 3]>,
}

/// Module-frame poses in the CAD model frame, explicitly authored for the demo.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterfaceFrame {
    pub origin_mm: [f64; 3],
    pub quaternion_wxyz: [f64; 4],
}

pub fn demo_module_frames() -> std::collections::BTreeMap<String, InterfaceFrame> {
    [
        ("base", [0., 0., 0.]),
        ("axis1", [0., 0., 230.]),
        ("upper", [0., 0., 390.]),
        ("forearm", [300., 0., 975.]),
        ("wrist4", [815., 0., 901.]),
        ("wrist5", [903., 0., 912.]),
        ("flange", [985., 0., 925.]),
        ("tool", [1011., 0., 925.]),
    ]
    .into_iter()
    .map(|(name, origin_mm)| (name.into(), InterfaceFrame { origin_mm, quaternion_wxyz: [1., 0., 0., 0.] }))
    .collect()
}

pub fn belongs_to_module(module: &str, tool: &str, body: &str) -> bool {
    match module {
        "base" => ["Fundament", "Sockel"].contains(&body),
        "axis1" | "shoulder" => ["A1-Drehkranz", "Schultertraeger"].contains(&body),
        "upper" => body.starts_with("Oberarm") || body.starts_with("A2-"),
        "forearm" => body.starts_with("Unterarm") || body.starts_with("A3-"),
        "wrist4" => body.starts_with("A4-"),
        "wrist5" => body.starts_with("A5-"),
        "wrist" => ["A4-", "A5-", "A6-"].iter().any(|prefix| body.starts_with(prefix)),
        "flange" => body.starts_with("A6-") || body == "Werkzeugflansch",
        "tool" => body.starts_with(match tool {
            "welder" => "Schweiss-",
            "payload" => "Nutzlast-",
            _ => "Greifer-",
        }),
        _ => body == module,
    }
}

impl State {
    /// Port location for the CAD overlay, using the authored module-frame pose.
    pub fn interface_origin_model_mm(&self, id: &str) -> Option<[f64; 3]> {
        let port = self.interfaces.iter().find(|p| p.id == id)?;
        let frame = self.module_frames.get(&port.owner)?;
        let [w, x, y, z] = frame.quaternion_wxyz;
        let [a, b, c] = port.origin_mm;
        Some([
            frame.origin_mm[0] + (1. - 2. * (y * y + z * z)) * a + 2. * (x * y - z * w) * b + 2. * (x * z + y * w) * c,
            frame.origin_mm[1] + 2. * (x * y + z * w) * a + (1. - 2. * (x * x + z * z)) * b + 2. * (y * z - x * w) * c,
            frame.origin_mm[2] + 2. * (x * z - y * w) * a + 2. * (y * z + x * w) * b + (1. - 2. * (x * x + y * y)) * c,
        ])
    }
}

/// Explicit demonstration grouping, not an automatic discovery of rigid CAD groups.
pub const MODULES: [(&str, &str, &str); 8] = [
    ("base", "Basis / Fundament", "Starr"),
    ("axis1", "Grundachse A1", "Starr"),
    ("upper", "Oberarm", "Starr / flexibel / ROM"),
    ("forearm", "Unterarm", "Starr / flexibel / ROM"),
    ("wrist4", "Handachse A4", "Starr"),
    ("wrist5", "Handachse A5", "Starr"),
    ("flange", "Werkzeugflansch A6", "Starr"),
    ("tool", "Werkzeugmodul", "Austauschbar"),
];

pub fn demo_interfaces() -> Vec<MechanicalInterface> {
    [
        ("base", "F_A1", [0., 0., 230.]),
        ("axis1", "F_A1", [0., 0., 0.]),
        ("axis1", "F_A2", [0., 0., 160.]),
        ("upper", "DH_A", [0., 0., 0.]),
        ("upper", "DH_E", [300., 0., 585.]),
        ("forearm", "DH_A", [0., 0., 0.]),
        ("forearm", "F_A4", [515., 0., -74.]),
        ("wrist4", "F_A4", [0., 0., 0.]),
        ("wrist4", "F_A5", [88., 0., 11.]),
        ("wrist5", "F_A5", [0., 0., 0.]),
        ("wrist5", "F_A6", [82., 0., 13.]),
        ("flange", "F_A6", [0., 0., 0.]),
        ("flange", "IF_06", [26., 0., 0.]),
        ("tool", "IF_06", [0., 0., 0.]),
        ("tool", "TCP", [180., 0., 0.]),
    ]
    .into_iter()
    .map(|(owner, label, origin_mm)| MechanicalInterface {
        id: format!("{owner}.{label}"),
        owner: owner.into(),
        label: label.into(),
        origin_mm,
        quaternion_wxyz: [1., 0., 0., 0.],
        support: if label == "TCP" { "point" } else { "surface_group" }.into(),
        transfer: if label == "TCP" { "frame_wrench" } else { "distributed_remote" }.into(),
        cad_selection: Vec::new(),
    })
    .collect()
}

pub fn demo_connections() -> Vec<MechanicalConnection> {
    [
        ("J1", "base.F_A1", "axis1.F_A1", "revolute"),
        ("J2", "axis1.F_A2", "upper.DH_A", "revolute"),
        ("J3", "upper.DH_E", "forearm.DH_A", "revolute"),
        ("J4", "forearm.F_A4", "wrist4.F_A4", "revolute"),
        ("J5", "wrist4.F_A5", "wrist5.F_A5", "revolute"),
        ("J6", "wrist5.F_A6", "flange.F_A6", "revolute"),
        ("Montage", "flange.IF_06", "tool.IF_06", "fixed"),
    ]
    .into_iter()
    .map(|(id, a, b, kind)| MechanicalConnection {
        id: id.into(),
        a: a.into(),
        b: b.into(),
        kind: kind.into(),
        axis_in_a: match id {
            "J1" => Some([0., 0., 1.]),
            "J2" | "J3" | "J5" => Some([0., 1., 0.]),
            "J4" => Some([0.9928768385, 0., 0.1191452206]),
            "J6" => Some([1., 0., 0.]),
            _ => None,
        },
    })
    .collect()
}

pub fn support_label(kind: &str) -> &'static str {
    match kind {
        "point" => "Punkt",
        "surface" => "Fläche",
        _ => "Flächengruppe",
    }
}

pub fn transfer_label(kind: &str) -> &'static str {
    match kind {
        "point_force" => "Punktkraft (3 Komponenten)",
        "frame_wrench" => "Frame: Kraft und Moment",
        "rigid_remote" => "Starre Remote-Kopplung",
        "pressure" => "Flächendruck / Normalbelastung",
        _ => "Verteilte Remote-Kopplung",
    }
}

pub const TOOLS: [(&str, &str, &str); 3] = [
    ("gripper", "Parallelgreifer", "Greifen & Handhaben"),
    ("welder", "Schweißwerkzeug", "Fügen & Bearbeiten"),
    ("payload", "Nutzlastmodul", "Lastfälle & Referenzen"),
];

pub fn tool_label(id: &str) -> &'static str {
    TOOLS.iter().find(|(key, _, _)| *key == id).map_or("Parallelgreifer", |(_, name, _)| *name)
}

/// Original, illustrative geometry in millimetres. These are CAD primitives, not a
/// manufacturer's robot or an identified six-axis dynamic model.
pub fn robot_script() -> Value {
    let mut commands = Vec::new();
    let mut orange = Vec::new();
    let mut dark = Vec::new();
    let mut steel = Vec::new();
    let mut tool = Vec::new();
    let boxes = [
        ("Fundament", [0., 0., 18.], [420., 360., 36.], 1),
        ("Sockel", [0., 0., 108.], [238., 230., 144.], 1),
        ("Schultertraeger", [0., 0., 290.], [150., 155., 180.], 0),
        ("Greifer-Gehaeuse", [1053., 0., 925.], [105., 112., 88.], 3),
        ("Greifer-Backe-L", [1140., -39., 925.], [100., 20., 62.], 2),
        ("Greifer-Backe-R", [1140., 39., 925.], [100., 20., 62.], 2),
        ("Greifer-Spitze-L", [1180., -29., 925.], [22., 40., 62.], 2),
        ("Greifer-Spitze-R", [1180., 29., 925.], [22., 40., 62.], 2),
        ("Schweiss-Gehaeuse", [1060., 0., 925.], [112., 72., 72.], 3),
        ("Nutzlast-Adapter", [1026., 0., 925.], [40., 124., 124.], 2),
        ("Nutzlast-Block", [1120., 0., 925.], [150., 150., 150.], 3),
    ];
    for (name, center, size, group) in boxes {
        commands.push(json!({"command":"solid.box", "params":{
            "length":size[0],"width":size[1],"height":size[2],"center":center,
            "operation":"new","body_name":name,"name":name
        }}));
        match group {
            0 => orange.push(name),
            1 => dark.push(name),
            2 => steel.push(name),
            _ => tool.push(name),
        }
    }
    let cylinders = [
        ("A1-Drehkranz", [0., 0., 178.], [0., 0., 1.], 135., 65., 2),
        ("A2-Schulter", [0., -104., 390.], [0., 1., 0.], 119., 208., 0),
        ("A2-Deckel", [0., -112., 390.], [0., 1., 0.], 80., 14., 1),
        ("Oberarm", [0., 0., 405.], [300., 0., 570.], 77., 644.1, 0),
        ("Oberarm-Ruecken", [-15., -60., 424.], [300., 0., 570.], 22., 570., 1),
        ("A3-Ellbogen", [300., -85., 975.], [0., 1., 0.], 97., 170., 0),
        ("A3-Deckel", [300., -96., 975.], [0., 1., 0.], 62., 15., 1),
        ("Unterarm", [320., 0., 972.], [500., 0., -72.], 65., 505.2, 0),
        ("Unterarm-Ruecken", [340., -56., 975.], [450., 0., -65.], 17., 430., 1),
        ("A4-Handgelenk", [815., 0., 901.], [1., 0., 0.12], 72., 92., 1),
        ("A5-Handachse", [903., -68., 912.], [0., 1., 0.], 65., 136., 0),
        ("A5-Deckel", [903., -77., 912.], [0., 1., 0.], 42., 12., 2),
        ("A6-Werkzeugachse", [923., 0., 925.], [1., 0., 0.], 48., 66., 0),
        ("Werkzeugflansch", [985., 0., 925.], [1., 0., 0.], 63., 26., 2),
        ("Schweiss-Hals", [1095., 0., 925.], [1., 0., -0.35], 17., 115., 2),
        ("Schweiss-Duese", [1200., 0., 888.], [1., 0., -0.35], 24., 55., 2),
    ];
    for (name, base, axis, radius, height, group) in cylinders {
        commands.push(json!({"command":"solid.cylinder", "params":{
            "radius":radius,"height":height,"base":base,"axis":axis,
            "operation":"new","body_name":name,"name":name
        }}));
        match group {
            0 => orange.push(name),
            1 => dark.push(name),
            _ => steel.push(name),
        }
    }
    for (bodies, color) in [(orange, "#ed862e"), (dark, "#35414f"), (steel, "#bac6d1"), (tool, "#3e99a5")] {
        commands.push(json!({"command":"appearance.assign","params":{"bodies":bodies,"color":color}}));
    }
    let bodies: Vec<String> = commands.iter().filter_map(|c| c["params"]["body_name"].as_str().map(str::to_owned)).collect();
    commands.extend(robot_assembly_commands(&bodies));
    json!({"commands":commands})
}

/// Native components and joints: each link owns all of its rigidly connected bodies.
pub fn robot_assembly_commands(bodies: &[String]) -> Vec<Value> {
    let modules = ["base", "axis1", "upper", "forearm", "wrist4", "wrist5", "flange", "tool"];
    let mut commands = Vec::new();
    for module in modules {
        let owned: Vec<&str> = bodies
            .iter()
            .filter(|body| if module == "tool" { is_tool(body) } else { belongs_to_module(module, "gripper", body) })
            .map(String::as_str)
            .collect();
        commands.push(json!({"command":"component.create","params":{"name":module,"parent":0,"activate":false}}));
        commands.push(json!({"command":"component.move_bodies","params":{"component":module,"bodies":owned}}));
    }
    commands.push(json!({"command":"occurrence.ground","params":{"occurrence":"base:1","grounded":true}}));
    for (index, (point, axis, limits)) in [
        ([0., 0., 230.], [0., 0., 1.], [-170., 170.]),
        ([0., 0., 390.], [0., 1., 0.], [-100., 100.]),
        ([300., 0., 975.], [0., 1., 0.], [-135., 135.]),
        ([815., 0., 901.], [1., 0., 0.12], [-180., 180.]),
        ([903., 0., 912.], [0., 1., 0.], [-120., 120.]),
        ([985., 0., 925.], [1., 0., 0.], [-180., 180.]),
    ]
    .into_iter()
    .enumerate()
    {
        commands.push(json!({"command":"joint.as_built","params":{
            "name":format!("J{}", index + 1),"type":"revolute","a":format!("{}:1", modules[index]),"b":format!("{}:1", modules[index + 1]),
            "at":{"occurrence":format!("{}:1", modules[index]),"point":point,"z":axis},"limits":[limits]
        }}));
    }
    commands.push(json!({"command":"joint.as_built","params":{"name":"Tool mount","type":"rigid","a":"flange:1","b":"tool:1"}}));
    commands
}

pub fn apply_tool_visibility(s: &mut crate::Session) {
    let names: Vec<String> = s.model.state().bodies.iter().map(|b| b.name.clone()).collect();
    s.visibility.hidden_bodies.retain(|name| !is_tool(name));
    for name in names {
        let visible = match s.studio.tool.as_str() {
            "welder" => name.starts_with("Schweiss-"),
            "payload" => name.starts_with("Nutzlast-"),
            _ => name.starts_with("Greifer-"),
        };
        if is_tool(&name) && !visible {
            s.visibility.hidden_bodies.push(name);
        }
    }
}

fn is_tool(name: &str) -> bool {
    name.starts_with("Greifer-") || name.starts_with("Schweiss-") || name.starts_with("Nutzlast-")
}
