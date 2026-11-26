//! Where an accessory part goes in an enclosure: the places shown while it is dragged
//! into the panel layout and taken when it is dropped. An accessory of the enclosure's
//! part with field widths and rows (part data `Fields: 1`, `Rows: 7`, a distribution
//! panel of a meter cabinet) goes into the free slots of the cabinet's field grid on
//! the mounting plate; a frame that joins two enclosures (part data `Opening`) into the
//! openings of the side walls made for it (part data `Side openings` of the enclosure);
//! another accessory of the enclosure anywhere on the faces it may be mounted on; an
//! accessory of a device's part onto the placed device. A panel with mounting rails (a
//! distribution panel) carries them, so that devices go onto them.

use super::{allowed_faces, new_accessory, part_by_number, Enclosure, Face, ItemKind, PanelItem};
use crate::geom::{p, R};
use crate::model::Project;
use crate::parts::vec3::Tri;
use crate::parts::{self, Part};

/// A place an accessory fits.
#[derive(Clone, Debug, PartialEq)]
pub enum Place {
    /// A free slot of the field grid on the mounting plate (plate coordinates).
    Slot(R),
    /// Anywhere on a face of the enclosure.
    Face(Face),
    /// A free opening in a side wall (face coordinates).
    Opening(Face, R),
    /// Onto a placed device whose part it fits (its item id and tag).
    Device(u64, String),
}

impl Place {
    /// The face the place is on.
    pub fn face(&self, e: &Enclosure) -> Option<Face> {
        match self {
            Place::Slot(_) => Some(Face::Plate),
            Place::Face(f) | Place::Opening(f, _) => Some(*f),
            Place::Device(id, _) => e.item(*id).map(|i| i.face),
        }
    }

    /// The area of the place on its face; a whole face is its mounting area.
    pub fn rect(&self, e: &Enclosure) -> Option<R> {
        match self {
            Place::Slot(r) | Place::Opening(_, r) => Some(*r),
            Place::Face(f) => Some(super::mounting_area(e, *f)),
            Place::Device(id, _) => e.item(*id).map(|i| i.rect()),
        }
    }
}

/// The field grid of an enclosure: its fields and rows (part data) and the size of one
/// cell on the mounting plate.
pub fn field_grid(prj: &Project, e: &Enclosure) -> Option<(u32, u32, f32, f32)> {
    let part = part_by_number(prj, &e.part)?;
    let (fields, rows) = (parts::count_value(part, "Fields")?, parts::count_value(part, "Rows")?);
    if fields == 0 || rows == 0 {
        return None;
    }
    let (w, h) = e.plate_size();
    Some((fields, rows, w / fields as f32, h / rows as f32))
}

/// True if `acc` is an accessory of the part of enclosure `ei` (whether a place is left
/// for it or not).
pub fn fits_enclosure(prj: &Project, ei: usize, acc: &Part) -> bool {
    prj.layout.enclosures.get(ei).and_then(|e| part_by_number(prj, &e.part)).is_some_and(|main| parts::accessory::fits(acc, main))
}

