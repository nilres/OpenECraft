use super::*;
use openecraft_core::sample;

/// A placed device of the demo: (enclosure, item id).
fn placed(prj: &Project) -> (usize, u64) {
    prj.layout.enclosures.iter().enumerate().find_map(|(ei, e)| e.items.iter().find(|i| i.kind == ItemKind::Device && !i.tag.is_empty() && i.terminal.is_empty() && panel::locate(prj, &i.tag).is_some_and(|(k, x)| k == ei && x.id == i.id)).map(|i| (ei, i.id))).unwrap()
}

#[test]
fn what_the_schematic_selects_is_selected_and_picked_back() {
    let prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let mut ui = PanelUi::default();
    let item = placed(&prj);
    let pick = LayoutPick { items: vec![item], ..Default::default() };
    assert!(ui.cross_select(&prj, &an, &pick));
    assert_eq!(ui.enclosure, item.0);
    assert_eq!(ui.pick(), pick);
    // A wire of the virtual wiring.
    let w = an.wiring.first().expect("the demo is wired");
    let pick = LayoutPick { wires: vec![w.key()], ..Default::default() };
    assert!(ui.cross_select(&prj, &an, &pick));
    assert!(ui.selection.is_empty());
    assert_eq!(ui.pick(), pick);
    // Nothing selected in the schematic: nothing here either.
    assert!(!ui.cross_select(&prj, &an, &LayoutPick::default()));
    assert!(ui.pick().is_empty());
}

#[test]
fn a_cable_selected_in_the_schematic_is_its_route() {
    let prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let Some(cable) = an.routing.items.iter().find(|i| i.kind == routing3d::RouteKind::Cable) else { return };
    let mut ui = PanelUi::default();
    let pick = LayoutPick { cables: vec![cable.key.clone()], ..Default::default() };
    assert!(ui.cross_select(&prj, &an, &pick));
    assert_eq!(ui.selected_route.as_deref(), Some(cable.key.as_str()));
    assert_eq!(ui.pick(), pick);
}

#[test]
fn items_deleted_meanwhile_are_not_selected() {
    let mut prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let item = placed(&prj);
    panel::remove_items(&mut prj.layout.enclosures[item.0], &[item.1]);
    let mut ui = PanelUi::default();
    assert!(!ui.cross_select(&prj, &an, &LayoutPick { items: vec![item], ..Default::default() }));
    assert!(ui.selection.is_empty());
}

#[test]
fn show_pick_brings_the_device_into_the_3d_view() {
    let prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let item = placed(&prj);
    let mut ui = PanelUi::default();
    assert!(ui.show_pick(&prj, &an, &LayoutPick { items: vec![item], ..Default::default() }, true));
    assert_eq!(ui.view, View::ThreeD);
    assert_eq!(ui.selection, [item.1].into_iter().collect());
    assert!(!ui.show_pick(&prj, &an, &LayoutPick::default(), true));
}

#[test]
fn what_another_view_deleted_leaves_the_selection() {
    let mut prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let item = placed(&prj);
    let mut ui = PanelUi::default();
    assert!(ui.cross_select(&prj, &an, &LayoutPick { items: vec![item], ..Default::default() }));
    ui.drop_missing(&prj, &an);
    assert_eq!(ui.selection.len(), 1, "still there");
    // Deleted in the schematic's view of the layout (another pane or window).
    panel::remove_items(&mut prj.layout.enclosures[item.0], &[item.1]);
    let an = Analysis::checked(&prj);
    ui.drop_missing(&prj, &an);
    assert!(ui.pick().is_empty());
    // A wire whose device is gone from the schematic.
    let mut prj = sample::demo_project();
    let an = Analysis::checked(&prj);
    let w = an.wiring.first().unwrap().clone();
    assert!(ui.cross_select(&prj, &an, &LayoutPick { wires: vec![w.key()], ..Default::default() }));
    let tag = prj.layout.enclosures[w.enclosure].item(w.items.0).unwrap().tag.clone();
    for page in &mut prj.pages {
        page.symbols.retain(|s| s.tag != tag);
    }
    let an = Analysis::checked(&prj);
    ui.drop_missing(&prj, &an);
    assert!(ui.selected_wire.is_none());
}
