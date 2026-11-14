use super::*;
use openecraft_core::collab::{Opened, Session};

fn failed(why: Failure, error: &str) -> Failed {
    Failed { why, error: error.into() }
}

fn offline() -> Failed {
    failed(Failure::Unreachable, "connection failed: the server refuses the connection")
}

/// Takes `replies` as answers of requests sent before.
fn answer(r: &mut Remote, ctx: &egui::Context, replies: Vec<Reply>) -> Vec<Reply> {
    r.pending += replies.len();
    r.inbox.lock().unwrap().extend(replies);
    r.poll(ctx)
}
use openecraft_core::model::{Page, PageKind};

fn opened() -> Remote {
    let mut r = Remote::default();
    let opened = Opened { session: Session { user: "anna".into(), session: "s1".into() }, first_id: 100, snapshot: Snapshot::default(), rights: Vec::new() };
    r.shared = Some(Shared::open("plant", opened).0);
    r
}

#[test]
fn answers_of_old_sessions_are_dropped() {
    let mut r = opened();
    let ctx = egui::Context::default();
    r.pending = 3;
    r.inbox.lock().unwrap().extend([
        Reply::Status("old".into(), Ok(Status::default())),
        Reply::Status("s1".into(), Ok(Status::default())),
        Reply::List(Ok(vec![ProjectSummary { name: "p".into(), ..Default::default() }])),
    ]);
    let replies = r.poll(&ctx);
    assert_eq!(replies.len(), 2);
    assert!(matches!(replies[0], Reply::Status(ref s, _) if s == "s1"));
    assert_eq!(r.projects.len(), 1);
}

#[test]
fn errors_are_shown() {
    let mut r = opened();
    let ctx = egui::Context::default();
    r.pending = 1;
    r.inbox.lock().unwrap().push(Reply::Locked("s1".into(), vec![Target::Project], Err(failed(Failure::Refused, "project data is locked by ben"))));
    r.poll(&ctx);
    assert!(r.status.contains("locked by ben"), "{}", r.status);
}

#[test]
fn lock_requests_are_remembered() {
    let mut r = opened();
    r.url = "http://127.0.0.1:9".into();
    r.lock(vec![Target::Page(4)]);
    assert!(r.shared.as_ref().unwrap().requested.contains(&Target::Page(4)));
    assert!(r.busy());
}

#[test]
fn targets_are_described() {
    let mut prj = Project::default();
    prj.pages.push(Page::new(42, "7", "", PageKind::Schematic));
    assert!(Remote::describe(&prj, Target::Page(42)).contains('7'));
    assert!(Remote::describe(&prj, Target::Page(5)).contains('5'));
}

#[test]
fn rights_decide_what_may_be_changed() {
    let mut r = opened();
    assert!(!r.viewer(), "token or open server: everything");
    r.role = Some(ProjectRole::Viewer);
    assert!(r.viewer(), "older servers: the role decides");
    r.role = None;
    r.rights = vec![ProjectRight::View, ProjectRight::Review];
    assert!(r.viewer() && r.may(ProjectRight::Review) && !r.may(ProjectRight::ManageAccess));
    r.rights.push(ProjectRight::EditPages);
    assert!(!r.viewer());

    // Opening takes the rights of the project; grants come with their own reply.
    let ctx = egui::Context::default();
    let opened = Opened { rights: vec![ProjectRight::View, ProjectRight::ManageAccess], ..Default::default() };
    let access = ProjectAccess { grants: vec![Grant::default()], roles: vec!["viewer".into()] };
    r.pending = 2;
    r.inbox.lock().unwrap().extend([Reply::Opened("p".into(), Ok(Box::new(opened))), Reply::Access("s1".into(), Ok(access.clone()))]);
    r.poll(&ctx);
    assert_eq!(r.rights, [ProjectRight::View, ProjectRight::ManageAccess]);
    assert_eq!(r.access, Some(access));
}

#[test]
fn a_failed_status_marks_the_connection_lost() {
    let mut r = opened();
    r.url = "http://srv:8091".into();
    let ctx = egui::Context::default();
    r.pending = 1;
    r.inbox.lock().unwrap().push(Reply::Status("s1".into(), Err(offline())));
    r.poll(&ctx);
    assert_eq!(r.link, Link::Offline);
    assert!(r.status.contains("refuses the connection"));
    r.pending = 1;
    r.inbox.lock().unwrap().push(Reply::Status("s1".into(), Ok(Status::default())));
    r.poll(&ctx);
    assert!(r.online());
    assert!(!r.status.contains("refuses the connection"), "the error of the connection is no longer shown: {}", r.status);
    assert_eq!(r.checked_rev, 0, "changes made meanwhile request their locks");
    assert_eq!(r.project(), Some(crate::startup::ServerProject { url: "http://srv:8091".into(), name: "plant".into() }));
}

fn cancelled() -> (Remote, Project) {
    let mut r = opened();
    r.url = "http://127.0.0.1:9".into();
    let project = Project { pages: vec![Page::new(4, "1", "original", PageKind::Schematic)], ..Default::default() };
    r.shared.as_mut().unwrap().base = project.clone();
    (r, project)
}

