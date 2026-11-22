//! Cable definition lines: a line drawn across wires makes them cores of a cable.

use crate::geom::P;
use crate::model::*;
use crate::issue::{Issue, Severity};
use openecraft_i18n::tr_fmt;

/// Non-mutating assignment preview, in order from the definition line's start.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
pub struct AssignmentPlan {
    pub crossed: Vec<u64>,
    pub kept: Vec<u64>,
    pub free_cores: Vec<String>,
    pub assignments: Vec<(u64, String)>,
    pub missing: Vec<u64>,
    pub conflicts: Vec<u64>,
}

impl AssignmentPlan {
    pub fn unassigned(&self) -> usize { self.crossed.len() - self.kept.len() }
    pub fn can_apply(&self) -> bool { !self.crossed.is_empty() && self.missing.is_empty() && self.conflicts.is_empty() }
    pub fn summary(&self) -> String {
        tr_fmt!("{} wires crossed, {} assigned, {} free cores, {} unassigned", self.crossed.len(), self.kept.len(), self.free_cores.len(), self.unassigned())
    }
}

fn cross(a: P, b: P) -> f32 {
    a.x * b.y - a.y * b.x
}

/// Parameter (0..1 along `a`–`b`) where the segment `c`–`d` crosses it.
pub(crate) fn crossing(a: P, b: P, c: P, d: P) -> Option<f32> {
    let r = b - a;
    let s = d - c;
    let den = cross(r, s);
    if den.abs() < 1e-9 {
        return None;
    }
    let t = cross(c - a, s) / den;
    let u = cross(c - a, r) / den;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then_some(t)
}

/// First intersection measured along the definition line, with the corresponding
/// parameter on the sheet wire. A wire can appear only once in a cable definition.
pub(crate) fn wire_crossing(w: &Wire, a: P, b: P) -> Option<(f32, f32)> {
    if w.bend.is_none() {
        return Some((crossing(a,b,w.a,w.b)?, crossing(w.a,w.b,a,b)?));
    }
    let points = w.drawn_points();
    let segments = (points.len() - 1) as f32;
    points.windows(2).enumerate().filter_map(|(i, s)| {
        let line_t = crossing(a, b, s[0], s[1])?;
        let local_t = crossing(s[0], s[1], a, b)?;
        Some((line_t, (i as f32 + local_t) / segments))
    }).min_by(|left, right| left.0.total_cmp(&right.0))
}

/// Wires of the page crossed by the line `a`–`b`, in order from `a`.
pub fn crossed_wires(page: &Page, a: P, b: P) -> Vec<u64> {
    let mut v: Vec<(f32, u64)> = page.wires.iter().filter_map(|w| wire_crossing(w, a, b).map(|(t, _)| (t, w.id))).collect();
    v.sort_by(|x, y| x.0.total_cmp(&y.0));
    v.into_iter().map(|(_, id)| id).collect()
}

/// Position along wire `w` (0..1) of its core stroke and designation: where a cable
/// definition line of its cable crosses it, else at 30 %.
pub fn core_position(page: &Page, w: &Wire) -> f32 {
    page.cable_defs.iter().filter(|d| d.cable == w.cable).find_map(|d| wire_crossing(w, d.a, d.b).map(|(_, t)| t)).unwrap_or(0.3)
}

/// Core designations "1 2 … n", with a protective earth core "GNYE" at the end.
pub fn numbered_cores(n: usize, pe: bool) -> String {
    let mut v: Vec<String> = (1..=n).map(|i| i.to_string()).collect();
    if pe {
        v.push("GNYE".into());
    }
    v.join(" ")
}

/// Preview existing or newly created cable cores without changing the project.
pub fn plan(prj: &Project, page: usize, a: P, b: P, tag: &str) -> AssignmentPlan {
    let used: Vec<_> = if prj.cable(tag).is_some() {
        prj.pages.iter().enumerate().flat_map(|(pi, pg)| pg.wires.iter().filter(move |w| w.cable == tag).map(move |w| (pi, w.id, w.core.clone()))).collect()
    } else { Vec::new() };
    plan_with_uses(prj, page, a, b, tag, &used)
}

