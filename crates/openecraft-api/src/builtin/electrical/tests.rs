use super::*;
use crate::builtin::testing::*;

#[test]
fn settings_and_calculations_of_the_demo() {
    let mut s = demo_session();
    let e = run(&mut s, "electrical_settings", json!({})).unwrap();
    assert_eq!((e["voltage"].as_f64(), e["earthing"].as_str(), e["capacity_factor"].as_f64()), (Some(400.0), Some("tn_s"), Some(1.0)));
    let e = run(&mut s, "electrical_settings", json!({ "earthing": "tt", "ik_max": 6.0, "ambient": 50.0, "insulation": "xlpe" })).unwrap();
    assert_eq!((e["earthing"].as_str(), e["ik_max_ka"].as_f64(), e["insulation"].as_str()), (Some("tt"), Some(6.0), Some("xlpe")));
    assert!(run(&mut s, "electrical_settings", json!({ "earthing": "tx" })).unwrap_err().contains("Unknown earthing system"));

    let sc = run(&mut s, "short_circuit", json!({})).unwrap();
    let devices = sc["devices"].as_array().unwrap();
    assert!(!devices.is_empty());
    assert!(devices.iter().all(|d| d["breaks"] == true && d["ik_max_a"].as_f64() == Some(6000.0)));
    // TT: final circuits within 0.2 s, by a residual current device.
    assert!(devices.iter().all(|d| (d["permitted_s"].as_f64().unwrap() - 0.2).abs() < 1e-6 && d["disconnects"] == false));

    let l = run(&mut s, "load_calculation", json!({})).unwrap();
    assert!(l["installed_kw"].as_f64().unwrap() > 0.5);
    assert_eq!(l["power_supplies"][0]["without_data"], json!([]));
    let emc = run(&mut s, "emc_check", json!({})).unwrap();
    assert_eq!(emc["problems"], json!([]));
}

#[test]
fn transformer_source_and_coordination_tables() {
    let mut s = demo_session();
    assert!(run(&mut s, "electrical_settings", json!({ "transformer_uk": 4.0 })).unwrap_err().contains("No transformer"));
    let e = run(&mut s, "electrical_settings", json!({ "transformer_kva": 400.0, "transformer_uk": 4.0, "network_mva": 0.0, "motors": false })).unwrap();
    assert_eq!((e["transformer"]["kva"].as_f64(), e["transformer"]["uk_percent"].as_f64(), e["motors"].as_bool()), (Some(400.0), Some(4.0), Some(false)));
    // 577 A / 4 % with c max and the correction of the transformer.
    let ik = e["supply_ik3_ka"].as_f64().unwrap();
    assert!(ik > 14.0 && ik < 17.0, "{ik}");
    let sc = run(&mut s, "short_circuit", json!({})).unwrap();
    assert!((sc["supply_ik3_a"].as_f64().unwrap() / 1000.0 - ik).abs() < 1e-3);
    let devices = sc["devices"].as_array().unwrap();
    assert!(devices.iter().all(|d| d["ip_a"].as_f64().unwrap() > d["ik_max_a"].as_f64().unwrap()));
    let e = run(&mut s, "electrical_settings", json!({ "transformer_kva": 0.0 })).unwrap();
    assert!(e["transformer"].is_null());

    let t = run(&mut s, "coordination_table", json!({ "upstream": "A.Q1", "downstream": "B.F1", "selectivity_ka": 4.5 })).unwrap();
    assert_eq!(t, json!([{ "upstream": "A.Q1", "downstream": "B.F1", "selectivity_ka": 4.5, "backup_ka": 0.0 }]));
    let t = run(&mut s, "coordination_table", json!({ "upstream": "A.Q1", "downstream": "B.F1", "backup_ka": 25.0 })).unwrap();
    assert_eq!((t[0]["selectivity_ka"].as_f64(), t[0]["backup_ka"].as_f64()), (Some(4.5), Some(25.0)));
    assert!(run(&mut s, "coordination_table", json!({ "upstream": "A.Q1" })).is_err());
    assert_eq!(run(&mut s, "coordination_table", json!({ "upstream": "A.Q1", "downstream": "B.F1", "remove": true })).unwrap(), json!([]));
    assert!(run(&mut s, "coordination_table", json!({ "upstream": "A.Q1", "downstream": "B.F1", "remove": true })).is_err());
}