#[test]
fn cancelling_waits_for_the_lock_answer_before_releasing() {
    use openecraft_core::collab::Lock;
    let (mut r, project) = cancelled();
    let target = Target::Page(4);
    r.shared.as_mut().unwrap().requested.insert(target);
    r.cancel_locks("s1", vec![target]);
    r.release_cancelled(&project);
    assert!(!r.busy(), "sending release first could leave a later lock held forever");
    assert_eq!(r.cancelled_locks.len(), 1);
    r.take_status(Status { revision: 0, locks: vec![Lock { target, user: "anna".into(), session: "s1".into() }] }, &[target]);
    r.release_cancelled(&project);
    assert!(r.busy(), "the acquired unused lock is now released");
    assert!(r.cancelled_locks.is_empty());
}

#[test]
fn cancellation_never_releases_locks_protecting_other_local_edits() {
    use openecraft_core::collab::Lock;
    let (mut r, mut project) = cancelled();
    let target = Target::Page(4);
    r.shared.as_mut().unwrap().locks.push(Lock { target, user: "anna".into(), session: "s1".into() });
    project.pages[0].description = "unrelated local edit".into();
    r.cancel_locks("s1", vec![target]);
    r.release_cancelled(&project);
    assert!(!r.busy());
    assert!(r.shared.as_ref().unwrap().owns(target));
    assert!(r.cancelled_locks.is_empty());
}

#[test]
fn failed_requests_and_old_sessions_cannot_release_current_locks() {
    let (mut r, project) = cancelled();
    r.cancel_locks("old", vec![Target::Page(4)]);
    assert!(r.cancelled_locks.is_empty());
    r.shared.as_mut().unwrap().requested.insert(Target::Page(4));
    r.cancel_locks("s1", vec![Target::Page(4)]);
    r.shared.as_mut().unwrap().denied(&[Target::Page(4)]);
    r.release_cancelled(&project);
    assert!(!r.busy());
    assert!(r.cancelled_locks.is_empty());
    r.cancel_locks("s1", vec![Target::Page(4)]);
    r.shared.as_mut().unwrap().session.session = "s2".into();
    r.release_cancelled(&project);
    assert!(!r.busy() && r.cancelled_locks.is_empty());
}

#[test]
fn a_failed_refresh_does_not_hold_unused_fresh_locks_forever() {
    use openecraft_core::collab::Lock;
    let (mut r, project) = cancelled();
    let target = Target::Page(4);
    let sh = r.shared.as_mut().unwrap();
    sh.locks.push(Lock { target, user: "anna".into(), session: "s1".into() });
    sh.fresh.insert(target);
    r.fetching = true;
    r.cancel_locks("s1", vec![target]);
    r.release_cancelled(&project);
    assert!(!r.busy() && !r.cancelled_locks.is_empty());
    r.fetching = false;
    r.release_cancelled(&project);
    assert!(r.busy() && r.cancelled_locks.is_empty());
}

/// A project server in memory, with the rules of the save route (`project-server`).
struct Server {
    revision: u64,
    stored: Vec<Project>,
    locks: Vec<openecraft_core::collab::Lock>,
    next_session: u64,
}

impl Server {
    /// Two pages at revision 1.
    fn new() -> Server {
        let mut prj = Project::default();
        prj.pages.clear();
        prj.pages.push(Page::new(1, "1", "Supply", PageKind::Schematic));
        prj.pages.push(Page::new(2, "2", "Motors", PageKind::Schematic));
        prj.next_id = 10;
        prj.normalize();
        Server { revision: 1, stored: vec![prj], locks: Vec::new(), next_session: 0 }
    }

    fn project(&self) -> &Project {
        self.stored.last().unwrap()
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot { revision: self.revision, project: self.project().clone(), locks: self.locks.clone(), ignored: Vec::new() }
    }

    fn open(&mut self, user: &str) -> (Remote, crate::session::Session) {
        self.next_session += 1;
        let opened = Opened {
            session: Session { user: user.into(), session: format!("s{}", self.next_session) },
            first_id: self.next_session * openecraft_core::collab::ID_BLOCK,
            snapshot: self.snapshot(),
            rights: Vec::new(),
        };
        let mut r = Remote { url: "http://srv:8091".into(), ..Remote::default() };
        let (shared, prj) = Shared::open("plant", opened);
        r.shared = Some(shared);
        let session = crate::session::Session::new(prj, None);
        r.checked_rev = session.history.rev;
        (r, session)
    }

    fn lock(&mut self, r: &Remote, targets: &[Target]) -> Result<Status, String> {
        let s = &r.shared.as_ref().unwrap().session;
        if let Some(l) = self.locks.iter().find(|l| targets.contains(&l.target) && l.session != s.session) {
            return Err(format!("locked by {}", l.user));
        }
        for t in targets {
            self.locks.push(openecraft_core::collab::Lock { target: *t, user: s.user.clone(), session: s.session.clone() });
        }
        Ok(Status { revision: self.revision, locks: self.locks.clone() })
    }

    /// A save of `prj` edited from the client's base revision; `strict` saves nothing
    /// if a change would be left out.
    fn save(&mut self, r: &Remote, prj: &Project, release: &[Target], strict: bool) -> Result<Snapshot, String> {
        self.save_from(r, r.shared.as_ref().unwrap().revision, prj, release, strict)
    }

    /// Processes a save request as it was sent.
    fn answer(&mut self, r: &Remote, sent: &Sent) -> Snapshot {
        self.save_from(r, sent.revision, &sent.project, &sent.release, false).unwrap()
    }

