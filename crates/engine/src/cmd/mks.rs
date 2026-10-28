//! Read-only reduction of native assembly placements to rigid blocks and joint edges.
//! A structural intermediate model, deliberately not a dynamics solver/export claim.
use super::CommandSpec;
use crate::{Result, Session, params::bad};
use serde_json::{Value, json};
use solidcraft_doc::{
    Mat,
    joints::{self, JointKind},
    mat_mul,
};
use std::collections::{BTreeMap, BTreeSet};

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("mks.mass_properties", "Calculate Block Mass Properties", super::mks_mass::calculate).noundo().params(
        "Read-only material-aware block masses, world COM in m and centroidal world-axis inertia in kg m^2. Closed welded meshes, two tessellation levels; incomplete blocks explicitly rejected. No dynamics simulation.",
    ),
    CommandSpec::new("mks.model", "Derive Assembly Blocks", model).noundo().params(
        "Read-only native CAD rigid blocks, exact occurrence paths, moving joints and project ports. Includes current pending poses; unresolved/ambiguous joints and unsupported couplings are reported. No dynamic integration or inertia calculation.",
    ),
];

fn root(parent: &[usize], mut i: usize) -> usize {
    for _ in 0..parent.len() {
        let next = parent.get(i).copied().unwrap_or(i);
        if next == i {
            break;
        }
        i = next;
    }
    i
}
fn union(parent: &mut [usize], a: usize, b: usize) {
    let a = root(parent, a);
    let b = root(parent, b);
    if let Some(slot) = parent.get_mut(a.max(b)) {
        *slot = a.min(b);
    }
}

