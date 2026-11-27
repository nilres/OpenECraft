//! Production data of the panel layout (in the spirit of EPLAN Pro Panel): drilling
//! pattern and cutouts of the mounting plate, the door and the walls with NC export
//! (DXF / CSV), wire end treatment (stripping length, ferrules). The clearance checks are in
//! [`crate::panel::validate`].

use crate::geom::{p, P, R};
use crate::model::{natural_cmp, Project};
use crate::panel::{doors, DoorLeaf, Enclosure, Face, Fastening, ItemKind, PanelItem, PanelRotation, Sheet};
use crate::parts::vec3::{add, cross, dot, norm, scale, sub, Tri, V3};
use crate::parts::{Cutout, Part, PlacedCutout};
pub use crate::model::panel::{FerruleCode, Termination, WireTermination};
use openecraft_i18n::{tr, tr_const, tr_fmt};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

/// Rails: M5 holes 12.5 mm from the ends, at most every 200 mm.
pub const RAIL_HOLES: (f32, f32, &str) = (12.5, 200.0, "M5");
/// Cable ducts: M4 holes 20 mm from the ends, at most every 300 mm.
pub const DUCT_HOLES: (f32, f32, &str) = (20.0, 300.0, "M4");
/// Busbar holders: M6 holes 25 mm from the ends, at most every 250 mm.
pub const BUSBAR_HOLES: (f32, f32, &str) = (25.0, 250.0, "M6");
/// Devices screwed onto the plate without mounting data: two M4 holes, 5 mm from
/// two opposite corners.
pub const DEVICE_HOLE_INSET: f32 = 5.0;
pub const BUSBAR_HOLDER: &str = "GEN.BB-HOLDER-3P";

