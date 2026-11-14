//! The cabling of a PV array as lines in the plant. Every module has a plus lead (red)
//! and a minus lead (black) out of its junction box; in a string the plus lead of a
//! module meets the minus lead of the next one at a connector between them. The minus
//! lead of the first module and the plus lead of the last one of each string run to the
//! generator junction box of the array (otherwise to its inverter), a DC main cable
//! (plus and minus) from the generator junction box to the inverters, and the earthing
//! conductor bonds the earthing points of the mounting to the earthing busbar.
//!
//! The leads to a device leave the array at the point of its edge nearest to the device,
//! run on the covering to a cable entry just beside it, through the roof and under it
//! to above the device and down onto it. A device that is not placed in the layout ends
//! them just under the roof. The leads of a freestanding array run on the back of its
//! modules, down into a cable trench, along it to below the device and up onto it.

use super::{junction, PvGeometry, Support};
use crate::model::panel::{ItemKind, PvArray, Roof};
use crate::model::Project;
use crate::panel::roofs::{self, normal_of};
use crate::panel::SitePose;
use crate::parts::vec3::{norm, scale, sub, V3};

/// Height of the leads above the covering under the modules, mm.
pub const LEAD_HEIGHT: f32 = 45.0;
/// How far beside the edge of the array the leads go through the roof, mm.
pub const ENTRY: f32 = 200.0;
/// The leads under the roof: this far below its underside, mm.
pub const UNDER_ROOF: f32 = 60.0;
/// Distance between the plus and the minus lead of a string, mm.
pub const LEAD_GAP: f32 = 30.0;
/// How far the DC main cable rises above the generator junction box and the inverter, mm.
pub const MAIN_RISE: f32 = 200.0;
/// How far apart the plus and the minus lead leave the junction box of a module, mm.
pub const BOX_GAP: f32 = 40.0;
/// Depth of the cable trench of freestanding arrays below the ground, mm.
pub const TRENCH: f32 = 700.0;

/// What a line of the cabling of a PV array is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lead {
    /// The plus lead of a module: to the connector with the next module of its string,
    /// or (with a device in `to`) the plus lead of the string to its generator junction
    /// box or inverter.
    Plus,
    /// The minus lead of a module (to the connector with the module before it, or the
    /// minus lead of the string).
    Minus,
    /// The connector (plug and socket) where the plus lead of a module meets the minus
    /// lead of the next one.
    Connector,
    /// The plus and the minus conductor of the DC main cable from the generator junction
    /// box to an inverter.
    MainPlus,
    MainMinus,
    /// The earthing conductor of the mounting to the earthing busbar.
    Earth,
}

/// A line of the cabling of a PV array in the plant (y up).
#[derive(Clone, Debug, PartialEq)]
pub struct PvLine {
    pub points: Vec<V3>,
    pub lead: Lead,
    /// The device a lead runs to (empty for the leads between modules), and whether it
    /// reaches it (the device is placed in the layout).
    pub to: String,
    pub reached: bool,
    /// The string it belongs to (index in [`PvArray::strings`]), none for the earthing
    /// conductor.
    pub string: Option<usize>,
}

/// The box of the device `tag` placed in the layout in the plant (y up), and the
/// enclosure or room it is placed in (`poses`: [`crate::panel::site_poses`]).
pub fn placed_device(prj: &Project, poses: &[SitePose], tag: &str) -> Option<(usize, (V3, V3))> {
    let tag = tag.trim();
    if tag.is_empty() {
        return None;
    }
    prj.layout.enclosures.iter().enumerate().find_map(|(k, e)| {
        let it = e.items.iter().find(|i| i.kind == ItemKind::Device && i.tag == tag)?;
        let (lo, hi) = crate::panel::item_box(e, it);
        // Enclosure coordinates (y down from its top) to the plant.
        let pose = poses.get(k).copied().unwrap_or_default();
        let corners: Vec<V3> = (0..8).map(|c| pose.apply([[lo[0], hi[0]][c & 1], e.height - [lo[1], hi[1]][c >> 1 & 1], [lo[2], hi[2]][c >> 2]])).collect();
        let min = [0, 1, 2].map(|i| corners.iter().map(|q| q[i]).fold(f32::INFINITY, f32::min));
        let max = [0, 1, 2].map(|i| corners.iter().map(|q| q[i]).fold(f32::NEG_INFINITY, f32::max));
        Some((k, (min, max)))
    })
}

