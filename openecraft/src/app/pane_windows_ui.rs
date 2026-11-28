//! The windows of panes (see `crate::pane_windows`): each one a window of the system
//! (an egui viewport) that the user moves to another monitor, maximizes or shows full
//! screen; in the browser egui draws them as windows inside the page. Gather windows
//! and full screen.

use super::*;
use crate::dock::{self, Pane};

/// Where a window of panes was put when it opened. Its viewport is given the same
/// place every frame, so that egui does not move it back while the user moves it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Placed {
    rect: [f32; 4],
    maximized: bool,
    fullscreen: bool,
    /// The frame it was last shown in: a window not shown for a frame was closed by
    /// egui and opens again where it was last.
    frame: u64,
}

fn viewport_id(id: u32) -> egui::ViewportId {
    egui::ViewportId::from_hash_of(("pane_window", id))
}

impl App {
    /// Opens a pane in a window of its own.
    pub(super) fn open_in_window(&mut self, pane: Pane) {
        if self.in_dock {
            self.deferred.push(Deferred::Window(pane));
            return;
        }
        self.floating.retain(|p| *p != pane);
        if let Some(id) = self.shell.windows.open(&mut self.dock, pane) {
            self.raise_window = Some(id);
            if pane.is_document() {
                self.active_doc = pane;
            }
            self.active_pane = Some(pane);
        }
    }

    /// The title of a window of panes: the panes it shows.
    fn pane_window_title(&self, id: u32) -> String {
        let Some(w) = self.shell.windows.get(id) else { return String::new() };
        let shown = dock::visible(&w.tree);
        let names: Vec<String> = shown.iter().map(|p| self.pane_title(*p).text().to_string()).collect();
        format!("{} – OpenECraft", names.join(" + "))
    }

    /// Draws the windows of panes of the workspace.
    pub(super) fn pane_windows(&mut self, ctx: &egui::Context, idx: &Index) {
        if self.shell.windows.is_empty() {
            return;
        }
        let frame = ctx.cumulative_frame_nr();
        let main = ctx.input(|i| i.viewport().outer_rect);
        let fill = ctx.global_style().visuals.panel_fill;
        self.window_placed.retain(|id, _| self.shell.windows.get(*id).is_some());
        for (n, id) in self.shell.windows.ids().into_iter().enumerate() {
            let title = self.pane_window_title(id);
            let Some(w) = self.shell.windows.get_mut(id) else { continue };
            // A new window opens over the main window, a little offset per window.
            let default = || {
                let at = main.map_or(egui::pos2(80.0, 80.0), |r| r.min) + egui::vec2(60.0, 60.0) * (n + 1) as f32;
                let size = main.map_or(egui::vec2(1100.0, 760.0), |r| r.size() * 0.75);
                [at.x, at.y, size.x, size.y]
            };
            let fresh = Placed { rect: w.rect.unwrap_or_else(default), maximized: w.maximized, fullscreen: w.fullscreen, frame };
            let placed = self.window_placed.entry(id).or_insert(fresh);
            if placed.frame + 1 < frame {
                *placed = fresh;
            }
            placed.frame = frame;
            let placed = *placed;
            let [x, y, width, height] = placed.rect;
            let builder = egui::ViewportBuilder::default()
                .with_title(title)
                .with_position([x, y])
                .with_inner_size([width, height])
                .with_min_inner_size([320.0, 240.0])
                .with_maximized(placed.maximized)
                .with_fullscreen(placed.fullscreen);
            let mut tree = std::mem::replace(&mut w.tree, egui_tiles::Tree::empty(egui::Id::unique("pane_window_swap")));
            let mut request = None;
            let mut closed = false;
            let mut info = None;
            // The window's tree is out of `windows`: what its panes and keys ask for
            // waits until it is back (`deferred`).
            self.in_dock = true;
            ctx.show_viewport_immediate(viewport_id(id), builder, |ui, class| {
                if class == egui::ViewportClass::EmbeddedWindow {
                    ui.set_min_size(egui::vec2(480.0, 320.0));
                    request = self.tree_ui(ui, idx, &mut tree, Some(id));
                    return;
                }
                let vctx = ui.ctx().clone();
                closed = vctx.input(|i| i.viewport().close_requested());
                info = Some(vctx.input(|i| {
                    let v = i.viewport();
                    (v.outer_rect, v.inner_rect, v.maximized, v.fullscreen)
                }));
                // The keys of the commands work in the window too.
                self.window_keys(&vctx);
                egui::CentralPanel::default().frame(egui::Frame::NONE.fill(fill)).show(ui, |ui| request = self.tree_ui(ui, idx, &mut tree, Some(id)));
            });
            self.in_dock = false;
            let Some(w) = self.shell.windows.get_mut(id) else { continue };
            w.tree = tree;
            if let Some((outer, inner, maximized, fullscreen)) = info {
                w.maximized = maximized.unwrap_or(w.maximized);
                w.fullscreen = fullscreen.unwrap_or(w.fullscreen);
                // The place to come back to: the one before it was maximized.
                if let (Some(o), Some(i), false, false) = (outer, inner, w.maximized, w.fullscreen) {
                    w.rect = Some([o.min.x, o.min.y, i.width(), i.height()]);
                }
            }
            // The close button of a tab closed its last pane.
            if w.tree.root().is_none() || w.panes().is_empty() {
                self.shell.windows.list.retain(|w| w.id != id);
            }
            if closed {
                self.shell.windows.close(&mut self.dock, id);
            }
            if let Some(r) = request {
                self.pane_request(r, Some(id));
            }
        }
        if let Some(id) = self.raise_window.take() {
            if self.shell.windows.get(id).is_some() {
                ctx.send_viewport_cmd_to(viewport_id(id), egui::ViewportCommand::Focus);
            }
        }
    }

    /// Gather windows: the windows of panes come onto the monitor of the main window,
    /// neither maximized nor full screen (a window may be on a monitor that is gone).
    pub(super) fn gather_windows(&mut self, ctx: &egui::Context) {
        let main = ctx.input(|i| i.viewport().outer_rect);
        let at = main.map_or(egui::pos2(40.0, 40.0), |r| r.min);
        let size = main.map_or(egui::vec2(1100.0, 760.0), |r| r.size() * 0.75);
        for (n, w) in self.shell.windows.list.iter_mut().enumerate() {
            let vid = viewport_id(w.id);
            let pos = at + egui::vec2(48.0, 48.0) * (n + 1) as f32;
            ctx.send_viewport_cmd_to(vid, egui::ViewportCommand::Fullscreen(false));
            ctx.send_viewport_cmd_to(vid, egui::ViewportCommand::Maximized(false));
            ctx.send_viewport_cmd_to(vid, egui::ViewportCommand::OuterPosition(pos));
            ctx.send_viewport_cmd_to(vid, egui::ViewportCommand::InnerSize(size));
            ctx.send_viewport_cmd_to(vid, egui::ViewportCommand::Focus);
            w.rect = Some([pos.x, pos.y, size.x, size.y]);
            w.maximized = false;
            w.fullscreen = false;
        }
    }

    /// Full screen for the window in use (the main window or a window of panes), or
    /// back.
    pub(super) fn toggle_full_screen(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(|i| i.raw.viewports.iter().find(|(_, v)| v.focused == Some(true)).map(|(id, v)| (*id, v.fullscreen.unwrap_or(false))));
        let (id, full) = focused.unwrap_or_else(|| (egui::ViewportId::ROOT, ctx.input(|i| i.viewport().fullscreen.unwrap_or(false))));
        ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Fullscreen(!full));
    }
}

#[cfg(test)]
mod tests;
