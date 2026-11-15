//! Report a bug (Help, the quick access row): the form of `openecraft_ui::bug_report`
//! in a dialog, with the technical details of the editor: what is open, how much the
//! project holds and the last messages, but no names or contents of the project.

use super::*;
use openecraft_ui::bug_report::{self, Action};
use std::fmt::Write as _;

/// The last messages of the session a report lists, each cut to `MESSAGE_CHARS`.
const REPORT_MESSAGES: usize = 8;
const MESSAGE_CHARS: usize = 300;

/// What the details say about the open project: counts only.
pub(super) struct ProjectFacts {
    pub pages: usize,
    pub page: usize,
    pub page_kind: String,
    pub devices: usize,
    pub connections: usize,
    pub errors: usize,
    pub warnings: usize,
    pub notes: usize,
    pub unsaved: bool,
    pub shared: bool,
}

/// The editor's part of the technical details (Markdown).
pub(super) fn details(facts: &ProjectFacts, shell: &[(&str, String)], plugins: &[String], messages: &[(crate::style::Level, String)]) -> String {
    let yes = |b: bool| if b { "yes" } else { "no" };
    let mut out = String::new();
    for (name, value) in shell {
        let _ = writeln!(out, "- {name}: {value}");
    }
    let _ = writeln!(out, "- External plugins: {}", if plugins.is_empty() { "none".to_string() } else { plugins.join(", ") });
    let _ = write!(
        out,
        "\n### Project\n- Pages: {} (current: {} of {}, {})\n- Devices: {}\n- Connections: {}\n- Check: {} errors, {} warnings, {} notes\n- Unsaved changes: {}\n- Shared on a project server: {}\n",
        facts.pages,
        facts.page + 1,
        facts.pages,
        facts.page_kind,
        facts.devices,
        facts.connections,
        facts.errors,
        facts.warnings,
        facts.notes,
        yes(facts.unsaved),
        yes(facts.shared),
    );
    let shown = messages.len().min(REPORT_MESSAGES);
    if shown == 0 {
        out.push_str("\n### Messages\nnone\n");
    } else {
        let _ = writeln!(out, "\n### Messages (last {shown} of {})", messages.len());
        for (level, text) in &messages[messages.len() - shown..] {
            let mut text: String = bug_report::hide_home(text).replace('\n', " ");
            if text.chars().count() > MESSAGE_CHARS {
                text = format!("{}…", text.chars().take(MESSAGE_CHARS).collect::<String>());
            }
            let _ = writeln!(out, "- {level:?}: {text}");
        }
    }
    out
}

impl App {
    /// Opens the bug report with the details as they are now and a screenshot: in
    /// `bug_report_window`, as the device index is taken out of the session while the
    /// ribbon and the panes (which run the command) are drawn.
    pub(super) fn report_bug(&mut self) {
        self.bug_report_wanted = true;
    }

    fn bug_report_details(&self, ctx: &egui::Context) -> String {
        let issues = &self.session.analysis.issues;
        let count = |s: Severity| issues.iter().filter(|i| i.severity == s && !i.accepted).count();
        let prj = &self.session.prj;
        let facts = ProjectFacts {
            pages: prj.pages.len(),
            page: self.session.page,
            page_kind: prj.pages.get(self.session.page).map(|p| format!("{:?}", p.kind)).unwrap_or_default(),
            devices: self.session.index.devices.len(),
            connections: self.session.analysis.connections.len(),
            errors: count(Severity::Error),
            warnings: count(Severity::Warning),
            notes: count(Severity::Info),
            unsaved: self.session.dirty(),
            shared: self.remote.project().is_some(),
        };
        let mut tool = format!("{:?}", self.editor.tool);
        tool.truncate(tool.char_indices().nth(80).map_or(tool.len(), |(i, _)| i));
        let shell = [
            ("Workspace", self.workspace.title().to_string()),
            ("Document pane", format!("{:?}", self.active_doc)),
            ("Last pane used", self.active_pane.map(|p| format!("{p:?}")).unwrap_or_else(|| "none".into())),
            ("Tool", tool),
            ("UI scale", format!("{:.2}", self.shell.ui_scale)),
        ];
        let plugins: Vec<String> = self
            .plugins
            .list()
            .iter()
            .filter(|p| matches!(p.origin, openecraft_api::Origin::External(_)))
            .map(|p| format!("{} {}{}", p.info.id, p.info.version, if p.enabled { "" } else { " (disabled)" }))
            .collect();
        let messages: Vec<(crate::style::Level, String)> = self
            .messages
            .iter()
            .map(|m| (m.level, if m.detail.is_empty() { m.text.clone() } else { format!("{} ({})", m.text, m.detail) }))
            .collect();
        let mut out = bug_report::environment(ctx, "OpenECraft", env!("CARGO_PKG_VERSION"));
        out.push_str(&details(&facts, &shell, &plugins, &messages));
        out
    }

    /// The dialog of the bug report; hidden while its screenshot is taken. Esc keeps it
    /// open: what was written is only dropped by Close.
    pub(super) fn bug_report_window(&mut self, ctx: &egui::Context) {
        if std::mem::take(&mut self.bug_report_wanted) {
            let details = self.bug_report_details(ctx);
            self.bug_report.begin(ctx, details);
        }
        self.bug_report.poll(ctx);
        if self.bug_report.draft.is_none() || self.bug_report.capturing() {
            return;
        }
        let (window, _esc) = self.dialogs.window(ctx, tr("Report a bug"), "bug_report", None);
        let mut open = true;
        let height = (ctx.content_rect().height() - 220.0).clamp(160.0, 640.0);
        let mut action = Action::None;
        window.open(&mut open).default_width(560.0).show(ctx, |ui| {
            action = bug_report::form(ui, &mut self.bug_report, height);
        });
        match action {
            Action::Close => open = false,
            Action::SaveScreenshot => {
                if let Some(draft) = &mut self.bug_report.draft {
                    if let Some(png) = draft.screenshot_png() {
                        let result = png.and_then(|bytes| self.io.save_bytes_as("openecraft-bug-report.png", ("PNG", "png"), &bytes)).map(|path| path.is_some());
                        draft.saved(result);
                    }
                }
            }
            Action::None => {}
        }
        if !open {
            self.bug_report.close();
        }
    }
}

#[cfg(test)]
mod tests;
