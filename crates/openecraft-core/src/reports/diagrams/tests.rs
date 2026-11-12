use super::*;
use crate::geom::p;
use crate::analysis::Analysis;
use crate::sample::demo_project;

fn rows<'a>(t: &'a Table, first: &str) -> Vec<&'a Vec<String>> {
    t.rows.iter().filter(|r| r[0] == first).collect()
}

#[test]
fn terminal_connections_list_both_sides() {
    let prj = demo_project();
    let an = Analysis::checked(&prj);
    let t = &an.report(&prj, ReportKind::TerminalConnections).unwrap();
    let x1 = rows(t, "-X1");
    // Terminal 1: -K1 inside the cabinet, the motor through cable -W1 outside.
    let one: Vec<(&str, &str, &str)> = x1.iter().filter(|r| r[1] == "1").map(|r| (r[2].as_str(), r[3].as_str(), r[5].as_str())).collect();
    assert!(one.contains(&("External", "-M1:U1", "-W1")) || one.contains(&("Internal", "-M1:U1", "-W1")), "{one:?}");
    assert!(one.iter().any(|r| r.1.starts_with("-K1")), "{one:?}");
    assert!(x1.iter().all(|r| r.len() == t.headers.len()));
    assert!(t.targets.iter().all(|t| t.is_some()), "every row jumps to its terminal");
}

#[test]
fn cable_connections_show_every_core() {
    let prj = demo_project();
    let an = Analysis::checked(&prj);
    let t = &an.report(&prj, ReportKind::CableConnections).unwrap();
    let w1 = rows(t, "-W1");
    let cores: Vec<(&str, &str, &str)> = w1.iter().map(|r| (r[3].as_str(), r[4].as_str(), r[5].as_str())).collect();
    assert_eq!(cores, vec![("-X1:1", "BN", "-M1:U1"), ("-X1:2", "BK", "-M1:V1"), ("-X1:3", "GY", "-M1:W1"), ("-X1:PE", "GNYE", "-M1:PE")]);
    assert!(w1.iter().all(|r| r[2] == "15"), "cable length");
}

#[test]
fn device_connections_list_every_connection_point() {
    let prj = demo_project();
    let an = Analysis::checked(&prj);
    let t = &an.report(&prj, ReportKind::DeviceConnections).unwrap();
    let k1 = rows(t, "-K1");
    for point in ["A1", "A2", "13", "14", "53", "54", "83", "84"] {
        assert!(k1.iter().any(|r| r[1] == point), "{point} missing: {k1:?}");
    }
    assert!(!t.rows.iter().any(|r| r[0] == "-X1"), "no terminals");
    // Every connection appears once at each of its device ends.
    let device_ends = an.connections.iter().flat_map(|c| [c.a, c.b]).filter(|p| {
        let s = &prj.pages[p.page].symbols[p.sym];
        !matches!(prj.symbol(&s.sym).map(|d| d.func), Some(Func::Terminal | Func::Plug(_)))
    });
    assert_eq!(t.rows.iter().filter(|r| !r[3].is_empty()).count(), device_ends.count());
    assert!(t.rows.iter().all(|r| r.len() == t.headers.len()));
}

#[test]
fn pin_connections_follow_the_plug_sides() {
    let mut prj = Project::default();
    for (key, x, y, tag, label) in [("pot_24", 100.0, 76.0, "", "+24V"), ("plug_pair", 100.0, 100.0, "-X10", "1"), ("coil", 100.0, 120.0, "-K1", "")] {
        let def = prj.symbol(key).unwrap();
        let id = prj.new_id();
        let pins = def.pins.iter().map(|x| x.label.to_string()).collect();
        prj.pages[0].symbols.push(SymbolInst { id, sym: key.into(), pos: p(x, y), tag: tag.into(), label: label.into(), pins, ..Default::default() });
    }
    for (y1, y2) in [(80.0, 96.0), (104.0, 116.0)] {
        let id = prj.new_id();
        prj.pages[0].wires.push(Wire { id, a: p(100.0, y1), b: p(100.0, y2), ..Default::default() });
    }
    let an = Analysis::checked(&prj);
    let t = &an.report(&prj, ReportKind::PinConnections).unwrap();
    let x10 = rows(t, "-X10");
    assert_eq!(x10.len(), 1, "{x10:?}");
    assert_eq!((x10[0][1].as_str(), x10[0][3].as_str()), ("1", "-K1:A1"));
    assert!(["Socket side", "Pin side"].contains(&x10[0][2].as_str()));
}