/// The places accessory `acc` fits in enclosure `ei`: free slots of the field grid,
/// faces, placed devices. Empty when it is no accessory of the enclosure or of a device
/// in it.
pub fn places(prj: &Project, ei: usize, acc: &Part) -> Vec<Place> {
    let Some(e) = prj.layout.enclosures.get(ei) else { return Vec::new() };
    let mut out = Vec::new();
    if let Some(main) = part_by_number(prj, &e.part).filter(|main| parts::accessory::fits(acc, main)) {
        match (field_grid(prj, e), parts::count_value(acc, "Fields"), parts::count_value(acc, "Rows")) {
            (Some((fields, rows, cw, ch)), Some(af), Some(ar)) if af >= 1 && ar >= 1 && af <= fields && ar <= rows => {
                // Slots taken by accessories already in the grid.
                let taken: Vec<R> = e.items.iter().filter(|i| i.face == Face::Plate && matches!(i.kind, ItemKind::Accessory | ItemKind::MountingPlate)).map(|i| i.rect().expand(-1.0)).collect();
                for row in 0..=rows - ar {
                    for field in 0..=fields - af {
                        let min = p(field as f32 * cw, row as f32 * ch);
                        let slot = R::from_points(min, p(min.x + af as f32 * cw, min.y + ar as f32 * ch));
                        if !taken.iter().any(|t| t.overlaps(&slot)) {
                            out.push(Place::Slot(slot));
                        }
                    }
                }
            }
            _ if parts::opening_size(acc).is_some() && !parts::side_openings(main).is_empty() => {
                out.extend(free_openings(e, main, acc).into_iter().map(|(f, r)| Place::Opening(f, r)));
            }
            _ => out.extend(allowed_faces(acc).into_iter().filter(|f| e.faces().contains(f)).map(Place::Face)),
        }
    }
    for it in e.items.iter().filter(|i| i.kind == ItemKind::Device && !i.tag.is_empty()) {
        let fits = device_parts(prj, &it.tag, &it.part).any(|main| parts::accessory::fits(acc, main));
        if fits && !out.iter().any(|pl| matches!(pl, Place::Device(_, t) if *t == it.tag)) {
            out.push(Place::Device(it.id, it.tag.clone()));
        }
    }
    out
}

/// Side opening `o` on side wall `face` in face coordinates (y from the top; x from the
/// back on the left wall, from the front on the right one).
pub fn opening_rect(e: &Enclosure, face: Face, o: &parts::SideOpening) -> R {
    let y = e.height - o.bottom - o.height;
    let x = if face == Face::Right { e.depth - o.back - o.depth } else { o.back };
    R::from_points(p(x, y), p(x + o.depth, y + o.height))
}

/// The openings of the side walls of enclosure `e` (part `main`) frame `acc` goes into:
/// those of its size (part data `Opening`) on the faces it may be mounted on; with
/// `free` only those no accessory sits in yet.
fn openings(e: &Enclosure, main: &Part, acc: &Part, free: bool) -> Vec<(Face, R)> {
    let Some([h, d]) = parts::opening_size(acc) else { return Vec::new() };
    let faces = allowed_faces(acc);
    let mut out = Vec::new();
    for o in parts::side_openings(main).iter().filter(|o| (o.height - h).abs() <= 1.0 && (o.depth - d).abs() <= 1.0) {
        for face in [Face::Left, Face::Right].into_iter().filter(|f| faces.contains(f)) {
            let r = opening_rect(e, face, o);
            let taken = e.items.iter().any(|i| i.face == face && i.kind == ItemKind::Accessory && i.rect().overlaps(&r));
            if !(free && taken) {
                out.push((face, r));
            }
        }
    }
    out
}

fn free_openings(e: &Enclosure, main: &Part, acc: &Part) -> Vec<(Face, R)> {
    openings(e, main, acc, true)
}

/// Puts frame item `it` into opening `r` of its face: centred on it, its larger sides
/// along the opening's, outside the wall (between the two enclosures it joins), so that
/// it keeps clear of what is mounted inside.
fn into(it: &mut PanelItem, r: R) {
    let mut dims = [it.w, it.h, it.d];
    dims.sort_by(f32::total_cmp);
    let [thin, mid, long] = dims;
    let (w, h) = if r.height() >= r.width() { (mid, long) } else { (long, mid) };
    let c = r.center();
    (it.x, it.y, it.w, it.h, it.d, it.z) = (c.x - w / 2.0, c.y - h / 2.0, w, h, thin, -thin);
}

