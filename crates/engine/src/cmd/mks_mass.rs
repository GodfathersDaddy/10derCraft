//! Mesh volume moments, rigid placement and density-weighted block aggregation in SI.
use crate::{Result, Session, params::bad};
use serde_json::{Value, json};
use solidcraft_doc::{Mat, apply_point};
use solidcraft_geom::{MassProperties, Mesh, Vec3};
use std::collections::BTreeMap;

#[derive(Clone)]
struct Part {
    mass: f64,
    com: Vec3,
    inertia: [[f64; 3]; 3],
}
struct Integrated {
    properties: MassProperties,
    volume_delta: f64,
    inertia_delta: f64,
    tolerance: f64,
    weld_tolerance: f64,
}
fn integrate(body: &solidcraft_kernel::Body) -> Result<Integrated> {
    let size = body.size();
    if !size.is_finite() || size <= 0. {
        return Err(bad("mks.mass_properties", "invalid body size"));
    }
    let tolerance = (size * 5e-5).max(1e-5);
    let evaluate = |tol| -> Result<(MassProperties, f64)> {
        let mut mesh = body.tessellate(tol)?;
        if mesh.triangles.len() > 1_000_000 || mesh.positions.len() > 2_000_000 {
            return Err(bad("mks.mass_properties", "body exceeds integration mesh limit"));
        }
        // Centre before integration to avoid subtracting huge raw second moments.
        let reference = mesh.bounds().center();
        for p in &mut mesh.positions {
            *p -= reference;
        }
        // Use the native export weld ceiling (size*1e-6), never the much larger
        // chord tolerance. Small translated caps can have seam round-off larger
        // than size*1e-9. Every attempt must still pass strict oriented closure.
        let mut result = None;
        for factor in [1e-9, 1e-8, 1e-7, 1e-6] {
            let eps = (size * factor).max(1e-9);
            let (positions, triangles) = solidcraft_io::weld(&mesh.positions, &mesh.triangles, eps);
            if let Some(mp) = (Mesh { positions, triangles, ..Mesh::default() }).mass_properties() {
                result = Some((mp, eps));
                break;
            }
        }
        let (mut mp, eps) = result.ok_or_else(|| {
            bad("mks.mass_properties", "mesh is open, inconsistently oriented or has no usable volume within bounded native export weld tolerance")
        })?;
        mp.centroid += reference;
        if mp.inertia.iter().enumerate().any(|(i, row)| row.get(i).is_none_or(|v| *v <= 0.)) {
            return Err(bad("mks.mass_properties", "non-positive centroidal inertia"));
        }
        Ok((mp, eps))
    };
    let (coarse, coarse_weld) = evaluate(tolerance * 2.)?;
    let (fine, fine_weld) = evaluate(tolerance)?;
    let volume_delta = (fine.volume - coarse.volume).abs() / fine.volume;
    let scale = fine.inertia.iter().flatten().map(|x| x.abs()).fold(0., f64::max);
    let inertia_delta = fine.inertia.iter().flatten().zip(coarse.inertia.iter().flatten()).map(|(a, b)| (a - b).abs() / scale).fold(0., f64::max);
    if !volume_delta.is_finite() || !inertia_delta.is_finite() || volume_delta > 0.005 || inertia_delta > 0.005 {
        return Err(bad("mks.mass_properties", format!("mesh convergence check failed: volume {volume_delta:.3e}, inertia {inertia_delta:.3e}")));
    }
    Ok(Integrated { properties: fine, volume_delta, inertia_delta, tolerance, weld_tolerance: coarse_weld.max(fine_weld) })
}
fn placed(mp: &MassProperties, density: f64, world: &Mat) -> Result<Part> {
    // Validate rather than silently applying the rigid tensor rule to scale/shear.
    let rows: Vec<Vec<f64>> = (0..4).map(|r| world.iter().map(|c| c.get(r).copied().unwrap_or(f64::NAN)).collect()).collect();
    let rigid = super::component::matrix::rigid(&json!(rows), 1e-8, "mks.mass_properties")?;
    let rho = density * 1e-6; // g/cm^3 -> kg/mm^3.
    let inertia = std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            mp.inertia.iter().enumerate().map(|(a, row)| row.iter().enumerate().map(|(b, x)| rigid[a][i] * x * rigid[b][j]).sum::<f64>()).sum::<f64>()
                * rho
                * 1e-6
        })
    });
    let part = Part { mass: mp.volume * rho, com: apply_point(&rigid, mp.centroid) * 0.001, inertia };
    if !part.mass.is_finite() || part.mass <= 0. || !part.com.is_finite() || part.inertia.iter().flatten().any(|x| !x.is_finite()) {
        return Err(bad("mks.mass_properties", "non-finite physical properties"));
    }
    Ok(part)
}
fn aggregate(parts: &[Part]) -> Result<Value> {
    let mass: f64 = parts.iter().map(|p| p.mass).sum();
    if parts.is_empty() {
        return Ok(json!({"status":"empty","complete":true,"mass_kg":0.,"com_world_m":null,"inertia_com_world_kg_m2":null}));
    }
    let reference = parts.first().map(|p| p.com).unwrap_or(Vec3::ZERO);
    let com = reference + parts.iter().fold(Vec3::ZERO, |sum, p| sum + (p.com - reference) * p.mass) / mass;
    let mut inertia = [[0.; 3]; 3];
    for p in parts {
        let d = p.com - com;
        let dv = [d.x, d.y, d.z];
        for (i, row) in inertia.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell += p.inertia.get(i).and_then(|r| r.get(j)).copied().unwrap_or(0.)
                    + p.mass * (if i == j { d.dot(d) } else { 0. } - dv.get(i).copied().unwrap_or(0.) * dv.get(j).copied().unwrap_or(0.));
            }
        }
    }
    if !mass.is_finite() || !com.is_finite() || inertia.iter().flatten().any(|x| !x.is_finite()) {
        return Err(bad("mks.mass_properties", "block aggregation overflow"));
    }
    Ok(json!({"status":"evaluated","complete":true,"mass_kg":mass,"com_world_m":com,"inertia_com_world_kg_m2":inertia}))
}
pub(super) fn calculate(s: &mut Session, p: &Value) -> Result<Value> {
    if !s.model.param_errors.is_empty() || s.model.results.iter().any(|r| r.error.is_some() || r.stale) {
        return Err(bad(
            "mks.mass_properties",
            "CAD has parameter/feature errors or stale feature geometry; repair and rebuild before integrating mass",
        ));
    }
    let model = super::mks::model(s, p)?;
    let state = s.model.state();
    let doc = s.placed_document();
    let mut cache: BTreeMap<(u64, String), std::result::Result<Integrated, String>> = BTreeMap::new();
    let mut blocks = Vec::new();
    let mut count = 0;
    for block in model["blocks"].as_array().into_iter().flatten() {
        let mut parts = Vec::new();
        let mut bodies = Vec::new();
        let mut errors = Vec::new();
        for b in block["bodies"].as_array().into_iter().flatten() {
            count += 1;
            if count > 1024 {
                return Err(bad("mks.mass_properties", "integration limited to 1024 placed bodies"));
            }
            let name = b["source"].as_str().ok_or_else(|| bad("mks.mass_properties", "missing body name"))?;
            let component = b["component"].as_u64().ok_or_else(|| bad("mks.mass_properties", "missing component"))?;
            let candidates: Vec<_> = state.bodies.iter().filter(|body| body.name == name && doc.model_body_component(body) == component).collect();
            let key = (component, name.to_string());
            if !cache.contains_key(&key)
                && let [body] = candidates.as_slice()
            {
                cache.insert(key, integrate(&body.body).map_err(|e| e.to_string()));
            }
            let outcome = || -> Result<(Part, Value)> {
                let body = match candidates.as_slice() {
                    [body] => *body,
                    _ => return Err(bad("mks.mass_properties", "missing or ambiguous source body")),
                };
                let integrated = cache.get(&(component, name.to_string())).ok_or_else(|| bad("mks.mass_properties", "integration missing"))?;
                let integrated: &std::result::Result<Integrated, String> = integrated;
                let integrated = integrated.as_ref().map_err(|e| bad("mks.mass_properties", e))?;
                let world: Mat = serde_json::from_value(b["world"].clone()).map_err(|e| bad("mks.mass_properties", e.to_string()))?;
                let material = doc.materials.get(&body.name);
                let assumed = material.and_then(|m| doc.material_density(m)).is_none();
                let density = doc.density(&body.name);
                let part = placed(&integrated.properties, density, &world)?;
                let details = json!({"source":name,"component":component,"path":b["path"],"material":material,
                    "density_assumed":assumed,"density_kg_m3":density*1000.,"mass_kg":part.mass,
                    "com_world_m":part.com,"inertia_com_world_kg_m2":part.inertia,
                    "mesh_tolerance_mm":integrated.tolerance,"mesh_weld_tolerance_mm":integrated.weld_tolerance,"relative_volume_change":integrated.volume_delta,
                    "relative_inertia_change":integrated.inertia_delta});
                Ok((part, details))
            };
            match outcome() {
                Ok((part, details)) => {
                    parts.push(part);
                    bodies.push(details);
                }
                Err(e) => errors.push(json!({"source":name,"path":b["path"],"error":e.to_string()})),
            }
        }
        let mut physical = if errors.is_empty() {
            aggregate(&parts)?
        } else {
            json!({"status":"incomplete","complete":false,"mass_kg":null,"com_world_m":null,"inertia_com_world_kg_m2":null})
        };
        physical["id"] = block["id"].clone();
        physical["grounded"] = block["grounded"].clone();
        physical["bodies"] = json!(bodies);
        physical["errors"] = json!(errors);
        blocks.push(physical);
    }
    Ok(json!({"schema":"solidcraft.block-mass-properties/1","blocks":blocks,
        "complete":blocks.iter().all(|b|b["complete"]==true),"dynamic_simulation":false,
        "method":"closed_mesh_tetrahedral_moments_two_resolutions_rigid_transform_parallel_axis",
        "units":{"mass":"kg","position":"m","inertia":"kg*m^2","tensor_axes":"world","tensor_origin":"block_center_of_mass"},
        "assumptions":["Each source body has uniform material density; missing/invalid material uses the explicitly flagged CAD default.",
        "Bodies are additive physical parts. Overlapping solids are not automatically unioned or subtracted.",
        "Two-resolution agreement is an empirical mesh check, not a rigorous error bound or manufacturing certification.",
        "No placeholder mass/inertia is inserted for empty or failed blocks."]}))
}
