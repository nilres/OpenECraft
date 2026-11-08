//! Roofs of a building ([`Roof`]): flat, shed, gable, hip and pyramid roofs over a
//! rectangle of the building's floor plan, with roof windows in their slopes.
//!
//! The underside of a roof is the lowest of a few planes ([`planes`]): one per slope,
//! each rising from its eaves at the pitch. It may reach down through several levels:
//! the rooms below it get its slopes as their ceiling where it is lower than their own
//! ([`Ceiling`]), so a room can be of different heights at different places. The
//! ceiling, the tops of the walls and the roof itself come out as exact flat pieces
//! for the floor plans and the 3D view.

use super::buildings::{building_pose, find_level};
use super::{cabinets_in, item_box, outline, rooms, site_poses, Face, ItemKind, Layout, SitePose};
use crate::issue::{Issue, Severity};
use crate::model::panel::{Building, Roof, RoofShape, RoofWindow};
use crate::model::Project;
use crate::parts::vec3::V3;
use openecraft_i18n::{tr, tr_const};

/// Height of the eaves of a new roof above the top of the highest level, mm (a knee
/// wall of the attic).
pub const KNEE_WALL: f32 = 0.0;
/// Size of a new roof window (width along the eaves, length up the slope), mm.
pub const WINDOW_SIZE: (f32, f32) = (780.0, 1180.0);
/// Thickness of the frame of a roof window, mm.
pub const WINDOW_FRAME: f32 = 60.0;

impl RoofShape {
    pub const ALL: [RoofShape; 5] = [RoofShape::Flat, RoofShape::Shed, RoofShape::Gable, RoofShape::Hip, RoofShape::Pyramid];

    pub fn key(&self) -> &'static str {
        match self {
            RoofShape::Flat => "flat",
            RoofShape::Shed => "shed",
            RoofShape::Gable => "gable",
            RoofShape::Hip => "hip",
            RoofShape::Pyramid => "pyramid",
        }
    }

    pub fn from_key(key: &str) -> Option<RoofShape> {
        Self::ALL.into_iter().find(|k| k.key().eq_ignore_ascii_case(key.trim()))
    }

    /// Untranslated title (`tr` where shown).
    pub fn title(&self) -> &'static str {
        match self {
            RoofShape::Flat => tr_const!("Flat roof"),
            RoofShape::Shed => tr_const!("Shed roof"),
            RoofShape::Gable => tr_const!("Gable roof"),
            RoofShape::Hip => tr_const!("Hip roof"),
            RoofShape::Pyramid => tr_const!("Pyramid roof"),
        }
    }
}

/// A linear function `a·x + b·z + c` of the floor plan (a plane of heights).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lin {
    pub a: f32,
    pub b: f32,
    pub c: f32,
}

impl Lin {
    pub const fn constant(c: f32) -> Lin {
        Lin { a: 0.0, b: 0.0, c }
    }

    pub fn at(&self, q: [f32; 2]) -> f32 {
        self.a * q[0] + self.b * q[1] + self.c
    }

    pub fn minus(self, o: Lin) -> Lin {
        Lin { a: self.a - o.a, b: self.b - o.b, c: self.c - o.c }
    }

    pub fn plus(self, h: f32) -> Lin {
        Lin { c: self.c + h, ..self }
    }

    /// Steepness: rise per mm of the floor plan.
    pub fn slope(&self) -> f32 {
        self.a.hypot(self.b)
    }

    pub fn is_flat(&self) -> bool {
        self.slope() < 1e-6
    }

    /// The function of a point `q` given as `m(q)` with an affine `m`.
    fn through(&self, m: impl Fn([f32; 2]) -> [f32; 2]) -> Lin {
        let o = self.at(m([0.0, 0.0]));
        Lin { a: self.at(m([1.0, 0.0])) - o, b: self.at(m([0.0, 1.0])) - o, c: o }
    }
}

/// The part of a convex polygon where `f <= 0` (Sutherland-Hodgman).
pub fn clip(poly: &[[f32; 2]], f: Lin) -> Vec<[f32; 2]> {
    let mut out = Vec::new();
    for (k, &p) in poly.iter().enumerate() {
        let q = poly[(k + 1) % poly.len()];
        let (fp, fq) = (f.at(p), f.at(q));
        if fp <= 0.0 {
            out.push(p);
        }
        if (fp < 0.0 && fq > 0.0) || (fp > 0.0 && fq < 0.0) {
            let t = fp / (fp - fq);
            out.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
        }
    }
    out
}

pub fn area(poly: &[[f32; 2]]) -> f32 {
    let n = poly.len();
    (0..n).map(|k| poly[k][0] * poly[(k + 1) % n][1] - poly[(k + 1) % n][0] * poly[k][1]).sum::<f32>() / 2.0
}

