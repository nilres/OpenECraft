use super::*;
use crate::netlist::Netlist;

/// The electrical room example: room +ER1 with the line-up +CAB1, whose -F2 (10 A)
/// feeds a service socket.
fn room() -> (Project, usize) {
    let prj = crate::sample::example("electrical_room").unwrap();
    let ri = prj.layout.enclosures.iter().position(|e| e.is_room() && e.name.ends_with("ER1")).unwrap();
    (prj, ri)
}

/// A point of the plant inside the room: `fx`, `fz` of the way across its extent.
fn inside(prj: &Project, ri: usize, fx: f32, fz: f32) -> [f32; 2] {
    let e = &prj.layout.enclosures[ri];
    let (lo, hi) = panel::bounds(&panel::outline(e));
    let q = panel::site_poses(prj)[ri].apply([lo[0] + (hi[0] - lo[0]) * fx, 0.0, lo[1] + (hi[1] - lo[1]) * fz]);
    [q[0], q[2]]
}

/// A point of the plant `off` mm in front of the middle of wall `n` of the room.
fn before_wall(prj: &Project, ri: usize, n: u16, off: f32) -> [f32; 2] {
    let e = &prj.layout.enclosures[ri];
    let w = panel::side(e, Face::Wall(n)).unwrap();
    let mid = [(w.a[0] + w.b[0]) / 2.0 + w.normal[0] * off, (w.a[1] + w.b[1]) / 2.0 + w.normal[1] * off];
    let q = panel::site_poses(prj)[ri].apply([mid[0], 0.0, mid[1]]);
    [q[0], q[2]]
}

#[test]
fn parts_are_sockets_switches_or_lights() {
    let cat = crate::parts::catalog();
    let kind = |n: &str| cat.iter().find(|p| p.number == n).and_then(kind_of);
    assert_eq!(kind("GEN.SOCKET-F-2X-UP"), Some(Kind::Socket));
    assert_eq!(kind("GEN.SW-2WAY-UP"), Some(Kind::Switch));
    assert_eq!(kind("GEN.DIM-LED-UP"), Some(Kind::Switch));
    assert_eq!(kind("GEN.LAMP-CEILING-LED"), Some(Kind::Light));
    assert_eq!(kind("GEN.MCB-1P-B10"), None);
    assert!(cat.iter().filter(|p| p.number.starts_with("GEN.LAMP-")).all(|p| p.wall_mountable()));
    assert!(on_ceiling(cat.iter().find(|p| p.number == "GEN.LAMP-DOWNLIGHT-LED").unwrap()));
    assert!(!on_ceiling(cat.iter().find(|p| p.number == "GEN.LAMP-WALL-LED").unwrap()));
}

#[test]
fn items_go_on_the_nearest_wall_or_the_ceiling() {
    let (mut prj, ri) = room();
    let socket = { let q = before_wall(&prj, ri, 0, 300.0); place(&mut prj, ri, "GEN.SOCKET-F-UP", q) }.unwrap();
    let light = { let q = inside(&prj, ri, 0.3, 0.3); place(&mut prj, ri, "GEN.LAMP-CEILING-LED", q) }.unwrap();
    let e = &prj.layout.enclosures[ri];
    let s = e.item(socket).unwrap();
    assert_eq!(s.face, Face::Wall(0));
    assert_eq!(e.item(light).unwrap().face, Face::Roof);
    let poses = panel::site_poses(&prj);
    let ps = placed(&prj, &poses, ri, s).unwrap();
    assert_eq!(ps.kind, Kind::Socket);
    assert!((ps.height - 300.0).abs() < 5.0, "{}", ps.height);
    let pl = placed(&prj, &poses, ri, e.item(light).unwrap()).unwrap();
    assert_eq!(pl.kind, Kind::Light);
    assert!(e.height - pl.height < 100.0, "{}", pl.height);
    let near = inside(&prj, ri, 0.3, 0.3);
    assert!((pl.at[0] - near[0]).abs() < 10.0 && (pl.at[1] - near[1]).abs() < 10.0);
    // Far from any wall, outside the room, or not a part.
    assert!({ let q = inside(&prj, ri, 0.5, 0.5); place(&mut prj, ri, "GEN.SW-ONOFF-UP", q) }.unwrap_err().contains("wall"));
    assert!({ let q = inside(&prj, ri, 3.0, 3.0); place(&mut prj, ri, "GEN.LAMP-CEILING-LED", q) }.is_err());
    assert!(place(&mut prj, ri, "NO.SUCH-PART", near).is_err());
    assert!(items(&prj, &poses).len() >= 2);
}

