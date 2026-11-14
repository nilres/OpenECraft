//! Client of the project server (`openecraft-project-server`): opening and saving
//! shared projects, page locks. Requests run in the background; [`Remote::poll`]
//! collects the answers. The rules of shared editing are in `openecraft_core::collab`.

use openecraft_client::{decode, decode_failure, ehttp, ProjectEndpoint};
use openecraft_core::collab::{
    self, CurrentAccount, Failed, Failure, Grant, Kept, KeptReason, LoginResponse, Opened, ProjectAccess, ProjectRight, ProjectRole, ProjectSummary, Shared, Snapshot, Status, Target,
};
use crate::session::Session;
use openecraft_core::model::Project;
use openecraft_i18n::{tr, tr_fmt};
use openecraft_ui::io::Store;
use std::sync::{Arc, Mutex};

mod credentials;
mod sign_in;
use sign_in::{After, SignIn, RestoreFailure};
use openecraft_client::project_credentials::Account;

/// Key of the kept local changes in the app's store.
const KEPT_KEY: &str = "shared_kept";

/// What taking a state of the server did to the edited project.
#[derive(Debug, Default, PartialEq)]
pub struct Taken {
    /// Another user changed a target before this session's lock on it was used.
    pub lost: bool,
    /// Local changes replaced by the server's version (as text), kept in `Remote::kept`.
    pub kept: Vec<String>,
}

/// State of the connection of the open shared project.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Link {
    #[default]
    Online,
    /// The server does not answer; the session is tried again with every heartbeat.
    Offline,
    /// The session ended on the server (it expired or the server restarted): the
    /// project is opened again with the sign-in, until the server answers.
    Ended,
    /// The access token or the sign-in is no longer valid: after signing in, the
    /// session goes on (or the project is opened again).
    SignedOut,
    /// The server no longer opens the project (removed, or no longer allowed).
    Refused,
}

/// A save request: what it sent, so that its answer confirms only that.
#[derive(Clone, Debug, Default)]
pub struct Sent {
    pub session: String,
    /// Server revision the project was edited from.
    pub revision: u64,
    pub project: Project,
    pub release: Vec<Target>,
}

/// Answer of a request. Answers for a session that is no longer open are dropped.
pub enum Reply {
    List(Result<Vec<ProjectSummary>, String>),
    /// Name on the server and the opened project.
    Opened(String, Result<Box<Opened>, String>),
    /// The open project opened again (for the ended session) in a new session.
    Reopened(String, Result<Box<Opened>, Failed>),
    Saved(Box<Sent>, Result<Box<Snapshot>, Failed>),
    Snapshot(String, Result<Box<Snapshot>, Failed>),
    Status(String, Result<Status, Failed>),
    /// Requested targets and the answer.
    Locked(String, Vec<Target>, Result<Status, Failed>),
    Login(Result<LoginResponse, String>),
    /// Internal: checked before a remembered token is used for project requests.
    Restored(Account, String, Result<CurrentAccount, RestoreFailure>),
    SignedOut(Result<(), String>),
    /// Grants on the open project (loaded or saved).
    Access(String, Result<ProjectAccess, Failed>),
    Closed(u64),
}

impl Reply {
    fn session(&self) -> Option<&str> {
        match self {
            Reply::Saved(sent, _) => Some(&sent.session),
            Reply::Snapshot(s, _) | Reply::Status(s, _) | Reply::Locked(s, _, _) | Reply::Access(s, _) | Reply::Reopened(s, _) => Some(s),
            Reply::List(_) | Reply::Opened(..) | Reply::Login(_) | Reply::Restored(..) | Reply::SignedOut(_) | Reply::Closed(_) => None,
        }
    }

    /// Answer of a request of the open session: Ok, or why it failed.
    fn outcome(&self) -> Option<Result<(), &Failed>> {
        match self {
            Reply::Reopened(_, r) => Some(r.as_ref().map(|_| ())),
            Reply::Saved(_, r) | Reply::Snapshot(_, r) => Some(r.as_ref().map(|_| ())),
            Reply::Status(_, r) | Reply::Locked(_, _, r) => Some(r.as_ref().map(|_| ())),
            Reply::Access(_, r) => Some(r.as_ref().map(|_| ())),
            Reply::List(_) | Reply::Opened(..) | Reply::Login(_) | Reply::Restored(..) | Reply::SignedOut(_) | Reply::Closed(_) => None,
        }
    }
}

