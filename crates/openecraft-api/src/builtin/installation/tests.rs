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
