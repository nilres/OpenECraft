//! Integration window: exchange with ERP, PDM, PLM, ECAD, MCAD, procurement and
//! wiring machines through the connectors of the integration layer. Imports are shown
//! as a list of changes (conflicts unchecked) and the checked ones are taken over.

use openecraft_ui::icons::{self, Icon};
use super::App;
use openecraft_core::integration::{self, Direction, Field, Preview, Profile, CONNECTORS};
use crate::io::Purpose;
use egui::{Color32, RichText};
use openecraft_i18n::{tr, tr_fmt};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Tab {
    #[default]
    Exchange,
    Profiles,
    Linked,
    Log,
}

/// A file of another system that was read.
struct Import {
    connector: usize,
    name: String,
    text: String,
    mapping: BTreeMap<String, String>,
    preview: Result<Preview, String>,
    /// History revision the preview was computed for.
    rev: u64,
    checked: BTreeSet<usize>,
}

#[derive(Default)]
pub struct IntegrationUi {
    pub open: bool,
    tab: Tab,
    connector: usize,
    /// Profile whose column mapping imports use, empty for none.
    profile: String,
    import: Option<Import>,
    new_profile: String,
}

enum Action {
    Export,
    ExportExchange,
    Import,
    Apply,
    Discard,
    AddProfile,
    RemoveProfile(usize),
}

impl App {
    pub(super) fn integration_pane(&mut self, ui: &mut egui::Ui) {
        let ctx = &ui.ctx().clone();
        self.refresh_integration_preview();
        let mut actions = Vec::new();
        ui.scope(|ui| {
            let ui_state = &mut self.integration_ui;
            ui.horizontal_wrapped(|ui| {
                ui.label(tr("Connector"));
                let c = &CONNECTORS[ui_state.connector.min(CONNECTORS.len() - 1)];
                egui::ComboBox::from_id_salt("integration_connector").width(320.0).selected_text(format!("{} – {}", c.system.title(), tr(c.title))).show_ui(ui, |ui| {
                    for (i, c) in CONNECTORS.iter().enumerate() {
                        ui.selectable_value(&mut ui_state.connector, i, format!("{} – {}", c.system.title(), tr(c.title)));
                    }
                });
                ui.label(tr("Profile"));
                let profiles: Vec<String> = self.session.prj.integration.profiles.iter().filter(|p| p.connector == c.id).map(|p| p.name.clone()).collect();
                if !profiles.contains(&ui_state.profile) {
                    ui_state.profile.clear();
                }
                egui::ComboBox::from_id_salt("integration_profile").width(140.0).selected_text(if ui_state.profile.is_empty() { tr("(standard columns)") } else { ui_state.profile.as_str() }).show_ui(ui, |ui| {
                    ui.selectable_value(&mut ui_state.profile, String::new(), tr("(standard columns)"));
                    for p in &profiles {
                        ui.selectable_value(&mut ui_state.profile, p.clone(), p);
                    }
                });
            });
            let c = &CONNECTORS[ui_state.connector.min(CONNECTORS.len() - 1)];
            ui.label(RichText::new(tr(c.help)).weak());
            ui.horizontal_wrapped(|ui| {
                if icons::text_button(ui, Icon::Download, tr("Export…")).on_hover_text(tr("Write the file for the other system; later imports detect conflicts against this export")).clicked() {
                    actions.push(Action::Export);
                }
                if icons::text_button(ui, Icon::Upload, tr("Import…")).on_hover_text(tr("Read a file of the other system (CSV or JSON) and review the changes")).clicked() {
                    actions.push(Action::Import);
                }
                ui.separator();
                if icons::text_button(ui, Icon::Download, tr("Neutral exchange file…")).on_hover_text(tr("Items, devices, placements, connections, wires and documents in one JSON file for systems without a connector")).clicked() {
                    actions.push(Action::ExportExchange);
                }
            });
            ui.horizontal(|ui| {
                ui.selectable_value(&mut ui_state.tab, Tab::Exchange, tr("Import changes"));
                ui.selectable_value(&mut ui_state.tab, Tab::Profiles, tr("Profiles"));
                ui.selectable_value(&mut ui_state.tab, Tab::Linked, tr("Linked data"));
                ui.selectable_value(&mut ui_state.tab, Tab::Log, tr("Exchange log"));
            });
            ui.separator();
            match self.integration_ui.tab {
                Tab::Exchange => self.integration_changes(ui, &mut actions),
                Tab::Profiles => self.integration_profiles(ui, &mut actions),
                Tab::Linked => self.integration_linked(ui),
                Tab::Log => self.integration_log(ui),
            }
        });
        for a in actions {
            self.integration_action(ctx, a);
        }
    }

