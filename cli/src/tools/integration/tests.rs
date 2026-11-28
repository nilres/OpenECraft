use super::*;
use crate::testing::*;

fn demo() -> Session {
    let mut s = session();
    run(&mut s, "new_project", json!({ "name": "Demo", "demo": true })).unwrap();
    s
}

#[test]
fn connectors_are_listed() {
    let mut s = session();
    let r = run(&mut s, "integration_connectors", json!({})).unwrap();
    let ids: Vec<&str> = r["connectors"].as_array().unwrap().iter().filter_map(|c| c["id"].as_str()).collect();
    for id in ["erp", "pdm", "plm", "ecad", "mcad", "procurement", "wiring_machine", "kicad", "freecad"] {
        assert!(ids.contains(&id), "{id}");
    }
}

#[test]
fn export_preview_apply_and_conflicts() {
    let mut s = demo();
    let csv = run(&mut s, "integration_export", json!({ "connector": "erp", "date": "2026-10-24" })).unwrap();
    let csv = csv.as_str().unwrap().to_string();
    let part = csv.lines().nth(1).unwrap().split(';').next().unwrap().to_string();
    run(&mut s, "set_integration_profile", json!({ "name": "SAP", "connector": "erp", "mapping": { "erp_number": "MATNR" } })).unwrap();
    let text = format!("Material;MATNR;Bestand\n{part};100-200;5\n");
    let preview = run(&mut s, "integration_import", json!({ "connector": "erp", "text": text, "profile": "SAP" })).unwrap();
    assert_eq!(preview["changes"].as_array().unwrap().len(), 2);
    assert!(preview["applied"].is_null());
    assert!(s.project().unwrap().integration.items.is_empty());
    let applied = run(&mut s, "integration_import", json!({ "connector": "erp", "text": text, "profile": "SAP", "apply": true })).unwrap();
    assert_eq!(applied["applied"], 2);
    let st = run(&mut s, "integration_status", json!({})).unwrap();
    assert_eq!(st["items"][&part]["erp_number"], "100-200");
    assert_eq!(st["log"].as_array().unwrap().len(), 2);
    let ex = run(&mut s, "integration_export", json!({ "connector": "exchange" })).unwrap();
    assert!(ex.as_str().unwrap().contains("openecraft.exchange/1"));
}

#[test]
fn wrong_input_is_rejected() {
    let mut s = demo();
    assert!(run(&mut s, "integration_export", json!({ "connector": "sap" })).unwrap_err().contains("Unknown connector"));
    assert!(run(&mut s, "integration_import", json!({ "connector": "erp" })).unwrap_err().contains("Missing parameter"));
    assert!(run(&mut s, "integration_import", json!({ "connector": "erp", "text": "a;b\n", "profile": "X" })).unwrap_err().contains("No profile"));
    assert!(run(&mut s, "set_integration_profile", json!({ "name": "X", "connector": "erp", "mapping": { "matnr": "MATNR" } })).unwrap_err().contains("Unknown field"));
    assert_eq!(run(&mut s, "set_integration_profile", json!({ "name": "X", "remove": true })).unwrap()["removed"], 0);
}

