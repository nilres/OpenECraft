//! Line-ups: enclosures bayed side by side, and the cable passages between them –
//! cutouts in the facing side walls, or sides open to each other – through which
//! wires run from one enclosure into the next.
//!
//! Line-up coordinates are enclosure coordinates of the whole line-up: x to the
//! right from the left end, y downwards, z from the rear walls (which are flush).
//! In the room ([`site_poses`]) every line-up stands at its own place, turned about
//! the vertical.

use super::{item_box, route_ends, Enclosure, Face, ItemKind, Layout, PanelItem, Route, WireEnd, DUCT_PROFILE, PLATE_MARGIN, PLATE_Z};
use crate::geom::{p, P};
use crate::model::Project;
use crate::parts::vec3::V3;
use std::collections::HashMap;
pub use crate::model::panel::SitePlace;

/// Gap between line-ups (and single enclosures) placed next to each other.
pub const LINEUP_GAP: f32 = 200.0;
/// Diameter of a new cable passage.
pub const PASSAGE_DIAMETER: f32 = 60.0;
/// Spacing of the places where wires cross between two sides open to each other.
const OPEN_STEP: f32 = 100.0;

/// Reasons why a wire between two enclosures is not routed (reports).
const NOT_BAYED: &str = "enclosures not bayed";
const NO_PASSAGE: &str = "no cable passage between the enclosures";

/// The line-ups of a layout, each from left to right; an enclosure that is bayed to
/// none and has none bayed to it is a line-up of its own. A broken chain (unknown
/// id, two enclosures bayed to the same one, a loop) is cut where it breaks.
pub fn lineups(layout: &Layout) -> Vec<Vec<usize>> {
    let n = layout.enclosures.len();
    let mut right: Vec<Option<usize>> = vec![None; n];
    let mut has_left = vec![false; n];
    for (i, e) in layout.enclosures.iter().enumerate().filter(|(_, e)| e.bayed_to != 0) {
        if let Some(l) = layout.enclosures.iter().position(|x| x.id == e.bayed_to) {
            if l != i && right[l].is_none() {
                right[l] = Some(i);
                has_left[i] = true;
            }
        }
    }
    let mut seen = vec![false; n];
    let mut out = Vec::new();
    // Chains from their left ends first, then what is left of loops.
    for start in (0..n).filter(|&i| !has_left[i]).chain(0..n) {
        if seen[start] {
            continue;
        }
        let mut lu = Vec::new();
        let mut at = Some(start);
        while let Some(i) = at.filter(|&i| !seen[i]) {
            seen[i] = true;
            lu.push(i);
            at = right[i];
        }
        out.push(lu);
    }
    out
}

/// The line-up an enclosure belongs to, from left to right.
pub fn lineup_of(layout: &Layout, ei: usize) -> Vec<usize> {
    lineups(layout).into_iter().find(|lu| lu.contains(&ei)).unwrap_or_default()
}

/// The enclosures standing directly left and right of an enclosure.
pub fn neighbours(layout: &Layout, ei: usize) -> (Option<usize>, Option<usize>) {
    let lu = lineup_of(layout, ei);
    match lu.iter().position(|&i| i == ei) {
        Some(k) => (k.checked_sub(1).map(|k| lu[k]), lu.get(k + 1).copied()),
        None => (None, None),
    }
}

/// Position of every enclosure (its top left rear corner) in line-up coordinates:
/// side by side from the left, rear walls flush, standing on the same floor (plinths
/// included). Line-ups follow each other with [`LINEUP_GAP`].
pub fn positions(layout: &Layout) -> Vec<V3> {
    let mut out = vec![[0.0; 3]; layout.enclosures.len()];
    let mut x = 0.0;
    for (k, lu) in lineups(layout).iter().enumerate() {
        if k > 0 {
            x += LINEUP_GAP;
        }
        let tall = |e: &Enclosure| e.height + e.build.plinth;
        let floor = lu.iter().map(|&i| tall(&layout.enclosures[i])).fold(0.0, f32::max);
        for &i in lu {
            let e = &layout.enclosures[i];
            out[i] = [x, floor - tall(e), 0.0];
            x += e.width;
        }
    }
    out
}

