use super::*;

#[test]
fn reads_lists_words_and_strings() {
    let e = parse("(export (version \"E\")\n  (comp (ref \"J1\") (value 10k) hide (at 1.27 -2.54 90)))").unwrap();
    assert_eq!(e.name(), "export");
    assert_eq!(e.value("version"), Some("E"));
    let comp = e.child("comp").unwrap();
    assert_eq!(comp.value("ref"), Some("J1"));
    assert_eq!(comp.value("value"), Some("10k"));
    assert!(comp.flag("hide"));
    assert!(!comp.flag("power"));
    let at = comp.child("at").unwrap();
    assert_eq!((at.num(0), at.num(1), at.num(2)), (Some(1.27), Some(-2.54), Some(90.0)));
    assert_eq!(at.num(3), None);
}

#[test]
fn flags_are_words_or_lists_that_say_yes() {
    let e = parse("(pin (hide yes) (dnp no))").unwrap();
    assert!(e.flag("hide"));
    assert!(!e.flag("dnp"));
}

#[test]
fn strings_keep_what_quote_escapes() {
    for s in ["plain", "with \"quotes\"", "back\\slash", "two\nlines", "(paren) ; ä µ"] {
        let e = parse(&format!("(t {})", quote(s))).unwrap();
        assert_eq!(e.arg(0), Some(s));
    }
}

#[test]
fn refuses_what_is_not_closed_or_too_deep() {
    assert!(parse("(a (b)").is_err());
    assert!(parse(")").is_err());
    assert!(parse("").is_err());
    assert!(parse("(a \"open").is_err());
    let deep = "(".repeat(MAX_DEPTH + 1);
    assert!(parse(&deep).is_err());
    // The line of the fault is named.
    assert!(parse("(a\n(b\n").unwrap_err().contains('3'));
}

#[test]
fn numbers_have_no_trailing_zeros() {
    assert_eq!(number(2.54), "2.54");
    assert_eq!(number(-0.00001), "0");
    assert_eq!(number(10.0), "10");
    assert_eq!(number(0.635), "0.635");
}
