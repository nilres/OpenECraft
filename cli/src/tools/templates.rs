//! Project template tools: the bundled templates and those of the user in the
//! templates folder (`--templates`, `OPENECRAFT_TEMPLATES`, otherwise in the data
//! folder of the editor, which shows the same templates).

use super::{optional, param, Args, Kind, Tool};
use crate::session::Session;
use openecraft_core::model::Project;
use openecraft_core::project_templates as templates;
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

pub static TOOLS: &[Tool] = &[
    Tool {
        name: "list_project_templates",
        help: tr_const!("Lists the project templates (the bundled ones and those of the templates folder: --templates / OPENECRAFT_TEMPLATES) with their parameters, formulas and the pages a condition leaves out"),
        params: &[],
        needs_project: false,
        run: list_project_templates,
    },
    Tool {
        name: "template_values",
        help: tr_const!("Works out the values of a project template for the given parameters (rated current, setting ranges, parts) without starting a project"),
        params: &[
            param("template", Kind::Str, tr_const!("Name of a template (list_project_templates) or a template file")),
            optional("values", Kind::Object, tr_const!("Values of the parameters (name: value); the others keep their default")),
        ],
        needs_project: false,
        run: template_values,
    },
    Tool {
        name: "save_project_template",
        help: tr_const!("Saves the project as a template in the templates folder, with its parameters, formulas and page conditions; a template of the user with the name of a bundled one takes its place"),
        params: &[
            optional("name", Kind::Str, tr_const!("Name of the template (default: its template name, otherwise the project name)")),
            optional("description", Kind::Str, tr_const!("Description")),
            optional("overwrite", Kind::Bool, tr_const!("Replace a template of the same name")),
        ],
        needs_project: true,
        run: save_project_template,
    },
    Tool {
        name: "delete_project_template",
        help: tr_const!("Deletes a template of the user from the templates folder (a bundled one of the same name comes back)"),
        params: &[param("name", Kind::Str, tr_const!("Name of the template"))],
        needs_project: false,
        run: delete_project_template,
    },
];

fn folder(s: &Session) -> Result<&Path, String> {
    s.templates.as_deref().ok_or_else(|| tr("No templates folder: pass --templates <folder> or set OPENECRAFT_TEMPLATES").to_string())
}

/// The templates to choose from and the files of the folder that could not be read.
pub fn library(s: &Session) -> (Vec<(Project, bool)>, Vec<String>) {
    let (found, errors) = s.templates.as_deref().map(templates::read_folder).unwrap_or_default();
    (templates::library(found.into_iter().map(|(_, t)| t).collect()), errors)
}

/// A template by name, otherwise read from the file `name`.
pub fn find(s: &Session, name: &str) -> Result<Project, String> {
    let (lib, _) = library(s);
    if let Some(t) = templates::find(&lib, name) {
        return Ok(t.clone());
    }
    if !Path::new(name).exists() {
        let names: Vec<String> = lib.iter().map(|(t, _)| templates::shown(&templates::name(t))).collect();
        return Err(tr_fmt!("Unknown template {}: give a built-in one ({}) or a project file", name, names.join(", ")));
    }
    Ok(crate::session::read_project(Path::new(name))?.0)
}

/// The values of `values` as texts (true / false as yes / no).
pub fn given(a: &Args) -> BTreeMap<String, String> {
    let Some(Value::Object(o)) = a.value("values") else { return BTreeMap::new() };
    o.iter()
        .map(|(k, v)| {
            let v = match v {
                Value::String(s) => s.clone(),
                Value::Bool(b) => if *b { "yes" } else { "no" }.to_string(),
                other => other.to_string(),
            };
            (k.clone(), v)
        })
        .collect()
}

fn template_json(t: &Project, own: bool) -> Value {
    let def = &t.template;
    json!({
        "name": templates::shown(&templates::name(t)),
        "description": templates::shown(&def.description),
        "own": own,
        "parameters": def.parameters.iter().map(|p| json!({ "name": p.name, "label": templates::shown(&p.label), "unit": p.unit, "default": p.default, "choices": p.choices, "number": p.number })).collect::<Vec<_>>(),
        "formulas": def.formulas.iter().map(|f| json!({ "name": f.name, "label": templates::shown(&f.label), "unit": f.unit, "formula": f.formula })).collect::<Vec<_>>(),
        "pages": t.pages.len(),
        "optional_pages": t.pages.iter().filter(|p| !p.condition.is_empty()).map(|p| json!({ "page": p.name, "description": p.description, "condition": p.condition })).collect::<Vec<_>>(),
    })
}

fn list_project_templates(s: &mut Session, _: &Args) -> Result<Value, String> {
    let (lib, errors) = library(s);
    Ok(json!({
        "folder": s.templates.as_ref().map(|p| p.display().to_string()),
        "templates": lib.iter().map(|(t, own)| template_json(t, *own)).collect::<Vec<_>>(),
        "errors": errors,
    }))
}

fn template_values(s: &mut Session, a: &Args) -> Result<Value, String> {
    let t = find(s, a.req("template"))?;
    let def = &t.template;
    let values = templates::values(def, &given(a))?;
    let label = |n: &str| {
        let p = def.parameters.iter().find(|p| p.name == n).map(|p| (p.label.as_str(), p.unit.as_str()));
        let (label, unit) = p.or_else(|| def.formulas.iter().find(|f| f.name == n).map(|f| (f.label.as_str(), f.unit.as_str()))).unwrap_or_default();
        (templates::shown(label), unit.to_string())
    };
    Ok(json!({
        "template": templates::shown(&templates::name(&t)),
        "values": values.iter().map(|(n, v)| {
            let (label, unit) = label(n);
            json!({ "name": n, "label": label, "unit": unit, "value": v })
        }).collect::<Vec<_>>(),
    }))
}

fn save_project_template(s: &mut Session, a: &Args) -> Result<Value, String> {
    let dir = folder(s)?.to_path_buf();
    let prj = s.project()?;
    let name = a.str("name").map(String::from).unwrap_or_else(|| templates::name(prj));
    if name.trim().is_empty() {
        return Err(tr("A template needs a name").to_string());
    }
    let description = a.str("description").map(String::from).unwrap_or_else(|| prj.template.description.clone());
    let t = templates::save_as(prj, &name, &description);
    if let Some(p) = templates::problems(&t).into_iter().next() {
        return Err(p);
    }
    let path = templates::write_to_folder(&dir, &t, a.flag("overwrite"))?;
    Ok(json!({ "name": templates::name(&t), "path": path.display().to_string() }))
}

fn delete_project_template(s: &mut Session, a: &Args) -> Result<Value, String> {
    let path = templates::remove_from_folder(folder(s)?, a.req("name"))?;
    Ok(json!({ "deleted": path.display().to_string() }))
}

#[cfg(test)]
mod tests;