#[test]
fn legs_run_in_the_installation_zones() {
    let a = RunPoint { at: [0.0, 0.0], height: 300.0, floor: 0.0, zone: 2200.0 };
    let b = RunPoint { at: [3000.0, 1000.0], height: 300.0, floor: 0.0, zone: 2200.0 };
    // 4 m along the walls, up 1.9 m and down 1.9 m.
    assert_eq!(leg(&a, &b), 4000.0 + 1900.0 + 1900.0);
    // One level up: the floor 3 m higher.
    let c = RunPoint { floor: 3000.0, ..b };
    assert_eq!(leg(&a, &c), 4000.0 + 1900.0 + 1900.0 + 3000.0);
}

#[test]
fn circuits_from_a_breaker_to_their_items() {
    let (mut prj, ri) = room();
    let nl = Netlist::build(&prj, false);
    let list = breakers(&prj, &nl);
    let f2 = list.iter().find(|b| b.tag.ends_with("-F2")).unwrap_or_else(|| panic!("{list:?}"));
    assert_eq!(f2.rated, 10.0);
    assert!(!f2.spare, "the service socket is behind it");
    let tag = f2.tag.clone();
    let s1 = { let q = before_wall(&prj, ri, 0, 200.0); place(&mut prj, ri, "GEN.SOCKET-F-UP", q) }.unwrap();
    let s2 = { let q = before_wall(&prj, ri, 1, 200.0); place(&mut prj, ri, "GEN.SOCKET-F-2X-UP", q) }.unwrap();
    let id = add_circuit(&mut prj, "", &tag, false).unwrap();
    assert_eq!(prj.layout.circuits[0].name, tr_fmt!("Circuit {}", 1));
    let first = prj.layout.circuits[0].name.clone();
    assert!(add_circuit(&mut prj, &first, "", false).is_err());
    connect(&mut prj, 0, s1).unwrap();
    connect(&mut prj, 0, s2).unwrap();
    assert!(connect(&mut prj, 0, 999_999).is_err());
    prj.layout.circuits[0].cos_phi = 1.0;
    let r = &runs(&prj, &Supply::default())[0];
    assert_eq!(r.circuit, id);
    assert!(r.from_board);
    assert_eq!(r.items, vec![s1, s2]);
    assert!(r.length > 2.0 && r.length < 40.0, "{}", r.length);
    // 10 A over the length of 1.5 mm² copper, single-phase.
    assert_eq!((r.rated, r.current), (Some(10.0), Some(10.0)));
    let (du, pct) = r.drop.unwrap();
    let expect = 2.0 * 10.0 * r.length * prj.electrical.conditions.rho_hot() / 1.5;
    assert!((du - expect).abs() < 1e-3 && (pct - du / r.voltage * 100.0).abs() < 1e-3);
    assert!((r.voltage - 230.94).abs() < 0.1);
    // A load given: its current.
    prj.layout.circuits[0].load = 2300.0;
    assert!((runs(&prj, &Supply::default())[0].current.unwrap() - 2300.0 / r.voltage).abs() < 1e-3);
    // The breaker now has a circuit; an item moves to another circuit.
    let nl = Netlist::build(&prj, false);
    assert_eq!(breakers(&prj, &nl).iter().find(|b| b.tag == tag).unwrap().circuit.as_deref(), Some(prj.layout.circuits[0].name.as_str()));
    add_circuit(&mut prj, "Kitchen", "", false).unwrap();
    connect(&mut prj, 1, s2).unwrap();
    assert_eq!((circuit_of(&prj, s1), circuit_of(&prj, s2)), (Some(0), Some(1)));
}

