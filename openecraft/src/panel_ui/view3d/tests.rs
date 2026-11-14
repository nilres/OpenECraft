use super::*;

#[test]
fn a_click_into_the_air_clears_the_selected_cable() {
    let prj = Project::default();
    let an = Analysis::build(&prj);
    let mut ui = PanelUi { selected_route: Some("=MD-W11".into()), selected_wire: Some(("-K1:A1".into(), "-K2:A1".into())), ..Default::default() };
    ui.selection.insert(7);
    assert!(!ui.pick_3d(&prj, &an, &Scene3d::default(), &[], &|_| None, Pos2::ZERO));
    assert!(ui.selected_route.is_none() && ui.selected_wire.is_none() && ui.selection.is_empty());
}

#[test]
fn selecting_a_device_or_wire_drops_the_selected_cable() {
    let mut ui = PanelUi { selected_route: Some("=MD-W11".into()), ..Default::default() };
    ui.select_item(7);
    assert!(ui.selected_route.is_none());
    ui.selected_route = Some("=MD-W11".into());
    ui.select_wire(("-K1:A1".into(), "-K2:A1".into()));
    assert!(ui.selected_route.is_none() && ui.selection.is_empty());
}

#[test]
fn a_cable_in_front_of_a_device_is_picked_one_behind_it_not() {
    use openecraft_core::panel::{Enclosure, PanelItem};
    let mut prj = Project::default();
    prj.layout.enclosures.push(Enclosure { items: vec![PanelItem { id: 7, kind: ItemKind::Device, ..Default::default() }], ..Default::default() });
    let an = Analysis::build(&prj);
    // A device face around the pointer, 1000 mm away; a cable across it in front or behind.
    let pts = [Pos2::new(0.0, 0.0), Pos2::new(100.0, 0.0), Pos2::new(100.0, 100.0), Pos2::new(0.0, 100.0)];
    let prims = [Prim::Face { pts, z: [1000.0; 4], n: 4, color: Color32::GRAY, item: Some((0, 7)), outline: false, pv: None }];
    let view = |q: V3| Some((Pos2::new(q[0], q[1]), q[2]));
    for (depth, cable) in [(900.0, true), (1100.0, false)] {
        let mut scene = Scene3d::default();
        scene.picks.push(([0.0, 50.0, depth], [100.0, 50.0, depth], RouteRef::Item("=W1".into())));
        let mut ui = PanelUi::default();
        ui.pick_3d(&prj, &an, &scene, &prims, &view, Pos2::new(50.0, 51.0));
        assert_eq!(ui.selected_route.is_some(), cable, "{depth}");
        assert_eq!(ui.selection.contains(&7), !cable, "{depth}");
    }
}

#[test]
fn a_pv_array_clicked_is_selected_with_its_properties() {
    let prj = openecraft_core::sample::example("office_building").unwrap();
    let an = Analysis::build(&prj);
    let id = prj.layout.buildings.iter().flat_map(|b| &b.roofs).flat_map(|r| &r.solar).next().unwrap().id;
    let pts = [Pos2::new(0.0, 0.0), Pos2::new(100.0, 0.0), Pos2::new(100.0, 100.0), Pos2::new(0.0, 100.0)];
    let prims = [Prim::Face { pts, z: [1000.0; 4], n: 4, color: Color32::GRAY, item: None, outline: false, pv: Some(id) }];
    let mut ui = PanelUi::default();
    ui.selection.insert(7);
    assert!(ui.pick_3d(&prj, &an, &Scene3d::default(), &prims, &|_| None, Pos2::new(50.0, 50.0)));
    assert_eq!(ui.selected_array, Some(id));
    assert!(ui.selection.is_empty() && ui.status.contains("PV1") && ui.status.contains("kWp"), "{}", ui.status);
    // A click into the air or on an item drops it.
    ui.pick_3d(&prj, &an, &Scene3d::default(), &[], &|_| None, Pos2::new(50.0, 50.0));
    assert_eq!(ui.selected_array, None);
    ui.selected_array = Some(id);
    ui.select_item(7);
    assert_eq!(ui.selected_array, None);
    // In front of an item under the roof drawn after it (an outlet in the attic below).
    let behind = Prim::Face { pts, z: [2000.0; 4], n: 4, color: Color32::GRAY, item: Some((0, 1_234_567)), outline: false, pv: None };
    let both = [Prim::Face { pts, z: [1000.0; 4], n: 4, color: Color32::GRAY, item: None, outline: false, pv: Some(id) }, behind];
    ui.pick_3d(&prj, &an, &Scene3d::default(), &both, &|_| None, Pos2::new(50.0, 50.0));
    assert_eq!(ui.selected_array, Some(id));
}