/// Preview using the caller's cable assignments (page, wire id, core). The caller
/// refreshes them after edits and undo; applying always uses a fresh plan.
pub fn plan_with_uses(prj: &Project, page: usize, a: P, b: P, tag: &str, uses: &[(usize, u64, String)]) -> AssignmentPlan {
    let crossed = crossed_wires(&prj.pages[page], a, b);
    let cable = prj.cable(tag);
    let mut cores = cable.map(Cable::core_list).unwrap_or_else(|| (1..=crossed.len().max(1)).map(|n| n.to_string()).collect());
    let used: Vec<&str> = uses.iter()
        .filter(|(pi, id, _)| cable.is_some() && !(*pi == page && crossed.contains(id)))
        .map(|(_, _, core)| core.as_str())
        .collect();
    let kept: Vec<u64> = prj.pages[page]
        .wires
        .iter()
        .filter(|w| crossed.contains(&w.id) && cable.is_some() && w.cable == tag && !w.core.is_empty())
        .map(|w| w.id)
        .collect();
    let kept_cores: Vec<&str> = prj.pages[page].wires.iter().filter(|w| kept.contains(&w.id)).map(|w| w.core.as_str()).collect();
    let mut unique = std::collections::BTreeSet::new();
    cores.retain(|c| !used.contains(&c.as_str()) && !kept_cores.contains(&c.as_str()) && unique.insert(c.clone()));
    let mut result = AssignmentPlan { crossed: crossed.clone(), kept, free_cores: cores.clone(), ..Default::default() };
    let mut free = cores.into_iter();
    for wid in &crossed {
        let Some(w) = prj.pages[page].wires.iter().find(|w| w.id == *wid) else { continue };
        if result.kept.contains(wid) { continue; }
        if !w.core.is_empty() || (!w.cable.is_empty() && (w.cable != tag || cable.is_none())) {
            result.conflicts.push(*wid);
        } else if let Some(core) = free.next() {
            result.assignments.push((*wid, core));
        } else {
            result.missing.push(*wid);
        }
    }
    result
}

/// Assign all pending crossed wires, or none when cores are missing or another
/// cable already owns a wire. Existing assignments and text positions are kept.
pub fn apply(prj: &mut Project, page: usize, id: u64) -> usize {
    let Some(def) = prj.pages[page].cable_defs.iter().find(|c| c.id == id).cloned() else { return 0 };
    if prj.cable(&def.cable).is_none() { return 0; }
    let plan = plan(prj, page, def.a, def.b, &def.cable);
    if !plan.can_apply() { return plan.kept.len(); }
    for (id, core) in &plan.assignments {
        let Some(w) = prj.pages[page].wires.iter_mut().find(|w| w.id == *id) else { continue };
        w.cable = def.cable.clone();
        w.core = core.clone();
        (w.core_offset, w.cable_offset) = (None, None);
    }
    plan.kept.len() + plan.assignments.len()
}

/// Adds a correctable definition line. A new cable is created only when the crossed
/// wires have no conflicting assignments, with the tag `cable` (the next free one when
/// empty; the caller checks it with `ops::new_cable_problem`); its cores are numbered
/// for the crossings.
pub fn add(prj: &mut Project, page: usize, a: P, b: P, cable: &str) -> u64 {
    let preview = plan(prj, page, a, b, cable);
    let tag = if prj.cable(cable).is_some() || !preview.conflicts.is_empty() {
        cable.to_string()
    } else {
        let st = crate::structure::Dt::of_page(&prj.pages[page]).structure();
        let tag = crate::ops::add_cable(prj, &st);
        let named = if cable.is_empty() { tag.clone() } else { cable.to_string() };
        if let Some(c) = prj.cables.iter_mut().find(|c| c.tag == tag) {
            c.cores = numbered_cores(preview.crossed.len().max(1), false);
            c.tag = named.clone();
        }
        named
    };
    let id = prj.new_id();
    prj.pages[page].cable_defs.push(CableDef { id, a, b, cable: tag, shield: false });
    apply(prj, page, id);
    id
}

/// Text at the end of the line: cable tag, type, cores × cross-section.
pub fn label(prj: &Project, page: &Page, def: &CableDef) -> Vec<String> {
    let mut v = vec![crate::structure::relative(page, &def.cable)];
    if let Some(c) = prj.cable(&def.cable) {
        if !c.cable_type.is_empty() {
            v.push(c.cable_type.clone());
        }
        let n = c.core_list().len();
        if n > 0 && !c.cross_section.is_empty() {
            v.push(format!("{n}x{}", c.cross_section));
        }
    }
    v
}

/// Lines of unknown cables and lines that cross no wire.
pub fn problems(prj: &Project) -> Vec<Issue> {
    let mut v = Vec::new();
    for (pi, pg) in prj.pages.iter().enumerate() {
        for def in &pg.cable_defs {
            let target = Some((pi, def.id));
            if def.cable.is_empty() {
                v.push(crate::issue!(Severity::Error, target, "Cable definition on page {} has no cable. Select a cable before assigning crossed wires.", pg.name));
            } else if prj.cable(&def.cable).is_none() {
                v.push(crate::issue!(Severity::Error, target, "Cable definition on page {} refers to the unknown cable {}", pg.name, def.cable));
            } else if crossed_wires(pg, def.a, def.b).is_empty() {
                v.push(crate::issue!(Severity::Warning, target, "Cable definition {} on page {} crosses no wire", def.cable, pg.name));
            } else {
                let plan = plan(prj, pi, def.a, def.b, &def.cable);
                if plan.unassigned() > 0 {
                    v.push(crate::issue!(Severity::Warning, target, "Cable definition {} on page {}: {} crossed wires, {} unassigned, {} free cores, {} additional cores required, {} conflicting assignments", def.cable, pg.name, plan.crossed.len(), plan.unassigned(), plan.free_cores.len(), plan.missing.len(), plan.conflicts.len()));
                }
            }
        }
    }
    v
}

#[cfg(test)]
mod tests;
