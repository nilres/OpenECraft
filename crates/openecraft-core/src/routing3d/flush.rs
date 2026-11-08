//! Flush-mounted routing in a room: cables, wires and tubes laid in slots in the walls
//! along the installation zones ([`InstallationZone`], by default those of DIN
//! 18015-3), or in the floor.
//!
//! The middle lines of the zones on the walls (cut where doors and windows are) and the
//! line of the zone in the floor along the walls form a graph, slot deep behind the
//! surfaces: horizontal lines join at the corners of the room, vertical ones lead down
//! into the floor. An end on a wall joins it straight back into the wall and from
//! there up or down to the horizontal zones (an outlet outside the zones is fed from
//! the nearest one) or sideways into a vertical zone it lies in; in the floor it goes
//! straight down the wall. A free end (a cable gland on top of a cabinet) goes to the
//! nearest wall first, one low above the floor straight down into it. The shortest way
//! through the graph is the route.
//!
//! Room coordinates as in [`super::room`]: x and z of the floor plan, y down from the
//! ceiling (the floor at the room's height).

use crate::model::routing3d::{Laying, RoutingRules, ZoneKind};
use crate::model::Project;
use crate::panel::{self, Enclosure, Face, Opening};
use crate::parts::vec3::{dist, V3};
use std::collections::{BinaryHeap, HashMap};

/// An end this close to the surface of a wall lies on it, mm.
const ON_WALL: f32 = 250.0;
/// A free end this low above the floor goes straight down into it; higher ones go to
/// the nearest wall and down it, mm.
const LOW: f32 = 500.0;

/// The zones on one wall of a room, in its face coordinates (x along the wall seen from
/// inside, y down from the ceiling).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WallZones {
    pub horizontal: Vec<ZoneLine>,
    pub vertical: Vec<ZoneLine>,
}

/// The middle line of a zone on a wall: a horizontal one at y `at`, its stretches along
/// x, or a vertical one at x `at`, its stretches along y.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ZoneLine {
    /// Index of the zone in the rules.
    pub zone: usize,
    pub at: f32,
    /// Half the width of the zone.
    pub half: f32,
    pub stretches: Vec<(f32, f32)>,
}

/// The openings (doors, windows) of a wall: their stretch of x and of y.
fn openings(e: &Enclosure, face: Face) -> Vec<(f32, f32, f32, f32, Opening)> {
    e.items
        .iter()
        .filter(|i| i.face == face && i.kind == panel::ItemKind::Cutout && matches!(i.opening, Opening::Door | Opening::Window))
        .map(|i| (i.x, i.x + i.w, i.y, i.y + i.h, i.opening))
        .collect()
}

/// `0..len` without the stretches `cuts`.
fn without(len: f32, mut cuts: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out = Vec::new();
    let mut at = 0.0f32;
    for (a, b) in cuts {
        if a > at + 1.0 {
            out.push((at, a.min(len)));
        }
        at = at.max(b);
    }
    if len > at + 1.0 {
        out.push((at, len));
    }
    out
}

/// The zones switched on on wall `face` of room `e`.
pub fn wall_zones(e: &Enclosure, face: Face, rules: &RoutingRules) -> WallZones {
    let (w, h) = e.face_size(face);
    let holes = openings(e, face);
    let mut out = WallZones::default();
    for (k, z) in rules.zones.iter().enumerate().filter(|(_, z)| z.on && z.to > z.from) {
        let (mid, half) = ((z.from + z.to) / 2.0, (z.to - z.from) / 2.0);
        match z.kind {
            ZoneKind::AboveFloor | ZoneKind::BelowCeiling => {
                let y = if z.kind == ZoneKind::AboveFloor { h - mid } else { mid };
                if y <= 0.0 || y >= h {
                    continue;
                }
                let cuts = holes.iter().filter(|o| o.2 <= y && y <= o.3).map(|o| (o.0, o.1)).collect();
                out.horizontal.push(ZoneLine { zone: k, at: y, half, stretches: without(w, cuts) });
            }
            ZoneKind::BesideCorners | ZoneKind::BesideDoors | ZoneKind::BesideWindows => {
                let xs: Vec<f32> = match z.kind {
                    ZoneKind::BesideCorners => vec![mid, w - mid],
                    kind => {
                        let wanted = if kind == ZoneKind::BesideDoors { Opening::Door } else { Opening::Window };
                        holes.iter().filter(|o| o.4 == wanted).flat_map(|o| [o.0 - mid, o.1 + mid]).collect()
                    }
                };
                for x in xs.into_iter().filter(|x| *x > 0.0 && *x < w) {
                    let cuts = holes.iter().filter(|o| o.0 <= x && x <= o.1).map(|o| (o.2, o.3)).collect();
                    out.vertical.push(ZoneLine { zone: k, at: x, half, stretches: without(h, cuts) });
                }
            }
            ZoneKind::Floor | ZoneKind::Ceiling => {}
        }
    }
    out
}

