//! Writes a static page in every language of the catalog beside its English
//! `index.html` ([`openecraft_i18n::page`]): `de.html`, `fr.html`, … The website has
//! them for search engines, which read a page without running its script
//! (`docs/website.md`).
//!
//! `cargo run --release -p openecraft-i18n --example pages -- <folder>`; as a hook of
//! trunk (no argument) the folder is the page being built.

use openecraft_i18n::page::{file_name, translated};
use openecraft_i18n::Lang;
use std::path::PathBuf;

fn main() {
    let Some(dir) = std::env::args().nth(1).or_else(|| std::env::var("TRUNK_STAGING_DIR").ok()).map(PathBuf::from) else {
        fail("usage: pages <folder>");
    };
    let index = dir.join(file_name(Lang::En));
    let html = std::fs::read_to_string(&index).unwrap_or_else(|e| fail(&format!("{}: {e}", index.display())));
    for lang in Lang::ALL.into_iter().filter(|l| *l != Lang::En) {
        let file = dir.join(file_name(lang));
        if let Err(e) = std::fs::write(&file, translated(&html, lang)) {
            fail(&format!("{}: {e}", file.display()));
        }
    }
    println!("{} pages in {}", Lang::ALL.len() - 1, dir.display());
}

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1)
}