    fn save_from(&mut self, r: &Remote, revision: u64, prj: &Project, release: &[Target], strict: bool) -> Result<Snapshot, String> {
        use openecraft_core::collab::{changes, merge, save_plan, Owned};
        let sh = r.shared.as_ref().unwrap();
        let base = &self.stored[revision as usize - 1];
        let plan = save_plan(&changes(base, prj), &changes(base, self.project()), &self.locks, &sh.session.session, |_| true);
        if strict && !plan.ignored.is_empty() {
            return Err("not saved".into());
        }
        for t in &plan.lock {
            self.locks.push(openecraft_core::collab::Lock { target: *t, user: sh.session.user.clone(), session: sh.session.session.clone() });
        }
        let merged = merge(self.project(), prj, &Owned::of(&self.locks, &sh.session.session));
        self.stored.push(merged);
        self.revision += 1;
        self.locks.retain(|l| !(l.session == sh.session.session && release.contains(&l.target)));
        Ok(Snapshot { ignored: plan.ignored, ..self.snapshot() })
    }
}

fn edit(session: &mut crate::session::Session, page: usize, text: &str) {
    session.prj.pages[page].description = text.into();
    assert!(session.commit(true));
}

/// Saves as the editor does and takes the answer.
fn save(server: &mut Server, r: &mut Remote, session: &mut crate::session::Session, release: &[Target]) -> Taken {
    let sent = r.begin_save(&session.prj, release.to_vec()).expect("no save under way");
    let snapshot = server.answer(r, &sent);
    deliver(r, session, Reply::Saved(Box::new(sent), Ok(Box::new(snapshot)))).0
}

/// Takes an answer as the editor does (`App::poll_remote`): returns what it did and
/// the queued save it sends next.
fn deliver(r: &mut Remote, session: &mut crate::session::Session, reply: Reply) -> (Taken, Option<Sent>) {
    r.pending += 1;
    r.inbox.lock().unwrap().push(reply);
    let mut taken = Taken::default();
    for reply in r.poll(&egui::Context::default()) {
        match reply {
            Reply::Saved(sent, Ok(snapshot)) => taken = r.take_snapshot(session, *snapshot, Some(&sent), "now".into()),
            Reply::Snapshot(_, Ok(snapshot)) => taken = r.take_snapshot(session, *snapshot, None, "now".into()),
            _ => {}
        }
    }
    (taken, r.next_save(session))
}

fn sent(session: &str) -> Box<Sent> {
    Box::new(Sent { session: session.into(), ..Default::default() })
}

/// Anna saves page 1 while Ben, still on the revision both opened, changes page 1 and
/// the free page 2.
fn partial_save() -> (Server, Remote, crate::session::Session) {
    let mut server = Server::new();
    let (mut anna, mut a) = server.open("anna");
    let (mut ben, mut b) = server.open("ben");
    edit(&mut a, 0, "Anna");
    save(&mut server, &mut anna, &mut a, &[Target::Page(1)]);
    assert!(!a.dirty());
    edit(&mut b, 0, "Ben");
    edit(&mut b, 1, "Ben 2");
    let taken = save(&mut server, &mut ben, &mut b, &[]);
    assert_eq!(taken.kept.len(), 1, "{taken:?}");
    (server, ben, b)
}

#[test]
fn a_partial_save_keeps_the_left_out_change_and_stays_unsaved() {
    let (server, mut ben, mut b) = partial_save();
    assert_eq!(server.project().pages[0].description, "Anna", "the foreign change is not overwritten");
    assert_eq!(server.project().pages[1].description, "Ben 2");
    assert_eq!(b.prj.pages[0].description, "Anna");
    assert_eq!(b.prj.pages[1].description, "Ben 2");
    assert!(b.dirty(), "the left out change is not marked as saved");
    let here = ben.kept_here();
    assert_eq!(here.len(), 1);
    let kept = &ben.kept[here[0]];
    assert_eq!((kept.reason, kept.targets.clone(), kept.revision), (KeptReason::NotSaved, vec![Target::Page(1)], 3));
    assert_eq!(kept.project.pages[0].description, "Ben");

    // Undo and redo keep the foreign change.
    b.undo();
    assert_eq!(b.prj.pages[0].description, "Anna");
    assert_eq!(b.prj.pages[1].description, "Motors");
    b.redo();
    assert_eq!(b.prj.pages[1].description, "Ben 2");

    // Discarding the copy confirms the rest of the save.
    ben.discard_kept(here[0], &mut b);
    assert!(ben.kept.is_empty());
    assert!(!b.dirty());
}

#[test]
fn discarding_the_copy_after_further_edits_keeps_them_unsaved() {
    let (_, mut ben, mut b) = partial_save();
    edit(&mut b, 1, "later");
    ben.discard_kept(0, &mut b);
    assert!(b.dirty());
}

#[test]
fn a_kept_change_is_taken_over_again_on_purpose() {
    let (mut server, mut ben, mut b) = partial_save();
    let kept = ben.kept[0].clone();
    let targets = kept.differing(&b.prj);
    b.prj = openecraft_core::collab::restore(&b.prj, &kept.project, &targets);
    assert!(b.commit(true));
    // The editor requests the lock of the change, then saves it.
    let sh = ben.shared.as_ref().unwrap();
    assert_eq!(sh.unlocked_changes(&b.prj), vec![Target::Page(1)]);
    let status = server.lock(&ben, &targets).unwrap();
    ben.take_status(status, &targets);
    let taken = save(&mut server, &mut ben, &mut b, &[]);
    assert!(taken.kept.is_empty());
    assert_eq!(server.project().pages[0].description, "Ben");
    assert!(!b.dirty());
    assert!(ben.kept[0].differing(&b.prj).is_empty(), "the copy is now the same as the project");
}

