//! Project templates in the editor: the templates of the backstage (the bundled ones
//! and those of the user: the templates folder the command line shares, in the
//! browser the storage of the editor), the values asked for by a parametric template,
//! saving the project as a template and the Template pane (name, parameters, formulas
//! and the conditions of the pages).

use super::{backstage::tile, App, Replace};
use egui::RichText;
use openecraft_core::model::Project;
use openecraft_core::project_templates::{self as templates, TemplateFormula, TemplateParameter};
use openecraft_i18n::{tr, tr_fmt};
use openecraft_ui::icons::{self, Icon};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Storage key of the templates of the user where there is no templates folder.
pub const STORAGE_KEY: &str = "project_templates";

#[derive(Default)]
pub struct TemplatesUi {
    pub open: bool,
    /// Templates of the user kept by the editor (browser).
    pub stored: Vec<Project>,
    /// The templates to choose from; read again after a change.
    library: Option<Vec<(Project, bool)>>,
    errors: Vec<String>,
    start: Option<Start>,
    save: Option<Save>,
}

/// A parametric template whose values are asked for.
struct Start {
    template: Project,
    values: BTreeMap<String, String>,
    name: String,
}

/// The project to save as a template.
struct Save {
    name: String,
    description: String,
}

impl TemplatesUi {
    /// With the templates of the user kept by the editor.
    pub fn new(stored: Vec<Project>) -> TemplatesUi {
        TemplatesUi { stored, ..Default::default() }
    }

    /// Reads the templates again when they are shown next.
    pub fn reload(&mut self) {
        self.library = None;
    }
}

/// Folder of the templates of the user; `None` in the browser.
fn folder() -> Option<PathBuf> {
    openecraft_client::places::templates_dir()
}

impl App {
    fn template_library(&mut self) -> Vec<(Project, bool)> {
        if self.templates_ui.library.is_none() {
            let user = match folder() {
                Some(dir) => {
                    let (found, errors) = templates::read_folder(&dir);
                    self.templates_ui.errors = errors;
                    found.into_iter().map(|(_, t)| t).collect()
                }
                None => self.templates_ui.stored.clone(),
            };
            self.templates_ui.library = Some(templates::library(user));
        }
        self.templates_ui.library.clone().unwrap_or_default()
    }

    /// The templates of the backstage page New: a click starts a project (asking for
    /// the values of a parametric template); the menu of a tile opens the template
    /// itself to change it or deletes one of the user.
    pub(super) fn backstage_templates(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let lib = self.template_library();
        let (mut start, mut edit, mut delete) = (None, None, None);
        ui.horizontal_wrapped(|ui| {
            for (i, (t, own)) in lib.iter().enumerate() {
                let title = templates::shown(&templates::name(t));
                let tip = if *own { tr_fmt!("{} (template of the user)", templates::shown(&t.template.description)) } else { templates::shown(&t.template.description) };
                let r = tile(ui, Icon::Template, &title).on_hover_text(tip);
                if r.clicked() {
                    start = Some(i);
                }
                r.context_menu(|ui| {
                    if ui.button(tr("Edit template")).clicked() {
                        edit = Some(i);
                    }
                    if *own && ui.button(tr("Delete template")).clicked() {
                        delete = Some(i);
                    }
                });
            }
        });
        for e in self.templates_ui.errors.clone() {
            ui.colored_label(crate::style::lock(), e);
        }
        if let Some(i) = start {
            let t = lib[i].0.clone();
            if t.template.parameters.is_empty() {
                self.start_from_template(&ctx, &t, &BTreeMap::new(), tr("New project"));
            } else {
                let values = t.template.parameters.iter().map(|p| (p.name.clone(), p.default.clone())).collect();
                self.templates_ui.start = Some(Start { template: t, values, name: String::new() });
            }
        }
        if let Some(i) = edit {
            let prj = templates::from_template(&lib[i].0, "", &openecraft_ui::io::today());
            self.request_replace(&ctx, Replace::Project(Box::new(prj), None));
            self.show_pane(crate::dock::Pane::Tool(crate::dock::ToolPane::ProjectTemplate));
        }
        if let Some(i) = delete {
            self.delete_template(&ctx, &templates::name(&lib[i].0));
        }
    }

