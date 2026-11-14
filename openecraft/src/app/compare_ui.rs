//! Compare window: differences of the project to a baseline, a revision or another
//! project file; selected differences are taken over. Baselines are created and
//! deleted here.

use openecraft_ui::icons::{self, Icon};
use super::App;
use openecraft_core::compare::{self, Difference, Item};
use crate::io::Purpose;
use openecraft_core::model::Project;
use openecraft_core::revision::ChangeKind;
use egui::{Color32, RichText};
use openecraft_i18n::{tr, tr_fmt};
use std::collections::BTreeSet;

#[derive(Default)]
pub struct CompareUi {
    pub open: bool,
    /// Name and project compared with.
    source: Option<(String, Project)>,
    diffs: Vec<Difference>,
    /// History revision the differences were computed for.
    diff_rev: u64,
    checked: BTreeSet<usize>,
    baseline_name: String,
    baseline_description: String,
}

impl CompareUi {
    /// Compares with a project file that finished loading.
    pub fn set_file(&mut self, name: &str, prj: Project) {
        self.set_source(tr_fmt!("File {}", name), prj);
        self.open = true;
    }

    /// Compares with a kept version of local changes of a shared project.
    pub fn set_kept(&mut self, name: String, prj: Project) {
        self.set_source(name, prj);
        self.open = true;
    }

    fn set_source(&mut self, name: String, prj: Project) {
        self.source = Some((name, prj));
        self.diff_rev = 0;
        self.checked.clear();
    }
}

fn kind_color(k: ChangeKind) -> Color32 {
    match k {
        ChangeKind::Added => crate::style::ok(),
        ChangeKind::Removed => crate::style::error(),
        ChangeKind::Modified => Color32::from_rgb(70, 130, 220),
    }
}

/// What taking a difference over does to the current project.
fn take_text(k: ChangeKind) -> &'static str {
    match k {
        ChangeKind::Added => tr("only here: taking over removes it"),
        ChangeKind::Removed => tr("only there: taking over adds it"),
        ChangeKind::Modified => tr("different: taking over replaces it"),
    }
}

