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
fn circuits_with_lengths_and_voltage_drops() {
    let (mut s, room, at) = room_session();
    let breakers = run(&mut s, "list_breakers", json!({})).unwrap();
    let f2 = breakers.as_array().unwrap().iter().find(|b| b["breaker"].as_str().unwrap().ends_with("-F2")).unwrap().clone();
    assert_eq!((f2["rated"].as_f64(), f2["spare"].as_bool()), (Some(10.0), Some(false)));
    let spare = run(&mut s, "list_breakers", json!({ "spare": true })).unwrap();
    assert!(spare.as_array().unwrap().iter().all(|b| b["spare"] == true));
    let c = run(&mut s, "add_circuit", json!({ "name": "Room sockets", "breaker": f2["breaker"] })).unwrap();
    assert_eq!(c[0]["cable_type"], inst::DEFAULT_CABLE);
    // A light in the middle of the ceiling, a socket on a wall near a corner.
    let q = at(s.project().unwrap(), 0.5, 0.5);
    let light = run(&mut s, "place_installation", json!({ "room": room, "part": "GEN.LAMP-CEILING-LED", "x": q[0], "z": q[1], "circuit": "Room sockets" })).unwrap();
    assert_eq!(light["face"], "ceiling");
    let q = at(s.project().unwrap(), 0.02, 0.5);
    let socket = run(&mut s, "place_installation", json!({ "room": room, "part": "GEN.SOCKET-F-UP", "x": q[0], "z": q[1] })).unwrap();
    assert!(socket["face"].as_str().unwrap().starts_with("wall"), "{socket}");
    let c = run(&mut s, "set_circuit", json!({ "circuit": "Room sockets", "add": [socket["id"]], "cross_section": 2.5 })).unwrap();
    assert_eq!(c[0]["items"].as_array().unwrap().len(), 2);
    assert_eq!(c[0]["items"][1]["kind"], "socket");
    assert!(c[0]["length"].as_f64().unwrap() > 1.0);
    assert!(c[0]["drop_percent"].as_f64().unwrap() > 0.0);
    assert_eq!(c[0]["cross_section"], 2.5);
    assert!(run(&mut s, "set_circuit", json!({ "circuit": "Room sockets", "cross_section": 0 })).is_err());
    assert!(run(&mut s, "set_circuit", json!({ "circuit": "Nope" })).is_err());
    // The cable list gets its length.
    let w = run(&mut s, "write_circuit_cables", json!({})).unwrap();
    assert_eq!(w["cables"], 1);
    let tag = w["circuits"][0]["cable"].as_str().unwrap().to_string();
    let prj = s.project().unwrap();
    assert!(prj.cables.iter().any(|c| c.tag == tag && c.length > 0.0));
    // The report and the check see it.
    let list = run(&mut s, "list_circuits", json!({ "circuit": "Room sockets" })).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    let c = run(&mut s, "set_circuit", json!({ "circuit": "Room sockets", "remove": [light["id"]] })).unwrap();
    assert_eq!(c[0]["items"].as_array().unwrap().len(), 1);
    run(&mut s, "set_circuit", json!({ "circuit": "Room sockets", "remove_circuit": true })).unwrap();
    assert!(s.project().unwrap().layout.circuits.is_empty());
}

/// The electrical room example with a socket outlet `+ER1-XD7` (no part) behind a new
/// breaker -F5 of 10 A on the infeed page.
fn with_socket_circuit() -> (Session, String, impl Fn(&Project, f32, f32) -> [f32; 2]) {
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
    (with_project(prj), name, at)
}

