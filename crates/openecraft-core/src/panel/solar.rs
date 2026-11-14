//! Solar (PV) arrays ([`PvArray`]) on roofs: rows of modules on one slope of a roof,
//! on rails parallel to the covering fastened with roof hooks, or on flat roofs tilted
//! towards one side or east-west on ballasted mounting bases; and freestanding on the
//! plot: rows of tilted tables, the modules on rails over table beams on rammed posts.
//!
//! An array lies on the slope under its middle, a freestanding one on the ground level
//! of the plant ([`Support`]). Its modules, rails, mounting bases and posts come out as
//! faces for the 3D view ([`faces_on`]); its mounting material ([`material`]) goes into
//! the bill of materials: the parts of its mounting system (a rail, a mounting base or a
//! post whose accessories are the hooks, beams, clamps, connectors and ballast) or
//! generic ones.

use super::buildings::building_pose;
use super::roofs::{self, inside_lins, normal_of, Lin, RoofFace, RoofFaceKind};
use super::SitePose;
use crate::issue::{Issue, Severity};
use crate::model::panel::{Building, Layout, PvArray, PvMounting, Roof};
use crate::model::Project;
use crate::parts::vec3::{add, cross, dot, norm, scale, sub, V3};
use crate::parts::Part;
use openecraft_i18n::{tr, tr_const, tr_fmt};

pub mod cabling;
pub mod strings;

/// Gap between neighbouring modules (the clamps), mm.
pub const CLAMP_GAP: f32 = 20.0;
/// Underside of the modules above the covering on rails and roof hooks, mm.
pub const STANDOFF: f32 = 110.0;
/// Low edge of tilted modules above the covering, mm.
pub const BASE_HEIGHT: f32 = 150.0;
/// Size of the rail profile, mm.
pub const RAIL: f32 = 40.0;
/// How far a rail reaches out beyond the outer modules, mm.
pub const RAIL_END: f32 = 50.0;
/// Length of a rail piece sold by the metre, mm (for the rail connectors).
pub const RAIL_PIECE: f32 = 6000.0;
/// East-west mounting: gap between the high edges of a pair and walkway between pairs, mm.
pub const RIDGE_GAP: f32 = 100.0;
pub const WALKWAY: f32 = 300.0;
/// Height of the sun at noon in winter for the distance of tilted rows, degrees.
pub const WINTER_SUN: f32 = 17.0;
/// Roofs flatter than this count as flat for tilted mounting, degrees.
pub const FLAT_PITCH: f32 = 5.0;
/// Weight of a ballast block without a part, kg.
pub const BALLAST_BLOCK: f32 = 12.5;
/// Freestanding tables: size of the posts and of the table beams, how far the outer
/// posts stand inside the ends of a table, and the height of the earthing point on a
/// post above the ground, mm.
pub const POST: f32 = 100.0;
pub const BEAM: f32 = 80.0;
pub const POST_INSET: f32 = 300.0;
pub const EARTH_HEIGHT: f32 = 300.0;
/// Free space in front of what stands in the plant for a new freestanding array, mm.
pub const GROUND_GAP: f32 = 3000.0;

impl PvMounting {
    pub const ALL: [PvMounting; 4] = [PvMounting::Parallel, PvMounting::Tilted, PvMounting::EastWest, PvMounting::Ground];
    /// The mountings on a roof (freestanding arrays stand on posts).
    pub const ROOF: [PvMounting; 3] = [PvMounting::Parallel, PvMounting::Tilted, PvMounting::EastWest];

    pub fn key(&self) -> &'static str {
        match self {
            PvMounting::Parallel => "parallel",
            PvMounting::Tilted => "tilted",
            PvMounting::EastWest => "east_west",
            PvMounting::Ground => "ground",
        }
    }

    pub fn from_key(key: &str) -> Option<PvMounting> {
        let key = key.trim().replace('-', "_");
        Self::ALL.into_iter().find(|m| m.key().eq_ignore_ascii_case(&key))
    }

    /// Untranslated title (`tr` where shown).
    pub fn title(&self) -> &'static str {
        match self {
            PvMounting::Parallel => tr_const!("Parallel to the roof on roof hooks"),
            PvMounting::Tilted => tr_const!("Tilted on a flat roof"),
            PvMounting::EastWest => tr_const!("East-west on a flat roof"),
            PvMounting::Ground => tr_const!("Freestanding on posts"),
        }
    }

    /// Tilted on mounting bases (flat roofs) or on posts (freestanding).
    pub fn is_tilted(&self) -> bool {
        *self != PvMounting::Parallel
    }

    /// The part a mounting system is named by: the rail, the mounting base or the post.
    pub fn main_role(&self) -> Role {
        match self {
            PvMounting::Parallel => Role::Rail,
            PvMounting::Tilted | PvMounting::EastWest => Role::Base,
            PvMounting::Ground => Role::Post,
        }
    }
}

/// A part of the mounting of a PV array.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Rail,
    Hook,
    Connector,
    EndClamp,
    MidClamp,
    Base,
    Ballast,
    Deflector,
    /// The earthing clamps that bond the rails or the rows of mounting bases to the
    /// earthing conductor.
    Earthing,
    /// Freestanding tables: the rammed posts and the table beams on them under the rails.
    Post,
    Beam,
}

impl Role {
    pub const ALL: [Role; 11] = [Role::Rail, Role::Hook, Role::Connector, Role::EndClamp, Role::MidClamp, Role::Base, Role::Ballast, Role::Deflector, Role::Earthing, Role::Post, Role::Beam];

