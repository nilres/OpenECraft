use super::*;
use openecraft_core::collab::{Access, Opened, ProjectAccess, ProjectRole, ProjectRight, ProjectSummary, ProjectUser};

// Query strings are ineligible for credential storage. All incidental cleanup
// requests stay on a closed loopback port, without touching any OS credentials.
fn remote() -> Remote {
    Remote { url: "http://127.0.0.1:9/?synthetic-sign-in-test".into(), user: "Alice".into(), ..Default::default() }
}

fn account() -> Account { Account::new("https://server.test/projects", "Alice").unwrap() }

fn login() -> LoginResponse {
    LoginResponse {
        token: "synthetic-session-token".into(),
        user: ProjectUser { name: "Alice".into(), role: Some(ProjectRole::Viewer), ..Default::default() },
        access: Access::default(),
    }
}

fn response(status: u16, bytes: Vec<u8>) -> ehttp::Result<ehttp::Response> {
    Ok(ehttp::Response { url: "https://server.test/api/v1/session".into(), ok: (200..300).contains(&status), status, status_text: String::new(), headers: Default::default(), bytes })
}

#[test]
fn default_login_never_opts_into_persistence_and_profile_json_has_no_secrets() {
    let mut remote = remote();
    assert!(!remote.sign_in.remember);
    assert!(remote.sign_in.profiles.is_empty());
    assert!(!remote.restore_before(None));
    assert!(!remote.sign_in.credentials.busy());
    remote.accept_login(&login(), false);
    assert!(remote.sign_in.signed_in);
    assert!(remote.sign_in.profiles.is_empty());
    assert!(!remote.sign_in.credentials.busy());

    let profile = Profile::of(&account());
    let json = serde_json::to_value(vec![profile.clone()]).unwrap();
    assert_eq!(json, serde_json::json!([{ "server": "https://server.test:443/projects", "user": "alice" }]));
    let encoded = serde_json::to_string(&json).unwrap();
    assert!(!encoded.contains(&login().token));
    assert!(!encoded.contains("password"));
    assert_eq!(serde_json::from_value::<Vec<Profile>>(json).unwrap(), [profile]);
}

#[test]
fn editing_the_account_clears_identity_but_preserves_the_typed_password() {
    let mut remote = remote();
    let previous = (remote.url.clone(), remote.user.clone(), remote.token.clone());
    remote.token = "old-token".into();
    remote.password = "unsent-password".into();
    remote.role = Some(ProjectRole::Admin);
    remote.rights = vec![ProjectRight::Administer];
    remote.access = Some(ProjectAccess::default());
    remote.shared = Some(openecraft_core::collab::Shared::open("old-project", Opened::default()).0);
    remote.projects = vec![ProjectSummary { name: "old-project".into(), ..Default::default() }];
    remote.sign_in.signed_in = true;
    remote.sign_in.restoring = true;
    remote.sign_in.after = Some(After::Open("old-project".into()));
    remote.pending = 3;
    remote.user = "Bob".into();
    let generation = remote.sign_in.generation;
    remote.credentials_edited(&previous.0, &previous.1, &previous.2);
    assert_eq!(remote.user, "Bob");
    assert_eq!(remote.sign_in.generation, generation + 1);
    assert!(remote.token.is_empty());
    assert_eq!(remote.password, "unsent-password");
    assert!(remote.role.is_none() && remote.rights.is_empty() && remote.access.is_none());
    assert!(remote.projects.is_empty() && remote.shared.is_none());
    assert!(!remote.sign_in.signed_in && !remote.sign_in.restoring);
    assert!(remote.sign_in.after.is_none());
    assert_eq!(remote.pending, 0);
}

#[test]
fn editing_only_a_manual_token_keeps_the_new_token_without_a_signed_in_identity() {
    let mut remote = remote();
    let previous = (remote.url.clone(), remote.user.clone(), "old-token".to_string());
    remote.token = "new-manual-token".into();
    remote.role = Some(ProjectRole::Admin);
    remote.sign_in.signed_in = true;
    remote.credentials_edited(&previous.0, &previous.1, &previous.2);
    assert_eq!(remote.token, "new-manual-token");
    assert!(remote.role.is_none());
    assert!(!remote.sign_in.signed_in);
    assert!(remote.sign_in.checked);
    assert!(!remote.sign_in.credentials.busy());
}

#[test]
fn selecting_a_startup_server_uses_its_profile_without_carrying_the_old_token() {
    let mut remote = remote();
    remote.sign_in.profiles = vec![Profile::of(&account()), Profile::of(&Account::new("https://other.test", "Bob").unwrap())];
    remote.token = "token-from-the-previous-server".into();
    remote.role = Some(ProjectRole::Admin);
    remote.select_server("https://other.test/");
    assert_eq!(remote.url, "https://other.test/");
    assert_eq!(remote.user, "bob");
    assert!(remote.token.is_empty());
    assert!(remote.role.is_none());
    assert!(remote.sign_in.remember);
    assert!(!remote.sign_in.checked);
    assert!(!remote.sign_in.credentials.busy());
}

