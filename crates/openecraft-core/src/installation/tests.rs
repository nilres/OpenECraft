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
    // Devices of cabinets are no installation devices.
    assert_eq!(kind("GEN.MPS-2.5-4A"), None);
    assert_eq!(kind("GEN.SOCKET-DIN-16A"), None);
    assert!(cat.iter().filter(|p| p.number.starts_with("GEN.LAMP-")).all(|p| p.wall_mountable()));
    assert!(on_ceiling(cat.iter().find(|p| p.number == "GEN.LAMP-DOWNLIGHT-LED").unwrap()));
    assert!(!on_ceiling(cat.iter().find(|p| p.number == "GEN.LAMP-WALL-LED").unwrap()));
    // Heating elements of rooms, not of cabinets; every kind has its generic part.
    assert_eq!(kind("GEN.HEATER-PANEL-1000"), Some(Kind::Heater));
    assert_eq!(kind("GEN.HEATER-TOWEL-500"), Some(Kind::Heater));
    assert_eq!(kind("GEN.HEATER-IR-600"), Some(Kind::Heater));
    assert!(on_ceiling(cat.iter().find(|p| p.number == "GEN.HEATER-IR-600").unwrap()));
    assert_eq!(kind("GEN.HEATER-100W"), None);
    for k in Kind::ALL {
        assert_eq!(kind(k.default_part()), Some(k), "{k:?}");
    }
}

/// Puts a symbol on page `page` (its pins as the symbol defines them).
fn sym(prj: &mut Project, page: usize, key: &str, at: P, tag: &str, text: &str) {
    let def = prj.symbol(key).unwrap();
    let id = prj.new_id();
    let pins = def.pins.iter().map(|x| x.label.to_string()).collect();
    prj.pages[page].symbols.push(crate::model::SymbolInst { id, sym: key.into(), pos: at, tag: tag.into(), function_text: text.into(), pins, ..Default::default() });
}

fn wire(prj: &mut Project, page: usize, a: P, b: P) {
    let id = prj.new_id();
    prj.pages[page].wires.push(crate::model::Wire { id, a, b, ..Default::default() });
}

/// A 1-pole breaker `tag` of 10 A at `x` of the infeed page (fed from L1 at the top),
/// its outgoing wire down to `to`.
fn breaker(prj: &mut Project, page: usize, x: f32, tag: &str, text: &str, to: f32) {
    sym(prj, page, "cb1", p(x, 80.0), tag, text);
    wire(prj, page, p(x, 24.0), p(x, 76.0));
    wire(prj, page, p(x, 84.0), p(x, to));
    prj.devices.insert(tag.into(), crate::model::DeviceData { parts: vec![crate::parts::PartRef { number: "GEN.MCB-1P-B10".into(), ..Default::default() }], ..Default::default() });
}

/// The electrical room with three more circuits in the schematic: -F5 a socket outlet
/// (no part), -F6 a light behind its switch, -F7 a heating element.
fn with_schematic() -> (Project, usize) {
    let (mut prj, ri) = room();
    let page = prj.pages.iter().position(|p| p.function == "SUP" && p.kind == crate::model::PageKind::Schematic).unwrap();
    breaker(&mut prj, page, 200.0, "=SUP+CAB1-F5", "Kitchen sockets", 180.0);
    sym(&mut prj, page, "socket", p(200.0, 184.0), "+ER1-XD7", "Worktop");
    breaker(&mut prj, page, 260.0, "=SUP+CAB1-F6", "", 116.0);
    sym(&mut prj, page, "no", p(260.0, 120.0), "+ER1-SF3", "");
    wire(&mut prj, page, p(260.0, 124.0), p(260.0, 146.0));
    sym(&mut prj, page, "lamp", p(260.0, 150.0), "+ER1-EA3", "Ceiling");
    breaker(&mut prj, page, 320.0, "=SUP+CAB1-F7", "Heating", 116.0);
    sym(&mut prj, page, "heater", p(320.0, 120.0), "+ER1-EB1", "");
    (prj, ri)
}

fn schematic(prj: &Project) -> Vec<Circuit> {
    let (nl, open) = (Netlist::build(prj, true), Netlist::build(prj, false));
    circuits(prj, &nl, &crate::netlist::connections(prj, &nl, &open), None)
}

/// The protective device of the service socket of the electrical room.
fn f2(prj: &Project) -> String {
    breakers(prj, &Netlist::build(prj, true)).into_iter().find(|b| b.tag.ends_with("-F2")).unwrap().tag
}