    fn refresh_integration_preview(&mut self) {
        let rev = self.session.history.rev;
        let Some(imp) = &mut self.integration_ui.import else { return };
        if imp.rev == rev {
            return;
        }
        imp.preview = integration::import(&self.session.prj, &self.session.analysis, &CONNECTORS[imp.connector], &imp.text, &imp.mapping);
        imp.rev = rev;
        imp.checked = match &imp.preview {
            Ok(p) => (0..p.changes.len()).filter(|&i| !p.changes[i].conflict).collect(),
            Err(_) => BTreeSet::new(),
        };
    }

    fn integration_changes(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let Some(imp) = &mut self.integration_ui.import else {
            ui.label(tr("Import a file of another system to see what it changes in the project."));
            return;
        };
        let c = &CONNECTORS[imp.connector];
        ui.label(RichText::new(format!("{} – {}", imp.name, tr(c.title))).strong());
        let p = match &imp.preview {
            Ok(p) => p,
            Err(e) => {
                ui.colored_label(ui.visuals().error_fg_color, tr_fmt!("Import failed: {}", e));
                if ui.button(tr("Discard")).clicked() {
                    actions.push(Action::Discard);
                }
                return;
            }
        };
        ui.label(tr_fmt!("{} records, {} changes, {} conflicts, {} not found in the project", p.records, p.changes.len(), p.conflicts(), p.unmatched.len()));
        if !p.unmatched.is_empty() {
            ui.label(RichText::new(tr_fmt!("Not found: {}", p.unmatched.join(", "))).weak());
        }
        ui.horizontal(|ui| {
            if ui.add_enabled(!imp.checked.is_empty(), egui::Button::new(tr_fmt!("Take over {} changes", imp.checked.len()))).clicked() {
                actions.push(Action::Apply);
            }
            if ui.button(tr("Check all")).clicked() {
                imp.checked = (0..p.changes.len()).collect();
            }
            if ui.button(tr("Check none")).clicked() {
                imp.checked.clear();
            }
            if ui.button(tr("Discard")).clicked() {
                actions.push(Action::Discard);
            }
        });
        if p.conflicts() > 0 {
            ui.colored_label(ui.visuals().warn_fg_color, tr("⚠ Conflicts: changed in the project and in the other system since the last export; checked conflicts are overwritten"));
        }
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            egui::Grid::new("integration_changes").striped(true).num_columns(6).show(ui, |ui| {
                ui.label("");
                for h in [tr("Object"), tr("Field"), tr("In the project"), tr("Imported"), ""] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for (i, ch) in p.changes.iter().enumerate() {
                    let mut on = imp.checked.contains(&i);
                    if ui.checkbox(&mut on, "").changed() {
                        if on {
                            imp.checked.insert(i);
                        } else {
                            imp.checked.remove(&i);
                        }
                    }
                    let key = if ch.key.is_empty() { self.session.prj.info.name.clone() } else { ch.key.replace('|', " – ") };
                    ui.label(format!("{} {}", ch.target.title(), key));
                    ui.label(ch.field.title());
                    ui.label(&ch.old);
                    ui.label(RichText::new(&ch.new).color(Color32::from_rgb(70, 130, 220)));
                    if ch.conflict {
                        ui.colored_label(ui.visuals().warn_fg_color, tr("⚠ conflict"));
                    } else {
                        ui.label("");
                    }
                    ui.end_row();
                }
            });
        });
    }

    fn integration_profiles(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.label(RichText::new(tr("A profile maps the neutral fields to the column names of a particular system (e.g. MATNR for the ERP number in SAP). Without a mapping, common names in English and German are recognized.")).weak());
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.integration_ui.new_profile).hint_text(tr("Profile name")).desired_width(160.0));
            let name = self.integration_ui.new_profile.trim();
            let ok = !name.is_empty() && self.session.prj.integration.profile(name).is_none();
            if ui.add_enabled(ok, egui::Button::new(tr("Add profile for this connector"))).clicked() {
                actions.push(Action::AddProfile);
            }
        });
        ui.separator();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let mut changed: Option<(usize, Profile)> = None;
            for (pi, p) in self.session.prj.integration.profiles.iter().enumerate() {
                let title = integration::connector(&p.connector).map(|c| tr(c.title)).unwrap_or_default();
                egui::CollapsingHeader::new(RichText::new(format!("{} – {}", p.name, title)).strong()).id_salt(("integration_profile", pi)).show(ui, |ui| {
                    let accepts: Vec<Field> = integration::connector(&p.connector).map(|c| c.accepts.to_vec()).unwrap_or_default();
                    let mut edited = p.clone();
                    egui::Grid::new(("integration_mapping", pi)).num_columns(2).show(ui, |ui| {
                        for f in Field::ALL {
                            let mut col = edited.mapping.get(f.code()).cloned().unwrap_or_default();
                            let label = if accepts.contains(&f) { RichText::new(f.title()).strong() } else { RichText::new(f.title()) };
                            ui.label(label).on_hover_text(f.code());
                            if ui.add(egui::TextEdit::singleline(&mut col).hint_text(f.code()).desired_width(180.0)).changed() {
                                if col.trim().is_empty() {
                                    edited.mapping.remove(f.code());
                                } else {
                                    edited.mapping.insert(f.code().to_string(), col);
                                }
                            }
                            ui.end_row();
                        }
                    });
                    if edited != *p {
                        changed = Some((pi, edited));
                    }
                    if ui.button(tr("Remove profile")).clicked() {
                        actions.push(Action::RemoveProfile(pi));
                    }
                });
            }
            if let Some((i, p)) = changed {
                self.session.prj.integration.profiles[i] = p;
            }
        });
    }

    fn integration_linked(&mut self, ui: &mut egui::Ui) {
        let i = &self.session.prj.integration;
        if i.items.is_empty() && i.documents.is_empty() && i.project.is_default() {
            ui.label(tr("No data of other systems imported yet."));
            return;
        }
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            if !i.items.is_empty() {
                ui.label(RichText::new(tr("Parts")).strong());
                egui::Grid::new("integration_items").striped(true).num_columns(10).show(ui, |ui| {
                    for f in [Field::PartNumber, Field::ErpNumber, Field::ItemId, Field::Revision, Field::Lifecycle, Field::Stock, Field::LeadTime, Field::OrderStatus, Field::Ordered, Field::Delivery] {
                        ui.label(RichText::new(f.title()).strong());
                    }
                    ui.end_row();
                    let num = |v: Option<f32>| v.map(integration::fmt_number).unwrap_or_default();
                    for (part, l) in &i.items {
                        ui.label(part);
                        ui.label(&l.erp_number);
                        ui.label(&l.item_id);
                        ui.label(&l.revision);
                        if integration::blocked_state(&l.lifecycle) {
                            ui.colored_label(ui.visuals().warn_fg_color, format!("⚠ {}", l.lifecycle));
                        } else {
                            ui.label(&l.lifecycle);
                        }
                        ui.label(num(l.stock));
                        ui.label(l.lead_time.map(|d| d.to_string()).unwrap_or_default());
                        ui.label(&l.order_status);
                        ui.label(num(l.ordered));
                        ui.label(&l.delivery);
                        ui.end_row();
                    }
                });
                ui.add_space(8.0);
            }
            if !i.documents.is_empty() || !i.project.is_default() {
                ui.label(RichText::new(tr("Documents")).strong());
                egui::Grid::new("integration_docs").striped(true).num_columns(4).show(ui, |ui| {
                    for f in [Field::Document, Field::DocumentNumber, Field::Revision, Field::State] {
                        ui.label(RichText::new(f.title()).strong());
                    }
                    ui.end_row();
                    let project = (self.session.prj.info.name.clone(), &i.project);
                    for (name, d) in std::iter::once(project).chain(i.documents.iter().map(|(k, v)| (k.clone(), v))) {
                        if d.is_default() {
                            continue;
                        }
                        ui.label(name);
                        ui.label(&d.number);
                        ui.label(&d.revision);
                        ui.label(&d.state);
                        ui.end_row();
                    }
                });
            }
        });
    }

    fn integration_log(&mut self, ui: &mut egui::Ui) {
        let log = &self.session.prj.integration.log;
        if log.is_empty() {
            ui.label(tr("Nothing exchanged yet."));
            return;
        }
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            egui::Grid::new("integration_log").striped(true).num_columns(5).show(ui, |ui| {
                for h in [tr("Date"), tr("Connector"), tr("Direction"), tr("Count"), tr("Conflicts")] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for e in log.iter().rev() {
                    ui.label(&e.date);
                    ui.label(integration::connector(&e.connector).map(|c| c.system.title()).unwrap_or(e.connector.as_str()));
                    ui.label(match e.direction {
                        Direction::Export => tr("Export"),
                        Direction::Import => tr("Import"),
                    });
                    ui.label(e.count.to_string());
                    ui.label(if e.conflicts > 0 { e.conflicts.to_string() } else { String::new() });
                    ui.end_row();
                }
            });
        });
    }

    fn integration_action(&mut self, ctx: &egui::Context, a: Action) {
        let c = &CONNECTORS[self.integration_ui.connector.min(CONNECTORS.len() - 1)];
        match a {
            Action::Export => {
                let today = self.today();
                let text = integration::export(&mut self.session.prj, &self.session.analysis, c, &today);
                let name = format!("{}_{}.{}", self.session.prj.info.name, c.id, c.extension);
                let filter = match c.extension {
                    "csv" => "CSV",
                    "json" => "JSON",
                    _ => c.system.title(),
                };
                self.export(ctx, name, (filter, c.extension), text);
            }
            Action::ExportExchange => {
                let text = integration::exchange(&self.session.prj, &self.session.analysis);
                self.export(ctx, format!("{}_exchange.json", self.session.prj.info.name), ("JSON", "json"), text);
            }
            Action::Import => self.io.open(ctx, Purpose::Integration),
            Action::Apply => {
                let Some(imp) = self.integration_ui.import.take() else { return };
                let Ok(p) = &imp.preview else { return };
                let picked: Vec<_> = imp.checked.iter().filter_map(|&i| p.changes.get(i).cloned()).collect();
                // Checked conflicts were chosen to overwrite the project's values.
                let today = self.today();
                let n = integration::apply(&mut self.session.prj, &self.session.analysis, &CONNECTORS[imp.connector], &picked, true, &today);
                self.notify(ctx, tr_fmt!("{} values taken over", n));
            }
            Action::Discard => self.integration_ui.import = None,
            Action::AddProfile => {
                let name = self.integration_ui.new_profile.trim().to_string();
                self.session.prj.integration.profiles.push(Profile { name: name.clone(), connector: c.id.to_string(), mapping: BTreeMap::new() });
                self.integration_ui.profile = name;
                self.integration_ui.new_profile.clear();
            }
            Action::RemoveProfile(i) => {
                if i < self.session.prj.integration.profiles.len() {
                    self.session.prj.integration.profiles.remove(i);
                }
            }
        }
    }

    /// Loaded file of another system: shown as import preview.
    /// Reviews a file of another system with the connector `id` (e.g. Vault items for plm).
    pub(super) fn integration_review(&mut self, id: &str, name: &str, text: String) {
        if let Some(i) = CONNECTORS.iter().position(|c| c.id == id) {
            if i != self.integration_ui.connector {
                self.integration_ui.connector = i;
                self.integration_ui.profile.clear();
            }
        }
        self.integration_text_loaded(name, text);
    }

    /// A file of another system was opened: files in a format of that system (a KiCad
    /// netlist, a FreeCAD document) are read by the connector first.
    pub(super) fn integration_file_loaded(&mut self, ctx: &egui::Context, name: &str, bytes: &[u8]) {
        let connector = self.integration_ui.connector.min(CONNECTORS.len() - 1);
        match integration::decode(&CONNECTORS[connector], bytes) {
            Ok(text) => self.integration_text_loaded(name, text),
            Err(e) => self.notify_error(ctx, tr_fmt!("Import failed: {}", e)),
        }
    }

    fn integration_text_loaded(&mut self, name: &str, text: String) {
        let ui_state = &mut self.integration_ui;
        let connector = ui_state.connector.min(CONNECTORS.len() - 1);
        let mapping = self.session.prj.integration.profile(&ui_state.profile).map(|p| p.mapping.clone()).unwrap_or_default();
        ui_state.import = Some(Import { connector, name: crate::io::short_name(name).to_string(), text, mapping, preview: Ok(Preview::default()), rev: u64::MAX, checked: BTreeSet::new() });
        ui_state.tab = Tab::Exchange;
        ui_state.open = true;
    }
}
