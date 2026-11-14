use super::*;

#[test]
fn points_inside_an_l_shaped_room() {
    let l = [pos2(0.0, 0.0), pos2(40.0, 0.0), pos2(40.0, 20.0), pos2(20.0, 20.0), pos2(20.0, 40.0), pos2(0.0, 40.0)];
    assert!(inside(&l, pos2(10.0, 30.0)));
    assert!(inside(&l, pos2(30.0, 10.0)));
    assert!(!inside(&l, pos2(30.0, 30.0)), "in the notch");
    assert!(!inside(&l, pos2(-5.0, 10.0)));
}

#[test]
fn earth_electrodes_and_lightning_protection_are_found_in_the_site_plan() {
    let prj = openecraft_core::sample::example("office_building").unwrap();
    let bi = prj.layout.buildings.iter().position(|b| b.name == "+B3").unwrap();
    let poses = panel::site_poses(&prj);
    let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0));
    let frame = frame(rect, &prj, &SitePlan::default());
    // On the air-termination rod: the rod.
    let lp = prj.layout.buildings[bi].lightning.as_ref().unwrap();
    let rod = &lp.rods[0];
    let at = panel::buildings::building_pose(&prj.layout.buildings[bi]).apply([rod.x, 0.0, rod.z]);
    assert_eq!(earthing_at(&prj, &poses, &frame, frame.at([at[0], at[2]])), Some((bi, panel::lightning::Owner::AirRod(rod.id))));
    // Far from everything: nothing.
    assert_eq!(earthing_at(&prj, &poses, &frame, egui::pos2(-500.0, -500.0)), None);
}
