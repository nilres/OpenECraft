//! The document state of an OpenECraft application ([`session::Session`]): the open
//! project, its undo history, the analysis (rebuilt after committed changes and
//! language changes, with the checks and reports of the plugins), the device index and
//! the saved state. No UI code; the editor adds its windows, the website its demos.

pub mod session;