/// Tap drill diameter of a metric thread (M3 … M10), or a plain diameter ("Ø4.5",
/// "4.5").
pub fn drill_diameter(spec: &str) -> Option<f32> {
    let s = spec.trim();
    let tap = match s.to_uppercase().as_str() {
        "M3" => Some(2.5),
        "M4" => Some(3.3),
        "M5" => Some(4.2),
        "M6" => Some(5.0),
        "M8" => Some(6.8),
        "M10" => Some(8.5),
        _ => None,
    };
    tap.or_else(|| s.trim_start_matches(['Ø', 'D', 'd']).trim().replace(',', ".").parse().ok())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hole {
    pub item: u64,
    /// Face coordinates (origin top left, y down).
    pub pos: P,
    /// Drill diameter (weld studs: their nominal diameter).
    pub diameter: f32,
    /// Thread to cut ("M5"), empty for through holes.
    pub thread: String,
    /// How the item is fixed at this point.
    pub fastening: Fastening,
    /// Size of the screw ("M5", also for through holes), empty for plain diameters.
    pub thread_size: String,
}

/// Positions along a length: `end` from both ends and evenly spaced in between with
/// at most `pitch` distance.
pub fn line_positions(len: f32, end: f32, pitch: f32) -> Vec<f32> {
    if len <= 2.0 * end + 1.0 {
        return vec![len / 2.0];
    }
    let span = len - 2.0 * end;
    let n = (span / pitch).ceil().max(1.0) as usize;
    (0..=n).map(|i| end + span * i as f32 / n as f32).collect()
}

/// The fixing points of a mounting rail along it (mm from its start): [`RAIL_HOLES`],
/// on a perforated rail each in the slot nearest to it, so that the screw or rivet goes
/// through the slot.
pub fn rail_fixings(it: &PanelItem, parts: &[Part]) -> Vec<f32> {
    use crate::parts::mechanics::{rail_slots, RailProfile};
    let part = parts.iter().chain(crate::parts::catalog()).find(|x| x.number == it.part);
    let profile = part.map_or(RailProfile::TopHat { perforated: true }, RailProfile::of);
    let len = it.length();
    let (end, pitch, _) = RAIL_HOLES;
    let at = line_positions(len, end, pitch);
    let slots = if profile == (RailProfile::TopHat { perforated: true }) { rail_slots(len) } else { Vec::new() };
    if slots.is_empty() {
        return at;
    }
    let mut out: Vec<f32> = Vec::new();
    for t in at {
        let s = slots.iter().copied().min_by(|a, b| (a - t).abs().total_cmp(&(b - t).abs())).unwrap_or(t);
        if !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

/// Mounting holes from the part data: `Mounting holes: M4: 5,5; 40,85` (positions
/// relative to the top left corner of the part, front view).
pub fn part_holes(part: &Part) -> Option<(String, Vec<P>)> {
    let spec = part.data_value("Mounting holes")?;
    let (thread, list) = spec.split_once(':')?;
    let pts: Vec<P> = list
        .split(';')
        .filter_map(|xy| {
            let (x, y) = xy.split_once(',')?;
            Some(p(x.trim().parse().ok()?, y.trim().parse().ok()?))
        })
        .collect();
    (!pts.is_empty()).then(|| (thread.trim().to_string(), pts))
}

fn hole_with(item: u64, pos: P, thread: &str, fastening: Fastening) -> Hole {
    let size = crate::parts::mechanics::thread(thread).unwrap_or("").to_string();
    let diameter = fastening.hole_diameter(thread).or_else(|| drill_diameter(thread)).unwrap_or(4.2);
    let tapped = fastening == Fastening::Tapped && !size.is_empty();
    Hole { item, pos, diameter, thread: if tapped { size.clone() } else { String::new() }, fastening, thread_size: size }
}

fn hole(item: u64, pos: P, thread: &str) -> Hole {
    hole_with(item, pos, thread, Fastening::Tapped)
}

/// Holes of one item by its fastening (devices on rails, glued or clipped items need
/// none; weld studs are fixing points without a hole).
pub fn item_holes(it: &PanelItem, parts: &[Part]) -> Vec<Hole> {
    let part = parts.iter().find(|x| x.number == it.part).or_else(|| crate::parts::catalog().iter().find(|x| x.number == it.part));
    let fastening = Fastening::resolve(it, part);
    if !fastening.has_points() {
        return Vec::new();
    }
    pattern_holes(it, parts)
        .into_iter()
        .map(|h| if h.thread_size.is_empty() { Hole { fastening, ..h } } else { hole_with(h.item, h.pos, &h.thread_size, fastening) })
        .collect()
}

/// Fixing points of an item as tapped holes (the pattern, independent of the
/// fastening).
fn pattern_holes(it: &PanelItem, parts: &[Part]) -> Vec<Hole> {
    let along = |(end, pitch, thread): (f32, f32, &str)| {
        let (a, _) = it.center_line();
        line_positions(it.length(), end, pitch)
            .into_iter()
            .map(|t| hole(it.id, if it.horizontal() { p(a.x + t, a.y) } else { p(a.x, a.y + t) }, thread))
            .collect::<Vec<_>>()
    };
    match it.kind {
        ItemKind::Rail => {
            let (a, _) = it.center_line();
            rail_fixings(it, parts).into_iter().map(|t| hole(it.id, if it.horizontal() { p(a.x + t, a.y) } else { p(a.x, a.y + t) }, RAIL_HOLES.2)).collect()
        }
        ItemKind::Duct => along(DUCT_HOLES),
        ItemKind::Busbar => {
            // Holders sit at the two outer bars; one hole each side of the system.
            let (a, _) = it.center_line();
            let half = if it.horizontal() { it.h / 2.0 - 6.0 } else { it.w / 2.0 - 6.0 };
            let off = if it.horizontal() { p(0.0, half) } else { p(half, 0.0) };
            let mut v = Vec::new();
            for t in line_positions(it.length(), BUSBAR_HOLES.0, BUSBAR_HOLES.1) {
                let c = if it.horizontal() { p(a.x + t, a.y) } else { p(a.x, a.y + t) };
                v.push(hole(it.id, c - off, BUSBAR_HOLES.2));
                v.push(hole(it.id, c + off, BUSBAR_HOLES.2));
            }
            v
        }
        ItemKind::Device | ItemKind::Accessory | ItemKind::MountingPlate => {
            let part = parts.iter().find(|x| x.number == it.part).or_else(|| crate::parts::catalog().iter().find(|x| x.number == it.part));
            match part.and_then(part_holes) {
                Some((thread, pts)) => {
                    // Turned devices (width and height swapped) turn their pattern.
                    let pd = part.unwrap();
                    let inferred=if (pd.width - pd.height).abs() > 0.5 && (it.w - pd.height).abs() < 0.5 && (it.h - pd.width).abs() < 0.5 {crate::panel::PanelRotation::Deg90} else {crate::panel::PanelRotation::Deg0};
                    let rotation=it.rotation.unwrap_or(inferred);
                    pts.into_iter()
                        .map(|q| {let [x,y]=rotation.map([q.x,q.y],[pd.width,pd.height]);p(it.x+x,it.y+y)})
                        .map(|q| hole(it.id, q, &thread))
                        .collect()
                }
                None if it.kind!=ItemKind::MountingPlate && it.w > 2.0 * DEVICE_HOLE_INSET && it.h > 2.0 * DEVICE_HOLE_INSET => {
                    let i = DEVICE_HOLE_INSET;
                    vec![hole(it.id, p(it.x + i, it.y + i), "M4"), hole(it.id, p(it.x + it.w - i, it.y + it.h - i), "M4")]
                }
                None => Vec::new(),
            }
        }
        ItemKind::Cutout => Vec::new(),
    }
}

/// Drilling pattern of a mounting plate.
pub fn holes(e: &Enclosure, parts: &[Part]) -> Vec<Hole> {
    holes_on(e, parts, Face::Plate)
}

/// Drilling pattern (and weld studs) of a face.
pub fn holes_on(e: &Enclosure, parts: &[Part], face: Face) -> Vec<Hole> {
    let mut v: Vec<Hole> = e.items.iter().filter(|it| it.face == face).flat_map(|it| item_holes(it, parts)).collect();
    v.sort_by(|a, b| a.pos.y.total_cmp(&b.pos.y).then(a.pos.x.total_cmp(&b.pos.x)));
    v
}

/// Material of busbar systems besides the bars: holders (one per hole pair).
pub fn busbar_holders(e: &Enclosure) -> usize {
    e.items
        .iter()
        .filter(|i| i.kind == ItemKind::Busbar)
        .map(|i| line_positions(i.length(), BUSBAR_HOLES.0, BUSBAR_HOLES.1).len())
        .sum()
}

/// Number of parallel bars of a busbar system (part data `Bars: 3`).
pub fn bar_count(parts: &[Part], part: &str) -> usize {
    parts
        .iter()
        .find(|x| x.number == part)
        .and_then(|x| x.tech_value("Bars"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
        .clamp(1, 8)
}

/// Centres of the bars of a busbar system across its width `across` (mm from its
/// edge): at the part's centre distance (`Centre distance: 185 mm`) where the bars fit,
/// otherwise evenly spaced.
pub fn bar_centres(parts: &[Part], part: &str, across: f32) -> Vec<f32> {
    let n = bar_count(parts, part);
    let distance = parts
        .iter()
        .find(|x| x.number == part)
        .and_then(|x| x.tech_value("Centre distance"))
        .and_then(|v| v.trim().trim_end_matches("mm").trim().parse::<f32>().ok())
        .filter(|c| n > 1 && *c > 0.0 && (n - 1) as f32 * c < across);
    match distance {
        Some(c) => {
            let first = (across - (n - 1) as f32 * c) / 2.0;
            (0..n).map(|k| first + k as f32 * c).collect()
        }
        None => (0..n).map(|k| across / n as f32 * (k as f32 + 0.5)).collect(),
    }
}

// ---------------------------------------------------------------------- cutouts

/// Opening in the mounting plate or the door, in the coordinates of its face.
#[derive(Clone, Debug, PartialEq)]
pub struct Opening {
    /// Cutout item, or the door device the opening is for.
    pub item: u64,
    pub center: P,
    pub w: f32,
    pub h: f32,
    pub round: bool,
    /// Turn of a rectangular opening, degrees counter-clockwise (quarter turns are in
    /// `w` and `h`).
    pub angle: f32,
    /// Radius of the corners of a rectangular opening, mm.
    pub radius: f32,
}

impl Opening {
    fn axes(&self) -> (P, P) {
        let (s, c) = self.angle.to_radians().sin_cos();
        // Counter-clockwise on the face, whose y points down.
        (p(c, -s), p(s, c))
    }

    /// A point in the frame of the opening: from its centre along its width and height.
    pub fn local(&self, q: P) -> P {
        let (u, v) = self.axes();
        let d = q - self.center;
        p(d.x * u.x + d.y * u.y, d.x * v.x + d.y * v.y)
    }

    /// The outline, round openings and rounded corners as polygons.
    pub fn outline(&self) -> Vec<P> {
        crate::parts::cutout_outline([self.center.x, self.center.y], [self.w, self.h], self.round, self.angle, self.radius).into_iter().map(|[x, y]| p(x, y)).collect()
    }

    /// The rectangle around the outline.
    pub fn bounds(&self) -> R {
        let mut r = R::nothing();
        for q in self.outline() {
            r.extend(q);
        }
        r
    }
}

/// The opening of cutout `c` of the part of item `it`: its centre moves and turns with
/// the item (the turn of the item is that of its front view, clockwise for a quarter
/// turn).
pub fn place_cutout(it: &PanelItem, part: Option<&Part>, c: &PlacedCutout) -> Opening {
    let rotation = part.and_then(|p| p.size()).map(|[_, h, _]| crate::panel::item_rotation(it, h)).unwrap_or(it.rotation.unwrap_or_default());
    let [ox, oy] = c.offset;
    let offset = match rotation {
        PanelRotation::Deg0 => p(ox, oy),
        PanelRotation::Deg90 => p(-oy, ox),
        PanelRotation::Deg180 => p(-ox, -oy),
        PanelRotation::Deg270 => p(oy, -ox),
    };
    let [w, h] = c.size();
    let mut angle = (c.angle - rotation.degrees() as f32).rem_euclid(360.0);
    let quarters = (angle / 90.0).round();
    let (w, h) = if (angle - quarters * 90.0).abs() < 0.01 {
        angle = 0.0;
        if quarters as i32 % 2 == 1 { (h, w) } else { (w, h) }
    } else {
        (w, h)
    };
    let round = matches!(c.shape, Cutout::Round { .. });
    Opening { item: it.id, center: it.center() + offset, w, h, round, angle: if round { 0.0 } else { angle }, radius: c.radius }
}

/// The cutouts of a device in the door: the part's door cutouts or cutouts; without
/// data Ø22.5 for command and signal devices, otherwise the device outline less 2 mm
/// on each side.
pub fn door_cutouts(part: Option<&Part>, it: &PanelItem) -> Vec<PlacedCutout> {
    let own = part.map(|p| p.cutouts(true)).unwrap_or_default();
    if !own.is_empty() {
        return own;
    }
    let command = part.map(|p| p.group == "Sensors, switches, pushbuttons" || p.group == "Signal devices").unwrap_or(false);
    if command {
        return vec![PlacedCutout::centered(Cutout::Round { d: 22.5 })];
    }
    // The outline of the front view: the part's, otherwise the item's turned back.
    let quarter = it.rotation.is_some_and(|r| r.is_quarter());
    let [w, h] = part.and_then(|p| p.size()).map(|[w, h, _]| [w, h]).unwrap_or(if quarter { [it.h, it.w] } else { [it.w, it.h] });
    vec![PlacedCutout::centered(Cutout::Rect { w: (w - 4.0).max(1.0), h: (h - 4.0).max(1.0) })]
}

/// Openings of a face: cutout items, one per door device on the door, and the
/// cutouts of accessories and devices whose parts have one (`Cutout: 177 x 177`, fans,
/// cable glands).
pub fn openings(e: &Enclosure, parts: &[Part], face: Face) -> Vec<Opening> {
    let mut v = Vec::new();
    for it in e.items.iter().filter(|i| i.face == face) {
        if it.kind == ItemKind::Cutout {
            v.push(Opening { item: it.id, center: it.center(), w: it.w, h: it.h, round: it.round, angle: 0.0, radius: 0.0 });
            continue;
        }
        if !matches!(it.kind, ItemKind::Device | ItemKind::Accessory) {
            continue;
        }
        let part = parts.iter().chain(crate::parts::catalog()).find(|p| p.number == it.part);
        let cutouts = if it.kind == ItemKind::Device && face == Face::Door { door_cutouts(part, it) } else { part.map(|p| p.cutouts(false)).unwrap_or_default() };
        v.extend(cutouts.iter().map(|c| place_cutout(it, part, c)));
    }
    v
}

// ---------------------------------------------------------------------- sheets

impl From<Face> for Sheet {
    fn from(face: Face) -> Self {
        Sheet { face, leaf: None }
    }
}

impl Sheet {
    /// The leaf of the door of enclosure `e` this sheet is.
    fn door_leaf(self, e: &Enclosure) -> Option<(usize, DoorLeaf)> {
        let i = self.leaf.filter(|_| self.face == Face::Door)?;
        doors::leaves(e).get(i).map(|l| (i, *l))
    }

    /// Where the sheet is across its face, and its size.
    fn area(self, e: &Enclosure) -> (f32, f32, f32) {
        let (fw, fh) = e.face_size(self.face);
        match self.door_leaf(e) {
            Some((_, l)) => (l.x, l.w, fh),
            None => (0.0, fw, fh),
        }
    }

    /// Its size (mm).
    pub fn size(self, e: &Enclosure) -> (f32, f32) {
        let (_, w, h) = self.area(e);
        (w, h)
    }

    /// Its key in file names and parameters: the face's, `door_left` and `door_right` for
    /// the leaves of a double door, `door_1`, `door_2`, … for more leaves.
    pub fn key(self, e: &Enclosure) -> String {
        match self.door_leaf(e) {
            Some((i, _)) if doors::leaves(e).len() == 2 => if i == 0 { "door_left".into() } else { "door_right".into() },
            Some((i, _)) => format!("door_{}", i + 1),
            None => self.face.key().into(),
        }
    }

    /// Its name in the current language.
    pub fn title(self, e: &Enclosure) -> String {
        match self.door_leaf(e) {
            Some((i, _)) if doors::leaves(e).len() == 2 => if i == 0 { tr("Left door leaf").into() } else { tr("Right door leaf").into() },
            Some((i, _)) => tr_fmt!("Door leaf {}", i + 1),
            None => self.face.title().into(),
        }
    }

    /// Its name over a drawing of its holes: the door and its leaves seen from the front.
    pub fn front_title(self, e: &Enclosure) -> String {
        match self.door_leaf(e) {
            Some((i, _)) if doors::leaves(e).len() == 2 => if i == 0 { tr("Left door leaf (front view)").into() } else { tr("Right door leaf (front view)").into() },
            Some((i, _)) => tr_fmt!("Door leaf {} (front view)", i + 1),
            None if self.face == Face::Door => tr("Door (front view)").into(),
            None => self.face.title().into(),
        }
    }

    /// True for the sheet `x` (face coordinates) is on.
    fn holds(self, e: &Enclosure, x: f32) -> bool {
        self.door_leaf(e).is_none_or(|(i, _)| doors::leaf_index(e, x) == i)
    }
}

/// The sheets of an enclosure: its faces, a door of several leaves as one per leaf.
pub fn sheets(e: &Enclosure) -> Vec<Sheet> {
    let leaves = e.door_leaves.len();
    Face::ALL
        .iter()
        .flat_map(|&face| {
            let n = if face == Face::Door && leaves > 1 { leaves } else { 0 };
            if n == 0 { vec![Sheet::from(face)] } else { (0..n).map(|i| Sheet { face, leaf: Some(i) }).collect() }
        })
        .collect()
}

/// The sheet of a key of [`Sheet::key`] (also the face keys: `door` is the whole door).
pub fn sheet_from_key(e: &Enclosure, key: &str) -> Option<Sheet> {
    if let Some(face) = Face::from_key(key) {
        return Some(face.into());
    }
    sheets(e).into_iter().find(|s| s.leaf.is_some() && s.key(e) == key)
}

/// The holes of a sheet ([`holes_on`]) in its own coordinates: a hole is on the leaf it
/// is drilled in, measured from the leaf's left edge.
pub fn sheet_holes(e: &Enclosure, parts: &[Part], s: impl Into<Sheet>) -> Vec<Hole> {
    let s = s.into();
    let (x0, _, _) = s.area(e);
    let mut v: Vec<Hole> = holes_on(e, parts, s.face).into_iter().filter(|h| s.holds(e, h.pos.x)).collect();
    for h in &mut v {
        h.pos.x -= x0;
    }
    v
}

/// The openings of a sheet ([`openings`]) in its own coordinates, as [`sheet_holes`]. An
/// opening reaching over the joint between two leaves: a straight rectangle is cut on
/// each leaf as far as it reaches onto it; a round or turned one stays on the leaf under
/// its middle, reaching over its edge ([`over_edge`]).
pub fn sheet_openings(e: &Enclosure, parts: &[Part], s: impl Into<Sheet>) -> Vec<Opening> {
    let s = s.into();
    let all = openings(e, parts, s.face);
    let Some((i, l)) = s.door_leaf(e) else { return all };
    let (a, z) = (l.x, l.x + l.w);
    let mut v = Vec::new();
    for mut o in all {
        let b = o.bounds();
        if b.max.x <= a + 0.01 || b.min.x >= z - 0.01 {
            continue;
        }
        let within = b.min.x >= a - 0.01 && b.max.x <= z + 0.01;
        if !within && !o.round && o.angle == 0.0 {
            let (left, right) = (b.min.x.max(a), b.max.x.min(z));
            o.center.x = (left + right) / 2.0;
            o.w = right - left;
            o.radius = o.radius.min(o.w / 2.0);
        } else if !within && doors::leaf_index(e, o.center.x) != i {
            continue;
        }
        o.center.x -= a;
        v.push(o);
    }
    v
}

/// True if opening `o` of a sheet `width` mm wide reaches over its left or right edge:
/// a round or turned cutout over the joint between two leaves of a door.
pub fn over_edge(o: &Opening, width: f32) -> bool {
    let b = o.bounds();
    b.min.x < -0.01 || b.max.x > width + 0.01
}

// ---------------------------------------------------------------------- drilling pattern

/// A kind of hole or cutout of a drilling pattern: its letter, size and how it is made.
#[derive(Clone, Debug, PartialEq)]
pub struct DrillKind {
    /// A, B, C, … in the order of the kinds on the face.
    pub label: String,
    /// The hole (`None` for a cutout).
    pub hole: Option<Hole>,
    /// The cutout (`None` for a hole).
    pub opening: Option<Opening>,
    pub count: usize,
}

impl DrillKind {
    /// Its size: `Ø4.2`, `Ø22.5`, `92 x 92`, `45 x 92 R2`.
    pub fn size(&self) -> String {
        match (&self.hole, &self.opening) {
            (Some(h), _) => format!("Ø{}", dim(h.diameter)),
            (_, Some(o)) if o.round => format!("Ø{}", dim(o.w)),
            (_, Some(o)) if o.radius > 0.0 => format!("{} x {} R{}", dim(o.w), dim(o.h), dim(o.radius)),
            (_, Some(o)) => format!("{} x {}", dim(o.w), dim(o.h)),
            _ => String::new(),
        }
    }

    /// The thread to cut or the screw it takes (`M5`, `(M5)`), empty for none.
    pub fn thread(&self) -> String {
        match &self.hole {
            Some(h) if !h.thread.is_empty() => h.thread.clone(),
            Some(h) if !h.thread_size.is_empty() => format!("({})", h.thread_size),
            _ => String::new(),
        }
    }

    /// How it is made, in the current language: the fastening of a hole, `Cutout`.
    pub fn operation(&self) -> &'static str {
        match &self.hole {
            Some(h) => h.fastening.title(),
            None => tr("Cutout"),
        }
    }
}

/// A hole or cutout of a drilling pattern: the item it is for, its kind (index in
/// [`FaceDrilling::kinds`]), its place on the face (face coordinates, y down) and its
/// dimensions from the bottom left corner of the face (as the NC data, y up).
#[derive(Clone, Debug, PartialEq)]
pub struct DrillFeature {
    pub item: u64,
    pub kind: usize,
    pub pos: P,
    pub x: f32,
    pub y: f32,
    /// Its outline on the face (cutouts), empty for holes.
    pub outline: Vec<P>,
}

/// The drilling pattern of one sheet (a face or a leaf of the door): its size, the kinds
/// of holes and cutouts and every one of them, by kind, then from the bottom left.
#[derive(Clone, Debug, PartialEq)]
pub struct FaceDrilling {
    pub face: Face,
    /// The leaf of a door of several leaves.
    pub leaf: Option<usize>,
    pub size: (f32, f32),
    pub kinds: Vec<DrillKind>,
    pub features: Vec<DrillFeature>,
}

/// A dimension in mm: whole millimetres without decimals, otherwise to a tenth.
pub fn dim(v: f32) -> String {
    let r = (v * 10.0).round() / 10.0;
    if (r - r.round()).abs() < 1e-3 {
        format!("{:.0}", r)
    } else {
        format!("{r:.1}")
    }
}

/// The letter of kind `i`: A … Z, then AA, AB, ….
fn kind_label(i: usize) -> String {
    let letter = |k: usize| char::from(b'A' + (k % 26) as u8);
    if i < 26 {
        letter(i).to_string()
    } else {
        format!("{}{}", letter(i / 26 - 1), letter(i))
    }
}

/// The drilling patterns of the faces of an enclosure that have holes or cutouts (the
/// mounting plate first): the holes of [`holes_on`] and the openings of [`openings`].
pub fn drilling(e: &Enclosure, parts: &[Part]) -> Vec<FaceDrilling> {
    let mut out = Vec::new();
    for sheet in sheets(e) {
        let (fw, fh) = sheet.size(e);
        let holes = sheet_holes(e, parts, sheet);
        let opens = sheet_openings(e, parts, sheet);
        if holes.is_empty() && opens.is_empty() {
            continue;
        }
        let mut kinds: Vec<DrillKind> = Vec::new();
        let mut features = Vec::new();
        let same_hole = |a: &Hole, b: &Hole| (a.diameter - b.diameter).abs() < 0.01 && a.thread == b.thread && a.thread_size == b.thread_size && a.fastening == b.fastening;
        let same_opening = |a: &Opening, b: &Opening| a.round == b.round && (a.w - b.w).abs() < 0.01 && (a.h - b.h).abs() < 0.01 && (a.radius - b.radius).abs() < 0.01 && (a.angle - b.angle).abs() < 0.01;
        for h in holes {
            let k = match kinds.iter().position(|k| k.hole.as_ref().is_some_and(|o| same_hole(o, &h))) {
                Some(k) => k,
                None => {
                    kinds.push(DrillKind { label: String::new(), hole: Some(h.clone()), opening: None, count: 0 });
                    kinds.len() - 1
                }
            };
            kinds[k].count += 1;
            features.push(DrillFeature { item: h.item, kind: k, pos: h.pos, x: h.pos.x, y: fh - h.pos.y, outline: Vec::new() });
        }
        for o in opens {
            let k = match kinds.iter().position(|k| k.opening.as_ref().is_some_and(|x| same_opening(x, &o))) {
                Some(k) => k,
                None => {
                    kinds.push(DrillKind { label: String::new(), hole: None, opening: Some(o.clone()), count: 0 });
                    kinds.len() - 1
                }
            };
            kinds[k].count += 1;
            features.push(DrillFeature { item: o.item, kind: k, pos: o.center, x: o.center.x, y: fh - o.center.y, outline: o.outline() });
        }
        for (i, k) in kinds.iter_mut().enumerate() {
            k.label = kind_label(i);
        }
        features.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.y.total_cmp(&b.y)).then(a.x.total_cmp(&b.x)));
        out.push(FaceDrilling { face: sheet.face, leaf: sheet.leaf, size: (fw, fh), kinds, features });
    }
    out
}

// ---------------------------------------------------------------------- NC export

fn csv_num(v: f32) -> String {
    format!("{v:.1}")
}

/// Drilling and cutout list of a sheet (mounting plate, door or one of its leaves, wall)
/// for NC machines: coordinates from the bottom left corner, y up.
pub fn nc_csv(prj: &Project, enc: usize, sheet: impl Into<Sheet>) -> String {
    let sheet = sheet.into();
    let e = &prj.layout.enclosures[enc];
    let (_, fh) = sheet.size(e);
    let title = sheet.title(e);
    let name = |id: u64| e.item(id).map(|i| if i.kind == ItemKind::Device { i.tag.clone() } else { i.kind.title().to_string() }).unwrap_or_default();
    let mut out = String::from("Enclosure;Face;Item;Operation;X;Y;Diameter;Thread;Width;Height;Angle;Radius\n");
    for h in sheet_holes(e, &prj.parts, sheet) {
        let op = h.fastening.operation();
        let thread = if h.fastening == Fastening::Tapped { &h.thread } else { &h.thread_size };
        let _ = writeln!(out, "{};{title};{};{op};{};{};{};{};;;;", e.name, name(h.item), csv_num(h.pos.x), csv_num(fh - h.pos.y), csv_num(h.diameter), thread);
    }
    for o in sheet_openings(e, &prj.parts, sheet) {
        let (op, dia) = if o.round { ("Cutout round", csv_num(o.w)) } else { ("Cutout rectangular", String::new()) };
        let (angle, radius) = (if o.angle != 0.0 { csv_num(o.angle) } else { String::new() }, if o.radius > 0.0 { csv_num(o.radius) } else { String::new() });
        let _ = writeln!(out, "{};{title};{};{op};{};{};{dia};;{};{};{angle};{radius}", e.name, name(o.item), csv_num(o.center.x), csv_num(fh - o.center.y), csv_num(o.w), csv_num(o.h));
    }
    out
}

/// A sheet (face or leaf of the door) as DXF for NC machines: outline, holes on one
/// layer per diameter (`HOLE_D4.2`), thread marks, weld studs (`STUD_M5`), cutouts;
/// millimetres, origin bottom left.
pub fn nc_dxf(prj: &Project, enc: usize, sheet: impl Into<Sheet>) -> String {
    let sheet = sheet.into();
    let e = &prj.layout.enclosures[enc];
    let (fw, fh) = sheet.size(e);
    let all = sheet_holes(e, &prj.parts, sheet);
    let (studs, hs): (Vec<Hole>, Vec<Hole>) = all.into_iter().partition(|h| h.fastening == Fastening::Stud);
    let mut diameters: Vec<f32> = Vec::new();
    for h in &hs {
        if !diameters.iter().any(|d| (d - h.diameter).abs() < 0.01) {
            diameters.push(h.diameter);
        }
    }
    let hole_layer = |d: f32| format!("HOLE_D{d:.1}");
    let stud_layer = |h: &Hole| format!("STUD_{}", h.thread_size);
    let outline = sheet.key(e).to_uppercase();
    let outline = outline.as_str();
    let mut layers: Vec<(String, u8)> = vec![(outline.into(), 7), ("CUTOUT".into(), 1), ("THREAD".into(), 3)];
    layers.extend(diameters.iter().map(|d| (hole_layer(*d), 5)));
    for h in &studs {
        if !layers.iter().any(|l| l.0 == stud_layer(h)) {
            layers.push((stud_layer(h), 6));
        }
    }
    let mut out = String::new();
    crate::dxf::header(&mut out, &layers);
    out.push_str("0\nSECTION\n2\nENTITIES\n");
    let pt = |out: &mut String, code: u16, q: P| {
        let _ = write!(out, "{}\n{:.3}\n{}\n{:.3}\n{}\n0.0\n", code, q.x, code + 10, fh - q.y, code + 20);
    };
    let rect = |out: &mut String, layer: &str, r: R| {
        let _ = write!(out, "0\nPOLYLINE\n8\n{layer}\n66\n1\n70\n1\n10\n0.0\n20\n0.0\n30\n0.0\n");
        for q in [r.min, p(r.max.x, r.min.y), r.max, p(r.min.x, r.max.y)] {
            let _ = write!(out, "0\nVERTEX\n8\n{layer}\n");
            pt(out, 10, q);
        }
        let _ = write!(out, "0\nSEQEND\n8\n{layer}\n");
    };
    let circle = |out: &mut String, layer: &str, c: P, r: f32| {
        let _ = write!(out, "0\nCIRCLE\n8\n{layer}\n");
        pt(out, 10, c);
        let _ = write!(out, "40\n{r:.3}\n");
    };
    rect(&mut out, outline, R::from_points(p(0.0, 0.0), p(fw, fh)));
    for h in &hs {
        circle(&mut out, &hole_layer(h.diameter), h.pos, h.diameter / 2.0);
        if !h.thread.is_empty() {
            // Thread marking: the nominal diameter on its own layer.
            let nominal: f32 = h.thread.trim_start_matches('M').parse().unwrap_or(h.diameter);
            circle(&mut out, "THREAD", h.pos, nominal / 2.0);
        }
    }
    for h in &studs {
        circle(&mut out, &stud_layer(h), h.pos, h.diameter / 2.0);
    }
    for o in sheet_openings(e, &prj.parts, sheet) {
        if o.round {
            circle(&mut out, "CUTOUT", o.center, o.w / 2.0);
        } else if o.angle == 0.0 && o.radius == 0.0 {
            let half = p(o.w / 2.0, o.h / 2.0);
            rect(&mut out, "CUTOUT", R::from_points(o.center - half, o.center + half));
        } else {
            let _ = write!(out, "0\nPOLYLINE\n8\nCUTOUT\n66\n1\n70\n1\n10\n0.0\n20\n0.0\n30\n0.0\n");
            for q in o.outline() {
                let _ = write!(out, "0\nVERTEX\n8\nCUTOUT\n");
                pt(&mut out, 10, q);
            }
            let _ = write!(out, "0\nSEQEND\n8\nCUTOUT\n");
        }
    }
    out.push_str("0\nENDSEC\n0\nEOF\n");
    out
}

// ---------------------------------------------------------------------- wire ends

/// Cross-section in mm² from texts like "1.5 mm²", "2,5" or "0.75mm2", of American
/// wire sizes too: "14 AWG", "AWG 2/0", "250 kcmil" (or MCM).
pub fn cross_section_mm2(s: &str) -> Option<f32> {
    let lower = s.trim().to_ascii_lowercase();
    if lower.contains("awg") {
        // The gauge before or after "AWG": "14 AWG", "AWG14", "#14 AWG".
        let spaced = lower.replace("awg", " awg ");
        let words: Vec<&str> = spaced.split_whitespace().collect();
        let i = words.iter().position(|w| *w == "awg")?;
        let gauge = |w: &str| awg_gauge(w.trim_start_matches('#'));
        let g = i.checked_sub(1).and_then(|j| gauge(words[j])).or_else(|| words.get(i + 1).and_then(|w| gauge(w)))?;
        return Some(crate::parts::awg_mm2(g));
    }
    let num: String = lower.chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',').collect();
    let v: f32 = num.replace(',', ".").parse().ok()?;
    let v = if lower.contains("kcmil") || lower.contains("mcm") { v * 0.506_707 } else { v };
    (v > 0.0).then_some(v)
}

/// The number of an AWG gauge: "14" → 14, "1/0" → 0, "4/0" (or "0000") → -3.
fn awg_gauge(s: &str) -> Option<f32> {
    let g = match s.split_once('/') {
        Some((zeros, "0")) => 1.0 - zeros.parse::<f32>().ok()?,
        Some(_) => return None,
        None if s.len() > 1 && s.chars().all(|c| c == '0') => 1.0 - s.len() as f32,
        None => s.parse().ok()?,
    };
    (-3.0..=40.0).contains(&g).then_some(g)
}

/// Wire end ferrule of a colour code ([`FerruleCode`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ferrule {
    pub cross_section: f32,
    pub color: &'static str,
    /// Sleeve length = stripping length.
    pub length: f32,
    pub part: &'static str,
    /// Part number of the twin ferrule of the size; empty where there is none.
    pub twin: &'static str,
}

