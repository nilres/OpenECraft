use super::*;
use crate::analysis::Analysis;
use crate::device_boxes;
use crate::geom::p;
use crate::model::Project;
use crate::sample::demo_project;
use crate::symbols::{Func, Prim};

/// A netlist as KiCad 8 writes it: a relay board with two terminal blocks.
const BOARD: &str = r##"(export (version "E")
  (design
    (source "/home/anna/boards/relay-io/relay-io.kicad_sch")
    (date "2026-11-20T10:11:12+0100")
    (tool "Eeschema 8.0.6")
    (sheet (number "1") (name "/") (tstamps "/")
      (title_block
        (title "Relay I/O board")
        (company "Example GmbH")
        (rev "B")
        (date "2026-11-20")
        (source "relay-io.kicad_sch")
        (comment (number "1") (value "")))))
  (components
    (comp (ref "J1")
      (value "Screw_Terminal_01x03")
      (footprint "TerminalBlock_Phoenix:TerminalBlock_Phoenix_MKDS-1,5-3_1x03_P5.00mm_Horizontal")
      (datasheet "~")
      (description "Supply and input")
      (fields
        (field (name "MPN") "1715734"))
      (libsource (lib "Connector") (part "Screw_Terminal_01x03") (description "Generic screw terminal, single row, 01x03"))
      (property (name "Sheetname") (value "Root"))
      (sheetpath (names "/") (tstamps "/"))
      (tstamps "5b8e6a0e-1c62-4c57-8d0c-1f4f6c0d7a11"))
    (comp (ref "J2")
      (value "Screw_Terminal_01x03")
      (footprint "TerminalBlock_Phoenix:TerminalBlock_Phoenix_MKDS-1,5-3_1x03_P5.00mm_Horizontal")
      (libsource (lib "Connector") (part "Screw_Terminal_01x03") (description "Generic screw terminal, single row, 01x03"))
      (property (name "Part number") (value "PXC.1715734"))
      (sheetpath (names "/") (tstamps "/"))
      (tstamps "0f7f3f0c-33a5-4c0e-9f38-0a6a3b6f2c55"))
    (comp (ref "K1")
      (value "G5LE-1")
      (footprint "Relay_THT:Relay_SPDT_Omron-G5LE-1")
      (libsource (lib "Relay") (part "G5LE-1") (description "Omron G5LE relay, SPDT"))
      (sheetpath (names "/") (tstamps "/"))
      (tstamps "c2a4b3de-6f7b-45f0-b9a1-4d1c8d2e7f90")))
  (nets
    (net (code "1") (name "+24V")
      (node (ref "J1") (pin "1") (pinfunction "Pin_1") (pintype "passive"))
      (node (ref "K1") (pin "1") (pintype "passive")))
    (net (code "2") (name "/IN1")
      (node (ref "J1") (pin "2") (pinfunction "Pin_2") (pintype "passive"))
      (node (ref "K1") (pin "5") (pintype "passive")))
    (net (code "3") (name "GND")
      (node (ref "J1") (pin "3") (pinfunction "Pin_3") (pintype "passive")))
    (net (code "4") (name "/OUT_NO")
      (node (ref "J2") (pin "1") (pinfunction "Pin_1") (pintype "passive"))
      (node (ref "K1") (pin "3") (pintype "passive")))
    (net (code "5") (name "/OUT_COM")
      (node (ref "J2") (pin "2") (pinfunction "Pin_2") (pintype "passive"))
      (node (ref "K1") (pin "2") (pintype "passive")))
    (net (code "6") (name "unconnected-(J2-Pin_3-Pad3)")
      (node (ref "J2") (pin "3") (pinfunction "Pin_3") (pintype "passive+no_connect")))))
"##;

