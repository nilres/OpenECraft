//! Cable glands added for cables: a cable that leaves an enclosure (not a room) for a
//! device outside it and its line-up needs a way in. Where none of the enclosure's free
//! cable glands it can pass takes it and it has no cable entry flange, a gland of the
//! size of the cable is added ([`Layout::auto_glands`], on by default): for a device in
//! the room of a cabinet standing there into its roof (a cabinet hanging on the wall:
//! into its bottom), the way the routing leaves it into the room; for a device in the
//! field into its bottom. Where that face is full, into the next face the cable can
//! leave through (roof or bottom, then the sides and the rear, see [`faces_for`]); a
//! cable that finds no free place on any of them is reported ([`Glands::unplaced`]).
//!
//! [`Layout::auto_glands`]: crate::panel::Layout::auto_glands

use super::{accessories, cable_diameter, cable_end_tags};
use crate::geom::p;
use crate::model::Project;
use openecraft_i18n::tr;
use crate::netlist::{self, Netlist};
use crate::panel::{self, Face, SitePose, WallLink};
use std::collections::BTreeMap;

/// Side walls of enclosures shallower than this take no cable glands: the wires inside
/// would find no way to them past the mounting plate, mm.
const MIN_SIDE_DEPTH: f32 = 150.0;
/// Room a cable needs behind the rear of a cabinet to leave through it, mm.
const REAR_CLEARANCE: f32 = 60.0;

/// A cable that needs a way into an enclosure: its diameter, whether it leaves into the
/// room the enclosure stands in (only through its room sides, see
/// [`super::room::room_side`]) and whether its other end there is below the bottom of
/// the enclosure (the routing leaves a cabinet hanging on the wall through its bottom
/// only towards such a place).
#[derive(Clone, Debug, PartialEq)]
struct Need {
    cable: String,
    diameter: f32,
    room: bool,
    below: bool,
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
    let (poses, links) = (panel::site_poses(prj), panel::wall_links(prj));
    for c in &prj.cables {
        let cores: Vec<&netlist::Connection> = conns.iter().filter(|x| x.cable == c.tag).collect();
        let Some((a, b)) = cable_end_tags(prj, c, &cores) else { continue };
        let at = |t: &str| panel::locate(prj, t).map(|(ei, _)| ei);
        let (ea, eb) = (at(&a), at(&b));
        for (k, other, far) in [(ea, eb, &b), (eb, ea, &a)] {
            let Some(k) = k.filter(|&k| !layout.enclosures[k].is_room()) else { continue };
            let lineup = panel::lineup_of(layout, k);
            if other.is_some_and(|o| o == k || lineup.contains(&o)) {
                continue;
            }
            // A device placed elsewhere is reached through the room (where the cabinet
            // stands in one); one placed nowhere is in the field.
            let ri = panel::room_of(layout, k).filter(|_| other.is_some());
            let room = ri.is_some();
            let below = ri.is_some_and(|ri| far_height(prj, &poses, &links, ri, far).is_some_and(|h| h < poses[k].at[1] - poses[ri].at[1]));
            need.entry(k).or_default().push(Need { cable: c.tag.clone(), diameter: cable_diameter(c), room, below });
        }
    }
    for ds in need.values_mut() {
        ds.sort_by(|a, b| b.room.cmp(&a.room).then(b.diameter.total_cmp(&a.diameter)));
    }
    need
}

/// How high above the floor of its room device `tag` is placed (on the ceiling or on
/// the roof: the height of the room); `None` where it is in no room.
fn height_of(prj: &Project, tag: &str) -> Option<f32> {
    let (ei, it) = panel::locate(prj, tag)?;
    let e = &prj.layout.enclosures[ei];
    e.is_room().then(|| match it.face {
        Face::Roof | Face::RoofTop => e.height,
        Face::Bottom => 0.0,
        _ => e.height - (it.y + it.h / 2.0),
    })
}

