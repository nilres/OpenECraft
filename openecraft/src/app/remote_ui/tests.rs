use super::*;
use openecraft_core::model::{Page, PageKind, Project};
use openecraft_core::collab::{Opened, ProjectAccess, ProjectRole, ProjectSummary, Shared};
use crate::startup::{Reconnect, ReconnectState, ServerProject};
use std::collections::HashMap;

fn frame(app: &mut App, ctx: &egui::Context) {
    ctx.run_ui(egui::RawInput { time: Some(0.), ..Default::default() }, |ui| {
        app.poll_remote(ui.ctx());
        app.check_locks(ui.ctx());
    })
    .drop_without_applying_deltas();
}

fn idle(app: &mut App, ctx: &egui::Context) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        frame(app, ctx);
        if !app.remote.busy() {
            break;
        }
        assert!(std::time::Instant::now() < until, "HTTP request did not finish: {}", app.remote.status);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn edit(app: &mut App, text: &str) {
    app.session.prj.pages[0].description = text.into();
    assert!(app.session.commit(true));
}

fn project() -> Project {
    let mut prj = Project { pages: vec![Page::new(1, "1", "Supply", PageKind::Schematic), Page::new(2, "2", "Motors", PageKind::Schematic)], ..Default::default() };
    prj.normalize();
    prj
}

