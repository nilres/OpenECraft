//! Control points of wires, cables and tubes ([`RouteControl`]): points added on a route in
//! the panel layout and moved there. The route is searched from its start to the
//! first of them, from one to the next and from the last to its end, so that it
//! keeps its bend radius and stays clear of everything; where that is not possible
//! the route is laid without them and the routing reports why.

use super::grid::{self, Request, Space, Way, PASSED};
use super::{Routing, RouteKind, RoutingRules};
use crate::model::routing3d::{ControlPoint, RouteControl};
use crate::model::Project;
use crate::panel;
use crate::parts::vec3::{add, dist, scale, sub, V3};
use openecraft_i18n::tr_fmt;
use std::collections::HashMap;

/// Why a route could not be led through its control points (the number of the point
/// in the route's order, from 0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Failure {
    /// The point lies in something, or outside the space of its leg.
    Blocked(usize),
    /// No way to the point (from the one before it or the start).
    NoWay(usize),
    /// The ways to and from the points leave no room for the bend radius between them.
    Tight,
    /// The way through the points comes back to itself.
    Itself,
}

/// The control points of each route in layout coordinates (`pos`, [`panel::positions`]),
/// in order, each with the line-up or room it lies in (`lineup_of`, from the enclosure
/// it is relative to); points of enclosures that no longer exist are left out.
pub(super) fn of_routes(prj: &Project, pos: &[V3], lineup_of: &dyn Fn(usize) -> Option<usize>) -> HashMap<String, Vec<(usize, V3)>> {
    let mut out = HashMap::new();
    for c in prj.layout.controls.iter().filter(|c| !c.points.is_empty()) {
        let points = c
            .points
            .iter()
            .filter_map(|p| {
                let k = prj.layout.enclosures.iter().position(|e| e.id == p.enclosure)?;
                Some((lineup_of(k)?, add(p.at, pos[k])))
            })
            .collect();
        out.insert(c.key.clone(), points);
    }
    out
}

/// The way of `req` from `a` to `b` through `space`, led through the control points
/// `vias` in order: each is passed at the centre of its cell, the way runs on out of it
/// in the direction it came in (as far as its bend radius at least), and the whole way
/// keeps its bends apart and does not come back to itself. With the points it passes
/// (the centres of their cells).
pub(super) fn route_through(space: &mut Space, a: V3, b: V3, vias: &[(usize, V3)], req: &Request, rules: &RoutingRules) -> Result<(Way, Vec<V3>), Failure> {
    // On through each point the way it came in; where that finds no way (a point on a
    // corner of the route), out of it any way.
    let mut first = None;
    for straight_on in [true, false] {
        let mut held = Vec::new();
        let found = parts_through(space, a, b, vias, req, rules, straight_on, &mut held);
        for h in held.into_iter().rev() {
            space.give_back(h);
        }
        match found {
            Err(f) if straight_on && !matches!(f, Failure::Blocked(_)) => first = Some(f),
            Err(f) => return Err(first.unwrap_or(f)),
            ok => return ok,
        }
    }
    Err(first.unwrap_or(Failure::NoWay(0)))
}

/// The way of [`route_through`], part by part: each part keeps clear of those before it
/// (their room is taken in `space` for now, `held`; not near the control point the next
/// part leaves, nor near the ends of the route), so that the way does not come back
/// along itself.
#[allow(clippy::too_many_arguments)]
fn parts_through(space: &mut Space, a: V3, b: V3, vias: &[(usize, V3)], req: &Request, rules: &RoutingRules, straight_on: bool, held: &mut Vec<grid::Held>) -> Result<(Way, Vec<V3>), Failure> {
    let zone = space.end_zone(req);
    let at_point = Request { at: [PASSED; 2], ..*req };
    let mut stops = vec![a];
    for &(n, q) in vias {
        stops.push(space.control_cell(q, &at_point).ok_or(Failure::Blocked(n))?);
    }
    stops.push(b);
    let last = stops.len() - 2;
    let mut leave = req.leave[0];
    let mut whole: Option<Way> = None;
    let mut close = 0;
    for i in 0..=last {
        let part = Request {
            at: [if i == 0 { a } else { PASSED }, if i == last { b } else { PASSED }],
            leave: [if i == 0 || straight_on { leave } else { [0.0; 3] }, if i == last { req.leave[1] } else { [0.0; 3] }],
            out: [if i == 0 { req.out[0] } else { 0.0 }, if i == last { req.out[1] } else { 0.0 }],
            straight: [if i == 0 { req.straight[0] } else { 0.0 }, if i == last { req.straight[1] } else { 0.0 }],
            ..*req
        };
        let n = vias.get(i).map_or_else(|| vias.last().map_or(0, |v| v.0), |v| v.0);
        let (way, _) = space.route_with(stops[i], stops[i + 1], &part, rules, None);
        let way = way.ok_or(Failure::NoWay(n))?;
        close += grid::close_bends(&way.points, 2.0 * req.turn);
        leave = arrival(&way.points);
        if i < last {
            let kept = short_of_end(&way.points, zone);
            if kept.len() > 1 {
                held.push(space.hold(&kept, req.half, req.half, &[(a, zone), (b, zone)]));
            }
        }
        whole = Some(match whole {
            None => way,
            Some(mut w) => {
                w.points.extend(way.points.into_iter().skip(1));
                let new: Vec<usize> = way.cells.into_iter().filter(|c| !w.cells.contains(c)).collect();
                w.cells.extend(new);
                w.free += way.free;
                w
            }
        });
    }
    let mut way = whole.ok_or(Failure::NoWay(0))?;
    way.points = grid::simplify(way.points);
    // The bends on both sides of a control point need room for their arcs as those
    // within one way do; the way may not come back to itself through the points.
    if grid::close_bends(&way.points, 2.0 * req.turn) > close {
        return Err(Failure::Tight);
    }
    // (Across itself in a cable duct is no meeting: wires lie across each other there.)
    if grid::meets_itself_where(&way.points, req.turn, 2.0 * req.half + 0.5, 0, &|q| !space.in_channel(q, req)).is_some() {
        return Err(Failure::Itself);
    }
    Ok((way, stops[1..stops.len() - 1].to_vec()))
}

