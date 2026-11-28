//! Cross selection with the schematic (`openecraft_core::cross_select`): what the panel
//! layout and its 3D view have selected, and selecting there what the schematic has.

use super::*;
use openecraft_core::cross_select::{self, Deleted, LayoutPick};

impl PanelUi {
    /// What is selected (items of the current enclosure, a wire, a cable), as the
    /// schematic knows it. A route says by its key what it is (wires `from|to`, tubes
    /// `hose:…`, cables their tag), also while the routing is searched again.
    pub fn pick(&self) -> LayoutPick {
        let mut pick = LayoutPick { items: self.selection.iter().map(|id| (self.enclosure, *id)).collect(), ..Default::default() };
        if let Some(w) = &self.selected_wire {
            pick.wires.push(w.clone());
        }
        match self.selected_route.as_deref() {
            Some(key) if key.starts_with("hose:") => {}
            Some(key) => match key.split_once('|') {
                Some((from, to)) => pick.wires.push((from.to_string(), to.to_string())),
                None => pick.cables.push(key.to_string()),
            },
            None => {}
        }
        pick
    }

    /// Takes out of the selection what is no longer there (deleted in another view or
    /// undone): items, the wire and the route.
    pub fn drop_missing(&mut self, prj: &Project, an: &Analysis) {
        let e = prj.layout.enclosures.get(self.enclosure);
        self.selection.retain(|id| e.is_some_and(|e| e.item(*id).is_some()));
        if self.selected_wire.as_ref().is_some_and(|k| !an.wiring.iter().any(|w| &w.key() == k)) {
            self.selected_wire = None;
        }
        if !an.routes_pending && self.selected_route.as_deref().is_some_and(|k| an.routing.item(k).is_none()) {
            self.selected_route = None;
            self.control_drag = None;
        }
    }

    /// Shows enclosure `ei`; the camera stays where it is if it is the current one.
    fn bring_enclosure(&mut self, prj: &Project, ei: usize) {
        self.switch_enclosure(ei);
        self.in_room = prj.layout.enclosures.get(ei).is_some_and(|e| e.is_room());
    }

    /// Selects what the schematic has selected: its devices and terminals (those in the
    /// current enclosure, otherwise those in the enclosure of the first one; the 2D
    /// view shows its face), otherwise its first wire, otherwise its first cable;
    /// nothing for an empty pick. True when something is selected.
    pub fn cross_select(&mut self, prj: &Project, an: &Analysis, pick: &LayoutPick) -> bool {
        self.deselect();
        let here = pick.items.iter().find(|i| i.0 == self.enclosure).or(pick.items.first()).copied();
        if let Some((ei, first)) = here.filter(|(ei, _)| *ei < prj.layout.enclosures.len()) {
            self.bring_enclosure(prj, ei);
            let e = &prj.layout.enclosures[ei];
            self.selection = pick.items.iter().filter(|i| i.0 == ei && e.item(i.1).is_some()).map(|i| i.1).collect();
            if let Some(it) = e.item(first).filter(|it| self.view != View::ThreeD && it.face != self.face) {
                self.face = it.face;
                self.fit_pending = true;
            }
            return !self.selection.is_empty();
        }
        if let Some(w) = pick.wires.iter().find_map(|k| an.wiring.iter().find(|w| &w.key() == k)) {
            let key = w.key();
            self.status = tr_fmt!("Wire {} – {}  {} mm", w.from, w.to, format!("{:.0}", w.length));
            self.bring_enclosure(prj, w.enclosure);
            self.select_wire(key);
            return true;
        }
        if let Some(it) = pick.cables.iter().find_map(|tag| an.routing.item(tag)) {
            let key = it.key.clone();
            if let Some(&ei) = it.enclosures.first().filter(|ei| !self.shown_3d(prj, self.enclosure).iter().any(|s| s.0 == **ei)) {
                self.bring_enclosure(prj, ei);
            }
            self.select_route(an, &key);
            return true;
        }
        false
    }

    /// The menu entry Show in schematic, for what is selected.
    pub(super) fn schematic_entry(&mut self, ui: &mut egui::Ui) {
        let def = crate::commands::def(Cmd::ShowInSchematic);
        let keys = crate::shortcuts::get(ui.ctx()).cmd_text(Cmd::ShowInSchematic);
        if ui.add_enabled(!self.pick().is_empty(), egui::Button::new(tr(def.title)).shortcut_text(keys)).on_hover_text(tr(def.tip)).clicked() {
            self.show_schematic = true;
            ui.close();
        }
    }

    /// Deletes the selected items of the current enclosure from the whole project: a
    /// device with its functions on all pages, its places and its device data
    /// (`cross_select::delete_completely`).
    pub fn delete_selection_completely(&mut self, prj: &mut Project) -> Deleted {
        // Taken first: finding the current enclosure may switch the face and clear it.
        let ids: Vec<u64> = self.selection.iter().copied().collect();
        let Some(ei) = self.enclosure_index(prj) else { return Deleted::default() };
        let items: Vec<(usize, u64)> = ids.into_iter().map(|id| (ei, id)).collect();
        let done = cross_select::delete_completely(prj, &items, &[]);
        self.selection.clear();
        self.status = Self::deleted_text(done);
        done
    }

    /// What Delete completely did, for the status line.
    pub fn deleted_text(done: Deleted) -> String {
        if done.kept > 0 {
            tr_fmt!("Deleted completely: {} items of the layout, {} functions on the pages; {} functions stay on pages that cannot be edited", done.items, done.symbols, done.kept)
        } else {
            tr_fmt!("Deleted completely: {} items of the layout, {} functions on the pages", done.items, done.symbols)
        }
    }

    /// The menu entry Delete completely, for the selected items.
    pub(super) fn delete_completely_entry(&mut self, ui: &mut egui::Ui, prj: &mut Project) {
        let def = crate::commands::def(Cmd::DeleteCompletely);
        let keys = crate::shortcuts::get(ui.ctx()).cmd_text(Cmd::DeleteCompletely);
        if ui.add_enabled(!self.selection.is_empty(), egui::Button::new(tr(def.title)).shortcut_text(keys)).on_hover_text(tr(def.tip)).clicked() {
            self.delete_selection_completely(prj);
            ui.close();
        }
    }

    /// Shows what the schematic has selected, as [`PanelUi::cross_select`] does, in the
    /// 3D view (`three_d`) or the 2D view; devices with the camera close to the first
    /// one. True when something is shown.
    pub fn show_pick(&mut self, prj: &Project, an: &Analysis, pick: &LayoutPick, three_d: bool) -> bool {
        let device = pick.items.first().and_then(|(ei, id)| prj.layout.enclosures.get(*ei)?.item(*id)).filter(|it| it.kind == ItemKind::Device).map(|it| it.tag.clone());
        if let Some(tag) = device {
            if self.focus_device(prj, &tag, three_d) {
                // All of the pick in the enclosure of the device focused.
                let ids: BTreeSet<u64> = pick.items.iter().filter(|i| i.0 == self.enclosure).map(|i| i.1).collect();
                if !ids.is_empty() {
                    self.selection = ids;
                }
                return true;
            }
        }
        if !self.cross_select(prj, an, pick) {
            return false;
        }
        if three_d {
            self.view = View::ThreeD;
        } else if self.view == View::ThreeD {
            self.view = View::Front;
        }
        true
    }
}

#[cfg(test)]
mod tests;