const fn fer(cross_section: f32, color: &'static str, length: f32, part: &'static str, twin: &'static str) -> Ferrule {
    Ferrule { cross_section, color, length, part, twin }
}

/// The ferrules of DIN 46228-4 (0.14 to 0.34 mm² not in the standard).
pub const FERRULES: [Ferrule; 15] = [
    fer(0.14, tr_const!("grey"), 8.0, "GEN.FER-0.14", ""),
    fer(0.25, tr_const!("yellow"), 8.0, "GEN.FER-0.25", ""),
    fer(0.34, tr_const!("turquoise"), 8.0, "GEN.FER-0.34", "GEN.FER2-0.34"),
    fer(0.5, tr_const!("white"), 8.0, "GEN.FER-0.5", "GEN.FER2-0.5"),
    fer(0.75, tr_const!("grey"), 8.0, "GEN.FER-0.75", "GEN.FER2-0.75"),
    fer(1.0, tr_const!("red"), 8.0, "GEN.FER-1.0", "GEN.FER2-1.0"),
    fer(1.5, tr_const!("black"), 8.0, "GEN.FER-1.5", "GEN.FER2-1.5"),
    fer(2.5, tr_const!("blue"), 8.0, "GEN.FER-2.5", "GEN.FER2-2.5"),
    fer(4.0, tr_const!("grey"), 10.0, "GEN.FER-4", "GEN.FER2-4"),
    fer(6.0, tr_const!("yellow"), 12.0, "GEN.FER-6", "GEN.FER2-6"),
    fer(10.0, tr_const!("red"), 12.0, "GEN.FER-10", "GEN.FER2-10"),
    fer(16.0, tr_const!("blue"), 12.0, "GEN.FER-16", "GEN.FER2-16"),
    fer(25.0, tr_const!("yellow"), 16.0, "GEN.FER-25", ""),
    fer(35.0, tr_const!("red"), 16.0, "GEN.FER-35", ""),
    fer(50.0, tr_const!("blue"), 20.0, "GEN.FER-50", ""),
];