#[test]
fn terminal_lineup_and_strip_overview() {
    let prj = demo_project();
    let an = Analysis::checked(&prj);
    let lineup_table = an.report(&prj, ReportKind::TerminalLineup).unwrap();
    let lineup = rows(&lineup_table, "-X1");
    let labels: Vec<&str> = lineup.iter().filter(|r| r[2] == "Terminal").map(|r| r[3].as_str()).collect();
    assert_eq!(labels, vec!["1", "2", "3", "PE"]);
    // Positions follow one another along the rail.
    let width: f32 = lineup.iter().map(|r| r[6].parse::<f32>().unwrap()).sum();
    assert!(lineup.last().unwrap()[7].ends_with(&format!("{width:.1}")), "{lineup:?}");

    let overview_table = an.report(&prj, ReportKind::TerminalStripOverview).unwrap();
    let overview = rows(&overview_table, "-X1");
    assert_eq!(overview.len(), 1);
    assert_eq!((overview[0][3].as_str(), overview[0][5].as_str()), ("4", "4"));
    assert_eq!(overview[0][7], format!("{width:.1}"));
}

#[test]
fn panel_legend_numbers_the_devices() {
    let prj = demo_project();
    let an = Analysis::checked(&prj);
    let t = &an.report(&prj, ReportKind::PanelLegend).unwrap();
    let cab = rows(t, "+CAB1");
    let items = &prj.layout.enclosures[0].items;
    let placed = items.iter().filter(|i| i.kind == ItemKind::Device).count();
    let profiles = items.iter().filter(|i| i.kind.is_profile()).count();
    assert!(placed > 0 && profiles > 0);
    assert_eq!(cab.len(), placed + profiles);
    let numbers: Vec<String> = cab.iter().map(|r| r[1].clone()).collect();
    assert_eq!(numbers, (1..=placed + profiles).map(|n| n.to_string()).collect::<Vec<_>>());
    assert!(cab.iter().any(|r| r[2] == "-K1" && !r[4].is_empty() && r[8].is_empty()));
}

#[test]
fn panel_legend_lists_profiles_with_section_and_cut_length() {
    let prj = demo_project();
    let an = Analysis::checked(&prj);
    let t = &an.report(&prj, ReportKind::PanelLegend).unwrap();
    let cab = rows(t, "+CAB1");
    // After the devices: rails, then ducts, each with its part, cross-section and length.
    let first = cab.iter().position(|r| r[2].is_empty()).expect("profiles");
    assert!(cab[first..].iter().all(|r| r[2].is_empty() && !r[8].is_empty()));
    let duct = prj.layout.enclosures[0].items.iter().find(|i| i.kind == ItemKind::Duct).unwrap();
    let row = cab.iter().find(|r| r[4] == duct.part && r[8] == crate::panel::mm_text(duct.length())).expect("duct row");
    assert_eq!(row[7], format!("{} × {}", crate::panel::mm_text(duct.w.min(duct.h)), crate::panel::mm_text(duct.d)));
    let kinds: Vec<ItemKind> = cab[first..].iter().map(|r| prj.layout.enclosures[0].items.iter().find(|i| i.kind.is_profile() && i.part == r[4]).unwrap().kind).collect();
    assert!(kinds.windows(2).all(|w| w[0].layer() <= w[1].layer()), "{kinds:?}");
}
