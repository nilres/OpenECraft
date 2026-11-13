//! Shared cable label layout, hit areas and deliberate page-local alignment.

use super::*;

#[derive(Clone, Debug)]
pub struct CoreLabel {
    pub wire: u64,
    pub which: CoreText,
    pub text: String,
    pub pos: P,
    pub anchor: Anchor,
    pub v: VAnchor,
    pub bounds: R,
    /// Relative to the original annotation position, including automatic clearance.
    pub offset: P,
    pub conflict: bool,
}

fn bounds(text: &str, q: P, anchor: Anchor, v: VAnchor) -> R {
    let wd = text::annotation_width(text, SMALL_SIZE).max(SMALL_SIZE);
    let x = match anchor { Anchor::Left => q.x, Anchor::Center => q.x - wd / 2., Anchor::Right => q.x - wd };
    let y = match v { VAnchor::Top => q.y, VAnchor::Middle => q.y - SMALL_SIZE * 0.6, VAnchor::Bottom => q.y - SMALL_SIZE * 1.2 };
    R::from_points(p(x, y), p(x + wd, y + SMALL_SIZE * 1.2))
}

fn obstacles(prj: &Project, pi: usize) -> Vec<R> {
    let page = &prj.pages[pi];
    let mut result: Vec<R> = page.symbols.iter().filter_map(|s| prj.symbol(&s.sym).map(|d| s.bounds(&d))).collect();
    let idx = Index::build(prj);
    for s in &page.symbols {
        result.extend(s.props.iter().map(|pp| prop_bounds(prj, pi, s, pp)));
        if let Some(def) = prj.symbol(&s.sym) {
            result.extend(tag_block_bounds(prj, &idx, pi, &def, s));
            result.extend(pin_label_places(prj, &idx, pi, &def, s).into_iter().map(|l| l.bounds));
        }
    }
    result.extend(page.texts.iter().map(|t| text_bounds(prj, pi, t)));
    result.extend(page.conn_defs.iter().map(|c| R::from_points(c.pos, c.pos).expand(1.)));
    result
}

fn blocked(page: &Page, r: R, occupied: &[R]) -> bool {
    let r = r.expand(0.25);
    occupied.iter().any(|o| r.overlaps(o))
        || page.wires.iter().any(|w| r.expand(WIRE_W * 0.5).crosses(w.a, w.b))
        || page.cable_defs.iter().any(|d| r.expand(SYMBOL_W * 0.5).crosses(d.a, d.b))
}

fn base(page: &Page, w: &Wire, tagged: bool, which: CoreText) -> Option<CoreLabel> {
    let text = match which { CoreText::Core => w.core.clone(), CoreText::Cable if tagged => structure::relative(page, &w.cable), _ => String::new() };
    if w.cable.is_empty() || w.hide_labels || text.is_empty() { return None; }
    let (pos, anchor, v) = wire_text_place(w, crate::cabledef::core_position(page, w), which == CoreText::Core);
    Some(CoreLabel { wire: w.id, which, bounds: bounds(&text, pos, anchor, v), text, pos, anchor, v, offset: P::default(), conflict: false })
}

fn shifted(mut label: CoreLabel, delta: P) -> CoreLabel {
    label.pos = label.pos + delta;
    label.offset = label.offset + delta;
    label.bounds = R::from_points(label.bounds.min + delta, label.bounds.max + delta);
    label
}

#[allow(clippy::too_many_arguments)]
fn place(page: &Page, w: &Wire, label: CoreLabel, definition: Option<&CableDef>, before: bool, gap: f32, occupied: &[R], alternate: bool, steps: usize) -> CoreLabel {
    let Some(def) = definition.or_else(|| page.cable_defs.iter().find(|d| d.cable == w.cable && crate::cabledef::crossing(w.a, w.b, d.a, d.b).is_some())) else { return label; };
    let Some(t) = crate::cabledef::crossing(w.a, w.b, def.a, def.b) else { return label; };
    let cross = w.a + (w.b - w.a) * t;
    let horizontal = (def.b.x - def.a.x).abs() >= (def.b.y - def.a.y).abs();
    let mut fallback = None;
    for side in [before, !before].into_iter().take(if alternate { 2 } else { 1 }) {
        for step in 0..steps {
            let clearance = gap + SYMBOL_W * 0.5 + step as f32 * 2.;
            let r = label.bounds;
            let delta = if horizontal {
                p(0., if side { cross.y - clearance - r.max.y } else { cross.y + clearance - r.min.y })
            } else {
                p(if side { cross.x - clearance - r.max.x } else { cross.x + clearance - r.min.x }, 0.)
            };
            let mut candidate = shifted(label.clone(), delta);
            candidate.conflict = blocked(page, candidate.bounds, occupied);
            if !candidate.conflict { return candidate; }
            if fallback.is_none() { fallback = Some(candidate); }
        }
    }
    fallback.unwrap_or(label)
}

