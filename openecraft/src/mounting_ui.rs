//! Mounting assistant for the workshop (tablet friendly): the items of the panel
//! layout as a checklist in mounting order, the current item with its place, size and
//! part, highlighted in 3D, and the mounting progress. Shares the Wiring workspace.

use openecraft_i18n::{tr, tr_fmt};
use openecraft_ui::icons::{self, Icon};
use openecraft_core::model::Project;
use openecraft_core::model::panel::PanelItem;
use openecraft_core::analysis::Analysis;
use openecraft_core::mounting::{self, Mark, Phase, Status, Step};
use crate::panel_ui::PanelUi;
use egui::{vec2, Color32, RichText};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Filter {
    #[default]
    Todo,
    Mounted,
    All,
}

impl Filter {
    const ALL: [Filter; 3] = [Filter::Todo, Filter::Mounted, Filter::All];

    fn title(self) -> &'static str {
        match self {
            Filter::Todo => tr("To do"),
            Filter::Mounted => tr("Mounted"),
            Filter::All => tr("All"),
        }
    }

    fn shows(self, s: Status) -> bool {
        match self {
            Filter::Todo => s != Status::Done,
            Filter::Mounted => matches!(s, Status::Done | Status::Changed),
            Filter::All => true,
        }
    }
}

#[derive(Default)]
pub struct MountingUi {
    /// The Wiring workspace shows the mounting assistant instead of the wires.
    pub active: bool,
    pub filter: Filter,
    /// Enclosure shown; None = all.
    pub enclosure: Option<usize>,
    /// Current item (enclosure, item id).
    pub current: Option<(usize, u64)>,
    /// The details of the current item are hidden (the app keeps it).
    pub details_hidden: bool,
    note: String,
    /// Export of the checklist as CSV requested.
    pub export: bool,
}

fn touch_spacing(ui: &mut egui::Ui) {
    let s = ui.spacing_mut();
    s.interact_size.y = 40.0;
    s.button_padding = vec2(12.0, 8.0);
    s.item_spacing = vec2(8.0, 8.0);
}

fn icon(s: Status) -> (&'static str, Color32) {
    match s {
        Status::Open => ("☐", Color32::GRAY),
        Status::Done => ("✔", Color32::from_rgb(50, 170, 80)),
        Status::Changed => ("⟳", crate::style::warning()),
        Status::Problem => ("⚠", crate::style::error()),
    }
}

fn item_of<'a>(prj: &'a Project, s: &Step) -> &'a PanelItem {
    &prj.layout.enclosures[s.enclosure].items[s.item]
}

