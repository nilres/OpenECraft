use super::*;
use crate::testing::*;

#[test]
fn projects_start_from_a_parametric_template() {
    let mut s = session();
    let list = run(&mut s, "list_project_templates", json!({})).unwrap();
    let names: Vec<&str> = list["templates"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Motor starter direct on line") && names.contains(&"Machine"), "{names:?}");
    let dol = list["templates"].as_array().unwrap().iter().find(|t| t["name"] == "Motor starter direct on line").unwrap();
    assert_eq!(dol["parameters"][1]["name"], "POWER_KW");
    assert_eq!(dol["optional_pages"].as_array().unwrap().len(), 4);

    let v = run(&mut s, "template_values", json!({ "template": "motor starter direct on line", "values": { "POWER_KW": 7.5 } })).unwrap();
    let value = |n: &str| v["values"].as_array().unwrap().iter().find(|x| x["name"] == n).unwrap()["value"].clone();
    assert_eq!((value("CURRENT"), value("PROTECTION")), (json!("14.6"), json!("10 - 16 A")));
    assert!(s.project().is_err(), "nothing started");

    let info = run(&mut s, "new_project", json!({ "template": "Motor starter direct on line", "values": { "POWER_KW": 7.5, "REPORTS": false }, "name": "Line 3" })).unwrap();
    assert_eq!((info["name"].as_str(), info["pages"].as_u64(), info["errors"].as_u64(), info["warnings"].as_u64()), (Some("Line 3"), Some(3), Some(0), Some(0)));
    assert_eq!(s.project().unwrap().devices["-K1"].parts[0].number, "GEN.CT-18A-24DC");
    let err = run(&mut s, "new_project", json!({ "template": "Motor starter direct on line", "values": { "SPEED": 3 }, "discard": true })).unwrap_err();
    assert!(err.contains("no parameter SPEED"), "{err}");
    assert!(run(&mut s, "new_project", json!({ "template": "Nothing", "discard": true })).unwrap_err().contains("Unknown template Nothing"));
}

#[test]
fn templates_of_the_user_are_saved_in_the_folder() {
    let dir = temp_dir("templates-folder");
    let mut s = session();
    run(&mut s, "new_project", json!({ "template": "Motor starter reversing", "edit": true })).unwrap();
    assert!(s.project().unwrap().pages.iter().any(|p| p.condition == "REPORTS=yes"), "the template itself");
    assert!(run(&mut s, "save_project_template", json!({})).unwrap_err().contains("No templates folder"));
    s.templates = Some(dir.clone());
    run(&mut s, "set_template_parameter", json!({ "name": "MOTOR", "default": "Gate" })).unwrap();
    let saved = run(&mut s, "save_project_template", json!({ "description": "Our gates" })).unwrap();
    assert_eq!(saved["name"], "Motor starter reversing");
    assert!(run(&mut s, "save_project_template", json!({})).unwrap_err().contains("exists already"));
    run(&mut s, "save_project_template", json!({ "name": "Gate drive", "overwrite": true })).unwrap();

    let list = run(&mut s, "list_project_templates", json!({})).unwrap();
    let own: Vec<&Value> = list["templates"].as_array().unwrap().iter().filter(|t| t["own"] == true).collect();
    assert_eq!(own.iter().map(|t| t["name"].as_str().unwrap()).collect::<Vec<_>>(), ["Motor starter reversing", "Gate drive"]);
    assert_eq!(own[0]["description"], "Our gates");
    run(&mut s, "new_project", json!({ "template": "Motor starter reversing", "discard": true })).unwrap();
    assert_eq!(s.project().unwrap().info.name, "Gate", "the changed default of the user's copy");

    run(&mut s, "delete_project_template", json!({ "name": "Motor starter reversing" })).unwrap();
    run(&mut s, "new_project", json!({ "template": "Motor starter reversing", "discard": true })).unwrap();
    assert_eq!(s.project().unwrap().info.name, "Conveyor", "the bundled one is back");
    assert!(run(&mut s, "delete_project_template", json!({ "name": "Motor starter reversing" })).unwrap_err().contains("No template"));
}
