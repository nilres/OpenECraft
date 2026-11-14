//! Editing aids: typed coordinates, grid size, align and distribute, move the
//! selection by a distance, multiple copy.

use openecraft_i18n::{tr, tr_fmt};
use super::App;
use openecraft_core::arrange::{self, Align};
use openecraft_core::geom::p;
use openecraft_core::routing;

/// Grid sizes offered (mm).
pub const GRIDS: [f32; 5] = [1.0, 2.0, 4.0, 8.0, 16.0];

pub struct ArrangeUi {
    coord: String,
    core_labels: Option<(u64, u64, bool, f32)>,
    pub show_move: bool,
    pub show_copy: bool,
    /// Distance of Move selection by (mm): nothing moves until one is typed.
    move_by: (f32, f32),
    /// Distance between the copies of Multiple copy (mm).
    copy_by: (f32, f32),
    count: usize,
}

impl Default for ArrangeUi {
    fn default() -> Self {
        ArrangeUi { core_labels: None, coord: String::new(), show_move: false, show_copy: false, move_by: (0.0, 0.0), copy_by: (0.0, 40.0), count: 3 }
    }
}

/// Move selection by and Multiple copy act only for a distance other than zero.
pub(super) fn moves((dx, dy): (f32, f32)) -> bool {
    dx != 0.0 || dy != 0.0
}

pub(super) fn align_title(a: Align) -> &'static str {
    match a {
        Align::Left => tr("Left edges"),
        Align::Center => tr("Horizontal centres"),
        Align::Right => tr("Right edges"),
        Align::Top => tr("Top edges"),
        Align::Middle => tr("Vertical centres"),
        Align::Bottom => tr("Bottom edges"),
    }
}