const PCB: &str = r##"(kicad_pcb (version 20240108) (generator "pcbnew") (generator_version "8.0")
  (general (thickness 1.6) (legacy_teardrops no))
  (layers (0 "F.Cu" signal) (44 "Edge.Cuts" user))
  (gr_line (start 100 80) (end 172 80) (stroke (width 0.1) (type default)) (layer "Edge.Cuts"))
  (gr_line (start 172 80) (end 172 125) (stroke (width 0.1) (type default)) (layer "Edge.Cuts"))
  (gr_arc (start 100 125) (mid 136 126) (end 172 125) (stroke (width 0.1) (type default)) (layer "Edge.Cuts"))
  (gr_line (start 100 125) (end 100 80) (stroke (width 0.1) (type default)) (layer "Edge.Cuts"))
  (gr_line (start 0 0) (end 500 500) (stroke (width 0.1) (type default)) (layer "Dwgs.User")))
"##;

/// A library as KiCad 8 writes it: a resistor, an amplifier of two units, a power
/// symbol and a derived symbol.
const LIBRARY: &str = r##"(kicad_symbol_lib (version 20231120) (generator "kicad_symbol_editor") (generator_version "8.0")
  (symbol "R" (pin_numbers hide) (pin_names (offset 0)) (exclude_from_sim no) (in_bom yes) (on_board yes)
    (property "Reference" "R" (at 2.032 0 90) (effects (font (size 1.27 1.27))))
    (property "Value" "R" (at 0 0 90) (effects (font (size 1.27 1.27))))
    (property "Footprint" "" (at -1.778 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Datasheet" "~" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Description" "Resistor" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))
    (symbol "R_0_1"
      (rectangle (start -1.016 -2.54) (end 1.016 2.54) (stroke (width 0.254) (type default)) (fill (type none))))
    (symbol "R_1_1"
      (pin passive line (at 0 3.81 270) (length 1.27) (name "~" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))
      (pin passive line (at 0 -3.81 90) (length 1.27) (name "~" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27)))))))
  (symbol "R_Small" (extends "R")
    (property "Reference" "R" (at 0.762 0.508 0) (effects (font (size 1.27 1.27)) (justify left)))
    (property "Value" "R_Small" (at 0.762 -1.016 0) (effects (font (size 1.27 1.27)) (justify left)))
    (property "Description" "Resistor, small symbol" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes))))
  (symbol "LM358" (pin_names (offset 0.127)) (in_bom yes) (on_board yes)
    (property "Reference" "U" (at 0 5.08 0) (effects (font (size 1.27 1.27)) (justify left)))
    (property "Value" "LM358" (at 0 -5.08 0) (effects (font (size 1.27 1.27)) (justify left)))
    (symbol "LM358_1_1"
      (polyline (pts (xy -5.08 5.08) (xy 5.08 0) (xy -5.08 -5.08) (xy -5.08 5.08)) (stroke (width 0.254) (type default)) (fill (type background)))
      (pin output line (at 7.62 0 180) (length 2.54) (name "~" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))
      (pin input line (at -7.62 -2.54 0) (length 2.54) (name "-" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27)))))
      (pin input line (at -7.62 2.54 0) (length 2.54) (name "+" (effects (font (size 1.27 1.27)))) (number "3" (effects (font (size 1.27 1.27))))))
    (symbol "LM358_2_1"
      (arc (start -2.54 0) (mid 0 2.54) (end 2.54 0) (stroke (width 0.254) (type default)) (fill (type none)))
      (pin output line (at 7.62 0 180) (length 2.54) (name "~" (effects (font (size 1.27 1.27)))) (number "7" (effects (font (size 1.27 1.27)))))
      (pin power_in line (at 0 7.62 270) (length 2.54) hide (name "V+" (effects (font (size 1.27 1.27)))) (number "8" (effects (font (size 1.27 1.27)))))))
  (symbol "GND" (power) (pin_numbers hide) (pin_names (offset 0) hide) (in_bom yes) (on_board yes)
    (property "Reference" "#PWR" (at 0 -6.35 0) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Value" "GND" (at 0 -3.81 0) (effects (font (size 1.27 1.27))))
    (symbol "GND_0_1"
      (polyline (pts (xy 0 0) (xy 0 -1.27) (xy 1.27 -1.27) (xy 0 -2.54) (xy -1.27 -1.27) (xy 0 -1.27)) (stroke (width 0) (type default)) (fill (type none))))
    (symbol "GND_1_1"
      (pin power_in line (at 0 0 270) (length 0) (name "GND" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27))))))))