#[test]
fn devices_of_the_circuits_of_the_schematic() {
    let (prj, _) = with_schematic();
    let list = schematic(&prj);
    let of = |b: &str| list.iter().find(|c| c.breaker.ends_with(b)).unwrap_or_else(|| panic!("{b}: {list:?}"));
    let f5 = of("-F5");
    assert_eq!(f5.text, "Kitchen sockets");
    assert_eq!(f5.devices.iter().map(|d| (d.tag.as_str(), d.kind, d.part.clone(), d.placed)).collect::<Vec<_>>(), vec![("+ER1-XD7", Kind::Socket, None, None)]);
    // Through the switch to the light.
    let f6 = of("-F6");
    assert_eq!(f6.devices.iter().map(|d| (d.tag.as_str(), d.kind)).collect::<Vec<_>>(), vec![("+ER1-SF3", Kind::Switch), ("+ER1-EA3", Kind::Light)]);
    assert_eq!(of("-F7").devices[0].kind, Kind::Heater);
    // The rail socket of -F2 has a part for cabinets: no device of the installation.
    assert!(!list.iter().any(|c| c.breaker.ends_with("-F2")), "{list:?}");
    assert!(list.iter().all(|c| c.placed() == 0 && c.page.is_none()));
    // Their protective devices are placed in no distribution board yet.
    assert!(list.iter().all(|c| c.board.is_none()));
}

#[test]
fn devices_of_the_schematic_are_placed() {
    let (mut prj, ri) = with_schematic();
    let sc = schematic(&prj).into_iter().find(|c| c.breaker.ends_with("-F5")).unwrap();
    let q = before_wall(&prj, ri, 0, 300.0);
    let id = place_device(&mut prj, &sc, "+ER1-XD7", ri, "", q).unwrap();
    let it = prj.layout.enclosures[ri].item(id).unwrap().clone();
    assert_eq!((it.kind, it.tag.as_str(), it.part.as_str(), it.face), (ItemKind::Device, "+ER1-XD7", "GEN.SOCKET-F-UP", Face::Wall(0)));
    assert_eq!(prj.devices["+ER1-XD7"].parts[0].number, "GEN.SOCKET-F-UP");
    let sc = schematic(&prj).into_iter().find(|c| c.breaker.ends_with("-F5")).unwrap();
    assert_eq!(sc.placed(), 1);
    assert!(sc.has_item(id));
    // Placed again: it moves.
    let q = before_wall(&prj, ri, 1, 300.0);
    let moved = place_device(&mut prj, &sc, "+ER1-XD7", ri, "", q).unwrap();
    assert!(prj.layout.enclosures[ri].item(id).is_none());
    assert_eq!(prj.layout.enclosures[ri].items.iter().filter(|i| i.tag == "+ER1-XD7").count(), 1);
    assert_eq!(prj.layout.enclosures[ri].item(moved).unwrap().face, Face::Wall(1));
    // A heating element with the generic part of its kind; a part of another kind is refused.
    let f7 = schematic(&prj).into_iter().find(|c| c.breaker.ends_with("-F7")).unwrap();
    let q = before_wall(&prj, ri, 2, 300.0);
    assert!(place_device(&mut prj, &f7, "+ER1-EB1", ri, "GEN.SOCKET-F-UP", q).unwrap_err().contains("+ER1-EB1"));
    let h = place_device(&mut prj, &f7, "+ER1-EB1", ri, "", q).unwrap();
    let poses = panel::site_poses(&prj);
    let hp = placed(&prj, &poses, ri, prj.layout.enclosures[ri].item(h).unwrap()).unwrap();
    assert_eq!((hp.kind, hp.part.as_str()), (Kind::Heater, "GEN.HEATER-PANEL-1000"));
    assert!((hp.height - 400.0).abs() < 5.0, "{}", hp.height);
    assert!(place_device(&mut prj, &f7, "+ER1-XD7", ri, "", q).is_err(), "not a device of -F7");
    // The connection of the socket outlet to its protective device: no length while
    // the protective device is placed in no distribution board.
    let f5 = schematic(&prj).into_iter().find(|c| c.breaker.ends_with("-F5")).unwrap();
    let l = &f5.legs[0];
    assert_eq!((l.from.as_str(), l.to.as_str(), l.cable.as_str(), l.routed, l.length), (f5.breaker.as_str(), "+ER1-XD7", "", false, None));
}

