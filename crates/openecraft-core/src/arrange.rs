//! Arranging the selection of a page: align items to a common edge or centre,
//! distribute them evenly, and copy them several times at a fixed distance (EPLAN's
//! multiple copy). Wire ends attached to moved items follow them.

use crate::geom::{p, P, R, GRID};
use crate::model::{Page, Project};
use crate::ops;
use crate::routing;
use crate::symlib::Symbols;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    /// Horizontal centres on one vertical line.
    Center,
    Right,
    Top,
    /// Vertical centres on one horizontal line.
    Middle,
    Bottom,
}

impl Align {
    pub const ALL: [Align; 6] = [Align::Left, Align::Center, Align::Right, Align::Top, Align::Middle, Align::Bottom];

    /// Code for commands (`left`, `center` …).
    pub fn code(self) -> &'static str {
        match self {
            Align::Left => "left",
            Align::Center => "center",
            Align::Right => "right",
            Align::Top => "top",
            Align::Middle => "middle",
            Align::Bottom => "bottom",
        }
    }

    pub fn from_code(s: &str) -> Option<Align> {
        Align::ALL.into_iter().find(|a| a.code() == s.trim().to_ascii_lowercase())
    }
}

/// The items of the selection that are arranged one by one: symbols, texts, shapes,
/// dimensions, images (wires and definitions follow or stay).
fn items(symbols: &Symbols, page: &Page, sel: &BTreeSet<u64>) -> Vec<(u64, R)> {
    let ids = page
        .symbols
        .iter()
        .map(|s| s.id)
        .chain(page.texts.iter().map(|t| t.id))
        .chain(page.shapes.iter().map(|s| s.id))
        .chain(page.dimensions.iter().map(|d| d.id))
        .chain(page.images.iter().map(|i| i.id));
    ids.filter(|id| sel.contains(id)).filter_map(|id| ops::item_bounds(symbols, page, id).map(|b| (id, b))).collect()
}

/// Moves one item by `d` (snapped so symbols stay on the grid), its attached wire
/// ends with it.
fn shift(symbols: &Symbols, page: &mut Page, id: u64, d: P) {
    let one: BTreeSet<u64> = [id].into_iter().collect();
    let d = if page.symbols.iter().any(|s| s.id == id) { d.snap(GRID) } else { d };
    if d.x.abs() < 1e-4 && d.y.abs() < 1e-4 {
        return;
    }
    let attached = ops::attached_wire_ends(symbols, page, &one);
    ops::translate(page, &one, &attached, d);
}

/// Aligns the selected items to the outermost edge (or the common centre) of the
/// selection; attached wires stay square. Returns how many items moved.
pub fn align(prj: &mut Project, page: usize, sel: &BTreeSet<u64>, how: Align) -> usize {
    let before = routing::Before::of(&prj.pages[page], sel);
    let moved = align_items(&prj.symbol_table(), &mut prj.pages[page], sel, how);
    routing::square_wires(prj, page, &before);
    moved
}

fn align_items(symbols: &Symbols, page: &mut Page, sel: &BTreeSet<u64>, how: Align) -> usize {
    let list = items(symbols, page, sel);
    if list.len() < 2 {
        return 0;
    }
    let mut all = R::nothing();
    for (_, b) in &list {
        all.union(*b);
    }
    let target = |b: &R| -> P {
        match how {
            Align::Left => p(all.min.x - b.min.x, 0.0),
            Align::Right => p(all.max.x - b.max.x, 0.0),
            Align::Center => p(all.center().x - b.center().x, 0.0),
            Align::Top => p(0.0, all.min.y - b.min.y),
            Align::Bottom => p(0.0, all.max.y - b.max.y),
            Align::Middle => p(0.0, all.center().y - b.center().y),
        }
    };
    let mut moved = 0;
    for (id, b) in list {
        let d = target(&b);
        if d.x.abs() > 1e-4 || d.y.abs() > 1e-4 {
            shift(symbols, page, id, d);
            moved += 1;
        }
    }
    moved
}

/// Distributes the selected items evenly between the first and the last one
/// (horizontally by their centres, or vertically); attached wires stay square.
/// Returns how many items moved.
pub fn distribute(prj: &mut Project, page: usize, sel: &BTreeSet<u64>, horizontal: bool) -> usize {
    let before = routing::Before::of(&prj.pages[page], sel);
    let moved = distribute_items(&prj.symbol_table(), &mut prj.pages[page], sel, horizontal);
    routing::square_wires(prj, page, &before);
    moved
}

fn distribute_items(symbols: &Symbols, page: &mut Page, sel: &BTreeSet<u64>, horizontal: bool) -> usize {
    let mut list = items(symbols, page, sel);
    if list.len() < 3 {
        return 0;
    }
    let c = |b: &R| if horizontal { b.center().x } else { b.center().y };
    list.sort_by(|a, b| c(&a.1).total_cmp(&c(&b.1)));
    let (first, last) = (c(&list[0].1), c(&list[list.len() - 1].1));
    let step = (last - first) / (list.len() - 1) as f32;
    let mut moved = 0;
    for (i, (id, b)) in list.iter().enumerate().skip(1).take(list.len() - 2) {
        let off = first + step * i as f32 - c(b);
        if off.abs() > 1e-4 {
            shift(symbols, page, *id, if horizontal { p(off, 0.0) } else { p(0.0, off) });
            moved += 1;
        }
    }
    moved
}

/// Copies the selection `count` times, each copy `offset` further than the one
/// before (devices get new tags, terminals and plug pins new numbers, as when
/// pasting). Returns the ids of all new items.
pub fn multiple_copy(prj: &mut Project, page_i: usize, sel: &BTreeSet<u64>, count: usize, offset: P) -> BTreeSet<u64> {
    let clip = ops::copy(&prj.pages[page_i], sel);
    let b = clip.bounds();
    if !b.is_valid() {
        return BTreeSet::new();
    }
    let mut out = BTreeSet::new();
    for k in 1..=count {
        out.extend(ops::paste(prj, page_i, &clip, b.min + offset * k as f32));
    }
    out
}

/// A typed point: `x y` or `x, y` (absolute, mm), `@dx dy` relative to `last`.
pub fn parse_point(text: &str, last: Option<P>) -> Option<P> {
    let t = text.trim();
    let (relative, t) = match t.strip_prefix('@') {
        Some(rest) => (true, rest),
        None => (false, t),
    };
    let nums: Vec<f32> = t.split(|c: char| c == ',' || c == ';' || c.is_whitespace()).filter(|s| !s.is_empty()).map(|s| s.replace(',', ".").parse::<f32>()).collect::<Result<_, _>>().ok()?;
    let [x, y] = nums[..] else { return None };
    if relative {
        last.map(|l| l + p(x, y))
    } else {
        Some(p(x, y))
    }
}

#[cfg(test)]
mod tests;
