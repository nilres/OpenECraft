//! Cable glands added for cables: a cable that leaves an enclosure (not a room) for a
//! device outside it and its line-up needs a way in. Where none of the enclosure's free
//! cable glands it can pass takes it and it has no cable entry flange, a gland of the
//! size of the cable is added ([`Layout::auto_glands`], on by default): for a device in
//! the room of a cabinet standing there into its roof (a cabinet hanging on the wall:
//! into its bottom), the way the routing leaves it into the room; for a device in the
//! field into its bottom. Where that face is full, into the next face the cable can
//! leave through (roof or bottom, then the sides and the rear); a cable that finds no
//! free place on any of them is reported ([`Glands::unplaced`]).
//!
//! [`Layout::auto_glands`]: crate::panel::Layout::auto_glands

use super::{accessories, cable_diameter, cable_end_tags};
use crate::geom::p;
use crate::model::Project;
use openecraft_i18n::tr;
use crate::netlist::{self, Netlist};
use crate::panel::{self, Face};
use std::collections::BTreeMap;

/// A cable that needs a way into an enclosure: its diameter, and whether it leaves into
/// the room the enclosure stands in (only through its room sides, see
/// [`super::room::room_side`]).
#[derive(Clone, Debug, PartialEq)]
struct Need {
    cable: String,
    diameter: f32,
    room: bool,
}

/// The cable glands added for the cables of a project, and the cables that got none.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Glands {
    /// The ids of the glands added.
    pub added: Vec<u64>,
    pub unplaced: Vec<Unplaced>,
}

/// A cable that needs a way into an enclosure and found no free place for a cable gland
/// on the faces it can leave through.
#[derive(Clone, Debug, PartialEq)]
pub struct Unplaced {
    pub cable: String,
    pub enclosure: String,
}

impl Unplaced {
    /// Why it got none and what to do.
    pub fn reason() -> String {
        tr("No free place for a cable gland on the faces the cable can leave through: make room or add a cable entry flange").to_string()
    }
}

/// The cables that need a way into each enclosure, those into the room first, thickest
/// first.
fn needed(prj: &Project) -> BTreeMap<usize, Vec<Need>> {
    let mut need: BTreeMap<usize, Vec<Need>> = BTreeMap::new();
    if prj.cables.is_empty() || !prj.layout.enclosures.iter().any(|e| !e.is_room()) {
        return need;
    }
    let (nl, open) = (Netlist::build(prj, true), Netlist::build(prj, false));
    let conns = netlist::connections(prj, &nl, &open);
    let layout = &prj.layout;
    for c in &prj.cables {
        let cores: Vec<&netlist::Connection> = conns.iter().filter(|x| x.cable == c.tag).collect();
        let Some((a, b)) = cable_end_tags(prj, c, &cores) else { continue };
        let at = |t: &str| panel::locate(prj, t).map(|(ei, _)| ei);
        let (ea, eb) = (at(&a), at(&b));
        for (k, other) in [(ea, eb), (eb, ea)] {
            let Some(k) = k.filter(|&k| !layout.enclosures[k].is_room()) else { continue };
            let lineup = panel::lineup_of(layout, k);
            if other.is_some_and(|o| o == k || lineup.contains(&o)) {
                continue;
            }
            // A device placed elsewhere is reached through the room (where the cabinet
            // stands in one); one placed nowhere is in the field.
            let room = other.is_some() && panel::room_of(layout, k).is_some();
            need.entry(k).or_default().push(Need { cable: c.tag.clone(), diameter: cable_diameter(c), room });
        }
    }
    for ds in need.values_mut() {
        ds.sort_by(|a, b| b.room.cmp(&a.room).then(b.diameter.total_cmp(&a.diameter)));
    }
    need
}

/// The cables of `need` in enclosure `k` no free cable gland they can pass takes: each
/// takes the fitting free gland with the smallest clamping range, else (like the
/// routing) another free one; none where a cable entry flange takes them all.
fn without_gland(prj: &Project, k: usize, need: &[Need]) -> Vec<Need> {
    if !accessories(prj, k, "Cable entry").is_empty() {
        return Vec::new();
    }
    let mut free: Vec<(Face, Option<(f32, f32)>)> = accessories(prj, k, "Cable gland").iter().map(|g| (g.face, panel::clamping_range(prj, &g.part))).collect();
    let passes = |n: &Need, face: Face| !n.room || super::room::room_side(prj, k, face);
    let mut unfit = Vec::new();
    for n in need {
        let fits = |r: &Option<(f32, f32)>| r.is_none_or(|(lo, hi)| n.diameter >= lo && n.diameter <= hi);
        let span = |r: &Option<(f32, f32)>| r.map_or(f32::INFINITY, |(lo, hi)| hi - lo);
        match free.iter().enumerate().filter(|(_, g)| passes(n, g.0) && fits(&g.1)).min_by(|a, b| span(&a.1 .1).total_cmp(&span(&b.1 .1))).map(|(i, _)| i) {
            Some(i) => {
                free.remove(i);
            }
            None => unfit.push(n.clone()),
        }
    }
    let mut left = Vec::new();
    for n in unfit {
        match free.iter().position(|g| passes(&n, g.0)) {
            Some(i) => {
                free.remove(i);
            }
            None => left.push(n),
        }
    }
    left
}

