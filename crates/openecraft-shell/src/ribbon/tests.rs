use super::*;

#[test]
fn tabs_and_groups_have_unique_keys_and_tips() {
    let mut keys = std::collections::BTreeSet::new();
    let mut tips = std::collections::BTreeSet::new();
    for t in TABS {
        assert!(keys.insert(t.key), "tab {} twice", t.key);
        assert!(tips.insert(t.key_tip), "key tip {} twice", t.key_tip);
        let mut groups = std::collections::BTreeSet::new();
        for g in t.groups {
            assert!(groups.insert(g.id), "group {}/{} twice", t.key, g.id);
            assert!(!g.items.is_empty(), "group {}/{} is empty", t.key, g.id);
        }
    }
    assert!(!tips.contains(&'F'), "F is the key tip of the File tab");
    assert_eq!(tab(TabId::Plugins).groups.len(), 1, "favorites, recent and external plugins are added from the registry");
}

#[test]
fn context_tabs_are_the_layout_space_and_the_wiring() {
    let ctx: Vec<TabId> = TABS.iter().filter(|t| t.context).map(|t| t.id).collect();
    assert_eq!(ctx, vec![TabId::LayoutSpace, TabId::Wiring]);
}

#[test]
fn the_backstage_offers_files_and_settings() {
    for c in [Cmd::Save, Cmd::SaveAs, Cmd::Settings, Cmd::Quit, Cmd::Open, Cmd::NewProject, Cmd::PrintAll, Cmd::ImportDrawing] {
        assert!(places(c).iter().any(|p| p.starts_with("backstage")), "{c:?}");
    }
    assert!(places(Cmd::Undo).contains(&"quick".to_string()));
}

#[test]
fn key_tips_take_free_letters_of_the_titles() {
    let titles: Vec<String> = ["Paste", "Cut", "Copy", "Paste special", "?"].iter().map(|s| s.to_string()).collect();
    let tips = key_tips(&titles, &['F']);
    assert_eq!(tips[0], 'P');
    assert_eq!(tips[1], 'C');
    assert_eq!(tips[2], 'O', "C is taken, the next letter of Copy");
    assert_eq!(tips[3], 'A');
    assert!(tips[4].is_ascii_alphanumeric());
    let unique: std::collections::BTreeSet<char> = tips.iter().copied().collect();
    assert_eq!(unique.len(), tips.len());
    assert!(!tips.contains(&'F'));
}

#[test]
fn customisation_hides_groups_and_adds_own_ones() {
    let home = tab(TabId::Home);
    let mut c = Custom::default();
    c.set_hidden(home, &home.groups[0], true);
    assert!(c.is_hidden(home, &home.groups[0]));
    c.groups.push(CustomGroup { tab: "home".into(), title: "Mine".into(), commands: vec!["save".into(), "nonsense".into(), "wire_tool".into()] });
    let own = c.groups_of(home);
    assert_eq!(own.len(), 1);
    assert_eq!(own[0].2, vec![Cmd::Save, Cmd::WireTool]);
    let back = Custom::from_json(&c.to_json()).unwrap();
    assert_eq!(back.groups[0].commands, vec!["save".to_string(), "wire_tool".to_string()], "unknown commands are dropped");
    assert!(back.is_hidden(home, &home.groups[0]));
    c.set_hidden(home, &home.groups[0], false);
    assert!(!c.is_hidden(home, &home.groups[0]));
    assert!(Custom::from_json("not json").is_err());
    let foreign = r#"{"hidden":["mars/rover"],"groups":[{"tab":"mars","title":"x","commands":["save"]}]}"#;
    let f = Custom::from_json(foreign).unwrap();
    assert!(f.hidden.is_empty() && f.groups.is_empty());
}

#[test]
fn groups_collapse_from_the_right_until_the_tab_fits() {
    let full = [100.0, 200.0, 150.0, 60.0];
    let short = [60.0, 60.0, 60.0, 60.0];
    assert_eq!(collapsed(&full, &short, 600.0), vec![false; 4], "everything fits");
    // 510 wide: the last group would not get narrower, the third one collapses.
    assert_eq!(collapsed(&full, &short, 450.0), vec![false, false, true, false]);
    assert_eq!(collapsed(&full, &short, 300.0), vec![false, true, true, false]);
    assert_eq!(collapsed(&full, &short, 10.0), vec![true, true, true, false], "as narrow as it gets");
    assert!(collapsed(&[], &[], 0.0).is_empty());
    // The wide group had to go; the narrow one collapsed before it fits again.
    assert_eq!(collapsed(&[100.0, 400.0, 100.0], &[60.0, 60.0, 60.0], 300.0), vec![false, true, false]);
}

#[test]
fn a_command_has_one_place_in_the_ribbon() {
    // Deliberate second entry points: the navigators of Home and the quick access row.
    let mut seen: std::collections::BTreeMap<Cmd, Vec<String>> = Default::default();
    for t in TABS {
        for g in t.groups {
            if t.id == TabId::Home && g.id == "navigators" {
                continue;
            }
            for i in g.items {
                seen.entry(i.cmd()).or_default().push(format!("{}/{}", t.key, g.id));
            }
        }
    }
    let twice: Vec<_> = seen.iter().filter(|(_, p)| p.len() > 1).collect();
    assert!(twice.is_empty(), "commands in more than one group: {twice:?}");
    let search: Vec<String> = places(Cmd::CommandSearch).into_iter().filter(|p| !p.starts_with("backstage")).collect();
    assert_eq!(search, vec!["home/find".to_string()], "the box of the quick access row and Home › Find");
    assert_eq!(places(Cmd::ManagePlugins).iter().filter(|p| !p.starts_with("backstage") && *p != "view/panes").count(), 1);
    for c in [Cmd::ZoomActual, Cmd::ZoomIn, Cmd::ZoomOut, Cmd::FitPage] {
        assert!(places(c).contains(&"view/zoom".to_string()), "{c:?}");
    }
    assert!(places(Cmd::DevicesPane).contains(&"devices/navigators".to_string()));
}
