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