"##;

fn setup() -> (Project, Analysis) {
    let prj = demo_project();
    let an = Analysis::checked(&prj);
    (prj, an)
}

#[test]
fn files_are_told_apart_by_their_first_keyword() {
    assert!(is_kind(BOARD, "export"));
    assert!(is_kind("\u{feff}  (kicad_symbol_lib (version 1))", "kicad_symbol_lib"));
    assert!(!is_kind(LIBRARY, "kicad_symbol"));
    assert!(!is_kind("tag;x\n-K1;2\n", "export"));
}

#[test]
fn the_netlist_names_devices_and_potentials() {
    let (prj, an) = setup();
    let text = net::export(&prj, &an);
    let nl = net::read(&text).unwrap();
    assert!(!nl.components.is_empty() && !nl.nets.is_empty());
    // Every pin on a net belongs to a component, every component is a device.
    for node in nl.nets.iter().flat_map(|n| &n.nodes) {
        assert!(nl.components.iter().any(|c| c.reference == node.reference), "{}", node.reference);
        assert!(!node.pin.is_empty());
    }
    for c in &nl.components {
        assert!(prj.pages.iter().flat_map(|pg| &pg.symbols).any(|s| s.tag == c.reference), "{}", c.reference);
        assert!(!c.value.is_empty(), "{}", c.reference);
    }
    // The part number travels in a field of its own.
    let (tag, device) = prj.devices.iter().find(|(tag, d)| !d.parts.is_empty() && nl.components.iter().any(|c| &c.reference == *tag)).expect("a device with a part and connections");
    let c = nl.components.iter().find(|c| &c.reference == tag).unwrap();
    assert_eq!(c.field(net::PART_FIELD), Some(device.parts[0].number.as_str()));
    // Potentials keep their names.
    let named = an.netlist.nets.iter().find(|n| n.named && !n.fluid && !n.pins.is_empty()).expect("a named potential");
    assert!(nl.nets.iter().any(|n| n.name == named.name), "{}", named.name);
    // The same project gives the same file: KiCad recognises its components again.
    assert_eq!(text, net::export(&prj, &an));
}

#[test]
fn a_netlist_of_kicad_is_read_with_fields_and_nets() {
    let nl = net::read(BOARD).unwrap();
    assert_eq!(nl.name(), "Relay I/O board");
    assert_eq!((nl.company.as_str(), nl.revision.as_str()), ("Example GmbH", "B"));
    assert_eq!(nl.components.iter().map(|c| c.reference.as_str()).collect::<Vec<_>>(), ["J1", "J2", "K1"]);
    let j1 = &nl.components[0];
    assert_eq!((j1.library.as_str(), j1.description.as_str()), ("Connector", "Supply and input"));
    assert_eq!(j1.field("mpn"), Some("1715734"));
    assert_eq!(nl.components[1].field("Part number"), Some("PXC.1715734"));
    assert_eq!(nl.nets.len(), 6);
    assert_eq!(nl.nets[0].nodes[0], net::Node { reference: "J1".into(), pin: "1".into(), function: "Pin_1".into() });
    assert!(nl.nets[5].unconnected() && !nl.nets[2].unconnected());
    // Without a title the file names the design.
    let untitled = net::read(&BOARD.replace("(title \"Relay I/O board\")", "")).unwrap();
    assert_eq!(untitled.name(), "relay-io");
    assert!(net::read(LIBRARY).is_err());
    assert!(net::read("tag;x").is_err());
}

#[test]
fn the_outline_of_a_board_gives_its_size() {
    let o = board::outline(PCB).unwrap();
    assert_eq!((o.width, o.thickness), (72.0, 1.6));
    assert!((o.height - 46.0).abs() < 1e-3, "{}", o.height);
    assert!(board::outline(BOARD).is_err());
    assert!(board::outline("(kicad_pcb (version 1))").is_err());
}

