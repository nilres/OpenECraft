//! The cabling of a PV array as lines in the plant: the leads from module to module of
//! each string under the modules, the plus and minus leads of each string from its
//! first and last module to the generator junction box of the array (otherwise to its
//! inverter), the DC main cables from the generator junction box to the inverters, and
//! the earthing conductor that bonds the earthing points of the mounting to the
//! earthing busbar.
//!
//! The leads to a device leave the array at the point of its edge nearest to the device,
//! run on the covering to a cable entry just beside it, through the roof and under it
//! to above the device and down onto it. A device that is not placed in the layout ends
//! them just under the roof.

use super::{geometry, junction, PvGeometry};
use crate::model::panel::{Building, ItemKind, PvArray, Roof};
use crate::model::Project;
use crate::panel::buildings::building_pose;
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

/// What a line of the cabling of a PV array is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lead {
    /// From the junction box of one module to that of the next one in its string.
    Link,
    /// The plus lead of a string to its inverter.
    Plus,
    /// The minus lead of a string to its inverter.
    Minus,
    /// The DC main cable from the generator junction box to an inverter.
    Main,
    /// The earthing conductor of the mounting to the earthing busbar.
    Earth,
}

/// A line of the cabling of a PV array in the plant (y up).
#[derive(Clone, Debug, PartialEq)]
pub struct PvLine {
    pub points: Vec<V3>,
    pub lead: Lead,
    /// The device a lead runs to (empty for the links between modules), and whether it
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

/// The cabling of array `a` on roof `r` of building `b` in the plant.
pub fn cabling(prj: &Project, poses: &[SitePose], b: &Building, r: &Roof, a: &PvArray) -> Vec<PvLine> {
    let Some(g) = geometry(r, a) else { return Vec::new() };
    let bp = building_pose(b);
    let h = a.module_height.max(5.0);
    let sec = (1.0 + g.plane.slope() * g.plane.slope()).sqrt();
    let lift = r.thickness.max(1.0) * sec;
    let cover = |q: [f32; 2]| g.plane.at(q) + lift;
    // The junction box of a module (on its back) and the point under it the leads run at.
    let boxes: Vec<V3> = g.modules.iter().map(|m| {
        let n = normal_of(m);
        sub(junction(m, h), scale(n, super::JUNCTION))
    }).collect();
    let low = |p: V3| [p[0], cover([p[0], p[2]]) + LEAD_HEIGHT, p[2]];
    // A device in the building's coordinates: the middle of its top.
    let target = |tag: &str| placed_device(prj, poses, tag).map(|(_, (lo, hi))| bp.local([(lo[0] + hi[0]) / 2.0, hi[1], (lo[2] + hi[2]) / 2.0]));
    let mut out = Vec::new();
    let to_plant = |pts: Vec<V3>| pts.into_iter().map(|q| bp.apply(q)).collect::<Vec<V3>>();
    let combiner = (!a.combiner.is_empty()).then(|| target(&a.combiner));
    let strings = string_modules(&g, a);
    // The leads side by side: each string's minus and plus lead a gap apart.
    let leads = 2 * strings.len();
    for (k, (si, modules)) in strings.iter().enumerate() {
        let (si, s) = (*si, &a.strings[*si]);
        for w in modules.windows(2) {
            let (p, q) = (boxes[w[0]], boxes[w[1]]);
            out.push(PvLine { points: to_plant(vec![p, low(p), low(q), q]), lead: Lead::Link, to: String::new(), reached: true, string: Some(si) });
        }
        let (to, device) = match combiner {
            Some(gak) => (a.combiner.clone(), gak),
            None => (s.inverter.clone(), target(&s.inverter)),
        };
        let (first, last) = (modules[0], modules[modules.len() - 1]);
        for (lead, m, j) in [(Lead::Minus, first, 2 * k), (Lead::Plus, last, 2 * k + 1)] {
            let p = boxes[m];
            let side = (j as f32 - (leads as f32 - 1.0) / 2.0) * LEAD_GAP;
            let mut pts = vec![p, low(p)];
            pts.extend(run_down(r, &g, low(p), device, side, &cover));
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
            let pts = vec![gak, [gak[0], y, gak[2]], [inv[0], y, inv[2]], inv];
            out.push(PvLine { points: to_plant(pts), lead: Lead::Main, to: tag.into(), reached: true, string: Some(*si) });
        }
    }
    // The earthing conductor from earthing point to earthing point, then to the busbar.
    if let Some(&last) = g.earth.last() {
        let device = target(&a.earthing);
        let mut pts: Vec<V3> = g.earth.clone();
        let down = [last[0], cover([last[0], last[2]]) + LEAD_HEIGHT * 0.5, last[2]];
        pts.push(down);
        pts.extend(run_down(r, &g, down, device, 0.0, &cover));
        out.push(PvLine { points: to_plant(pts), lead: Lead::Earth, to: a.earthing.clone(), reached: device.is_some(), string: None });
    }
    out
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