/// The ferrules of the French colour code NF C 63-023.
pub const FRENCH_FERRULES: [Ferrule; 13] = [
    fer(0.34, tr_const!("green"), 8.0, "GEN.FERF-0.34", ""),
    fer(0.5, tr_const!("white"), 8.0, "GEN.FERF-0.5", "GEN.FER2F-0.5"),
    fer(0.75, tr_const!("blue"), 8.0, "GEN.FERF-0.75", "GEN.FER2F-0.75"),
    fer(1.0, tr_const!("red"), 8.0, "GEN.FERF-1.0", "GEN.FER2F-1.0"),
    fer(1.5, tr_const!("black"), 8.0, "GEN.FERF-1.5", "GEN.FER2F-1.5"),
    fer(2.5, tr_const!("grey"), 8.0, "GEN.FERF-2.5", "GEN.FER2F-2.5"),
    fer(4.0, tr_const!("orange"), 10.0, "GEN.FERF-4", "GEN.FER2F-4"),
    fer(6.0, tr_const!("green"), 12.0, "GEN.FERF-6", "GEN.FER2F-6"),
    fer(10.0, tr_const!("brown"), 12.0, "GEN.FERF-10", "GEN.FER2F-10"),
    fer(16.0, tr_const!("ivory"), 12.0, "GEN.FERF-16", "GEN.FER2F-16"),
    fer(25.0, tr_const!("black"), 16.0, "GEN.FERF-25", ""),
    fer(35.0, tr_const!("red"), 16.0, "GEN.FERF-35", ""),
    fer(50.0, tr_const!("blue"), 20.0, "GEN.FERF-50", ""),
];