#[test]
fn a_device_clicked_in_another_cabinet_shows_that_cabinet_fitted() {
    use openecraft_core::panel::PanelItem;
    let mut prj = openecraft_core::sample::example("building_services").unwrap();
    let room = prj.layout.enclosures.iter().position(|e| e.is_room()).unwrap();
    let cabinet = panel::cabinets_in(&prj.layout, room)[0];
    prj.layout.enclosures[cabinet].items.push(PanelItem { id: 9_999, kind: ItemKind::Device, ..Default::default() });
    let an = Analysis::build(&prj);
    let pts = [Pos2::new(0.0, 0.0), Pos2::new(100.0, 0.0), Pos2::new(100.0, 100.0), Pos2::new(0.0, 100.0)];
    let prims = [Prim::Face { pts, z: [1000.0; 4], n: 4, color: Color32::GRAY, item: Some((cabinet, 9_999)), outline: false, pv: None }];
    let mut ui = PanelUi { enclosure: room, fit_pending: false, ..Default::default() };
    ui.cam = Some(default_cam(&prj, &ui.shown_3d(&prj, room)));
    ui.pick_3d(&prj, &an, &Scene3d::default(), &prims, &|_| None, Pos2::new(50.0, 50.0));
    assert_eq!(ui.enclosure, cabinet);
    assert!(ui.selection.contains(&9_999) && ui.cam.is_none() && ui.fit_pending);
    // A device of the cabinet shown keeps the camera.
    ui.cam = Some(default_cam(&prj, &ui.shown_3d(&prj, cabinet)));
    ui.fit_pending = false;
    ui.pick_3d(&prj, &an, &Scene3d::default(), &prims, &|_| None, Pos2::new(50.0, 50.0));
    assert!(ui.cam.is_some() && !ui.fit_pending);
}

#[test]
fn earth_electrodes_and_lightning_protection_are_picked_in_3d() {
    let prj = openecraft_core::sample::example("office_building").unwrap();
    let an = Analysis::build(&prj);
    let bi = prj.layout.buildings.iter().position(|b| b.name == "+B3").unwrap();
    let id = prj.layout.buildings[bi].electrodes[0].id;
    let mut scene = Scene3d::default();
    scene.earthing_picks.push(([0.0, 50.0, 900.0], [100.0, 50.0, 900.0], (bi, panel::lightning::Owner::Electrode(id))));
    let view = |q: V3| Some((Pos2::new(q[0], q[1]), q[2]));
    let mut ui = PanelUi::default();
    assert!(ui.pick_3d(&prj, &an, &scene, &[], &view, Pos2::new(50.0, 52.0)));
    assert_eq!(ui.selected_earthing, Some((bi, panel::lightning::Owner::Electrode(id))));
    assert!(ui.status.contains("E1") && ui.status.contains("+B3"), "{}", ui.status);
    // Too far from it: nothing.
    ui.pick_3d(&prj, &an, &scene, &[], &view, Pos2::new(50.0, 80.0));
    assert_eq!(ui.selected_earthing, None);
}

#[test]
fn a_roof_clicked_is_selected_with_the_point_on_it() {
    let mut prj = Project::default();
    let bi = panel::buildings::add_building(&mut prj, "+B1").unwrap();
    prj.layout.buildings[bi].roofs.push(panel::Roof { id: 5, name: "R1".into(), ..Default::default() });
    let an = Analysis::build(&prj);
    // The top of the roof as a square in view, 1000 mm away, over 0 … 10 m of the
    // building's floor plan in x and z.
    let mut scene = Scene3d::default();
    let plan = vec![[0.0, 3000.0, 0.0], [10_000.0, 3000.0, 0.0], [10_000.0, 3000.0, 10_000.0], [0.0, 3000.0, 10_000.0]];
    scene.roof_picks.push(super::super::scene3d::RoofPick { pts: vec![[0.0, 0.0, 1000.0], [100.0, 0.0, 1000.0], [100.0, 100.0, 1000.0], [0.0, 100.0, 1000.0]], plan, building: bi, roof: 5 });
    let view = |q: V3| Some((Pos2::new(q[0], q[1]), q[2]));
    let mut ui = PanelUi { selected_array: Some(9), ..Default::default() };
    assert!(ui.pick_3d(&prj, &an, &scene, &[], &view, Pos2::new(25.0, 75.0)));
    assert_eq!(ui.selected_roof, Some(RoofTarget { building: bi, roof: 5, at: [2500.0, 7500.0] }));
    assert!(ui.selected_array.is_none() && ui.open_properties && ui.status.contains("R1"), "{}", ui.status);
    // Beside it: nothing picked.
    ui.pick_3d(&prj, &an, &scene, &[], &view, Pos2::new(150.0, 50.0));
    assert!(ui.selected_roof.is_none());
}
