use super::*;

#[test]
fn panel_layout_pages_are_nested_without_moving_other_pages() {
    let prj = Project {
        pages: vec![
            Page::new(1, "1", "", PageKind::Cover),
            Page::new(2, "8", "", PageKind::Mounting(10)),
            Page::new(3, "2", "", PageKind::Schematic),
            Page::new(4, "9", "", PageKind::Mounting(20)),
            Page::new(5, "8.1", "", PageKind::Mounting(10)),
        ],
        ..Default::default()
    };
    assert_eq!(page_groups(&prj, &[0, 1, 2, 3, 4]), vec![
        PageGroup::Page(0),
        PageGroup::PanelLayout(vec![PanelPages { enclosure: 10, pages: vec![1, 4] }, PanelPages { enclosure: 20, pages: vec![3] }]),
        PageGroup::Page(2),
    ]);
    assert_eq!(page_groups(&prj, &[0, 2]), vec![PageGroup::Page(0), PageGroup::Page(2)], "no empty layout group");
    assert_eq!(page_groups(&prj, &[2, 3]), vec![PageGroup::Page(2), PageGroup::PanelLayout(vec![PanelPages { enclosure: 20, pages: vec![3] }])], "only the pages of this section or work area");
    assert!(page_groups(&prj, &[]).is_empty());
}

#[test]
fn panel_page_groups_follow_the_panel_identity_after_renaming() {
    let mut prj = Project {
        pages: vec![Page::new(1, "8", "", PageKind::Mounting(10)), Page::new(2, "9", "", PageKind::Mounting(20))],
        ..Default::default()
    };
    prj.layout.enclosures = vec![
        openecraft_core::panel::Enclosure { id: 10, name: "+CAB1".into(), ..Default::default() },
        openecraft_core::panel::Enclosure { id: 20, name: "+CAB2".into(), ..Default::default() },
    ];
    let groups = page_groups(&prj, &[0, 1]);
    assert_eq!(panel_page_title(&prj, 10), "+CAB1");
    prj.layout.enclosures[0].name = "+MAIN".into();
    assert_eq!(panel_page_title(&prj, 10), "+MAIN");
    assert_eq!(page_groups(&prj, &[0, 1]), groups);
    prj.layout.enclosures.remove(0);
    assert!(!panel_page_title(&prj, 10).is_empty());
    assert_eq!(page_groups(&prj, &[0, 1]), groups, "pages remain accessible if their panel is missing");
}

#[test]
fn the_schematic_icon_keeps_the_page_row_clickable() {
    let ctx = egui::Context::default();
    let mut at = egui::Pos2::ZERO;
    let mut clicked = false;
    let mut frame = |events| {
        ctx.run_ui(egui::RawInput { events, ..Default::default() }, |ui| {
            let row = ui.add(egui::Button::selectable(false, (page_icon(PageKind::Schematic), tr("Schematic"))));
            at = egui::pos2(row.rect.left() + ui.spacing().button_padding.x + 8.0, row.rect.center().y);
            clicked |= row.clicked();
        }).textures_delta.clear();
        at
    };
    frame(Vec::new());
    let at = frame(Vec::new());
    frame(vec![egui::Event::PointerMoved(at)]);
    for pressed in [true, false] {
        frame(vec![egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() }]);
    }
    assert!(clicked, "clicking the circuit icon must select the page");
}