    fn start_from_template(&mut self, ctx: &egui::Context, t: &Project, values: &BTreeMap<String, String>, name: &str) {
        match templates::instantiate(t, name, &openecraft_ui::io::today(), values) {
            Ok(prj) => self.request_replace(ctx, Replace::Project(Box::new(prj), None)),
            Err(e) => self.notify_error(ctx, e),
        }
    }

    fn delete_template(&mut self, ctx: &egui::Context, name: &str) {
        let result = match folder() {
            Some(dir) => templates::remove_from_folder(&dir, name).map(|_| ()),
            None => {
                self.templates_ui.stored.retain(|t| templates::name(t) != name);
                Ok(())
            }
        };
        self.templates_ui.library = None;
        match result {
            Ok(()) => self.notify(ctx, tr_fmt!("Template {} deleted", name)),
            Err(e) => self.notify_error(ctx, e),
        }
    }

    /// File › Save as project template: asks for its name and description.
    pub(super) fn save_as_template(&mut self) {
        let prj = &self.session.prj;
        self.templates_ui.save = Some(Save { name: templates::name(prj), description: prj.template.description.clone() });
    }

    pub(super) fn template_windows(&mut self, ctx: &egui::Context) {
        self.start_project_template_window(ctx);
        self.save_project_template_window(ctx);
    }

    fn start_project_template_window(&mut self, ctx: &egui::Context) {
        let Some(mut s) = self.templates_ui.start.take() else { return };
        let title = tr_fmt!("New project from {}", templates::shown(&templates::name(&s.template)));
        let (window, esc) = self.dialogs.window(ctx, &title, "start_template", None);
        let mut open = !esc;
        let (mut create, mut cancel) = (false, false);
        let computed = templates::values(&s.template.template, &s.values);
        window.open(&mut open).resizable(false).show(ctx, |ui| {
            egui::Grid::new("template_values").num_columns(3).spacing([10.0, 4.0]).show(ui, |ui| {
                ui.label(tr("Project name"));
                ui.add(egui::TextEdit::singleline(&mut s.name).hint_text(tr("New project")).desired_width(200.0));
                ui.end_row();
                for p in &s.template.template.parameters {
                    ui.label(label(&p.label, &p.name));
                    let v = s.values.entry(p.name.clone()).or_default();
                    if p.choices.is_empty() {
                        ui.add(egui::TextEdit::singleline(v).desired_width(200.0));
                    } else {
                        egui::ComboBox::from_id_salt(("template_choice", &p.name)).selected_text(v.clone()).width(200.0).show_ui(ui, |ui| {
                            for c in &p.choices {
                                ui.selectable_value(v, c.clone(), c);
                            }
                        });
                    }
                    ui.weak(&p.unit);
                    ui.end_row();
                }
            });
            ui.separator();
            match &computed {
                Ok(values) => {
                    egui::Grid::new("template_results").num_columns(3).spacing([10.0, 2.0]).show(ui, |ui| {
                        for f in &s.template.template.formulas {
                            let v = values.iter().find(|(n, _)| *n == f.name).map(|(_, v)| v.as_str()).unwrap_or_default();
                            ui.weak(label(&f.label, &f.name));
                            ui.label(v);
                            ui.weak(&f.unit);
                            ui.end_row();
                        }
                    });
                }
                Err(e) => {
                    ui.colored_label(crate::style::lock(), e);
                }
            }
            ui.separator();
            ui.horizontal(|ui| {
                create = ui.add_enabled(computed.is_ok(), egui::Button::new(tr("Create"))).clicked();
                cancel = ui.button(tr("Cancel")).clicked();
            });
        });
        if create {
            let name = if s.name.trim().is_empty() { String::new() } else { s.name.clone() };
            self.start_from_template(ctx, &s.template, &s.values, &name);
        } else if open && !cancel {
            self.templates_ui.start = Some(s);
        }
    }

