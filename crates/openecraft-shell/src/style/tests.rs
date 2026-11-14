use super::*;

#[test]
fn state_colours_reach_4_5_to_1_on_the_panel_background() {
    for dark in [false, true] {
        let bg = if dark { egui::Visuals::dark() } else { egui::Visuals::light() }.panel_fill;
        let c = colors(dark);
        for (name, col) in [("error", c.error), ("warning", c.warning), ("ok", c.ok), ("lock", c.lock)] {
            let ratio = contrast(col, bg);
            assert!(ratio >= 4.5, "{name} on {} background: {ratio:.2}", if dark { "dark" } else { "light" });
        }
    }
}

#[test]
fn contrast_is_symmetric_and_bounded() {
    assert!((contrast(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
    assert!((contrast(Color32::WHITE, Color32::BLACK) - 21.0).abs() < 0.01);
    assert!((contrast(Color32::GRAY, Color32::GRAY) - 1.0).abs() < 0.001);
}

#[test]
fn errors_are_shown_in_the_error_colour_not_as_success() {
    for dark in [false, true] {
        let c = colors(dark);
        assert_eq!(Level::Error.color(dark, Color32::GRAY), c.error);
        assert_eq!(Level::Warning.color(dark, Color32::GRAY), c.warning);
        assert_eq!(Level::Ok.color(dark, Color32::GRAY), c.ok);
        assert_eq!(Level::Info.color(dark, Color32::GRAY), Color32::GRAY);
        assert_ne!(Level::Error.color(dark, Color32::GRAY), Level::Ok.color(dark, Color32::GRAY));
    }
}
