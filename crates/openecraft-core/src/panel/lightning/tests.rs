use super::*;
use crate::model::panel::{RoofShape, Roof};
use crate::panel::buildings::add_building;
use crate::panel::site_poses;

/// A building of 30 × 12 m outer walls under a roof of `shape` with a ring earth
/// electrode and a lightning protection system of class III.
fn building(shape: RoofShape) -> (Project, usize) {
    let mut prj = Project::default();
    let bi = add_building(&mut prj, "+B1").unwrap();
    let pitch = if shape == RoofShape::Flat { 0.0 } else { 30.0 };
    prj.layout.buildings[bi].roofs.push(Roof { name: "R1".into(), shape, x: 0.0, z: 0.0, width: 30_000.0, depth: 12_000.0, eaves: 6000.0, pitch, overhang: 400.0, thickness: 300.0, ..Default::default() });
    earthing::add_electrode(&mut prj, bi, ElectrodeKind::Ring, None).unwrap();
    prj.layout.buildings[bi].lightning = Some(Lightning::default());
    (prj, bi)
}

#[test]
fn the_ridge_carries_the_air_termination_conductor_and_downs_go_round_the_walls() {
    let (prj, bi) = building(RoofShape::Gable);
    let poses = site_poses(&prj);
    let g = geometry(&prj, &poses, bi).unwrap();
    // The ridge along x at z = 6 m, 50 mm over the covering.
    let air: Vec<&LpsLine> = g.lines.iter().filter(|l| l.kind == LpsKind::AirTermination).collect();
    assert_eq!(air.len(), 1);
    let r = &prj.layout.buildings[bi].roofs[0];
    assert!(air[0].points.iter().all(|q| (q[2] - 6000.0).abs() < 1.0 && (q[1] - roofs::covering_at(r, [q[0], q[2]]).unwrap() - CLEARANCE).abs() < 0.5));
    assert!((g.length(LpsKind::AirTermination) - 30_800.0).abs() < 1.0, "with the overhang");
    // 84 m of walls at most 15 m apart: six down conductors, each with its test joint
    // and its earth entry to the ring.
    assert_eq!(needed_downs(3, 84_000.0), 6);
    assert_eq!((g.count(LpsKind::Down), g.joints.len(), g.count(LpsKind::Entry), g.count(LpsKind::Roof)), (6, 6, 6, 6));
    let ring = earthing::paths(&prj, &poses, bi, &prj.layout.buildings[bi].electrodes[0]);
    for l in g.lines.iter().filter(|l| l.kind == LpsKind::Entry) {
        let end = *l.points.last().unwrap();
        assert!(dist(earthing::nearest(&ring, end).unwrap(), end) < 1.0, "on the ring");
    }
    // Each roof conductor ends on the ridge.
    for l in g.lines.iter().filter(|l| l.kind == LpsKind::Roof) {
        assert!((l.points.last().unwrap()[2] - 6000.0).abs() < 1.0, "{:?}", l.points);
    }
    // Under the overhang down to the test joint a metre above the ground.
    let first = g.lines.iter().find(|l| l.kind == LpsKind::Down).unwrap();
    assert_eq!(first.points[1][1], TEST_JOINT);
    assert!(first.points[0][1] > 5500.0 && first.points[0][2] < 0.0, "{:?}", first.points);
    let m = material(&prj, &poses, bi);
    for p in ["GEN.LP-WIRE-AL8", "GEN.LP-HOLDER-ROOF", "GEN.LP-HOLDER-WALL", "GEN.LP-TEST-JOINT", "GEN.LP-EARTH-ENTRY", "GEN.LP-CLAMP-MV"] {
        assert!(m.iter().any(|x| x.0 == p && x.1 > 0.0), "{p}: {m:?}");
    }
    assert!(m.contains(&("GEN.LP-TEST-JOINT".into(), 6.0)));
    assert!(problems(&prj).is_empty(), "{:?}", problems(&prj));
}

#[test]
fn flat_roofs_get_an_edge_and_a_mesh() {
    let (prj, bi) = building(RoofShape::Flat);
    let poses = site_poses(&prj);
    let g = geometry(&prj, &poses, bi).unwrap();
    // The edge and, at 15 m for class III, two lines across the 30.8 m and none across
    // the 12.8 m.
    assert_eq!(g.count(LpsKind::AirTermination), 4 + 2);
}

