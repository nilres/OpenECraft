//! Workspaces: the built-in ones (schematic, panel layout, wiring, mounting, review,
//! pre-planning), each with its own arrangement of the panes, and the user's saved ones
//! (View › Workspaces), which remember the workspace, the arrangement and the ribbon,
//! saved by the app and restored with one click.

use openecraft_i18n::tr_const;
use serde::{Deserialize, Serialize};

/// Storage key of the saved workspaces.
pub const STORAGE_KEY: &str = "workspaces";

/// A built-in workspace: which panes are open and where (see `crate::dock`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Workspace {
    Schematic,
    Panel,
    /// Wiring assistant for the workshop.
    Wiring,
    /// Mounting assistant for the workshop.
    Mounting,
    Review,
    Preplanning,
}

impl Workspace {
    pub const ALL: [Workspace; 6] = [Workspace::Schematic, Workspace::Panel, Workspace::Wiring, Workspace::Mounting, Workspace::Review, Workspace::Preplanning];

    pub fn key(self) -> &'static str {
        match self {
            Workspace::Schematic => "schematic",
            Workspace::Panel => "panel",
            Workspace::Wiring => "wiring",
            Workspace::Mounting => "mounting",
            Workspace::Review => "review",
            Workspace::Preplanning => "preplanning",
        }
    }

    pub fn from_key(k: &str) -> Option<Workspace> {
        Workspace::ALL.into_iter().find(|w| w.key() == k)
    }

    pub fn title(self) -> &'static str {
        match self {
            Workspace::Schematic => tr_const!("Schematic"),
            Workspace::Panel => tr_const!("Panel layout"),
            Workspace::Wiring => tr_const!("Wiring"),
            Workspace::Mounting => tr_const!("Mounting"),
            Workspace::Review => tr_const!("Review"),
            Workspace::Preplanning => tr_const!("Pre-planning"),
        }
    }

    /// The wiring and mounting assistants share their panes.
    pub fn is_workshop(self) -> bool {
        matches!(self, Workspace::Wiring | Workspace::Mounting)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Layout {
    pub name: String,
    /// Key of the workspace (`schematic`, `panel`, …).
    pub workspace: String,
    /// Key of the navigator tab.
    pub left_tab: String,
    /// Ids of the open windows (workspaces saved before the tools were panes).
    pub windows: Vec<String>,
    /// Arrangement of the docked panes (JSON, see `crate::dock`); empty in workspaces
    /// saved before panels could be docked.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub dock: String,
    /// The ribbon customisation (JSON, see `crate::ribbon::Custom`); empty: unchanged.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ribbon: String,
}

/// Adds a layout or replaces the one with the same name; names are trimmed, empty
/// names are refused.
pub fn save(list: &mut Vec<Layout>, mut layout: Layout) -> bool {
    layout.name = layout.name.trim().to_string();
    if layout.name.is_empty() {
        return false;
    }
    match list.iter_mut().find(|l| l.name == layout.name) {
        Some(l) => *l = layout,
        None => list.push(layout),
    }
    true
}

pub fn remove(list: &mut Vec<Layout>, name: &str) {
    list.retain(|l| l.name != name);
}

/// The tool panes of the windows a workspace of the earlier editor had open.
pub fn window_panes(layout: &Layout) -> Vec<crate::dock::ToolPane> {
    use crate::dock::ToolPane as T;
    layout
        .windows
        .iter()
        .filter_map(|w| {
            Some(match w.as_str() {
                "reports" => T::Reports,
                "cables" => T::Cables,
                "macros" => T::Macros,
                "terminal_strips" => T::TerminalStrips,
                "potentials" => T::Potentials,
                "plc_overview" => T::PlcOverview,
                "revisions" => T::Revisions,
                "find" => T::Find,
                "forms" => T::Forms,
                "report_blocks" => T::ReportBlocks,
                "harnesses" => T::Harnesses,
                "field_wiring" => T::FieldWiring,
                "buildings" => T::Buildings,
                "plugs" => T::Plugs,
                "plc_hardware" => T::PlcHardware,
                "integration" => T::Integration,
                "process" => T::Process,
                "preplanning" => T::Preplanning,
                "electrical" => T::Electrical,
                "single_line" => T::SingleLine,
                "languages" => T::Languages,
                "tables" => T::Tables,
                "configurator" => T::Configurator,
                "compare" => T::Compare,
                "change_requests" => T::ChangeRequests,
                "structure" => T::Structure,
                "property_database" => T::PropertyDatabase,
                "projects" => T::Projects,
                "parts" => T::Parts,
                "symbol_editor" => T::SymbolEditor,
                _ => return None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
