use super::*;
use crate::builtin::testing::*;

#[test]
fn buildings_with_levels_rooms_and_shafts() {
    let mut s = demo_session();
    let b = run(&mut s, "add_building", json!({ "name": "+B1", "x": 20000, "z": 0 })).unwrap();
    assert_eq!(b["levels"][0]["name"], "L0");
    assert!(run(&mut s, "add_building", json!({ "name": "+B1" })).unwrap_err().contains("exists"));
    run(&mut s, "add_level", json!({ "building": "+B1" })).unwrap();
    let b = run(&mut s, "add_level", json!({ "building": "B1", "elevation": -3000, "height": 3000 })).unwrap();
    let names: Vec<&str> = b["levels"].as_array().unwrap().iter().map(|l| l["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["B1", "L0", "L1"]);
    assert!(run(&mut s, "add_level", json!({ "building": "+B1", "elevation": 1000 })).unwrap_err().contains("overlap"));
    // Rooms on the levels: as high as the level less its slab, side by side.
    let r = run(&mut s, "add_room", json!({ "name": "+R0", "building": "+B1" })).unwrap();
    assert_eq!((r["building"].as_str(), r["level"].as_str()), (Some("+B1"), Some("L0")));
    assert_eq!(r["size"]["height"], 2700.0);
    assert_eq!(r["at"], json!([20000.0, 0.0, 0.0]));
    let r = run(&mut s, "add_room", json!({ "name": "+R1", "building": "+B1", "level": "L1", "x": 0, "z": 0 })).unwrap();
    assert_eq!(r["at"], json!([20000.0, 3000.0, 0.0]));
    let r = run(&mut s, "add_room", json!({ "name": "+R2", "building": "+B1", "level": "L0", "width": 2000 })).unwrap();
    assert_eq!(r["at"], json!([24000.0, 0.0, 0.0]), "right of +R0");
    // A shaft through all levels joins the rooms it passes.
    let b = run(&mut s, "add_shaft", json!({ "building": "+B1", "x": 3000, "z": 2000 })).unwrap();
    assert_eq!(b["shafts"][0]["name"], "S1");
    assert_eq!((b["shafts"][0]["bottom"].as_str(), b["shafts"][0]["top"].as_str()), (Some("B1"), Some("L1")));
    assert_eq!(b["shafts"][0]["rooms"], json!(["+R0", "+R1"]));
    let b = run(&mut s, "set_shaft", json!({ "building": "+B1", "shaft": "s1", "bottom": "L0", "width": 400 })).unwrap();
    assert_eq!((b["shafts"][0]["bottom"].as_str(), b["shafts"][0]["width"].as_f64()), (Some("L0"), Some(400.0)));
    assert!(run(&mut s, "set_shaft", json!({ "building": "+B1", "shaft": "S1", "width": 10 })).is_err());
    let list = run(&mut s, "list_buildings", json!({})).unwrap();
    assert_eq!(list["problems"], json!([]));
    assert_eq!(list["buildings"][0]["levels"][1]["rooms"].as_array().unwrap().len(), 2);
    // The building moves, its rooms with it.
    run(&mut s, "set_building", json!({ "building": "+B1", "x": 0 })).unwrap();
    let r = run(&mut s, "list_rooms", json!({ "room": "+R1" })).unwrap();
    assert_eq!(r[0]["at"], json!([0.0, 3000.0, 0.0]));
    // Out of the building: it stays where it is.
    let r = run(&mut s, "place_on_level", json!({ "room": "+R1" })).unwrap();
    assert_eq!(r["building"], Value::Null);
    let r = run(&mut s, "list_rooms", json!({ "room": "+R1" })).unwrap();
    assert_eq!((r[0]["at"].clone(), r[0]["building"].clone()), (json!([0.0, 3000.0, 0.0]), Value::Null));
    let b = run(&mut s, "place_on_level", json!({ "room": "+R1", "building": "+B1", "level": "3", "x": 0, "z": 0 })).unwrap();
    assert_eq!(b["levels"][2]["rooms"][0]["room"], "+R1");
    run(&mut s, "set_level", json!({ "building": "+B1", "level": "B1", "remove": true })).unwrap();
    run(&mut s, "set_shaft", json!({ "building": "+B1", "shaft": "S1", "remove": true })).unwrap();
    let r = run(&mut s, "set_building", json!({ "building": "+B1", "remove": true })).unwrap();
    assert_eq!(r["removed"], "+B1");
}

#[test]
fn links_between_buildings() {
    let mut s = demo_session();
    for k in 1..=2 {
        run(&mut s, "add_building", json!({ "name": format!("+B{k}") })).unwrap();
        run(&mut s, "add_room", json!({ "name": format!("+R{k}"), "building": format!("+B{k}") })).unwrap();
    }
    assert!(run(&mut s, "add_site_link", json!({ "from": "+R1", "to": "+R2" })).unwrap_err().contains("building entry"));
    for k in 1..=2 {
        run(&mut s, "add_wall_opening", json!({ "room": format!("+R{k}"), "kind": "entry", "wall": "wall3" })).unwrap();
    }
    let l = run(&mut s, "add_site_link", json!({ "from": "+R1", "to": "+R2", "kind": "conduit", "points": [[2000, 6000]] })).unwrap();
    assert_eq!((l["name"].as_str(), l["kind"].as_str(), l["to"].as_str()), (Some("SL1"), Some("conduit"), Some("+R2")));
    assert!(l["length"].as_f64().unwrap() > 10_000.0, "{l}");
    let l = run(&mut s, "set_site_link", json!({ "link": "sl1", "kind": "bridge", "length": 25000 })).unwrap();
    assert_eq!((l["kind"].as_str(), l["length"].as_f64()), (Some("bridge"), Some(25000.0)));
    assert!(run(&mut s, "set_site_link", json!({ "link": "SL1", "kind": "rope" })).is_err());
    let list = run(&mut s, "list_buildings", json!({})).unwrap();
    assert_eq!(list["links"].as_array().unwrap().len(), 1);
    assert_eq!(list["problems"], json!([]));
    run(&mut s, "set_site_link", json!({ "link": "SL1", "remove": true })).unwrap();
}

#[test]
fn devices_on_the_facade_and_the_roof() {
    let mut s = demo_session();
    run(&mut s, "add_building", json!({ "name": "+B1" })).unwrap();
    run(&mut s, "add_room", json!({ "name": "+R1", "building": "+B1" })).unwrap();
    let r = run(&mut s, "list_rooms", json!({ "room": "+R1" })).unwrap();
    assert_eq!(r[0]["problems"], json!([]));
    // A contactor needs a rail outside too.
    assert!(run(&mut s, "mount_on_wall", json!({ "room": "+R1", "device": "-K1", "face": "outside3", "x": 1000, "y": 500 })).unwrap_err().contains("wall mounting"));
    run(&mut s, "add_wall_profile", json!({ "room": "+R1", "kind": "rail", "face": "outside3", "x1": 900, "y1": 540, "x2": 1500, "y2": 540 })).unwrap();
    let m = run(&mut s, "mount_on_wall", json!({ "room": "+R1", "device": "-K1", "face": "outside3", "x": 1000, "y": 500 })).unwrap();
    assert_eq!(m["on_rail"], true);
    let r = run(&mut s, "list_rooms", json!({ "room": "+R1" })).unwrap();
    let faces: Vec<&str> = r[0]["faces"].as_array().unwrap().iter().map(|f| f["face"].as_str().unwrap()).collect();
    assert_eq!(faces, ["outside3"]);
    assert_eq!(r[0]["problems"], json!([]));
    assert!(run(&mut s, "mount_on_wall", json!({ "room": "+R1", "device": "-K2", "face": "outside9", "x": 0, "y": 0 })).unwrap_err().contains("Unknown face"));
}
