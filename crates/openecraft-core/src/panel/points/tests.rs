use super::*;
use crate::panel::Face;
use crate::parts::ConnectionPoint;
use crate::sample::demo_project;

fn item(prj: &Project, tag: &str) -> PanelItem {
    prj.layout.find_device(tag).unwrap().1.clone()
}

fn pin<'a>(points: &'a [PlacedPoint], name: &str) -> &'a PlacedPoint {
    points.iter().find(|p| p.pin == name && p.terminal.is_empty()).unwrap_or_else(|| panic!("no point {name}"))
}

fn inside(p: &PlacedPoint, it: &PanelItem) -> bool {
    let tol = 0.01;
    p.pos[0] >= it.x - tol && p.pos[0] <= it.x + it.w + tol && p.pos[1] >= it.y - tol && p.pos[1] <= it.y + it.h + tol
        && p.pos[2] >= it.z - tol && p.pos[2] <= it.z + it.d + tol
}

#[test]
fn contactor_with_front_aux_block() {
    let prj = demo_project();
    let it = item(&prj, "-K1");
    let points = device_points(&prj, &it);
    // Contactor A1 A2, 1-6, 13 14 plus the block 53 54 61 62 71 72 83 84.
    assert_eq!(points.len(), 18);
    assert!(points.iter().all(|p| inside(p, &it) && !p.exact));
    let a1 = pin(&points, "A1");
    assert_eq!((a1.pos[1], a1.exit), (it.y, Exit::Up));
    assert_eq!((pin(&points, "A2").pos[1], pin(&points, "A2").exit), (it.y + it.h, Exit::Down));
    // The block sits centred in front of the contactor (74 mm high, block 40 mm).
    let b = pin(&points, "53");
    assert_eq!(b.pos[1], it.y + 17.0);
    assert!(b.pos[2] > a1.pos[2], "in front of the contactor");
}

#[test]
fn rotated_devices_turn_their_points() {
    let mut prj = demo_project();
    let it = item(&prj, "-K1");
    let mut turned = it.clone();
    turned.rotate();
    let (ei, _) = prj.layout.find_device("-K1").unwrap();
    *prj.layout.enclosures[ei].items.iter_mut().find(|i| i.id == it.id).unwrap() = turned.clone();
    let points = device_points(&prj, &turned);
    let a1 = pin(&points, "A1");
    // The top of the contactor now points to the right.
    assert_eq!((a1.pos[0], a1.exit), (turned.x + turned.w, Exit::Right));
    assert_eq!(pin(&points, "A2").exit, Exit::Left);
    assert!(points.iter().all(|p| inside(p, &turned)));
}

#[test]
fn terminal_strips_have_points_per_terminal() {
    let prj = demo_project();
    let it = item(&prj, "-X1");
    let labels: Vec<String> = crate::strips::terminals(&prj, "-X1").into_iter().map(|t| t.label).collect();
    let points = device_points(&prj, &it);
    assert_eq!(points.len(), 2 * labels.len());
    // The terminals sit in their cells of the strip, between brackets and end plates.
    let cells = crate::panel::strip_cells(&prj, &it);
    let layout = crate::strips::strip_layout(&prj, "-X1");
    for label in &labels {
        let i = layout.iter().position(|e| e.kind == crate::strips::ElementKind::Terminal(label.clone())).unwrap();
        let centre = it.x + cells[i].at + cells[i].len / 2.0;
        let top = points.iter().find(|p| &p.terminal == label && p.pin == "1").unwrap();
        let bottom = points.iter().find(|p| &p.terminal == label && p.pin == "2").unwrap();
        assert!((top.pos[0] - centre).abs() < 0.01, "terminal {label} in its cell");
        assert_eq!((top.exit, bottom.exit), (Exit::Up, Exit::Down));
    }
}

#[test]
fn door_devices_are_wired_from_behind() {
    let prj = demo_project();
    let it = item(&prj, "-S3");
    assert_eq!(it.face, Face::Door);
    let points = device_points(&prj, &it);
    assert_eq!(points.len(), 2);
    assert!(points.iter().all(|p| p.exit == Exit::Front && (p.pos[2] - (it.z + it.d)).abs() < 0.01));
}

#[test]
fn own_points_are_placed_exactly() {
    let mut prj = demo_project();
    let part = prj.parts.iter_mut().find(|p| p.number == "GEN.MCB-1P-C6").unwrap();
    part.connection_points = vec![ConnectionPoint { pin: "1".into(), pos: [9.0, 10.0, 50.0], exit: Exit::Left }];
    let it = item(&prj, "-F1");
    let points = device_points(&prj, &it);
    assert_eq!(points.len(), 1);
    assert_eq!(points[0].pos, [it.x + 9.0, it.y + 10.0, it.z + 50.0]);
    assert_eq!((points[0].exit, points[0].exact), (Exit::Left, true));
}