    /// The subgroup of its parts.
    pub fn subgroup(&self) -> &'static str {
        match self {
            Role::Rail => "PV mounting rail",
            Role::Hook => "Roof hook",
            Role::Connector => "PV rail connector",
            Role::EndClamp => "PV end clamp",
            Role::MidClamp => "PV middle clamp",
            Role::Base => "PV mounting base",
            Role::Ballast => "Ballast",
            Role::Deflector => "Wind deflector",
            Role::Earthing => "PV earthing clamp",
            Role::Post => "PV ground post",
            Role::Beam => "PV table beam",
        }
    }

    /// The generic part of the role (for a mounting).
    pub fn generic(&self, m: PvMounting) -> &'static str {
        match self {
            Role::Rail => "GEN.PV-RAIL-40",
            Role::Hook => "GEN.PV-HOOK",
            Role::Connector => "GEN.PV-RAIL-CONN",
            Role::EndClamp => "GEN.PV-CLAMP-END",
            Role::MidClamp => "GEN.PV-CLAMP-MID",
            Role::Base if m == PvMounting::EastWest => "GEN.PV-BASE-EW10",
            Role::Base => "GEN.PV-BASE-S15",
            Role::Ballast => "GEN.PV-BALLAST",
            Role::Deflector => "GEN.PV-DEFLECTOR",
            Role::Earthing => "GEN.PV-EARTH-CLAMP",
            Role::Post => "GEN.PV-POST-RAM",
            Role::Beam => "GEN.PV-TABLE-BEAM",
        }
    }
}

/// The modules and the mounting of an array (building floor plan, y up from its ground
/// level).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PvGeometry {
    /// The top of each module, counter-clockwise seen from above.
    pub modules: Vec<[V3; 4]>,
    /// Axis of each rail under the modules (its middle), from end to end.
    pub rails: Vec<(V3, V3)>,
    /// Mounting bases and wind deflectors as upright plates.
    pub plates: Vec<[V3; 4]>,
    /// The roof hooks under the rails: the foot on the covering and the top under the rail.
    pub hooks: Vec<(V3, V3)>,
    /// Freestanding tables: the posts (the foot on the ground and the top under the table
    /// beam) and the axis of each table beam under the rails, from the front to the rear.
    pub posts: Vec<(V3, V3)>,
    pub beams: Vec<(V3, V3)>,
    /// The earthing points of the mounting, one on each rail or row of mounting bases,
    /// in the order the earthing conductor bonds them.
    pub earth: Vec<V3>,
    /// The outline of the array in the floor plan.
    pub outline: Vec<[f32; 2]>,
    /// The plane of the roof's underside under the array.
    pub plane: Lin,
}

/// Size of a module along the rows and across them (portrait: the short side along the
/// rows).
fn module_size(a: &PvArray) -> (f32, f32) {
    let (w, l) = (a.module_width.min(a.module_length), a.module_width.max(a.module_length));
    if a.landscape { (l, w) } else { (w, l) }
}

/// Number of modules of an array.
pub fn module_count(a: &PvArray) -> u32 {
    let n = a.columns.max(1) * a.rows.max(1);
    match a.mounting {
        PvMounting::EastWest => 2 * n,
        PvMounting::Ground => a.stack.max(1) * n,
        _ => n,
    }
}

/// Length of a row of modules up its slope: of a table of freestanding modules on top of
/// each other, otherwise of a module.
fn table_length(a: &PvArray) -> f32 {
    let (_, across) = module_size(a);
    if a.mounting == PvMounting::Ground {
        let n = a.stack.max(1) as f32;
        n * across + (n - 1.0) * CLAMP_GAP
    } else {
        across
    }
}

/// The direction up tilted modules in the floor plan: away from the direction they face.
fn facing_up(a: &PvArray) -> [f32; 2] {
    let t = a.azimuth.to_radians();
    [-t.sin(), -t.cos()]
}

/// The plane under the middle of an array and the direction up it in the floor plan
/// (on flat roofs away from the direction the modules face).
fn frame(r: &Roof, a: &PvArray) -> Option<(Lin, [f32; 2])> {
    let c = [a.x, a.z];
    roofs::underside_at(r, c)?;
    let plane = *roofs::planes(r).iter().min_by(|p, q| p.at(c).total_cmp(&q.at(c)))?;
    let up = if plane.slope() < FLAT_PITCH.to_radians().tan() || a.mounting.is_tilted() { facing_up(a) } else { [plane.a / plane.slope(), plane.b / plane.slope()] };
    Some((plane, up))
}

/// Distance from row to row of a tilted, east-west or freestanding array in the floor
/// plan.
pub fn row_pitch(a: &PvArray) -> f32 {
    let (_, across) = module_size(a);
    let table = table_length(a);
    let t = a.tilt.clamp(0.0, 60.0).to_radians();
    let depth = table * t.cos();
    match a.mounting {
        PvMounting::Parallel => across + CLAMP_GAP,
        _ if a.row_gap > 0.0 => a.row_gap,
        PvMounting::Tilted | PvMounting::Ground => depth + table * t.sin() / WINTER_SUN.to_radians().tan(),
        PvMounting::EastWest => 2.0 * depth + RIDGE_GAP + WALKWAY,
    }
}

/// The posts along a table of a freestanding array: at most the span apart, the outer
/// ones inside the ends of the table (positions along the rows from its middle).
fn post_places(a: &PvArray) -> Vec<f32> {
    let (along, _) = module_size(a);
    let cols = a.columns.max(1) as f32;
    let width = cols * along + (cols - 1.0) * CLAMP_GAP;
    let inset = (width / 4.0).min(POST_INSET);
    let free = width - 2.0 * inset;
    let n = (free / a.span.max(300.0)).ceil().max(1.0) as usize + 1;
    (0..n).map(|i| -free / 2.0 + free * i as f32 / (n - 1) as f32).collect()
}

/// The modules and mounting of array `a` on roof `r`; `None` where its middle is off the
/// roof.
pub fn geometry(r: &Roof, a: &PvArray) -> Option<PvGeometry> {
    let (plane, s) = frame(r, a)?;
    let sec = (1.0 + plane.slope() * plane.slope()).sqrt();
    Some(layout_on(plane, s, r.thickness.max(1.0) * sec, a))
}

/// The modules and mounting of the freestanding array `a` on the ground level of the
/// plant (plant coordinates, y up).
pub fn ground_geometry(a: &PvArray) -> PvGeometry {
    layout_on(Lin::constant(0.0), facing_up(a), 0.0, a)
}