fn delayed_answers(remote: &mut Remote) -> Vec<impl FnOnce(Reply) + Send + 'static> {
    (0..5).map(|_| remote.receiver("old request".into())).collect()
}

fn return_delayed(answers: Vec<impl FnOnce(Reply)>, account: Account) {
    let login = login();
    let replies = [
        Reply::Login(Ok(login.clone())),
        Reply::Login(Err("old login failure".into())),
        Reply::List(Ok(vec![ProjectSummary { name: "old-project".into(), ..Default::default() }])),
        Reply::Opened("old-project".into(), Ok(Box::default())),
        Reply::Restored(account, login.token, Ok(CurrentAccount { user: login.user, access: login.access })),
    ];
    for (receive, reply) in answers.into_iter().zip(replies) { receive(reply); }
}

#[test]
fn stale_login_list_open_and_restore_replies_cannot_change_a_new_server_selection() {
    let mut remote = remote();
    let answers = delayed_answers(&mut remote);
    remote.select_server("https://other.test");
    remote.status = "new server selected".into();
    remote.pending = 2;
    return_delayed(answers, account());
    assert!(remote.poll(&egui::Context::default()).is_empty());
    assert_eq!(remote.url, "https://other.test");
    assert_eq!(remote.status, "new server selected");
    assert_eq!(remote.pending, 2);
    assert!(remote.token.is_empty());
    assert!(!remote.sign_in.signed_in);
    assert!(remote.projects.is_empty() && remote.shared.is_none());
    assert!(remote.rights.is_empty());
}

#[test]
fn stale_replies_cannot_sign_back_in_after_explicit_sign_out() {
    let mut remote = remote();
    remote.token = "old-manual-token".into();
    remote.password = "unsent-password".into();
    let answers = delayed_answers(&mut remote);
    remote.sign_out();
    let status = remote.status.clone();
    return_delayed(answers, account());
    assert!(remote.poll(&egui::Context::default()).is_empty());
    assert_eq!(remote.status, status);
    assert!(remote.token.is_empty() && remote.password.is_empty());
    assert!(!remote.sign_in.signed_in && !remote.sign_in.remember);
    assert!(remote.sign_in.checked);
    assert!(remote.projects.is_empty() && remote.shared.is_none());
    assert_eq!(remote.pending, 0);
    assert!(!remote.sign_in.credentials.busy());
}

#[test]
fn only_401_marks_a_saved_sign_in_invalid() {
    let invalid = checked_account(response(401, Vec::new())).err().unwrap();
    assert!(invalid.invalid);
    for status in [403, 404, 405, 410, 500] {
        let failure = checked_account(response(status, Vec::new())).err().unwrap();
        assert!(!failure.invalid, "HTTP {status} must not revoke a saved sign-in");
    }
    assert!(!checked_account(Err("offline".into())).err().unwrap().invalid);
    assert!(!checked_account(response(200, b"malformed".to_vec())).err().unwrap().invalid);
}

#[test]
fn connection_or_legacy_server_failure_keeps_the_opt_in_for_a_later_retry() {
    for result in [response(404, Vec::new()), Err("offline".into())] {
        let mut remote = remote();
        remote.sign_in.remember = true;
        remote.sign_in.restoring = true;
        remote.sign_in.after = Some(After::Open("plant".into()));
        remote.sign_in.set_profile(&account(), true);
        let failure = checked_account(result).err().unwrap();
        assert!(remote.finish_restore(account(), "saved-token".into(), Err(failure)).is_err());
        assert!(remote.sign_in.remembered(&account()));
        assert!(remote.sign_in.remember);
        assert!(!remote.sign_in.restoring);
        assert!(remote.sign_in.after.is_none());
        assert!(remote.token.is_empty());
        assert!(!remote.sign_in.credentials.busy());
    }
}

#[test]
fn a_restored_identity_must_match_the_expected_account_and_use_the_fresh_role() {
    let mut remote = remote();
    remote.sign_in.restoring = true;
    remote.sign_in.remember = true;
    let wrong = CurrentAccount { user: ProjectUser { name: "Bob".into(), ..Default::default() }, access: Access::default() };
    assert!(remote.finish_restore(account(), "wrong-identity-token".into(), Ok(wrong)).is_err());
    assert!(!remote.sign_in.remember);
    assert!(remote.token.is_empty());
    while remote.sign_in.credentials.busy() { remote.poll_credentials(); std::thread::yield_now(); }

    let current = CurrentAccount { user: login().user, access: Access::default() };
    let restored = remote.finish_restore(account(), "valid-saved-token".into(), Ok(current.clone())).ok().unwrap();
    assert_eq!(restored.user, current.user);
    assert_eq!(restored.access, current.access);
    remote.role = Some(ProjectRole::Admin);
    remote.accept_login(&restored, true);
    assert_eq!(remote.role, Some(ProjectRole::Viewer));
    assert_eq!(remote.token, "valid-saved-token");
    assert!(remote.sign_in.signed_in);
    assert!(!remote.sign_in.credentials.busy());
}

