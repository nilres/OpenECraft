use super::*;
use crate::panel::buildings::add_building;
use crate::panel::{outline, Enclosure};

/// A drawn floor plan, a pixel 0.5 mm on paper (at 1:20 10 mm in the building): two
/// rooms of 2.1 × 2.4 m and 2.18 × 2.4 m inside walls of 300 mm, a door of 900 mm
/// between them swinging into the right one (hinged at the top), a window of 1200 mm
/// in the top wall of the left one.
fn drawing() -> Gray {
    let mut img = Gray::new(600, 400, 240);
    let wall = 170;
    img.fill([50, 50], [550, 80], wall);
    img.fill([50, 320], [550, 350], wall);
    img.fill([50, 50], [80, 350], wall);
    img.fill([520, 50], [550, 350], wall);
    img.fill([120, 50], [240, 80], 240);
    img.fill([120, 60], [240, 62], 60);
    img.fill([120, 68], [240, 70], 60);
    img.fill([290, 80], [302, 320], wall);
    img.fill([290, 180], [302, 270], 240);
    for i in 0..=200 {
        let t = std::f32::consts::FRAC_PI_2 * i as f32 / 200.0;
        img.set((302.0 + 90.0 * t.sin()) as usize, (180.0 + 90.0 * t.cos()) as usize, 40);
    }
    img
}

/// A project with a building and its level carrying the drawing at 1:20.
fn project() -> (Project, u64) {
    let mut prj = Project::default();
    let bi = add_building(&mut prj, "+B1").unwrap();
    let id = prj.layout.buildings[bi].levels[0].id;
    let mut plan = new_plan("ground floor.pdf", &drawing(), 0.5);
    plan.scale = 20.0;
    set_plan(&mut prj, id, Some(plan)).unwrap();
    (prj, id)
}

#[test]
fn a_new_plan_keeps_the_picture_in_grey() {
    let plan = new_plan("scan.jpg", &drawing(), paper_of_dpi(150.0));
    assert_eq!(plan.image.name, "scan.png");
    assert_eq!(plan.px, [600, 400]);
    assert_eq!(image(&plan).unwrap(), drawing());
    assert!((plan.paper - 0.16933).abs() < 1e-4);
    assert_eq!((plan.scale, plan.opacity), (DEFAULT_SCALE, DEFAULT_OPACITY));
    // A huge picture is shrunk, its pixels larger.
    let big = new_plan("big.png", &Gray::new(MAX_SIDE + 10, 10, 255), 0.1);
    assert_eq!(big.px, [(MAX_SIDE + 10).div_ceil(2) as u32, 5]);
    assert!((big.paper - 0.2).abs() < 1e-6);
}

#[test]
fn pixels_and_building_points() {
    let mut plan = FloorPlan { paper: 0.5, scale: 20.0, x: 1000.0, z: 2000.0, px: [100, 50], ..Default::default() };
    assert_eq!(plan.mm_per_px(), 10.0);
    assert_eq!(to_building(&plan, [10.0, 5.0]), [1100.0, 2050.0]);
    assert_eq!(corners(&plan)[2], [2000.0, 2500.0]);
    // Turned a quarter counter-clockwise: the picture's x runs towards the rear (-z).
    plan.angle = 90.0;
    let b = to_building(&plan, [10.0, 0.0]);
    assert!((b[0] - 1000.0).abs() < 1e-3 && (b[1] - 1900.0).abs() < 1e-3, "{b:?}");
    let q = to_pixel(&plan, to_building(&plan, [33.0, 21.0]));
    assert!((q[0] - 33.0).abs() < 1e-3 && (q[1] - 21.0).abs() < 1e-3);
}

#[test]
fn calibration_gives_the_scale() {
    let plan = FloorPlan { paper: 0.5, scale: 100.0, ..Default::default() };
    // 300 pixels of 0.5 mm are 150 mm on paper; 3 m in the building: 1:20.
    assert_eq!(calibrated(&plan, [0.0, 0.0], [300.0, 0.0], 3000.0).unwrap(), 20.0);
    assert!(calibrated(&plan, [0.0, 0.0], [0.0, 0.0], 3000.0).is_err());
    assert!(calibrated(&plan, [0.0, 0.0], [10.0, 0.0], 0.0).is_err());
}

