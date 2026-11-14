//! Project server window and shared editing: opening and saving projects on the
//! project server, locks on pages and on the project data. Changes without a lock
//! request it; changes of what another user has locked are taken back. Local changes
//! replaced by the server's version are kept (`Remote::kept`) to be compared, exported
//! or taken over again.

use openecraft_ui::icons::{self, Icon};
use super::App;
use openecraft_core::history::History;
use crate::server_save::{Facts, State};
use crate::remote::{Blocked, Link, NameError, Remote, Reply, Sent};
use egui::{Color32, RichText};
use openecraft_core::collab::{restore, valid_name, CopySource, Failure, Grant, KeptReason, ProjectMeta, ProjectRight, Right, Shared, Snapshot, Subject, Target};
use openecraft_parts::date_time_from_unix;
use openecraft_i18n::{tr, tr_fmt};
use crate::startup::ReconnectState;

/// What the dialog of a server project's properties asks for.
#[derive(Clone, Copy)]
enum DraftAction {
    /// Create the copy, or save the changed properties.
    Confirm,
    Reload,
    Close,
}

/// Who created a project on the server and when, and what it was copied from; none
/// if the server has no record (projects of earlier servers).
fn created_text(meta: &ProjectMeta, from: Option<&CopySource>) -> Option<String> {
    if meta.created_by.is_empty() && meta.created_at == 0 {
        return None;
    }
    let by = if meta.created_by.is_empty() { tr("Unknown") } else { meta.created_by.as_str() };
    let when = if meta.created_at == 0 { tr("Unknown").to_string() } else { utc(meta.created_at) };
    Some(match from {
        Some(from) => tr_fmt!("{}, {} (copy of {}, revision {})", by, when, from.name, from.revision),
        None => format!("{by}, {when}"),
    })
}

/// What to do with a kept copy of local changes.
#[derive(Clone, Copy)]
enum KeptAction {
    Compare,
    TakeOver,
    Export,
    Discard,
}

/// Created by, last changed by and last changed at as the project list shows them:
/// what the server does not know (projects of earlier servers) is named so.
fn meta_texts(meta: &ProjectMeta) -> [String; 3] {
    let known = |name: &str| if name.is_empty() { tr("Unknown").to_string() } else { name.to_string() };
    [known(&meta.created_by), known(&meta.changed_by), if meta.changed_at == 0 { tr("Unknown").to_string() } else { utc(meta.changed_at) }]
}

fn utc(secs: u64) -> String {
    format!("{} UTC", date_time_from_unix(secs))
}

impl App {
    /// Handles the answers of the project server.
    pub(super) fn poll_remote(&mut self, ctx: &egui::Context) {
        let link = self.remote.link;
        let replies = self.remote.poll(ctx);
        if self.remote.link != link {
            self.link_changed(ctx, link);
        }
        for reply in replies {
            match reply {
                Reply::Opened(name, Ok(opened)) => {
                    let (shared, prj) = Shared::open(&name, *opened);
                    let next_id = prj.next_id;
                    self.replace_project(prj, None);
                    self.session.prj.next_id = next_id;
                    // Undo must not go back to the project opened before.
                    let rev = self.session.history.rev + 1;
                    self.session.history = History::new(&self.session.prj);
                    self.session.history.rev = rev;
                    self.session.mark_saved();
                    self.remote.checked_rev = rev;
                    self.remote.shared = Some(shared);
                    self.remote.window = false;
                    self.reconnect = None;
                    self.notify(ctx, self.remote.status.clone());
                }
                // The project of the start: the dialog offers retry, another project, the local copy.
                Reply::Opened(name, Err(e)) if self.reconnect.as_ref().is_some_and(|r| r.project.name == name) => {
                    if let Some(r) = &mut self.reconnect {
                        r.state = ReconnectState::Failed(e);
                    }
                }
                Reply::Login(Ok(_)) if self.remote.shared.is_none() => {
                    let open = self.remote.take_authenticated_open().or_else(|| {
                        self.reconnect.as_ref().filter(|r| r.state == ReconnectState::SignIn && r.project.url == self.remote.url).map(|r| r.project.name.clone())
                    });
                    if let Some(name) = open {
                        if let Some(r) = &mut self.reconnect { r.state = ReconnectState::Connecting; }
                        self.request_replace(ctx, super::Replace::Shared(name));
                    }
                }
                Reply::Login(Ok(_)) => {}
                Reply::Saved(sent, Ok(snapshot)) => {
                    // Changes the server left out: locked or changed by another user.
                    let ignored: Vec<String> = snapshot.ignored.iter().map(|t| Remote::describe(&self.session.prj, *t)).collect();
                    let revision = snapshot.revision;
                    let status = self.remote.status.clone();
                    self.take_snapshot(ctx, *snapshot, Some(&sent));
                    if ignored.is_empty() {
                        // The automatic save shows in the state of the save only.
                        if !sent.auto {
                            self.notify_ok(ctx, status);
                        }
                    } else {
                        self.notify_warning(
                            ctx,
                            tr_fmt!("Partly saved (revision {}). Not saved, locked or changed by another user: {}. Your version is kept in the Project server pane.", revision, ignored.join(", ")),
                        );
                    }
                }
                Reply::Reopened(_, Ok(opened)) => {
                    let taken = self.remote.take_reopened(&mut self.session, *opened, openecraft_ui::io::now_text());
                    if taken.kept.is_empty() {
                        self.notify_ok(ctx, tr("The project is open on the project server again (a new session). Your changes are kept and lock again as needed."));
                    } else {
                        self.notify_warning(
                            ctx,
                            tr_fmt!("The project is open on the project server again. Changed by another user meanwhile: {}. Your version is kept in the Project server pane.", taken.kept.join(", ")),
                        );
                    }
                }
                Reply::Snapshot(_, Ok(snapshot)) => self.take_snapshot(ctx, *snapshot, None),
                Reply::Status(_, Ok(status)) => self.remote.take_status(status, &[]),
                Reply::Locked(_, targets, Ok(status)) => self.remote.take_status(status, &targets),
                // No answer, or the session ended: the change waits for the lock until
                // the connection is back.
                Reply::Locked(_, targets, Err(e)) if e.why != Failure::Refused => {
                    if let Some(sh) = &mut self.remote.shared {
                        sh.denied(&targets);
                    }
                }
                Reply::Locked(_, targets, Err(_)) => {
                    self.notify(ctx, self.remote.status.clone());
                    if let Some(sh) = &mut self.remote.shared {
                        sh.denied(&targets);
                    }
                    let kept = self.remote.revert(&mut self.session.prj, &targets, KeptReason::Refused, openecraft_ui::io::now_text());
                    if !kept.is_empty() {
                        self.notify_warning(ctx, tr_fmt!("Not changed, the lock was refused: {}. Your version is kept in the Project server pane.", kept.join(", ")));
                    }
                }
                Reply::Saved(sent, Err(e)) => {
                    // The automatic save tells its first failure; a lost connection
                    // or session is told once, with its state. It tries again.
                    if !sent.auto || (e.why == Failure::Refused && self.remote.save_failures == 1) {
                        self.notify_error(ctx, self.remote.status.clone());
                    }
                    self.write_recovery();
                }
                Reply::Opened(_, Err(_)) | Reply::Access(_, Err(_)) => {
                    self.notify_error(ctx, self.remote.status.clone());
                }
                Reply::Login(Err(_)) => {
                    if let Some(r) = self.reconnect.as_mut().filter(|r| r.state == ReconnectState::Connecting) {
                        r.state = ReconnectState::Failed(self.remote.status.clone());
                    }
                }
                // Failures of the connection are told once, when its state changes.
                Reply::Snapshot(_, Err(e)) if e.why == Failure::Refused => self.notify_error(ctx, self.remote.status.clone()),
                Reply::Snapshot(_, Err(_)) | Reply::Status(_, Err(_)) | Reply::Reopened(_, Err(_)) => {}
                Reply::List(_) | Reply::SignedOut(_) | Reply::Restored(..) | Reply::Access(_, Ok(_)) | Reply::Closed(_) => {}
                // Taken by the draft (`Remote::take_draft`).
                Reply::Properties(..) | Reply::PropertiesSaved(..) | Reply::Copied(..) => {}
            }
        }
        self.remote.release_cancelled(&self.session.prj);
        self.remote.flush(&self.session);
    }

