//! View › Workspaces: the built-in workspaces in their default arrangement, and saving
//! and restoring the arrangement of the editor (workspace, panes and ribbon).

use super::*;
use crate::workspaces::{self, Layout};

impl App {
    fn current_layout(&mut self, name: &str) -> Layout {
        let left_tab = crate::dock::active_tab(&self.dock).unwrap_or(LeftTab::Pages).key().to_string();
        let ribbon = if self.ribbon.custom == crate::ribbon::Custom::default() { String::new() } else { self.ribbon.custom.to_json() };
        let pane_windows = if self.shell.windows.is_empty() { String::new() } else { self.shell.windows.to_json() };
        Layout { name: name.into(), workspace: self.workspace.key().to_string(), left_tab, windows: Vec::new(), dock: crate::dock::to_json(&self.dock), pane_windows, ribbon }
    }

    fn apply_layout(&mut self, layout: &Layout) {
        if let Some(w) = Workspace::from_key(&layout.workspace) {
            self.set_workspace(w);
        }
        if let Some(tree) = crate::dock::from_json(&layout.dock) {
            self.dock = tree;
            self.shell.windows = crate::pane_windows::PaneWindows::from_json(&layout.pane_windows, &self.dock);
            self.window_placed.clear();
        }
        if let Some(t) = LeftTab::from_key(&layout.left_tab) {
            self.show_tab(t);
        }
        // Workspaces of the earlier editor list the windows that were open.
        for t in workspaces::window_panes(layout) {
            self.show_pane(Pane::Tool(t));
        }
        if let Ok(c) = crate::ribbon::Custom::from_json(&layout.ribbon) {
            self.ribbon.custom = c;
        }
    }

    /// Switches to a built-in workspace in its default arrangement, whatever was
    /// changed in it before.
    pub(super) fn restore_workspace(&mut self, w: Workspace) {
        self.set_workspace(w);
        self.reset_layout();
    }

    /// The standard and the saved workspaces (View › Workspaces).
    pub(super) fn workspaces_menu(&mut self, ui: &mut egui::Ui) {
        let mut standard = None;
        let mut apply = None;
        let mut confirm = None;
        ui.weak(tr("Standard workspaces"));
        for w in Workspace::ALL {
            let r = ui.add(egui::Button::selectable(w == self.workspace, tr(w.title())));
            if r.on_hover_text(tr("Switches to this workspace with its panes at their default places")).clicked() {
                standard = Some(w);
            }
        }
        ui.separator();
        ui.weak(tr("Saved workspaces"));
        for l in &self.layouts {
            ui.horizontal(|ui| {
                let ws = Workspace::from_key(&l.workspace).map(|w| tr(w.title())).unwrap_or("");
                if ui.button(&l.name).on_hover_text(ws).clicked() {
                    apply = Some(l.clone());
                }
                if icons::button(ui, Icon::Close, tr("Delete this workspace")).clicked() {
                    confirm = Some(l.name.clone());
                }
            });
        }
        if self.layouts.is_empty() {
            ui.weak(tr("No saved workspaces yet."));
        }
        ui.separator();
        if ui.button(tr("Save current…")).on_hover_text(tr("Saves the workspace, the arrangement of the panels and the open windows")).clicked() {
            self.layout_name = Some(String::new());
            ui.close();
        }
        if let Some(w) = standard {
            self.restore_workspace(w);
            ui.close();
        }
        if let Some(l) = apply {
            self.apply_layout(&l);
            ui.close();
        }
        if confirm.is_some() {
            // Asked in a dialog (`remove_layout_window`): the menu closes.
            self.confirm_remove_layout = confirm;
            ui.close();
        }
    }

    /// Asks before a saved workspace is deleted (there is no undo for it).
    pub(super) fn remove_layout_window(&mut self, ctx: &egui::Context) {
        let Some(name) = self.confirm_remove_layout.clone() else { return };
        let mut answer = None;
        egui::Modal::new(egui::Id::unique("confirm_remove_layout")).show(ctx, |ui| {
            ui.label(tr_fmt!("Delete the workspace {}?", name));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(tr("Delete")).clicked() {
                    answer = Some(true);
                }
                if ui.button(tr("Cancel")).clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    answer = Some(false);
                }
            });
        });
        if let Some(delete) = answer {
            if delete {
                workspaces::remove(&mut self.layouts, &name);
            }
            self.confirm_remove_layout = None;
        }
    }

    /// The small window that asks for the name of the workspace to save.
    pub(super) fn save_layout_window(&mut self, ctx: &egui::Context) {
        let Some(mut name) = self.layout_name.take() else { return };
        let (window, esc) = self.dialogs.window(ctx, tr("Save workspace"), "save_workspace", None);
        let mut open = !esc;
        let mut save = false;
        let mut cancel = false;
        window.open(&mut open).resizable(false).show(ctx, |ui| {
            ui.horizontal(|ui| {
                let r = ui.add(egui::TextEdit::singleline(&mut name).hint_text(tr("name")).desired_width(180.0));
                let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if name.is_empty() && !r.has_focus() {
                    r.request_focus();
                }
                save = (ui.add_enabled(!name.trim().is_empty(), egui::Button::new(tr("Save"))).clicked() || enter) && !name.trim().is_empty();
                cancel = ui.button(tr("Cancel")).clicked();
            });
        });
        if save {
            let layout = self.current_layout(&name);
            workspaces::save(&mut self.layouts, layout);
        } else if open && !cancel {
            self.layout_name = Some(name);
        }
    }
}
