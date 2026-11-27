use super::*;
use crate::builtin::testing::*;

/// The electrical room example and a point of the plant inside its room, `fx`, `fz`
/// of the way across it.
fn room_session() -> (Session, String, impl Fn(&Project, f32, f32) -> [f32; 2]) {
    let s = with_project(openecraft_core::sample::example("electrical_room").unwrap());
    let prj = s.project().unwrap();
    let ri = prj.layout.enclosures.iter().position(|e| e.is_room()).unwrap();
    let name = prj.layout.enclosures[ri].name.clone();
    let at = move |prj: &Project, fx: f32, fz: f32| {
        let e = &prj.layout.enclosures[ri];
        let (lo, hi) = panel::bounds(&panel::outline(e));
        let q = panel::site_poses(prj)[ri].apply([lo[0] + (hi[0] - lo[0]) * fx, 0.0, lo[1] + (hi[1] - lo[1]) * fz]);
        [q[0], q[2]]
    };
    (s, name, at)
}

#[test]
fn devices_go_into_the_schematic_and_their_circuits() {
    let (mut s, room, at) = room_session();
    let breakers = run(&mut s, "list_breakers", json!({})).unwrap();
    let f2 = breakers.as_array().unwrap().iter().find(|b| b["breaker"].as_str().unwrap().ends_with("-F2")).unwrap().clone();
    assert_eq!((f2["rated"].as_f64(), f2["spare"].as_bool(), f2["text"].as_str()), (Some(10.0), Some(false), Some("Service socket")));
    let spare = run(&mut s, "list_breakers", json!({ "spare": true })).unwrap();
    assert!(spare.as_array().unwrap().iter().all(|b| b["spare"] == true));
    // A light in the middle of the ceiling, a socket on a wall near a corner.
    let q = at(s.project().unwrap(), 0.5, 0.5);
    let light = run(&mut s, "place_installation", json!({ "breaker": f2["breaker"], "room": room, "part": "GEN.LAMP-CEILING-LED", "x": q[0], "z": q[1] })).unwrap();
    assert_eq!(light["face"], "ceiling");
    assert!(light["device"].as_str().unwrap().ends_with("-EA1") && light["page"].is_string() && light["cable"].is_string(), "{light}");
    let q = at(s.project().unwrap(), 0.02, 0.5);
    let socket = run(&mut s, "place_installation", json!({ "breaker": f2["breaker"], "room": room, "part": "GEN.SOCKET-F-UP", "x": q[0], "z": q[1] })).unwrap();
    assert!(socket["face"].as_str().unwrap().starts_with("wall"), "{socket}");
    assert!(run(&mut s, "place_installation", json!({ "breaker": "-NOPE", "room": room, "part": "GEN.SOCKET-F-UP", "x": q[0], "z": q[1] })).is_err());
    // The circuit of -F2: both devices, their cables with lengths, the voltage drop.
    let c = run(&mut s, "list_circuits", json!({ "breaker": f2["breaker"], "estimate": true })).unwrap();
    let devices: Vec<&str> = c[0]["devices"].as_array().unwrap().iter().map(|d| d["device"].as_str().unwrap()).collect();
    assert!(devices.contains(&light["device"].as_str().unwrap()) && devices.contains(&socket["device"].as_str().unwrap()), "{c}");
    let legs = c[0]["connections"].as_array().unwrap();
    assert!(legs.iter().any(|l| l["cable"] == light["cable"] && l["length"].as_f64().unwrap() > 0.5), "{c}");
    assert!(c[0]["length"].as_f64().unwrap() > 1.0);
    assert!(c[0]["drop_percent"].as_f64().unwrap() > 0.0);
    assert_eq!(c[0]["page"], light["page"]);
    assert!(run(&mut s, "list_circuits", json!({ "breaker": "-NOPE", "estimate": true })).is_err());
    // The cable list gets the lengths.
    let w = run(&mut s, "write_circuit_cables", json!({})).unwrap();
    assert!(w["cables"].as_u64().unwrap() >= 2, "{w}");
    let prj = s.project().unwrap();
    assert!(prj.cables.iter().any(|c| c.tag == light["cable"].as_str().unwrap() && c.length > 0.0));
}

