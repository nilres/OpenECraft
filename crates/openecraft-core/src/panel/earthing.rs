//! The earth electrodes of a building ([`Electrode`]): the foundation earth electrode in
//! the foundation slab of its lowest level (a closed ring along the outer walls,
//! divided into meshes of at most 20 × 20 m), the ring earth electrode in the soil round
//! it, and earth rods driven into the soil. Each may be connected to a main earthing
//! busbar; the down conductors of the lightning protection ([`super::lightning`]) end on
//! them.
//!
//! The electrodes come out as lines in the building's coordinates (y up from its ground
//! level) for the 3D view and the site plan ([`paths`]); their material goes into the
//! bill of materials ([`material`]).

use super::buildings::{building_pose, building_rooms, SLAB};
use super::solar::cabling::placed_device;
use super::{outline, SitePose};
use crate::issue::{Issue, Severity};
use crate::model::panel::{Building, Electrode, ElectrodeKind, Material};
use crate::parts::Part;
use crate::model::Project;
use crate::parts::vec3::{dist, V3};
use openecraft_i18n::{tr, tr_const};

/// Concrete under a foundation earth electrode in the slab, mm.
pub const COVER: f32 = 50.0;
/// Distance of the spacers of a foundation earth electrode, mm.
pub const SPACER: f32 = 2000.0;
/// Length of a section of an earth rod, mm.
pub const ROD_SECTION: f32 = 1500.0;

impl Material {
    pub const ALL: [Material; 5] = [Material::GalvanisedSteel, Material::StainlessSteel, Material::Copper, Material::CopperBondedSteel, Material::Aluminium];

    pub fn key(&self) -> &'static str {
        match self {
            Material::GalvanisedSteel => "galvanised_steel",
            Material::StainlessSteel => "stainless_steel",
            Material::Copper => "copper",
            Material::CopperBondedSteel => "copper_bonded_steel",
            Material::Aluminium => "aluminium",
        }
    }

    pub fn from_key(key: &str) -> Option<Material> {
        let key = key.trim().replace(['-', ' '], "_");
        Self::ALL.into_iter().find(|m| m.key().eq_ignore_ascii_case(&key))
    }

    /// Untranslated title (`tr` where shown).
    pub fn title(&self) -> &'static str {
        match self {
            Material::GalvanisedSteel => tr_const!("Steel, hot-dip galvanised"),
            Material::StainlessSteel => tr_const!("Stainless steel V4A"),
            Material::Copper => tr_const!("Copper"),
            Material::CopperBondedSteel => tr_const!("Copper-bonded steel"),
            Material::Aluminium => tr_const!("Aluminium"),
        }
    }

    /// Whether the technical value `Material` of a part names this material.
    pub fn names(&self, text: &str) -> bool {
        let t = text.to_lowercase();
        match self {
            Material::GalvanisedSteel => t.contains("galvanised"),
            Material::StainlessSteel => t.contains("stainless"),
            Material::Copper => t.contains("copper") && !t.contains("copper-bonded"),
            Material::CopperBondedSteel => t.contains("copper-bonded"),
            Material::Aluminium => t.contains("aluminium"),
        }
    }
}

/// The diameter a part has by its technical data ("Diameter: 17.2 mm"), mm.
pub fn part_diameter(p: &Part) -> Option<f32> {
    let v = p.tech_value("Diameter")?;
    v.trim().split(|c: char| !(c.is_ascii_digit() || c == '.')).find(|t| !t.is_empty())?.parse().ok()
}

/// The part of the project or the built-in catalog of `subgroup` of `material` nearest to
/// `diameter` (`0`: one without a diameter, a flat strip); one of another material
/// where there is none of it.
pub fn pick_part(prj: &Project, subgroup: &str, material: Material, diameter: f32) -> Option<String> {
    let mut seen: Vec<&Part> = Vec::new();
    for p in prj.parts.iter().chain(crate::parts::catalog()) {
        if p.subgroup == subgroup && !seen.iter().any(|q| q.number == p.number) {
            seen.push(p);
        }
    }
    let of = |p: &&Part| p.tech_value("Material").is_some_and(|m| material.names(m));
    let cost = |p: &&Part| match (part_diameter(p), diameter > 0.0) {
        (Some(d), true) => (d - diameter).abs(),
        (None, false) => 0.0,
        _ => 1000.0,
    };
    let best = |list: Vec<&Part>| list.into_iter().min_by(|a, b| cost(a).total_cmp(&cost(b))).map(|p| p.number.clone());
    best(seen.iter().copied().filter(of).collect()).or_else(|| best(seen))
}

