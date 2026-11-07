use super::*;
use crate::panel::buildings::{add_building, add_level};

/// A building with three levels of 2800 mm and its level ids.
fn building() -> (Project, usize, Vec<u64>) {
    let mut prj = Project::default();
    let bi = add_building(&mut prj, "+B1").unwrap();
    for _ in 0..2 {
        add_level(&mut prj, bi, None, None, Some(2800.0)).unwrap();
    }
    prj.layout.buildings[bi].levels[0].height = 2800.0;
    let b = &mut prj.layout.buildings[bi];
    for (k, l) in b.levels.iter_mut().enumerate() {
        l.elevation = 2800.0 * k as f32;
    }
    let ids = b.levels.iter().map(|l| l.id).collect();
    (prj, bi, ids)
}

fn stair(prj: &Project, bi: usize, kind: StairKind) -> StairGeometry {
    let b = &prj.layout.buildings[bi];
    let s = Stair { kind, bottom: b.levels[0].id, top: b.levels[1].id, ..Default::default() };
    geometry(b, &s).unwrap()
}

#[test]
fn steps_rise_evenly_from_floor_to_floor() {
    let (prj, bi, _) = building();
    let g = stair(&prj, bi, StairKind::Straight);
    // 2800 mm in 16 risers of 175 mm, 15 treads; the last riser leads onto the floor.
    assert_eq!((g.risers, g.rise), (16, 175.0));
    assert_eq!(g.pieces.len(), 15);
    assert_eq!(g.pieces.last().unwrap().top, 2800.0 - 175.0);
    // Straight up along +x, 270 mm per tread, 1 m wide about the foot.
    let (lo, hi) = plan_bounds(&g);
    assert_eq!((lo, hi), ([0.0, -500.0], [15.0 * 270.0, 500.0]));
    assert!(problems(&prj).is_empty());
}

#[test]
fn landings_turn_the_flights() {
    let (prj, bi, _) = building();
    for (kind, landings) in [(StairKind::QuarterLanding, 1), (StairKind::TwoQuarterLandings, 2), (StairKind::HalfLanding, 1)] {
        let g = stair(&prj, bi, kind);
        assert_eq!(g.pieces.iter().filter(|p| p.landing).count(), landings, "{kind:?}");
        assert_eq!(g.pieces.len(), 15, "{kind:?}: every tread a step or a landing");
        // The tops rise one riser after the other.
        for (k, p) in g.pieces.iter().enumerate() {
            assert_eq!(p.top, 175.0 * (k + 1) as f32, "{kind:?} {k}");
        }
    }
    // A left turn goes towards -z (the rear), the half landing runs back towards the foot.
    let l = stair(&prj, bi, StairKind::QuarterLanding);
    assert!(l.walk.last().unwrap()[1] < -1000.0, "{:?}", l.walk);
    let u = stair(&prj, bi, StairKind::HalfLanding);
    let (lo, hi) = plan_bounds(&u);
    assert!(hi[0] - lo[0] < 8.0 * 270.0 + 1000.0 + 1.0 && u.walk.last().unwrap()[0] < 300.0, "{lo:?} {hi:?} {:?}", u.walk);
    assert!((hi[1] - lo[1] - (2000.0 + WELL)).abs() < 1.0, "two flights and the well side by side");
}

#[test]
fn right_turns_and_placement_mirror_and_move_the_stair() {
    let (prj, bi, _) = building();
    let b = &prj.layout.buildings[bi];
    let base = Stair { kind: StairKind::QuarterLanding, bottom: b.levels[0].id, top: b.levels[1].id, ..Default::default() };
    let left = geometry(b, &base).unwrap();
    let right = geometry(b, &Stair { turn: Turn::Right, ..base.clone() }).unwrap();
    assert!(right.walk.last().unwrap()[1] > 1000.0);
    assert_eq!(left.walk.last().unwrap()[1], -right.walk.last().unwrap()[1]);
    // Turned by 90° the first flight runs towards -z; moved, everything moves.
    let turned = geometry(b, &Stair { angle: 90.0, x: 1000.0, z: 2000.0, kind: StairKind::Straight, ..base }).unwrap();
    let end = turned.walk.last().unwrap();
    assert!((end[0] - 1000.0).abs() < 0.1 && (end[1] - (2000.0 - 15.0 * 270.0)).abs() < 0.1, "{end:?}");
}

#[test]
fn spiral_stairs_wind_round_their_newel() {
    let (prj, bi, _) = building();
    let g = stair(&prj, bi, StairKind::Spiral);
    // 15 wedges and the newel from floor to floor.
    assert_eq!(g.pieces.len(), 16);
    let newel = g.pieces.last().unwrap();
    assert_eq!((newel.bottom, newel.top), (0.0, 2800.0));
    let centre = [0.0, -(NEWEL + 500.0)];
    for p in &g.pieces[..15] {
        for q in &p.outline {
            let r = (q[0] - centre[0]).hypot(q[1] - centre[1]);
            assert!(r > NEWEL - 0.5 && r < NEWEL + 1000.0 + 0.5, "{r}");
        }
    }
}

#[test]
fn stairs_need_a_level_above_and_good_steps() {
    let (mut prj, bi, ids) = building();
    let id = add_stair(&mut prj, bi, StairKind::Straight, 0.0, 0.0, ids[0], None).unwrap();
    let s = prj.layout.buildings[bi].stairs.iter().find(|s| s.id == id).unwrap();
    assert_eq!((s.top, s.name.as_str()), (ids[1], "ST1"));
    assert!(add_stair(&mut prj, bi, StairKind::Straight, 0.0, 0.0, ids[2], None).is_err(), "nothing above the top level");
    assert!(add_stair(&mut prj, bi, StairKind::Straight, 0.0, 0.0, ids[1], Some(ids[0])).is_err(), "down is not up");
    // A short going breaks the step rule; a small difference in height gives low steps.
    prj.layout.buildings[bi].stairs[0].tread = 180.0;
    let texts: Vec<String> = problems(&prj).iter().map(|i| i.message()).collect();
    assert!(texts.len() == 1 && texts[0].contains("step rule"), "{texts:?}");
    prj.layout.buildings[bi].stairs[0].tread = 270.0;
    prj.layout.buildings[bi].levels[1].elevation = 260.0;
    let texts: Vec<String> = problems(&prj).iter().map(|i| i.message()).collect();
    assert!(texts.iter().any(|t| t.contains("rise of")), "{texts:?}");
    // On the levels they join, not on others.
    prj.layout.buildings[bi].levels[1].elevation = 2800.0;
    assert_eq!(stairs_at_level(&prj, ids[1]).len(), 1);
    assert!(stairs_at_level(&prj, ids[2]).is_empty());
}
