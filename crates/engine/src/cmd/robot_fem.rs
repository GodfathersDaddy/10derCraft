//! Native-cylinder idealization with assembled cubic Hermite bending elements.
//! Each arm is fixed at its root; all other modules and actuators are rigid.
use crate::{Result, Session, params::bad};
use serde_json::{Value, json};
use solidcraft_doc::{FeatureKind, apply_point, apply_vector, expr::Kind};
use solidcraft_geom::Vec3;
use std::f64::consts::PI;
fn error(s: impl Into<String>) -> crate::EngineError {
    bad("robot.fem.solve", s)
}

/// One bending plane: [v, dv/dx] per node; clamp the first two DOFs.
/// Assemble K, eliminate essential constraints and solve with partial pivoting.
fn bending(length: f64, ei: f64, force: f64, moment: f64, n: usize) -> Result<Vec<[f64; 2]>> {
    if !(1..=32).contains(&n) || !length.is_finite() || length <= 0. || !ei.is_finite() || ei <= 0. {
        return Err(error("invalid beam mesh or rigidity"));
    }
    let h = length / n as f64;
    let nd = 2 * (n + 1);
    let mut k = vec![vec![0.; nd]; nd];
    let local = [
        [12., 6. * h, -12., 6. * h],
        [6. * h, 4. * h * h, -6. * h, 2. * h * h],
        [-12., -6. * h, 12., -6. * h],
        [6. * h, 2. * h * h, -6. * h, 4. * h * h],
    ];
    for e in 0..n {
        for (i, row) in local.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                k[2 * e + i][2 * e + j] += v * ei / h.powi(3);
            }
        }
    }
    let mut rhs = vec![0.; nd];
    rhs[nd - 2] = force;
    rhs[nd - 1] = moment;
    let mut a: Vec<Vec<f64>> = k.iter().skip(2).map(|r| r.iter().skip(2).copied().collect()).collect();
    let mut b = rhs[2..].to_vec();
    let size = b.len();
    for col in 0..size {
        let pivot = (col..size).max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs())).ok_or_else(|| error("empty stiffness matrix"))?;
        a.swap(col, pivot);
        b.swap(col, pivot);
        if !a[col][col].is_finite() || a[col][col].abs() < 1e-18 {
            return Err(error("singular beam stiffness"));
        }
        for row in col + 1..size {
            let factor = a[row][col] / a[col][col];
            let pivot_row = a[col].clone();
            for (target, pivot) in a[row].iter_mut().zip(&pivot_row).skip(col) {
                *target -= factor * pivot;
            }
            b[row] -= factor * b[col];
        }
    }
    let mut u = vec![0.; nd];
    for i in (0..size).rev() {
        let value = (b[i] - (i + 1..size).map(|j| a[i][j] * u[j + 2]).sum::<f64>()) / a[i][i];
        if !value.is_finite() {
            return Err(error("non-finite beam displacement"));
        }
        u[i + 2] = value;
    }
    let residual = (2..nd).map(|i| (k[i].iter().zip(&u).map(|(a, b)| a * b).sum::<f64>() - rhs[i]).abs()).fold(0., f64::max);
    if residual > 1e-5 * (1. + force.abs() + moment.abs()) {
        return Err(error("beam equilibrium residual exceeds tolerance"));
    }
    Ok(u.as_chunks::<2>().0.to_vec())
}