#[test]
fn what_was_placed_becomes_the_device_of_the_schematic() {
    let (mut prj, ri) = with_schematic();
    let light = { let q = inside(&prj, ri, 0.4, 0.4); place(&mut prj, ri, "GEN.LAMP-CEILING-LED", q) }.unwrap();
    let sw = { let q = before_wall(&prj, ri, 0, 200.0); place(&mut prj, ri, "GEN.SW-ONOFF-UP", q) }.unwrap();
    let f6 = schematic(&prj).into_iter().find(|c| c.breaker.ends_with("-F6")).unwrap();
    assert!(link_device(&mut prj, &f6, "+ER1-SF3", light).is_err(), "a light is no switch");
    link_device(&mut prj, &f6, "+ER1-EA3", light).unwrap();
    link_device(&mut prj, &f6, "+ER1-SF3", sw).unwrap();
    let it = prj.layout.enclosures[ri].item(light).unwrap();
    assert_eq!((it.kind, it.tag.as_str()), (ItemKind::Device, "+ER1-EA3"));
    assert_eq!(prj.devices["+ER1-EA3"].parts[0].number, "GEN.LAMP-CEILING-LED");
    // A device item of another tag stays what it is.
    assert!(link_device(&mut prj, &f6, "+ER1-SF3", light).is_err());
    let f6 = schematic(&prj).into_iter().find(|c| c.breaker.ends_with("-F6")).unwrap();
    assert_eq!(f6.placed(), 2);
    assert_eq!(f6.devices[1].part.as_deref(), Some("GEN.LAMP-CEILING-LED"));
    // Through the switch to the light.
    assert_eq!(f6.legs.iter().map(|l| (l.from.as_str(), l.to.as_str())).collect::<Vec<_>>(), vec![(f6.breaker.as_str(), "+ER1-SF3"), ("+ER1-SF3", "+ER1-EA3")]);
}

#[test]
fn lights_of_the_manufacturer_catalogs_are_placed_and_go_into_the_project() {
    let (mut prj, ri) = with_schematic();
    assert!(bundled_parts().iter().any(|p| p.number == "SIG.32581100"));
    assert!(!prj.parts.iter().any(|p| p.number == "SIG.32581100"));
    let q = inside(&prj, ri, 0.3, 0.6);
    let id = place(&mut prj, ri, "SIG.32581100", q).unwrap();
    assert_eq!(prj.layout.enclosures[ri].item(id).unwrap().face, Face::Roof);
    assert_eq!(prj.parts.iter().filter(|p| p.number == "SIG.32581100").count(), 1);
    // A device of the schematic placed with it takes it as its part.
    let f6 = schematic(&prj).into_iter().find(|c| c.breaker.ends_with("-F6")).unwrap();
    let q = inside(&prj, ri, 0.6, 0.6);
    let lamp = place_device(&mut prj, &f6, "+ER1-EA3", ri, "LDV.4058075110144", q).unwrap();
    assert_eq!(prj.layout.enclosures[ri].item(lamp).unwrap().part, "LDV.4058075110144");
    assert_eq!(prj.devices["+ER1-EA3"].parts[0].number, "LDV.4058075110144");
    assert!(place_device(&mut prj, &f6, "+ER1-SF3", ri, "LDV.4058075110144", q).is_err(), "a light is no switch");
}

/// The electrical room with socket outlets on two walls, a switch with a light and a
/// heating element put in for -F2: they go into the schematic.
fn with_devices() -> (Project, usize, String, Vec<String>) {
    let (mut prj, ri) = room();
    let f2 = f2(&prj);
    let mut tags = Vec::new();
    for (part, q) in [
        ("GEN.SOCKET-F-UP", before_wall(&prj, ri, 0, 200.0)),
        ("GEN.SOCKET-F-2X-UP", before_wall(&prj, ri, 1, 200.0)),
        ("GEN.SW-ONOFF-UP", before_wall(&prj, ri, 2, 200.0)),
        ("GEN.LAMP-CEILING-LED", inside(&prj, ri, 0.5, 0.3)),
        ("GEN.HEATER-PANEL-1000", before_wall(&prj, ri, 5, 200.0)),
    ] {
        tags.push(add_device(&mut prj, &f2, part, ri, q, &Options::default()).unwrap_or_else(|e| panic!("{part}: {e}")));
    }
    (prj, ri, f2, tags)
}