#[test]
fn a_complete_save_is_clean_and_keeps_nothing() {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    edit(&mut b, 1, "Ben 2");
    let taken = save(&mut server, &mut ben, &mut b, &[]);
    assert_eq!(taken, Taken::default());
    assert!(!b.dirty());
    assert!(ben.kept.is_empty() && ben.unconfirmed.is_none());
}

#[test]
fn a_strict_save_or_transport_error_changes_nothing() {
    let mut server = Server::new();
    let (mut anna, mut a) = server.open("anna");
    let (mut ben, mut b) = server.open("ben");
    edit(&mut a, 0, "Anna");
    save(&mut server, &mut anna, &mut a, &[Target::Page(1)]);
    edit(&mut b, 0, "Ben");
    let before = (server.revision, b.prj.clone());
    assert!(server.save(&ben, &b.prj, &[], true).is_err());
    assert_eq!(server.revision, before.0);
    let ctx = egui::Context::default();
    ben.pending = 1;
    ben.inbox.lock().unwrap().push(Reply::Saved(sent("s2"), Err(offline())));
    assert_eq!(ben.poll(&ctx).len(), 1);
    assert!(ben.status.contains("refuses the connection"));
    assert_eq!(ben.link, Link::Offline, "a failed save alone shows the lost connection");
    assert_eq!(b.prj, before.1);
    assert!(b.dirty() && ben.kept.is_empty());
}

#[test]
fn a_lock_granted_after_a_foreign_change_keeps_the_local_one() {
    let mut server = Server::new();
    let (mut anna, mut a) = server.open("anna");
    let (mut ben, mut b) = server.open("ben");
    // Ben edits page 1 and asks for its lock while Anna holds it.
    edit(&mut b, 0, "Ben");
    ben.shared.as_mut().unwrap().requested.insert(Target::Page(1));
    edit(&mut a, 0, "Anna");
    server.lock(&anna, &[Target::Page(1)]).unwrap();
    save(&mut server, &mut anna, &mut a, &[Target::Page(1)]);
    // The lock is granted on a newer revision: the newer project is loaded.
    let status = server.lock(&ben, &[Target::Page(1)]).unwrap();
    assert!(ben.shared.as_mut().unwrap().status(status, &[Target::Page(1)]));
    let taken = ben.take_snapshot(&mut b, server.snapshot(), None, "now".into());
    assert!(taken.lost);
    assert_eq!(b.prj.pages[0].description, "Anna");
    assert_eq!(ben.kept[0].reason, KeptReason::Changed);
    assert_eq!(ben.kept[0].project.pages[0].description, "Ben");
    assert!(b.dirty());
}

#[test]
fn a_pending_lock_request_keeps_the_change_on_a_refresh() {
    let mut server = Server::new();
    let (mut anna, mut a) = server.open("anna");
    let (mut ben, mut b) = server.open("ben");
    edit(&mut b, 0, "Ben");
    ben.shared.as_mut().unwrap().requested.insert(Target::Page(1));
    edit(&mut a, 1, "Anna 2");
    save(&mut server, &mut anna, &mut a, &[]);
    let taken = ben.take_snapshot(&mut b, server.snapshot(), None, "now".into());
    assert_eq!(taken, Taken::default());
    assert_eq!(b.prj.pages[0].description, "Ben");
    assert_eq!(b.prj.pages[1].description, "Anna 2");
}

#[test]
fn a_lost_lock_keeps_the_change() {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    let status = server.lock(&ben, &[Target::Page(2)]).unwrap();
    ben.take_status(status, &[Target::Page(2)]);
    edit(&mut b, 1, "Ben 2");
    // The session expired on the server: its locks are gone.
    server.locks.clear();
    let taken = ben.take_snapshot(&mut b, server.snapshot(), None, "now".into());
    assert!(!taken.lost && taken.kept.len() == 1);
    assert_eq!(b.prj.pages[1].description, "Motors");
    assert_eq!(ben.kept[0].reason, KeptReason::NoLock);
    assert_eq!(ben.kept[0].project.pages[1].description, "Ben 2");
}

#[test]
fn a_refused_lock_keeps_the_change() {
    let mut server = Server::new();
    let (anna, _) = server.open("anna");
    let (mut ben, mut b) = server.open("ben");
    server.lock(&anna, &[Target::Page(1)]).unwrap();
    edit(&mut b, 0, "Ben");
    assert!(server.lock(&ben, &[Target::Page(1)]).is_err());
    let kept = ben.revert(&mut b.prj, &[Target::Page(1)], KeptReason::Refused, "now".into());
    assert_eq!(kept.len(), 1);
    assert_eq!(b.prj.pages[0].description, "Supply");
    assert_eq!(ben.kept[0].project.pages[0].description, "Ben");
}

#[test]
fn a_partial_save_is_told_apart() {
    let mut r = opened();
    let ctx = egui::Context::default();
    r.pending = 2;
    r.inbox.lock().unwrap().push(Reply::Saved(sent("s1"), Ok(Box::new(Snapshot { revision: 7, ignored: vec![Target::Page(1)], ..Default::default() }))));
    r.poll(&ctx);
    assert!(r.status.starts_with("Partly saved") && r.status.contains('7'), "{}", r.status);
    r.inbox.lock().unwrap().push(Reply::Saved(sent("s1"), Ok(Box::new(Snapshot { revision: 8, ..Default::default() }))));
    r.poll(&ctx);
    assert!(r.status.starts_with("Saved") && r.status.contains('8'), "{}", r.status);
}