/// Run explicitly against a disposable loopback server with authenticated accounts;
/// never points at a user's project server or uses customer data.
#[test]
#[ignore = "requires a disposable loopback project server and OPENECRAFT_LIVE_TEST_* credentials; see scripts/test-project-server-live.ps1"]
fn live_http_shared_saves_are_serialized_and_confirm_only_what_they_sent() {
    let url = std::env::var("OPENECRAFT_LIVE_TEST_URL").expect("test server URL");
    assert!(url.starts_with("http://127.0.0.1:"), "only disposable loopback servers are permitted");
    let name = format!("save-test-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let connect = |user: &str| {
        let ctx = egui::Context::default();
        let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
        app.replace_project(project(), None);
        app.remote = Remote::default();
        app.remote.url = url.clone();
        app.remote.user = user.into();
        app.remote.token = std::env::var(format!("OPENECRAFT_LIVE_TEST_{}", user.to_uppercase())).expect("test account token");
        (app, ctx)
    };
    let (mut alice, a) = connect("alice");
    let (mut bob, b) = connect("bob");
    alice.remote.create(&name, &project());
    idle(&mut alice, &a);
    assert!(alice.remote.shared.is_some(), "{}", alice.remote.status);
    // What the server holds, as another user opening the project sees it.
    let stored = |bob: &mut App| {
        bob.remote.open(&name);
        idle(bob, &b);
        assert!(bob.remote.shared.is_some(), "{}", bob.remote.status);
        let text = bob.session.prj.pages[0].description.clone();
        bob.remote.close();
        idle(bob, &b);
        text
    };

    // Ctrl+S again while the first save is under way: the second waits and sends B.
    edit(&mut alice, "A");
    alice.save_remote(Vec::new());
    edit(&mut alice, "B");
    alice.save_remote(Vec::new());
    idle(&mut alice, &a);
    assert!(!alice.session.dirty(), "{}", alice.remote.status);
    assert_eq!(stored(&mut bob), "B");
    println!("PASS: a repeated save waits for the first and stores the later edit");

    // An edit made while saving stays unsaved.
    edit(&mut alice, "A2");
    alice.save_remote(Vec::new());
    edit(&mut alice, "B2");
    idle(&mut alice, &a);
    assert!(alice.session.dirty());
    assert_eq!(stored(&mut bob), "A2");
    alice.save_remote(Vec::new());
    idle(&mut alice, &a);
    assert!(!alice.session.dirty());
    assert_eq!(stored(&mut bob), "B2");
    println!("PASS: the answer of a save does not mark later edits saved");

    // Undo while saving.
    edit(&mut alice, "U");
    alice.save_remote(Vec::new());
    alice.session.undo();
    idle(&mut alice, &a);
    assert!(alice.session.dirty());
    assert_eq!(alice.session.prj.pages[0].description, "B2");
    assert_eq!(stored(&mut bob), "U");
    alice.save_remote(Vec::new());
    idle(&mut alice, &a);
    assert!(!alice.session.dirty());
    assert_eq!(stored(&mut bob), "B2");
    println!("PASS: undo while saving stays unsaved");

    // Save and release, then edit the page again before the answer.
    assert!(alice.remote.shared.as_ref().unwrap().owns(Target::Page(1)));
    edit(&mut alice, "R1");
    alice.save_remote(vec![Target::Page(1)]);
    edit(&mut alice, "R2");
    idle(&mut alice, &a);
    assert_eq!(alice.session.prj.pages[0].description, "R2");
    assert!(alice.session.dirty() && alice.remote.kept_here().is_empty());
    assert!(alice.remote.shared.as_ref().unwrap().owns(Target::Page(1)), "the lock is taken again for the later edit");
    assert_eq!(stored(&mut bob), "R1");
    alice.save_remote(vec![Target::Page(1)]);
    idle(&mut alice, &a);
    assert!(!alice.session.dirty() && !alice.remote.shared.as_ref().unwrap().owns(Target::Page(1)));
    assert_eq!(stored(&mut bob), "R2");
    println!("PASS: save and release keeps an edit made while saving");

    // A failed save neither marks saved nor releases; the next one goes through.
    alice.remote.lock(vec![Target::Page(1)]);
    idle(&mut alice, &a);
    edit(&mut alice, "F");
    alice.remote.url = "http://127.0.0.1:1".into();
    alice.save_remote(vec![Target::Page(1)]);
    idle(&mut alice, &a);
    assert!(alice.session.dirty() && alice.remote.shared.as_ref().unwrap().owns(Target::Page(1)));
    alice.remote.url = url.clone();
    assert_eq!(stored(&mut bob), "R2");
    alice.save_remote(vec![Target::Page(1)]);
    idle(&mut alice, &a);
    assert!(!alice.session.dirty(), "{}", alice.remote.status);
    assert_eq!(stored(&mut bob), "F");
    println!("PASS: a failed save leaves the changes unsaved");

    // The unsaved-changes question: Save waits for the queued save before replacing.
    edit(&mut alice, "G1");
    alice.save_remote(Vec::new());
    edit(&mut alice, "G2");
    alice.request_replace(&a, crate::app::Replace::Project(Box::new(project()), None));
    assert!(alice.discard.pending().is_some());
    alice.discard.save();
    alice.save(&a, false);
    let until = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        frame(&mut alice, &a);
        if let Some(r) = alice.discard.after_save(alice.session.dirty(), alice.remote.busy()) {
            alice.perform(&a, r);
            break;
        }
        assert!(alice.discard.saving(), "the save ended unsaved: {}", alice.remote.status);
        assert!(std::time::Instant::now() < until, "HTTP request did not finish: {}", alice.remote.status);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(alice.remote.shared.is_none());
    idle(&mut alice, &a);
    assert_eq!(stored(&mut bob), "G2");
    println!("PASS: the project is replaced only after the last edit is saved");

    // Closing while a save is under way: the session ends after the save.
    alice.remote.open(&name);
    idle(&mut alice, &a);
    edit(&mut alice, "S");
    alice.save_remote(Vec::new());
    alice.remote.close();
    idle(&mut alice, &a);
    assert_eq!(stored(&mut bob), "S");
    bob.remote.open(&name);
    idle(&mut bob, &b);
    bob.remote.lock(vec![Target::Page(1)]);
    idle(&mut bob, &b);
    assert!(bob.remote.shared.as_ref().unwrap().owns(Target::Page(1)), "the closed session released its lock: {}", bob.remote.status);
    bob.remote.close();
    idle(&mut bob, &b);
    println!("PASS: closing during a save keeps the save");
}

#[derive(Default)]
struct MemoryStorage(HashMap<String, String>);

impl eframe::Storage for MemoryStorage {
    fn remove_string(&mut self, key: &str) { self.0.remove(key); }
    fn get_string(&self, key: &str) -> Option<String> { self.0.get(key).cloned() }
    fn set_string(&mut self, key: &str, value: String) { self.0.insert(key.into(), value); }
    fn flush(&mut self) {}
}