#[test]
fn plans_are_checked() {
    let (mut prj, id) = project();
    assert!(edit_plan(&mut prj, id, |p| p.scale = 0.0).is_err());
    assert!(edit_plan(&mut prj, id, |p| p.area = Some([10, 10, 5, 20])).is_err());
    edit_plan(&mut prj, id, |p| p.x = 500.0).unwrap();
    assert_eq!(plan_of(&prj.layout, id).unwrap().x, 500.0);
    set_plan(&mut prj, id, None).unwrap();
    assert!(plan_of(&prj.layout, id).is_none());
    assert!(edit_plan(&mut prj, id, |p| p.x = 1.0).is_err());
    assert!(set_plan(&mut prj, 999_999, None).is_err());
}

#[test]
fn rooms_come_from_the_plan() {
    let (mut prj, id) = project();
    let traced = trace(&mut prj, id).unwrap();
    assert_eq!(traced.rooms.len(), 2);
    assert_eq!((traced.doors, traced.windows), (1, 1));
    let room = |k: usize| -> &Enclosure { &prj.layout.enclosures[traced.rooms[k]] };
    let (left, right) = if room(0).site.unwrap().x < room(1).site.unwrap().x { (room(0), room(1)) } else { (room(1), room(0)) };
    for e in [left, right] {
        assert_eq!(e.level, id);
        assert_eq!(e.height, room_height_on(&prj.layout, id));
    }
    // Places and sizes in mm: 10 mm a pixel, the inside of the walls.
    let near = |a: f32, b: f32| (a - b).abs() <= 25.0;
    let (lp, rp) = (left.site.unwrap(), right.site.unwrap());
    assert!(near(lp.x, 800.0) && near(lp.z, 800.0) && near(rp.x, 3020.0), "{lp:?} {rp:?}");
    assert!(near(left.width, 2100.0) && near(left.depth, 2400.0) && near(right.width, 2180.0), "{} {} {}", left.width, left.depth, right.width);
    assert_eq!(outline(left).len(), 4);
    // The window in the top wall of the left room, 400 mm from its left end.
    let win = left.items.iter().find(|it| it.opening == Opening::Window).unwrap();
    assert!(near(win.w, 1200.0) && near(win.x, 400.0), "{} {}", win.w, win.x);
    // The door opens into the right room, hinged at the rear (its left seen from
    // inside the right room, whose wall runs from the front to the rear).
    let door = right.items.iter().find(|it| it.opening == Opening::Door).unwrap();
    assert!(near(door.w, 900.0), "{}", door.w);
    assert_eq!(door.swing, Swing::RightIn);
    assert!(left.items.iter().all(|it| it.opening != Opening::Door));
}

#[test]
fn the_area_limits_what_is_found() {
    let (mut prj, id) = project();
    // The house in the area; a closed box beside it outside.
    edit_plan(&mut prj, id, |p| {
        let mut img = image(p).unwrap();
        img.fill([560, 100], [595, 300], 170);
        img.fill([570, 120], [585, 280], 240);
        p.image = Document::new("plan.png", &img.to_png());
    })
    .unwrap();
    assert_eq!(detect(plan_of(&prj.layout, id).unwrap()).unwrap().rooms.len(), 2);
    edit_plan(&mut prj, id, |p| p.area = Some([40, 40, 556, 360])).unwrap();
    let found = detect(plan_of(&prj.layout, id).unwrap()).unwrap();
    assert_eq!(found.rooms.len(), 2);
    assert!(found.rooms.iter().all(|r| r.outline.iter().all(|q| q[0] < 556.0)));
    edit_plan(&mut prj, id, |p| p.area = Some([0, 0, 20, 20])).unwrap();
    assert!(trace(&mut prj, id).is_err());
}

#[test]
fn turned_plans_make_turned_rooms() {
    let (mut prj, id) = project();
    edit_plan(&mut prj, id, |p| p.angle = 90.0).unwrap();
    let traced = trace(&mut prj, id).unwrap();
    for ri in traced.rooms {
        assert_eq!(prj.layout.enclosures[ri].site.unwrap().angle, 90.0);
    }
}
