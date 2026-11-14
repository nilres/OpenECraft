use super::*;
use openecraft_core::collab::SessionEnd;

fn record(number: u64) -> SessionRecord {
    SessionRecord { number, user: "anna".into(), project: "Plant".into(), started: 100, last_seen: 160, ended: Some(170), end: Some(SessionEnd::Closed), ..Default::default() }
}

fn dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("openecraft-history-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn ended_sessions_survive_a_restart() {
    let dir = dir("restart");
    let mut h = History::open(&dir).unwrap();
    h.add(record(1));
    h.add(record(2));
    std::fs::write(dir.join(FILE), format!("{}not json\n", std::fs::read_to_string(dir.join(FILE)).unwrap())).unwrap();
    let again = History::open(&dir).unwrap();
    assert_eq!(again.recent(10), vec![record(2), record(1)], "newest first, broken lines skipped");
    assert_eq!(again.recent(1).len(), 1);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn only_the_last_sessions_are_kept_in_memory_and_in_the_file() {
    let dir = dir("limit");
    let mut h = History::open_keeping(&dir, 3).unwrap();
    for n in 1..=10 {
        h.add(record(n));
    }
    assert_eq!(h.recent(100).iter().map(|r| r.number).collect::<Vec<_>>(), [10, 9, 8]);
    let lines = std::fs::read_to_string(dir.join(FILE)).unwrap().lines().count();
    assert!(lines <= 6, "the file is written anew when it grows: {lines} lines");
    let again = History::open_keeping(&dir, 3).unwrap();
    assert_eq!(again.recent(100), h.recent(100));
    std::fs::remove_dir_all(&dir).unwrap();
}
