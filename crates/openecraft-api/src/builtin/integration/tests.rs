use crate::builtin::testing::*;
use serde_json::json;

const NETLIST: &str = r#"(export (version "E")
  (design (source "io-board.kicad_sch") (tool "Eeschema 8.0.6"))
  (components
    (comp (ref "J1") (value "Conn_01x02") (footprint "TerminalBlock:TB_1x02") (libsource (lib "Connector") (part "Conn_01x02") (description "")))
    (comp (ref "R1") (value "10k") (footprint "Resistor_SMD:R_0603") (libsource (lib "Device") (part "R") (description "Resistor"))))
  (nets
    (net (code "1") (name "+24V") (node (ref "J1") (pin "1") (pintype "passive")) (node (ref "R1") (pin "1") (pintype "passive")))
    (net (code "2") (name "GND") (node (ref "J1") (pin "2") (pintype "passive")) (node (ref "R1") (pin "2") (pintype "passive")))))
"#;

const PCB: &str = "(kicad_pcb (version 20240108) (general (thickness 1.6)) (gr_rect (start 10 10) (end 60 40) (layer \"Edge.Cuts\")))";

const SYMBOLS: &str = r#"(kicad_symbol_lib (version 20231120) (generator "kicad_symbol_editor")
  (symbol "R" (in_bom yes) (on_board yes)
    (property "Reference" "R" (at 2.032 0 90) (effects (font (size 1.27 1.27))))
    (property "Value" "R" (at 0 0 90) (effects (font (size 1.27 1.27))))
    (symbol "R_0_1" (rectangle (start -1.016 -2.54) (end 1.016 2.54) (stroke (width 0.254) (type default)) (fill (type none))))
    (symbol "R_1_1"
      (pin passive line (at 0 3.81 270) (length 1.27) (name "~" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))
      (pin passive line (at 0 -3.81 90) (length 1.27) (name "~" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27))))))))
"#;

#[test]
fn a_kicad_board_is_added_as_a_device() {
    let mut s = demo_session();
    let pages = s.project().unwrap().pages.len();
    let first = s.project().unwrap().pages[0].name.clone();
    let r = run(&mut s, "import_kicad_board", json!({ "netlist": NETLIST, "pcb": PCB, "tag": "-A50", "after": first })).unwrap();
    assert_eq!(r["connectors"], json!(["J1"]));
    assert_eq!(r["connection_points"], 2);
    assert!(r["tag"].as_str().unwrap().ends_with("-A50"));
    let prj = s.project().unwrap();
    assert_eq!(prj.pages.len(), pages + 1);
    // The page follows the one named.
    assert_eq!(r["pages"], json!([prj.pages[1].name]));
    let part = prj.parts.iter().find(|p| p.number == r["part"].as_str().unwrap()).unwrap();
    assert_eq!((part.width, part.height), (50.0, 30.0));
    // The components of the board are named instead.
    let r = run(&mut s, "import_kicad_board", json!({ "netlist": NETLIST, "connectors": "R" })).unwrap();
    assert_eq!(r["connectors"], json!(["R1"]));
    assert!(run(&mut s, "import_kicad_board", json!({ "netlist": "x" })).is_err());
    assert!(run(&mut s, "import_kicad_board", json!({ "netlist": NETLIST, "pcb": NETLIST })).is_err());
    assert!(run(&mut s, "import_kicad_board", json!({ "netlist": NETLIST, "after": "no such page" })).is_err());
}

#[test]
fn kicad_symbols_are_added_and_replaced() {
    let mut s = new_session();
    let r = run(&mut s, "import_kicad_symbols", json!({ "symbols": SYMBOLS, "library": "Device" })).unwrap();
    assert_eq!((r["added"].as_u64(), r["replaced"].as_u64()), (Some(1), Some(0)));
    let key = r["keys"][0].as_str().unwrap().to_string();
    let listed = run(&mut s, "list_symbols", json!({ "query": "kicad_device" })).unwrap();
    assert!(listed.to_string().contains(&key), "{listed}");
    let again = run(&mut s, "import_kicad_symbols", json!({ "symbols": SYMBOLS, "library": "Device" })).unwrap();
    assert_eq!((again["added"].as_u64(), again["replaced"].as_u64()), (Some(0), Some(1)));
    // It can be placed like any other symbol.
    let page = s.project().unwrap().pages[0].name.clone();
    run(&mut s, "place_symbol", json!({ "page": page, "symbol": key, "x": 100, "y": 100 })).unwrap();
    assert!(run(&mut s, "import_kicad_symbols", json!({ "symbols": NETLIST })).is_err());
}
