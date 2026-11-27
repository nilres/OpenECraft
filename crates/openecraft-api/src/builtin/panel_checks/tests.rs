use super::*;
use crate::{Origin, Plugins};
use openecraft_core::analysis::Analysis;
use openecraft_core::issue::Severity;
use openecraft_core::sample::demo_project;
use serde_json::Map;

fn run(p: &Plugins, prj: &mut Project, name: &str, args: Value) -> Result<Value, String> {
    let args: Map<String, Value> = args.as_object().cloned().unwrap_or_default();
    p.run(p.command(name).ok_or("no command")?, prj, &args)
}

fn findings(p: &Plugins, prj: &mut Project) -> Vec<Value> {
    run(p, prj, "check_panel", json!({})).unwrap()["findings"].as_array().unwrap().clone()
}

#[test]
fn check_is_part_of_the_project_check() {
    let p = Plugins::new(Vec::<String>::new());
    assert!(p.list().iter().any(|l| l.info.id == "panel_checks" && l.origin == Origin::BuiltIn));
    let mut prj = demo_project();
    let an = Analysis::build(&prj);
    assert!(p.check(&prj, &an).iter().all(|i| i.severity != Severity::Warning));
    assert_eq!(run(&p, &mut prj, "check_panel", json!({})).unwrap()["warnings"], 0);
    assert!(run(&p, &mut prj, "check_panel", json!({ "enclosure": "+CAB9" })).unwrap_err().contains("+CAB9"));
    // Two devices on top of each other: a collision in the project check.
    let e = &mut prj.layout.enclosures[0];
    let mut copy = e.items.iter().find(|i| i.tag == "-K1").unwrap().clone();
    copy.id = 99_999;
    copy.tag = "-K9".into();
    e.items.push(copy);
    let issues = p.check(&prj, &Analysis::build(&prj));
    assert!(issues.iter().any(|i| i.severity == Severity::Warning && i.message().contains("-K9")), "{issues:?}");
}

#[test]
fn rules_classes_and_heights() {
    let p = Plugins::new(Vec::<String>::new());
    let mut prj = demo_project();
    // Values out of range are refused, not corrected.
    let err = run(&p, &mut prj, "panel_rules", json!({ "device_gap": 30, "stratification": 5 })).unwrap_err();
    assert_eq!(err, "stratification must be between 1 and 2");
    let r = run(&p, &mut prj, "panel_rules", json!({ "device_gap": 30, "stratification": 2 })).unwrap();
    assert_eq!((r["device_gap"].as_f64(), r["stratification"].as_f64()), (Some(30.0), Some(2.0)));
    assert!(findings(&p, &mut prj).iter().any(|f| f["check"] == "mounting"));
    run(&p, &mut prj, "panel_rules", json!({ "reset": true })).unwrap();
    assert!(prj.layout.checks.is_default());
    // Contactor and PLC close together: a power / control rule of 50 mm.
    let r = run(&p, &mut prj, "set_collision_rule", json!({ "a": "power", "b": "control", "distance": 50 })).unwrap();
    assert_eq!(r["classes"].as_array().unwrap().len(), 2);
    assert!(findings(&p, &mut prj).iter().any(|f| f["check"] == "class_distance"));
    run(&p, &mut prj, "set_collision_rule", json!({ "a": "control", "b": "power", "remove": true })).unwrap();
    assert!(run(&p, &mut prj, "set_collision_rule", json!({ "a": "control", "b": "power", "remove": true })).is_err());
    let id = prj.layout.find_device("-K1").unwrap().1.id;
    let r = run(&p, &mut prj, "set_item_class", json!({ "enclosure": "+CAB1", "id": id, "class": "" })).unwrap();
    assert_eq!((r["class"].as_str(), r["automatic"].as_bool()), (Some("power"), Some(true)));
    let r = run(&p, &mut prj, "set_item_class", json!({ "enclosure": "+CAB1", "id": id, "class": "Noisy" })).unwrap();
    assert_eq!(r["class"], "noisy");
    // Wall mounted at 1.9 m: devices reach above 2 m.
    let r = run(&p, &mut prj, "set_enclosure_elevation", json!({ "enclosure": "+CAB1", "elevation": 1900 })).unwrap();
    assert_eq!(r["floor_height"].as_f64(), Some(1900.0));
    assert!(findings(&p, &mut prj).iter().any(|f| f["check"] == "height"));
}

#[test]
fn enclosures_are_placed_in_the_room() {
    let p = Plugins::new(Vec::<String>::new());
    let mut prj = demo_project();
    let r = run(&p, &mut prj, "set_enclosure_site", json!({ "enclosure": "+CAB1", "x": 2000, "z": 500, "angle": 180 })).unwrap();
    assert_eq!(r["automatic"], false);
    let cab = &r["lineup"][0];
    assert_eq!((cab["enclosure"].as_str(), cab["at"][0].as_f64(), cab["at"][2].as_f64()), (Some("+CAB1"), Some(2000.0), Some(500.0)));
    assert!((cab["angle"].as_f64().unwrap() - 180.0).abs() < 0.01);
    assert!(prj.layout.enclosures[0].site.is_some());
    let r = run(&p, &mut prj, "set_enclosure_site", json!({ "enclosure": "+CAB1" })).unwrap();
    assert_eq!(r["automatic"], true);
    assert!(prj.layout.enclosures[0].site.is_none());
    assert!(run(&p, &mut prj, "set_enclosure_site", json!({ "enclosure": "+NONE" })).is_err());
}

#[test]
fn thermal_simulation() {
    let p = Plugins::new(Vec::<String>::new());
    let mut prj = demo_project();
    let r = run(&p, &mut prj, "panel_thermal", json!({ "enclosure": "+CAB1" })).unwrap();
    assert!(r["max_c"].as_f64().unwrap() >= r["min_c"].as_f64().unwrap());
    assert!(r["top_c"].as_f64().unwrap() > r["bottom_c"].as_f64().unwrap());
    let devices = r["devices"].as_array().unwrap();
    assert!(devices.iter().any(|d| d["device"] == "-K1"), "{r:#}");
    assert!(devices.iter().all(|d| d["too_warm"] == false), "{r:#}");
    assert_eq!(r["model"], "enclosure");
    // An open assembly: room air at the plate, no stratification.
    let e = prj.layout.enclosures.iter_mut().find(|e| e.name == "+CAB1").unwrap();
    e.kind = openecraft_core::panel::EnclosureKind::Assembly;
    let ambient = f64::from(e.climate.ambient_max);
    let r = run(&p, &mut prj, "panel_thermal", json!({ "enclosure": "+CAB1" })).unwrap();
    assert_eq!(r["model"], "open_assembly");
    assert_eq!((r["mean_c"].as_f64(), r["top_c"].as_f64(), r["bottom_c"].as_f64()), (Some(ambient), Some(ambient), Some(ambient)));
}
