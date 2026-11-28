use super::*;
use openecraft_core::panel;

/// The editor with the demo project open and analysed.
fn app() -> (App, egui::Context) {
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    app.session = crate::session::Session::new(openecraft_core::sample::demo_project(), None);
    app.closed = false;
    analyse(&mut app);
    (app, ctx)
}

/// The analysis of the project as it is (the session builds the first one in the
/// background).
fn analyse(app: &mut App) {
    app.session.analysis = openecraft_core::analysis::Analysis::checked(&app.session.prj);
    app.session.index = openecraft_core::index::Index::build(&app.session.prj);
}

/// A placed device of the demo: the page and the symbol of its main function, the item.
fn placed_main(app: &App) -> (usize, u64, (usize, u64)) {
    let prj = &app.session.prj;
    app.session
        .analysis
        .index
        .devices
        .values()
        .flat_map(|d| d.mains.iter())
        .find_map(|l| {
            let s = prj.pages[l.page].symbols.iter().find(|s| s.id == l.sym_id)?;
            let (ei, it) = panel::locate_symbol(prj, s)?;
            Some((l.page, s.id, (ei, it.id)))
        })
        .unwrap()
}

/// An edit made: committed and analysed, as the next frame does.
fn commit(app: &mut App) {
    app.session.commit(true);
    analyse(app);
}

#[test]
fn a_change_is_taken_from_the_view_that_changed() {
    let none = LayoutPick::default();
    let item = LayoutPick { items: vec![(0, 5)], ..Default::default() };
    assert_eq!(changed(None, &(1, vec![]), &item), Changed::Nothing, "nothing selected on the page yet");
    assert_eq!(changed(None, &(1, vec![7]), &none), Changed::Page);
    let last = ((1, vec![7]), item.clone());
    assert_eq!(changed(Some(&last), &(1, vec![7]), &item), Changed::Nothing);
    assert_eq!(changed(Some(&last), &(1, vec![8]), &item), Changed::Page);
    assert_eq!(changed(Some(&last), &(1, vec![]), &item), Changed::Page, "deselected on the page");
    assert_eq!(changed(Some(&last), &(2, vec![]), &item), Changed::Nothing, "another page shown: the layout keeps its selection");
    assert_eq!(changed(Some(&last), &(1, vec![7]), &none), Changed::Layout);
}

#[test]
fn cross_select_mode_keeps_both_selections_in_step() {
    let (mut app, ctx) = app();
    let (page, sym, item) = placed_main(&app);
    app.run(&ctx, Cmd::CrossSelect);
    assert!(app.shell.cross_select);
    // Selected on the page: selected in the panel layout.
    app.session.page = page;
    app.editor.selection = [sym].into_iter().collect();
    app.cross_select_frame();
    assert_eq!(app.panel_ui.pick().items, vec![item]);
    assert!(app.cmd_enabled(Cmd::ShowInSchematic));
    // A wire picked in the 3D view: its wires selected on their page.
    let an = &app.session.analysis;
    let (c, key) = an.connections.iter().find_map(|c| {
        let key = an.wiring.iter().find(|w| w.from == openecraft_core::netlist::pin_designation(&app.session.prj, c.a) && w.to == openecraft_core::netlist::pin_designation(&app.session.prj, c.b))?.key();
        (!c.wires.is_empty()).then_some((c.clone(), key))
    }).unwrap();
    app.panel_ui.cross_select(&app.session.prj, &app.session.analysis, &LayoutPick { wires: vec![key], ..Default::default() });
    app.cross_select_frame();
    assert_eq!(app.session.page, c.wires[0].0);
    assert!(app.editor.selection.contains(&c.wires[0].1));
    // Back to the device, then deleted in the 3D view: the page has nothing selected.
    app.session.page = page;
    app.editor.selection = [sym].into_iter().collect();
    app.cross_select_frame();
    panel::remove_items(&mut app.session.prj.layout.enclosures[item.0], &[item.1]);
    commit(&mut app);
    app.cross_select_frame();
    assert!(app.panel_ui.pick().is_empty());
    assert!(app.editor.selection.is_empty(), "the deletion reaches the schematic at once");
    // Off: the views select on their own.
    app.run(&ctx, Cmd::CrossSelect);
    app.editor.selection = [sym].into_iter().collect();
    app.cross_select_frame();
    assert!(app.panel_ui.pick().is_empty());
}