/// The edges of a convex polygon as functions that are negative inside it.
pub(super) fn inside_lins(poly: &[[f32; 2]]) -> Vec<Lin> {
    let sign = if area(poly) >= 0.0 { 1.0 } else { -1.0 };
    (0..poly.len())
        .map(|k| {
            let (p, q) = (poly[k], poly[(k + 1) % poly.len()]);
            // Left of p → q is inside for a counter-clockwise polygon.
            let (nx, nz) = (q[1] - p[1], p[0] - q[0]);
            Lin { a: sign * nx, b: sign * nz, c: -sign * (nx * p[0] + nz * p[1]) }
        })
        .collect()
}

/// A convex polygon split into the part inside convex `region` and the parts outside it.
fn split_by(poly: &[[f32; 2]], region: &[[f32; 2]]) -> (Vec<[f32; 2]>, Vec<Vec<[f32; 2]>>) {
    let mut rest = poly.to_vec();
    let mut outside = Vec::new();
    for f in inside_lins(region) {
        let out = clip(&rest, Lin { a: -f.a, b: -f.b, c: -f.c });
        if area(&out).abs() > 1.0 {
            outside.push(out);
        }
        rest = clip(&rest, f);
        if rest.len() < 3 {
            return (Vec::new(), outside);
        }
    }
    (rest, outside)
}

/// The corners of the rectangle a roof covers with its overhang (building floor plan).
pub fn region(r: &Roof) -> Vec<[f32; 2]> {
    let o = r.overhang.max(0.0);
    let (x0, z0, x1, z1) = (r.x - o, r.z - o, r.x + r.width + o, r.z + r.depth + o);
    vec![[x0, z0], [x1, z0], [x1, z1], [x0, z1]]
}

/// The planes of the underside of a roof (building floor plan → height above the
/// building's ground level): the roof is the lowest of them.
pub fn planes(r: &Roof) -> Vec<Lin> {
    let t = r.pitch.clamp(0.0, 75.0).to_radians().tan();
    let (x0, z0, x1, z1) = (r.x, r.z, r.x + r.width.max(1.0), r.z + r.depth.max(1.0));
    let e = r.eaves;
    // Rising from the edge at x0, x1, z0, z1 into the rectangle.
    let from_x0 = Lin { a: t, b: 0.0, c: e - t * x0 };
    let from_x1 = Lin { a: -t, b: 0.0, c: e + t * x1 };
    let from_z0 = Lin { a: 0.0, b: t, c: e - t * z0 };
    let from_z1 = Lin { a: 0.0, b: -t, c: e + t * z1 };
    match r.shape {
        RoofShape::Flat => vec![Lin::constant(e)],
        RoofShape::Shed => vec![match (r.along_z, r.flip) {
            (false, false) => from_z1,
            (false, true) => from_z0,
            (true, false) => from_x1,
            (true, true) => from_x0,
        }],
        RoofShape::Gable if r.along_z => vec![from_x0, from_x1],
        RoofShape::Gable => vec![from_z0, from_z1],
        RoofShape::Hip => vec![from_x0, from_x1, from_z0, from_z1],
        RoofShape::Pyramid => {
            let peak = t * (x1 - x0).min(z1 - z0) / 2.0;
            let (hx, hz) = ((x1 - x0) / 2.0, (z1 - z0) / 2.0);
            vec![
                Lin { a: peak / hx, b: 0.0, c: e - peak / hx * x0 },
                Lin { a: -peak / hx, b: 0.0, c: e + peak / hx * x1 },
                Lin { a: 0.0, b: peak / hz, c: e - peak / hz * z0 },
                Lin { a: 0.0, b: -peak / hz, c: e + peak / hz * z1 },
            ]
        }
    }
}

/// Height of the underside of a roof at a point of the building's floor plan, `None`
/// outside it.
pub fn underside_at(r: &Roof, q: [f32; 2]) -> Option<f32> {
    let reg = region(r);
    inside_lins(&reg).iter().all(|f| f.at(q) <= 0.5).then(|| planes(r).iter().map(|l| l.at(q)).fold(f32::INFINITY, f32::min))
}

/// The slopes of a roof in the building's floor plan (their edges are the eaves, the
/// ridge and the hips).
pub fn slopes(r: &Roof) -> Vec<Vec<[f32; 2]>> {
    lowest_pieces(&region(r), &planes(r)).into_iter().map(|(p, _)| p).collect()
}

