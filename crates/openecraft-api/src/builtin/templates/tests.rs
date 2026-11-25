use super::*;
use crate::builtin::testing::{new_session, run, with_project};
use openecraft_core::project_templates::builtin;

#[test]
fn the_open_project_becomes_a_template() {
    let mut s = new_session();
    let t = run(&mut s, "set_template", json!({ "name": "Pump", "description": "One pump" })).unwrap();
    assert_eq!(t["name"], "Pump");
    run(&mut s, "set_template_parameter", json!({ "name": "POWER_KW", "label": "Rated power", "unit": "kW", "default": "2.2", "number": true })).unwrap();
    run(&mut s, "set_template_parameter", json!({ "name": "REPORTS", "default": "yes", "choices": "yes, no" })).unwrap();
    let t = run(&mut s, "set_template_formula", json!({ "name": "CURRENT", "formula": "round(POWER_KW * 2, 1)", "unit": "A" })).unwrap();
    assert_eq!(t["formulas"][0]["value"], "4.4");
    assert_eq!(t["parameters"][1]["choices"], json!(["yes", "no"]));
    let pg = run(&mut s, "set_page", json!({ "page": "1", "condition": "REPORTS=yes" })).unwrap();
    assert_eq!(pg["condition"], "REPORTS=yes");
    assert_eq!(run(&mut s, "get_template", json!({})).unwrap()["pages"], json!([{ "page": "1", "condition": "REPORTS=yes" }]));

    // Mistakes are refused and leave the template as it was.
    assert!(run(&mut s, "set_page", json!({ "page": "1", "condition": "SIZE=big" })).unwrap_err().contains("SIZE"));
    assert!(run(&mut s, "set_template_formula", json!({ "name": "LOSS", "formula": "CURRENT *" })).unwrap_err().starts_with("LOSS: "));
    assert!(run(&mut s, "set_template_formula", json!({ "name": "EMPTY" })).unwrap_err().contains("needs its formula"));
    assert!(run(&mut s, "set_template_parameter", json!({ "name": "2X" })).unwrap_err().contains("no valid name"));
    assert!(run(&mut s, "remove_template_parameter", json!({ "name": "POWER_KW" })).unwrap_err().contains("POWER_KW"), "still used by CURRENT");
    run(&mut s, "remove_template_parameter", json!({ "name": "CURRENT" })).unwrap();
    let t = run(&mut s, "remove_template_parameter", json!({ "name": "POWER_KW" })).unwrap();
    assert_eq!(t["parameters"].as_array().unwrap().len(), 1);
    assert!(run(&mut s, "remove_template_parameter", json!({ "name": "POWER_KW" })).unwrap_err().contains("no parameter POWER_KW"));
    assert_eq!(s.project().unwrap().pages[0].condition, "REPORTS=yes");
}

#[test]
fn a_bundled_template_shows_its_sizing() {
    let mut s = with_project(builtin("Motor starter direct on line").unwrap());
    let t = run(&mut s, "get_template", json!({})).unwrap();
    assert_eq!(t["problems"], json!([]));
    let current = t["formulas"].as_array().unwrap().iter().find(|f| f["name"] == "CURRENT").unwrap();
    assert_eq!(current["value"], "14.6");
    assert_eq!(t["pages"].as_array().unwrap().len(), 4, "the report pages");
}
