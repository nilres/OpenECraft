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

/// Tiny scripted loopback peer: checks the real request order and credentials,
/// without contacting a configured server or relying on the production router.
fn snapshot_peer(script: Vec<(&str, u16, String)>) -> Result<openecraft_core::collab::Snapshot, String> {
    use std::io::{Read, Write};
    use std::time::{Duration, Instant};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    listener.set_nonblocking(true).unwrap();
    let script: Vec<_> = script.into_iter().map(|(line, status, body)| (line.to_string(), status, body)).collect();
    let server = std::thread::spawn(move || {
        for (line, status, body) in script {
            let until = Instant::now() + Duration::from_secs(5);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < until => std::thread::sleep(Duration::from_millis(5)),
                    Err(e) => panic!("expected {line}: {e}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut received = Vec::new();
            loop {
                let mut buffer = [0; 4096];
                let n = stream.read(&mut buffer).unwrap();
                assert!(n > 0);
                received.extend_from_slice(&buffer[..n]);
                if let Some(end) = received.windows(4).position(|b| b == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&received[..end]).to_lowercase();
                    let length = header.lines().find_map(|s| s.strip_prefix("content-length:").map(|s| s.trim().parse::<usize>().unwrap())).unwrap_or(0);
                    if received.len() >= end + 4 + length { break; }
                }
            }
            let request = String::from_utf8(received).unwrap();
            assert!(request.starts_with(&format!("{line} HTTP/1.1\r\n")), "{request}");
            assert!(request.to_lowercase().contains("authorization: bearer test-token\r\n"));
            if line.ends_with("/status") || line.ends_with("/snapshot") {
                let (_, body) = request.split_once("\r\n\r\n").unwrap();
                assert_eq!(serde_json::from_str::<Session>(body).unwrap(), session());
            }
            write!(stream, "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        }
    });
    let e = ProjectEndpoint { url, token: "test-token".into() };
    let (tx, rx) = std::sync::mpsc::channel();
    e.fetch_session_snapshot("plant", &session(), move |result| { tx.send(result).unwrap(); });
    let result = rx.recv_timeout(Duration::from_secs(10)).unwrap();
    server.join().unwrap();
    result
}

fn snapshots() -> (openecraft_core::collab::Snapshot, openecraft_core::collab::Status) {
    use openecraft_core::collab::{Lock, Snapshot, Status};
    let own = Lock { target: Target::Page(1), user: "anna".into(), session: session().session };
    // Two sessions can have the same display name. Only the status authenticates
    // which lock is ours; the GET snapshot redacts both session identifiers.
    let other = Lock { target: Target::Page(2), user: "anna".into(), session: String::new() };
    let status = Status { revision: 7, locks: vec![own.clone(), other.clone()] };
    let mut snapshot = Snapshot { revision: 7, locks: vec![own, other], ..Default::default() };
    snapshot.locks[0].session.clear();
    snapshot.project.pages.push(openecraft_core::model::Page::new(1, "1", "foreign edit", openecraft_core::model::PageKind::Schematic));
    (snapshot, status)
}

#[test]
fn session_snapshots_keep_the_modern_single_request() {
    let (mut snapshot, status) = snapshots();
    snapshot.locks = status.locks;
    let result = snapshot_peer(vec![("POST /api/v1/projects/plant/snapshot", 200, serde_json::to_string(&snapshot).unwrap())]).unwrap();
    assert_eq!(result, snapshot);
}

#[test]
fn older_servers_load_foreign_edits_and_keep_own_locks_without_guessing_names() {
    for missing in [404, 405] {
        let (snapshot, status) = snapshots();
        let result = snapshot_peer(vec![
            ("POST /api/v1/projects/plant/snapshot", missing, "{}".into()),
            ("GET /api/v1/projects/plant", 200, serde_json::to_string(&snapshot).unwrap()),
            ("POST /api/v1/projects/plant/status", 200, serde_json::to_string(&status).unwrap()),
        ]).unwrap();
        assert_eq!(result.project, snapshot.project);
        assert_eq!(result.locks, status.locks);
        assert_eq!(result.locks[0].session, session().session);
        assert!(result.locks[1].session.is_empty());
    }
}

#[test]
fn snapshot_authentication_and_other_errors_never_downgrade_to_get() {
    for status in [401, 403, 410, 500] {
        assert!(snapshot_peer(vec![("POST /api/v1/projects/plant/snapshot", status, r#"{"error":"refused"}"#.into())]).is_err());
    }
}

#[test]
fn legacy_snapshots_never_mix_revisions_or_ignore_a_failed_session_check() {
    let (snapshot, mut status) = snapshots();
    status.revision += 1;
    let result = snapshot_peer(vec![
        ("POST /api/v1/projects/plant/snapshot", 404, "{}".into()),
        ("GET /api/v1/projects/plant", 200, serde_json::to_string(&snapshot).unwrap()),
        ("POST /api/v1/projects/plant/status", 200, serde_json::to_string(&status).unwrap()),
    ]);
    assert!(result.unwrap_err().contains("changed while loading"));
    for code in [401, 410] {
        assert!(snapshot_peer(vec![
            ("POST /api/v1/projects/plant/snapshot", 404, "{}".into()),
            ("GET /api/v1/projects/plant", 200, serde_json::to_string(&snapshot).unwrap()),
            ("POST /api/v1/projects/plant/status", code, r#"{"error":"expired"}"#.into()),
        ]).is_err());
    }
    assert!(snapshot_peer(vec![
        ("POST /api/v1/projects/plant/snapshot", 404, "{}".into()),
        ("GET /api/v1/projects/plant", 403, r#"{"error":"refused"}"#.into()),
    ]).is_err());
}