/// Where an enclosure stands in the room (3D view coordinates in mm: x right, y up
/// from the floor, z towards the front): the rear left corner of its bottom and its
/// turn about the vertical in radians (see [`SitePlace::angle`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SitePose {
    pub at: V3,
    pub angle: f32,
}

impl SitePose {
    /// A direction in the enclosure's 3D coordinates (x right, y up, z from the rear
    /// wall) in the room.
    pub fn turn(&self, v: V3) -> V3 {
        let (s, c) = self.angle.sin_cos();
        [v[0] * c + v[2] * s, v[1], -v[0] * s + v[2] * c]
    }

    /// A direction in the room in the enclosure's 3D coordinates.
    pub fn unturn(&self, v: V3) -> V3 {
        let (s, c) = self.angle.sin_cos();
        [v[0] * c - v[2] * s, v[1], v[0] * s + v[2] * c]
    }

    /// A point of the enclosure's 3D coordinates in the room.
    pub fn apply(&self, q: V3) -> V3 {
        let r = self.turn(q);
        [self.at[0] + r[0], self.at[1] + r[1], self.at[2] + r[2]]
    }

    /// A point in the room in the enclosure's 3D coordinates.
    pub fn local(&self, q: V3) -> V3 {
        self.unturn([q[0] - self.at[0], q[1] - self.at[1], q[2] - self.at[2]])
    }
}

/// Where every enclosure stands in the room. A line-up stands at the place set on its
/// left enclosure, otherwise where the plant layout places one of its enclosures (the
/// centre of its footprint on the node), otherwise unturned next to the other
/// line-ups as in [`positions`]. Its enclosures stand side by side on the floor, all
/// raised by the elevation of the left one.
///
/// A line-up in a room ([`super::room_of`]) stands at its place in the room's floor
/// plan, turned with the room; without a place at the rear of the room after the
/// other line-ups of that room. A room on a level of a building stands at its place in
/// the building's floor plan, on the level's floor ([`super::buildings`]).
pub fn site_poses(prj: &Project) -> Vec<SitePose> {
    let layout = &prj.layout;
    let pos = positions(layout);
    let mut out = vec![SitePose::default(); layout.enclosures.len()];
    let lus = lineups(layout);
    let mut along_rear: HashMap<usize, f32> = HashMap::new();
    let mut along_level: HashMap<u64, f32> = HashMap::new();
    // Rooms and the line-ups outside rooms first, then those in rooms.
    for in_room in [false, true] {
        for lu in &lus {
            let Some(&first) = lu.first() else { continue };
            let room = super::room_of(layout, first);
            if room.is_some() != in_room {
                continue;
            }
            let fe = &layout.enclosures[first];
            let lift = fe.elevation.unwrap_or(0.0);
            // In the line-up: x from its left end, the bottom above the floor.
            let local = |i: usize| [pos[i][0] - pos[first][0], lift + layout.enclosures[i].build.plinth, 0.0];
            let pose = match room {
                Some(r) => {
                    let rp = out[r];
                    let place = fe.site.unwrap_or_else(|| {
                        let (lo, _) = super::bounds(&super::outline(&layout.enclosures[r]));
                        let x = along_rear.entry(r).or_insert(lo[0]);
                        let width: f32 = lu.iter().map(|&i| layout.enclosures[i].width).sum();
                        let place = SitePlace { x: *x, z: lo[1], angle: 0.0 };
                        *x += width + LINEUP_GAP;
                        place
                    });
                    SitePose { at: rp.apply([place.x, 0.0, place.z]), angle: rp.angle + place.angle.to_radians() }
                }
                None => match fe.is_room().then(|| super::buildings::find_level(layout, fe.level)).flatten() {
                    // A room on a level of a building: at its place in the building.
                    Some((bi, li)) => {
                        let b = &layout.buildings[bi];
                        let place = fe.site.unwrap_or_else(|| super::buildings::next_place(layout, &mut along_level, first));
                        let bp = super::buildings::building_pose(b);
                        SitePose { at: bp.apply([place.x, b.levels[li].elevation, place.z]), angle: bp.angle + place.angle.to_radians() }
                    }
                    None => {
                        let place = fe.site.or_else(|| plant_place(prj, lu, &local)).unwrap_or(SitePlace { x: pos[first][0], z: 0.0, angle: 0.0 });
                        SitePose { at: [place.x, 0.0, place.z], angle: place.angle.to_radians() }
                    }
                },
            };
            for &i in lu {
                out[i] = SitePose { at: pose.apply(local(i)), angle: pose.angle };
            }
        }
    }
    out
}

