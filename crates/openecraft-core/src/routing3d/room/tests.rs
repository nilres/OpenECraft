use super::*;
use crate::analysis::Analysis;
use crate::geom::p;
use crate::model::routing3d::RouteKind;
use crate::panel::{SitePlace, ENCLOSURE_PART};
use crate::parts::FnKind;
use crate::routing3d::{freeze, FrozenRoute, RoutedItem, Status};
use crate::testkit::Kit;

/// Room +ROOM1 (4 x 3 m, 2.8 m high) with the cabinet +CAB1 (600 x 800 x 250) against
/// its rear wall at x 500. Ball valve -QM1 in the cabinet, -QM2 and -QM3 on the right
/// wall; hoses -QM1:2 – -QM2:1 and -QM2:2 – -QM3:1. Returns the project, the room and
/// the cabinet.
fn pump_room() -> (Project, usize, usize) {
    let mut k = Kit::new();
    let q1 = k.sym("fl_shutoff", 100.0, 40.0, "-QM1", "");
    let q2 = k.sym("fl_shutoff", 100.0, 80.0, "-QM2", "");
    let q3 = k.sym("fl_shutoff", 100.0, 120.0, "-QM3", "");
    k.wire((q1, "2"), (q2, "1"));
    k.wire((q2, "2"), (q3, "1"));
    k.part("-QM1", FnKind::FluidComponent, "1 2", |p| (p.width, p.height, p.depth) = (40.0, 60.0, 50.0));
    for tag in ["-QM2", "-QM3"] {
        k.part(tag, FnKind::FluidComponent, "1 2", |p| {
            (p.width, p.height, p.depth) = (160.0, 120.0, 140.0);
            p.mounting_data.wall_mount = true;
        });
    }
    let mut prj = k.prj;
    let room = panel::new_room(&mut prj, "+ROOM1", 4000.0, 3000.0, 2800.0);
    prj.layout.enclosures.push(room);
    let cab = panel::new_enclosure(&mut prj, ENCLOSURE_PART, "+CAB1");
    prj.layout.enclosures.push(cab);
    let (r, c) = (0, 1);
    panel::set_room(&mut prj, c, Some(r), Some(SitePlace { x: 500.0, z: 0.0, angle: 0.0 })).unwrap();
    panel::place_device_on(&mut prj, c, "-QM1", p(250.0, 300.0), Face::Plate).unwrap();
    panel::mount_on_wall(&mut prj, r, "-QM2", Face::Wall(1), p(1000.0, 1200.0)).unwrap();
    panel::mount_on_wall(&mut prj, r, "-QM3", Face::Wall(1), p(2400.0, 1200.0)).unwrap();
    (prj, r, c)
}

fn with_roof_gland(prj: &mut Project, c: usize) -> u64 {
    panel::place_accessory(prj, c, "GEN.GLAND-M25", Face::Roof, p(282.0, 107.0)).unwrap()
}

fn tube<'a>(an: &'a Analysis, tag: &str) -> &'a RoutedItem {
    an.routing.items.iter().find(|i| i.kind == RouteKind::Tube && i.key.contains(tag)).unwrap_or_else(|| panic!("hose at {tag}"))
}