#[test]
fn a_board_becomes_a_device_with_a_point_per_connector_pin() {
    let (mut prj, _) = setup();
    let pages = prj.pages.len();
    let nl = net::read(BOARD).unwrap();
    let r = board::import(&mut prj, &nl, &board::Options { after: pages - 1, outline: board::outline(PCB).ok(), ..Default::default() }).unwrap();
    assert_eq!((r.pages, r.page, r.points), (1, pages, 6));
    assert_eq!(r.connectors, ["J1", "J2"]);
    assert_eq!(prj.pages.len(), pages + 1);
    let page = &prj.pages[r.page];
    assert!(page.description.contains("Relay I/O board"));
    let boxes: Vec<_> = page.symbols.iter().filter(|s| device_boxes::is_box(s)).collect();
    assert_eq!(boxes.len(), 1);
    assert_eq!((boxes[0].tag.as_str(), boxes[0].function_text.as_str()), (r.tag.as_str(), "Relay I/O board"));
    assert!(!r.tag.is_empty());
    // The points sit on the box and are the connection points of the device.
    assert_eq!(device_boxes::points_of(&prj, page, boxes[0].id).len(), 6);
    assert_eq!(device_boxes::device_pins(&prj, &r.tag), ["J1.1", "J1.2", "J1.3", "J2.1", "J2.2", "J2.3"]);
    // The nets are written at the pins; a pin that connects nothing shows its name.
    let text_of = |designation: &str| page.symbols.iter().find(|s| s.pins.first().map(String::as_str) == Some(designation)).unwrap().function_text.clone();
    assert_eq!((text_of("J1.1"), text_of("J1.2"), text_of("J2.3")), ("+24V".to_string(), "IN1".to_string(), "Pin_3".to_string()));
    assert!(page.texts.iter().any(|t| t.text == "OUT_NO" && t.angle == 90.0));
    // Its part has the size of the board and lists what is on it.
    let part = prj.parts.iter().find(|x| x.number == r.part).unwrap();
    assert_eq!((part.width, part.manufacturer.as_str(), part.type_number.as_str()), (72.0, "Example GmbH", "Relay I/O board"));
    assert_eq!(part.source_data.as_ref().unwrap()["kicad"]["components"].as_array().unwrap().len(), 3);
    assert_eq!(prj.devices[&r.tag].parts[0].number, r.part);
    // The project still checks without errors, and a second board gets a tag and a
    // part of its own.
    let an = Analysis::checked(&prj);
    assert!(!an.issues.iter().any(|i| i.severity == crate::issue::Severity::Error), "{:?}", an.issues.iter().filter(|i| i.severity == crate::issue::Severity::Error).map(|i| i.message()).collect::<Vec<_>>());
    let second = board::import(&mut prj, &nl, &board::Options { after: r.page, ..Default::default() }).unwrap();
    assert!(second.tag != r.tag && second.part != r.part);
}

#[test]
fn the_connectors_of_a_board_can_be_named() {
    let (mut prj, _) = setup();
    let nl = net::read(BOARD).unwrap();
    let r = board::import(&mut prj, &nl, &board::Options { tag: "-A77".into(), connectors: "K1, J2".into(), ..Default::default() }).unwrap();
    assert_eq!(r.connectors, ["J2", "K1"]);
    assert!(r.tag.ends_with("-A77"), "{}", r.tag);
    assert_eq!(r.points, 3 + 4);
    assert!(board::import(&mut prj, &nl, &board::Options { connectors: "Q".into(), ..Default::default() }).is_err());
    // A board without connectors asks for them.
    let none = net::Netlist { components: nl.components.iter().filter(|c| c.reference == "K1").cloned().collect(), ..nl.clone() };
    assert!(board::import(&mut prj, &none, &Default::default()).is_err());
}

#[test]
fn symbols_named_alike_keep_keys_of_their_own() {
    let one = |name: &str| format!("(symbol \"{name}\" (property \"Reference\" \"D\" (at 0 0 0)) (symbol \"{name}_1_1\" (pin passive line (at 0 2.54 270) (length 2.54) (name \"~\") (number \"1\"))))");
    let lib = format!("(kicad_symbol_lib (version 20231120) (generator \"kicad_symbol_editor\") {} {} {})", one("D_Bridge_+AA-"), one("D_Bridge_-AA+"), one("D_Bridge_+A-A"));
    let symbols = sym::import(&lib, "Device").unwrap();
    let keys: Vec<&str> = symbols.iter().map(|u| u.key.as_str()).collect();
    assert_eq!(keys, ["user:kicad_device_d_bridge_aa", "user:kicad_device_d_bridge_aa_2", "user:kicad_device_d_bridge_a_a"]);
    // The same library gives the same keys again.
    assert_eq!(sym::import(&lib, "Device").unwrap(), symbols);
}