pub(super) fn solve(s: &mut Session, p: &Value) -> Result<Value> {
    let config = super::robot::config(s)?;
    let n = match p.get("elements") {
        None => 8,
        Some(v) => v.as_u64().filter(|v| (1..=32).contains(v)).ok_or_else(|| error("elements must be an integer in 1..32"))? as usize,
    };
    let source = super::mks_simulation::snapshot(s)?;
    let get = |name, kind| super::robot::number(s, name, kind);
    let young = get("E_MPa", Kind::Unitless)?;
    let nu = get("nu", Kind::Unitless)?;
    let load = get("F_TCP_N", Kind::Unitless)?;
    let limit = get("Grenze_TCP", Kind::Length)?;
    if !(1. ..=1e7).contains(&young) || !(-0.99..0.499).contains(&nu) || !load.is_finite() || load.abs() > 1e6 || !(1e-6..=1e6).contains(&limit) {
        return Err(error("invalid elastic, load or deflection inputs"));
    }
    let network = s.execute("mechanics.state", &json!({}))?;
    // Resolve through the same exact native port frame used by the MKS graph.
    let tcp_frame = network["resolved_ports"]
        .as_array()
        .and_then(|v| v.iter().find(|p| p["id"] == "robot-tcp" && p["valid"] == true))
        .ok_or_else(|| error("bound TCP port required"))?;
    let tcp: Vec3 = serde_json::from_value(tcp_frame["frame"]["origin"].clone()).map_err(|e| error(e.to_string()))?;
    let port_z: Vec3 = serde_json::from_value(tcp_frame["frame"]["axes"].get(2).cloned().ok_or_else(|| error("TCP orientation missing"))?)
        .map_err(|e| error(e.to_string()))?;
    let force = port_z * -load;
    let mut reports = Vec::new();
    let mut tcp_displacement = Vec3::ZERO;
    let mut warnings = vec![
        "Linear locked-actuator cylinder idealization; TCP local -Z force only. No gravity, joint/gearbox compliance, shear, contact, solid mesh, local notch stress or dynamic stress recovery.",
    ];
    for (module, name) in [("upper", "Oberarm"), ("forearm", "Unterarm")] {
        let feature = s.doc.features.iter().find(|f| f.name == name && !f.suppressed).ok_or_else(|| error("native arm cylinder missing"))?;
        let FeatureKind::Cylinder { base, axis, radius, height, .. } = &feature.kind else {
            return Err(error("arm must be a native cylinder"));
        };
        let l = s.doc.eval(height, Kind::Length)?;
        let r = s.doc.eval(radius, Kind::Length)?;
        if !(1. ..=10000.).contains(&l) || !(0.1..=1000.).contains(&r) {
            return Err(error("cylinder dimensions outside beam solver range"));
        }
        let coord =
            |i: usize| -> Result<f64> { s.doc.eval(base.get(i).ok_or_else(|| error("cylinder base incomplete"))?, Kind::Length).map_err(Into::into) };
        let origin = Vec3::new(coord(0)?, coord(1)?, coord(2)?);
        let occurrence = config["occurrences"][module].as_u64().ok_or_else(|| error("arm occurrence missing"))?;
        let world = s.occurrence_placement(&[occurrence])?.world;
        let x = apply_vector(&world, axis.0).normalized().ok_or_else(|| error("beam axis invalid"))?;
        let seed = if x.dot(Vec3::Y).abs() < 0.9 { Vec3::Y } else { Vec3::Z };
        let y = (seed - x * x.dot(seed)).normalized().ok_or_else(|| error("beam transverse axis invalid"))?;
        let z = x.cross(y);
        let start = apply_point(&world, origin);
        let end = start + x * l;
        let moment = (tcp - end).cross(force); // N mm
        let local_f = [force.dot(x), force.dot(y), force.dot(z)];
        let local_m = [moment.dot(x), moment.dot(y), moment.dot(z)];
        let area = PI * r * r;
        let inertia = PI * r.powi(4) / 4.;
        let polar = 2. * inertia;
        let shear = young / (2. * (1. + nu));
        let v = bending(l, young * inertia, local_f[1], local_m[2], n)?;
        let w = bending(l, young * inertia, local_f[2], -local_m[1], n)?;
        let ve = v.last().ok_or_else(|| error("missing FE endpoint"))?;
        let we = w.last().ok_or_else(|| error("missing FE endpoint"))?;
        let axial = local_f[0] * l / (young * area);
        let twist = local_m[0] * l / (shear * polar);
        let displacement = x * axial + y * ve[0] + z * we[0];
        let rotation = x * twist - y * we[1] + z * ve[1];
        let contribution = displacement + rotation.cross(tcp - end);
        tcp_displacement += contribution;
        let root_m = (tcp - start).cross(force);
        let bend = (root_m.dot(y).powi(2) + root_m.dot(z).powi(2)).sqrt();
        let stress = local_f[0].abs() / area + bend * r / inertia;
        let nodes: Vec<_> = v
            .iter()
            .zip(&w)
            .enumerate()
            .map(|(i, (v, w))| {
                let t = i as f64 / n as f64;
                json!({"position_mm":start+x*(l*t),"displacement_mm":x*(axial*t)+y*v[0]+z*w[0],"rotation_rad":x*(twist*t)-y*w[1]+z*v[1]})
            })
            .collect();
        if l / (2. * r) < 10. {
            warnings.push("Arm L/diameter <10: Euler-Bernoulli shear omission may be significant; use Timoshenko or solid FEM for design decisions.");
        }
        reports.push(json!({"body":name,"feature":feature.id,"occurrence":occurrence,"idealization":"native cylinder dimensions; all attached geometry rigid, omitted from section stiffness","length_mm":l,"radius_mm":r,"young_mpa":young,"poisson_ratio":nu,"force_world_n":force,"tip_moment_world_nmm":moment,"root_moment_world_nmm":root_m,"tip_displacement_mm":displacement,"tip_rotation_rad":rotation,"tcp_contribution_mm":contribution,"max_normal_stress_mpa":stress,"nodes":nodes,"elements":n,"bending_dofs_per_plane":2*(n+1)}));
    }
    let magnitude = tcp_displacement.len();
    if magnitude > limit {
        warnings.push("TCP deflection exceeds calculation reference limit.");
    }
    if magnitude > 0.02 * 500. {
        warnings.push("Large displacement: linear superposition needs review.");
    }
    let report = json!({"schema":"solidcraft.robot-beam-fem/1","source":source,"method":"assembled cubic Hermite Euler-Bernoulli bending FE, pivoted elimination; axial/torsion closed-form","load_case":{"tcp_world_mm":tcp,"force_world_n":force,"gravity_included":false,"actuators":"locked"},"beams":reports,"tcp_displacement_mm":tcp_displacement,"tcp_deflection_mm":magnitude,"limit_mm":limit,"feasible":magnitude<=limit,"warnings":warnings,"coupling":"quasi-static CAD pose; no flexible feedback into MKS","cad_changed":false});
    s.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, super::robot::FEM_KEY, report.clone())?;
    Ok(report)
}