#[test]
fn decoded_current_account_preserves_its_fresh_identity_and_permissions() {
    let current = CurrentAccount { user: login().user, access: Access::default() };
    let checked = checked_account(response(200, serde_json::to_vec(&current).unwrap())).ok().unwrap();
    assert_eq!(checked, current);
}

#[test]
fn same_account_reauthentication_preserves_the_shared_project_and_ignores_old_401_answers() {
    let mut remote = remote();
    let opened = Opened {
        session: openecraft_core::collab::Session { user: "Alice".into(), session: "old-page-session".into() },
        ..Default::default()
    };
    remote.shared = Some(openecraft_core::collab::Shared::open("synthetic-project", opened).0);
    remote.link = super::super::Link::SignedOut;
    remote.password = "synthetic-password".into();
    let generation = remote.sign_in.generation;
    remote.sign_in();
    assert!(remote.sign_in.generation > generation);
    assert_eq!(remote.shared.as_ref().unwrap().session.session, "old-page-session");
    let stale = remote.receiver("old status".into());
    let generation = remote.sign_in.generation;
    remote.accept_login(&login(), false);
    assert!(remote.sign_in.generation > generation);
    assert!(remote.sign_in.signed_in);
    assert_eq!(remote.shared.as_ref().unwrap().name, "synthetic-project");
    assert_eq!(remote.shared.as_ref().unwrap().session.session, "old-page-session");
    assert!(remote.sign_in.after.is_none());
    stale(Reply::Status("old-page-session".into(), Err(openecraft_core::collab::Failed {
        why: openecraft_core::collab::Failure::SignedOut, error: "old 401".into(),
    })));
    remote.poll(&egui::Context::default());
    assert!(remote.sign_in.signed_in);
    assert_eq!(remote.token, login().token);
}

#[test]
fn a_further_account_switch_keeps_pending_old_account_revocation_and_preference_warnings() {
    let mut remote = remote();
    let cleanup_generation = remote.sign_in.generation;
    remote.sign_in.revoking.push((cleanup_generation, account()));
    remote.sign_in.preference_error = "synthetic preference write failure".into();
    remote.select_server("https://other.test");
    assert_eq!(remote.sign_in.generation, cleanup_generation + 1);
    assert_eq!(remote.sign_in.revoking, [(cleanup_generation, account())]);
    assert_eq!(remote.sign_in.preference_error, "synthetic preference write failure");
    assert!(remote.token.is_empty());
    assert!(!remote.sign_in.credentials.busy());
}

#[test]
fn disabling_a_profile_removes_only_that_server_account_from_nonsecret_metadata() {
    let mut sign_in = SignIn::default();
    let alice = account();
    let bob = Account::new(alice.server(), "Bob").unwrap();
    let other_alice = Account::new("https://other.test", alice.user()).unwrap();
    assert!(sign_in.set_profile(&alice, true));
    assert!(sign_in.set_profile(&bob, true));
    assert!(sign_in.set_profile(&other_alice, true));
    assert!(sign_in.set_profile(&alice, false));
    assert!(!sign_in.remembered(&alice));
    assert!(sign_in.remembered(&bob));
    assert!(sign_in.remembered(&other_alice));
    let encoded = serde_json::to_vec(&sign_in.profiles).unwrap();
    let decoded: Vec<Profile> = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, [Profile::of(&bob), Profile::of(&other_alice)]);
    assert!(!sign_in.credentials.busy());
}

