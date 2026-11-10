use super::*;
use crate::panel::{DuctRun, EndKind, Enclosure, PanelItem, Route};
use crate::analysis::Analysis;
use crate::issue::Severity;

fn project_with_duct(part: &str) -> Project {
    let mut prj = Project::default();
    let mut e = Enclosure::default();
    e.items.push(PanelItem { id: 7, kind: ItemKind::Duct, part: part.into(), x: 0.0, y: 0.0, w: 400.0, h: 40.0, d: 60.0, ..Default::default() });
    prj.layout.enclosures.push(e);
    prj
}

fn wire(cross_section: &str, runs: &[(f32, f32)]) -> RoutedWire {
    RoutedWire {
        enclosure: 0,
        from: "-K1:13".into(),
        to: "-K2:14".into(),
        net: String::new(),
        color: String::new(),
        cross_section: cross_section.into(),
        number: String::new(),
        cable: String::new(),
        route: Route {
            points: Vec::new(),
            plate_length: 0.0,
            via_ducts: true,
            runs: runs.iter().map(|&(from, to)| DuctRun { duct: 7, from, to }).collect(),
        },
        length: 0.0,
        items: (1, 2),
        ends: (EndKind::Generated, EndKind::Generated),
        door: (None, None),
        door_routes: (None, None),
        terminals: (None, None),
        axes: (None, None),
        legs: Vec::new(),
    }
}

#[test]
fn fullest_stretch_of_overlapping_wires() {
    let prj = project_with_duct("NONE");
    let fill = duct_fill(&prj, &[wire("1.5 mm²", &[(0.0, 100.0)]), wire("1.5 mm²", &[(50.0, 150.0)])], &[]);
    assert_eq!(fill.len(), 1);
    let f = &fill[0];
    // Default duct profile 40 x 60 mm: inside 36 x 55.5 mm.
    assert_eq!(f.capacity, 1998.0);
    assert_eq!((f.from, f.to, f.wires, f.total_wires), (50.0, 100.0, 2, 2));
    let expected = 2.0 * wire_area("1.5 mm²") / 1998.0 * 100.0;
    assert!((f.percent - expected).abs() < 1e-3, "{} vs {expected}", f.percent);
}

#[test]
fn wires_that_only_touch_are_not_side_by_side() {
    let prj = project_with_duct("NONE");
    let fill = duct_fill(&prj, &[wire("1.5", &[(0.0, 50.0)]), wire("1.5", &[(50.0, 100.0)])], &[]);
    assert_eq!(fill[0].wires, 1);
    // The fullest stretch reaches over both, they are equally full.
    assert_eq!((fill[0].from, fill[0].to), (0.0, 100.0));
}

#[test]
fn capacity_inside_the_duct_part_and_wire_sizes() {
    let mut prj = project_with_duct("NONE");
    prj.parts.push(crate::parts::Part { number: "OWN.DUCT".into(), width: 1000.0, height: 60.0, depth: 80.0, ..Default::default() });
    assert!(crate::panel::change_profile(&mut prj, 0, 7, "OWN.DUCT"));
    let fill = duct_fill(&prj, &[wire("2.5", &[(0.0, 10.0)])], &[]);
    // 60 x 80 mm less the walls (2 mm), the base and the cover: 56 x 75.5 mm.
    assert_eq!(fill[0].capacity, 4228.0);
    assert!(wire_area("2.5") > wire_area("1.5"));
    assert!(wire_area("0.75") < wire_area("1.5"));
    // Unused ducts and zero length stretches are not listed.
    assert!(duct_fill(&prj, &[wire("1.5", &[(10.0, 10.0)])], &[]).is_empty());
    assert!(duct_fill(&prj, &[], &[]).is_empty());
}

#[test]
fn demo_ducts_in_check_and_report() {
    let mut prj = crate::sample::demo_project();
    let an = Analysis::checked(&prj);
    assert!(!an.duct_fill.is_empty());
    assert!(an.duct_fill.iter().all(|f| f.percent > 0.0 && f.percent < prj.layout.duct_fill_limit));
    let report = &an.report(&prj, crate::model::ReportKind::DuctFill).unwrap();
    assert_eq!(report.rows.len(), an.duct_fill.len());
    assert!(!an.issues.iter().any(|i| i.message().contains("cable duct at")), "demo stays below the limit");

    let fullest = an.duct_fill.iter().map(|f| f.percent).fold(0.0, f32::max);
    prj.layout.duct_fill_limit = fullest / 2.0;
    let an = Analysis::checked(&prj);
    let warnings: Vec<String> = an.issues.iter().filter(|i| i.severity == Severity::Warning).map(|i| i.message()).filter(|m| m.contains("cable duct at")).collect();
    assert!(!warnings.is_empty());
    assert!(warnings[0].starts_with("+CAB1: cable duct at "), "{}", warnings[0]);
    assert!(warnings[0].ends_with(&format!("(limit {:.0} %)", fullest / 2.0)));
}

#[test]
fn loads_make_full_stretches_expensive() {
    let mut prj = project_with_duct("NONE");
    prj.layout.duct_fill_limit = 1.0; // 20 mm² of the 1998 mm² inside the duct
    let mut loads = DuctLoads::new(&prj);
    let area = wire_area("1.5");
    assert_eq!(loads.cost(7, 0.0, 100.0, area), 1.0);
    for _ in 0..3 {
        loads.add(&wire("1.5", &[(0.0, 50.0)]).route, area);
    }
    assert!((loads.load(7, 0.0, 100.0) - 3.0 * area).abs() < 1e-3);
    assert_eq!(loads.cost(7, 0.0, 100.0, area), OVERFILL_COST, "another wire would exceed 1 %");
    // Beyond the loaded stretch the duct is still empty.
    assert_eq!(loads.load(7, 50.0, 100.0), 0.0);
    assert_eq!(loads.cost(7, 60.0, 100.0, area), 1.0);
    // Unknown ducts have no capacity to protect.
    assert_eq!(loads.cost(99, 0.0, 10.0, area), 1.0);
}

