use super::*;
use crate::model::panel::Opening;

/// A room of 4 × 3 m, 2.7 m high, with a door in its front wall (wall 3, from 1 m).
fn room() -> (Project, usize) {
    let mut prj = Project::default();
    let r = panel::new_room(&mut prj, "+R1", 4000.0, 3000.0, 2700.0);
    prj.layout.enclosures.push(r);
    panel::add_opening(&mut prj, 0, Opening::Door, Face::Wall(2), Some(1000.0), None, None).unwrap();
    (prj, 0)
}

/// A point on wall `w` at face point (x, y), `out` in front of it.
fn on_wall(prj: &Project, w: u16, x: f32, y: f32, out: f32) -> V3 {
    panel::face_point(&prj.layout.enclosures[0], Face::Wall(w), [x, y, out])
}

#[test]
fn the_german_zones_lie_on_the_walls() {
    let (prj, ri) = room();
    let e = &prj.layout.enclosures[ri];
    let rules = RoutingRules::default();
    // The rear wall: below the ceiling and above the floor (the middle zone of kitchens
    // is off), beside both corners.
    let rear = wall_zones(e, Face::Wall(0), &rules);
    let ys: Vec<f32> = rear.horizontal.iter().map(|h| h.at).collect();
    assert_eq!(ys, vec![300.0, 2400.0]);
    assert!(rear.horizontal.iter().all(|h| h.stretches == vec![(0.0, 4000.0)] && h.half == 150.0));
    let xs: Vec<f32> = rear.vertical.iter().map(|v| v.at).collect();
    assert_eq!(xs, vec![200.0, 3800.0]);
    // The front wall: the lower zone stops at the door, vertical zones beside it.
    let front = wall_zones(e, Face::Wall(2), &rules);
    let door = e.items.iter().find(|i| i.opening == Opening::Door).unwrap();
    let low = front.horizontal.iter().find(|h| h.at == 2400.0).unwrap();
    assert_eq!(low.stretches, vec![(0.0, door.x), (door.x + door.w, 4000.0)]);
    let xs: Vec<f32> = front.vertical.iter().map(|v| v.at).collect();
    assert!(xs.contains(&(door.x - 200.0)) && xs.contains(&(door.x + door.w + 200.0)), "{xs:?}");
    // Bands to show: on the walls, in the floor and the ceiling.
    let bands = bands(&prj, ri, &rules);
    assert!(bands.iter().any(|b| rules.zones[b.zone].kind == ZoneKind::Floor && b.corners.iter().all(|q| (q[1] - 2699.0).abs() < 0.01)));
    assert!(bands.iter().any(|b| rules.zones[b.zone].kind == ZoneKind::Ceiling));
}

#[test]
fn outlets_on_one_wall_join_in_their_zone_behind_the_surface() {
    let (prj, ri) = room();
    let rules = RoutingRules::default();
    // Two socket outlets 300 mm above the floor (in the lower zone), 30 mm in front of the wall.
    let (a, b) = (on_wall(&prj, 0, 1000.0, 2400.0, 30.0), on_wall(&prj, 0, 3000.0, 2400.0, 30.0));
    let way = route(&prj, ri, a, b, Laying::Wall, 7.0, &rules).expect("in the zone");
    assert_eq!((way[0], *way.last().unwrap()), (a, b));
    // Between the ends in the wall, 30 mm deep less half the thickness.
    let depth = -(30.0 - 3.5);
    assert!(way[1..way.len() - 1].iter().all(|q| (q[2] - depth).abs() < 0.01 && (q[1] - 2400.0).abs() < 0.01), "{way:?}");
    let len: f32 = way.windows(2).map(|w| dist(w[0], w[1])).sum();
    assert!((len - (2000.0 + 2.0 * 56.5)).abs() < 0.5, "{len}");
    // A switch at 1050 mm is fed from the zone below the ceiling or above the floor:
    // straight up or down from it.
    let s = on_wall(&prj, 0, 2000.0, 2700.0 - 1050.0, 20.0);
    let way = route(&prj, ri, a, s, Laying::Wall, 7.0, &rules).unwrap();
    assert!(way.windows(2).all(|w| (w[0][0] - w[1][0]).abs() < 0.01 || (w[0][1] - w[1][1]).abs() < 0.01), "only horizontal and vertical: {way:?}");
}

#[test]
fn routes_go_round_the_corners_and_through_the_floor() {
    let (prj, ri) = room();
    let mut rules = RoutingRules::default();
    let a = on_wall(&prj, 0, 3500.0, 2400.0, 30.0);
    // On the right wall: round the rear right corner in the lower zone.
    let b = on_wall(&prj, 1, 1000.0, 2400.0, 30.0);
    let way = route(&prj, ri, a, b, Laying::Wall, 7.0, &rules).unwrap();
    let corner = way.iter().any(|q| q[0] > 4000.0 && q[2] < 0.0);
    assert!(corner, "behind the corner: {way:?}");
    // To the opposite wall through the floor: down into it and along its zone.
    let c = on_wall(&prj, 2, 3200.0, 2400.0, 30.0);
    let floor = route(&prj, ri, a, c, Laying::Floor, 7.0, &rules).unwrap();
    assert!(floor.iter().any(|q| (q[1] - (2700.0 + 60.0 - 3.5)).abs() < 0.01), "in the floor: {floor:?}");
    // Floor-standing: straight down into the floor.
    let pump = [2000.0, 2200.0, 1500.0];
    let way = route(&prj, ri, a, pump, Laying::Floor, 7.0, &rules).unwrap();
    assert_eq!(way[way.len() - 2], [2000.0, 2700.0 + 60.0 - 3.5, 1500.0]);
    // Without zones there is no way.
    for z in &mut rules.zones {
        z.on = false;
    }
    assert!(route(&prj, ri, a, b, Laying::Wall, 7.0, &rules).is_none());
    assert!(route(&prj, ri, a, c, Laying::Floor, 7.0, &rules).is_none());
}