/// A frame that joins two enclosures placed on a side wall of enclosure `ei` goes into
/// the free opening of that wall nearest to it. False when it is no such frame or no
/// opening is free.
pub fn into_opening(prj: &Project, ei: usize, it: &mut PanelItem) -> bool {
    let (Some(e), Some(acc)) = (prj.layout.enclosures.get(ei), part_by_number(prj, &it.part)) else { return false };
    let Some(main) = part_by_number(prj, &e.part) else { return false };
    let at = it.rect().center();
    let nearest = free_openings(e, main, acc).into_iter().filter(|(f, _)| *f == it.face).min_by(|a, b| a.1.center().dist(at).total_cmp(&b.1.center().dist(at)));
    let Some((_, r)) = nearest else { return false };
    into(it, r);
    true
}

/// True if accessory item `it` of enclosure `e` sits in an opening of a side wall made
/// for it (it may reach over the edge of the wall's mounting area there).
pub fn at_opening(prj: &Project, e: &Enclosure, it: &PanelItem) -> bool {
    opening_of(prj, e, it).is_some()
}

/// The opening of a side wall of enclosure `e` frame item `it` sits in (face
/// coordinates).
fn opening_of(prj: &Project, e: &Enclosure, it: &PanelItem) -> Option<R> {
    let (acc, main) = (part_by_number(prj, &it.part)?, part_by_number(prj, &e.part)?);
    let c = it.rect().center();
    openings(e, main, acc, false).into_iter().find(|(f, r)| *f == it.face && r.contains(c)).map(|(_, r)| r)
}

/// The side of enclosure `ei` the frame item `id` joins it to another enclosure at:
/// `Left` or `Right` when it sits in an opening of that side wall made for it.
pub fn joining_side(prj: &Project, ei: usize, id: u64) -> Option<Face> {
    let e = prj.layout.enclosures.get(ei)?;
    let it = e.item(id)?;
    (it.kind == ItemKind::Accessory && matches!(it.face, Face::Left | Face::Right) && at_opening(prj, e, it)).then_some(it.face)
}

/// Bays enclosure `other` to enclosure `ei` at the side the joining frame item `id`
/// sits in ([`joining_side`]): to its right for a frame in the right wall, to its left
/// for one in the left wall.
pub fn attach_at_frame(prj: &mut Project, ei: usize, id: u64, other: usize) -> Result<(), String> {
    let Some(side) = joining_side(prj, ei, id) else {
        return Err(openecraft_i18n::tr("The item is no frame in an opening of a side wall").to_string());
    };
    let done = if side == Face::Right { super::bay_right_of(prj, other, ei) } else { super::lineup::bay_left_of(prj, other, ei) };
    if done {
        Ok(())
    } else {
        Err(openecraft_i18n::tr("Only another enclosure can be bayed there").to_string())
    }
}

/// Bays a new enclosure like enclosure `ei` (its part, or its size), named after it, to
/// it at the side of its joining frame item `id` ([`attach_at_frame`]); its index.
pub fn attach_new_at_frame(prj: &mut Project, ei: usize, id: u64) -> Result<usize, String> {
    if joining_side(prj, ei, id).is_none() {
        return Err(openecraft_i18n::tr("The item is no frame in an opening of a side wall").to_string());
    }
    let e = &prj.layout.enclosures[ei];
    let (part, size) = (e.part.clone(), (e.width, e.height, e.depth));
    let prefix = e.name.trim_end_matches(|c: char| c.is_ascii_digit());
    let prefix = if prefix.is_empty() { "+CAB".to_string() } else { prefix.to_string() };
    let name = super::free_name(&prj.layout, &prefix);
    let mut new = super::new_enclosure(prj, &part, &name);
    if part_by_number(prj, &part).is_none() {
        (new.width, new.height, new.depth) = size;
    }
    prj.layout.enclosures.push(new);
    let at = prj.layout.enclosures.len() - 1;
    attach_at_frame(prj, ei, id, at)?;
    Ok(at)
}

