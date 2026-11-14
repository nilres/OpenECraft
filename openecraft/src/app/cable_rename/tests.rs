use super::*;
use openecraft_core::{collab::{Lock, Opened, Shared, Snapshot}, geom::p};

fn live_frame(app: &mut App, ctx: &egui::Context, enter: bool) {
    live_frame_key(app, ctx, enter.then_some(egui::Key::Enter));
}

fn live_frame_key(app: &mut App, ctx: &egui::Context, key: Option<egui::Key>) {
    let events = key.into_iter().map(|key| egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }).collect();
    ctx.run_ui(egui::RawInput { time: Some(0.), screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000., 700.))), events, ..Default::default() }, |ui| {
        app.dialogs.begin(ui.input(|i| i.key_pressed(egui::Key::Escape)), &[]);
        app.poll_remote(ui.ctx());
        app.rename_cable_window(ui.ctx());
    }).drop_without_applying_deltas();
}

fn live_idle(app: &mut App, ctx: &egui::Context) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        live_frame(app, ctx, false);
        if !app.remote.busy() && !app.rename_cable.as_ref().is_some_and(|d| d.waiting) { break; }
        assert!(std::time::Instant::now() < until, "HTTP request did not finish: {}", app.remote.status);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn live_rename(app: &mut App, ctx: &egui::Context, tag: &str) {
    app.open_cable_rename(10);
    app.rename_cable.as_mut().unwrap().text = tag.into();
    live_frame(app, ctx, false);
    live_frame(app, ctx, false);
    live_frame(app, ctx, true);
    live_idle(app, ctx);
}