/// Uses only the disposable server, unique accounts and isolated metadata folder
/// created by the companion script; no existing Windows credentials are read.
#[cfg(windows)]
#[test]
#[ignore = "requires disposable loopback server; scripts/test-project-sign-in.ps1"]
fn live_auth_http_remembered_project_sign_in_is_isolated_revocable_and_safe_during_restore() {
    use openecraft_client::project_credentials::CredentialStore;
    use openecraft_core::model::{Page, PageKind, Project};
    use std::{sync::mpsc, time::{Duration, Instant}};

    fn setting(key: &str) -> String { std::env::var(key).unwrap_or_else(|_| panic!("synthetic test setting is required")) }
    fn fixture(url: &str, user: &str, directory: &str, ctx: &egui::Context) -> Remote {
        let mut remote = Remote { url: url.into(), user: user.into(), ctx: Some(ctx.clone()), ..Default::default() };
        let store = Store::at(directory);
        store.load(ctx, PROFILES_KEY);
        remote.sign_in.store = Some(store);
        remote.sign_in.load_profiles();
        remote
    }
    fn wait(remote: &mut Remote, ctx: &egui::Context, done: impl Fn(&Remote) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut idle = 0;
        loop {
            remote.poll(ctx);
            if done(remote) && !remote.busy() && remote.sign_in.revoking.is_empty() {
                idle += 1;
                if idle == 2 { return; }
            } else { idle = 0; }
            assert!(Instant::now() < deadline, "synthetic sign-in operation did not finish");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn status(request: ehttp::Request) -> u16 {
        let (tx, rx) = mpsc::channel();
        ehttp::fetch(request, move |result| { let _ = tx.send(result.map(|response| response.status)); });
        match rx.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(status)) => status,
            _ => panic!("synthetic HTTP check did not finish"),
        }
    }
    fn request<T: serde::de::DeserializeOwned + Send + 'static>(request: ehttp::Request) -> T {
        let (tx, rx) = mpsc::channel();
        ehttp::fetch(request, move |r| { let _ = tx.send(decode::<T>(r)); });
        match rx.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(value)) => value,
            _ => panic!("synthetic JSON request failed"),
        }
    }
    fn wait_shared(remote: &mut Remote, session: &mut crate::session::Session, ctx: &egui::Context, done: impl Fn(&Remote) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            for reply in remote.poll(ctx) {
                match reply {
                    Reply::Opened(name, Ok(opened)) => {
                        let (shared, project) = openecraft_core::collab::Shared::open(&name, *opened);
                        *session = crate::session::Session::new(project, None);
                        remote.shared = Some(shared);
                    }
                    Reply::Reopened(_, Ok(opened)) => { remote.take_reopened(session, *opened, "synthetic reauthentication".into()); }
                    Reply::Status(_, Ok(status)) => remote.take_status(status, &[]),
                    _ => {}
                }
            }
            if done(remote) && !remote.busy() { return; }
            assert!(Instant::now() < deadline, "synthetic shared reauthentication did not finish");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn stored(account: &Account) -> Option<String> {
        match SystemCredentialStore.load(account) {
            Ok(token) => token,
            Err(_) => panic!("synthetic Windows credential check failed"),
        }
    }
    struct Cleanup(Vec<Account>);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            for account in &self.0 { let _ = SystemCredentialStore.forget(account); }
        }
    }

    let url = setting("OPENECRAFT_AUTH_TEST_URL");
    let port = url.strip_prefix("http://127.0.0.1:").and_then(|rest| rest.split('/').next()).and_then(|port| port.parse::<u16>().ok()).expect("only a disposable loopback server is allowed");
    assert!(port > 1024 && port != 8091, "the test must use its own loopback port");
    let user = setting("OPENECRAFT_AUTH_TEST_USER");
    let password = setting("OPENECRAFT_AUTH_TEST_PASSWORD");
    let other_user = setting("OPENECRAFT_AUTH_TEST_OTHER_USER");
    let other_password = setting("OPENECRAFT_AUTH_TEST_OTHER_PASSWORD");
    assert!(user != other_user, "two synthetic accounts are required");
    let directory = setting("OPENECRAFT_AUTH_TEST_PROFILE_DIR");
    assert!(std::path::Path::new(&directory).is_absolute(), "an isolated absolute metadata folder is required");
    let alice = Account::new(&url, &user).expect("valid synthetic account");
    let bob = Account::new(&url, &other_user).expect("valid synthetic account");
    let _cleanup = Cleanup(vec![alice.clone(), bob.clone()]);
    let ctx = egui::Context::default();
    assert!(stored(&alice).is_none(), "synthetic credentials must start absent");
    assert!(stored(&bob).is_none(), "synthetic credentials must start absent");

    let mut opt_out = fixture(&url, &user, &directory, &ctx);
    opt_out.password = password.clone();
    opt_out.sign_in();
    wait(&mut opt_out, &ctx, |remote| remote.sign_in.signed_in);
    assert!(!opt_out.has_saved_sign_in(), "opt-out must not create remembered login metadata");
    assert!(stored(&alice).is_none(), "opt-out must not store a token");
    let mut restarted = fixture(&url, &user, &directory, &ctx);
    restarted.restore_sign_in();
    assert!(restarted.token.is_empty() && !restarted.sign_in.signed_in && !restarted.busy(), "opt-out restart must stay signed out");
    opt_out.sign_out();
    wait(&mut opt_out, &ctx, |remote| remote.token.is_empty());

    let mut first = fixture(&url, &user, &directory, &ctx);
    first.password = password.clone();
    first.sign_in.remember = true;
    first.sign_in();
    wait(&mut first, &ctx, |remote| remote.sign_in.signed_in && remote.has_saved_sign_in());
    let first_token = first.token.clone();
    assert!(stored(&alice).as_deref() == Some(first_token.as_str()), "opt-in must store this account's token in Windows");
    drop(first);
    let mut restored = fixture(&url, &user, &directory, &ctx);
    restored.restore_sign_in();
    wait(&mut restored, &ctx, |remote| remote.sign_in.signed_in);
    assert!(restored.token == first_token, "restart must use the saved token");
    assert!(restored.user == user, "restart must validate the expected account");
    println!("PASS: opt-out restart stays signed out; opt-in restart validates the protected Windows token");

    let mut second = fixture(&url, &other_user, &directory, &ctx);
    second.password = other_password.clone();
    second.sign_in.remember = true;
    second.sign_in();
    wait(&mut second, &ctx, |remote| remote.sign_in.signed_in && remote.has_saved_sign_in());
    let second_token = second.token.clone();
    assert!(first_token != second_token, "different accounts must have different sessions");
    assert!(stored(&alice).as_deref() == Some(first_token.as_str()), "remembering another account must preserve the first credential");
    assert!(stored(&bob).as_deref() == Some(second_token.as_str()), "the second account must use its own credential key");
    for entry in std::fs::read_dir(&directory).expect("isolated metadata folder").flatten() {
        if !entry.file_name().to_string_lossy().starts_with("project_server_sign_in") { continue; }
        let data = std::fs::read_to_string(entry.path()).expect("synthetic metadata");
        assert!(!data.contains(&first_token) && !data.contains(&second_token) && !data.contains(&password) && !data.contains(&other_password), "ordinary metadata must not contain a token or password");
    }
    restored.select_server(&format!("{}/other-prefix", url.trim_end_matches('/')));
    assert!(restored.token.is_empty() && !restored.sign_in.signed_in && restored.role.is_none() && restored.projects.is_empty(), "server switch must clear all authentication context");
    restored.select_server(&url);
    restored.restore_sign_in();
    wait(&mut restored, &ctx, |remote| remote.sign_in.signed_in);
    restored.sign_out();
    wait(&mut restored, &ctx, |remote| remote.token.is_empty() && status(ProjectEndpoint { url: url.clone(), token: first_token.clone() }.current_account()) == 401);
    assert!(stored(&alice).is_none(), "sign-out must remove only its own OS credential");
    assert!(status(ProjectEndpoint { url: url.clone(), token: first_token.clone() }.current_account()) == 401, "signed-out token must be refused by the real server");
    let mut second_restart = fixture(&url, &other_user, &directory, &ctx);
    second_restart.restore_sign_in();
    wait(&mut second_restart, &ctx, |remote| remote.sign_in.signed_in);
    assert!(second_restart.token == second_token, "sign-out in another window must preserve this account's opt-in");
    second_restart.change_user();
    assert!(second_restart.user.is_empty());
    assert_eq!(stored(&bob).as_deref(), Some(second_token.as_str()));
    assert_eq!(status(ProjectEndpoint { url: url.clone(), token: second_token.clone() }.current_account()), 200);
    second_restart.user = other_user.clone();
    second_restart.credentials_edited(&url, "", "");
    second_restart.restore_sign_in();
    wait(&mut second_restart, &ctx, |remote| remote.sign_in.signed_in);
    assert_eq!(second_restart.token, second_token);
    println!("PASS: Change user keeps the remembered token valid and switching back restores it without a password");
    println!("PASS: two accounts and simultaneous windows keep separate credentials and opt-in preferences; logout revokes the old token");

    let mut revoked = fixture(&url, &user, &directory, &ctx);
    revoked.password = password.clone();
    revoked.sign_in.remember = true;
    revoked.sign_in();
    wait(&mut revoked, &ctx, |remote| remote.sign_in.signed_in && remote.has_saved_sign_in());
    let revoked_token = revoked.token.clone();
    assert!(status(ProjectEndpoint { url: url.clone(), token: revoked_token }.logout()) == 204, "synthetic external revocation must succeed");
    drop(revoked);
    let mut rejected = fixture(&url, &user, &directory, &ctx);
    rejected.restore_sign_in();
    wait(&mut rejected, &ctx, |remote| !remote.sign_in.restoring && !remote.has_saved_sign_in());
    assert!(rejected.token.is_empty() && !rejected.sign_in.signed_in, "revoked credential must never restore a login");
    assert!(stored(&alice).is_none(), "revoked credential must be forgotten locally");
    println!("PASS: server-side revocation causes 401 and removes only the rejected saved token");

    let mut cancelling = fixture(&url, &user, &directory, &ctx);
    cancelling.password = password.clone();
    cancelling.sign_in.remember = true;
    cancelling.sign_in();
    wait(&mut cancelling, &ctx, |remote| remote.sign_in.signed_in && remote.has_saved_sign_in());
    let cancelled_token = cancelling.token.clone();
    drop(cancelling);
    let mut cancelled = fixture(&url, &user, &directory, &ctx);
    cancelled.restore_sign_in();
    assert!(cancelled.sign_in.restoring, "restore must have started before cancellation");
    cancelled.sign_out();
    wait(&mut cancelled, &ctx, |remote| remote.token.is_empty() && !remote.sign_in.restoring && status(ProjectEndpoint { url: url.clone(), token: cancelled_token.clone() }.current_account()) == 401);
    assert!(!cancelled.sign_in.signed_in && !cancelled.has_saved_sign_in(), "late OS and HTTP replies must not sign back in");
    assert!(stored(&alice).is_none(), "cancelled restore must forget its protected credential");
    assert!(status(ProjectEndpoint { url: url.clone(), token: cancelled_token }.current_account()) == 401, "sign-out during restore must still revoke the real server token");
    println!("PASS: immediate sign-out during restore loads, revokes and forgets in order without signing back in");

    let mut reauth = fixture(&url, &user, &directory, &ctx);
    reauth.password = password.clone();
    reauth.sign_in();
    wait(&mut reauth, &ctx, |remote| remote.sign_in.signed_in);
    let mut project = Project { pages: vec![Page::new(1, "1", "Synthetic baseline", PageKind::Schematic)], ..Default::default() };
    project.normalize();
    let name = format!("reauth-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let mut session = crate::session::Session::new(project.clone(), None);
    reauth.create(&name, &project);
    wait_shared(&mut reauth, &mut session, &ctx, |remote| remote.shared.is_some());
    let old_session = reauth.shared.as_ref().unwrap().session.session.clone();
    session.prj.pages[0].description = "Synthetic unsaved reauthentication draft".into();
    assert!(session.commit(true));
    let before = session.prj.clone();
    let history = (session.history.state(), session.history.rev);
    assert_eq!(status(reauth.endpoint().logout()), 204);
    reauth.heartbeat(0.0);
    wait(&mut reauth, &ctx, |remote| remote.link == super::super::Link::SignedOut);
    assert!(!reauth.sign_in.signed_in);
    assert_eq!(reauth.shared.as_ref().unwrap().session.session, old_session);
    reauth.password = password;
    reauth.sign_in();
    assert_eq!(reauth.shared.as_ref().unwrap().session.session, old_session);
    assert_eq!(session.prj, before);
    assert_eq!((session.history.state(), session.history.rev), history);
    wait_shared(&mut reauth, &mut session, &ctx, |remote| remote.online() && remote.shared.as_ref().is_some_and(|shared| shared.session.session != old_session));
    assert_eq!(session.prj.pages[0].description, "Synthetic unsaved reauthentication draft");
    assert!(session.dirty());
    session.undo();
    assert_eq!(session.prj.pages[0].description, "Synthetic baseline");
    session.redo();
    assert_eq!(session.prj.pages[0].description, "Synthetic unsaved reauthentication draft");
    println!("PASS: revoked authentication returns 401, same-account sign-in reopens the shared project and preserves its unsaved draft and undo/redo");

    let observer = second_restart.endpoint();
    let observer_open: Opened = request(observer.open(&name, &other_user));
    let endpoint = reauth.endpoint();
    let sent = reauth.begin_save(&session.prj, Vec::new()).unwrap();
    let delayed = reauth.receiver("synthetic delayed save".into());
    let shared = reauth.shared.as_ref().unwrap();
    let snapshot: openecraft_core::collab::Snapshot = request(endpoint.save(&name, &shared.session, &sent.project, sent.release.clone(), sent.revision));
    assert_eq!(snapshot.revision, 2);
    assert!(!snapshot.locks.is_empty(), "the real save must hold a lock before logout");
    reauth.sign_out();
    reauth.poll(&ctx);
    assert!(!reauth.busy(), "waiting for save/close/logout must not block the local sign-in pane");
    assert_eq!(status(endpoint.current_account()), 200, "the token is needed until deferred close completes");
    delayed(Reply::Saved(Box::new(sent), Ok(Box::new(snapshot))));
    wait(&mut reauth, &ctx, |remote| remote.token.is_empty() && status(endpoint.current_account()) == 401);
    let released: openecraft_core::collab::Status = request(observer.status(&name, &observer_open.session));
    assert!(released.locks.is_empty(), "logout must release the actual server locks");
    let readback: Opened = request(observer.open(&name, &other_user));
    assert_eq!(readback.snapshot.revision, 2);
    assert_eq!(readback.snapshot.project.pages[0].description, "Synthetic unsaved reauthentication draft");
    assert_eq!(status(observer.close(&name, &observer_open.session)), 200);
    assert_eq!(status(observer.close(&name, &readback.session)), 200);
    println!("PASS: delayed real Save -> Close answer -> Logout; observer confirms revision 2, expected contents and zero server locks");

    second_restart.sign_out();
    wait(&mut second_restart, &ctx, |remote| remote.token.is_empty() && status(ProjectEndpoint { url: url.clone(), token: second_token.clone() }.current_account()) == 401);
    assert!(stored(&bob).is_none(), "second synthetic credential must be cleaned up");
    assert!(status(ProjectEndpoint { url, token: second_token }.current_account()) == 401, "second sign-out must revoke only that synthetic token");
}