/// Stable deterministic layout. Manual offsets keep their original meaning.
pub fn core_labels(prj: &Project, pi: usize) -> Vec<CoreLabel> {
    let page = &prj.pages[pi];
    if !page.wires.iter().any(|w| !w.cable.is_empty() && !w.hide_labels) { return Vec::new(); }
    let mut occupied = obstacles(prj, pi);
    let mut labels = Vec::new();
    let mut automatic = Vec::new();
    for (w, tagged) in cabled_wires(page) {
        for which in [CoreText::Cable, CoreText::Core] {
            let Some(label) = base(page, w, tagged, which) else { continue };
            if let Some(offset) = which.offset(w) {
                let mut label = shifted(label, offset);
                label.conflict = blocked(page, label.bounds, &occupied);
                occupied.push(label.bounds);
                labels.push(label);
            } else { automatic.push((w, label)); }
        }
    }
    for (w, label) in automatic {
        let label = place(page, w, label, None, true, 0.8, &occupied, true, 9);
        occupied.push(label.bounds);
        labels.push(label);
    }
    labels
}

pub fn core_text_bounds(prj: &Project, pi: usize, w: &Wire, which: CoreText) -> Option<R> {
    core_labels(prj, pi).into_iter().find(|l| l.wire == w.id && l.which == which).map(|l| l.bounds)
}

/// A read-only preview for the core labels crossed by one definition on this page.
pub fn preview_core_labels(prj: &Project, pi: usize, id: u64, before: bool, gap: f32) -> Vec<CoreLabel> {
    if !gap.is_finite() || !(0.5..=10.).contains(&gap) { return Vec::new(); }
    let page = &prj.pages[pi];
    let Some(def) = page.cable_defs.iter().find(|d| d.id == id) else { return Vec::new() };
    let ids = crate::cabledef::crossed_wires(page, def.a, def.b);
    let targets = |l: &CoreLabel| l.which == CoreText::Core && ids.contains(&l.wire) && page.wires.iter().any(|w| w.id == l.wire && w.cable == def.cable);
    let all = core_labels(prj, pi);
    let mut occupied = obstacles(prj, pi);
    occupied.extend(all.iter().filter(|l| !targets(l)).map(|l| l.bounds));
    let mut fallback = Vec::new();
    for step in 0..9 {
        let mut occupied = occupied.clone();
        let mut result = Vec::new();
        for wid in &ids {
            let Some(w) = page.wires.iter().find(|w| w.id == *wid && w.cable == def.cable) else { continue };
            let Some(label) = base(page, w, false, CoreText::Core) else { continue };
            let label = place(page, w, label, Some(def), before, gap + step as f32 * 2., &occupied, false, 1);
            occupied.push(label.bounds);
            result.push(label);
        }
        if result.iter().all(|l| !l.conflict) { return result; }
        if fallback.is_empty() { fallback = result; }
    }
    fallback
}

/// One project mutation; leaves geometry, assignments, cable tags and other pages alone.
pub fn align_core_labels(prj: &mut Project, pi: usize, id: u64, before: bool, gap: f32) -> usize {
    let labels = preview_core_labels(prj, pi, id, before, gap);
    if labels.iter().any(|l| l.conflict) { return 0; }
    for label in &labels {
        if let Some(w) = prj.pages[pi].wires.iter_mut().find(|w| w.id == label.wire) { w.core_offset = Some(label.offset); }
    }
    labels.len()
}

#[cfg(test)]
mod tests;