/// Dark grey plastic of a joining frame, the copper of the busbars through it.
const FRAME_PLASTIC: [u8; 3] = [92, 96, 102];
const COPPER: [u8; 3] = [184, 115, 51];
/// Wall of a joining frame's collar and thickness of its flange.
const COLLAR: f32 = 4.0;
const FLANGE: f32 = 4.0;

/// The 3D model of a frame that joins two enclosures ([`at_opening`]) in face
/// coordinates: its flange on the outside of the side wall and its collar through the
/// opening into the enclosure beside, with the busbars of a busbar connector (part data
/// `Poles: 5`, `Busbars: Cu 2 x 12 x 5 mm`) through it. `None` for other items.
pub fn frame_mesh(prj: &Project, it: &PanelItem) -> Option<Vec<(Tri, Option<[u8; 3]>)>> {
    if it.kind != ItemKind::Accessory || !matches!(it.face, Face::Left | Face::Right) {
        return None;
    }
    let e = prj.layout.enclosures.iter().find(|e| e.item(it.id).is_some())?;
    let r = opening_of(prj, e, it)?;
    let part = part_by_number(prj, &it.part)?;
    // The opening in the item's coordinates, the collar 1 mm inside it.
    let (x0, y0, x1, y1) = (r.min.x - it.x + 1.0, r.min.y - it.y + 1.0, r.max.x - it.x - 1.0, r.max.y - it.y - 1.0);
    let (z0, z1) = (it.z, it.z + it.d);
    let mut m = crate::parts::Mesh3d::default();
    // The collar, open through.
    let t = COLLAR;
    m.add_painted_box([x0, y0, z0], [x1, y0 + t, z1], FRAME_PLASTIC);
    m.add_painted_box([x0, y1 - t, z0], [x1, y1, z1], FRAME_PLASTIC);
    m.add_painted_box([x0, y0 + t, z0], [x0 + t, y1 - t, z1], FRAME_PLASTIC);
    m.add_painted_box([x1 - t, y0 + t, z0], [x1, y1 - t, z1], FRAME_PLASTIC);
    // The flange against the outside of the wall, round the collar.
    let f1 = (-e.wall().max(0.0)).clamp(z0 + FLANGE, z1);
    let (w, h) = (it.w, it.h);
    if x0 > 0.0 && y0 > 0.0 && x1 < w && y1 < h {
        m.add_painted_box([0.0, 0.0, f1 - FLANGE], [w, y0, f1], FRAME_PLASTIC);
        m.add_painted_box([0.0, y1, f1 - FLANGE], [w, h, f1], FRAME_PLASTIC);
        m.add_painted_box([0.0, y0, f1 - FLANGE], [x0, y1, f1], FRAME_PLASTIC);
        m.add_painted_box([x1, y0, f1 - FLANGE], [w, y1, f1], FRAME_PLASTIC);
    }
    // The busbars, one pole after the other along the long side of the opening.
    let poles = parts::count_value(part, "Poles").unwrap_or(0).min(12) as usize;
    if let Some((n, bw, bt)) = part.tech_value("Busbars").and_then(busbar_size).filter(|_| poles > 0) {
        let long_y = y1 - y0 >= x1 - x0;
        let (along, across) = if long_y { (y1 - y0 - 2.0 * t, x1 - x0 - 2.0 * t) } else { (x1 - x0 - 2.0 * t, y1 - y0 - 2.0 * t) };
        let pitch = along / poles as f32;
        // The bars of a pole with a gap of their thickness between them.
        let pack = (2 * n - 1) as f32 * bt;
        if pack < pitch && bw < across {
            for k in 0..poles {
                for j in 0..n {
                    let a = t + pitch * (k as f32 + 0.5) - pack / 2.0 + j as f32 * 2.0 * bt;
                    let b = t + (across - bw) / 2.0;
                    let (lo, hi) = if long_y { ([x0 + b, y0 + a], [x0 + b + bw, y0 + a + bt]) } else { ([x0 + a, y0 + b], [x0 + a + bt, y0 + b + bw]) };
                    m.add_painted_box([lo[0], lo[1], z0], [hi[0], hi[1], z1], COPPER);
                }
            }
        }
    }
    Some(laid(&m, it))
}