fn plant_place(prj: &Project, lu: &[usize], local: &dyn Fn(usize) -> V3) -> Option<SitePlace> {
    lu.iter().find_map(|&i| {
        let e = &prj.layout.enclosures[i];
        let name = e.name.trim_start_matches('+');
        let place = prj.field.places.iter().find(|p| !name.is_empty() && p.tag.trim_start_matches('+') == name)?;
        let n = prj.field.node(place.node)?;
        Some(SitePlace { x: n.x * 1000.0 - local(i)[0] - e.width / 2.0, z: n.y * 1000.0 - e.depth / 2.0, angle: 0.0 })
    })
}

/// Sets the place in the room of the line-up an enclosure belongs to (on its left
/// enclosure); `None` places it automatically again.
pub fn set_site(prj: &mut Project, ei: usize, place: Option<SitePlace>) {
    let lu = lineup_of(&prj.layout, ei);
    for &i in &lu {
        prj.layout.enclosures[i].site = None;
    }
    if let Some(&first) = lu.first() {
        prj.layout.enclosures[first].site = place;
    }
}

fn ensure_id(prj: &mut Project, ei: usize) -> u64 {
    if prj.layout.enclosures[ei].id == 0 {
        let id = prj.new_id().max(1);
        prj.layout.enclosures[ei].id = id;
    }
    prj.layout.enclosures[ei].id
}

/// Takes an enclosure out of its line-up; its neighbours close the gap.
pub fn unbay(prj: &mut Project, ei: usize) {
    if ei >= prj.layout.enclosures.len() {
        return;
    }
    let (l, r) = neighbours(&prj.layout, ei);
    if let Some(r) = r {
        prj.layout.enclosures[r].bayed_to = l.map(|l| prj.layout.enclosures[l].id).unwrap_or(0);
    }
    prj.layout.enclosures[ei].bayed_to = 0;
}

/// Bays an enclosure to the right side of `left`, taking it out of its line-up first;
/// an enclosure that stood right of `left` moves on to its right. False if that is
/// not possible (same enclosure, unknown index, a room).
pub fn bay_right_of(prj: &mut Project, ei: usize, left: usize) -> bool {
    let n = prj.layout.enclosures.len();
    if ei == left || ei >= n || left >= n || prj.layout.enclosures[ei].is_room() || prj.layout.enclosures[left].is_room() {
        return false;
    }
    unbay(prj, ei);
    let (id, left_id) = (ensure_id(prj, ei), ensure_id(prj, left));
    if let Some(r) = neighbours(&prj.layout, left).1 {
        prj.layout.enclosures[r].bayed_to = id;
    }
    prj.layout.enclosures[ei].bayed_to = left_id;
    true
}

/// A place where wires cross from an enclosure into its right neighbour.
#[derive(Clone, Debug, PartialEq)]
pub struct Passage {
    pub left: usize,
    pub right: usize,
    /// Centre in line-up coordinates, on the boundary between the two enclosures.
    pub at: V3,
    /// Half height and half depth of the opening.
    pub half: [f32; 2],
    /// Cutouts in the left enclosure's right wall and in the right enclosure's left
    /// wall; none where the side is open.
    pub cutouts: (Option<u64>, Option<u64>),
}

