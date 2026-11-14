//! The canvases of the website: the page editor and the panel layout view of the
//! editor (`openecraft-canvas`), each as an application of its own on the project of
//! the shared [`Studio`].

use crate::studio::Studio;
use openecraft_i18n::tr;
use openecraft_shell::{shortcuts, style};

/// What a canvas shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The page editor with the page chosen on the website.
    Schematic,
    /// The panel layout: front view or 3D.
    Layout,
}

impl Kind {
    /// The kind by its name on the page.
    pub fn from_name(name: &str) -> Option<Kind> {
        match name {
            "schematic" => Some(Kind::Schematic),
            "layout" => Some(Kind::Layout),
            _ => None,
        }
    }
}

/// Fonts, spacing and the dark theme of the website.
pub fn setup(ctx: &egui::Context) {
    openecraft_ui::fonts::install(ctx, &[]);
    openecraft_ui::fonts::follow_language(ctx);
    style::apply(ctx);
    ctx.set_theme(egui::Theme::Dark);
}

/// Draws a canvas: what the editor does around its views every frame (keys and
/// gestures, the analysis before, the undo step after) with the view in between.
pub fn show(ui: &mut egui::Ui, kind: Kind, studio: &mut Studio, gl: bool) {
    let ctx = ui.ctx().clone();
    style::set_dark(ui.visuals().dark_mode);
    openecraft_ui::nav::set(&ctx, studio.editor.gestures.nav(Default::default()));
    shortcuts::set(&ctx, &studio.editor.shortcuts);
    let pointer_down = ctx.input(|i| i.pointer.any_down());
    let busy = studio.editor.busy() || studio.panel.busy();
    studio.session.refresh(&studio.plugins, pointer_down || busy);
    egui::CentralPanel::default().show(ui, |ui| {
        if studio.session.opening().is_some() {
            ui.centered_and_justified(|ui| ui.label(tr("Loading…")));
            ctx.request_repaint();
            return;
        }
        match kind {
            Kind::Schematic => schematic(ui, studio),
            Kind::Layout => {
                studio.panel.gpu_3d = gl;
                studio.panel.keys_active = true;
                let s = &mut studio.session;
                studio.panel.canvas(ui, &mut s.prj, &s.analysis, &studio.parts);
            }
        }
    });
    let idle = !ctx.input(|i| i.pointer.any_down()) && ctx.memory(|m| m.focused().is_none());
    if studio.session.commit(idle && !studio.editor.busy() && !studio.panel.busy()) {
        studio.repaint();
    }
}

fn schematic(ui: &mut egui::Ui, studio: &mut Studio) {
    let s = &mut studio.session;
    s.clamp_page();
    studio.editor.keys_active = true;
    let index = std::mem::take(&mut s.index);
    let out = studio.editor.ui(ui, &mut s.prj, s.page, &index, &s.analysis);
    s.index = index;
    if let Some(message) = out.message {
        studio.message = message;
    }
    if let Some(url) = out.open_url {
        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
    }
}

#[cfg(test)]
mod tests;
