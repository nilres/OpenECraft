use super::*;

/// Draws a frame of a canvas without a window.
fn frame(ctx: &egui::Context, kind: Kind, studio: &mut Studio) {
    let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 600.0))), ..Default::default() };
    ctx.run_ui(input, |ui| show(ui, kind, studio, false)).drop_without_applying_deltas();
}

#[test]
fn both_views_draw_the_examples() {
    let ctx = egui::Context::default();
    setup(&ctx);
    let mut studio = Studio::default();
    // A first circuit, the project the page starts with and one with a line-up and rooms.
    let shown = ["lamp_circuit", crate::studio::FIRST_EXAMPLE, "cabinet_lineup"];
    for example in openecraft_core::sample::EXAMPLES.iter().filter(|e| shown.contains(&e.key)) {
        studio.open(example.key).unwrap();
        // The frame while the project opens shows the progress.
        frame(&ctx, Kind::Schematic, &mut studio);
        studio.settle();
        let before = studio.session.prj.clone();
        for kind in [Kind::Schematic, Kind::Layout] {
            frame(&ctx, kind, &mut studio);
            frame(&ctx, kind, &mut studio);
        }
        assert!(studio.session.prj == before, "{}: drawing changes nothing", example.key);
        assert!(!studio.session.history.can_undo(), "{}", example.key);
    }
}

#[test]
fn views_have_names() {
    assert_eq!(Kind::from_name("schematic"), Some(Kind::Schematic));
    assert_eq!(Kind::from_name("layout"), Some(Kind::Layout));
    assert_eq!(Kind::from_name("x"), None);
}
