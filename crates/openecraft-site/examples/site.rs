//! Writes the generated pages of the website into the page trunk built
//! (`openecraft_site`, `docs/website.md`): the symbol library, the example projects,
//! the documentation, the release notes with their feed, the pages of the content
//! files (`website/content`), the pictures of links to the pages of the symbols and
//! the examples, the page for a missing address, `llms.txt`, `build.js`
//! and, last, when the pages of the site changed (`pages.tsv`).
//!
//! `cargo run --release -p openecraft-site --example site -- <folder>`; as a hook of
//! trunk (no argument) the folder is the page being built.

use openecraft_site::frame::File;
use openecraft_site::{build, changes, content, docs, examples, llms, missing, news, preview, symbols};
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
    // The pictures of links to the pages of the symbols and the examples.
    files.extend(preview::symbols().unwrap_or_else(|e| fail(&e)));
    for example in &openecraft_core::sample::EXAMPLES {
        files.push(preview::example(example).unwrap_or_else(|e| fail(&e)));
    }
    let webp: Vec<String> = std::fs::read_dir(website.join("images"))
        .map(|entries| entries.filter_map(Result::ok).filter_map(|e| e.file_name().to_str().and_then(|n| n.strip_suffix(".webp")).map(String::from)).collect())
        .unwrap_or_default();
    files.extend(docs::site(&root, &webp).unwrap_or_else(|e| fail(&e)));
    // The address of the repository for the feed: the one the page links (config.js).
    let config = std::fs::read_to_string(dir.join("config.js")).unwrap_or_default();
    let repository = config.split("repository: \"").nth(1).and_then(|rest| rest.split('"').next()).unwrap_or_default();
    files.extend(news::site(&read(root.join("CHANGELOG.md")), repository));
    files.extend(missing::site());

    // The list of the pages for language models: those written here and those trunk
    // copied, the main page and the pages written by hand.
    let copied = std::iter::once("index.html".to_string()).chain(llms::written_by_hand().into_iter().map(|folder| format!("{folder}index.html")));
    let mut entries: Vec<llms::Entry> = copied.filter_map(|file| llms::entry(&file, &std::fs::read_to_string(dir.join(&file)).ok()?)).collect();
    entries.extend(files.iter().filter_map(|file| llms::entry(&file.path, std::str::from_utf8(&file.bytes).ok()?)));
    files.push(File::text(llms::FILE, llms::text(&entries, repository)));

    // The recording of the first screen, when the site has one.
    let [tour, poster] = build::TOUR.map(|name| std::fs::read(website.join(name)).ok().map(|bytes| File { path: name.to_string(), bytes }));
    let index = std::fs::read_to_string(dir.join("index.html")).unwrap_or_default();
    let [stars, contributors] = build::NUMBERS.map(|name| build::number(std::env::var(name).ok().as_deref()));
    files.push(File::text(build::FILE, build::script(build::module(&index).as_ref(), tour.is_some(), poster.is_some())));
    files.push(File::text(build::NUMBERS_FILE, build::numbers(stars, contributors)));
    files.push(File::text(build::VIDEOS_FILE, build::videos()));
    files.extend(tour.into_iter().chain(poster.filter(|_| files.iter().any(|f| f.path == build::TOUR[0]))));

    for file in &files {
        let path = dir.join(&file.path);
        let written = path.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|()| std::fs::write(&path, &file.bytes));
        if let Err(e) = written {
            fail(&format!("{}: {e}", path.display()));
        }
    }
    println!("{} files of the site in {}", files.len(), dir.display());

    // When the pages last changed: every page of the built site against the file of
    // the site that is online (website/pages.tsv, installer/web/pages.sh).
    let mut pages = Vec::new();
    collect(&dir, "", 0, &mut pages);
    let before = std::fs::read_to_string(website.join(changes::FILE)).unwrap_or_default();
    let now = std::env::var("SOURCE_DATE_EPOCH").ok().and_then(|seconds| seconds.trim().parse().ok());
    let now = now.unwrap_or_else(|| std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |since| since.as_secs()));
    let list = changes::file(&pages, &before, &changes::day(now));
    if let Err(e) = std::fs::write(dir.join(changes::FILE), &list) {
        fail(&format!("{}: {e}", changes::FILE));
    }
    println!("{} pages in {}, {} of them changed", pages.len(), changes::FILE, changes::changed(&list).len());
}

/// The pages of the built site in `dir` (`below` it, `depth` folders deep): file and
/// content.
fn collect(dir: &Path, below: &str, depth: usize, pages: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else { continue };
        if path.is_dir() {
            if depth < 2 {
                collect(&path, &format!("{below}{name}/"), depth + 1, pages);
            }
        } else if name.ends_with(".html") {
            let file = format!("{below}{name}");
            if let Some(html) = std::fs::read_to_string(&path).ok().filter(|html| changes::is_page(&file, html)) {
                pages.push((file, html));
            }
        }
    }
}

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1)
}
