//! The order of the navigator's tree of buildings, rooms and enclosures, changed by
//! dragging a row onto another: it goes before or after that row. A room moves among
//! the rooms (onto the level of the room it is dropped on); an enclosure moves within
//! its line-up, bayed again in the new order, or with its whole line-up before or after
//! another line-up, into the room that one stands in (or the room it is dropped on).

use super::{buildings, cabinets_in, lineup_of, room_of, set_room, Enclosure};
use crate::model::Project;
use openecraft_i18n::{tr, tr_fmt};

/// Gives every enclosure an id, so that it can be found again after a move.
fn ensure_ids(prj: &mut Project) {
    for i in 0..prj.layout.enclosures.len() {
        if prj.layout.enclosures[i].id == 0 {
            prj.layout.enclosures[i].id = prj.new_id().max(1);
        }
    }
}

/// Moves the enclosures of `block` (in its order) right before index `at` of the
/// enclosures as they stand (`at` past the end: to the end).
fn move_block(prj: &mut Project, block: &[usize], at: usize) {
    let mut moved: Vec<Option<Enclosure>> = vec![None; block.len()];
    let mut rest = Vec::new();
    for (i, e) in std::mem::take(&mut prj.layout.enclosures).into_iter().enumerate() {
        match block.iter().position(|&b| b == i) {
            Some(k) => moved[k] = Some(e),
            None => rest.push((i, e)),
        }
    }
    let pos = rest.iter().position(|(i, _)| *i >= at).unwrap_or(rest.len());
    let mut out: Vec<Enclosure> = rest.into_iter().map(|(_, e)| e).collect();
    out.splice(pos..pos, moved.into_iter().flatten());
    prj.layout.enclosures = out;
}

/// Bays the enclosures of line-up `old` (from left to right) again in `order`: the room
/// and the place of the line-up stay with its left end.
fn rebay(prj: &mut Project, old: &[usize], order: &[usize]) {
    let first = &mut prj.layout.enclosures[old[0]];
    let (room, site) = (std::mem::take(&mut first.room), first.site.take());
    let ids: Vec<u64> = order.iter().map(|&i| prj.layout.enclosures[i].id).collect();
    for (k, &i) in order.iter().enumerate() {
        prj.layout.enclosures[i].bayed_to = if k == 0 { 0 } else { ids[k - 1] };
    }
    let first = &mut prj.layout.enclosures[order[0]];
    (first.room, first.site) = (room, site);
}

/// Moves row `from` of the navigator's tree (a room or an enclosure) before row `to`,
/// or after it; the index `from` has afterwards.
pub fn move_in_tree(prj: &mut Project, from: usize, to: usize, after: bool) -> Result<usize, String> {
    let n = prj.layout.enclosures.len();
    if from >= n || to >= n {
        return Err(tr_fmt!("No enclosure {}", from.max(to)));
    }
    if from == to {
        return Ok(from);
    }
    ensure_ids(prj);
    let id = prj.layout.enclosures[from].id;
    let room = |prj: &Project, i: usize| if prj.layout.enclosures[i].is_room() { Some(i) } else { room_of(&prj.layout, i) };
    if prj.layout.enclosures[from].is_room() {
        // Among the rooms: dropped onto a cabinet, before or after its room.
        let target = room(prj, to).ok_or_else(|| tr("A room goes before or after another room").to_string())?;
        if target == from {
            return Ok(from);
        }
        let level = prj.layout.enclosures[target].level;
        if prj.layout.enclosures[from].level != level {
            let on = buildings::level_by_id(&prj.layout, level).map(|_| level);
            buildings::set_room_level(prj, from, on, None)?;
        }
        move_block(prj, &[from], target + usize::from(after));
    } else {
        let lu = lineup_of(&prj.layout, from);
        if lu.contains(&to) {
            // Within its line-up: bayed in the new order.
            let mut order: Vec<usize> = lu.iter().copied().filter(|&i| i != from).collect();
            let k = order.iter().position(|&i| i == to).map_or(order.len(), |k| k + usize::from(after));
            order.insert(k, from);
            if order != lu {
                rebay(prj, &lu, &order);
            }
        } else {
            // The whole line-up, into the room of the row it is dropped on.
            let into = room(prj, to);
            let at = if prj.layout.enclosures[to].is_room() {
                // Dropped onto a room: after its cabinets.
                cabinets_in(&prj.layout, to).into_iter().filter(|i| !lu.contains(i)).max().map_or(to + 1, |i| i + 1)
            } else {
                let other = lineup_of(&prj.layout, to);
                if after { other.iter().max().map_or(n, |&i| i + 1) } else { other.iter().copied().min().unwrap_or(to) }
            };
            if into != room_of(&prj.layout, from) {
                set_room(prj, from, into, None)?;
            }
            move_block(prj, &lu, at);
        }
    }
    Ok(prj.layout.enclosures.iter().position(|e| e.id == id).unwrap_or(from))
}

/// Moves building `from` before building `to`, or after it; the index `from` has
/// afterwards.
pub fn move_building(prj: &mut Project, from: usize, to: usize, after: bool) -> Result<usize, String> {
    let n = prj.layout.buildings.len();
    if from >= n || to >= n {
        return Err(tr("No such building").to_string());
    }
    if from == to {
        return Ok(from);
    }
    let b = prj.layout.buildings.remove(from);
    let to = if to > from { to - 1 } else { to };
    let at = (to + usize::from(after)).min(n - 1);
    prj.layout.buildings.insert(at, b);
    Ok(at)
}

#[cfg(test)]
mod tests;