/// Run explicitly against a disposable loopback server with three authenticated
/// accounts; never points at a user's project server or uses customer data.
#[test]
#[ignore = "requires a disposable loopback project server and OPENECRAFT_LIVE_TEST_* credentials; see scripts/test-project-server-live.ps1"]
fn live_http_multiuser_rename_is_atomic_and_preserves_other_users_changes() {
    let url = std::env::var("OPENECRAFT_LIVE_TEST_URL").expect("test server URL");
    assert!(url.starts_with("http://127.0.0.1:"), "only disposable loopback servers are permitted");
    let name = format!("rename-test-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let connect = |user: &str| {
        let ctx = egui::Context::default();
        let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
        app.replace_project(project(), None);
        app.remote = crate::remote::Remote::default();
        app.remote.url = url.clone();
        app.remote.user = user.into();
        app.remote.token = std::env::var(format!("OPENECRAFT_LIVE_TEST_{}", user.to_uppercase())).expect("test account token");
        (app, ctx)
    };
    let (mut alice, a) = connect("alice");
    let (mut bob, b) = connect("bob");
    let (mut viewer, v) = connect("viewer");
    alice.remote.create(&name, &project()); live_idle(&mut alice, &a);
    assert!(alice.remote.shared.is_some(), "{}", alice.remote.status);
    bob.remote.open(&name); live_idle(&mut bob, &b);
    assert!(bob.remote.shared.is_some());
    assert_ne!(alice.remote.shared.as_ref().unwrap().session, bob.remote.shared.as_ref().unwrap().session);
    assert!(alice.remote.may(ProjectRight::EditProject));
    assert!(bob.remote.may(ProjectRight::EditPages));

    // Cancel after sending the request, before the UI has processed its answer.
    // A pre-existing page lock belongs to the user, not to this dialog.
    alice.remote.lock(vec![Target::Page(1)]); live_idle(&mut alice, &a);
    let before_cancel = alice.session.prj.clone();
    alice.open_cable_rename(10);
    alice.rename_cable.as_mut().unwrap().text = "-Cancelled".into();
    live_frame(&mut alice, &a, false); live_frame(&mut alice, &a, false);
    live_frame(&mut alice, &a, true);
    assert!(alice.rename_cable.as_ref().unwrap().waiting);
    live_frame_key(&mut alice, &a, Some(egui::Key::Escape)); live_idle(&mut alice, &a);
    assert!(alice.rename_cable.is_none());
    assert_eq!(alice.session.prj, before_cancel);
    let shared = alice.remote.shared.as_ref().unwrap();
    assert!(shared.owns(Target::Page(1)));
    assert!(!shared.owns(Target::Project) && !shared.owns(Target::Page(2)));
    bob.remote.lock(vec![Target::Project, Target::Page(2)]); live_idle(&mut bob, &b);
    let shared = bob.remote.shared.as_ref().unwrap();
    assert!(shared.owns(Target::Project) && shared.owns(Target::Page(2)));
    bob.remote.release(vec![Target::Project, Target::Page(2)]); live_idle(&mut bob, &b);
    alice.remote.release(vec![Target::Page(1)]); live_idle(&mut alice, &a);
    println!("PASS: cancelling a pending rename releases new locks and keeps existing locks");

    // Alice has an older snapshot and does not yet know Bob holds page 2.
    bob.remote.lock(vec![Target::Page(2)]); live_idle(&mut bob, &b);
    bob.session.prj.pages[1].description = "Bob's concurrent page edit".into();
    bob.session.commit(true);
    bob.save_remote(Vec::new()); live_idle(&mut bob, &b);
    let before = alice.session.prj.clone();
    live_rename(&mut alice, &a, "-4W1");
    assert!(alice.rename_cable.as_ref().is_some_and(|d| d.error.is_some()));
    assert_eq!(alice.session.prj, before, "a rejected HTTP lock must not partially rename anything");
    println!("PASS: authenticated page lock conflict leaves the entire project unchanged");
    alice.rename_cable = None;
    bob.remote.release(vec![Target::Page(2)]); live_idle(&mut bob, &b);

    // The lock response loads the newer server page before applying the draft.
    live_rename(&mut alice, &a, "-4W1");
    assert!(alice.rename_cable.is_none(), "status: {}; draft error: {:?}; current cable: {:?}", alice.remote.status, alice.rename_cable.as_ref().and_then(|d| d.error.as_ref()), alice.session.prj.cables.first());
    assert_eq!(alice.session.prj.pages[1].description, "Bob's concurrent page edit");
    for pg in &alice.session.prj.pages { assert_eq!(pg.wires[0].cable, "-4W1"); assert_eq!(pg.cable_defs[0].cable, "-4W1"); }
    alice.session.commit(true);
    alice.session.undo(); assert_eq!(alice.session.prj.cables[0].tag, "=A+CC-W1");
    assert_eq!(alice.session.prj.pages[1].description, "Bob's concurrent page edit");
    alice.session.redo(); assert_eq!(alice.session.prj.cables[0].tag, "-4W1");
    alice.save_remote(vec![Target::Project, Target::Page(1), Target::Page(2)]); live_idle(&mut alice, &a);
    bob.remote.close(); live_idle(&mut bob, &b);
    bob.remote.open(&name); live_idle(&mut bob, &b);
    assert_eq!(bob.session.prj.cables[0].tag, "-4W1");
    for pg in &bob.session.prj.pages { assert_eq!(pg.wires[0].cable, "-4W1"); assert_eq!(pg.cable_defs[0].cable, "-4W1"); }
    println!("PASS: fresh server revision preserved; undo/redo and saved references agree in both sessions");

    viewer.remote.open(&name); live_idle(&mut viewer, &v);
    assert!(viewer.remote.viewer());
    let before = viewer.session.prj.clone();
    live_rename(&mut viewer, &v, "-Forbidden");
    assert!(viewer.rename_cable.as_ref().is_some_and(|d| d.error.is_some()));
    assert_eq!(viewer.session.prj, before);
    println!("PASS: authenticated viewer cannot rename even through a directly opened dialog");

    // Bob renames while Alice still has her old-name draft open.
    alice.open_cable_rename(10);
    alice.rename_cable.as_mut().unwrap().text = "-Stale".into();
    live_frame(&mut alice, &a, false); live_frame(&mut alice, &a, false);
    live_rename(&mut bob, &b, "-Bob"); assert!(bob.rename_cable.is_none());
    bob.session.commit(true);
    bob.save_remote(vec![Target::Project, Target::Page(1), Target::Page(2)]); live_idle(&mut bob, &b);
    live_frame(&mut alice, &a, true); live_idle(&mut alice, &a);
    assert_eq!(alice.session.prj.cables[0].tag, "-Bob");
    assert!(alice.rename_cable.as_ref().is_some_and(|d| d.error.is_some()));
    println!("PASS: a draft from before another user's rename cannot overwrite the new server name");

    alice.rename_cable = None;
    alice.remote.close(); live_idle(&mut alice, &a);
    alice.remote.open(&name); live_idle(&mut alice, &a);
    let before = alice.session.prj.clone();
    alice.remote.url = "http://127.0.0.1:1".into();
    live_rename(&mut alice, &a, "-Offline");
    assert!(alice.rename_cable.as_ref().is_some_and(|d| d.error.is_some()));
    assert_eq!(alice.session.prj, before);
    println!("PASS: real transport failure during lock acquisition does not apply the rename");
    alice.remote.url = url;
    for (app, ctx) in [(&mut alice, &a), (&mut bob, &b), (&mut viewer, &v)] { app.rename_cable = None; app.remote.close(); live_idle(app, ctx); }
}