#[test]
fn kept_changes_are_stored_and_found_after_reopening() {
    let place = std::env::temp_dir().join(format!("openecraft-kept-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&place);
    let ctx = egui::Context::default();
    let attach = |r: &mut Remote| {
        let store = Store::at(place.display().to_string());
        store.load(&ctx, KEPT_KEY);
        r.kept_store = Some(store);
    };
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    attach(&mut ben);
    // A copy made before the stored ones were read is not lost when they arrive.
    let (mut anna, mut a) = server.open("anna");
    edit(&mut a, 0, "Anna");
    save(&mut server, &mut anna, &mut a, &[Target::Page(1)]);
    edit(&mut b, 0, "Ben");
    save(&mut server, &mut ben, &mut b, &[]);
    assert_eq!(ben.kept.len(), 1);
    ben.poll(&ctx);
    assert!(ben.kept_loaded && ben.kept.len() == 1);

    // The app closes; it opens the project again in a new session.
    let (mut again, _) = server.open("ben");
    attach(&mut again);
    again.poll(&ctx);
    assert_eq!(again.kept.len(), 1);
    assert_eq!(again.kept_here(), vec![0]);
    assert_eq!(again.kept[0].project.pages[0].description, "Ben");
    again.discard_kept(0, &mut crate::session::Session::new(Project::default(), None));
    let (mut third, _) = server.open("ben");
    attach(&mut third);
    third.poll(&ctx);
    assert!(third.kept.is_empty());
    let _ = std::fs::remove_dir_all(&place);
}

/// The answer to `sent`, delivered later.
fn answered(server: &mut Server, r: &Remote, sent: Sent) -> Reply {
    let snapshot = server.answer(r, &sent);
    Reply::Saved(Box::new(sent), Ok(Box::new(snapshot)))
}

#[test]
fn a_save_answer_does_not_mark_later_edits_saved() {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    edit(&mut b, 1, "A");
    let first = ben.begin_save(&b.prj, Vec::new()).unwrap();
    edit(&mut b, 1, "B");
    assert!(ben.begin_save(&b.prj, Vec::new()).is_none(), "one save at a time");
    let reply = answered(&mut server, &ben, first);
    let (taken, next) = deliver(&mut ben, &mut b, reply);
    assert_eq!(taken, Taken::default());
    assert_eq!(server.project().pages[1].description, "A");
    assert_eq!(b.prj.pages[1].description, "B");
    assert!(b.dirty(), "B was edited after the request");
    let next = next.expect("the queued save follows");
    assert_eq!((next.project.pages[1].description.as_str(), next.revision), ("B", 2));
    let reply = answered(&mut server, &ben, next);
    let (_, after) = deliver(&mut ben, &mut b, reply);
    assert!(after.is_none());
    assert_eq!(server.project().pages[1].description, "B");
    assert!(!b.dirty());
}

#[test]
fn repeated_saves_without_changes_send_one_request() {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    edit(&mut b, 1, "A");
    let first = ben.begin_save(&b.prj, Vec::new()).unwrap();
    assert!(ben.begin_save(&b.prj, Vec::new()).is_none());
    assert!(ben.begin_save(&b.prj, Vec::new()).is_none());
    let reply = answered(&mut server, &ben, first);
    let (_, next) = deliver(&mut ben, &mut b, reply);
    assert!(next.is_none() && ben.queued.is_none() && ben.saving.is_none());
    assert_eq!(server.revision, 2);
    assert!(!b.dirty());
}

#[test]
fn undo_while_saving_stays_unsaved() {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    edit(&mut b, 1, "A");
    let sent = ben.begin_save(&b.prj, Vec::new()).unwrap();
    b.undo();
    let reply = answered(&mut server, &ben, sent);
    deliver(&mut ben, &mut b, reply);
    assert_eq!(server.project().pages[1].description, "A");
    assert_eq!(b.prj.pages[1].description, "Motors");
    assert!(b.dirty(), "the undone state differs from the saved one");
    save(&mut server, &mut ben, &mut b, &[]);
    assert_eq!(server.project().pages[1].description, "Motors");
    assert!(!b.dirty());
}

#[test]
fn an_older_snapshot_does_not_roll_the_revision_back() {
    let mut server = Server::new();
    let (mut anna, mut a) = server.open("anna");
    let (mut ben, mut b) = server.open("ben");
    edit(&mut a, 0, "one");
    save(&mut server, &mut anna, &mut a, &[]);
    let old = server.snapshot();
    edit(&mut a, 0, "two");
    save(&mut server, &mut anna, &mut a, &[]);
    deliver(&mut ben, &mut b, Reply::Snapshot("s2".into(), Ok(Box::new(server.snapshot()))));
    assert_eq!((ben.shared.as_ref().unwrap().revision, b.prj.pages[0].description.as_str()), (3, "two"));
    let rev = b.history.rev;
    let (taken, _) = deliver(&mut ben, &mut b, Reply::Snapshot("s2".into(), Ok(Box::new(old))));
    assert_eq!(taken, Taken::default());
    assert_eq!((ben.shared.as_ref().unwrap().revision, b.prj.pages[0].description.as_str()), (3, "two"));
    assert_eq!(b.history.rev, rev, "nothing was taken");
    assert!(!b.dirty());
}

#[test]
fn a_save_answered_after_a_newer_state_is_saved_when_nothing_differs() {
    let mut server = Server::new();
    let (mut anna, mut a) = server.open("anna");
    let (mut ben, mut b) = server.open("ben");
    edit(&mut b, 1, "Ben 2");
    let sent = ben.begin_save(&b.prj, Vec::new()).unwrap();
    let reply = answered(&mut server, &ben, sent);
    edit(&mut a, 0, "Anna");
    save(&mut server, &mut anna, &mut a, &[]);
    // A refresh sent before the save is answered first, with both changes.
    deliver(&mut ben, &mut b, Reply::Snapshot("s2".into(), Ok(Box::new(server.snapshot()))));
    assert!(b.dirty());
    let (taken, next) = deliver(&mut ben, &mut b, reply);
    assert!(taken.kept.is_empty() && next.is_none());
    assert_eq!(ben.shared.as_ref().unwrap().revision, 3);
    assert_eq!((b.prj.pages[0].description.as_str(), b.prj.pages[1].description.as_str()), ("Anna", "Ben 2"));
    assert!(!b.dirty());
}

#[test]
fn save_and_release_keeps_an_edit_made_while_saving() {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    ben.url = "http://127.0.0.1:9".into();
    let status = server.lock(&ben, &[Target::Page(2)]).unwrap();
    ben.take_status(status, &[Target::Page(2)]);
    edit(&mut b, 1, "first");
    let sent = ben.begin_save(&b.prj, vec![Target::Page(2)]).unwrap();
    edit(&mut b, 1, "second");
    let reply = answered(&mut server, &ben, sent);
    assert!(server.locks.is_empty(), "released by the save");
    let (taken, _) = deliver(&mut ben, &mut b, reply);
    assert!(taken.kept.is_empty(), "{taken:?}");
    assert_eq!(server.project().pages[1].description, "first");
    assert_eq!(b.prj.pages[1].description, "second");
    assert!(b.dirty());
    assert!(ben.shared.as_ref().unwrap().requested.contains(&Target::Page(2)), "its lock is requested again");
    assert!(ben.busy());
}

#[test]
fn a_failed_save_sends_and_releases_nothing_more() {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    let status = server.lock(&ben, &[Target::Page(2)]).unwrap();
    ben.take_status(status, &[Target::Page(2)]);
    edit(&mut b, 1, "first");
    let sent = ben.begin_save(&b.prj, Vec::new()).unwrap();
    edit(&mut b, 1, "second");
    assert!(ben.begin_save(&b.prj, vec![Target::Page(2)]).is_none());
    let (_, next) = deliver(&mut ben, &mut b, Reply::Saved(Box::new(sent), Err(offline())));
    assert!(next.is_none() && ben.queued.is_none() && ben.saving.is_none());
    assert!(b.dirty());
    assert!(ben.shared.as_ref().unwrap().owns(Target::Page(2)) && !server.locks.is_empty());
    // The next save is sent at once.
    assert!(ben.begin_save(&b.prj, vec![Target::Page(2)]).is_some());
}

#[test]
fn closing_waits_for_the_save_under_way() {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    ben.url = "http://127.0.0.1:9".into();
    edit(&mut b, 1, "A");
    let sent = ben.begin_save(&b.prj, Vec::new()).unwrap();
    let reply = answered(&mut server, &ben, sent);
    ben.close();
    assert!(!ben.busy() && ben.closing.len() == 1, "the session ends after the save");
    // Another session saves at once; the old answer leaves its save alone.
    let opened = Opened { session: Session { user: "ben".into(), session: "s9".into() }, snapshot: server.snapshot(), ..Default::default() };
    ben.shared = Some(Shared::open("plant", opened).0);
    assert!(ben.begin_save(&b.prj, Vec::new()).is_some());
    let (taken, _) = deliver(&mut ben, &mut b, reply);
    assert_eq!(taken, Taken::default());
    assert_eq!(ben.saving.as_deref(), Some("s9"));
    assert!(ben.closing.is_empty() && ben.busy(), "the close is sent now");
}

#[test]
fn the_unsaved_changes_question_waits_for_the_queued_save() {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    let mut guard = crate::session::Guard::default();
    edit(&mut b, 1, "A");
    let first = ben.begin_save(&b.prj, Vec::new()).unwrap();
    edit(&mut b, 1, "B");
    assert!(guard.request(b.dirty(), "replace").is_none());
    guard.save();
    assert!(ben.begin_save(&b.prj, Vec::new()).is_none());
    let reply = answered(&mut server, &ben, first);
    let (_, next) = deliver(&mut ben, &mut b, reply);
    let next = next.expect("B is saved next");
    assert_eq!(guard.after_save(b.dirty(), ben.saving.is_some()), None);
    assert!(guard.saving());
    let reply = answered(&mut server, &ben, next);
    deliver(&mut ben, &mut b, reply);
    assert_eq!(guard.after_save(b.dirty(), ben.saving.is_some()), Some("replace"));
    assert_eq!(server.project().pages[1].description, "B");
}

#[test]
fn a_newer_project_announced_while_saving_is_loaded_after_the_save() {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    ben.url = "http://127.0.0.1:9".into();
    edit(&mut b, 1, "A");
    let sent = ben.begin_save(&b.prj, Vec::new()).unwrap();
    ben.take_status(Status { revision: 2, locks: Vec::new() }, &[]);
    assert!(!ben.fetching && ben.refetch == Some(2));
    let reply = answered(&mut server, &ben, sent);
    deliver(&mut ben, &mut b, reply);
    // The answer of the save brought revision 2: nothing more to load.
    ben.flush(&b);
    assert!(!ben.fetching && ben.refetch.is_none());
    // Announced newer than the answer: loaded once the save is answered.
    ben.begin_save(&b.prj, Vec::new()).unwrap();
    ben.take_status(Status { revision: 9, locks: Vec::new() }, &[]);
    ben.flush(&b);
    assert!(!ben.fetching);
    ben.saving = None;
    ben.flush(&b);
    assert!(ben.fetching && ben.refetch.is_none());
}

impl Server {
    /// The server restarted (or the sessions expired): no session and no lock is left.
    fn restart(&mut self) {
        self.locks.clear();
    }

    /// Opens the project again for `r` in a new session with a new id range.
    fn reopen(&mut self, r: &Remote, rights: Vec<ProjectRight>) -> Opened {
        self.next_session += 1;
        let user = r.shared.as_ref().unwrap().session.user.clone();
        Opened {
            session: Session { user, session: format!("s{}", self.next_session) },
            first_id: self.next_session * openecraft_core::collab::ID_BLOCK,
            snapshot: self.snapshot(),
            rights,
        }
    }
}

#[test]
fn a_failed_save_stays_shown_after_the_connection_is_back() {
    let mut r = opened();
    let ctx = egui::Context::default();
    answer(&mut r, &ctx, vec![Reply::Saved(sent("s1"), Err(offline()))]);
    assert_eq!(r.link, Link::Offline);
    assert!(r.save_error.is_some());
    answer(&mut r, &ctx, vec![Reply::Status("s1".into(), Ok(Status::default()))]);
    assert!(r.online());
    assert!(r.save_error.is_some(), "the changes are still not on the server");
    assert!(r.status.contains("save again"), "{}", r.status);
    answer(&mut r, &ctx, vec![Reply::Saved(sent("s1"), Ok(Box::default()))]);
    assert!(r.save_error.is_none());
}

#[test]
fn an_outage_keeps_the_session_and_goes_on_with_it() {
    let mut r = opened();
    let ctx = egui::Context::default();
    // Status, lock and save requests fail without answer: the same session goes on.
    answer(&mut r, &ctx, vec![Reply::Status("s1".into(), Err(offline())), Reply::Locked("s1".into(), vec![Target::Page(1)], Err(offline()))]);
    assert_eq!(r.link, Link::Offline);
    answer(&mut r, &ctx, vec![Reply::Status("s1".into(), Err(offline()))]);
    assert_eq!(r.link, Link::Offline, "repeated failures change nothing");
    answer(&mut r, &ctx, vec![Reply::Status("s1".into(), Ok(Status::default()))]);
    assert!(r.online() && !r.reopening);
    assert_eq!(r.session_id(), "s1");
}

#[test]
fn an_ended_session_is_opened_again_but_a_lost_sign_in_waits() {
    let mut r = opened();
    r.url = "http://127.0.0.1:9".into();
    let ctx = egui::Context::default();
    let ended = || failed(Failure::SessionEnded, "the session has expired, open the project again");
    answer(&mut r, &ctx, vec![Reply::Status("s1".into(), Err(ended()))]);
    assert_eq!(r.link, Link::Ended);
    assert!(r.reopening, "the project is opened again at once");
    // Without answer the project is opened again with the next heartbeat.
    answer(&mut r, &ctx, vec![Reply::Reopened("s1".into(), Err(offline()))]);
    assert_eq!(r.link, Link::Ended);
    assert!(!r.reopening);
    assert!(!r.save(&Project::default(), Vec::new()), "nothing is saved with the ended session");
    assert!(r.due);

    // 401: the sign-in is gone. Nothing is sent until the user signs in again.
    let mut r = opened();
    r.url = "http://127.0.0.1:9".into();
    answer(&mut r, &ctx, vec![Reply::Status("s1".into(), Err(failed(Failure::SignedOut, "missing or wrong access token")))]);
    assert_eq!(r.link, Link::SignedOut);
    assert!(!r.polling && !r.reopening);
    assert!(!r.save(&Project::default(), Vec::new()));
    let login = LoginResponse { token: "new".into(), user: openecraft_core::collab::ProjectUser { name: "anna".into(), ..Default::default() }, ..Default::default() };
    answer(&mut r, &ctx, vec![Reply::Login(Ok(login))]);
    assert_eq!(r.token, "new");
    assert!(r.polling, "the session is checked with the new sign-in");
    assert_eq!(r.link, Link::Offline);
}

#[test]
fn a_project_the_server_no_longer_opens_offers_a_retry() {
    let mut r = opened();
    r.url = "http://127.0.0.1:9".into();
    let ctx = egui::Context::default();
    r.link = Link::Ended;
    r.reopening = true;
    answer(&mut r, &ctx, vec![Reply::Reopened("s1".into(), Err(failed(Failure::Refused, "project 'plant' not found")))]);
    assert_eq!(r.link, Link::Refused);
    assert!(!r.reopening && !r.polling, "nothing is tried again by itself");
    r.retry();
    r.poll(&ctx);
    assert_eq!(r.link, Link::Ended);
    assert!(r.reopening);
}

/// Ben edits pages 1 and 2 while the server restarts; the session is gone.
fn restarted() -> (Server, Remote, crate::session::Session) {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    edit(&mut b, 0, "Ben 1");
    edit(&mut b, 1, "Ben 2");
    let status = server.lock(&ben, &[Target::Page(1), Target::Page(2)]).unwrap();
    ben.take_status(status, &[Target::Page(1), Target::Page(2)]);
    server.restart();
    ben.link = Link::Ended;
    (server, ben, b)
}

#[test]
fn reopening_keeps_unsaved_edits_with_a_new_session_and_id_range() {
    let (mut server, mut ben, mut b) = restarted();
    let old_end = ben.shared.as_ref().unwrap().id_end;
    let mine = b.prj.new_id();
    assert!(mine < old_end);
    let opened = server.reopen(&ben, Vec::new());
    let taken = ben.take_reopened(&mut b, opened, "now".into());
    assert_eq!(taken, Taken::default());
    let sh = ben.shared.as_ref().unwrap();
    assert_eq!(sh.session.session, "s2");
    assert_eq!(b.prj.pages[0].description, "Ben 1");
    assert_eq!(b.prj.pages[1].description, "Ben 2");
    assert!(b.dirty(), "the edits are not saved yet");
    assert_eq!(ben.checked_rev, 0);
    assert_eq!(sh.unlocked_changes(&b.prj), vec![Target::Page(1), Target::Page(2)], "the edits lock again");
    let id = b.prj.new_id();
    assert!(id >= 2 * openecraft_core::collab::ID_BLOCK && id < sh.id_end, "new objects get ids of the new range");
    // Undo goes back without leaving the new range or the server's project.
    b.undo();
    assert_eq!(b.prj.pages[1].description, "Motors");
    assert!(b.prj.next_id >= 2 * openecraft_core::collab::ID_BLOCK);
    b.redo();

    // Locked again, the edits are saved.
    let targets = sh.unlocked_changes(&b.prj);
    let status = server.lock(&ben, &targets).unwrap();
    ben.take_status(status, &targets);
    let taken = save(&mut server, &mut ben, &mut b, &[]);
    assert!(taken.kept.is_empty());
    assert_eq!(server.project().pages[1].description, "Ben 2");
    assert!(!b.dirty());
}

#[test]
fn reopening_never_puts_stale_edits_over_foreign_work() {
    let (mut server, mut ben, mut b) = restarted();
    // Anna changed page 1 while Ben was away.
    let (mut anna, mut a) = server.open("anna");
    edit(&mut a, 0, "Anna");
    save(&mut server, &mut anna, &mut a, &[Target::Page(1)]);
    let opened = server.reopen(&ben, Vec::new());
    let taken = ben.take_reopened(&mut b, opened, "now".into());
    assert!(taken.lost);
    assert_eq!(taken.kept.len(), 1, "{taken:?}");
    assert_eq!(b.prj.pages[0].description, "Anna", "the foreign change is not overwritten");
    assert_eq!(b.prj.pages[1].description, "Ben 2");
    let kept = &ben.kept[ben.kept_here()[0]];
    assert_eq!((kept.reason, kept.targets.clone()), (KeptReason::Changed, vec![Target::Page(1)]));
    assert_eq!(kept.project.pages[0].description, "Ben 1");
    // Undo keeps the foreign change.
    b.undo();
    assert_eq!(b.prj.pages[0].description, "Anna");
    assert!(b.dirty());
}

#[test]
fn reopening_keeps_a_copy_of_edits_of_a_removed_page() {
    let (mut server, mut ben, mut b) = restarted();
    let (mut anna, mut a) = server.open("anna");
    a.prj.pages.remove(1);
    assert!(a.commit(true));
    save(&mut server, &mut anna, &mut a, &[Target::Project]);
    assert_eq!(server.project().pages.len(), 1);
    let opened = server.reopen(&ben, Vec::new());
    let taken = ben.take_reopened(&mut b, opened, "now".into());
    assert_eq!(b.prj.pages.len(), 1);
    assert_eq!(b.prj.pages[0].description, "Ben 1");
    assert!(taken.lost);
    let kept = &ben.kept[ben.kept_here()[0]];
    assert_eq!(kept.targets, vec![Target::Page(2)]);
    assert_eq!(kept.project.pages[1].description, "Ben 2");
}

#[test]
fn reopening_without_edits_is_clean_and_takes_the_rights() {
    let mut server = Server::new();
    let (mut ben, mut b) = server.open("ben");
    server.restart();
    let (mut anna, mut a) = server.open("anna");
    edit(&mut a, 0, "Anna");
    save(&mut server, &mut anna, &mut a, &[Target::Page(1)]);
    let opened = server.reopen(&ben, vec![ProjectRight::View]);
    let taken = ben.take_reopened(&mut b, opened, "now".into());
    assert_eq!(taken, Taken::default());
    assert_eq!(b.prj.pages[0].description, "Anna");
    assert!(!b.dirty());
    assert!(ben.viewer(), "a read-only user stays read-only");
    assert!(ben.kept.is_empty());
}

#[test]
fn answers_of_the_ended_session_are_dropped_after_reopening() {
    let (mut server, mut ben, mut b) = restarted();
    let opened = server.reopen(&ben, Vec::new());
    ben.take_reopened(&mut b, opened, "now".into());
    let ctx = egui::Context::default();
    ben.link = Link::Online;
    let replies = answer(&mut ben, &ctx, vec![Reply::Saved(sent("s1"), Err(failed(Failure::SessionEnded, "expired")))]);
    assert!(replies.is_empty());
    assert!(ben.online() && ben.save_error.is_none());
}