#[test]
fn the_room_space_is_free_inside_the_walls_only() {
    let (mut prj, r, c) = pump_room();
    panel::set_outline(&mut prj.layout.enclosures[r], &[[0.0, 0.0], [4000.0, 0.0], [4000.0, 3000.0], [2000.0, 3000.0], [2000.0, 1500.0], [0.0, 1500.0]]).unwrap();
    panel::add_inner_wall(&mut prj, r, [3000.0, 0.0], [3000.0, 1000.0], 100.0).unwrap();
    panel::add_opening(&mut prj, r, panel::Opening::Door, Face::Wall(1), Some(1500.0), None, None).unwrap();
    let pos = panel::positions(&prj.layout);
    let poses = panel::site_poses(&prj);
    let s = room_space(&prj, r, &pos, &poses, &prj.layout.routing);
    let at = |x: f32, y: f32, z: f32| add(pos[r], [x, y, z]);
    assert!(!s.blocked(at(1000.0, 1000.0, 1000.0)), "inside");
    assert!(s.blocked(at(1000.0, 1000.0, 2500.0)), "in the cut-out corner of the L");
    assert!(s.blocked(at(3000.0, 1400.0, 500.0)), "in the inner wall");
    assert!(!s.blocked(at(3000.0, 1400.0, 1300.0)), "beyond the end of the inner wall");
    // The cabinet from the floor to its top (800 mm), free above it.
    assert!(s.blocked(at(800.0, 2500.0, 125.0)));
    assert!(!s.blocked(at(800.0, 1500.0, 125.0)));
    assert_eq!(cabinet_box(&prj, &poses, r, c), ([500.0, 2000.0, 0.0], [1100.0, 2800.0, 250.0]));
    // The door in the right wall swings 1 m into the room, up to its height.
    assert!(s.blocked(at(3500.0, 2500.0, 2000.0)));
    assert!(!s.blocked(at(3500.0, 300.0, 2000.0)), "above the door");
}

#[test]
fn a_hose_from_a_cabinet_to_the_wall_runs_through_its_gland() {
    let (mut prj, r, c) = pump_room();
    let an = Analysis::checked(&prj);
    let h = tube(&an, "-QM1");
    assert!(h.routed(), "{:?}", h.status);
    // Without a gland through the middle of the roof, and reported.
    assert_eq!(h.legs.len(), 2);
    assert!(an.issues.iter().any(|i| i.message().contains("enters through the roof")), "{:?}", an.issues);
    let gland = with_roof_gland(&mut prj, c);
    let an = Analysis::checked(&prj);
    assert!(!an.issues.iter().any(|i| i.message().contains("enters through the roof")));
    let h = tube(&an, "-QM1");
    assert!(h.routed(), "{:?}", h.status);
    assert_eq!(h.legs.iter().map(|l| l.enclosure).collect::<Vec<_>>(), vec![c, r]);
    assert_eq!(h.enclosures, vec![c, r]);
    assert_eq!(h.points.len(), h.legs.iter().map(|l| l.points.len()).sum::<usize>());
    // The cabinet leg ends at the gland, the room leg starts above the roof.
    let pos = panel::positions(&prj.layout);
    let poses = panel::site_poses(&prj);
    let cab = &prj.layout.enclosures[c];
    let g = cab.item(gland).unwrap();
    let inside = add(panel::face_point(cab, Face::Roof, [g.x + g.w / 2.0, g.y + g.h / 2.0, 5.0]), pos[c]);
    assert!(manhattan(*h.legs[0].points.last().unwrap(), inside) < 1.0);
    let start = sub(h.legs[1].points[0], pos[r]);
    assert!(start[1] < 2800.0 - 800.0, "above the cabinet: {start:?}");
    let outside = panel::face_point(cab, Face::Roof, [g.x + g.w / 2.0, g.y + g.h / 2.0, -super::OUTSIDE]);
    assert!(manhattan(start, cabinet_in_room(&prj, &poses, r, c, outside)) < 1.0);
    // The length is that of both legs (less the rounded bends) and the allowances.
    let legs: f32 = h.legs.iter().map(|l| grid_length(&l.points)).sum();
    let extra = 2.0 * prj.fluid.allowance;
    assert!(h.length <= legs + extra + 0.5 && h.length > 0.9 * legs, "{} {}", h.length, legs);
    // The hose between the two wall valves stays in the room.
    let h2 = tube(&an, "-QM3");
    assert!(h2.routed() && h2.legs.is_empty());
    assert_eq!(h2.enclosures, vec![r]);
}