/// The electrical room example with a socket outlet `+ER1-XD7` (no part) behind a new
/// breaker -F5 of 10 A on the infeed page, and a socket outlet placed in the room
/// without a device tag.
fn with_socket_circuit() -> (Session, String, impl Fn(&Project, f32, f32) -> [f32; 2], u64) {
    use openecraft_core::geom::p;
    use openecraft_core::model::{DeviceData, PageKind, SymbolInst, Wire};
    let mut prj = openecraft_core::sample::example("electrical_room").unwrap();
    let page = prj.pages.iter().position(|pg| pg.function == "SUP" && pg.kind == PageKind::Schematic).unwrap();
    for (key, at, tag) in [("cb1", p(200.0, 80.0), "=SUP+CAB1-F5"), ("socket", p(200.0, 184.0), "+ER1-XD7")] {
        let pins = prj.symbol(key).unwrap().pins.iter().map(|x| x.label.to_string()).collect();
        let id = prj.new_id();
        prj.pages[page].symbols.push(SymbolInst { id, sym: key.into(), pos: at, tag: tag.into(), pins, ..Default::default() });
    }
    for (a, b) in [(p(200.0, 24.0), p(200.0, 76.0)), (p(200.0, 84.0), p(200.0, 180.0))] {
        let id = prj.new_id();
        prj.pages[page].wires.push(Wire { id, a, b, ..Default::default() });
    }
    prj.devices.insert("=SUP+CAB1-F5".into(), DeviceData { parts: vec![openecraft_core::parts::PartRef { number: "GEN.MCB-1P-B10".into(), ..Default::default() }], ..Default::default() });
    let (_, name, at) = room_session();
    let ri = prj.layout.enclosures.iter().position(|e| e.is_room()).unwrap();
    let q = at(&prj, 0.5, 0.02);
    let free = inst::place(&mut prj, ri, "GEN.SOCKET-F-UP", q).unwrap();
    (with_project(prj), name, at, free)
}

#[test]
fn devices_of_schematic_circuits_are_placed() {
    let (mut s, room, at, free) = with_socket_circuit();
    let list = run(&mut s, "list_circuits", json!({ "estimate": true })).unwrap();
    let f5 = list.as_array().unwrap().iter().find(|c| c["breaker"] == "=SUP+CAB1-F5").unwrap_or_else(|| panic!("{list}")).clone();
    assert_eq!((f5["devices"][0]["device"].as_str(), f5["devices"][0]["kind"].as_str(), f5["devices"][0]["part"].as_str()), (Some("+ER1-XD7"), Some("socket"), Some("GEN.SOCKET-F-UP")));
    assert!(f5["devices"][0]["room"].is_null() && f5["page"].is_null());
    // Placed by a point of the plant.
    let q = at(s.project().unwrap(), 0.02, 0.5);
    let r = run(&mut s, "place_circuit_device", json!({ "device": "+ER1-XD7", "room": room, "x": q[0], "z": q[1] })).unwrap();
    assert!(r["face"].as_str().unwrap().starts_with("wall"), "{r}");
    assert_eq!(r["breaker"], "=SUP+CAB1-F5");
    let list = run(&mut s, "list_circuits", json!({ "breaker": "=SUP+CAB1-F5", "unplaced": true, "estimate": true })).unwrap();
    assert!(list[0]["devices"].as_array().unwrap().is_empty() && list[0]["placed"] == 1);
    // A socket outlet placed before becomes the device; the first one goes.
    let r2 = run(&mut s, "place_circuit_device", json!({ "device": "+ER1-XD7", "item": free })).unwrap();
    assert_eq!(r2["id"], free);
    let prj = s.project().unwrap();
    assert_eq!(prj.layout.enclosures.iter().flat_map(|e| e.items.iter()).filter(|it| it.tag == "+ER1-XD7").count(), 1);
    assert!(run(&mut s, "place_circuit_device", json!({ "device": "-NOPE", "room": room })).is_err());
}