impl App {
    #[allow(clippy::too_many_lines)]
    pub(super) fn compare_pane(&mut self, ui: &mut egui::Ui) {
        let ctx = &ui.ctx().clone();
        if let Some((_, other)) = &self.compare_ui.source {
            if self.compare_ui.diff_rev != self.session.history.rev {
                self.compare_ui.diffs = compare::diff(&self.session.prj, other);
                self.compare_ui.diff_rev = self.session.history.rev;
                let n = self.compare_ui.diffs.len();
                self.compare_ui.checked.retain(|i| *i < n);
            }
        }
        let mut source = None;
        let mut open_file = false;
        let mut add_baseline = false;
        let mut delete_baseline = None;
        let mut take = false;
        let mut jump = None;
        ui.scope(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(tr("Compare with")).strong());
                let shown = self.compare_ui.source.as_ref().map(|s| s.0.clone()).unwrap_or_else(|| tr("(choose)").to_string());
                egui::ComboBox::from_id_salt("compare_source").selected_text(shown).width(260.0).show_ui(ui, |ui| {
                    for (i, b) in self.session.prj.baselines.iter().enumerate() {
                        if ui.selectable_label(false, tr_fmt!("Baseline {} {}", b.name, b.date)).clicked() {
                            source = Some((tr_fmt!("Baseline {}", b.name), Some(i), None));
                        }
                    }
                    for (i, r) in self.session.prj.revisions.iter().enumerate() {
                        if ui.selectable_label(false, tr_fmt!("Pages of revision {}", r.name)).clicked() {
                            source = Some((tr_fmt!("Pages of revision {}", r.name), None, Some(i)));
                        }
                    }
                });
                if icons::text_button(ui, Icon::FolderOpen, tr("Project file…")).clicked() {
                    open_file = true;
                }
            });
            egui::CollapsingHeader::new(tr_fmt!("Baselines ({})", self.session.prj.baselines.len())).id_salt("baselines").show(ui, |ui| {
                ui.weak(tr("A baseline stores the whole project (pages, devices, parts, settings). Releasing a revision stores one."));
                egui::Grid::new("baseline_list").striped(true).num_columns(5).show(ui, |ui| {
                    for (i, b) in self.session.prj.baselines.iter().enumerate() {
                        ui.label(&b.name);
                        ui.label(&b.date);
                        ui.label(&b.author);
                        ui.label(&b.description);
                        if icons::button(ui, Icon::Delete, tr("Delete")).clicked() {
                            delete_baseline = Some(i);
                        }
                        ui.end_row();
                    }
                });
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.compare_ui.baseline_name).hint_text(tr("Name")).desired_width(120.0));
                    ui.add(egui::TextEdit::singleline(&mut self.compare_ui.baseline_description).hint_text(tr("Description")).desired_width(220.0));
                    add_baseline = ui.add_enabled(!self.compare_ui.baseline_name.trim().is_empty(), egui::Button::new(tr("Create baseline"))).clicked();
                });
            });
            ui.separator();
            let Some((name, _)) = &self.compare_ui.source else {
                ui.weak(tr("Choose a baseline, a revision or a project file to compare the project with."));
                return;
            };
            let diffs = &self.compare_ui.diffs;
            if diffs.is_empty() {
                ui.label(tr_fmt!("No differences to {}.", name));
                return;
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new(tr_fmt!("{} differences to {}", diffs.len(), name)).strong());
                if ui.small_button(tr("All")).clicked() {
                    self.compare_ui.checked = (0..diffs.len()).collect();
                }
                if ui.small_button(tr("None")).clicked() {
                    self.compare_ui.checked.clear();
                }
                let n = self.compare_ui.checked.len();
                take = ui
                    .add_enabled(n > 0, egui::Button::new(tr_fmt!("Take over selected ({})", n)))
                    .on_hover_text(tr("Make the selected elements of this project equal to the compared ones"))
                    .clicked();
            });
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for (i, d) in diffs.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let mut on = self.compare_ui.checked.contains(&i);
                        if ui.checkbox(&mut on, "").on_hover_text(take_text(d.kind)).changed() {
                            if on {
                                self.compare_ui.checked.insert(i);
                            } else {
                                self.compare_ui.checked.remove(&i);
                            }
                        }
                        ui.label(RichText::new(d.kind.title()).color(kind_color(d.kind)));
                        ui.label(RichText::new(d.item.kind_title()).weak());
                        if let Item::Page(id) = d.item {
                            match self.session.prj.pages.iter().position(|p| p.id == id) {
                                Some(pi) => {
                                    if ui.link(&d.name).clicked() {
                                        jump = Some(pi);
                                    }
                                }
                                None => {
                                    ui.label(&d.name);
                                }
                            }
                        } else {
                            ui.label(&d.name);
                        }
                    });
                    if !d.details.is_empty() {
                        egui::CollapsingHeader::new(tr_fmt!("{} details", d.details.len())).id_salt(("diff", i)).show(ui, |ui| {
                            for t in &d.details {
                                ui.label(t);
                            }
                        });
                    }
                }
            });
        });
        if let Some((name, baseline, rev)) = source {
            let prj = match (baseline, rev) {
                (Some(i), _) => self.session.prj.baselines[i].project.0.as_ref().clone(),
                (_, Some(i)) => compare::with_revision_pages(&self.session.prj, &self.session.prj.revisions[i].pages),
                _ => return,
            };
            self.compare_ui.set_source(name, prj);
        }
        if open_file {
            self.io.open(ctx, Purpose::Compare);
        }
        if add_baseline {
            let (user, date) = (self.user_name(), self.today());
            let name = self.compare_ui.baseline_name.trim().to_string();
            compare::add_baseline(&mut self.session.prj, &name, &date, &user, self.compare_ui.baseline_description.trim());
            self.compare_ui.baseline_name.clear();
            self.compare_ui.baseline_description.clear();
            self.notify(ctx, tr_fmt!("Baseline {} created", name));
        }
        if let Some(i) = delete_baseline {
            self.session.prj.baselines.remove(i);
        }
        if take {
            if let Some((_, other)) = &self.compare_ui.source {
                let items: Vec<Item> = self.compare_ui.checked.iter().filter_map(|i| self.compare_ui.diffs.get(*i)).map(|d| d.item.clone()).collect();
                let msg = compare::take_all(&mut self.session.prj, other, &items);
                self.compare_ui.checked.clear();
                self.session.page = self.session.page.min(self.session.prj.pages.len().saturating_sub(1));
                self.notify(ctx, msg);
            }
        }
        if let Some(pi) = jump {
            self.goto(pi, 0);
        }
    }
}