#[test]
fn a_cabinet_hanging_on_the_wall_is_left_through_its_bottom() {
    let (mut prj, r, c) = pump_room();
    prj.layout.enclosures[c].elevation = Some(1800.0);
    let pos = panel::positions(&prj.layout);
    let poses = panel::site_poses(&prj);
    // It takes the space down to the floor.
    assert_eq!(cabinet_box(&prj, &poses, r, c), ([500.0, 200.0, 0.0], [1100.0, 2800.0, 250.0]));
    assert!(shafts_below(&prj, &poses, r, c).is_empty());
    // A gland in its bottom takes the hose down to the valve on the wall below it: the
    // shaft below the gland and out to the front of the cabinet is free, the rest below
    // it and the cabinet itself blocked.
    let gland = panel::place_accessory(&mut prj, c, "GEN.GLAND-M25", Face::Bottom, p(282.0, 107.0)).unwrap();
    assert_eq!(shafts_below(&prj, &poses, r, c), vec![([730.0, 998.0, 59.0], [866.0, 2800.0, 386.0])]);
    let s = room_space(&prj, r, &pos, &poses, &prj.layout.routing);
    assert!(!s.blocked(add(pos[r], [800.0, 2500.0, 125.0])), "below the gland");
    assert!(s.blocked(add(pos[r], [600.0, 2500.0, 125.0])), "below the rest of the cabinet");
    assert!(s.blocked(add(pos[r], [800.0, 600.0, 125.0])));
    let an = Analysis::checked(&prj);
    assert!(!an.issues.iter().any(|i| i.message().contains("enters through the roof")), "{:?}", an.issues);
    let h = tube(&an, "-QM1");
    assert!(h.routed(), "{:?}", h.status);
    assert_eq!(h.enclosures, vec![c, r]);
    let cab = &prj.layout.enclosures[c];
    let g = cab.item(gland).unwrap();
    let inside = add(panel::face_point(cab, Face::Bottom, [g.x + g.w / 2.0, g.y + g.h / 2.0, 5.0]), pos[c]);
    assert!(manhattan(*h.legs[0].points.last().unwrap(), inside) < 1.0, "{:?}", h.legs[0].points);
    // Standing on the floor its bottom is no way out.
    assert!(room_side(&prj, c, Face::Bottom));
    prj.layout.enclosures[c].elevation = None;
    assert!(!room_side(&prj, c, Face::Bottom) && room_side(&prj, c, Face::Roof));
}

fn grid_length(points: &[V3]) -> f32 {
    super::super::grid::length(points)
}

#[test]
fn routes_in_the_room_go_round_inner_walls_and_prefer_pipe_runs() {
    let (mut prj, r, _) = pump_room();
    // An inner wall from the right wall between both wall valves.
    let wall = panel::add_inner_wall(&mut prj, r, [3000.0, 1900.0], [4000.0, 1900.0], 100.0).unwrap();
    let an = Analysis::checked(&prj);
    let h = tube(&an, "-QM3");
    assert!(h.routed(), "{:?}", h.status);
    let fp = panel::inner_footprint(prj.layout.enclosures[r].inner_walls.iter().find(|w| w.id == wall).unwrap());
    let pos = panel::positions(&prj.layout);
    for s in h.points.windows(2) {
        for t in 0..=20 {
            let q = sub([0, 1, 2].map(|i| s[0][i] + (s[1][i] - s[0][i]) * t as f32 / 20.0), pos[r]);
            assert!(!panel::inside_outline(&fp, [q[0], q[2]]) || q[1] < 1.0, "through the inner wall at {q:?}");
        }
    }
    // Without the inner wall the hose runs in free air; a pipe run along the right wall
    // above the valves takes it.
    panel::remove_inner_wall(&mut prj.layout.enclosures[r], wall);
    let free = tube(&Analysis::checked(&prj), "-QM3").free;
    panel::add_pipe_run(&mut prj, r, Face::Wall(1), &[p(1000.0, 1000.0), p(2600.0, 1000.0)], None, None, None).unwrap();
    let an = Analysis::checked(&prj);
    let h = tube(&an, "-QM3");
    assert!(h.routed() && h.free < free * 0.5, "{} {}", h.free, free);
    let path = an.routing.paths.iter().find(|u| u.enclosure == r).expect("the pipe run is used");
    assert!(path.items >= 1);
}