/// Read-only display model. Only calculated beam displacement fields deform geometry;
/// all other CAD is shown as a translucent reference, never as solved solid FEM.
pub fn result_scene(s: &mut Session, scale: f64, palette: crate::view::Palette) -> Result<(solidcraft_render::Scene, solidcraft_geom::Aabb3, f64)> {
    use solidcraft_geom::Aabb3;
    use solidcraft_render::{Scene, SceneMesh};
    use std::sync::Arc;
    if !scale.is_finite() || !(0. ..=10000.).contains(&scale) {
        return Err(error("display scale must be 0..10000"));
    }
    let state = s.execute("robot.inspect", &json!({}))?;
    if state["fem_current"] != true {
        return Err(error("current robot FEM result required"));
    }
    let model = s.execute("mks.model", &json!({}))?;
    let report = &state["fem"];
    let beams = report["beams"].as_array().ok_or_else(|| error("beam results missing"))?;
    if beams.len() != 2 {
        return Err(error("two arm beam results required"));
    }
    let mut fields = Vec::<(usize, Vec<f64>)>::new();
    let read = |v: &Value| -> Result<Vec3> { serde_json::from_value(v.clone()).map_err(|e| error(e.to_string())) };
    let mut out = Scene { background: Some(palette.background), ..Scene::default() };
    let mut bounds = Aabb3::EMPTY;
    let mut triangles = 0;
    let native = s.model.state();
    for placed in model["blocks"].as_array().into_iter().flatten().flat_map(|b| b["bodies"].as_array().into_iter().flatten()) {
        let Some(body) = native
            .bodies
            .iter()
            .find(|b| Some(b.name.as_str()) == placed["source"].as_str() && Some(s.doc.model_body_component(b)) == placed["component"].as_u64())
        else {
            return Err(error("CAD source missing"));
        };
        let world: solidcraft_doc::Mat = serde_json::from_value(placed["world"].clone()).map_err(|e| error(e.to_string()))?;
        let source = body.mesh();
        triangles += source.triangles.len();
        if triangles > 100_000 {
            return Err(error("FEM picture limited to 100000 triangles"));
        }
        let mut mesh = (*source).clone();
        for p in &mut mesh.positions {
            *p = apply_point(&world, *p);
            bounds.add(*p);
        }
        for n in &mut mesh.normals {
            *n = apply_vector(&world, *n);
        }
        mesh.edges.clear();
        let beam_index =
            beams.iter().position(|b| b["body"] == placed["source"] && placed["path"].as_array().is_some_and(|p| p.last() == Some(&b["occurrence"])));
        if let Some(index) = beam_index {
            let beam = beams.get(index).ok_or_else(|| error("beam missing"))?;
            let nodes = beam["nodes"].as_array().filter(|v| (2..=33).contains(&v.len())).ok_or_else(|| error("beam nodes missing"))?;
            let start = read(&nodes.first().ok_or_else(|| error("beam root missing"))?["position_mm"])?;
            let end = read(&nodes.last().ok_or_else(|| error("beam tip missing"))?["position_mm"])?;
            let axis = (end - start).normalized().ok_or_else(|| error("zero beam length"))?;
            let length = (end - start).len();
            // A CAD cylinder usually has only two axial rings. Render the SAME circular
            // beam section with axial subdivisions so cubic bending is actually visible.
            let radius =
                beam["radius_mm"].as_f64().filter(|r| r.is_finite() && (0.1..=1000.).contains(r)).ok_or_else(|| error("invalid beam radius"))?;
            let seed = if axis.dot(Vec3::Y).abs() < 0.9 { Vec3::Y } else { Vec3::Z };
            let transverse = (seed - axis * seed.dot(axis)).normalized().ok_or_else(|| error("invalid beam frame"))?;
            let binormal = axis.cross(transverse);
            let axial_segments = (nodes.len() - 1) * 4;
            let sides = 48usize;
            mesh = solidcraft_geom::Mesh::default();
            for ring in 0..=axial_segments {
                let centre = start + axis * (length * ring as f64 / axial_segments as f64);
                for j in 0..sides {
                    let angle = 2. * PI * j as f64 / sides as f64;
                    let normal = transverse * angle.cos() + binormal * angle.sin();
                    mesh.positions.push(centre + normal * radius);
                    mesh.normals.push(normal);
                }
            }
            for ring in 0..axial_segments {
                for j in 0..sides {
                    let a = (ring * sides + j) as u32;
                    let b = (ring * sides + (j + 1) % sides) as u32;
                    let c = ((ring + 1) * sides + j) as u32;
                    let d = ((ring + 1) * sides + (j + 1) % sides) as u32;
                    mesh.triangles.extend([[a, b, c], [b, d, c]]);
                }
            }
            for (ring, centre, normal) in [(0, start, -axis), (axial_segments, end, axis)] {
                let centre_id = mesh.positions.len() as u32;
                mesh.positions.push(centre);
                mesh.normals.push(normal);
                for j in 0..sides {
                    let a = (ring * sides + j) as u32;
                    let b = (ring * sides + (j + 1) % sides) as u32;
                    mesh.triangles.push(if ring == 0 { [centre_id, b, a] } else { [centre_id, a, b] });
                }
            }
            mesh.tri_face.resize(mesh.triangles.len(), 0);
            let transform = |p: Vec3| -> Result<(Vec3, Vec3, f64)> {
                let t = ((p - start).dot(axis) / length).clamp(0., 1.) * (nodes.len() - 1) as f64;
                let i = (t.floor() as usize).min(nodes.len() - 2);
                let a = nodes.get(i).ok_or_else(|| error("node missing"))?;
                let b = nodes.get(i + 1).ok_or_else(|| error("node missing"))?;
                let f = t - i as f64;
                let centre = start + axis * ((p - start).dot(axis).clamp(0., length));
                let ua = read(&a["displacement_mm"])?;
                let ub = read(&b["displacement_mm"])?;
                let ra = read(&a["rotation_rad"])?;
                let rb = read(&b["rotation_rad"])?;
                let h = length / (nodes.len() - 1) as f64;
                let linear = ua * (1. - f) + ub * f;
                let h00 = 2. * f.powi(3) - 3. * f * f + 1.;
                let h10 = f.powi(3) - 2. * f * f + f;
                let h01 = -2. * f.powi(3) + 3. * f * f;
                let h11 = f.powi(3) - f * f;
                let cubic = ua * h00 + ra.cross(axis) * (h * h10) + ub * h01 + rb.cross(axis) * (h * h11);
                let slope = ua * ((6. * f * f - 6. * f) / h)
                    + ra.cross(axis) * (3. * f * f - 4. * f + 1.)
                    + ub * ((-6. * f * f + 6. * f) / h)
                    + rb.cross(axis) * (3. * f * f - 2. * f);
                let mut u = cubic - axis * cubic.dot(axis) + axis * linear.dot(axis);
                let mut r = axis.cross(slope) + axis * (ra * (1. - f) + rb * f).dot(axis);
                // Serial upstream elastic motion, as in the solver's TCP superposition.
                for previous in beams.iter().take(index) {
                    let tip = previous["nodes"].as_array().and_then(|v| v.last()).ok_or_else(|| error("previous beam tip missing"))?;
                    let pr = read(&previous["tip_rotation_rad"])?;
                    u += read(&previous["tip_displacement_mm"])? + pr.cross(centre - read(&tip["position_mm"])?);
                    r += pr;
                }
                let displacement = u + r.cross(p - centre);
                Ok((p + displacement * scale, r, displacement.len()))
            };
            let mut field = Vec::with_capacity(mesh.positions.len());
            for (i, p) in mesh.positions.iter_mut().enumerate() {
                let (position, rotation, magnitude) = transform(*p)?;
                field.push(magnitude);
                if let Some(normal) = mesh.normals.get_mut(i) {
                    *normal = (*normal + rotation.cross(*normal) * scale).normalized().unwrap_or(*normal);
                }
                *p = position;
                bounds.add(*p);
            }
            fields.push((out.meshes.len(), field));
            out.meshes.push(SceneMesh::solid(Arc::new(mesh), palette.body_selected));
        } else {
            let mut reference = SceneMesh::solid(Arc::new(mesh), palette.body);
            reference.opacity = 0.22;
            out.meshes.push(reference);
        }
    }
    if bounds.is_empty() {
        return Err(error("no result geometry"));
    }
    if fields.len() != 2 {
        return Err(error("both calculated beam bodies must be present in the CAD assembly"));
    }
    let maximum = fields.iter().flat_map(|(_, values)| values.iter()).copied().fold(0., f64::max);
    for (mesh_index, values) in fields {
        let display = out.meshes.get_mut(mesh_index).ok_or_else(|| error("display field missing"))?;
        let mesh = Arc::make_mut(&mut display.mesh);
        mesh.tri_face = mesh
            .triangles
            .iter()
            .map(|triangle| {
                let value = triangle.iter().filter_map(|i| values.get(*i as usize)).sum::<f64>() / 3.;
                ((value / maximum.max(1e-30) * 31.).round().clamp(0., 31.)) as u32
            })
            .collect();
        display.face_colors = (0..32)
            .map(|band| {
                let t = f64::from(band) / 31.;
                let (a, b, f) = if t < 0.5 { (palette.axes[2], palette.axes[1], t * 2.) } else { (palette.axes[1], palette.axes[0], (t - 0.5) * 2.) };
                let channel = |a: u8, b: u8| (f64::from(a) * (1. - f) + f64::from(b) * f).round() as u8;
                (band, solidcraft_render::Rgb(channel(a.0, b.0), channel(a.1, b.1), channel(a.2, b.2)))
            })
            .collect();
    }
    out.radius = bounds.diagonal().max(1.);
    Ok((out, bounds, maximum))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn component_rom_uses_native_faces_and_rejects_stale_geometry() {
        let mut s = crate::Session::default();
        s.execute("parameters.add", &json!({"name":"L","expression":"1000 mm"})).unwrap();
        s.execute("solid.box", &json!({"length":"L","width":100,"height":100,"body_name":"Arm"})).unwrap();
        let faces = s.execute("model.faces", &json!({"body":"Arm"})).unwrap();
        for (x, name) in [(0., "Root"), (1000., "Tip")] {
            let face = faces["faces"].as_array().unwrap().iter().find(|v| (v["centroid"][0].as_f64().unwrap() - x).abs() < 1e-6).unwrap();
            s.execute("select.set", &json!({"items":[{"type":"face","body":"Arm","index":face["index"],"point":face["point"]}]})).unwrap();
            s.execute("mks.interface.capture", &json!({"name":name,"occurrence":0,"transfer":"frame_wrench","surface":true})).unwrap();
        }
        let before = s.doc.clone();
        s.execute("mks.interface.display", &json!({"enabled":true})).unwrap();
        let (scene, _, count) = crate::cmd::mks_interface_scene(&mut s, crate::view::Palette::DEFAULT).unwrap();
        assert_eq!(count, 4.);
        assert!(scene.meshes.iter().any(|m| m.opacity == 0.12));
        assert!(scene.meshes.iter().any(|m| m.opacity == 1. && m.color == crate::view::Palette::DEFAULT.axes[0]));
        assert_eq!(s.doc, before, "inspection must not alter saved CAD/materials");
        let graph = s.execute("mks.model", &json!({})).unwrap();
        let block = graph["blocks"][0]["id"].clone();
        s.execute("fem.model.create",&json!({"id":"Arm_ROM","block":block,"interfaces":["Root","Tip"],"material":{"young_mpa":210000,"poisson_ratio":0.3,"density_kg_m3":7850}})).unwrap();
        let report = s.execute("fem.model.read", &json!({"id":"Arm_ROM","detail":true})).unwrap();
        assert_eq!(report["models"][0]["current"], true);
        assert_eq!(report["data"]["job"]["mesh"]["shape"]["kind"], "box");
        assert_eq!(report["data"]["job"]["interfaces"][0]["area_mm2"], 10000.);
        assert_eq!(report["data"]["job"]["interfaces"][1]["origin_mm"][0], 1000.);
        s.execute("parameters.change", &json!({"name":"L","expression":"1100 mm"})).unwrap();
        assert_eq!(s.execute("fem.model.read", &json!({})).unwrap()["models"][0]["current"], false);
        assert!(s.execute("fem.rom.reduce", &json!({"id":"Arm_ROM"})).is_err());
    }
    #[test]
    fn bending_matches_force_and_moment_solutions() {
        for n in [1, 2, 8, 32] {
            let (l, ei, f, m) = (1000., 2.1e10, 100., -12000.);
            let u = bending(l, ei, f, m, n).unwrap();
            let end = u.last().unwrap();
            let v = f * l.powi(3) / (3. * ei) + m * l * l / (2. * ei);
            let slope = f * l * l / (2. * ei) + m * l / ei;
            assert!((end[0] - v).abs() < 1e-7);
            assert!((end[1] - slope).abs() < 1e-9);
        }
        assert!(bending(1000., 2.1e10, 100., 0., 0).is_err());
    }
}
