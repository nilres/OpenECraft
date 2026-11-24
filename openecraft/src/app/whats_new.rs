//! What is new in OpenECraft: the version history of `CHANGELOG.md` up to this version
//! (Help › Version history: one page per version, grouped by series) and the window
//! that shows the new features once, at the first start after an update. The changes
//! are written in English, as the changelog is.

use super::*;
use openecraft_parts::changelog::{self, Release};
use std::sync::OnceLock;

const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");
pub(super) const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The versions up to this one, newest first.
pub(super) fn history() -> &'static [Release] {
    static HISTORY: OnceLock<Vec<Release>> = OnceLock::new();
    HISTORY.get_or_init(|| changelog::history(CHANGELOG, VERSION))
}

/// The versions (indices of [`history`]) the window shows at start: none at the first
/// start of all, else those since the version used last (`seen`; empty: a version
/// before the ones that remember it).
pub(super) fn at_start(first_start: bool, seen: &str) -> Vec<usize> {
    if first_start {
        return Vec::new();
    }
    changelog::new_since(history(), (!seen.is_empty()).then_some(seen), VERSION)
}

/// The name of a version in the history: `0.1.3 – 2026-11-23`.
pub(super) fn title(release: &Release) -> String {
    match &release.date {
        Some(date) => format!("{} – {date}", release.version),
        None => tr_fmt!("{} (in development)", release.version),
    }
}

/// The changes of a version, rendered.
pub(super) fn changes(ui: &mut egui::Ui, release: &Release) {
    let lines: Vec<&str> = release.text.lines().collect();
    super::help_pane::markdown(ui, &lines);
}

/// Shown when the editor does not speak English: the changes are written in English.
pub(super) fn english_note(ui: &mut egui::Ui) {
    if openecraft_i18n::lang() != openecraft_i18n::Lang::En {
        ui.weak(tr("The changes are written in English."));
    }
}

impl App {
    /// The new features of the versions since the one used last, once after an update.
    pub(super) fn whats_new_window(&mut self, ctx: &egui::Context) {
        if self.whats_new.is_empty() {
            return;
        }
        let history = history();
        let (window, esc) = self.dialogs.window(ctx, tr_fmt!("What's new in OpenECraft {}", VERSION), "whats_new", None);
        let mut open = !esc;
        let mut close = false;
        let mut versions = false;
        let height = (ctx.content_rect().height() - 120.0).clamp(240.0, 680.0);
        window.open(&mut open).resizable(true).default_size([640.0, height]).show(ctx, |ui| {
            ui.label(if self.whats_new.len() == 1 {
                tr_fmt!("OpenECraft was updated to version {}. These are its new features:", VERSION)
            } else {
                tr_fmt!("OpenECraft was updated to version {}. These are the new features of the versions since the one you used last:", VERSION)
            });
            english_note(ui);
            ui.separator();
            // The rest of the window but the row of buttons below.
            let buttons = ui.spacing().interact_size.y + 3.0 * ui.spacing().item_spacing.y + 6.0;
            let rest = (ui.available_height() - buttons).max(80.0);
            egui::ScrollArea::vertical().id_salt("whats_new").max_height(rest).auto_shrink([false, false]).show(ui, |ui| {
                for &i in &self.whats_new {
                    let Some(release) = history.get(i) else { continue };
                    ui.heading(tr_fmt!("OpenECraft {}", title(release)));
                    changes(ui, release);
                    ui.add_space(8.0);
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                versions = ui.button(tr("Version history")).clicked();
                close = ui.button(tr("Close")).clicked();
            });
        });
        if versions {
            let newest = self.whats_new.first().copied();
            self.help_version(newest);
        }
        if !open || close || versions {
            self.whats_new.clear();
        }
    }
}

#[cfg(test)]
mod tests;