#[test]
fn frozen_routes_through_the_room_keep_their_legs() {
    let (mut prj, r, c) = pump_room();
    with_roof_gland(&mut prj, c);
    let an = Analysis::checked(&prj);
    let key = tube(&an, "-QM1").key.clone();
    assert_eq!(freeze(&mut prj, &an.routing, std::slice::from_ref(&key)), 1);
    let f = prj.layout.frozen.iter().find(|f| f.key == key).unwrap();
    assert_eq!(f.legs.len(), 2);
    assert!(f.points.is_empty());
    assert_eq!((f.legs[0].enclosure, f.legs[1].enclosure), (prj.layout.enclosures[c].id, prj.layout.enclosures[r].id));
    let json = serde_json::to_string(&prj.layout.frozen).unwrap();
    assert_eq!(serde_json::from_str::<Vec<FrozenRoute>>(&json).unwrap(), prj.layout.frozen);
    let an = Analysis::checked(&prj);
    let h = tube(&an, "-QM1");
    assert_eq!(h.status, Status::Frozen);
    assert!(an.routing.changes.is_empty(), "{:?}", an.routing.changes);
    // The cabinet moves along the wall: its gland with it, the frozen route no longer fits.
    panel::set_room(&mut prj, c, Some(r), Some(SitePlace { x: 1500.0, z: 0.0, angle: 0.0 })).unwrap();
    let an = Analysis::checked(&prj);
    assert_eq!(tube(&an, "-QM1").status, Status::Rerouted);
    assert!(an.issues.iter().any(|i| i.message().contains("Frozen route of")));
}

#[test]
fn cables_from_a_cabinet_to_a_room_wall_and_ends_elsewhere() {
    let mut k = Kit::new();
    let f1 = k.sym("cb1", 100.0, 60.0, "-F1", "");
    let e1 = k.sym("heater", 100.0, 80.0, "-E1", "");
    k.cable("-W1", "1 2", "1.5 mm\u{b2}", 0.0);
    k.core((f1, "2"), (e1, "1"), "-W1", "1");
    k.part("-F1", FnKind::Breaker1, "1 2", |p| (p.width, p.height, p.depth) = (18.0, 90.0, 70.0));
    k.part("-E1", FnKind::Generic, "1 2", |p| {
        (p.width, p.height, p.depth) = (300.0, 200.0, 100.0);
        p.mounting_data.wall_mount = true;
    });
    let mut prj = k.prj;
    let room = panel::new_room(&mut prj, "+ROOM1", 4000.0, 3000.0, 2800.0);
    prj.layout.enclosures.push(room);
    let cab = panel::new_enclosure(&mut prj, ENCLOSURE_PART, "+CAB1");
    prj.layout.enclosures.push(cab);
    panel::place_device_on(&mut prj, 1, "-F1", p(250.0, 300.0), Face::Plate).unwrap();
    panel::mount_on_wall(&mut prj, 0, "-E1", Face::Wall(3), p(1000.0, 1000.0)).unwrap();
    // The cabinet outside the room: not routed.
    let an = Analysis::checked(&prj);
    let w1 = an.routing.item("-W1").expect("cable");
    assert_eq!(w1.status, Status::Unrouted(openecraft_i18n::tr("not in the same line-up or room").to_string()));
    // In the room: through the cabinet and the room.
    panel::set_room(&mut prj, 1, Some(0), Some(SitePlace { x: 1500.0, z: 0.0, angle: 0.0 })).unwrap();
    with_roof_gland(&mut prj, 1);
    let an = Analysis::checked(&prj);
    let w1 = an.routing.item("-W1").unwrap();
    assert!(w1.routed() && w1.legs.len() == 2, "{:?}", w1.status);
    let mut tags: Vec<&str> = w1.ends.iter().map(|e| e.tag.as_str()).collect();
    tags.sort_unstable();
    assert_eq!(tags, ["-E1", "-F1"]);
    // A heater on the wall alone, with the breaker not placed: nothing to route.
    let i = prj.layout.enclosures[1].items.iter().position(|i| i.tag == "-F1").unwrap();
    prj.layout.enclosures[1].items.remove(i);
    let an = Analysis::checked(&prj);
    assert!(an.routing.item("-W1").is_none());
}