#[test]
fn devices_put_in_for_a_protective_device_go_into_the_schematic() {
    let (prj, ri, f2, tags) = with_devices();
    let room = prj.layout.enclosures[ri].name.clone();
    assert_eq!(tags, ["XD1", "XD2", "SF1", "EA1", "EB1"].map(|t| format!("{room}-{t}")));
    // Each is a device with its part, placed in the room.
    for t in &tags {
        assert!(placed_in_room(&prj, t).is_some_and(|p| p.0 == ri), "{t}");
        assert_eq!(prj.devices[t].parts.len(), 1, "{t}");
    }
    // Drawn on the installation page of -F2, after the page of -F2, which points to it.
    let c = schematic(&prj).into_iter().find(|c| c.breaker == f2).unwrap();
    let page = c.page.expect("installation page");
    let at = prj.pages.iter().position(|pg| pg.symbols.iter().any(|s| s.tag == f2 && crate::sizing::is_protective(&s.sym))).unwrap();
    assert_eq!(page, at + 1);
    assert_eq!(prj.pages[page].kind, crate::model::PageKind::Schematic);
    assert_eq!(prj.pages[at].symbols.iter().filter(|s| s.sym == "intr" && s.label == "L-F2").count(), 1, "one interruption point at -F2");
    let syms: Vec<(&str, &str)> = prj.pages[page].symbols.iter().filter(|s| !s.tag.is_empty()).map(|s| (s.tag.as_str(), s.sym.as_str())).collect();
    assert_eq!(syms.len(), 5, "{syms:?}");
    assert!(syms.contains(&(tags[0].as_str(), "socket")) && syms.contains(&(tags[3].as_str(), "luminaire")) && syms.contains(&(tags[4].as_str(), "heater")), "{syms:?}");
    // In the circuit of -F2, all placed.
    let kinds: Vec<(&str, Kind)> = c.devices.iter().map(|d| (d.tag.as_str(), d.kind)).collect();
    for (t, k) in tags.iter().zip([Kind::Socket, Kind::Socket, Kind::Switch, Kind::Light, Kind::Heater]) {
        assert!(kinds.contains(&(t.as_str(), k)), "{t}: {kinds:?}");
    }
    assert_eq!(c.placed(), 5);
    // A cable for each, from the protective device to the first, then from device to
    // device; the light behind the switch.
    // (The service socket drawn behind -F2 before has a wire of its own from it.)
    let legs: Vec<(&str, &str)> = c.legs.iter().map(|l| (l.from.as_str(), l.to.as_str())).collect();
    assert!(legs.contains(&(f2.as_str(), "=SUP+ER1-X2")) && legs.contains(&(f2.as_str(), tags[0].as_str())), "{legs:?}");
    assert!(legs.contains(&(tags[0].as_str(), tags[1].as_str())) && legs.contains(&(tags[2].as_str(), tags[3].as_str())), "{legs:?}");
    let cables = c.cables();
    assert_eq!(cables.len(), 5, "{:?}", c.legs);
    // Thick enough for the 16 A of the socket outlets.
    for t in cables {
        let cable = prj.cables.iter().find(|x| x.tag == t).unwrap();
        let mm2 = mm2_of(&cable.cross_section).unwrap();
        assert_eq!(cable.cable_type, format!("{CABLE_TYPE}{mm2}"));
        assert!(crate::sizing::capacity(&prj, mm2).unwrap() >= 16.0, "{mm2}");
    }
    // Each cable runs between the two devices of its leg.
    let (nl, open) = (Netlist::build(&prj, true), Netlist::build(&prj, false));
    let conns = crate::netlist::connections(&prj, &nl, &open);
    for l in c.legs.iter().filter(|l| !l.cable.is_empty()) {
        let cable = prj.cables.iter().find(|x| x.tag == l.cable).unwrap();
        let cores: Vec<&crate::netlist::Connection> = conns.iter().filter(|x| x.cable == l.cable).collect();
        let ends = crate::routing3d::cable_end_tags(&prj, cable, &cores).unwrap();
        assert!(ends == (l.from.clone(), l.to.clone()) || ends == (l.to.clone(), l.from.clone()), "{}: {ends:?} {l:?}", l.cable);
    }
}

