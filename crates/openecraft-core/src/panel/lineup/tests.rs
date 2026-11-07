use super::*;
use crate::netlist::Netlist;
use crate::panel::{new_enclosure, virtual_wiring, OpenWire, RoutedWire, ENCLOSURE_PART};
use crate::analysis::Analysis;
use crate::sample::demo_project;

fn wiring(prj: &Project) -> (Vec<RoutedWire>, Vec<OpenWire>) {
    virtual_wiring(prj, &crate::netlist::connections(prj, &Netlist::build(prj, true), &Netlist::build(prj, false)))
}

fn three() -> Project {
    let mut prj = Project::default();
    for (name, w, h, plinth) in [("+A", 600.0, 2000.0, 100.0), ("+B", 800.0, 1800.0, 0.0), ("+C", 400.0, 2000.0, 100.0)] {
        let mut e = new_enclosure(&mut prj, "", name);
        (e.width, e.height, e.build.plinth) = (w, h, plinth);
        prj.layout.enclosures.push(e);
    }
    prj
}

/// The demo project with a second enclosure +CAB2 that the terminal strip -X1 moved
/// into.
fn demo_with_cab2() -> Project {
    let mut prj = demo_project();
    let cab2 = new_enclosure(&mut prj, ENCLOSURE_PART, "+CAB2");
    prj.layout.enclosures.push(cab2);
    let i = prj.layout.enclosures[0].items.iter().position(|i| i.tag == "-X1").unwrap();
    let x1 = prj.layout.enclosures[0].items.remove(i);
    prj.layout.enclosures[1].items.push(x1);
    prj
}

#[test]
fn enclosures_line_up_side_by_side_on_one_floor() {
    let mut prj = three();
    assert_eq!(lineups(&prj.layout), vec![vec![0], vec![1], vec![2]]);
    assert!(bay_right_of(&mut prj, 1, 0) && bay_right_of(&mut prj, 2, 1));
    assert_eq!(lineups(&prj.layout), vec![vec![0, 1, 2]]);
    assert_eq!(neighbours(&prj.layout, 1), (Some(0), Some(2)));
    let pos = positions(&prj.layout);
    assert_eq!(pos.iter().map(|o| o[0]).collect::<Vec<_>>(), vec![0.0, 600.0, 1400.0]);
    // Floors level: +B has no plinth and is lower.
    let floor = |i: usize| pos[i][1] + prj.layout.enclosures[i].height + prj.layout.enclosures[i].build.plinth;
    assert_eq!((floor(0), floor(1), floor(2)), (2100.0, 2100.0, 2100.0));
    assert_eq!(pos[1][1], 300.0);
    // Taking +B out closes the gap; baying it again in the middle moves +C on.
    unbay(&mut prj, 1);
    assert_eq!(lineups(&prj.layout), vec![vec![0, 2], vec![1]]);
    assert_eq!(positions(&prj.layout)[1][0], 1000.0 + LINEUP_GAP);
    assert!(bay_right_of(&mut prj, 1, 0));
    assert_eq!(lineups(&prj.layout), vec![vec![0, 1, 2]]);
    assert!(!bay_right_of(&mut prj, 1, 1));
}

#[test]
fn broken_chains_keep_every_enclosure_once() {
    let mut prj = three();
    let ids: Vec<u64> = prj.layout.enclosures.iter().map(|e| e.id).collect();
    // A loop and an unknown id.
    prj.layout.enclosures[0].bayed_to = ids[1];
    prj.layout.enclosures[1].bayed_to = ids[0];
    prj.layout.enclosures[2].bayed_to = 999;
    let mut all: Vec<usize> = lineups(&prj.layout).into_iter().flatten().collect();
    all.sort();
    assert_eq!(all, vec![0, 1, 2]);
    // Two enclosures bayed to the same one: the first one gets the place.
    prj.layout.enclosures[0].bayed_to = 0;
    prj.layout.enclosures[1].bayed_to = ids[0];
    prj.layout.enclosures[2].bayed_to = ids[0];
    assert_eq!(lineups(&prj.layout), vec![vec![0, 1], vec![2]]);
}