/// The cable passages between an enclosure and its right neighbour (`pos`: see
/// [`positions`]): cutouts in both facing walls that overlap, a cutout in one wall
/// where the other side is open, and places along the plates where both sides are
/// open.
pub fn passages(layout: &Layout, pos: &[V3], left: usize) -> Vec<Passage> {
    let Some(right) = neighbours(layout, left).1 else { return Vec::new() };
    let (a, b) = (&layout.enclosures[left], &layout.enclosures[right]);
    let (oa, ob) = (pos[left], pos[right]);
    let x = oa[0] + a.width;
    // The side both enclosures have in common.
    let common_y = [oa[1].max(ob[1]), (oa[1] + a.height).min(ob[1] + b.height)];
    let common_z = [0.0, a.depth.min(b.depth)];
    // (cutout, y range, z range) in line-up coordinates
    let openings = |e: &Enclosure, o: V3, face: Face| -> Vec<(u64, [f32; 2], [f32; 2])> {
        e.items
            .iter()
            .filter(|i| i.kind == ItemKind::Cutout && i.face == face)
            .map(|i| {
                let (min, max) = item_box(e, i);
                (i.id, [min[1] + o[1], max[1] + o[1]], [min[2] + o[2], max[2] + o[2]])
            })
            .collect()
    };
    let clip = |r: [f32; 2], to: [f32; 2]| [r[0].max(to[0]), r[1].min(to[1])];
    let mut out = Vec::new();
    let mut push = |y: [f32; 2], z: [f32; 2], cutouts| {
        let (y, z) = (clip(y, common_y), clip(z, common_z));
        if y[0] < y[1] && z[0] < z[1] {
            let half = [(y[1] - y[0]) / 2.0, (z[1] - z[0]) / 2.0];
            out.push(Passage { left, right, at: [x, (y[0] + y[1]) / 2.0, (z[0] + z[1]) / 2.0], half, cutouts });
        }
    };
    let wall_a = (!a.build.right_open).then(|| openings(a, oa, Face::Right));
    let wall_b = (!b.build.left_open).then(|| openings(b, ob, Face::Left));
    match (wall_a, wall_b) {
        (Some(ca), Some(cb)) => {
            for (ia, ya, za) in &ca {
                for (ib, yb, zb) in &cb {
                    push(clip(*ya, *yb), clip(*za, *zb), (Some(*ia), Some(*ib)));
                }
            }
        }
        (Some(ca), None) => ca.into_iter().for_each(|(i, y, z)| push(y, z, (Some(i), None))),
        (None, Some(cb)) => cb.into_iter().for_each(|(i, y, z)| push(y, z, (None, Some(i)))),
        (None, None) => {
            // Open to each other: across anywhere beside the plates, at duct depth (the
            // openings of the places together cover the whole side).
            let (y0, y1) = (common_y[0] + PLATE_MARGIN, common_y[1] - PLATE_MARGIN);
            let z = (PLATE_Z + DUCT_PROFILE.1 / 2.0).min(common_z[1] / 2.0);
            let steps = ((y1 - y0) / OPEN_STEP).floor().max(0.0) as usize;
            for k in 0..=steps {
                let y = if y1 > y0 { y0 + k as f32 * OPEN_STEP } else { (common_y[0] + common_y[1]) / 2.0 };
                push([y - 0.5, y + 0.5], [z - 0.5, z + 0.5], (None, None));
            }
            let half = [OPEN_STEP / 2.0, z.max(common_z[1] - z)];
            out.iter_mut().for_each(|p| p.half = half);
        }
    }
    out
}

/// Where the next cable passage between an enclosure and its right neighbour goes
/// (`y` from the top of the left enclosure, `z` from the rear wall): just in front of
/// the mounting plate, below the passages that are there already.
pub fn next_passage_place(layout: &Layout, left: usize) -> (f32, f32) {
    let e = &layout.enclosures[left];
    let pos = positions(layout);
    let n = passages(layout, &pos, left).iter().filter(|x| x.cutouts != (None, None)).count();
    // The height both enclosures have in common, in the left one's coordinates.
    let (top, bottom) = match neighbours(layout, left).1 {
        Some(r) => ((pos[r][1] - pos[left][1]).max(0.0), (pos[r][1] - pos[left][1] + layout.enclosures[r].height).min(e.height)),
        None => (0.0, e.height),
    };
    let y = (top + PLATE_MARGIN + 100.0 + n as f32 * 2.0 * PASSAGE_DIAMETER).min(bottom - PLATE_MARGIN - PASSAGE_DIAMETER);
    let z = (PLATE_Z + PASSAGE_DIAMETER / 2.0 + 10.0).min(e.depth / 2.0);
    (y, z)
}

