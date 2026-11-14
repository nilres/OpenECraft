//! Account selection and opt-in remembered sign-ins. Project/history ownership
//! stays with the app; this module never replaces an open project.

use super::{Answer, Link, Remote, Reply};
use super::credentials::{Credentials, Job, Kind};
use openecraft_client::{decode, ehttp, ProjectEndpoint};
use openecraft_client::project_credentials::{Account, SystemCredentialStore};
use openecraft_core::collab::{CurrentAccount, LoginResponse};
use openecraft_i18n::tr;
use openecraft_ui::io::Store;
use serde::{Deserialize, Serialize};

const PROFILES_KEY: &str = "project_server_sign_ins";

/// Nonsecret opt-in metadata. Tokens live exclusively in the OS credential store.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Profile { server: String, user: String }

impl Profile {
    fn of(account: &Account) -> Self { Self { server: account.server().into(), user: account.user().into() } }
    fn account(&self) -> Option<Account> { Account::new(&self.server, &self.user) }
    fn key(&self) -> String {
        format!("project_server_sign_in_{}", openecraft_client::update::sha256(&serde_json::to_vec(self).expect("account metadata")))
    }
}

#[derive(Serialize, Deserialize)]
struct Preference { profile: Profile, enabled: bool, revision: String }

#[derive(Clone)]
pub(super) enum After { List, Open(String) }

#[derive(Default)]
pub struct SignIn {
    pub remember: bool,
    pub signed_in: bool,
    pub notice: String,
    pub preference_error: String,
    pub restoring: bool,
    pub(super) generation: u64,
    profiles: Vec<Profile>,
    checked: bool,
    after: Option<After>,
    credentials: Credentials,
    /// Explicit sign-out still revokes a token whose restore was cancelled.
    revoking: Vec<(u64, Account)>,
    /// A later opt-in/out in another window supersedes this window's old save.
    intent: Option<(Account, String)>,
    store: Option<Store>,
    loaded: bool,
}

impl SignIn {
    pub(super) fn attach(&mut self, ctx: &egui::Context) {
        if self.store.is_none() {
            let store = Store::new("OpenECraft");
            store.load(ctx, PROFILES_KEY);
            self.store = Some(store);
            self.load_profiles();
        }
    }

    fn load_profiles(&mut self) {
        let Some(store) = &self.store else { return };
        while let Some((key, value)) = store.poll() {
            if key != PROFILES_KEY || self.loaded { continue; }
            self.profiles = value.and_then(|v| serde_json::from_slice::<Vec<Profile>>(&v).ok()).unwrap_or_default()
                .into_iter().filter(|p| p.account().is_some()).collect();
            self.loaded = true;
        }
    }