pub(super) fn model(s: &mut Session, _: &Value) -> Result<Value> {
    let doc = s.placed_document();
    let placements = super::component::properties::placements(&doc, "mks.model")?;
    if placements.len() > 1024 || doc.assembly.joints.len() > 10_000 {
        return Err(bad("mks.model", "block extraction limit: 1024 placements / 10000 joints"));
    }
    let state = s.model.state();
    let mut paths = BTreeMap::<u64, Vec<usize>>::new();
    for (i, (_, path, _)) in placements.iter().enumerate() {
        paths.entry(path.last().copied().unwrap_or(0)).or_default().push(i);
    }
    let endpoint = |id| -> std::result::Result<usize, String> {
        match paths.get(&id).map(Vec::as_slice) {
            Some([i]) => Ok(*i),
            Some(_) => Err(format!("occurrence {id} has multiple placement paths; native joint has no exact path")),
            None => Err(format!("occurrence {id} has no placement")),
        }
    };
    let mut parent: Vec<_> = (0..placements.len()).collect();
    let mut issues = Vec::<Value>::new();
    // A grounded occurrence is fixed relative to its parent component. Only top-level
    // ground is unambiguously world-fixed; nested cases stay explicit until supported.
    for o in doc.occurrences.iter().filter(|o| o.grounded) {
        match endpoint(o.id) {
            Ok(i) if o.parent == 0 => union(&mut parent, 0, i),
            Ok(_) => issues.push(json!({"kind":"nested_ground","occurrence":o.id,"message":"Nested grounding is not reduced to world ground."})),
            Err(e) => issues.push(json!({"kind":"ambiguous_ground","occurrence":o.id,"message":e})),
        }
    }
    let mut resolved = Vec::new();
    let mut suppressed = Vec::new();
    for j in &doc.assembly.joints {
        if j.suppressed {
            suppressed.push(j.id);
            continue;
        }
        let resolve = || -> std::result::Result<(usize, usize, Mat, Mat), String> {
            let a = endpoint(j.a.occurrence)?;
            let b = endpoint(j.b.occurrence)?;
            // Invalid parameter expressions must not turn into apparently valid stale numbers.
            joints::expressed(&doc, j).map_err(|e| e.to_string())?;
            let fa = joints::resolve_origin(&doc, &state, &j.a).map_err(|e| e.to_string())?;
            let fb = joints::resolve_origin(&doc, &state, &j.b).map_err(|e| e.to_string())?;
            let wa = placements.get(a).ok_or("missing A placement")?.2;
            let wb = placements.get(b).ok_or("missing B placement")?.2;
            Ok((a, b, mat_mul(&wa, &fa), mat_mul(&wb, &fb)))
        };
        match resolve() {
            Ok((a, b, fa, fb)) => {
                if j.kind == JointKind::Rigid {
                    union(&mut parent, a, b);
                }
                resolved.push((j, a, b, fa, fb));
            }
            Err(e) => issues.push(json!({"kind":"unresolved_joint","joint":j.id,"message":e})),
        }
    }
    let mut blocks = BTreeMap::<usize, Value>::new();
    let mut path_block = BTreeMap::new();
    for (i, (component, path, world)) in placements.iter().enumerate() {
        let r = root(&parent, i);
        let id = format!("block_{r}");
        path_block.insert(path.clone(), id.clone());
        let block = blocks.entry(r).or_insert_with(|| json!({"id":id,"grounded":r==root(&parent,0),"members":[],"bodies":[],"ports":[]}));
        if let Some(members) = block["members"].as_array_mut() {
            let name = path.last().and_then(|id| doc.occurrences.iter().find(|o| o.id == *id)).map(|o| o.name.as_str()).unwrap_or("Root");
            members.push(json!({"component":component,"path":path,"world":world,"name":name}));
        }
        for body in state.bodies.iter().filter(|b| doc.model_body_component(b) == *component) {
            let transform = doc.model_body_offset(body).map(|m| mat_mul(world, m)).unwrap_or(*world);
            if let Some(bodies) = block["bodies"].as_array_mut() {
                bodies.push(json!({"source":body.name,"component":component,"path":path,"world":transform}));
            }
        }
    }
    let mut rigid_joints = Vec::new();
    let mut edges = Vec::new();
    let mut connectivity: Vec<_> = (0..placements.len()).collect();
    for (j, a, b, fa, fb) in resolved {
        let a = root(&parent, a);
        let b = root(&parent, b);
        if j.kind == JointKind::Rigid {
            rigid_joints.push(j.id);
            continue;
        }
        let cycle = root(&connectivity, a) == root(&connectivity, b);
        union(&mut connectivity, a, b);
        let relation = j.kind.is_relation();
        if relation {
            issues.push(
                json!({"kind":"geometric_relation","joint":j.id,"message":"Geometric relation retained, not translated to a simulation connector."}),
            );
        }
        if a == b {
            issues.push(json!({"kind":"internal_joint","joint":j.id,"message":"Moving joint lies inside one rigid block; check redundant or contradictory constraints."}));
        }
        let (offset, angle, values) = joints::expressed(&doc, j)?;
        edges.push(json!({"id":j.id,"name":j.name,"a":format!("block_{a}"),"b":format!("block_{b}"),
            "kind":j.kind,"native":j,"frame_a_world":fa,"frame_b_world":fb,
            "values":values,"offset_mm":offset,"angle_rad":angle,"local_dofs":j.kind.dofs(),
            "closes_graph_cycle":cycle,"connector_supported":!relation && a!=b}));
    }
    let ground = root(&connectivity, root(&parent, 0));
    for (i, block) in &mut blocks {
        block["connected_to_ground"] = json!(root(&connectivity, *i) == ground);
    }
    let mut unbound_ports = Vec::new();
    for mut p in super::mechanics::model_ports(s)? {
        let path: Vec<u64> = serde_json::from_value(p["port"]["occurrence"].clone()).map_err(|e| bad("mks.model", e.to_string()))?;
        if p["valid"] == true
            && let Some(id) = path_block.get(&path)
        {
            p["block"] = json!(id);
            if let Some(block) = blocks.values_mut().find(|b| b["id"] == *id)
                && let Some(ports) = block["ports"].as_array_mut()
            {
                ports.push(p);
                continue;
            }
        }
        unbound_ports.push(p);
    }
    if !doc.assembly.links.is_empty() {
        issues.push(json!({"kind":"motion_links","message":"Native motion links retained as source data; no dynamics adapter yet."}));
    }
    if doc.assembly.contact_enabled || doc.assembly.contact_all {
        issues.push(json!({"kind":"contact","message":"Native CAD contact is not transferred to a dynamics contact model."}));
    }
    let floating: BTreeSet<_> = blocks.values().filter(|b| b["connected_to_ground"] == false).filter_map(|b| b["id"].as_str()).collect();
    Ok(json!({"schema":"solidcraft.rigid-blocks/1","blocks":blocks.values().collect::<Vec<_>>(),"joints":edges,
        "rigid_joints":rigid_joints,"suppressed_joints":suppressed,"unbound_ports":unbound_ports,"issues":issues,
        "floating_blocks":floating,"motion_links":doc.assembly.links,
        "units":{"position":"mm","angle":"rad","matrix":"column_major"},
        "dynamic_simulation":false,"inertia_available":false,"source":"current_native_assembly_with_pending_poses",
        "assumptions":["Bodies owned by one component placement are rigid together; child placements remain independent unless constrained.",
        "Graph cycles are topological diagnostics, not a mobility or constraint consistency solution.",
        "World frames reflect the current CAD pose; native flip, angle, offset, limits and expressions are preserved.",
        "No mass/inertia integration, gravity, deformation or solver export is performed."]}))
}
