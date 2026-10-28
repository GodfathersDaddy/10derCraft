//! SolidCraft-authored analytic involutes and exact ruled extrusion surfaces.
#![allow(missing_docs)]
use super::*;
use serde::{Deserialize, Serialize};
use std::ops::Bound;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InvoluteCurve {
    pub origin: Point3,
    pub x_axis: Vector3,
    pub y_axis: Vector3,
    pub base_radius: f64,
    pub parameter: [f64; 2],
}
impl ParametricCurve for InvoluteCurve {
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, u: f64) -> Point3 {
        let t = self.parameter[0] + u * (self.parameter[1] - self.parameter[0]);
        let (s, c) = t.sin_cos();
        self.origin + (self.x_axis * (c + t * s) + self.y_axis * (s - t * c)) * self.base_radius
    }
    fn der(&self, u: f64) -> Vector3 {
        let d = self.parameter[1] - self.parameter[0];
        let t = self.parameter[0] + u * d;
        self.base_radius * t * d * (self.x_axis * t.cos() + self.y_axis * t.sin())
    }
    fn der2(&self, u: f64) -> Vector3 {
        let d = self.parameter[1] - self.parameter[0];
        let t = self.parameter[0] + u * d;
        let (s, c) = t.sin_cos();
        self.base_radius * d * d * (self.x_axis * (c - t * s) + self.y_axis * (s + t * c))
    }
    fn parameter_range(&self) -> ParameterRange {
        (Bound::Included(0.), Bound::Included(1.))
    }
}
impl BoundedCurve for InvoluteCurve {}
impl Invertible for InvoluteCurve {
    fn invert(&mut self) {
        self.parameter.swap(0, 1);
    }
    fn inverse(&self) -> Self {
        let mut out = self.clone();
        out.invert();
        out
    }
}
impl Cut for InvoluteCurve {
    fn cut(&mut self, u: f64) -> Self {
        let t = self.parameter[0] + u * (self.parameter[1] - self.parameter[0]);
        let mut out = self.clone();
        out.parameter[0] = t;
        self.parameter[1] = t;
        out
    }
}
impl Transformed<Matrix4> for InvoluteCurve {
    fn transform_by(&mut self, m: Matrix4) {
        self.origin = m.transform_point(self.origin);
        self.x_axis = m.transform_vector(self.x_axis);
        self.y_axis = m.transform_vector(self.y_axis);
    }
    fn transformed(&self, m: Matrix4) -> Self {
        let mut out = self.clone();
        out.transform_by(m);
        out
    }
}
impl ParameterDivision1D for InvoluteCurve {
    type Point = Point3;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Point3>) {
        let d = self.parameter[1] - self.parameter[0];
        let t = range.0.abs().max(range.1.abs()) * d.abs() + self.parameter[0].abs();
        // ||c''|| bounds chord error by M*h^2/8. This is DISPLAY sampling only.
        let m = self.base_radius.abs() * d * d * (1. + t) * (self.x_axis.magnitude() + self.y_axis.magnitude());
        let n = ((range.1 - range.0).abs() * (m / (8. * tol.max(1e-10))).sqrt()).ceil().clamp(1., 8192.) as usize;
        let params: Vec<_> = (0..=n).map(|i| range.0 + (range.1 - range.0) * i as f64 / n as f64).collect();
        let points = params.iter().map(|u| self.subs(*u)).collect();
        (params, points)
    }
}
impl SearchNearestParameter<D1> for InvoluteCurve {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint1D>>(&self, p: Point3, _: H, trials: usize) -> Option<f64> {
        let mut u = (0..=32).map(|i| i as f64 / 32.).min_by(|a, b| (self.subs(*a) - p).magnitude2().total_cmp(&(self.subs(*b) - p).magnitude2()))?;
        for _ in 0..trials.clamp(8, 64) {
            let q = self.subs(u) - p;
            let v = self.der(u);
            let denom = v.magnitude2() + q.dot(self.der2(u));
            if !denom.is_finite() || denom.abs() < 1e-20 {
                break;
            }
            let next = (u - q.dot(v) / denom).clamp(0., 1.);
            if (next - u).abs() < 1e-13 {
                u = next;
                break;
            }
            u = next;
        }
        Some(u)
    }
}
impl SearchParameter<D1> for InvoluteCurve {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint1D>>(&self, p: Point3, h: H, trials: usize) -> Option<f64> {
        let u = self.search_nearest_parameter(p, h, trials)?;
        (self.subs(u) - p).magnitude().lt(&1e-6).then_some(u)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvoluteSurface {
    pub profile: InvoluteCurve,
    pub direction: Vector3,
    pub swapped: bool,
}
impl InvoluteSurface {
    fn uv(&self, u: f64, v: f64) -> (f64, f64) {
        if self.swapped {
            (v, u)
        } else {
            (u, v)
        }
    }
}
impl ParametricSurface for InvoluteSurface {
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, u: f64, v: f64) -> Point3 {
        let (u, v) = self.uv(u, v);
        self.profile.subs(u) + v * self.direction
    }
    fn uder(&self, u: f64, v: f64) -> Vector3 {
        if self.swapped {
            self.direction
        } else {
            self.profile.der(self.uv(u, v).0)
        }
    }
    fn vder(&self, u: f64, v: f64) -> Vector3 {
        if self.swapped {
            self.profile.der(self.uv(u, v).0)
        } else {
            self.direction
        }
    }
    fn uuder(&self, u: f64, v: f64) -> Vector3 {
        if self.swapped {
            Vector3::zero()
        } else {
            self.profile.der2(self.uv(u, v).0)
        }
    }
    fn uvder(&self, _: f64, _: f64) -> Vector3 {
        Vector3::zero()
    }
    fn vvder(&self, u: f64, v: f64) -> Vector3 {
        if self.swapped {
            self.profile.der2(self.uv(u, v).0)
        } else {
            Vector3::zero()
        }
    }
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        let r = (Bound::Included(0.), Bound::Included(1.));
        (r, r)
    }
}
impl ParametricSurface3D for InvoluteSurface {
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        let mut n = self.uder(u, v).cross(self.vder(u, v));
        if n.magnitude2() < 1e-24 {
            let d = self.profile.parameter[1] - self.profile.parameter[0];
            let tangent = self.profile.x_axis * d;
            n = if self.swapped { self.direction.cross(tangent) } else { tangent.cross(self.direction) };
        }
        if n.magnitude2() > 1e-24 {
            n.normalize()
        } else {
            Vector3::zero()
        }
    }
}
impl BoundedSurface for InvoluteSurface {}
impl Invertible for InvoluteSurface {
    fn invert(&mut self) {
        self.swapped = !self.swapped;
    }
    fn inverse(&self) -> Self {
        let mut s = self.clone();
        s.invert();
        s
    }
}
impl Transformed<Matrix4> for InvoluteSurface {
    fn transform_by(&mut self, m: Matrix4) {
        self.profile.transform_by(m);
        self.direction = m.transform_vector(self.direction);
    }
    fn transformed(&self, m: Matrix4) -> Self {
        let mut s = self.clone();
        s.transform_by(m);
        s
    }
}
impl ParameterDivision2D for InvoluteSurface {
    fn parameter_division(&self, r: ((f64, f64), (f64, f64)), tol: f64) -> (Vec<f64>, Vec<f64>) {
        let (a, b) = if self.swapped { (r.1, r.0) } else { r };
        let u = self.profile.parameter_division(a, tol).0;
        let v = vec![b.0, b.1];
        if self.swapped {
            (v, u)
        } else {
            (u, v)
        }
    }
}
impl SearchNearestParameter<D2> for InvoluteSurface {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint2D>>(&self, p: Point3, _: H, trials: usize) -> Option<(f64, f64)> {
        let n = self.profile.x_axis.cross(self.profile.y_axis);
        let d = n.dot(self.direction);
        if d.abs() < 1e-20 {
            return None;
        }
        let v = (p - self.profile.origin).dot(n) / d;
        let u = self.profile.search_nearest_parameter(p - v * self.direction, None, trials)?;
        Some(self.uv(u, v))
    }
}
impl SearchParameter<D2> for InvoluteSurface {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(&self, p: Point3, h: H, trials: usize) -> Option<(f64, f64)> {
        let (u, v) = self.search_nearest_parameter(p, h, trials)?;
        ((self.subs(u, v) - p).magnitude() < 1e-6).then_some((u, v))
    }
}