    fn set_profile(&mut self, account: &Account, enabled: bool) -> bool {
        let profile = Profile::of(account);
        self.profiles.retain(|p| *p != profile);
        if enabled { self.profiles.push(profile); }
        let revision = format!("{}-{}", std::process::id(), openecraft_ui::io::now_text());
        // A process-local counter also separates several decisions in one frame.
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let revision = format!("{revision}-{}", NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        self.intent = enabled.then(|| (account.clone(), revision.clone()));
        if let Some(store) = self.store.as_ref().filter(|_| self.loaded) {
            // Best-effort discovery index; only the per-account write below
            // decides whether remembering this account succeeded.
            store.store(PROFILES_KEY, serde_json::to_vec(&self.profiles).expect("account metadata"));
            // Each account owns its opt-in. A stale window's index must never
            // change another account's preference.
            let profile = Profile::of(account);
            store.store(&profile.key(), serde_json::to_vec(&Preference { profile, enabled, revision }).expect("account preference"));
            if !store.works() {
                self.preference_error = tr("Could not save the sign-in preference on this device.").into();
                return false;
            }
            self.preference_error.clear();
        }
        true
    }

    /// The index only suggests user names. Read this account's authoritative
    /// opt-in again before restoring; another window may have signed it out.
    fn load_selected(&mut self, account: &Account, ctx: &egui::Context) -> Option<String> {
        let Some(store) = &self.store else {
            return self.intent.as_ref().filter(|(a, _)| a == account && self.remembered(a)).map(|(_, r)| r.clone());
        };
        let profile = Profile::of(account);
        let key = profile.key();
        store.load(ctx, &key);
        while let Some((read, value)) = store.poll() {
            if read != key { continue; }
            let preference = value.and_then(|v| serde_json::from_slice::<Preference>(&v).ok()).filter(|p| p.profile == profile && p.enabled);
            self.profiles.retain(|p| *p != profile);
            if let Some(preference) = preference {
                self.profiles.push(profile.clone());
                return Some(preference.revision);
            }
        }
        None
    }

    fn intent_matches(&mut self, account: &Account, ctx: &egui::Context) -> bool {
        let revision = self.load_selected(account, ctx);
        self.intent.as_ref().is_some_and(|(a, r)| a == account && revision.as_ref() == Some(r))
    }

    fn remembered(&self, account: &Account) -> bool { self.profiles.contains(&Profile::of(account)) }

    pub fn busy(&self) -> bool { self.restoring || self.credentials.busy() }
    pub fn credential_changes_pending(&self) -> bool { self.credentials.busy() }
}

pub struct RestoreFailure { message: String, invalid: bool }

fn checked_account(result: ehttp::Result<ehttp::Response>) -> Result<CurrentAccount, RestoreFailure> {
    match result {
        Ok(r) if r.status == 401 => Err(RestoreFailure { message: tr("The saved sign-in has expired or was revoked. Sign in again.").into(), invalid: true }),
        Ok(r) if matches!(r.status, 404 | 405) => Err(RestoreFailure { message: tr("This server cannot validate a saved sign-in. Sign in with your password.").into(), invalid: false }),
        result => decode(result).map_err(|_| RestoreFailure { message: tr("The saved sign-in could not be checked. Check the connection or sign in with your password.").into(), invalid: false }),
    }
}

impl Remote {
    pub fn can_remember(&self) -> bool { SystemCredentialStore::available_for(&self.url, &self.user) }

    /// Invalidates every outstanding reply, without touching the local project.
    fn clear_identity(&mut self) {
        self.sign_in.generation += 1;
        self.sign_in.signed_in = false;
        self.sign_in.restoring = false;
        self.sign_in.checked = false;
        self.sign_in.after = None;
        self.sign_in.notice.clear();
        self.token.clear();
        self.password.clear();
        self.role = None;
        self.rights.clear();
        self.access = None;
        self.projects.clear();
        self.shared = None;
        self.pending = 0;
        self.polling = false;
        self.fetching = false;
        self.saving = None;
        self.queued = None;
        self.refetch = None;
        self.cancelled_locks.clear();
        self.link = Link::Online;
        self.save_error = None;
        self.reopening = false;
        self.due = false;
        self.status.clear();
    }

    /// Reauthentication changes credentials, not the open project or its base.
    fn reset_sign_in(&mut self, keep_project: bool) {
        let shared = if keep_project { self.shared.take() } else { None };
        let rights = std::mem::take(&mut self.rights);
        let access = self.access.take();
        let save_error = self.save_error.take();
        self.clear_identity();
        if let Some(mut shared) = shared {
            // The old generation's lock answers will no longer be applied.
            shared.requested.clear();
            self.shared = Some(shared);
            self.rights = rights;
            self.access = access;
            self.save_error = save_error;
            self.link = Link::SignedOut;
        }
    }

