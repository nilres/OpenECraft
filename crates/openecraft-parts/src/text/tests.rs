use super::*;

#[test]
fn natural_order() {
    let mut v = vec!["-K10", "-K2", "-F1", "-K1"];
    v.sort_by(|a, b| natural_cmp(a, b));
    assert_eq!(v, vec!["-F1", "-K1", "-K2", "-K10"]);
}

#[test]
fn dates() {
    assert_eq!(date_from_unix(0), "1970-01-01");
    assert_eq!(date_from_unix(951_782_400), "2000-02-29");
    assert_eq!(date_from_unix(1_792_713_600), "2026-10-23");
    assert_eq!(unix_from_date("1970-01-01"), Some(0));
    assert_eq!(unix_from_date("2000-02-29"), Some(951_782_400));
    assert_eq!(unix_from_date("2026-10-23T01:02:03+02:00"), Some(1_792_713_600 + 3723));
    assert_eq!(unix_from_date("2026-10-23 01:02:03"), Some(1_792_713_600 + 3723));
    assert_eq!(unix_from_date("23.10.2026"), None);
    assert_eq!(unix_from_date("2026-13-01"), None);
    assert_eq!(unix_from_date(""), None);
}

#[test]
fn wildcards() {
    let yes = [("", ""), ("*", ""), ("*", "abc"), ("a*", "abc"), ("*c", "abc"), ("a?c", "abc"), ("*b*", "abc"), ("a*b*c", "aXbYc"), ("**", "x"), ("a*a", "aa"), ("*ab", "aab")];
    for (p, t) in yes {
        assert!(wildcard(p, t), "{p} matches {t}");
    }
    let no = [("", "a"), ("?", ""), ("a", "A"), ("a*", "ba"), ("*b", "bc"), ("a*a", "a"), ("a?", "a"), ("*x*", "abc")];
    for (p, t) in no {
        assert!(!wildcard(p, t), "{p} does not match {t}");
    }
}

#[test]
fn wildcards_of_long_texts_are_cheap() {
    // The recursive match before overflowed the stack and took exponential time here.
    let long = "x".repeat(1_000_000);
    assert!(wildcard("*", &long));
    assert!(!wildcard("*y", &long));
    let a = "a".repeat(5_000);
    assert!(!wildcard("*a*a*a*a*a*a*a*a*a*b", &a));
}