/// The modules and mounting of array `a` on the covering `lift` (vertically) above
/// `plane`, `s` the direction up its rows in the floor plan.
fn layout_on(plane: Lin, s: [f32; 2], lift: f32, a: &PvArray) -> PvGeometry {
    let e = [-s[1], s[0]];
    let sec = (1.0 + plane.slope() * plane.slope()).sqrt();
    let c = [a.x, a.z];
    // A point `u` along the rows and `v` up the slope (both in the floor plan), `h` above
    // the covering (vertically).
    let at = |u: f32, v: f32, h: f32| {
        let q = [c[0] + e[0] * u + s[0] * v, c[1] + e[1] * u + s[1] * v];
        [q[0], plane.at(q) + lift + h, q[1]]
    };
    let plan = |u: f32, v: f32| [c[0] + e[0] * u + s[0] * v, c[1] + e[1] * u + s[1] * v];
    let (along, across) = module_size(a);
    let (cols, rows) = (a.columns.max(1), a.rows.max(1));
    let width = cols as f32 * along + (cols - 1) as f32 * CLAMP_GAP;
    let h = a.module_height.max(5.0);
    let mut g = PvGeometry { plane, ..Default::default() };
    // Left edge of each module along the rows.
    let lefts: Vec<f32> = (0..cols).map(|i| -width / 2.0 + i as f32 * (along + CLAMP_GAP)).collect();
    match a.mounting {
        PvMounting::Parallel => {
            // Lengths up the slope in the floor plan.
            let run = across / sec;
            let gap = CLAMP_GAP / sec;
            let depth = rows as f32 * run + (rows - 1) as f32 * gap;
            let top = (STANDOFF + h) * sec;
            for j in 0..rows {
                let v0 = -depth / 2.0 + j as f32 * (run + gap);
                for &u0 in &lefts {
                    g.modules.push([at(u0, v0, top), at(u0 + along, v0, top), at(u0 + along, v0 + run, top), at(u0, v0 + run, top)]);
                }
            }
            let y = (STANDOFF - RAIL / 2.0) * sec;
            // The rails in the floor plan: (u, v) of both ends.
            let mut ends: Vec<([f32; 2], [f32; 2])> = Vec::new();
            if a.landscape {
                // Two rails up the slope under each column of modules.
                let end = RAIL_END / sec;
                for &u0 in &lefts {
                    for k in [0.25, 0.75] {
                        let u = u0 + along * k;
                        ends.push(([u, -depth / 2.0 - end], [u, depth / 2.0 + end]));
                    }
                }
            } else {
                // Two rails along each row of modules.
                for j in 0..rows {
                    let v0 = -depth / 2.0 + j as f32 * (run + gap);
                    for k in [0.25, 0.75] {
                        let v = v0 + run * k;
                        ends.push(([-width / 2.0 - RAIL_END, v], [width / 2.0 + RAIL_END, v]));
                    }
                }
            }
            let hooks = |len: f32| ((len - 2.0 * RAIL_END) / a.span.max(300.0)).ceil().max(1.0) as usize + 1;
            for &(p, q) in &ends {
                g.rails.push((at(p[0], p[1], y), at(q[0], q[1], y)));
                // The roof hooks evenly from end to end, inside the overhang of the rail.
                let len = ((q[0] - p[0]) * (q[0] - p[0]) + (q[1] - p[1]) * (q[1] - p[1]) * sec * sec).sqrt();
                let n = hooks(len);
                let inset = (RAIL_END / len.max(1.0)).min(0.4);
                for i in 0..n {
                    let t = inset + (1.0 - 2.0 * inset) * i as f32 / (n - 1).max(1) as f32;
                    let (u, v) = (p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t);
                    g.hooks.push((at(u, v, 0.0), at(u, v, (STANDOFF - RAIL) * sec)));
                }
                // The earthing point on top of the rail, in its overhang beyond the modules.
                let t = (RAIL_END * 0.5 / len.max(1.0)).min(0.5);
                g.earth.push(at(p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t, STANDOFF * sec));
            }
            g.outline = vec![plan(-width / 2.0, -depth / 2.0), plan(width / 2.0, -depth / 2.0), plan(width / 2.0, depth / 2.0), plan(-width / 2.0, depth / 2.0)];
        }
        PvMounting::Tilted | PvMounting::EastWest => {
            let t = a.tilt.clamp(0.0, 60.0).to_radians();
            let (run, rise) = (across * t.cos(), across * t.sin());
            let pitch = row_pitch(a);
            let pair = a.mounting == PvMounting::EastWest;
            let unit = if pair { 2.0 * run + RIDGE_GAP } else { run };
            let depth = (rows - 1) as f32 * pitch + unit;
            let low = BASE_HEIGHT + h;
            // The base positions of a row: at the gaps and the outer edges.
            let bases: Vec<f32> = std::iter::once(-width / 2.0).chain(lefts.iter().skip(1).map(|u| u - CLAMP_GAP / 2.0)).chain(std::iter::once(width / 2.0)).collect();
            for j in 0..rows {
                let v0 = -depth / 2.0 + j as f32 * pitch;
                // Each sloping module surface: from its low edge `a` to its high edge `b`.
                let sides: Vec<(f32, f32)> = if pair { vec![(v0, v0 + run), (v0 + unit, v0 + unit - run)] } else { vec![(v0, v0 + run)] };
                for &(va, vb) in &sides {
                    for &u0 in &lefts {
                        let q = [at(u0, va, low), at(u0 + along, va, low), at(u0 + along, vb, low + rise), at(u0, vb, low + rise)];
                        g.modules.push(q);
                    }
                    for &u in &bases {
                        g.plates.push([at(u, va, 0.0), at(u, vb, 0.0), at(u, vb, BASE_HEIGHT + rise), at(u, va, BASE_HEIGHT)]);
                    }
                }
                if !pair {
                    // The wind deflector along the high edge.
                    g.plates.push([at(-width / 2.0, v0 + run, 0.0), at(width / 2.0, v0 + run, 0.0), at(width / 2.0, v0 + run, BASE_HEIGHT + rise), at(-width / 2.0, v0 + run, BASE_HEIGHT + rise)]);
                }
                // The earthing point on the outer base of the row.
                g.earth.push(at(-width / 2.0, v0 + run / 2.0, BASE_HEIGHT / 2.0));
            }
            g.outline = vec![plan(-width / 2.0, -depth / 2.0), plan(width / 2.0, -depth / 2.0), plan(width / 2.0, depth / 2.0), plan(-width / 2.0, depth / 2.0)];
        }
        PvMounting::Ground => {
            let t = a.tilt.clamp(0.0, 60.0).to_radians();
            let (ct, st) = (t.cos(), t.sin());
            let table = table_length(a);
            let pitch = row_pitch(a);
            let depth = (rows - 1) as f32 * pitch + table * ct;
            // Up a table and square to it (upwards).
            let up = [s[0] * ct, st, s[1] * ct];
            let mut n = norm(cross(up, [e[0], 0.0, e[1]]));
            if n[1] < 0.0 {
                n = scale(n, -1.0);
            }
            let low = a.clearance;
            // A point `w` up the top of the modules of the table starting at `v0`, `u`
            // along it and `d` below.
            let on = |u: f32, v0: f32, w: f32, d: f32| sub(at(u, v0 + w * ct, low + w * st), scale(n, d));
            let posts = post_places(a);
            let below = h + RAIL + BEAM / 2.0;
            for j in 0..rows {
                let v0 = -depth / 2.0 + j as f32 * pitch;
                for k in 0..a.stack.max(1) {
                    let w0 = k as f32 * (across + CLAMP_GAP);
                    for &u0 in &lefts {
                        g.modules.push([on(u0, v0, w0, 0.0), on(u0 + along, v0, w0, 0.0), on(u0 + along, v0, w0 + across, 0.0), on(u0, v0, w0 + across, 0.0)]);
                    }
                    // Two rails along each row of modules of the table.
                    for f in [0.25, 0.75] {
                        let w = w0 + across * f;
                        g.rails.push((on(-width / 2.0 - RAIL_END, v0, w, h + RAIL / 2.0), on(width / 2.0 + RAIL_END, v0, w, h + RAIL / 2.0)));
                    }
                }
                // A table beam under the rails at each post place, on a front and a rear post.
                for (i, &u) in posts.iter().enumerate() {
                    g.beams.push((on(u, v0, -RAIL_END, below), on(u, v0, table + RAIL_END, below)));
                    for f in [0.2, 0.8] {
                        let top = on(u, v0, table * f, below + BEAM / 2.0);
                        let foot = [top[0], plane.at([top[0], top[2]]) + lift, top[2]];
                        g.posts.push((foot, top));
                        if i == 0 && f < 0.5 {
                            // The earthing point on the outer side of the first front post.
                            g.earth.push([foot[0] - e[0] * POST / 2.0, foot[1] + EARTH_HEIGHT, foot[2] - e[1] * POST / 2.0]);
                        }
                    }
                }
            }
            g.outline = vec![plan(-width / 2.0, -depth / 2.0), plan(width / 2.0, -depth / 2.0), plan(width / 2.0, depth / 2.0), plan(-width / 2.0, depth / 2.0)];
        }
    }
    for m in &mut g.modules {
        if normal_of(m)[1] < 0.0 {
            m.reverse();
        }
    }
    g
}

