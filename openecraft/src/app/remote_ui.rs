//! Project server window and shared editing: opening and saving projects on the
//! project server, locks on pages and on the project data. Changes without a lock
//! request it; changes of what another user has locked are taken back. Local changes
//! replaced by the server's version are kept (`Remote::kept`) to be compared, exported
//! or taken over again.

use openecraft_ui::icons::{self, Icon};
use super::App;
use openecraft_core::history::History;
use crate::remote::{Remote, Reply, Sent};
use egui::{Color32, RichText};
use openecraft_core::collab::{restore, valid_name, Grant, KeptReason, ProjectRight, Right, Shared, Snapshot, Subject, Target};
use openecraft_i18n::{tr, tr_fmt};
use crate::startup::ReconnectState;

/// What to do with a kept copy of local changes.
#[derive(Clone, Copy)]
enum KeptAction {
    Compare,
    TakeOver,
    Export,
    Discard,
}

impl App {
    /// Handles the answers of the project server.
    pub(super) fn poll_remote(&mut self, ctx: &egui::Context) {
        for reply in self.remote.poll(ctx) {
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
                Reply::Login(Ok(_)) => {
                    // Signed in for the project of the start: open it now.
                    if let Some(r) = self.reconnect.as_mut().filter(|r| r.state == ReconnectState::SignIn) {
                        r.state = ReconnectState::Connecting;
                        let name = r.project.name.clone();
                        self.request_replace(ctx, super::Replace::Shared(name));
                    }
                }
                Reply::Saved(sent, Ok(snapshot)) => {
                    // Changes the server left out: locked or changed by another user.
                    let ignored: Vec<String> = snapshot.ignored.iter().map(|t| Remote::describe(&self.session.prj, *t)).collect();
                    let revision = snapshot.revision;
                    let status = self.remote.status.clone();
                    self.take_snapshot(ctx, *snapshot, Some(&sent));
                    if ignored.is_empty() {
                        self.notify_ok(ctx, status);
                    } else {
                        self.notify_warning(
                            ctx,
                            tr_fmt!("Partly saved (revision {}). Not saved, locked or changed by another user: {}. Your version is kept in the Project server pane.", revision, ignored.join(", ")),
                        );
                    }
                }
                Reply::Snapshot(_, Ok(snapshot)) => self.take_snapshot(ctx, *snapshot, None),
                Reply::Status(_, Ok(status)) => self.remote.take_status(status, &[]),
                Reply::Locked(_, targets, Ok(status)) => self.remote.take_status(status, &targets),
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
                Reply::Opened(_, Err(_)) | Reply::Saved(_, Err(_)) | Reply::Snapshot(_, Err(_)) | Reply::Status(_, Err(_)) | Reply::Access(_, Err(_)) => {
                    self.notify_error(ctx, self.remote.status.clone());
                }
                Reply::List(_) | Reply::Login(Err(_)) | Reply::Access(_, Ok(_)) | Reply::Closed => {}
            }
        }
        self.remote.release_cancelled(&self.session.prj);
        self.remote.flush(&self.session);
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

    /// Saves to the project server; `release` locks are released afterwards.
    pub(super) fn save_remote(&mut self, release: Vec<Target>) {
        self.remote.save(&self.session.prj, release);
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
            egui::Grid::new("remote_login").num_columns(2).show(ui, |ui| {
                ui.label(tr("Server"));
                ui.add_enabled(!connected, egui::TextEdit::singleline(&mut self.remote.url).desired_width(300.0));
                ui.end_row();
                ui.label(tr("User name"));
                ui.add_enabled(!connected, egui::TextEdit::singleline(&mut self.remote.user).desired_width(300.0));
                ui.end_row();
                ui.label(tr("Password"));
                ui.horizontal(|ui| {
                    let r = ui.add_enabled(!connected, egui::TextEdit::singleline(&mut self.remote.password).password(true).desired_width(200.0));
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let can = !connected && !self.remote.password.is_empty() && !self.remote.user.trim().is_empty() && !self.remote.busy();
                    if (ui.add_enabled(can, egui::Button::new(tr("Sign in"))).on_hover_text(tr("With a user account of the server; replaces the access token")).clicked() || enter) && can {
                        self.remote.sign_in();
                    }
                });
                ui.end_row();
                ui.label(tr("Access token"));
                ui.add_enabled(!connected, egui::TextEdit::singleline(&mut self.remote.token).password(true).desired_width(300.0));
                ui.end_row();
                if let Some(role) = self.remote.role {
                    ui.label(tr("Role"));
                    ui.label(role.title());
                    ui.end_row();
                }
                if connected && !self.remote.rights.is_empty() {
                    ui.label(tr("Rights"));
                    let rights: Vec<&str> = self.remote.rights.iter().map(|r| tr(r.title())).collect();
                    ui.add(egui::Label::new(rights.join(", ")).wrap());
                    ui.end_row();
                }
            });
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

    fn projects_section(&mut self, ui: &mut egui::Ui) {
        let has_user = !self.remote.user.trim().is_empty();
        if !has_user {
            ui.colored_label(crate::style::lock(), tr("Enter a user name: other users see it on your locks."));
        }
        let mut open = None;
        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
            egui::Grid::new("remote_projects").striped(true).num_columns(5).show(ui, |ui| {
                for h in [tr("Name"), tr("Project"), tr("Pages"), tr("Users"), ""] {
                    ui.strong(h);
                }
                ui.end_row();
                for p in &self.remote.projects {
                    ui.label(&p.name);
                    ui.label(&p.title);
                    ui.label(p.pages.to_string());
                    ui.label(p.users.join(", "));
                    if ui.add_enabled(has_user, egui::Button::new(tr("Open project"))).clicked() {
                        open = Some(p.name.clone());
                    }
                    ui.end_row();
                }
            });
        });
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
        if ui.button(tr("Close shared project")).on_hover_text(tr("Releases all locks; the project stays open as a local copy")).clicked() {
            self.recovery = self.remote.project();
            self.remote.close();
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
