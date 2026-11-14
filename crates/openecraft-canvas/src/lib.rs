//! The drawing views of the OpenECraft editor as a library: the page editor
//! ([`editor`]: schematics, forms, graphics; tools, selection, snapping) and the panel
//! layout ([`panel_ui`]: mounting plan, floor and site plan, the 3D view with the
//! routed wiring). A view takes the project and its analysis and draws into an
//! `egui::Ui`; the application around it (the editor, the website) owns both. Keys,
//! mouse gestures and colours come from `openecraft-shell`. [`jobs`] runs work that
//! would hold up the window on another thread.

#![cfg_attr(not(test), warn(clippy::too_many_lines))]

pub mod editor;
pub mod jobs;
pub mod panel_ui;

use openecraft_shell::{commands, gestures, shortcuts, style};
