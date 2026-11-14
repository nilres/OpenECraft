//! Quick cable renaming from the schematic and the cable navigator.

use super::App;
use crate::remote::Remote;
use openecraft_core::{collab::{self, ProjectRight, Target}, model::*, ops};
use openecraft_i18n::{tr, tr_fmt};
use std::collections::BTreeSet;

pub(super) struct Draft {
    id: u64,
    old: String,
    text: String,
    focus: bool,
    waiting: bool,
    error: Option<String>,
    locks: Option<(String, Vec<Target>)>,
}

impl Draft {
    fn new(cable: &Cable) -> Self {
        Self { id: cable.id, old: cable.tag.clone(), text: cable.tag.clone(), focus: true, waiting: false, error: None, locks: None }
    }

    fn prepare(&self, prj: &Project) -> Result<Project, String> {
        if !prj.cables.iter().any(|c| c.id == self.id && c.tag == self.old) {
            return Err(tr("The cable changed while this dialog was open. Reopen it to rename the cable.").into());
        }
        let mut next = prj.clone();
        ops::rename_project_cable(&mut next, self.id, &self.text)?;
        Ok(next)
    }

    fn cancel(&mut self, remote: &mut Remote) {
        if let Some((session, targets)) = self.locks.take() { remote.cancel_locks(&session, targets); }
    }

    fn request_locks(&mut self, remote: &mut Remote, targets: Vec<Target>) {
        if let Some(sh) = &remote.shared {
            let acquired = targets.iter().copied().filter(|t| !sh.owns(*t) && !sh.requested.contains(t)).collect();
            self.locks = Some((sh.session.session.clone(), acquired));
        }
        remote.lock(targets);
        self.waiting = true;
    }
}

/// Only a single, unambiguous cable-bearing selection is a rename target.
fn selected(prj: &Project, page: usize, ids: &BTreeSet<u64>) -> Option<u64> {
    if ids.len() != 1 { return None; }
    let id = *ids.first()?;
    let pg = prj.pages.get(page)?;
    let tag = pg.cable_defs.iter().find(|c| c.id == id).map(|c| c.cable.as_str())
        .or_else(|| pg.wires.iter().find(|w| w.id == id).map(|w| w.cable.as_str()))
        .or_else(|| pg.symbols.iter().find(|s| s.id == id)?.values.get("function.cable").map(String::as_str))?;
    prj.cable(tag).map(|c| c.id)
}

/// Acquire every changed target before applying a rename, so a rejected page lock
/// cannot leave a renamed inventory with references under the old name.
fn locks(remote: &Remote, before: &Project, after: &Project) -> Result<Vec<Target>, String> {
    let Some(shared) = &remote.shared else { return Ok(Vec::new()); };
    if !remote.online() { return Err(tr("Reconnect to the project server before renaming this cable.").into()); }
    let mut free = Vec::new();
    for target in collab::changes(before, after) {
        let right = match target { Target::Project => ProjectRight::EditProject, Target::Page(_) => ProjectRight::EditPages };
        if !remote.may(right) { return Err(tr("Renaming this cable requires permission to edit the project data and all affected pages.").into()); }
        if let Some(holder) = shared.holder(target) {
            return Err(tr_fmt!("{} (locked by {})", Remote::describe(before, target), holder.user));
        }
        if !shared.owned().contains(target) { free.push(target); }
    }
    Ok(free)
}

impl App {
    pub(super) fn selected_cable(&self) -> Option<u64> {
        selected(&self.session.prj, self.session.page, &self.editor.selection)
    }

    pub(super) fn open_cable_rename(&mut self, id: u64) {
        if let Some(mut draft) = self.rename_cable.take() { draft.cancel(&mut self.remote); }
        self.rename_cable = self.session.prj.cables.iter().find(|c| c.id == id).map(Draft::new);
    }

    pub(super) fn rename_cable_window(&mut self, ctx: &egui::Context) {
        let Some(mut draft) = self.rename_cable.take() else { return; };
        let (window, esc) = self.dialogs.window(ctx, tr("Rename cable"), "rename_cable", None);
        if esc { draft.cancel(&mut self.remote); return; }
        let mut open = true;
        let mut submit = draft.waiting && !self.remote.busy();
        let mut cancel = false;
        window.open(&mut open).default_width(520.0).show(ctx, |ui| {
            ui.label(tr_fmt!("New tag for {}", draft.old));
            let output = ui.add_enabled_ui(!draft.waiting, |ui| {
                egui::TextEdit::singleline(&mut draft.text).desired_width(f32::INFINITY).show(ui)
            }).inner;
            if std::mem::take(&mut draft.focus) {
                output.response.request_focus();
                let mut state = output.state;
                state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(draft.text.chars().count()))));
                state.store(ctx, output.response.id);
            }
            if output.response.changed() { draft.error = None; }
            let problem = ops::cable_rename_problem(&self.session.prj, draft.id, &draft.text);
            if let Some(error) = problem.as_ref().or(draft.error.as_ref()) { ui.colored_label(crate::style::error(), error); }
            if draft.waiting { ui.label(tr("Acquiring locks for the cable rename…")); }
            let enter = (output.response.has_focus() || output.response.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.horizontal(|ui| {
                if !draft.waiting && problem.is_none() {
                    submit |= ui.button(tr("Rename")).clicked() || enter;
                } else {
                    ui.add_enabled(false, egui::Button::new(tr("Rename")));
                }
                cancel = ui.button(tr("Cancel")).clicked();
            });
            if submit || cancel { output.response.surrender_focus(); }
        });
        if !open || cancel { draft.cancel(&mut self.remote); return; }
        if submit {
            let result = draft.prepare(&self.session.prj).and_then(|next| locks(&self.remote, &self.session.prj, &next).map(|free| (next, free)));
            match result {
                Ok((next, free)) if free.is_empty() => {
                    self.session.prj = next;
                    if self.editor.cable_target == draft.old { self.editor.cable_target = draft.text.trim().into(); }
                    return;
                }
                Ok((_, free)) if !draft.waiting => {
                    draft.request_locks(&mut self.remote, free);
                }
                Ok(_) => {
                    draft.cancel(&mut self.remote);
                    draft.waiting = false;
                    draft.error = Some(tr("Could not acquire all locks. The cable was not renamed.").into());
                }
                Err(error) => { draft.cancel(&mut self.remote); draft.waiting = false; draft.error = Some(error); }
            }
        }
        self.rename_cable = Some(draft);
    }
}

#[cfg(test)]
mod tests;