/// Highest point of a roof's underside above the building's ground level.
pub fn ridge_height(r: &Roof) -> f32 {
    let cx = r.x + r.width / 2.0;
    let cz = r.z + r.depth / 2.0;
    let pts = [[cx, cz], [r.x, r.z], [r.x + r.width, r.z], [r.x, r.z + r.depth], [r.x + r.width, r.z + r.depth]];
    pts.iter().filter_map(|&q| underside_at(r, q)).fold(f32::NEG_INFINITY, f32::max)
}

/// The pieces of a convex polygon where one of `lins` is the lowest: each piece with
/// its function.
fn lowest_pieces(poly: &[[f32; 2]], lins: &[Lin]) -> Vec<(Vec<[f32; 2]>, Lin)> {
    let mut out = Vec::new();
    for (i, li) in lins.iter().enumerate() {
        let mut piece = poly.to_vec();
        for (j, lj) in lins.iter().enumerate() {
            if i != j && piece.len() >= 3 {
                piece = clip(&piece, li.minus(*lj));
            }
        }
        if piece.len() >= 3 && area(&piece).abs() > 1.0 {
            out.push((piece, *li));
        }
    }
    out
}

/// One roof as seen from a room: the region it covers and the planes of its underside,
/// in the room's floor plan and heights above the room's floor.
#[derive(Clone, Debug, PartialEq)]
pub struct RoofPart {
    pub region: Vec<[f32; 2]>,
    pub planes: Vec<Lin>,
}

/// The ceiling of a room under roofs: its own height and the roofs over it.
#[derive(Clone, Debug, PartialEq)]
pub struct Ceiling {
    pub flat: f32,
    pub roofs: Vec<RoofPart>,
}

impl Ceiling {
    /// Height of the ceiling above the floor at a point of the room's floor plan.
    pub fn at(&self, q: [f32; 2]) -> f32 {
        let mut h = self.flat;
        for r in &self.roofs {
            if inside_lins(&r.region).iter().all(|f| f.at(q) <= 0.5) {
                h = r.planes.iter().fold(h, |h, l| h.min(l.at(q)));
            }
        }
        h
    }

    /// A convex polygon of the floor plan split into pieces over which the ceiling is
    /// flat or one plane of a roof: each with the function of its height.
    pub fn pieces(&self, poly: &[[f32; 2]]) -> Vec<(Vec<[f32; 2]>, Lin)> {
        let mut pieces = vec![(poly.to_vec(), Lin::constant(self.flat))];
        for r in &self.roofs {
            let mut next = Vec::new();
            for (p, lin) in pieces {
                let (inside, outside) = split_by(&p, &r.region);
                next.extend(outside.into_iter().map(|o| (o, lin)));
                if inside.len() < 3 {
                    continue;
                }
                for (q, plane) in lowest_pieces(&inside, &r.planes) {
                    let below = clip(&q, plane.minus(lin));
                    if below.len() >= 3 && area(&below).abs() > 1.0 {
                        next.push((below, plane));
                    }
                    let above = clip(&q, lin.minus(plane));
                    if above.len() >= 3 && area(&above).abs() > 1.0 {
                        next.push((above, lin));
                    }
                }
            }
            pieces = next;
        }
        pieces
    }

    /// The height of the ceiling along a line from `a` to `b` of the floor plan: the
    /// points (share of the way, height) where it bends, from `a` to `b`.
    pub fn profile(&self, a: [f32; 2], b: [f32; 2]) -> Vec<(f32, f32)> {
        let at = |t: f32| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        let mut lins = vec![Lin::constant(self.flat)];
        let mut ts = vec![0.0, 1.0];
        for r in &self.roofs {
            lins.extend(&r.planes);
            for f in inside_lins(&r.region) {
                let (fa, fb) = (f.at(a), f.at(b));
                if (fa - fb).abs() > 1e-6 {
                    let t = fa / (fa - fb);
                    ts.extend([t - 1e-4, t + 1e-4]);
                }
            }
        }
        for (i, li) in lins.iter().enumerate() {
            for lj in &lins[i + 1..] {
                let d = li.minus(*lj);
                let (da, db) = (d.at(a), d.at(b));
                if (da - db).abs() > 1e-6 {
                    ts.push(da / (da - db));
                }
            }
        }
        ts.retain(|t| (0.0..=1.0).contains(t));
        ts.sort_by(f32::total_cmp);
        ts.dedup_by(|x, y| (*x - *y).abs() < 1e-5);
        let pts: Vec<(f32, f32)> = ts.into_iter().map(|t| (t, self.at(at(t)))).collect();
        // Without the points on a straight stretch.
        let mut out: Vec<(f32, f32)> = Vec::new();
        for (k, &p) in pts.iter().enumerate() {
            if let (Some(&prev), Some(next)) = (out.last(), pts.get(k + 1)) {
                let between = prev.1 + (next.1 - prev.1) * (p.0 - prev.0) / (next.0 - prev.0).max(1e-9);
                if (between - p.1).abs() < 0.05 {
                    continue;
                }
            }
            out.push(p);
        }
        out
    }