    fn save_project_template_window(&mut self, ctx: &egui::Context) {
        let Some(mut s) = self.templates_ui.save.take() else { return };
        let (window, esc) = self.dialogs.window(ctx, tr("Save as project template"), "save_template", None);
        let mut open = !esc;
        let (mut save, mut export, mut cancel) = (false, false, false);
        let exists = self.template_library().iter().any(|(t, own)| *own && templates::name(t).eq_ignore_ascii_case(s.name.trim()));
        let bundled = templates::BUILTIN.iter().any(|(n, _)| n.eq_ignore_ascii_case(s.name.trim()) || tr(n).eq_ignore_ascii_case(s.name.trim()));
        let problems = templates::problems(&self.session.prj);
        window.open(&mut open).resizable(false).show(ctx, |ui| {
            egui::Grid::new("save_template").num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
                ui.label(tr("Name"));
                ui.add(egui::TextEdit::singleline(&mut s.name).desired_width(260.0));
                ui.end_row();
                ui.label(tr("Description"));
                ui.add(egui::TextEdit::multiline(&mut s.description).desired_rows(2).desired_width(260.0));
                ui.end_row();
            });
            match folder() {
                Some(dir) => ui.weak(tr_fmt!("Saved in {}, where the command line finds it too", dir.display())),
                None => ui.weak(tr("Saved in the browser")),
            };
            if exists {
                ui.colored_label(crate::style::warning(), tr("A template of this name is replaced."));
            } else if bundled {
                ui.colored_label(crate::style::warning(), tr("It takes the place of the bundled template of this name."));
            }
            for p in &problems {
                ui.colored_label(crate::style::lock(), p);
            }
            ui.horizontal(|ui| {
                save = ui.add_enabled(!s.name.trim().is_empty() && problems.is_empty(), egui::Button::new(tr("Save"))).clicked();
                export = ui.button(tr("Export as file…")).on_hover_text(tr("A template file to pass on; File › Open template file… starts projects from it")).clicked();
                cancel = ui.button(tr("Cancel")).clicked();
            });
        });
        let t = templates::save_as(&self.session.prj, &s.name, &s.description);
        if save {
            let result = match folder() {
                Some(dir) => templates::write_to_folder(&dir, &t, true).map(|_| ()),
                None => {
                    self.templates_ui.stored.retain(|x| !templates::name(x).eq_ignore_ascii_case(&templates::name(&t)));
                    self.templates_ui.stored.push(t);
                    Ok(())
                }
            };
            self.templates_ui.library = None;
            match result {
                Ok(()) => self.notify_ok(ctx, tr_fmt!("Template {} saved", s.name.trim())),
                Err(e) => self.notify_error(ctx, e),
            }
        } else if export {
            let file = templates::file_name(&s.name);
            self.export(ctx, file, (tr("OpenECraft project template"), "oec"), t.to_json());
        } else if open && !cancel {
            self.templates_ui.save = Some(s);
        }
    }

    /// The Template pane: the project as a template.
    pub(super) fn template_pane(&mut self, ui: &mut egui::Ui) {
        let defaults: BTreeMap<String, String> = templates::values(&self.session.prj.template, &BTreeMap::new()).unwrap_or_default().into_iter().collect();
        let problems = templates::problems(&self.session.prj);
        let mut save = false;
        let prj = &mut self.session.prj;
        ui.weak(tr("Parameters are asked for when a project is started from the template; formulas work out further values. Both fill {NAME} in the texts of the project. A page with a condition (REPORTS=yes) is left out when it does not hold."));
        ui.separator();
        egui::Grid::new("template_info").num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
            ui.label(tr("Name"));
            ui.add_sized([260.0, 20.0], egui::TextEdit::singleline(&mut prj.template.name).hint_text(prj.info.name.clone()));
            ui.end_row();
            ui.label(tr("Description"));
            ui.add_sized([420.0, 20.0], egui::TextEdit::singleline(&mut prj.template.description));
            ui.end_row();
        });
        ui.separator();
        ui.label(RichText::new(tr("Parameters")).strong());
        let mut remove = None;
        egui::ScrollArea::horizontal().id_salt("template_parameters_scroll").show(ui, |ui| {
            egui::Grid::new("template_parameters").striped(true).num_columns(7).spacing([10.0, 4.0]).show(ui, |ui| {
                for h in [tr("Name"), tr("Label"), tr("Unit"), tr("Default"), tr("Choices (separated by commas)"), tr("Numeric"), ""] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for (i, p) in prj.template.parameters.iter_mut().enumerate() {
                    ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut p.name));
                    ui.add_sized([160.0, 20.0], egui::TextEdit::singleline(&mut p.label));
                    ui.add_sized([40.0, 20.0], egui::TextEdit::singleline(&mut p.unit));
                    ui.add_sized([90.0, 20.0], egui::TextEdit::singleline(&mut p.default));
                    let mut choices = p.choices.join(", ");
                    if ui.add_sized([140.0, 20.0], egui::TextEdit::singleline(&mut choices)).changed() {
                        p.choices = choices.split(',').map(|c| c.trim().to_string()).filter(|c| !c.is_empty()).collect();
                    }
                    ui.checkbox(&mut p.number, "");
                    if icons::button(ui, Icon::Delete, tr("Remove")).clicked() {
                        remove = Some(i);
                    }
                    ui.end_row();
                }
            });
        });
        if let Some(i) = remove {
            prj.template.parameters.remove(i);
        }
        if icons::text_button(ui, Icon::Plus, tr("Parameter")).clicked() {
            let n = prj.template.parameters.len() + 1;
            prj.template.parameters.push(TemplateParameter { name: format!("PARAMETER{n}"), ..Default::default() });
        }
        ui.separator();
        ui.label(RichText::new(tr("Formulas")).strong());
        let mut remove = None;
        egui::ScrollArea::horizontal().id_salt("template_formulas_scroll").show(ui, |ui| {
            egui::Grid::new("template_formulas").striped(true).num_columns(6).spacing([10.0, 4.0]).show(ui, |ui| {
                for h in [tr("Name"), tr("Label"), tr("Unit"), tr("Formula"), tr("Value"), ""] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for (i, f) in prj.template.formulas.iter_mut().enumerate() {
                    ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut f.name));
                    ui.add_sized([160.0, 20.0], egui::TextEdit::singleline(&mut f.label));
                    ui.add_sized([40.0, 20.0], egui::TextEdit::singleline(&mut f.unit));
                    ui.add_sized([320.0, 20.0], egui::TextEdit::singleline(&mut f.formula).hint_text(tr("e.g. round(POWER_KW * 2, 1)")));
                    ui.label(defaults.get(&f.name).cloned().unwrap_or_default());
                    if icons::button(ui, Icon::Delete, tr("Remove")).clicked() {
                        remove = Some(i);
                    }
                    ui.end_row();
                }
            });
        });
        if let Some(i) = remove {
            prj.template.formulas.remove(i);
        }
        if icons::text_button(ui, Icon::Plus, tr("Formula")).on_hover_text(tr("sqrt, abs, round(x, digits), ceil, floor, min, max, if(condition, then, else), step(x, limit, value, …, otherwise); & joins texts")).clicked() {
            let n = prj.template.formulas.len() + 1;
            prj.template.formulas.push(TemplateFormula { name: format!("VALUE{n}"), formula: "0".into(), ..Default::default() });
        }
        ui.separator();
        ui.label(RichText::new(tr("Page conditions")).strong());
        egui::Grid::new("template_pages").striped(true).num_columns(3).spacing([10.0, 4.0]).show(ui, |ui| {
            for pg in &mut prj.pages {
                ui.label(&pg.name);
                ui.label(&pg.description);
                ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut pg.condition).hint_text(tr("always")));
                ui.end_row();
            }
        });
        ui.separator();
        for p in &problems {
            ui.colored_label(crate::style::lock(), p);
        }
        if icons::text_button(ui, Icon::Template, tr("Save as project template…")).clicked() {
            save = true;
        }
        if save {
            self.save_as_template();
        }
    }
}

/// The label of a parameter or formula in the current language, else its name.
fn label(label: &str, name: &str) -> String {
    if label.is_empty() { name.to_string() } else { templates::shown(label) }
}
