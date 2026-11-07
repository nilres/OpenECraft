use super::*;

fn sym(prj: &mut Project, key: &str, pos: P) -> u64 {
    let id = prj.new_id();
    prj.pages[0].symbols.push(SymbolInst { id, sym: key.into(), pos, ..Default::default() });
    id
}

fn area() -> R {
    R::from_points(p(0.0, 0.0), p(400.0, 280.0))
}

#[test]
fn autoconnect_joins_facing_pins() {
    let mut prj = Project::default();
    let top = sym(&mut prj, "no", p(40.0, 40.0));
    sym(&mut prj, "coil", p(40.0, 80.0));
    // Not aligned: no connection.
    sym(&mut prj, "coil", p(80.0, 80.0));
    let segs = autoconnect(&prj.symbol_table(), &prj.pages[0], &BTreeSet::from([top]), 400.0);
    assert_eq!(segs, vec![(p(40.0, 44.0), p(40.0, 76.0))]);
    assert_eq!(apply_autoconnect(&mut prj, 0, &BTreeSet::from([top])), 1);
    // The pins are connected now.
    assert!(autoconnect(&prj.symbol_table(), &prj.pages[0], &BTreeSet::from([top]), 400.0).is_empty());
}

#[test]
fn autoconnect_stops_at_symbols_in_between() {
    let mut prj = Project::default();
    let top = sym(&mut prj, "no", p(40.0, 40.0));
    sym(&mut prj, "coil", p(40.0, 120.0));
    // A symbol beside the line but overlapping it.
    sym(&mut prj, "no", p(41.0, 80.0));
    assert!(autoconnect(&prj.symbol_table(), &prj.pages[0], &BTreeSet::from([top]), 400.0).is_empty());
}

#[test]
fn route_goes_around_symbols() {
    let mut prj = Project::default();
    sym(&mut prj, "coil", p(40.0, 40.0));
    let path = route(&prj.symbol_table(), &prj.pages[0], area(), p(20.0, 40.0), p(60.0, 40.0));
    assert_eq!(path.first(), Some(&p(20.0, 40.0)));
    assert_eq!(path.last(), Some(&p(60.0, 40.0)));
    assert!(path.len() > 2, "{path:?}");
    // Orthogonal and outside the coil.
    for w in path.windows(2) {
        assert!(w[0].x == w[1].x || w[0].y == w[1].y);
    }
    let coil = prj.symbol("coil").unwrap().world_bounds(p(40.0, 40.0), 0, false);
    for q in &path[1..path.len() - 1] {
        assert!(!coil.contains(*q));
    }
}

#[test]
fn route_does_not_run_along_wires() {
    let mut prj = Project::default();
    prj.pages[0].wires.push(Wire { id: 99, a: p(0.0, 60.0), b: p(200.0, 60.0), ..Default::default() });
    let path = route(&prj.symbol_table(), &prj.pages[0], area(), p(40.0, 40.0), p(80.0, 80.0));
    for w in path.windows(2) {
        let horizontal_on_wire = w[0].y == 60.0 && w[1].y == 60.0;
        assert!(!horizontal_on_wire, "{path:?}");
    }
    // No corner on the wire.
    assert!(path.iter().all(|q| q.y != 60.0));
}

#[test]
fn straight_route_is_one_segment() {
    let prj = Project::default();
    assert_eq!(route(&prj.symbol_table(), &prj.pages[0], area(), p(20.0, 40.0), p(60.0, 40.0)), vec![p(20.0, 40.0), p(60.0, 40.0)]);
}

#[test]
fn collinear_wires_are_merged() {
    let mut page = Page::default();
    page.wires.push(Wire { id: 1, a: p(0.0, 0.0), b: p(8.0, 0.0), ..Default::default() });
    page.wires.push(Wire { id: 2, a: p(8.0, 0.0), b: p(16.0, 0.0), ..Default::default() });
    page.wires.push(Wire { id: 3, a: p(16.0, 0.0), b: p(16.0, 8.0), ..Default::default() });
    assert_eq!(merge_collinear(&Symbols::default(), &mut page), 1);
    assert_eq!(page.wires.len(), 2);
    assert_eq!((page.wires[0].a, page.wires[0].b), (p(0.0, 0.0), p(16.0, 0.0)));
}

#[test]
fn junctions_are_not_merged() {
    let mut page = Page::default();
    page.wires.push(Wire { id: 1, a: p(0.0, 0.0), b: p(8.0, 0.0), ..Default::default() });
    page.wires.push(Wire { id: 2, a: p(8.0, 0.0), b: p(16.0, 0.0), ..Default::default() });
    page.wires.push(Wire { id: 3, a: p(8.0, 0.0), b: p(8.0, 8.0), ..Default::default() });
    assert_eq!(merge_collinear(&Symbols::default(), &mut page), 0);
}

#[test]
fn autoconnect_wires_into_the_free_arms_of_connection_symbols() {
    let mut prj = Project::default();
    // A contact above the corner, its lower pin at (60, 44) facing down; a coil to
    // the right turned by 90°, its A2 at (116, 80) facing left.
    let contact = sym(&mut prj, "no", p(60.0, 40.0));
    let coil = sym(&mut prj, "coil", p(120.0, 80.0));
    prj.pages[0].symbols.iter_mut().find(|s| s.id == coil).unwrap().rot = 1;
    let corner = prj.new_id();
    prj.pages[0].junctions.push(Junction { id: corner, ..Junction::corner(p(60.0, 80.0), TeeDir::Up, TeeDir::Right) });
    // Placing the corner wires both pins in.
    let segs = autoconnect(&prj.symbol_table(), &prj.pages[0], &BTreeSet::from([corner]), 400.0);
    assert_eq!(segs, vec![(p(60.0, 80.0), p(60.0, 44.0)), (p(60.0, 80.0), p(116.0, 80.0))]);
    // A symbol placed facing a free arm connects to it; a used arm takes no second wire.
    let wid = prj.new_id();
    prj.pages[0].wires.push(Wire { id: wid, a: p(60.0, 80.0), b: p(60.0, 44.0), ..Default::default() });
    assert_eq!(autoconnect(&prj.symbol_table(), &prj.pages[0], &BTreeSet::from([contact]), 400.0), vec![]);
    assert_eq!(autoconnect(&prj.symbol_table(), &prj.pages[0], &BTreeSet::from([coil]), 400.0), vec![(p(116.0, 80.0), p(60.0, 80.0))]);
    assert_eq!(apply_autoconnect(&mut prj, 0, &BTreeSet::from([corner])), 1);
    assert!(autoconnect(&prj.symbol_table(), &prj.pages[0], &BTreeSet::from([corner, coil, contact]), 400.0).is_empty());
    assert!(crate::netlist::junctions::fits(&prj, &prj.pages[0], &prj.pages[0].junctions[0]));
    // A T-junction's three arms are wired the same way; its arm away from any pin stays open.
    let mut prj = Project::default();
    sym(&mut prj, "no", p(60.0, 40.0));
    let tee = prj.new_id();
    prj.pages[0].junctions.push(Junction { id: tee, pos: p(60.0, 80.0), dir: TeeDir::Down, variant: 3, ..Default::default() });
    assert_eq!(autoconnect(&prj.symbol_table(), &prj.pages[0], &BTreeSet::from([tee]), 400.0), vec![]);
    prj.pages[0].junctions[0].dir = TeeDir::Up;
    assert_eq!(autoconnect(&prj.symbol_table(), &prj.pages[0], &BTreeSet::from([tee]), 400.0), vec![(p(60.0, 80.0), p(60.0, 44.0))]);
}
