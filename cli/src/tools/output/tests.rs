use super::*;
use crate::testing::*;
use openecraft_core::model::ReportKind;

#[test]
fn reports_and_exports() {
    let dir = temp_dir("project-export");
    let mut s = session();
    run(&mut s, "new_project", json!({ "demo": true })).unwrap();
    let bom = run(&mut s, "report", json!({ "kind": "bill_of_materials" })).unwrap();
    assert!(!bom["rows"].as_array().unwrap().is_empty());
    assert!(run(&mut s, "report", json!({ "kind": "device_list", "format": "csv" })).unwrap().as_str().unwrap().contains(';'));
    assert!(run(&mut s, "report", json!({ "kind": "parts" })).is_err());
    let xlsx = dir.join("devices.xlsx");
    assert!(run(&mut s, "report", json!({ "kind": "device_list", "format": "xlsx" })).is_err(), "xlsx needs a path");
    run(&mut s, "report", json!({ "kind": "device_list", "format": "xlsx", "path": xlsx.to_str().unwrap() })).unwrap();
    let sheets = openecraft_core::xlsx::read(&std::fs::read(&xlsx).unwrap()).unwrap();
    assert_eq!(sheets[0].1[0][0], "Device tag");
    assert!(sheets[0].1.len() > 3);

    let svg = run(&mut s, "export", json!({ "format": "svg", "page": "1" })).unwrap();
    assert!(svg.as_str().unwrap().contains("<svg"));
    assert!(run(&mut s, "export", json!({ "format": "svg" })).is_err(), "svg needs a page");
    let file = dir.join("all.dxf");
    let out = run(&mut s, "export", json!({ "format": "dxf", "path": file.to_str().unwrap() })).unwrap();
    assert!(out["bytes"].as_u64().unwrap() > 0);
    assert!(std::fs::read_to_string(&file).unwrap().contains("SECTION"));
    let file = dir.join("page.dwg");
    run(&mut s, "export", json!({ "format": "dwg", "page": "2", "path": file.to_str().unwrap() })).unwrap();
    assert!(std::fs::read(&file).unwrap().starts_with(b"AC1018"));
    assert!(run(&mut s, "export", json!({ "format": "dwg" })).unwrap_err().contains("output file"));
    for format in ["html", "pdf", "wire_list", "plc_table", "kbl"] {
        assert!(run(&mut s, "export", json!({ "format": format })).is_ok(), "{format}");
    }
}

#[test]
fn panel_layout_in_3d() {
    let mut s = session();
    run(&mut s, "new_project", json!({ "demo": true })).unwrap();
    let gltf = run(&mut s, "export", json!({ "format": "gltf" })).unwrap();
    let v: Value = serde_json::from_str(gltf.as_str().unwrap()).unwrap();
    assert_eq!(v["asset"]["version"], "2.0");
    let file = temp_dir("export-3d").join("panel.stp");
    let r = run(&mut s, "export", json!({ "format": "step", "path": file.to_str().unwrap() })).unwrap();
    assert!(r["bytes"].as_u64().unwrap() > 1000);
    let step = std::fs::read_to_string(&file).unwrap();
    assert!(step.starts_with("ISO-10303-21;") && step.contains("FILE_NAME('panel.stp'"));
    let iges = run(&mut s, "export", json!({ "format": "iges" })).unwrap();
    assert!(iges.as_str().unwrap().contains(",9Hpanel.igs,"));
    let ifc = run(&mut s, "export", json!({ "format": "ifc" })).unwrap();
    assert!(ifc.as_str().unwrap().contains("FILE_SCHEMA(('IFC4'))"));
    // JT is binary: only to a file.
    assert!(run(&mut s, "export", json!({ "format": "jt" })).unwrap_err().contains("path"));
    let file = temp_dir("export-3d").join("panel.jt");
    run(&mut s, "export", json!({ "format": "jt", "path": file.to_str().unwrap() })).unwrap();
    assert!(std::fs::read(&file).unwrap().starts_with(b"Version 8.1 JT"));
}

#[test]
fn every_built_in_report_is_listed_and_accepted() {
    let mut s = session();
    run(&mut s, "new_project", json!({ "demo": true })).unwrap();
    let listed = run(&mut s, "list_reports", json!({})).unwrap();
    let names: Vec<String> = listed.as_array().unwrap().iter().map(|r| r["name"].as_str().unwrap().to_string()).collect();
    for k in ReportKind::ALL {
        let name = snake(k);
        assert!(names.contains(&name), "{name} is not listed");
        run(&mut s, "report", json!({ "kind": name })).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    assert!(names.contains(&"project_statistics".to_string()), "plugin reports are listed");
    // A search finds a report by the standard it follows.
    let found = run(&mut s, "list_reports", json!({ "search": "VDE 0100 600" })).unwrap();
    let found: Vec<&str> = found.as_array().unwrap().iter().map(|r| r["name"].as_str().unwrap()).collect();
    assert_eq!(found, ["inspection", "inspection_summary", "inspection_sources"]);
    assert!(listed.as_array().unwrap().iter().any(|r| r["keywords"].as_array().is_some_and(|k| k.iter().any(|w| w == "IEC 60364-6"))));
}

#[test]
fn check_reports_the_template_and_arguments_of_an_issue() {
    let mut s = new_session();
    run(&mut s, "place_symbol", json!({ "page": "1", "symbol": "coil", "x": 100, "y": 100, "tag": "-K1" })).unwrap();
    let c = run(&mut s, "check", json!({})).unwrap();
    let issue = c["issues"].as_array().unwrap().iter().find(|i| i["args"][0] == "-K1").unwrap_or_else(|| panic!("{c}"));
    assert!(issue["key"].as_str().unwrap().contains("{}"), "{issue}");
    assert!(issue["message"].as_str().unwrap().starts_with("-K1"), "{issue}");
}

#[test]
fn exporter_parameters_are_passed_as_options() {
    let mut s = session();
    run(&mut s, "new_project", json!({ "demo": true })).unwrap();
    let svg = run(&mut s, "export", json!({ "format": "nailboard", "options": "harness=-WH1" })).unwrap();
    assert!(svg.as_str().unwrap().starts_with("<svg"));
    assert!(run(&mut s, "export", json!({ "format": "nailboard" })).unwrap_err().contains("harness"));
    let csv = run(&mut s, "export", json!({ "format": "marking", "options": "scheme=Partex" })).unwrap();
    assert!(!csv.as_str().unwrap().starts_with('\u{feff}'), "Partex has no byte order mark");
    assert!(run(&mut s, "export", json!({ "format": "marking", "options": "colour=red" })).unwrap_err().contains("colour"));
    assert!(run(&mut s, "export", json!({ "format": "marking", "options": "scheme" })).unwrap_err().contains("name=value"));
}

#[test]
fn options_keep_the_commas_of_page_lists() {
    let opts = split_options("pages=1-3, 5,harness=-WH1").unwrap();
    assert_eq!(opts, [("pages".to_string(), "1-3, 5".to_string()), ("harness".to_string(), "-WH1".to_string())]);
    assert!(split_options("5,pages=1").is_err());
    assert!(split_options("").unwrap().is_empty());
    let mut s = session();
    run(&mut s, "new_project", json!({ "demo": true })).unwrap();
    let pdf = run(&mut s, "export", json!({ "format": "pdf", "options": "pages=1-3, 5" })).unwrap();
    let pdf = pdf.as_str().unwrap();
    assert_eq!(pdf.matches("/Type /Page").count() - pdf.matches("/Type /Pages").count(), 4, "four pages");
}