/// A house built from commands: L1 to L3 feed the 1-pole breaker -F1 (16 A) and the
/// 3-pole breaker -F2 (16 A), both put into the board +MD in room +ROOM1 (placed after
/// the room, the case the chain of the connections got wrong).
fn house() -> Session {
    let mut s = new_session();
    let mut go = |name: &str, args: Value| run(&mut s, name, args).unwrap_or_else(|e| panic!("{name}: {e}"));
    for (sym, x) in [("pot_l1", 40), ("pot_l1", 100), ("pot_l2", 108), ("pot_l3", 116)] {
        go("place_symbol", json!({ "page": "1", "symbol": sym, "x": x, "y": 36 }));
    }
    go("place_symbol", json!({ "page": "1", "symbol": "cb1", "x": 40, "y": 60, "tag": "-F1" }));
    go("place_symbol", json!({ "page": "1", "symbol": "cb3", "x": 100, "y": 60, "tag": "-F2" }));
    for x in [40, 100, 108, 116] {
        go("add_wire", json!({ "page": "1", "points": [[x, 40], [x, 56]] }));
    }
    go("add_room", json!({ "name": "+ROOM1", "width": 5000, "depth": 4000, "height": 2700, "x": 0, "z": 0 }));
    go("add_enclosure", json!({ "name": "+MD", "system": "plastic", "width": 348, "height": 630, "depth": 95 }));
    go("place_in_room", json!({ "enclosure": "+MD", "room": "+ROOM1", "x": 100, "z": 0 }));
    for (tag, number) in [("-F1", "GEN.MCB-1P-B16"), ("-F2", "GEN.MCB-3P-B16")] {
        let part = openecraft_core::parts::PartRef { number: number.into(), ..Default::default() };
        s.project_mut().unwrap().devices.insert(tag.into(), openecraft_core::model::DeviceData { parts: vec![part], ..Default::default() });
        let p = openecraft_core::parts::catalog().iter().find(|p| p.number == number).unwrap().clone();
        s.project_mut().unwrap().parts.push(p);
    }
    let mut go = |name: &str, args: Value| run(&mut s, name, args).unwrap_or_else(|e| panic!("{name}: {e}"));
    // The breakers into the board, sized by their parts.
    for (tag, x) in [("-F1", 20), ("-F2", 60)] {
        let it = go("place_cabinet_device", json!({ "enclosure": "+MD", "tag": tag, "x": x, "y": 100 }));
        assert_eq!(it["tag"], tag, "{it}");
    }
    s
}

/// The connections of circuit `breaker`: from, to and cable.
fn legs(s: &mut Session, breaker: &str) -> (Vec<(String, String, String)>, Value) {
    let c = run(s, "list_circuits", json!({ "breaker": breaker, "estimate": true })).unwrap();
    let legs = c[0]["connections"].as_array().unwrap().iter().map(|l| (l["from"].as_str().unwrap().to_string(), l["to"].as_str().unwrap().to_string(), l["cable"].as_str().unwrap().to_string())).collect();
    (legs, c[0].clone())
}

#[test]
fn a_circuit_runs_from_its_breaker_through_its_devices() {
    let mut s = house();
    let it = s.project().unwrap().layout.enclosures.iter().find(|e| e.name == "+MD").unwrap().items.iter().find(|i| i.tag == "-F1").unwrap().clone();
    assert_eq!(it.part, "GEN.MCB-1P-B16");
    // Put in again: it moves.
    run(&mut s, "place_cabinet_device", json!({ "enclosure": "+MD", "tag": "-F1", "x": 20, "y": 120 })).unwrap();
    assert_eq!(s.project().unwrap().layout.enclosures.iter().flat_map(|e| &e.items).filter(|i| i.tag == "-F1").count(), 1);
    assert!(run(&mut s, "place_cabinet_device", json!({ "enclosure": "+MD", "tag": "-NOPE" })).is_err());
    assert!(run(&mut s, "place_cabinet_device", json!({ "enclosure": "+ROOM1", "tag": "-F1" })).is_err());
    let a = run(&mut s, "place_installation", json!({ "breaker": "-F1", "room": "+ROOM1", "part": "GEN.SOCKET-F-UP", "x": 2000, "z": 3990 })).unwrap();
    let b = run(&mut s, "place_installation", json!({ "breaker": "-F1", "room": "+ROOM1", "part": "GEN.SOCKET-F-UP", "x": 3000, "z": 3990 })).unwrap();
    assert_eq!((a["device"].as_str(), a["cable"].as_str(), a["from"].as_str()), (Some("+ROOM1-XD1"), Some("-W1"), Some("-F1")));
    assert_eq!((b["cable"].as_str(), b["from"].as_str()), (Some("-W2"), Some("+ROOM1-XD1")));
    let (l, c) = legs(&mut s, "-F1");
    let want = |a: &str, b: &str, w: &str| (a.to_string(), b.to_string(), w.to_string());
    assert_eq!(l, [want("-F1", "+ROOM1-XD1", "-W1"), want("+ROOM1-XD1", "+ROOM1-XD2", "-W2")], "{c}");
    assert!(c["total_drop_percent"].as_f64().is_some_and(|v| v > 0.0), "{c}");
    // Each cable its own cores: the phase conductor of -W1 from the breaker (its neutral
    // and protective conductor come from the potentials of the board), all of -W2.
    let prj = s.project().unwrap();
    let (nl, open) = (Netlist::build(prj, true), Netlist::build(prj, false));
    let conns = openecraft_core::netlist::connections(prj, &nl, &open);
    let cores = |w: &str| conns.iter().filter(|c| c.cable == w).map(|c| c.core.clone()).collect::<Vec<_>>();
    assert_eq!(cores("-W1"), ["BN"]);
    assert_eq!(cores("-W2"), ["BN", "BU", "GNYE"]);
}

