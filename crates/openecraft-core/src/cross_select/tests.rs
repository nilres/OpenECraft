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

/// The symbols with the tag `tag` on all pages.
fn with_tag(prj: &Project, tag: &str) -> usize {
    prj.pages.iter().flat_map(|p| &p.symbols).filter(|s| s.tag == tag).count()
}

fn items_with_tag(prj: &Project, tag: &str) -> usize {
    prj.layout.enclosures.iter().flat_map(|e| &e.items).filter(|it| it.tag == tag).count()
}

#[test]
fn a_device_deleted_completely_leaves_the_pages_the_layout_and_the_device_data() {
    let mut prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let (page, sym, item) = placed_main(&prj, &an);
    let tag = prj.pages[page].symbols.iter().find(|s| s.id == sym).unwrap().tag.clone();
    let symbols = with_tag(&prj, &tag);
    assert!(prj.devices.contains_key(&tag));
    let done = delete_completely(&mut prj, &[item], &[]);
    assert_eq!(done, Deleted { items: 1, symbols, kept: 0 });
    assert_eq!(with_tag(&prj, &tag), 0);
    assert_eq!(items_with_tag(&prj, &tag), 0);
    assert!(!prj.devices.contains_key(&tag));
}

#[test]
fn a_function_on_a_page_deletes_its_whole_device() {
    let mut prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let (page, sym, _) = placed_main(&prj, &an);
    let tag = prj.pages[page].symbols.iter().find(|s| s.id == sym).unwrap().tag.clone();
    let done = delete_completely(&mut prj, &[], &[(page, sym)]);
    assert!(done.items >= 1 && done.symbols >= 1);
    assert_eq!(with_tag(&prj, &tag) + items_with_tag(&prj, &tag), 0);
    // Gone already: nothing more to delete.
    assert_eq!(delete_device(&mut prj, &tag), Deleted::default());
}

#[test]
fn a_terminal_goes_alone() {
    let mut prj = sample::demo_project();
    let terminals: Vec<(usize, u64, String, String)> = prj
        .pages
        .iter()
        .enumerate()
        .flat_map(|(pi, p)| p.symbols.iter().map(move |s| (pi, s)))
        .filter(|(_, s)| prj.symbol(&s.sym).is_some_and(|d| d.func == Func::Terminal) && !s.tag.is_empty() && !s.label.is_empty())
        .map(|(pi, s)| (pi, s.id, s.tag.clone(), s.label.clone()))
        .collect();
    let (pi, sid, tag, label) = terminals.first().cloned().expect("the demo has terminals");
    let others = terminals.iter().filter(|t| t.2 == tag && t.3 != label).count();
    assert!(others > 0, "the strip has more terminals");
    // Its place in the layout.
    let id = prj.new_id();
    prj.layout.enclosures[0].items.push(panel::PanelItem { id, kind: ItemKind::Device, tag: tag.clone(), terminal: label.clone(), ..Default::default() });
    let done = delete_completely(&mut prj, &[], &[(pi, sid)]);
    assert_eq!(done.items, 1);
    assert!(prj.layout.enclosures[0].item(id).is_none());
    assert!(!prj.pages.iter().flat_map(|p| &p.symbols).any(|s| s.tag == tag && s.label == label));
    assert_eq!(prj.pages.iter().flat_map(|p| &p.symbols).filter(|s| s.tag == tag).count(), others, "the other terminals of the strip stay");
}

#[test]
fn pages_that_cannot_be_edited_keep_their_functions() {
    let mut prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let (page, sym, item) = placed_main(&prj, &an);
    let tag = prj.pages[page].symbols.iter().find(|s| s.id == sym).unwrap().tag.clone();
    let on_page = prj.pages[page].symbols.iter().filter(|s| s.tag == tag).count();
    prj.pages[page].locked = true;
    let done = delete_completely(&mut prj, &[item], &[]);
    assert_eq!(done.kept, on_page);
    assert_eq!(with_tag(&prj, &tag), on_page);
    assert!(prj.devices.contains_key(&tag), "still used on the locked page");
}

#[test]
fn an_item_without_a_device_is_deleted_alone() {
    let mut prj = sample::demo_project();
    let (ei, id) = prj
        .layout
        .enclosures
        .iter()
        .enumerate()
        .find_map(|(ei, e)| e.items.iter().find(|it| it.kind != ItemKind::Device).map(|it| (ei, it.id)))
        .expect("the demo has a rail or a duct");
    let symbols: usize = prj.pages.iter().map(|p| p.symbols.len()).sum();
    assert_eq!(delete_completely(&mut prj, &[(ei, id)], &[]), Deleted { items: 1, symbols: 0, kept: 0 });
    assert_eq!(prj.pages.iter().map(|p| p.symbols.len()).sum::<usize>(), symbols);
}