/// Busbars of part data `Cu 2 x 12 x 5 mm`: how many per pole, width and thickness.
fn busbar_size(v: &str) -> Option<(usize, f32, f32)> {
    let nums: Vec<f32> = v.split(|c: char| !(c.is_ascii_digit() || c == '.')).filter_map(|t| t.parse().ok()).collect();
    match nums[..] {
        [w, t] if w > 0.0 && t > 0.0 => Some((1, w, t)),
        [n, w, t] if (1.0..=4.0).contains(&n) && w > 0.0 && t > 0.0 => Some((n as usize, w, t)),
        _ => None,
    }
}

/// The panels of the field grid ([`is_insert`]) of enclosure `ei`, of every enclosure
/// for `None`: how many have their plastic cover fitted and how many not.
pub fn covers(prj: &Project, ei: Option<usize>) -> (usize, usize) {
    let panels = prj.layout.enclosures.iter().enumerate().filter(|(k, _)| ei.is_none_or(|i| i == *k)).flat_map(|(_, e)| e.items.iter().filter(|it| is_insert(prj, it)));
    panels.fold((0, 0), |(on, off), it| if it.uncovered { (on, off + 1) } else { (on + 1, off) })
}

/// Fits the plastic covers of the panels of the field grid of enclosure `ei` (of every
/// enclosure for `None`), or takes them all off; how many panels changed.
pub fn set_covers(prj: &mut Project, ei: Option<usize>, fitted: bool) -> usize {
    let panels: Vec<(usize, u64)> = prj
        .layout
        .enclosures
        .iter()
        .enumerate()
        .filter(|(k, _)| ei.is_none_or(|i| i == *k))
        .flat_map(|(k, e)| e.items.iter().filter(|it| is_insert(prj, it) && it.uncovered == fitted).map(move |it| (k, it.id)))
        .collect();
    for &(k, id) in &panels {
        if let Some(it) = prj.layout.enclosures[k].items.iter_mut().find(|i| i.id == id) {
            it.uncovered = !fitted;
        }
    }
    panels.len()
}

/// The parts of a device: those assigned to it, else the part of its item.
fn device_parts<'a>(prj: &'a Project, tag: &str, item_part: &'a str) -> impl Iterator<Item = &'a Part> {
    let assigned: Vec<&str> = prj.devices.get(tag).map(|d| d.parts.iter().map(|r| r.number.as_str()).collect()).unwrap_or_default();
    let numbers = if assigned.is_empty() { vec![item_part] } else { assigned };
    numbers.into_iter().filter_map(move |n| part_by_number(prj, n))
}

/// The place under point `at` on `face` (face coordinates): the smallest one there, so
/// that a device on a face wins over the face.
pub fn place_at<'a>(e: &Enclosure, places: &'a [Place], face: Face, at: crate::geom::P) -> Option<&'a Place> {
    places
        .iter()
        .filter(|pl| pl.face(e) == Some(face))
        .filter_map(|pl| pl.rect(e).filter(|r| r.contains(at)).map(|r| (pl, (r.max.x - r.min.x) * (r.max.y - r.min.y))))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(pl, _)| pl)
}

