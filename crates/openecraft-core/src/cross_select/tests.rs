use super::*;
use crate::sample;

fn one(id: u64) -> BTreeSet<u64> {
    [id].into_iter().collect()
}

/// A placed device of the demo with a main function: its page and symbol, the item.
fn placed_main(prj: &Project, an: &Analysis) -> (usize, u64, (usize, u64)) {
    an.index
        .devices
        .values()
        .flat_map(|d| d.mains.iter())
        .find_map(|l| {
            let s = prj.pages[l.page].symbols.iter().find(|s| s.id == l.sym_id)?;
            let (ei, it) = panel::locate_symbol(prj, s)?;
            Some((l.page, s.id, (ei, it.id)))
        })
        .expect("the demo has a placed device")
}

#[test]
fn a_device_is_its_item_in_the_layout_and_back() {
    let prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let (page, sym, item) = placed_main(&prj, &an);
    let pick = in_layout(&prj, &an, page, &one(sym));
    assert_eq!(pick.items, vec![item]);
    let found = in_schematic(&prj, &an, &LayoutPick { items: vec![item], ..Default::default() });
    assert_eq!(found.first(), Some(&(page, sym)), "the main function comes first");
    let tag = &prj.pages[page].symbols.iter().find(|s| s.id == sym).unwrap().tag;
    assert!(found.iter().all(|(pi, id)| prj.pages[*pi].symbols.iter().any(|s| s.id == *id && &s.tag == tag)));
}

#[test]
fn a_wire_is_the_wire_of_the_virtual_wiring_and_back() {
    let prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let (c, key) = an
        .connections
        .iter()
        .find_map(|c| Some((c, routed_wire(&an, &ends(&prj, c)).filter(|_| !c.wires.is_empty())?)))
        .expect("the demo has a wired connection");
    let (page, wire) = c.wires[0];
    let pick = in_layout(&prj, &an, page, &one(wire));
    assert!(pick.wires.contains(&key));
    assert!(pick.items.is_empty());
    let found = in_schematic(&prj, &an, &LayoutPick { wires: vec![key], ..Default::default() });
    assert!(found.contains(&(page, wire)));
}

#[test]
fn a_cable_is_its_route_and_its_cores() {
    let mut prj = sample::demo_project();
    crate::testkit::with_cable(&mut prj);
    let an = Analysis::checked(&prj);
    let cores: Vec<(usize, u64)> = prj.pages.iter().enumerate().flat_map(|(pi, p)| p.wires.iter().filter(|w| w.cable == "-W77").map(move |w| (pi, w.id))).collect();
    assert!(!cores.is_empty());
    let found = in_schematic(&prj, &an, &LayoutPick { cables: vec!["-W77".into()], ..Default::default() });
    assert!(cores.iter().all(|c| found.contains(c)));
    // A cable definition on a page names the cable.
    let id = prj.new_id();
    prj.pages[cores[0].0].cable_defs.push(crate::model::CableDef { id, cable: "-W77".into(), ..Default::default() });
    let an = Analysis::checked(&prj);
    assert_eq!(in_layout(&prj, &an, cores[0].0, &one(id)).cables, vec!["-W77".to_string()]);
    assert_eq!(in_schematic(&prj, &an, &LayoutPick { cables: vec!["-W77".into()], ..Default::default() }).first(), Some(&(cores[0].0, id)));
}

#[test]
fn a_deleted_device_has_no_counterpart() {
    let mut prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let (page, sym, item) = placed_main(&prj, &an);
    // Gone from the layout: the symbol has nothing to select there.
    let e = &mut prj.layout.enclosures[item.0];
    panel::remove_items(e, &[item.1]);
    let an = Analysis::checked(&prj);
    assert!(in_layout(&prj, &an, page, &one(sym)).items.is_empty());
    assert!(in_schematic(&prj, &an, &LayoutPick { items: vec![item], ..Default::default() }).is_empty());
}

#[test]
fn the_current_page_is_kept_when_it_has_counterparts() {
    let found = [(3, 30), (1, 10), (3, 31), (1, 11)];
    assert_eq!(on_one_page(&found, 1), Some((1, vec![10, 11])));
    assert_eq!(on_one_page(&found, 5), Some((3, vec![30, 31])));
    assert_eq!(on_one_page(&[], 0), None);
}