/// The ferrules of the special colour series.
pub const SPECIAL_FERRULES: [Ferrule; 15] = [
    fer(0.14, tr_const!("grey"), 8.0, "GEN.FERS-0.14", ""),
    fer(0.25, tr_const!("light blue"), 8.0, "GEN.FERS-0.25", ""),
    fer(0.34, tr_const!("turquoise"), 8.0, "GEN.FERS-0.34", ""),
    fer(0.5, tr_const!("orange"), 8.0, "GEN.FERS-0.5", "GEN.FER2S-0.5"),
    fer(0.75, tr_const!("white"), 8.0, "GEN.FERS-0.75", "GEN.FER2S-0.75"),
    fer(1.0, tr_const!("yellow"), 8.0, "GEN.FERS-1.0", "GEN.FER2S-1.0"),
    fer(1.5, tr_const!("red"), 8.0, "GEN.FERS-1.5", "GEN.FER2S-1.5"),
    fer(2.5, tr_const!("blue"), 8.0, "GEN.FERS-2.5", "GEN.FER2S-2.5"),
    fer(4.0, tr_const!("grey"), 10.0, "GEN.FERS-4", "GEN.FER2S-4"),
    fer(6.0, tr_const!("black"), 12.0, "GEN.FERS-6", "GEN.FER2S-6"),
    fer(10.0, tr_const!("ivory"), 12.0, "GEN.FERS-10", "GEN.FER2S-10"),
    fer(16.0, tr_const!("green"), 12.0, "GEN.FERS-16", "GEN.FER2S-16"),
    fer(25.0, tr_const!("brown"), 16.0, "GEN.FERS-25", ""),
    fer(35.0, tr_const!("beige"), 16.0, "GEN.FERS-35", ""),
    fer(50.0, tr_const!("olive"), 20.0, "GEN.FERS-50", ""),
];

/// How much a wire may be larger than the nominal cross-section of its ferrule: AWG 18
/// (0.82 mm²) takes the ferrule of 0.75 mm², AWG 1/0 (53.5 mm²) the one of 50 mm².
const FERRULE_OVERSIZE: f32 = 1.1;

impl FerruleCode {
    pub const ALL: [FerruleCode; 3] = [FerruleCode::Din, FerruleCode::French, FerruleCode::Special];

    /// Key in the project file and on the command line.
    pub fn key(self) -> &'static str {
        match self {
            FerruleCode::Din => "din",
            FerruleCode::French => "french",
            FerruleCode::Special => "special",
        }
    }

    pub fn from_key(key: &str) -> Option<FerruleCode> {
        let key = key.trim().to_ascii_lowercase();
        FerruleCode::ALL.into_iter().find(|c| c.key() == key)
    }

    /// Name, translated.
    pub fn title(self) -> &'static str {
        tr(match self {
            FerruleCode::Din => tr_const!("DIN 46228-4 / UL 486F"),
            FerruleCode::French => tr_const!("NF C 63-023 (French colour code)"),
            FerruleCode::Special => tr_const!("Special colour series"),
        })
    }

    /// Its ferrules, smallest first.
    pub fn ferrules(self) -> &'static [Ferrule] {
        match self {
            FerruleCode::Din => &FERRULES,
            FerruleCode::French => &FRENCH_FERRULES,
            FerruleCode::Special => &SPECIAL_FERRULES,
        }
    }

    /// The ferrule of a wire of `cross_section` mm²: the smallest that takes it, metric
    /// and AWG wires alike ([`FERRULE_OVERSIZE`]); a twin ferrule only of a size there
    /// are twin ferrules of. None for a wire larger than the largest one.
    pub fn ferrule_for(self, cross_section: f32, twin: bool) -> Option<Ferrule> {
        self.ferrules().iter().find(|f| cross_section <= f.cross_section * FERRULE_OVERSIZE + 1e-4).filter(|f| !twin || !f.twin.is_empty()).copied()
    }

    /// The colour code and ferrule of a part number (single or twin ferrule).
    pub fn of_part(part: &str) -> Option<(FerruleCode, Ferrule)> {
        FerruleCode::ALL.into_iter().find_map(|c| c.ferrules().iter().find(|f| f.part == part || f.twin == part).map(|f| (c, *f)))
    }
}

impl Ferrule {
    /// Colour of its plastic collar.
    pub fn rgb(&self) -> [u8; 3] {
        match self.color {
            "turquoise" => [64, 200, 200],
            "white" => [235, 235, 235],
            "grey" => [140, 140, 145],
            "red" => [210, 40, 40],
            "black" => [35, 35, 38],
            "blue" => [40, 90, 200],
            "light blue" => [120, 180, 230],
            "yellow" => [235, 200, 40],
            "orange" => [235, 130, 30],
            "green" => [40, 150, 60],
            "brown" => [120, 70, 35],
            "ivory" => [240, 232, 200],
            "beige" => [215, 195, 155],
            "olive" => [120, 125, 45],
            _ => [128, 128, 128],
        }
    }

    /// Length of its plastic collar over the insulation (DIN 46228-4).
    pub fn collar(&self) -> f32 {
        if self.cross_section <= 4.0 { 6.0 } else if self.cross_section <= 6.0 { 8.0 } else if self.cross_section <= 16.0 { 10.0 } else { 12.0 }
    }
}