#[test]
fn schematic_connection_points_find_their_place() {
    let prj = demo_project();
    let symbols: Vec<&SymbolInst> = prj.pages.iter().flat_map(|pg| pg.symbols.iter()).collect();

    let coil = symbols.iter().find(|s| s.tag == "-K1" && s.sym == "coil").unwrap();
    let points = device_points(&prj, &item(&prj, "-K1"));
    assert_eq!(point_of(&prj, &points, coil, 0).unwrap().pin, "A1");
    assert_eq!(point_of(&prj, &points, coil, 1).unwrap().pin, "A2");

    let input = symbols.iter().find(|s| s.tag == "-A1" && s.label == "I0.2").unwrap();
    let points = device_points(&prj, &item(&prj, "-A1"));
    let at = point_of(&prj, &points, input, 0).unwrap();
    assert_eq!((at.pin.as_str(), at.exit), ("I0.2", Exit::Down), "PLC points by address");

    let terminal = symbols.iter().find(|s| s.tag == "-X1").unwrap();
    let points = device_points(&prj, &item(&prj, "-X1"));
    let bottom = point_of(&prj, &points, terminal, 1).unwrap();
    assert_eq!((bottom.terminal.as_str(), bottom.pin.as_str()), (terminal.label.as_str(), "2"));
}

#[test]
fn nothing_to_place_without_parts() {
    let prj = demo_project();
    let mut it = item(&prj, "-K1");
    it.tag = "-K99".into();
    assert!(device_points(&prj, &it).is_empty());
    it.kind = ItemKind::Duct;
    assert!(device_points(&prj, &item(&prj, "-K1")).len() > 1);
    assert!(device_points(&prj, &it).is_empty());
}

#[test]
fn terminals_use_the_points_of_their_own_part() {
    let mut prj = demo_project();
    let pe = prj.parts.iter_mut().find(|p| p.number == "GEN.TB-PE-2.5").unwrap();
    pe.connection_points = vec![
        ConnectionPoint { pin: "1".into(), pos: [2.6, 4.0, 30.0], exit: Exit::Up },
        ConnectionPoint { pin: "2".into(), pos: [2.6, 44.0, 30.0], exit: Exit::Down },
    ];
    let it = item(&prj, "-X1");
    let points = device_points(&prj, &it);
    let of = |terminal: &str| points.iter().filter(|p| p.terminal == terminal).collect::<Vec<_>>();
    assert!(of("PE").iter().all(|p| p.exact), "PE terminal: points of the PE part");
    assert!(of("1").iter().all(|p| !p.exact), "grey terminal: generated points");
    let top = of("PE").into_iter().find(|p| p.pin == "1").unwrap();
    assert!((top.pos[1] - (it.y + 4.0)).abs() < 0.01);
    // In the cell of the PE terminal.
    let cells = crate::panel::strip_cells(&prj, &it);
    let layout = crate::strips::strip_layout(&prj, "-X1");
    let i = layout.iter().position(|e| e.kind == crate::strips::ElementKind::Terminal("PE".into())).unwrap();
    let centre = it.x + cells[i].at + cells[i].len / 2.0;
    assert!((top.pos[0] - centre).abs() < 0.01, "{} vs {centre}", top.pos[0]);
}


#[test]
fn the_points_of_repeated_parts_are_told_apart_by_their_number() {
    // A measured feeder strip: a current transformer behind each of its three poles,
    // their connection points numbered in the schematic (1S1 ... 3S2).
    let prj = crate::sample::example("industrial_main_distribution").unwrap();
    let it = item(&prj, "=DIST+HV2-Q21");
    let points = device_points(&prj, &it);
    assert_eq!(points.iter().filter(|p| p.pin == "S1").count(), 3);
    let symbol = |pin: &str| prj.pages.iter().flat_map(|p| &p.symbols).find(|s| s.tag == "=DIST+HV2-Q21" && s.pins.iter().any(|x| x == pin)).unwrap_or_else(|| panic!("{pin}"));
    let place = |pin: &str| {
        let s = symbol(pin);
        point_of(&prj, &points, s, s.pins.iter().position(|x| x == pin).unwrap()).unwrap_or_else(|| panic!("no point for {pin}")).pos
    };
    // Each on its own transformer, along the strip.
    let (first, second, third) = (place("1S1"), place("2S1"), place("3S1"));
    assert!(first[1] < second[1] && second[1] < third[1], "{first:?} {second:?} {third:?}");
    assert_eq!((first[0], first[2]), (second[0], second[2]));
    assert_ne!(place("1S1"), place("1S2"));
    // The connection points of the strip itself keep their own.
    assert_eq!(place("2"), pin(&points, "2").pos);
}