/// Puts accessory `number` (in the project's parts) at a place of enclosure `ei`: into
/// a slot or onto a face at `at` (its middle, face coordinates) as an accessory item,
/// onto a device as an accessory of its part. The item's id (0 for a device), or why
/// not.
pub fn put(prj: &mut Project, db: &crate::parts::PartsDb, ei: usize, number: &str, place: &Place, at: crate::geom::P) -> Result<u64, String> {
    if ei >= prj.layout.enclosures.len() {
        return Err(openecraft_i18n::tr_fmt!("No enclosure {}", ei));
    }
    match place {
        Place::Device(_, tag) => parts::add_accessory(prj, db, tag, number, 1).map(|_| 0),
        Place::Slot(r) => {
            let mut item = new_accessory(prj, number, Face::Plate, r.min);
            // Exactly in its slot, whatever the snap grid and the size of the part.
            (item.x, item.y, item.w, item.h) = (r.min.x, r.min.y, r.width(), r.height());
            Ok(add_panel(prj, ei, item))
        }
        Place::Opening(f, r) => {
            let mut item = new_accessory(prj, number, *f, r.min);
            into(&mut item, *r);
            Ok(add_panel(prj, ei, item))
        }
        Place::Face(f) => {
            let probe = new_accessory(prj, number, *f, p(0.0, 0.0));
            let id = super::place_accessory(prj, ei, number, *f, p(at.x - probe.w / 2.0, at.y - probe.h / 2.0)).unwrap_or(0);
            Ok(id)
        }
    }
}

/// Modular devices (DIN 43880) stand this far out of their rail: a panel holds its
/// rails so deep that the fronts of the devices on them are at its front.
pub const MODULAR_DEPTH: f32 = 70.0;
/// Thickness of the frame that carries the rails of a panel.
const PANEL_SHEET: f32 = 2.0;
/// Module spaces of a row of one field width.
const MODULES_PER_FIELD: u32 = 12;
/// Space left and right of the rails of a panel.
const RAIL_MARGIN: f32 = 10.0;

/// The mounting rails a panel accessory of height `h` carries: as many as its part data
/// `Device rows` says, for a distribution panel without it one per row of its module
/// spaces (12 per field width), else none. Their centre lines (y from the top of the panel) in its
/// rows (`Rows`, `Row spacing`), in the lowest rows when it has fewer rails than rows
/// (a connection point zone above them).
pub fn panel_rails(part: &Part, h: f32) -> Vec<f32> {
    let rows = parts::count_value(part, "Rows");
    let distribution = || {
        let fields = parts::count_value(part, "Fields").unwrap_or(1).max(1);
        parts::count_value(part, "Module spaces").map(|m| m / (MODULES_PER_FIELD * fields)).or(rows)
    };
    let rails = parts::count_value(part, "Device rows").or_else(|| (part.subgroup == "Distribution panel").then(distribution).flatten()).unwrap_or(0);
    if rails == 0 || h <= 0.0 {
        return Vec::new();
    }
    let rows = rows.unwrap_or(rails).max(rails);
    let pitch = super::tech_number(part, "Row spacing").filter(|s| *s > 0.0 && *s * rows as f32 <= h + 0.5).unwrap_or(h / rows as f32);
    let top = (h - pitch * rows as f32) / 2.0;
    (rows - rails..rows).map(|i| top + pitch * (i as f32 + 0.5)).collect()
}

/// Zinc-plated sheet steel of the frame of a panel and its brackets, the white plastic
/// of its cover.
const FRAME_STEEL: [u8; 3] = [176, 181, 188];
const COVER_PLASTIC: [u8; 3] = [236, 236, 231];
/// Width and depth of the side profiles of a panel's frame; thickness of its sheet
/// steel and of its cover.
const PROFILE: f32 = 20.0;
const SHEET: f32 = 1.5;
const COVER: f32 = 2.5;
/// Height of the slot in the cover for a row of modular devices (fronts of 45 mm).
const SLOT: f32 = 46.0;
/// How far the edges of the cover are folded back.
const COVER_EDGE: f32 = 25.0;

/// True if item `it` is a panel of a cabinet's field grid (part data `Fields` and
/// `Rows`) on the mounting plate.
pub fn is_insert(prj: &Project, it: &PanelItem) -> bool {
    it.face == Face::Plate
        && matches!(it.kind, ItemKind::Accessory | ItemKind::MountingPlate)
        && part_by_number(prj, &it.part).is_some_and(|p| parts::count_value(p, "Fields").is_some() && parts::count_value(p, "Rows").is_some())
}

