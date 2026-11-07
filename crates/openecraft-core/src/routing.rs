//! Routing of connections: orthogonal paths on the grid around symbols and other
//! wires, and automatic connections between connection points facing each other.

use crate::geom::{dist_to_segment, on_segment_interior, p, P, R, GRID};
use crate::symlib::Symbols;
use crate::model::*;
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap, HashMap};

/// Connection points of the symbols on a page with their outward direction, and the
/// free arms of its connection symbols (T-junctions, corners) as points facing their
/// direction.
fn pins(symbols: &Symbols, page: &Page) -> Vec<(u64, P, P)> {
    let mut v: Vec<(u64, P, P)> = crate::netlist::junctions::free_arms(page).into_iter().map(|(id, q, d)| (id, q, d.vector())).collect();
    for s in &page.symbols {
        let Some(def) = symbols.get(&s.sym) else { continue };
        let c = def.world_bounds(s.pos, s.rot, s.mirror).center();
        for i in 0..def.pins.len() {
            let q = def.pin_world(i, s.pos, s.rot, s.mirror);
            let d = q - c;
            let dir = if d.x.abs() < 1e-3 && d.y.abs() < 1e-3 {
                p(0.0, 0.0)
            } else if d.y.abs() >= d.x.abs() {
                p(0.0, d.y.signum())
            } else {
                p(d.x.signum(), 0.0)
            };
            v.push((s.id, q, dir));
        }
    }
    v
}

/// A connection point is in use when a wire ends there or another pin touches it. The
/// arm of a connection symbol is in use when a wire leaves it in its direction.
fn pin_used(page: &Page, all: &[(u64, P, P)], sym: u64, q: P, dir: P) -> bool {
    if page.junctions.iter().any(|j| j.id == sym) {
        return page.wires.iter().any(|w| {
            let other = if w.a.same(q) { Some(w.b) } else if w.b.same(q) { Some(w.a) } else { None };
            let parallel = |d: P| (d.x * dir.y - d.y * dir.x).abs() < 0.01;
            other.is_some_and(|o| parallel(o - q) && (o.x - q.x) * dir.x + (o.y - q.y) * dir.y > 0.0) || (on_segment_interior(q, w.a, w.b) && parallel(w.b - w.a))
        }) || all.iter().any(|(id, o, _)| *id != sym && o.same(q));
    }
    page.wires.iter().any(|w| w.a.same(q) || w.b.same(q) || on_segment_interior(q, w.a, w.b))
        || all.iter().any(|(id, o, _)| *id != sym && o.same(q))
}