#[test]
fn checks_and_cables() {
    let (mut prj, ri) = room();
    let tag = {
        let nl = Netlist::build(&prj, false);
        breakers(&prj, &nl).into_iter().find(|b| b.tag.ends_with("-F2")).unwrap().tag
    };
    let s = { let q = before_wall(&prj, ri, 0, 200.0); place(&mut prj, ri, "GEN.SOCKET-F-UP", q) }.unwrap();
    let light = { let q = inside(&prj, ri, 0.4, 0.4); place(&mut prj, ri, "GEN.LAMP-CEILING-LED", q) }.unwrap();
    add_circuit(&mut prj, "Sockets", &tag, false).unwrap();
    add_circuit(&mut prj, "Lights", "", false).unwrap();
    connect(&mut prj, 0, s).unwrap();
    let keys = |prj: &Project| -> Vec<String> {
        let (nl, open) = (Netlist::build(prj, true), Netlist::build(prj, false));
        problems(prj, &nl, &crate::netlist::connections(prj, &nl, &open)).iter().map(|i| i.message()).collect()
    };
    let p = keys(&prj);
    assert!(p.iter().any(|m| m.contains("Lights") && m.contains("no protective device")), "{p:?}");
    assert!(p.iter().any(|m| m.contains("in no circuit")), "{p:?}");
    assert!(!p.iter().any(|m| m.contains("Sockets")), "{p:?}");
    // Too thin for 10 A, and a load far too high for it.
    prj.layout.circuits[0].cross_section = 0.5;
    prj.layout.circuits[0].load = 20_000.0;
    let p = keys(&prj);
    assert!(p.iter().any(|m| m.contains("Sockets") && m.contains("mm²")), "{p:?}");
    assert!(p.iter().any(|m| m.contains("Sockets") && m.contains("voltage drop")), "{p:?}");
    connect(&mut prj, 1, light).unwrap();
    prj.layout.circuits[1].breaker = "-NOPE".into();
    assert!(keys(&prj).iter().any(|m| m.contains("-NOPE")));
    // The cables get the lengths.
    assert_eq!(write_cables(&mut prj), 2);
    let c = &prj.layout.circuits[0];
    let cable = prj.cables.iter().find(|x| x.tag == c.cable).unwrap();
    assert_eq!(cable.cable_type, DEFAULT_CABLE);
    assert_eq!(cable.length, (runs(&prj, &Supply::default())[0].length * 2.0).ceil() / 2.0);
    assert_ne!(prj.layout.circuits[0].cable, prj.layout.circuits[1].cable);
    // Gone items are reported.
    prj.layout.enclosures[ri].items.retain(|i| i.id != s);
    assert!(keys(&prj).iter().any(|m| m.contains("are gone")));
}

#[test]
fn reactance_counts_with_the_power_factor() {
    let (prj, _) = room();
    let rho = prj.electrical.conditions.rho_hot();
    let (du1, _) = drop(&prj, 10.0, 20.0, 1.5, 1.0, 230.0, false);
    assert!((du1 - 2.0 * 10.0 * 20.0 * rho / 1.5).abs() < 1e-4);
    let (du, pct) = drop(&prj, 10.0, 20.0, 1.5, 0.8, 230.0, false);
    let expect = 2.0 * 10.0 * 20.0 * (rho / 1.5 * 0.8 + crate::shortcircuit::CABLE_REACTANCE * 0.6);
    assert!((du - expect).abs() < 1e-4 && (pct - du / 230.0 * 100.0).abs() < 1e-4);
    let (d3, _) = drop(&prj, 10.0, 20.0, 1.5, 1.0, 400.0, true);
    assert!((d3 - du1 * 3f32.sqrt() / 2.0).abs() < 1e-4);
}

#[test]
fn the_drop_counts_from_the_supply() {
    // The building example: its distribution has feeder cables with lengths.
    let mut prj = crate::sample::example("building_services").unwrap();
    let supply = Supply::build(&prj);
    let nl = Netlist::build(&prj, true);
    let fed = breakers(&prj, &nl).into_iter().find(|b| supply.feeders.get(&b.tag).is_some_and(|f| !f.is_empty()) && prj.layout.find_device(&b.tag).is_some()).expect("a breaker behind a feeder cable");
    let ri = panel::rooms(&prj.layout)[0];
    let s = { let q = before_wall(&prj, ri, 0, 200.0); place(&mut prj, ri, "GEN.SOCKET-F-UP", q) }.unwrap();
    add_circuit(&mut prj, "Sockets", &fed.tag, false).unwrap();
    connect(&mut prj, 0, s).unwrap();
    let r = &runs(&prj, &supply)[0];
    let up = r.upstream.expect("feeder drop");
    assert!(up > 0.0, "{up}");
    assert!((r.total().unwrap() - r.drop.unwrap().1 - up).abs() < 1e-5);
    // Without the supply's cables only the circuit's own.
    assert_eq!(runs(&prj, &Supply::default())[0].upstream, None);
}

