//! The lightning protection system of a building ([`Lightning`], IEC 62305-3): its
//! air-termination system (conductors on the ridges and hips of pitched roofs, round
//! the edge and as a mesh on flat and shed roofs, and air-termination rods), down
//! conductors along the outer walls at most as far apart as its class allows, each with
//! a test joint, and the earth entries from the test joints to the earth electrodes
//! ([`super::earthing`]).
//!
//! It comes out as lines in the building's coordinates (y up from its ground level) for
//! the 3D view and the site plan ([`geometry`]); its material goes into the bill of
//! materials ([`material`]). The PV arrays on its roofs have to lie in the protection of
//! its air-termination system by the protection angle method ([`protected`]).

use super::buildings::building_pose;
use super::earthing::{self, walls};
use super::roofs;
use super::solar;
use super::SitePose;
use crate::issue::{Issue, Severity};
use crate::model::panel::{AirRod, Building, ElectrodeKind, Lightning, Roof};
use crate::model::Project;
use crate::parts::vec3::{dist, V3};
use openecraft_i18n::tr;

/// Height of the test joints of the down conductors above the ground, mm.
pub const TEST_JOINT: f32 = 1000.0;
/// Distance of the down conductors from the outer walls, mm.
pub const WALL_GAP: f32 = 100.0;
/// Height of the air-termination conductors above the covering, mm.
pub const CLEARANCE: f32 = 50.0;
/// Distance of the conductor holders on the roof and on the walls, mm.
pub const HOLDER: f32 = 1000.0;
/// How far inside the edge of a flat roof its edge conductor runs, mm.
pub const EDGE: f32 = 100.0;

/// What a line of a lightning protection system is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LpsKind {
    /// Air-termination conductor on a ridge or hip, or of the mesh of a flat roof.
    AirTermination,
    /// From the air-termination system over the roof to a down conductor, or to an
    /// air-termination rod.
    Roof,
    /// Down the outer wall to the test joint.
    Down,
    /// From the test joint into the soil to the earth electrode.
    Entry,
    /// An air-termination rod.
    Rod,
}

/// A line of a lightning protection system (building coordinates, y up).
#[derive(Clone, Debug, PartialEq)]
pub struct LpsLine {
    pub points: Vec<V3>,
    pub kind: LpsKind,
}

/// The lines of a lightning protection system and its test joints.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LpsGeometry {
    pub lines: Vec<LpsLine>,
    pub joints: Vec<V3>,
}

impl LpsGeometry {
    /// Length of its lines of `kind`, mm.
    pub fn length(&self, kind: LpsKind) -> f32 {
        self.lines.iter().filter(|l| l.kind == kind).map(|l| l.points.windows(2).map(|w| dist(w[0], w[1])).sum::<f32>()).sum()
    }

    pub fn count(&self, kind: LpsKind) -> usize {
        self.lines.iter().filter(|l| l.kind == kind).count()
    }
}

/// Roman numeral of a class.
pub fn class_name(class: u8) -> &'static str {
    ["I", "II", "III", "IV"][(class.clamp(1, 4) - 1) as usize]
}

/// Greatest distance of the down conductors of a class, mm.
pub fn down_spacing(class: u8) -> f32 {
    [10_000.0, 10_000.0, 15_000.0, 20_000.0][(class.clamp(1, 4) - 1) as usize]
}

/// Mesh size of the air-termination system of a flat roof of a class, mm.
pub fn mesh(class: u8) -> f32 {
    [5000.0, 10_000.0, 15_000.0, 20_000.0][(class.clamp(1, 4) - 1) as usize]
}

/// Protection angle of an air-termination element `h` mm above the surface it
/// protects, degrees: at most 2 m high the angle of its class, down to 23° at the radius
/// of its rolling sphere (20, 30, 45, 60 m), none above it.
pub fn protection_angle(class: u8, h: f32) -> Option<f32> {
    let k = (class.clamp(1, 4) - 1) as usize;
    let (low, top) = ([70.0, 72.0, 76.0, 79.0][k], [20_000.0, 30_000.0, 45_000.0, 60_000.0][k]);
    if h <= 0.0 || h > top {
        return None;
    }
    if h <= 2000.0 {
        return Some(low);
    }
    Some(low + (23.0 - low) * (h - 2000.0) / (top - 2000.0))
}

/// Number of down conductors of a building whose outer walls run `perimeter` mm round
/// it: at least two, at most a class's distance apart.
pub fn needed_downs(class: u8, perimeter: f32) -> u32 {
    ((perimeter / down_spacing(class)).ceil() as u32).max(2)
}

