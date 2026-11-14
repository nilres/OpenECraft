use super::*;
use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use openecraft_core::model::{Page, PageKind, Project};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tower::ServiceExt;

const TOKEN: &str = "secret-token";

#[tokio::test]
async fn session_snapshot_keeps_own_locks_and_never_exposes_another_sessions_id() {
    let app = app(Some(TOKEN));
    let mut tokens = Vec::new();
    for user in ["anna", "ben"] {
        let update = UserUpdate { password: Some(format!("{user}-password")), role: Some(ProjectRole::Editor), ..Default::default() };
        assert!(with(&app, "PUT", &format!("/api/v1/users/{user}"), TOKEN, &update).await.0.is_success());
        let (_, body) = with(&app, "POST", "/api/v1/login", TOKEN, &LoginRequest { user: user.into(), password: format!("{user}-password"), ..Default::default() }).await;
        tokens.push(serde_json::from_str::<LoginResponse>(&body).unwrap().token);
    }
    let (_, body) = with(&app, "POST", "/api/v1/projects", &tokens[0], &CreateRequest { name: "Snapshot test".into(), user: "anna".into(), project: two_pages() }).await;
    let anna: Opened = serde_json::from_str(&body).unwrap();
    let path = "/api/v1/projects/Snapshot%20test";
    let (_, body) = with(&app, "POST", &format!("{path}/open"), &tokens[1], &OpenRequest { user: "ben".into() }).await;
    let ben: Opened = serde_json::from_str(&body).unwrap();
    for (opened, token, target) in [(&anna, &tokens[0], Target::Project), (&ben, &tokens[1], Target::Page(anna.snapshot.project.pages[1].id))] {
        assert!(with(&app, "POST", &format!("{path}/lock"), token, &LockRequest { session: opened.session.clone(), targets: vec![target] }).await.0.is_success());
    }
    let (code, body) = with(&app, "POST", &format!("{path}/snapshot"), &tokens[0], &anna.session).await;
    assert!(code.is_success());
    let snap: Snapshot = serde_json::from_str(&body).unwrap();
    assert!(snap.locks.iter().any(|l| l.target == Target::Project && l.session == anna.session.session));
    assert!(snap.locks.iter().filter(|l| l.user == "ben").all(|l| l.session != ben.session.session));
    assert!(!body.contains(&ben.session.session));
    let (_, body) = with(&app, "GET", path, &tokens[0], &()).await;
    assert!(!body.contains(&anna.session.session));
    assert!(!body.contains(&ben.session.session));
    assert_eq!(with(&app, "POST", &format!("{path}/snapshot"), &tokens[0], &ben.session).await.0, StatusCode::FORBIDDEN);
    let invalid = Session { session: "not-a-session".into(), ..anna.session };
    assert!(!with(&app, "POST", &format!("{path}/snapshot"), &tokens[0], &invalid).await.0.is_success());
}

fn app(token: Option<&str>) -> Router {
    app_with(token, false)
}

fn app_with(token: Option<&str>, web: bool) -> Router {
    let server = Server { store: Store::in_memory(), sessions: Sessions::default(), users: Users::default(), access: AccessFile::in_memory(Default::default()), throttle: Throttle::default(), views: Views::default() };
    router(AppState { server: Arc::new(Mutex::new(server)), token: token.map(String::from), web })
}

async fn call(app: &Router, method: &str, uri: &str, token: Option<&str>, body: String) -> (StatusCode, String) {
    let mut req = Request::builder().method(method).uri(uri).header(header::CONTENT_TYPE, "application/json");
    if let Some(t) = token {
        req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let resp = app.clone().oneshot(req.body(Body::from(body)).unwrap()).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&bytes).to_string())
}

/// Request that must succeed, decoded.
async fn ok<T: DeserializeOwned>(app: &Router, method: &str, uri: &str, body: &impl Serialize) -> T {
    let (status, text) = call(app, method, uri, None, serde_json::to_string(body).unwrap()).await;
    assert!(status.is_success(), "{method} {uri}: {status} {text}");
    serde_json::from_str(&text).unwrap()
}

async fn error(app: &Router, method: &str, uri: &str, body: &impl Serialize) -> (StatusCode, String) {
    let (status, text) = call(app, method, uri, None, serde_json::to_string(body).unwrap()).await;
    (status, serde_json::from_str::<ApiError>(&text).map(|e| e.error).unwrap_or(text))
}

fn two_pages() -> Project {
    let mut prj = Project::default();
    let id = prj.new_id();
    prj.pages.push(Page::new(id, "2", "Motors", PageKind::Schematic));
    prj
}

async fn create(app: &Router, user: &str) -> Opened {
    let req = CreateRequest { name: "Plant 1".into(), user: user.into(), project: two_pages() };
    ok(app, "POST", "/api/v1/projects", &req).await
}

async fn open(app: &Router, user: &str) -> Opened {
    ok(app, "POST", "/api/v1/projects/Plant%201/open", &OpenRequest { user: user.into() }).await
}