/// What a PV array stands on: a roof of a building, or the ground of the plot
/// (freestanding).
#[derive(Clone, Copy, Debug)]
pub enum Support<'a> {
    Roof(&'a Building, &'a Roof),
    Ground,
}

impl Support<'_> {
    /// The modules and mounting of `a` on it (in the floor plan of the building, or of
    /// the plant).
    pub fn geometry(&self, a: &PvArray) -> Option<PvGeometry> {
        match self {
            Support::Roof(_, r) => geometry(r, a),
            Support::Ground => Some(ground_geometry(a)),
        }
    }

    /// From the floor plan its arrays are placed in to the plant.
    pub fn pose(&self) -> SitePose {
        match self {
            Support::Roof(b, _) => building_pose(b),
            Support::Ground => SitePose::default(),
        }
    }
}

/// Where a PV array is: `index` among the arrays of roof `roof.1` of building `roof.0`,
/// or (no roof) among the freestanding arrays of the plot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ArrayRef {
    pub roof: Option<(usize, usize)>,
    pub index: usize,
}

/// The arrays of roof `roof` (building, roof), or (`None`) the freestanding ones.
pub fn arrays_at(layout: &Layout, roof: Option<(usize, usize)>) -> Option<&Vec<PvArray>> {
    match roof {
        Some((bi, k)) => layout.buildings.get(bi).and_then(|b| b.roofs.get(k)).map(|r| &r.solar),
        None => Some(&layout.plot.solar),
    }
}

pub fn arrays_at_mut(layout: &mut Layout, roof: Option<(usize, usize)>) -> Option<&mut Vec<PvArray>> {
    match roof {
        Some((bi, k)) => layout.buildings.get_mut(bi).and_then(|b| b.roofs.get_mut(k)).map(|r| &mut r.solar),
        None => Some(&mut layout.plot.solar),
    }
}

/// The array at `at`.
pub fn array(layout: &Layout, at: ArrayRef) -> Option<&PvArray> {
    arrays_at(layout, at.roof)?.get(at.index)
}

pub fn array_mut(layout: &mut Layout, at: ArrayRef) -> Option<&mut PvArray> {
    arrays_at_mut(layout, at.roof)?.get_mut(at.index)
}

/// What the arrays of roof `roof` (or the freestanding ones) stand on.
pub fn support(layout: &Layout, roof: Option<(usize, usize)>) -> Option<Support<'_>> {
    match roof {
        Some((bi, k)) => {
            let b = layout.buildings.get(bi)?;
            Some(Support::Roof(b, b.roofs.get(k)?))
        }
        None => Some(Support::Ground),
    }
}

/// Every PV array of the plant, those on the roofs first: where it is, what it stands on
/// and the array.
pub fn all_arrays(layout: &Layout) -> Vec<(ArrayRef, Support<'_>, &PvArray)> {
    let mut out = Vec::new();
    for (bi, b) in layout.buildings.iter().enumerate() {
        for (k, r) in b.roofs.iter().enumerate() {
            out.extend(r.solar.iter().enumerate().map(|(index, a)| (ArrayRef { roof: Some((bi, k)), index }, Support::Roof(b, r), a)));
        }
    }
    out.extend(layout.plot.solar.iter().enumerate().map(|(index, a)| (ArrayRef { roof: None, index }, Support::Ground, a)));
    out
}