/// The pump room with a second room +ROOM2 (3 x 3 m) right of it behind a 200 mm wall,
/// -QM3 moved onto its rear wall. Returns the project and both rooms.
fn two_rooms() -> (Project, usize, usize) {
    let (mut prj, r, _) = pump_room();
    let room = panel::new_room(&mut prj, "+ROOM2", 3000.0, 3000.0, 2800.0);
    prj.layout.enclosures.push(room);
    let r2 = prj.layout.enclosures.len() - 1;
    panel::set_site(&mut prj, r2, Some(SitePlace { x: 4200.0, z: 0.0, angle: 0.0 }));
    prj.layout.enclosures[r].items.retain(|i| i.tag != "-QM3");
    panel::mount_on_wall(&mut prj, r2, "-QM3", Face::Wall(0), p(1500.0, 1200.0)).unwrap();
    (prj, r, r2)
}

#[test]
fn routes_pass_from_room_to_room_through_wall_penetrations() {
    let (mut prj, r, r2) = two_rooms();
    let an = Analysis::checked(&prj);
    let h = tube(&an, "-QM3");
    assert_eq!(h.status, Status::Unrouted(openecraft_i18n::tr("no wall penetration leads from room to room").to_string()));
    // A penetration in the right wall of the first room leads into the second one.
    let pen = panel::add_opening(&mut prj, r, panel::Opening::Penetration, Face::Wall(1), Some(1000.0), None, None).unwrap();
    let it = prj.layout.enclosures[r].item(pen).unwrap().clone();
    assert_eq!((it.w, it.h, it.y), (300.0, 200.0, 300.0), "below the ceiling");
    let links = panel::wall_links(&prj);
    assert_eq!(links.len(), 1);
    let l = links[0];
    assert_eq!((l.from, l.to, l.item), (r, r2, pen));
    assert!((l.length - (200.0 + 120.0)).abs() <= 5.0, "{}", l.length);
    assert!((l.a[0] - 3940.0).abs() < 0.5 && (l.b[0] - 60.0).abs() <= 5.0, "{:?} {:?}", l.a, l.b);
    let an = Analysis::checked(&prj);
    let h = tube(&an, "-QM3");
    assert!(h.routed(), "{:?}", h.status);
    assert_eq!(h.legs.iter().map(|lg| lg.enclosure).collect::<Vec<_>>(), vec![r, r2]);
    assert_eq!(h.enclosures, vec![r, r2]);
    // Through the penetration: the first leg ends in front of it, the second starts behind it.
    let pos = panel::positions(&prj.layout);
    assert!(manhattan(sub(*h.legs[0].points.last().unwrap(), pos[r]), l.a) <= 2.0 * super::ROOM_GRID);
    assert!(manhattan(sub(h.legs[1].points[0], pos[r2]), l.b) <= 2.0 * super::ROOM_GRID);
    let legs: f32 = h.legs.iter().map(|lg| grid_length(&lg.points)).sum();
    // Both legs (less the rounded bends), the way through the wall and the allowances.
    let extra = 2.0 * prj.fluid.allowance;
    assert!(h.length <= legs + l.length + extra + 0.5 && h.length > 0.9 * legs + l.length, "{} {}", h.length, legs);
    let nowhere = |prj: &Project| panel::validate(prj, &prj.layout.enclosures[r]).iter().any(|f| f.message.contains("leads into no room"));
    assert!(!nowhere(&prj));
    // Moved away, the second room is out of reach and the penetration leads nowhere.
    panel::set_site(&mut prj, r2, Some(SitePlace { x: 6000.0, z: 0.0, angle: 0.0 }));
    let an = Analysis::checked(&prj);
    assert!(!tube(&an, "-QM3").routed());
    assert!(nowhere(&prj));
}

