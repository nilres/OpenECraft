use super::*;
use crate::model::panel::{Roof, RoofShape};
use crate::panel::buildings::add_building;
use crate::panel::site_poses;

/// A building with a gable roof over 30 × 12 m of outer walls.
fn building() -> (Project, usize) {
    let mut prj = Project::default();
    let bi = add_building(&mut prj, "+B1").unwrap();
    prj.layout.buildings[bi].roofs.push(Roof { name: "R1".into(), shape: RoofShape::Gable, x: 0.0, z: 0.0, width: 30_000.0, depth: 12_000.0, eaves: 3000.0, pitch: 30.0, ..Default::default() });
    (prj, bi)
}

#[test]
fn a_foundation_earth_electrode_runs_inside_the_walls_in_meshes() {
    let (mut prj, bi) = building();
    let id = add_electrode(&mut prj, bi, ElectrodeKind::Foundation, None).unwrap();
    let poses = site_poses(&prj);
    let e = prj.layout.buildings[bi].electrodes.iter().find(|e| e.id == id).unwrap().clone();
    assert_eq!((e.name.as_str(), e.distance, e.mesh), ("E1", 100.0, 20_000.0));
    let lines = paths(&prj, &poses, bi, &e);
    // The ring 100 mm inside the walls, one strip across: the 29.8 m are two meshes.
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0][0], [100.0, -SLAB + COVER, 100.0]);
    assert!((length(&lines[..1]) - 2.0 * (29_800.0 + 11_800.0)).abs() < 1.0);
    assert!((lines[1][0][0] - 15_000.0).abs() < 1.0 && (lines[1][0][2] - 100.0).abs() < 1.0);
    // Its material: strip, spacers every 2 m and the two crossings of the strip.
    let m = material(&prj, &poses, bi);
    let qty = |p: &str| m.iter().find(|x| x.0 == p).map(|x| x.1);
    assert_eq!(qty("GEN.ER-STRIP-FE30"), Some(95.0));
    assert_eq!((qty("GEN.ER-SPACER"), qty("GEN.ER-CROSS"), qty("GEN.FE-LUG-V4A-10")), (Some(48.0), Some(2.0), None));
    assert!(problems(&prj).is_empty());
    // Connected to a busbar that is missing: reported, a lug for it.
    prj.layout.buildings[bi].electrodes[0].busbar = "=X1".into();
    assert_eq!(problems(&prj).len(), 1);
    assert_eq!(material(&prj, &poses, bi).iter().find(|x| x.0 == "GEN.FE-LUG-V4A-10").map(|x| x.1), Some(1.0));
}

#[test]
fn ring_electrodes_lie_round_the_walls_and_rods_outside_them() {
    let (mut prj, bi) = building();
    add_electrode(&mut prj, bi, ElectrodeKind::Ring, None).unwrap();
    add_electrode(&mut prj, bi, ElectrodeKind::Rod, None).unwrap();
    let poses = site_poses(&prj);
    let b = &prj.layout.buildings[bi];
    let ring = paths(&prj, &poses, bi, &b.electrodes[0]);
    assert_eq!(ring[0][0], [-1000.0, -500.0, -1000.0], "a metre outside, half a metre deep");
    assert!((length(&ring) - 2.0 * (32_000.0 + 14_000.0)).abs() < 1.0);
    // The rod at the rear left corner, 3 m long from half a metre down: two sections.
    let rod = paths(&prj, &poses, bi, &b.electrodes[1]);
    assert_eq!(rod, vec![vec![[-1000.0, -500.0, -1000.0], [-1000.0, -3500.0, -1000.0]]]);
    let m = material(&prj, &poses, bi);
    assert!(m.contains(&("GEN.ER-ROD-1500".into(), 2.0)) && m.contains(&("GEN.ER-ROD-TIP".into(), 1.0)));
    assert!(m.contains(&("GEN.ER-STRIP-V4A30".into(), 92.0)));
    assert!(problems(&prj).is_empty());
    // A rod inside the walls is reported.
    prj.layout.buildings[bi].electrodes[1].x = 5000.0;
    prj.layout.buildings[bi].electrodes[1].z = 5000.0;
    assert_eq!(problems(&prj).len(), 1);
    // Without walls a ring has nowhere to run.
    prj.layout.buildings[bi].roofs.clear();
    assert!(problems(&prj).iter().any(|i| i.message().contains("no walls")));
}

#[test]
fn an_electrode_is_led_up_to_its_busbar() {
    let (mut prj, bi) = building();
    add_electrode(&mut prj, bi, ElectrodeKind::Foundation, None).unwrap();
    let level = prj.layout.buildings[bi].levels[0].id;
    let ri = crate::panel::buildings::add_room(&mut prj, "+R1", Some(level)).unwrap();
    {
        let e = &mut prj.layout.enclosures[ri];
        (e.width, e.depth) = (8000.0, 6000.0);
        e.site = Some(crate::panel::SitePlace { x: 1000.0, z: 1000.0, angle: 0.0 });
    }
    let mut d = crate::model::DeviceData::default();
    d.parts.push(crate::parts::PartRef { number: "GEN.MET-BAR-10".into(), count: 1, total: false });
    prj.devices.insert("=X1".into(), d);
    prj.parts.push(crate::parts::catalog().iter().find(|p| p.number == "GEN.MET-BAR-10").unwrap().clone());
    let h = prj.layout.enclosures[ri].height;
    crate::panel::place_device_on(&mut prj, ri, "=X1", crate::geom::p(1000.0, h - 600.0), crate::model::panel::Face::Wall(0)).unwrap();
    prj.layout.buildings[bi].electrodes[0].busbar = "=X1".into();
    let poses = site_poses(&prj);
    let e = &prj.layout.buildings[bi].electrodes[0];
    let lead = busbar_lead(&prj, &poses, bi, e).unwrap();
    assert_eq!(lead.len(), 3);
    assert!((lead[0][1] - (-SLAB + COVER)).abs() < 0.5, "from the electrode");
    assert!(lead[2][1] > 300.0 && lead[2][1] < 700.0, "up to the bottom of the busbar: {lead:?}");
    assert!(problems(&prj).is_empty());
}
