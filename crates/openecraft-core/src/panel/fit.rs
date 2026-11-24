//! Where an accessory part goes in an enclosure: the places shown while it is dragged
//! into the panel layout and taken when it is dropped. An accessory of the enclosure's
//! part with field widths and rows (part data `Fields: 1`, `Rows: 7`, a distribution
//! panel of a meter cabinet) goes into the free slots of the cabinet's field grid on
//! the mounting plate; another accessory of the enclosure anywhere on the faces it may
//! be mounted on; an accessory of a device's part onto the placed device.

use super::{allowed_faces, new_accessory, part_by_number, Enclosure, Face, ItemKind};
use crate::geom::{p, R};
use crate::model::Project;
use crate::parts::{self, Part};

/// A place an accessory fits.
#[derive(Clone, Debug, PartialEq)]
pub enum Place {
    /// A free slot of the field grid on the mounting plate (plate coordinates).
    Slot(R),
    /// Anywhere on a face of the enclosure.
    Face(Face),
    /// Onto a placed device whose part it fits (its item id and tag).
    Device(u64, String),
}

impl Place {
    /// The face the place is on.
    pub fn face(&self, e: &Enclosure) -> Option<Face> {
        match self {
            Place::Slot(_) => Some(Face::Plate),
            Place::Face(f) => Some(*f),
            Place::Device(id, _) => e.item(*id).map(|i| i.face),
        }
    }

    /// The area of the place on its face; a whole face is its mounting area.
    pub fn rect(&self, e: &Enclosure) -> Option<R> {
        match self {
            Place::Slot(r) => Some(*r),
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
    if part_by_number(prj, &e.part).is_some_and(|main| parts::accessory::fits(acc, main)) {
        match (field_grid(prj, e), parts::count_value(acc, "Fields"), parts::count_value(acc, "Rows")) {
            (Some((fields, rows, cw, ch)), Some(af), Some(ar)) if af >= 1 && ar >= 1 && af <= fields && ar <= rows => {
                // Slots taken by accessories already in the grid.
                let taken: Vec<R> = e.items.iter().filter(|i| i.face == Face::Plate && i.kind == ItemKind::Accessory).map(|i| i.rect().expand(-1.0)).collect();
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
            let id = item.id;
            prj.layout.enclosures[ei].items.push(item);
            Ok(id)
        }
        Place::Face(f) => {
            let probe = new_accessory(prj, number, *f, p(0.0, 0.0));
            let id = super::place_accessory(prj, ei, number, *f, p(at.x - probe.w / 2.0, at.y - probe.h / 2.0)).unwrap_or(0);
            Ok(id)
        }
    }
}

#[cfg(test)]
mod tests;