/// A band of an installation zone to show: the zone and its corners (room coordinates,
/// on the surface of the wall, floor or ceiling).
#[derive(Clone, Debug, PartialEq)]
pub struct ZoneBand {
    pub zone: usize,
    pub corners: [V3; 4],
}

/// Where the lines of the offsets `d` inward of walls `i` and `j` of a room meet in the
/// floor plan.
fn offset_corner(walls: &[panel::Wall], i: usize, j: usize, d: f32) -> [f32; 2] {
    let (a, b) = (&walls[i], &walls[j]);
    let pa = [a.a[0] + a.normal[0] * d, a.a[1] + a.normal[1] * d];
    let pb = [b.a[0] + b.normal[0] * d, b.a[1] + b.normal[1] * d];
    let (da, db) = (a.dir(), b.dir());
    let den = da[0] * db[1] - da[1] * db[0];
    if den.abs() < 1e-6 {
        return pb;
    }
    let t = ((pb[0] - pa[0]) * db[1] - (pb[1] - pa[1]) * db[0]) / den;
    [pa[0] + da[0] * t, pa[1] + da[1] * t]
}

/// The bands of the installation zones of room `ri` switched on in `rules`.
pub fn bands(prj: &Project, ri: usize, rules: &RoutingRules) -> Vec<ZoneBand> {
    let e = &prj.layout.enclosures[ri];
    let walls = panel::walls(e);
    let n = walls.len();
    let mut out = Vec::new();
    for k in 0..n {
        let face = Face::Wall(k as u16);
        let at = |x: f32, y: f32| panel::face_point(e, face, [x, y, 1.0]);
        let zones = wall_zones(e, face, rules);
        for l in &zones.horizontal {
            for &(x0, x1) in &l.stretches {
                out.push(ZoneBand { zone: l.zone, corners: [at(x0, l.at - l.half), at(x1, l.at - l.half), at(x1, l.at + l.half), at(x0, l.at + l.half)] });
            }
        }
        for l in &zones.vertical {
            for &(y0, y1) in &l.stretches {
                out.push(ZoneBand { zone: l.zone, corners: [at(l.at - l.half, y0), at(l.at + l.half, y0), at(l.at + l.half, y1), at(l.at - l.half, y1)] });
            }
        }
    }
    // In the floor and the ceiling: a ring along the walls.
    for (z, zone) in rules.zones.iter().enumerate().filter(|(_, z)| z.on && z.to > z.from && matches!(z.kind, ZoneKind::Floor | ZoneKind::Ceiling)) {
        let y = if zone.kind == ZoneKind::Floor { e.height - 1.0 } else { 1.0 };
        for i in 0..n {
            let (prev, next) = ((i + n - 1) % n, (i + 1) % n);
            let c = |w0: usize, w1: usize, d: f32| {
                let q = offset_corner(&walls, w0, w1, d);
                [q[0], y, q[1]]
            };
            out.push(ZoneBand { zone: z, corners: [c(prev, i, zone.from), c(i, next, zone.from), c(i, next, zone.to), c(prev, i, zone.to)] });
        }
    }
    out
}