/// Requests belong to the server/account selection that sent them. No Debug:
/// endpoints and login answers may contain a bearer token.
struct Answer {
    generation: u64,
    counted: bool,
    endpoint: ProjectEndpoint,
    project: Option<String>,
    reply: Reply,
}

pub struct Remote {
    pub url: String,
    /// Access token of the server (kept in memory only).
    pub token: String,
    pub user: String,
    /// Password of a user account (kept until signing in).
    pub password: String,
    pub sign_in: SignIn,
    /// Role of the signed-in user; None without user account (token or open server)
    /// or without built-in role (rights from grants only).
    pub role: Option<ProjectRole>,
    /// What the user may do with the open project; empty from older servers.
    pub rights: Vec<ProjectRight>,
    /// Grants on the open project, as edited by a project manager.
    pub access: Option<ProjectAccess>,
    /// Projects on the server (last list).
    pub projects: Vec<ProjectSummary>,
    /// The open shared project.
    pub shared: Option<Shared>,
    /// Name for uploading the current project.
    pub new_name: String,
    pub status: String,
    pub window: bool,
    /// State of the connection of the open shared project.
    pub link: Link,
    /// Error of the last save, until a save succeeds (the changes are not on the server).
    pub save_error: Option<String>,
    pending: usize,
    /// A status request is under way.
    polling: bool,
    /// The project is being opened again (`Link::Ended`).
    reopening: bool,
    /// Send the next status or reopen request now, not after the heartbeat interval.
    due: bool,
    /// A newer project is being loaded.
    fetching: bool,
    last_status: f64,
    /// History revision checked for changes without lock.
    pub checked_rev: u64,
    /// Session of the save under way: one at a time, the next waits for its answer.
    saving: Option<String>,
    /// A save asked for while one was under way, with the locks to release; it sends
    /// the project as it is when the previous save is answered.
    queued: Option<Vec<Target>>,
    /// Revision of a newer project announced while saving: loaded after the save.
    refetch: Option<u64>,
    /// Ends of sessions closed while their save was under way: sent after its answer.
    closing: Vec<(String, ehttp::Request, String, u64)>,
    /// Local changes replaced by the server's version (of every shared project), until
    /// the user discards them.
    pub kept: Vec<Kept>,
    /// Where `kept` is stored (none in tests).
    kept_store: Option<Store>,
    /// `kept` was read from the store: written only from then on.
    kept_loaded: bool,
    /// History state after a save that left changes out: it is not the saved state
    /// until their copies are discarded.
    pub unconfirmed: Option<u64>,
    ctx: Option<egui::Context>,
    inbox: Arc<Mutex<Vec<Answer>>>,
    cancelled_locks: Vec<(String, Vec<Target>)>,
}

impl Default for Remote {
    fn default() -> Self {
        let ProjectEndpoint { url, token } = ProjectEndpoint::default();
        let user = std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_default();
        Remote {
            url,
            token,
            user,
            password: String::new(),
            sign_in: SignIn::default(),
            role: None,
            rights: Vec::new(),
            access: None,
            projects: Vec::new(),
            shared: None,
            new_name: String::new(),
            status: String::new(),
            window: false,
            link: Link::Online,
            save_error: None,
            pending: 0,
            polling: false,
            reopening: false,
            due: false,
            fetching: false,
            last_status: 0.0,
            checked_rev: 0,
            saving: None,
            queued: None,
            refetch: None,
            closing: Vec::new(),
            kept: Vec::new(),
            kept_store: None,
            kept_loaded: false,
            unconfirmed: None,
            ctx: None,
            inbox: Arc::default(),
            cancelled_locks: Vec::new(),
        }
    }
}

impl Remote {
    fn save_status(result: &Result<Box<Snapshot>, Failed>) -> String {
        match result {
            Ok(s) if !s.ignored.is_empty() => tr_fmt!("Partly saved to the project server (revision {})", s.revision),
            Ok(s) => tr_fmt!("Saved to the project server (revision {})", s.revision),
            Err(e) => tr_fmt!("Project server: {}", e),
        }
    }

    fn endpoint(&self) -> ProjectEndpoint {
        ProjectEndpoint { url: self.url.clone(), token: self.token.clone() }
    }

