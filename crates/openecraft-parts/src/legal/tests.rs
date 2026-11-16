use super::*;

#[test]
fn every_field_is_read_and_set_by_its_name() {
    let mut notice = LegalNotice::default();
    assert!(!notice.shown());
    for (name, _) in FIELDS {
        assert!(notice.set(name, format!(" {name} value ")), "{name}");
        assert_eq!(notice.field(name), format!("{name} value"));
    }
    assert!(notice.shown());
    assert!(!notice.set("other", "x") && notice.field("other").is_empty());
    // The fields are those of the JSON and of the configuration files.
    let json = serde_json::to_value(&notice).unwrap();
    let keys: Vec<&str> = json.as_object().unwrap().keys().map(String::as_str).collect();
    let mut names: Vec<&str> = FIELDS.iter().map(|f| f.0).collect();
    names.sort_unstable();
    assert_eq!(keys, names);
    assert!(serde_json::from_str::<LegalNotice>("{\"nmae\":\"x\"}").is_err(), "a misspelt field is refused");
}

#[test]
fn lines_are_separated_by_breaks_or_bars() {
    let notice = LegalNotice { address: "Street 1 | 12345 City\n\nGermany |".into(), ..Default::default() };
    assert_eq!(notice.lines("address"), ["Street 1", "12345 City", "Germany"]);
    assert!(notice.lines("text").is_empty());
}

#[test]
fn the_environment_wins_over_the_file() {
    let file = LegalNotice { name: "ACME".into(), email: "info@example.com".into(), ..Default::default() };
    let notice = file.with_environment(|var| match var {
        "OPENECRAFT_LEGAL_NAME" => Some("ACME GmbH".into()),
        "OPENECRAFT_LEGAL_EMAIL" => Some("  ".into()),
        "OPENECRAFT_LEGAL_VAT_ID" => Some("DE123".into()),
        _ => None,
    });
    assert_eq!(notice, LegalNotice { name: "ACME GmbH".into(), email: "info@example.com".into(), vat: "DE123".into(), ..Default::default() });
}
