//! The look of the editor: the colours of states (errors, warnings, locks, success) in a
//! light and a dark variant with enough contrast against the panel background, and the
//! spacing of the shell. No widgets.

use egui::Color32;
use std::sync::atomic::{AtomicBool, Ordering};

/// Colours of states.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateColors {
    pub error: Color32,
    pub warning: Color32,
    pub ok: Color32,
    pub lock: Color32,
}

/// The theme shown (set by the app every frame, read by the colour functions below).
static DARK: AtomicBool = AtomicBool::new(true);

pub fn set_dark(dark: bool) {
    DARK.store(dark, Ordering::Relaxed);
}

/// Colour of errors in the theme shown.
pub fn error() -> Color32 {
    colors(DARK.load(Ordering::Relaxed)).error
}

pub fn warning() -> Color32 {
    colors(DARK.load(Ordering::Relaxed)).warning
}

pub fn ok() -> Color32 {
    colors(DARK.load(Ordering::Relaxed)).ok
}

/// Locked, released and open (not connected) states.
pub fn lock() -> Color32 {
    colors(DARK.load(Ordering::Relaxed)).lock
}

/// The state colours for a light or a dark theme.
pub fn colors(dark: bool) -> StateColors {
    if dark {
        StateColors {
            error: Color32::from_rgb(240, 110, 105),
            warning: Color32::from_rgb(235, 175, 60),
            ok: Color32::from_rgb(110, 200, 130),
            lock: Color32::from_rgb(235, 165, 70),
        }
    } else {
        StateColors {
            error: Color32::from_rgb(190, 30, 30),
            warning: Color32::from_rgb(150, 95, 0),
            ok: Color32::from_rgb(30, 120, 55),
            lock: Color32::from_rgb(160, 90, 0),
        }
    }
}

/// Severity of a status message.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Level {
    #[default]
    Info,
    /// Something was done (saved, opened, exported).
    Ok,
    Warning,
    /// Something failed.
    Error,
}

impl Level {
    /// Colour of a message of this level; info messages keep `normal`.
    pub fn color(self, dark: bool, normal: Color32) -> Color32 {
        let c = colors(dark);
        match self {
            Level::Info => normal,
            Level::Ok => c.ok,
            Level::Warning => c.warning,
            Level::Error => c.error,
        }
    }
}

/// Relative luminance (WCAG 2.1).
#[cfg(test)]
pub fn luminance(c: Color32) -> f32 {
    let ch = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.039_28 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
}

/// Contrast ratio of two colours (1 … 21).
#[cfg(test)]
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Spacing of the shell (applied once at start).
pub fn apply(ctx: &egui::Context) {
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 4.0);
        s.spacing.button_padding = egui::vec2(6.0, 3.0);
        s.spacing.interact_size.y = 20.0;
    });
}

/// Touch sizes (tablet mode at the workbench): larger buttons and rows, no smaller
/// than 40 points.
pub fn touch(ctx: &egui::Context, on: bool) {
    ctx.all_styles_mut(|s| {
        if on {
            s.spacing.interact_size.y = 40.0;
            s.spacing.button_padding = egui::vec2(12.0, 8.0);
            s.spacing.item_spacing = egui::vec2(10.0, 8.0);
        } else {
            s.spacing.interact_size.y = 20.0;
            s.spacing.button_padding = egui::vec2(6.0, 3.0);
            s.spacing.item_spacing = egui::vec2(6.0, 4.0);
        }
    });
}

#[cfg(test)]
mod tests;