impl MountingUi {
    /// Switch between wiring and mounting at the top of the checklist.
    pub fn mode_switch(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active, false, RichText::new(tr("Wires")).strong());
            ui.selectable_value(&mut self.active, true, RichText::new(tr("Mounting")).strong())
                .on_hover_text(tr("Mounting assistant: the panel items in mounting order with their place and part"));
        });
    }

    fn steps(&self, prj: &Project) -> Vec<Step> {
        mounting::steps(prj, self.enclosure)
    }

    fn current_step(&self, steps: &[Step]) -> Option<usize> {
        let (e, id) = self.current?;
        steps.iter().position(|s| s.enclosure == e && s.id == id)
    }

    fn select(&mut self, prj: &Project, panel: &mut PanelUi, s: &Step) {
        self.current = Some((s.enclosure, s.id));
        self.note = s.note.clone();
        panel.focus_item(prj, s.enclosure, s.id);
    }

    fn advance(&mut self, prj: &Project, panel: &mut PanelUi, at: usize) {
        let steps = self.steps(prj);
        match mounting::next_open(&steps, Some(at)) {
            Some(i) => self.select(prj, panel, &steps[i]),
            None => self.current = None,
        }
    }

    /// Keeps the current item valid and in sync with picking in the 3D view.
    pub fn sync(&mut self, prj: &Project, panel: &mut PanelUi) {
        self.enclosure = self.enclosure.filter(|e| *e < prj.layout.enclosures.len());
        panel.selected_wire = None;
        let steps = self.steps(prj);
        if let (1, Some(&id)) = (panel.selection.len(), panel.selection.iter().next()) {
            if self.current.map(|c| c.1) != Some(id) {
                if let Some(s) = steps.iter().find(|s| s.id == id) {
                    self.current = Some((s.enclosure, s.id));
                    self.note = s.note.clone();
                }
            }
        }
        if self.current.is_some() && self.current_step(&steps).is_none() {
            self.current = None;
        }
        if self.current.is_none() {
            if let Some(i) = mounting::next_open(&steps, None) {
                self.select(prj, panel, &steps[i]);
            }
        }
        panel.selection = self.current.map(|c| c.1).into_iter().collect();
        panel.done_items = steps.iter().filter(|s| matches!(s.status, Status::Done | Status::Changed)).map(|s| s.id).collect();
    }

    /// Progress, filters and the checklist grouped by phase (left side).
    pub fn checklist(&mut self, ui: &mut egui::Ui, prj: &mut Project, panel: &mut PanelUi) {
        touch_spacing(ui);
        let prog = mounting::progress(prj, self.enclosure);
        ui.heading(tr("Mounting"));
        ui.add(
            egui::ProgressBar::new(prog.fraction())
                .desired_height(28.0)
                .text(RichText::new(tr_fmt!("{} of {} items", prog.done - prog.changed, prog.total)).strong()),
        );
        if prog.changed > 0 || prog.problems > 0 {
            ui.colored_label(crate::style::warning(), tr_fmt!("{} changed since mounted · {} problems", prog.changed, prog.problems));
        }
        ui.separator();
        let all = tr("All enclosures");
        let name = |e: Option<usize>| e.and_then(|e| prj.layout.enclosures.get(e)).map(|e| e.name.clone()).unwrap_or_else(|| all.to_string());
        egui::ComboBox::from_id_salt("mounting_enc").width(ui.available_width()).selected_text(name(self.enclosure)).show_ui(ui, |ui| {
            ui.selectable_value(&mut self.enclosure, None, all);
            for (i, e) in prj.layout.enclosures.iter().enumerate() {
                ui.selectable_value(&mut self.enclosure, Some(i), &e.name);
            }
        });
        ui.horizontal(|ui| {
            for f in Filter::ALL {
                ui.selectable_value(&mut self.filter, f, f.title());
            }
        });
        ui.separator();
        let steps = self.steps(prj);
        let current = self.current_step(&steps);
        let mut clicked: Option<usize> = None;
        let mut toggled: Option<usize> = None;
        egui::ScrollArea::vertical().id_salt("mounting_steps").auto_shrink([false, false]).show(ui, |ui| {
            let mut phase: Option<(usize, Phase)> = None;
            let mut any = false;
            for (i, s) in steps.iter().enumerate().filter(|(_, s)| self.filter.shows(s.status)) {
                any = true;
                if phase != Some((s.enclosure, s.phase)) {
                    phase = Some((s.enclosure, s.phase));
                    let e = &prj.layout.enclosures[s.enclosure].name;
                    let head = if self.enclosure.is_none() { format!("{e} · {}", s.phase.title()) } else { s.phase.title().to_string() };
                    ui.label(RichText::new(head).strong());
                }
                let it = item_of(prj, s);
                let (sym, color) = icon(s.status);
                ui.horizontal(|ui| {
                    let tip = if s.status == Status::Open { tr("Mark as mounted") } else { tr("Mark as open") };
                    if ui.add_sized([44.0, 44.0], egui::Button::new(RichText::new(sym).size(22.0).color(color))).on_hover_text(tip).clicked() {
                        toggled = Some(i);
                    }
                    let mut text = egui::text::LayoutJob::default();
                    let strong = egui::TextFormat { font_id: egui::FontId::proportional(16.0), color: ui.visuals().strong_text_color(), ..Default::default() };
                    let weak = egui::TextFormat { font_id: egui::FontId::proportional(13.0), color: ui.visuals().weak_text_color(), ..Default::default() };
                    text.append(&format!("{}\n", mounting::item_title(it)), 0.0, strong);
                    let part = if it.part.is_empty() { String::new() } else { format!("{} · ", it.part) };
                    text.append(&format!("{part}X {:.0} · Y {:.0} · {:.0} × {:.0} mm", it.x, it.y, it.w, it.h), 0.0, weak);
                    if ui.selectable_label(Some(i) == current, text).clicked() {
                        clicked = Some(i);
                    }
                });
            }
            if !any {
                ui.weak(if steps.is_empty() { tr("No panel items: place the devices in the panel layout.") } else { tr("Nothing left in this list.") });
            }
        });
        if let Some(i) = toggled {
            let it = item_of(prj, &steps[i]).clone();
            let mark = if steps[i].status == Status::Open { Some(Mark::Done) } else { None };
            mounting::set_mark(prj, &it, mark, "");
        }
        if let Some(i) = clicked {
            self.select(prj, panel, &steps[i]);
        }
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if ui.button(tr("Export CSV")).on_hover_text(tr("Checklist with the mounting state")).clicked() {
                self.export = true;
            }
            let stale = mounting::stale(prj).len();
            if stale > 0 && ui.button(tr_fmt!("Remove {} old marks", stale)).on_hover_text(tr("Marks of items that no longer exist")).clicked() {
                mounting::remove_stale(prj);
            }
        });
    }

    /// The current item in large print with its actions, above the 3D view.
    pub fn workbench(&mut self, ui: &mut egui::Ui, prj: &mut Project, an: &Analysis, panel: &mut PanelUi) {
        touch_spacing(ui);
        let steps = self.steps(prj);
        let Some(at) = self.current_step(&steps) else {
            ui.add_space(8.0);
            let done = !steps.is_empty() && mounting::progress(prj, self.enclosure).fraction() >= 1.0;
            ui.heading(if done { tr("✔ All items are mounted.") } else { tr("Select an item in the list or in the 3D view.") });
            ui.separator();
            self.view(ui, prj, an, panel);
            return;
        };
        let s = steps[at].clone();
        let it = item_of(prj, &s).clone();
        let enclosure = prj.layout.enclosures[s.enclosure].name.clone();
        egui::Frame::group(ui.style()).inner_margin(12.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(mounting::item_title(&it)).size(26.0).strong());
                let (sym, color) = icon(s.status);
                ui.label(RichText::new(format!("{sym} {}", s.status.title())).size(18.0).color(color));
                crate::wiring_ui::details_toggle(ui, &mut self.details_hidden);
            });
            if self.details_hidden {
                return;
            }
            ui.horizontal_wrapped(|ui| {
                let big = |t: String| RichText::new(t).size(18.0);
                ui.label(big(format!("{enclosure} · {}", s.phase.title())));
                ui.separator();
                if !it.part.is_empty() {
                    ui.label(big(tr_fmt!("Part {}", it.part)));
                    ui.separator();
                }
                ui.label(big(tr_fmt!("Position X {} mm, Y {} mm", format!("{:.0}", it.x), format!("{:.0}", it.y))));
                ui.separator();
                ui.label(big(tr_fmt!("Size {} × {} mm", format!("{:.0}", it.w), format!("{:.0}", it.h))));
            });
            if s.status == Status::Changed {
                if let Some(m) = mounting::mark_of(prj, it.id) {
                    let part = if m.part.is_empty() { "–" } else { &m.part };
                    ui.colored_label(
                        crate::style::warning(),
                        tr_fmt!("Changed since mounted: was at X {} mm, Y {} mm with part {}", format!("{:.0}", m.x), format!("{:.0}", m.y), part),
                    );
                }
            }
            ui.horizontal_wrapped(|ui| {
                let big = |t: &str| RichText::new(t).size(20.0);
                if ui.add(egui::Button::new(big(tr("✔ Mounted"))).fill(crate::style::ok()).min_size(vec2(160.0, 52.0))).clicked() {
                    mounting::set_mark(prj, &it, Some(Mark::Done), &self.note);
                    self.advance(prj, panel, at);
                }
                if ui.add(egui::Button::new(big(tr("⚠ Problem"))).min_size(vec2(130.0, 52.0))).on_hover_text(tr("Mark as not mountable; describe the problem in the note")).clicked() {
                    mounting::set_mark(prj, &it, Some(Mark::Problem), &self.note);
                    self.advance(prj, panel, at);
                }
                if s.status != Status::Open && ui.add(egui::Button::new(big(tr("Reset"))).min_size(vec2(100.0, 52.0))).on_hover_text(tr("Mark as open")).clicked() {
                    mounting::set_mark(prj, &it, None, "");
                }
                let n = steps.len();
                if n > 1 {
                    if ui.add(egui::Button::new(big("◀")).min_size(vec2(60.0, 52.0))).on_hover_text(tr("Previous item")).clicked() {
                        self.select(prj, panel, &steps[(at + n - 1) % n]);
                    }
                    if ui.add(egui::Button::new(big("▶")).min_size(vec2(60.0, 52.0))).on_hover_text(tr("Next item")).clicked() {
                        self.select(prj, panel, &steps[(at + 1) % n]);
                    }
                    ui.label(RichText::new(format!("{} / {n}", at + 1)).size(18.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label(tr("Note"));
                ui.add(egui::TextEdit::singleline(&mut self.note).desired_width(f32::INFINITY).hint_text(tr("e.g. cutout too small, part missing")));
            });
        });
        self.view(ui, prj, an, panel);
    }

    fn view(&mut self, ui: &mut egui::Ui, prj: &mut Project, an: &Analysis, panel: &mut PanelUi) {
        ui.horizontal(|ui| {
            panel.door_button(ui);
            if icons::text_button(ui, Icon::Fit, tr("Fit")).clicked() {
                panel.reset_camera();
                if let Some((e, id)) = self.current {
                    panel.focus_item(prj, e, id);
                }
            }
        });
        let show = panel.show_wires;
        panel.show_wires = false;
        panel.view_3d_current(ui, prj, an);
        panel.show_wires = show;
    }
}

#[cfg(test)]
mod tests;
