//! Cross selection between the page and the panel layout (`openecraft_core::cross_select`):
//! Cross-select mode keeps the selections of the schematic, the panel layout and its
//! 3D view in step, in both directions and across the windows of panes; Show in
//! schematic, Show in panel layout and Show in 3D view go to the counterpart, and a
//! double-click does where the user chose it. What one view deletes is taken out of the
//! selection of the others at once.

use super::*;
use crate::panel_ui::View;
use openecraft_core::cross_select::{self, LayoutPick};

/// The page shown (its id) and what is selected on it.
type PageSelection = (u64, Vec<u64>);

/// The selections of the page and of the panel layout.
type Selections = (PageSelection, LayoutPick);

/// The selections of the views when they were last compared.
#[derive(Default)]
pub(super) struct CrossState {
    /// What the selection of the page is in the panel layout, with what it was worked
    /// out for: the page, the selection, the analysis.
    page_pick: Option<((PageSelection, u64), LayoutPick)>,
    /// The selections as last kept in step: the page (its id, the selection) and the
    /// panel layout.
    last: Option<Selections>,
    /// The project revision the selections were last checked against.
    rev: u64,
}

/// Which of the two views changed its selection since they were last in step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Changed {
    Nothing,
    Page,
    Layout,
}

/// What changed between `last` and now: a selection of the page (another page shown with
/// nothing selected is no change), otherwise the one of the panel layout; with nothing
/// compared yet the page's selection, if any.
pub(super) fn changed(last: Option<&Selections>, page: &PageSelection, layout: &LayoutPick) -> Changed {
    match last {
        None if page.1.is_empty() => Changed::Nothing,
        None => Changed::Page,
        Some((p, _)) if p != page && (p.0 == page.0 || !page.1.is_empty()) => Changed::Page,
        Some((_, l)) if l != layout => Changed::Layout,
        _ => Changed::Nothing,
    }
}

impl App {
    /// The page shown (its id) and what is selected on it.
    fn page_selection(&self) -> PageSelection {
        let id = self.session.prj.pages.get(self.session.page).map_or(0, |p| p.id);
        (id, self.editor.selection.iter().copied().collect())
    }

    /// What the selection of the page is in the panel layout, worked out again when the
    /// selection or the analysis changed.
    pub(super) fn page_pick(&mut self) -> LayoutPick {
        let key = (self.page_selection(), self.session.analysis.id);
        if let Some((k, pick)) = &self.cross.page_pick {
            if *k == key {
                return pick.clone();
            }
        }
        let pick = cross_select::in_layout(&self.session.prj, &self.session.analysis, self.session.page, &self.editor.selection);
        self.cross.page_pick = Some((key, pick.clone()));
        pick
    }

    /// The selection of the page has a counterpart in the panel layout (as last worked
    /// out by [`App::page_pick`]).
    pub(super) fn page_pick_shown(&self) -> bool {
        self.cross.page_pick.as_ref().is_some_and(|(_, p)| !p.is_empty())
    }

    /// The layout view in front: where it is seen (beside the page, in a window of its
    /// own), otherwise in the panel layout workspace.
    pub(super) fn bring_layout_view(&mut self) {
        if crate::dock::find(&self.dock, Pane::LayoutView).is_none() && self.shell.windows.find(Pane::LayoutView).is_none() {
            self.set_workspace(Workspace::Panel);
        }
        self.show_pane(Pane::LayoutView);
    }

    /// Show in panel layout / Show in 3D view: what is selected on the page.
    pub(super) fn show_selection_in_layout(&mut self, three_d: bool) {
        let pick = self.page_pick();
        if self.panel_ui.show_pick(&self.session.prj, &self.session.analysis, &pick, three_d) {
            self.bring_layout_view();
            self.cross_in_step();
        }
    }