impl Termination {
    pub const ALL: [Termination; 8] =
        [Termination::Bare, Termination::Ferrule, Termination::TwinFerrule, Termination::Ring, Termination::Fork, Termination::Pin, Termination::Blade, Termination::Receptacle];

    /// Key in the project file and on the command line.
    pub fn key(self) -> &'static str {
        match self {
            Termination::Bare => "bare",
            Termination::Ferrule => "ferrule",
            Termination::TwinFerrule => "twin_ferrule",
            Termination::Ring => "ring",
            Termination::Fork => "fork",
            Termination::Pin => "pin",
            Termination::Blade => "blade",
            Termination::Receptacle => "receptacle",
        }
    }

    pub fn from_key(key: &str) -> Option<Termination> {
        let key = key.trim().to_ascii_lowercase().replace([' ', '-'], "_");
        Termination::ALL.into_iter().find(|t| t.key() == key)
    }

    /// Name, translated.
    pub fn title(self) -> &'static str {
        tr(match self {
            Termination::Bare => tr_const!("Bare conductor"),
            Termination::Ferrule => tr_const!("Ferrule"),
            Termination::TwinFerrule => tr_const!("Twin ferrule"),
            Termination::Ring => tr_const!("Ring terminal"),
            Termination::Fork => tr_const!("Fork terminal"),
            Termination::Pin => tr_const!("Pin terminal"),
            Termination::Blade => tr_const!("Flat plug"),
            Termination::Receptacle => tr_const!("Flat receptacle"),
        })
    }
}

/// Insulation of a crimp terminal, coloured by the cross-sections it takes: red up to
/// 1 mm², blue 1.5 to 2.5 mm², yellow 4 to 6 mm²; uninsulated from 10 mm².
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    Red,
    Blue,
    Yellow,
    Uninsulated,
}

/// A crimped terminal (ring, fork, pin, flat plug or receptacle) for a cross-section.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crimp {
    pub kind: Termination,
    pub band: Band,
    pub cross_section: f32,
}

impl Crimp {
    /// The terminal of `kind` for `cross_section` (mm²); forks, flat plugs and
    /// receptacles only up to 6 mm².
    pub fn for_cross_section(kind: Termination, cross_section: f32) -> Option<Crimp> {
        let band = match cross_section {
            c if c <= 1.0 => Band::Red,
            c if c <= 2.5 => Band::Blue,
            c if c <= 6.0 => Band::Yellow,
            _ => Band::Uninsulated,
        };
        let fits = matches!(kind, Termination::Ring | Termination::Pin) || (matches!(kind, Termination::Fork | Termination::Blade | Termination::Receptacle) && band != Band::Uninsulated);
        fits.then_some(Crimp { kind, band, cross_section })
    }

    /// Colour of the insulation ("" uninsulated).
    pub fn color(&self) -> &'static str {
        match self.band {
            Band::Red => "red",
            Band::Blue => "blue",
            Band::Yellow => "yellow",
            Band::Uninsulated => "",
        }
    }

    pub fn rgb(&self) -> [u8; 3] {
        match self.band {
            Band::Red => [210, 40, 40],
            Band::Blue => [40, 90, 200],
            Band::Yellow => [235, 200, 40],
            Band::Uninsulated => SLEEVE_RGB,
        }
    }

    /// Generic part number: the kind and the colour, uninsulated ones the cross-section.
    pub fn part(&self) -> String {
        let kind = match (self.kind, self.band) {
            (Termination::Ring, Band::Uninsulated) => "LUG",
            (Termination::Ring, _) => "RING",
            (Termination::Fork, _) => "FORK",
            (Termination::Blade, _) => "TAB",
            (Termination::Receptacle, _) => "REC",
            _ => "PIN",
        };
        match self.band {
            Band::Red => format!("GEN.{kind}-R"),
            Band::Blue => format!("GEN.{kind}-B"),
            Band::Yellow => format!("GEN.{kind}-Y"),
            Band::Uninsulated => format!("GEN.{kind}-{}", self.cross_section),
        }
    }

    /// Length of the crimp barrel, mm.
    pub fn barrel(&self) -> f32 {
        match self.band {
            Band::Red => 5.0,
            Band::Blue => 6.0,
            Band::Yellow => 7.0,
            Band::Uninsulated => 8.0 + 2.0 * self.cross_section.sqrt(),
        }
    }

    /// Hole of a ring or fork terminal, mm: M4, M5, M6 or M8.
    pub fn hole(&self) -> f32 {
        match self.band {
            Band::Red | Band::Blue => 4.3,
            Band::Yellow => 5.3,
            Band::Uninsulated if self.cross_section <= 16.0 => 6.4,
            Band::Uninsulated => 8.4,
        }
    }
}

/// End treatment of a wire end: how it is terminated, its stripping length, the
/// ferrule or the crimped terminal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EndTreatment {
    pub kind: Termination,
    pub strip: f32,
    pub ferrule: Option<Ferrule>,
    pub crimp: Option<Crimp>,
    /// The twin ferrule is the one of the other wire ([`Twin::first`]): it is listed,
    /// counted and shown there.
    pub shared: bool,
}

/// The shape of an [`EndPiece`]: round along the wire, or flat (lying across the
/// direction it is seen from best, see [`EndPiece::mesh`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EndShape {
    Round,
    /// A plate (flat plug, receptacle).
    Flat { width: f32, thickness: f32 },
    /// The ring of a ring terminal round the connection point.
    Ring { outer: f32, hole: f32, thickness: f32 },
    /// The open ring of a fork terminal.
    Fork { width: f32, slot: f32, thickness: f32 },
}

/// A piece of a wire end as it is seen (3D view): length along the wire and diameter in
/// mm, colour and shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EndPiece {
    pub length: f32,
    pub diameter: f32,
    pub color: [u8; 3],
    pub shape: EndShape,
}

/// Tin-plated copper of a ferrule sleeve or a terminal.
pub const SLEEVE_RGB: [u8; 3] = [196, 198, 204];
/// Bare copper of a stripped conductor.
pub const COPPER_RGB: [u8; 3] = [196, 112, 52];

/// Diameter (mm) of the stranded conductor of a cross-section (mm²).
pub fn conductor_diameter(cross_section: f32) -> f32 {
    1.15 * 2.0 * (cross_section / std::f32::consts::PI).sqrt()
}

impl EndTreatment {
    /// Part number of the ferrule or terminal; empty for the bare conductor.
    pub fn part(&self) -> String {
        match (self.kind, self.ferrule, self.crimp) {
            (Termination::TwinFerrule, Some(f), _) => f.twin.to_string(),
            (_, Some(f), _) => f.part.to_string(),
            (_, _, Some(c)) => c.part(),
            _ => String::new(),
        }
    }

    /// What the end gets, translated: "Ferrule 1.5 mm² black", "Ring terminal red", …
    pub fn describe(&self) -> String {
        match (self.kind, self.ferrule, self.crimp) {
            (Termination::TwinFerrule, Some(f), _) => tr_fmt!("Twin ferrule {} mm² {}", f.cross_section, tr(f.color)),
            (_, Some(f), _) => tr_fmt!("Ferrule {} mm² {}", f.cross_section, tr(f.color)),
            (_, _, Some(c)) if c.band == Band::Uninsulated => self.kind.title().to_string(),
            (_, _, Some(c)) => format!("{} {}", self.kind.title(), tr(c.color())),
            (Termination::Bare, ..) => self.kind.title().to_string(),
            _ => tr_fmt!("{}: none for this cross-section", self.kind.title()),
        }
    }

    /// It should have a ferrule or terminal but none fits its cross-section.
    pub fn missing(&self) -> bool {
        self.kind != Termination::Bare && self.ferrule.is_none() && self.crimp.is_none()
    }

    /// How far from its tip the end of a wire of `cross_section` (with the outer
    /// diameter `outer`) cannot bend: the ferrule with its collar, the terminal with its
    /// sleeve, the conductor inside the twin ferrule of the other wire ([`Self::pieces`]);
    /// nothing for a bare conductor. The wire runs straight that far out of its
    /// connection point.
    pub fn straight(&self, cross_section: &str, outer: f32) -> f32 {
        if self.ferrule.is_none() && self.crimp.is_none() {
            return 0.0;
        }
        self.pieces(cross_section, outer).iter().map(|p| p.length).sum()
    }