#[test]
fn wires_avoid_overfull_ducts() {
    let mut prj = crate::sample::demo_project();
    let before = Analysis::checked(&prj);
    // Only wires on the mounting plate are routed again (door routes stay the shortest).
    let on_plate = |prj: &Project, f: &&DuctFill| prj.layout.enclosures[f.enclosure].item(f.duct).is_some_and(|d| d.face == crate::panel::Face::Plate);
    let fullest = before.duct_fill.iter().filter(|f| on_plate(&prj, f)).map(|f| f.percent).fold(0.0, f32::max);
    prj.layout.duct_fill_limit = fullest * 0.6;
    let after = Analysis::checked(&prj);
    assert_eq!(after.wiring.len(), before.wiring.len());
    let moved: Vec<&RoutedWire> = after
        .wiring
        .iter()
        .filter(|w| before.wiring.iter().find(|o| o.key() == w.key()).is_some_and(|o| o.route.runs != w.route.runs))
        .collect();
    assert!(!moved.is_empty(), "some wires take another way");
    let new_fullest = after.duct_fill.iter().filter(|f| on_plate(&prj, f)).map(|f| f.percent).fold(0.0, f32::max);
    assert!(new_fullest < fullest, "{new_fullest} < {fullest}");
    // Lengths follow the new routes.
    for w in &moved {
        let old = before.wiring.iter().find(|o| o.key() == w.key()).unwrap();
        assert!((w.length - old.length - (w.route.plate_length - old.route.plate_length)).abs() < 0.01);
    }
}

#[test]
fn wires_lie_in_rows_from_the_base_of_the_duct() {
    // Inside of a duct 40 wide and 60 deep: walls of 2 mm, the cover 2.5 mm.
    let it = PanelItem { kind: ItemKind::Duct, w: 400.0, h: 40.0, d: 60.0, ..Default::default() };
    assert_eq!(duct_inside(&it), (36.0, 55.5));
    // Three 11.8 mm wires a row, centred across the duct, the next row on top.
    let (lanes, fits) = duct_lanes(36.0, 55.5, &[11.8; 5]);
    assert!(fits);
    for (i, a) in lanes.iter().enumerate() {
        assert!(a[0].abs() + 5.9 <= 18.0 + 1e-3 && a[1] >= 5.9, "{lanes:?}");
        assert!(lanes[i + 1..].iter().all(|b| (a[0] - b[0]).hypot(a[1] - b[1]) >= 11.8), "{lanes:?}");
    }
    assert_eq!(lanes.iter().filter(|l| l[1] < 11.8).count(), 3);
    // The thickest at the base.
    let (lanes, _) = duct_lanes(36.0, 55.5, &[5.0, 30.0, 5.0]);
    assert!((lanes[1][1] - 15.05).abs() < 1e-3 && lanes[0][1] < lanes[1][1] && lanes[2][1] > 30.1, "{lanes:?}");
    // Five rows of them do not fit 55.5 mm.
    assert!(!duct_lanes(36.0, 55.5, &[11.8; 13]).1);
    // Wider than the duct: never.
    assert!(!duct_lanes(36.0, 55.5, &[40.0]).1);
}

#[test]
fn the_fill_knows_the_thickness_of_every_wire() {
    let prj = project_with_duct("NONE");
    let fill = duct_fill(&prj, &[wire("300 mm²", &[(0.0, 100.0)]), wire("1.5 mm²", &[(200.0, 300.0)])], &[]);
    let mut d = fill[0].diameters.clone();
    d.sort_by(f32::total_cmp);
    assert_eq!(d, vec![crate::harness::outer_diameter("1.5 mm²"), crate::harness::outer_diameter("300 mm²")]);
}

#[test]
fn cables_and_tubes_fill_the_ducts_like_wires() {
    let prj = project_with_duct("NONE");
    let cable = DuctContent { enclosure: 0, runs: vec![DuctRun { duct: 7, from: 40.0, to: 300.0 }], diameter: 12.0 };
    let fill = duct_fill(&prj, &[wire("1.5", &[(0.0, 100.0)])], std::slice::from_ref(&cable));
    let f = &fill[0];
    assert_eq!((f.from, f.to, f.wires, f.total_wires), (40.0, 100.0, 2, 2));
    let expected = (wire_area("1.5") + std::f32::consts::PI / 4.0 * 144.0) / 1998.0 * 100.0;
    assert!((f.percent - expected).abs() < 1e-3, "{} vs {expected}", f.percent);
    assert!(f.diameters.contains(&12.0));
    // A duct with only cables in it has a fill level too.
    assert_eq!(duct_fill(&prj, &[], &[cable])[0].total_wires, 1);
}

#[test]
fn a_wire_counts_once_however_often_it_enters_a_duct() {
    let prj = project_with_duct("NONE");
    let fill = duct_fill(&prj, &[wire("1.5", &[(0.0, 50.0), (200.0, 250.0)]), wire("1.5", &[(0.0, 100.0)])], &[]);
    assert_eq!((fill[0].wires, fill[0].total_wires, fill[0].diameters.len()), (2, 2, 2));
}
