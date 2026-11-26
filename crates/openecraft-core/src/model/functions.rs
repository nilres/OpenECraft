//! Data of `crate::functions` that the project file holds (see `docs/architecture.md`).


use openecraft_parts::FnKind;
use super::properties::Values;
use serde::{Deserialize, Serialize};

/// A function definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FunctionDef {
    pub key: String,
    /// English name of a built-in definition, the name of a project one.
    pub name: String,
    pub category: FnCategory,
    /// Kind in the function templates of parts.
    pub kind: FnKind,
    /// Identifying letter of new devices (IEC 81346-2).
    pub letter: String,
    /// Default symbol (library key).
    pub symbol: String,
    /// Connection point designations separated by blanks; empty: the symbol's.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub pins: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Default property values of objects with this definition.
    #[serde(skip_serializing_if = "Values::is_empty")]
    pub defaults: Values,
    /// Placement scheme applied when the definition is assigned.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub scheme: String,
    #[serde(skip)]
    pub builtin: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FnCategory {
    Relays,
    Contacts,
    Switches,
    Sensors,
    Protection,
    Motors,
    Drives,
    Signal,
    Measuring,
    PowerSupply,
    Installation,
    Terminals,
    Plugs,
    Plc,
    Passive,
    Semiconductors,
    BlackBox,
    Potentials,
    Fluid,
    Process,
    #[default]
    General,
}
