use super::*;

#[test]
fn locks_are_exclusive() {
    let now = Instant::now();
    let mut s = Sessions::default();
    let anna = s.open("p", "anna", now);
    let ben = s.open("p", "ben", now);
    assert_ne!(anna.session, ben.session);
    s.lock("p", &anna, &[Target::Page(1), Target::Project], now).unwrap();
    s.lock("p", &ben, &[Target::Page(2)], now).unwrap();
    // All or nothing.
    match s.lock("p", &ben, &[Target::Page(3), Target::Page(1)], now) {
        Err(LockError::Held(l)) => assert_eq!((l.user.as_str(), l.target), ("anna", Target::Page(1))),
        other => panic!("{other:?}"),
    }
    let locks = s.locks("p");
    assert_eq!(locks.iter().map(|l| l.target).collect::<Vec<_>>(), vec![Target::Project, Target::Page(1), Target::Page(2)]);
    assert_eq!(s.users("p"), vec!["anna".to_string(), "ben".to_string()]);
    // Locking again is fine.
    s.lock("p", &anna, &[Target::Page(1)], now).unwrap();

    s.release("p", &anna, &[Target::Page(1)], now).unwrap();
    s.lock("p", &ben, &[Target::Page(1)], now).unwrap();
    s.close("p", &anna).unwrap();
    assert_eq!(s.locks("p").len(), 2);
    assert_eq!(s.users("p"), vec!["ben".to_string()]);
}

#[test]
fn sessions_belong_to_user_and_project() {
    let now = Instant::now();
    let mut s = Sessions::default();
    let anna = s.open("p", "anna", now);
    assert_eq!(s.touch("q", &anna, now), Err(LockError::Expired));
    let fake = Session { user: "mallory".into(), session: anna.session.clone() };
    assert_eq!(s.lock("p", &fake, &[Target::Project], now), Err(LockError::Expired));
    assert!(s.locks("q").is_empty());
}

#[test]
fn idle_sessions_expire_with_their_locks() {
    let start = Instant::now();
    let mut s = Sessions::default();
    let anna = s.open("p", "anna", start);
    let ben = s.open("p", "ben", start);
    s.lock("p", &anna, &[Target::Page(1)], start).unwrap();
    let later = start + Duration::from_secs(SESSION_TIMEOUT_SECS - 10);
    s.touch("p", &ben, later).unwrap();
    s.expire(start + Duration::from_secs(SESSION_TIMEOUT_SECS + 1));
    assert!(s.locks("p").is_empty());
    assert_eq!(s.touch("p", &anna, later), Err(LockError::Expired));
    assert!(s.touch("p", &ben, later).is_ok());
}

#[test]
fn signing_out_and_deleting_the_user_end_their_sessions() {
    let now = Instant::now();
    let mut s = Sessions::default();
    let anna = s.open("p", "anna", now);
    let anna_tablet = s.open("q", "anna", now);
    let ben = s.open("p", "Ben", now);
    s.set_owner(&anna.session, "token-a".into());
    s.set_owner(&anna_tablet.session, "token-b".into());
    s.lock("p", &anna, &[Target::Page(1)], now).unwrap();
    assert_eq!(s.close_owned("token-a"), 1);
    assert!(s.locks("p").is_empty());
    // The session of the other sign-in stays.
    assert!(s.touch("q", &anna_tablet, now).is_ok());
    assert_eq!(s.close_user("ben"), 1);
    assert_eq!(s.touch("p", &ben, now), Err(LockError::Expired));
}

#[test]
fn ended_sessions_go_to_the_history_with_how_they_ended() {
    let start = Instant::now();
    let mut s = Sessions::default();
    let anna = s.open("p", "anna", start);
    let ben = s.open("p", "ben", start);
    let cleo = s.open("q", "cleo", start);
    let dan = s.open("q", "dan", start);
    s.set_owner(&cleo.session, "token-c".into());
    s.lock("p", &anna, &[Target::Page(1)], start).unwrap();
    s.saved(&anna.session);
    s.saved(&anna.session);
    let active = s.active(start);
    assert_eq!(active.iter().map(|r| r.user.as_str()).collect::<Vec<_>>(), ["dan", "cleo", "ben", "anna"], "newest first");
    assert_eq!((active[3].saves, active[3].locks.clone()), (2, vec![Target::Page(1)]));
    assert!(!format!("{active:?}").contains(&anna.session), "the key of a session is never shown");

    s.close("p", &anna).unwrap();
    s.close_owned("token-c");
    s.close_user("DAN");
    assert_eq!(s.touch("q", &dan, start), Err(LockError::Expired));
    s.touch("p", &ben, start).unwrap();
    s.expire(start + Duration::from_secs(SESSION_TIMEOUT_SECS + 1));
    assert!(s.active(start).is_empty());
    let history = s.history.recent(10);
    let ends: Vec<(&str, Option<SessionEnd>)> = history.iter().map(|r| (r.user.as_str(), r.end)).collect();
    assert_eq!(ends, [("ben", Some(SessionEnd::Expired)), ("dan", Some(SessionEnd::UserDeleted)), ("cleo", Some(SessionEnd::SignedOut)), ("anna", Some(SessionEnd::Closed))]);
    let anna = &history[3];
    assert_eq!((anna.project.as_str(), anna.saves, anna.number), ("p", 2, 1));
    assert!(anna.locks.is_empty() && anna.ended.is_some_and(|e| e >= anna.started));
}
