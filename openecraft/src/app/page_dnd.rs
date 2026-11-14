//! Moving pages and sections in the Pages navigator by drag and drop: where a drop
//! lands (between pages, onto a section, between sections, below the list for the
//! top level), the marker in the list, the label under the pointer that tells the
//! name the page or section gets, opening a collapsed section the drag is held over
//! and scrolling at the edges of the list.

use super::*;
use openecraft_core::sections;

/// What is dragged in the page list (the drag payload).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Dragged {
    Page(usize),
    Section(u64),
}

/// Where a drop puts the dragged row.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum DropAt {
    /// A page into a section (0: top level) before one of its pages (`None`: after them).
    Page { section: u64, before: Option<usize> },
    /// A section into a section (0: top level) before one of its subsections (`None`:
    /// after them).
    Section { parent: u64, before: Option<u64> },
}

/// How the list shows the drop.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Mark {
    /// An insertion line at this height, from `left` to the right edge.
    Line { y: f32, left: f32 },
    /// The section header the drop goes into.
    Into(egui::Rect),
}

/// Part of a row the pointer is over.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Zone {
    Above,
    Into,
    Below,
}

/// The zone of a row from `top` to `bottom` at height `y`: halves for a page row;
/// with `into` the middle half of the row means into it (a section header).
pub(super) fn zone(top: f32, bottom: f32, y: f32, into: bool) -> Zone {
    let t = (y - top) / (bottom - top).max(1.0);
    match (into, t) {
        (true, t) if t < 0.25 => Zone::Above,
        (true, t) if t > 0.75 => Zone::Below,
        (true, _) => Zone::Into,
        (false, t) if t < 0.5 => Zone::Above,
        (false, _) => Zone::Below,
    }
}

/// The page after `page` in its section, if any.
fn next_in_section(prj: &Project, page: usize) -> Option<usize> {
    (page + 1 < prj.pages.len() && sections::of_page(prj, page + 1) == sections::of_page(prj, page)).then_some(page + 1)
}

/// The section after `id` among its siblings, if any.
fn next_sibling(prj: &Project, id: u64) -> Option<u64> {
    let parent = sections::get(prj, id).map_or(0, |s| s.parent);
    let siblings = sections::children(prj, parent);
    let k = siblings.iter().position(|&c| c == id)?;
    siblings.get(k + 1).copied()
}

fn parent(prj: &Project, id: u64) -> u64 {
    sections::get(prj, id).map_or(0, |s| if sections::get(prj, s.parent).is_some() { s.parent } else { 0 })
}

/// Whether the drop leaves everything where it is.
pub(super) fn unchanged(prj: &Project, dragged: Dragged, drop: DropAt) -> bool {
    match (dragged, drop) {
        (Dragged::Page(p), DropAt::Page { section, before }) => sections::of_page(prj, p) == section && (before == Some(p) || before == next_in_section(prj, p)),
        (Dragged::Section(id), DropAt::Section { parent: to, before }) => parent(prj, id) == to && (before == Some(id) || before == next_sibling(prj, id)),
        _ => true,
    }
}

/// Collects the drop target while the rows of the page list are drawn.
pub(super) struct Probe {
    /// Rows can be dragged (not in the list grouped by structure).
    pub enabled: bool,
    pub dragged: Option<Dragged>,
    pointer: Option<egui::Pos2>,
    /// The section header under the pointer (its id and the header's widget id).
    pub over_section: Option<(u64, egui::Id)>,
    pub target: Option<(DropAt, Mark)>,
    /// The dragged section and the sections below it.
    blocked: Vec<u64>,
    /// The pointer is over a row the drag cannot go to.
    pub refused: bool,
    /// Bottom and left edge of the rows drawn so far.
    bottom: f32,
    left: f32,
}

impl Probe {
    /// A probe for a list whose rows are not dragged.
    pub fn off() -> Probe {
        Probe { enabled: false, dragged: None, pointer: None, over_section: None, target: None, blocked: Vec::new(), refused: false, bottom: f32::NEG_INFINITY, left: f32::INFINITY }
    }