/// The devices that are earthing or equipotential bonding busbars (by the subgroup of
/// their parts), by tag: what the earthing conductor of a mounting runs to.
pub fn earthing_busbars(prj: &Project) -> Vec<String> {
    devices_of(prj, |s| s.contains("earthing busbar") || s.contains("bonding"))
}

/// The devices that are generator junction boxes (by the subgroup of their parts), by
/// tag: what the strings of an array run to.
pub fn junction_boxes(prj: &Project) -> Vec<String> {
    devices_of(prj, |s| s.contains("generator junction box"))
}

/// The devices one of whose parts has a subgroup (lower case) that `wanted` accepts.
fn devices_of(prj: &Project, wanted: impl Fn(&str) -> bool) -> Vec<String> {
    let fits = |number: &str| prj.parts.iter().chain(crate::parts::catalog()).find(|p| p.number == number).is_some_and(|p| wanted(&p.subgroup.to_lowercase()));
    prj.devices.iter().filter(|(_, d)| d.parts.iter().any(|r| fits(&r.number))).map(|(t, _)| t.clone()).collect()
}

/// The modules of an array in the order its strings take them: row by row, every other
/// row the other way round.
pub fn string_order(g: &PvGeometry, a: &PvArray) -> Vec<usize> {
    let cols = a.columns.max(1) as usize;
    let mut out = Vec::new();
    for (k, chunk) in (0..g.modules.len()).collect::<Vec<_>>().chunks(cols).enumerate() {
        if k % 2 == 0 {
            out.extend(chunk.iter().copied());
        } else {
            out.extend(chunk.iter().rev().copied());
        }
    }
    out
}

/// The strings of an array each as the modules (indices in [`PvGeometry::modules`]) of
/// its strings in series: one list per string of each kind and each of its parallel
/// strings.
pub fn string_modules(g: &PvGeometry, a: &PvArray) -> Vec<(usize, Vec<usize>)> {
    let order = string_order(g, a);
    let mut next = 0;
    let mut out = Vec::new();
    for (si, s) in a.strings.iter().enumerate() {
        for _ in 0..s.parallel.max(1) {
            let n = (s.series.max(1) as usize).min(order.len().saturating_sub(next));
            if n == 0 {
                return out;
            }
            out.push((si, order[next..next + n].to_vec()));
            next += n;
        }
    }
    out
}

