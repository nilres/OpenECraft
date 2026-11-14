use super::*;

const ON: Settings = Settings { enabled: true, secs: 30 };

fn facts(changed: bool) -> Facts {
    Facts { changed, committed: true, saving: false, link: Link::Online }
}

#[test]
fn thirty_seconds_and_on_by_default() {
    assert_eq!(Settings::default(), Settings { enabled: true, secs: 30 });
    assert_eq!(Settings { enabled: true, secs: 1 }.interval(), 10.0, "never faster than the shortest interval");
    let stored: Settings = serde_json::from_str("{}").unwrap();
    assert_eq!(stored, Settings::default(), "settings stored before the option existed");
}

#[test]
fn an_unchanged_project_is_never_saved() {
    let mut auto = Auto::default();
    for t in 0..200 {
        assert!(!auto.due(f64::from(t), &ON, &facts(false)));
    }
    assert_eq!(auto.wait(200.0, &ON), None);
}

#[test]
fn a_change_is_saved_after_the_interval_and_then_every_interval() {
    let mut auto = Auto::default();
    assert!(!auto.due(100.0, &ON, &facts(true)));
    assert_eq!(auto.wait(110.0, &ON), Some(20.0));
    assert!(!auto.due(129.9, &ON, &facts(true)));
    assert!(auto.due(130.0, &ON, &facts(true)));
    // Edits go on: the next save is one interval after this one.
    assert!(!auto.due(130.1, &ON, &facts(true)));
    assert!(!auto.due(159.0, &ON, &facts(true)));
    assert!(auto.due(160.0, &ON, &facts(true)));
    // Saved: the timer starts with the next change.
    assert!(!auto.due(161.0, &ON, &facts(false)));
    assert!(!auto.due(500.0, &ON, &facts(true)));
    assert!(auto.due(530.0, &ON, &facts(true)));
}

#[test]
fn input_under_way_is_not_saved_until_it_is_finished() {
    let mut auto = Auto::default();
    let typing = Facts { committed: false, ..facts(true) };
    assert!(!auto.due(0.0, &ON, &typing));
    assert!(!auto.due(31.0, &ON, &typing));
    assert!(!auto.due(90.0, &ON, &typing));
    assert_eq!(auto.wait(90.0, &ON), Some(0.0));
    assert!(auto.due(90.1, &ON, &facts(true)), "saved as soon as the input is finished");
}

#[test]
fn a_slow_save_is_not_overtaken_by_the_next() {
    let mut auto = Auto::default();
    auto.due(0.0, &ON, &facts(true));
    assert!(auto.due(30.0, &ON, &facts(true)));
    let slow = Facts { saving: true, ..facts(true) };
    for t in 31..120 {
        assert!(!auto.due(f64::from(t), &ON, &slow), "one save at a time");
    }
    assert!(auto.due(120.0, &ON, &facts(true)), "edits made meanwhile are saved after its answer");
}

#[test]
fn a_failed_save_is_tried_again_after_the_interval() {
    let mut auto = Auto::default();
    auto.due(0.0, &ON, &facts(true));
    assert!(auto.due(30.0, &ON, &facts(true)));
    // The answer is an error: the project stays changed.
    assert!(!auto.due(31.0, &ON, &facts(true)));
    assert!(!auto.due(59.0, &ON, &facts(true)));
    assert!(auto.due(60.0, &ON, &facts(true)));
}

#[test]
fn nothing_is_sent_without_a_session_and_the_changes_are_saved_once_it_is_back() {
    for link in [Link::Offline, Link::Ended, Link::SignedOut, Link::Refused] {
        let mut auto = Auto::default();
        let away = Facts { link, ..facts(true) };
        for t in 0..300 {
            assert!(!auto.due(f64::from(t), &ON, &away), "{link:?}");
        }
        assert!(auto.due(300.0, &ON, &facts(true)), "{link:?}: saved when the session goes on");
        assert!(!auto.due(300.1, &ON, &facts(true)));
    }
}

#[test]
fn switched_off_nothing_is_saved() {
    let mut auto = Auto::default();
    let off = Settings { enabled: false, secs: 30 };
    assert!(!auto.due(0.0, &off, &facts(true)));
    assert!(!auto.due(100.0, &off, &facts(true)));
    assert_eq!(auto.wait(100.0, &off), None);
    // Switched on: the interval starts then.
    assert!(!auto.due(101.0, &ON, &facts(true)));
    assert!(auto.due(131.0, &ON, &facts(true)));
}

#[test]
fn the_interval_can_be_changed() {
    let mut auto = Auto::default();
    let slow = Settings { enabled: true, secs: 120 };
    auto.due(0.0, &slow, &facts(true));
    assert!(!auto.due(60.0, &slow, &facts(true)));
    assert!(auto.due(120.0, &slow, &facts(true)));
}

#[test]
fn the_states_are_told_apart() {
    let of = State::of;
    assert_eq!(of(Link::Online, false, false, false, false), State::Saved);
    assert_eq!(of(Link::Online, false, false, false, true), State::Pending);
    assert_eq!(of(Link::Online, true, false, false, true), State::Saving);
    assert_eq!(of(Link::Offline, false, true, false, true), State::Offline);
    assert_eq!(of(Link::Ended, false, false, false, true), State::Offline);
    assert_eq!(of(Link::Online, false, true, false, true), State::Failed);
    assert_eq!(of(Link::Online, false, false, true, true), State::Partial);
    // Without changes nothing waits, whatever the connection.
    assert_eq!(of(Link::Offline, false, false, false, false), State::Saved);
    let all = [State::Saved, State::Pending, State::Saving, State::Offline, State::Failed, State::Partial];
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a.label(), b.label());
            assert_ne!(a.text(Some("e"), Some(30)), b.text(Some("e"), Some(30)));
        }
    }
}

#[test]
fn the_text_says_when_the_changes_are_saved() {
    assert!(State::Pending.text(None, Some(45)).contains("45"));
    assert!(!State::Pending.text(None, None).contains("automatically"));
    assert!(State::Failed.text(Some("disk full"), Some(30)).contains("disk full"));
    assert!(State::Failed.text(Some("disk full"), None).contains("disk full"));
}