    /// A probe for the rows of `ui` (the inside of the list's scroll area).
    pub fn new(ui: &egui::Ui, prj: &Project) -> Probe {
        let dragged = egui::DragAndDrop::payload::<Dragged>(ui.ctx()).map(|d| *d);
        let pointer = ui.ctx().pointer_latest_pos().filter(|p| ui.clip_rect().contains(*p));
        let blocked = match dragged {
            Some(Dragged::Section(id)) => sections::subtree(prj, id),
            _ => Vec::new(),
        };
        Probe { enabled: true, dragged, pointer: pointer.filter(|_| dragged.is_some()), blocked, ..Probe::off() }
    }

    fn over(&mut self, rect: egui::Rect) -> Option<f32> {
        self.bottom = self.bottom.max(rect.bottom());
        self.left = self.left.min(rect.left());
        self.pointer.map(|p| p.y).filter(|y| (rect.top()..rect.bottom()).contains(y))
    }

    /// A page row: a dragged page goes before or after it.
    pub fn page(&mut self, prj: &Project, rect: egui::Rect, i: usize) {
        let Some(y) = self.over(rect) else { return };
        if !matches!(self.dragged, Some(Dragged::Page(_))) {
            return;
        }
        let section = sections::of_page(prj, i);
        self.target = Some(match zone(rect.top(), rect.bottom(), y, false) {
            Zone::Above => (DropAt::Page { section, before: Some(i) }, Mark::Line { y: rect.top(), left: rect.left() }),
            _ => (DropAt::Page { section, before: next_in_section(prj, i) }, Mark::Line { y: rect.bottom(), left: rect.left() }),
        });
    }

    /// A section header (`block_bottom`: the bottom of its open body): a dragged page
    /// goes into it, a dragged section into it or before or after it.
    pub fn section(&mut self, prj: &Project, header: &egui::Response, block_bottom: f32, id: u64) {
        self.section_at(prj, header.rect, block_bottom, id);
        if self.over_section.is_some_and(|(s, _)| s == id) {
            self.over_section = Some((id, header.id));
        }
    }

    fn section_at(&mut self, prj: &Project, header: egui::Rect, block_bottom: f32, id: u64) {
        self.bottom = self.bottom.max(block_bottom);
        let Some(y) = self.over(header) else { return };
        self.over_section = Some((id, egui::Id::NULL));
        match self.dragged {
            Some(Dragged::Page(_)) => self.target = Some((DropAt::Page { section: id, before: None }, Mark::Into(header))),
            Some(Dragged::Section(_)) => {
                let zone = zone(header.top(), header.bottom(), y, true);
                // Into itself or before / after a section below it.
                if self.blocked.contains(&id) && (zone == Zone::Into || self.blocked.contains(&parent(prj, id))) {
                    self.refused = true;
                    return;
                }
                let p = parent(prj, id);
                self.target = Some(match zone {
                    Zone::Above => (DropAt::Section { parent: p, before: Some(id) }, Mark::Line { y: header.top(), left: header.left() }),
                    Zone::Below => (DropAt::Section { parent: p, before: next_sibling(prj, id) }, Mark::Line { y: block_bottom, left: header.left() }),
                    Zone::Into => (DropAt::Section { parent: id, before: None }, Mark::Into(header)),
                });
            }
            None => {}
        }
    }

    /// The space below the rows (`strip`: the top-level row shown while dragging):
    /// the end of the top level.
    pub fn below(&mut self, strip: Option<egui::Rect>) {
        let Some(y) = self.pointer.map(|p| p.y) else { return };
        if self.target.is_some() || self.refused || y < self.bottom {
            return;
        }
        let mark = strip.map_or(Mark::Line { y: self.bottom, left: self.left }, Mark::Into);
        self.target = match self.dragged {
            Some(Dragged::Page(_)) => Some((DropAt::Page { section: 0, before: None }, mark)),
            Some(Dragged::Section(_)) => Some((DropAt::Section { parent: 0, before: None }, mark)),
            None => None,
        };
    }