/// The polyline `points` without its last `cut` mm (along it).
fn short_of_end(points: &[V3], cut: f32) -> Vec<V3> {
    let mut left = cut;
    for k in (1..points.len()).rev() {
        let step = dist(points[k - 1], points[k]);
        if step > left {
            let mut kept = points[..k].to_vec();
            kept.push(add(points[k - 1], scale(sub(points[k], points[k - 1]), (step - left) / step)));
            return kept;
        }
        left -= step;
    }
    Vec::new()
}

/// The direction a way arrives at its last point in, along the axis it runs along most
/// (zero for a way of one point).
fn arrival(points: &[V3]) -> V3 {
    let [.., p, q] = points else { return [0.0; 3] };
    let d = sub(*q, *p);
    let k = (0..3).max_by(|&i, &j| d[i].abs().total_cmp(&d[j].abs())).unwrap_or(0);
    let mut v = [0.0; 3];
    if d[k].abs() > 1e-3 {
        v[k] = d[k].signum();
    }
    v
}

/// The message of a failure of route `label` (a `kind`, bend radius `radius`).
pub(super) fn message(f: Failure, kind: RouteKind, label: &str, radius: f32) -> String {
    let r = format!("{radius:.0}");
    match f {
        Failure::Blocked(n) => tr_fmt!("Control point {} of {} {} lies inside something or outside the enclosure: it is laid without its control points. Move the control point into free space.", n + 1, kind.title(), label),
        Failure::NoWay(n) => tr_fmt!("{} {} cannot be routed through its control point {}: no way that keeps half its thickness clear of everything and bends with its radius of {} mm. It is laid without its control points; move the control point.", kind.title(), label, n + 1, r),
        Failure::Tight => tr_fmt!("{} {}: its control points leave no room to bend with its radius of {} mm. It is laid without its control points; move them further apart.", kind.title(), label, r),
        Failure::Itself => tr_fmt!("{} {} runs into itself through its control points. It is laid without its control points; move them.", kind.title(), label),
    }
}

// ---------------------------------------------------------------------- editing

/// The control points of route `key` in layout coordinates, in order.
pub fn control_points(prj: &Project, key: &str) -> Vec<V3> {
    let pos = panel::positions(&prj.layout);
    let Some(c) = prj.layout.controls.iter().find(|c| c.key == key) else { return Vec::new() };
    c.points.iter().filter_map(|p| prj.layout.enclosures.iter().position(|e| e.id == p.enclosure).map(|k| add(p.at, pos[k]))).collect()
}

/// Where a point comes nearest to a route ([`along`]).
type Nearest = (f32, V3, usize, f32, (V3, V3));

/// Where along the stretches of a route (each with the enclosure it starts in) a point
/// comes nearest to it: how far along the whole route, the point on it, the enclosure of
/// that stretch, the distance and the stretch's segment.
fn along(stretches: &[(usize, &[V3])], q: V3) -> Option<Nearest> {
    let mut best: Option<Nearest> = None;
    let mut s0 = 0.0;
    for &(k, pts) in stretches {
        for seg in pts.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            let ab = sub(b, a);
            let l2 = crate::parts::vec3::dot(ab, ab);
            let t = if l2 <= f32::EPSILON { 0.0 } else { (crate::parts::vec3::dot(sub(q, a), ab) / l2).clamp(0.0, 1.0) };
            let on = add(a, crate::parts::vec3::scale(ab, t));
            let d = dist(q, on);
            if best.is_none_or(|b| d < b.3) {
                best = Some((s0 + t * l2.sqrt(), on, k, d, (a, b)));
            }
            s0 += l2.sqrt();
        }
    }
    best
}