/// How high above the floor of room `ri` the route of a cable to device `far` heads for
/// there, as the routing leaves a cabinet ([`super::room::room_entry`]): the device in
/// the room or in a cabinet standing there, else the first wall penetration on the way
/// to its room; `None` where it is in no room or no way leads there.
fn far_height(prj: &Project, poses: &[SitePose], links: &[WallLink], ri: usize, far: &str) -> Option<f32> {
    let layout = &prj.layout;
    let (fe, it) = panel::locate(prj, far)?;
    let fr = if layout.enclosures[fe].is_room() { fe } else { panel::room_of(layout, fe)? };
    let h = layout.enclosures[ri].height;
    if fr != ri {
        return super::room::room_chain(links, ri, fr)?.first().map(|l| h - l.a[1]);
    }
    if fe == ri {
        return height_of(prj, far);
    }
    let (lo, hi) = panel::item_box(&layout.enclosures[fe], it);
    let mid = [0, 1, 2].map(|i| (lo[i] + hi[i]) / 2.0);
    Some(h - super::room::cabinet_in_room(prj, poses, ri, fe, mid)[1])
}

/// The rear of cabinet `k` standing in a room is free: a cable can leave it, it is not
/// against a wall of the room.
fn rear_free(prj: &Project, k: usize) -> bool {
    let Some(ri) = panel::room_of(&prj.layout, k) else { return true };
    let e = &prj.layout.enclosures[k];
    let behind = panel::face_point(e, Face::Rear, [e.width / 2.0, e.height / 2.0, -REAR_CLEARANCE]);
    let q = super::room::cabinet_in_room(prj, &panel::site_poses(prj), ri, k, behind);
    panel::inside_outline(&panel::outline(&prj.layout.enclosures[ri]), [q[0], q[2]])
}

/// Cable `n` can leave enclosure `k` through `face`: a cable into the room only through
/// its room sides, through the bottom of a hanging cabinet only towards a place below it.
fn can_leave(prj: &Project, k: usize, n: &Need, face: Face) -> bool {
    !n.room || (super::room::room_side(prj, k, face) && (face != Face::Bottom || n.below))
}

/// The cables of `need` in enclosure `k` no free cable gland they can pass takes: each
/// takes the fitting free gland with the smallest clamping range, else (like the
/// routing) another free one; none where a cable entry flange takes them all.
fn without_gland(prj: &Project, k: usize, need: &[Need]) -> Vec<Need> {
    if !accessories(prj, k, "Cable entry").is_empty() {
        return Vec::new();
    }
    let mut free: Vec<(Face, Option<(f32, f32)>)> = accessories(prj, k, "Cable gland").iter().map(|g| (g.face, panel::clamping_range(prj, &g.part))).collect();
    let passes = |n: &Need, face: Face| can_leave(prj, k, n, face);
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
/// a cable into the room the roof (of a cabinet hanging on the wall towards a place below
/// it: the bottom, then the roof), otherwise the bottom, then the roof; then the sides
/// and the rear, where the cable can leave through them ([`can_leave`]). Not a side
/// against the next enclosure of the line-up, nor the sides of an enclosure shallower
/// than [`MIN_SIDE_DEPTH`], nor the rear of one hanging on the wall or standing against
/// it.
fn faces_for(prj: &Project, k: usize, n: &Need) -> Vec<Face> {
    let hangs = super::room::hangs(prj, k);
    let first = if n.room && !(hangs && n.below) { [Face::Roof, Face::Bottom] } else { [Face::Bottom, Face::Roof] };
    let lineup = panel::lineup_of(&prj.layout, k);
    let at = lineup.iter().position(|&i| i == k);
    let deep = prj.layout.enclosures[k].depth >= MIN_SIDE_DEPTH;
    let side = |f: Face| match f {
        Face::Left => deep && !at.is_some_and(|i| i > 0),
        Face::Right => deep && !at.is_some_and(|i| i + 1 < lineup.len()),
        Face::Rear => !hangs && rear_free(prj, k),
        _ => true,
    };
    first.into_iter().chain([Face::Left, Face::Right, Face::Rear]).filter(|&f| side(f) && can_leave(prj, k, n, f)).collect()
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