/// Where the PV array with id `id` is.
pub fn find(layout: &Layout, id: u64) -> Option<ArrayRef> {
    all_arrays(layout).into_iter().find(|t| t.2.id == id).map(|t| t.0)
}

/// What the messages about the arrays on `s` name first: the building, or the plot (its
/// name, otherwise "Plot").
pub fn place_name(layout: &Layout, s: Support) -> String {
    match s {
        Support::Roof(b, _) => b.name.clone(),
        Support::Ground if layout.plot.name.trim().is_empty() => tr("Plot").to_string(),
        Support::Ground => layout.plot.name.clone(),
    }
}

/// The corners of the modules of an array in the floor plan.
pub fn module_outlines(r: &Roof, a: &PvArray) -> Vec<[[f32; 2]; 4]> {
    geometry(r, a).map(|g| g.modules.iter().map(|m| m.map(|v| [v[0], v[2]])).collect()).unwrap_or_default()
}

/// The corners of the modules of a freestanding array in the plant.
pub fn ground_module_outlines(a: &PvArray) -> Vec<[[f32; 2]; 4]> {
    ground_geometry(a).modules.iter().map(|m| m.map(|v| [v[0], v[2]])).collect()
}

fn face(pts: Vec<V3>, kind: RoofFaceKind, pose: SitePose) -> RoofFace {
    let n = pose.turn(normal_of(&pts));
    RoofFace { pts: pts.into_iter().map(|v| pose.apply(v)).collect(), normal: n, outward: n, kind }
}

/// A box over the quad `top` down by `depth` along its normal: the top (cells inset in
/// the frame), the frame round them, the sides and the bottom.
fn module_faces(top: &[V3; 4], depth: f32, out: &mut Vec<RoofFace>, pose: SitePose) {
    let n = normal_of(top);
    let down = |v: V3| sub(v, scale(n, depth));
    let m = [0, 1, 2].map(|k| top.iter().map(|p| p[k]).sum::<f32>() / 4.0);
    // The cells inside the frame (about 15 mm wide on each side).
    let span = (sub(top[1], top[0]), sub(top[3], top[0]));
    let size = (crate::parts::vec3::length(span.0).max(1.0), crate::parts::vec3::length(span.1).max(1.0));
    let inner: Vec<V3> = top
        .iter()
        .map(|&p| {
            let d = sub(p, m);
            let k = 1.0 - 30.0 / size.0.min(size.1);
            add(add(m, scale(d, k)), scale(n, 1.0))
        })
        .collect();
    out.push(face(inner.clone(), RoofFaceKind::Module, pose));
    for k in 0..4 {
        let (a, b) = (top[k], top[(k + 1) % 4]);
        out.push(face(vec![a, b, inner[(k + 1) % 4], inner[k]], RoofFaceKind::ModuleFrame, pose));
        out.push(face(vec![down(a), down(b), b, a].into_iter().rev().collect(), RoofFaceKind::ModuleFrame, pose));
    }
    out.push(face(top.iter().rev().map(|&v| down(v)).collect(), RoofFaceKind::ModuleFrame, pose));
}

/// A bar of square section `size` from `p` to `q` (its axis): its four sides and its
/// ends; `level` is a direction across it (the side faces stand square to it).
fn bar(p: V3, q: V3, size: f32, level: V3, kind: RoofFaceKind, out: &mut Vec<RoofFace>, pose: SitePose) {
    let along = norm(sub(q, p));
    let mut side = norm(cross(along, level));
    if side == [0.0; 3] {
        side = norm(cross(along, [1.0, 0.0, 0.0]));
    }
    let up = norm(cross(side, along));
    let (hs, hu) = (scale(side, size / 2.0), scale(up, size / 2.0));
    let corners = [sub(sub(p, hs), hu), add(sub(p, hu), hs), add(add(p, hs), hu), sub(add(p, hu), hs)];
    let off = sub(q, p);
    for k in 0..4 {
        let (c0, c1) = (corners[k], corners[(k + 1) % 4]);
        out.push(face(vec![c0, c1, add(c1, off), add(c0, off)], kind, pose));
    }
    out.push(face(corners.iter().rev().copied().collect(), kind, pose));
    out.push(face(corners.iter().map(|&c| add(c, off)).collect(), kind, pose));
}

/// The faces of the PV arrays of a roof in the plant: modules (cells and frames, with
/// their junction boxes), rails, roof hooks, mounting bases, wind deflectors and the
/// earthing clamps.
pub fn faces(b: &Building, r: &Roof) -> Vec<RoofFace> {
    faces_of(b, r, &r.solar)
}

/// The faces of `arrays` of roof `r` (see [`faces`]).
pub fn faces_of(b: &Building, r: &Roof, arrays: &[PvArray]) -> Vec<RoofFace> {
    faces_on(Support::Roof(b, r), arrays)
}

/// The faces of `arrays` standing on `s` in the plant (see [`faces`]); freestanding
/// tables with their posts and table beams.
pub fn faces_on(s: Support, arrays: &[PvArray]) -> Vec<RoofFace> {
    let pose = s.pose();
    let mut out = Vec::new();
    for a in arrays {
        let Some(g) = s.geometry(a) else { continue };
        let h = a.module_height.max(5.0);
        for m in &g.modules {
            module_faces(m, h, &mut out, pose);
            // The junction box on the back, near the upper edge.
            let (n, c) = (normal_of(m), junction(m, h));
            let top = sub(c, scale(n, JUNCTION));
            bar(c, top, 90.0, sub(m[1], m[0]), RoofFaceKind::ModuleFrame, &mut out, pose);
        }
        // Square to the covering, upwards.
        let n = norm([-g.plane.a, 1.0, -g.plane.b]);
        for &(p, q) in &g.rails {
            bar(p, q, RAIL, [0.0, 1.0, 0.0], RoofFaceKind::Mounting, &mut out, pose);
        }
        for &(foot, top) in &g.hooks {
            // The hook from the covering up to the rail and its plate on the rafter.
            let toward = norm(sub(top, foot));
            bar(foot, top, 30.0, if dot(toward, n).abs() > 0.99 { [1.0, 0.0, 0.0] } else { n }, RoofFaceKind::Mounting, &mut out, pose);
            bar(foot, add(foot, scale(n, 6.0)), 70.0, [1.0, 0.0, 0.0], RoofFaceKind::Mounting, &mut out, pose);
        }
        // Posts square to the rows, the table beams standing upright.
        let rows = g.modules.first().map_or([1.0, 0.0, 0.0], |m| sub(m[1], m[0]));
        for &(foot, top) in &g.posts {
            bar(foot, top, POST, rows, RoofFaceKind::Mounting, &mut out, pose);
        }
        for &(p, q) in &g.beams {
            bar(p, q, BEAM, [0.0, 1.0, 0.0], RoofFaceKind::Mounting, &mut out, pose);
        }
        for plate in &g.plates {
            // Seen from both sides.
            out.push(face(plate.to_vec(), RoofFaceKind::Mounting, pose));
            out.push(face(plate.iter().rev().copied().collect(), RoofFaceKind::Mounting, pose));
        }
        for &e in &g.earth {
            bar(e, add(e, scale(n, EARTH_CLAMP)), EARTH_CLAMP, [1.0, 0.0, 0.0], RoofFaceKind::Earthing, &mut out, pose);
        }
    }
    out
}