#[test]
fn routes_take_the_rooms_between_and_turned_rooms() {
    let (mut prj, r, r2) = two_rooms();
    // A corridor +ROOM3 between both rooms, turned by 90 degrees.
    panel::set_site(&mut prj, r2, Some(SitePlace { x: 6400.0, z: 0.0, angle: 0.0 }));
    let room = panel::new_room(&mut prj, "+ROOM3", 3000.0, 2000.0, 2800.0);
    prj.layout.enclosures.push(room);
    let r3 = prj.layout.enclosures.len() - 1;
    // Turned counter-clockwise, its floor plan covers x 4100 to 6100, z 0 to 3000.
    panel::set_site(&mut prj, r3, Some(SitePlace { x: 4100.0, z: 3000.0, angle: 90.0 }));
    for (room, wall) in [(r, Face::Wall(1)), (r2, Face::Wall(3))] {
        panel::add_opening(&mut prj, room, panel::Opening::Penetration, wall, Some(1000.0), None, None).unwrap();
    }
    let links = panel::wall_links(&prj);
    assert_eq!(links.iter().map(|l| (l.from, l.to)).collect::<Vec<_>>(), vec![(r, r3), (r2, r3)], "{links:?}");
    let chain = super::room_chain(&links, r, r2).unwrap();
    assert_eq!(chain.iter().map(|l| (l.from, l.to)).collect::<Vec<_>>(), vec![(r, r3), (r3, r2)]);
    assert!(super::room_chain(&links, r, r).unwrap().is_empty());
    let an = Analysis::checked(&prj);
    let h = tube(&an, "-QM3");
    assert!(h.routed(), "{:?}", h.status);
    assert_eq!(h.enclosures, vec![r, r3, r2]);
}

#[test]
fn routes_take_the_nearest_of_parallel_wall_penetrations() {
    let link = |item: u64, z: f32| panel::WallLink { from: 0, item, to: 1, via: panel::Via::Wall, a: [3900.0, 500.0, z], b: [60.0, 500.0, z], length: 320.0, leave: [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]] };
    let links = [link(1, 500.0), link(2, 2500.0)];
    // From near the second one into the other room.
    let from = [3000.0, 1500.0, 2200.0];
    let to = [500.0, 1500.0, 2600.0];
    assert_eq!(super::nearest_link(&links, links[0], from, to).item, 2);
    assert_eq!(super::nearest_link(&links, links[1], [3000.0, 500.0, 200.0], [500.0, 500.0, 300.0]).item, 1);
    // The other way round through the same penetrations.
    let back = super::nearest_link(&links, links[0].reversed(), to, from);
    assert_eq!((back.item, back.from, back.to), (2, 1, 0));
}

#[test]
fn lines_to_the_outside_run_to_a_building_entry() {
    let mut k = Kit::new();
    let q2 = k.sym("fl_shutoff", 100.0, 80.0, "-QM2", "");
    let q3 = k.sym("fl_shutoff", 100.0, 120.0, "+YARD-QM3", "");
    k.wire((q2, "2"), (q3, "1"));
    k.part("-QM2", FnKind::FluidComponent, "1 2", |p| {
        (p.width, p.height, p.depth) = (160.0, 120.0, 140.0);
        p.mounting_data.wall_mount = true;
    });
    k.part("+YARD-QM3", FnKind::FluidComponent, "1 2", |_| {});
    let mut prj = k.prj;
    let room = panel::new_room(&mut prj, "+ROOM1", 4000.0, 3000.0, 2800.0);
    prj.layout.enclosures.push(room);
    panel::mount_on_wall(&mut prj, 0, "-QM2", Face::Wall(1), p(1000.0, 1200.0)).unwrap();
    let tubes = |prj: &Project| Analysis::checked(prj).routing.items.into_iter().filter(|i| i.kind == RouteKind::Tube).collect::<Vec<_>>();
    // Without a building entry the line to the yard is not routed.
    assert!(tubes(&prj).is_empty());
    let entry = panel::add_opening(&mut prj, 0, panel::Opening::Entry, Face::Wall(3), Some(1000.0), None, None).unwrap();
    let it = prj.layout.enclosures[0].item(entry).unwrap().clone();
    assert_eq!((it.w, it.h, panel::opening_sill(&prj.layout.enclosures[0], &it)), (panel::ENTRY_SIZE.0, panel::ENTRY_SIZE.1, panel::ENTRY_SILL));
    assert!(panel::add_opening(&mut prj, 0, panel::Opening::Entry, Face::Roof, None, None, None).is_err(), "outer walls only");
    let t = tubes(&prj);
    assert!(t.len() == 1 && t[0].routed(), "{t:?}");
    assert!(t[0].ends.iter().any(|e| e.item == Some(entry)));
    assert_eq!(t[0].enclosures, [0]);
    // A device of the room that is not placed yet is no line to the outside.
    crate::ops::rename_device(&mut prj, "+YARD-QM3", "+ROOM1-QM3");
    assert!(tubes(&prj).is_empty());
}