#[test]
fn circuits_branch_at_junction_boxes() {
    let mut s = house();
    let put = |s: &mut Session, part: &str, x: f32, from: Value| run(s, "place_installation", json!({ "breaker": "-F1", "room": "+ROOM1", "part": part, "x": x, "z": 3990, "from": from })).unwrap();
    let jb = put(&mut s, "GEN.JBOX-UP", 1500.0, Value::Null);
    assert_eq!(jb["device"], "+ROOM1-XD1");
    let a = put(&mut s, "GEN.SOCKET-F-UP", 1000.0, jb["device"].clone());
    let b = put(&mut s, "GEN.SOCKET-F-UP", 3000.0, jb["device"].clone());
    let c = put(&mut s, "GEN.SOCKET-F-UP", 4000.0, Value::Null);
    let (l, _) = legs(&mut s, "-F1");
    let has = |x: &Value, y: &Value| l.iter().any(|(f, t, _)| f == x.as_str().unwrap() && t == y.as_str().unwrap());
    assert!(has(&json!("-F1"), &jb["device"]) && has(&jb["device"], &a["device"]) && has(&jb["device"], &b["device"]), "{l:?}");
    // Without one, from the device before it.
    assert!(has(&b["device"], &c["device"]), "{l:?}");
    assert_eq!(l.len(), 4);
    assert!(run(&mut s, "place_installation", json!({ "breaker": "-F1", "room": "+ROOM1", "part": "GEN.SOCKET-F-UP", "x": 2000, "z": 3990, "from": "-NOPE" })).is_err());
}

#[test]
fn three_phase_outlets_and_fixed_equipment() {
    let mut s = house();
    let cee = run(&mut s, "place_installation", json!({ "breaker": "-F2", "room": "+ROOM1", "part": "GEN.SOCKET-CEE-16-5P", "x": 2500, "z": 3990 })).unwrap();
    let cooker = run(&mut s, "place_installation", json!({ "breaker": "-F2", "room": "+ROOM1", "part": "GEN.CONN-BOX-COOKER-UP", "x": 4990, "z": 2000 })).unwrap();
    let (l, c) = legs(&mut s, "-F2");
    assert_eq!(l.len(), 2, "{c}");
    assert_eq!(l[0].0, "-F2");
    assert_eq!(c["three_phase"], true);
    let prj = s.project().unwrap();
    let cable = prj.cables.iter().find(|x| x.tag == cee["cable"].as_str().unwrap()).unwrap();
    assert_eq!(cable.cores, "BN BK GY BU GNYE");
    assert!(cable.cable_type.starts_with("NYM-J 5x"), "{}", cable.cable_type);
    // On the installation page: the three phase conductors from -F2.
    let page = prj.pages.iter().find(|pg| pg.symbols.iter().any(|x| x.tag == cooker["device"].as_str().unwrap())).unwrap();
    let intr: Vec<&str> = page.symbols.iter().filter(|x| x.sym == "intr").map(|x| x.label.as_str()).collect();
    assert_eq!(intr, ["L1-F2", "L2-F2", "L3-F2"]);
    // Not on a single-phase circuit.
    let e = run(&mut s, "place_installation", json!({ "breaker": "-F1", "room": "+ROOM1", "part": "GEN.SOCKET-CEE-16-5P", "x": 2500, "z": 3990 })).unwrap_err();
    assert!(e.contains("three-phase"), "{e}");
    // A three-phase sauna heater too.
    let h = run(&mut s, "place_installation", json!({ "breaker": "-F2", "room": "+ROOM1", "part": "GEN.HEATER-SAUNA-9000", "x": 10, "z": 2000 })).unwrap();
    assert!(h["device"].as_str().unwrap().contains("-EB"), "{h}");
}

