//! Cached Manhattan routing around actual block rectangles. A blocked wire is explicit.
use egui::{Pos2, Rect, pos2, vec2};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};
pub fn route_channel(ui: &egui::Ui, id: &str, from: Pos2, to: Pos2, blocks: &[Rect], area: Rect, channel: usize) -> Vec<Pos2> {
    let mut h = DefaultHasher::new();
    channel.hash(&mut h);
    for p in [from, to, area.min, area.max].into_iter().chain(blocks.iter().flat_map(|r| [r.min, r.max])) {
        p.x.to_bits().hash(&mut h);
        p.y.to_bits().hash(&mut h);
    }
    let stamp = h.finish();
    let key = ui.id().with(("mks_route", id));
    if let Some((_, path)) = ui.ctx().data(|d| d.get_temp::<(u64, Vec<Pos2>)>(key)).filter(|(v, _)| *v == stamp) {
        return path;
    }
    let result = calculate(from, to, blocks, area, channel);
    ui.ctx().data_mut(|d| d.insert_temp(key, (stamp, result.clone())));
    result
}
/// Keep independent measurements on distinct visible lanes outside the CAD blocks.
pub fn measurement(ui: &egui::Ui, id: &str, from: Pos2, to: Pos2, blocks: &[Rect], area: Rect, track: (usize, f32)) -> Vec<Pos2> {
    let (channel, lane) = track;
    let anchors = [from, pos2(lane, from.y), pos2(lane, to.y), to];
    let mut result = Vec::new();
    for (i, pair) in anchors.windows(2).enumerate() {
        let path = route_channel(ui, &format!("{id}:{i}"), pair[0], pair[1], blocks, area, channel);
        if path.is_empty() {
            return Vec::new();
        }
        for p in path {
            if result.last() != Some(&p) {
                result.push(p);
            }
        }
    }
    result
}
fn blocked(a: Pos2, b: Pos2, rects: &[Rect]) -> bool {
    rects.iter().any(|r| {
        if (a.x - b.x).abs() < 0.001 {
            a.x > r.left() + 0.01 && a.x < r.right() - 0.01 && a.y.min(b.y) < r.bottom() - 0.01 && a.y.max(b.y) > r.top() + 0.01
        } else {
            a.y > r.top() + 0.01 && a.y < r.bottom() - 0.01 && a.x.min(b.x) < r.right() - 0.01 && a.x.max(b.x) > r.left() + 0.01
        }
    })
}
/// Ports leave the nearest face of their block, in either wire direction.
fn outward(point: Pos2, blocks: &[Rect], distance: f32) -> egui::Vec2 {
    let Some(r) = blocks.iter().find(|r| r.expand(0.5).contains(point)) else { return vec2(0., 0.) };
    [
        ((point.x - r.left()).abs(), vec2(-distance, 0.)),
        ((point.x - r.right()).abs(), vec2(distance, 0.)),
        ((point.y - r.top()).abs(), vec2(0., -distance)),
        ((point.y - r.bottom()).abs(), vec2(0., distance)),
    ]
    .into_iter()
    .min_by(|a, b| a.0.total_cmp(&b.0))
    .map_or(vec2(0., 0.), |(_, direction)| direction)
}
fn endpoint(point: Pos2, blocks: &[Rect], distance: f32, padding: f32) -> Pos2 {
    let direction = outward(point, blocks, distance);
    let mut end = point + direction;
    for r in blocks.iter().filter(|r| !r.expand(0.5).contains(point)).map(|r| r.expand(padding + 1.)) {
        if point.y > r.top() && point.y < r.bottom() {
            if direction.x > 0. && r.left() > point.x {
                end.x = end.x.min(r.left());
            }
            if direction.x < 0. && r.right() < point.x {
                end.x = end.x.max(r.right());
            }
        }
        if point.x > r.left() && point.x < r.right() {
            if direction.y > 0. && r.top() > point.y {
                end.y = end.y.min(r.top());
            }
            if direction.y < 0. && r.bottom() < point.y {
                end.y = end.y.max(r.bottom());
            }
        }
    }
    end
}
fn calculate(from: Pos2, to: Pos2, blocks: &[Rect], area: Rect, channel: usize) -> Vec<Pos2> {
    let stub = 24. + (channel % 8) as f32 * 3.;
    let padding = 6. + (channel % 4) as f32 * 2.;
    let start = endpoint(from, blocks, stub, padding);
    let end = endpoint(to, blocks, 24., padding);
    let rects: Vec<_> = blocks.iter().map(|r| r.expand(padding)).collect();
    // End stubs may touch their own block, but must not cut any other block.
    if blocks.iter().any(|r| !r.expand(0.5).contains(from) && blocked(from, start, &[r.expand(padding)]))
        || blocks.iter().any(|r| !r.expand(0.5).contains(to) && blocked(end, to, &[r.expand(padding)]))
    {
        return Vec::new();
    }
    let mut xs = vec![start.x, end.x, area.left() + 12., area.right() - 12.];
    let mut ys = vec![start.y, end.y, area.top() + 12., area.bottom() - 12.];
    for r in &rects {
        xs.extend([r.left(), r.right()]);
        ys.extend([r.top(), r.bottom()]);
    }
    xs.retain(|x| x.is_finite() && *x >= area.left() && *x <= area.right());
    ys.retain(|y| y.is_finite() && *y >= area.top() && *y <= area.bottom());
    xs.sort_by(f32::total_cmp);
    ys.sort_by(f32::total_cmp);
    xs.dedup();
    ys.dedup();
    let n = xs.len() * ys.len();
    if n == 0 || n > 100000 {
        return Vec::new();
    }
    let locate = |p: Pos2| -> Option<usize> { Some(ys.iter().position(|v| *v == p.y)? * xs.len() + xs.iter().position(|v| *v == p.x)?) };
    let (Some(first), Some(last)) = (locate(start), locate(end)) else {
        return Vec::new();
    };
    let point = |i: usize| pos2(xs[i % xs.len()], ys[i / xs.len()]);
    let valid: Vec<bool> = (0..n).map(|i| !rects.iter().any(|r| r.shrink(0.01).contains(point(i)))).collect();
    if !valid[first] || !valid[last] {
        return Vec::new();
    }
    // Direction is part of the search state: penalise bends, not just length.
    let mut costs = vec![u64::MAX; n * 3];
    let mut parents = vec![usize::MAX; n * 3];
    let mut queue = BinaryHeap::new();
    let initial = first * 3;
    costs[initial] = 0;
    queue.push(Reverse((0u64, 0u64, initial)));
    let mut final_state = None;
    while let Some(Reverse((_, distance, state))) = queue.pop() {
        if distance != costs[state] {
            continue;
        }
        let current = state / 3;
        let previous = state % 3;
        if current == last {
            final_state = Some(state);
            break;
        }
        let x = current % xs.len();
        let y = current / xs.len();
        let neighbours = [
            (x > 0).then(|| current - 1),
            (x + 1 < xs.len()).then(|| current + 1),
            (y > 0).then(|| current - xs.len()),
            (y + 1 < ys.len()).then(|| current + xs.len()),
        ];
        for next in neighbours.into_iter().flatten() {
            if !valid[next] || blocked(point(current), point(next), &rects) {
                continue;
            }
            let direction = if next / xs.len() == y { 1 } else { 2 };
            let next_state = next * 3 + direction;
            let delta = point(current) - point(next);
            let length = ((delta.x.abs() + delta.y.abs()) * 100.).round() as u64;
            let bend = if previous != 0 && previous != direction { 1800 } else { 0 };
            let cost = distance.saturating_add(length).saturating_add(bend);
            if cost < costs[next_state] {
                costs[next_state] = cost;
                parents[next_state] = state;
                let d = point(next) - end;
                queue.push(Reverse((cost.saturating_add(((d.x.abs() + d.y.abs()) * 100.).round() as u64), cost, next_state)));
            }
        }
    }
    let Some(mut cursor) = final_state else {
        return Vec::new();
    };
    let mut path = vec![to];
    for _ in 0..n * 3 {
        path.push(point(cursor / 3));
        if cursor == initial {
            break;
        }
        cursor = parents[cursor];
        if cursor == usize::MAX {
            return Vec::new();
        }
    }
    path.push(from);
    path.reverse();
    let mut result: Vec<Pos2> = Vec::new();
    for p in path {
        if result.last() == Some(&p) {
            continue;
        }
        while result.len() >= 2 {
            let a = result[result.len() - 2];
            let b = result[result.len() - 1];
            if ((a.x - b.x).abs() < 0.01 && (b.x - p.x).abs() < 0.01) || ((a.y - b.y).abs() < 0.01 && (b.y - p.y).abs() < 0.01) {
                result.pop();
            } else {
                break;
            }
        }
        result.push(p);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_facing_ports_keep_a_route() {
        let blocks = [Rect::from_min_max(pos2(40., 100.), pos2(140., 200.)), Rect::from_min_max(pos2(160., 100.), pos2(260., 200.))];
        let area = Rect::from_min_max(pos2(0., 0.), pos2(400., 400.));
        for (a, b) in [(pos2(140., 150.), pos2(160., 150.)), (pos2(160., 150.), pos2(140., 150.))] {
            let path = calculate(a, b, &blocks, area, 0);
            assert!(!path.is_empty());
            assert!(path.windows(2).all(|p| !blocked(p[0], p[1], &blocks)));
        }
    }

    #[test]
    fn wires_leave_all_four_port_faces_without_crossing_blocks() {
        let body = Rect::from_min_max(pos2(180., 180.), pos2(280., 280.));
        let area = Rect::from_min_max(pos2(0., 0.), pos2(600., 600.));
        for (port, destination, axis, sign) in [
            (pos2(180., 230.), pos2(40., 80.), 0, -1.),
            (pos2(280., 230.), pos2(500., 80.), 0, 1.),
            (pos2(230., 180.), pos2(500., 40.), 1, -1.),
            (pos2(230., 280.), pos2(500., 500.), 1, 1.),
        ] {
            let path = calculate(port, destination, &[body], area, 0);
            assert!(path.len() >= 2, "missing route for {port:?}");
            assert_eq!(path.first(), Some(&port));
            assert_eq!(path.last(), Some(&destination));
            let first = path[1] - port;
            assert!(first[axis] * sign > 0.);
            for pair in path.windows(2) {
                assert!(pair[0].x == pair[1].x || pair[0].y == pair[1].y);
                assert!(!blocked(pair[0], pair[1], &[body]));
            }
        }
    }

    #[test]
    fn moved_blocks_route_around_an_intervening_body() {
        let blocks = [
            Rect::from_min_max(pos2(40., 200.), pos2(140., 300.)),
            Rect::from_min_max(pos2(420., 200.), pos2(520., 300.)),
            Rect::from_min_max(pos2(240., 100.), pos2(340., 400.)),
        ];
        let area = Rect::from_min_max(pos2(0., 0.), pos2(600., 600.));
        for (from, to) in [(pos2(140., 250.), pos2(420., 250.)), (pos2(420., 250.), pos2(140., 250.))] {
            let path = calculate(from, to, &blocks, area, 1);
            assert!(path.len() >= 4);
            assert!(path.windows(2).all(|p| !blocked(p[0], p[1], &blocks)));
        }
    }
}
