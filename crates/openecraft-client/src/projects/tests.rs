use super::*;

fn endpoint() -> ProjectEndpoint {
    ProjectEndpoint { url: "http://host:8091/".into(), token: " secret ".into() }
}

fn session() -> Session {
    Session { user: "anna".into(), session: "1-ab".into() }
}

#[test]
fn requests_of_the_api() {
    let e = endpoint();
    assert_eq!(e.info().url, "http://host:8091/api/v1/info");
    assert_eq!(e.list().url, "http://host:8091/api/v1/projects");
    assert_eq!(e.snapshot("Plant 1").url, "http://host:8091/api/v1/projects/Plant%201");
    let snapshot = e.session_snapshot("Plant 1", &session());
    assert_eq!(snapshot.url, "http://host:8091/api/v1/projects/Plant%201/snapshot");
    assert_eq!(snapshot.method, ehttp::Method::POST);
    assert_eq!(serde_json::from_slice::<Session>(&snapshot.body).unwrap(), session());
    assert_eq!(snapshot.headers.get("Authorization"), Some("Bearer secret"));
    assert_eq!(e.open("Plant 1", "anna").url, "http://host:8091/api/v1/projects/Plant%201/open");
    assert_eq!(e.status("p", &session()).url, "http://host:8091/api/v1/projects/p/status");
    assert_eq!(e.release("p", &session(), Vec::new()).url, "http://host:8091/api/v1/projects/p/release");
    assert_eq!(e.close("p", &session()).url, "http://host:8091/api/v1/projects/p/close");
    assert_eq!(e.delete("Plant 1").method, ehttp::Method::DELETE);
    assert_eq!(e.project_access("Plant 1").url, "http://host:8091/api/v1/projects/Plant%201/access");
    let grants = [Grant::new(openecraft_core::collab::Subject::Group("corp:QA".into()), "reviewer", "Plant 1")];
    let put = e.save_project_access("Plant 1", &grants);
    assert_eq!((put.method, serde_json::from_slice::<Vec<Grant>>(&put.body).unwrap()), (ehttp::Method::PUT, grants.to_vec()));
    assert_eq!(e.access().url, "http://host:8091/api/v1/access");
    assert_eq!(e.save_access(&Policy::default()).method, ehttp::Method::PUT);
    // Every request carries the token.
    assert_eq!(e.list().headers.get("Authorization"), Some("Bearer secret"));
    let anonymous = ProjectEndpoint { token: " ".into(), ..endpoint() };
    assert_eq!(anonymous.list().headers.get("Authorization"), None);
}

#[test]
fn bodies_are_json() {
    let e = endpoint();
    let lock = e.lock("p", &session(), vec![Target::Page(4)]);
    assert_eq!(lock.method, ehttp::Method::POST);
    assert_eq!(lock.url, "http://host:8091/api/v1/projects/p/lock");
    assert_eq!(lock.headers.get_all("Content-Type").collect::<Vec<_>>(), ["application/json"]);
    let body: LockRequest = serde_json::from_slice(&lock.body).unwrap();
    assert_eq!(body.targets, vec![Target::Page(4)]);
    assert_eq!(body.session, session());

    let prj = Project::default();
    let save = e.save("p", &session(), &prj, vec![Target::Project], 7);
    assert_eq!(save.method, ehttp::Method::PUT);
    let body: SaveRequest = serde_json::from_slice(&save.body).unwrap();
    assert_eq!((body.release, body.base_revision), (vec![Target::Project], 7));
    assert_eq!(body.project, prj);

    let create = e.create("p", "anna", &prj);
    assert_eq!(create.url, "http://host:8091/api/v1/projects");
    let body: CreateRequest = serde_json::from_slice(&create.body).unwrap();
    assert_eq!((body.name.as_str(), body.user.as_str()), ("p", "anna"));
}

#[test]
fn account_requests() {
    let e = ProjectEndpoint { url: "http://server:8091/".into(), token: "t".into() };
    let login = e.login("anna", "pw");
    assert_eq!((login.method, login.url.as_str()), (ehttp::Method::POST, "http://server:8091/api/v1/login"));
    assert!(String::from_utf8_lossy(&login.body).contains("\"password\":\"pw\""));
    assert_eq!(e.logout().url, "http://server:8091/api/v1/logout");
    assert_eq!(e.users().url, "http://server:8091/api/v1/users");
    let del = e.delete_user("anna b");
    assert_eq!((del.method, del.url.as_str()), (ehttp::Method::DELETE, "http://server:8091/api/v1/users/anna%20b"));
    assert_eq!(e.save_user("anna", &Default::default()).method, ehttp::Method::PUT);
}