#[test]
fn outlets_and_alarms_without_a_circuit_and_devices_outside() {
    let mut s = house();
    for (part, x, z, face) in [("GEN.DATA-RJ45-2X-UP", 1000, 3990, "wall"), ("GEN.TV-SAT-UP", 1500, 3990, "wall"), ("GEN.SMOKE-ALARM", 2500, 2000, "ceiling")] {
        let r = run(&mut s, "place_installation", json!({ "room": "+ROOM1", "part": part, "x": x, "z": z })).unwrap();
        assert!(r["face"].as_str().unwrap().starts_with(face) && r["device"].is_null(), "{part}: {r}");
    }
    let e = run(&mut s, "place_installation", json!({ "breaker": "-F1", "room": "+ROOM1", "part": "GEN.SMOKE-ALARM", "x": 2500, "z": 2000 })).unwrap_err();
    assert!(e.contains("mains"), "{e}");
    // Outside the room: on the outside of its nearest wall, in the schematic.
    let out = run(&mut s, "place_installation", json!({ "breaker": "-F1", "room": "+ROOM1", "part": "GEN.SOCKET-F-LID-UP", "x": 2500, "z": 4300 })).unwrap();
    assert!(out["face"].as_str().unwrap().starts_with("outside"), "{out}");
    let light = run(&mut s, "place_installation", json!({ "breaker": "-F1", "room": "+ROOM1", "part": "GEN.LAMP-CEILING-LED", "x": -300, "z": 2000 })).unwrap();
    assert!(light["face"].as_str().unwrap().starts_with("outside"), "{light}");
}

#[test]
fn devices_with_their_tags_cables_and_heights() {
    let mut s = house();
    let r = run(&mut s, "place_installation", json!({
        "breaker": "-F1", "room": "+ROOM1", "part": "GEN.SOCKET-F-UP", "x": 2000, "z": 3990,
        "tag": "+ROOM1-XD10", "cable": "-W100", "cores": 4, "cross_section": 4, "cable_part": "CAB.NYM-4X4", "length": 12.5, "height": 1100,
    })).unwrap();
    assert_eq!((r["device"].as_str(), r["cable"].as_str()), (Some("+ROOM1-XD10"), Some("-W100")));
    let prj = s.project().unwrap();
    let cable = prj.cables.iter().find(|x| x.tag == "-W100").unwrap();
    assert_eq!((cable.cores.as_str(), cable.cable_type.as_str(), cable.part.as_str(), cable.length), ("BN BK BU GNYE", "NYM-J 4x4", "CAB.NYM-4X4", 12.5));
    let poses = panel::site_poses(prj);
    let ri = prj.layout.enclosures.iter().position(|e| e.name == "+ROOM1").unwrap();
    let it = prj.layout.enclosures[ri].items.iter().find(|i| i.tag == "+ROOM1-XD10").unwrap();
    let placed = inst::placed(prj, &poses, ri, it).unwrap();
    assert!((placed.height - 1100.0).abs() < 50.0, "{}", placed.height);
    // The known length is taken and kept.
    let (_, c) = legs(&mut s, "-F1");
    assert_eq!(c["connections"][0]["length"].as_f64(), Some(12.5), "{c}");
    run(&mut s, "write_circuit_cables", json!({})).unwrap();
    assert_eq!(s.project().unwrap().cables.iter().find(|x| x.tag == "-W100").unwrap().length, 12.5);
    // Tags in use and too few cores are refused.
    for extra in [json!({ "tag": "+ROOM1-XD10" }), json!({ "cable": "-W100" }), json!({ "cores": 2 })] {
        let mut args = json!({ "breaker": "-F1", "room": "+ROOM1", "part": "GEN.SOCKET-F-UP", "x": 3000, "z": 3990 });
        args.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        assert!(run(&mut s, "place_installation", args).is_err());
    }
}

/// The house with its +ROOM1 on the ground level of a building, the level's id.
fn house_on_a_level() -> (Session, u64) {
    let mut s = house();
    let prj = s.project_mut().unwrap();
    let b = openecraft_core::panel::buildings::add_building(prj, "+H1").unwrap();
    let level = prj.layout.buildings[b].levels[0].id;
    let room = prj.layout.enclosures.iter().position(|e| e.name == "+ROOM1").unwrap();
    prj.layout.enclosures[room].level = level;
    (s, level)
}

