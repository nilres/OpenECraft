//! The update of the editor in the shell: the messages of the check, its segment of the
//! status bar and its settings; `crate::updater` does the work in the background.

use super::*;
use crate::updater::{Install, State};

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl App {
    /// Tells what the update came to: news always, the rest only after File › Help ›
    /// Check for updates (a check at start without a connection stays quiet).
    pub(super) fn poll_update(&mut self, ctx: &egui::Context) {
        let Some(state) = self.updater.poll() else { return };
        let manual = self.updater.manual;
        match state {
            State::UpToDate if manual => self.notify_ok(ctx, tr_fmt!("OpenECraft {} is up to date", VERSION)),
            State::Available { version, .. } => self.notify(ctx, tr_fmt!("OpenECraft {} is available", version)),
            State::Ready { version, .. } => {
                let text = self.update_ready_text(&version);
                self.notify_ok(ctx, text);
            }
            State::Failed { error, page } if manual || !page.is_empty() => self.notify_failure(ctx, tr("The update failed"), error),
            _ => {}
        }
    }

    fn update_ready_text(&self, version: &str) -> String {
        match self.updater.install() {
            Install::WindowsSetup(_) => tr_fmt!("OpenECraft {} is installed when you close OpenECraft", version),
            _ => tr_fmt!("OpenECraft {} is installed and starts next time", version),
        }
    }

    pub(super) fn check_for_updates(&mut self, ctx: &egui::Context) {
        if !self.updater.start(ctx, true) {
            if let State::Ready { version, .. } = self.updater.state() {
                let text = self.update_ready_text(&version);
                self.notify_ok(ctx, text);
            }
        }
    }

    /// Closes the editor (asking for unsaved changes first); it starts again updated.
    pub(super) fn restart_to_update(&mut self, ctx: &egui::Context) {
        self.updater.restart = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    /// The segment of the status bar (right to left): the download, the update to
    /// restart for or the release to fetch by hand.
    pub(super) fn update_status(&mut self, ui: &mut egui::Ui) {
        match self.updater.state() {
            State::Downloading { version } => {
                ui.label(RichText::new(tr_fmt!("Downloading OpenECraft {}", version)).small());
                ui.add(egui::Spinner::new().size(12.0));
                ui.separator();
            }
            State::Ready { version, .. } => {
                let color = crate::style::Level::Ok.color(self.dark, ui.visuals().text_color());
                let r = icons::button(ui, Icon::Download, tr("Restart and update"));
                let hint = self.update_ready_text(&version);
                let label = ui.add(egui::Button::new(RichText::new(tr("Restart and update")).small().color(color)).frame(false)).on_hover_text(&hint);
                if r.on_hover_text(&hint).clicked() || label.clicked() {
                    self.restart_to_update(ui.ctx());
                }
                ui.separator();
            }
            State::Available { version, page } => {
                let r = ui.add(egui::Button::new(RichText::new(tr_fmt!("OpenECraft {} is available", version)).small()).frame(false));
                if r.on_hover_text(tr("Opens the page of the release")).clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(page));
                }
                ui.separator();
            }
            _ => {}
        }
    }

    /// Settings › Updates.
    pub(super) fn update_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(tr_fmt!("Version {}", VERSION));
        if !crate::updater::supported() {
            ui.label(tr("The web version is updated on its server."));
            return;
        }
        ui.add_space(4.0);
        ui.checkbox(&mut self.shell.update_check, tr("Look for updates when OpenECraft starts"));
        if !crate::updater::allowed_at_start() {
            ui.colored_label(crate::style::warning(), tr_fmt!("Switched off on this computer ({})", crate::updater::DISABLE_VAR));
        }
        ui.label(match self.updater.install() {
            Install::WindowsSetup(_) => tr("A new release is downloaded in the background and installed when you close OpenECraft."),
            Install::AppImage(_) | Install::MacBundle(_) => tr("A new release is downloaded in the background and replaces this installation; it is used from the next start."),
            Install::Other => tr("This installation is updated by its package manager or by hand: OpenECraft only tells of a new release."),
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            self.cmd_button(ui, Cmd::CheckForUpdates);
            match self.updater.state() {
                State::Idle => {}
                State::Checking => {
                    ui.add(egui::Spinner::new().size(12.0));
                    ui.label(tr("Looking for updates…"));
                }
                State::UpToDate => {
                    ui.label(tr_fmt!("OpenECraft {} is up to date", VERSION));
                }
                State::Available { version, page } => {
                    ui.label(tr_fmt!("OpenECraft {} is available", version));
                    ui.hyperlink_to(tr("Release page"), page);
                }
                State::Downloading { version } => {
                    ui.add(egui::Spinner::new().size(12.0));
                    ui.label(tr_fmt!("Downloading OpenECraft {}", version));
                }
                State::Ready { version, .. } => {
                    ui.label(self.update_ready_text(&version));
                    self.cmd_button(ui, Cmd::InstallUpdate);
                }
                State::Failed { error, page } => {
                    ui.colored_label(crate::style::Level::Error.color(self.dark, ui.visuals().text_color()), tr("The update failed")).on_hover_text(&error);
                    if !page.is_empty() {
                        ui.hyperlink_to(tr("Release page"), page);
                    }
                }
            }
        });
    }
}