#[test]
fn a_device_deleted_in_one_view_leaves_the_selection_of_the_other() {
    let (mut app, _ctx) = app();
    let (_, _, item) = placed_main(&app);
    app.panel_ui.cross_select(&app.session.prj, &app.session.analysis, &LayoutPick { items: vec![item], ..Default::default() });
    app.cross_select_frame();
    // Deleted with all of its functions in the schematic and from the layout (undo of
    // a placement in another window): the 3D view no longer has it selected.
    let tag = app.session.prj.layout.enclosures[item.0].item(item.1).unwrap().tag.clone();
    for p in &mut app.session.prj.pages {
        p.symbols.retain(|s| s.tag != tag);
    }
    panel::remove_items(&mut app.session.prj.layout.enclosures[item.0], &[item.1]);
    commit(&mut app);
    app.cross_select_frame();
    assert!(app.panel_ui.pick().is_empty());
}

#[test]
fn show_in_schematic_goes_to_the_main_function() {
    let (mut app, ctx) = app();
    let (page, sym, item) = placed_main(&app);
    app.session.page = (page + 1) % app.session.prj.pages.len();
    app.panel_ui.cross_select(&app.session.prj, &app.session.analysis, &LayoutPick { items: vec![item], ..Default::default() });
    app.run(&ctx, Cmd::ShowInSchematic);
    assert_eq!(app.session.page, page);
    assert!(app.editor.selection.contains(&sym));
    assert!(crate::dock::find(&app.dock, Pane::Page).is_some(), "the page is shown");
}

#[test]
fn show_in_3d_view_and_the_double_click_show_the_selection_in_the_layout() {
    let (mut app, ctx) = app();
    let (page, sym, item) = placed_main(&app);
    app.session.page = page;
    app.editor.selection = [sym].into_iter().collect();
    app.page_pick();
    assert!(app.cmd_enabled(Cmd::ShowIn3d));
    app.run(&ctx, Cmd::ShowIn3d);
    assert_eq!(app.panel_ui.view, View::ThreeD);
    assert_eq!(app.panel_ui.pick().items, vec![item]);
    assert!(app.pane_shown(Pane::LayoutView));
    // The double-click opens the properties unless the user chose the other view.
    app.set_workspace(Workspace::Schematic);
    app.panel_ui.cross_select(&app.session.prj, &app.session.analysis, &LayoutPick::default());
    assert!(!app.double_click_shows_layout(&ctx));
    app.shell.double_click_other = true;
    assert!(app.double_click_shows_layout(&ctx));
    assert_eq!(app.panel_ui.pick().items, vec![item]);
}

#[test]
fn show_in_3d_view_keeps_the_selection_of_the_page_in_cross_select_mode() {
    let (mut app, ctx) = app();
    let (page, sym, item) = placed_main(&app);
    let tag = app.session.prj.pages[page].symbols.iter().find(|s| s.id == sym).unwrap().tag.clone();
    app.shell.cross_select = true;
    app.session.page = page;
    app.cross_select_frame();
    // Another function of the device on the page, selected alone and shown in 3D.
    let other = app.session.prj.pages[page].symbols.iter().find(|s| s.tag == tag && s.id != sym).map(|s| s.id);
    app.editor.selection = [other.unwrap_or(sym)].into_iter().collect();
    app.cross_select_frame();
    let before = app.editor.selection.clone();
    app.page_pick();
    app.run(&ctx, Cmd::ShowIn3d);
    assert_eq!(app.panel_ui.pick().items, vec![item]);
    app.cross_select_frame();
    assert_eq!(app.editor.selection, before, "the page keeps what was selected there");
}
