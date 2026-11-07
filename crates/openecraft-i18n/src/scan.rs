//! Source scan of the workspace: the English texts passed to `tr` / `tr_fmt!` /
//! `tr_const!` / `issue!` and the names of the built-in symbols, each with the catalog
//! area of the crate it appears in. Shared by the catalog tests and the tidy tool
//! (`examples/tidy.rs`), which sorts the catalog files and moves every text to the file
//! of its area.
//!
//! An area is a group of crates (see [`area_of_member`]); a text used by crates of two
//! areas belongs to `common`. Texts the scan cannot see (passed to `tr` through a
//! variable: symbol names of tables, product groups, titles of enumerations) stay in
//! the file they are in.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub use crate::catalog::AREAS;

/// The area of a workspace member (its path in `[workspace] members`).
pub fn area_of_member(member: &str) -> &'static str {
    match member.trim_matches('/') {
        "openecraft" => "editor",
        "crates/openecraft-core" => "core",
        "crates/openecraft-api" => "plugins",
        "cli" => "cli",
        "crates/openecraft-ui" => "ui",
        "crates/openecraft-i18n" => "common",
        _ => "parts",
    }
}

/// The area a text belongs to by the areas of its uses: the only one, `common` for
/// several, none for a text no scanned source uses.
pub fn area_for(areas: &BTreeSet<&'static str>) -> Option<&'static str> {
    match areas.len() {
        0 => None,
        1 => areas.iter().next().copied(),
        _ => Some("common"),
    }
}

/// The root of the workspace (two levels above this crate; run through cargo).
pub fn workspace_root() -> PathBuf {
    Path::new(&std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo")).join("../..")
}

/// The paths of `[workspace] members` of the root manifest.
pub fn members(root: &Path) -> Vec<String> {
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("root Cargo.toml");
    let list = manifest.split("members = [").nth(1).and_then(|r| r.split(']').next()).expect("workspace members");
    list.split(',').map(|m| m.trim().trim_matches('"').to_string()).filter(|m| !m.is_empty()).collect()
}

/// The source files of a crate (`src/**/*.rs` without the test files).
pub fn source_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            source_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") && path.file_name().is_some_and(|n| n != "tests.rs") {
            out.push(path);
        }
    }
}

/// Whether the file defines built-in symbols (`symbols.rs` and the files of `symbols/`).
pub fn is_symbol_file(path: &Path) -> bool {
    path.file_name().is_some_and(|n| n == "symbols.rs") || path.parent().and_then(Path::file_name).is_some_and(|n| n == "symbols")
}

/// Resolves the escapes of a Rust string literal body (`\n`, `\t`, `\"`, `\\`).
pub fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

/// The string literal that starts right after `from` in `src` (raw body, no unescaping)
/// and the position after its closing quote.
fn literal_after(src: &str, from: usize) -> Option<(&str, usize)> {
    let rest = &src[from..];
    let start = rest.find('"')?;
    let mut end = start + 1;
    let bytes = rest.as_bytes();
    while end < bytes.len() && bytes[end] != b'"' {
        end += if bytes[end] == b'\\' { 2 } else { 1 };
    }
    Some((&rest[start + 1..end], from + end + 1))
}

/// All texts passed to `tr("…")`, `tr_fmt!("…"`, `tr_const!("…")`, the templates of
/// `issue!(severity, target, "…")` and, in a symbol file, the names of the built-in
/// symbols.
pub fn texts_in_source(src: &str, is_symbols: bool) -> Vec<String> {
    let mut out = Vec::new();
    // the template of an issue follows the severity and the target
    let mut at = 0;
    while let Some(i) = src[at..].find("issue!(") {
        at += i + "issue!(".len();
        if let Some((text, _)) = literal_after(src, at) {
            out.push(unescape(text));
        }
    }
    for marker in ["tr(", "tr_fmt!(", "tr_const!("] {
        let mut at = 0;
        while let Some(i) = src[at..].find(marker) {
            let pos = at + i;
            at = pos + marker.len();
            let prev = src[..pos].chars().last();
            if prev.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '!') {
                continue;
            }
            // only a literal directly after the parenthesis (tr(some_variable) is skipped)
            if !src[at..].trim_start().starts_with('"') {
                continue;
            }
            if let Some((text, _)) = literal_after(src, at) {
                out.push(unescape(text));
            }
        }
    }
    if is_symbols {
        // def("key", "Name", …) and the potential table ("pot_x", "Name", "label")
        for marker in ["def(\"", "(\"pot"] {
            let mut at = 0;
            while let Some(i) = src[at..].find(marker) {
                let pos = at + i + marker.find('"').unwrap();
                at = pos + 1;
                if let Some((_, after_key)) = literal_after(src, pos) {
                    if let Some((name, _)) = literal_after(src, after_key) {
                        out.push(unescape(name));
                    }
                }
            }
        }
    }
    out
}

/// A text the scan found, with the areas of the crates using it and one file of each.
#[derive(Default, Debug)]
pub struct Use {
    pub areas: BTreeSet<&'static str>,
    pub files: Vec<PathBuf>,
}

/// Every text the sources of the workspace translate, by text.
pub fn used_texts(root: &Path) -> BTreeMap<String, Use> {
    let mut out: BTreeMap<String, Use> = BTreeMap::new();
    for member in members(root) {
        let area = area_of_member(&member);
        let mut files = Vec::new();
        source_files(&root.join(&member).join("src"), &mut files);
        for file in files {
            // this module names the markers it looks for
            if area == "common" && file.ends_with("scan.rs") {
                continue;
            }
            let src = std::fs::read_to_string(&file).unwrap();
            for text in texts_in_source(&src, is_symbol_file(&file)) {
                let entry = out.entry(text).or_default();
                if entry.areas.insert(area) {
                    entry.files.push(file.clone());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