impl App {
    /// Field for a typed point (tool bar): Enter places it like a click.
    pub(super) fn coordinate_field(&mut self, ui: &mut egui::Ui) {
        let r = ui
            .add(egui::TextEdit::singleline(&mut self.arrange_ui.coord).desired_width(80.0).hint_text(tr("x y")))
            .on_hover_text(tr("Type a point and press Enter: x y in mm, or @dx dy relative to the last point; it acts like a click of the current tool"));
        if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            match arrange::parse_point(&self.arrange_ui.coord, self.editor.last_point) {
                Some(q) => {
                    self.editor.typed = Some(q);
                    self.arrange_ui.coord.clear();
                }
                None => {
                    let ctx = ui.ctx().clone();
                    self.notify(&ctx, tr("Not a point: type x y or @dx dy"));
                }
            }
        }
    }

    pub(super) fn core_label_definition(&self) -> Option<u64> {
        if self.editor.selection.len() != 1 { return None; }
        self.session.prj.pages[self.session.page].cable_defs.iter().find(|d| self.editor.selection.contains(&d.id)).map(|d| d.id)
    }

    pub(super) fn start_core_label_alignment(&mut self) {
        if let Some(id) = self.core_label_definition().filter(|_| self.page_editable()) {
            self.arrange_ui.core_labels = Some((self.session.prj.pages[self.session.page].id, id, true, 0.8));
        }
    }

    fn core_label_window(&mut self, ctx: &egui::Context) {
        use openecraft_core::render;
        self.editor.core_label_preview.clear();
        let Some((page_id, id, mut before, mut gap)) = self.arrange_ui.core_labels else { return };
        let pi = self.session.page;
        if self.session.prj.pages[pi].id != page_id || !self.page_editable() {
            self.arrange_ui.core_labels = None; return;
        }
        let mut apply = false;
        let mut cancel = false;
        let (window, esc) = self.dialogs.window(ctx, tr("Align core labels…"), "core_label_align", None);
        let mut open = !esc;
        window.open(&mut open).resizable(false).show(ctx, |ui| {
            ui.label(tr("Preview only. Applying replaces the manual positions of these core labels."));
            ui.radio_value(&mut before, true, tr("Above / left of the definition line"));
            ui.radio_value(&mut before, false, tr("Below / right of the definition line"));
            ui.horizontal(|ui| {
                ui.label(tr("Clearance (mm)"));
                ui.add(egui::DragValue::new(&mut gap).range(0.5..=10.).speed(0.1));
            });
            let labels = render::preview_core_labels(&self.session.prj, pi, id, before, gap);
            let conflicts = labels.iter().filter(|l| l.conflict).count();
            ui.label(tr_fmt!("{} core labels; {} placement conflicts", labels.len(), conflicts));
            if conflicts > 0 { ui.colored_label(crate::style::warning(), tr("No clear placement found. Change the side or clearance, or move the labels individually.")); }
            ui.horizontal(|ui| {
                apply = ui.add_enabled(!labels.is_empty() && conflicts == 0, egui::Button::new(tr("Apply"))).clicked();
                cancel = ui.button(tr("Cancel")).clicked();
            });
            self.editor.core_label_preview = labels;
            self.editor.core_label_preview_page = Some(page_id);
        });
        if apply && self.page_editable() {
            render::align_core_labels(&mut self.session.prj, pi, id, before, gap);
        }
        if !open || apply || cancel {
            self.arrange_ui.core_labels = None;
            self.editor.core_label_preview.clear();
        } else { self.arrange_ui.core_labels = Some((page_id, id, before, gap)); }
    }

    pub(super) fn arrange_windows(&mut self, ctx: &egui::Context) {
        self.core_label_window(ctx);
        let page = self.session.page;
        if self.arrange_ui.show_move {
            let mut apply = false;
            let mut cancel = false;
            let (window, esc) = self.dialogs.window(ctx, tr("Move selection by"), "move_by", None);
            let mut open = !esc;
            let st = &mut self.arrange_ui;
            window.open(&mut open).resizable(false).show(ctx, |ui| {
                egui::Grid::new("movesel").num_columns(2).show(ui, |ui| {
                    ui.label(tr("Horizontal (mm)"));
                    ui.add(egui::DragValue::new(&mut st.move_by.0).speed(1.0));
                    ui.end_row();
                    ui.label(tr("Vertical (mm, downwards)"));
                    ui.add(egui::DragValue::new(&mut st.move_by.1).speed(1.0));
                    ui.end_row();
                });
                ui.horizontal(|ui| {
                    apply = ui.add_enabled(moves(st.move_by), egui::Button::new(tr("Move"))).clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
                });
            });
            if apply && !self.editor.selection.is_empty() && self.page_editable() {
                let (dx, dy) = self.arrange_ui.move_by;
                let items = self.editor.items(&self.session.prj, page);
                routing::move_selection_keeping(&mut self.session.prj, page, &items, p(dx, dy), self.editor.keep_connections);
            }
            self.arrange_ui.show_move = open && !apply && !cancel;
        }
        if self.arrange_ui.show_copy {
            let mut apply = false;
            let mut cancel = false;
            let (window, esc) = self.dialogs.window(ctx, tr("Multiple copy"), "multiple_copy", None);
            let mut open = !esc;
            let st = &mut self.arrange_ui;
            window.open(&mut open).resizable(false).show(ctx, |ui| {
                ui.label(tr("Copies the selection several times; devices get new tags, terminals new numbers."));
                egui::Grid::new("multicopy").num_columns(2).show(ui, |ui| {
                    ui.label(tr("Copies"));
                    ui.add(egui::DragValue::new(&mut st.count).range(1..=100));
                    ui.end_row();
                    ui.label(tr("Horizontal (mm)"));
                    ui.add(egui::DragValue::new(&mut st.copy_by.0).speed(1.0));
                    ui.end_row();
                    ui.label(tr("Vertical (mm, downwards)"));
                    ui.add(egui::DragValue::new(&mut st.copy_by.1).speed(1.0));
                    ui.end_row();
                });
                ui.horizontal(|ui| {
                    apply = ui.add_enabled(moves(st.copy_by), egui::Button::new(tr("Copy"))).clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
                });
            });
            if apply && !self.editor.selection.is_empty() && self.page_editable() {
                let (count, (dx, dy)) = (self.arrange_ui.count, self.arrange_ui.copy_by);
                let items = self.editor.items(&self.session.prj, page);
                let ids = arrange::multiple_copy(&mut self.session.prj, page, &items, count, p(dx, dy));
                self.notify(ctx, if ids.len() == 1 { tr("1 item copied").to_string() } else { tr_fmt!("{} items copied", ids.len()) });
                self.editor.selection = ids;
            }
            self.arrange_ui.show_copy = open && !apply && !cancel;
        }
    }
}

#[cfg(test)]
mod tests;