#[test]
fn pipes_between_devices_in_the_room_keep_to_the_ceiling() {
    // -QM2 and -QM3 stand on the floor in the middle of the room, 2 m apart.
    let mut k = Kit::new();
    let q2 = k.sym("fl_shutoff", 100.0, 80.0, "-QM2", "");
    let q3 = k.sym("fl_shutoff", 100.0, 120.0, "-QM3", "");
    k.wire((q2, "2"), (q3, "1"));
    for tag in ["-QM2", "-QM3"] {
        k.part(tag, FnKind::FluidComponent, "1 2", |p| {
            (p.width, p.height, p.depth) = (300.0, 500.0, 300.0);
            p.mounting_data.floor_mount = true;
        });
    }
    let mut prj = k.prj;
    let room = panel::new_room(&mut prj, "+ROOM1", 4000.0, 3000.0, 2800.0);
    prj.layout.enclosures.push(room);
    panel::mount_on_wall(&mut prj, 0, "-QM2", Face::Bottom, p(850.0, 1350.0)).unwrap();
    panel::mount_on_wall(&mut prj, 0, "-QM3", Face::Bottom, p(2850.0, 1350.0)).unwrap();
    let an = Analysis::checked(&prj);
    let h = tube(&an, "-QM3");
    assert!(h.routed(), "{:?}", h.status);
    // Up from the one, along the ceiling, down to the other: not through the open room.
    let top = h.points.iter().map(|q| q[1]).fold(f32::INFINITY, f32::min);
    assert!(top <= KEEP_TO_SURFACES + ROOM_GRID, "highest point {top} mm below the ceiling: {:?}", h.points);
    for s in h.points[1..h.points.len() - 1].windows(2).filter(|s| (s[0][1] - s[1][1]).abs() < 1.0 && crate::parts::vec3::dist(s[0], s[1]) > 400.0) {
        assert!(s[0][1] <= KEEP_TO_SURFACES + ROOM_GRID, "a long stretch through the room at {} mm: {:?}", s[0][1], h.points);
    }
}

#[test]
fn hoses_laid_flush_run_in_the_walls_and_the_floor() {
    let (mut prj, r, c) = pump_room();
    with_roof_gland(&mut prj, c);
    let an = Analysis::build(&prj);
    let (wall_key, floor_key) = (tube(&an, "-QM3").key.clone(), tube(&an, "-QM1").key.clone());
    prj.layout.laying.insert(wall_key.clone(), crate::routing3d::Laying::Wall);
    prj.layout.laying.insert(floor_key.clone(), crate::routing3d::Laying::Floor);
    let an = Analysis::build(&prj);
    let pos = panel::positions(&prj.layout);
    let local = |q: &V3| sub(*q, pos[r]);
    // Between the valves on the right wall: in the wall (x beyond 4000), behind them.
    let w = an.routing.item(&wall_key).unwrap();
    assert!(w.routed() && w.laying == crate::routing3d::Laying::Wall, "{:?}", w.status);
    let inner: Vec<V3> = w.points.iter().map(local).filter(|q| q[0] > 4000.0 + 1.0).collect();
    assert!(inner.len() >= 2, "{:?}", w.points);
    // From the cabinet: to the rear wall, down it and through the floor.
    let f = an.routing.item(&floor_key).unwrap();
    assert!(f.routed(), "{:?}", f.status);
    let room_leg = f.legs.iter().find(|l| l.enclosure == r).expect("a leg through the room");
    assert!(room_leg.points.iter().any(|q| local(q)[1] > 2800.0 + 10.0), "below the floor: {:?}", room_leg.points);
    // Surface-mounted again: through the room as before.
    prj.layout.laying.clear();
    let an = Analysis::build(&prj);
    assert!(an.routing.item(&wall_key).unwrap().points.iter().all(|q| local(q)[0] <= 4000.0 + 0.5));
}
