use super::*;
use crate::model::panel::{Enclosure, PanelItem, SitePlace};
use crate::panel::buildings::{add_building, add_room};

fn gable(eaves: f32) -> Roof {
    Roof { shape: RoofShape::Gable, x: 0.0, z: 0.0, width: 10_000.0, depth: 8000.0, eaves, pitch: 45.0, overhang: 400.0, ..Default::default() }
}

/// A building with one level, a room of 10 × 8 m filling it and a gable roof with its
/// eaves at `eaves` over it; the room's index.
fn attic(eaves: f32) -> (Project, usize) {
    let mut prj = Project::default();
    let bi = add_building(&mut prj, "+B1").unwrap();
    let level = prj.layout.buildings[bi].levels[0].id;
    let ri = add_room(&mut prj, "+R1", Some(level)).unwrap();
    let e = &mut prj.layout.enclosures[ri];
    (e.width, e.depth) = (10_000.0, 8000.0);
    e.site = Some(SitePlace::default());
    prj.layout.buildings[bi].roofs.push(gable(eaves));
    (prj, ri)
}

#[test]
fn roofs_are_the_lowest_of_their_slopes() {
    let r = gable(3000.0);
    assert_eq!(underside_at(&r, [5000.0, 4000.0]), Some(7000.0), "ridge 4 m above the eaves at 45°");
    assert_eq!(underside_at(&r, [5000.0, 0.0]), Some(3000.0));
    assert!((underside_at(&r, [5000.0, -400.0]).unwrap() - 2600.0).abs() < 0.1, "the overhang reaches below the eaves");
    assert_eq!(underside_at(&r, [5000.0, -500.0]), None);
    assert_eq!(ridge_height(&r), 7000.0);
    // Hip: the ends slope too; pyramid: up to one point; shed: one slope.
    let hip = Roof { shape: RoofShape::Hip, ..r.clone() };
    assert_eq!(underside_at(&hip, [1000.0, 4000.0]), Some(4000.0));
    assert_eq!(ridge_height(&hip), 7000.0);
    let pyramid = Roof { shape: RoofShape::Pyramid, ..r.clone() };
    assert_eq!(underside_at(&pyramid, [5000.0, 4000.0]), Some(7000.0));
    assert!(underside_at(&pyramid, [3000.0, 4000.0]).unwrap() < 7000.0);
    let shed = Roof { shape: RoofShape::Shed, ..r.clone() };
    assert_eq!((underside_at(&shed, [0.0, 8000.0]), underside_at(&shed, [0.0, 0.0])), (Some(3000.0), Some(11_000.0)));
    let flat = Roof { shape: RoofShape::Flat, ..r };
    assert_eq!(planes(&flat), vec![Lin::constant(3000.0)]);
}

#[test]
fn a_low_roof_slopes_the_ceiling_of_the_rooms_below() {
    let (prj, ri) = attic(1500.0);
    let c = ceiling(&prj, ri).expect("the roof comes lower than the room's ceiling");
    assert_eq!(c.flat, 2700.0);
    assert_eq!(c.at([5000.0, 0.0]), 1500.0);
    assert_eq!(c.at([5000.0, 4000.0]), 2700.0);
    assert!((c.at([5000.0, 600.0]) - 2100.0).abs() < 0.1);
    let cells = floor_polys(&prj.layout.enclosures[ri]);
    assert_eq!(c.range(&cells), (1500.0, 2700.0));
    // The ceiling in exact pieces: the two slopes and the flat middle.
    let pieces = c.pieces(&cells[0]);
    assert_eq!(pieces.iter().filter(|(_, l)| !l.is_flat()).count(), 2);
    let total: f32 = pieces.iter().map(|(p, _)| area(p).abs()).sum();
    assert!((total - 80_000_000.0).abs() < 10.0, "{total}");
    for (p, l) in &pieces {
        for q in p {
            assert!((l.at(*q) - c.at(*q)).abs() < 0.5, "{q:?}");
        }
    }
    // Along the gable wall up the slope, along the ceiling and down again.
    let prof = c.profile([0.0, 0.0], [0.0, 8000.0]);
    let expect = [(0.0, 1500.0), (0.15, 2700.0), (0.85, 2700.0), (1.0, 1500.0)];
    assert_eq!(prof.len(), 4, "{prof:?}");
    for ((t, h), (et, eh)) in prof.iter().zip(expect) {
        assert!((t - et).abs() < 1e-3 && (h - eh).abs() < 0.5, "{prof:?}");
    }
    // High roofs leave the ceiling flat.
    let (high, ri) = attic(3000.0);
    assert!(ceiling(&high, ri).is_none());
}