    /// Lowest and highest ceiling over the floor of a room.
    pub fn range(&self, cells: &[Vec<[f32; 2]>]) -> (f32, f32) {
        let mut lo = self.flat;
        let mut hi = f32::NEG_INFINITY;
        for c in cells {
            for (p, lin) in self.pieces(c) {
                for q in p {
                    let h = lin.at(q);
                    lo = lo.min(h);
                    hi = hi.max(h);
                }
            }
        }
        (lo, hi.max(lo))
    }
}

/// The floor of a room as rectangles (room floor plan).
pub fn floor_polys(e: &super::Enclosure) -> Vec<Vec<[f32; 2]>> {
    super::floor_cells(e).into_iter().map(|(lo, hi)| vec![[lo[0], lo[2]], [hi[0], lo[2]], [hi[0], hi[2]], [lo[0], hi[2]]]).collect()
}

/// The sloped ceiling over a wall of a room in the wall's front view: the corners (x
/// along the wall, y down from the top of the wall) of the line where the wall ends,
/// `None` where the ceiling is flat over it.
pub fn wall_line(e: &super::Enclosure, c: &Ceiling, face: Face) -> Option<Vec<[f32; 2]>> {
    if !matches!(face, Face::Wall(_) | Face::Inner(..)) {
        return None;
    }
    let (w, _) = e.face_size(face);
    // The plan point of a place on the face.
    let at = |x: f32| {
        let (lo, _) = item_box(e, &super::PanelItem { face, x, ..Default::default() });
        [lo[0], lo[2]]
    };
    let prof = c.profile(at(0.0), at(w));
    prof.iter().any(|&(_, h)| h < c.flat - 0.5).then(|| prof.into_iter().map(|(t, h)| [t * w, e.height - h]).collect())
}

/// The building room `ri` stands in, if it stands on a level.
fn room_in_building(layout: &Layout, ri: usize) -> Option<usize> {
    let e = layout.enclosures.get(ri).filter(|e| e.is_room())?;
    find_level(layout, e.level).map(|(bi, _)| bi)
}

/// The ceiling of a room under the roofs of its building, `None` where no roof comes
/// lower than its own ceiling (`poses`: [`site_poses`]).
pub fn ceiling_with(prj: &Project, poses: &[SitePose], ri: usize) -> Option<Ceiling> {
    let layout = &prj.layout;
    let bi = room_in_building(layout, ri)?;
    let b = &layout.buildings[bi];
    if b.roofs.is_empty() {
        return None;
    }
    let e = &layout.enclosures[ri];
    let (rp, bp) = (poses[ri], building_pose(b));
    let floor = bp.local(rp.at)[1];
    // Room floor plan → building floor plan, and back.
    let to_building = |q: [f32; 2]| {
        let v = bp.local(rp.apply([q[0], 0.0, q[1]]));
        [v[0], v[2]]
    };
    let to_room = |q: [f32; 2]| {
        let v = rp.local(bp.apply([q[0], 0.0, q[1]]));
        [v[0], v[2]]
    };
    let roofs = b
        .roofs
        .iter()
        .map(|r| RoofPart { region: region(r).into_iter().map(to_room).collect(), planes: planes(r).iter().map(|l| l.through(to_building).plus(-floor)).collect() })
        .collect();
    let c = Ceiling { flat: e.height, roofs };
    let cells = floor_polys(e);
    let cuts = cells.iter().any(|cell| c.pieces(cell).iter().any(|(_, lin)| *lin != Lin::constant(c.flat)));
    cuts.then_some(c)
}

pub fn ceiling(prj: &Project, ri: usize) -> Option<Ceiling> {
    ceiling_with(prj, &site_poses(prj), ri)
}