#[test]
fn safety_functions_and_rule_sets() {
    let mut s = demo_session();
    let f = json!({
        "name": "SF1",
        "required": "C",
        "subsystems": [{ "name": "Output", "devices": ["-K1"], "category": "One", "operations": 1000.0 }],
    });
    let r = run(&mut s, "safety_functions", json!({ "set": f })).unwrap();
    assert_eq!(r[0]["achieved"], "c");
    assert_eq!((r[0]["sil"].as_u64(), r[0]["met"].as_bool()), (Some(1), Some(true)));
    assert!(run(&mut s, "safety_functions", json!({ "remove": "SF9" })).is_err());
    assert_eq!(run(&mut s, "safety_functions", json!({ "remove": "SF1" })).unwrap(), json!([]));

    let r = run(&mut s, "rule_sets", json!({ "enable": "ul508a" })).unwrap();
    assert_eq!(r["libraries"].as_array().unwrap().iter().filter(|l| l["enabled"] == true).count(), 1);
    assert_eq!(r["limits"]["min_cross_section_power_mm2"].as_f64(), Some(2.08f32 as f64));
    let v = run(&mut s, "verify_project", json!({})).unwrap();
    assert!(v["failed_checks"].as_u64().unwrap() > 0);
    let custom = json!({ "name": "ACME", "active": true, "checks": [{ "scope": "Project", "require": { "field": "company", "op": "Equals", "value": "ACME" }, "level": "Error", "message": "Made by ACME" }] });
    let r = run(&mut s, "rule_sets", json!({ "add": custom, "disable": "ul508a" })).unwrap();
    assert_eq!(r["rule_sets"][0]["active"], true);
    let v = run(&mut s, "verify_project", json!({})).unwrap();
    assert_eq!(v["checks"].as_array().unwrap().len(), 1);
    assert!(v["problems"][0].as_str().unwrap().ends_with("Made by ACME (ACME)"));
    run(&mut s, "rule_sets", json!({ "remove": "ACME" })).unwrap();
    assert!(run(&mut s, "rule_sets", json!({ "enable": "nfpa" })).is_err());
}

#[test]
fn inspection_results_are_set_and_checked() {
    let mut s = demo_session();
    let r = run(&mut s, "inspection_report", json!({})).unwrap();
    let f1 = r["circuits"].as_array().unwrap().iter().find(|c| c["circuit"] == "-F1").unwrap().clone();
    assert_eq!((f1["characteristic"].as_str(), f1["rated"].as_f64(), f1["verdict"].as_str()), (Some("C"), Some(6.0), Some("not_tested")));
    assert!(f1["zs_max"].as_f64().unwrap() > 1.0);
    let results = json!({ "continuity": "0.3", "insulation_voltage": 500, "insulation": 500, "loop_impedance": "0,9", "polarity": "ok", "function": "ok", "tester": "Kim" });
    let r = run(&mut s, "inspection_report", json!({ "circuit": "-F1", "results": results })).unwrap();
    let f1 = r["circuits"].as_array().unwrap().iter().find(|c| c["circuit"] == "-F1").unwrap();
    assert_eq!((f1["result"]["insulation"].as_str(), f1["verdict"].as_str()), (Some("500"), Some("passed")));
    // All or nothing: a bad value leaves the others unset.
    let bad = json!({ "circuit": "-F1", "results": { "remark": "loose", "rcd_time": "fast" } });
    assert!(run(&mut s, "inspection_report", bad).unwrap_err().contains("fast"));
    assert!(s.project().unwrap().inspection.result("-F1").unwrap().remark.is_empty());
    assert!(run(&mut s, "inspection_report", json!({ "circuit": "-K1", "results": { "remark": "x" } })).unwrap_err().contains("-K1"));
    assert!(run(&mut s, "inspection_report", json!({ "circuit": "-F1", "results": { "colour": "red" } })).unwrap_err().contains("colour"));
    // The data of the report and the visual inspection.
    let r = run(&mut s, "inspection_report", json!({ "header": { "inspector": "Kim", "check.fire": "failed", "ze": 0.3 } })).unwrap();
    assert_eq!((r["header"]["inspector"].as_str(), r["header"]["ze"].as_str(), r["verdict"].as_str()), (Some("Kim"), Some("0.3"), Some("failed")));
    assert_eq!(r["failed"], json!(["check.fire"]));
    assert_eq!(r["checks"].as_array().unwrap().len(), inspection::CHECKS.len());
    assert!(run(&mut s, "inspection_report", json!({ "header": { "check.fire": "maybe" } })).is_err());
    // The demo's power supply as the source of PELV circuits.
    let source = r["sources"][0]["device"].as_str().unwrap().to_string();
    let r = run(&mut s, "inspection_report", json!({ "source": source, "source_results": { "measure": "pelv", "insulation_voltage": 250, "insulation": 2 } })).unwrap();
    assert_eq!((r["sources"][0]["result"]["measure"].as_str(), r["sources"][0]["verdict"].as_str()), (Some("pelv"), Some("passed")));
    assert!(run(&mut s, "inspection_report", json!({ "source": "-X9", "source_results": { "measure": "pelv" } })).unwrap_err().contains("-X9"));
}