#[test]
fn passages_through_facing_cutouts_and_open_sides() {
    let mut prj = three();
    bay_right_of(&mut prj, 1, 0);
    let pos = positions(&prj.layout);
    assert!(passages(&prj.layout, &pos, 0).is_empty(), "closed walls");
    assert!(passages(&prj.layout, &pos, 1).is_empty(), "no right neighbour");
    let (y, z) = next_passage_place(&prj.layout, 0);
    let ids = add_passage(&mut prj, 0, y, z, PASSAGE_DIAMETER);
    assert_eq!(ids.len(), 2);
    let list = passages(&prj.layout, &pos, 0);
    assert_eq!(list.len(), 1);
    let at = list[0].at;
    assert_eq!(list[0].cutouts, (Some(ids[0]), Some(ids[1])));
    assert!((at[0] - 600.0).abs() < 0.01 && (at[1] - y).abs() < 0.01 && (at[2] - z).abs() < 0.01, "{at:?}");
    // The cutout in +B sits at the same height of the line-up although +B is lower.
    let cb = prj.layout.enclosures[1].item(ids[1]).unwrap();
    assert!((cb.center().y + pos[1][1] - y).abs() < 0.01);
    // The next passage goes below the first one.
    assert!(next_passage_place(&prj.layout, 0).0 > y + PASSAGE_DIAMETER);
    // Cutouts that do not face each other are no passage.
    prj.layout.enclosures[1].items.iter_mut().for_each(|i| i.y += 300.0);
    assert!(passages(&prj.layout, &pos, 0).is_empty());
    // A side open (no side panel): the cutout in the other wall is enough.
    prj.layout.enclosures[1].build.left_open = true;
    assert_eq!(passages(&prj.layout, &pos, 0)[0].cutouts, (Some(ids[0]), None));
    // Both sides open: wires cross anywhere beside the plates.
    prj.layout.enclosures[0].build.right_open = true;
    assert!(passages(&prj.layout, &pos, 0).len() > 5);
    assert_eq!(add_passage(&mut prj, 0, y, z, PASSAGE_DIAMETER), Vec::<u64>::new());
}

#[test]
fn a_drawn_cutout_gets_its_facing_cutout() {
    let mut prj = three();
    bay_right_of(&mut prj, 1, 0);
    bay_right_of(&mut prj, 2, 1);
    let pos = positions(&prj.layout);
    // A rectangular cutout in the right wall of +B, near its front.
    let id = prj.new_id();
    let cut = PanelItem { id, kind: ItemKind::Cutout, face: Face::Right, x: 30.0, y: 900.0, w: 80.0, h: 40.0, z: -2.0, d: 2.0, ..Default::default() };
    prj.layout.enclosures[1].items.push(cut);
    assert!(passages(&prj.layout, &pos, 1).is_empty());
    let facing = add_facing_cutout(&mut prj, 1, id).unwrap();
    let list = passages(&prj.layout, &pos, 1);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].cutouts, (Some(id), Some(facing)));
    // Mirrored: near the front of +C as well (its left wall has the front on the right).
    let f = prj.layout.enclosures[2].item(facing).unwrap();
    assert_eq!((f.x, f.w, f.h), (250.0 - 30.0 - 80.0, 80.0, 40.0));
    assert_eq!(f.y + pos[2][1], 900.0 + pos[1][1]);
    // And back from the left wall; nothing for the plate or a wall without neighbour.
    assert!(add_facing_cutout(&mut prj, 2, facing).is_some());
    assert_eq!(passages(&prj.layout, &pos, 1).len(), 2);
    let plate = prj.new_id();
    prj.layout.enclosures[2].items.push(PanelItem { id: plate, kind: ItemKind::Cutout, w: 10.0, h: 10.0, ..Default::default() });
    assert_eq!(add_facing_cutout(&mut prj, 2, plate), None);
    prj.layout.enclosures[2].items.last_mut().unwrap().face = Face::Right;
    assert_eq!(add_facing_cutout(&mut prj, 2, plate), None);
}

