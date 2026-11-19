//! Writes the generated pages of the website into the page trunk built
//! (`openecraft_site`, `docs/website.md`): the symbol library, the example projects,
//! the documentation, the release notes with their feed, the pages of the content
//! files (`website/content`) and `build.js`.
//!
//! `cargo run --release -p openecraft-site --example site -- <folder>`; as a hook of
//! trunk (no argument) the folder is the page being built.

use openecraft_site::frame::File;
use openecraft_site::{build, content, docs, examples, news, symbols};
use std::path::{Path, PathBuf};

fn main() {
    let Some(dir) = std::env::args().nth(1).or_else(|| std::env::var("TRUNK_STAGING_DIR").ok()).map(PathBuf::from) else {
        fail("usage: site <folder>");
    };
    // The repository this was built from: the content files, the documents, the changelog.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let website = root.join("website");
    let read = |path: PathBuf| std::fs::read_to_string(&path).unwrap_or_else(|e| fail(&format!("{}: {e}", path.display())));

    let mut files: Vec<File> = Vec::new();
    files.extend(content::site(&content::read(&website.join("content")).unwrap_or_else(|e| fail(&e))));
    files.extend(symbols::site());
    files.extend(examples::site());
    let webp: Vec<String> = std::fs::read_dir(website.join("images"))
        .map(|entries| entries.filter_map(Result::ok).filter_map(|e| e.file_name().to_str().and_then(|n| n.strip_suffix(".webp")).map(String::from)).collect())
        .unwrap_or_default();
    files.extend(docs::site(&root, &webp).unwrap_or_else(|e| fail(&e)));
    // The address of the repository for the feed: the one the page links (config.js).
    let config = std::fs::read_to_string(dir.join("config.js")).unwrap_or_default();
    let repository = config.split("repository: \"").nth(1).and_then(|rest| rest.split('"').next()).unwrap_or_default();
    files.extend(news::site(&read(root.join("CHANGELOG.md")), repository));

    // The recording of the first screen, when the site has one.
    let [tour, poster] = build::TOUR.map(|name| std::fs::read(website.join(name)).ok().map(|bytes| File { path: name.to_string(), bytes }));
    let index = std::fs::read_to_string(dir.join("index.html")).unwrap_or_default();
    let [stars, contributors] = build::NUMBERS.map(|name| build::number(std::env::var(name).ok().as_deref()));
    files.push(File::text(build::FILE, build::script(build::module(&index).as_ref(), tour.is_some(), poster.is_some())));
    files.push(File::text(build::NUMBERS_FILE, build::numbers(stars, contributors)));
    files.extend(tour.into_iter().chain(poster.filter(|_| files.iter().any(|f| f.path == build::TOUR[0]))));

    for file in &files {
        let path = dir.join(&file.path);
        let written = path.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|()| std::fs::write(&path, &file.bytes));
        if let Err(e) = written {
            fail(&format!("{}: {e}", path.display()));
        }
    }
    println!("{} files of the site in {}", files.len(), dir.display());
}

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1)
}
