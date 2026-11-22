//! Connection definition points: placed on a wire, they define the number, colour
//! and cross-section of the whole connection and the cable core of that wire. The
//! wires take the values when a point is placed, changed or applied again.

use crate::geom::P;
use crate::model::*;
use crate::netlist::{Net, Netlist};
use crate::issue::{Issue, Severity};

/// Wire of the page the point `q` lies on.
pub fn wire_at(page: &Page, q: P) -> Option<&Wire> {
    page.wires.iter().find(|w| w.distance_to(q) < 0.01)
}

fn net_of<'a>(page: &Page, open: &'a Netlist, cd: &ConnDef) -> Option<&'a Net> {
    wire_at(page, cd.pos).and_then(|w| open.net_of_wire(w.id))
}

/// A new point at `pos` that shows the current values of the connection there.
pub fn adopt(prj: &Project, open: &Netlist, page: usize, pos: P) -> ConnDef {
    let pg = &prj.pages[page];
    let mut cd = ConnDef { pos, ..Default::default() };
    if let Some(w) = wire_at(pg, pos) {
        cd.color = w.color.clone();
        cd.cross_section = w.cross_section.clone();
        cd.cable = w.cable.clone();
        cd.core = w.core.clone();
        cd.number = w.number.clone();
        if let Some(net) = open.net_of_wire(w.id) {
            cd.number = crate::wirenum::number_of(prj, net);
        }
    }
    cd
}

/// Values of the point that differ from its connection (empty values define nothing).
fn differs(prj: &Project, page: usize, cd: &ConnDef, net: &Net) -> bool {
    let pg = &prj.pages[page];
    let wires = net.wires.iter().filter_map(|(pi, id)| prj.pages[*pi].wires.iter().find(|w| w.id == *id));
    let mut differs = false;
    for w in wires {
        differs |= !cd.number.is_empty() && (w.number != cd.number || !w.number_fixed);
        differs |= !cd.color.is_empty() && w.color != cd.color;
        differs |= !cd.cross_section.is_empty() && w.cross_section != cd.cross_section;
    }
    if let Some(w) = wire_at(pg, cd.pos) {
        differs |= w.cable != cd.cable || (!cd.cable.is_empty() && w.core != cd.core);
    }
    differs
}

/// Gives the connection under point `id` its values. Returns the number of wires changed.
pub fn apply(prj: &mut Project, open: &Netlist, page: usize, id: u64) -> usize {
    let Some(cd) = prj.pages[page].conn_defs.iter().find(|c| c.id == id).cloned() else { return 0 };
    let Some(net) = net_of(&prj.pages[page], open, &cd).cloned() else { return 0 };
    let before = prj.clone();
    if !cd.number.is_empty() {
        crate::wirenum::set(prj, &net, &cd.number, true);
    }
    for (pi, wid) in &net.wires {
        if let Some(w) = prj.pages[*pi].wires.iter_mut().find(|w| w.id == *wid) {
            if !cd.color.is_empty() {
                w.color = cd.color.clone();
            }
            if !cd.cross_section.is_empty() {
                w.cross_section = cd.cross_section.clone();
            }
        }
    }
    let at = wire_at(&prj.pages[page], cd.pos).map(|w| w.id);
    if let Some(w) = prj.pages[page].wires.iter_mut().find(|w| Some(w.id) == at) {
        w.cable = cd.cable.clone();
        w.core = if cd.cable.is_empty() { String::new() } else { cd.core.clone() };
    }
    count_changed(&before, prj)
}

fn count_changed(before: &Project, after: &Project) -> usize {
    before
        .pages
        .iter()
        .zip(&after.pages)
        .map(|(a, b)| a.wires.iter().zip(&b.wires).filter(|(x, y)| x != y).count())
        .sum()
}

/// Applies all connection definition points of the project. Returns the number of
/// wires changed.
pub fn apply_all(prj: &mut Project) -> usize {
    let open = Netlist::build(prj, false);
    let ids: Vec<(usize, u64)> = prj.pages.iter().enumerate().flat_map(|(pi, pg)| pg.conn_defs.iter().map(move |c| (pi, c.id))).collect();
    let before = prj.clone();
    for (pi, id) in ids {
        apply(prj, &open, pi, id);
    }
    count_changed(&before, prj)
}

/// Text shown beside the point.
pub fn label(cd: &ConnDef) -> String {
    let mut parts: Vec<&str> = [&cd.number, &cd.color, &cd.cross_section].into_iter().map(String::as_str).filter(|s| !s.is_empty()).collect();
    let core = if cd.cable.is_empty() { String::new() } else { format!("{}:{}", cd.cable, cd.core) };
    if !core.is_empty() {
        parts.push(&core);
    }
    parts.join("  ")
}

/// Points beside wires, several points on one connection that contradict each
/// other, and points whose values the wires do not have.
pub fn problems(prj: &Project, open: &Netlist) -> Vec<Issue> {
    let mut v = Vec::new();
    let mut seen: Vec<(usize, usize, u64)> = Vec::new();
    for (pi, pg) in prj.pages.iter().enumerate() {
        for cd in &pg.conn_defs {
            let target = Some((pi, cd.id));
            let Some(w) = wire_at(pg, cd.pos) else {
                v.push(crate::issue!(Severity::Warning, target, "Connection definition point on page {} is not on a wire", pg.name));
                continue;
            };
            let Some(&n) = open.wire_net.get(&w.id) else { continue };
            if let Some(&(_, opi, oid)) = seen.iter().find(|s| s.0 == n) {
                let other = prj.pages[opi].conn_defs.iter().find(|c| c.id == oid);
                let same = other.is_some_and(|o| o.number == cd.number && o.color == cd.color && o.cross_section == cd.cross_section);
                if !same {
                    v.push(crate::issue!(Severity::Warning, target, "Connection {} has contradicting connection definition points", crate::wirenum::number_of(prj, &open.nets[n])));
                }
            } else {
                seen.push((n, pi, cd.id));
            }
            if differs(prj, pi, cd, &open.nets[n]) {
                v.push(crate::issue!(Severity::Info, target, "Connection definition point on page {} is not applied to its connection", pg.name));
            }
        }
    }
    v
}

#[cfg(test)]
mod tests;
