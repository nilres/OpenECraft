//! Read-only views of a project, shared by the editor (its navigator) and the web page
//! of the project server: the pages by section or grouped by structure, the devices
//! grouped by structure with the places they are shown on, the icons of the page
//! kinds. No UI: the editor draws the views with egui, the project server sends them
//! to its web page as JSON ([`ProjectView`]).

mod project;

pub use project::*;