fn project() -> Project {
    let mut prj = Project {
        pages: vec![Page::new(1, "1", "", PageKind::Schematic), Page::new(2, "2", "", PageKind::Schematic)],
        cables: vec![Cable { id: 10, tag: "=A+CC-W1".into(), cores: "BN BU GNYE".into(), ..Default::default() }, Cable { id: 11, tag: "-W2".into(), ..Default::default() }],
        ..Default::default()
    };
    for (i, pg) in prj.pages.iter_mut().enumerate() {
        pg.wires.push(Wire { id: 20 + i as u64, a: p(40., 40.), b: p(40., 80.), cable: "=A+CC-W1".into(), core: "BN".into(), ..Default::default() });
        pg.cable_defs.push(CableDef { id: 30 + i as u64, a: p(30., 60.), b: p(50., 60.), cable: "=A+CC-W1".into(), shield: false });
    }
    prj.normalize();
    prj
}

fn remote(prj: &Project) -> Remote {
    let opened = Opened { session: collab::Session { session: "mine".into(), user: "tester".into() }, snapshot: Snapshot { project: prj.clone(), ..Default::default() }, ..Default::default() };
    let mut remote = Remote::default();
    remote.shared = Some(Shared::open("test", opened).0);
    remote
}

#[test]
fn selection_resolves_definitions_wires_and_cable_symbols_but_not_ambiguous_selections() {
    let mut prj = project();
    for id in [20, 30] { assert_eq!(selected(&prj, 0, &BTreeSet::from([id])), Some(10)); }
    prj.pages[0].symbols.push(SymbolInst { id: 40, values: std::collections::BTreeMap::from([("function.cable".into(), "=A+CC-W1".into())]), ..Default::default() });
    assert_eq!(selected(&prj, 0, &BTreeSet::from([40])), Some(10));
    for ids in [BTreeSet::new(), BTreeSet::from([20, 30]), BTreeSet::from([999])] { assert_eq!(selected(&prj, 0, &ids), None); }
    prj.cables.clear();
    assert_eq!(selected(&prj, 0, &BTreeSet::from([20])), None);
}

#[test]
fn drafts_cancel_without_changes_and_reject_invalid_or_stale_targets() {
    let mut prj = project();
    let before = prj.clone();
    let mut draft = Draft::new(&prj.cables[0]);
    draft.text = "-4W1".into();
    drop(draft);
    assert_eq!(prj, before);
    let mut draft = Draft::new(&prj.cables[0]);
    for text in ["", "W1", "-W2", "-W 3"] {
        draft.text = text.into();
        assert!(draft.prepare(&prj).is_err());
        assert_eq!(prj, before);
    }
    draft.text = "-4W1".into();
    prj.pages[1].locked = true;
    assert!(draft.prepare(&prj).is_err(), "released second page");
    prj.pages[1].locked = false;
    prj.cables[0].tag = "-Elsewhere".into();
    assert!(draft.prepare(&prj).is_err(), "renamed while dialog open");
    prj.cables.clear();
    assert!(draft.prepare(&prj).is_err(), "deleted while dialog open");
}

#[test]
fn rename_updates_both_pages_and_survives_undo_redo_and_file_roundtrip() {
    let mut session = crate::session::Session::new(project(), None);
    let before = session.prj.clone();
    let mut draft = Draft::new(&session.prj.cables[0]);
    draft.text = " -4W1 ".into();
    session.prj = draft.prepare(&session.prj).unwrap();
    assert!(session.commit(true));
    let after = session.prj.clone();
    for (i, page) in after.pages.iter().enumerate() {
        assert_eq!(page.wires[0].cable, "-4W1");
        assert_eq!(page.cable_defs[0].cable, "-4W1");
        let mut wire = page.wires[0].clone();
        wire.cable = before.pages[i].wires[0].cable.clone();
        assert_eq!(wire, before.pages[i].wires[0], "geometry/core/labels preserved");
    }
    assert_eq!(Project::from_json(&after.to_json()).unwrap(), after);
    session.undo(); assert_eq!(session.prj, before);
    session.redo(); assert_eq!(session.prj, after);
}