/// The material of electrode `e`: its own, otherwise galvanised steel for foundation
/// earth electrodes and rods, stainless steel for rings in the soil.
pub fn material_of(e: &Electrode) -> Material {
    e.material.unwrap_or(match e.kind {
        ElectrodeKind::Ring => Material::StainlessSteel,
        _ => Material::GalvanisedSteel,
    })
}

/// The diameter of electrode `e`: a rod's (20 mm unless given), that of a round wire
/// or 0 for a flat strip, mm.
pub fn diameter_of(e: &Electrode) -> f32 {
    match e.kind {
        ElectrodeKind::Rod if e.diameter <= 0.0 => 20.0,
        _ => e.diameter.max(0.0),
    }
}

impl ElectrodeKind {
    pub const ALL: [ElectrodeKind; 3] = [ElectrodeKind::Foundation, ElectrodeKind::Ring, ElectrodeKind::Rod];

    pub fn key(&self) -> &'static str {
        match self {
            ElectrodeKind::Foundation => "foundation",
            ElectrodeKind::Ring => "ring",
            ElectrodeKind::Rod => "rod",
        }
    }

    pub fn from_key(key: &str) -> Option<ElectrodeKind> {
        Self::ALL.into_iter().find(|k| k.key().eq_ignore_ascii_case(key.trim()))
    }

    /// Untranslated title (`tr` where shown).
    pub fn title(&self) -> &'static str {
        match self {
            ElectrodeKind::Foundation => tr_const!("Foundation earth electrode"),
            ElectrodeKind::Ring => tr_const!("Ring earth electrode"),
            ElectrodeKind::Rod => tr_const!("Earth rod"),
        }
    }

    /// The generic part of its strip, wire or rod.
    pub fn generic(&self) -> &'static str {
        match self {
            ElectrodeKind::Foundation => "GEN.ER-STRIP-FE30",
            ElectrodeKind::Ring => "GEN.ER-STRIP-V4A30",
            ElectrodeKind::Rod => "GEN.ER-ROD-1500",
        }
    }

    /// The subgroup of the parts of its strip, wire or rod.
    pub fn subgroup(&self) -> &'static str {
        match self {
            ElectrodeKind::Rod => "Earth rod",
            _ => "Earth electrode material",
        }
    }

    /// Default distance from the outer walls (foundation: inside, ring: outside), mm.
    pub fn default_distance(&self) -> f32 {
        match self {
            ElectrodeKind::Ring => 1000.0,
            _ => 100.0,
        }
    }
}

/// The rectangle of the outer walls of a building in its floor plan: the rectangles
/// its roofs cover, otherwise its rooms; `None` for a building without either.
pub fn walls(prj: &Project, poses: &[SitePose], bi: usize) -> Option<([f32; 2], [f32; 2])> {
    let layout = &prj.layout;
    let b = layout.buildings.get(bi)?;
    let mut pts: Vec<[f32; 2]> = b.roofs.iter().flat_map(|r| [[r.x, r.z], [r.x + r.width, r.z + r.depth]]).collect();
    if pts.is_empty() {
        let bp = building_pose(b);
        for ri in building_rooms(layout, bi) {
            pts.extend(outline(&layout.enclosures[ri]).into_iter().map(|[x, z]| {
                let v = bp.local(poses[ri].apply([x, 0.0, z]));
                [v[0], v[2]]
            }));
        }
    }
    (!pts.is_empty()).then(|| super::bounds(&pts))
}

/// Elevation of the lowest level of a building (its floor) above its ground level.
pub fn lowest_floor(b: &Building) -> f32 {
    b.levels.iter().map(|l| l.elevation).reduce(f32::min).unwrap_or(0.0)
}

/// The closed rectangle `(lo, hi)` grown by `by` (shrunk where negative) at height `y`.
fn ring(lo: [f32; 2], hi: [f32; 2], by: f32, y: f32) -> Vec<V3> {
    let (x0, z0, x1, z1) = (lo[0] - by, lo[1] - by, hi[0] + by, hi[1] + by);
    vec![[x0, y, z0], [x1, y, z0], [x1, y, z1], [x0, y, z1], [x0, y, z0]]
}