#[test]
fn authenticated_startup_open_waits_for_the_app_and_is_taken_exactly_once() {
    let mut remote = remote();
    remote.sign_in.after = Some(After::Open("synthetic-start-project".into()));
    remote.accept_login(&login(), true);
    assert!(remote.sign_in.signed_in);
    assert_eq!(remote.pending, 0, "authentication must not send a project request before the app checks current unsaved work");
    assert!(!remote.sign_in.credentials.busy());
    assert_eq!(remote.take_authenticated_open().as_deref(), Some("synthetic-start-project"));
    assert!(remote.take_authenticated_open().is_none(), "the app must receive only one opening request");
    assert_eq!(remote.pending, 0);
}

#[test]
fn another_windows_opt_out_and_new_opt_in_each_supersede_an_old_save_intent() {
    use std::path::PathBuf;
    struct Cleanup { directory: PathBuf, flag: String }
    impl Drop for Cleanup {
        fn drop(&mut self) {
            for key in [PROFILES_KEY, self.flag.as_str()] { let _ = std::fs::remove_file(self.directory.join(key)); }
            let _ = std::fs::remove_dir(&self.directory);
        }
    }
    let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let directory = std::env::temp_dir().join(format!("openecraft-auth-intent-{}-{unique}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let account = account();
    let _cleanup = Cleanup { directory: directory.clone(), flag: Profile::of(&account).key() };
    let store = || Some(Store::at(directory.to_string_lossy()));
    let mut first = SignIn { store: store(), loaded: true, ..Default::default() };
    let mut second = SignIn { store: store(), loaded: true, ..Default::default() };
    let ctx = egui::Context::default();
    assert!(first.set_profile(&account, true));
    assert!(first.intent_matches(&account, &ctx));
    let first_intent = first.intent.clone();
    assert!(second.set_profile(&account, false));
    assert!(!first.intent_matches(&account, &ctx), "another window's sign-out must invalidate this window's pending token save");
    assert_eq!(first.intent, first_intent);
    assert!(second.set_profile(&account, true));
    assert!(second.intent_matches(&account, &ctx));
    assert!(!first.intent_matches(&account, &ctx), "a later opt-in must not revive a previous window's stale save intent");
    assert_eq!(first.intent, first_intent);
    assert!(first.remembered(&account), "the new opt-in remains discoverable without adopting its revision");
    assert!(!first.credentials.busy() && !second.credentials.busy());
}

#[test]
fn canonical_login_name_preserves_the_intent_for_case_insensitive_server_accounts() {
    let mut sign_in = SignIn::default();
    let typed = Account::new("https://server.test/projects", "alice").unwrap();
    assert!(sign_in.set_profile(&typed, true));
    let canonical_login = login();
    assert_eq!(canonical_login.user.name, "Alice");
    let canonical_account = Account::new(typed.server(), &canonical_login.user.name).unwrap();
    assert_eq!(canonical_account, typed);
    let ctx = egui::Context::default();
    assert!(sign_in.intent_matches(&canonical_account, &ctx), "a canonicalized login must keep the explicit opt-in for the same account");
    assert!(!sign_in.credentials.busy());

    let mut remote = remote();
    let current = CurrentAccount { user: canonical_login.user, access: canonical_login.access };
    let restored = remote.finish_restore(typed, "synthetic-restored-token".into(), Ok(current)).ok().unwrap();
    assert_eq!(restored.user.name, "Alice", "the UI keeps the server's canonical display name");
    assert!(!remote.sign_in.credentials.busy());
}


#[test]
fn review193_correcting_identity_preserves_typed_password_and_opt_in() {
    for change_server in [false, true] {
        let mut r = remote();
        r.password = "synthetic-typed-password".into();
        r.sign_in.remember = true;
        let old = (r.url.clone(), r.user.clone(), r.token.clone());
        if change_server { r.url.push_str("-corrected"); } else { r.user = "Corrected".into(); }
        r.credentials_edited(&old.0, &old.1, &old.2);
        assert_eq!(r.password, "synthetic-typed-password");
        assert!(r.sign_in.remember);
        assert!(r.token.is_empty());
    }
}

#[test]
fn review193_offline_logout_does_not_block_the_next_sign_in() {
    let mut r = remote();
    r.token = "synthetic-token".into();
    r.sign_in.signed_in = true;
    r.sign_out();
    assert!(!r.busy(), "revocation must not disable sign-in while the host is unreachable");
    assert!(r.token.is_empty());
}

#[cfg(windows)]
#[test]
fn review193_unremembered_login_does_not_delete_or_create_preferences() {
    let mut r = remote();
    r.url = format!("http://127.0.0.1:9/review193-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    r.password = "synthetic".into();
    r.sign_in();
    assert!(r.sign_in.notice.is_empty(), "no removal notice for an account that was never remembered");
    assert!(!r.sign_in.credentials.busy(), "no OS deletion for an account without a profile");
}

#[test]
fn review193_directory_login_moves_the_opt_in_to_the_canonical_name() {
    for typed_name in [r"DOMAIN\Alice", "alice@example.test"] {
        let mut r = remote();
        r.url = format!("http://127.0.0.1:9/canonical-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        r.user = typed_name.into();
        r.sign_in.remember = true;
        let typed = Account::new(&r.url, &r.user).unwrap();
        let canonical = Account::new(&r.url, "Alice").unwrap();
        #[cfg(windows)]
        struct Cleanup(Account);
        #[cfg(windows)]
        impl Drop for Cleanup {
            fn drop(&mut self) {
                use openecraft_client::project_credentials::CredentialStore;
                let _ = SystemCredentialStore.forget(&self.0);
            }
        }
        #[cfg(windows)]
        let _cleanup = Cleanup(canonical.clone());
        r.sign_in.set_profile(&typed, true);
        r.accept_login(&login(), false);
        assert!(r.sign_in.remember, "the server's account name must retain the explicit opt-in");
        assert!(!r.sign_in.remembered(&typed));
        assert!(r.sign_in.remembered(&canonical));
        review193_drain_credentials(&mut r);
        #[cfg(windows)] {
            use openecraft_client::project_credentials::CredentialStore;
            assert_eq!(SystemCredentialStore.load(&canonical).unwrap().as_deref(), Some("synthetic-session-token"));
        }
    }
}

fn review193_drain_credentials(r: &mut Remote) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while r.sign_in.credentials.busy() {
        r.poll_credentials();
        assert!(std::time::Instant::now() < deadline, "synthetic credential operation timed out");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

#[cfg(windows)]
#[test]
fn review193_rejected_old_restore_cannot_delete_a_new_windows_token_or_preference() {
    use openecraft_client::project_credentials::CredentialStore;
    let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let url = format!("http://127.0.0.1:9/stale-{unique}");
    let account = Account::new(&url, "synthetic-Alice").unwrap();
    let directory = std::env::temp_dir().join(format!("openecraft-review193-{unique}"));
    std::fs::create_dir(&directory).unwrap();
    struct Cleanup(Account, std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = SystemCredentialStore.forget(&self.0);
            for key in [PROFILES_KEY.to_string(), Profile::of(&self.0).key()] { let _ = std::fs::remove_file(self.1.join(key)); }
            let _ = std::fs::remove_dir(&self.1);
        }
    }
    let _cleanup = Cleanup(account.clone(), directory.clone());
    let fixture = || {
        let mut r = Remote { url: url.clone(), user: "synthetic-Alice".into(), ..Default::default() };
        r.sign_in.store = Some(Store::at(directory.to_string_lossy()));
        r.sign_in.loaded = true;
        r
    };
    let ctx = egui::Context::default();
    let mut old = fixture();
    old.sign_in.set_profile(&account, true);
    SystemCredentialStore.save(&account, "old-fixture").unwrap();
    let mut new = fixture();
    new.sign_in.set_profile(&account, true);
    SystemCredentialStore.save(&account, "new-fixture").unwrap();
    let revision = new.sign_in.load_selected(&account, &ctx);
    let failure = checked_account(response(401, Vec::new())).err().unwrap();
    assert!(old.finish_restore(account.clone(), "old-fixture".into(), Err(failure)).is_err());
    review193_drain_credentials(&mut old);
    assert_eq!(SystemCredentialStore.load(&account).unwrap().as_deref(), Some("new-fixture"));
    assert_eq!(new.sign_in.load_selected(&account, &ctx), revision);
    assert!(new.sign_in.remembered(&account));

    // A Load(None) completed before the newer window saved. Deliver it last.
    old.take_credentials(vec![super::super::credentials::Finished {
        generation: old.sign_in.generation, account: account.clone(),
        result: Ok(None), kind: Kind::Load, removed: false,
    }]);
    assert_eq!(SystemCredentialStore.load(&account).unwrap().as_deref(), Some("new-fixture"));
    assert_eq!(new.sign_in.load_selected(&account, &ctx), revision);
    assert!(new.sign_in.remembered(&account));
}

#[test]
fn review193_remember_hint_distinguishes_windows_input_from_platform_support() {
    let mut r = remote();
    r.user.clear();
    if cfg!(windows) { assert!(r.remember_hint().contains("Enter a user name")); }
    else { assert!(r.remember_hint().contains("Browser")); }
    r.user = "Alice".into();
    r.url = "server-without-scheme".into();
    if cfg!(windows) { assert!(r.remember_hint().contains("HTTP or HTTPS")); }
    r.url = "http://127.0.0.1:9".into();
    if cfg!(windows) { assert!(r.remember_hint().contains("Windows Credential Manager")); }
}


#[test]
fn review193_no_removal_notice_for_an_already_absent_entry() {
    let mut r = remote();
    for removed in [false, true] {
        r.sign_in.notice = "Removing the saved sign-in…".into();
        r.take_credentials(vec![super::super::credentials::Finished {
            generation: r.sign_in.generation, account: account(),
            result: Ok(None), kind: Kind::Forget, removed,
        }]);
        assert_eq!(!r.sign_in.notice.is_empty(), removed);
    }
}
