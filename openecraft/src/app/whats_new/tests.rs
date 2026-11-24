use super::*;

#[test]
fn the_history_starts_with_this_version() {
    let history = history();
    assert_eq!(history[0].version, VERSION);
    assert!(history.iter().all(|r| changelog::number(&r.version) <= changelog::number(VERSION)));
    assert!(title(&history[history.len() - 1]).starts_with("0.1.0 – 2026-11-04"));
}

#[test]
fn the_window_shows_after_an_update_only() {
    // The first start of all has the start page instead.
    assert!(at_start(true, "").is_empty());
    // The same version again: nothing new.
    assert!(at_start(false, VERSION).is_empty());
    // From a version that did not remember it: this version's features.
    assert_eq!(at_start(false, ""), [0]);
    // From an older version: every version since.
    let all = at_start(false, "0.0.1");
    assert_eq!(all.len(), history().len());
    assert_eq!(all[0], 0);
}