#[test]
fn shared_rename_needs_every_changed_target_before_committing() {
    let prj = project();
    let mut draft = Draft::new(&prj.cables[0]); draft.text = "-4W1".into();
    let after = draft.prepare(&prj).unwrap();
    let mut remote = remote(&prj);
    let targets = locks(&remote, &prj, &after).unwrap();
    assert_eq!(targets, vec![Target::Project, Target::Page(1), Target::Page(2)]);
    remote.shared.as_mut().unwrap().locks.push(Lock { target: Target::Page(2), session: "other".into(), user: "another".into() });
    assert!(locks(&remote, &prj, &after).is_err());
    remote.shared.as_mut().unwrap().locks = targets.iter().map(|target| Lock { target: *target, session: "mine".into(), user: "tester".into() }).collect();
    assert!(locks(&remote, &prj, &after).unwrap().is_empty());
    remote.shared.as_mut().unwrap().fresh.insert(Target::Page(2));
    assert_eq!(locks(&remote, &prj, &after).unwrap(), [Target::Page(2)], "newer server page must load first");
    remote.rights = vec![ProjectRight::View, ProjectRight::EditPages];
    assert!(locks(&remote, &prj, &after).is_err(), "project data permission required");
    remote.rights = vec![ProjectRight::View, ProjectRight::EditProject];
    assert!(locks(&remote, &prj, &after).is_err(), "page permission required");
    remote.lost = true;
    assert!(locks(&remote, &prj, &after).is_err());
}

#[test]
fn dialog_selects_the_complete_name_and_enter_applies_one_undoable_change() {
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    app.replace_project(project(), None);
    app.editor.selection.insert(30);
    app.in_dock = true;
    app.run(&ctx, crate::commands::Cmd::RenameCable);
    assert!(app.rename_cable.is_none());
    assert_eq!(app.deferred.len(), 1, "context menu defers the command until the dock is restored");
    app.in_dock = false;
    let crate::app::Deferred::Cmd(command) = app.deferred.pop().unwrap() else { panic!("expected rename command") };
    app.run(&ctx, command);
    assert!(app.rename_cable.is_some());
    let frame = |app: &mut App, events: Vec<egui::Event>| {
        let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000., 700.))), events, ..Default::default() };
        ctx.run_ui(input, |ui| app.rename_cable_window(ui.ctx())).drop_without_applying_deltas();
    };
    frame(&mut app, vec![]);
    frame(&mut app, vec![]);
    frame(&mut app, vec![egui::Event::Text("-4W1".into())]);
    assert_eq!(app.rename_cable.as_ref().unwrap().text, "-4W1", "typing replaces the full selection");
    frame(&mut app, vec![egui::Event::Key { key: egui::Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }]);
    assert!(app.rename_cable.is_none());
    assert_eq!(app.session.prj.cables[0].tag, "-4W1");
    app.session.commit(true);
    app.session.undo();
    assert_eq!(app.session.prj.cables[0].tag, "=A+CC-W1");
}

#[test]
fn escape_and_switching_projects_discard_the_dialog() {
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    app.replace_project(project(), None);
    let before = app.session.prj.clone();
    app.open_cable_rename(10);
    app.rename_cable.as_mut().unwrap().text = "-4W1".into();
    ctx.run_ui(egui::RawInput::default(), |ui| app.rename_cable_window(ui.ctx())).drop_without_applying_deltas();
    assert!(app.dialogs.begin(true, &[]));
    ctx.run_ui(egui::RawInput::default(), |ui| app.rename_cable_window(ui.ctx())).drop_without_applying_deltas();
    assert!(app.rename_cable.is_none()); assert_eq!(app.session.prj, before);
    app.open_cable_rename(10);
    app.replace_project(project(), None);
    assert!(app.rename_cable.is_none());
}

#[test]
fn a_cancelled_draft_only_owns_its_new_requests() {
    let prj = project();
    let mut remote = remote(&prj);
    remote.url = "http://127.0.0.1:9".into();
    let sh = remote.shared.as_mut().unwrap();
    sh.locks.push(Lock { target: Target::Project, user: "tester".into(), session: "mine".into() });
    sh.fresh.insert(Target::Project);
    sh.requested.insert(Target::Page(1));
    let mut draft = Draft::new(&prj.cables[0]);
    draft.request_locks(&mut remote, vec![Target::Project, Target::Page(1), Target::Page(2)]);
    assert_eq!(draft.locks, Some(("mine".into(), vec![Target::Page(2)])));
    draft.cancel(&mut remote);
    assert!(draft.locks.is_none());
    // An unrelated in-flight page request is not this dialog's to release.
    remote.release_cancelled(&prj);
    assert!(remote.shared.as_ref().unwrap().requested.contains(&Target::Page(1)));
}
