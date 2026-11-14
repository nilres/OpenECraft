use super::*;

#[test]
fn shortcuts_parse_and_print() {
    let s = Shortcut::parse("Ctrl+Shift+Z").unwrap();
    assert!(s.command && s.shift && !s.alt && s.key == Key::Z);
    assert_eq!(s.text(), "Ctrl+Shift+Z");
    assert_eq!(Shortcut::parse("PageDown").unwrap().text(), "PageDown");
    assert_eq!(Shortcut::parse("alt + F1").unwrap().text(), "Alt+F1");
    assert_eq!(Shortcut::parse("Hyper+Z"), None);
    assert_eq!(Shortcut::parse("Ctrl+"), None);
    for id in binding_ids() {
        for k in defaults(id) {
            assert_eq!(Shortcut::parse(&k.text()), Some(k), "{id}");
        }
    }
}

#[test]
fn every_default_key_of_the_command_table_is_read() {
    for d in commands::ALL {
        let written = d.keys.split('|').filter(|k| !k.is_empty()).count();
        assert_eq!(d.default_keys().len(), written, "{}: {}", d.id, d.keys);
    }
}

#[test]
fn modifiers_must_match_exactly() {
    let undo = Shortcut::parse("Ctrl+Z").unwrap();
    assert!(undo.held(Modifiers::COMMAND));
    assert!(!undo.held(Modifiers::COMMAND | Modifiers::SHIFT), "Ctrl+Shift+Z is redo, not undo");
    assert!(!Shortcut::parse("R").unwrap().held(Modifiers::COMMAND));
}

#[test]
fn defaults_have_no_conflicts_and_ids_are_unique() {
    assert_eq!(Shortcuts::default().conflicts(), Vec::new());
    let ids = binding_ids();
    let unique: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
    assert_eq!(unique.len(), ids.len());
}

#[test]
fn ids_of_earlier_versions_still_name_the_same_function() {
    for id in ["open", "save", "undo", "redo", "find", "select_all", "previous_page", "next_page", "fit_page", "wire_tool", "text_tool", "rotate", "mirror", "wire_corner", "autoroute", "finish_polyline"] {
        assert!(!defaults(id).is_empty(), "{id} lost its keys");
    }
}

#[test]
fn changed_keys_are_stored_and_conflicts_found() {
    let mut s = Shortcuts::default();
    s.set("rotate", vec![Shortcut::parse("Ctrl+R").unwrap()]);
    s.set("text_tool", vec![Shortcut::parse("W").unwrap()]);
    s.set("autoroute", vec![]);
    assert_eq!(s.text("rotate"), "Ctrl+R");
    assert_eq!(s.text("autoroute"), "");
    assert_eq!(s.text("redo"), "Ctrl+Y / Ctrl+Shift+Z");
    assert_eq!(s.cmd_text(Cmd::Undo), "Ctrl+Z");
    let conflicts = s.conflicts();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].1, vec!["wire_tool", "text_tool"]);
    let stored = s.to_storage();
    assert_eq!(stored.len(), 3);
    assert_eq!(Shortcuts::from_storage(&stored), s);
    s.set("rotate", defaults("rotate"));
    assert!(s.is_default("rotate"), "setting the defaults again removes the change");
    s.reset_all();
    assert_eq!(s, Shortcuts::default());
}

#[test]
fn unknown_stored_entries_are_ignored() {
    let stored: BTreeMap<String, Vec<String>> = [("teleport".to_string(), vec!["X".to_string()]), ("rotate".to_string(), vec!["Bogus+R".to_string(), "E".to_string()])].into();
    let s = Shortcuts::from_storage(&stored);
    assert_eq!(s.text("rotate"), "E");
    assert_eq!(s.to_storage().len(), 1);
}

#[test]
fn shortcut_groups_hold_every_binding_once() {
    let groups = groups();
    let mut all: Vec<&str> = groups.iter().flat_map(|(_, ids)| ids.iter().copied()).collect();
    let n = all.len();
    all.sort_unstable();
    all.dedup();
    assert_eq!(all.len(), n, "a binding in two groups");
    let mut ids = binding_ids();
    ids.sort_unstable();
    assert_eq!(all, ids);
    // The Home tab comes first with its commands.
    assert!(groups[0].1.contains(&commands::def(Cmd::Paste).id));
}
