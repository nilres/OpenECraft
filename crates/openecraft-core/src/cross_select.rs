//! Cross selection between the schematic and the panel layout (2D and 3D): what a
//! selection on a page is in the layout – the placed devices and terminals of its
//! symbols, the wires of the virtual wiring along its wires, its cables – and where a
//! selection of the layout is on the pages. No UI: the editor keeps the views in step
//! with it (cross-select mode, Show in schematic, Show in 3D view).

use crate::analysis::Analysis;
use crate::model::Project;
use crate::netlist::{pin_designation, Connection};
use crate::panel::{self, ItemKind};
use std::collections::BTreeSet;

/// A selection of the panel layout and its 3D view.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayoutPick {
    /// Placed devices and terminals: (enclosure, item id).
    pub items: Vec<(usize, u64)>,
    /// Wires of the virtual wiring: their (from, to) connection points.
    pub wires: Vec<(String, String)>,
    /// Cables by tag (their key in the 3D routing).
    pub cables: Vec<String>,
}

impl LayoutPick {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.wires.is_empty() && self.cables.is_empty()
    }
}

fn push<T: PartialEq>(v: &mut Vec<T>, x: T) {
    if !v.contains(&x) {
        v.push(x);
    }
}

/// The connection points of a connection, as the virtual wiring names them.
fn ends(prj: &Project, c: &Connection) -> (String, String) {
    (pin_designation(prj, c.a), pin_designation(prj, c.b))
}

/// The wire of the virtual wiring between `a` and `b`, either way round.
fn routed_wire(an: &Analysis, (a, b): &(String, String)) -> Option<(String, String)> {
    an.wiring.iter().find(|w| (&w.from == a && &w.to == b) || (&w.from == b && &w.to == a)).map(|w| w.key())
}

/// What the items `sel` of page `page_i` are in the panel layout: the placed devices
/// and terminals of its symbols, the wires of the virtual wiring along its wires (a
/// core of a cable that is not wired in an enclosure: the cable), the cables of its
/// cable definitions and cable symbols.
pub fn in_layout(prj: &Project, an: &Analysis, page_i: usize, sel: &BTreeSet<u64>) -> LayoutPick {
    let mut pick = LayoutPick::default();
    let Some(page) = prj.pages.get(page_i) else { return pick };
    for s in page.symbols.iter().filter(|s| sel.contains(&s.id)) {
        if let Some(tag) = s.values.get("function.cable").filter(|t| !t.is_empty()) {
            push(&mut pick.cables, tag.clone());
        }
        if s.tag.is_empty() {
            continue;
        }
        if prj.cable(&s.tag).is_some() {
            push(&mut pick.cables, s.tag.clone());
        }
        if let Some((ei, it)) = panel::locate_symbol(prj, s) {
            push(&mut pick.items, (ei, it.id));
        }
    }
    for cd in page.cable_defs.iter().filter(|c| sel.contains(&c.id) && !c.cable.is_empty()) {
        push(&mut pick.cables, cd.cable.clone());
    }
    for w in page.wires.iter().filter(|w| sel.contains(&w.id)) {
        let mut wired = false;
        for c in an.connections.iter().filter(|c| c.wires.contains(&(page_i, w.id))) {
            if let Some(key) = routed_wire(an, &ends(prj, c)) {
                push(&mut pick.wires, key);
                wired = true;
            }
        }
        if !wired && !w.cable.is_empty() && prj.cable(&w.cable).is_some() {
            push(&mut pick.cables, w.cable.clone());
        }
    }
    pick
}

/// Where a selection of the panel layout is on the pages: (page, item id) of the
/// symbols of its devices and terminals (main functions first, then in page order), of
/// the wires along its wires and of the cable definitions and cores of its cables.
pub fn in_schematic(prj: &Project, an: &Analysis, pick: &LayoutPick) -> Vec<(usize, u64)> {
    let mut found: Vec<(usize, u64)> = Vec::new();
    for &(ei, id) in &pick.items {
        let Some(it) = prj.layout.enclosures.get(ei).and_then(|e| e.item(id)) else { continue };
        if it.kind != ItemKind::Device || it.tag.is_empty() {
            continue;
        }
        let mains: Vec<(usize, u64)> = an.index.devices.get(&it.tag).map(|d| d.mains.iter().map(|l| (l.page, l.sym_id)).collect()).unwrap_or_default();
        let mut of_item: Vec<(bool, usize, u64)> = Vec::new();
        for (pi, page) in prj.pages.iter().enumerate() {
            for s in page.symbols.iter().filter(|s| s.tag == it.tag) {
                if panel::locate_symbol(prj, s).is_some_and(|(k, x)| k == ei && x.id == it.id) {
                    of_item.push((!mains.contains(&(pi, s.id)), pi, s.id));
                }
            }
        }
        of_item.sort_by_key(|&(aux, pi, _)| (aux, pi));
        for (_, pi, sid) in of_item {
            push(&mut found, (pi, sid));
        }
    }
    for key in &pick.wires {
        for c in an.connections.iter().filter(|c| {
            let (a, b) = ends(prj, c);
            (a == key.0 && b == key.1) || (a == key.1 && b == key.0)
        }) {
            if c.wires.is_empty() {
                for pin in [c.a, c.b] {
                    if let Some(s) = prj.pages.get(pin.page).and_then(|p| p.symbols.get(pin.sym)) {
                        push(&mut found, (pin.page, s.id));
                    }
                }
            }
            for &w in &c.wires {
                push(&mut found, w);
            }
        }
    }
    for tag in &pick.cables {
        for (pi, page) in prj.pages.iter().enumerate() {
            for cd in page.cable_defs.iter().filter(|c| &c.cable == tag) {
                push(&mut found, (pi, cd.id));
            }
        }
        for (pi, page) in prj.pages.iter().enumerate() {
            for w in page.wires.iter().filter(|w| &w.cable == tag) {
                push(&mut found, (pi, w.id));
            }
        }
    }
    found
}

/// The part of `found` ([`in_schematic`]) to select: what is on page `current`, otherwise
/// what is on the page of the first one. Gives the page and its items (the first one
/// to show first).
pub fn on_one_page(found: &[(usize, u64)], current: usize) -> Option<(usize, Vec<u64>)> {
    let page = if found.iter().any(|f| f.0 == current) { current } else { found.first()?.0 };
    Some((page, found.iter().filter(|f| f.0 == page).map(|f| f.1).collect()))
}

#[cfg(test)]
mod tests;