#[test]
fn devices_put_in_again_keep_their_cables() {
    let (mut prj, ri, f2, tags) = with_devices();
    let before = schematic(&prj).into_iter().find(|c| c.breaker == f2).unwrap();
    let q = before_wall(&prj, ri, 0, 200.0);
    let t = add_device(&mut prj, &f2, "GEN.SOCKET-F-UP", ri, q, &Options::default()).unwrap();
    let room = prj.layout.enclosures[ri].name.clone();
    assert_eq!(t, format!("{room}-XD3"));
    let after = schematic(&prj).into_iter().find(|c| c.breaker == f2).unwrap();
    assert_eq!(after.page, before.page);
    assert_eq!(after.devices.len(), 6);
    for l in &before.legs {
        assert!(after.legs.iter().any(|a| a.to == l.to && a.cable == l.cable), "{l:?} {:?}", after.legs);
    }
    // Still one interruption point at the protective device.
    let at = prj.pages.iter().position(|pg| pg.symbols.iter().any(|s| s.tag == f2 && crate::sizing::is_protective(&s.sym))).unwrap();
    assert_eq!(prj.pages[at].symbols.iter().filter(|s| s.sym == "intr" && s.label == "L-F2").count(), 1);
    // Not for a device that is no protective device of a final circuit, nor a part of
    // the installation.
    let q = before_wall(&prj, ri, 0, 200.0);
    assert!(add_device(&mut prj, "-NOPE", "GEN.SOCKET-F-UP", ri, q, &Options::default()).is_err());
    assert!(add_device(&mut prj, &f2, "GEN.MCB-1P-B10", ri, q, &Options::default()).is_err());
    assert!(tags.iter().all(|t| prj.layout.find_device(t).is_some()));
}

#[test]
fn the_installation_adds_no_problems() {
    let (prj, _) = room();
    let key = |i: &Issue| i.message();
    let count = |prj: &Project| -> Vec<String> {
        let an = crate::reports::Analysis::checked(prj);
        an.issues.iter().filter(|i| i.severity != Severity::Info).map(key).collect()
    };
    let before = count(&prj);
    let (prj, ..) = with_devices();
    let after = count(&prj);
    let new: Vec<&String> = after.iter().filter(|m| !before.contains(m)).collect();
    assert!(new.is_empty(), "{new:?}");
}

#[test]
fn lengths_come_from_the_3d_routing() {
    let (mut prj, _, f2, _) = with_devices();
    let an = crate::analysis::Analysis::build(&prj);
    let list = circuits(&prj, &an.netlist, &an.connections, Some(&|k: &str| an.routing.routed_length(k)));
    let c = list.iter().find(|c| c.breaker == f2).unwrap();
    let routed: Vec<&Leg> = c.legs.iter().filter(|l| l.routed).collect();
    assert!(!routed.is_empty(), "{:?} {:?}", c.legs, an.routing.items.iter().map(|i| (i.key.clone(), i.status_text())).collect::<Vec<_>>());
    for l in &routed {
        let item = an.routing.item(&l.cable).unwrap();
        assert!((l.length.unwrap() - item.length / 1000.0).abs() < 1e-4);
    }
    // The cable list gets the lengths.
    assert_eq!(write_cables(&mut prj, &list), c.legs.iter().filter(|l| l.length.is_some() && !l.cable.is_empty()).count());
    let first = &routed[0];
    let cable = prj.cables.iter().find(|x| x.tag == first.cable).unwrap();
    assert_eq!(cable.length, (first.length.unwrap() * 2.0).ceil() / 2.0);
}