/// Adds a round cable passage between an enclosure and its right neighbour: a cutout
/// in each of the facing side walls (none in a side that is open), centred at `y`
/// (from the top of the left enclosure) and `z` (from the rear wall). Returns the
/// ids of the new cutouts; none without a right neighbour.
pub fn add_passage(prj: &mut Project, left: usize, y: f32, z: f32, diameter: f32) -> Vec<u64> {
    let Some(right) = neighbours(&prj.layout, left).1 else { return Vec::new() };
    let pos = positions(&prj.layout);
    let depth = prj.layout.enclosures[left].depth;
    // Face coordinates: the right wall is seen from the right (the front on the left),
    // the left wall from the left (the front on the right).
    let places = [(left, Face::Right, depth - z, y), (right, Face::Left, z, y + pos[left][1] - pos[right][1])];
    let mut ids = Vec::new();
    for (ei, face, fx, fy) in places {
        let e = &prj.layout.enclosures[ei];
        if (face == Face::Left && e.build.left_open) || (face == Face::Right && e.build.right_open) {
            continue;
        }
        let t = e.wall();
        let id = prj.new_id();
        let r = diameter / 2.0;
        let cutout = PanelItem { id, kind: ItemKind::Cutout, round: true, x: fx - r, y: fy - r, w: diameter, h: diameter, z: -t, d: t, face, ..Default::default() };
        prj.layout.enclosures[ei].items.push(cutout);
        ids.push(id);
    }
    ids
}

/// Adds the cutout facing a cutout in a side wall to the neighbour on that side: same
/// size and place in the line-up, so that the two form a cable passage. Returns its
/// id; none if the item is no side wall cutout, there is no neighbour or its side is
/// open.
pub fn add_facing_cutout(prj: &mut Project, ei: usize, id: u64) -> Option<u64> {
    let e = prj.layout.enclosures.get(ei)?;
    let it = e.item(id).filter(|i| i.kind == ItemKind::Cutout)?.clone();
    let (left, right) = neighbours(&prj.layout, ei);
    let (other, face) = match it.face {
        Face::Right => (right?, Face::Left),
        Face::Left => (left?, Face::Right),
        _ => return None,
    };
    // Range from the rear wall: the right wall is seen with the front on the left.
    let (z0, z1) = if it.face == Face::Right { (e.depth - it.x - it.w, e.depth - it.x) } else { (it.x, it.x + it.w) };
    let o = &prj.layout.enclosures[other];
    if (face == Face::Left && o.build.left_open) || (face == Face::Right && o.build.right_open) {
        return None;
    }
    let x = if face == Face::Right { o.depth - z1 } else { z0 };
    let pos = positions(&prj.layout);
    let t = o.wall();
    let y = it.y + pos[ei][1] - pos[other][1];
    let id = prj.new_id();
    let cutout = PanelItem { id, kind: ItemKind::Cutout, round: it.round, x, y, w: it.w, h: it.h, z: -t, d: t, face, ..Default::default() };
    prj.layout.enclosures[other].items.push(cutout);
    Some(id)
}

/// Stretch of a wire in a further enclosure of the line-up.
#[derive(Clone, Debug, PartialEq)]
pub struct Leg {
    pub enclosure: usize,
    pub route: Route,
    /// The cable passage the wire comes in through (line-up coordinates).
    pub passage: V3,
    /// Way from the plate edge of the enclosure before through the passage to the
    /// plate edge of this one.
    pub passage_length: f32,
}