/// A straight line of the graph from `p0` to `p1`, with the shares of the way where it
/// has nodes.
struct Line {
    p0: V3,
    p1: V3,
    ts: Vec<f32>,
}

impl Line {
    fn new(p0: V3, p1: V3) -> Line {
        Line { p0, p1, ts: vec![0.0, 1.0] }
    }

    fn at(&self, t: f32) -> V3 {
        [0, 1, 2].map(|k| self.p0[k] + (self.p1[k] - self.p0[k]) * t)
    }
}

#[derive(Default)]
struct Graph {
    nodes: Vec<V3>,
    index: HashMap<[i32; 3], usize>,
    /// Per node: (other node, length, the corners on the way there).
    edges: Vec<Vec<(usize, f32, Vec<V3>)>>,
}

impl Graph {
    fn node(&mut self, q: V3) -> usize {
        let key = q.map(|v| v.round() as i32);
        if let Some(&i) = self.index.get(&key) {
            return i;
        }
        self.nodes.push(q);
        self.edges.push(Vec::new());
        self.index.insert(key, self.nodes.len() - 1);
        self.nodes.len() - 1
    }

    fn edge(&mut self, a: usize, b: usize, via: Vec<V3>) {
        if a == b {
            return;
        }
        let mut len = 0.0;
        let mut prev = self.nodes[a];
        for &q in via.iter().chain([&self.nodes[b]]) {
            len += dist(prev, q);
            prev = q;
        }
        let back: Vec<V3> = via.iter().rev().copied().collect();
        self.edges[a].push((b, len, via));
        self.edges[b].push((a, len, back));
    }

    /// The shortest way from node `a` to node `b`: its corners.
    fn shortest(&self, a: usize, b: usize) -> Option<Vec<V3>> {
        let mut best = vec![f32::INFINITY; self.nodes.len()];
        let mut came: Vec<Option<(usize, usize)>> = vec![None; self.nodes.len()];
        let mut heap = BinaryHeap::new();
        best[a] = 0.0;
        heap.push(std::cmp::Reverse((0u32, a)));
        while let Some(std::cmp::Reverse((bits, i))) = heap.pop() {
            let cost = f32::from_bits(bits);
            if cost > best[i] {
                continue;
            }
            if i == b {
                break;
            }
            for (k, (j, len, _)) in self.edges[i].iter().enumerate() {
                let c = cost + len;
                if c < best[*j] {
                    best[*j] = c;
                    came[*j] = Some((i, k));
                    heap.push(std::cmp::Reverse((c.to_bits(), *j)));
                }
            }
        }
        if !best[b].is_finite() {
            return None;
        }
        let mut pts = vec![self.nodes[b]];
        let mut i = b;
        while let Some((p, k)) = came[i] {
            let (_, _, via) = &self.edges[p][k];
            pts.extend(via.iter().rev());
            pts.push(self.nodes[p]);
            i = p;
        }
        pts.reverse();
        Some(simplify(pts))
    }
}

/// Without repeated points and the points in the middle of a straight stretch.
fn simplify(pts: Vec<V3>) -> Vec<V3> {
    let mut out: Vec<V3> = Vec::new();
    for q in pts {
        if out.last().is_some_and(|p| dist(*p, q) < 0.5) {
            continue;
        }
        if out.len() >= 2 {
            let (a, b) = (out[out.len() - 2], out[out.len() - 1]);
            let (u, v) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [q[0] - b[0], q[1] - b[1], q[2] - b[2]]);
            let cross = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let dot = u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
            if (cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2)).sqrt() < 1e-3 * dist(a, b).max(1.0) * dist(b, q).max(1.0) && dot > 0.0 {
                out.pop();
            }
        }
        out.push(q);
    }
    out
}

/// Where an end lies: on wall `n` at face point (x, y), or free in the room.
enum Place {
    Wall(usize, f32, f32),
    Free,
}