/// The cable gland of the catalog for a cable of `diameter`: of polyamide, the smallest
/// whose clamping range takes it (the largest for thicker cables).
pub fn gland_for(prj: &Project, diameter: f32) -> Option<String> {
    let glands: Vec<(&str, (f32, f32))> = crate::parts::catalog()
        .iter()
        .filter(|p| p.subgroup == "Cable gland" && p.number.starts_with("GEN.GLAND-M") && !p.number.ends_with("-MS"))
        .filter_map(|p| panel::clamping_range(prj, &p.number).map(|r| (p.number.as_str(), r)))
        .collect();
    let fitting = glands.iter().filter(|(_, (lo, hi))| diameter >= *lo && diameter <= *hi).min_by(|a, b| (a.1 .1 - a.1 .0).total_cmp(&(b.1 .1 - b.1 .0)));
    fitting.or_else(|| glands.iter().max_by(|a, b| a.1 .1.total_cmp(&b.1 .1))).map(|(n, _)| n.to_string())
}

/// Puts a cable gland `part` into `face` of enclosure `k`, in the first free place of
/// rows across its depth from the middle, clear of what is on that face; `None` where it
/// is full.
fn put_into(prj: &mut Project, k: usize, part: &str, face: Face) -> Option<u64> {
    let size = panel::part_by_number(prj, part).and_then(|p| p.size()).map_or(30.0, |s| s[0].max(s[1]));
    let e = &prj.layout.enclosures[k];
    let (w, d) = e.face_size(face);
    let gap = 10.0;
    let taken: Vec<(f32, f32, f32, f32)> = e.items.iter().filter(|i| i.face == face).map(|i| (i.x, i.y, i.x + i.w, i.y + i.h)).collect();
    let clear = |x: f32, y: f32| taken.iter().all(|t| x + size + gap <= t.0 || x >= t.2 + gap || y + size + gap <= t.1 || y >= t.3 + gap);
    let margin = 2.0 * e.wall() + gap;
    let middle = ((d - size) / 2.0).max(0.0);
    let rows = (0..).map(|k: i32| middle + (if k % 2 == 0 { 1.0 } else { -1.0 }) * ((k + 1) / 2) as f32 * (size + gap)).take_while(|y| *y >= 0.0 && *y + size <= d.max(size));
    let mut spot = None;
    'rows: for y in rows {
        let mut x = margin;
        while x + size + margin <= w {
            if clear(x, y) {
                spot = Some((x, y));
                break 'rows;
            }
            x += 5.0;
        }
    }
    let (x, y) = spot?;
    let it = panel::new_accessory(prj, part, face, p(x, y));
    let id = it.id;
    prj.layout.enclosures[k].items.push(it);
    Some(id)
}

/// After a change of the cables or of the schematic since `before` (with
/// [`crate::panel::Layout::auto_glands`]): a cable gland for every cable that needs a
/// way into an enclosure and finds none ([`add_missing`]). A change of the layout alone
/// adds none (a gland deleted by hand stays deleted). Gives the glands added and the
/// cables that found no place for one.
pub fn follow(prj: &mut Project, before: &Project) -> Glands {
    // Called after every edit in the editor: without cables there is nothing to do.
    if !prj.layout.auto_glands || prj.cables.is_empty() || (prj.cables == before.cables && prj.pages == before.pages) {
        return Glands::default();
    }
    add_missing(prj)
}

/// Adds a cable gland for every cable that needs a way into an enclosure and finds no
/// free fitting gland there (whether or not glands are added automatically). Gives the
/// glands added and the cables that found no place for one.
pub fn add_missing(prj: &mut Project) -> Glands {
    let mut out = Glands::default();
    for (k, ds) in needed(prj) {
        let missing = without_gland(prj, k, &ds);
        add(prj, k, &missing, &mut out);
    }
    out
}

/// The faces of enclosure `k` a cable gland for cable `n` goes into, in this order: for
/// a cable into the room of a cabinet standing there the roof (hanging on the wall: the
/// bottom, then the roof), otherwise the bottom, then the roof; then the sides and the
/// rear, where the cable can leave through them.
fn faces_for(prj: &Project, k: usize, n: &Need) -> Vec<Face> {
    let first = if n.room && !super::room::hangs(prj, k) { [Face::Roof, Face::Bottom] } else { [Face::Bottom, Face::Roof] };
    first.into_iter().chain([Face::Left, Face::Right, Face::Rear]).filter(|&f| !n.room || super::room::room_side(prj, k, f)).collect()
}

/// Glands of the catalog for the cables of `need` in enclosure `k`, each into the first
/// of its faces ([`faces_for`]) with a free place.
fn add(prj: &mut Project, k: usize, need: &[Need], out: &mut Glands) {
    for n in need {
        let placed = gland_for(prj, n.diameter).and_then(|part| faces_for(prj, k, n).into_iter().find_map(|f| put_into(prj, k, &part, f)));
        match placed {
            Some(id) => out.added.push(id),
            None => out.unplaced.push(Unplaced { cable: n.cable.clone(), enclosure: prj.layout.enclosures[k].name.clone() }),
        }
    }
}

#[cfg(test)]
mod tests;