/// The cabling of array `a` standing on `s` in the plant.
pub fn cabling(prj: &Project, poses: &[SitePose], s: Support, a: &PvArray) -> Vec<PvLine> {
    let Some(g) = s.geometry(a) else { return Vec::new() };
    let bp = s.pose();
    let h = a.module_height.max(5.0);
    let sec = (1.0 + g.plane.slope() * g.plane.slope()).sqrt();
    let lift = match s {
        Support::Roof(_, r) => r.thickness.max(1.0) * sec,
        Support::Ground => 0.0,
    };
    let cover = |q: [f32; 2]| g.plane.at(q) + lift;
    // The junction box of a module (on its back) and the point under it the leads run at:
    // on the covering of a roof, on the back of freestanding modules.
    let boxes: Vec<V3> = g.modules.iter().map(|m| {
        let n = normal_of(m);
        sub(junction(m, h), scale(n, super::JUNCTION))
    }).collect();
    let back = g.modules.first().map_or([0.0, 1.0, 0.0], |m| normal_of(m));
    let low = |p: V3| match s {
        Support::Roof(..) => [p[0], cover([p[0], p[2]]) + LEAD_HEIGHT, p[2]],
        Support::Ground => sub(p, scale(back, LEAD_HEIGHT)),
    };
    // From a point under the modules to a device (building coordinates), `side` to the
    // right of the way.
    let way = |from: V3, device: Option<V3>, side: f32| match s {
        Support::Roof(_, r) => run_down(r, &g, from, device, side, &cover),
        Support::Ground => run_ground(from, device, side),
    };
    // A device in the building's coordinates: the middle of its top.
    let target = |tag: &str| placed_device(prj, poses, tag).map(|(_, (lo, hi))| bp.local([(lo[0] + hi[0]) / 2.0, hi[1], (lo[2] + hi[2]) / 2.0]));
    let mut out = Vec::new();
    let to_plant = |pts: Vec<V3>| pts.into_iter().map(|q| bp.apply(q)).collect::<Vec<V3>>();
    let combiner = (!a.combiner.is_empty()).then(|| target(&a.combiner));
    let strings = string_modules(&g, a);
    // The leads side by side: each string's minus and plus lead a gap apart.
    let leads = 2 * strings.len();
    let line = |points: Vec<V3>, lead: Lead, si: usize| PvLine { points: to_plant(points), lead, to: String::new(), reached: true, string: Some(si) };
    for (k, (si, modules)) in strings.iter().enumerate() {
        let (si, s) = (*si, &a.strings[*si]);
        // Along the string at each module (towards the next one; at the last one from the
        // one before): the plus lead leaves the box on that side, the minus lead on the other.
        let along = |i: usize| {
            let (a, b) = if i + 1 < modules.len() { (i, i + 1) } else if i > 0 { (i - 1, i) } else { (i, i) };
            let d = sub(boxes[modules[b]], boxes[modules[a]]);
            let d = [d[0], 0.0, d[2]];
            if crate::parts::vec3::length(d) < 1.0 {
                let m = &g.modules[modules[i]];
                crate::parts::vec3::norm(sub(m[1], m[0]))
            } else {
                crate::parts::vec3::norm(d)
            }
        };
        let plus = |i: usize| crate::parts::vec3::add(boxes[modules[i]], scale(along(i), BOX_GAP / 2.0));
        let minus = |i: usize| sub(boxes[modules[i]], scale(along(i), BOX_GAP / 2.0));
        for i in 0..modules.len().saturating_sub(1) {
            let (p, q) = (plus(i), minus(i + 1));
            let c = scale(crate::parts::vec3::add(low(p), low(q)), 0.5);
            out.push(line(vec![p, low(p), c], Lead::Plus, si));
            out.push(line(vec![q, low(q), c], Lead::Minus, si));
            let d = along(i);
            out.push(line(vec![sub(c, scale(d, 25.0)), crate::parts::vec3::add(c, scale(d, 25.0))], Lead::Connector, si));
        }
        let (to, device) = match combiner {
            Some(gak) => (a.combiner.clone(), gak),
            None => (s.inverter.clone(), target(&s.inverter)),
        };
        let n = modules.len();
        for (lead, p, j) in [(Lead::Minus, minus(0), 2 * k), (Lead::Plus, plus(n - 1), 2 * k + 1)] {
            let side = (j as f32 - (leads as f32 - 1.0) / 2.0) * LEAD_GAP;
            let mut pts = vec![p, low(p)];
            pts.extend(way(low(p), device, side));
            out.push(PvLine { points: to_plant(pts), lead, to: to.clone(), reached: device.is_some(), string: Some(si) });
        }
    }
    // From the generator junction box a DC main cable to each inverter: up out of its top,
    // across above them and down onto the inverter.
    if let Some(Some(gak)) = combiner {
        let mut done: Vec<&str> = Vec::new();
        for (si, _) in &strings {
            let tag = a.strings[*si].inverter.as_str();
            if tag.is_empty() || done.contains(&tag) {
                continue;
            }
            done.push(tag);
            let Some(inv) = target(tag) else { continue };
            let y = gak[1].max(inv[1]) + MAIN_RISE;
            for (lead, dx) in [(Lead::MainMinus, -LEAD_GAP / 2.0), (Lead::MainPlus, LEAD_GAP / 2.0)] {
                let (a, b) = ([gak[0] + dx, gak[1], gak[2]], [inv[0] + dx, inv[1], inv[2]]);
                let pts = vec![a, [a[0], y + dx, a[2]], [b[0], y + dx, b[2]], b];
                out.push(PvLine { points: to_plant(pts), lead, to: tag.into(), reached: true, string: Some(*si) });
            }
        }
    }
    // The earthing conductor from earthing point to earthing point, then to the busbar.
    if let Some(&last) = g.earth.last() {
        let device = target(&a.earthing);
        let mut pts: Vec<V3> = g.earth.clone();
        // On a roof down onto the covering; on a post from where it is.
        let down = match s {
            Support::Roof(..) => {
                let down = [last[0], cover([last[0], last[2]]) + LEAD_HEIGHT * 0.5, last[2]];
                pts.push(down);
                down
            }
            Support::Ground => last,
        };
        pts.extend(way(down, device, 0.0));
        out.push(PvLine { points: to_plant(pts), lead: Lead::Earth, to: a.earthing.clone(), reached: device.is_some(), string: None });
    }
    out
}

