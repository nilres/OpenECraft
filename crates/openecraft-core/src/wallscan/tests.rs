use super::*;

/// A drawn floor plan at 10 mm per pixel: a house of 4.4 × 2.4 m inside with walls of
/// 300 mm, split by an inner wall of 120 mm with a door of 900 mm (its swing drawn
/// into the right room, hinged at the top), a window of 1200 mm in the top wall (two
/// lines through it), a dimension line and some text.
fn plan() -> Gray {
    let mut img = Gray::new(600, 400, 240);
    let wall = 170;
    // Outer walls.
    img.fill([50, 50], [550, 80], wall);
    img.fill([50, 320], [550, 350], wall);
    img.fill([50, 50], [80, 350], wall);
    img.fill([520, 50], [550, 350], wall);
    // The window: a gap with its glass lines.
    img.fill([120, 50], [240, 80], 240);
    img.fill([120, 60], [240, 62], 60);
    img.fill([120, 68], [240, 70], 60);
    // The inner wall with its door.
    img.fill([290, 80], [302, 320], wall);
    img.fill([290, 180], [302, 270], 240);
    // The swing: a quarter circle from the hinge at the top of the door.
    for i in 0..=200 {
        let t = std::f32::consts::FRAC_PI_2 * i as f32 / 200.0;
        let (x, y) = (302.0 + 90.0 * t.sin(), 180.0 + 90.0 * t.cos());
        img.set(x as usize, y as usize, 40);
    }
    // A dimension line and letters.
    img.fill([100, 300], [280, 302], 40);
    for k in 0..5 {
        img.fill([360 + 12 * k, 150], [363 + 12 * k, 170], 40);
    }
    img
}

#[test]
fn finds_rooms_doors_and_windows() {
    let s = Settings::new(10.0);
    let found = scan(&plan(), &s);
    assert_eq!(found.rooms.len(), 2, "{:?}", found.rooms);
    let (left, right) = if found.rooms[0].outline[0][0] < 200.0 { (0, 1) } else { (1, 0) };
    // The inside of the rooms: corners at the faces of the walls.
    let bounds = |r: &Region| {
        let xs = r.outline.iter().map(|q| q[0]);
        let ys = r.outline.iter().map(|q| q[1]);
        [xs.clone().fold(f32::MAX, f32::min), ys.clone().fold(f32::MAX, f32::min), xs.fold(0.0, f32::max), ys.fold(0.0, f32::max)]
    };
    let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(a, b)| (a - b).abs() <= 2.5);
    assert!(near(bounds(&found.rooms[left]), [80.0, 80.0, 290.0, 320.0]), "{:?}", bounds(&found.rooms[left]));
    assert!(near(bounds(&found.rooms[right]), [302.0, 80.0, 520.0, 320.0]), "{:?}", bounds(&found.rooms[right]));
    assert_eq!(found.rooms[left].outline.len(), 4);
    assert_eq!(found.openings.len(), 2, "{:?}", found.openings);
    let door = found.openings.iter().find(|g| g.kind == GapKind::Door).unwrap();
    assert!(!door.along_x);
    assert_eq!(door.sides, [Some(left), Some(right)]);
    let (w, t) = door.size();
    assert!((w - 90.0).abs() <= 3.0 && (t - 12.0).abs() <= 3.0, "{w} {t}");
    assert_eq!(door.swing, Some((1, true)));
    let window = found.openings.iter().find(|g| g.kind == GapKind::Window).unwrap();
    assert!(window.along_x);
    assert_eq!(window.sides, [None, Some(left)]);
    assert!((window.size().0 - 120.0).abs() <= 3.0);
}

#[test]
fn text_and_lines_are_no_walls() {
    let s = Settings::new(10.0);
    let found = scan(&plan(), &s);
    // The letters and the dimension line stay out of the walls.
    assert!(!found.walls.get(361, 160) && !found.walls.get(150, 301));
    assert!(found.walls.get(60, 200) && found.walls.get(295, 100));
    assert!(found.ink.get(361, 160));
}

#[test]
fn an_empty_sheet_has_nothing() {
    let found = scan(&Gray::new(200, 100, 250), &Settings::new(10.0));
    assert!(found.rooms.is_empty() && found.openings.is_empty() && found.bands.is_empty());
}

#[test]
fn a_door_without_swing_and_a_doorway_to_the_outside() {
    let mut img = plan();
    // No swing; a door in the bottom wall of the right room.
    img.fill([303, 181], [400, 275], 240);
    img.fill([400, 320], [490, 350], 240);
    let found = scan(&img, &Settings::new(10.0));
    assert_eq!(found.rooms.len(), 2);
    let inner = found.openings.iter().find(|g| !g.along_x).unwrap();
    assert_eq!(inner.swing, None);
    let entry = found.openings.iter().find(|g| g.along_x && g.lo[1] > 300.0).unwrap();
    assert_eq!(entry.kind, GapKind::Door);
    assert!(entry.sides[1].is_none() && entry.sides[0].is_some());
}

#[test]
fn outlines_follow_l_shaped_rooms() {
    let mut img = Gray::new(400, 400, 240);
    // An L: the top right quarter of a square is outside.
    img.fill([40, 40], [200, 70], 150);
    img.fill([40, 40], [70, 360], 150);
    img.fill([40, 330], [360, 360], 150);
    img.fill([330, 170], [360, 360], 150);
    img.fill([170, 40], [200, 200], 150);
    img.fill([170, 170], [360, 200], 150);
    let found = scan(&img, &Settings::new(10.0));
    assert_eq!(found.rooms.len(), 1);
    let r = &found.rooms[0];
    assert_eq!(r.outline.len(), 6, "{:?}", r.outline);
    assert!((r.area - (100.0 * 260.0 + 160.0 * 130.0)).abs() < 1500.0, "{}", r.area);
}

#[test]
fn polygon_area_of_a_square() {
    assert_eq!(polygon_area(&[[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]), 4.0);
}
