//! Exact equation-defined spur gears. Only the display tessellation is sampled.
use crate::body::{p3, v3};
use crate::{Body, KernelError, Result, guard};
use solidcraft_geom::Vec3;
use std::f64::consts::{PI, TAU};
use truck_modeling::{self as mt, InnerSpace, Invertible, ParameterDivision1D, ParametricCurve};

#[derive(Clone, Copy, Debug)]
pub struct InvoluteGear {
    pub center: Vec3,
    pub module: f64,
    pub teeth: usize,
    pub pressure_angle: f64,
    pub width: f64,
    /// Bore radius (external gear) or outside radius (internal gear).
    pub radius: f64,
    pub internal: bool,
    /// Tooth-space phase for an internal gear, tooth-centre phase for external gears.
    pub phase: f64,
    pub backlash: f64,
}

enum Segment {
    Curve(mt::Curve),
    Arc { center: mt::Point3, radius: f64, start: f64, sweep: f64 },
}

/// Inspect the actual B-rep, independent of its display mesh or feature metadata.
pub fn involute_geometry(body: &Body) -> (Vec<solidcraft_geom::brep::Curve>, usize) {
    let mut ids = std::collections::HashSet::new();
    let curves = body
        .solid
        .edge_iter()
        .filter(|e| ids.insert(e.id()))
        .filter_map(|e| {
            let mt::Curve::Involute(c) = e.curve() else {
                return None;
            };
            let p = |v: mt::Vector3| [v.x, v.y, v.z];
            Some(solidcraft_geom::brep::Curve::Involute {
                origin: [c.origin.x, c.origin.y, c.origin.z],
                x_axis: p(c.x_axis),
                y_axis: p(c.y_axis),
                base_radius: c.base_radius,
                parameter: c.parameter,
            })
        })
        .collect();
    let faces = body.solid.face_iter().filter(|f| matches!(f.surface(), mt::Surface::InvoluteExtrusion(_))).count();
    (curves, faces)
}

/// Diagnostic samples of the lowest horizontal cap, including its hole loops.
/// These are measurements of the B-rep, never its stored geometry.
pub fn involute_boundary(body: &Body, tolerance: f64) -> Result<Vec<Vec<Vec3>>> {
    use mt::BoundedCurve;
    if !tolerance.is_finite() || tolerance < 1e-5 {
        return Err(KernelError::Invalid("boundary sample tolerance must be at least 1e-5 mm".into()));
    }
    let face = body
        .solid
        .face_iter()
        .filter_map(|f| match f.surface() {
            mt::Surface::Plane(p) if p.normal().z.abs() > 0.99 => Some((p.origin().z, f)),
            _ => None,
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, f)| f)
        .ok_or_else(|| KernelError::Invalid("no horizontal gear cap".into()))?;
    let mut total = 0usize;
    let mut loops = Vec::new();
    for wire in face.absolute_boundaries() {
        let mut points = Vec::new();
        for edge in wire.edge_iter() {
            let c = edge.oriented_curve();
            let mut samples = c.parameter_division(c.range_tuple(), tolerance).1;
            samples.pop();
            total = total.saturating_add(samples.len());
            if total > 200_000 {
                return Err(KernelError::Invalid("gear boundary sample budget exceeded".into()));
            }
            points.extend(samples.into_iter().map(|p| Vec3::new(p.x, p.y, p.z)));
        }
        loops.push(points);
    }
    Ok(loops)
}
impl Segment {
    fn point(&self, u: f64) -> mt::Point3 {
        match self {
            Self::Curve(c) => c.subs(u),
            Self::Arc { center, radius, start, sweep } => {
                *center + mt::Vector3::new((start + u * sweep).cos(), (start + u * sweep).sin(), 0.) * *radius
            }
        }
    }
}
fn arc(center: mt::Point3, radius: f64, start: f64, end: f64, ccw: bool) -> Segment {
    let mut sweep = (end - start).rem_euclid(TAU);
    if !ccw {
        sweep -= TAU;
    }
    Segment::Arc { center, radius, start, sweep }
}
fn wire(segments: &[Segment]) -> Result<mt::Wire> {
    let vertices: Vec<_> = segments.iter().map(|s| mt::builder::vertex(s.point(0.))).collect();
    let mut edges = Vec::new();
    for (i, s) in segments.iter().enumerate() {
        let (Some(a), Some(b)) = (vertices.get(i), vertices.get((i + 1) % vertices.len())) else {
            return Err(KernelError::Invalid("gear wire".into()));
        };
        if (s.point(1.) - b.point()).magnitude() > 1e-6 {
            return Err(KernelError::Invalid("gear curve endpoints do not meet".into()));
        }
        let edge = match s {
            Segment::Curve(c) => mt::Edge::try_new(a, b, c.clone()).map_err(|e| KernelError::Invalid(e.to_string()))?,
            Segment::Arc { .. } => mt::builder::circle_arc(a, b, s.point(0.5)),
        };
        edges.push(edge);
    }
    Ok(edges.into())
}
fn circle(c: mt::Point3, r: f64) -> Result<mt::Wire> {
    wire(&[arc(c, r, 0., PI, true), arc(c, r, PI, TAU, true)])
}