    /// How the end of a wire of `cross_section` (with the outer diameter `outer`) looks,
    /// from the tip inwards: the metal sleeve of a ferrule and its collar over the
    /// insulation, the ring, fork, pin or plate of a terminal and its insulated sleeve
    /// (its barrel when uninsulated), or the bare copper conductor as long as it is
    /// stripped.
    pub fn pieces(&self, cross_section: &str, outer: f32) -> Vec<EndPiece> {
        let core = conductor_diameter(cross_section_mm2(cross_section).unwrap_or(1.0)).min(outer * 0.8);
        let round = |length: f32, diameter: f32, color: [u8; 3]| EndPiece { length, diameter, color, shape: EndShape::Round };
        if self.shared {
            // Only its conductor, inside the twin ferrule of the other wire.
            return vec![round(self.strip, core, COPPER_RGB)];
        }
        if let Some(f) = self.ferrule {
            return match self.kind {
                Termination::TwinFerrule => vec![round(f.length, core * 1.5 + 0.3, SLEEVE_RGB), round(f.collar() * 1.4, outer * 1.7, f.rgb())],
                _ => vec![round(f.length, core + 0.3, SLEEVE_RGB), round(f.collar(), outer + 0.8, f.rgb())],
            };
        }
        let Some(c) = self.crimp else { return vec![round(self.strip, core, COPPER_RGB)] };
        let insulated = c.band != Band::Uninsulated;
        let thickness = if insulated { 0.8 } else { 1.5 };
        // The ring and the fork lie round the connection point at the tip.
        let eye = c.hole() * 1.9;
        let neck = 2.5;
        let tongue = match c.kind {
            Termination::Ring => EndPiece { length: eye / 2.0 + neck, diameter: eye, color: SLEEVE_RGB, shape: EndShape::Ring { outer: eye, hole: c.hole(), thickness } },
            Termination::Fork => EndPiece { length: eye / 2.0 + neck, diameter: eye, color: SLEEVE_RGB, shape: EndShape::Fork { width: eye, slot: c.hole(), thickness } },
            Termination::Blade => EndPiece { length: 8.0, diameter: 6.3, color: SLEEVE_RGB, shape: EndShape::Flat { width: 6.3, thickness: 0.8 } },
            Termination::Receptacle => EndPiece { length: 9.0, diameter: 7.4, color: SLEEVE_RGB, shape: EndShape::Flat { width: 7.4, thickness: 3.0 } },
            _ => {
                let pin = match c.band {
                    Band::Red | Band::Blue => 1.9,
                    Band::Yellow => 2.8,
                    Band::Uninsulated => core * 0.8,
                };
                round(if insulated { 10.0 } else { 16.0 }, pin, SLEEVE_RGB)
            }
        };
        let sleeve = if insulated { round(c.barrel() + 3.0, outer + 1.6, c.rgb()) } else { round(c.barrel(), core + 1.6, SLEEVE_RGB) };
        vec![tongue, sleeve]
    }
}

impl EndPiece {
    /// Triangles of a flat piece (none for a round one) with its tip at `tip`, running
    /// along `dir` (a unit vector, into the wire) and lying flat across `up` (a unit
    /// vector square to `dir`): the ring and the fork round the tip, a plate from it.
    pub fn mesh(&self, tip: V3, dir: V3, up: V3) -> Vec<Tri> {
        let side = cross(dir, up);
        let at = |along: f32, across: f32| add(add(tip, scale(dir, along)), scale(side, across));
        // A plate from `from` to `to` along the wire, `width` wide, round `across`.
        let plate = |from: f32, to: f32, across: f32, width: f32, thickness: f32| slab(at(from, across), at(to, across), side, width / 2.0, up, thickness / 2.0);
        match self.shape {
            EndShape::Round => Vec::new(),
            EndShape::Flat { width, thickness } => plate(0.0, self.length, 0.0, width, thickness),
            EndShape::Ring { outer, hole, thickness } => {
                let mut v = annulus(tip, dir, side, up, outer / 2.0, hole / 2.0, thickness / 2.0, 0.0, std::f32::consts::TAU);
                v.extend(plate(outer * 0.3, self.length, 0.0, outer * 0.45, thickness));
                v
            }
            EndShape::Fork { width, slot, thickness } => {
                // The closed half towards the wire, the prongs open away from it.
                let (r, prong) = (width / 2.0, (width - slot) / 2.0);
                let half = std::f32::consts::PI;
                let mut v = annulus(tip, dir, side, up, r, slot / 2.0, thickness / 2.0, -half / 2.0, half / 2.0);
                for s in [-1.0, 1.0] {
                    v.extend(plate(-r, 0.0, s * (slot / 2.0 + prong / 2.0), prong, thickness));
                }
                v.extend(plate(r * 0.6, self.length, 0.0, width * 0.45, thickness));
                v
            }
        }
    }
}

/// Two triangles of a quad, wound to face `out`.
fn quad(q: [V3; 4], out: V3) -> [Tri; 2] {
    let n = cross(sub(q[1], q[0]), sub(q[2], q[0]));
    if dot(n, out) >= 0.0 {
        [[q[0], q[1], q[2]], [q[0], q[2], q[3]]]
    } else {
        [[q[0], q[2], q[1]], [q[0], q[3], q[2]]]
    }
}

/// A box from `a` to `b`, `half_w` to both sides along `side` and `half_t` along `up`.
fn slab(a: V3, b: V3, side: V3, half_w: f32, up: V3, half_t: f32) -> Vec<Tri> {
    let c = |p: V3, s: f32, u: f32| add(add(p, scale(side, s * half_w)), scale(up, u * half_t));
    let dir = norm(sub(b, a));
    let faces = [
        ([c(a, -1.0, 1.0), c(b, -1.0, 1.0), c(b, 1.0, 1.0), c(a, 1.0, 1.0)], up),
        ([c(a, -1.0, -1.0), c(b, -1.0, -1.0), c(b, 1.0, -1.0), c(a, 1.0, -1.0)], scale(up, -1.0)),
        ([c(a, 1.0, -1.0), c(b, 1.0, -1.0), c(b, 1.0, 1.0), c(a, 1.0, 1.0)], side),
        ([c(a, -1.0, -1.0), c(b, -1.0, -1.0), c(b, -1.0, 1.0), c(a, -1.0, 1.0)], scale(side, -1.0)),
        ([c(a, -1.0, -1.0), c(a, 1.0, -1.0), c(a, 1.0, 1.0), c(a, -1.0, 1.0)], scale(dir, -1.0)),
        ([c(b, -1.0, -1.0), c(b, 1.0, -1.0), c(b, 1.0, 1.0), c(b, -1.0, 1.0)], dir),
    ];
    faces.into_iter().flat_map(|(q, out)| quad(q, out)).collect()
}

/// A flat ring round `centre` (in the plane of `dir` and `side`, `half_t` thick along
/// `up`) from the angle `from` to `to`, measured from `dir` towards `side`.
#[allow(clippy::too_many_arguments)]
fn annulus(centre: V3, dir: V3, side: V3, up: V3, outer: f32, inner: f32, half_t: f32, from: f32, to: f32) -> Vec<Tri> {
    const STEPS: usize = 16;
    let radial = |a: f32| add(scale(dir, a.cos()), scale(side, a.sin()));
    let at = |a: f32, r: f32, u: f32| add(add(centre, scale(radial(a), r)), scale(up, u * half_t));
    let mut v = Vec::new();
    for k in 0..STEPS {
        let (a0, a1) = (from + (to - from) * k as f32 / STEPS as f32, from + (to - from) * (k + 1) as f32 / STEPS as f32);
        let mid = (a0 + a1) / 2.0;
        for u in [1.0, -1.0] {
            v.extend(quad([at(a0, inner, u), at(a0, outer, u), at(a1, outer, u), at(a1, inner, u)], scale(up, u)));
        }
        v.extend(quad([at(a0, outer, -1.0), at(a1, outer, -1.0), at(a1, outer, 1.0), at(a0, outer, 1.0)], radial(mid)));
        v.extend(quad([at(a0, inner, -1.0), at(a1, inner, -1.0), at(a1, inner, 1.0), at(a0, inner, 1.0)], scale(radial(mid), -1.0)));
    }
    v
}

/// End treatment for a wire cross-section (None if it is unknown): a ferrule of the
/// colour code `code` where the wire ends get them (`ferrules`), otherwise the bare
/// conductor.
pub fn end_treatment(cross_section: &str, ferrules: bool, code: FerruleCode) -> Option<EndTreatment> {
    treatment(cross_section, if ferrules { Termination::Ferrule } else { Termination::Bare }, code)
}

/// End treatment of a wire of `cross_section` terminated as `kind` with the ferrules of
/// the colour code `code` (None for an unknown cross-section); a cross-section without
/// such a ferrule or terminal is stripped bare and has none ([`EndTreatment::missing`]).
pub fn treatment(cross_section: &str, kind: Termination, code: FerruleCode) -> Option<EndTreatment> {
    let cs = cross_section_mm2(cross_section)?;
    let bare = EndTreatment { kind: Termination::Bare, strip: if cs <= 2.5 { 8.0 } else if cs <= 6.0 { 10.0 } else { 12.0 }, ferrule: None, crimp: None, shared: false };
    Some(match kind {
        Termination::Bare => bare,
        Termination::Ferrule | Termination::TwinFerrule => match code.ferrule_for(cs, kind == Termination::TwinFerrule) {
            Some(f) => EndTreatment { kind, strip: f.length, ferrule: Some(f), crimp: None, shared: false },
            None => EndTreatment { kind, ..bare },
        },
        _ => {
            let crimp = Crimp::for_cross_section(kind, cs);
            EndTreatment { kind, strip: crimp.map(|c| c.barrel() + 1.0).unwrap_or(bare.strip), ferrule: None, crimp, shared: false }
        }
    })
}

