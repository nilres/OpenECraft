//! The light or dark theme of the desktop for the theme setting "System".
//!
//! Windows, macOS and the browser report their theme to egui. On Linux winit reports none
//! (X11, Wayland) and egui would use its fallback, dark. There the colour scheme of the
//! desktop chooses: the setting `org.freedesktop.appearance color-scheme` of the settings
//! portal (GNOME, KDE and others, also in Flatpak and snap), otherwise a GTK theme named
//! dark (`GTK_THEME`). A change of the setting while the program runs is followed.
//! Without either the theme is light.

/// Makes the theme setting "System" light unless the desktop is set to dark.
pub fn follow_system(ctx: &egui::Context) {
    set(ctx, egui::Theme::Light);
    #[cfg(target_os = "linux")]
    linux::follow(ctx);
}

fn set(ctx: &egui::Context, theme: egui::Theme) {
    if ctx.options(|o| o.fallback_theme) != theme {
        ctx.options_mut(|o| o.fallback_theme = theme);
        ctx.request_repaint();
    }
}

/// The theme of the portal's colour scheme (0 no preference, 1 dark, 2 light) or, without
/// a preference, of the GTK theme name (`Adwaita:dark`, `Breeze-Dark`).
pub fn desktop_theme(color_scheme: Option<u32>, gtk_theme: Option<&str>) -> egui::Theme {
    match color_scheme {
        Some(1) => egui::Theme::Dark,
        Some(2) => egui::Theme::Light,
        _ if gtk_theme.is_some_and(|t| t.to_lowercase().contains("dark")) => egui::Theme::Dark,
        _ => egui::Theme::Light,
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::sync::mpsc;
    use std::time::Duration;
    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::{OwnedValue, Value};

    const NAMESPACE: &str = "org.freedesktop.appearance";
    const KEY: &str = "color-scheme";
    /// How long the start waits for the portal, so that the first frame has the theme.
    const FIRST_ANSWER: Duration = Duration::from_millis(300);

    /// Reads and follows the setting in a thread; a slow portal changes the theme later.
    pub fn follow(ctx: &egui::Context) {
        let (answered, first) = mpsc::channel();
        let ctx = ctx.clone();
        let spawned = std::thread::Builder::new().name("system theme".into()).spawn(move || {
            let gtk_theme = std::env::var("GTK_THEME").ok();
            let theme = |scheme| super::desktop_theme(scheme, gtk_theme.as_deref());
            let portal = portal();
            // Subscribed before reading, so that no change gets lost in between.
            let changes = portal.as_ref().and_then(|p| p.receive_signal("SettingChanged").ok());
            super::set(&ctx, theme(portal.as_ref().and_then(read)));
            answered.send(()).ok();
            for msg in changes.into_iter().flatten() {
                if let Ok((namespace, key, value)) = msg.body().deserialize::<(String, String, OwnedValue)>() {
                    if namespace == NAMESPACE && key == KEY {
                        super::set(&ctx, theme(scheme(&value)));
                    }
                }
            }
        });
        if spawned.is_ok() {
            first.recv_timeout(FIRST_ANSWER).ok();
        }
    }

    fn portal() -> Option<Proxy<'static>> {
        let conn = Connection::session().ok()?;
        Proxy::new(&conn, "org.freedesktop.portal.Desktop", "/org/freedesktop/portal/desktop", "org.freedesktop.portal.Settings").ok()
    }

    /// The colour scheme; `Read` for portals older than `ReadOne` (version 2).
    fn read(portal: &Proxy<'_>) -> Option<u32> {
        portal
            .call::<_, _, OwnedValue>("ReadOne", &(NAMESPACE, KEY))
            .or_else(|_| portal.call::<_, _, OwnedValue>("Read", &(NAMESPACE, KEY)))
            .ok()
            .and_then(|v| scheme(&v))
    }

    /// The number in the answer, also inside the extra variant of `Read`.
    pub(super) fn scheme(value: &Value<'_>) -> Option<u32> {
        match value {
            Value::U32(n) => Some(*n),
            Value::Value(inner) => scheme(inner),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
