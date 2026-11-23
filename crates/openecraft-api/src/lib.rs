//! Extension API of OpenECraft. A [`Plugin`] adds commands, checks, reports and
//! exporters to a [`Registry`]; the front ends (editor, command line, MCP server) take
//! them from [`Plugins`], so every extension is available everywhere.
//!
//! Plugins are Rust types compiled in (the built-in ones: [`builtin`]) or external
//! programs that answer JSON requests on stdin / stdout ([`external`], see
//! `docs/plugins.md`). The extension vocabulary itself (registry kinds, parameters,
//! tables) lives in `openecraft_core::{ext, params, table}` and is re-exported here.

pub mod builtin;
pub mod external;
mod host;

/// Typed parameters of commands (`openecraft_core::params`).
pub mod params {
    pub use openecraft_core::params::*;
}

pub use host::{run_checks, Loaded, Origin, Plugins, PLUGIN_PATH_VAR};
pub use openecraft_core::ext::{title, Check, Command, Effect, Export, Exporter, Registry, Report};
pub use openecraft_core::table::Table;
pub use params::{optional, param, Args, Kind, Param, Text};

use serde::{Deserialize, Serialize};

/// Version of the API; external plugins declare the one they were written for.
pub const API_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Info {
    /// Unique, snake_case.
    pub id: String,
    pub name: Text,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: Text,
}

pub trait Plugin {
    fn info(&self) -> Info;

    /// Adds the plugin's extensions.
    fn register(&self, reg: &mut Registry) -> Result<(), String>;
}

/// Name rule of plugin ids, commands, checks, reports and exporters.
pub fn is_snake_case(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase()) && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}