/// From `from` (under a freestanding table) down into the cable trench, along it to
/// below `device` (the middle of its top) and up onto it, otherwise down to the ground;
/// `side`: how far to the right of the way it runs.
fn run_ground(from: V3, device: Option<V3>, side: f32) -> Vec<V3> {
    let Some(t) = device else { return vec![[from[0], 0.0, from[2]]] };
    let d = [t[0] - from[0], t[2] - from[2]];
    let l = d[0].hypot(d[1]).max(1e-6);
    let right = [-d[1] / l, d[0] / l];
    let shift = |q: [f32; 2]| [q[0] + right[0] * side, q[1] + right[1] * side];
    let (a, b) = (shift([from[0], from[2]]), shift([t[0], t[2]]));
    vec![[a[0], from[1], a[1]], [a[0], -TRENCH, a[1]], [b[0], -TRENCH, b[1]], [b[0], t[1], b[1]]]
}

/// From `from` (on the covering) to the cable entry beside the array nearest to
/// `device` (the middle of its top, building coordinates), through the roof, under it to
/// above the device and down onto it; `side`: how far to the right of the way it runs.
fn run_down(r: &Roof, g: &PvGeometry, from: V3, device: Option<V3>, side: f32, cover: &dyn Fn([f32; 2]) -> f32) -> Vec<V3> {
    let planes = roofs::planes(r);
    let outline = &g.outline;
    let n = outline.len().max(1) as f32;
    let centre = [outline.iter().map(|q| q[0]).sum::<f32>() / n, outline.iter().map(|q| q[1]).sum::<f32>() / n];
    // The point of the edge of the array nearest to the device, otherwise the middle of
    // its lowest edge.
    let edge = |k: usize| (outline[k], outline[(k + 1) % outline.len()]);
    let nearest = |t: [f32; 2]| {
        (0..outline.len())
            .map(|k| {
                let (a, b) = edge(k);
                let d = [b[0] - a[0], b[1] - a[1]];
                let len2 = (d[0] * d[0] + d[1] * d[1]).max(1e-6);
                let s = (((t[0] - a[0]) * d[0] + (t[1] - a[1]) * d[1]) / len2).clamp(0.05, 0.95);
                [a[0] + d[0] * s, a[1] + d[1] * s]
            })
            .min_by(|p, q| ((p[0] - t[0]).hypot(p[1] - t[1])).total_cmp(&(q[0] - t[0]).hypot(q[1] - t[1])))
    };
    let at_edge = match device {
        Some(t) => nearest([t[0], t[2]]),
        None => (0..outline.len()).map(edge).map(|(a, b)| [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]).min_by(|p, q| g.plane.at(*p).total_cmp(&g.plane.at(*q))),
    };
    let Some(e) = at_edge else { return Vec::new() };
    // Beside the edge, while that is still on the roof.
    let out_dir = {
        let d = [e[0] - centre[0], e[1] - centre[1]];
        let l = d[0].hypot(d[1]).max(1e-6);
        [d[0] / l, d[1] / l]
    };
    let mut entry = [e[0] + out_dir[0] * ENTRY, e[1] + out_dir[1] * ENTRY];
    if roofs::underside_at(r, entry).is_none() {
        entry = e;
    }
    // To the right of the way out of the array.
    let right = [-out_dir[1], out_dir[0]];
    let shift = |q: [f32; 2]| [q[0] + right[0] * side, q[1] + right[1] * side];
    let entry = shift(entry);
    let under = |q: [f32; 2]| roofs::underside_at(r, q).map(|_| roofs::lowest(&planes, q) - UNDER_ROOF);
    let mut pts = vec![[entry[0], cover(entry) + LEAD_HEIGHT.min(from[1] - cover([from[0], from[2]])).max(10.0), entry[1]]];
    let Some(below) = under(entry) else { return pts };
    pts.push([entry[0], below, entry[1]]);
    let Some(t) = device else {
        return pts;
    };
    let goal = shift([t[0], t[2]]);
    // Under the roof to above the device, bent where the roof bends.
    let steps = 16;
    let mut y = below;
    for i in 1..=steps {
        let s = i as f32 / steps as f32;
        let q = [entry[0] + (goal[0] - entry[0]) * s, entry[1] + (goal[1] - entry[1]) * s];
        y = under(q).unwrap_or(y).max(t[1] + 1.0);
        pts.push([q[0], y, q[1]]);
    }
    pts.push([goal[0], t[1], goal[1]]);
    // Without the points on a straight stretch.
    let mut kept: Vec<V3> = Vec::new();
    for (k, &p) in pts.iter().enumerate() {
        if let (Some(&a), Some(&b)) = (kept.last(), pts.get(k + 1)) {
            let (u, v) = (norm(sub(p, a)), norm(sub(b, p)));
            if crate::parts::vec3::dot(u, v) > 0.9999 {
                continue;
            }
        }
        kept.push(p);
    }
    kept
}

#[cfg(test)]
mod tests;
