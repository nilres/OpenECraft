//! Report blocks window: report pages generated together (report, scheme, one page
//! set per strip, plug, cable …), generating them again, and the output of report
//! pages into another project.

use openecraft_ui::icons::{self, Icon};
use openecraft_i18n::{tr, tr_fmt};
use super::App;
use openecraft_core::forms;
use crate::io::{self, Purpose};
use openecraft_core::model::ReportId;
use openecraft_core::report_blocks::{self, ReportBlock};
use egui::RichText;

#[derive(Default)]
pub struct ReportBlocksUi {
    pub open: bool,
    selected: Option<usize>,
    /// Name being edited (applied when it is unique).
    name: String,
    /// Output only the pages of the selected block, otherwise all report pages.
    output_block: bool,
    /// Pages to output, until the target project is loaded.
    pending_output: Option<Vec<usize>>,
}

enum Action {
    Generate(usize),
    GenerateAll,
    Delete(usize),
    Output,
}

impl App {
    pub(super) fn report_blocks_pane(&mut self, ui: &mut egui::Ui) {
        let ctx = &ui.ctx().clone();
        let mut action = None;
        let reports = self.report_list("");
        let title_of = |id: ReportId| reports.iter().find(|(k, _)| *k == id).map(|(_, t)| t.clone()).unwrap_or_else(|| id.title());
        ui.scope(|ui| {
            let ru = &mut self.report_blocks_ui;
            let prj = &mut self.session.prj;
            ui.horizontal(|ui| {
                if icons::text_button(ui, Icon::Plus, tr("New block")).clicked() {
                    let name = (1..).map(|i| tr_fmt!("Block {}", i)).find(|n| !prj.report_blocks.iter().any(|b| &b.name == n)).unwrap_or_default();
                    prj.report_blocks.push(ReportBlock { name, ..Default::default() });
                    ru.selected = Some(prj.report_blocks.len() - 1);
                    ru.name.clear();
                }
                let any = !prj.report_blocks.is_empty();
                if ui.add_enabled_ui(any, |ui| icons::text_button(ui, Icon::Rotate, tr("Generate all"))).inner.on_hover_text(tr("Generates the pages of all report blocks again")).clicked() {
                    action = Some(Action::GenerateAll);
                }
            });
            ui.separator();
            if ru.selected.is_some_and(|i| i >= prj.report_blocks.len()) {
                ru.selected = None;
            }
            ui.columns(2, |cols| {
                for (i, b) in prj.report_blocks.iter().enumerate() {
                    let pages = report_blocks::report_pages(prj, Some(&b.name)).len();
                    if cols[0].selectable_label(ru.selected == Some(i), tr_fmt!("{} – {} ({} pages)", b.name, b.kind.title(), pages)).clicked() {
                        ru.selected = Some(i);
                        ru.name.clear();
                    }
                }
                if prj.report_blocks.is_empty() {
                    cols[0].weak(tr("A report block generates the pages of a report, e.g. one terminal diagram per strip, and updates them."));
                }
                let Some(i) = ru.selected else {
                    cols[1].weak(tr("Select a report block."));
                    return;
                };
                let schemes = forms::scheme_names(prj, prj.report_blocks[i].kind);
                if ru.name.is_empty() {
                    ru.name = prj.report_blocks[i].name.clone();
                }
                let ui = &mut cols[1];
                let mut rename = false;
                egui::Grid::new("rblock").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                    let b = &mut prj.report_blocks[i];
                    ui.label(tr("Name"));
                    rename = ui.add(egui::TextEdit::singleline(&mut ru.name).desired_width(240.0)).lost_focus();
                    ui.end_row();
                    ui.label(tr("Report"));
                    let kind = b.kind;
                    egui::ComboBox::from_id_salt("rbkind").selected_text(title_of(kind)).width(240.0).show_ui(ui, |ui| {
                        for (k, title) in &reports {
                            ui.selectable_value(&mut b.kind, *k, title);
                        }
                    });
                    if b.kind != kind {
                        b.scheme.clear();
                    }
                    ui.end_row();
                    ui.label(tr("Scheme"));
                    let standard = tr("Standard form of the report");
                    let current = if b.scheme.is_empty() { standard.to_string() } else { b.scheme.clone() };
                    egui::ComboBox::from_id_salt("rbscheme").selected_text(current).width(240.0).show_ui(ui, |ui| {
                        ui.selectable_value(&mut b.scheme, String::new(), standard);
                        for s in &schemes {
                            ui.selectable_value(&mut b.scheme, s.clone(), s);
                        }
                    });
                    ui.end_row();
                    ui.label("");
                    ui.checkbox(&mut b.per_group, tr("New pages for every group (strip, plug, cable, device …)"));
                    ui.end_row();
                    ui.label(tr("Groups"));
                    ui.add_enabled(b.per_group, egui::TextEdit::singleline(&mut b.groups).hint_text(tr("all, or e.g. -X*, -W1")).desired_width(240.0));
                    ui.end_row();
                    ui.label(tr("Function"));
                    ui.add(egui::TextEdit::singleline(&mut b.function).desired_width(120.0).prefix("="));
                    ui.end_row();
                    ui.label(tr("Location"));
                    ui.add(egui::TextEdit::singleline(&mut b.location).desired_width(120.0).prefix("+"));
                    ui.end_row();
                });
                if rename && ru.name != prj.report_blocks[i].name {
                    let old = prj.report_blocks[i].name.clone();
                    if !report_blocks::rename_block(prj, &old, &ru.name) {
                        ru.name = old;
                    }
                }
                let n = report_blocks::block_pages(prj, &self.session.analysis, &prj.report_blocks[i]).len();
                ui.weak(tr_fmt!("Generates {} pages", n));
                ui.horizontal(|ui| {
                    if icons::text_button(ui, Icon::Rotate, tr("Generate")).on_hover_text(tr("Replaces the pages of this block")).clicked() {
                        action = Some(Action::Generate(i));
                    }
                    if icons::text_button(ui, Icon::Delete, tr("Delete")).on_hover_text(tr("The pages it generated stay")).clicked() {
                        action = Some(Action::Delete(i));
                    }
                });
            });
            ui.separator();
            ui.label(RichText::new(tr("Output to another project")).strong());
            ui.horizontal(|ui| {
                ui.radio_value(&mut ru.output_block, false, tr("All report pages"));
                ui.add_enabled_ui(ru.selected.is_some(), |ui| ui.radio_value(&mut ru.output_block, true, tr("Pages of the selected block")));
                if ui
                    .button(tr("Output to project…"))
                    .on_hover_text(tr("Copies the report pages as fixed pages into another project file; pages output from this project before are replaced"))
                    .clicked()
                {
                    action = Some(Action::Output);
                }
            });
        });
        match action {
            Some(Action::Generate(i)) => {
                let name = self.session.prj.report_blocks[i].name.clone();
                match report_blocks::generate(&mut self.session.prj, &self.session.analysis, i) {
                    Ok(n) => self.notify(ctx, tr_fmt!("Report block {}: {} pages generated", name, n)),
                    Err(e) => self.notify_error(ctx, e),
                }
            }
            Some(Action::GenerateAll) => self.generate_report_blocks(ctx),
            Some(Action::Delete(i)) => {
                self.session.prj.report_blocks.remove(i);
                self.report_blocks_ui.selected = None;
            }
            Some(Action::Output) => {
                let ru = &self.report_blocks_ui;
                let block = ru.selected.filter(|_| ru.output_block).map(|i| self.session.prj.report_blocks[i].name.clone());
                let pages = report_blocks::report_pages(&self.session.prj, block.as_deref());
                if pages.is_empty() {
                    self.notify(ctx, tr("No report pages to output").to_string());
                } else {
                    self.report_blocks_ui.pending_output = Some(pages);
                    self.io.open(ctx, Purpose::OutputTarget);
                }
            }
            None => {}
        }
        self.session.page = self.session.page.min(self.session.prj.pages.len() - 1);
    }

    pub(super) fn generate_report_blocks(&mut self, ctx: &egui::Context) {
        match report_blocks::generate_all(&mut self.session.prj) {
            Ok(n) => self.notify(ctx, tr_fmt!("Report blocks updated: {} pages", n)),
            Err(e) => self.notify_error(ctx, e),
        }
        self.session.page = self.session.page.min(self.session.prj.pages.len() - 1);
    }

    /// Writes the chosen report pages into the loaded project file.
    pub(super) fn output_reports(&mut self, ctx: &egui::Context, loaded: &io::Loaded) {
        let pages = self.report_blocks_ui.pending_output.take().unwrap_or_else(|| report_blocks::report_pages(&self.session.prj, None));
        let result = report_blocks::output_to_file(&self.session.prj, &self.session.analysis, &pages, &loaded.text())
            .and_then(|(json, n)| self.io.save_to(&loaded.name, &json).map(|_| n));
        match result {
            Ok(n) => self.notify(ctx, tr_fmt!("{} report pages output to {}", n, io::short_name(&loaded.name))),
            Err(e) => self.notify_error(ctx, tr_fmt!("Output failed: {}", e)),
        }
    }
}