#[test]
fn every_symbol_of_kicad_is_usable_here() {
    // As in KiCad's own libraries: a pin between the grid lines (25 mil), pins stacked
    // on one point, a symbol without pins.
    let lib = r#"(kicad_symbol_lib (version 20231120) (generator "kicad_symbol_editor")
      (symbol "Gate" (property "Reference" "U" (at 0 0 0))
        (symbol "Gate_1_1"
          (pin input line (at -7.62 0.635 0) (length 2.54) (name "~") (number "1"))
          (pin power_in line (at 0 -5.08 90) (length 2.54) (name "GND") (number "2"))
          (pin power_in line (at 0 -5.08 90) (length 2.54) (name "GND") (number "3"))))
      (symbol "MountingHole" (property "Reference" "H" (at 0 0 0))
        (symbol "MountingHole_0_1" (circle (center 0 0) (radius 1.27) (stroke (width 0.254)) (fill (type none))))))"#;
    let symbols = sym::import(lib, "Test").unwrap();
    for u in &symbols {
        assert_eq!(crate::symlib::problems(u), Vec::<String>::new(), "{}", u.name);
    }
    let gate = &symbols[0];
    assert_eq!(gate.pins.iter().map(|x| x.label.as_str()).collect::<Vec<_>>(), ["1", "2"]);
    // The pin off the grid is on its next line, with a lead from where KiCad has it.
    assert!(gate.pins[0].pos.same(p(-24.0, -4.0)), "{:?}", gate.pins[0].pos);
    assert!(gate.prims.iter().any(|x| matches!(x, Prim::Line { a, b, .. } if a.same(p(-24.0, -2.0)) && b.same(p(-24.0, -4.0)))));
    assert_eq!(symbols[1].func, Func::Graphic);
}

#[test]
fn test_points_and_jumpers_are_no_connectors() {
    let mut prj = Project::default();
    let comp = |reference: &str, library: &str, footprint: &str| net::Component { reference: reference.into(), library: library.into(), footprint: footprint.into(), ..Default::default() };
    let nl = net::Netlist {
        components: vec![
            comp("CAN1", "Connector_Generic", ""),
            comp("JP1", "Jumper", "Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical"),
            comp("TP1", "Connector", "TestPoint:TestPoint_Pad_D1.5mm"),
            comp("X1", "", ""),
        ],
        nets: ["CAN1", "JP1", "TP1", "X1"].iter().map(|r| net::Net { name: format!("N{r}"), nodes: vec![net::Node { reference: r.to_string(), pin: "1".into(), function: String::new() }] }).collect(),
        ..Default::default()
    };
    assert_eq!(board::import(&mut prj, &nl, &Default::default()).unwrap().connectors, ["CAN1", "X1"]);
    // They are connections of the board when they are named.
    let named = board::import(&mut prj, &nl, &board::Options { connectors: "TP, JP1".into(), ..Default::default() }).unwrap();
    assert_eq!(named.connectors, ["JP1", "TP1"]);
}