/// The point on the mounting plate nearest to a passage (`at` in line-up coordinates,
/// `o` the enclosure's position) and the way from there to the passage.
fn passage_end(e: &Enclosure, o: V3, at: V3) -> (P, f32) {
    let l = [at[0] - o[0] - PLATE_MARGIN, at[1] - o[1] - PLATE_MARGIN, at[2] - o[2]];
    let (pw, ph) = e.plate_size();
    let q = p(l[0].clamp(0.0, pw), l[1].clamp(0.0, ph));
    (q, (l[0] - q.x).abs() + (l[1] - q.y).abs() + (l[2] - PLATE_Z).abs())
}

/// Routes a wire from an end in one enclosure to an end in another of the same
/// line-up, through the enclosures between them and their cable passages: the
/// passages are chosen so that the whole way is shortest. Returns the route in the
/// first enclosure and the legs in the others, or the reason why there is no way.
pub fn route_across(layout: &Layout, pos: &[V3], (a, from): (usize, WireEnd), (b, to): (usize, WireEnd)) -> Result<(Route, Vec<Leg>), &'static str> {
    let lu = lineup_of(layout, a);
    let (Some(ia), Some(ib)) = (lu.iter().position(|&i| i == a), lu.iter().position(|&i| i == b)) else { return Err(NOT_BAYED) };
    let chain: Vec<usize> = if ia <= ib { lu[ia..=ib].to_vec() } else { lu[ib..=ia].iter().rev().copied().collect() };
    // Passages of each hop between neighbours of the chain.
    let hops: Vec<Vec<V3>> = chain.windows(2).map(|w| passages(layout, pos, if ia <= ib { w[0] } else { w[1] }).into_iter().map(|x| x.at).collect()).collect();
    if hops.is_empty() || hops.iter().any(|h| h.is_empty()) {
        return Err(NO_PASSAGE);
    }
    let end = |k: usize, at: V3| passage_end(&layout.enclosures[chain[k]], pos[chain[k]], at);
    let route = |k: usize, s: WireEnd, t: WireEnd| route_ends(&layout.enclosures[chain[k]], s, t);
    let n = hops.len();
    // Shortest way to each passage of a hop, and the passage of the hop before it.
    let mut cost: Vec<f32> = hops[0]
        .iter()
        .map(|&at| {
            let (q, x) = end(0, at);
            route(0, from, WireEnd::at(q)).plate_length + x
        })
        .collect();
    let mut back: Vec<Vec<usize>> = vec![Vec::new()];
    for h in 1..n {
        let (mut next, mut prev) = (Vec::new(), Vec::new());
        for &at in &hops[h] {
            let (qj, xj) = end(h, at);
            let (i, c) = hops[h - 1]
                .iter()
                .enumerate()
                .map(|(i, &before)| {
                    let (qi, xi) = end(h, before);
                    (i, cost[i] + xi + route(h, WireEnd::at(qi), WireEnd::at(qj)).plate_length + xj)
                })
                .min_by(|x, y| x.1.total_cmp(&y.1))
                .unwrap_or((0, f32::INFINITY));
            next.push(c);
            prev.push(i);
        }
        cost = next;
        back.push(prev);
    }
    let mut j = hops[n - 1]
        .iter()
        .enumerate()
        .map(|(i, &at)| {
            let (q, x) = end(n, at);
            (i, cost[i] + x + route(n, WireEnd::at(q), to).plate_length)
        })
        .min_by(|x, y| x.1.total_cmp(&y.1))
        .map(|x| x.0)
        .unwrap_or(0);
    let mut at = vec![[0.0; 3]; n];
    for h in (0..n).rev() {
        at[h] = hops[h][j];
        if h > 0 {
            j = back[h][j];
        }
    }
    let first = route(0, from, WireEnd::at(end(0, at[0]).0));
    let legs = (1..=n)
        .map(|k| {
            let (q, x) = end(k, at[k - 1]);
            let target = if k == n { to } else { WireEnd::at(end(k, at[k]).0) };
            Leg { enclosure: chain[k], route: route(k, WireEnd::at(q), target), passage: at[k - 1], passage_length: end(k - 1, at[k - 1]).1 + x }
        })
        .collect();
    Ok((first, legs))
}

#[cfg(test)]
mod tests;
