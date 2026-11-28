//! Panes in windows of their own, which the user moves to another monitor, maximizes
//! or shows full screen (the 3D view beside the schematic on a second monitor). Each
//! window has its own tile tree, so it holds several panes as tabs or side by side. A
//! pane is in one place only: the main window's tree or one of the windows. The page
//! stays in the main window with the ribbon. The windows belong to their workspace
//! like its tree (`crate::dock::Side`). No UI code: `app/pane_windows_ui.rs` draws them.

use crate::dock::{self, Pane};
use egui_tiles::{Tile, Tree};
use serde::{Deserialize, Serialize};

/// A window with panes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaneWindow {
    /// Number of the window, for its viewport and its title ("window 2").
    pub id: u32,
    pub tree: Tree<Pane>,
    /// Where the window was last: outer position and inner size in points
    /// (`[x, y, width, height]`), `None` until it was shown.
    #[serde(default)]
    pub rect: Option<[f32; 4]>,
    #[serde(default)]
    pub maximized: bool,
    #[serde(default)]
    pub fullscreen: bool,
}

impl PaneWindow {
    fn new(id: u32, pane: Pane) -> PaneWindow {
        PaneWindow { id, tree: Tree::new_tabs(egui::Id::unique(("pane_window", id)), vec![pane]), rect: None, maximized: false, fullscreen: false }
    }

    pub fn panes(&self) -> Vec<Pane> {
        dock::panes(&self.tree)
    }
}

/// The windows of a workspace.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaneWindows {
    pub list: Vec<PaneWindow>,
}

impl PaneWindows {
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// The window a pane is in.
    pub fn find(&self, pane: Pane) -> Option<u32> {
        self.list.iter().find(|w| dock::find(&w.tree, pane).is_some()).map(|w| w.id)
    }

    pub fn get(&self, id: u32) -> Option<&PaneWindow> {
        self.list.iter().find(|w| w.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut PaneWindow> {
        self.list.iter_mut().find(|w| w.id == id)
    }

    /// The panes of all windows.
    pub fn panes(&self) -> Vec<Pane> {
        self.list.iter().flat_map(PaneWindow::panes).collect()
    }

    /// The ids of the windows.
    pub fn ids(&self) -> Vec<u32> {
        self.list.iter().map(|w| w.id).collect()
    }

    /// A pane can go to a window of its own unless it is the page (it stays with the
    /// ribbon) or the last document of the main window.
    pub fn can_move(main: &Tree<Pane>, pane: Pane) -> bool {
        pane != Pane::Page && (dock::find(main, pane).is_none() || dock::can_close(main, pane))
    }

    /// Takes a pane out of the main tree or out of its window (a window left empty
    /// closes).
    fn detach(&mut self, main: &mut Tree<Pane>, pane: Pane) {
        if let Some(id) = dock::find(main, pane) {
            main.remove_recursively(id);
        }
        for w in &mut self.list {
            if let Some(id) = dock::find(&w.tree, pane) {
                w.tree.remove_recursively(id);
            }
        }
        self.list.retain(|w| !w.panes().is_empty());
    }

    /// Opens a pane in a new window; the window's id, `None` if the pane cannot move.
    pub fn open(&mut self, main: &mut Tree<Pane>, pane: Pane) -> Option<u32> {
        if !Self::can_move(main, pane) {
            return None;
        }
        // A pane alone in its window has that window already.
        if let Some(w) = self.find(pane).and_then(|id| self.get(id)) {
            if w.panes() == [pane] {
                return Some(w.id);
            }
        }
        self.detach(main, pane);
        let id = self.list.iter().map(|w| w.id).max().unwrap_or(1) + 1;
        self.list.push(PaneWindow::new(id, pane));
        Some(id)
    }

    /// Moves a pane into the window `id`, as a tab where panes of its kind are.
    pub fn move_to(&mut self, main: &mut Tree<Pane>, pane: Pane, id: u32) -> bool {
        if self.get(id).is_none() || self.find(pane) == Some(id) || !Self::can_move(main, pane) {
            return false;
        }
        self.detach(main, pane);
        // The window may have closed because the pane was its last one.
        match self.get_mut(id) {
            Some(w) => dock::show(&mut w.tree, pane),
            None => self.list.push(PaneWindow::new(id, pane)),
        }
        true
    }

    /// Puts a pane of a window back into the main window.
    pub fn back(&mut self, main: &mut Tree<Pane>, pane: Pane) {
        if self.find(pane).is_some() {
            self.detach(main, pane);
            dock::show(main, pane);
        }
    }

    /// Closes the window `id`: its panes go back into the main window, so that none is
    /// lost.
    pub fn close(&mut self, main: &mut Tree<Pane>, id: u32) {
        let Some(i) = self.list.iter().position(|w| w.id == id) else { return };
        let w = self.list.remove(i);
        let shown = dock::visible(&w.tree);
        for p in w.panes() {
            dock::show(main, p);
        }
        // The panes seen in the window are the ones seen after it closed.
        for p in shown {
            main.make_active(|_, tile| matches!(tile, Tile::Pane(q) if *q == p));
        }
    }

    /// Closes every window (Reset pane layout, Gather windows of a missing monitor).
    pub fn close_all(&mut self, main: &mut Tree<Pane>) {
        for id in self.ids() {
            self.close(main, id);
        }
    }

    /// Closes a pane in its window (the close button of its tab); a window left empty
    /// closes.
    pub fn remove(&mut self, pane: Pane) {
        let mut scratch = Tree::empty(egui::Id::unique("pane_windows_scratch"));
        self.detach(&mut scratch, pane);
    }

    /// Makes the read windows consistent with the main tree: the page and panes
    /// docked in the main window or in an earlier window are dropped from them, empty
    /// windows and windows with the same id are dropped.
    pub fn tidy(&mut self, main: &Tree<Pane>) {
        let mut seen: Vec<Pane> = dock::panes(main);
        let mut ids = Vec::new();
        self.list.retain(|w| {
            let fresh = !ids.contains(&w.id);
            ids.push(w.id);
            fresh
        });
        for w in &mut self.list {
            for p in w.panes() {
                if p == Pane::Page || seen.contains(&p) {
                    if let Some(id) = dock::find(&w.tree, p) {
                        w.tree.remove_recursively(id);
                    }
                } else {
                    seen.push(p);
                }
            }
        }
        self.list.retain(|w| w.tree.root().is_some() && !w.panes().is_empty());
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Saved windows, tidied against the main tree; none if they cannot be read.
    pub fn from_json(text: &str, main: &Tree<Pane>) -> PaneWindows {
        let mut w: PaneWindows = serde_json::from_str(text).unwrap_or_default();
        w.tidy(main);
        w
    }
}

#[cfg(test)]
mod tests;