#[test]
fn wires_cross_into_the_neighbouring_enclosure() {
    let mut prj = demo_with_cab2();
    let x1 = |from: &str, to: &str| from.starts_with("-X1") || to.starts_with("-X1");
    // The motor in the field is not placed.
    let reasons = |prj: &Project| -> Vec<&'static str> { wiring(prj).1.iter().filter(|w| x1(&w.from, &w.to) && w.reason != "device not placed").map(|w| w.reason).collect() };
    assert!(!wiring(&prj).0.iter().any(|w| x1(&w.from, &w.to)));
    assert!(!reasons(&prj).is_empty() && reasons(&prj).iter().all(|r| *r == NOT_BAYED), "{:?}", reasons(&prj));
    bay_right_of(&mut prj, 1, 0);
    assert!(!reasons(&prj).is_empty() && reasons(&prj).iter().all(|r| *r == NO_PASSAGE));
    let (y, z) = next_passage_place(&prj.layout, 0);
    add_passage(&mut prj, 0, y, z, PASSAGE_DIAMETER);
    assert!(reasons(&prj).is_empty());
    let an = Analysis::checked(&prj);
    let w = an.wiring.iter().find(|w| w.from.starts_with("-X1") || w.to.starts_with("-X1")).unwrap();
    assert_eq!(w.legs.len(), 1);
    assert_ne!(w.enclosure, w.last_enclosure());
    assert!(w.in_enclosure(0) && w.in_enclosure(1));
    // +CAB2 has no cable ducts: the wire is routed through the space, through the passage.
    let passage = &passages(&prj.layout, &positions(&prj.layout), 0)[0];
    let through = w.legs[0].passage;
    assert_eq!(through[0], passage.at[0]);
    assert!((through[1] - passage.at[1]).abs() <= passage.half[0] && (through[2] - passage.at[2]).abs() <= passage.half[1], "{through:?} in {passage:?}");
    // From connection point to connection point: its way and the reserve, nothing for
    // a way down to the plate.
    assert!((w.length - w.plate_length() - prj.layout.wire_reserve).abs() < 0.5, "{} of {}", w.length, w.plate_length());
    assert_eq!(w.stretch_passages().iter().map(|s| (s.0, s.2.is_some(), s.3.is_some())).collect::<Vec<_>>(), vec![(w.enclosure, false, true), (w.last_enclosure(), true, false)]);
    // Both stretches end at the plate edge beside the passage and show in both enclosures.
    let pw = prj.layout.enclosures[0].plate_size().0;
    let edge = if w.enclosure == 0 { *w.route.points.last().unwrap() } else { w.legs[0].route.points[0] };
    assert!((edge.x - pw).abs() < 0.01, "{edge:?}");
    for ei in [0, 1] {
        let path = w.paths_in(ei, Face::Plate)[0];
        assert!(crate::panel::wires_at(&an.wiring, ei, Face::Plate, path[path.len() / 2], 1.0).contains(&an.wiring.iter().position(|x| x.key() == w.key()).unwrap()));
    }
    // The ducts of +CAB1 carry the wire.
    let fill_before = an.duct_fill.iter().filter(|f| f.enclosure == 0).map(|f| f.total_wires).sum::<usize>();
    assert!(fill_before > 0);
    assert!(w.routing().contains("passage"));
}

