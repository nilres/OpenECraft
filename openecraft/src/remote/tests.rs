use super::*;
use openecraft_core::collab::{Opened, Session};
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
    r.inbox.lock().unwrap().push(Reply::Locked("s1".into(), vec![Target::Project], Err("project data is locked by ben".into())));
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
    r.inbox.lock().unwrap().push(Reply::Status("s1".into(), Err("connection refused".into())));
    r.poll(&ctx);
    assert!(r.lost);
    r.pending = 1;
    r.inbox.lock().unwrap().push(Reply::Status("s1".into(), Ok(Status::default())));
    r.poll(&ctx);
    assert!(!r.lost);
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
        use openecraft_core::collab::{changes, merge, save_plan, Owned};
        let sh = r.shared.as_ref().unwrap();
        let base = &self.stored[sh.revision as usize - 1];
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
    r.saving_rev = session.history.rev;
    let snapshot = server.save(r, &session.prj, release, false).unwrap();
    r.take_snapshot(session, snapshot, true, "now".into())
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
    ben.inbox.lock().unwrap().push(Reply::Saved("s2".into(), Err("connection refused".into())));
    assert_eq!(ben.poll(&ctx).len(), 1);
    assert!(ben.status.contains("connection refused"));
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
    let taken = ben.take_snapshot(&mut b, server.snapshot(), false, "now".into());
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
    let taken = ben.take_snapshot(&mut b, server.snapshot(), false, "now".into());
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
    let taken = ben.take_snapshot(&mut b, server.snapshot(), false, "now".into());
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
    r.inbox.lock().unwrap().push(Reply::Saved("s1".into(), Ok(Box::new(Snapshot { revision: 7, ignored: vec![Target::Page(1)], ..Default::default() }))));
    r.poll(&ctx);
    assert!(r.status.starts_with("Partly saved") && r.status.contains('7'), "{}", r.status);
    r.inbox.lock().unwrap().push(Reply::Saved("s1".into(), Ok(Box::new(Snapshot { revision: 8, ..Default::default() }))));
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
