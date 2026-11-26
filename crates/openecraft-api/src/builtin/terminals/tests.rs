use super::*;
use crate::builtin::testing::*;

#[test]
fn new_strips_and_unplaced_terminals_are_available_through_the_registry() {
    let mut s = new_session();
    assert!(run(&mut s, "list_strips", json!({})).unwrap().as_array().unwrap().is_empty());
    assert_eq!(run(&mut s, "create_strip", json!({ "strip": " -X1 " })).unwrap()["strip"], "-X1");
    assert_eq!(run(&mut s, "list_strips", json!({})).unwrap()[0]["strip"], "-X1");
    assert_eq!(s.project().unwrap().next_tag("", "X"), "-X2");
    for label in ["1", "2"] {
        let terminal = run(&mut s, "add_terminal", json!({ "strip": "-X1" })).unwrap();
        assert_eq!(terminal["terminal"], label);
        assert_eq!(terminal["spare"], true);
    }
    let strip = run(&mut s, "get_strip", json!({ "strip": "-X1" })).unwrap();
    assert_eq!(strip["terminals"].as_array().unwrap().len(), 2);
    assert!(strip["terminals"].as_array().unwrap().iter().all(|t| t["spare"] == true));
    let mut restored: Project = serde_json::from_str(&serde_json::to_string(s.project().unwrap()).unwrap()).unwrap();
    restored.normalize();
    assert_eq!(strips::strip_tags(&restored), ["-X1"]);
    assert_eq!(restored.strips["-X1"].spares, 2);
}

#[test]
fn strip_creation_rejects_invalid_or_used_tags_without_mutating_the_project() {
    let mut s = demo_session();
    let before = serde_json::to_value(s.project().unwrap()).unwrap();
    for tag in ["", "X2", "-X 2", "-X2;X3", "-X1", "-K1"] {
        assert!(run(&mut s, "create_strip", json!({ "strip": tag })).is_err(), "{tag}");
        assert_eq!(serde_json::to_value(s.project().unwrap()).unwrap(), before);
    }
    assert!(run(&mut s, "add_terminal", json!({ "strip": "-X404" })).is_err());
    assert_eq!(serde_json::to_value(s.project().unwrap()).unwrap(), before);
    assert_eq!(run(&mut s, "add_terminal", json!({ "strip": "-X1" })).unwrap()["terminal"], "4");
}

#[test]
fn strips_levels_jumpers_and_order() {
    let mut s = demo_session();
    let list = run(&mut s, "list_strips", json!({})).unwrap();
    assert_eq!(list[0]["strip"], "-X1");
    assert_eq!(list[0]["terminals"], 4);
    assert!(run(&mut s, "get_strip", json!({ "strip": "-X9" })).is_err());

    let strip = run(&mut s, "set_terminal", json!({ "strip": "-X1", "terminal": "2", "level": 2 })).unwrap();
    let blocks: Vec<Value> = strip["rail"].as_array().unwrap().iter().filter(|e| e["kind"] == "terminal").map(|e| e["terminals"].clone()).collect();
    assert_eq!(blocks[0], json!(["1", "2"]));
    assert_eq!(strip["terminals"][1]["level"], 2);
    assert!(run(&mut s, "set_terminal", json!({ "strip": "-X1", "terminal": "77", "level": 2 })).is_err());

    run(&mut s, "set_terminal", json!({ "strip": "-X1", "terminal": "1", "jumper": 5 })).unwrap();
    let strip = run(&mut s, "set_terminal", json!({ "strip": "-X1", "terminal": "2", "jumper": 5 })).unwrap();
    assert!(strip["jumpers"][0]["problem"].as_str().unwrap().contains("different levels"));
    let strip = run(&mut s, "set_jumper_kind", json!({ "strip": "-X1", "jumper": 5, "kind": "wire" })).unwrap();
    assert_eq!(strip["jumpers"][0]["kind"], "wire");
    assert!(strip["jumpers"][0]["problem"].is_null());
    assert!(run(&mut s, "set_jumper_kind", json!({ "strip": "-X1", "jumper": 5, "kind": "glue" })).is_err());

    let strip = run(&mut s, "set_strip", json!({ "strip": "-X1", "spares": 2, "order": "3,1,2,PE" })).unwrap();
    let order: Vec<&str> = strip["terminals"].as_array().unwrap().iter().map(|t| t["terminal"].as_str().unwrap()).collect();
    assert_eq!(&order[..4], ["3", "1", "2", "PE"]);
    assert_eq!(strip["terminals"].as_array().unwrap().len(), 6);
    assert!(run(&mut s, "set_strip", json!({ "strip": "-X1", "order": "9" })).is_err());
    let strip = run(&mut s, "renumber_strip", json!({ "strip": "-X1" })).unwrap();
    assert_eq!(strip["renumbered"], 3);
}