#[test]
fn the_shortest_passage_and_enclosures_between() {
    let mut prj = demo_with_cab2();
    bay_right_of(&mut prj, 1, 0);
    let lengths = |prj: &Project| -> Vec<((String, String), f32)> {
        let mut v: Vec<_> = wiring(prj).0.iter().filter(|w| !w.legs.is_empty()).map(|w| (w.key(), w.length)).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    };
    let mut top = prj.clone();
    add_passage(&mut top, 0, 150.0, 80.0, PASSAGE_DIAMETER);
    let mut bottom = prj.clone();
    add_passage(&mut bottom, 0, 650.0, 80.0, PASSAGE_DIAMETER);
    add_passage(&mut prj, 0, 150.0, 80.0, PASSAGE_DIAMETER);
    add_passage(&mut prj, 0, 650.0, 80.0, PASSAGE_DIAMETER);
    let (both, top, bottom) = (lengths(&prj), lengths(&top), lengths(&bottom));
    assert!(!both.is_empty() && both.len() == top.len() && both.len() == bottom.len());
    // Every wire takes the passage that makes it shortest.
    for ((b, t), m) in both.iter().zip(&top).zip(&bottom) {
        assert!((b.1 - t.1.min(m.1)).abs() < 0.5, "{b:?} {t:?} {m:?}");
    }
    assert!(both.iter().zip(&top).any(|(b, t)| b.1 < t.1 - 1.0) && both.iter().zip(&bottom).any(|(b, m)| b.1 < m.1 - 1.0));
    // A third enclosure between them: the wires run through it.
    let mid = new_enclosure(&mut prj, ENCLOSURE_PART, "+CAB3");
    prj.layout.enclosures.push(mid);
    bay_right_of(&mut prj, 2, 0);
    assert!(wiring(&prj).1.iter().any(|w| w.reason == NO_PASSAGE));
    add_passage(&mut prj, 2, 400.0, 80.0, PASSAGE_DIAMETER);
    assert!(wiring(&prj).1.iter().any(|w| w.reason == NO_PASSAGE), "+CAB3 has no passage on its left yet");
    // The passages of +CAB1 face +CAB3 now: a cutout in its left wall opens them.
    let place = PanelItem { face: Face::Left, ..prj.layout.enclosures[1].items.iter().find(|i| i.kind == ItemKind::Cutout).unwrap().clone() };
    prj.layout.enclosures[2].items.push(place);
    let (routed, _) = wiring(&prj);
    let w = routed.iter().find(|w| !w.legs.is_empty()).unwrap();
    assert_eq!(w.stretches().map(|s| s.0).collect::<Vec<_>>().len(), 3);
    assert!(w.in_enclosure(2));
}

fn near(a: V3, b: V3) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() < 0.01)
}

#[test]
fn line_ups_stand_at_their_places_in_the_room() {
    let mut prj = three();
    assert!(bay_right_of(&mut prj, 1, 0));
    // Without places: side by side as in the line-up coordinates, on their plinths.
    let poses = site_poses(&prj);
    assert!(near(poses[0].at, [0.0, 100.0, 0.0]) && near(poses[1].at, [600.0, 0.0, 0.0]));
    assert!(near(poses[2].at, [1400.0 + LINEUP_GAP, 100.0, 0.0]) && poses[2].angle == 0.0);
    // A place set on any enclosure belongs to the left one of its line-up; the line-up
    // turns as a whole.
    set_site(&mut prj, 1, Some(SitePlace { x: 5000.0, z: 2000.0, angle: 90.0 }));
    assert!(prj.layout.enclosures[0].site.is_some() && prj.layout.enclosures[1].site.is_none());
    let poses = site_poses(&prj);
    assert!(near(poses[0].at, [5000.0, 100.0, 2000.0]), "{:?}", poses[0]);
    assert!(near(poses[1].at, [5000.0, 0.0, 1400.0]), "{:?}", poses[1]);
    assert!(near(poses[0].turn([0.0, 0.0, 1.0]), [1.0, 0.0, 0.0]), "the fronts face to the right");
    let q = [120.0, 340.0, 56.0];
    assert!(near(poses[1].local(poses[1].apply(q)), q));
    // From the plant layout: the centre of the footprint on the node, raised by the
    // elevation.
    prj.field.nodes.push(crate::field::Node { id: 9, x: 3.0, y: 4.0, ..Default::default() });
    prj.field.places.push(crate::field::Place { tag: "C".into(), node: 9, drop: 0.0 });
    prj.layout.enclosures[2].elevation = Some(1500.0);
    let (w, d) = (prj.layout.enclosures[2].width, prj.layout.enclosures[2].depth);
    assert!(near(site_poses(&prj)[2].at, [3000.0 - w / 2.0, 1600.0, 4000.0 - d / 2.0]), "{:?}", site_poses(&prj)[2]);
    set_site(&mut prj, 0, None);
    assert!(near(site_poses(&prj)[0].at, [0.0, 100.0, 0.0]));
}
