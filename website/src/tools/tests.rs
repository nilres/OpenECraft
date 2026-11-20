use super::*;
use openecraft_core::sample;
use openecraft_i18n::{with_lang, Lang};

fn values(section: &str, phases: &str) -> String {
    json!({ "current": "16", "length": "25", "section": section, "voltage": if phases == "3" { "400" } else { "230" }, "limit": "3", "cos_phi": "1", "phases": phases, "material": "copper" }).to_string()
}

#[test]
fn the_voltage_drop_is_the_one_of_the_installation_circuits() {
    with_lang(Lang::En, || {
        let answer = call("voltage_drop", &values("1.5", "1"), &[]);
        // 2 × 16 A × 25 m × 0.0225 Ω·mm²/m / 1.5 mm² = 12 V of 230 V.
        let (volts, percent) = installation::drop(&Project::default(), 16.0, 25.0, 1.5, 1.0, 230.0, false);
        assert!((volts - 12.0).abs() < 0.01 && (percent - 5.217).abs() < 0.01);
        assert_eq!(answer["ok"], false);
        assert_eq!(answer["facts"][0][1], "12.00 V");
        assert_eq!(answer["facts"][1][1], "5.22 %");
        assert_eq!(answer["facts"][2][1], "4.0 mm²", "the smallest cross-section within 3 %");
        assert_eq!(answer["summary"], "5.22 % is over the limit of 3.0 %.");
        assert_eq!(answer["rows"].as_array().unwrap().len(), SECTIONS.len());
        assert_eq!(answer["rows"][0], json!(["1.5 mm²", "12.00 V", "5.22 %", "over the limit"]));
        assert_eq!(answer["headers"].as_array().unwrap().len(), 4);
        // Three-phase: √3 instead of 2, on the voltage between the lines.
        let three = call("voltage_drop", &values("2.5", "3"), &[]);
        assert_eq!(three["ok"], true);
        assert_eq!(three["facts"][0][1], "6.24 V");
    });
}

#[test]
fn aluminium_drops_more_and_a_comma_is_a_decimal_point() {
    with_lang(Lang::En, || {
        let copper = call("voltage_drop", &values("2,5", "1"), &[]);
        let aluminium = call("voltage_drop", &values("2,5", "1").replace("copper", "aluminium"), &[]);
        assert_eq!(copper["facts"][0][1], "7.20 V");
        assert_eq!(aluminium["facts"][0][1], "11.52 V");
    });
    // The answer is in the language of the page, numbers included.
    let german = with_lang(Lang::De, || call("voltage_drop", &values("1.5", "1"), &[]));
    assert_eq!(german["facts"][0][1], "12,00 V");
    assert_ne!(german["facts"][0][0], "Voltage drop");
}

#[test]
fn a_value_that_is_no_number_is_refused() {
    with_lang(Lang::En, || {
        let answer = call("voltage_drop", &values("0", "1"), &[]);
        assert_eq!(answer["error"], "Cross-section: a number above 0 is needed");
        assert!(call("voltage_drop", "{}", &[])["error"].is_string());
        assert!(call("nothing", "", &[])["error"].is_string());
    });
}

#[test]
fn a_drawing_is_shown_and_converted() {
    with_lang(Lang::En, || {
        let project = sample::demo_project();
        let dxf = openecraft_core::dxf::pages_dxf(&project, &[0]);
        let answer = call("drawing", "", dxf.as_bytes());
        assert!(answer["error"].is_null(), "{answer}");
        assert!(answer["svg"].as_str().unwrap().starts_with("<svg "));
        assert_eq!(answer["facts"][0], json!(["Format", "DXF"]));
        assert!(answer["facts"][1][1].as_u64().unwrap() > 10, "lines of the page");
        assert_eq!(answer["files"], json!(["drawing_svg", "drawing_pdf", "drawing_dwg"]));
        assert!(file("drawing_svg", dxf.as_bytes()).unwrap().starts_with(b"<svg "));
        assert!(file("drawing_pdf", dxf.as_bytes()).unwrap().starts_with(b"%PDF"));
        // DXF to DWG and back.
        let dwg = file("drawing_dwg", dxf.as_bytes()).unwrap();
        assert!(dwg::is_dwg(&dwg));
        let back = call("drawing", "", &dwg);
        assert_eq!(back["facts"][0], json!(["Format", "DWG"]));
        assert_eq!(back["files"], json!(["drawing_svg", "drawing_pdf", "drawing_dxf"]));
        assert!(String::from_utf8(file("drawing_dxf", &dwg).unwrap()).unwrap().contains("ENTITIES"));
        // No file, or one that is no drawing.
        assert_eq!(call("drawing", "", &[])["error"], "Choose a file first");
        assert!(call("drawing", "", b"no drawing")["error"].is_string());
    });
}

#[test]
fn a_parts_file_is_listed_and_converted() {
    with_lang(Lang::En, || {
        let catalog = openecraft_parts::default_catalog();
        let csv = openecraft_parts::to_csv(&catalog[..3]);
        let answer = call("parts", "", csv.as_bytes());
        assert!(answer["error"].is_null(), "{answer}");
        assert_eq!(answer["summary"], "3 parts");
        assert_eq!(answer["rows"].as_array().unwrap().len(), 3);
        assert_eq!(answer["rows"][0][0], catalog[0].number.as_str());
        assert_eq!(answer["headers"].as_array().unwrap().len(), answer["rows"][0].as_array().unwrap().len());
        let json = file("parts_json", csv.as_bytes()).unwrap();
        let read: Vec<openecraft_parts::Part> = serde_json::from_slice(&json).unwrap();
        assert_eq!(read.len(), 3);
        assert_eq!(file("parts_csv", &json).unwrap(), csv.as_bytes(), "JSON back to the same CSV");
        assert_eq!(call("parts", "", &[])["error"], "Choose a file first");
    });
}

#[test]
fn every_tool_and_file_is_known() {
    with_lang(Lang::En, || {
        for tool in TOOLS {
            assert_ne!(call(tool, "{}", &[])["error"], format!("Unknown command: {tool}"), "{tool}");
        }
        for (name, ending) in FILES {
            assert!(name.ends_with(ending), "{name}");
            assert_ne!(file(name, &[]).err(), Some(format!("Unknown export: {name}")), "{name}");
        }
        assert!(file("nothing", &[]).is_err());
    });
}