/// A flat face of a roof or of a wall under it in the plant (y up): its corners in
/// order and its outward normal.
#[derive(Clone, Debug, PartialEq)]
pub struct RoofFace {
    pub pts: Vec<V3>,
    pub normal: V3,
    /// The outward normal of the shell it belongs to: the face is seen from outside the
    /// building where the eye is on this side of it.
    pub outward: V3,
    pub kind: RoofFaceKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoofFaceKind {
    /// The covering on top.
    Top,
    /// The underside (seen from the attic).
    Underside,
    /// The edges round its overhang (fascia, verge).
    Edge,
    /// A gable or knee wall between the eaves and the roof.
    Gable,
    /// The glass of a roof window.
    Glass,
    /// The frame of a roof window.
    Frame,
    /// The cells of a PV module.
    Module,
    /// The frame of a PV module.
    ModuleFrame,
    /// Rails, mounting bases and wind deflectors of a PV array.
    Mounting,
}

pub(super) fn normal_of(pts: &[V3]) -> V3 {
    let n = crate::parts::vec3::cross(crate::parts::vec3::sub(pts[1], pts[0]), crate::parts::vec3::sub(pts[2], pts[0]));
    crate::parts::vec3::norm(n)
}

/// Vertical faces along a line of the floor plan between two height profiles, facing
/// to the right of `a` → `b` (`inward`: into the building).
#[allow(clippy::too_many_arguments)]
fn band(a: [f32; 2], b: [f32; 2], low: &dyn Fn([f32; 2]) -> f32, high: &dyn Fn([f32; 2]) -> f32, ts: &[f32], kind: RoofFaceKind, inward: bool, out: &mut Vec<RoofFace>, pose: SitePose) {
    let at = |t: f32| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
    let normal = pose.turn(crate::parts::vec3::norm([b[1] - a[1], 0.0, a[0] - b[0]]));
    for w in ts.windows(2) {
        let (p, q) = (at(w[0]), at(w[1]));
        let (lp, lq, hp, hq) = (low(p), low(q), high(p), high(q));
        if hp - lp < 1.0 && hq - lq < 1.0 {
            continue;
        }
        let pts = vec![[p[0], lp.min(hp), p[1]], [q[0], lq.min(hq), q[1]], [q[0], hq.max(lq), q[1]], [p[0], hp.max(lp), p[1]]];
        let outward = if inward { [-normal[0], -normal[1], -normal[2]] } else { normal };
        out.push(RoofFace { pts: pts.into_iter().map(|v| pose.apply(v)).collect(), normal, outward, kind });
    }
}

/// Breakpoints along a line where the lowest of `lins` changes.
fn breaks(a: [f32; 2], b: [f32; 2], lins: &[Lin]) -> Vec<f32> {
    let mut ts = vec![0.0, 1.0];
    for (i, li) in lins.iter().enumerate() {
        for lj in &lins[i + 1..] {
            let d = li.minus(*lj);
            let (da, db) = (d.at(a), d.at(b));
            if (da - db).abs() > 1e-6 {
                ts.push(da / (da - db));
            }
        }
    }
    ts.retain(|t| (0.0..=1.0).contains(t));
    ts.sort_by(f32::total_cmp);
    ts.dedup_by(|x, y| (*x - *y).abs() < 1e-5);
    ts
}

pub(super) fn lowest(lins: &[Lin], q: [f32; 2]) -> f32 {
    lins.iter().map(|l| l.at(q)).fold(f32::INFINITY, f32::min)
}

/// The faces of a roof in the plant: covering, underside, the edges round it, the gable
/// and knee walls from the eaves up to it and its roof windows.
pub fn roof_faces(b: &Building, r: &Roof) -> Vec<RoofFace> {
    let pose = building_pose(b);
    let mut out = Vec::new();
    let reg = region(r);
    let under = planes(r);
    // The covering: each plane raised by the thickness across it.
    let top: Vec<Lin> = under.iter().map(|l| l.plus(r.thickness.max(1.0) * (1.0 + l.slope() * l.slope()).sqrt())).collect();
    let up = |p: &[[f32; 2]], l: Lin| p.iter().map(|&q| [q[0], l.at(q), q[1]]).collect::<Vec<V3>>();
    for (poly, l) in lowest_pieces(&reg, &top) {
        let mut pts = up(&poly, l);
        if normal_of(&pts)[1] < 0.0 {
            pts.reverse();
        }
        let n = pose.turn(normal_of(&pts));
        out.push(RoofFace { normal: n, outward: n, pts: pts.into_iter().map(|v| pose.apply(v)).collect(), kind: RoofFaceKind::Top });
    }
    for (poly, l) in lowest_pieces(&reg, &under) {
        let mut pts = up(&poly, l);
        if normal_of(&pts)[1] > 0.0 {
            pts.reverse();
        }
        let n = pose.turn(normal_of(&pts));
        out.push(RoofFace { normal: n, outward: [-n[0], -n[1], -n[2]], pts: pts.into_iter().map(|v| pose.apply(v)).collect(), kind: RoofFaceKind::Underside });
    }
    // The edges round the overhang, counter-clockwise seen from above (outward right).
    let ccw: Vec<[f32; 2]> = if area(&reg) < 0.0 { reg.clone() } else { reg.iter().rev().copied().collect() };
    for k in 0..ccw.len() {
        let (a, b2) = (ccw[k], ccw[(k + 1) % ccw.len()]);
        let mut ts = breaks(a, b2, &under);
        ts.extend(breaks(a, b2, &top));
        ts.sort_by(f32::total_cmp);
        ts.dedup_by(|x, y| (*x - *y).abs() < 1e-5);
        band(a, b2, &|q| lowest(&under, q), &|q| lowest(&top, q), &ts, RoofFaceKind::Edge, false, &mut out, pose);
    }
    // Gable and knee walls over the walls from the eaves up to the underside.
    let walls = [[r.x, r.z], [r.x, r.z + r.depth], [r.x + r.width, r.z + r.depth], [r.x + r.width, r.z]];
    for k in 0..4 {
        let (a, b2) = (walls[k], walls[(k + 1) % 4]);
        let ts = breaks(a, b2, &under);
        let eaves = |_: [f32; 2]| r.eaves;
        band(a, b2, &eaves, &|q| lowest(&under, q), &ts, RoofFaceKind::Gable, false, &mut out, pose);
        // The inside of the wall, facing into the attic.
        let rev: Vec<f32> = ts.iter().rev().map(|t| 1.0 - t).collect();
        band(b2, a, &eaves, &|q| lowest(&under, q), &rev, RoofFaceKind::Gable, true, &mut out, pose);
    }
    for w in &r.windows {
        if let Some((corners, _)) = window_corners(r, w) {
            window_faces(&corners, &mut out, pose);
        }
    }
    out
}

/// The faces of a prism over a convex polygon of the floor plan from the height
/// `bottom` up to the height `top` (y up): its top, its bottom and the sides along the
/// edges `side` keeps; each face with its outward normal.
pub fn prism(poly: &[[f32; 2]], bottom: Lin, top: Lin, side: impl Fn([f32; 2], [f32; 2]) -> bool) -> Vec<(Vec<V3>, V3)> {
    let mut out = Vec::new();
    if poly.len() < 3 {
        return out;
    }
    let up = |q: [f32; 2], l: Lin| [q[0], l.at(q), q[1]];
    let mut top_pts: Vec<V3> = poly.iter().map(|&q| up(q, top)).collect();
    let mut bottom_pts: Vec<V3> = poly.iter().map(|&q| up(q, bottom)).collect();
    if normal_of(&top_pts)[1] < 0.0 {
        top_pts.reverse();
    }
    if normal_of(&bottom_pts)[1] > 0.0 {
        bottom_pts.reverse();
    }
    out.push((top_pts.clone(), normal_of(&top_pts)));
    out.push((bottom_pts.clone(), normal_of(&bottom_pts)));
    let n = poly.len() as f32;
    let centre = [poly.iter().map(|q| q[0]).sum::<f32>() / n, poly.iter().map(|q| q[1]).sum::<f32>() / n];
    for k in 0..poly.len() {
        let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
        if !side(a, b) || (a[0] - b[0]).hypot(a[1] - b[1]) < 0.5 {
            continue;
        }
        let mut normal = crate::parts::vec3::norm([b[1] - a[1], 0.0, a[0] - b[0]]);
        let mid = [(a[0] + b[0]) / 2.0 - centre[0], (a[1] + b[1]) / 2.0 - centre[1]];
        let mut pts = vec![up(a, bottom), up(b, bottom), up(b, top), up(a, top)];
        if normal[0] * mid[0] + normal[2] * mid[1] < 0.0 {
            normal = [-normal[0], 0.0, -normal[2]];
        }
        if crate::parts::vec3::dot(normal_of(&pts), normal) < 0.0 {
            pts.reverse();
        }
        out.push((pts, normal));
    }
    out
}

/// The four corners of a roof window on the covering of its slope (building floor
/// plan and heights), and the plane it lies in; `None` outside the roof.
pub fn window_corners(r: &Roof, w: &RoofWindow) -> Option<([V3; 4], Lin)> {
    let under = planes(r);
    let c = [w.x, w.z];
    underside_at(r, c)?;
    let plane = *under.iter().min_by(|a, b| a.at(c).total_cmp(&b.at(c)))?;
    let lift = r.thickness.max(1.0) * (1.0 + plane.slope() * plane.slope()).sqrt();
    // Up the slope in the floor plan, and along the eaves.
    let (sx, sz) = if plane.is_flat() { (0.0, -1.0) } else { (plane.a / plane.slope(), plane.b / plane.slope()) };
    let (ex, ez) = (-sz, sx);
    let run = w.length / 2.0 / (1.0 + plane.slope() * plane.slope()).sqrt();
    let half = w.width / 2.0;
    let at = |u: f32, v: f32| {
        let q = [c[0] + ex * u + sx * v, c[1] + ez * u + sz * v];
        [q[0], plane.at(q) + lift + 5.0, q[1]]
    };
    Some(([at(-half, -run), at(half, -run), at(half, run), at(-half, run)], plane))
}

fn window_faces(c: &[V3; 4], out: &mut Vec<RoofFace>, pose: SitePose) {
    let mut pts = c.to_vec();
    if normal_of(&pts)[1] < 0.0 {
        pts.reverse();
    }
    let n = normal_of(&pts);
    let lift = |v: V3, k: f32| [v[0] + n[0] * k, v[1] + n[1] * k, v[2] + n[2] * k];
    // The frame stands out of the covering, the glass lies in it.
    let m = [0, 1, 2].map(|k| pts.iter().map(|p| p[k]).sum::<f32>() / 4.0);
    let inner: Vec<V3> = pts.iter().map(|&p| [0, 1, 2].map(|k| m[k] + (p[k] - m[k]) * 0.85)).collect();
    for k in 0..4 {
        let (a, b, bi, ai) = (pts[k], pts[(k + 1) % 4], inner[(k + 1) % 4], inner[k]);
        let quad = vec![lift(a, WINDOW_FRAME), lift(b, WINDOW_FRAME), lift(bi, WINDOW_FRAME), lift(ai, WINDOW_FRAME)];
        out.push(RoofFace { pts: quad.into_iter().map(|v| pose.apply(v)).collect(), normal: pose.turn(n), outward: pose.turn(n), kind: RoofFaceKind::Frame });
        let side = vec![a, b, lift(b, WINDOW_FRAME), lift(a, WINDOW_FRAME)];
        let sn = normal_of(&side);
        let sn = pose.turn([-sn[0], -sn[1], -sn[2]]);
        out.push(RoofFace { pts: side.into_iter().map(|v| pose.apply(v)).collect(), normal: sn, outward: sn, kind: RoofFaceKind::Frame });
    }
    out.push(RoofFace { pts: inner.iter().map(|&v| pose.apply(lift(v, WINDOW_FRAME * 0.5))).collect(), normal: pose.turn(n), outward: pose.turn(n), kind: RoofFaceKind::Glass });
}

/// The roof windows of the roofs over room `ri` in the room's 3D coordinates (y up from
/// its floor): the glass seen from below, on the underside of the slope.
pub fn windows_in_room(prj: &Project, poses: &[SitePose], ri: usize) -> Vec<[V3; 4]> {
    let layout = &prj.layout;
    let Some(bi) = room_in_building(layout, ri) else { return Vec::new() };
    let b = &layout.buildings[bi];
    let (rp, bp) = (poses[ri], building_pose(b));
    let mut out = Vec::new();
    for r in &b.roofs {
        for w in &r.windows {
            let Some((corners, plane)) = window_corners(r, w) else { continue };
            let local = corners.map(|v| {
                let q = [v[0], plane.at([v[0], v[2]]) - 5.0, v[2]];
                rp.local(bp.apply(q))
            });
            let e = &layout.enclosures[ri];
            let (lo, hi) = super::bounds(&outline(e));
            if local.iter().all(|q| q[0] >= lo[0] - 1.0 && q[0] <= hi[0] + 1.0 && q[2] >= lo[1] - 1.0 && q[2] <= hi[1] + 1.0 && q[1] <= e.height + 1.0) {
                out.push(local);
            }
        }
    }
    out
}

/// A new roof over building `bi`: over its rooms (or a rectangle of 10 × 8 m) with its
/// eaves on top of the highest level; its id.
pub fn add_roof(prj: &mut Project, bi: usize, shape: RoofShape) -> Result<u64, String> {
    let poses = site_poses(prj);
    let layout = &prj.layout;
    let b = layout.buildings.get(bi).ok_or_else(|| tr("No such building").to_string())?;
    let bp = building_pose(b);
    // The rooms of the building in its floor plan.
    let mut lo = [f32::INFINITY; 2];
    let mut hi = [f32::NEG_INFINITY; 2];
    for ri in super::buildings::building_rooms(layout, bi) {
        let e = &layout.enclosures[ri];
        let t = e.wall();
        for q in outline(e) {
            for (dx, dz) in [(-t, -t), (t, t)] {
                let v = bp.local(poses[ri].apply([q[0] + dx, 0.0, q[1] + dz]));
                lo = [lo[0].min(v[0]), lo[1].min(v[2])];
                hi = [hi[0].max(v[0]), hi[1].max(v[2])];
            }
        }
    }
    let (x, z, width, depth) = if lo[0].is_finite() { (lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]) } else { (0.0, 0.0, 10_000.0, 8000.0) };
    let eaves = b.levels.last().map_or(super::buildings::LEVEL_HEIGHT, |l| l.elevation + l.height) + KNEE_WALL;
    let n = b.roofs.len() + 1;
    let name = (n..).map(|k| format!("R{k}")).find(|c| !b.roofs.iter().any(|r| &r.name == c)).unwrap_or_default();
    let id = prj.new_id().max(1);
    let roof = Roof { id, name, shape, x, z, width, depth, along_z: depth > width, eaves, pitch: if shape == RoofShape::Flat { 0.0 } else if shape == RoofShape::Shed { 15.0 } else { 35.0 }, ..Default::default() };
    prj.layout.buildings[bi].roofs.push(checked_roof(roof));
    Ok(id)
}