fn shared_app() -> (App, Project, ServerProject) {
    let ctx = egui::Context::default();
    let mut storage = MemoryStorage::default();
    let shell = crate::app::ShellState { update_check: false, ..Default::default() };
    eframe::set_value(&mut storage, crate::app::UI_KEY, &shell);
    let mut creation = eframe::CreationContext::_new_kittest(ctx);
    creation.storage = Some(&storage);
    let mut app = App::new(&creation);
    let mut baseline = Project::default();
    baseline.info.name = "Synthetic baseline".into();
    app.replace_project(baseline, None);
    let baseline = app.session.prj.clone();
    // A fresh Remote has no real Store attached; the query prevents OS lookup.
    app.remote = Remote::default();
    app.remote.url = "http://127.0.0.1:9/?synthetic-app-sign-out-test".into();
    app.remote.user = "synthetic-Alice".into();
    app.remote.token = "synthetic-token".into();
    app.remote.password = "synthetic-unsent-password".into();
    app.remote.role = Some(ProjectRole::Editor);
    app.remote.rights = vec![ProjectRight::View, ProjectRight::EditPages];
    app.remote.access = Some(ProjectAccess::default());
    app.remote.projects = vec![ProjectSummary { name: "synthetic-project".into(), ..Default::default() }];
    app.remote.sign_in.signed_in = true;
    let opened = Opened {
        session: openecraft_core::collab::Session { user: app.remote.user.clone(), session: "synthetic-page-session".into() },
        snapshot: Snapshot { project: baseline.clone(), ..Default::default() },
        ..Default::default()
    };
    app.remote.shared = Some(Shared::open("synthetic-project", opened).0);
    let project = app.remote.project().unwrap();
    app.reconnect = Some(Reconnect { project: project.clone(), state: ReconnectState::Failed("synthetic disconnect".into()) });
    app.session.prj.info.name = "Synthetic local work".into();
    assert!(app.session.commit(true));
    assert!(app.session.dirty());
    (app, baseline, project)
}

fn assert_signed_out_preserves_work(app: &App, before: &Project, state: u64, revision: u64, recovery: &ServerProject) {
    assert_eq!(&app.session.prj, before);
    assert_eq!(app.session.history.state(), state);
    assert_eq!(app.session.history.rev, revision);
    assert!(app.session.dirty());
    assert!(app.session.file_name.is_none());
    assert_eq!(app.recovery.as_ref(), Some(recovery));
    assert!(app.reconnect.is_none());
    assert_eq!(app.remote.url, recovery.url);
    assert!(app.remote.shared.is_none());
    assert!(app.remote.token.is_empty() && app.remote.password.is_empty());
    assert!(app.remote.role.is_none() && app.remote.rights.is_empty() && app.remote.access.is_none());
    assert!(app.remote.projects.is_empty());
    assert!(!app.remote.sign_in.signed_in && !app.remote.sign_in.remember);
    assert!(!app.remote.sign_in.credential_changes_pending());
}

#[test]
fn signing_out_keeps_unsaved_project_undo_and_local_recovery() {
    let (mut app, baseline, project) = shared_app();
    let before = app.session.prj.clone();
    let state = app.session.history.state();
    let revision = app.session.history.rev;
    app.sign_out_remote(false);
    assert_signed_out_preserves_work(&app, &before, state, revision, &project);
    assert_eq!(app.remote.user, "synthetic-Alice");
    app.session.undo();
    assert_eq!(app.session.prj, baseline);
    app.session.redo();
    assert_eq!(app.session.prj, before);
}

#[test]
fn changing_user_clears_only_the_account_and_keeps_local_work_and_undo() {
    let (mut app, baseline, project) = shared_app();
    let before = app.session.prj.clone();
    let state = app.session.history.state();
    let revision = app.session.history.rev;
    app.sign_out_remote(true);
    assert_signed_out_preserves_work(&app, &before, state, revision, &project);
    assert!(app.remote.user.is_empty());
    app.session.undo();
    assert_eq!(app.session.prj, baseline);
    app.session.redo();
    assert_eq!(app.session.prj, before);
}

#[test]
fn same_account_sign_in_keeps_the_shared_session_unsaved_work_and_undo() {
    let (mut app, baseline, _) = shared_app();
    app.remote.link = crate::remote::Link::SignedOut;
    app.remote.sign_in.signed_in = false;
    let before = app.session.prj.clone();
    let history = (app.session.history.state(), app.session.history.rev);
    let shared = app.remote.shared.as_ref().unwrap().clone();
    app.remote.sign_in();
    let retained = app.remote.shared.as_ref().expect("reauthentication must keep the open shared project");
    assert_eq!(retained.session, shared.session);
    assert_eq!(retained.base, shared.base);
    assert_eq!(retained.revision, shared.revision);
    assert_eq!(app.session.prj, before);
    assert_eq!((app.session.history.state(), app.session.history.rev), history);
    assert!(app.session.dirty());
    app.session.undo();
    assert_eq!(app.session.prj, baseline);
    app.session.redo();
    assert_eq!(app.session.prj, before);
}