/// The wall an end at `q` (room coordinates) lies on, if it is near one.
fn place_of(e: &Enclosure, walls: &[panel::Wall], q: V3) -> Place {
    let mut best: Option<(f32, usize, f32)> = None;
    for (n, w) in walls.iter().enumerate() {
        let s = w.start();
        let off = (q[0] - w.a[0]) * w.normal[0] + (q[2] - w.a[1]) * w.normal[1];
        let r = w.right();
        let x = (q[0] - s[0]) * r[0] + (q[2] - s[1]) * r[1];
        let len = e.face_size(Face::Wall(n as u16)).0;
        if (-50.0..=ON_WALL).contains(&off) && (-1.0..=len + 1.0).contains(&x) && best.is_none_or(|b| off < b.0) {
            best = Some((off, n, x.clamp(0.0, len)));
        }
    }
    match best {
        Some((_, n, x)) => Place::Wall(n, x, q[1].clamp(0.0, e.height)),
        None => Place::Free,
    }
}

/// The nearest wall to `q` and the place on it (face x) where `q` faces it.
fn nearest_wall(e: &Enclosure, walls: &[panel::Wall], q: V3) -> (usize, f32) {
    let mut best = (f32::INFINITY, 0, 0.0);
    for (n, w) in walls.iter().enumerate() {
        let s = w.start();
        let r = w.right();
        let len = e.face_size(Face::Wall(n as u16)).0;
        let x = ((q[0] - s[0]) * r[0] + (q[2] - s[1]) * r[1]).clamp(0.0, len);
        let p = [s[0] + r[0] * x, s[1] + r[1] * x];
        let d = (p[0] - q[0]).hypot(p[1] - q[2]);
        if d < best.0 {
            best = (d, n, x);
        }
    }
    (best.1, best.2)
}