/// Height of the covering of the roofs of `b` at `q` (building floor plan): the highest
/// of the roofs there.
fn cover(b: &Building, q: [f32; 2]) -> Option<f32> {
    b.roofs.iter().filter_map(|r| roofs::covering_at(r, q)).reduce(f32::max)
}

/// The lines along the covering from `a` to `b` (floor plan), `CLEARANCE` above it,
/// bent where it bends; `None` where a point is off the roofs.
fn over_roof(bld: &Building, a: [f32; 2], b: [f32; 2]) -> Vec<V3> {
    let n = 12;
    (0..=n)
        .filter_map(|i| {
            let t = i as f32 / n as f32;
            let q = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
            cover(bld, q).map(|y| [q[0], y + CLEARANCE, q[1]])
        })
        .collect()
}

/// The air-termination conductors of a roof: its ridges and hips (where two slopes
/// meet), on a flat or shed roof its edge and a mesh of the class's size.
fn air_termination(b: &Building, r: &Roof, class: u8) -> Vec<Vec<V3>> {
    let reg = roofs::region(r);
    let (lo, hi) = super::bounds(&reg);
    let on_edge = |q: [f32; 2]| (q[0] - lo[0]).abs() < 1.0 || (q[0] - hi[0]).abs() < 1.0 || (q[1] - lo[1]).abs() < 1.0 || (q[1] - hi[1]).abs() < 1.0;
    let mut ridges: Vec<([f32; 2], [f32; 2])> = Vec::new();
    for s in roofs::slopes(r) {
        for k in 0..s.len() {
            let (p, q) = (s[k], s[(k + 1) % s.len()]);
            let mid = [(p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0];
            let same = |e: &([f32; 2], [f32; 2])| {
                let d = |u: [f32; 2], v: [f32; 2]| (u[0] - v[0]).hypot(u[1] - v[1]) < 1.0;
                (d(e.0, p) && d(e.1, q)) || (d(e.0, q) && d(e.1, p))
            };
            if (p[0] - q[0]).hypot(p[1] - q[1]) > 1.0 && !on_edge(mid) && !ridges.iter().any(same) {
                ridges.push((p, q));
            }
        }
    }
    let line = |p: [f32; 2], q: [f32; 2]| over_roof(b, p, q);
    if !ridges.is_empty() {
        return ridges.into_iter().map(|(p, q)| line(p, q)).filter(|l| l.len() >= 2).collect();
    }
    // Flat or shed: round the edge and across as a mesh.
    let (x0, z0, x1, z1) = (lo[0] + EDGE, lo[1] + EDGE, hi[0] - EDGE, hi[1] - EDGE);
    let mut out = vec![line([x0, z0], [x1, z0]), line([x1, z0], [x1, z1]), line([x1, z1], [x0, z1]), line([x0, z1], [x0, z0])];
    let m = mesh(class);
    for (from, to, along_x) in [(x0, x1, true), (z0, z1, false)] {
        let n = ((to - from) / m).ceil().max(1.0) as usize;
        for k in 1..n {
            let t = from + (to - from) * k as f32 / n as f32;
            out.push(if along_x { line([t, z0], [t, z1]) } else { line([x0, t], [x1, t]) });
        }
    }
    out.retain(|l| l.len() >= 2);
    out
}

/// The point of `lines` nearest to `q` in the floor plan.
fn nearest_plan(lines: &[Vec<V3>], q: [f32; 2]) -> Option<V3> {
    let flat: Vec<Vec<V3>> = lines.iter().map(|l| l.iter().map(|v| [v[0], 0.0, v[2]]).collect()).collect();
    let near = earthing::nearest(&flat, [q[0], 0.0, q[1]])?;
    // Back on the line: the height of its nearest point.
    lines.iter().flat_map(|l| l.iter()).copied().min_by(|a, b| (a[0] - near[0]).hypot(a[2] - near[2]).total_cmp(&(b[0] - near[0]).hypot(b[2] - near[2]))).map(|v| [near[0], v[1], near[2]])
}

/// The places of the down conductors of a building whose outer walls are `(lo, hi)`:
/// `n` of them evenly round the walls from the rear left corner, each with the
/// direction out of its wall.
pub fn down_places(lo: [f32; 2], hi: [f32; 2], n: u32) -> Vec<([f32; 2], [f32; 2])> {
    let (w, d) = (hi[0] - lo[0], hi[1] - lo[1]);
    let perimeter = 2.0 * (w + d);
    (0..n.max(1))
        .map(|k| {
            let s = perimeter * k as f32 / n.max(1) as f32;
            if s < w {
                ([lo[0] + s, lo[1]], [0.0, -1.0])
            } else if s < w + d {
                ([hi[0], lo[1] + s - w], [1.0, 0.0])
            } else if s < 2.0 * w + d {
                ([hi[0] - (s - w - d), hi[1]], [0.0, 1.0])
            } else {
                ([lo[0], hi[1] - (s - 2.0 * w - d)], [-1.0, 0.0])
            }
        })
        .collect()
}

/// The lightning protection system of building `bi` (building coordinates); `None`
/// without one or without walls.
pub fn geometry(prj: &Project, poses: &[SitePose], bi: usize) -> Option<LpsGeometry> {
    let b = prj.layout.buildings.get(bi)?;
    let lp = b.lightning.as_ref()?;
    let (lo, hi) = walls(prj, poses, bi)?;
    let mut g = LpsGeometry::default();
    let air: Vec<Vec<V3>> = b.roofs.iter().flat_map(|r| air_termination(b, r, lp.class)).collect();
    g.lines.extend(air.iter().map(|l| LpsLine { points: l.clone(), kind: LpsKind::AirTermination }));
    // The air-termination rods, each joined to the nearest air-termination conductor.
    for rod in &lp.rods {
        let Some(y) = cover(b, [rod.x, rod.z]) else { continue };
        let foot = [rod.x, y, rod.z];
        g.lines.push(LpsLine { points: vec![foot, [rod.x, y + rod.height.max(100.0), rod.z]], kind: LpsKind::Rod });
        if let Some(n) = nearest_plan(&air, [rod.x, rod.z]) {
            let mut lead = vec![[foot[0], foot[1] + CLEARANCE, foot[2]]];
            lead.extend(over_roof(b, [rod.x, rod.z], [n[0], n[2]]).into_iter().skip(1));
            if lead.len() >= 2 {
                g.lines.push(LpsLine { points: lead, kind: LpsKind::Roof });
            }
        }
    }
    let perimeter = 2.0 * ((hi[0] - lo[0]) + (hi[1] - lo[1]));
    let n = if lp.down_conductors > 0 { lp.down_conductors } else { needed_downs(lp.class, perimeter) };
    let top_floor = b.levels.iter().map(|l| l.elevation + l.height).fold(0.0, f32::max);
    // The electrodes the down conductors end on: the ring, otherwise the foundation
    // earth electrode, otherwise the earth rods.
    let electrode_lines: Vec<Vec<V3>> = [ElectrodeKind::Ring, ElectrodeKind::Foundation, ElectrodeKind::Rod]
        .into_iter()
        .map(|kind| b.electrodes.iter().filter(|e| e.kind == kind).flat_map(|e| earthing::paths(prj, poses, bi, e)).collect::<Vec<_>>())
        .find(|l| !l.is_empty())
        .unwrap_or_default();
    for (w, out) in down_places(lo, hi, n) {
        let p = [w[0] + out[0] * WALL_GAP, w[1] + out[1] * WALL_GAP];
        // Up the wall under the overhang, round its edge onto the covering and over it
        // to the air-termination system.
        let roof = b.roofs.iter().find(|r| roofs::underside_at(r, p).is_some());
        let top = roof.and_then(|r| roofs::underside_at(r, p)).map_or(top_floor, |u| u - 30.0);
        g.lines.push(LpsLine { points: vec![[p[0], top, p[1]], [p[0], TEST_JOINT, p[1]]], kind: LpsKind::Down });
        g.joints.push([p[0], TEST_JOINT, p[1]]);
        let mut over: Vec<V3> = vec![[p[0], top, p[1]]];
        if let Some(r) = roof {
            let (rlo, rhi) = super::bounds(&roofs::region(r));
            let reach = if out[0] < 0.0 { p[0] - rlo[0] } else if out[0] > 0.0 { rhi[0] - p[0] } else if out[1] < 0.0 { p[1] - rlo[1] } else { rhi[1] - p[1] };
            let e = [p[0] + out[0] * (reach - 1.0).max(0.0), p[1] + out[1] * (reach - 1.0).max(0.0)];
            if let Some(u) = roofs::underside_at(r, e) {
                over.push([e[0], u, e[1]]);
            }
            if let Some(n) = nearest_plan(&air, e) {
                over.extend(over_roof(b, e, [n[0], n[2]]));
            }
        } else if let Some(n) = nearest_plan(&air, p) {
            over.extend(over_roof(b, p, [n[0], n[2]]));
        }
        if over.len() >= 2 {
            g.lines.push(LpsLine { points: over, kind: LpsKind::Roof });
        }
        // From the test joint into the soil and to the earth electrode.
        if let Some(e) = earthing::nearest(&electrode_lines, [p[0], 0.0, p[1]]) {
            g.lines.push(LpsLine { points: vec![[p[0], TEST_JOINT, p[1]], [p[0], e[1].min(0.0), p[1]], e], kind: LpsKind::Entry });
        }
    }
    Some(g)
}

/// The air-termination points of building `bi`: the tips of its rods and the
/// air-termination conductors every half metre.
fn terminals(g: &LpsGeometry) -> Vec<V3> {
    let mut out = Vec::new();
    for l in &g.lines {
        match l.kind {
            LpsKind::Rod => out.extend(l.points.last().copied()),
            LpsKind::AirTermination => {
                for w in l.points.windows(2) {
                    let n = (dist(w[0], w[1]) / 500.0).ceil().max(1.0) as usize;
                    out.extend((0..=n).map(|k| {
                        let t = k as f32 / n as f32;
                        [0, 1, 2].map(|i| w[0][i] + (w[1][i] - w[0][i]) * t)
                    }));
                }
            }
            _ => {}
        }
    }
    out
}

/// Whether point `q` (building coordinates) lies in the protection of the
/// air-termination system `g` of a system of `class`: under an air-termination point
/// within its protection angle.
pub fn protected(g: &LpsGeometry, class: u8, q: V3) -> bool {
    terminals(g).into_iter().any(|t| {
        let h = t[1] - q[1];
        protection_angle(class, h).is_some_and(|a| (t[0] - q[0]).hypot(t[2] - q[2]) <= h * a.to_radians().tan() + 1.0)
    })
}

/// The PV arrays on the roofs of building `bi` not wholly in the protection of its
/// air-termination system (their names).
pub fn unprotected_arrays(prj: &Project, poses: &[SitePose], bi: usize) -> Vec<String> {
    let Some(b) = prj.layout.buildings.get(bi) else { return Vec::new() };
    let (Some(lp), Some(g)) = (b.lightning.as_ref(), geometry(prj, poses, bi)) else { return Vec::new() };
    let mut out = Vec::new();
    for r in &b.roofs {
        for a in &r.solar {
            let Some(pv) = solar::geometry(r, a) else { continue };
            if pv.modules.iter().flatten().any(|q| !protected(&g, lp.class, *q)) {
                out.push(a.name.clone());
            }
        }
    }
    out
}

/// A system with its numbers in range.
pub fn checked(mut lp: Lightning) -> Lightning {
    lp.class = lp.class.clamp(1, 4);
    lp.down_conductors = lp.down_conductors.min(200);
    lp.part = lp.part.trim().into();
    for r in &mut lp.rods {
        r.height = r.height.clamp(100.0, 20_000.0);
    }
    lp
}

/// A new air-termination rod on the roofs of building `bi` at `at` (floor plan),
/// otherwise in the middle of its first roof; its id. The building gets a lightning
/// protection system (class III) if it has none.
pub fn add_rod(prj: &mut Project, bi: usize, at: Option<[f32; 2]>) -> Result<u64, String> {
    let id = prj.new_id().max(1);
    let b = prj.layout.buildings.get_mut(bi).ok_or_else(|| tr("No such building").to_string())?;
    let middle = b.roofs.first().map_or([0.0, 0.0], |r| [r.x + r.width / 2.0, r.z + r.depth / 2.0]);
    let [x, z] = at.unwrap_or(middle);
    let lp = b.lightning.get_or_insert_with(Lightning::default);
    let name = (1..).map(|k| format!("F{k}")).find(|c| !lp.rods.iter().any(|r| &r.name == c)).unwrap_or_default();
    lp.rods.push(AirRod { id, name, x, z, ..Default::default() });
    Ok(id)
}

/// The part of the conductors.
pub fn conductor(lp: &Lightning) -> String {
    if lp.part.is_empty() { "GEN.LP-WIRE-AL8".into() } else { lp.part.clone() }
}

/// The material of the lightning protection system of building `bi`, with its
/// quantities: the round wire of the conductors in metres, their holders on the roof
/// and the walls, the air-termination rods, the test joints, the earth entries and the
/// clamps joining the conductors.
pub fn material(prj: &Project, poses: &[SitePose], bi: usize) -> Vec<(String, f32)> {
    let Some(b) = prj.layout.buildings.get(bi) else { return Vec::new() };
    let (Some(lp), Some(g)) = (b.lightning.as_ref(), geometry(prj, poses, bi)) else { return Vec::new() };
    let (air, roof, down) = (g.length(LpsKind::AirTermination), g.length(LpsKind::Roof), g.length(LpsKind::Down));
    let downs = g.count(LpsKind::Down) as f32;
    let rods = g.count(LpsKind::Rod) as f32;
    let mut out: Vec<(String, f32)> = vec![
        (conductor(lp), ((air + roof + down) / 1000.0 * 100.0).ceil() / 100.0),
        ("GEN.LP-HOLDER-ROOF".into(), ((air + roof) / HOLDER).ceil()),
        ("GEN.LP-HOLDER-WALL".into(), g.lines.iter().filter(|l| l.kind == LpsKind::Down).map(|l| (dist(l.points[0], l.points[1]) / HOLDER).ceil()).sum()),
        ("GEN.LP-ROD-AL16".into(), rods),
        ("GEN.LP-TEST-JOINT".into(), downs),
        ("GEN.LP-EARTH-ENTRY".into(), g.count(LpsKind::Entry) as f32),
        ("GEN.LP-CLAMP-MV".into(), downs + g.count(LpsKind::Entry) as f32 + rods),
    ];
    out.retain(|(_, q)| *q > 0.0);
    out
}

/// Lightning protection systems without earth electrodes or walls, with fewer down
/// conductors than their class needs, air-termination rods off the roofs, and PV
/// arrays outside the protection of their air-termination system.
pub fn problems(prj: &Project) -> Vec<Issue> {
    let poses = super::site_poses(prj);
    let mut out = Vec::new();
    for (bi, b) in prj.layout.buildings.iter().enumerate() {
        let Some(lp) = &b.lightning else { continue };
        let Some((lo, hi)) = walls(prj, &poses, bi) else {
            out.push(crate::issue!(Severity::Warning, None, "{}: the lightning protection has no walls to run along: the building has no rooms or roofs", b.name));
            continue;
        };
        if !earthing::earthed(b) {
            out.push(crate::issue!(Severity::Warning, None, "{}: the lightning protection has no earth electrode", b.name));
        }
        let need = needed_downs(lp.class, 2.0 * ((hi[0] - lo[0]) + (hi[1] - lo[1])));
        if lp.down_conductors > 0 && lp.down_conductors < need {
            out.push(crate::issue!(Severity::Warning, None, "{}: {} down conductors, class {} needs {} round the building", b.name, lp.down_conductors, class_name(lp.class), need));
        }
        for rod in &lp.rods {
            if cover(b, [rod.x, rod.z]).is_none() {
                out.push(crate::issue!(Severity::Warning, None, "{}: air-termination rod {} stands off the roofs", b.name, rod.name));
            }
        }
        for a in unprotected_arrays(prj, &poses, bi) {
            out.push(crate::issue!(Severity::Warning, None, "{}: PV array {} lies outside the protection of the air-termination system", b.name, a));
        }
    }
    out
}

/// The lines of the lightning protection system and the electrodes of building `bi` in
/// the plant, with what each is: for the 3D view and the site plan.
pub fn plant_lines(prj: &Project, poses: &[SitePose], bi: usize) -> Vec<(Vec<V3>, Line)> {
    let Some(b) = prj.layout.buildings.get(bi) else { return Vec::new() };
    let bp = building_pose(b);
    let to_plant = |p: &[V3]| p.iter().map(|&q| bp.apply(q)).collect::<Vec<V3>>();
    let mut out = Vec::new();
    for e in &b.electrodes {
        for p in earthing::paths(prj, poses, bi, e) {
            out.push((to_plant(&p), Line::Electrode(e.kind)));
        }
        if let Some(p) = earthing::busbar_lead(prj, poses, bi, e) {
            out.push((to_plant(&p), Line::Busbar));
        }
    }
    if let Some(g) = geometry(prj, poses, bi) {
        out.extend(g.lines.iter().map(|l| (to_plant(&l.points), Line::Lps(l.kind))));
        out.extend(g.joints.iter().map(|&j| (to_plant(&[j]), Line::Joint)));
    }
    out
}

/// What a line of [`plant_lines`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Line {
    Electrode(ElectrodeKind),
    /// From an electrode to its main earthing busbar.
    Busbar,
    Lps(LpsKind),
    /// A test joint (one point).
    Joint,
}

#[cfg(test)]
mod tests;
