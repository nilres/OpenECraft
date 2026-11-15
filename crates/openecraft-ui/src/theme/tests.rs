use super::*;
use egui::Theme::{Dark, Light};

#[test]
fn the_portal_setting_chooses() {
    assert_eq!(desktop_theme(Some(1), None), Dark);
    assert_eq!(desktop_theme(Some(2), None), Light);
    // An explicit light scheme wins over a dark GTK theme.
    assert_eq!(desktop_theme(Some(2), Some("Adwaita:dark")), Light);
}

#[test]
fn without_a_preference_the_gtk_theme_chooses() {
    assert_eq!(desktop_theme(Some(0), Some("Adwaita:dark")), Dark);
    assert_eq!(desktop_theme(None, Some("Breeze-Dark")), Dark);
    assert_eq!(desktop_theme(None, Some("Adwaita")), Light);
}

#[test]
fn light_without_any_setting() {
    // The test systems of the AppImage catalog have neither: egui would choose dark.
    assert_eq!(desktop_theme(None, None), Light);
    assert_eq!(desktop_theme(Some(7), None), Light);
}

#[test]
fn system_falls_back_to_light() {
    let ctx = egui::Context::default();
    assert_eq!(ctx.options(|o| o.fallback_theme), Dark);
    set(&ctx, Light);
    assert_eq!(ctx.options(|o| o.fallback_theme), Light);
    assert_eq!(ctx.theme(), Light);
}

#[cfg(target_os = "linux")]
#[test]
fn reads_the_scheme_also_inside_a_variant() {
    use zbus::zvariant::Value;
    assert_eq!(linux::scheme(&Value::U32(1)), Some(1));
    assert_eq!(linux::scheme(&Value::Value(Box::new(Value::U32(2)))), Some(2));
    assert_eq!(linux::scheme(&Value::Str("dark".into())), None);
}