    /// Called after the user edits the disconnected server/account/token fields.
    /// The caller keeps the local project before detaching a shared session.
    pub fn credentials_edited(&mut self, url: &str, user: &str, token: &str) {
        let identity_changed = self.url != url || self.user != user;
        if !identity_changed && self.token == token { return; }
        let typed_token = (!identity_changed).then(|| self.token.clone());
        self.reset_sign_in(!identity_changed && self.shared.is_some() && self.link == Link::SignedOut);
        if let Some(token) = typed_token { self.token = token; }
        self.sign_in.checked = !self.token.is_empty();
        self.sign_in.remember = Account::new(&self.url, &self.user).is_some_and(|a| self.sign_in.remembered(&a));
    }

    /// Startup/settings select the final server before any credential is loaded.
    pub fn select_server(&mut self, url: &str) {
        if self.url == url { return; }
        self.close();
        self.clear_identity();
        self.url = url.into();
        self.sign_in.load_profiles();
        if let Some(server) = Account::new(url, "_") {
            if !self.sign_in.profiles.iter().any(|p| p.server == server.server() && p.user.eq_ignore_ascii_case(self.user.trim())) {
                if let Some(p) = self.sign_in.profiles.iter().rev().find(|p| p.server == server.server()) {
                    self.user = p.user.clone();
                }
            }
        }
        self.sign_in.remember = Account::new(&self.url, &self.user).is_some_and(|a| self.sign_in.remembered(&a));
    }

    pub fn sign_in(&mut self) {
        let password = std::mem::take(&mut self.password);
        let remember = self.sign_in.remember && self.can_remember();
        self.reset_sign_in(self.shared.is_some() && self.link == Link::SignedOut);
        self.sign_in.checked = true;
        self.sign_in.remember = remember;
        if remember {
            if let Some(account) = Account::new(&self.url, &self.user) {
                if !self.sign_in.set_profile(&account, true) { self.sign_in.remember = false; }
            }
        } else { self.forget_selected(); }
        let req = self.endpoint().login(self.user.trim(), &password);
        self.send(req, tr("Signing in…").into(), |r| Reply::Login(decode(r)));
    }

    /// Keep the active login in RAM even if the protected store is unavailable.
    pub fn remember_changed(&mut self) {
        let Some(account) = Account::new(&self.url, &self.user) else { self.sign_in.remember = false; return };
        if !self.sign_in.remember {
            self.forget_selected();
        } else if self.sign_in.signed_in {
            if self.sign_in.set_profile(&account, true) { self.save_remembered_login(account); }
            else { self.sign_in.remember = false; }
        }
    }

    fn save_remembered_login(&mut self, account: Account) {
        if !self.sign_in.intent_matches(&account, &self.ctx.clone().unwrap_or_default()) {
            self.sign_in.remember = false;
            return;
        }
        self.sign_in.notice = tr("Saving the sign-in on this device…").into();
        self.sign_in.credentials.enqueue(self.sign_in.generation, Job::Save(account, self.token.clone()), self.ctx.clone());
    }

    fn forget_selected(&mut self) {
        if let Some(account) = Account::new(&self.url, &self.user) {
            if !self.can_remember() {
                self.sign_in.profiles.retain(|p| *p != Profile::of(&account));
                return;
            }
            self.sign_in.set_profile(&account, false);
            self.sign_in.notice = tr("Removing the saved sign-in…").into();
            self.sign_in.credentials.enqueue(self.sign_in.generation, Job::Forget(account), self.ctx.clone());
        }
    }

    /// Local sign-out is immediate and works offline. OS deletion is serialized
    /// after pending writes; server revocation reports its own result.
    pub fn sign_out(&mut self) {
        let endpoint = self.endpoint();
        let signed_in = self.sign_in.signed_in;
        let read_remembered = !signed_in && (self.sign_in.restoring || self.has_saved_sign_in());
        self.close();
        self.clear_identity();
        self.sign_in.checked = true;
        self.sign_in.remember = false;
        if read_remembered {
            if let Some(account) = Account::new(&self.url, &self.user) {
                self.sign_in.revoking.push((self.sign_in.generation, account.clone()));
                self.sign_in.credentials.enqueue(self.sign_in.generation, Job::Load(account), self.ctx.clone());
            }
        }
        self.forget_selected();
        self.status = tr("Signed out on this device.").into();
        if signed_in {
            self.send(endpoint.logout(), self.status.clone(), |r| Reply::SignedOut(openecraft_client::check(r).map(|_| ())));
        }
    }