/// The device of a connection point designation (`-K1:13` → `-K1`).
fn device_of(at: &str) -> &str {
    at.rsplit_once(':').map(|(d, _)| d).unwrap_or(at)
}

/// The termination chosen for the end `at` of the wire from `at` to `to`: for that
/// wire, for every wire at `at`, or for every wire at its device.
pub fn termination_of(prj: &Project, at: &str, to: &str) -> Option<Termination> {
    let chosen = &prj.layout.terminations;
    chosen
        .iter()
        .find(|t| t.at == at && !t.to.is_empty() && t.to == to)
        .or_else(|| chosen.iter().find(|t| t.at == at && t.to.is_empty()))
        .or_else(|| chosen.iter().find(|t| t.at == device_of(at) && t.to.is_empty()))
        .map(|t| t.kind)
}

/// End treatment of the end `at` of the wire of `cross_section` from `at` to `to`: as
/// chosen ([`termination_of`]), otherwise ferrules or the bare conductor as the layout
/// sets; without the other wires at `at` (twin ferrules are not paired, see [`Ends`]).
pub fn end_at(prj: &Project, at: &str, to: &str, cross_section: &str) -> Option<EndTreatment> {
    let code = prj.layout.ferrule_code;
    match termination_of(prj, at, to) {
        Some(kind) => treatment(cross_section, kind, code),
        None => end_treatment(cross_section, prj.layout.ferrules, code),
    }
}

/// Chooses the termination of the wire ends at `at` (a connection point or a device),
/// only of the wire to `to` where it is given; `None` goes back to the layout setting.
pub fn set_termination(prj: &mut Project, at: &str, to: &str, kind: Option<Termination>) {
    let (at, to) = (at.trim(), to.trim());
    prj.layout.terminations.retain(|t| !(t.at == at && t.to == to));
    if let Some(kind) = kind {
        prj.layout.terminations.push(WireTermination { at: at.into(), to: to.into(), kind });
    }
}

/// A twin ferrule the wire end shares with another wire at its connection point
/// ([`Ends::twin`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Twin {
    /// The far end of the other wire.
    pub with: String,
    /// Cross-section of the other wire.
    pub cross_section: String,
    /// This end carries the ferrule (it is listed, counted and shown here); the other
    /// one shares it.
    pub first: bool,
}

/// The wires ending at each connection point, to tell which of them share a twin
/// ferrule: the termination of a wire end with all it depends on.
#[derive(Clone, Debug, Default)]
pub struct Ends {
    /// Connection point → (far end, cross-section) of each wire ending there.
    at: HashMap<String, Vec<(String, String)>>,
}

impl Ends {
    /// The ends of the wires (from, to, cross-section).
    pub fn of(wires: impl IntoIterator<Item = (String, String, String)>) -> Ends {
        let mut at: HashMap<String, Vec<(String, String)>> = HashMap::new();
        for (from, to, cs) in wires {
            if from == to {
                continue;
            }
            at.entry(from.clone()).or_default().push((to.clone(), cs.clone()));
            at.entry(to).or_default().push((from, cs));
        }
        for v in at.values_mut() {
            v.sort_by(|a, b| natural_cmp(&a.0, &b.0));
        }
        Ends { at }
    }

    /// The ends of the connections of a project, cable cores included.
    pub fn new(prj: &Project, conns: &[crate::netlist::Connection]) -> Ends {
        Ends::of(conns.iter().map(|c| (crate::netlist::pin_designation(prj, c.a), crate::netlist::pin_designation(prj, c.b), c.cross_section.clone())))
    }

    /// The far ends and cross-sections of the wires at the connection point `at`.
    pub fn wires_at(&self, at: &str) -> &[(String, String)] {
        self.at.get(at).map_or(&[], Vec::as_slice)
    }

    /// The connection points with wires, in no particular order.
    pub fn points(&self) -> impl Iterator<Item = &str> {
        self.at.keys().map(String::as_str)
    }

    /// Termination of the end `at` of the wire to `to` if it had none chosen for this
    /// wire: as chosen for the connection point or its device, otherwise a twin ferrule
    /// where the layout sets them and exactly two wires end at `at` (the other one
    /// without a choice), otherwise ferrules or the bare conductor as the layout sets.
    pub fn default_kind(&self, prj: &Project, at: &str, to: &str) -> Termination {
        if let Some(kind) = termination_of(prj, at, "") {
            return kind;
        }
        let wires = self.wires_at(at);
        if !prj.layout.ferrules {
            Termination::Bare
        } else if prj.layout.twin_ferrules && wires.len() == 2 && wires.iter().filter(|(o, _)| o != to).all(|(o, _)| termination_of(prj, at, o).is_none()) {
            Termination::TwinFerrule
        } else {
            Termination::Ferrule
        }
    }

    /// Termination of the end `at` of the wire to `to`: as chosen
    /// ([`termination_of`]), otherwise [`Ends::default_kind`].
    pub fn kind(&self, prj: &Project, at: &str, to: &str) -> Termination {
        termination_of(prj, at, to).unwrap_or_else(|| self.default_kind(prj, at, to))
    }

    /// The other wire sharing the twin ferrule of the end `at` of the wire to `to`: the
    /// wire ends at `at` with twin ferrules pair up in the order of their far ends.
    pub fn twin(&self, prj: &Project, at: &str, to: &str) -> Option<Twin> {
        let twins: Vec<&(String, String)> = self.wires_at(at).iter().filter(|(o, _)| self.kind(prj, at, o) == Termination::TwinFerrule).collect();
        let i = twins.iter().position(|(o, _)| o == to)?;
        let (with, cross_section) = twins.get(i ^ 1)?;
        Some(Twin { with: with.clone(), cross_section: cross_section.clone(), first: i % 2 == 0 })
    }

    /// End treatment of the end `at` of the wire of `cross_section` to `to`; a shared
    /// twin ferrule is the one for the larger of the two cross-sections.
    pub fn end(&self, prj: &Project, at: &str, to: &str, cross_section: &str) -> Option<EndTreatment> {
        let kind = self.kind(prj, at, to);
        let twin = if kind == Termination::TwinFerrule { self.twin(prj, at, to) } else { None };
        let code = prj.layout.ferrule_code;
        let Some(t) = twin else { return treatment(cross_section, kind, code) };
        let mm2 = |cs: &str| cross_section_mm2(cs).unwrap_or(0.0);
        let larger = if mm2(&t.cross_section) > mm2(cross_section) { t.cross_section.as_str() } else { cross_section };
        treatment(larger, kind, code).map(|e| EndTreatment { shared: !t.first, ..e })
    }
}

/// Chooses the termination of the end `at` of the wire to `to` (`to` empty: of all
/// wires at `at`, see [`set_termination`]). A twin ferrule chosen for one wire is
/// chosen for the other wire at `at` too, where there is exactly one; choosing
/// something else for a wire with a twin ferrule takes it from the other one as well.
pub fn choose_termination(prj: &mut Project, ends: &Ends, at: &str, to: &str, kind: Option<Termination>) {
    let (at, to) = (at.trim(), to.trim());
    if !to.is_empty() {
        let others: Vec<String> = ends.wires_at(at).iter().filter(|(o, _)| o != to).map(|(o, _)| o.clone()).collect();
        let was_twin = ends.kind(prj, at, to) == Termination::TwinFerrule;
        if let [other] = others.as_slice() {
            if kind == Some(Termination::TwinFerrule) {
                set_termination(prj, at, other, kind);
            } else if was_twin && termination_of(prj, at, other) == Some(Termination::TwinFerrule) {
                set_termination(prj, at, other, None);
            }
        }
    }
    set_termination(prj, at, to, kind);
}

/// Ferrules and terminals needed for the routed wires of the panel: (part, count); a
/// twin ferrule is counted once for its two wires.
pub fn terminal_material(prj: &Project, ends: &Ends, wiring: &[crate::panel::RoutedWire]) -> Vec<(String, usize)> {
    let mut parts: Vec<(String, usize)> = Vec::new();
    // The twin ferrules counted: (connection point, far ends of both wires).
    let mut twins: HashSet<(String, String, String)> = HashSet::new();
    for w in wiring {
        for (at, to) in [(&w.from, &w.to), (&w.to, &w.from)] {
            let Some(t) = ends.end(prj, at, to, &w.cross_section) else { continue };
            let part = t.part();
            if part.is_empty() {
                continue;
            }
            if let Some(tw) = ends.twin(prj, at, to).filter(|_| t.kind == Termination::TwinFerrule) {
                let pair = if natural_cmp(to, &tw.with).is_le() { (to.clone(), tw.with) } else { (tw.with, to.clone()) };
                if !twins.insert((at.clone(), pair.0, pair.1)) {
                    continue;
                }
            }
            match parts.iter_mut().find(|x| x.0 == part) {
                Some(x) => x.1 += 1,
                None => parts.push((part, 1)),
            }
        }
    }
    parts
}

#[cfg(test)]
mod tests;