#[test]
fn many_pins_go_on_to_further_boxes_and_pages() {
    let mut prj = Project::default();
    let before = prj.pages.len();
    let pins = 400;
    let nl = net::Netlist {
        title: "Backplane".into(),
        components: vec![net::Component { reference: "J1".into(), ..Default::default() }],
        nets: (1..=pins).map(|i| net::Net { name: format!("D{i}"), nodes: vec![net::Node { reference: "J1".into(), pin: i.to_string(), function: String::new() }] }).collect(),
        ..Default::default()
    };
    let r = board::import(&mut prj, &nl, &Default::default()).unwrap();
    assert_eq!(r.points, pins);
    assert!(r.pages > 1);
    assert_eq!(prj.pages.len(), before + r.pages);
    let designations = device_boxes::device_pins(&prj, &r.tag);
    assert_eq!(designations.len(), pins);
    assert_eq!((designations[0].as_str(), designations[pins - 1].as_str()), ("J1.1", "J1.400"));
    // One box is the device, the others continue it; everything stays on the sheet.
    let roles: Vec<_> = prj.pages.iter().flat_map(|pg| &pg.symbols).filter(|s| device_boxes::is_box(s)).map(|s| s.box_role).collect();
    assert_eq!(roles.iter().filter(|r| r.is_device()).count(), 1);
    assert!(roles.len() > r.pages);
    let area = crate::frame::project_default(&prj).area();
    for s in prj.pages[before..].iter().flat_map(|pg| &pg.symbols) {
        assert!(area.expand(0.01).contains(s.pos), "{:?}", s.pos);
    }
}

#[test]
fn the_symbols_of_a_project_are_a_kicad_library() {
    let (prj, _) = setup();
    let text = sym::export_project(&prj, false);
    let root = crate::sexpr::parse(&text).unwrap();
    assert_eq!(root.name(), "kicad_symbol_lib");
    let symbols: Vec<_> = root.children("symbol").collect();
    let used: std::collections::BTreeSet<&str> = prj.pages.iter().flat_map(|pg| &pg.symbols).map(|s| crate::symlib::scale_of(&s.sym).0).filter(|k| *k != crate::symbols::DEVICE_BOX).collect();
    assert_eq!(symbols.len(), used.len());
    // Names are unique, every symbol has its four properties and its two units.
    let names: std::collections::BTreeSet<_> = symbols.iter().map(|s| s.arg(0).unwrap()).collect();
    assert_eq!(names.len(), symbols.len());
    for s in &symbols {
        assert!(s.children("property").count() >= 4);
        assert_eq!(s.children("symbol").count(), 2);
    }
    // The whole library is larger and still a valid file.
    let all = sym::export_project(&prj, true);
    assert!(crate::sexpr::parse(&all).unwrap().children("symbol").count() >= crate::symbols::library().len() - 1);
}

#[test]
fn exported_symbols_read_back_with_their_pins_in_place() {
    let defs: Vec<&crate::symbols::SymbolDef> = crate::symbols::library().iter().filter(|d| d.key != crate::symbols::DEVICE_BOX).collect();
    let back = sym::import(&sym::export(&defs), "Round trip").unwrap();
    assert_eq!(back.len(), defs.len());
    for (def, u) in defs.iter().zip(&back) {
        let pins: Vec<_> = def.pins.iter().filter(|x| !x.internal).collect();
        assert_eq!(pins.len(), u.pins.len(), "{}", def.key);
        for (a, b) in pins.iter().zip(&u.pins) {
            assert!(a.pos.dist(b.pos) < 0.011, "{}: {:?} {:?}", def.key, a.pos, b.pos);
            if !a.label.trim().is_empty() {
                assert_eq!(a.label.trim(), b.label, "{}", def.key);
            }
        }
        // A potential comes back as one, with its name.
        if def.func == Func::Potential && pins.len() == 1 {
            assert_eq!((u.func, u.default_label.as_str()), (Func::Potential, if def.default_label.is_empty() { def.name } else { def.default_label }), "{}", def.key);
        }
        assert_eq!(u.library, "Round trip");
        assert!(u.key.starts_with("user:kicad_round_trip_"), "{}", u.key);
    }
    // Pins on the 4 mm grid are on KiCad's 100 mil grid; every line of its 50 mil grid
    // is a line of the 4 mm grid when a library of KiCad is read.
    assert!((sym::SCALE * 4.0 - 2.54).abs() < 1e-6 && (sym::KICAD_SCALE * 4.0 - 1.27).abs() < 1e-6);
}

