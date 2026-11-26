use super::*;

#[test]
fn right_click_opens_the_pane_menu_and_preserves_existing_item_menus() {
    for item_menu in [false, true] {
        let ctx = egui::Context::default();
        let mut at = egui::Pos2::ZERO;
        let mut pane_menu_seen = false;
        let mut item_menu_seen = false;
        let mut frame = |events| {
            ctx.run_ui(egui::RawInput { events, ..Default::default() }, |ui| {
                let background = pane_context_response(ui);
                let row = ui.button(tr("Help"));
                at = row.rect.center();
                if item_menu {
                    row.context_menu(|ui| { item_menu_seen = true; ui.label(tr("Help")); });
                }
                pane_context_popup(ui, &background, |ui| { pane_menu_seen = true; placeholder(ui); });
            }).textures_delta.clear();
            at
        };
        frame(Vec::new());
        let at = frame(Vec::new());
        frame(vec![egui::Event::PointerMoved(at)]);
        for pressed in [true, false] {
            frame(vec![egui::Event::PointerButton { pos: at, button: egui::PointerButton::Secondary, pressed, modifiers: Default::default() }]);
        }
        frame(Vec::new());
        assert_eq!(pane_menu_seen, !item_menu);
        assert_eq!(item_menu_seen, item_menu);
    }
}

#[test]
fn a_pane_context_menu_reopens_at_the_pointer_and_placeholder_closes_it() {
    for tab in [false, true] { check_context_reopening(tab); }
}

fn check_context_reopening(tab: bool) {
    let ctx = egui::Context::default();
    let frame = |events| {
        let mut id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600.0, 400.0))),
            events, ..Default::default()
        }, |ui| {
            ui.set_min_size(ui.available_size());
            let background = if tab {
                ui.interact(ui.available_rect_before_wrap(), ui.scope_id().with("tab_context"), egui::Sense::click())
            } else {
                pane_context_response(ui)
            };
            id = egui::Popup::default_response_id(&background);
            pane_context_popup(ui, &background, placeholder);
        });
        out.textures_delta.clear();
        (out, id)
    };
    let click = |pos, button, pressed| egui::Event::PointerButton { pos, button, pressed, modifiers: Default::default() };
    frame(Vec::new());
    for at in [egui::pos2(20.0, 20.0), egui::pos2(250.0, 200.0)] {
        frame(vec![egui::Event::PointerMoved(at)]);
        frame(vec![click(at, egui::PointerButton::Secondary, true)]);
        let (_, id) = frame(vec![click(at, egui::PointerButton::Secondary, false)]);
        assert!(egui::Popup::is_id_open(&ctx, id), "context menu did not open at {at:?}");
        assert_eq!(egui::Popup::position_of_id(&ctx, id), Some(at));
    }
    let (out, id) = frame(Vec::new());
    let at = text_position(&out.shapes, tr("Placeholder"));
    frame(vec![egui::Event::PointerMoved(at)]);
    frame(vec![click(at, egui::PointerButton::Primary, true)]);
    frame(vec![click(at, egui::PointerButton::Primary, false)]);
    assert!(!egui::Popup::is_id_open(&ctx, id));
}

fn text_position(shapes: &[egui::epaint::ClippedShape], text: &str) -> egui::Pos2 {
    shapes.iter().rev().find_map(|s| match &s.shape {
        egui::Shape::Text(t) if t.galley.job.text == text => Some(t.pos + t.galley.size() * 0.5),
        _ => None,
    }).unwrap_or_else(|| panic!("menu text missing: {text}"))
}

#[test]
fn pane_menu_defers_opening_and_closing_until_after_drawing() {
    for visible in [false, true] {
        let ctx = egui::Context::default();
        let target = Pane::Navigator(LeftTab::Devices);
        let mut request = None;
        let mut frame = |events| {
            let mut output = ctx.run_ui(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 700.0))),
                events,
                ..Default::default()
            }, |ui| {
                let picked = panes_menu(ui, |p| p == target && visible, |p| {
                    tr(crate::commands::def(crate::commands::for_pane(p).unwrap()).title).into()
                });
                if picked.is_some() {
                    request = picked;
                }
            });
            output.textures_delta.clear();
            output
        };
        let click = |pos, pressed| egui::Event::PointerButton {
            pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE,
        };
        frame(Vec::new());
        let output = frame(Vec::new());
        let group = text_position(&output.shapes, tr("Navigators"));
        frame(vec![egui::Event::PointerMoved(group), click(group, true)]);
        frame(vec![click(group, false)]);
        let output = frame(Vec::new());
        let label = tr(crate::commands::def(crate::commands::for_pane(target).unwrap()).title);
        let entry = text_position(&output.shapes, label);
        frame(vec![egui::Event::PointerMoved(entry)]);
        frame(vec![click(entry, true)]);
        frame(vec![click(entry, false)]);
        assert_eq!(request, Some(if visible { PaneRequest::Close(target) } else { PaneRequest::Show(target) }));
    }
}