/// The lines of electrode `e` of building `bi` (building coordinates): its ring and the
/// strips dividing its meshes, or its rod; empty where it has nowhere to lie.
pub fn paths(prj: &Project, poses: &[SitePose], bi: usize, e: &Electrode) -> Vec<Vec<V3>> {
    let Some(b) = prj.layout.buildings.get(bi) else { return Vec::new() };
    match e.kind {
        ElectrodeKind::Rod => {
            let top = -e.depth.max(0.0);
            vec![vec![[e.x, top, e.z], [e.x, top - e.length.max(100.0), e.z]]]
        }
        ElectrodeKind::Ring => match walls(prj, poses, bi) {
            Some((lo, hi)) => vec![ring(lo, hi, e.distance.max(0.0), -e.depth.max(0.0))],
            None => Vec::new(),
        },
        ElectrodeKind::Foundation => {
            let Some((lo, hi)) = walls(prj, poses, bi) else { return Vec::new() };
            let y = lowest_floor(b) - SLAB + COVER;
            let d = e.distance.max(0.0);
            let mut out = vec![ring(lo, hi, -d, y)];
            let (x0, z0, x1, z1) = (lo[0] + d, lo[1] + d, hi[0] - d, hi[1] - d);
            let mesh = e.mesh.max(2000.0);
            // Strips across, so that no mesh is larger than allowed.
            for (from, to, along_x) in [(x0, x1, true), (z0, z1, false)] {
                let n = ((to - from) / mesh).ceil().max(1.0) as usize;
                for k in 1..n {
                    let t = from + (to - from) * k as f32 / n as f32;
                    out.push(if along_x { vec![[t, y, z0], [t, y, z1]] } else { vec![[x0, y, t], [x1, y, t]] });
                }
            }
            out
        }
    }
}

/// Length of lines, mm.
pub fn length(paths: &[Vec<V3>]) -> f32 {
    paths.iter().map(|p| p.windows(2).map(|w| dist(w[0], w[1])).sum::<f32>()).sum()
}

/// The point of the lines of an electrode nearest to `q` (building coordinates).
pub fn nearest(paths: &[Vec<V3>], q: V3) -> Option<V3> {
    paths
        .iter()
        .flat_map(|p| p.windows(2))
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let len2 = (ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2]).max(1e-9);
            let t = (((q[0] - a[0]) * ab[0] + (q[1] - a[1]) * ab[1] + (q[2] - a[2]) * ab[2]) / len2).clamp(0.0, 1.0);
            [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t]
        })
        .min_by(|a, b| dist(*a, q).total_cmp(&dist(*b, q)))
}

/// The conductor from electrode `e` up to its busbar (building coordinates): from the
/// point of the electrode nearest to the busbar straight up under it and onto its
/// bottom; `None` without a busbar placed in the layout.
pub fn busbar_lead(prj: &Project, poses: &[SitePose], bi: usize, e: &Electrode) -> Option<Vec<V3>> {
    let b = prj.layout.buildings.get(bi)?;
    let (_, (lo, hi)) = placed_device(prj, poses, &e.busbar)?;
    let bp = building_pose(b);
    let bottom = bp.local([(lo[0] + hi[0]) / 2.0, lo[1], (lo[2] + hi[2]) / 2.0]);
    let from = nearest(&paths(prj, poses, bi, e), bottom)?;
    Some(vec![from, [bottom[0], from[1], bottom[2]], bottom])
}

/// An electrode with its numbers in range.
pub fn checked(mut e: Electrode) -> Electrode {
    e.distance = e.distance.clamp(0.0, 10_000.0);
    e.depth = e.depth.clamp(0.0, 5000.0);
    e.mesh = e.mesh.clamp(2000.0, 50_000.0);
    e.length = e.length.clamp(500.0, 30_000.0);
    e.diameter = e.diameter.clamp(0.0, 60.0);
    e.part = e.part.trim().into();
    e.busbar = e.busbar.trim().into();
    e
}

/// A new electrode of `kind` for building `bi` (a rod at `at` of its floor plan,
/// otherwise at the rear left corner of its walls); its id.
pub fn add_electrode(prj: &mut Project, bi: usize, kind: ElectrodeKind, at: Option<[f32; 2]>) -> Result<u64, String> {
    let poses = super::site_poses(prj);
    let corner = walls(prj, &poses, bi).map_or([0.0, 0.0], |(lo, _)| [lo[0] - 1000.0, lo[1] - 1000.0]);
    let id = prj.new_id().max(1);
    let b = prj.layout.buildings.get_mut(bi).ok_or_else(|| tr("No such building").to_string())?;
    let name = (1..).map(|k| format!("E{k}")).find(|c| !b.electrodes.iter().any(|e| &e.name == c)).unwrap_or_default();
    let [x, z] = at.unwrap_or(corner);
    let diameter = if kind == ElectrodeKind::Rod { 20.0 } else { 0.0 };
    b.electrodes.push(checked(Electrode { id, name, kind, distance: kind.default_distance(), x, z, diameter, ..Default::default() }));
    Ok(id)
}