#[test]
fn items_become_devices_on_a_single_line_page() {
    let (mut prj, ri) = room();
    let tag = {
        let nl = Netlist::build(&prj, false);
        breakers(&prj, &nl).into_iter().find(|b| b.tag.ends_with("-F2")).unwrap().tag
    };
    let s = { let q = before_wall(&prj, ri, 0, 200.0); place(&mut prj, ri, "GEN.SOCKET-F-UP", q) }.unwrap();
    let l = { let q = inside(&prj, ri, 0.4, 0.4); place(&mut prj, ri, "GEN.LAMP-CEILING-LED", q) }.unwrap();
    add_circuit(&mut prj, "Room", &tag, false).unwrap();
    connect(&mut prj, 0, s).unwrap();
    connect(&mut prj, 0, l).unwrap();
    let pages = prj.pages.len();
    let made = overview(&mut prj).unwrap();
    assert_eq!((made.len(), prj.pages.len()), (1, pages + 1));
    let room_name = prj.layout.enclosures[ri].name.clone();
    let socket = prj.layout.enclosures[ri].item(s).unwrap().clone();
    assert_eq!((socket.kind, socket.tag.clone()), (ItemKind::Device, format!("{room_name}-XD1")));
    assert_eq!(prj.layout.enclosures[ri].item(l).unwrap().tag, format!("{room_name}-EA1"));
    assert_eq!(prj.devices[&socket.tag].parts[0].number, "GEN.SOCKET-F-UP");
    let page = &prj.pages[made[0]];
    assert_eq!(page.kind, crate::model::PageKind::Overview);
    let syms: Vec<(&str, &str)> = page.symbols.iter().map(|x| (x.sym.as_str(), x.tag.as_str())).collect();
    assert!(syms.contains(&("sl_cb", tag.as_str())) && syms.contains(&("inst_socket2", socket.tag.as_str())) && syms.iter().any(|x| x.0 == "inst_lamp"), "{syms:?}");
    // The breaker drawn single-line is no mismatch with its 1-pole schematic symbol.
    let idx = crate::index::Index::build(&prj);
    let dev = crate::singleline::devices(&prj, &idx).into_iter().find(|d| d.tag == tag).unwrap();
    assert_eq!(dev.status, crate::singleline::Status::Engineered);
    // Made again: the page is replaced, the tags stay.
    assert_eq!(link_devices(&mut prj), 0);
    overview(&mut prj).unwrap();
    assert_eq!(prj.pages.len(), pages + 1);
}

#[test]
fn planned_breakers_for_new_circuits() {
    let (mut prj, ri) = room();
    let s = { let q = before_wall(&prj, ri, 0, 200.0); place(&mut prj, ri, "GEN.SOCKET-F-UP", q) }.unwrap();
    let f2 = {
        let nl = Netlist::build(&prj, false);
        breakers(&prj, &nl).into_iter().find(|b| b.tag.ends_with("-F2")).unwrap().tag
    };
    add_circuit(&mut prj, "Old", &f2, false).unwrap();
    add_circuit(&mut prj, "New", "", false).unwrap();
    connect(&mut prj, 1, s).unwrap();
    assert!(new_breaker(&mut prj, 1, 13.0).is_err());
    let tag = new_breaker(&mut prj, 1, 16.0).unwrap();
    assert!(tag.starts_with("-F"));
    assert_eq!(crate::protection::of_device(&prj, &tag).unwrap().rated, 16.0);
    // It starts at the board of the other circuit.
    assert_eq!(board_of(&prj, &prj.layout.circuits[1]), board_of(&prj, &prj.layout.circuits[0]));
    let r = &runs(&prj, &Supply::default())[1];
    assert_eq!(r.rated, Some(16.0));
    assert!(r.from_board && r.drop.is_some());
    let (nl, open) = (Netlist::build(&prj, true), Netlist::build(&prj, false));
    let msgs: Vec<String> = problems(&prj, &nl, &crate::netlist::connections(&prj, &nl, &open)).iter().map(|i| i.message()).collect();
    assert!(msgs.iter().any(|m| m.contains(&tag) && m.contains("planned")), "{msgs:?}");
    // The single-line diagram turns it into a 1-pole breaker of the schematic.
    let page = overview(&mut prj).unwrap()[0];
    let made = crate::singleline::to_multiline(&mut prj, page).unwrap();
    assert!(prj.pages[made.page].symbols.iter().any(|x| x.tag == tag && x.sym == "cb1"));
}
