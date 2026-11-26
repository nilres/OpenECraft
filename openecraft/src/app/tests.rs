use super::*;

#[test]
fn page_tabs_follow_numbers_and_subpages_instead_of_opening_order() {
    let names = ["6", "1", "2", "4", "9", "2.1", "3.1", "3", "5", "8", "10", "2.10", "2.2"];
    let prj = Project {
        pages: names.iter().enumerate().map(|(i, name)| Page::new(i as u64 + 1, name, "", PageKind::Schematic)).collect(),
        ..Default::default()
    };
    let opened: Vec<u64> = prj.pages.iter().map(|p| p.id).rev().collect();
    let sorted = ordered_page_tabs(&prj, &opened);
    let actual: Vec<&str> = sorted.iter().map(|id| prj.pages.iter().find(|p| p.id == *id).unwrap().name.as_str()).collect();
    assert_eq!(actual, ["1", "2", "2.1", "2.2", "2.10", "3", "3.1", "4", "5", "6", "8", "9", "10"]);
    assert_eq!(prj.pages.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), names, "project page order stays unchanged");
    assert_eq!(opened, (1..=13).rev().collect::<Vec<u64>>(), "opening order is retained for the tab limit");
}

#[test]
fn page_tabs_follow_renumbering_and_ignore_closed_or_deleted_pages() {
    let mut prj = Project {
        pages: vec![
            Page::new(1, "10", "", PageKind::Schematic),
            Page::new(2, "2", "", PageKind::Schematic),
            Page::new(3, "1", "", PageKind::Schematic),
        ],
        ..Default::default()
    };
    let opened = vec![1, 2, 999];
    assert_eq!(ordered_page_tabs(&prj, &opened), vec![2, 1]);
    prj.pages[0].name = "1.1".into();
    assert_eq!(ordered_page_tabs(&prj, &opened), vec![1, 2]);
    prj.pages.remove(1);
    assert_eq!(ordered_page_tabs(&prj, &opened), vec![1]);
    assert!(ordered_page_tabs(&prj, &[]).is_empty());
}

#[test]
fn tablet_mode_of_a_wiring_start_is_not_stored() {
    let mut shell = ShellState { tablet_once: true, ..Default::default() };
    assert!(shell.tablet());
    let stored: ShellState = serde_json::from_str(&serde_json::to_string(&shell).unwrap()).unwrap();
    assert!(!stored.tablet(), "the next normal start has no tablet mode");
    // Switching it off ends it for the session too; switching it on is stored.
    shell.set_tablet(false);
    assert!(!shell.tablet());
    shell.set_tablet(true);
    let stored: ShellState = serde_json::from_str(&serde_json::to_string(&shell).unwrap()).unwrap();
    assert!(stored.tablet());
}

#[test]
fn the_unsaved_state_is_stored_with_the_shell() {
    let shell = ShellState { dirty: true, ..Default::default() };
    let stored: ShellState = serde_json::from_str(&serde_json::to_string(&shell).unwrap()).unwrap();
    assert!(stored.dirty);
    let old: ShellState = serde_json::from_str("{}").unwrap();
    assert!(!old.dirty, "stores written before have no flag");
}

#[test]
fn the_schematic_crosshair_setting_is_stored_with_the_shell() {
    let shell = ShellState { full_crosshair: true, ..Default::default() };
    let stored: ShellState = serde_json::from_str(&serde_json::to_string(&shell).unwrap()).unwrap();
    assert!(stored.full_crosshair);
    let old: ShellState = serde_json::from_str("{}").unwrap();
    assert!(!old.full_crosshair, "stores written before use the short crosshair");
}

#[test]
fn catalog_logos_are_shown_unless_switched_off() {
    let shell = ShellState { catalog_logos: false, ..Default::default() };
    let stored: ShellState = serde_json::from_str(&serde_json::to_string(&shell).unwrap()).unwrap();
    assert!(!stored.catalog_logos);
    let old: ShellState = serde_json::from_str("{}").unwrap();
    assert!(old.catalog_logos, "stores written before show the logos");
}

#[test]
fn wheel_source_policy_is_stored_with_the_shell() {
    let shell = ShellState {
        nav: openecraft_ui::nav::Settings {
            wheel_source_policy: openecraft_ui::nav::WheelSourcePolicy::AssumeWindowsTrackpad,
            ..Default::default()
        },
        ..Default::default()
    };
    let stored = serde_json::to_value(&shell).unwrap();
    let restored: ShellState = serde_json::from_value(stored.clone()).unwrap();
    assert_eq!(
        restored.nav.wheel_source_policy,
        openecraft_ui::nav::WheelSourcePolicy::AssumeWindowsTrackpad
    );

    let mut old_value = serde_json::to_value(ShellState {
        nav: openecraft_ui::nav::Settings {
            wheel_source_policy: openecraft_ui::nav::WheelSourcePolicy::Strict,
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();
    old_value
        .as_object_mut()
        .unwrap()
        .remove("wheel_source_policy_migrated");
    let mut old: ShellState = serde_json::from_value(old_value).unwrap();
    old.migrate_wheel_source_policy();
    assert_eq!(
        old.nav.wheel_source_policy,
        openecraft_ui::nav::WheelSourcePolicy::AssumeWindowsTrackpad,
        "previously saved default settings pan on unknown-source input"
    );

    let mut customized = ShellState {
        wheel_source_policy_migrated: true,
        nav: openecraft_ui::nav::Settings {
            wheel_source_policy: openecraft_ui::nav::WheelSourcePolicy::Strict,
            ..Default::default()
        },
        ..Default::default()
    };
    customized.migrate_wheel_source_policy();
    assert_eq!(
        customized.nav.wheel_source_policy,
        openecraft_ui::nav::WheelSourcePolicy::Strict,
        "an explicit user choice is preserved"
    );
}

#[test]
fn the_startup_setting_is_stored_with_the_shell() {
    use crate::startup::{ServerProject, Startup, Target};
    let startup = Startup::Project(Target::Server(ServerProject { url: "http://srv:8091".into(), name: "plant".into() }));
    let shell = ShellState { startup: startup.clone(), ..Default::default() };
    let stored: ShellState = serde_json::from_str(&serde_json::to_string(&shell).unwrap()).unwrap();
    assert_eq!(stored.startup, startup);
    let old: ShellState = serde_json::from_str("{}").unwrap();
    assert_eq!(old.startup, Startup::ReopenLast, "stores written before reopen the last project");
}

#[test]
fn counts_of_one_use_their_own_sentence() {
    openecraft_i18n::with_lang(openecraft_i18n::Lang::En, || {
        assert_eq!(errors_warnings(1, 1), "1 error, 1 warning");
        assert_eq!(errors_warnings(1, 3), "1 error, 3 warnings");
        assert_eq!(errors_warnings(2, 1), "2 errors, 1 warning");
        assert_eq!(errors_warnings(0, 0), "0 errors, 0 warnings");
        assert_eq!(counted(1, "1 note", "x".into()), "1 note");
        assert_eq!(counted(4, "1 note", "4 notes".into()), "4 notes");
    });
}