#[test]
fn heating_elements_in_the_schematic() {
    let (prj, ri, _, tags) = with_devices();
    let poses = panel::site_poses(&prj);
    let all = items(&prj, &poses);
    let heater = all.iter().find(|p| p.tag == tags[4]).unwrap();
    assert_eq!((heater.kind, heater.room), (Kind::Heater, ri));
    assert!(prj.pages.iter().any(|pg| pg.symbols.iter().any(|s| s.sym == "heater" && s.tag == tags[4])));
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
fn a_point_on_the_wall_belongs_to_the_room() {
    // Clicked on the surface of a wall in the 3D view, a hair outside the outline: on the
    // wall in the room; beyond the wall: on its outside.
    let (mut prj, ri) = room();
    let t = prj.layout.enclosures[ri].wall();
    for off in [0.0, -2.0, -0.4 * t] {
        let id = { let q = before_wall(&prj, ri, 0, off); place(&mut prj, ri, "GEN.SOCKET-F-UP", q) }.unwrap();
        assert_eq!(prj.layout.enclosures[ri].item(id).unwrap().face, Face::Wall(0), "{off}");
    }
    let id = { let q = before_wall(&prj, ri, 0, -t - 30.0); place(&mut prj, ri, "GEN.SOCKET-F-UP", q) }.unwrap();
    assert_eq!(prj.layout.enclosures[ri].item(id).unwrap().face, Face::Outside(0));
    // A ceiling light clicked at the edge of the ceiling stays on the ceiling.
    let id = { let q = before_wall(&prj, ri, 0, -1.0); place(&mut prj, ri, "GEN.LAMP-CEILING-LED", q) }.unwrap();
    assert_eq!(prj.layout.enclosures[ri].item(id).unwrap().face, Face::Roof);
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
fn circuits_from_a_breaker_to_their_devices() {
    let (prj, _) = room();
    let nl = Netlist::build(&prj, true);
    let list = breakers(&prj, &nl);
    let f2 = list.iter().find(|b| b.tag.ends_with("-F2")).unwrap_or_else(|| panic!("{list:?}"));
    assert_eq!((f2.rated, f2.text.as_str()), (10.0, "Service socket"));
    assert!(!f2.spare, "the service socket is behind it");
    let (prj, _, tag, _) = with_devices();
    let c = schematic(&prj).into_iter().find(|c| c.breaker == tag).unwrap();
    let r = run(&prj, &Supply::default(), &c);
    assert!(r.length > 2.0 && r.length < 80.0, "{}", r.length);
    assert!(!r.routed, "estimated");
    // 10 A over the connections of 1.5 mm² copper to the farthest device, single-phase.
    assert_eq!((r.rated, r.current, r.mm2), (Some(10.0), Some(10.0), Some(1.5)));
    let (du, pct) = r.drop.unwrap();
    let longest = 2.0 * 10.0 * r.length * prj.electrical.conditions.rho_hot() / 1.5;
    assert!(du > 0.0 && du <= longest * (DEFAULT_COS_PHI + 0.1), "{du} {longest}");
    assert!((pct - du / r.voltage * 100.0).abs() < 1e-3);
    assert!((r.voltage - 230.94).abs() < 0.1);
}

#[test]
fn checks_of_the_circuits() {
    let (mut prj, ri, tag, _) = with_devices();
    let keys = |prj: &Project| -> Vec<String> {
        let (nl, open) = (Netlist::build(prj, true), Netlist::build(prj, false));
        problems(prj, &nl, &crate::netlist::connections(prj, &nl, &open), None).iter().map(|i| i.message()).collect()
    };
    let p = keys(&prj);
    assert!(!p.iter().any(|m| m.contains(&tag)), "{p:?}");
    // Too thin for 10 A.
    let c = schematic(&prj).into_iter().find(|c| c.breaker == tag).unwrap();
    let cable = c.cables()[0].to_string();
    prj.cables.iter_mut().find(|x| x.tag == cable).unwrap().cross_section = "0.5 mm²".into();
    let p = keys(&prj);
    assert!(p.iter().any(|m| m.contains(&tag) && m.contains("mm²")), "{p:?}");
    // Something placed without the schematic, and a device of the schematic not placed.
    let q = before_wall(&prj, ri, 0, 300.0);
    place(&mut prj, ri, "GEN.SOCKET-F-UP", q).unwrap();
    let p = keys(&prj);
    assert!(p.iter().any(|m| m.contains("not in the schematic")), "{p:?}");
    let first = c.devices[0].tag.clone();
    prj.layout.enclosures[ri].items.retain(|i| i.tag != first);
    assert!(keys(&prj).iter().any(|m| m.contains(&tag) && m.contains("placed in no room")));
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
    let (nl, open) = (Netlist::build(&prj, true), Netlist::build(&prj, false));
    let supply = Supply::of(&prj, &nl, &crate::netlist::connections(&prj, &nl, &open));
    let fed = breakers(&prj, &nl).into_iter().find(|b| supply.feeders.get(&b.tag).is_some_and(|f| !f.is_empty()) && board_of(&prj, &b.tag).is_some()).expect("a breaker behind a feeder cable");
    let ri = panel::rooms(&prj.layout)[0];
    let q = before_wall(&prj, ri, 0, 200.0);
    add_device(&mut prj, &fed.tag, "GEN.SOCKET-F-UP", ri, q, &Options::default()).unwrap();
    let c = schematic(&prj).into_iter().find(|c| c.breaker == fed.tag).unwrap();
    let r = run(&prj, &supply, &c);
    assert!(r.drop.is_some(), "{:?} {:?}", c.legs, c.board);
    let up = r.upstream.expect("feeder drop");
    assert!(up > 0.0, "{up}");
    assert!((r.total().unwrap() - r.drop.unwrap().1 - up).abs() < 1e-5);
    // Without the supply's cables only the circuit's own.
    assert_eq!(run(&prj, &Supply::default(), &c).upstream, None);
}

#[test]
fn what_was_placed_goes_into_the_schematic() {
    let (mut prj, ri) = room();
    let f2 = f2(&prj);
    let q = before_wall(&prj, ri, 0, 200.0);
    let id = place(&mut prj, ri, "GEN.SOCKET-F-UP", q).unwrap();
    let tag = into_schematic(&mut prj, &f2, id, &Options::default()).unwrap();
    assert_eq!(prj.layout.enclosures[ri].item(id).unwrap().tag, tag);
    assert!(schematic(&prj).into_iter().find(|c| c.breaker == f2).unwrap().has_item(id));
    // Once only.
    assert!(into_schematic(&mut prj, &f2, id, &Options::default()).is_err());
    // Not for a protective device that is none, and then nothing stays placed.
    let items = prj.layout.enclosures[ri].items.len();
    assert!(add_device(&mut prj, "-NOPE", "GEN.SOCKET-F-UP", ri, q, &Options::default()).is_err());
    assert_eq!(prj.layout.enclosures[ri].items.len(), items);
}

#[test]
fn a_junction_box_of_a_maker_hangs_on_the_wall_as_an_enclosure_with_a_connector() {
    let (mut prj, ri) = room();
    let f2 = f2(&prj);
    let q = before_wall(&prj, ri, 0, 100.0);
    let tag = add_device(&mut prj, &f2, "SPE.80440001", ri, q, &Options::default()).unwrap();
    // No item in the room: an enclosure of the room, on the wall at 2.3 m, its front
    // into the room, with the WAGO connector inside that is the device.
    assert!(prj.layout.enclosures[ri].items.iter().all(|i| i.part != "SPE.80440001"));
    let k = prj.layout.enclosures.iter().position(|e| e.part == "SPE.80440001").expect("the box");
    let b = &prj.layout.enclosures[k];
    assert_eq!((b.width, b.height, b.depth), (102.0, 102.0, 57.0));
    assert_eq!(crate::panel::room_of(&prj.layout, k), Some(ri));
    let bottom = b.elevation.unwrap();
    assert!((bottom + 51.0 - 2300.0).abs() < 5.0, "{bottom}");
    let it = b.items.iter().find(|i| i.part == BOX_CONNECTOR).expect("connector");
    assert_eq!(it.tag, tag);
    assert_eq!(crate::panel::locate(&prj, &tag).map(|l| l.0), Some(k));
    // A connector per conductor; drawn as a junction box on the installation page.
    assert_eq!(prj.devices[&tag].parts.iter().find(|r| r.number == BOX_CONNECTOR).map(|r| r.count), Some(3));
    assert!(prj.pages.iter().any(|pg| pg.symbols.iter().any(|s| s.tag == tag && s.sym == "jbox")));
    // Its rear stands against the wall it was put on.
    let wall = panel::side(&prj.layout.enclosures[ri], Face::Wall(0)).unwrap();
    let place = crate::panel::room_place(&prj, k).unwrap();
    let off = (place.x - wall.a[0]) * wall.normal[0] + (place.z - wall.a[1]) * wall.normal[1];
    assert!(off.abs() < 1.0, "{off}");
    // A socket outlet behind it takes its cable from the box.
    let q = before_wall(&prj, ri, 1, 200.0);
    let socket = add_device(&mut prj, &f2, "GEN.SOCKET-F-UP", ri, q, &Options { from: Some(tag.clone()), ..Default::default() }).unwrap();
    assert_eq!(feed_of(&prj, &socket).map(|f| f.1), Some(tag));
}

#[test]
fn a_click_meets_the_inside_of_a_wall_and_goes_there_off_the_zones() {
    let (prj, ri) = room();
    let e = &prj.layout.enclosures[ri];
    let w = panel::side(e, Face::Wall(0)).unwrap();
    let mid = [(w.a[0] + w.b[0]) / 2.0, (w.a[1] + w.b[1]) / 2.0];
    // From the middle of the room at 1.6 m towards the middle of wall 0.
    let eye = [mid[0] + w.normal[0] * 1500.0, 1600.0, mid[1] + w.normal[1] * 1500.0];
    let dir = [-w.normal[0], 0.0, -w.normal[1]];
    let (q, t) = inside_hit(e, eye, dir).unwrap();
    assert!((t - 1500.0).abs() < 1.0 && (q[1] - 1600.0).abs() < 1.0, "{q:?} {t}");
    // Seen from outside the wall is not met from behind; upwards the ceiling is.
    let behind = [mid[0] - w.normal[0] * 500.0, 1600.0, mid[1] - w.normal[1] * 500.0];
    assert!(inside_hit(e, behind, [w.normal[0], 0.0, w.normal[1]]).is_some_and(|(_, t)| t > 600.0));
    let (up, _) = inside_hit(e, eye, [0.0, 1.0, 0.0]).unwrap();
    assert_eq!(up[1], e.height);
    // Near its mounting height a socket outlet goes there, farther off where clicked.
    assert_eq!(height_for(&prj, "GEN.SOCKET-F-UP", 350.0), None);
    assert_eq!(height_for(&prj, "GEN.SOCKET-F-UP", 1600.0), Some(1600.0));
    assert_eq!(height_for(&prj, "GEN.LAMP-CEILING-LED", 1600.0), None);
    // Placed at the height clicked: its middle there.
    let mut prj = prj;
    let at = before_wall(&prj, ri, 0, 0.0);
    let id = place_at(&mut prj, ri, "GEN.SOCKET-F-UP", at, Some(1600.0)).unwrap();
    let poses = panel::site_poses(&prj);
    let it = prj.layout.enclosures[ri].item(id).unwrap().clone();
    let pl = placed(&prj, &poses, ri, &it).unwrap();
    assert!((pl.height - 1600.0).abs() < 5.0, "{}", pl.height);
    // The preview shows that place without changing the project.
    let before = prj.clone();
    let ghost = preview(&prj, ri, "GEN.SOCKET-F-UP", at, Some(1600.0)).unwrap();
    assert_eq!((ghost.face, ghost.y), (it.face, it.y));
    assert_eq!(prj, before);
}



#[test]
fn a_full_installation_page_is_continued_on_the_next_one() {
    // More socket outlets on one protective device than one page has columns for: all
    // go into the schematic, the later ones on a page after the first.
    let (mut prj, ri) = room();
    let f2 = f2(&prj);
    let mut tags = Vec::new();
    for k in 0..20 {
        let q = before_wall(&prj, ri, (k % 4) as u16, 200.0);
        tags.push(add_device(&mut prj, &f2, "GEN.SOCKET-F-UP", ri, q, &Options::default()).unwrap_or_else(|e| panic!("{k}: {e}")));
    }
    let nl = Netlist::build(&prj, true);
    let pages = pages_of(&prj, &finals(&prj, &nl), &f2);
    assert_eq!(pages.len(), 2, "{pages:?}");
    assert_eq!(pages[1], pages[0] + 1);
    let on = |k: usize| drawn_on(&prj, k).len();
    assert_eq!(on(pages[0]) + on(pages[1]), 20);
    // The chain goes on from the last device of the first page.
    let first_next = drawn_on(&prj, pages[1])[0].tag.clone();
    let before = tags[tags.iter().position(|t| *t == first_next).unwrap() - 1].clone();
    assert_eq!(feed_of(&prj, &first_next).map(|f| f.1), Some(before));
    // Every device is in a circuit of the protective device.
    let c = schematic(&prj).into_iter().find(|c| c.breaker == f2).unwrap();
    assert!(tags.iter().all(|t| c.devices.iter().any(|d| &d.tag == t)), "{:?}", c.devices.iter().map(|d| &d.tag).collect::<Vec<_>>());
}

#[test]
fn cables_into_the_room_get_glands_in_the_roof_of_the_cabinet() {
    let (mut prj, ri) = room();
    let f2 = f2(&prj);
    let cab = crate::panel::locate(&prj, &f2).unwrap().0;
    assert_eq!(crate::panel::room_of(&prj.layout, cab), Some(ri), "the board stands in the room");
    let before = prj.clone();
    for k in 0..2u16 {
        let q = before_wall(&prj, ri, k, 200.0);
        add_device(&mut prj, &f2, "GEN.SOCKET-F-UP", ri, q, &Options::default()).unwrap();
    }
    let added = crate::routing3d::glands::follow(&mut prj, &before);
    assert!(!added.is_empty());
    // On a face a route into the room leaves through: the roof of a standing cabinet.
    for id in &added {
        assert_eq!(prj.layout.enclosures[cab].item(*id).unwrap().face, Face::Roof);
    }
    let an = crate::analysis::Analysis::build(&prj);
    assert!(an.routing.problems.iter().all(|p| !p.message.contains("every cable gland")), "{:?}", an.routing.problems.iter().map(|p| &p.message).collect::<Vec<_>>());
    assert!(an.routing.item("-W1").is_some_and(|i| i.routed()));
}