/// A roof with its sizes in range.
pub fn checked_roof(mut r: Roof) -> Roof {
    r.width = r.width.max(500.0);
    r.depth = r.depth.max(500.0);
    r.pitch = if r.shape == RoofShape::Flat { 0.0 } else { r.pitch.clamp(1.0, 75.0) };
    r.overhang = r.overhang.clamp(0.0, 3000.0);
    r.thickness = r.thickness.clamp(50.0, 1000.0);
    r
}

/// A new roof window in the middle of the slope of roof `k` of building `bi` that
/// faces `towards` (+z: front, -z: rear, +x: right, -x: left), or at `x`, `z`; its id.
pub fn add_window(prj: &mut Project, bi: usize, k: usize, at: Option<[f32; 2]>) -> Result<u64, String> {
    let id = prj.new_id().max(1);
    let r = prj.layout.buildings.get_mut(bi).and_then(|b| b.roofs.get_mut(k)).ok_or_else(|| tr("No such roof").to_string())?;
    let at = at.unwrap_or_else(|| {
        // Halfway up the front (or right) slope.
        let (cx, cz) = (r.x + r.width / 2.0, r.z + r.depth / 2.0);
        if r.along_z { [cx + r.width / 4.0, cz] } else { [cx, cz + r.depth / 4.0] }
    });
    r.windows.push(RoofWindow { id, x: at[0], z: at[1], width: WINDOW_SIZE.0, length: WINDOW_SIZE.1, part: String::new() });
    Ok(id)
}