#[test]
fn cabinets_and_wall_items_must_stay_below_the_slope() {
    let (mut prj, ri) = attic(1500.0);
    assert!(problems(&prj).is_empty(), "{:?}", problems(&prj).iter().map(|i| i.message()).collect::<Vec<_>>());
    let room = prj.layout.enclosures[ri].id;
    prj.layout.enclosures.push(Enclosure { name: "+CAB1".into(), width: 600.0, height: 2200.0, depth: 400.0, room, site: Some(SitePlace { x: 1000.0, z: 0.0, angle: 0.0 }), ..Default::default() });
    prj.layout.enclosures[ri].items.push(PanelItem { id: 77, kind: ItemKind::Accessory, face: Face::Wall(0), x: 5000.0, y: 200.0, w: 80.0, h: 80.0, d: 50.0, ..Default::default() });
    let texts: Vec<String> = problems(&prj).iter().map(|i| i.message()).collect();
    assert!(texts.iter().any(|t| t.contains("+CAB1") && t.contains("sloped ceiling")), "{texts:?}");
    assert!(texts.iter().any(|t| t.contains("above the sloped ceiling") && !t.contains("+CAB1")), "{texts:?}");
    // Moved to the middle of the room both fit under the ridge.
    prj.layout.enclosures.last_mut().unwrap().site = Some(SitePlace { x: 1000.0, z: 3800.0, angle: 0.0 });
    prj.layout.enclosures[ri].items[0].y = 2000.0;
    let texts: Vec<String> = problems(&prj).iter().map(|i| i.message()).collect();
    assert!(texts.is_empty(), "{texts:?}");
}

#[test]
fn roof_windows_lie_in_one_slope() {
    let (mut prj, _) = attic(1500.0);
    let b = &mut prj.layout.buildings[0];
    b.roofs[0].windows.push(RoofWindow { id: 1, x: 5000.0, z: 6000.0, width: 780.0, length: 1180.0, part: String::new() });
    let (corners, plane) = window_corners(&b.roofs[0], &b.roofs[0].windows[0]).unwrap();
    assert!(!plane.is_flat());
    // Along the slope its corners are 1180 mm apart, along the eaves 780 mm.
    let d = |a: V3, b: V3| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    assert!((d(corners[0], corners[1]) - 780.0).abs() < 0.5 && (d(corners[1], corners[2]) - 1180.0).abs() < 0.5, "{corners:?}");
    assert!(problems(&prj).is_empty());
    prj.layout.buildings[0].roofs[0].windows[0].z = 4000.0;
    let texts: Vec<String> = problems(&prj).iter().map(|i| i.message()).collect();
    assert!(texts.len() == 1 && texts[0].contains("ridge"), "{texts:?}");
    prj.layout.buildings[0].roofs[0].windows[0].z = 20_000.0;
    let texts: Vec<String> = problems(&prj).iter().map(|i| i.message()).collect();
    assert!(texts.len() == 1 && texts[0].contains("outside the roof"), "{texts:?}");
}

