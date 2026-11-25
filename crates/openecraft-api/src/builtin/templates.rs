//! Project template commands: the name, parameters and formulas of the open project
//! as a template, with the conditions of its pages (`set_page --condition`).

use crate::params::{optional, param, Args, Kind};
use crate::{Info, Plugin, Registry};
use super::{add_commands, info, Spec};
use openecraft_core::model::Project;
use openecraft_core::project_templates::{self as templates, TemplateFormula, TemplateParameter};
use openecraft_i18n::{tr_const, tr_fmt};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub struct Templates;

impl Plugin for Templates {
    fn info(&self) -> Info {
        info("project_templates", tr_const!("Project templates"), tr_const!("Name, parameters, formulas and page conditions of a project template"))
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        add_commands(reg, COMMANDS)
    }
}

const NAME: crate::params::Param = param("name", Kind::Str, tr_const!("Name of the placeholder, e.g. POWER_KW for {POWER_KW}: letters, digits and _"));

pub static COMMANDS: &[Spec] = &[
    Spec {
        name: "get_template",
        help: tr_const!("Shows the open project as a template: name, parameters, formulas with their values at the defaults, the pages with a condition and the mistakes"),
        params: &[],
        run: get_template,
    },
    Spec {
        name: "set_template",
        help: tr_const!("Changes the name and description of the open project as a template (save_project_template saves it)"),
        params: &[
            optional("name", Kind::Str, tr_const!("Name in the list of templates")),
            optional("description", Kind::Str, tr_const!("Description")),
        ],
        run: set_template,
    },
    Spec {
        name: "set_template_parameter",
        help: tr_const!("Creates or changes a parameter of the template: a value asked for when a project is started from it, filling {NAME} in the texts of the project"),
        params: &[
            NAME,
            optional("label", Kind::Str, tr_const!("Label shown when the value is asked for")),
            optional("unit", Kind::Str, tr_const!("Unit, e.g. kW")),
            optional("default", Kind::Str, tr_const!("Default value")),
            optional("choices", Kind::Str, tr_const!("Allowed values separated by commas (empty: any value)")),
            optional("number", Kind::Bool, tr_const!("The value must be a number")),
        ],
        run: set_template_parameter,
    },
    Spec {
        name: "set_template_formula",
        help: tr_const!("Creates or changes a formula of the template: a value worked out from the parameters and the formulas before it, filling {NAME}"),
        params: &[
            NAME,
            optional("formula", Kind::Str, tr_const!("Formula: numbers, \"texts\", names, + - * / ^, comparisons, & and sqrt, abs, round, ceil, floor, min, max, if, step")),
            optional("label", Kind::Str, tr_const!("Label shown with the value")),
            optional("unit", Kind::Str, tr_const!("Unit, e.g. A")),
        ],
        run: set_template_formula,
    },
    Spec {
        name: "remove_template_parameter",
        help: tr_const!("Removes a parameter or formula of the template"),
        params: &[NAME],
        run: remove_template_parameter,
    },
];

fn template_json(prj: &Project) -> Value {
    let def = &prj.template;
    let values: BTreeMap<String, String> = templates::values(def, &BTreeMap::new()).unwrap_or_default().into_iter().collect();
    json!({
        "name": def.name,
        "description": def.description,
        "parameters": def.parameters.iter().map(|p| json!({ "name": p.name, "label": p.label, "unit": p.unit, "default": p.default, "choices": p.choices, "number": p.number })).collect::<Vec<_>>(),
        "formulas": def.formulas.iter().map(|f| json!({ "name": f.name, "label": f.label, "unit": f.unit, "formula": f.formula, "value": values.get(&f.name) })).collect::<Vec<_>>(),
        "pages": prj.pages.iter().filter(|p| !p.condition.is_empty()).map(|p| json!({ "page": p.name, "condition": p.condition })).collect::<Vec<_>>(),
        "problems": templates::problems(prj),
    })
}

fn get_template(prj: &mut Project, _: &Args) -> Result<Value, String> {
    Ok(template_json(prj))
}

/// The changed template, refused with its first mistake.
fn checked(prj: &Project) -> Result<Value, String> {
    match templates::problems(prj).into_iter().next() {
        Some(p) => Err(p),
        None => Ok(template_json(prj)),
    }
}

fn set_template(prj: &mut Project, a: &Args) -> Result<Value, String> {
    if let Some(n) = a.str("name") {
        prj.template.name = n.trim().into();
    }
    if let Some(d) = a.str("description") {
        prj.template.description = d.into();
    }
    checked(prj)
}

fn set_template_parameter(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let name = a.req("name").trim();
    let def = &mut prj.template;
    let i = match def.parameters.iter().position(|p| p.name == name) {
        Some(i) => i,
        None => {
            def.parameters.push(TemplateParameter { name: name.into(), ..Default::default() });
            def.parameters.len() - 1
        }
    };
    let p = &mut def.parameters[i];
    for (key, field) in [("label", &mut p.label), ("unit", &mut p.unit), ("default", &mut p.default)] {
        if let Some(v) = a.str(key) {
            *field = v.into();
        }
    }
    if let Some(c) = a.str("choices") {
        p.choices = c.split(',').map(str::trim).filter(|c| !c.is_empty()).map(String::from).collect();
    }
    if a.value("number").is_some() {
        p.number = a.flag("number");
    }
    checked(prj)
}

fn set_template_formula(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let name = a.req("name").trim();
    let def = &mut prj.template;
    let i = match def.formulas.iter().position(|f| f.name == name) {
        Some(i) => i,
        None => {
            if a.str("formula").is_none_or(|f| f.trim().is_empty()) {
                return Err(tr_fmt!("A new formula {} needs its formula", name));
            }
            def.formulas.push(TemplateFormula { name: name.into(), ..Default::default() });
            def.formulas.len() - 1
        }
    };
    let f = &mut def.formulas[i];
    for (key, field) in [("formula", &mut f.formula), ("label", &mut f.label), ("unit", &mut f.unit)] {
        if let Some(v) = a.str(key) {
            *field = v.into();
        }
    }
    checked(prj)
}

fn remove_template_parameter(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let name = a.req("name").trim();
    let def = &mut prj.template;
    let before = def.parameters.len() + def.formulas.len();
    def.parameters.retain(|p| p.name != name);
    def.formulas.retain(|f| f.name != name);
    if def.parameters.len() + def.formulas.len() == before {
        return Err(tr_fmt!("The template has no parameter {}", name));
    }
    checked(prj)
}

#[cfg(test)]
mod tests;