    pub fn busy(&self) -> bool {
        self.pending > 0 || self.sign_in.busy()
    }

    fn send(&mut self, request: ehttp::Request, status: String, wrap: impl 'static + Send + FnOnce(ehttp::Result<ehttp::Response>) -> Reply) {
        let receive = self.receiver(status);
        ehttp::fetch(request, move |result| receive(wrap(result)));
    }

    fn receiver(&mut self, status: String) -> impl FnOnce(Reply) + Send + 'static {
        self.status = status;
        self.pending += 1;
        self.answer_receiver(true)
    }

    fn answer_receiver(&self, counted: bool) -> impl FnOnce(Reply) + Send + 'static {
        let inbox = self.inbox.clone();
        let ctx = self.ctx.clone();
        let generation = self.sign_in.generation;
        let endpoint = self.endpoint();
        let project = self.shared.as_ref().map(|shared| shared.name.clone());
        move |reply| {
            if let Ok(mut inbox) = inbox.lock() {
                inbox.push(Answer { generation, counted, endpoint, project, reply });
            }
            if let Some(ctx) = ctx {
                ctx.request_repaint();
            }
        }
    }

    /// Answers wake up the window from the start on; the kept local changes are read.
    pub fn attach(&mut self, ctx: &egui::Context) {
        self.ctx = Some(ctx.clone());
        self.sign_in.attach(ctx);
        if self.kept_store.is_none() {
            let store = Store::new("OpenECraft");
            store.load(ctx, KEPT_KEY);
            self.kept_store = Some(store);
        }
    }

    /// Keeps a copy of the local changes (`before`, edited from the server's `base`)
    /// that `after` replaces. Returns the targets kept.
    pub fn keep(&mut self, base: &Project, before: &Project, after: &Project, reason: KeptReason, time: String) -> Vec<Target> {
        let Some(sh) = &self.shared else { return Vec::new() };
        let Some(mut kept) = Kept::capture(base, before, after, reason) else { return Vec::new() };
        kept.server = self.url.clone();
        kept.name = sh.name.clone();
        kept.user = sh.session.user.clone();
        kept.revision = sh.revision;
        kept.time = time;
        let targets = kept.targets.clone();
        if collab::keep(&mut self.kept, kept) {
            self.store_kept();
        }
        targets
    }

    /// Indexes of the copies kept of the open shared project.
    pub fn kept_here(&self) -> Vec<usize> {
        let Some(sh) = &self.shared else { return Vec::new() };
        (0..self.kept.len()).filter(|i| self.kept[*i].of(&self.url, &sh.name)).collect()
    }

    /// Deletes a kept copy. After a save that left changes out, the project is saved
    /// once no copy of them is left (and nothing was changed since).
    pub fn discard_kept(&mut self, i: usize, session: &mut Session) {
        if i < self.kept.len() {
            self.kept.remove(i);
            self.store_kept();
        }
        if self.kept_here().is_empty() && self.unconfirmed == Some(session.history.state()) {
            session.mark_saved();
            self.unconfirmed = None;
        }
    }

    /// Takes back the local changes of `targets` (the server's version); they are kept.
    /// Returns the kept targets as text.
    pub fn revert(&mut self, prj: &mut Project, targets: &[Target], reason: KeptReason, time: String) -> Vec<String> {
        let Some(sh) = &self.shared else { return Vec::new() };
        let (base, after) = (sh.base.clone(), sh.revert(prj, targets));
        let before = std::mem::replace(prj, after);
        self.keep(&base, &before, prj, reason, time).iter().map(|t| Remote::describe(&before, *t)).collect()
    }

    /// Takes a newer state of the server (`sent`: the save it answers); undo steps
    /// keep the changes of others. Local changes it replaces are kept, and with them
    /// the project is not marked saved. Edits made after the save request stay
    /// unsaved; a state older than the one taken already is ignored.
    pub fn take_snapshot(&mut self, session: &mut Session, snapshot: Snapshot, sent: Option<&Sent>, time: String) -> Taken {
        let saved = sent.is_some();
        // Nothing changed since the request: the answer covers the whole project.
        let complete = sent.is_some_and(|s| s.project == session.prj);
        // After a save the saved state is the one sent: Undo back to an older saved
        // state while saving leaves changes.
        let clean = if saved { complete } else { !session.dirty() };
        let left_out = saved && !snapshot.ignored.is_empty();
        let Some(sh) = &mut self.shared else { return Taken::default() };
        if snapshot.revision < sh.revision {
            // A save answered after a newer state that holds it: saved if nothing differs.
            if saved && !left_out && sh.in_sync(&session.prj) && self.kept_here().is_empty() {
                session.mark_saved();
                self.unconfirmed = None;
            }
            return Taken::default();
        }
        // Released by the save but changed again meanwhile: still this session's change.
        let relock: Vec<Target> = sent
            .map(|s| {
                let since = collab::changes(&s.project, &session.prj);
                s.release.iter().copied().filter(|t| since.contains(t)).collect()
            })
            .unwrap_or_default();
        let base = sh.base.clone();
        let (lost, apply) = sh.update_keeping(snapshot, &relock);
        let before = session.prj.clone();
        session.history.rebase(&mut session.prj, apply);
        if !relock.is_empty() {
            self.lock(relock);
        }
        let reason = if left_out {
            KeptReason::NotSaved
        } else if !lost.is_empty() {
            KeptReason::Changed
        } else {
            KeptReason::NoLock
        };
        let kept = self.keep(&base, &before, &session.prj, reason, time);
        self.checked_rev = session.history.rev;
        let synced = saved && !left_out && self.shared.as_ref().is_some_and(|sh| sh.in_sync(&session.prj));
        if (clean || synced) && kept.is_empty() {
            session.mark_saved();
            self.unconfirmed = None;
        } else if complete {
            // Saved, but not all of it: discarding the copies confirms this state.
            self.unconfirmed = Some(session.history.state());
        }
        session.clamp_page();
        Taken { lost: !lost.is_empty(), kept: kept.iter().map(|t| Remote::describe(&before, *t)).collect() }
    }

    /// Takes the new session of the open project after the old one ended on the server
    /// (`Shared::reopen`): local changes stay unless another user changed the same
    /// target meanwhile; those are kept and replaced by the server's version. The local
    /// changes request their locks again.
    pub fn take_reopened(&mut self, session: &mut Session, opened: Opened, time: String) -> Taken {
        let Some(old) = self.shared.take() else { return Taken::default() };
        let clean = !session.dirty();
        self.rights = opened.rights.clone();
        self.access = None;
        let (shared, apply) = old.reopen(opened, &session.prj);
        let before = session.prj.clone();
        session.history.rebase(&mut session.prj, apply);
        self.shared = Some(shared);
        let kept = self.keep(&old.base, &before, &session.prj, KeptReason::Changed, time);
        self.unconfirmed = None;
        // The new session has the current project: nothing to load, no save queued
        // for the ended one.
        self.queued = None;
        self.refetch = None;
        // Changes without a lock request it (`App::check_locks`).
        self.checked_rev = 0;
        if clean && kept.is_empty() {
            session.mark_saved();
        }
        session.clamp_page();
        Taken { lost: !kept.is_empty(), kept: kept.iter().map(|t| Remote::describe(&before, *t)).collect() }
    }

    fn store_kept(&self) {
        if let Some(store) = self.kept_store.as_ref().filter(|_| self.kept_loaded) {
            store.store(KEPT_KEY, serde_json::to_vec(&self.kept).unwrap_or_default());
        }
    }

    /// Takes the stored copies once they are read; those kept meanwhile stay.
    fn load_kept(&mut self) {
        let Some(store) = &self.kept_store else { return };
        while let Some((key, value)) = store.poll() {
            if key != KEPT_KEY || self.kept_loaded {
                continue;
            }
            let mut stored: Vec<Kept> = value.and_then(|v| serde_json::from_slice(&v).ok()).unwrap_or_default();
            let since = std::mem::take(&mut self.kept);
            let changed = !since.is_empty();
            for k in since {
                collab::keep(&mut stored, k);
            }
            self.kept = stored;
            self.kept_loaded = true;
            if changed {
                self.store_kept();
            }
        }
    }

    /// The open shared project with the address of its server.
    pub fn project(&self) -> Option<crate::startup::ServerProject> {
        self.shared.as_ref().map(|sh| crate::startup::ServerProject { url: self.url.clone(), name: sh.name.clone() })
    }

    fn session_id(&self) -> String {
        self.shared.as_ref().map(|s| s.session.session.clone()).unwrap_or_default()
    }

    /// The user may do this with the open project. Older servers send no rights: then
    /// the role decides (none: token or open server, everything).
    pub fn may(&self, right: ProjectRight) -> bool {
        if self.rights.is_empty() {
            return self.role.is_none_or(|r| r.rights().contains(&right));
        }
        self.rights.contains(&right)
    }

    /// A viewer (or reviewer) may not change shared projects.
    pub fn viewer(&self) -> bool {
        self.shared.is_some() && !self.may(ProjectRight::EditPages) && !self.may(ProjectRight::EditProject)
    }

    /// Loads the grants on the open project.
    pub fn load_access(&mut self) {
        let Some(sh) = &self.shared else { return };
        let req = self.endpoint().project_access(&sh.name);
        let s = self.session_id();
        self.send(req, tr("Loading the project access…").into(), move |r| Reply::Access(s, decode_failure(r)));
    }

    /// Replaces the grants on the open project.
    pub fn save_access(&mut self, grants: &[Grant]) {
        let Some(sh) = &self.shared else { return };
        let req = self.endpoint().save_project_access(&sh.name, grants);
        let s = self.session_id();
        self.send(req, tr("Saving the project access…").into(), move |r| Reply::Access(s, decode_failure(r)));
    }

    /// Lists the projects of the server.
    pub fn list(&mut self) {
        if self.restore_before(Some(After::List)) { return; }
        let req = self.endpoint().list();
        self.send(req, tr("Connecting…").into(), |r| Reply::List(decode(r)));
    }

    pub fn open(&mut self, name: &str) {
        if self.restore_before(Some(After::Open(name.into()))) { return; }
        let req = self.endpoint().open(name, self.user.trim());
        let n = name.to_string();
        self.send(req, tr_fmt!("Opening {}…", name), move |r| Reply::Opened(n, decode(r).map(Box::new)));
    }

    /// Stores `prj` as a new project on the server and opens it.
    pub fn create(&mut self, name: &str, prj: &Project) {
        let req = self.endpoint().create(name, self.user.trim(), prj);
        let n = name.to_string();
        self.send(req, tr_fmt!("Uploading {}…", name), move |r| Reply::Opened(n, decode(r).map(Box::new)));
    }

    /// The open shared project's server answers its session.
    pub fn online(&self) -> bool {
        self.link == Link::Online
    }

    /// Saves what this session has locked, then releases `release`. While a save is
    /// under way the next one waits for its answer. Returns false without a session
    /// to save with (`status` tells why).
    pub fn save(&mut self, prj: &Project, release: Vec<Target>) -> bool {
        if self.shared.is_none() {
            return false;
        }
        self.status = match self.link {
            Link::Online | Link::Offline => {
                if let Some(sent) = self.begin_save(prj, release) {
                    self.send_save(sent);
                }
                return true;
            }
            Link::Ended => {
                self.due = true;
                tr("Not saved: the project session has ended. The project is being opened again with your changes; save again afterwards.").into()
            }
            Link::SignedOut => tr("Not saved: sign in to the project server again. Your changes are kept.").into(),
            Link::Refused => tr("Not saved: the project server no longer opens this project. Keep working on a local copy to save your changes.").into(),
        };
        false
    }

    /// Tries the connection again now: the status of the session, or opening the
    /// project again when the session ended or was refused.
    pub fn retry(&mut self) {
        if self.link == Link::Refused {
            self.link = Link::Ended;
        }
        self.due = true;
    }

    /// The save to send now, or none: queued behind the one under way.
    fn begin_save(&mut self, prj: &Project, release: Vec<Target>) -> Option<Sent> {
        self.shared.as_ref()?;
        if self.saving.is_some() {
            let queued = self.queued.get_or_insert_with(Vec::new);
            for t in release {
                if !queued.contains(&t) {
                    queued.push(t);
                }
            }
            return None;
        }
        let revision = self.shared.as_ref().map_or(0, |sh| sh.revision);
        let sent = Sent { session: self.session_id(), revision, project: prj.clone(), release };
        self.saving = Some(sent.session.clone());
        Some(sent)
    }

    fn send_save(&mut self, sent: Sent) {
        let Some(sh) = &self.shared else { return };
        let req = self.endpoint().save(&sh.name, &sh.session, &sent.project, sent.release.clone(), sent.revision);
        self.send(req, tr("Saving to the project server…").into(), move |r| Reply::Saved(Box::new(sent), decode_failure(r).map(Box::new)));
    }

    /// The queued save, once the one before it is answered; none if it has nothing
    /// left to do or no session to save with.
    fn next_save(&mut self, session: &Session) -> Option<Sent> {
        if self.saving.is_some() || !matches!(self.link, Link::Online | Link::Offline) {
            return None;
        }
        let release = self.queued.take()?;
        if release.is_empty() && !session.dirty() {
            return None;
        }
        self.begin_save(&session.prj, release)
    }

    /// Call after the answers were taken: sends the queued save, then loads a newer
    /// project announced while saving.
    pub fn flush(&mut self, session: &Session) {
        if let Some(sent) = self.next_save(session) {
            self.send_save(sent);
        }
        let newer = self.refetch.is_some_and(|r| self.shared.as_ref().is_some_and(|sh| r > sh.revision));
        if self.saving.is_none() && self.refetch.take().is_some() && newer {
            self.fetch();
        }
    }

    pub fn lock(&mut self, targets: Vec<Target>) {
        let endpoint = self.endpoint();
        let Some(sh) = &mut self.shared else { return };
        sh.requested.extend(targets.iter().copied());
        let req = endpoint.lock(&sh.name, &sh.session, targets.clone());
        let s = self.session_id();
        self.send(req, tr("Locking…").into(), move |r| Reply::Locked(s, targets, decode_failure(r)));
    }

    /// Releases locks without saving.
    pub fn release(&mut self, targets: Vec<Target>) {
        let Some(sh) = &self.shared else { return };
        let req = self.endpoint().release(&sh.name, &sh.session, targets);
        let s = self.session_id();
        self.send(req, tr("Releasing…").into(), move |r| Reply::Status(s, decode_failure(r)));
    }

    /// A cancelled operation may still have a lock request in flight.
    pub fn cancel_locks(&mut self, session: &str, targets: Vec<Target>) {
        if targets.is_empty() || session != self.session_id() { return; }
        self.cancelled_locks.push((session.into(), targets));
        if let Some(ctx) = &self.ctx { ctx.request_repaint(); }
    }

    /// Called after lock answers and snapshots were applied. Keep locks protecting
    /// local edits, and never release a lock before its request has finished.
    pub fn release_cancelled(&mut self, project: &Project) {
        if self.cancelled_locks.is_empty() { return; }
        let Some(sh) = &self.shared else { self.cancelled_locks.clear(); return; };
        let changed = openecraft_core::collab::changes(&sh.base, project);
        let fetching = self.fetching;
        let mut release = std::collections::BTreeSet::new();
        self.cancelled_locks.retain_mut(|(session, targets)| {
            if *session != sh.session.session { return false; }
            targets.retain(|target| {
                if sh.requested.contains(target) || fetching && sh.fresh.contains(target) { return true; }
                if sh.owns(*target) && !changed.contains(target) { release.insert(*target); }
                false
            });
            !targets.is_empty()
        });
        if !release.is_empty() { self.release(release.into_iter().collect()); }
    }

    /// Ends the session (releases its locks); the project stays open locally. A save
    /// under way is answered first: the session ends after it.
    pub fn close(&mut self) {
        let Some(sh) = self.shared.take() else { return };
        self.link = Link::Online;
        self.save_error = None;
        self.reopening = false;
        self.due = false;
        self.queued = None;
        self.refetch = None;
        let endpoint = self.endpoint();
        let req = endpoint.close(&sh.name, &sh.session);
        let close = self.track_close(endpoint);
        let status = tr_fmt!("Closed {} on the project server", sh.name);
        if self.saving.as_deref() == Some(sh.session.session.as_str()) {
            self.saving = None;
            self.closing.push((sh.session.session, req, status, close));
            return;
        }
        self.send(req, status, move |_| Reply::Closed(close));
    }

    fn fetch(&mut self) {
        let Some(sh) = &self.shared else { return };
        if self.fetching {
            return;
        }
        self.fetching = true;
        let endpoint = self.endpoint();
        let name = sh.name.clone();
        let session = sh.session.clone();
        let s = self.session_id();
        let receive = self.receiver(tr("Loading changes…").into());
        endpoint.fetch_session_snapshot(&name, &session, move |result| receive(Reply::Snapshot(s, result.map(Box::new))));
    }

    /// Takes the finished requests (of the open session) and keeps the session alive.
    pub fn poll(&mut self, ctx: &egui::Context) -> Vec<Reply> {
        self.ctx = Some(ctx.clone());
        self.load_kept();
        self.poll_credentials();
        let replies: Vec<Answer> = self.inbox.lock().map(|mut i| std::mem::take(&mut *i)).unwrap_or_default();
        let current = self.session_id();
        let mut out = Vec::new();
        for answer in replies {
            if let Reply::Closed(close) = &answer.reply { self.finish_close(*close); }
            if answer.generation != self.sign_in.generation {
                if self.kept_save(&answer) {
                    self.saving = None;
                    if let Reply::Saved(_, result) = &answer.reply {
                        self.status = Self::save_status(result);
                        self.save_error = result.as_ref().err().map(|e| e.error.clone());
                        if result.is_err() { self.queued = None; }
                    }
                    // The project still owns this save, but an old result must
                    // never restore or invalidate the current authentication.
                    out.push(answer.reply);
                } else { self.discard_answer(answer); }
                continue;
            }
            let mut reply = answer.reply;
            if answer.counted { self.pending = self.pending.saturating_sub(1); }
            let restored = matches!(reply, Reply::Restored(..));
            if let Reply::Restored(account, token, result) = reply {
                reply = Reply::Login(self.finish_restore(account, token, result));
            }
            match &reply {
                Reply::Status(..) => self.polling = false,
                Reply::Snapshot(..) => self.fetching = false,
                Reply::Reopened(..) => self.reopening = false,
                Reply::Saved(sent, result) => {
                    if self.saving.as_ref() == Some(&sent.session) {
                        self.saving = None;
                        if result.is_err() {
                            // Nothing more is sent (nor released) after a failed save.
                            self.queued = None;
                        }
                    }
                    if let Some(i) = self.closing.iter().position(|(s, _, _, _)| *s == sent.session) {
                        let (_, req, status, close) = self.closing.remove(i);
                        self.send(req, status, move |_| Reply::Closed(close));
                    }
                }
                _ => {}
            }
            if reply.session().is_some_and(|s| s != current) {
                continue;
            }
            self.status = match &reply {
                Reply::List(Ok(list)) if list.len() == 1 => tr("1 project on the server").to_string(),
                Reply::List(Ok(list)) => tr_fmt!("{} projects on the server", list.len()),
                Reply::Opened(name, Ok(_)) => tr_fmt!("Opened {} from the project server", name),
                Reply::Reopened(_, Ok(_)) => tr("Opened the project again on the project server: a new session").into(),
                Reply::Saved(_, result) => Self::save_status(result),
                Reply::Login(Ok(r)) => match r.user.role {
                    Some(role) => tr_fmt!("Signed in as {} ({})", r.user.name, tr(role.title())),
                    None => tr_fmt!("Signed in as {}", r.user.name),
                },
                Reply::Login(Err(e)) => tr_fmt!("Project server: {}", e),
                Reply::Access(_, Ok(a)) => tr_fmt!("{} grants on the project", a.grants.len()),
                // The "Locking…" of the request is over.
                Reply::Locked(_, _, Ok(_)) => String::new(),
                Reply::SignedOut(Ok(())) => tr("Signed out on the project server.").into(),
                Reply::SignedOut(Err(_)) => tr("Signed out on this device. The server sign-in could not be ended; check the connection.").into(),
                Reply::List(Err(e)) | Reply::Opened(_, Err(e)) => tr_fmt!("Project server: {}", e),
                Reply::Access(_, Err(e))
                | Reply::Reopened(_, Err(e))
                | Reply::Snapshot(_, Err(e))
                | Reply::Status(_, Err(e))
                | Reply::Locked(_, _, Err(e)) => tr_fmt!("Project server: {}", e),
                _ => std::mem::take(&mut self.status),
            };
            if let Reply::Saved(_, r) = &reply {
                self.save_error = r.as_ref().err().map(|e| e.error.clone());
            }
            if let Reply::Opened(_, Ok(_)) = &reply {
                self.link = Link::Online;
                self.save_error = None;
            }
            self.follow(&reply);
            if let Reply::List(Ok(list)) = &reply {
                self.projects = list.clone();
            }
            if let Reply::Login(Ok(r)) = &reply {
                self.accept_login(r, restored);
            }
            if let Reply::Opened(_, Ok(o)) = &reply {
                self.rights = o.rights.clone();
                self.access = None;
            }
            if let Reply::Access(_, Ok(a)) = &reply {
                self.access = Some(a.clone());
            }
            out.push(reply);
        }
        if self.shared.is_some() {
            let now = ctx.input(|i| i.time);
            let due = self.due || now - self.last_status >= openecraft_core::collab::HEARTBEAT_SECS;
            match self.link {
                Link::Ended if due && !self.reopening => self.reopen(now),
                Link::Online | Link::Offline if due && !self.polling => self.heartbeat(now),
                Link::SignedOut if self.due && !self.polling => self.heartbeat(now),
                _ => {}
            }
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(openecraft_core::collab::HEARTBEAT_SECS));
        }
        out
    }

    /// The state of the connection after an answer of the open session. An answer
    /// shows the server is back; the error of the connection is no longer shown then,
    /// unless a save failed (its changes are not on the server).
    fn follow(&mut self, reply: &Reply) {
        let Some(outcome) = reply.outcome() else { return };
        let status = matches!(reply, Reply::Status(..));
        match outcome {
            Ok(()) if self.link != Link::Online => {
                self.link = Link::Online;
                // Changes made meanwhile request their locks.
                self.checked_rev = 0;
                if matches!(reply, Reply::Status(..) | Reply::Locked(..) | Reply::Snapshot(..)) {
                    self.status = match &self.save_error {
                        Some(_) => tr("Connection to the project server restored. The last save failed: save again.").into(),
                        None => tr("Connection to the project server restored").into(),
                    };
                }
            }
            Ok(()) => {}
            Err(e) => match e.why {
                Failure::Unreachable if self.link == Link::Online => self.link = Link::Offline,
                Failure::Unreachable => {}
                Failure::SessionEnded => {
                    if self.link != Link::Ended {
                        self.due = true;
                    }
                    self.link = Link::Ended;
                }
                Failure::SignedOut => {
                    self.link = Link::SignedOut;
                    self.sign_in.signed_in = false;
                }
                Failure::Refused if status || matches!(reply, Reply::Reopened(..)) => self.link = Link::Refused,
                Failure::Refused => {}
            },
        }
    }

    /// Opens the open project again, its session ended on the server.
    fn reopen(&mut self, now: f64) {
        if self.saving.is_some() { return; }
        let Some(sh) = &self.shared else { return };
        self.reopening = true;
        self.due = false;
        self.last_status = now;
        let req = self.endpoint().open(&sh.name, &sh.session.user);
        let s = self.session_id();
        self.send(req, tr("The project session has ended: opening the project again…").into(), move |r| Reply::Reopened(s, decode_failure(r).map(Box::new)));
    }

    fn heartbeat(&mut self, now: f64) {
        let Some(sh) = &self.shared else { return };
        self.polling = true;
        self.due = false;
        self.last_status = now;
        let req = self.endpoint().status(&sh.name, &sh.session);
        let s = self.session_id();
        let status = std::mem::take(&mut self.status);
        self.send(req, status, move |r| Reply::Status(s, decode_failure(r)));
    }

    /// Answer to a status or lock request: loads the newer project if there is one.
    pub fn take_status(&mut self, status: Status, granted: &[Target]) {
        let Some(sh) = &mut self.shared else { return };
        let revision = status.revision;
        if !sh.status(status, granted) {
            return;
        }
        if self.saving.is_some() {
            // The answer of the save brings it, or it is loaded after the save.
            self.refetch = Some(self.refetch.unwrap_or_default().max(revision));
        } else {
            self.fetch();
        }
    }

    /// Text for a target, e.g. "page 3".
    pub fn describe(prj: &Project, target: Target) -> String {
        match target {
            Target::Project => tr("the project data (page list, devices, settings)").into(),
            Target::Page(id) => match prj.pages.iter().find(|p| p.id == id) {
                Some(p) => tr_fmt!("page {}", openecraft_core::structure::page_full_name(p)),
                None => tr_fmt!("page {}", id),
            },
        }
    }
}

#[cfg(test)]
mod tests;