/// The 3D model of a panel of a cabinet's field grid ([`is_insert`]) in plate
/// coordinates: its frame of two side profiles and a cross bar at the top and the
/// bottom (a distribution panel's rails are screwed onto it), the brackets with their
/// screws that hold it on the rear wall of the cabinet behind it, and its plastic cover
/// in front with a slot for each row of devices, left out when the cover is taken off
/// ([`PanelItem::uncovered`]). `None` for other items.
pub fn insert_mesh(prj: &Project, it: &PanelItem) -> Option<Vec<(Tri, Option<[u8; 3]>)>> {
    if !is_insert(prj, it) {
        return None;
    }
    let part = part_by_number(prj, &it.part)?;
    let (w, h) = (it.w, it.h);
    // The front of the frame and of the cover: a panel with rails holds them so deep
    // that modular devices on them end at its front ([`add_panel`]).
    let (rails, frame, front) = if it.kind == ItemKind::MountingPlate {
        let (_, pd) = super::part_profile(ItemKind::Rail, part_by_number(prj, rail_part(part)));
        (panel_rails(part, h), it.z + it.d, it.z + it.d + pd + MODULAR_DEPTH)
    } else {
        let front = it.z + it.d;
        (Vec::new(), (front - MODULAR_DEPTH - 7.5).max(it.z + PROFILE), front)
    };
    let mut m = crate::parts::Mesh3d::default();
    let pw = PROFILE.min(w / 4.0);
    // The side profiles, open to the back.
    for x in [0.0, w - pw] {
        m.add_painted_box([x, 0.0, frame - SHEET], [x + pw, h, frame], FRAME_STEEL);
        m.add_painted_box([x, 0.0, frame - PROFILE], [x + SHEET, h, frame - SHEET], FRAME_STEEL);
        m.add_painted_box([x + pw - SHEET, 0.0, frame - PROFILE], [x + pw, h, frame - SHEET], FRAME_STEEL);
    }
    let bar = 15.0f32.min(h / 8.0);
    for y in [0.0, h - bar] {
        m.add_painted_box([pw, y, frame - SHEET], [w - pw, y + bar, frame], FRAME_STEEL);
    }
    // The brackets at the ends of the profiles: back to the rear wall (behind the plate,
    // which the cabinet of a field grid need not have), their feet screwed on there.
    let back = frame - PROFILE;
    let wall = prj.layout.enclosures.iter().find(|e| e.item(it.id).is_some()).map_or(0.0, |e| (e.wall() - e.mounting_z()).min(0.0));
    if back > wall + SHEET {
        let foot = 25.0f32.min(h / 6.0);
        for x in [0.0, w - pw] {
            let (x0, x1) = (x + 2.0f32.min(pw / 4.0), x + pw - 2.0f32.min(pw / 4.0));
            for (y0, y1, fy0, fy1) in [(0.0, SHEET, 0.0, foot), (h - SHEET, h, h - foot, h)] {
                m.add_painted_box([x0, y0, wall + SHEET], [x1, y1, back], FRAME_STEEL);
                m.add_painted_box([x0, fy0, wall], [x1, fy1, wall + SHEET], FRAME_STEEL);
                let r = ((x1 - x0) / 2.0 - 1.0).clamp(1.0, 4.0);
                m.add_cylinder([(x0 + x1) / 2.0, (fy0 + fy1) / 2.0, wall + SHEET], 2, r, 0.6 * r, 12, super::STEEL_ZINC);
            }
        }
    }
    if !it.uncovered {
        let (z0, z1) = (front - COVER, front);
        // A slot for each row of devices, between the rails' ends.
        let (sx0, sx1) = (RAIL_MARGIN.min(w / 4.0) + 5.0, w - RAIL_MARGIN.min(w / 4.0) - 5.0);
        let mut y = 0.0;
        for line in rails.iter().filter(|_| sx1 > sx0) {
            let (a, b) = ((line - SLOT / 2.0).max(y), (line + SLOT / 2.0).min(h));
            if b <= a {
                continue;
            }
            if a > y {
                m.add_painted_box([0.0, y, z0], [w, a, z1], COVER_PLASTIC);
            }
            m.add_painted_box([0.0, a, z0], [sx0, b, z1], COVER_PLASTIC);
            m.add_painted_box([sx1, a, z0], [w, b, z1], COVER_PLASTIC);
            y = b;
        }
        if y < h {
            m.add_painted_box([0.0, y, z0], [w, h, z1], COVER_PLASTIC);
        }
        // Its edges folded back.
        let edge = COVER_EDGE.min(z0 - frame).max(0.0);
        if edge > 0.0 {
            for (lo, hi) in [([0.0, 0.0], [COVER, h]), ([w - COVER, 0.0], [w, h]), ([COVER, 0.0], [w - COVER, COVER]), ([COVER, h - COVER], [w - COVER, h])] {
                m.add_painted_box([lo[0], lo[1], z0 - edge], [hi[0], hi[1], z0], COVER_PLASTIC);
            }
        }
    }
    Some(laid(&m, it))
}

