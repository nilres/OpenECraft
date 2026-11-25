//! Project templates (in the spirit of EPLAN's basic projects): a project whose
//! settings, forms, plot frames, structure, potentials, numbering schemes, symbols,
//! property definitions and standard pages start new projects. A template is an
//! ordinary project file; [`as_template`] removes what belongs to one project only
//! (revisions, baselines, review, wiring marks, accepted check messages, subprojects),
//! [`from_template`] starts a project from it. Built-in templates cover the usual
//! starting points.
//!
//! A template can ask for parameters (`POWER_KW`) and work out further values with
//! formulas ([`crate::formula`]: the rated current, the setting range of the motor
//! protection switch, the part of the contactor). Their values fill the placeholders
//! `{NAME}` in the texts of the project (tags, function texts, texts, device data,
//! part numbers, page descriptions), and pages with a condition (`REPORTS=yes`) are
//! left out when it does not hold ([`instantiate`]). The templates of the user (a
//! folder of template files) come beside the bundled ones and take the place of a
//! bundled one of the same name ([`library`]), so a bundled template is extended by
//! saving a changed copy under its name.

use crate::config::Configuration;
use crate::formula;
use crate::model::config::ConfigOption;
use crate::model::{Page, PageKind, Project, ReportKind};
use openecraft_i18n::{tr, tr_const, tr_fmt};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
pub use crate::model::project_template::{TemplateDefinition, TemplateFormula, TemplateParameter};

mod starters;

/// Built-in templates: (name, description).
pub const BUILTIN: [(&str, &str); 6] = [
    (tr_const!("Empty (IEC 60204-1)"), tr_const!("One schematic page, the potentials of IEC 60204-1")),
    (tr_const!("Machine"), tr_const!("Cover sheet, table of contents, schematic pages with function and location, device list, bill of materials, terminal and cable diagrams")),
    (tr_const!("Distribution board (IEC 61439)"), tr_const!("Cover sheet, single-line overview, schematic pages, bill of materials, enclosure climate and short circuit reports")),
    (tr_const!("Motor starter direct on line"), tr_const!("A three-phase motor switched on directly: power and control circuit sized from the motor power, optional terminal diagram, connection list, device list and bill of materials")),
    (tr_const!("Motor starter star-delta"), tr_const!("A three-phase motor started in star and run in delta: power and control circuit sized from the motor power, optional report pages")),
    (tr_const!("Motor starter reversing"), tr_const!("A three-phase motor in both directions with interlocked contactors: power and control circuit sized from the motor power, optional report pages")),
];

/// The project without what belongs to one project only.
pub fn as_template(prj: &Project) -> Project {
    let mut t = prj.clone();
    t.revisions.clear();
    t.baselines.clear();
    t.review = Default::default();
    t.inspection = Default::default();
    t.wiring_marks.clear();
    t.mounting_marks.clear();
    t.preplanning.segments.clear();
    t.subprojects = Default::default();
    t.check_settings.accepted.clear();
    t.info.date.clear();
    t.info.revision = "A".into();
    for pg in &mut t.pages {
        pg.locked = false;
    }
    t
}

/// A new project from a template: its name and date set, released pages unlocked.
/// Placeholders and page conditions stay as they are: [`instantiate`] fills them.
pub fn from_template(template: &Project, name: &str, date: &str) -> Project {
    let mut prj = as_template(template);
    if !name.trim().is_empty() {
        prj.info.name = name.trim().to_string();
    }
    prj.info.date = date.to_string();
    // A new project counts its ids on, whatever the template was read from.
    prj.id_floor = Default::default();
    prj.normalize();
    prj
}

/// The project as a template named `name`, with its parameters and formulas.
pub fn save_as(prj: &Project, name: &str, description: &str) -> Project {
    let mut t = as_template(prj);
    t.template.name = name.trim().to_string();
    t.template.description = description.trim().to_string();
    t
}

/// Name of a template in the list: its own, otherwise that of the project.
pub fn name(t: &Project) -> String {
    if t.template.name.is_empty() { t.info.name.clone() } else { t.template.name.clone() }
}