/// Depth of the junction box on the back of a module, mm.
pub const JUNCTION: f32 = 18.0;
/// Size of an earthing clamp on a rail or a mounting base, mm.
pub const EARTH_CLAMP: f32 = 30.0;

/// The point of the back of a module (`top` its upper face, `h` its height) where its
/// junction box sits: in the middle, a fifth of the way from its upper edge.
fn junction(top: &[V3; 4], h: f32) -> V3 {
    let n = normal_of(top);
    let m = [0, 1, 2].map(|k| top.iter().map(|p| p[k]).sum::<f32>() / 4.0);
    // The upper edge: the one of the two pairs of opposite edges whose middle is highest.
    let mids: Vec<V3> = (0..4).map(|k| scale(add(top[k], top[(k + 1) % 4]), 0.5)).collect();
    let high = mids.iter().copied().max_by(|a, b| a[1].total_cmp(&b[1])).unwrap_or(m);
    let c = add(m, scale(sub(high, m), 0.6));
    sub(c, scale(n, h))
}

/// A part of the project or of the built-in catalog.
fn find_part<'a>(prj: &'a Project, number: &str) -> Option<&'a Part> {
    prj.parts.iter().chain(crate::parts::catalog()).find(|p| p.number == number)
}

/// The range of module frame heights a clamp takes (technical data "Module frame
/// height: 30 - 40 mm").
fn frame_range(p: &Part) -> Option<(f32, f32)> {
    let v = p.tech_value("Module frame height")?;
    let nums: Vec<f32> = v.split(|c: char| !(c.is_ascii_digit() || c == '.')).filter_map(|t| t.parse().ok()).collect();
    match nums.as_slice() {
        [a, b, ..] => Some((*a, *b)),
        [a] => Some((*a, *a)),
        _ => None,
    }
}

/// The part of `role` in the mounting of an array: the system's own part, a fitting
/// accessory of it, otherwise the generic one.
pub fn part_for(prj: &Project, a: &PvArray, role: Role) -> String {
    let main = a.mounting.main_role();
    let system = (!a.system.is_empty()).then(|| find_part(prj, &a.system)).flatten();
    if let Some(sys) = system {
        if role == main {
            return sys.number.clone();
        }
        let fits = |p: &Part| frame_range(p).is_none_or(|(lo, hi)| a.module_height >= lo - 0.5 && a.module_height <= hi + 0.5);
        let candidates: Vec<&Part> = sys.accessories.iter().filter_map(|n| find_part(prj, n)).filter(|p| p.subgroup == role.subgroup()).collect();
        if let Some(p) = candidates.iter().find(|p| fits(p)).or(candidates.first()) {
            return p.number.clone();
        }
    }
    role.generic(a.mounting).to_string()
}

/// The roles of the mounting parts of an array with their quantities, without the
/// modules.
pub fn mounting(prj: &Project, a: &PvArray) -> Vec<(Role, f32)> {
    let (cols, rows) = (a.columns.max(1) as f32, a.rows.max(1) as f32);
    let modules = module_count(a) as f32;
    let (along, across) = module_size(a);
    let mut out = Vec::new();
    // `rails` rails of `length` each: by the metre or in pieces, with the connectors
    // between the pieces.
    let rails_of = |out: &mut Vec<(Role, f32)>, rails: f32, length: f32| {
        let rail = find_part(prj, &part_for(prj, a, Role::Rail));
        let by_metre = rail.is_none_or(|p| p.quantity_unit == "m" || p.width <= 0.0);
        let piece = rail.filter(|_| !by_metre).map_or(RAIL_PIECE, |p| p.width);
        let pieces = (length / piece).ceil();
        out.push((Role::Rail, if by_metre { (rails * length / 1000.0 * 100.0).ceil() / 100.0 } else { rails * pieces }));
        out.push((Role::Connector, rails * (pieces - 1.0)));
    };
    match a.mounting {
        PvMounting::Parallel => {
            // Rails across the long side of the modules: along the rows in portrait, up
            // the slope in landscape.
            let (rails, length, on_rail) = if a.landscape {
                (2.0 * cols, rows * across + (rows - 1.0) * CLAMP_GAP + 2.0 * RAIL_END, rows)
            } else {
                (2.0 * rows, cols * along + (cols - 1.0) * CLAMP_GAP + 2.0 * RAIL_END, cols)
            };
            rails_of(&mut out, rails, length);
            out.push((Role::Hook, rails * ((length - 2.0 * RAIL_END) / a.span.max(300.0)).ceil().max(1.0) + rails));
            out.push((Role::EndClamp, 2.0 * rails));
            out.push((Role::MidClamp, rails * (on_rail - 1.0)));
            out.push((Role::Earthing, rails));
        }
        PvMounting::Tilted | PvMounting::EastWest => {
            let pair = a.mounting == PvMounting::EastWest;
            let sides = if pair { 2.0 } else { 1.0 };
            out.push((Role::Base, rows * (cols + 1.0)));
            out.push((Role::EndClamp, 4.0 * rows * sides));
            out.push((Role::MidClamp, 2.0 * rows * sides * (cols - 1.0)));
            if a.ballast > 0.0 {
                let block = find_part(prj, &part_for(prj, a, Role::Ballast)).map(|p| p.weight).filter(|w| *w > 0.0).unwrap_or(BALLAST_BLOCK);
                out.push((Role::Ballast, (modules * a.ballast / block).ceil()));
            }
            if !pair {
                out.push((Role::Deflector, modules));
            }
            out.push((Role::Earthing, rows));
        }
        PvMounting::Ground => {
            // Per table: two rails along each row of modules, a table beam on a front and a
            // rear post at each post place.
            let rails = 2.0 * a.stack.max(1) as f32 * rows;
            let posts = post_places(a).len() as f32 * rows;
            out.push((Role::Post, 2.0 * posts));
            out.push((Role::Beam, posts));
            rails_of(&mut out, rails, cols * along + (cols - 1.0) * CLAMP_GAP + 2.0 * RAIL_END);
            out.push((Role::EndClamp, 2.0 * rails));
            out.push((Role::MidClamp, rails * (cols - 1.0)));
            out.push((Role::Earthing, rows));
        }
    }
    out.retain(|(_, q)| *q > 0.0);
    out
}

