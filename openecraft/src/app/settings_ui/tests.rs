use super::*;
use egui::{Context, Pos2, RawInput, Rect};
use openecraft_i18n::Lang;

/// Width of the settings dialog showing `section` of the keys and gestures, after a few
/// frames in `lang`, with `recording` going on.
fn dialog_width(section: Section, lang: Lang, recording: Option<(MouseAction, bool)>) -> f32 {
    openecraft_i18n::with_lang(lang, || {
        let ctx = Context::default();
        let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
        app.settings.open_at(Node::Shortcuts);
        app.settings.section = section;
        app.settings.recording = recording;
        for n in 0..4 {
            let input = RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1280.0, 900.0))), time: Some(f64::from(n) * 0.1), ..Default::default() };
            ctx.run_ui(input, |ui| app.settings_window(ui.ctx())).drop_without_applying_deltas();
        }
        ctx.memory(|m| m.layer_ids().filter(|l| l.order == egui::Order::Middle).filter_map(|l| m.area_rect(l.id)).map(|r| r.width()).fold(0.0, f32::max))
    })
}

#[test]
fn every_section_of_the_keys_and_gestures_fits_the_dialog() {
    for lang in [Lang::En, Lang::De, Lang::Fr] {
        let keys = dialog_width(Section::Keys, lang, None);
        assert!(keys > 500.0, "{lang:?}: {keys}");
        for (section, recording) in [
            (Section::Modifiers, None),
            (Section::Mouse, None),
            (Section::Mouse, Some((MouseAction::Pan3d, true))),
            (Section::WheelTouch, None),
        ] {
            let w = dialog_width(section, lang, recording);
            assert!(w <= keys + 1.0, "{lang:?} {section:?}: the dialog grows from {keys} to {w}");
        }
    }
}