/// A text of a template (name, description, label) in the current language: the
/// bundled ones are in the catalog, those of the user as they were written.
pub fn shown(text: &str) -> String {
    openecraft_i18n::translation(text, openecraft_i18n::lang()).unwrap_or(text).to_string()
}

/// Whether `t` is the template `name` (also translated).
fn named(t: &str, name: &str) -> bool {
    let name = name.trim();
    t.eq_ignore_ascii_case(name) || shown(t).to_lowercase() == name.to_lowercase()
}

/// File name of a template in a templates folder: its name with the characters that
/// file systems do not allow replaced.
pub fn file_name(name: &str) -> String {
    let base: String = name.trim().chars().map(|c| if c.is_control() || "<>:\"/\\|?*".contains(c) { '_' } else { c }).collect();
    let base = base.trim_matches(['.', ' ']);
    format!("{}.oec", if base.is_empty() { "template" } else { base })
}

fn add(prj: &mut Project, name: &str, description: &str, kind: PageKind, function: &str, location: &str) {
    let id = prj.new_id();
    let mut pg = Page::new(id, name, description, kind);
    pg.function = function.into();
    pg.location = location.into();
    prj.pages.push(pg);
}

/// A built-in template by name (English or translated); None for an unknown name.
pub fn builtin(name: &str) -> Option<Project> {
    let i = BUILTIN.iter().position(|(n, _)| named(n, name))?;
    let mut prj = Project::new_default();
    match i {
        0 => {}
        1 => {
            prj.pages.clear();
            crate::structure::apply_template(&mut prj.structures, crate::structure::template("Machine (example identifiers)")?);
            add(&mut prj, "1", tr("Cover sheet"), PageKind::Cover, "", "");
            add(&mut prj, "2", tr("Table of contents"), PageKind::Report(ReportKind::TableOfContents.into()), "", "");
            add(&mut prj, "3", tr("Power supply"), PageKind::Schematic, "POWER", "CAB1");
            add(&mut prj, "4", tr("Control"), PageKind::Schematic, "CTRL", "CAB1");
            add(&mut prj, "5", tr("Drives"), PageKind::Schematic, "DRIVE", "CAB1");
            for (n, k) in [("6", ReportKind::DeviceList), ("7", ReportKind::BillOfMaterials), ("8", ReportKind::TerminalDiagram), ("9", ReportKind::CableDiagram)] {
                add(&mut prj, n, k.title(), PageKind::Report(k.into()), "", "");
            }
        }
        2 => {
            prj.pages.clear();
            add(&mut prj, "1", tr("Cover sheet"), PageKind::Cover, "", "");
            add(&mut prj, "2", tr("Single-line diagram"), PageKind::Overview, "", "");
            add(&mut prj, "3", tr("Incoming supply"), PageKind::Schematic, "", "");
            add(&mut prj, "4", tr("Outgoing circuits"), PageKind::Schematic, "", "");
            for (n, k) in [("5", ReportKind::BillOfMaterials), ("6", ReportKind::Climate), ("7", ReportKind::ShortCircuit)] {
                add(&mut prj, n, k.title(), PageKind::Report(k.into()), "", "");
            }
        }
        3 => prj = starters::direct_on_line(),
        4 => prj = starters::star_delta(),
        _ => prj = starters::reversing(),
    }
    prj.template.name = BUILTIN[i].0.into();
    prj.template.description = BUILTIN[i].1.into();
    Some(prj)
}

/// The bundled templates.
pub fn bundled() -> Vec<Project> {
    BUILTIN.iter().filter_map(|(n, _)| builtin(n)).collect()
}

/// The templates to choose from: the bundled ones, a template of the user in the
/// place of a bundled one of the same name, then the other templates of the user by
/// name. The flag tells the templates of the user.
pub fn library(user: Vec<Project>) -> Vec<(Project, bool)> {
    let mut user = user;
    user.sort_by(|a, b| crate::model::natural_cmp(&name(a), &name(b)));
    let mut out: Vec<(Project, bool)> = Vec::new();
    for b in bundled() {
        match user.iter().position(|u| named(&b.template.name, &name(u))) {
            Some(i) => out.push((user.remove(i), true)),
            None => out.push((b, false)),
        }
    }
    out.extend(user.into_iter().map(|u| (u, true)));
    out
}

