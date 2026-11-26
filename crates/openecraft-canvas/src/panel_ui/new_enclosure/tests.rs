use super::*;

/// The Striebel & John wall cabinet of 3 field widths and 7 rows (shipped catalog).
const CABINET: &str = "SJ.2CPX052032R9999";

#[test]
fn enclosures_of_the_shipped_catalogs_are_found_by_rows_and_fields() {
    let (prj, db) = (Project::default(), PartsDb::default());
    let filter = EnclosureFilter { rows: Some(7), fields: Some(3), manufacturer: "Striebel & John".into(), ..Default::default() };
    let found: Vec<&str> = filter.apply(sources(&prj, &db)).into_iter().map(|p| p.number.as_str()).collect();
    assert!(found.contains(&CABINET), "{found:?}");
    assert!(found.iter().all(|n| n.starts_with("SJ.")));
    // The generic enclosures of the example catalog are there too.
    let compact = EnclosureFilter { system: "compact".into(), ..Default::default() };
    assert!(!compact.apply(sources(&prj, &db)).is_empty());
}

#[test]
fn a_dropped_enclosure_part_becomes_an_enclosure_in_the_room() {
    let (mut prj, db) = (Project::default(), PartsDb::default());
    let mut ui = PanelUi::default();
    ui.new_room(&mut prj);
    let ei = ui.drop_part(&mut prj, &db, CABINET, Some(0)).unwrap();
    let e = &prj.layout.enclosures[ei];
    assert_eq!((e.name.as_str(), e.part.as_str(), e.width, e.height, e.depth), ("+CAB1", CABINET, 800.0, 1100.0, 215.0));
    assert_eq!(panel::room_of(&prj.layout, ei), Some(0));
    assert_eq!((ui.enclosure, ui.in_room), (ei, false));
    // The part comes into the project with its accessories from the shipped catalog.
    let part = prj.parts.iter().find(|p| p.number == CABINET).expect("part in the project");
    assert!(part.accessories.iter().all(|a| prj.parts.iter().any(|p| &p.number == a)));
    // Dropped again outside the rooms: a second enclosure of its own.
    let second = ui.drop_part(&mut prj, &db, CABINET, None).unwrap();
    assert_eq!((prj.layout.enclosures[second].name.as_str(), panel::room_of(&prj.layout, second)), ("+CAB2", None));
    assert_eq!(prj.parts.iter().filter(|p| p.number == CABINET).count(), 1);
}

#[test]
fn other_parts_are_not_dropped_and_unknown_ones_are_fetched() {
    let (mut prj, db) = (Project::default(), PartsDb::default());
    let mut ui = PanelUi::default();
    let breaker = parts::catalog().iter().find(|p| !parts::is_enclosure(p)).unwrap().number.clone();
    assert!(ui.drop_part(&mut prj, &db, &breaker, None).is_err());
    ui.part_dropped(&mut prj, &db, &breaker, None);
    assert!(prj.layout.enclosures.is_empty());
    assert!(!ui.status.is_empty());
    // Not in any parts list: the application asks the parts server for it.
    ui.part_dropped(&mut prj, &db, "XYZ.UNKNOWN", Some(3));
    assert_eq!(ui.fetch_part, Some(PartDrop { number: "XYZ.UNKNOWN".into(), room: Some(3) }));
    assert!(prj.layout.enclosures.is_empty());
}

#[test]
fn the_dialog_takes_the_room_and_keeps_the_search() {
    let (mut prj, db) = (Project::default(), PartsDb::default());
    let mut ui = PanelUi::default();
    ui.new_room(&mut prj);
    ui.new_enclosure_dialog_in(&prj, &db, Some(0));
    let ne = ui.new_enclosure.as_mut().unwrap();
    assert_eq!((ne.name.as_str(), ne.room), ("+CAB1", Some(0)));
    ne.filter.rows = Some(7);
    ne.part = CABINET.into();
    apply_part_size(ne, &prj, &db);
    assert_eq!((ne.width, ne.height, ne.depth), (800.0, 1100.0, 215.0));
    let ne = ui.new_enclosure.take().unwrap();
    ui.enclosure_filter = Some(ne.filter.clone());
    let ei = ui.create_enclosure(&mut prj, &db, ne);
    assert_eq!(panel::room_of(&prj.layout, ei), Some(0));
    // The next dialog starts with the same search; a cabinet is no room to stand in.
    ui.new_enclosure_dialog_in(&prj, &db, Some(ei));
    let ne = ui.new_enclosure.as_ref().unwrap();
    assert_eq!((ne.filter.rows, ne.room, ne.name.as_str()), (Some(7), None, "+CAB2"));
}

#[test]
fn a_custom_size_takes_the_system_chosen() {
    let (mut prj, db) = (Project::default(), PartsDb::default());
    let mut ui = PanelUi::default();
    ui.new_enclosure_dialog(&prj, &db);
    let mut ne = ui.new_enclosure.take().unwrap();
    ne.part.clear();
    ne.filter.system = "baying".into();
    (ne.width, ne.height, ne.depth) = (800.0, 2000.0, 600.0);
    let ei = ui.create_enclosure(&mut prj, &db, ne);
    let e = &prj.layout.enclosures[ei];
    // The body of the system of that size.
    assert_eq!((e.width, e.height, e.depth, e.part.as_str()), (800.0, 2000.0, 600.0, "GEN.TS-800X2000X600"));
    assert_eq!(e.system().map(|s| s.id()), Some("baying"));
}