#[tokio::test]
async fn create_list_and_open() {
    let app = app(None);
    let anna = create(&app, "anna").await;
    assert_eq!(anna.snapshot.revision, 1);
    assert_eq!(anna.snapshot.project.pages.len(), 2);
    let ben = open(&app, "ben").await;
    assert_ne!(anna.session.session, ben.session.session);
    // Every session gets its own ids.
    assert!(ben.first_id >= anna.first_id + openecraft_core::collab::ID_BLOCK);

    let list: Vec<ProjectSummary> = ok(&app, "GET", "/api/v1/projects", &()).await;
    assert_eq!(list.len(), 1);
    assert_eq!((list[0].name.as_str(), list[0].pages), ("Plant 1", 2));
    assert_eq!(list[0].users, vec!["anna".to_string(), "ben".to_string()]);

    let info: ServerInfo = ok(&app, "GET", "/api/v1/info", &()).await;
    assert_eq!(info.projects, 1);

    let req = CreateRequest { name: "Plant 1".into(), user: "ben".into(), project: Project::default() };
    assert_eq!(error(&app, "POST", "/api/v1/projects", &req).await.0, StatusCode::CONFLICT);
    // Windows and macOS would store it in the same file.
    let req = CreateRequest { name: "plant 1".into(), user: "ben".into(), project: Project::default() };
    let (status, text) = error(&app, "POST", "/api/v1/projects", &req).await;
    assert_eq!((status, text.as_str()), (StatusCode::CONFLICT, "project 'Plant 1' already exists"));
    let req = CreateRequest { name: "../x".into(), user: "ben".into(), project: Project::default() };
    assert_eq!(error(&app, "POST", "/api/v1/projects", &req).await.0, StatusCode::BAD_REQUEST);
    let req = CreateRequest { name: "x".into(), user: " ".into(), project: Project::default() };
    assert_eq!(error(&app, "POST", "/api/v1/projects", &req).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(error(&app, "POST", "/api/v1/projects/nope/open", &OpenRequest { user: "ben".into() }).await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn two_users_edit_different_pages() {
    let app = app(None);
    let anna = create(&app, "anna").await;
    let ben = open(&app, "ben").await;
    let (p1, p2) = (anna.snapshot.project.pages[0].id, anna.snapshot.project.pages[1].id);
    let path = "/api/v1/projects/Plant%201";

    let lock = |s: &Opened, t: Target| LockRequest { session: s.session.clone(), targets: vec![t] };
    let _: Status = ok(&app, "POST", &format!("{path}/lock"), &lock(&anna, Target::Page(p1))).await;
    let status: Status = ok(&app, "POST", &format!("{path}/lock"), &lock(&ben, Target::Page(p2))).await;
    assert_eq!(status.locks.len(), 2);
    let (code, msg) = error(&app, "POST", &format!("{path}/lock"), &lock(&ben, Target::Page(p1))).await;
    assert_eq!(code, StatusCode::CONFLICT);
    assert!(msg.contains("anna"), "{msg}");
    assert_eq!(error(&app, "POST", &format!("{path}/lock"), &lock(&ben, Target::Page(999_999))).await.0, StatusCode::NOT_FOUND);

    // Both change both pages and the project name; each keeps only what they locked.
    let mut a = anna.snapshot.project.clone();
    a.pages[0].description = "anna 1".into();
    a.pages[1].description = "anna 2".into();
    a.info.name = "anna".into();
    let mut b = ben.snapshot.project.clone();
    b.pages[0].description = "ben 1".into();
    b.pages[1].description = "ben 2".into();
    let saved: Snapshot = ok(&app, "PUT", path, &SaveRequest { session: anna.session.clone(), project: a, release: Vec::new(), ..Default::default() }).await;
    assert_eq!(saved.revision, 2);
    let saved: Snapshot = ok(&app, "PUT", path, &SaveRequest { session: ben.session.clone(), project: b, release: vec![Target::Page(p2)], ..Default::default() }).await;
    assert_eq!(saved.revision, 3);
    assert_eq!(saved.project.pages[0].description, "anna 1");
    assert_eq!(saved.project.pages[1].description, "ben 2");
    assert_eq!(saved.project.info.name, anna.snapshot.project.info.name);
    assert_eq!(saved.locks.len(), 1);

    let snap: Snapshot = ok(&app, "GET", path, &()).await;
    assert_eq!(snap, saved);
    let status: Status = ok(&app, "POST", &format!("{path}/status"), &ben.session).await;
    assert_eq!(status.revision, 3);
}

#[tokio::test]
async fn pages_locked_by_others_cannot_be_deleted() {
    let app = app(None);
    let anna = create(&app, "anna").await;
    let ben = open(&app, "ben").await;
    let p2 = anna.snapshot.project.pages[1].id;
    let path = "/api/v1/projects/Plant%201";
    let _: Status = ok(&app, "POST", &format!("{path}/lock"), &LockRequest { session: anna.session.clone(), targets: vec![Target::Project] }).await;
    let _: Status = ok(&app, "POST", &format!("{path}/lock"), &LockRequest { session: ben.session.clone(), targets: vec![Target::Page(p2)] }).await;
    let mut a = anna.snapshot.project.clone();
    a.pages.pop();
    let req = SaveRequest { session: anna.session.clone(), project: a, release: Vec::new(), ..Default::default() };
    let (code, msg) = error(&app, "PUT", path, &req).await;
    assert_eq!(code, StatusCode::CONFLICT);
    assert!(msg.contains("ben"), "{msg}");

    // Released: now it can go.
    let _: Status = ok(&app, "POST", &format!("{path}/close"), &ben.session).await;
    let saved: Snapshot = ok(&app, "PUT", path, &req).await;
    assert_eq!(saved.project.pages.len(), 1);
    // The closed session is gone.
    assert_eq!(error(&app, "POST", &format!("{path}/status"), &ben.session).await.0, StatusCode::GONE);
}

#[tokio::test]
async fn token_is_required_for_everything() {
    let app = app(Some(TOKEN));
    assert_eq!(call(&app, "GET", "/api/v1/projects", None, String::new()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(call(&app, "GET", "/api/v1/projects", Some("wrong"), String::new()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(call(&app, "GET", "/api/v1/projects", Some(TOKEN), String::new()).await.0, StatusCode::OK);
    let req = serde_json::to_string(&CreateRequest { name: "p".into(), user: "anna".into(), project: Project::default() }).unwrap();
    assert_eq!(call(&app, "POST", "/api/v1/projects", None, req.clone()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(call(&app, "POST", "/api/v1/projects", Some(TOKEN), req).await.0, StatusCode::CREATED);
}

/// Request with a bearer token; returns status and body.
async fn with(app: &Router, method: &str, uri: &str, token: &str, body: &impl Serialize) -> (StatusCode, String) {
    call(app, method, uri, Some(token), serde_json::to_string(body).unwrap()).await
}

#[tokio::test]
async fn users_roles_and_projects() {
    let app = app(Some(TOKEN));
    // The server token creates the users.
    for (name, role, projects) in [("anna", ProjectRole::Editor, vec![]), ("vic", ProjectRole::Viewer, vec![]), ("eve", ProjectRole::Editor, vec!["Office".to_string()])] {
        let req = UserUpdate { password: Some(format!("{name}-password")), role: Some(role), projects, groups: None };
        let (s, text) = with(&app, "PUT", &format!("/api/v1/users/{name}"), TOKEN, &req).await;
        assert!(s.is_success(), "{s} {text}");
    }
    let (s, _) = with(&app, "PUT", "/api/v1/users/nopw", TOKEN, &UserUpdate::default()).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "a new user needs a password");
    // Without a token nothing works any more.
    let (s, _) = call(&app, "GET", "/api/v1/projects", None, String::new()).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let login = |name: &str| LoginRequest { user: name.into(), password: format!("{name}-password"), ..Default::default() };
    let (s, _) = call(&app, "POST", "/api/v1/login", None, serde_json::to_string(&LoginRequest { user: "anna".into(), password: "wrong".into(), ..Default::default() }).unwrap()).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let session = |text: String| serde_json::from_str::<LoginResponse>(&text).unwrap().token;
    let anna = session(call(&app, "POST", "/api/v1/login", None, serde_json::to_string(&login("anna")).unwrap()).await.1);
    let vic = session(call(&app, "POST", "/api/v1/login", None, serde_json::to_string(&login("vic")).unwrap()).await.1);
    let eve = session(call(&app, "POST", "/api/v1/login", None, serde_json::to_string(&login("eve")).unwrap()).await.1);

    // Anna creates the project; the user name is hers whatever the request says.
    let req = CreateRequest { name: "Plant 1".into(), user: "someone".into(), project: two_pages() };
    let (s, text) = with(&app, "POST", "/api/v1/projects", &anna, &req).await;
    assert_eq!(s, StatusCode::CREATED, "{text}");
    let opened: Opened = serde_json::from_str(&text).unwrap();
    assert_eq!(opened.session.user, "anna");

    // Vic may look, but not lock.
    let (s, text) = with(&app, "POST", "/api/v1/projects/Plant%201/open", &vic, &OpenRequest { user: "x".into() }).await;
    assert!(s.is_success(), "{text}");
    let vic_session: Opened = serde_json::from_str(&text).unwrap();
    let lock = LockRequest { session: vic_session.session.clone(), targets: vec![Target::Project] };
    let (s, _) = with(&app, "POST", "/api/v1/projects/Plant%201/lock", &vic, &lock).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    // Nobody uses the session of another user.
    let (s, _) = with(&app, "POST", "/api/v1/projects/Plant%201/status", &vic, &opened.session).await;
    assert_eq!(s, StatusCode::FORBIDDEN);

    // Eve only sees the office project.
    let (_, text) = with(&app, "GET", "/api/v1/projects", &eve, &()).await;
    assert_eq!(serde_json::from_str::<Vec<ProjectSummary>>(&text).unwrap().len(), 0);
    let (s, _) = with(&app, "GET", "/api/v1/projects/Plant%201", &eve, &()).await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // Only administrators manage users.
    let (s, _) = with(&app, "GET", "/api/v1/users", &anna, &()).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, text) = with(&app, "GET", "/api/v1/users", TOKEN, &()).await;
    assert!(s.is_success());
    assert_eq!(serde_json::from_str::<Vec<ProjectUser>>(&text).unwrap().len(), 3);

    // Signing out ends the session; deleting a user removes the account.
    let (s, _) = with(&app, "POST", "/api/v1/logout", &vic, &()).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    let (s, _) = with(&app, "GET", "/api/v1/projects", &vic, &()).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let (s, _) = with(&app, "DELETE", "/api/v1/users/eve", TOKEN, &()).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    let (s, _) = with(&app, "DELETE", "/api/v1/users/eve", TOKEN, &()).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}

/// Creates local users (`<name>-pw`) with the server token and signs them in.
async fn sign_in(app: &Router, users: &[(&str, Option<ProjectRole>, &[&str])]) -> Vec<String> {
    let mut tokens = Vec::new();
    for (name, role, groups) in users {
        let req = UserUpdate { password: Some(format!("{name}-password")), role: *role, projects: vec![], groups: Some(groups.iter().map(|g| g.to_string()).collect()) };
        let (s, text) = with(app, "PUT", &format!("/api/v1/users/{name}"), TOKEN, &req).await;
        assert!(s.is_success(), "{s} {text}");
        let login = LoginRequest { user: name.to_string(), password: format!("{name}-password"), ..Default::default() };
        let (s, text) = call(app, "POST", "/api/v1/login", None, serde_json::to_string(&login).unwrap()).await;
        assert!(s.is_success(), "{s} {text}");
        tokens.push(serde_json::from_str::<LoginResponse>(&text).unwrap().token);
    }
    tokens
}

#[tokio::test]
async fn rights_per_project() {
    let app = app(Some(TOKEN));
    let t = sign_in(&app, &[("olga", Some(ProjectRole::Editor), &[]), ("rita", None, &["qa"]), ("max", None, &["qa", "externals"])]).await;
    let (olga, rita, max) = (&t[0], &t[1], &t[2]);

    // Olga creates the project and manages it from then on.
    let req = CreateRequest { name: "Plant 1".into(), user: String::new(), project: two_pages() };
    let (s, text) = with(&app, "POST", "/api/v1/projects", olga, &req).await;
    assert_eq!(s, StatusCode::CREATED, "{text}");
    let opened: Opened = serde_json::from_str(&text).unwrap();
    assert!(opened.rights.contains(&ProjectRight::ManageAccess) && opened.rights.contains(&ProjectRight::Delete));

    // Users without role or grant see nothing.
    let (_, text) = with(&app, "GET", "/api/v1/projects", rita, &()).await;
    assert!(serde_json::from_str::<Vec<ProjectSummary>>(&text).unwrap().is_empty());

    // Olga lets the QA group review, but not the externals among them.
    let (s, text) = with(&app, "GET", "/api/v1/projects/Plant%201/access", olga, &()).await;
    assert!(s.is_success(), "{text}");
    let access: ProjectAccess = serde_json::from_str(&text).unwrap();
    assert!(access.roles.contains(&"reviewer".to_string()) && !access.roles.contains(&"admin".to_string()));
    assert_eq!(access.grants, [Grant::new(Subject::User("olga".into()), "manager", "Plant 1")]);
    let mut grants = access.grants.clone();
    grants.push(Grant::new(Subject::Group("qa".into()), "reviewer", "ignored"));
    grants.push(Grant { deny: true, ..Grant::new(Subject::Group("externals".into()), "viewer", "") });
    let (s, text) = with(&app, "PUT", "/api/v1/projects/Plant%201/access", olga, &grants).await;
    assert!(s.is_success(), "{text}");
    let stored = serde_json::from_str::<ProjectAccess>(&text).unwrap().grants;
    assert_eq!(stored.len(), 3);
    assert!(stored.iter().all(|g| g.scope == "Plant 1"));
    let (s, _) = with(&app, "PUT", "/api/v1/projects/Plant%201/access", olga, &vec![Grant::new(Subject::User("rita".into()), "admin", "")]).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "nobody gives rights they do not have");
    let (s, _) = with(&app, "PUT", "/api/v1/projects/Plant%201/access", rita, &grants).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "reviewers do not manage access");

    // Rita reviews: she saves comments, but locks nothing.
    let (_, text) = with(&app, "GET", "/api/v1/projects", rita, &()).await;
    let list: Vec<ProjectSummary> = serde_json::from_str(&text).unwrap();
    assert_eq!(list[0].rights, [ProjectRight::View, ProjectRight::Review]);
    let (_, text) = with(&app, "POST", "/api/v1/projects/Plant%201/open", rita, &OpenRequest::default()).await;
    let rita_session: Opened = serde_json::from_str(&text).unwrap();
    let lock = LockRequest { session: rita_session.session.clone(), targets: vec![Target::Page(opened.snapshot.project.pages[0].id)] };
    let (s, _) = with(&app, "POST", "/api/v1/projects/Plant%201/lock", rita, &lock).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let mut changed = rita_session.snapshot.project.clone();
    changed.info.name = "renamed".into();
    let save = SaveRequest { session: rita_session.session.clone(), project: changed, release: vec![], ..Default::default() };
    let (s, text) = with(&app, "PUT", "/api/v1/projects/Plant%201", rita, &save).await;
    assert!(s.is_success(), "{text}");
    assert_eq!(serde_json::from_str::<Snapshot>(&text).unwrap().project.info.name, opened.snapshot.project.info.name, "only reviews are taken");

    // Max is in QA, but an external: denied.
    let (s, _) = with(&app, "GET", "/api/v1/projects/Plant%201", max, &()).await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // Open projects are not deleted; deleting removes their grants.
    let (s, _) = with(&app, "DELETE", "/api/v1/projects/Plant%201", rita, &()).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, _) = with(&app, "DELETE", "/api/v1/projects/Plant%201", olga, &()).await;
    assert_eq!(s, StatusCode::CONFLICT);
    for (who, session) in [(olga, &opened.session), (rita, &rita_session.session)] {
        with(&app, "POST", "/api/v1/projects/Plant%201/close", who, session).await;
    }
    let (s, text) = with(&app, "DELETE", "/api/v1/projects/Plant%201", olga, &()).await;
    assert_eq!(s, StatusCode::NO_CONTENT, "{text}");
    let (_, text) = with(&app, "GET", "/api/v1/access", TOKEN, &()).await;
    assert!(serde_json::from_str::<Settings<ProjectRight>>(&text).unwrap().policy.grants.is_empty());
}

#[tokio::test]
async fn access_settings_and_anonymous_requests() {
    let app = app(Some(TOKEN));
    let t = sign_in(&app, &[("ann", Some(ProjectRole::Editor), &[]), ("root", Some(ProjectRole::Admin), &[])]).await;
    create_with(&app, TOKEN).await;
    let (s, _) = with(&app, "GET", "/api/v1/access", &t[0], &()).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let policy = Policy::<ProjectRight> {
        anonymous: "viewer".into(),
        roles: vec![openecraft_parts::access::Role { name: "Checker".into(), description: String::new(), rights: [ProjectRight::View, ProjectRight::Review].into() }],
        grants: vec![Grant::new(Subject::User("ann".into()), "checker", "Office*")],
    };
    let (s, text) = with(&app, "PUT", "/api/v1/access", &t[1], &policy).await;
    assert!(s.is_success(), "{text}");
    assert_eq!(serde_json::from_str::<Settings<ProjectRight>>(&text).unwrap().policy, policy);
    let invalid = Policy::<ProjectRight> { grants: vec![Grant::new(Subject::User("x".into()), "nobody", "")], ..Default::default() };
    let (s, _) = with(&app, "PUT", "/api/v1/access", &t[1], &invalid).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    // Everyone may look now, without signing in; wrong tokens are still refused.
    let (s, text) = call(&app, "GET", "/api/v1/projects", None, String::new()).await;
    assert!(s.is_success(), "{text}");
    assert_eq!(serde_json::from_str::<Vec<ProjectSummary>>(&text).unwrap()[0].rights, [ProjectRight::View]);
    let (s, _) = call(&app, "GET", "/api/v1/projects", Some("wrong"), String::new()).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let lock = LockRequest { session: Session { user: "x".into(), session: "1".into() }, targets: vec![Target::Project] };
    let (s, _) = call(&app, "POST", "/api/v1/projects/Plant%201/lock", None, serde_json::to_string(&lock).unwrap()).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
}

async fn create_with(app: &Router, token: &str) {
    let req = CreateRequest { name: "Plant 1".into(), user: "admin".into(), project: two_pages() };
    let (s, text) = with(app, "POST", "/api/v1/projects", token, &req).await;
    assert_eq!(s, StatusCode::CREATED, "{text}");
}

#[tokio::test]
async fn directory_sign_in() {
    let mut server = Server { store: Store::in_memory(), sessions: Sessions::default(), users: Users::default(), access: AccessFile::in_memory(Default::default()), throttle: Throttle::default(), views: Views::default() };
    let corp = openecraft_auth::Directory {
        id: "corp".into(),
        url: "ldap://127.0.0.1:1".into(),
        bind_dn: "CN=svc".into(),
        base_dn: "DC=corp".into(),
        timeout_secs: 1,
        ..Default::default()
    };
    server.access.config.directories.push(corp);
    server.users.set_admin("root", "root-password").unwrap();
    let app = router(AppState { server: Arc::new(Mutex::new(server)), token: None, web: false });
    // A directory alone makes the server private.
    let (s, _) = call(&app, "GET", "/api/v1/projects", None, String::new()).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    // Local accounts come first; unknown names go to the directory.
    let login = |user: &str, password: &str| serde_json::to_string(&LoginRequest { user: user.into(), password: password.into(), ..Default::default() }).unwrap();
    let (s, _) = call(&app, "POST", "/api/v1/login", None, login("root", "wrong")).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let (s, text) = call(&app, "POST", "/api/v1/login", None, login("root", "root-password")).await;
    assert!(s.is_success(), "{text}");
    assert!(serde_json::from_str::<LoginResponse>(&text).unwrap().access.allows(SERVER, ProjectRight::Administer));
    let (s, text) = call(&app, "POST", "/api/v1/login", None, login("anna", "pw")).await;
    assert_eq!(s, StatusCode::SERVICE_UNAVAILABLE, "{text}");
    let (_, text) = call(&app, "POST", "/api/v1/login", None, login("anna", "")).await;
    assert!(text.contains("wrong user name or password"), "{text}");
}

#[tokio::test]
async fn lock_errors_name_the_page_and_hide_other_sessions() {
    let app = app(None);
    let anna = create(&app, "anna").await;
    let ben = open(&app, "ben").await;
    let page = &anna.snapshot.project.pages[1];
    let path = "/api/v1/projects/Plant%201";
    let lock = |s: &Session| LockRequest { session: s.clone(), targets: vec![Target::Page(page.id)] };
    let _: Status = ok(&app, "POST", &format!("{path}/lock"), &lock(&anna.session)).await;
    let (status, text) = error(&app, "POST", &format!("{path}/lock"), &lock(&ben.session)).await;
    assert_eq!((status, text), (StatusCode::CONFLICT, format!("page {} is locked by anna", page.name)));
    // The session of a lock is the key to it: only its owner sees it.
    let seen: Status = ok(&app, "POST", &format!("{path}/status"), &ben.session).await;
    assert_eq!(seen.locks.len(), 1);
    assert_ne!(seen.locks[0].session, anna.session.session);
    let own: Status = ok(&app, "POST", &format!("{path}/status"), &anna.session).await;
    assert_eq!(own.locks[0].session, anna.session.session);
    let snapshot: Snapshot = ok(&app, "GET", path, &()).await;
    assert_ne!(snapshot.locks[0].session, anna.session.session);
}

#[tokio::test]
async fn user_names_passwords_and_the_last_administrator() {
    let app = app(None);
    let admin = UserUpdate { password: Some("root-password".into()), role: Some(ProjectRole::Admin), projects: vec![], groups: None };
    let _: ProjectUser = ok(&app, "PUT", "/api/v1/users/root", &admin).await;
    let login = LoginRequest { user: "root".into(), password: "root-password".into(), ..Default::default() };
    let (_, text) = call(&app, "POST", "/api/v1/login", None, serde_json::to_string(&login).unwrap()).await;
    let root = serde_json::from_str::<LoginResponse>(&text).unwrap().token;
    let user = |password: &str| UserUpdate { password: Some(password.into()), role: Some(ProjectRole::Editor), projects: vec![], groups: None };
    assert_eq!(with(&app, "PUT", "/api/v1/users/a%20b", &root, &user("long-password")).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(with(&app, "PUT", "/api/v1/users/weak", &root, &user("1")).await.0, StatusCode::BAD_REQUEST);
    // Without server token someone must stay able to manage the users.
    assert_eq!(with(&app, "DELETE", "/api/v1/users/root", &root, &()).await.0, StatusCode::CONFLICT);
    assert_eq!(with(&app, "PUT", "/api/v1/users/root", &root, &user("")).await.0, StatusCode::CONFLICT);
    // Signing out without a session is no error.
    assert_eq!(call(&app, "POST", "/api/v1/logout", None, String::new()).await.0, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn failed_sign_ins_are_throttled() {
    let app = app(None);
    let admin = UserUpdate { password: Some("root-password".into()), role: Some(ProjectRole::Admin), projects: vec![], groups: None };
    let _: ProjectUser = ok(&app, "PUT", "/api/v1/users/root", &admin).await;
    let login = |password: &str| serde_json::to_string(&LoginRequest { user: "root".into(), password: password.into(), ..Default::default() }).unwrap();
    for _ in 0..openecraft_auth::throttle::FREE_ATTEMPTS {
        assert_eq!(call(&app, "POST", "/api/v1/login", None, login("guess")).await.0, StatusCode::UNAUTHORIZED);
    }
    assert_eq!(call(&app, "POST", "/api/v1/login", None, login("root-password")).await.0, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn saves_with_a_base_revision_lock_free_changes_and_report_the_rest() {
    let app = app(None);
    let anna = create(&app, "anna").await;
    let ben = open(&app, "ben").await;
    let (p1, p2) = (anna.snapshot.project.pages[0].id, anna.snapshot.project.pages[1].id);
    let path = "/api/v1/projects/Plant%201";
    // Ben holds page 2; anna changes both pages without any lock.
    let _: Status = ok(&app, "POST", &format!("{path}/lock"), &LockRequest { session: ben.session.clone(), targets: vec![Target::Page(p2)] }).await;
    let mut a = anna.snapshot.project.clone();
    a.pages[0].description = "anna 1".into();
    a.pages[1].description = "anna 2".into();
    let base = anna.snapshot.revision;
    let req = SaveRequest { session: anna.session.clone(), project: a.clone(), base_revision: base, ..Default::default() };
    let saved: Snapshot = ok(&app, "PUT", path, &req).await;
    assert_eq!(saved.project.pages[0].description, "anna 1", "the free page was locked and saved");
    assert_eq!(saved.project.pages[1].description, anna.snapshot.project.pages[1].description);
    assert_eq!(saved.ignored, vec![Target::Page(p2)]);
    assert!(saved.locks.iter().any(|l| l.target == Target::Page(p1) && l.session == anna.session.session));

    // Strict: nothing is saved when a change would be left out.
    a.pages[0].description = "anna 1b".into();
    let strict = SaveRequest { session: anna.session.clone(), project: a, base_revision: saved.revision, strict: true, ..Default::default() };
    let (code, msg) = error(&app, "PUT", path, &strict).await;
    assert_eq!(code, StatusCode::CONFLICT);
    assert!(msg.contains(&format!("page {}", saved.project.pages[1].name)), "{msg}");
    let snap: Snapshot = ok(&app, "GET", path, &()).await;
    assert_eq!(snap.project.pages[0].description, "anna 1");
}

#[tokio::test]
async fn changes_made_by_others_since_the_base_are_not_overwritten() {
    let app = app(None);
    let anna = create(&app, "anna").await;
    let ben = open(&app, "ben").await;
    let path = "/api/v1/projects/Plant%201";
    // Ben changes page 1 (locking it on the way) and releases it again.
    let mut b = ben.snapshot.project.clone();
    b.pages[0].description = "ben".into();
    let req = SaveRequest { session: ben.session.clone(), project: b, release: vec![Target::Page(ben.snapshot.project.pages[0].id)], base_revision: 1, ..Default::default() };
    let saved: Snapshot = ok(&app, "PUT", path, &req).await;
    assert_eq!(saved.project.pages[0].description, "ben");
    // Anna still edits revision 1: her change of the same page would overwrite ben's.
    let mut a = anna.snapshot.project.clone();
    a.pages[0].description = "anna".into();
    let req = SaveRequest { session: anna.session.clone(), project: a, base_revision: 1, ..Default::default() };
    let saved: Snapshot = ok(&app, "PUT", path, &req).await;
    assert_eq!(saved.project.pages[0].description, "ben");
    assert_eq!(saved.ignored, vec![Target::Page(anna.snapshot.project.pages[0].id)]);
}

#[tokio::test]
async fn requests_are_checked_and_names_found_in_any_case() {
    let app = app(None);
    // User names with control characters are refused.
    let req = CreateRequest { name: "Plant 1".into(), user: "a\nb".into(), project: two_pages() };
    assert_eq!(error(&app, "POST", "/api/v1/projects", &req).await.0, StatusCode::BAD_REQUEST);
    // A project needs its content.
    let (status, text) = error(&app, "POST", "/api/v1/projects", &serde_json::json!({ "name": "Empty", "user": "anna" })).await;
    assert_eq!((status, text.as_str()), (StatusCode::BAD_REQUEST, "the request needs the project"));
    create(&app, "anna").await;
    // The project is found whatever the case of its name.
    let snapshot: Snapshot = ok(&app, "GET", "/api/v1/projects/plant%201", &()).await;
    assert_eq!(snapshot.project.pages.len(), 2);
    let ben: Opened = ok(&app, "POST", "/api/v1/projects/PLANT%201/open", &OpenRequest { user: "ben".into() }).await;
    let list: Vec<ProjectSummary> = ok(&app, "GET", "/api/v1/projects", &()).await;
    assert_eq!(list[0].users, vec!["anna".to_string(), "ben".to_string()]);
    let _: Status = ok(&app, "POST", "/api/v1/projects/plant%201/close", &ben.session).await;
}

#[tokio::test]
async fn signing_out_or_deleting_a_user_closes_their_projects() {
    let app = app(Some(TOKEN));
    let t = sign_in(&app, &[("olga", Some(ProjectRole::Editor), &[]), ("vic", Some(ProjectRole::Editor), &[])]).await;
    // A new user without role, groups or grants could do nothing.
    let nobody = UserUpdate { password: Some("nobody-password".into()), ..Default::default() };
    let (s, _) = with(&app, "PUT", "/api/v1/users/nobody", TOKEN, &nobody).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    // The server's name and version need no token.
    let (s, text) = call(&app, "GET", "/api/v1/info", None, String::new()).await;
    assert_eq!(s, StatusCode::OK, "{text}");
    // Both servers read the user name as `user` and as `name`.
    let (s, text) = call(&app, "POST", "/api/v1/login", None, r#"{"name": "olga", "password": "olga-password"}"#.into()).await;
    assert_eq!(s, StatusCode::OK, "{text}");

    let req = CreateRequest { name: "Plant 1".into(), user: String::new(), project: two_pages() };
    let (s, text) = with(&app, "POST", "/api/v1/projects", &t[0], &req).await;
    assert_eq!(s, StatusCode::CREATED, "{text}");
    let olga: Opened = serde_json::from_str(&text).unwrap();
    let lock = LockRequest { session: olga.session.clone(), targets: vec![Target::Project] };
    assert_eq!(with(&app, "POST", "/api/v1/projects/Plant%201/lock", &t[0], &lock).await.0, StatusCode::OK);
    // Signing out releases the locks of the projects opened with that sign-in.
    assert_eq!(with(&app, "POST", "/api/v1/logout", &t[0], &()).await.0, StatusCode::NO_CONTENT);
    let snapshot: Snapshot = serde_json::from_str(&with(&app, "GET", "/api/v1/projects/Plant%201", TOKEN, &()).await.1).unwrap();
    assert!(snapshot.locks.is_empty());

    let (_, text) = with(&app, "POST", "/api/v1/projects/Plant%201/open", &t[1], &OpenRequest::default()).await;
    let vic: Opened = serde_json::from_str(&text).unwrap();
    let lock = LockRequest { session: vic.session.clone(), targets: vec![Target::Project] };
    assert_eq!(with(&app, "POST", "/api/v1/projects/Plant%201/lock", &t[1], &lock).await.0, StatusCode::OK);
    // A deleted user keeps nothing locked.
    assert_eq!(with(&app, "DELETE", "/api/v1/users/vic", TOKEN, &()).await.0, StatusCode::NO_CONTENT);
    let list: Vec<ProjectSummary> = serde_json::from_str(&with(&app, "GET", "/api/v1/projects", TOKEN, &()).await.1).unwrap();
    assert!(list[0].users.is_empty());
}

/// A request from the client at `ip`.
async fn from(app: &Router, ip: [u8; 4], method: &str, uri: &str, body: &impl Serialize) -> StatusCode {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .extension(axum::extract::ConnectInfo(std::net::SocketAddr::from((ip, 50000))))
        .body(Body::from(serde_json::to_string(body).unwrap()))
        .unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

#[tokio::test]
async fn open_servers_are_not_taken_over_from_outside() {
    let app = app(None);
    let admin = UserUpdate { password: Some("intruder-password".into()), role: Some(ProjectRole::Admin), projects: vec![], groups: None };
    assert_eq!(from(&app, [203, 0, 113, 9], "PUT", "/api/v1/users/intruder", &admin).await, StatusCode::FORBIDDEN);
    assert_eq!(from(&app, [203, 0, 113, 9], "PUT", "/api/v1/access", &Policy::<ProjectRight>::default()).await, StatusCode::FORBIDDEN);
    // The machine of the server sets up the first administrator.
    assert_eq!(from(&app, [127, 0, 0, 1], "PUT", "/api/v1/users/root", &admin).await, StatusCode::OK);
}

#[tokio::test]
async fn approvals_are_given_in_the_own_name() {
    let app = app(Some(TOKEN));
    let t = sign_in(&app, &[("rev", Some(ProjectRole::Reviewer), &[])]).await;
    let req = CreateRequest { name: "Plant 1".into(), user: "admin".into(), project: two_pages() };
    let (s, _) = with(&app, "POST", "/api/v1/projects", TOKEN, &req).await;
    assert_eq!(s, StatusCode::CREATED);
    let (_, text) = with(&app, "POST", "/api/v1/projects/Plant%201/open", &t[0], &OpenRequest { user: "rev".into() }).await;
    let opened: Opened = serde_json::from_str(&text).unwrap();
    let mut project = opened.snapshot.project.clone();
    project.review.approvals.push(openecraft_core::review::Approval { id: 999, rev: 1, revision: "A".into(), user: "Chief Engineer".into(), approved: true, ..Default::default() });
    let save = SaveRequest { session: opened.session.clone(), project, release: vec![], base_revision: opened.snapshot.revision, strict: false };
    let (s, text) = with(&app, "PUT", "/api/v1/projects/Plant%201", &t[0], &save).await;
    assert!(s.is_success(), "{text}");
    let snapshot: Snapshot = serde_json::from_str(&text).unwrap();
    assert_eq!(snapshot.project.review.approvals[0].user, "rev");
}

#[tokio::test]
async fn managers_cannot_lift_denials() {
    let app = app(Some(TOKEN));
    let t = sign_in(&app, &[("olga", Some(ProjectRole::Editor), &[])]).await;
    let req = CreateRequest { name: "Plant 1".into(), user: String::new(), project: two_pages() };
    assert_eq!(with(&app, "POST", "/api/v1/projects", &t[0], &req).await.0, StatusCode::CREATED);
    let owner = Grant::new(Subject::User("olga".into()), "manager", "Plant 1");
    let denial = Grant { deny: true, ..Grant::new(Subject::Group("externals".into()), "viewer", "Plant 1") };
    let (s, _) = with(&app, "PUT", "/api/v1/projects/Plant%201/access", TOKEN, &vec![owner.clone(), denial.clone()]).await;
    assert!(s.is_success());
    let (s, text) = with(&app, "PUT", "/api/v1/projects/Plant%201/access", &t[0], &vec![owner.clone()]).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "{text}");
    // Denying more stays possible.
    let more = Grant { deny: true, ..Grant::new(Subject::Group("guests".into()), "viewer", "Plant 1") };
    let (s, text) = with(&app, "PUT", "/api/v1/projects/Plant%201/access", &t[0], &vec![owner, denial, more]).await;
    assert!(s.is_success(), "{text}");
}

#[tokio::test]
async fn bodies_of_unknown_callers_are_not_read() {
    let app = app(Some(TOKEN));
    // Refused for the missing token before the body is parsed (else 422 or 400).
    assert_eq!(call(&app, "POST", "/api/v1/projects", None, "{not json".into()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(call(&app, "PUT", "/api/v1/projects/x", Some("wrong"), "{not json".into()).await.0, StatusCode::UNAUTHORIZED);
    let big = format!(r#"{{"user": "x", "password": "{}"}}"#, "x".repeat(SMALL_BODY));
    assert_eq!(call(&app, "POST", "/api/v1/login", None, big).await.0, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn anonymous_callers_cannot_take_names_of_accounts() {
    let app = app(Some(TOKEN));
    sign_in(&app, &[("olga", Some(ProjectRole::Editor), &[])]).await;
    let policy = Policy::<ProjectRight> { anonymous: "viewer".into(), ..Default::default() };
    assert!(with(&app, "PUT", "/api/v1/access", TOKEN, &policy).await.0.is_success());
    let req = CreateRequest { name: "Plant 1".into(), user: "admin".into(), project: two_pages() };
    assert_eq!(with(&app, "POST", "/api/v1/projects", TOKEN, &req).await.0, StatusCode::CREATED);
    assert_eq!(error(&app, "POST", "/api/v1/projects/Plant%201/open", &OpenRequest { user: "Olga".into() }).await.0, StatusCode::FORBIDDEN);
    let opened: Opened = ok(&app, "POST", "/api/v1/projects/Plant%201/open", &OpenRequest { user: "visitor".into() }).await;
    // The key to a session cannot be guessed.
    assert!(opened.session.session.len() > 64, "{}", opened.session.session);
}

#[tokio::test]
async fn the_last_administrator_stays_also_by_grant_and_access_settings() {
    let app = app(None);
    let login = |name: &str| serde_json::to_string(&LoginRequest { user: name.into(), password: format!("{name}-password"), ..Default::default() }).unwrap();
    let account = |role, password: &str| UserUpdate { password: Some(password.into()), role: Some(role), projects: vec![], groups: None };
    let _: ProjectUser = ok(&app, "PUT", "/api/v1/users/root", &account(ProjectRole::Admin, "root-password")).await;
    let root = serde_json::from_str::<LoginResponse>(&call(&app, "POST", "/api/v1/login", None, login("root")).await.1).unwrap().token;
    // Access settings that take the administration from everybody are refused.
    let deny = Policy::<ProjectRight> { grants: vec![Grant { deny: true, ..Grant::new(Subject::User("root".into()), "admin", "") }], ..Default::default() };
    assert_eq!(with(&app, "PUT", "/api/v1/access", &root, &deny).await.0, StatusCode::CONFLICT);
    // Anna administers by a grant; root steps down.
    assert!(with(&app, "PUT", "/api/v1/users/anna", &root, &account(ProjectRole::Editor, "anna-password")).await.0.is_success());
    let anna = serde_json::from_str::<LoginResponse>(&call(&app, "POST", "/api/v1/login", None, login("anna")).await.1).unwrap().token;
    let grant = Policy::<ProjectRight> { grants: vec![Grant::new(Subject::User("anna".into()), "admin", "")], ..Default::default() };
    assert!(with(&app, "PUT", "/api/v1/access", &root, &grant).await.0.is_success());
    let editor = UserUpdate { password: None, ..account(ProjectRole::Editor, "") };
    assert!(with(&app, "PUT", "/api/v1/users/root", &root, &editor).await.0.is_success());
    // The last administrator by grant can neither go nor lose the grant.
    assert_eq!(with(&app, "DELETE", "/api/v1/users/anna", &anna, &()).await.0, StatusCode::CONFLICT);
    assert_eq!(with(&app, "PUT", "/api/v1/access", &anna, &Policy::<ProjectRight>::default()).await.0, StatusCode::CONFLICT);
    assert!(with(&app, "GET", "/api/v1/users", &anna, &()).await.0.is_success());
}

#[tokio::test]
async fn current_account_has_fresh_identity_and_rights_without_credentials() {
    let app = app(Some(TOKEN));
    let tokens = sign_in(&app, &[("Anna", Some(ProjectRole::Editor), &["qa"])]).await;
    let token = &tokens[0];
    let (code, body) = with(&app, "GET", "/api/v1/session", token, &()).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    let current: CurrentAccount = serde_json::from_str(&body).unwrap();
    assert_eq!(current.user.name, "Anna");
    assert_eq!(current.user.role, Some(ProjectRole::Editor));
    assert_eq!(current.user.groups, ["qa"]);
    assert!(current.access.allows("Plant 1", ProjectRight::EditPages));
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json.get("token").is_none());
    assert!(!body.contains(token));
    assert!(!body.contains("password"));

    let update = UserUpdate { role: Some(ProjectRole::Viewer), projects: vec!["Plant 1".into()], ..Default::default() };
    assert_eq!(with(&app, "PUT", "/api/v1/users/anna", TOKEN, &update).await.0, StatusCode::OK);
    let (code, body) = with(&app, "GET", "/api/v1/session", token, &()).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    let current: CurrentAccount = serde_json::from_str(&body).unwrap();
    assert_eq!(current.user.name, "Anna");
    assert_eq!(current.user.role, Some(ProjectRole::Viewer));
    assert_eq!(current.user.projects, ["Plant 1"]);
    assert!(current.access.allows("Plant 1", ProjectRight::View));
    assert!(!current.access.allows("Plant 1", ProjectRight::EditPages));
    assert!(!current.access.allows("Other", ProjectRight::View));

    assert_eq!(with(&app, "POST", "/api/v1/logout", token, &()).await.0, StatusCode::NO_CONTENT);
    assert_eq!(with(&app, "GET", "/api/v1/session", token, &()).await.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn current_account_refuses_root_open_anonymous_and_invalid_tokens() {
    let open = app(None);
    assert_eq!(call(&open, "GET", "/api/v1/projects", None, String::new()).await.0, StatusCode::OK);
    assert_eq!(call(&open, "GET", "/api/v1/session", None, String::new()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(with(&open, "GET", "/api/v1/session", "wrong", &()).await.0, StatusCode::UNAUTHORIZED);

    let app = app(Some(TOKEN));
    for token in [None, Some(TOKEN), Some("wrong")] {
        assert_eq!(call(&app, "GET", "/api/v1/session", token, String::new()).await.0, StatusCode::UNAUTHORIZED);
    }
    let policy = Policy::<ProjectRight> { anonymous: "viewer".into(), ..Default::default() };
    assert_eq!(with(&app, "PUT", "/api/v1/access", TOKEN, &policy).await.0, StatusCode::OK);
    assert_eq!(call(&app, "GET", "/api/v1/projects", None, String::new()).await.0, StatusCode::OK);
    for token in [None, Some(TOKEN), Some("wrong")] {
        assert_eq!(call(&app, "GET", "/api/v1/session", token, String::new()).await.0, StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn current_account_does_not_renew_tokens_and_rejects_expired_sessions() {
    let now = openecraft_auth::unix_now();
    let mut sessions = Vec::new();
    let token = openecraft_auth::start_session(&mut sessions, now - 60);
    let valid = Account {
        user: ProjectUser { name: "DirectoryUser".into(), provider: "corp".into(), groups: vec!["corp:qa".into()], ..Default::default() },
        password: String::new(), sessions,
    };
    let mut sessions = Vec::new();
    let expired_token = openecraft_auth::start_session(&mut sessions, now - openecraft_auth::SESSION_LIFETIME_SECS);
    let expired = Account { user: ProjectUser { name: "expired".into(), ..Default::default() }, password: String::new(), sessions };
    let accounts = vec![valid, expired];
    let mut users = Users::default();
    users.accounts = accounts.clone();
    let server = Server { store: Store::in_memory(), sessions: Sessions::default(), users, access: AccessFile::in_memory(Default::default()), throttle: Throttle::default(), views: Views::default() };
    let state = AppState { server: Arc::new(Mutex::new(server)), token: Some(TOKEN.into()), web: false };
    let app = router(state.clone());
    for _ in 0..2 {
        let request = Request::builder().uri("/api/v1/session").header(header::AUTHORIZATION, format!("Bearer {token}")).body(Body::empty()).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get(header::CACHE_CONTROL).unwrap(), "no-store");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let current: CurrentAccount = serde_json::from_slice(&body).unwrap();
        assert_eq!(current.user, accounts[0].user);
    }
    assert_eq!(with(&app, "GET", "/api/v1/session", &expired_token, &()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(lock_server(&state).users.accounts, accounts);
}

async fn get(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, HeaderMap, String) {
    let mut req = Request::builder().uri(uri);
    if let Some(t) = token {
        req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let resp = app.clone().oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
    let (status, headers) = (resp.status(), resp.headers().clone());
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, headers, String::from_utf8_lossy(&bytes).to_string())
}

#[tokio::test]
async fn the_web_page_is_served_only_with_the_option() {
    let off = app(Some(TOKEN));
    let (status, _, text) = get(&off, "/", None).await;
    assert!(status.is_success() && text.contains("--web"), "{text}");
    for uri in ["/project.js", "/common.js", "/api/v1/texts", "/api/v1/projects/Plant/view"] {
        assert_eq!(get(&off, uri, Some(TOKEN)).await.0, StatusCode::NOT_FOUND, "{uri}");
    }
    let on = app_with(Some(TOKEN), true);
    let (status, headers, page) = get(&on, "/", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[header::CONTENT_SECURITY_POLICY], openecraft_web::PAGE_CSP);
    assert!(page.contains("<script src=\"/project.js\"></script>"));
    for (uri, kind) in [("/project.js", "text/javascript"), ("/admin.js", "text/javascript"), ("/common.js", "text/javascript"), ("/access.js", "text/javascript"), ("/web.css", "text/css"), ("/logo.svg", "image/svg+xml")] {
        let (status, headers, _) = get(&on, uri, None).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert!(headers[header::CONTENT_TYPE].to_str().unwrap().starts_with(kind), "{uri}");
    }
    let (_, _, texts) = get(&on, "/api/v1/texts?lang=de", None).await;
    let texts: openecraft_web::WebTexts = serde_json::from_str(&texts).unwrap();
    assert_eq!(texts.lang, "de");
    let info: ServerInfo = serde_json::from_str(&get(&on, "/api/v1/info", None).await.2).unwrap();
    assert!(info.web);
    assert!(!serde_json::from_str::<ServerInfo>(&get(&off, "/api/v1/info", None).await.2).unwrap().web);
}

#[tokio::test]
async fn info_tells_who_calls_and_what_they_may_do() {
    let app = app_with(Some(TOKEN), true);
    let tokens = sign_in(&app, &[("vic", Some(ProjectRole::Viewer), &[])]).await;
    let info: ServerInfo = serde_json::from_str(&get(&app, "/api/v1/info", Some(TOKEN)).await.2).unwrap();
    let admin = info.user.expect("the token signs in as administrator");
    assert_eq!(admin.role, Some(ProjectRole::Admin));
    assert!(info.access.allows(SERVER, ProjectRight::Administer));
    let info: ServerInfo = serde_json::from_str(&get(&app, "/api/v1/info", Some(&tokens[0])).await.2).unwrap();
    assert_eq!(info.user.map(|u| u.name).as_deref(), Some("vic"));
    assert!(info.access.allows("Any", ProjectRight::View) && !info.access.allows("Any", ProjectRight::EditPages));
    let info: ServerInfo = serde_json::from_str(&get(&app, "/api/v1/info", Some("wrong")).await.2).unwrap();
    assert!(info.user.is_none() && info.access.is_empty());
}

#[tokio::test]
async fn the_view_shows_projects_read_only_to_who_may_see_them() {
    let app = app_with(Some(TOKEN), true);
    let tokens = sign_in(&app, &[("vic", Some(ProjectRole::Viewer), &[]), ("nobody", None, &["Somewhere"])]).await;
    let demo = openecraft_core::sample::demo_project();
    let req = CreateRequest { name: "Demo".into(), user: "admin".into(), project: demo.clone() };
    let (s, text) = with(&app, "POST", "/api/v1/projects", TOKEN, &req).await;
    assert!(s.is_success(), "{text}");
    let opened: Opened = serde_json::from_str(&text).unwrap();
    let page = opened.snapshot.project.pages[1].id;
    let lock = LockRequest { session: opened.session.clone(), targets: vec![Target::Page(page)] };
    assert!(with(&app, "POST", "/api/v1/projects/Demo/lock", TOKEN, &lock).await.0.is_success());

    let (status, _, text) = get(&app, "/api/v1/projects/demo/view?lang=de", Some(&tokens[0])).await;
    assert_eq!(status, StatusCode::OK, "{text}");
    let view: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(view["name"], "Demo", "found in any case");
    assert_eq!(view["title"], demo.info.name.as_str());
    assert_eq!(view["pages"].as_array().unwrap().len(), demo.pages.len());
    assert_eq!(view["rights"], serde_json::json!(["view"]));
    assert_eq!(view["users"], serde_json::json!(["admin"]));
    assert_eq!(view["locks"][0]["target"]["page"], page);
    assert!(!text.contains(&opened.session.session), "the key of the session stays hidden");

    let (status, headers, svg) = get(&app, &format!("/api/v1/projects/Demo/pages/{page}/svg"), Some(&tokens[0])).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[header::CONTENT_TYPE], "image/svg+xml");
    assert!(svg.starts_with("<svg"));
    assert_eq!(get(&app, "/api/v1/projects/Demo/pages/999999/svg", Some(&tokens[0])).await.0, StatusCode::NOT_FOUND);
    // Projects the caller may not see do not exist for them; without sign-in nothing.
    assert_eq!(get(&app, "/api/v1/projects/Demo/view", Some(&tokens[1])).await.0, StatusCode::NOT_FOUND);
    assert_eq!(get(&app, &format!("/api/v1/projects/Demo/pages/{page}/svg"), Some(&tokens[1])).await.0, StatusCode::NOT_FOUND);
    assert_eq!(get(&app, "/api/v1/projects/Demo/view", None).await.0, StatusCode::UNAUTHORIZED);

    // A save is a new revision: the view follows it.
    let mut changed = opened.snapshot.project.clone();
    changed.pages[1].description = "Changed on the server".into();
    let save = SaveRequest { session: opened.session.clone(), project: changed, ..Default::default() };
    assert!(with(&app, "PUT", "/api/v1/projects/Demo", TOKEN, &save).await.0.is_success());
    let view: serde_json::Value = serde_json::from_str(&get(&app, "/api/v1/projects/Demo/view", Some(&tokens[0])).await.2).unwrap();
    assert_eq!(view["server_revision"], opened.snapshot.revision + 1);
    assert_eq!(view["pages"][1]["description"], "Changed on the server");
}

#[tokio::test]
async fn directories_are_managed_by_administrators() {
    let app = app(Some(TOKEN));
    let tokens = sign_in(&app, &[("vic", Some(ProjectRole::Viewer), &[])]).await;
    let corp = Directory { id: "corp".into(), url: "ldaps://dc1.corp.example".into(), base_dn: "DC=corp".into(), bind_dn: "CN=svc".into(), bind_password: "secret".into(), ..Default::default() };
    assert_eq!(with(&app, "PUT", "/api/v1/directories/corp", &tokens[0], &corp).await.0, StatusCode::FORBIDDEN);
    let (s, text) = with(&app, "PUT", "/api/v1/directories/corp", TOKEN, &corp).await;
    assert!(s.is_success(), "{text}");
    let (_, _, list) = get(&app, "/api/v1/directories", Some(TOKEN)).await;
    let list: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
    assert_eq!((list[0]["id"].as_str(), list[0]["password_set"].as_bool()), (Some("corp"), Some(true)));
    assert!(list[0]["bind_password"].as_str().unwrap_or_default().is_empty(), "the password stays on the server");
    // An empty password keeps the stored one.
    let renamed = Directory { title: "Corporate".into(), bind_password: String::new(), ..corp.clone() };
    assert!(with(&app, "PUT", "/api/v1/directories/CORP", TOKEN, &renamed).await.0.is_success());
    let (_, _, settings) = get(&app, "/api/v1/access", Some(TOKEN)).await;
    assert!(settings.contains("Corporate"));
    assert_eq!(get(&app, "/api/v1/directories", Some(&tokens[0])).await.0, StatusCode::FORBIDDEN);
    assert_eq!(with(&app, "DELETE", "/api/v1/directories/corp", TOKEN, &()).await.0, StatusCode::NO_CONTENT);
    assert_eq!(with(&app, "DELETE", "/api/v1/directories/corp", TOKEN, &()).await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn administrators_see_the_open_sessions_and_the_ended_ones() {
    let app = app(Some(TOKEN));
    let tokens = sign_in(&app, &[("anna", Some(ProjectRole::Editor), &[]), ("vic", Some(ProjectRole::Viewer), &[])]).await;
    let (_, text) = with(&app, "POST", "/api/v1/projects", &tokens[0], &CreateRequest { name: "Plant 1".into(), user: "anna".into(), project: two_pages() }).await;
    let anna: Opened = serde_json::from_str(&text).unwrap();
    let page = anna.snapshot.project.pages[0].id;
    let lock = LockRequest { session: anna.session.clone(), targets: vec![Target::Page(page), Target::Project] };
    assert!(with(&app, "POST", "/api/v1/projects/Plant%201/lock", &tokens[0], &lock).await.0.is_success());
    let mut changed = anna.snapshot.project.clone();
    changed.pages[0].description = "Changed".into();
    let save = SaveRequest { session: anna.session.clone(), project: changed, ..Default::default() };
    assert!(with(&app, "PUT", "/api/v1/projects/Plant%201", &tokens[0], &save).await.0.is_success());

    assert_eq!(get(&app, "/api/v1/sessions", Some(&tokens[1])).await.0, StatusCode::FORBIDDEN, "administrators only");
    let (status, _, text) = get(&app, "/api/v1/sessions", Some(TOKEN)).await;
    assert_eq!(status, StatusCode::OK, "{text}");
    assert!(!text.contains(&anna.session.session), "the key of the session stays hidden");
    let list: SessionList = serde_json::from_str(&text).unwrap();
    let open = &list.active[0];
    assert_eq!((open.user.as_str(), open.project.as_str(), open.saves), ("anna", "Plant 1", 1));
    assert_eq!(open.pages, [anna.snapshot.project.pages[0].name.clone()]);
    assert!(open.locks.contains(&Target::Project));
    assert!(list.history.is_empty());

    assert!(with(&app, "POST", "/api/v1/projects/Plant%201/close", &tokens[0], &anna.session).await.0.is_success());
    let list: SessionList = serde_json::from_str(&get(&app, "/api/v1/sessions?limit=5", Some(TOKEN)).await.2).unwrap();
    assert!(list.active.is_empty());
    assert_eq!(list.history.len(), 1);
    let ended = &list.history[0];
    assert_eq!((ended.user.as_str(), ended.end, ended.saves), ("anna", Some(openecraft_core::collab::SessionEnd::Closed), 1));
    assert!(ended.locks.is_empty() && ended.ended.is_some());
    assert_eq!(list.kept, crate::history::KEPT);
}