/// Can a wire run from `a` to `b` without touching anything but its end points?
fn clear(symbols: &Symbols, page: &Page, all: &[(u64, P, P)], a: P, b: P, ends: [u64; 2]) -> bool {
    let inner = |q: P| on_segment_interior(q, a, b);
    if all.iter().any(|(_, q, _)| inner(*q)) {
        return false;
    }
    for w in &page.wires {
        if inner(w.a) || inner(w.b) || on_segment_interior(a, w.a, w.b) || on_segment_interior(b, w.a, w.b) {
            return false;
        }
    }
    for s in page.symbols.iter().filter(|s| !ends.contains(&s.id)) {
        let Some(def) = symbols.get(&s.sym) else { continue };
        let r = def.world_bounds(s.pos, s.rot, s.mirror);
        let mid = p((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
        // Axis aligned segment against the box.
        let seg = R::from_points(a, b);
        let overlaps = seg.min.x <= r.max.x && seg.max.x >= r.min.x && seg.min.y <= r.max.y && seg.max.y >= r.min.y;
        if overlaps || r.contains(mid) {
            return false;
        }
    }
    true
}

/// Wires that connect free connection points of the selected symbols (and free arms
/// of the selected connection symbols) with free connection points or arms lying
/// straight ahead and facing them.
pub fn autoconnect(symbols: &Symbols, page: &Page, sel: &BTreeSet<u64>, max_len: f32) -> Vec<(P, P)> {
    let all = pins(symbols, page);
    let mut out: Vec<(P, P)> = Vec::new();
    // (position, direction) of the points wired already.
    let mut taken: Vec<(P, P)> = Vec::new();
    let is_taken = |taken: &[(P, P)], q: P, dir: P| taken.iter().any(|(t, td)| t.same(q) && (*td == dir || td.len() == 0.0));
    for (sym, q, dir) in all.iter().filter(|(id, _, _)| sel.contains(id)) {
        if dir.len() == 0.0 || is_taken(&taken, *q, *dir) || pin_used(page, &all, *sym, *q, *dir) {
            continue;
        }
        let mut best: Option<(f32, u64, P, P)> = None;
        for (osym, o, odir) in &all {
            if osym == sym || !(odir.x == -dir.x && odir.y == -dir.y) || is_taken(&taken, *o, *odir) {
                continue;
            }
            let d = *o - *q;
            let along = d.x * dir.x + d.y * dir.y;
            let across = (d.x * dir.y - d.y * dir.x).abs();
            if across > 0.01 || along <= 0.01 || along > max_len || best.is_some_and(|b| b.0 <= along) {
                continue;
            }
            if pin_used(page, &all, *osym, *o, *odir) {
                continue;
            }
            best = Some((along, *osym, *o, *odir));
        }
        let Some((_, osym, o, odir)) = best else { continue };
        let is_arm = |id: u64| page.junctions.iter().any(|j| j.id == id);
        // A pin gets one wire; the arms of a connection symbol share its point.
        if clear(symbols, page, &all, *q, o, [*sym, osym]) && (is_arm(*sym) || !out.iter().any(|(a, b)| dist_to_segment(*q, *a, *b) < 0.01)) {
            out.push((*q, o));
            // A symbol's pin is taken as a whole, an arm only in its direction.
            let whole = |id: u64, d: P| if is_arm(id) { d } else { p(0.0, 0.0) };
            taken.push((*q, whole(*sym, *dir)));
            taken.push((o, whole(osym, odir)));
        }
    }
    out
}

/// Adds the wires found by [`autoconnect`]. Returns their number.
pub fn apply_autoconnect(prj: &mut Project, page: usize, sel: &BTreeSet<u64>) -> usize {
    let segs = autoconnect(&prj.symbol_table(), &prj.pages[page], sel, 400.0);
    for (a, b) in &segs {
        let id = prj.new_id();
        prj.pages[page].wires.push(Wire { id, a: *a, b: *b, ..Default::default() });
    }
    segs.len()
}

const DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const BEND: u32 = 4;

/// Orthogonal path on the grid from `a` to `b` inside `area` that avoids symbols,
/// other connection points, wire ends and running along other wires (crossing them
/// is allowed). Falls back to a single corner when there is no such path.
pub fn route(symbols: &Symbols, page: &Page, area: R, a: P, b: P) -> Vec<P> {
    search(symbols, page, area, a, b).unwrap_or_else(|| {
        if a.x == b.x || a.y == b.y {
            vec![a, b]
        } else {
            vec![a, p(b.x, a.y), b]
        }
    })
}

fn search(symbols: &Symbols, page: &Page, area: R, a: P, b: P) -> Option<Vec<P>> {
    let cell = |q: P| ((q.x / GRID).round() as i32, (q.y / GRID).round() as i32);
    let at = |c: (i32, i32)| p(c.0 as f32 * GRID, c.1 as f32 * GRID);
    let (x0, y0) = cell(area.min);
    let (x1, y1) = cell(area.max);
    let (start, goal) = (cell(a), cell(b));
    if !at(start).same(a) || !at(goal).same(b) {
        return None;
    }
    let all = pins(symbols, page);
    let boxes: Vec<R> = page
        .symbols
        .iter()
        .filter_map(|s| symbols.get(&s.sym).map(|d| d.world_bounds(s.pos, s.rot, s.mirror)))
        .collect();
    let blocked = |c: (i32, i32)| {
        if c == start || c == goal {
            return false;
        }
        let q = at(c);
        boxes.iter().any(|r| r.contains(q))
            || all.iter().any(|(_, o, _)| o.same(q))
            || page.wires.iter().any(|w| w.a.same(q) || w.b.same(q))
    };
    // Lies inside a horizontal / vertical wire.
    let on_wire = |c: (i32, i32), horizontal: bool| {
        let q = at(c);
        page.wires.iter().any(|w| (w.a.y == w.b.y) == horizontal && on_segment_interior(q, w.a, w.b))
    };
    let h = |c: (i32, i32)| (c.0 - goal.0).unsigned_abs() + (c.1 - goal.1).unsigned_abs();
    // State: cell and direction of arrival (4 = start).
    type State = ((i32, i32), usize);
    let mut dist: HashMap<State, u32> = HashMap::new();
    let mut prev: HashMap<State, State> = HashMap::new();
    let mut heap = BinaryHeap::new();
    dist.insert((start, 4), 0);
    heap.push(Reverse((h(start), 0u32, start, 4usize)));
    let mut found = None;
    while let Some(Reverse((_, g, c, dir))) = heap.pop() {
        if dist.get(&(c, dir)).is_some_and(|&d| d < g) {
            continue;
        }
        if c == goal {
            found = Some((c, dir));
            break;
        }
        let crossing_here = c != start && (on_wire(c, true) || on_wire(c, false));
        for (k, (dx, dy)) in DIRS.iter().enumerate() {
            let bend = dir != 4 && dir != k;
            if bend && crossing_here {
                continue;
            }
            if dir != 4 && DIRS[dir] == (-dx, -dy) {
                continue;
            }
            let n = (c.0 + dx, c.1 + dy);
            if n.0 < x0 || n.0 > x1 || n.1 < y0 || n.1 > y1 || blocked(n) {
                continue;
            }
            if on_wire(n, *dy == 0) {
                continue;
            }
            let ng = g + 1 + if bend { BEND } else { 0 };
            if dist.get(&(n, k)).is_none_or(|&d| ng < d) {
                dist.insert((n, k), ng);
                prev.insert((n, k), (c, dir));
                heap.push(Reverse((ng + h(n), ng, n, k)));
            }
        }
    }
    let mut s = found?;
    let mut cells = vec![s.0];
    while let Some(&q) = prev.get(&s) {
        cells.push(q.0);
        s = q;
    }
    cells.reverse();
    // Keep the corners only.
    let mut pts: Vec<P> = Vec::new();
    for (i, c) in cells.iter().enumerate() {
        let corner = i == 0 || i + 1 == cells.len() || {
            let (a, b) = (cells[i - 1], cells[i + 1]);
            (c.0 - a.0, c.1 - a.1) != (b.0 - c.0, b.1 - c.1)
        };
        if corner {
            pts.push(at(*c));
        }
    }
    Some(pts)
}

/// Joins collinear wires that meet end to end where nothing else connects. Returns
/// the number of wires removed.
pub fn merge_collinear(symbols: &Symbols, page: &mut Page) -> usize {
    let mut removed = 0;
    loop {
        let all = pins(symbols, page);
        let mut merged = None;
        'outer: for i in 0..page.wires.len() {
            for j in i + 1..page.wires.len() {
                let (w, v) = (&page.wires[i], &page.wires[j]);
                let shared = [(w.a, w.b, v.a, v.b), (w.a, w.b, v.b, v.a), (w.b, w.a, v.a, v.b), (w.b, w.a, v.b, v.a)]
                    .into_iter()
                    .find(|(m, _, n, _)| m.same(*n));
                let Some((m, far_w, _, far_v)) = shared else { continue };
                let collinear = (far_w.x == m.x && far_v.x == m.x) || (far_w.y == m.y && far_v.y == m.y);
                let opposite = (far_w - m).x * (far_v - m).x + (far_w - m).y * (far_v - m).y < 0.0;
                let alone = page.wires.iter().filter(|x| x.a.same(m) || x.b.same(m) || on_segment_interior(m, x.a, x.b)).count() == 2
                    && !all.iter().any(|(_, q, _)| q.same(m));
                let same_props = w.color == v.color && w.cross_section == v.cross_section && w.cable == v.cable && w.core == v.core;
                if collinear && opposite && alone && same_props {
                    merged = Some((i, j, far_w, far_v));
                    break 'outer;
                }
            }
        }
        let Some((i, j, a, b)) = merged else { break };
        page.wires[i].a = a;
        page.wires[i].b = b;
        page.wires.remove(j);
        removed += 1;
    }
    removed
}

#[cfg(test)]
mod tests;
