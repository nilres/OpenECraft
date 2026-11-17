//! Writes the catalog of every further language as one file, `<code>.tsv`
//! ([`openecraft_i18n::catalog_file`]): a web page built without the catalogs
//! (feature `lazy`) has them beside it and fetches the one of the language in use
//! (`docs/installation.md`).
//!
//! `cargo run --release -p openecraft-i18n --example catalogs -- <folder>`; as a hook
//! of trunk (no argument) the folder is `i18n` in the page being built.

use openecraft_i18n::{catalog_file, Lang};
use std::path::{Path, PathBuf};

fn main() {
    let dir = match (std::env::args().nth(1), std::env::var("TRUNK_STAGING_DIR")) {
        (Some(dir), _) => PathBuf::from(dir),
        (None, Ok(staging)) => Path::new(&staging).join("i18n"),
        (None, Err(_)) => fail("usage: catalogs <folder>"),
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        fail(&format!("{}: {e}", dir.display()));
    }
    let further: Vec<Lang> = Lang::ALL.into_iter().filter(|l| !l.is_required()).collect();
    for lang in &further {
        let file = dir.join(format!("{}.tsv", lang.code()));
        if let Err(e) = std::fs::write(&file, catalog_file(*lang)) {
            fail(&format!("{}: {e}", file.display()));
        }
    }
    println!("{} catalogs in {}", further.len(), dir.display());
}

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1)
}
