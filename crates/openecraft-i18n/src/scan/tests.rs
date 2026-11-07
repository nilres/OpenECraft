use super::*;

#[test]
fn members_map_to_areas() {
    assert_eq!(area_of_member("openecraft"), "editor");
    assert_eq!(area_of_member("crates/openecraft-core"), "core");
    assert_eq!(area_of_member("crates/openecraft-api"), "plugins");
    assert_eq!(area_of_member("cli"), "cli");
    assert_eq!(area_of_member("crates/openecraft-ui"), "ui");
    assert_eq!(area_of_member("parts-server"), "parts");
    assert_eq!(area_of_member("parts-manager"), "parts");
    for member in members(&workspace_root()) {
        assert!(AREAS.contains(&area_of_member(&member)), "{member}");
    }
}

#[test]
fn texts_used_by_two_areas_are_common() {
    assert_eq!(area_for(&BTreeSet::new()), None);
    assert_eq!(area_for(&BTreeSet::from(["core"])), Some("core"));
    assert_eq!(area_for(&BTreeSet::from(["core", "editor"])), Some("common"));
}

#[test]
fn literals_are_found_and_unescaped() {
    let src = r#"
        ui.label(tr("Open"));
        let s = tr_fmt!("Saved {}", name);
        const T: &str = tr_const!("Say \"hi\"\n");
        issue!(Severity::Error, target, "Cable {} defined {} times", tag, n);
        other_tr("not this"); tr(variable); matr("nor this");
    "#;
    let texts = texts_in_source(src, false);
    assert_eq!(texts, ["Cable {} defined {} times", "Open", "Saved {}", "Say \"hi\"\n"]);
}

#[test]
fn symbol_names_are_found_in_symbol_files() {
    let src = r#"lib.push(def("coil", "Relay coil", C::Coils, "K", MAIN)); ("pot_l1", "L1", "L1")"#;
    assert_eq!(texts_in_source(src, true), ["Relay coil", "L1"]);
    assert!(texts_in_source(src, false).is_empty());
    assert!(is_symbol_file(Path::new("crates/openecraft-core/src/symbols.rs")));
    assert!(is_symbol_file(Path::new("crates/openecraft-core/src/symbols/iec.rs")));
    assert!(!is_symbol_file(Path::new("crates/openecraft-core/src/model.rs")));
}

#[test]
fn unescapes_rust_escapes() {
    assert_eq!(unescape(r#"a\nb\t\"c\"\\"#), "a\nb\t\"c\"\\");
    assert_eq!(unescape("plain"), "plain");
}

#[test]
fn the_workspace_is_scanned() {
    let used = used_texts(&workspace_root());
    assert!(used.len() > 500, "source scan found only {} texts – scanner broken?", used.len());
    let cancel = used.get("Cancel").expect("Cancel is translated somewhere");
    assert_eq!(area_for(&cancel.areas), Some("common"));
    assert_eq!(cancel.files.len(), cancel.areas.len());
}