/// A template of the list by name (bundled ones also by their translated name).
pub fn find<'a>(lib: &'a [(Project, bool)], name: &str) -> Option<&'a Project> {
    lib.iter().map(|(t, _)| t).find(|t| named(&self::name(t), name))
}

fn valid_name(n: &str) -> bool {
    let mut c = n.chars();
    c.next().is_some_and(|f| f.is_ascii_alphabetic() || f == '_') && c.all(|x| x.is_ascii_alphanumeric() || x == '_')
}

/// The values of a project started from the template: the parameters (given or
/// their default) and the formulas, in the order of the definition.
pub fn values(def: &TemplateDefinition, given: &BTreeMap<String, String>) -> Result<Vec<(String, String)>, String> {
    if let Some(n) = given.keys().find(|n| !def.parameters.iter().any(|p| &p.name == *n)) {
        return Err(tr_fmt!("The template has no parameter {}", n));
    }
    let mut known = BTreeMap::new();
    let mut out = Vec::new();
    for p in &def.parameters {
        let v = given.get(&p.name).cloned().unwrap_or_else(|| p.default.clone());
        if p.number && !matches!(formula::Value::parse(&v), formula::Value::Num(_)) {
            return Err(tr_fmt!("{}: {} is not a number", p.name, v));
        }
        if !p.choices.is_empty() && !p.choices.contains(&v) {
            return Err(tr_fmt!("{}: {} is not one of its choices", p.name, v));
        }
        known.insert(p.name.clone(), v.clone());
        out.push((p.name.clone(), v));
    }
    for f in &def.formulas {
        let v = formula::eval(&f.formula, &known).map_err(|e| tr_fmt!("{}: {}", f.name, e))?.text();
        known.insert(f.name.clone(), v.clone());
        out.push((f.name.clone(), v));
    }
    Ok(out)
}

fn holds(condition: &str, values: &[(String, String)]) -> Result<bool, String> {
    let options = values.iter().map(|(n, v)| ConfigOption { name: n.clone(), choices: Vec::new(), value: v.clone() }).collect();
    Configuration { options, rules: Vec::new() }.holds(condition)
}

/// Mistakes in the definition of a template: names, defaults, formulas and page
/// conditions (worked out with the defaults).
pub fn problems(t: &Project) -> Vec<String> {
    let def = &t.template;
    let mut v = Vec::new();
    let mut seen = Vec::new();
    for n in def.parameters.iter().map(|p| &p.name).chain(def.formulas.iter().map(|f| &f.name)) {
        if !valid_name(n) {
            v.push(tr_fmt!("{} is no valid name: letters, digits and _, not starting with a digit", n));
        } else if seen.contains(&n) {
            v.push(tr_fmt!("{} is defined twice", n));
        }
        seen.push(n);
    }
    match values(def, &BTreeMap::new()) {
        Err(e) => v.push(e),
        Ok(values) => {
            for pg in t.pages.iter().filter(|p| !p.condition.trim().is_empty()) {
                if let Err(e) = holds(&pg.condition, &values) {
                    v.push(tr_fmt!("Page {}: {}", pg.name, e));
                }
            }
        }
    }
    v
}