#[test]
fn devices_of_schematic_circuits_are_placed() {
    let (mut s, room, at) = with_socket_circuit();
    let list = run(&mut s, "list_schematic_circuits", json!({})).unwrap();
    let f5 = list.as_array().unwrap().iter().find(|c| c["breaker"] == "=SUP+CAB1-F5").unwrap_or_else(|| panic!("{list}")).clone();
    assert_eq!((f5["devices"][0]["device"].as_str(), f5["devices"][0]["kind"].as_str(), f5["devices"][0]["part"].as_str()), (Some("+ER1-XD7"), Some("socket"), Some("GEN.SOCKET-F-UP")));
    assert!(f5["circuit"].is_null() && f5["devices"][0]["room"].is_null());
    assert!(run(&mut s, "list_schematic_circuits", json!({ "breaker": "-NOPE" })).is_err());
    // Placed by a point of the plant: into a new final circuit of -F5.
    let q = at(s.project().unwrap(), 0.02, 0.5);
    let r = run(&mut s, "place_circuit_device", json!({ "device": "+ER1-XD7", "room": room, "x": q[0], "z": q[1] })).unwrap();
    assert!(r["face"].as_str().unwrap().starts_with("wall"), "{r}");
    assert_eq!(r["circuit"], "=SUP+CAB1-F5");
    let c = run(&mut s, "list_circuits", json!({ "circuit": "=SUP+CAB1-F5" })).unwrap();
    assert_eq!(c[0]["items"][0]["id"], r["id"]);
    let list = run(&mut s, "list_schematic_circuits", json!({ "breaker": "=SUP+CAB1-F5", "unplaced": true })).unwrap();
    assert!(list[0]["devices"].as_array().unwrap().is_empty() && list[0]["placed"] == 1);
    // A socket outlet placed before becomes the device (of its part now); the first
    // one goes.
    let q = at(s.project().unwrap(), 0.98, 0.5);
    let other = run(&mut s, "place_installation", json!({ "room": room, "part": "GEN.SOCKET-F-2X-UP", "x": q[0], "z": q[1] })).unwrap();
    assert!(run(&mut s, "place_circuit_device", json!({ "device": "+ER1-XD7", "item": other["id"] })).is_err());
    let q = at(s.project().unwrap(), 0.5, 0.02);
    let free = run(&mut s, "place_installation", json!({ "room": room, "part": "GEN.SOCKET-F-UP", "x": q[0], "z": q[1] })).unwrap();
    let r2 = run(&mut s, "place_circuit_device", json!({ "device": "+ER1-XD7", "item": free["id"] })).unwrap();
    assert_eq!(r2["id"], free["id"]);
    let prj = s.project().unwrap();
    assert_eq!(prj.layout.enclosures.iter().flat_map(|e| e.items.iter()).filter(|it| it.tag == "+ER1-XD7").count(), 1);
    assert_eq!(prj.layout.circuits[0].items, vec![free["id"].as_u64().unwrap()]);
    assert!(run(&mut s, "place_circuit_device", json!({ "device": "-NOPE", "room": room })).is_err());
}

#[test]
fn planned_breakers_and_the_single_line_page() {
    let (mut s, room, at) = room_session();
    run(&mut s, "add_circuit", json!({ "name": "Lights" })).unwrap();
    let q = at(s.project().unwrap(), 0.5, 0.5);
    run(&mut s, "place_installation", json!({ "room": room, "part": "GEN.LAMP-CEILING-LED", "x": q[0], "z": q[1], "circuit": "Lights" })).unwrap();
    assert!(run(&mut s, "new_circuit_breaker", json!({ "circuit": "Lights", "rated": 20 })).is_err());
    let c = run(&mut s, "new_circuit_breaker", json!({ "circuit": "Lights" })).unwrap();
    assert!(c[0]["breaker"].as_str().unwrap().starts_with("-F"));
    assert_eq!(c[0]["rated"], 16.0);
    assert!(c[0]["total_drop_percent"].is_number() || c[0]["board"].is_null());
    let o = run(&mut s, "installation_overview", json!({})).unwrap();
    assert_eq!(o["pages"].as_array().unwrap().len(), 1);
    let prj = s.project().unwrap();
    assert!(prj.layout.enclosures.iter().flat_map(|e| e.items.iter()).any(|it| it.tag.ends_with("-EA1")));
}