    pub fn use_saved_sign_in(&mut self) {
        if self.shared.is_some() && self.link == Link::SignedOut { self.reset_sign_in(true); }
        self.sign_in.checked = false;
        self.list();
    }

    pub fn has_saved_sign_in(&self) -> bool {
        self.can_remember() && Account::new(&self.url, &self.user).is_some_and(|a| self.sign_in.remembered(&a))
    }

    pub fn restore_sign_in(&mut self) { self.restore_before(None); }

    pub(super) fn restore_before(&mut self, after: Option<After>) -> bool {
        self.sign_in.load_profiles();
        if self.sign_in.restoring {
            if after.is_some() { self.sign_in.after = after; }
            return true;
        }
        if self.sign_in.checked || !self.token.is_empty() || !self.can_remember() { return false; }
        let Some(account) = Account::new(&self.url, &self.user) else { return false };
        self.sign_in.load_selected(&account, &self.ctx.clone().unwrap_or_default());
        if !self.sign_in.remembered(&account) { return false; }
        self.sign_in.checked = true;
        self.sign_in.restoring = true;
        self.sign_in.remember = true;
        self.sign_in.after = after;
        self.status = tr("Checking the saved sign-in…").into();
        self.sign_in.credentials.enqueue(self.sign_in.generation, Job::Load(account), self.ctx.clone());
        true
    }

    pub(crate) fn poll_credentials(&mut self) {
        for finished in self.sign_in.credentials.poll() {
            if finished.kind == Kind::Load {
                if let Some(i) = self.sign_in.revoking.iter().position(|(generation, account)| *generation == finished.generation && *account == finished.account) {
                    self.sign_in.revoking.remove(i);
                    match finished.result {
                        Ok(Some(token)) => {
                            let endpoint = ProjectEndpoint { url: finished.account.server().into(), token };
                            if finished.generation == self.sign_in.generation {
                                self.send(endpoint.logout(), self.status.clone(), |r| Reply::SignedOut(openecraft_client::check(r).map(|_| ())));
                            } else {
                                ehttp::fetch(endpoint.logout(), |_| {});
                            }
                        }
                        Err(e) if finished.generation == self.sign_in.generation => {
                            let receive = self.receiver(self.status.clone());
                            receive(Reply::SignedOut(Err(e)));
                        }
                        _ => {}
                    }
                    continue;
                }
            }
            if finished.generation != self.sign_in.generation { continue; }
            match (finished.kind, finished.result) {
                (Kind::Load, Ok(Some(token))) => {
                    let endpoint = ProjectEndpoint { url: self.url.clone(), token: token.clone() };
                    let account = finished.account;
                    self.send(endpoint.current_account(), tr("Checking the saved sign-in…").into(), move |r| Reply::Restored(account, token, checked_account(r)));
                }
                (Kind::Load, result) => {
                    self.sign_in.restoring = false;
                    self.sign_in.after = None;
                    let message = match result {
                        Err(e) => e,
                        _ => {
                            self.sign_in.set_profile(&finished.account, false);
                            tr("No saved sign-in is available. Sign in with your password.").into()
                        }
                    };
                    let receive = self.receiver(message.clone());
                    receive(Reply::Login(Err(message)));
                }
                (Kind::Save, Ok(_)) => {
                    if self.sign_in.remember {
                        if !self.sign_in.intent_matches(&finished.account, &self.ctx.clone().unwrap_or_default()) {
                            self.sign_in.remember = false;
                            self.sign_in.credentials.enqueue(self.sign_in.generation, Job::ForgetIf(finished.account, self.token.clone()), self.ctx.clone());
                        } else {
                            self.sign_in.notice = tr("Sign-in saved in Windows Credential Manager. The server expiry still applies.").into();
                        }
                    }
                }
                (Kind::Forget, Ok(_)) => self.sign_in.notice = tr("The saved sign-in was removed from this device.").into(),
                (Kind::ForgetIf, Ok(_)) => self.sign_in.notice = tr("The sign-in preference was changed in another window.").into(),
                (_, Err(e)) => {
                    self.sign_in.remember = false;
                    if finished.kind == Kind::Save && self.sign_in.intent_matches(&finished.account, &self.ctx.clone().unwrap_or_default()) {
                        self.sign_in.set_profile(&finished.account, false);
                    }
                    self.sign_in.notice = e;
                }
            }
        }
    }