/// The enclosure of the line-up (or room) of enclosure `k` that point `q` (layout
/// coordinates) lies in, `k` where none takes it.
fn enclosure_at(prj: &Project, pos: &[V3], k: usize, q: V3) -> usize {
    let lineup = panel::lineup_of(&prj.layout, k);
    lineup.into_iter().find(|&i| q[0] >= pos[i][0] - 0.5 && q[0] <= pos[i][0] + prj.layout.enclosures[i].width + 0.5).unwrap_or(k)
}

/// The routed wire, cable or tube `key` of `routing`.
fn routed<'a>(routing: &'a Routing, key: &str) -> Result<&'a super::RoutedItem, String> {
    routing.item(key).filter(|i| i.routed()).ok_or_else(|| tr_fmt!("No routed wire, cable or tube {}", key))
}

/// Adds a control point to the route `key` of `routing` where the route runs nearest
/// to `at` (layout coordinates), between the control points before and after it
/// there. The route is no longer frozen. Returns the number of the new point (from 0).
pub fn add_control(prj: &mut Project, routing: &Routing, key: &str, at: V3) -> Result<usize, String> {
    let it = routed(routing, key)?;
    let pos = panel::positions(&prj.layout);
    let first = it.enclosures.first().copied().ok_or_else(|| tr_fmt!("No routed wire, cable or tube {}", key))?;
    let stretches: Vec<(usize, &[V3])> = it.stretches().into_iter().map(|(leg, pts)| (leg.unwrap_or(first), pts)).collect();
    let (s, q, k, _, (a, b)) = along(&stretches, at).ok_or_else(|| tr_fmt!("No routed wire, cable or tube {}", key))?;
    // Not on a corner of the route (or at its end): there it has no room for the arcs
    // of its bends; back along the segment by the bend radius, at most to its middle.
    let length = dist(a, b);
    let q = if length > 1e-3 {
        let least = (it.bend_radius.max(1.0) / length).min(0.5);
        add(a, scale(sub(b, a), (dist(a, q) / length).clamp(least, 1.0 - least)))
    } else {
        q
    };
    let k = enclosure_at(prj, &pos, k, q);
    // Where the points it has lie along the route (where it passes them).
    let before = control_points(prj, key);
    let index = before.iter().enumerate().filter(|(i, p)| it.controls.get(*i).copied().or(Some(**p)).and_then(|p| along(&stretches, p)).is_some_and(|b| b.0 < s)).count();
    let point = ControlPoint { enclosure: prj.layout.enclosures[k].id, at: sub(q, pos[k]) };
    match prj.layout.controls.iter_mut().find(|c| c.key == key) {
        Some(c) => c.points.insert(index.min(c.points.len()), point),
        None => prj.layout.controls.push(RouteControl { key: key.to_string(), points: vec![point] }),
    }
    prj.layout.controls.sort_by(|a, b| a.key.cmp(&b.key));
    prj.layout.frozen.retain(|f| f.key != key);
    Ok(index)
}

/// Moves control point `index` of route `key` to `at` (layout coordinates; it stays
/// in the line-up or room it was in, in the enclosure of it there).
pub fn move_control(prj: &mut Project, key: &str, index: usize, at: V3) -> Result<(), String> {
    let pos = panel::positions(&prj.layout);
    let encs = &prj.layout.enclosures;
    let c = prj.layout.controls.iter().find(|c| c.key == key).and_then(|c| c.points.get(index)).ok_or_else(|| tr_fmt!("Route {} has no control point {}", key, index + 1))?;
    let k = encs.iter().position(|e| e.id == c.enclosure).ok_or_else(|| tr_fmt!("Route {} has no control point {}", key, index + 1))?;
    let k = enclosure_at(prj, &pos, k, at);
    let point = ControlPoint { enclosure: prj.layout.enclosures[k].id, at: sub(at, pos[k]) };
    if let Some(p) = prj.layout.controls.iter_mut().find(|c| c.key == key).and_then(|c| c.points.get_mut(index)) {
        *p = point;
    }
    Ok(())
}

/// Removes control point `index` of route `key` (all of them for `None`). Returns how
/// many were removed.
pub fn remove_control(prj: &mut Project, key: &str, index: Option<usize>) -> Result<usize, String> {
    let Some(ci) = prj.layout.controls.iter().position(|c| c.key == key) else {
        return match index {
            Some(i) => Err(tr_fmt!("Route {} has no control point {}", key, i + 1)),
            None => Ok(0),
        };
    };
    let c = &mut prj.layout.controls[ci];
    let n = match index {
        Some(i) if i < c.points.len() => {
            c.points.remove(i);
            1
        }
        Some(i) => return Err(tr_fmt!("Route {} has no control point {}", key, i + 1)),
        None => std::mem::take(&mut c.points).len(),
    };
    if c.points.is_empty() {
        prj.layout.controls.remove(ci);
    }
    Ok(n)
}

#[cfg(test)]
mod tests;
