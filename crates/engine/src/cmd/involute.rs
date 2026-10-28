use super::CommandSpec;
use crate::params::req_expr;
use crate::{Result, Session};
use serde_json::{Value, json};
use solidcraft_doc::{FeatureKind, point_expr};
use solidcraft_geom::Vec3;
pub static COMMANDS:&[CommandSpec]=&[
    CommandSpec::new("solid.involute_gear","Analytic Involute Gear",create).icon("cylinder").params("module, teeth, pressure_angle, width, radius, phase, backlash: expressions; internal?: bool; center?: [x,y,z]; body_name?; exact function-defined flanks and extrusion surfaces; tooth count rebuilds topology"),
    CommandSpec::new("inspect.involutes","Inspect Analytic Involutes",inspect).icon("params").noundo().doc_only().params("body: body name; returns the actual B-rep equation edges and extrusion face count, not fitted display geometry"),
];
fn create(s: &mut Session, p: &Value) -> Result<Value> {
    let kind = definition(s, p)?;
    super::features::add_feature(s, p, kind)
}
pub(super) fn definition(s: &Session, p: &Value) -> Result<FeatureKind> {
    let cmd = "solid.involute_gear";
    let mut p = p.clone();
    for (k, v) in [("pressure_angle", "20 deg"), ("phase", "0 deg"), ("backlash", "0 mm")] {
        if p.get(k).is_none() {
            p[k] = json!(v);
        }
    }
    if p.get("center").is_none() {
        p["center"] = json!([
            p.get("center_x").cloned().unwrap_or(json!(0)),
            p.get("center_y").cloned().unwrap_or(json!(0)),
            p.get("center_z").cloned().unwrap_or(json!(0))
        ]);
    }
    Ok(FeatureKind::InvoluteGear {
        center: super::features::point_param(s, &p, "center", cmd)?.unwrap_or_else(|| point_expr(Vec3::ZERO)),
        module: req_expr(cmd, &p, "module")?,
        teeth: req_expr(cmd, &p, "teeth")?,
        pressure_angle: req_expr(cmd, &p, "pressure_angle")?,
        width: req_expr(cmd, &p, "width")?,
        radius: req_expr(cmd, &p, "radius")?,
        internal: p["internal"].as_bool().unwrap_or(false),
        phase: req_expr(cmd, &p, "phase")?,
        backlash: req_expr(cmd, &p, "backlash")?,
    })
}
fn inspect(s: &mut Session, p: &Value) -> Result<Value> {
    let name = p["body"].as_str().ok_or_else(|| crate::params::bad("inspect.involutes", "body required"))?;
    let state = s.model.state();
    let body = state.body(name).ok_or_else(|| crate::params::bad("inspect.involutes", "body not found"))?;
    let (curves, faces) = solidcraft_kernel::involute_geometry(&body.body);
    let samples = if let Some(tolerance) = p["boundary_tolerance"].as_f64() {
        Some(solidcraft_kernel::involute_boundary(&body.body, tolerance)?)
    } else {
        None
    };
    Ok(
        json!({"body":name,"curve_count":curves.len(),"extrusion_faces":faces,"curves":curves,"boundary_samples":samples,"equation":"rb*(cos(t)+t*sin(t), sin(t)-t*cos(t))"}),
    )
}