/// A flush-mounted way through room `ri` from `a` to `b` (room coordinates) for a line
/// of `diameter`: in the walls along their installation zones or (`Laying::Floor`) down
/// into the floor and along its zone; `None` where the zones do not join the ends.
#[allow(clippy::too_many_lines)]
pub fn route(prj: &Project, ri: usize, a: V3, b: V3, laying: Laying, diameter: f32, rules: &RoutingRules) -> Option<Vec<V3>> {
    let e = &prj.layout.enclosures[ri];
    let walls = panel::walls(e);
    let n = walls.len();
    let h = e.height;
    let depth = (rules.slot_depth - diameter / 2.0).max(diameter / 2.0);
    let floor = h + (rules.floor_depth - diameter / 2.0).max(diameter / 2.0);
    let in_floor = laying == Laying::Floor;
    let p = |w: usize, x: f32, y: f32| panel::face_point(e, Face::Wall(w as u16), [x, y, -depth]);
    let zones: Vec<WallZones> = (0..n).map(|w| wall_zones(e, Face::Wall(w as u16), rules)).collect();
    let ring = rules.zones.iter().find(|z| z.on && z.kind == ZoneKind::Floor && z.to > z.from).map(|z| (z.from + z.to) / 2.0);
    if in_floor && ring.is_none() {
        return None;
    }
    // The lines: per wall the horizontal (in the walls only) and the vertical ones, the
    // ring in the floor; with what joins them.
    let mut lines: Vec<Line> = Vec::new();
    // (wall, y) → line, (wall, x) → line, of each stretch.
    let mut h_lines: Vec<(usize, f32, (f32, f32), usize)> = Vec::new();
    let mut v_lines: Vec<(usize, f32, (f32, f32), usize)> = Vec::new();
    for (w, z) in zones.iter().enumerate() {
        if !in_floor {
            for l in &z.horizontal {
                for &(x0, x1) in &l.stretches {
                    lines.push(Line::new(p(w, x0, l.at), p(w, x1, l.at)));
                    h_lines.push((w, l.at, (x0, x1), lines.len() - 1));
                }
            }
        }
        for l in &z.vertical {
            for &(y0, y1) in &l.stretches {
                lines.push(Line::new(p(w, l.at, y0), p(w, l.at, y1)));
                v_lines.push((w, l.at, (y0, y1), lines.len() - 1));
            }
        }
    }
    // The ring in the floor: one line along each wall between the corners.
    let mut ring_lines: Vec<usize> = Vec::new();
    if let (true, Some(m)) = (in_floor, ring) {
        for i in 0..n {
            let (c0, c1) = (offset_corner(&walls, (i + n - 1) % n, i, m), offset_corner(&walls, i, (i + 1) % n, m));
            lines.push(Line::new([c0[0], floor, c0[1]], [c1[0], floor, c1[1]]));
            ring_lines.push(lines.len() - 1);
        }
    }
    let share = |v: f32, (lo, hi): (f32, f32)| (v - lo) / (hi - lo).max(1e-6);
    // Where horizontal and vertical lines of a wall cross.
    for &(w, y, xs, li) in &h_lines {
        for &(wv, x, ys, lv) in &v_lines {
            if w == wv && xs.0 <= x && x <= xs.1 && ys.0 <= y && y <= ys.1 {
                lines[li].ts.push(share(x, xs));
                lines[lv].ts.push(share(y, ys));
            }
        }
    }
    // The point of the ring in front of a place of wall `w` (its line and share).
    let ring_at = |lines: &[Line], w: usize, x: f32| -> Option<(usize, f32, V3)> {
        let m = ring?;
        let li = *ring_lines.get(w)?;
        let s = walls[w].start();
        let r = walls[w].right();
        let q = [s[0] + r[0] * x + walls[w].normal[0] * m, s[1] + r[1] * x + walls[w].normal[1] * m];
        let (p0, p1) = (lines[li].p0, lines[li].p1);
        let d = [p1[0] - p0[0], p1[2] - p0[2]];
        let t = (((q[0] - p0[0]) * d[0] + (q[1] - p0[2]) * d[1]) / (d[0] * d[0] + d[1] * d[1]).max(1e-6)).clamp(0.0, 1.0);
        Some((li, t, lines[li].at(t)))
    };
    let mut extra: Vec<(V3, V3, Vec<V3>)> = Vec::new();
    // The feet of the vertical zones down into the floor.
    if in_floor {
        for &(w, x, ys, li) in &v_lines {
            if ys.1 >= h - 1.0 {
                if let Some((rl, t, rq)) = ring_at(&lines, w, x) {
                    lines[rl].ts.push(t);
                    extra.push((lines[li].p1, rq, vec![p(w, x, floor)]));
                }
            }
        }
    }
    // The corners: horizontal lines of the same height on neighbouring walls.
    for w in 0..n {
        let next = (w + 1) % n;
        let len = e.face_size(Face::Wall(w as u16)).0;
        for &(wa, y, _, _) in h_lines.iter().filter(|l| l.0 == w && l.2 .1 >= len - 1.0) {
            if h_lines.iter().any(|l| l.0 == next && (l.1 - y).abs() < 0.5 && l.2 .0 <= 1.0) {
                let (pa, pb) = (p(wa, len, y), p(next, 0.0, y));
                let c = offset_corner(&walls, wa, next, -depth);
                let corner = [c[0], y, c[1]];
                // Round a corner the walls make outwards, straight across one inwards.
                let dir = walls[wa].dir();
                let outward = (corner[0] - pa[0]) * dir[0] + (corner[2] - pa[2]) * dir[1] >= -0.5;
                extra.push((pa, pb, if outward { vec![corner] } else { Vec::new() }));
            }
        }
    }
    // The ends: on a wall back into it and on to the zones, free ones to the nearest wall
    // or down into the floor.
    let mut ends: Vec<Vec<(V3, Vec<V3>)>> = Vec::new();
    for q in [a, b] {
        let mut ways: Vec<(V3, Vec<V3>)> = Vec::new();
        let (w, x, y, mut lead) = match place_of(e, &walls, q) {
            Place::Wall(w, x, y) => (w, x, y, vec![q]),
            Place::Free if in_floor && q[1] >= h - LOW => {
                // Straight down into the floor, then to the nearest point of the ring.
                let down = [q[0], floor, q[2]];
                let mut best: Option<(f32, usize, f32, V3)> = None;
                for &li in &ring_lines {
                    let (p0, p1) = (lines[li].p0, lines[li].p1);
                    let d = [p1[0] - p0[0], p1[2] - p0[2]];
                    let t = (((down[0] - p0[0]) * d[0] + (down[2] - p0[2]) * d[1]) / (d[0] * d[0] + d[1] * d[1]).max(1e-6)).clamp(0.0, 1.0);
                    let at = lines[li].at(t);
                    if best.is_none_or(|b| dist(at, down) < b.0) {
                        best = Some((dist(at, down), li, t, at));
                    }
                }
                let (_, li, t, at) = best?;
                lines[li].ts.push(t);
                ways.push((at, vec![q, [q[0], h, q[2]], down]));
                ends.push(ways);
                continue;
            }
            Place::Free => {
                let (w, x) = nearest_wall(e, &walls, q);
                let y = q[1].clamp(0.0, h);
                (w, x, y, vec![q])
            }
        };
        let at = p(w, x, y);
        lead.push(at);
        let holes = openings(e, Face::Wall(w as u16));
        let blocked = |y0: f32, y1: f32| holes.iter().any(|o| o.0 < x && x < o.1 && o.2 < y0.max(y1) && y0.min(y1) < o.3);
        if in_floor {
            // Straight down the wall into the floor, to the ring.
            if !blocked(y, h) {
                if let Some((li, t, rq)) = ring_at(&lines, w, x) {
                    lines[li].ts.push(t);
                    let mut via = lead.clone();
                    via.push(p(w, x, h));
                    via.push(p(w, x, floor));
                    ways.push((rq, via));
                }
            }
        } else {
            // Up or down to the horizontal zones of the wall.
            for &(lw, ly, xs, li) in &h_lines {
                if lw == w && xs.0 <= x && x <= xs.1 && !blocked(y, ly) {
                    lines[li].ts.push(share(x, xs));
                    ways.push((p(w, x, ly), lead.clone()));
                }
            }
        }
        // Sideways into a vertical zone it lies in.
        for &(lw, lx, ys, li) in &v_lines {
            let half = zones[lw].vertical.iter().find(|v| (v.at - lx).abs() < 0.5).map_or(0.0, |v| v.half);
            if lw == w && (lx - x).abs() <= half && ys.0 <= y && y <= ys.1 {
                lines[li].ts.push(share(y, ys));
                ways.push((p(w, lx, y), lead.clone()));
            }
        }
        if ways.is_empty() {
            return None;
        }
        ends.push(ways);
    }
    // The graph: the nodes along the lines, joined one after the other.
    let mut g = Graph::default();
    for l in &mut lines {
        l.ts.retain(|t| (-1e-4..=1.0 + 1e-4).contains(t));
        l.ts.sort_by(f32::total_cmp);
        l.ts.dedup_by(|x, y| (*x - *y).abs() < 1e-5);
        let ids: Vec<usize> = l.ts.iter().map(|&t| l.at(t)).collect::<Vec<_>>().into_iter().map(|q| g.node(q)).collect();
        for k in ids.windows(2) {
            g.edge(k[0], k[1], Vec::new());
        }
    }
    for (pa, pb, via) in extra {
        let (ia, ib) = (g.node(pa), g.node(pb));
        g.edge(ia, ib, via);
    }
    // Each end a node of its own, joined to where its ways reach the lines.
    let mut end_nodes = Vec::new();
    for (k, ways) in ends.into_iter().enumerate() {
        let q = [a, b][k];
        // A node of its own: two ends at one place stay two nodes.
        let id = g.nodes.len();
        g.nodes.push(q);
        g.edges.push(Vec::new());
        for (to, via) in ways {
            let t = g.node(to);
            g.edge(id, t, via.into_iter().skip(1).collect());
        }
        end_nodes.push(id);
    }
    g.shortest(end_nodes[0], end_nodes[1])
}

#[cfg(test)]
mod tests;