#[test]
fn roof_faces_close_the_building() {
    let r = gable(3000.0);
    let b = Building { roofs: vec![r.clone()], ..Default::default() };
    let faces = roof_faces(&b, &r);
    let kinds = |k: RoofFaceKind| faces.iter().filter(|f| f.kind == k).count();
    assert_eq!((kinds(RoofFaceKind::Top), kinds(RoofFaceKind::Underside)), (2, 2));
    assert!(faces.iter().filter(|f| f.kind == RoofFaceKind::Top).all(|f| f.normal[1] > 0.5));
    assert!(faces.iter().filter(|f| f.kind == RoofFaceKind::Underside).all(|f| f.normal[1] < -0.5));
    // Two gable triangles (each seen from outside and inside), no walls on the eaves sides.
    assert_eq!(kinds(RoofFaceKind::Gable), 8, "two halves of each gable, both sides");
    let hip = Roof { shape: RoofShape::Hip, ..r };
    assert_eq!(roof_faces(&b, &hip).iter().filter(|f| f.kind == RoofFaceKind::Gable).count(), 0);
}

#[test]
fn new_roofs_cover_the_rooms_of_their_building() {
    let (mut prj, ri) = attic(1500.0);
    prj.layout.buildings[0].roofs.clear();
    let id = add_roof(&mut prj, 0, RoofShape::Hip).unwrap();
    let r = prj.layout.buildings[0].roofs.iter().find(|r| r.id == id).unwrap();
    let t = prj.layout.enclosures[ri].wall();
    assert_eq!((r.x, r.z, r.width, r.depth), (-t, -t, 10_000.0 + 2.0 * t, 8000.0 + 2.0 * t));
    assert_eq!(r.eaves, prj.layout.buildings[0].levels[0].height, "on top of the highest level");
    assert_eq!((r.name.as_str(), r.along_z), ("R1", false));
    let w = add_window(&mut prj, 0, 0, None).unwrap();
    assert!(prj.layout.buildings[0].roofs[0].windows.iter().any(|x| x.id == w));
}

#[test]
fn prisms_face_outwards() {
    let square = [[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]];
    let faces = prism(&square, Lin::constant(0.0), Lin { a: 1.0, b: 0.0, c: 50.0 }, |_, _| true);
    assert_eq!(faces.len(), 6);
    assert!(faces[0].1[1] > 0.5 && faces[1].1[1] < -0.5);
    for (pts, n) in &faces[2..] {
        let m = [0, 2].map(|k| pts.iter().map(|p| p[k]).sum::<f32>() / 4.0 - 50.0);
        assert!(n[0] * m[0] + n[2] * m[1] > 0.0, "{n:?} {m:?}");
        let c = crate::parts::vec3::cross(crate::parts::vec3::sub(pts[1], pts[0]), crate::parts::vec3::sub(pts[2], pts[0]));
        assert!(crate::parts::vec3::dot(c, *n) > 0.0, "corners in the order of the normal");
    }
    // The top follows its slope.
    assert!(faces[0].0.iter().any(|p| (p[1] - 150.0).abs() < 0.01));
    // Only the sides kept.
    assert_eq!(prism(&square, Lin::constant(0.0), Lin::constant(10.0), |a, b| a[1] == 0.0 && b[1] == 0.0).len(), 3);
}

#[test]
fn walls_under_the_slope_show_where_they_end() {
    let (prj, ri) = attic(1500.0);
    let e = &prj.layout.enclosures[ri];
    let c = ceiling(&prj, ri).unwrap();
    // The rear wall runs along the eaves: 1500 mm high all along it.
    let rear = wall_line(e, &c, Face::Wall(0)).unwrap();
    assert!(rear.iter().all(|q| (q[1] - (e.height - 1500.0)).abs() < 0.5), "{rear:?}");
    // A gable wall rises to the flat ceiling and comes down again.
    let side = (1..4).find_map(|k| wall_line(e, &c, Face::Wall(k)).filter(|l| l.len() == 4)).expect("a gable wall");
    assert_eq!(side.iter().map(|q| q[1].round()).collect::<Vec<_>>(), vec![1200.0, 0.0, 0.0, 1200.0]);
    assert!(wall_line(e, &c, Face::Roof).is_none());
}