/// The material of an array for the bill of materials: its modules (unless a device
/// counts them) and its mounting parts, with their quantities.
pub fn material(prj: &Project, a: &PvArray) -> Vec<(String, f32)> {
    let mut out: Vec<(String, f32)> = Vec::new();
    if a.device.is_empty() && !a.module.is_empty() {
        out.push((a.module.clone(), module_count(a) as f32));
    }
    for (role, qty) in mounting(prj, a) {
        let part = part_for(prj, a, role);
        match out.iter_mut().find(|x| x.0 == part) {
            Some(x) => x.1 += qty,
            None => out.push((part, qty)),
        }
    }
    out
}

/// Peak power of a module in W from its part ("430 Wp"), if known.
pub fn module_power(prj: &Project, a: &PvArray) -> Option<f32> {
    let p = find_part(prj, &a.module)?;
    let text = if p.power.is_empty() { p.tech_value("Maximum power")? } else { p.power.as_str() };
    let num: String = text.trim().chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    let w: f32 = num.parse().ok()?;
    Some(if text.to_ascii_lowercase().contains("kw") { w * 1000.0 } else { w })
}

/// Peak power of an array in kWp, if its module's power is known.
pub fn peak_power(prj: &Project, a: &PvArray) -> Option<f32> {
    module_power(prj, a).map(|w| w * module_count(a) as f32 / 1000.0)
}

/// An array with its numbers in range.
pub fn checked_array(mut a: PvArray) -> PvArray {
    a.columns = a.columns.clamp(1, 200);
    a.rows = a.rows.clamp(1, 100);
    a.stack = a.stack.clamp(1, 10);
    a.clearance = a.clearance.clamp(300.0, 5000.0);
    a.module_width = a.module_width.clamp(200.0, 3000.0);
    a.module_length = a.module_length.clamp(200.0, 3000.0);
    a.module_height = a.module_height.clamp(5.0, 100.0);
    a.tilt = a.tilt.clamp(0.0, 60.0);
    a.row_gap = a.row_gap.clamp(0.0, 20_000.0);
    a.span = a.span.clamp(300.0, 3000.0);
    a.ballast = a.ballast.clamp(0.0, 500.0);
    a
}

/// Sets the module of an array: its part number and its size from the part.
pub fn set_module(prj: &Project, a: &mut PvArray, number: &str) -> Result<(), String> {
    let number = number.trim();
    let p = find_part(prj, number).ok_or_else(|| tr_fmt!("No part {} in the project", number))?;
    if p.width > 0.0 && p.height > 0.0 {
        a.module_width = p.width.min(p.height);
        a.module_length = p.width.max(p.height);
    }
    if p.depth > 0.0 {
        a.module_height = p.depth;
    }
    a.module = p.number.clone();
    Ok(())
}

/// A new PV array on roof `k` of building `bi` at `at` (building floor plan), otherwise
/// in the middle of its front or right slope (flat roofs: of the roof, tilted towards the
/// front); its id.
pub fn add_array(prj: &mut Project, bi: usize, k: usize, at: Option<[f32; 2]>) -> Result<u64, String> {
    let id = prj.new_id().max(1);
    let r = prj.layout.buildings.get_mut(bi).and_then(|b| b.roofs.get_mut(k)).ok_or_else(|| tr("No such roof").to_string())?;
    let (cx, cz) = (r.x + r.width / 2.0, r.z + r.depth / 2.0);
    let flat = r.shape == crate::model::panel::RoofShape::Flat || r.pitch < FLAT_PITCH;
    let at = at.unwrap_or(if flat {
        [cx, cz]
    } else if r.along_z {
        [cx + r.width / 4.0, cz]
    } else {
        [cx, cz + r.depth / 4.0]
    });
    let mut a = PvArray { id, name: free_name(&r.solar), x: at[0], z: at[1], ..Default::default() };
    if flat {
        a.mounting = PvMounting::Tilted;
        a.landscape = true;
        a.rows = 3;
    }
    r.solar.push(checked_array(a));
    Ok(id)
}

/// A name for a new array among `arrays`: PV1, PV2, …
fn free_name(arrays: &[PvArray]) -> String {
    (arrays.len() + 1..).map(|k| format!("PV{k}")).find(|c| !arrays.iter().any(|a| &a.name == c)).unwrap_or_default()
}

/// A new freestanding PV array on the plot at `at` (the plant), otherwise in front of
/// (+z) what stands in the plant: two tables of two rows of ten modules on posts facing
/// the front; its id.
pub fn add_ground_array(prj: &mut Project, at: Option<[f32; 2]>) -> Result<u64, String> {
    let id = prj.new_id().max(1);
    let name = free_name(&prj.layout.plot.solar);
    let mut a = checked_array(PvArray { id, name, mounting: PvMounting::Ground, columns: 10, rows: 2, tilt: 25.0, span: 2500.0, ballast: 0.0, ..Default::default() });
    let at = at.unwrap_or_else(|| {
        let spots: Vec<[f32; 2]> = super::plot::footprints(prj).into_iter().flat_map(|f| f.1).collect();
        let Some((lo, hi)) = (!spots.is_empty()).then(|| super::bounds(&spots)) else { return [0.0, 0.0] };
        // The array is still at the origin: the half of its depth.
        let (_, half) = super::bounds(&ground_geometry(&a).outline);
        [((lo[0] + hi[0]) / 2.0).round(), (hi[1] + GROUND_GAP + half[1]).round()]
    });
    (a.x, a.z) = (at[0], at[1]);
    prj.layout.plot.solar.push(a);
    Ok(id)
}

