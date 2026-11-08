use super::*;

fn ip(last: u8) -> Option<IpAddr> {
    Some(IpAddr::from([192, 0, 2, last]))
}

#[test]
fn delays_after_repeated_failures() {
    let mut t = Throttle::default();
    let now = Instant::now();
    for _ in 0..FREE_ATTEMPTS {
        assert_eq!(t.wait("Anna", ip(1), now), None);
        t.failed("anna ", ip(1), now);
    }
    assert_eq!(t.wait("ANNA", ip(1), now), Some(1));
    assert_eq!(t.wait("anna", ip(1), now + Duration::from_secs(2)), None);
    t.failed("anna", ip(1), now);
    assert_eq!(t.wait("anna", ip(1), now), Some(2));
    // Other names are not affected; a success forgets the failures.
    assert_eq!(t.wait("ben", ip(1), now), None);
    t.succeeded("Anna", ip(1));
    assert_eq!(t.wait("anna", ip(1), now), None);
}

#[test]
fn others_cannot_lock_a_user_out() {
    let mut t = Throttle::default();
    let now = Instant::now();
    for _ in 0..20 {
        t.failed("anna", ip(66), now);
    }
    assert!(t.wait("anna", ip(66), now).is_some());
    assert_eq!(t.wait("anna", ip(1), now), None);
}

#[test]
fn many_names_from_one_client_are_slowed_down() {
    let mut t = Throttle::default();
    let now = Instant::now();
    for i in 0..FREE_CLIENT_ATTEMPTS {
        assert_eq!(t.wait(&format!("user{i}"), ip(66), now), None);
        t.failed(&format!("user{i}"), ip(66), now);
    }
    assert_eq!(t.wait("someone else", ip(66), now), Some(1));
    assert_eq!(t.wait("someone else", ip(1), now), None);
    // Signing in to an own account does not reset the client.
    t.succeeded("mallory", ip(66));
    assert_eq!(t.wait("someone else", ip(66), now), Some(1));
}

#[test]
fn failures_are_forgotten() {
    let mut t = Throttle::default();
    let now = Instant::now();
    for _ in 0..FREE_ATTEMPTS + 3 {
        t.failed("anna", ip(1), now);
    }
    assert!(t.wait("anna", ip(1), now).is_some());
    assert_eq!(t.wait("anna", ip(1), now + FORGET_AFTER), None);
    t.failed("anna", ip(1), now + FORGET_AFTER);
    assert_eq!(t.wait("anna", ip(1), now + FORGET_AFTER), None, "counting starts again");
}

#[test]
fn wait_is_capped() {
    let mut t = Throttle::default();
    let now = Instant::now();
    for _ in 0..40 {
        t.failed("x", None, now);
    }
    assert_eq!(t.wait("x", None, now), Some(MAX_WAIT.as_secs()));
}

#[test]
fn a_blank_name_is_not_the_client() {
    let now = Instant::now();
    // Failures with other names do not delay a blank one before the client's limit.
    let mut t = Throttle::default();
    for i in 0..FREE_ATTEMPTS {
        t.failed(&format!("user{i}"), ip(1), now);
    }
    assert_eq!(t.wait(" ", ip(1), now), None);
    // A failure with a blank name counts once for the client.
    let mut t = Throttle::default();
    for _ in 0..FREE_CLIENT_ATTEMPTS / 2 + 1 {
        t.failed("", ip(2), now);
    }
    assert_eq!(t.wait("someone", ip(2), now), None);
    // Its success does not forget the failures of the client.
    let mut t = Throttle::default();
    for i in 0..FREE_CLIENT_ATTEMPTS {
        t.failed(&format!("user{i}"), ip(3), now);
    }
    t.succeeded("", ip(3));
    assert!(t.wait("someone", ip(3), now).is_some());
}
