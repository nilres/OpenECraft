use super::*;
use egui::{pos2, vec2, Rect, Sense};
use openecraft_core::panel::ItemKind;

#[test]
fn hulls_and_points_inside_them() {
    let square = vec![pos2(0.0, 0.0), pos2(10.0, 0.0), pos2(10.0, 10.0), pos2(0.0, 10.0)];
    // The corners of a box seen from the front: the inner ones are left out.
    let mut pts = square.clone();
    pts.extend([pos2(2.0, 2.0), pos2(8.0, 3.0), pos2(10.0, 10.0)]);
    let hull = convex_hull(pts);
    assert_eq!(hull.len(), 4);
    assert!(square.iter().all(|q| hull.contains(q)));
    assert!(inside(&hull, pos2(5.0, 5.0)));
    assert!(!inside(&hull, pos2(11.0, 5.0)));
    assert_eq!(area(&hull), 100.0);
    // Points on a line make no outline.
    assert_eq!(area(&convex_hull(vec![pos2(0.0, 0.0), pos2(1.0, 1.0), pos2(2.0, 2.0)])), 0.0);
}

/// The Striebel & John wall cabinet of 3 fields and 7 rows and its distribution panel
/// of one field (FV17), from the shipped catalog.
const CABINET: &str = "SJ.2CPX052032R9999";
const FV17: &str = "SJ.2CPX054002R9999";

#[test]
fn a_distribution_panel_dragged_over_the_cabinet_drops_into_a_free_field() {
    let mut prj = Project::default();
    let mut ui = PanelUi::default();
    let db = PartsDb::default();
    ui.drop_part(&mut prj, &db, CABINET, None).unwrap();
    let ctx = egui::Context::default();
    // Face coordinates at half scale on the screen.
    let to_screen = |q: P| pos2(q.x * 0.5, q.y * 0.5);
    let frame = |ui: &mut PanelUi, prj: &mut Project, t: f64, events: Vec<egui::Event>, at: Option<P>| {
        egui::DragAndDrop::set_payload(&ctx, PartDrag { number: FV17.into() });
        let input = egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0))), time: Some(t), events, ..Default::default() };
        ctx.run_ui(input, |ui_| {
            ui.track_dragged_part(ui_.ctx(), prj, &db);
            let resp = ui_.allocate_rect(Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0)), Sense::click_and_drag());
            ui.accessory_drop_2d(ui_, &resp, prj, 0, at, &to_screen);
        })
        .textures_delta
        .clear();
    };
    // Over the middle field: dropped there.
    let at = p(400.0, 500.0);
    let screen = to_screen(at);
    frame(&mut ui, &mut prj, 0.0, vec![egui::Event::PointerMoved(screen)], Some(at));
    frame(&mut ui, &mut prj, 0.1, vec![egui::Event::PointerMoved(screen)], Some(at));
    assert!(ui.status.contains(FV17), "{}", ui.status);
    let release = egui::Event::PointerButton { pos: screen, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() };
    frame(&mut ui, &mut prj, 0.2, vec![release], Some(at));
    let e = &prj.layout.enclosures[0];
    let panel = e.items.iter().find(|i| i.part == FV17).expect("dropped into the cabinet");
    assert_eq!((panel.kind, panel.face, panel.x, panel.y, panel.w, panel.h), (ItemKind::MountingPlate, Face::Plate, 250.0, 0.0, 250.0, 1050.0));
    assert!(prj.parts.iter().any(|p| p.number == FV17), "into the project's parts");
    assert_eq!(ui.selection.iter().copied().collect::<Vec<_>>(), [panel.id]);
}

#[test]
fn the_places_in_the_3d_view_are_outlined_where_they_are() {
    let mut prj = Project::default();
    PanelUi::default().drop_part(&mut prj, &PartsDb::default(), CABINET, None).unwrap();
    let part = find_part(&prj, &PartsDb::default(), FV17).unwrap().clone();
    let places = fit::places(&prj, 0, &part);
    let e = &prj.layout.enclosures[0];
    // Seen straight from the front, 1 mm a point, y up.
    let project_view = |q: V3| Some((pos2(q[0], 2000.0 - q[1]), 1.0));
    let pose = SitePose::default();
    let outline = outline_3d(e, &places[1], &pose, &project_view).unwrap();
    assert_eq!(outline.len(), 4);
    assert!((area(&outline) - 250.0 * 1050.0).abs() < 1.0, "{}", area(&outline));
    // The point of the face under the pointer, to a step of the grid searched.
    let (lo, _) = panel::item_box(e, &PanelItem { face: Face::Plate, x: 300.0, y: 400.0, ..Default::default() });
    let screen = project_view(pose.apply([lo[0], e.height - lo[1], lo[2]])).unwrap().0;
    assert!(inside(&outline, screen));
    let at = point_on_face(e, &places[1], &pose, &project_view, screen);
    assert!((at.x - 300.0).abs() <= 250.0 / 24.0 && (at.y - 400.0).abs() <= 1050.0 / 24.0, "{at:?}");
}
