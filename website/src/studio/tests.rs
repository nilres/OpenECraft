use super::*;

fn studio() -> Studio {
    let mut s = Studio::default();
    s.settle();
    s
}

fn ok(s: &mut Studio, method: &str, arg: &str) -> Value {
    let answer = s.call(method, arg);
    assert!(answer.get("error").is_none(), "{method} {arg}: {answer}");
    answer
}

#[test]
fn the_first_example_has_pages_devices_and_a_panel_layout() {
    let mut s = studio();
    let state = ok(&mut s, "state", "");
    assert_eq!(state["example"], FIRST_EXAMPLE);
    assert!(state["pages"].as_array().unwrap().len() > 3);
    assert!(state["devices"].as_u64().unwrap() > 5);
    assert!(state["enclosures"].as_u64().unwrap() >= 1, "the 3D view needs an enclosure");
    assert!(state["issues"].as_array().unwrap().iter().all(|i| i["severity"] == "info"), "the examples pass the checks");
    assert_eq!(state["loading"], false);
}

#[test]
fn every_example_opens() {
    let mut s = studio();
    let examples = ok(&mut s, "examples", "");
    assert_eq!(examples.as_array().unwrap().len(), sample::EXAMPLES.len());
    ok(&mut s, "open", "lamp_circuit");
    s.settle();
    let state = ok(&mut s, "state", "");
    assert_eq!((state["example"].as_str(), state["page"].as_u64()), (Some("lamp_circuit"), Some(s.session.page as u64)));
    assert!(s.call("open", "nothing").get("error").is_some());
}

#[test]
fn commands_run_as_on_the_command_line_and_can_be_undone() {
    let mut s = studio();
    let pages = s.run("list_pages").unwrap();
    assert!(pages.contains("\"name\""), "{pages}");
    let tag = s.session.index.sorted_tags().into_iter().find(|t| t.contains("-K")).cloned().expect("a contactor");
    let before = s.session.prj.clone();
    s.run(&format!("rename_device --old {tag} --new=-K77")).unwrap();
    s.settle();
    assert!(s.session.index.devices.keys().any(|t| t.ends_with("-K77")));
    assert_eq!(ok(&mut s, "state", "")["undo"], true);
    ok(&mut s, "undo", "");
    assert!(s.session.prj == before);
    // Errors are answers, not panics; a failed command changes nothing.
    assert!(s.run("no_such_command").is_err());
    assert!(s.run("rename_device --old").is_err());
    assert!(s.run("list_pages extra").is_err());
    assert!(s.run("").is_err());
    assert!(s.session.prj == before);
}

#[test]
fn a_change_shows_in_the_checks() {
    let mut s = studio();
    let page = s.session.page;
    let wires = s.session.prj.pages[page].wires.len();
    assert!(wires > 0);
    assert_eq!(ok(&mut s, "state", "")["issues"].as_array().unwrap().iter().filter(|i| i["severity"] != "info").count(), 0);
    s.session.prj.pages[page].wires.truncate(wires / 2);
    s.settle();
    let problems = |s: &mut Studio| ok(s, "state", "")["issues"].as_array().unwrap().iter().filter(|i| i["severity"] != "info").count();
    assert!(problems(&mut s) > 0, "open connections are reported");
    ok(&mut s, "issue", "0");
    ok(&mut s, "undo", "");
    s.settle();
    assert_eq!(problems(&mut s), 0);
}

#[test]
fn reports_and_exports_come_from_the_plugins() {
    let mut s = studio();
    let reports = ok(&mut s, "reports", "");
    let reports = reports.as_array().unwrap();
    assert!(reports.len() > 5);
    let mut filled = 0;
    for r in reports {
        let table = ok(&mut s, "report", r["name"].as_str().unwrap());
        assert!(!table["headers"].as_array().unwrap().is_empty(), "{r}");
        filled += usize::from(table["total"].as_u64().unwrap() > 0);
    }
    assert!(filled > 5);
    let exporters = ok(&mut s, "exporters", "");
    let exporters = exporters.as_array().unwrap();
    assert!(exporters.iter().any(|e| e["extension"] == "pdf"), "{exporters:?}");
    for e in exporters {
        let (name, bytes) = s.export(e["name"].as_str().unwrap()).unwrap_or_else(|err| panic!("{e}: {err}"));
        assert!(name.starts_with(FIRST_EXAMPLE) && !bytes.is_empty(), "{e}");
    }
}

#[test]
fn the_language_changes_generated_texts() {
    let mut s = studio();
    openecraft_i18n::with_lang(Lang::En, || {
        let english = ok(&mut s, "reports", "").to_string();
        let texts = ok(&mut s, "texts", "");
        assert_eq!(texts.as_object().unwrap().len(), crate::texts::TEXTS.len());
        assert!(s.call("language", "xx").get("error").is_some());
        let german = openecraft_i18n::with_lang(Lang::De, || ok(&mut s, "reports", "").to_string());
        assert_ne!(english, german);
    });
    assert_eq!(ok(&mut s, "languages", "").as_array().unwrap().len(), Lang::ALL.len());
}

#[test]
fn pages_tools_and_views_follow_the_page() {
    let mut s = studio();
    ok(&mut s, "page", "1");
    assert_eq!(s.session.page, 1);
    assert!(s.call("page", "x").get("error").is_some());
    ok(&mut s, "page", "9999");
    assert_eq!(s.session.page, 1, "an unknown page changes nothing");
    for (name, _) in SYMBOLS {
        ok(&mut s, "tool", name);
        assert_eq!(ok(&mut s, "state", "")["tool"], name);
        assert!(matches!(&s.editor.tool, Tool::Place(k) if openecraft_core::symbols::builtin(k).is_some()), "{name}");
    }
    ok(&mut s, "tool", "wire");
    assert_eq!(s.editor.tool, Tool::Wire);
    assert!(s.call("tool", "hammer").get("error").is_some());
    ok(&mut s, "view", "front");
    assert_eq!(ok(&mut s, "state", "")["view"], "front");
    let door = s.panel.door_open;
    ok(&mut s, "door", "");
    assert_eq!(ok(&mut s, "state", "")["door"], !door);
    let svg = ok(&mut s, "svg", "0");
    assert!(svg.as_str().unwrap().starts_with("<svg"));
}

#[test]
fn typed_lines_are_split_at_blanks_outside_quotes() {
    assert_eq!(split_line("  a  --b \"c d\" 'e \"f\"' g''h ").unwrap(), ["a", "--b", "c d", "e \"f\"", "gh"]);
    assert_eq!(split_line("--text \"\"").unwrap(), ["--text", ""]);
    assert!(split_line("a \"b").is_err());
    assert!(split_line("   ").unwrap().is_empty());
}