/// The part of electrode `e`'s strip, wire or rod: its own, otherwise the one of its
/// material and diameter.
pub fn part_of(prj: &Project, e: &Electrode) -> String {
    if !e.part.is_empty() {
        return e.part.clone();
    }
    pick_part(prj, e.kind.subgroup(), material_of(e), diameter_of(e)).unwrap_or_else(|| e.kind.generic().to_string())
}

/// The material of the electrodes of building `bi`, with their quantities: strips and
/// wires in metres, the spacers of foundation earth electrodes, the connectors at the
/// crossings of their strips, the sections, tips and clamps of the rods and a connection
/// lug for each connection to a busbar.
pub fn material(prj: &Project, poses: &[SitePose], bi: usize) -> Vec<(String, f32)> {
    let mut out: Vec<(String, f32)> = Vec::new();
    let mut add = |part: &str, qty: f32| {
        if qty <= 0.0 {
            return;
        }
        match out.iter_mut().find(|x| x.0 == part) {
            Some(x) => x.1 += qty,
            None => out.push((part.to_string(), qty)),
        }
    };
    let Some(b) = prj.layout.buildings.get(bi) else { return Vec::new() };
    for e in &b.electrodes {
        let lines = paths(prj, poses, bi, e);
        match e.kind {
            ElectrodeKind::Rod => {
                let sections = (e.length / ROD_SECTION).ceil().max(1.0);
                add(&part_of(prj, e), sections);
                add("GEN.ER-ROD-TIP", 1.0);
                add("GEN.ER-CLAMP-ROD", 1.0);
            }
            ElectrodeKind::Foundation | ElectrodeKind::Ring => {
                let mm = length(&lines);
                add(&part_of(prj, e), (mm / 1000.0 * 100.0).ceil() / 100.0);
                // The strips across meet the ring at both ends and each other.
                let across = lines.len().saturating_sub(1) as f32;
                let xs = lines.iter().skip(1).filter(|l| (l[0][0] - l[1][0]).abs() < 0.5).count() as f32;
                let crossings = 2.0 * across + xs * (across - xs);
                add("GEN.ER-CROSS", crossings);
                if e.kind == ElectrodeKind::Foundation {
                    add("GEN.ER-SPACER", (mm / SPACER).ceil());
                }
            }
        }
        if !e.busbar.is_empty() {
            add("GEN.FE-LUG-V4A-10", 1.0);
        }
    }
    out
}

/// Electrodes whose busbar is missing, foundation and ring earth electrodes of
/// buildings without walls and earth rods inside the walls of their building.
pub fn problems(prj: &Project) -> Vec<Issue> {
    let poses = super::site_poses(prj);
    let mut out = Vec::new();
    for (bi, b) in prj.layout.buildings.iter().enumerate() {
        let rect = walls(prj, &poses, bi);
        for e in &b.electrodes {
            if !e.busbar.is_empty() && !prj.devices.contains_key(&e.busbar) {
                out.push(crate::issue!(Severity::Warning, None, "{}: the busbar {} of earth electrode {} does not exist", b.name, e.busbar, e.name));
            }
            match (e.kind, rect) {
                (ElectrodeKind::Foundation | ElectrodeKind::Ring, None) => {
                    out.push(crate::issue!(Severity::Warning, None, "{}: earth electrode {} has no walls to run along: the building has no rooms or roofs", b.name, e.name));
                }
                (ElectrodeKind::Rod, Some((lo, hi))) if e.x > lo[0] && e.x < hi[0] && e.z > lo[1] && e.z < hi[1] => {
                    out.push(crate::issue!(Severity::Warning, None, "{}: earth rod {} stands inside the walls of the building", b.name, e.name));
                }
                _ => {}
            }
        }
    }
    out
}

/// Whether building `b` has an earth electrode.
pub fn earthed(b: &Building) -> bool {
    !b.electrodes.is_empty()
}

#[cfg(test)]
mod tests;
