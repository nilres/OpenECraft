use super::*;

#[test]
fn layout_pages_belong_to_the_selected_panel_and_keep_the_current_page() {
    let prj = Project {
        pages: vec![
            Page::new(1, "10", "", PageKind::Mounting(100)),
            Page::new(2, "2", "", PageKind::Mounting(100)),
            Page::new(3, "1", "", PageKind::Mounting(200)),
            Page::new(4, "3", "", PageKind::Schematic),
        ],
        ..Default::default()
    };
    assert_eq!(layout_page(&prj, 100, 0), Some(0), "keep the currently selected layout sheet");
    assert_eq!(layout_page(&prj, 100, 2), Some(1), "otherwise use the first numbered sheet of this panel");
    assert_eq!(layout_page(&prj, 200, 3), Some(2));
    assert_eq!(layout_page(&prj, 999, 0), None);
}

#[test]
fn the_panel_view_dropdown_offers_layout_3d_and_mounting_surface() {
    for has_layout in [false, true] {
        let ctx = egui::Context::default();
        let mut picked = None;
        let mut frame = |events| {
            let mut output = ctx.run_ui(egui::RawInput { events, ..Default::default() }, |ui| {
                picked = picked.or(layout_view_combo(ui, Some(LayoutView::ThreeD), has_layout, true));
            });
            output.textures_delta.clear();
            output.shapes.into_iter().filter_map(|shape| {
                if let egui::Shape::Text(text) = shape.shape {
                    Some((text.galley.text().to_string(), text.pos + text.galley.size() * 0.5))
                } else { None }
            }).collect::<Vec<_>>()
        };
        frame(Vec::new());
        let labels = frame(Vec::new());
        let at = labels.iter().find(|(text, _)| text == LayoutView::ThreeD.title()).unwrap().1;
        for pressed in [true, false] {
            frame(vec![egui::Event::PointerMoved(at), egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() }]);
        }
        frame(Vec::new());
        let labels = frame(Vec::new());
        for view in LayoutView::ALL {
            assert!(labels.iter().any(|(text, _)| text == view.title()), "{view:?} missing");
        }
        let at = labels.iter().find(|(text, _)| text == LayoutView::Layout.title()).unwrap().1;
        for pressed in [true, false] {
            frame(vec![egui::Event::PointerMoved(at), egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() }]);
        }
        assert_eq!(picked, has_layout.then_some(LayoutView::Layout), "no page is created merely by changing view");
    }
}