/// The triangles of model `m` (in the coordinates of item `it`, from its top left
/// corner) laid on its face: right-handed only with its y down turned round (as
/// `plate_box`).
fn laid(m: &crate::parts::Mesh3d, it: &PanelItem) -> Vec<(Tri, Option<[u8; 3]>)> {
    let at = |v: [f32; 3]| [it.x + v[0], it.y + v[1], v[2]];
    m.triangles
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let [p, q, s] = t.map(|k| at(m.vertices[k as usize]));
            ([p, s, q], m.color(i))
        })
        .collect()
}

/// The rail part of a panel: 35 × 15 mm when its description says so, else 35 × 7.5.
fn rail_part(part: &Part) -> &'static str {
    if part.description.contains("35 x 15") {
        "GEN.RAIL-TS35X15-1M"
    } else {
        super::RAIL_PART
    }
}

/// Adds accessory item `it` to enclosure `ei`. A panel on the mounting plate with
/// mounting rails ([`panel_rails`]) becomes a mounting plate that carries them: the
/// rails across the panel less a margin on each side, mounted on it, not billed (they
/// come with the panel) and so deep that modular devices on them end at the front of
/// the panel. The item's id.
pub fn add_panel(prj: &mut Project, ei: usize, mut it: PanelItem) -> u64 {
    let id = it.id;
    let part = part_by_number(prj, &it.part).cloned();
    let lines = part.as_ref().filter(|_| it.face == Face::Plate && it.kind == ItemKind::Accessory).map(|p| panel_rails(p, it.h)).unwrap_or_default();
    let mut rails = Vec::new();
    if let (Some(part), false) = (&part, lines.is_empty()) {
        let number = rail_part(part);
        let (pw, pd) = super::part_profile(ItemKind::Rail, part_by_number(prj, number));
        it.kind = ItemKind::MountingPlate;
        it.d = PANEL_SHEET;
        it.z = (part.depth - MODULAR_DEPTH - pd - PANEL_SHEET).max(0.0);
        let margin = RAIL_MARGIN.min(it.w / 4.0);
        for y in lines {
            rails.push(PanelItem {
                id: prj.new_id(),
                kind: ItemKind::Rail,
                part: number.into(),
                face: it.face,
                x: it.x + margin,
                y: it.y + y - pw / 2.0,
                w: it.w - 2.0 * margin,
                h: pw,
                z: it.z + it.d,
                d: pd,
                mounted_on: id,
                mounting_explicit: true,
                bom_excluded: true,
                ..Default::default()
            });
        }
    }
    let e = &mut prj.layout.enclosures[ei];
    e.items.push(it);
    e.items.extend(rails);
    id
}

#[cfg(test)]
mod tests;
