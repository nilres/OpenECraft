//! Data of `crate::project_templates` that the project file holds (see `docs/architecture.md`).

use serde::{Deserialize, Serialize};

/// What makes a project a project template: its name in the list of templates, the
/// parameters asked for when a project is started from it and the values worked out
/// from them. Both fill the placeholders `{NAME}` of the project.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TemplateDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<TemplateParameter>,
    pub formulas: Vec<TemplateFormula>,
}

impl TemplateDefinition {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.description.is_empty() && self.parameters.is_empty() && self.formulas.is_empty()
    }
}

/// A value asked for when a project is started from the template.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TemplateParameter {
    /// Name of the placeholder (`POWER_KW` fills `{POWER_KW}`).
    pub name: String,
    pub label: String,
    pub unit: String,
    pub default: String,
    /// Allowed values; empty: any value.
    pub choices: Vec<String>,
    /// The value must be a number.
    pub number: bool,
}

/// A value worked out from the parameters and the formulas before it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TemplateFormula {
    pub name: String,
    pub label: String,
    pub unit: String,
    /// See `crate::formula`.
    pub formula: String,
}
