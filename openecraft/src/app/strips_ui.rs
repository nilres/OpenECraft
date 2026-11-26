//! Creation of terminal strips and unplaced terminals from the pane and navigator.

use super::*;

pub(super) struct Draft {
    tag: String,
    focus: bool,
}

pub(super) fn creation_menu(ui: &mut egui::Ui, strip: bool, terminal: bool) -> Option<Cmd> {
    let mut picked = None;
    for (cmd, enabled) in [(Cmd::NewTerminalStrip, strip), (Cmd::NewTerminal, terminal)] {
        if ui.add_enabled(enabled, egui::Button::new(tr(crate::commands::def(cmd).title))).clicked() {
            picked = Some(cmd);
            ui.close();
        }
    }
    picked
}

pub(super) fn add_menu(ui: &mut egui::Ui, strip: bool, terminal: bool) -> Option<Cmd> {
    let mut picked = None;
    let response = ui.menu_button(tr("Add New"), |ui| picked = creation_menu(ui, strip, terminal)).response;
    response.context_menu(|ui| {
        picked = creation_menu(ui, strip, terminal);
        ui.separator();
        super::dock_ui::placeholder(ui);
    });
    picked
}

impl App {
    pub(super) fn selected_strip(&self) -> String {
        let tags = openecraft_core::strips::strip_tags(&self.session.prj);
        if tags.contains(&self.eng.strip) { self.eng.strip.clone() } else { tags.first().cloned().unwrap_or_default() }
    }

    pub(super) fn new_terminal_strip(&mut self) {
        self.eng.new_strip = Some(Draft { tag: self.session.prj.next_tag("", "X"), focus: true });
    }

    fn run_strip_creation(&mut self, ctx: &egui::Context, name: &str, tag: &str) -> bool {
        let Some(command) = self.plugins.command(name) else { return false };
        let args = serde_json::Map::from_iter([("strip".to_string(), serde_json::Value::String(tag.to_string()))]);
        match self.plugins.run(command, &mut self.session.prj, &args) {
            Ok(_) => {
                self.eng.strip = tag.trim().to_string();
                self.unplaced_cache = Default::default();
                self.show_pane(Pane::Tool(ToolPane::TerminalStrips));
                true
            }
            Err(error) => {
                self.notify_error(ctx, error);
                false
            }
        }
    }

    pub(super) fn add_strip_terminal(&mut self, ctx: &egui::Context) {
        let tag = self.selected_strip();
        self.run_strip_creation(ctx, "add_terminal", &tag);
    }

    pub(super) fn new_strip_window(&mut self, ctx: &egui::Context) {
        let enabled = self.cmd_enabled(Cmd::NewTerminalStrip);
        let Some(draft) = &mut self.eng.new_strip else { return };
        let (window, esc) = self.dialogs.window(ctx, tr("New terminal strip"), "new_terminal_strip", None);
        let (mut open, mut create, mut cancel) = (true, false, false);
        window.open(&mut open).resizable(false).show(ctx, |ui| {
            ui.label(tr("Device tag"));
            let field = ui.text_edit_singleline(&mut draft.tag);
            if std::mem::take(&mut draft.focus) { field.request_focus(); }
            let problem = openecraft_core::ops::new_strip_problem(&self.session.prj, &draft.tag);
            if let Some(error) = &problem { ui.colored_label(crate::style::error(), error); }
            let enter = (field.has_focus() || field.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.horizontal(|ui| {
                create = enabled && problem.is_none() && (ui.button(tr("Add New")).clicked() || enter);
                if !enabled || problem.is_some() { ui.add_enabled(false, egui::Button::new(tr("Add New"))); }
                cancel = ui.button(tr("Cancel")).clicked();
            });
        });
        if !open || cancel || esc {
            self.eng.new_strip = None;
        } else if create {
            let tag = self.eng.new_strip.as_ref().unwrap().tag.clone();
            if self.run_strip_creation(ctx, "create_strip", &tag) { self.eng.new_strip = None; }
        }
    }
}

#[cfg(test)]
mod tests;