#[test]
fn files_of_kicad_and_freecad_are_read_by_their_connectors() {
    let mut s = demo();
    let dir = temp_dir("kicad-freecad");
    // The netlist written for KiCad comes back with another part number for a device.
    let net = dir.join("demo.net");
    run(&mut s, "integration_export", json!({ "connector": "kicad", "path": net.to_str().unwrap() })).unwrap();
    let text = std::fs::read_to_string(&net).unwrap();
    assert!(text.starts_with("(export (version \"E\")"));
    let (tag, number) = {
        let prj = s.project().unwrap();
        let (tag, d) = prj.devices.iter().find(|(tag, d)| !d.parts.is_empty() && text.contains(&format!("(comp (ref \"{tag}\")"))).unwrap();
        (tag.clone(), d.parts[0].number.clone())
    };
    let field = format!("(field (name \"Part number\") \"{number}\")");
    assert!(text.contains(&field));
    // Only this device gets the other number.
    let at = text.find(&format!("(comp (ref \"{tag}\")")).unwrap();
    let changed = format!("{}{}", &text[..at], text[at..].replacen(&field, "(field (name \"Part number\") \"OTHER.123\")", 1).replacen(&format!("(property (name \"Part number\") (value \"{number}\"))"), "", 1));
    std::fs::write(&net, changed).unwrap();
    let preview = run(&mut s, "integration_import", json!({ "connector": "kicad", "path": net.to_str().unwrap() })).unwrap();
    let changes = preview["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 1, "{preview}");
    assert_eq!((changes[0]["key"].as_str(), changes[0]["field"].as_str(), changes[0]["new"].as_str()), (Some(tag.as_str()), Some("part_number"), Some("OTHER.123")));
    // A bill of materials of KiCad names the device in its column Reference.
    let bom = format!("\"Reference\",\"Value\",\"Part number\"
\"{tag}\",\"x\",\"BOM.7\"
");
    let preview = run(&mut s, "integration_import", json!({ "connector": "kicad", "text": bom })).unwrap();
    assert_eq!(preview["changes"][0]["new"], "BOM.7", "{preview}");
    // The macro for FreeCAD reads back unchanged; a file that is no document is refused.
    let fcmacro = dir.join("demo.FCMacro");
    run(&mut s, "integration_export", json!({ "connector": "freecad", "path": fcmacro.to_str().unwrap() })).unwrap();
    let preview = run(&mut s, "integration_import", json!({ "connector": "freecad", "path": fcmacro.to_str().unwrap() })).unwrap();
    assert!(preview["records"].as_u64().unwrap() > 0 && preview["changes"].as_array().unwrap().is_empty(), "{preview}");
    let broken = dir.join("broken.FCStd");
    std::fs::write(&broken, b"PK Document.xml").unwrap();
    assert!(run(&mut s, "integration_import", json!({ "connector": "freecad", "path": broken.to_str().unwrap() })).is_err());
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn kicad_files_are_imported_by_their_content() {
    let mut s = demo();
    let dir = temp_dir("kicad-import");
    let (net, pcb, lib) = (dir.join("io.net"), dir.join("io.kicad_pcb"), dir.join("Device.kicad_sym"));
    std::fs::write(&net, "(export (version \"E\") (design (source \"io.kicad_sch\")) (components (comp (ref \"J1\") (value \"Conn\"))) (nets (net (code \"1\") (name \"+24V\") (node (ref \"J1\") (pin \"1\"))) (net (code \"2\") (name \"GND\") (node (ref \"J1\") (pin \"2\")))))").unwrap();
    std::fs::write(&pcb, "(kicad_pcb (version 20240108) (gr_rect (start 0 0) (end 80 50) (layer \"Edge.Cuts\")))").unwrap();
    std::fs::write(&lib, "(kicad_symbol_lib (version 20231120) (symbol \"R\" (property \"Reference\" \"R\") (symbol \"R_1_1\" (pin passive line (at 0 3.81 270) (length 1.27) (name \"~\") (number \"1\")) (pin passive line (at 0 -3.81 90) (length 1.27) (name \"~\") (number \"2\")))))").unwrap();
    let board = run(&mut s, "import_kicad", json!({ "path": net.to_str().unwrap(), "board": pcb.to_str().unwrap(), "tag": "-A70" })).unwrap();
    assert_eq!((board["connection_points"].as_u64(), board["connectors"][0].as_str()), (Some(2), Some("J1")));
    let part = board["part"].as_str().unwrap().to_string();
    assert_eq!(s.project().unwrap().parts.iter().find(|p| p.number == part).unwrap().width, 80.0);
    // The library is named after its file.
    let symbols = run(&mut s, "import_kicad", json!({ "path": lib.to_str().unwrap() })).unwrap();
    assert_eq!(symbols["keys"], json!(["user:kicad_device_r"]));
    assert_eq!(s.project().unwrap().symbols.iter().find(|u| u.key == "user:kicad_device_r").unwrap().library, "Device");
    assert!(run(&mut s, "import_kicad", json!({ "path": pcb.to_str().unwrap() })).is_err());
    assert!(run(&mut s, "import_kicad", json!({ "path": dir.join("missing.net").to_str().unwrap() })).is_err());
    std::fs::remove_dir_all(dir).ok();
}
