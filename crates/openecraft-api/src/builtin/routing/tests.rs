use super::*;
use crate::builtin::testing::*;

fn demo() -> Session {
    let mut s = demo_session();
    s.project_mut().unwrap().info.name = "Demo".into();
    s
}

#[test]
fn rules_paths_and_reservations() {
    let mut s = demo();
    let r = run(&mut s, "routing_rules", json!({ "grid": 20, "segregation": 150, "wires_3d": true })).unwrap();
    assert_eq!((r["grid"].as_f64(), r["segregation"].as_f64(), r["wires_3d"].as_bool()), (Some(20.0), Some(150.0), Some(true)));
    let p = run(&mut s, "add_routing_path", json!({ "enclosure": "+CAB1", "points": [[300, 50], [300, 700]], "kind": "cables", "classes": "signal, control", "width": 40 })).unwrap();
    assert_eq!((p["kind"].as_str(), p["points"][0][2].as_f64()), (Some("Cables"), Some(70.0)));
    assert!(run(&mut s, "add_routing_path", json!({ "enclosure": "+CAB1", "points": [[1, 1]] })).unwrap_err().contains("2 points"));
    assert!(run(&mut s, "add_routing_path", json!({ "enclosure": "+CAB1", "points": [[1, 1], [2, 2]], "classes": "mains" })).unwrap_err().contains("Unknown class"));
    let res = run(&mut s, "add_routing_reservation", json!({ "enclosure": "+CAB1", "x": 300, "y": 600, "w": 200, "h": 100, "mode": "power" })).unwrap();
    assert_eq!(res["mode"]["For"], "Power");
    let list = run(&mut s, "list_routing", json!({})).unwrap();
    assert!(list["paths"].as_array().unwrap().iter().any(|x| x["id"] == p["id"]));
    let id = res["id"].as_u64().unwrap();
    run(&mut s, "remove_routing_object", json!({ "enclosure": "+CAB1", "id": id })).unwrap();
    assert!(run(&mut s, "remove_routing_object", json!({ "enclosure": "+CAB1", "id": id })).unwrap_err().contains("No routing path"));
}

#[test]
fn routing_list_freezing_and_shields() {
    let mut s = demo();
    let list = run(&mut s, "list_routing", json!({ "kind": "cable" })).unwrap();
    let items = list["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!((items[0]["key"].as_str(), items[0]["class"].as_str()), (Some("-W1"), Some("power")));
    assert!(items[0]["cut_mm"].as_f64().unwrap() >= items[0]["length_mm"].as_f64().unwrap());
    let all = run(&mut s, "list_routing", json!({})).unwrap();
    assert!(!all["bundles"].as_array().unwrap().is_empty());
    let f = run(&mut s, "freeze_routes", json!({ "keys": "-W1" })).unwrap();
    assert_eq!((f["frozen"].as_u64(), f["total"].as_u64()), (Some(1), Some(1)));
    assert!(run(&mut s, "freeze_routes", json!({ "keys": "-W99" })).unwrap_err().contains("No routed item"));
    let list = run(&mut s, "list_routing", json!({ "kind": "cable" })).unwrap();
    assert_eq!(list["items"][0]["status"], "frozen");
    let r = run(&mut s, "freeze_routes", json!({ "release": true })).unwrap();
    assert_eq!(r["released"], 1);
    let c = run(&mut s, "set_cable_shield", json!({ "cable": "-W1", "shielded": true, "ends": "first", "diameter": 11.5 })).unwrap();
    assert_eq!((c["shielded"].as_bool(), c["ends"].as_str(), c["diameter_mm"].as_f64()), (Some(true), Some("first"), Some(11.5)));
    let list = run(&mut s, "list_routing", json!({})).unwrap();
    let shield = &list["shields"][0];
    assert_eq!(shield["cable"], "-W1");
    assert_eq!(shield["ends"][1]["termination"], "not connected");
    assert!(run(&mut s, "set_cable_shield", json!({ "cable": "-W1", "ends": "middle" })).unwrap_err().contains("Unknown shield ends"));
}

#[test]
fn laying_and_installation_zones() {
    let mut s = demo();
    let r = run(&mut s, "routing_rules", json!({ "laying": "wall", "slot_depth": 40 })).unwrap();
    assert_eq!((r["laying"].as_str(), r["slot_depth"].as_f64(), r["floor_depth"].as_f64()), (Some("Wall"), Some(40.0), Some(60.0)));
    assert!(run(&mut s, "routing_rules", json!({ "laying": "air" })).unwrap_err().contains("air"));
    // One cable in the floor, back to the default.
    let l = run(&mut s, "set_laying", json!({ "item": "-W1", "laying": "floor" })).unwrap();
    assert_eq!((l["default"].as_str(), l["items"]["-W1"].as_str()), (Some("wall"), Some("floor")));
    let l = run(&mut s, "set_laying", json!({ "item": "-W1", "laying": "default" })).unwrap();
    assert_eq!(l["items"], json!({}));
    // The zones of DIN 18015-3; the middle one switched on, a zone of its own, reset.
    let z = run(&mut s, "installation_zones", json!({})).unwrap();
    let names: Vec<&str> = z.as_array().unwrap().iter().map(|z| z["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["ZW-o", "ZW-m", "ZW-u", "ZS-t", "ZS-f", "ZS-e", "ZD-r", "ZF"]);
    let z = run(&mut s, "installation_zones", json!({ "name": "zw-m", "on": true })).unwrap();
    assert_eq!(z[1]["on"], true);
    let z = run(&mut s, "installation_zones", json!({ "name": "ZX", "kind": "above_floor", "from": 600, "to": 800 })).unwrap();
    assert_eq!((z[8]["kind"].as_str(), z[8]["to"].as_f64()), (Some("above_floor"), Some(800.0)));
    assert!(run(&mut s, "installation_zones", json!({ "name": "ZX", "from": 900 })).unwrap_err().contains("ZX"));
    let z = run(&mut s, "installation_zones", json!({ "reset": true })).unwrap();
    assert_eq!((z.as_array().unwrap().len(), z[1]["on"].as_bool()), (8, Some(false)));
}