    /// Tells once what a change of the connection's state means for the open project.
    fn link_changed(&mut self, ctx: &egui::Context, before: Link) {
        if self.remote.link != Link::Online {
            self.write_recovery();
        }
        match self.remote.link {
            // A reopened project is told with its new session; after signing in again
            // the session is checked first.
            Link::Online if before == Link::Ended => {}
            Link::Offline if before == Link::SignedOut => {}
            Link::Online => self.notify_ok(ctx, self.remote.status.clone()),
            Link::Offline => self.notify_error(ctx, tr("Connection lost: the server does not answer. It is tried again every 15 s; your changes stay here.")),
            Link::Ended => self.notify(ctx, tr("The project session has ended (it expired or the server restarted). The project is opened again with your sign-in; your changes are kept.")),
            Link::SignedOut => self.notify_error(ctx, tr("Your sign-in to the project server is no longer valid. Sign in again in the Project server pane; your changes are kept.")),
            Link::Refused => self.notify_error(ctx, tr("The project server does not open this project again. Try again, or keep working on a local copy.")),
        }
    }

    /// The server project of the start could not be opened: try again, sign in, choose
    /// another project or go on with the local copy (never shown as the shared project).
    pub(super) fn reconnect_window(&mut self, ctx: &egui::Context) {
        let Some(r) = &self.reconnect else { return };
        let ReconnectState::Failed(error) = &r.state else { return };
        let copy = self.recovery.as_ref() == Some(&r.project) && self.remote.shared.is_none();
        let mut answer = None;
        egui::Modal::new(egui::Id::unique("reconnect_project")).show(ctx, |ui| {
            ui.set_width(440.0);
            ui.label(RichText::new(tr_fmt!("The project server {} could not open {}.", r.project.url, r.project.name)).strong());
            ui.weak(error);
            if copy {
                ui.add_space(4.0);
                ui.label(tr("The open project is a local recovery copy. Its changes are not saved on the project server; before the shared project opens, you decide whether to save them locally or discard them."));
            }
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if icons::text_button(ui, Icon::Rotate, tr("Retry connection")).clicked() {
                    answer = Some(0);
                }
                if ui.button(tr("Sign in…")).on_hover_text(tr("Sign in in the Project server pane; the project opens afterwards")).clicked() {
                    answer = Some(1);
                }
                if ui.button(tr("Choose another project")).clicked() {
                    answer = Some(2);
                }
                if copy && ui.button(tr("Open local recovery copy")).on_hover_text(tr("Go on working locally; Save saves to a local file")).clicked() {
                    answer = Some(3);
                }
            });
        });
        let Some(answer) = answer else { return };
        let Some(r) = &mut self.reconnect else { return };
        match answer {
            0 => {
                // The unsaved changes were decided on before the first attempt.
                r.state = ReconnectState::Connecting;
                let name = r.project.name.clone();
                self.remote.open(&name);
            }
            1 => {
                r.state = ReconnectState::SignIn;
                self.show_pane(crate::dock::Pane::Tool(crate::dock::ToolPane::ProjectServer));
            }
            2 => {
                self.reconnect = None;
                self.show_pane(crate::dock::Pane::Tool(crate::dock::ToolPane::ProjectServer));
                self.remote.list();
            }
            _ => self.reconnect = None,
        }
    }

    /// Takes a newer state of the server (`Remote::take_snapshot`) and tells what of
    /// the local changes it replaced; a save that left changes out is reported by the caller.
    fn take_snapshot(&mut self, ctx: &egui::Context, snapshot: Snapshot, sent: Option<&Sent>) {
        let left_out = sent.is_some() && !snapshot.ignored.is_empty();
        let taken = self.remote.take_snapshot(&mut self.session, snapshot, sent, openecraft_ui::io::now_text());
        if taken.kept.is_empty() || left_out {
            return;
        }
        if taken.lost {
            self.notify_warning(ctx, tr_fmt!("Changed by another user in the meantime: {}. Your version is kept in the Project server pane.", taken.kept.join(", ")));
        } else {
            self.notify_warning(ctx, tr_fmt!("Replaced by the version on the project server: {}. Your version is kept in the Project server pane.", taken.kept.join(", ")));
        }
    }

    /// Requests the locks for new changes; changes of what others have locked are
    /// taken back. Call after the history has committed the frame's changes.
    pub(super) fn check_locks(&mut self, ctx: &egui::Context) {
        if self.remote.checked_rev == self.session.history.rev {
            return;
        }
        self.remote.checked_rev = self.session.history.rev;
        let Some(sh) = &self.remote.shared else { return };
        let (held, free): (Vec<Target>, Vec<Target>) = sh.unlocked_changes(&self.session.prj).into_iter().partition(|t| sh.holder(*t).is_some());
        if !held.is_empty() {
            let who: Vec<String> = held
                .iter()
                .map(|t| tr_fmt!("{} (locked by {})", Remote::describe(&self.session.prj, *t), sh.holder(*t).map(|l| l.user.as_str()).unwrap_or_default()))
                .collect();
            if self.remote.revert(&mut self.session.prj, &held, KeptReason::Refused, openecraft_ui::io::now_text()).is_empty() {
                self.notify_warning(ctx, tr_fmt!("Not changed: {}", who.join(", ")));
            } else {
                self.notify_warning(ctx, tr_fmt!("Not changed: {}. Your version is kept in the Project server pane.", who.join(", ")));
            }
        }
        if !free.is_empty() {
            self.remote.lock(free);
        }
    }

    /// Saves to the project server; `release` locks are released afterwards. Returns
    /// false without a session to save with (`Remote::status` tells why).
    pub(super) fn save_remote(&mut self, release: Vec<Target>) -> bool {
        self.remote.save(&self.session.prj, release)
    }

    /// Writes the local recovery copy now (it is written every 30 s otherwise): the
    /// changes cannot go to the server for the time being.
    fn write_recovery(&mut self) {
        if self.session.dirty() {
            self.autosave.save(&self.session, &self.parts_db);
        }
    }

    /// The automatic save of the shared project: sends the finished changes when they
    /// are due (`committed`: no drag or text input is under way). Call after the
    /// frame's edits were committed and their locks checked.
    pub(super) fn autosave_remote(&mut self, ctx: &egui::Context, committed: bool) {
        if self.remote.shared.is_none() {
            self.server_auto = crate::server_save::Auto::default();
            return;
        }
        let now = ctx.input(|i| i.time);
        let facts = Facts {
            // Changes a save left out wait for the user, not for the next save.
            // A viewer has nothing the server takes.
            changed: self.session.dirty()
                && self.remote.unconfirmed != Some(self.session.history.state())
                && (!self.remote.viewer() || self.remote.may(ProjectRight::Review)),
            committed,
            saving: self.remote.saving(),
            link: self.remote.link,
        };
        let settings = self.shell.server_autosave;
        if self.server_auto.due(now, &settings, &facts) {
            self.remote.autosave(&self.session.prj);
        }
        if let Some(wait) = self.server_auto.wait(now, &settings).filter(|w| *w > 0.0) {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(wait));
        }
    }

    /// Whether the changes of the shared project are on the server.
    pub(super) fn save_state(&self) -> State {
        let partial = self.remote.unconfirmed == Some(self.session.history.state());
        State::of(self.remote.link, self.remote.saving(), self.remote.save_error.is_some(), partial, self.session.dirty())
    }

    /// The state of the save as a sentence, with what the automatic save does about it.
    pub(super) fn save_state_text(&self) -> String {
        let auto = self.shell.server_autosave;
        self.save_state().text(self.remote.save_error.as_deref(), auto.enabled.then_some(auto.secs))
    }

    /// The setting of the automatic save to the project server.
    pub(super) fn server_autosave_setting(&mut self, ui: &mut egui::Ui) {
        use crate::server_save::{MAX_SECS, MIN_SECS};
        let s = &mut self.shell.server_autosave;
        ui.checkbox(&mut s.enabled, tr("Save shared projects to the project server automatically"))
            .on_hover_text(tr("Finished changes of a project opened from the project server are saved to it by themselves; input under way is saved once it is finished. Locks are kept. Save (Ctrl+S) still saves at once."));
        ui.horizontal(|ui| {
            ui.label(tr("Interval"));
            ui.add_enabled(s.enabled, egui::DragValue::new(&mut s.secs).range(MIN_SECS..=MAX_SECS).suffix(" s"))
                .on_hover_text(tr("Seconds from a change to its automatic save (10 to 600)"));
        });
    }

    /// Ends the shared session; the project stays open as a local recovery copy.
    fn work_locally(&mut self) {
        self.recovery = self.remote.project();
        self.remote.close();
    }

    fn my_locks(&self) -> Vec<Target> {
        let Some(sh) = &self.remote.shared else { return Vec::new() };
        sh.locks.iter().filter(|l| l.session == sh.session.session).map(|l| l.target).collect()
    }

    /// Text and colour of a page's lock in the page list.
    pub(super) fn page_lock_mark(&self, id: u64) -> Option<(String, Color32)> {
        let sh = self.remote.shared.as_ref()?;
        if let Some(l) = sh.holder(Target::Page(id)) {
            return Some((format!("🔒 {}", l.user), crate::style::lock()));
        }
        sh.owns(Target::Page(id)).then(|| ("✏".to_string(), crate::style::ok()))
    }

    /// The user holding the lock of the current page (another session).
    /// Page `page` is locked by another user of the shared project.
    pub(super) fn page_held_by_other(&self, page: usize) -> bool {
        let Some(id) = self.session.prj.pages.get(page).map(|p| p.id) else { return false };
        self.remote.shared.as_ref().is_some_and(|sh| sh.holder(Target::Page(id)).is_some())
    }

    pub(super) fn page_locked_by(&self) -> Option<String> {
        let id = self.session.prj.pages.get(self.session.page)?.id;
        Some(self.remote.shared.as_ref()?.holder(Target::Page(id))?.user.clone())
    }

    /// Context menu entries of a page in the page list: the target to lock, or to
    /// save and release (true).
    pub(super) fn page_lock_menu(&self, ui: &mut egui::Ui, id: u64) -> Option<(Target, bool)> {
        let sh = self.remote.shared.as_ref()?;
        let target = Target::Page(id);
        if sh.owns(target) {
            ui.button(tr("Save and release page")).clicked().then_some((target, true))
        } else if sh.holder(target).is_none() {
            ui.button(tr("Lock page")).clicked().then_some((target, false))
        } else {
            None
        }
    }

    pub(super) fn remote_pane(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            let connected = self.remote.shared.is_some();
            let editable = !connected && !self.remote.sign_in.signed_in;
            let previous = (self.remote.url.clone(), self.remote.user.clone(), self.remote.token.clone());
            let mut login = false;
            let mut remember_changed = false;
            let sign_in = editable || self.remote.link == Link::SignedOut;
            let mut retry_token = false;
            egui::Grid::new("remote_login").num_columns(2).show(ui, |ui| {
                ui.label(tr("Server"));
                ui.add_enabled(editable, egui::TextEdit::singleline(&mut self.remote.url).desired_width(300.0));
                ui.end_row();
                ui.label(tr("User name"));
                ui.add_enabled(editable, egui::TextEdit::singleline(&mut self.remote.user).desired_width(300.0));
                ui.end_row();
                ui.label(tr("Password"));
                ui.horizontal(|ui| {
                    let r = ui.add_enabled(sign_in, egui::TextEdit::singleline(&mut self.remote.password).password(true).desired_width(200.0));
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let can = sign_in && !self.remote.password.is_empty() && !self.remote.user.trim().is_empty() && !self.remote.busy();
                    if (ui.add_enabled(can, egui::Button::new(tr("Sign in"))).on_hover_text(tr("With a user account of the server; replaces the access token")).clicked() || enter) && can {
                        login = true;
                    }
                });
                ui.end_row();
                ui.label(tr("Sign-in"));
                let can_remember = self.remote.can_remember();
                remember_changed = ui.add_enabled_ui(can_remember && !self.remote.busy(), |ui| ui.checkbox(&mut self.remote.sign_in.remember, tr("Keep me signed in"))).inner.changed();
                ui.end_row();
                ui.label(tr("Access token"));
                let token = ui.add_enabled(sign_in, egui::TextEdit::singleline(&mut self.remote.token).password(true).desired_width(300.0));
                if connected && token.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    retry_token = true;
                }
                ui.end_row();
                if let Some(role) = self.remote.role {
                    ui.label(tr("Role"));
                    ui.label(tr(role.title()));
                    ui.end_row();
                }
                if connected && !self.remote.rights.is_empty() {
                    ui.label(tr("Rights"));
                    let rights: Vec<&str> = self.remote.rights.iter().map(|r| tr(r.title())).collect();
                    ui.add(egui::Label::new(rights.join(", ")).wrap());
                    ui.end_row();
                }
            });
            if previous.0 != self.remote.url { self.reconnect = None; }
            self.remote.credentials_edited(&previous.0, &previous.1, &previous.2);
            if retry_token { self.remote.retry(); }
            if login { self.remote.sign_in(); }
            if remember_changed { self.remote.remember_changed(); }
            ui.weak(self.remote.remember_hint());
            ui.horizontal_wrapped(|ui| {
                let can_sign_out = self.remote.sign_in.signed_in || !self.remote.token.is_empty() || self.remote.busy();
                if ui.add_enabled(can_sign_out, egui::Button::new(tr("Sign out"))).clicked() {
                    self.sign_out_remote(false);
                }
                if ui.add_enabled(can_sign_out || !self.remote.user.is_empty(), egui::Button::new(tr("Change user…"))).clicked() {
                    self.sign_out_remote(true);
                }
                if !self.remote.sign_in.signed_in && self.remote.has_saved_sign_in() && ui.add_enabled(!self.remote.busy(), egui::Button::new(tr("Use saved sign-in"))).clicked() {
                    self.remote.use_saved_sign_in();
                }
            });
            if !self.remote.sign_in.notice.is_empty() { ui.label(&self.remote.sign_in.notice); }
            if !self.remote.sign_in.preference_error.is_empty() { ui.colored_label(crate::style::error(), &self.remote.sign_in.preference_error); }
            ui.horizontal(|ui| {
                if ui.add_enabled_ui(!self.remote.busy(), |ui| icons::text_button(ui, Icon::Rotate, tr("List projects"))).inner.clicked() {
                    self.remote.list();
                }
                if self.remote.busy() {
                    ui.spinner();
                }
                ui.label(&self.remote.status);
            });
            ui.separator();
            if connected {
                self.shared_section(ui);
            } else {
                self.projects_section(ui);
            }
        });
    }

    /// Authentication never discards the open work or its undo history.
    fn sign_out_remote(&mut self, change_user: bool) {
        self.session.commit(true);
        if let Some(project) = self.remote.project() { self.recovery = Some(project); }
        self.reconnect = None;
        if change_user { self.remote.change_user(); }
        else { self.remote.sign_out(); }
    }

    fn projects_section(&mut self, ui: &mut egui::Ui) {
        let has_user = !self.remote.user.trim().is_empty();
        if !has_user {
            ui.colored_label(crate::style::lock(), tr("Enter a user name: other users see it on your locks."));
        }
        let mut open = None;
        let mut select = None;
        let selected = self.remote.selected.clone().filter(|s| self.remote.projects.iter().any(|p| &p.name == s));
        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
            egui::Grid::new("remote_projects").striped(true).num_columns(7).show(ui, |ui| {
                for h in [tr("Name"), tr("Project"), tr("Pages"), tr("Users"), tr("Created by"), tr("Last changed by"), tr("Last changed")] {
                    ui.strong(h);
                }
                ui.end_row();
                for p in &self.remote.projects {
                    let [created_by, changed_by, changed_at] = meta_texts(&p.meta);
                    let r = ui.selectable_label(selected.as_deref() == Some(p.name.as_str()), &p.name);
                    if r.clicked() {
                        select = Some(p.name.clone());
                    }
                    if r.double_clicked() && has_user {
                        open = Some(p.name.clone());
                    }
                    ui.label(&p.title);
                    ui.label(p.pages.to_string());
                    ui.label(p.users.join(", "));
                    let created = ui.label(created_by);
                    if p.meta.created_at > 0 {
                        created.on_hover_text(tr_fmt!("Created {}", utc(p.meta.created_at)));
                    }
                    ui.label(changed_by);
                    ui.label(changed_at);
                    ui.end_row();
                }
            });
        });
        // What is done with the selected project.
        let mut show = None;
        ui.horizontal_wrapped(|ui| {
            let none = tr("Select a project in the list");
            if ui.add_enabled(has_user && selected.is_some(), egui::Button::new(tr("Open project"))).on_disabled_hover_text(none).clicked() {
                open = selected.clone();
            }
            if ui
                .add_enabled(selected.is_some(), egui::Button::new(tr("Properties…")))
                .on_hover_text(tr("Shows the properties saved on the project server; you change them here if you may and nobody has locked the project data"))
                .on_disabled_hover_text(none)
                .clicked()
            {
                show = Some(false);
            }
            if ui
                .add_enabled(has_user && selected.is_some(), egui::Button::new(tr("Copy project…")))
                .on_hover_text(tr("Creates a new project on the server from the saved state of this one; you set its name and properties first"))
                .on_disabled_hover_text(none)
                .clicked()
            {
                show = Some(true);
            }
        });
        if select.is_some() {
            self.remote.selected = select;
        }
        if let (Some(copy), Some(name)) = (show, &selected) {
            self.remote.show_properties(name, copy);
        }
        if let Some(name) = open {
            self.request_replace(&ui.ctx().clone(), super::Replace::Shared(name));
        }
        ui.separator();
        ui.label(tr("Upload the current project as a new shared project:"));
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.remote.new_name).hint_text(tr("Name on the server")));
            let ok = has_user && valid_name(&self.remote.new_name);
            if ui.add_enabled_ui(ok, |ui| icons::text_button(ui, Icon::Upload, tr("Upload"))).inner
                .on_hover_text(tr("Letters, digits, space, - _ . (up to 64 characters)"))
                .clicked()
            {
                let name = self.remote.new_name.clone();
                self.remote.create(&name, &self.session.prj);
            }
        });
        self.kept_section(ui, false);
    }

    /// Your versions of changes the server's version replaced: of the open project
    /// (`here`) to compare and take over again, otherwise of every shared project to
    /// export or discard.
    fn kept_section(&mut self, ui: &mut egui::Ui, here: bool) {
        let list: Vec<usize> = if here { self.remote.kept_here() } else { (0..self.remote.kept.len()).collect() };
        if list.is_empty() {
            return;
        }
        ui.separator();
        ui.strong(tr("Kept changes"));
        ui.weak(tr("Your versions of changes that were replaced by the version on the project server. They stay here until you discard them."));
        let mut action = None;
        for &i in list.iter().rev() {
            let k = &self.remote.kept[i];
            let what: Vec<String> = k.targets.iter().map(|t| Remote::describe(&k.project, *t)).collect();
            let reason = match k.reason {
                KeptReason::NotSaved => tr("Not saved, locked or changed by another user"),
                KeptReason::Changed => tr("Changed by another user before you got the lock"),
                KeptReason::Refused => tr("The lock was refused"),
                KeptReason::NoLock => tr("Replaced while you held no lock"),
            };
            ui.group(|ui| {
                if !here {
                    ui.label(RichText::new(tr_fmt!("{} on {}", k.name, k.server)).strong());
                }
                ui.label(what.join(", "));
                ui.weak(tr_fmt!("{} – revision {}, {}", reason, k.revision, k.time));
                ui.horizontal_wrapped(|ui| {
                    if let Some(sh) = self.remote.shared.as_ref().filter(|_| here) {
                        if ui.button(tr("Compare")).on_hover_text(tr("Shows the differences of your version to the open project; selected ones can be taken over")).clicked() {
                            action = Some((i, KeptAction::Compare));
                        }
                        let differing = k.differing(&self.session.prj);
                        let held: Vec<String> = differing.iter().filter_map(|t| sh.holder(*t).map(|l| l.user.clone())).collect();
                        let hover = if differing.is_empty() {
                            tr("The open project already has your version").to_string()
                        } else if !held.is_empty() {
                            tr_fmt!("Locked by {}", held.join(", "))
                        } else {
                            tr("Replaces the current version with yours as a change you can undo; the next save stores it").to_string()
                        };
                        let can = !differing.is_empty() && held.is_empty() && !self.remote.viewer();
                        if ui.add_enabled(can, egui::Button::new(tr("Take over again"))).on_hover_text(&hover).on_disabled_hover_text(&hover).clicked() {
                            action = Some((i, KeptAction::TakeOver));
                        }
                    }
                    if ui.button(tr("Export…")).on_hover_text(tr("Saves your version of the whole project as a project file")).clicked() {
                        action = Some((i, KeptAction::Export));
                    }
                    if icons::text_button(ui, Icon::Delete, tr("Discard")).on_hover_text(tr("Deletes this copy of your changes for good")).clicked() {
                        action = Some((i, KeptAction::Discard));
                    }
                });
            });
        }
        let Some((i, action)) = action else { return };
        let ctx = ui.ctx().clone();
        let k = self.remote.kept[i].clone();
        match action {
            KeptAction::Compare => self.compare_ui.set_kept(tr_fmt!("Your version of {}", k.time), k.project),
            KeptAction::TakeOver => {
                let targets = k.differing(&self.session.prj);
                self.session.prj = restore(&self.session.prj, &k.project, &targets);
                let what: Vec<String> = targets.iter().map(|t| Remote::describe(&k.project, *t)).collect();
                self.notify(&ctx, tr_fmt!("Your version taken over: {}. Save to store it on the project server.", what.join(", ")));
            }
            KeptAction::Export => {
                let name = format!("{} r{}.{}", k.name, k.revision, crate::io::PROJECT_EXT);
                match self.io.save_as(&name, (tr("OpenECraft project"), crate::io::PROJECT_EXT), &k.project.to_json()) {
                    Ok(Some(path)) => self.notify_ok(&ctx, tr_fmt!("Saved {}", crate::io::short_name(&path))),
                    Ok(None) => {}
                    Err(e) => self.notify_error(&ctx, e),
                }
            }
            KeptAction::Discard => self.remote.discard_kept(i, &mut self.session),
        }
    }

    fn shared_section(&mut self, ui: &mut egui::Ui) {
        let Some(sh) = &self.remote.shared else { return };
        ui.label(RichText::new(tr_fmt!("Shared project {} – revision {}, user {}", sh.name, sh.revision, sh.session.user)).strong());
        if self.remote.viewer() && self.remote.may(ProjectRight::Review) {
            ui.colored_label(crate::style::lock(), tr("You may review this project: comments, markup and approvals, no other changes."));
        } else if self.remote.viewer() {
            ui.colored_label(crate::style::lock(), tr("You may only view this project."));
        }
        let mine = self.my_locks();
        let others: Vec<(Target, String)> = sh.locks.iter().filter(|l| l.session != sh.session.session).map(|l| (l.target, l.user.clone())).collect();
        let page_id = self.session.prj.pages.get(self.session.page).map(|p| p.id);
        self.connection_state(ui);
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if icons::text_button(ui, Icon::Save, tr("Save")).clicked() {
                self.save_remote(Vec::new());
            }
            if ui.add_enabled(!mine.is_empty(), egui::Button::new(tr("Save and release all locks"))).clicked() {
                self.save_remote(mine.clone());
            }
            if let Some(id) = page_id {
                if !mine.contains(&Target::Page(id)) && ui.button(tr("Lock current page")).clicked() {
                    self.remote.lock(vec![Target::Page(id)]);
                }
            }
            if !mine.contains(&Target::Project)
                && ui.button(tr("Lock project data")).on_hover_text(tr("Page list, devices, parts, panel layout and settings")).clicked()
            {
                self.remote.lock(vec![Target::Project]);
            }
        });
        ui.add_space(4.0);
        ui.strong(tr("My locks"));
        if mine.is_empty() {
            ui.weak(tr("None – editing a page or the project data locks it."));
        }
        for t in &mine {
            ui.horizontal(|ui| {
                ui.label(format!("✏ {}", Remote::describe(&self.session.prj, *t)));
                if ui.small_button(tr("Save and release")).clicked() {
                    self.save_remote(vec![*t]);
                }
                if ui.small_button(tr("Release without saving")).on_hover_text(tr("Your unsaved changes of it are taken back")).clicked() {
                    if let Some(sh) = &self.remote.shared {
                        self.session.prj = sh.revert(&self.session.prj, &[*t]);
                    }
                    self.remote.release(vec![*t]);
                }
            });
        }
        ui.add_space(4.0);
        ui.strong(tr("Locked by other users"));
        if others.is_empty() {
            ui.weak(tr("None"));
        }
        for (t, user) in &others {
            ui.label(format!("🔒 {} – {}", Remote::describe(&self.session.prj, *t), user));
        }
        self.kept_section(ui, true);
        if self.remote.may(ProjectRight::ManageAccess) && !self.remote.rights.is_empty() {
            ui.separator();
            self.access_section(ui);
        }
        ui.separator();
        let mut close = false;
        ui.horizontal_wrapped(|ui| {
            if ui.button(tr("Properties…")).on_hover_text(tr("The properties of the open project; changing them locks the project data")).clicked() {
                self.show_project = true;
            }
            if ui.button(tr("Copy project…")).on_hover_text(tr("Creates a new project on the server from the saved state of this one; you set its name and properties first")).clicked() {
                if let Some(name) = self.remote.shared.as_ref().map(|sh| sh.name.clone()) {
                    self.remote.show_properties(&name, true);
                }
            }
            close = ui.button(tr("Close shared project")).on_hover_text(tr("Releases all locks; the project stays open as a local copy")).clicked();
        });
        if close {
            self.work_locally();
        }
    }

    /// The properties of a project of the server (`Remote::draft`), loaded from it:
    /// those of an existing project to change, or of a copy to create and open. The
    /// copy exists only in this dialog until it is confirmed.
    pub(super) fn server_project_window(&mut self, ctx: &egui::Context) {
        let Some(d) = &self.remote.draft else { return };
        let (copy, source) = (d.copy, d.source.clone());
        // What the open project has to do with the source of the copy.
        let connected = self.remote.shared.as_ref().is_some_and(|sh| sh.name == source);
        let unsaved = copy && connected && self.session.dirty();
        let recovery = copy && self.remote.shared.is_none() && self.recovery.as_ref().is_some_and(|r| r.url == self.remote.url && r.name == source);
        let has_user = !self.remote.user.trim().is_empty();
        let (title, salt) = if copy { (tr("Copy project"), "server_project_copy") } else { (tr("Server project properties"), "server_project_properties") };
        let (window, esc) = self.dialogs.window(ctx, title, salt, None);
        let mut open = !esc;
        let mut action = None;
        window.open(&mut open).default_width(520.0).min_width(380.0).show(ctx, |ui| {
            let Remote { draft: Some(d), projects, .. } = &mut self.remote else { return };
            let c = crate::style::colors(ui.visuals().dark_mode);
            if !d.loaded {
                if d.busy {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(tr_fmt!("Loading the properties of {}…", d.source));
                    });
                } else {
                    ui.colored_label(c.error, tr_fmt!("Project server: {}", d.error));
                }
                ui.horizontal(|ui| {
                    if !d.busy && icons::text_button(ui, Icon::Rotate, tr("Try again")).clicked() {
                        action = Some(DraftAction::Reload);
                    }
                    if ui.button(tr("Cancel")).clicked() {
                        action = Some(DraftAction::Close);
                    }
                });
                return;
            }
            let blocked = d.blocked();
            egui::Grid::new("server_project").num_columns(2).show(ui, |ui| {
                if d.copy {
                    ui.label(tr("Copy of"));
                    if d.revision > 0 {
                        ui.label(tr_fmt!("{} (revision {})", d.source, d.revision));
                    } else {
                        ui.label(&d.source);
                    }
                    ui.end_row();
                    ui.label(tr("Name on the server"));
                    ui.add(egui::TextEdit::singleline(&mut d.name).desired_width(f32::INFINITY)).on_hover_text(tr("Letters, digits, space, - _ . (up to 64 characters)"));
                    ui.end_row();
                } else {
                    ui.label(tr("Name on the server"));
                    ui.label(tr_fmt!("{} (revision {})", d.source, d.revision));
                    ui.end_row();
                    ui.label(tr("Created on the server")).on_hover_text(tr("Who created the project on the project server and when; not the drawing author below"));
                    match created_text(&d.meta, d.copied_from.as_ref()) {
                        Some(text) => ui.label(text),
                        None => ui.weak(tr("Unknown")),
                    };
                    ui.end_row();
                }
                if blocked.is_some() {
                    ui.disable();
                }
                super::windows::info_rows(ui, &mut d.info);
            });
            ui.separator();
            ui.add_enabled_ui(blocked.is_none(), |ui| super::windows::property_list(ui, &mut d.info.properties, 140.0));
            ui.separator();
            match &blocked {
                Some(Blocked::Rights) => {
                    ui.colored_label(crate::style::lock(), tr("You may only view this project."));
                }
                Some(Blocked::Locked(user)) => {
                    ui.colored_label(crate::style::lock(), tr_fmt!("The project data is locked by {}: the properties cannot be changed now.", user));
                }
                None => {}
            }
            if d.copy {
                match d.name_error(projects) {
                    Some(NameError::Invalid) => {
                        ui.colored_label(c.error, tr("The name on the server needs letters, digits, space, - _ . (up to 64 characters)."));
                    }
                    Some(NameError::Taken(name)) => {
                        ui.colored_label(c.error, tr_fmt!("{} already exists on the project server.", name));
                    }
                    None => {}
                }
                ui.checkbox(&mut d.records, tr("Also copy revisions, baselines and review records"))
                    .on_hover_text(tr("Released revisions, baselines, comments, markup, approvals and change requests of the source. Without them the copy starts as a new project."));
                ui.weak(tr("The copy is created from the state saved on the project server. Changes other users have not saved yet are not part of it. Sessions, locks and access rights are not copied; the original stays as it is."));
                if unsaved {
                    ui.colored_label(c.warning, tr("The open project has changes that are not saved on the project server yet. You are asked whether to save them first; only saved changes are part of the copy."));
                }
                if recovery {
                    ui.colored_label(c.warning, tr("The open project is a local recovery copy of this project. Its changes are not on the project server and not part of the copy."));
                }
                if d.changed {
                    ui.colored_label(c.warning, tr_fmt!("{} was changed on the project server meanwhile. Your entries are kept; create the copy again to copy the current saved state.", d.source));
                }
                if !has_user {
                    ui.colored_label(crate::style::lock(), tr("Enter a user name: other users see it on your locks."));
                }
            } else if d.changed {
                ui.colored_label(c.warning, tr("The properties were changed on the project server meanwhile. Reload shows them; your entries are replaced."));
            }
            if !d.error.is_empty() {
                ui.colored_label(c.error, tr_fmt!("Project server: {}", d.error));
            }
            ui.horizontal(|ui| {
                if d.copy {
                    if ui.add_enabled(has_user && d.ready(projects), egui::Button::new(tr("Create and open"))).clicked() {
                        action = Some(DraftAction::Confirm);
                    }
                } else {
                    if blocked.is_none() && ui.add_enabled(d.ready(projects), egui::Button::new(tr("Save"))).clicked() {
                        action = Some(DraftAction::Confirm);
                    }
                    if ui.add_enabled_ui(!d.busy, |ui| icons::text_button(ui, Icon::Rotate, tr("Reload"))).inner.clicked() {
                        action = Some(DraftAction::Reload);
                    }
                }
                // A copy being created is opened when the server answers.
                if ui.add_enabled(!d.creating, egui::Button::new(tr("Cancel"))).clicked() {
                    action = Some(DraftAction::Close);
                }
                if d.busy {
                    ui.spinner();
                }
            });
        });
        if !open && action.is_none() {
            action = Some(DraftAction::Close);
        }
        match action {
            Some(DraftAction::Close) => {
                if self.remote.draft.as_ref().is_some_and(|d| !d.creating) {
                    self.remote.draft = None;
                }
            }
            Some(DraftAction::Reload) => self.remote.load_properties(),
            Some(DraftAction::Confirm) if copy => {
                // What is saved first is decided by the question about the unsaved changes.
                if let Some(d) = self.remote.draft.as_mut().filter(|_| unsaved) {
                    d.revision = 0;
                }
                self.request_replace(ctx, super::Replace::Copy);
            }
            Some(DraftAction::Confirm) => self.remote.save_properties(),
            None => {}
        }
    }

    /// The state of the connection and, apart from it, whether the local changes are
    /// saved on the server; ways on when the session cannot go on by itself.
    fn connection_state(&mut self, ui: &mut egui::Ui) {
        let c = crate::style::colors(ui.visuals().dark_mode);
        let (color, text) = match self.remote.link {
            Link::Online => (c.ok, tr("Connected to the project server").to_string()),
            Link::Offline => (c.error, tr("Connection lost: the server does not answer. It is tried again every 15 s; your changes stay here.").to_string()),
            Link::Ended => (c.warning, tr("The project session has ended (it expired or the server restarted). The project is opened again with your sign-in; your changes are kept.").to_string()),
            Link::SignedOut => (c.error, tr("Your sign-in to the project server is no longer valid. Sign in again in the Project server pane; your changes are kept.").to_string()),
            Link::Refused => (c.error, tr("The project server does not open this project again. Try again, or keep working on a local copy.").to_string()),
        };
        ui.colored_label(color, text);
        let state = self.save_state();
        let color = match state {
            State::Saved | State::Saving => ui.visuals().weak_text_color(),
            State::Pending | State::Partial => c.warning,
            State::Offline | State::Failed => c.error,
        };
        ui.colored_label(color, self.save_state_text());
        match &self.remote.last_saved {
            Some(time) => ui.weak(tr_fmt!("Last save confirmed by the project server: {}", time)),
            None => ui.weak(tr("The project server has confirmed no save of this session yet")),
        };
        // During an outage: whether the changes are in the local recovery copy. Only
        // a copy that was written is claimed.
        if matches!(state, State::Offline | State::Failed) {
            let now = self.session.history.state();
            match self.autosave.backup() {
                Some((written, time)) if *written == now => ui.weak(tr_fmt!("Local recovery copy with these changes written: {}", time)),
                Some((_, time)) => ui.colored_label(c.warning, tr_fmt!("The local recovery copy is of {}: your latest changes are not in it yet", time)),
                None => ui.colored_label(c.warning, tr("No local recovery copy of these changes has been written yet")),
            };
        }
        self.server_autosave_setting(ui);
        if self.remote.online() {
            return;
        }
        let mut local = false;
        ui.horizontal_wrapped(|ui| {
            if self.remote.link != Link::SignedOut && icons::text_button(ui, Icon::Rotate, tr("Try again now")).clicked() {
                self.remote.retry();
            }
            local = ui.button(tr("Keep working on a local copy")).on_hover_text(tr("Ends the shared session; the project with your changes stays open as a local recovery copy, Save saves it to a file")).clicked();
        });
        if local {
            self.work_locally();
        }
    }

    /// Who may use the open project: grants of roles to users and groups (with
    /// wildcards; directory groups as `directory:group`), for project managers.
    fn access_section(&mut self, ui: &mut egui::Ui) {
        ui.strong(tr("Project access"));
        let busy = self.remote.busy();
        let Some(access) = &mut self.remote.access else {
            if ui.add_enabled(!busy, egui::Button::new(tr("Show who may use the project"))).clicked() {
                self.remote.load_access();
            }
            return;
        };
        let mut remove = None;
        egui::Grid::new("project_access").striped(true).num_columns(5).show(ui, |ui| {
            for h in [tr("User / group"), tr("Name"), tr("Role"), tr("Deny"), ""] {
                ui.strong(h);
            }
            ui.end_row();
            for (i, g) in access.grants.iter_mut().enumerate() {
                let group = matches!(g.subject, Subject::Group(_));
                let mut kind = group;
                egui::ComboBox::from_id_salt(("grant_kind", i)).selected_text(if group { tr("Group") } else { tr("User") }).show_ui(ui, |ui| {
                    ui.selectable_value(&mut kind, false, tr("User"));
                    ui.selectable_value(&mut kind, true, tr("Group"));
                });
                let mut name = g.subject.name().to_string();
                ui.add(egui::TextEdit::singleline(&mut name).desired_width(160.0).hint_text(tr("e.g. corp:Engineering")));
                g.subject = if kind { Subject::Group(name) } else { Subject::User(name) };
                egui::ComboBox::from_id_salt(("grant_role", i)).selected_text(g.role.as_str()).show_ui(ui, |ui| {
                    for r in &access.roles {
                        ui.selectable_value(&mut g.role, r.clone(), r.as_str());
                    }
                });
                ui.checkbox(&mut g.deny, "").on_hover_text(tr("Takes the rights of the role away, whatever other grants give"));
                if icons::button(ui, Icon::Delete, tr("Remove")).clicked() {
                    remove = Some(i);
                }
                ui.end_row();
            }
        });
        if let Some(i) = remove {
            access.grants.remove(i);
        }
        let role = access.roles.first().cloned().unwrap_or_default();
        let mut save = None;
        let mut reload = false;
        ui.horizontal(|ui| {
            if ui.button(tr("Add user")).clicked() {
                access.grants.push(Grant::new(Subject::User(String::new()), &role, ""));
            }
            if ui.button(tr("Add group")).clicked() {
                access.grants.push(Grant::new(Subject::Group(String::new()), &role, ""));
            }
            let complete = access.grants.iter().all(|g| !g.subject.name().trim().is_empty() && !g.role.is_empty());
            if ui.add_enabled(complete && !busy, egui::Button::new(tr("Save access"))).clicked() {
                save = Some(access.grants.clone());
            }
            reload = ui.add_enabled_ui(!busy, |ui| icons::text_button(ui, Icon::Rotate, tr("Reload"))).inner.clicked();
        });
        if let Some(grants) = save {
            self.remote.save_access(&grants);
        }
        if reload {
            self.remote.access = None;
            self.remote.load_access();
        }
    }
}

#[cfg(test)]
mod tests;
