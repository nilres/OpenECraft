use super::*;

#[test]
fn registry_lists_all_names() {
    let mut r = Registry::default();
    r.command("a", "A", vec![], |_, _| Ok(Value::Null)).report("b", "B", |_, _| Ok(Table::default())).exporter("c", "C", "txt", |_| Ok(String::new()));
    assert_eq!(r.names().collect::<Vec<_>>(), [("command", "a"), ("report", "b"), ("exporter", "c")]);
}

#[test]
fn reports_are_found_by_title_name_and_keywords() {
    let mut r = Registry::default();
    r.report("inspection", "Inspection and test report", |_, _| Ok(Table::default())).keywords(["DIN VDE 0100-600", "IEC 60364-6"]);
    let report = &r.reports[0];
    let found = |q: &str| report.matches("Prüfbericht", q);
    // The title as shown (translated), the name and the keywords; case, spaces and
    // punctuation do not matter.
    for q in ["", "  ", "prüf", "INSPECTION", "VDE 0100 600", "vde0100-600", "0100-600", "DIN VDE", "60364-6", "iec 60364"] {
        assert!(found(q), "{q:?} finds the report");
    }
    // Every word in one of the texts.
    for q in ["VDE 0100 410", "60364 VDE", "test report", "-"] {
        assert_eq!(found(q), q == "-", "{q:?}");
    }
    r.report("other", "Other", |_, _| Ok(Table::default()));
    assert!(r.reports[1].keywords.is_empty() && !r.reports[1].matches("Other", "VDE"));
}