    /// Scrolls the list while a drag is held near its top or bottom edge.
    pub fn scroll_at_edges(ui: &egui::Ui) {
        if !egui::DragAndDrop::has_payload_of_type::<Dragged>(ui.ctx()) {
            return;
        }
        let (clip, Some(p)) = (ui.clip_rect(), ui.ctx().pointer_latest_pos()) else { return };
        if !clip.x_range().contains(p.x) {
            return;
        }
        const EDGE: f32 = 24.0;
        let dy = if (clip.top() - EDGE..clip.top() + EDGE).contains(&p.y) {
            8.0
        } else if (clip.bottom() - EDGE..clip.bottom() + EDGE).contains(&p.y) {
            -8.0
        } else {
            return;
        };
        ui.scroll_with_delta_animation(egui::vec2(0.0, dy), egui::style::ScrollAnimation::none());
        ui.ctx().request_repaint();
    }
}

/// What a drop does, for the label at the pointer: `Ok(None)` for no change, `Err` for
/// a drag that cannot be dropped.
type Outcome = Result<Option<String>, String>;

/// The drag of the page list between frames: the last target and its description,
/// and the section the drag is held over.
#[derive(Default)]
pub(super) struct DragState {
    described: Option<(Dragged, Option<DropAt>, Outcome)>,
    hold: Option<(u64, f64)>,
}

/// After this time over a collapsed section it opens.
const HOLD_OPEN: f64 = 0.6;

impl App {
    /// What the drop does.
    fn describe_drop(&self, dragged: Dragged, drop: Option<DropAt>) -> Outcome {
        let prj = &self.session.prj;
        match dragged {
            Dragged::Page(p) => {
                let pg = prj.pages.get(p).ok_or_else(|| tr("No such page").to_string())?;
                if pg.locked {
                    return Err(tr("Released pages cannot be moved").to_string());
                }
                let holder = self.remote.shared.as_ref().and_then(|sh| sh.holder(openecraft_core::collab::Target::Page(pg.id)));
                if let Some(l) = holder {
                    return Err(tr_fmt!("Locked by {} on the project server – read only", l.user));
                }
            }
            Dragged::Section(id) => {
                if sections::get(prj, id).is_none() {
                    return Err(tr("No such section").to_string());
                }
            }
        }
        let Some(drop) = drop else {
            return Ok(Some(if prj.sections.is_empty() { tr("Drop between the pages") } else { tr("Drop between the rows or onto a section") }.to_string()));
        };
        if unchanged(prj, dragged, drop) {
            return Ok(None);
        }
        match (dragged, drop) {
            (Dragged::Page(p), DropAt::Page { section, .. }) => Ok(Some(if sections::of_page(prj, p) == section {
                tr("Changes the page order; the name stays").to_string()
            } else if section == 0 {
                tr_fmt!("Moves to the top level as page {}", sections::moved_name(prj, p, 0))
            } else {
                tr_fmt!("Moves to {} as page {}", sections::title(prj, section), sections::moved_name(prj, p, section))
            })),
            (Dragged::Section(id), DropAt::Section { parent, before }) => {
                let number = sections::number_after_move(prj, id, parent, before).map_err(|e| tr(e).to_string())?;
                let mut text = tr_fmt!("Becomes section {}", number);
                let ids = sections::subtree(prj, id);
                if prj.pages.iter().any(|p| p.locked && ids.contains(&p.section)) {
                    text = format!("{text}\n{}", tr("Released pages in it keep their names"));
                }
                Ok(Some(text))
            }
            _ => Ok(None),
        }
    }