#[test]
fn kicad_symbols_become_user_symbols() {
    let list = sym::import(LIBRARY, "").unwrap();
    assert_eq!(list.iter().map(|u| u.name.as_str()).collect::<Vec<_>>(), ["R", "R_Small", "LM358 A", "LM358 B", "GND"]);
    let keys: std::collections::BTreeSet<_> = list.iter().map(|u| u.key.as_str()).collect();
    assert_eq!(keys.len(), list.len());
    let r = &list[0];
    assert_eq!((r.library.as_str(), r.letter.as_str(), r.show_pin_labels), ("KiCad", "R", false));
    // 3.81 mm (150 mil) in KiCad are three grid steps here; y points down.
    assert_eq!(r.pins.iter().map(|x| (x.pos.x, x.pos.y, x.label.as_str())).collect::<Vec<_>>(), [(0.0, -12.0, "1"), (0.0, 12.0, "2")]);
    assert!(r.prims.iter().any(|x| matches!(x, Prim::Poly { pts, closed: true, .. } if pts.len() == 4)));
    assert_eq!(r.prims.iter().filter(|x| matches!(x, Prim::Line { .. })).count(), 2);
    assert_eq!(r.source_data.as_ref().unwrap()["kicad"]["description"], "Resistor");
    // The derived symbol has the graphics of its parent.
    assert_eq!((list[1].pins.len(), list[1].prims.len()), (r.pins.len(), r.prims.len()));
    assert_eq!(list[1].source_data.as_ref().unwrap()["kicad"]["description"], "Resistor, small symbol");
    // A symbol per unit; pin names are written inside, hidden pins are left out.
    let (a, b) = (&list[2], &list[3]);
    assert_eq!(a.pins.iter().map(|x| x.label.as_str()).collect::<Vec<_>>(), ["1", "2", "3"]);
    assert_eq!(b.pins.iter().map(|x| x.label.as_str()).collect::<Vec<_>>(), ["7"]);
    assert_eq!(a.letter, "U");
    assert!(a.prims.iter().any(|x| matches!(x, Prim::Text { text, size, .. } if text == "+" && *size == 2.5)));
    assert!(b.prims.iter().any(|x| matches!(x, Prim::Poly { pts, closed: false, .. } if pts.len() > 8)));
    // A power symbol names a potential.
    let gnd = &list[4];
    assert_eq!((gnd.func, gnd.default_label.as_str(), gnd.pins.len()), (Func::Potential, "GND", 1));
    assert!(!gnd.prims.iter().any(|x| matches!(x, Prim::Text { .. })));
    // They are symbols of the project once merged; reading the library again replaces them.
    let mut prj = Project::default();
    assert_eq!(crate::symlib::merge(&mut prj.symbols, crate::symlib::SymbolLibrary { symbols: list.clone() }), (5, 0));
    assert_eq!(crate::symlib::merge(&mut prj.symbols, crate::symlib::SymbolLibrary { symbols: sym::import(LIBRARY, "").unwrap() }), (0, 5));
    let def = prj.symbol(&list[2].key).unwrap();
    assert_eq!(def.pins.len(), 3);
    for u in &list {
        assert!(crate::symlib::problems(u).is_empty(), "{}: {:?}", u.name, crate::symlib::problems(u));
    }
    assert!(sym::import(BOARD, "").is_err());
    assert!(sym::import("(kicad_symbol_lib (version 20231120))", "").is_err());
}

#[test]
fn an_arc_passes_its_middle_point() {
    let pts = super::sym::import(
        "(kicad_symbol_lib (symbol \"A\" (symbol \"A_0_1\" (arc (start 2.54 0) (mid 0 2.54) (end -2.54 0) (stroke (width 0)) (fill (type none))))))",
        "",
    )
    .unwrap()[0]
        .prims
        .iter()
        .find_map(|x| match x {
            Prim::Poly { pts, .. } => Some(pts.clone()),
            _ => None,
        })
        .unwrap();
    // A half circle of radius 8 about the origin, over the top (y down: negative).
    assert!(pts.iter().all(|q| (q.len() - 8.0).abs() < 1e-3 && q.y < 1e-3), "{pts:?}");
    assert!(pts.iter().any(|q| q.dist(crate::geom::p(0.0, -8.0)) < 1e-3));
    assert!(pts[0].dist(crate::geom::p(8.0, 0.0)) < 1e-3 && pts[pts.len() - 1].dist(crate::geom::p(-8.0, 0.0)) < 1e-3);
}