/// Roof windows that lie off their roof or across a ridge or hip, rooms cut off by a
/// roof, cabinets higher than the ceiling at their place and items on the walls or
/// the ceiling above the slope of the roof.
pub fn problems(prj: &Project) -> Vec<Issue> {
    let layout = &prj.layout;
    let mut out = Vec::new();
    for b in &layout.buildings {
        for r in &b.roofs {
            for w in &r.windows {
                let Some((corners, plane)) = window_corners(r, w) else {
                    out.push(crate::issue!(Severity::Warning, None, "{}: a roof window of roof {} lies outside the roof", b.name, r.name));
                    continue;
                };
                let under = planes(r);
                let off = corners.iter().any(|v| {
                    let q = [v[0], v[2]];
                    underside_at(r, q).is_none() || (plane.at(q) - lowest(&under, q)).abs() > 1.0
                });
                if off {
                    out.push(crate::issue!(Severity::Warning, None, "{}: a roof window of roof {} lies across a ridge, a hip or the edge of the roof", b.name, r.name));
                }
            }
        }
    }
    let poses = site_poses(prj);
    for ri in rooms(layout) {
        let Some(c) = ceiling_with(prj, &poses, ri) else { continue };
        let e = &layout.enclosures[ri];
        let (lo, _) = c.range(&floor_polys(e));
        if lo < 1.0 {
            out.push(crate::issue!(Severity::Warning, None, "Room {}: the roof comes down to its floor; part of the room lies outside the building", e.name));
        }
        for ci in cabinets_in(layout, ri) {
            let cab = &layout.enclosures[ci];
            let p = poses[ci];
            let top = p.at[1] + cab.height;
            let corners = [[0.0, 0.0], [cab.width, 0.0], [0.0, cab.depth], [cab.width, cab.depth]];
            let low = corners.iter().any(|&[x, z]| {
                let v = poses[ri].local(p.apply([x, 0.0, z]));
                c.at([v[0], v[2]]) + poses[ri].at[1] < top - 1.0
            });
            if low {
                out.push(crate::issue!(Severity::Warning, None, "{}: {} is higher than the sloped ceiling at its place", e.name, cab.name));
            }
        }
        for it in e.items.iter().filter(|i| matches!(i.face, Face::Wall(_) | Face::Inner(..) | Face::Roof) && i.kind != ItemKind::Cutout) {
            let (blo, bhi) = item_box(e, it);
            let top = e.height - blo[1].min(bhi[1]);
            let above = [[blo[0], blo[2]], [bhi[0], blo[2]], [blo[0], bhi[2]], [bhi[0], bhi[2]]].iter().any(|&q| c.at(q) < top - 1.0);
            if above {
                out.push(crate::issue!(Severity::Warning, None, "{}: {} lies above the sloped ceiling", e.name, super::item_name(e, it.id)));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