#[test]
fn pv_arrays_have_to_lie_in_the_protection_of_the_air_termination() {
    let (mut prj, bi) = building(RoofShape::Flat);
    // A tilted array in the middle of the flat roof, 6 m from the edge and the mesh:
    // not protected by conductors lying flat on the roof.
    let id = solar::add_array(&mut prj, bi, 0, None).unwrap();
    let poses = site_poses(&prj);
    assert_eq!(unprotected_arrays(&prj, &poses, bi), vec!["PV1".to_string()]);
    assert!(problems(&prj).iter().any(|i| i.message().contains("PV1")));
    // Air-termination rods round it protect it.
    let a = prj.layout.buildings[bi].roofs[0].solar.iter().find(|a| a.id == id).unwrap().clone();
    let outline = solar::geometry(&prj.layout.buildings[bi].roofs[0], &a).unwrap().outline;
    for q in outline {
        let rod = add_rod(&mut prj, bi, Some([q[0], q[1]])).unwrap();
        let lp = prj.layout.buildings[bi].lightning.as_mut().unwrap();
        lp.rods.iter_mut().find(|r| r.id == rod).unwrap().height = 3000.0;
    }
    assert!(unprotected_arrays(&prj, &poses, bi).is_empty());
    assert_eq!(material(&prj, &poses, bi).iter().find(|x| x.0 == "GEN.LP-ROD-AL16").map(|x| x.1), Some(4.0));
    // On a pitched roof the ridge conductor protects the slope below it.
    let (mut prj, bi) = building(RoofShape::Gable);
    solar::add_array(&mut prj, bi, 0, None).unwrap();
    assert!(unprotected_arrays(&prj, &site_poses(&prj), bi).is_empty());
}

#[test]
fn missing_electrodes_and_too_few_downs_are_reported() {
    let (mut prj, bi) = building(RoofShape::Gable);
    prj.layout.buildings[bi].electrodes.clear();
    let lp = prj.layout.buildings[bi].lightning.as_mut().unwrap();
    lp.down_conductors = 3;
    let texts: Vec<String> = problems(&prj).iter().map(|i| i.message()).collect();
    assert!(texts.iter().any(|t| t.contains("no earth electrode")), "{texts:?}");
    assert!(texts.iter().any(|t| t.contains("3 down conductors, class III needs 6")), "{texts:?}");
    // The protection angle: 76° up to 2 m for class III, 23° at 45 m, none above.
    assert_eq!(protection_angle(3, 1500.0), Some(76.0));
    assert!((protection_angle(3, 45_000.0).unwrap() - 23.0).abs() < 1e-3);
    assert_eq!(protection_angle(3, 46_000.0), None);
    // A rod off the roofs.
    add_rod(&mut prj, bi, Some([50_000.0, 0.0])).unwrap();
    assert!(problems(&prj).iter().any(|i| i.message().contains("off the roofs")));
}

#[test]
fn conductors_rods_and_holders_follow_their_material_and_mounting() {
    let (mut prj, bi) = building(RoofShape::Gable);
    let id = add_rod(&mut prj, bi, Some([15_000.0, 6000.0])).unwrap();
    let poses = site_poses(&prj);
    let qty = |prj: &Project, part: &str| material(prj, &poses, bi).iter().find(|x| x.0 == part).map(|x| x.1);
    assert_eq!((qty(&prj, "GEN.LP-WIRE-AL8").is_some(), qty(&prj, "GEN.LP-ROD-AL16"), qty(&prj, "GEN.LP-ROD-HOLDER")), (true, Some(1.0), Some(1.0)));
    let roof_holders = qty(&prj, "GEN.LP-HOLDER-ROOF").unwrap();
    {
        let lp = prj.layout.buildings[bi].lightning.as_mut().unwrap();
        (lp.material, lp.diameter, lp.holder_spacing) = (Some(Material::Copper), 8.0, 500.0);
        lp.roof_holder = "GEN.LP-HOLDER-RIDGE".into();
        let r = lp.rods.iter_mut().find(|r| r.id == id).unwrap();
        (r.material, r.base) = (Some(Material::StainlessSteel), RodBase::ConcreteBase);
    }
    assert!(qty(&prj, "GEN.LP-WIRE-CU8").is_some() && qty(&prj, "GEN.LP-WIRE-AL8").is_none());
    assert_eq!((qty(&prj, "GEN.LP-ROD-V4A16"), qty(&prj, "GEN.LP-ROD-BASE"), qty(&prj, "GEN.LP-HOLDER-ROOF")), (Some(1.0), Some(1.0), None));
    // Twice as many holders at half the distance.
    let ridge = qty(&prj, "GEN.LP-HOLDER-RIDGE").unwrap();
    assert!((ridge - 2.0 * roof_holders).abs() <= 2.0, "{ridge} {roof_holders}");
    // The lines know what they belong to: the rod and its lead, the conductors.
    let lines = plant_lines(&prj, &poses, bi);
    assert_eq!(lines.iter().filter(|l| l.owner == Owner::AirRod(id)).count(), 2);
    assert!(lines.iter().any(|l| l.owner == Owner::Lightning && l.what == Line::Lps(LpsKind::Down) && (l.diameter - 8.0).abs() < 0.01));
}