    /// Shows the drop of the page list (marker, label at the pointer, cursor), opens a
    /// section held over and carries out the drop when the button is released.
    pub(super) fn page_drop(&mut self, ui: &mut egui::Ui, probe: &Probe) -> Option<sections_ui::SectionAction> {
        let ctx = ui.ctx().clone();
        let Some(dragged) = probe.dragged else {
            self.page_drag = DragState::default();
            return None;
        };
        let now = ctx.input(|i| i.time);
        // Opening a collapsed section the drag is held over.
        match (probe.over_section, self.page_drag.hold) {
            (Some((id, header)), Some((held, since))) if id == held => {
                let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(&ctx, header, true);
                if now - since < HOLD_OPEN {
                    ctx.request_repaint_after(std::time::Duration::from_secs_f64(HOLD_OPEN - (now - since)));
                } else if !state.is_open() {
                    state.set_open(true);
                    state.store(&ctx);
                    ctx.request_repaint();
                }
            }
            (Some((id, _)), _) => self.page_drag.hold = Some((id, now)),
            (None, _) => self.page_drag.hold = None,
        }
        let drop = probe.target.map(|(d, _)| d);
        let text = match &self.page_drag.described {
            Some((d, t, text)) if *d == dragged && *t == drop => text.clone(),
            _ => {
                let text = self.describe_drop(dragged, drop);
                self.page_drag.described = Some((dragged, drop, text.clone()));
                text
            }
        };
        let valid = drop.is_some() && matches!(text, Ok(Some(_)));
        let color = ui.visuals().selection.stroke.color;
        if let (true, Some((_, mark))) = (valid, probe.target) {
            let painter = ui.painter();
            match mark {
                Mark::Line { y, left } => {
                    let right = ui.clip_rect().right() - 2.0;
                    painter.hline(left..=right, y, egui::Stroke::new(2.0, color));
                    painter.circle_stroke(egui::pos2(left, y), 3.0, egui::Stroke::new(1.5, color));
                }
                Mark::Into(rect) => {
                    painter.rect_filled(rect, 2.0, ui.visuals().selection.bg_fill.gamma_multiply(0.4));
                    painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.5, color), egui::StrokeKind::Inside);
                }
            }
        }
        if text.is_err() || probe.refused {
            ctx.set_cursor_icon(egui::CursorIcon::NoDrop);
        }
        self.drag_label(&ctx, dragged, &text, probe.refused);
        if !ctx.input(|i| i.pointer.primary_released()) {
            return None;
        }
        egui::DragAndDrop::clear_payload(&ctx);
        self.page_drag = DragState::default();
        match (valid && !probe.refused, dragged, drop?) {
            (true, Dragged::Page(p), DropAt::Page { section, before }) => Some(sections_ui::SectionAction::MovePageTo(p, section, before)),
            (true, Dragged::Section(id), DropAt::Section { parent, before }) => Some(sections_ui::SectionAction::MoveSectionTo(id, parent, before)),
            _ => None,
        }
    }

    /// The label under the pointer: what is dragged and what the drop does.
    fn drag_label(&self, ctx: &egui::Context, dragged: Dragged, text: &Outcome, refused: bool) {
        let Some(pos) = ctx.pointer_latest_pos() else { return };
        let prj = &self.session.prj;
        let title = match dragged {
            Dragged::Page(p) => prj.pages.get(p).map(|pg| format!("{} {}   {}", openecraft_view::page_icon(pg.kind), structure::page_full_name(pg), pg.description)).unwrap_or_default(),
            Dragged::Section(id) => sections::title(prj, id),
        };
        egui::Area::new(egui::Id::unique("page_drag_label")).order(egui::Order::Tooltip).fixed_pos(pos + egui::vec2(16.0, 12.0)).interactable(false).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(320.0);
                ui.label(RichText::new(title).strong());
                match (text, refused) {
                    (Err(e), _) => {
                        ui.colored_label(crate::style::error(), e);
                    }
                    (_, true) => {
                        ui.colored_label(crate::style::error(), tr("A section cannot be moved into itself"));
                    }
                    (Ok(Some(t)), _) => {
                        ui.label(t);
                    }
                    (Ok(None), _) => {}
                }
            });
        });
    }
}

#[cfg(test)]
mod tests;