pub fn involute_gear(g: InvoluteGear) -> Result<Body> {
    if !(g.center.is_finite()
        && (18..=240).contains(&g.teeth)
        && g.module.is_finite()
        && (0.1..=200.).contains(&g.module)
        && g.width.is_finite()
        && g.width > 0.
        && g.width < 1e5
        && g.pressure_angle.is_finite()
        && (15f64.to_radians()..=30f64.to_radians()).contains(&g.pressure_angle)
        && g.radius.is_finite()
        && g.radius > 0.
        && g.phase.is_finite()
        && g.backlash.is_finite()
        && (0. ..=g.module * 0.1).contains(&g.backlash))
    {
        return Err(KernelError::Invalid(
            "involute gear: invalid dimensions, 18..240 teeth, module 0.1..200 mm, pressure angle 15..30 degrees".into(),
        ));
    }
    let rp = g.module * g.teeth as f64 / 2.;
    let rb = rp * g.pressure_angle.cos();
    let root = rp - if g.internal { 1. } else { 1.25 } * g.module;
    let tip = rp + if g.internal { 1.25 } else { 1. } * g.module;
    if g.internal && g.radius <= tip + g.module || !g.internal && g.radius >= root - g.module * 0.2 {
        return Err(KernelError::Invalid("gear bore or rim leaves no root material".into()));
    }
    if (g.teeth as f64) < 2. / g.pressure_angle.sin().powi(2) {
        return Err(KernelError::Invalid("unshifted external geometry would require undercut; increase tooth count or pressure angle".into()));
    }
    let half = PI / (2. * g.teeth as f64) + if g.internal { 1. } else { -1. } * g.backlash / (4. * rp);
    let inv_pitch = g.pressure_angle.tan() - g.pressure_angle;
    let end = ((tip / rb).powi(2) - 1.).sqrt();
    // Below the base circle an involute does not exist. A short radial tangent
    // and a circular root fillet close that non-working region exactly.
    let radial_root = !g.internal && root < rb;
    let fillet = if radial_root { (0.3 * g.module).min((rb * rb - root * root) / (2. * root) * 0.8) } else { 0. };
    let join_radius = if radial_root { (root * root + 2. * root * fillet).sqrt() } else { root };
    let start = if root < rb { 0. } else { ((root / rb).powi(2) - 1.).sqrt() };
    if !(start >= 0. && end > start && half + inv_pitch - (end - end.atan()) > 0.) {
        return Err(KernelError::Invalid("gear has no valid involute flank or tip land".into()));
    }
    guard("analytic involute gear", || {
        let c = p3(g.center);
        let mut segments = Vec::new();
        for k in 0..g.teeth {
            let theta = g.phase + TAU * k as f64 / g.teeth as f64;
            let make = |side: f64| {
                let angle = theta + side * (half + inv_pitch);
                mt::InvoluteCurve {
                    origin: c,
                    x_axis: mt::Vector3::new(angle.cos(), angle.sin(), 0.),
                    y_axis: mt::Vector3::new(angle.sin() * side, -angle.cos() * side, 0.),
                    base_radius: rb,
                    parameter: [start, end],
                }
            };
            let lower = make(-1.);
            let upper = make(1.);
            let foot = |curve: &mt::InvoluteCurve, side: f64| {
                let tangent = curve.x_axis;
                let n = mt::Vector3::new(-tangent.y, tangent.x, 0.);
                let p = c + tangent * join_radius;
                let fc = p + n * (side * fillet);
                let q = c + (fc - c) * (root / (root + fillet));
                (q, fc, p)
            };
            let (l0, lc, l1) = foot(&lower, -1.);
            let (u0, uc, u1) = foot(&upper, 1.);
            if fillet > 0. {
                segments.push(arc(lc, fillet, (l0.y - lc.y).atan2(l0.x - lc.x), (l1.y - lc.y).atan2(l1.x - lc.x), false));
            }
            if radial_root {
                segments.push(Segment::Curve(mt::Curve::Line(mt::Line(l1, lower.subs(0.)))));
            }
            segments.push(Segment::Curve(mt::Curve::Involute(lower.clone())));
            let a = lower.subs(1.);
            let b = upper.subs(1.);
            segments.push(arc(c, tip, (a.y - c.y).atan2(a.x - c.x), (b.y - c.y).atan2(b.x - c.x), true));
            segments.push(Segment::Curve(mt::Curve::Involute(upper.inverse())));
            if radial_root {
                segments.push(Segment::Curve(mt::Curve::Line(mt::Line(upper.subs(0.), u1))));
            }
            if fillet > 0. {
                segments.push(arc(uc, fillet, (u1.y - uc.y).atan2(u1.x - uc.x), (u0.y - uc.y).atan2(u0.x - uc.x), false));
            }
            let from = if fillet > 0. { (u0.y - c.y).atan2(u0.x - c.x) } else { (upper.subs(0.).y - c.y).atan2(upper.subs(0.).x - c.x) };
            let next_lower = if fillet > 0. { (l0.y - c.y).atan2(l0.x - c.x) } else { (lower.subs(0.).y - c.y).atan2(lower.subs(0.).x - c.x) };
            segments.push(arc(c, root, from, next_lower + TAU / g.teeth as f64, true));
        }
        let teeth = wire(&segments)?;
        let boundaries = if g.internal { vec![circle(c, g.radius)?, teeth.inverse()] } else { vec![teeth, circle(c, g.radius)?.inverse()] };
        let plane = mt::Surface::Plane(mt::Plane::new(c, c + mt::Vector3::unit_x(), c + mt::Vector3::unit_y()));
        let face = mt::Face::try_new(boundaries, plane).map_err(|e| KernelError::Invalid(e.to_string()))?;
        let solid = mt::builder::tsweep(&face, v3(Vec3::Z * g.width));
        Body::new(solid)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn involute_gears_keep_equations_after_native_roundtrip() {
        for (teeth, internal) in [(30, false), (21, false), (72, true)] {
            let g = InvoluteGear {
                center: Vec3::ZERO,
                module: 2.,
                teeth,
                pressure_angle: 20f64.to_radians(),
                width: 20.,
                radius: if internal { 84. } else { 6. },
                internal,
                phase: 0.,
                backlash: 0.04,
            };
            let body = involute_gear(g).unwrap();
            assert!(body.unmeshed_faces() == 0);
            let edges: Vec<_> = body
                .solid
                .edge_iter()
                .filter_map(|e| match e.curve() {
                    mt::Curve::Involute(c) => Some(c),
                    _ => None,
                })
                .collect();
            assert!(!edges.is_empty());
            for c in &edges {
                let u = 0.314159;
                let t = c.parameter[0] + u * (c.parameter[1] - c.parameter[0]);
                let expected = c.origin + c.base_radius * (c.x_axis * (t.cos() + t * t.sin()) + c.y_axis * (t.sin() - t * t.cos()));
                assert!((expected - c.subs(u)).magnitude() < 1e-12);
                assert!(mt::Curve::Involute(c.clone()).lift_up().is_err());
            }
            let bytes = crate::encode_native_body(&body).unwrap();
            let loaded = crate::decode_native_body(&bytes).unwrap();
            let n = loaded.solid.edge_iter().filter(|e| matches!(e.curve(), mt::Curve::Involute(_))).count();
            assert_eq!(n, edges.len());
        }
    }
}
