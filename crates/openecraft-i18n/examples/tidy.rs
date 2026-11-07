//! Tidies the translation catalog: sorts every file by its English text and moves each
//! text to the folder of the area that shows it (the crate group found by the source
//! scan, `common` for several; texts the scan cannot see stay where they are). Run it
//! after adding texts anywhere in the catalog:
//!
//!     cargo run -p openecraft-i18n --example tidy [--unused]
//!
//! `--unused` lists the texts no scanned source passes to `tr` (symbol names of tables,
//! product groups and other texts reaching `tr` through a variable are among them).

use openecraft_i18n::scan::{self, AREAS};
use openecraft_i18n::Lang;
use std::collections::BTreeMap;
use std::path::Path;

/// What each area holds, for the file headers.
const DESCRIPTIONS: [(&str, &str); AREAS.len()] = [
    ("common", "texts shown by several areas"),
    ("editor", "the editor (openecraft/)"),
    ("core", "openecraft-core: project model, checks, reports, exports"),
    ("plugins", "openecraft-api: the built-in plugins and their commands"),
    ("cli", "the command line and MCP server"),
    ("ui", "openecraft-ui: shared widgets and dialogs"),
    ("parts", "parts library, parts server, parts manager and clients"),
];

/// The entry lines of a catalog file (comments and blank lines skipped); a missing
/// file is empty.
fn read(path: &Path, eol: &mut Option<&'static str>) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
    if eol.is_none() {
        *eol = Some(if text.contains("\r\n") { "\r\n" } else { "\n" });
    }
    text.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')).map(str::to_string).collect()
}

fn header(area: &str, lang: Option<Lang>) -> String {
    let what = DESCRIPTIONS.iter().find(|(a, _)| *a == area).map(|(_, d)| *d).unwrap_or(area);
    match lang {
        None => format!("# English<TAB>Deutsch<TAB>Français<TAB>Español – {what}. Sorted by the English text: cargo run -p openecraft-i18n --example tidy"),
        Some(l) => format!(
            "# English<TAB>{} – {what} (texts without a line are shown in English). Sorted by the English text: cargo run -p openecraft-i18n --example tidy",
            l.native_name()
        ),
    }
}

/// Writes the header and the entries sorted by their raw English text, if that differs
/// from the file.
fn write(path: &Path, header: String, mut entries: Vec<String>, eol: &str, written: &mut usize) {
    entries.sort_by(|a, b| a.split('\t').next().cmp(&b.split('\t').next()));
    let mut text = header;
    for line in entries {
        text.push_str(eol);
        text.push_str(&line);
    }
    text.push_str(eol);
    if std::fs::read_to_string(path).ok().as_deref() != Some(text.as_str()) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
        *written += 1;
    }
}

fn main() {
    let list_unused = std::env::args().any(|a| a == "--unused");
    let root = scan::workspace_root();
    let dir = root.join("crates/openecraft-i18n/src/catalog");
    let used = scan::used_texts(&root);
    let further_langs: Vec<Lang> = Lang::ALL.into_iter().filter(|l| !l.is_required()).collect();
    let mut eol = None;
    let mut errors = Vec::new();

    // Every entry with the area of the file it is in, by its (unescaped) English text.
    let mut required: BTreeMap<String, (&str, String)> = BTreeMap::new();
    let mut further: BTreeMap<&str, BTreeMap<String, String>> = BTreeMap::new();
    for area in AREAS {
        for line in read(&dir.join(area).join("required.tsv"), &mut eol) {
            if line.split('\t').count() != 4 {
                errors.push(format!("{area}/required.tsv: not 4 tab separated columns: {line}"));
                continue;
            }
            let key = scan::unescape(line.split('\t').next().unwrap());
            match required.get(&key) {
                None => {
                    required.insert(key, (area, line));
                }
                Some((other, same)) if *same != line => errors.push(format!("{area}/required.tsv: {key:?} differs from {other}/required.tsv")),
                Some(_) => {}
            }
        }
        for lang in &further_langs {
            for line in read(&dir.join(area).join(format!("{}.tsv", lang.code())), &mut eol) {
                if line.split('\t').count() != 2 {
                    errors.push(format!("{area}/{}.tsv: not 2 tab separated columns: {line}", lang.code()));
                    continue;
                }
                let key = scan::unescape(line.split('\t').next().unwrap());
                let entries = further.entry(lang.code()).or_default();
                match entries.get(&key) {
                    None => {
                        entries.insert(key, line);
                    }
                    Some(same) if *same != line => errors.push(format!("{area}/{}.tsv: {key:?} is translated twice, differently", lang.code())),
                    Some(_) => {}
                }
            }
        }
    }

    // The target area of every text.
    let mut placed: BTreeMap<&str, Vec<String>> = AREAS.iter().map(|a| (*a, Vec::new())).collect();
    let mut target: BTreeMap<&str, &str> = BTreeMap::new();
    let mut moved = 0;
    let mut unused = Vec::new();
    for (key, (area, line)) in &required {
        let to = match used.get(key).and_then(|u| scan::area_for(&u.areas)) {
            Some(to) => to,
            None => {
                unused.push(key.as_str());
                area
            }
        };
        if to != *area {
            moved += 1;
        }
        target.insert(key, to);
        placed.get_mut(to).unwrap().push(line.clone());
    }
    let mut placed_further: BTreeMap<(&str, &str), Vec<String>> = BTreeMap::new();
    for (code, entries) in &further {
        for (key, line) in entries {
            match target.get(key.as_str()) {
                Some(to) => placed_further.entry((*code, to)).or_default().push(line.clone()),
                None => errors.push(format!("{code}.tsv: not in the main catalog: {key:?}")),
            }
        }
    }
    if !errors.is_empty() {
        eprintln!("{}", errors.join("\n"));
        eprintln!("{} errors, nothing written", errors.len());
        std::process::exit(1);
    }

    let eol = eol.unwrap_or("\n");
    let mut written = 0;
    for area in AREAS {
        write(&dir.join(area).join("required.tsv"), header(area, None), placed.remove(area).unwrap(), eol, &mut written);
        for lang in &further_langs {
            let entries = placed_further.remove(&(lang.code(), area)).unwrap_or_default();
            write(&dir.join(area).join(format!("{}.tsv", lang.code())), header(area, Some(*lang)), entries, eol, &mut written);
        }
    }
    println!("{} texts in {} areas: {moved} moved, {written} files written", required.len(), AREAS.len());
    println!("{} texts no scanned source passes to tr (symbol names, product groups, …){}", unused.len(), if list_unused { ":" } else { "; --unused lists them" });
    if list_unused {
        for key in unused {
            println!("{key:?}");
        }
    }
}