fn fill(text: &str, values: &BTreeMap<String, String>) -> String {
    let mut out = text.to_string();
    for (k, v) in values {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

/// Fills the placeholders in every text and key below `v`.
fn fill_json(v: &mut serde_json::Value, values: &BTreeMap<String, String>) {
    match v {
        serde_json::Value::String(s) if s.contains('{') => *s = fill(s, values),
        serde_json::Value::Array(a) => a.iter_mut().for_each(|x| fill_json(x, values)),
        serde_json::Value::Object(o) => {
            for (k, mut x) in std::mem::take(o) {
                fill_json(&mut x, values);
                o.insert(if k.contains('{') { fill(&k, values) } else { k }, x);
            }
        }
        _ => {}
    }
}

/// What keeps its placeholders: the definition itself, the forms (whose fields are
/// placeholders too), the configurator and the page templates.
const UNFILLED: [&str; 4] = ["template", "forms", "configuration", "page_templates"];

/// A new project from a template (see [`from_template`]) with the values of its
/// parameters (`given`, the others at their default): the pages whose condition does
/// not hold left out, the placeholders filled, the parts named in the device data
/// taken from the built-in catalog where the template does not have them.
pub fn instantiate(template: &Project, name: &str, date: &str, given: &BTreeMap<String, String>) -> Result<Project, String> {
    let values = values(&template.template, given)?;
    let mut prj = from_template(template, name, date);
    let mut keep = Vec::new();
    for pg in &prj.pages {
        keep.push(pg.condition.trim().is_empty() || holds(&pg.condition, &values).map_err(|e| tr_fmt!("Page {}: {}", pg.name, e))?);
    }
    let mut keep = keep.into_iter();
    prj.pages.retain(|_| keep.next().unwrap_or(true));
    if prj.pages.is_empty() {
        let id = prj.new_id();
        prj.pages.push(Page::new(id, "1", "", PageKind::Schematic));
    }
    let map: BTreeMap<String, String> = values.into_iter().collect();
    let mut json = serde_json::to_value(&prj).map_err(|e| e.to_string())?;
    if let serde_json::Value::Object(o) = &mut json {
        for (_, v) in o.iter_mut().filter(|(k, _)| !UNFILLED.contains(&k.as_str())) {
            fill_json(v, &map);
        }
    }
    let mut prj = Project::from_json(&json.to_string())?;
    let db = crate::parts::PartsDb::default();
    let mut numbers = Vec::new();
    for d in prj.devices.values_mut() {
        d.parts.retain(|p| !p.number.trim().is_empty());
        numbers.extend(d.parts.iter().map(|p| p.number.clone()));
    }
    for n in numbers {
        crate::parts::ensure_in_project(&mut prj, &db, &n);
    }
    prj.template = Default::default();
    for pg in &mut prj.pages {
        pg.condition.clear();
    }
    prj.normalize();
    Ok(prj)
}

/// The templates of a folder (its `.oec` files, by name) and the files that could
/// not be read.
pub fn read_folder(dir: &Path) -> (Vec<(PathBuf, Project)>, Vec<String>) {
    let (mut found, mut errors) = (Vec::new(), Vec::new());
    let Ok(entries) = std::fs::read_dir(dir) else { return (found, errors) };
    let mut files: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("oec"))).collect();
    files.sort();
    for f in files {
        match std::fs::read_to_string(&f).map_err(|e| openecraft_i18n::io_error(&e)).and_then(|text| Project::from_json(&text)) {
            Ok(t) => found.push((f, t)),
            Err(e) => errors.push(tr_fmt!("Cannot read {}: {}", f.display(), e)),
        }
    }
    found.sort_by(|a, b| crate::model::natural_cmp(&name(&a.1), &name(&b.1)));
    (found, errors)
}

/// Writes the template into the folder (created when missing) under the name of its
/// file ([`file_name`]); a template of the same name is only replaced with `overwrite`.
pub fn write_to_folder(dir: &Path, t: &Project, overwrite: bool) -> Result<PathBuf, String> {
    let existing = read_folder(dir).0.into_iter().find(|(_, x)| named(&name(x), &name(t))).map(|(p, _)| p);
    let path = existing.clone().unwrap_or_else(|| dir.join(file_name(&name(t))));
    if (existing.is_some() || path.exists()) && !overwrite {
        return Err(tr_fmt!("{} exists already: pass overwrite to replace it", path.display()));
    }
    std::fs::create_dir_all(dir).and_then(|_| std::fs::write(&path, t.to_json())).map_err(|e| tr_fmt!("Cannot write {}: {}", path.display(), openecraft_i18n::io_error(&e)))?;
    Ok(path)
}

/// Removes the template `name` from the folder; returns its file.
pub fn remove_from_folder(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let (path, _) = read_folder(dir).0.into_iter().find(|(_, t)| named(&self::name(t), name)).ok_or_else(|| tr_fmt!("No template of the user is named {}", name))?;
    std::fs::remove_file(&path).map_err(|e| tr_fmt!("Cannot delete {}: {}", path.display(), openecraft_i18n::io_error(&e)))?;
    Ok(path)
}

#[cfg(test)]
mod tests;