    /// Show in schematic: what is selected in the panel layout, on the current page if
    /// it is there, otherwise on the page of its main function.
    pub(super) fn show_selection_in_schematic(&mut self, ctx: &egui::Context) {
        let pick = self.panel_ui.pick();
        let found = cross_select::in_schematic(&self.session.prj, &self.session.analysis, &pick);
        match cross_select::on_one_page(&found, self.session.page) {
            Some((page, ids)) => self.goto_items(page, ids),
            None => self.notify(ctx, tr("Not in the schematic").to_string()),
        }
    }

    /// Delete completely: the selection of the panel layout while it is the document
    /// worked in, otherwise the one of the page (its devices with their functions on
    /// all pages and their places; the other items of the page as Delete does).
    pub(super) fn delete_completely(&mut self, ctx: &egui::Context) {
        let done = if self.active_doc == Pane::LayoutView {
            self.panel_ui.delete_selection_completely(&mut self.session.prj)
        } else {
            let page = self.session.page;
            let sel: Vec<(usize, u64)> = self.editor.selection.iter().map(|id| (page, *id)).collect();
            let items = self.editor.items(&self.session.prj, page);
            let done = cross_select::delete_completely(&mut self.session.prj, &[], &sel);
            if self.page_editable() {
                openecraft_core::ops::delete(&mut self.session.prj.pages[page], &items);
            }
            self.editor.selection.clear();
            done
        };
        self.notify(ctx, crate::panel_ui::PanelUi::deleted_text(done));
    }

    /// Shows page `page` with the items `ids` selected, the first one in the middle.
    pub(super) fn goto_items(&mut self, page: usize, ids: Vec<u64>) {
        if self.in_dock {
            self.deferred.push(Deferred::Select(page, ids));
            return;
        }
        self.goto(page, ids.first().copied().unwrap_or(0));
        self.editor.selection.extend(ids);
        self.cross_in_step();
    }

    /// The selections as they are now count as in step: what Show in … selected in the
    /// other view does not come back to the view it was asked in.
    fn cross_in_step(&mut self) {
        if self.cross.last.is_some() {
            self.cross.last = Some((self.page_selection(), self.panel_ui.pick()));
        }
    }

    /// A double-click on the page that would open the properties: shows the selection in
    /// the panel layout instead where the user chose it (and it is placed there).
    pub(super) fn double_click_shows_layout(&mut self, ctx: &egui::Context) -> bool {
        if !self.shell.double_click_other || self.page_pick().is_empty() {
            return false;
        }
        let cmd = if self.panel_ui.view == View::ThreeD { Cmd::ShowIn3d } else { Cmd::ShowInLayout };
        self.run(ctx, cmd);
        true
    }

    /// After the panes: what was deleted is taken out of the selection of the panel
    /// layout, and in Cross-select mode the selection that changed is taken over by the
    /// other view.
    pub(super) fn cross_select_frame(&mut self) {
        if self.cross.rev != self.session.history.rev {
            self.cross.rev = self.session.history.rev;
            self.panel_ui.drop_missing(&self.session.prj, &self.session.analysis);
        }
        if !self.shell.cross_select || self.closed {
            self.cross.last = None;
            return;
        }
        // Not while something is being drawn, dragged or moved.
        if self.editor.busy() || self.panel_ui.busy() {
            return;
        }
        let page = self.page_selection();
        let layout = self.panel_ui.pick();
        match changed(self.cross.last.as_ref(), &page, &layout) {
            Changed::Page => {
                let pick = self.page_pick();
                self.panel_ui.cross_select(&self.session.prj, &self.session.analysis, &pick);
            }
            Changed::Layout => {
                let found = cross_select::in_schematic(&self.session.prj, &self.session.analysis, &layout);
                match cross_select::on_one_page(&found, self.session.page) {
                    Some((pi, ids)) => {
                        let other = pi != self.session.page;
                        self.session.page = pi;
                        self.editor.select_items(&self.session.prj, pi, &ids, other);
                    }
                    None => self.editor.selection.clear(),
                }
            }
            Changed::Nothing => {}
        }
        self.cross.last = Some((self.page_selection(), self.panel_ui.pick()));
    }
}

#[cfg(test)]
mod tests;