/// The area two convex polygons of the floor plan share.
fn overlap(p: &[[f32; 2]], q: &[[f32; 2]]) -> f32 {
    let mut rest = p.to_vec();
    for f in inside_lins(q) {
        rest = roofs::clip(&rest, f);
        if rest.len() < 3 {
            return 0.0;
        }
    }
    roofs::area(&rest).abs()
}

/// The devices of array `a` (named after `place`) that do not exist, and a device that
/// counts another number of modules.
fn device_problems(prj: &Project, place: &str, a: &PvArray, out: &mut Vec<Issue>) {
    if !a.combiner.is_empty() && !prj.devices.contains_key(&a.combiner) {
        out.push(crate::issue!(Severity::Warning, None, "{}: the generator junction box {} of PV array {} does not exist", place, a.combiner, a.name));
    }
    if !a.earthing.is_empty() && !prj.devices.contains_key(&a.earthing) {
        out.push(crate::issue!(Severity::Warning, None, "{}: the earthing busbar {} of PV array {} does not exist", place, a.earthing, a.name));
    }
    if !a.device.is_empty() {
        match prj.devices.get(&a.device) {
            None => out.push(crate::issue!(Severity::Warning, None, "{}: the device {} of PV array {} does not exist", place, a.device, a.name)),
            Some(d) => {
                let n: u32 = d.parts.iter().filter(|p| p.number == a.module).map(|p| p.quantity()).sum();
                if n != module_count(a) {
                    out.push(crate::issue!(Severity::Warning, None, "{}: PV array {} has {} modules, its device {} has {}", place, a.name, module_count(a), a.device, n));
                }
            }
        }
    }
}

/// Freestanding PV arrays over each other or standing on a room or a cabinet (outside
/// the plot boundary: [`super::plot::problems`]), with the problems of their devices.
fn ground_problems(prj: &Project, out: &mut Vec<Issue>) {
    let arrays = &prj.layout.plot.solar;
    if arrays.is_empty() {
        return;
    }
    let place = place_name(&prj.layout, Support::Ground);
    let outlines: Vec<Vec<[f32; 2]>> = arrays.iter().map(|a| ground_geometry(a).outline).collect();
    let standing: Vec<(String, Vec<[f32; 2]>)> = super::plot::footprints(prj)
        .into_iter()
        .filter_map(|(s, f)| match s {
            super::plot::Standing::Room(i) | super::plot::Standing::Cabinet(i) => Some((prj.layout.enclosures[i].name.clone(), f)),
            _ => None,
        })
        .collect();
    for (k, a) in arrays.iter().enumerate() {
        for (j, other) in outlines.iter().enumerate().skip(k + 1) {
            if overlap(&outlines[k], other) > 1.0 {
                out.push(crate::issue!(Severity::Warning, None, "{}: the freestanding PV arrays {} and {} stand on each other", place, a.name, arrays[j].name));
            }
        }
        for (name, f) in &standing {
            if overlap(f, &outlines[k]) > 1.0 {
                out.push(crate::issue!(Severity::Warning, None, "{}: the freestanding PV array {} stands on {}", place, a.name, name));
            }
        }
        device_problems(prj, &place, a, out);
    }
}

/// PV arrays off their roof or across a ridge, a hip or its edge, tilted arrays on a
/// pitched roof, arrays over a roof window or over each other, freestanding arrays on
/// each other or on a room, and arrays whose device is missing or counts another number
/// of modules.
pub fn problems(prj: &Project) -> Vec<Issue> {
    let mut out = Vec::new();
    for b in &prj.layout.buildings {
        for r in &b.roofs {
            let under = roofs::planes(r);
            let windows: Vec<Vec<[f32; 2]>> = r.windows.iter().filter_map(|w| roofs::window_corners(r, w)).map(|(c, _)| c.iter().map(|v| [v[0], v[2]]).collect()).collect();
            let outlines: Vec<Option<Vec<[f32; 2]>>> = r.solar.iter().map(|a| geometry(r, a).map(|g| g.outline)).collect();
            for (k, a) in r.solar.iter().enumerate() {
                let Some(g) = geometry(r, a) else {
                    out.push(crate::issue!(Severity::Warning, None, "{}: PV array {} of roof {} lies outside the roof", b.name, a.name, r.name));
                    continue;
                };
                let off = g.modules.iter().flatten().any(|v| {
                    let q = [v[0], v[2]];
                    roofs::underside_at(r, q).is_none() || (g.plane.at(q) - roofs::lowest(&under, q)).abs() > 1.0
                });
                if off {
                    out.push(crate::issue!(Severity::Warning, None, "{}: PV array {} of roof {} lies across a ridge, a hip or the edge of the roof", b.name, a.name, r.name));
                }
                if a.mounting.is_tilted() && g.plane.slope() > FLAT_PITCH.to_radians().tan() {
                    out.push(crate::issue!(Severity::Warning, None, "{}: PV array {} is mounted tilted on a pitched roof", b.name, a.name));
                }
                let modules = module_outlines(r, a);
                if windows.iter().any(|w| modules.iter().any(|m| overlap(m, w) > 1.0)) {
                    out.push(crate::issue!(Severity::Warning, None, "{}: PV array {} covers a roof window", b.name, a.name));
                }
                for (j, other) in outlines.iter().enumerate().skip(k + 1) {
                    if other.as_ref().is_some_and(|o| overlap(&g.outline, o) > 1.0) {
                        out.push(crate::issue!(Severity::Warning, None, "{}: PV arrays {} and {} lie over each other", b.name, a.name, r.solar[j].name));
                    }
                }
                device_problems(prj, &b.name, a, &mut out);
            }
        }
    }
    ground_problems(prj, &mut out);
    out
}

#[cfg(test)]
mod tests;
