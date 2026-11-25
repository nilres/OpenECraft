//! Folders of the user on the desktop: the data folder of an app, where eframe keeps
//! its storage, and the folder of the user's project templates, which the editor and
//! the command line share.

use std::path::PathBuf;

/// Environment variable naming the folder of the project templates.
pub const TEMPLATES_VAR: &str = "OPENECRAFT_TEMPLATES";

fn env(var: &str) -> Option<PathBuf> {
    std::env::var_os(var).filter(|s| !s.is_empty()).map(PathBuf::from)
}

/// Folder for the data of the app `app`, as eframe chooses it for its own storage;
/// `None` without a home folder (in the browser).
pub fn data_dir(app: &str) -> Option<PathBuf> {
    let home = env("HOME").or_else(|| env("USERPROFILE"));
    let base = if cfg!(target_os = "windows") {
        env("APPDATA").or_else(|| home.map(|h| h.join("AppData").join("Roaming")))?
    } else if cfg!(target_os = "macos") {
        home?.join("Library").join("Application Support")
    } else {
        env("XDG_DATA_HOME").or_else(|| home.map(|h| h.join(".local").join("share")))?
    };
    Some(base.join(app).join("data"))
}

/// Folder of the project templates: [`TEMPLATES_VAR`], otherwise `templates` in the
/// data folder of the editor.
pub fn templates_dir() -> Option<PathBuf> {
    templates_dir_from(env(TEMPLATES_VAR), data_dir("OpenECraft"))
}

fn templates_dir_from(var: Option<PathBuf>, data: Option<PathBuf>) -> Option<PathBuf> {
    var.or_else(|| data.map(|d| d.join("templates")))
}

#[cfg(test)]
mod tests;
