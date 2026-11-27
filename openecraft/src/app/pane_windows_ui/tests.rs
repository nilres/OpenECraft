use super::*;
use super::super::dock_ui::PaneRequest;

fn app() -> (App, egui::Context) {
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    app.own_windows = true;
    app.set_workspace(Workspace::Panel);
    (app, ctx)
}

/// A frame with the windows of panes (embedded: the test has no window system).
fn frame(app: &mut App, ctx: &egui::Context) {
    ctx.run_ui(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1400., 900.))), ..Default::default() }, |ui| {
        let idx = std::mem::take(&mut app.session.index);
        app.pane_windows(ui.ctx(), &idx);
        app.session.index = idx;
    })
    .drop_without_applying_deltas();
}

#[test]
fn the_3d_view_opens_in_a_window_of_its_own() {
    let (mut app, ctx) = app();
    app.run(&ctx, Cmd::View3dInWindow);
    assert_eq!(app.panel_ui.view, crate::panel_ui::View::ThreeD);
    let id = app.shell.windows.find(Pane::LayoutView).expect("in a window");
    assert!(dock::find(&app.dock, Pane::LayoutView).is_none(), "not docked as well");
    assert_eq!(app.active_doc, Pane::LayoutView);
    assert!(app.pane_shown(Pane::LayoutView));
    frame(&mut app, &ctx);
    frame(&mut app, &ctx);
    assert_eq!(app.shell.windows.find(Pane::LayoutView), Some(id), "drawn without losing its pane");
    assert!(app.window_placed.contains_key(&id));
    assert!(app.pane_window_title(id).contains("OpenECraft"));

    // Showing it again brings its window to the front, nothing else.
    app.show_pane(Pane::LayoutView);
    assert_eq!(app.raise_window, Some(id));
    assert!(dock::find(&app.dock, Pane::LayoutView).is_none());
}

#[test]
fn windows_belong_to_their_workspace() {
    let (mut app, ctx) = app();
    app.run(&ctx, Cmd::View3dInWindow);
    app.set_workspace(Workspace::Schematic);
    assert!(app.shell.windows.is_empty(), "the schematic has no window");
    app.set_workspace(Workspace::Panel);
    assert!(app.shell.windows.find(Pane::LayoutView).is_some(), "back with its workspace");
    app.reset_layout();
    assert!(app.shell.windows.is_empty());
    assert!(dock::find(&app.dock, Pane::LayoutView).is_some());
}

#[test]
fn the_page_and_the_3d_view_side_by_side() {
    let (mut app, ctx) = app();
    app.run(&ctx, Cmd::View3dInWindow);
    app.run(&ctx, Cmd::PageBeside3d);
    assert!(app.shell.windows.is_empty(), "the view came back from its window");
    let shown = dock::visible(&app.dock);
    assert!(shown.contains(&Pane::Page) && shown.contains(&Pane::LayoutView), "{shown:?}");
}

#[test]
fn pane_menu_requests_move_panes_between_windows() {
    let (mut app, ctx) = app();
    app.pane_request(PaneRequest::NewWindow(Pane::LayoutProperties), None);
    let a = app.shell.windows.find(Pane::LayoutProperties).unwrap();
    app.pane_request(PaneRequest::NewWindow(Pane::LayoutView), None);
    app.pane_request(PaneRequest::MoveTo(Pane::LayoutView, a), Some(a + 1));
    assert_eq!(app.shell.windows.find(Pane::LayoutView), Some(a));
    assert_eq!(app.shell.windows.list.len(), 1);
    app.pane_request(PaneRequest::Split(Pane::LayoutView, egui_tiles::LinearDir::Horizontal), Some(a));
    let shown = dock::visible(&app.shell.windows.get(a).unwrap().tree);
    assert_eq!(shown.len(), 2, "side by side in the window");
    app.pane_request(PaneRequest::Back(Pane::LayoutView), Some(a));
    assert!(dock::find(&app.dock, Pane::LayoutView).is_some());
    app.pane_request(PaneRequest::Close(Pane::LayoutProperties), Some(a));
    assert!(app.shell.windows.is_empty());
    frame(&mut app, &ctx);
    // The page never leaves the main window.
    app.pane_request(PaneRequest::NewWindow(Pane::Page), None);
    assert!(app.shell.windows.is_empty());
}

#[test]
fn gather_windows_forgets_maximized_and_full_screen() {
    let (mut app, ctx) = app();
    app.run(&ctx, Cmd::View3dInWindow);
    let id = app.shell.windows.find(Pane::LayoutView).unwrap();
    let w = app.shell.windows.get_mut(id).unwrap();
    w.rect = Some([5000.0, 0.0, 800.0, 600.0]);
    w.maximized = true;
    assert!(app.cmd_enabled(Cmd::GatherWindows));
    app.run(&ctx, Cmd::GatherWindows);
    let w = app.shell.windows.get(id).unwrap();
    assert!(!w.maximized && !w.fullscreen);
    assert!(w.rect.unwrap()[0] < 1000.0, "on the main window's monitor");
}
