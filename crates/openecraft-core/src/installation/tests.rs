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
    let r = &runs(&prj)[0];
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
    assert!((runs(&prj)[0].current.unwrap() - 2300.0 / r.voltage).abs() < 1e-3);
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
    let keys = |prj: &Project| -> Vec<String> { problems(prj, &Netlist::build(prj, false)).iter().map(|i| i.message()).collect() };
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
    assert_eq!(cable.length, (runs(&prj)[0].length * 2.0).ceil() / 2.0);
    assert_ne!(prj.layout.circuits[0].cable, prj.layout.circuits[1].cable);
    // Gone items are reported.
    prj.layout.enclosures[ri].items.retain(|i| i.id != s);
    assert!(keys(&prj).iter().any(|m| m.contains("are gone")));
}