#[test]
fn installation_plan_pages_take_their_rooms() {
    let (mut s, level) = house_on_a_level();
    let id = s.project_mut().unwrap().new_id();
    s.project_mut().unwrap().pages.push(openecraft_core::model::Page::new(id, "IP1", "", openecraft_core::model::PageKind::InstallationPlan(level)));
    run(&mut s, "set_page", json!({ "page": "IP1", "rooms": "+ROOM1", "tag_parts": "location,product" })).unwrap();
    let pg = s.project().unwrap().pages.iter().find(|p| p.name == "IP1").unwrap();
    assert_eq!(pg.plan.rooms, ["+ROOM1"]);
    assert!(run(&mut s, "set_page", json!({ "page": "IP1", "rooms": "+NOWHERE" })).is_err());
    run(&mut s, "set_page", json!({ "page": "IP1", "rooms": "" })).unwrap();
    assert!(s.project().unwrap().pages.iter().find(|p| p.name == "IP1").unwrap().plan.rooms.is_empty());
}

#[test]
fn installation_plan_pages_take_legend_and_text_options() {
    let mut s = house();
    let level = s.project().unwrap().layout.enclosures.iter().find(|e| e.name == "+ROOM1").unwrap().level;
    let id = s.project_mut().unwrap().new_id();
    s.project_mut().unwrap().pages.push(openecraft_core::model::Page::new(id, "IP1", "", openecraft_core::model::PageKind::InstallationPlan(level)));
    run(&mut s, "set_page", json!({ "page": "IP1", "legend": false, "legend_at": "bottom", "room_descriptions": false, "tag_size": 2.5, "room_size": 4, "legend_size": 3 })).unwrap();
    let plan = &s.project().unwrap().pages.iter().find(|p| p.name == "IP1").unwrap().plan;
    assert!(!plan.legend && !plan.room_descriptions);
    assert_eq!((plan.legend_at, plan.tag_size, plan.room_size, plan.legend_size), (openecraft_core::model::LegendAt::Bottom, 2.5, 4.0, 3.0));
    assert!(run(&mut s, "set_page", json!({ "page": "IP1", "legend_at": "middle" })).is_err());
    assert!(run(&mut s, "set_page", json!({ "page": "IP1", "tag_size": 0.1 })).is_err());
}

#[test]
fn installation_plan_pages_take_only_rooms_of_their_level() {
    use openecraft_core::panel::buildings;
    let (mut s, level) = house_on_a_level();
    let prj = s.project_mut().unwrap();
    let bi = buildings::building_index(&prj.layout, "+H1").unwrap();
    let attic = buildings::add_level(prj, bi, Some("AT"), None, None).unwrap();
    buildings::add_room(prj, "+AT1", Some(attic)).unwrap();
    let id = prj.new_id();
    prj.pages.push(openecraft_core::model::Page::new(id, "IP1", "", openecraft_core::model::PageKind::InstallationPlan(level)));
    let err = run(&mut s, "set_page", json!({ "page": "IP1", "rooms": "+AT1" })).unwrap_err();
    assert!(err.contains("+AT1"), "{err}");
    run(&mut s, "set_page", json!({ "page": "IP1", "rooms": "+ROOM1" })).unwrap();
}

#[test]
fn a_wrong_plan_option_leaves_the_page_as_it_was() {
    let mut s = house();
    let level = s.project().unwrap().layout.enclosures.iter().find(|e| e.name == "+ROOM1").unwrap().level;
    let id = s.project_mut().unwrap().new_id();
    s.project_mut().unwrap().pages.push(openecraft_core::model::Page::new(id, "IP1", "", openecraft_core::model::PageKind::InstallationPlan(level)));
    assert!(run(&mut s, "set_page", json!({ "page": "IP1", "legend": false, "legend_at": "middle" })).is_err());
    assert!(s.project().unwrap().pages.iter().find(|p| p.name == "IP1").unwrap().plan.legend);
    // Plan options are refused on other pages.
    let first = s.project().unwrap().pages[0].name.clone();
    assert!(run(&mut s, "set_page", json!({ "page": first, "legend": false })).is_err());
}
