use super::*;
use crate::testing::*;

#[test]
fn devices_are_renamed_and_get_parts() {
    let mut s = new_session();
    run(&mut s, "place_symbol", json!({ "page": "1", "symbol": "coil", "x": 100, "y": 100 })).unwrap();
    run(&mut s, "place_symbol", json!({ "page": "1", "symbol": "no", "x": 60, "y": 60, "tag": "-K1" })).unwrap();
    run(&mut s, "place_symbol", json!({ "page": "1", "symbol": "coil", "x": 140, "y": 100 })).unwrap();
    assert!(run(&mut s, "rename_device", json!({ "old": "-K1", "new": "-K2" })).unwrap_err().contains("already exists"));
    run(&mut s, "rename_device", json!({ "old": "-K1", "new": "-K10" })).unwrap();
    let devices = run(&mut s, "list_devices", json!({})).unwrap();
    let k10 = devices.as_array().unwrap().iter().find(|d| d["tag"] == "-K10").unwrap();
    assert_eq!(k10["main"], json!(["/1.3"]));
    assert_eq!(k10["auxiliary"].as_array().unwrap().len(), 1);

    let number = parts::catalog()[0].number.clone();
    run(&mut s, "assign_part", json!({ "tag": "-K10", "number": number })).unwrap();
    let out = run(&mut s, "assign_part", json!({ "tag": "-K10", "number": number, "count": 2 })).unwrap();
    assert_eq!(out["parts"], json!([{ "number": number, "count": 3 }]));
    assert!(s.project().unwrap().parts.iter().any(|p| p.number == number));
    assert!(run(&mut s, "assign_part", json!({ "tag": "-K99", "number": number })).is_err());
}

#[test]
fn an_unknown_part_gets_suggestions() {
    let mut s = new_session();
    run(&mut s, "place_symbol", json!({ "page": "1", "symbol": "coil", "x": 100, "y": 100, "tag": "-K1" })).unwrap();
    let err = run(&mut s, "assign_part", json!({ "tag": "-K1", "number": "CBL-4G1.5" })).unwrap_err();
    assert!(err.contains("similar parts") && err.contains("GEN.CBL-4G1.5"), "{err}");
    let err = run(&mut s, "assign_part", json!({ "tag": "-K1", "number": "zzz" })).unwrap_err();
    assert!(!err.contains("similar"), "{err}");
}