    pub(super) fn finish_restore(&mut self, account: Account, token: String, result: Result<CurrentAccount, RestoreFailure>) -> Result<LoginResponse, String> {
        self.sign_in.restoring = false;
        match result {
            Ok(current) if current.user.name.eq_ignore_ascii_case(account.user()) => Ok(LoginResponse { token, user: current.user, access: current.access }),
            result => {
                self.sign_in.after = None;
                let failure = result.err().unwrap_or_else(|| RestoreFailure { message: tr("The saved sign-in belongs to a different account. Sign in again.").into(), invalid: true });
                if failure.invalid {
                    self.sign_in.remember = false;
                    self.forget_selected();
                }
                Err(failure.message)
            }
        }
    }

    pub(super) fn accept_login(&mut self, login: &LoginResponse, restored: bool) {
        let reauthenticated = self.shared.is_some() && self.link == Link::SignedOut;
        if reauthenticated {
            // Requests sent before the new token must not undo its authentication.
            self.sign_in.generation += 1;
            self.pending = 0;
            self.polling = false;
            self.fetching = false;
            self.reopening = false;
            if let Some(shared) = &mut self.shared { shared.requested.clear(); }
        }
        self.token = login.token.clone();
        self.user = login.user.name.clone();
        self.role = login.user.role;
        self.sign_in.signed_in = true;
        self.sign_in.checked = true;
        if !restored && self.sign_in.remember {
            if let Some(account) = Account::new(&self.url, &self.user) { self.save_remembered_login(account); }
        }
        if reauthenticated {
            self.sign_in.after = None;
            self.link = Link::Offline;
            self.due = true;
            return;
        }
        if !matches!(self.sign_in.after, Some(After::Open(_))) {
            self.sign_in.after = None;
            self.list();
        }
    }

    /// The app checks current unsaved work again after the asynchronous login,
    /// rather than opening over edits made while the saved token was checked.
    pub fn take_authenticated_open(&mut self) -> Option<String> {
        match self.sign_in.after.take() {
            Some(After::Open(name)) => Some(name),
            _ => None,
        }
    }

    /// An abandoned login/open may have created a server session; end only that
    /// old session, never forward its token to the new server selection.
    pub(super) fn discard_answer(&mut self, answer: Answer) {
        match answer.reply {
            Reply::Login(Ok(login)) => {
                let endpoint = ProjectEndpoint { token: login.token, ..answer.endpoint };
                ehttp::fetch(endpoint.logout(), |_| {});
            }
            Reply::Opened(name, Ok(opened)) => ehttp::fetch(answer.endpoint.close(&name, &opened.session), |_| {}),
            Reply::Reopened(_, Ok(opened)) => {
                if let Some(name) = answer.project {
                    ehttp::fetch(answer.endpoint.close(&name, &opened.session), |_| {});
                }
            }
            Reply::Saved(sent, _) => {
                // Keep #182's close-after-save order even after the account was
                // changed. The request already carries its original endpoint.
                if let Some(i) = self.closing.iter().position(|(session, _, _)| *session == sent.session) {
                    let (_, request, _) = self.closing.remove(i);
                    ehttp::fetch(request, |_| {});
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
