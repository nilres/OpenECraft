//! Cable glands added for new cables: a cable that leaves an enclosure (not a room)
//! for a device outside it and its line-up needs a way in. Where none of the
//! enclosure's free cable glands takes its diameter and it has no cable entry flange,
//! a gland of the size of the cable is put into its bottom ([`Layout::auto_glands`],
//! on by default).
//!
//! [`Layout::auto_glands`]: crate::panel::Layout::auto_glands

use super::{accessories, cable_diameter, cable_end_tags};
use crate::geom::p;
use crate::model::Project;
use crate::netlist::{self, Netlist};
use crate::panel::{self, Face};
use std::collections::BTreeMap;

/// The diameters of the cables that need a way into each enclosure, thickest first.
fn needed(prj: &Project) -> BTreeMap<usize, Vec<f32>> {
    let mut need: BTreeMap<usize, Vec<f32>> = BTreeMap::new();
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
            need.entry(k).or_default().push(cable_diameter(c));
        }
    }
    for ds in need.values_mut() {
        ds.sort_by(|a, b| b.total_cmp(a));
    }
    need
}

/// The cables of `need` in enclosure `k` no free cable gland takes: each takes the
/// fitting free gland with the smallest clamping range, else another free one; none
/// where a cable entry flange takes them all.
fn without_gland(prj: &Project, k: usize, need: &[f32]) -> Vec<f32> {
    if !accessories(prj, k, "Cable entry").is_empty() {
        return Vec::new();
    }
    let mut free: Vec<Option<(f32, f32)>> = accessories(prj, k, "Cable gland").iter().map(|g| panel::clamping_range(prj, &g.part)).collect();
    let mut unfit = Vec::new();
    for &d in need {
        let fits = |r: &Option<(f32, f32)>| r.is_none_or(|(lo, hi)| d >= lo && d <= hi);
        let span = |r: &Option<(f32, f32)>| r.map_or(f32::INFINITY, |(lo, hi)| hi - lo);
        match free.iter().enumerate().filter(|(_, r)| fits(r)).min_by(|a, b| span(a.1).total_cmp(&span(b.1))).map(|(i, _)| i) {
            Some(i) => {
                free.remove(i);
            }
            None => unfit.push(d),
        }
    }
    // Like the routing, a cable no free gland fits takes another free one.
    unfit.split_off(free.len().min(unfit.len()))
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

/// Puts a cable gland `part` into the bottom of enclosure `k`, in the first free place
/// of rows across its depth from the middle, clear of what is on the bottom; `None`
/// where the bottom is full.
fn put_into_bottom(prj: &mut Project, k: usize, part: &str) -> Option<u64> {
    let size = panel::part_by_number(prj, part).and_then(|p| p.size()).map_or(30.0, |s| s[0].max(s[1]));
    let e = &prj.layout.enclosures[k];
    let (w, d) = e.face_size(Face::Bottom);
    let gap = 10.0;
    let taken: Vec<(f32, f32, f32, f32)> = e.items.iter().filter(|i| i.face == Face::Bottom).map(|i| (i.x, i.y, i.x + i.w, i.y + i.h)).collect();
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
    let it = panel::new_accessory(prj, part, Face::Bottom, p(x, y));
    let id = it.id;
    prj.layout.enclosures[k].items.push(it);
    Some(id)
}

/// Adds the cable glands the cables added or moved since `before` need (with
/// [`crate::panel::Layout::auto_glands`]): in each enclosure as many as it needs more
/// ways in than before and finds no free fitting gland for. Gives the ids of the glands.
pub fn follow(prj: &mut Project, before: &Project) -> Vec<u64> {
    // Called after every edit in the editor: without cables there is nothing to do.
    if !prj.layout.auto_glands || prj.cables.is_empty() || (prj.cables == before.cables && prj.layout.enclosures == before.layout.enclosures && prj.pages == before.pages) {
        return Vec::new();
    }
    let now = needed(prj);
    if now.is_empty() {
        return Vec::new();
    }
    let was = needed(before);
    let more: Vec<(usize, usize)> = now.iter().filter_map(|(k, ds)| ds.len().checked_sub(was.get(k).map_or(0, Vec::len)).filter(|n| *n > 0).map(|n| (*k, n))).collect();
    let mut added = Vec::new();
    for (k, n) in more {
        let missing = without_gland(prj, k, &now[&k]);
        added.extend(add(prj, k, &missing[..n.min(missing.len())]));
    }
    added
}

/// Adds a cable gland for every cable that needs a way into an enclosure and finds no
/// free fitting gland there (whether or not glands are added automatically). Gives the
/// ids of the glands.
pub fn add_missing(prj: &mut Project) -> Vec<u64> {
    let mut added = Vec::new();
    for (k, ds) in needed(prj) {
        let missing = without_gland(prj, k, &ds);
        added.extend(add(prj, k, &missing));
    }
    added
}

/// Glands of the catalog for the cables of `diameters` in enclosure `k`.
fn add(prj: &mut Project, k: usize, diameters: &[f32]) -> Vec<u64> {
    let mut ids = Vec::new();
    for &d in diameters {
        let Some(part) = gland_for(prj, d) else { continue };
        ids.extend(put_into_bottom(prj, k, &part));
    }
    ids
}

#[cfg(test)]
mod tests;
