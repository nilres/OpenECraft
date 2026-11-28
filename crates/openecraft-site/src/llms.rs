//! `llms.txt` beside the main page: what OpenECraft is and where the site says what,
//! as Markdown for the language models that answer questions about software. It
//! lists the English pages by their titles and descriptions; the pages of the single
//! symbols and of the single releases are reached from their overviews. The links
//! start at the folder of the site (`./docs/tutorial/`); `website-site.sh` writes them
//! whole where the public address is known.

use crate::frame::BAR;

/// The file below the site.
pub const FILE: &str = "llms.txt";

/// A page of the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Its folder below the site with the slash after it; empty for the main page.
    pub folder: String,
    pub title: String,
    pub description: String,
}

/// The headings of the list with the folders of their pages, in their order. A page
/// of a folder no heading names is left out.
const GROUPS: [(&str, &[&str]); 8] = [
    ("Who it is for", &["panel-builders", "electricians", "education"]),
    ("Documentation", &["docs", "faq"]),
    ("Guides", &["how-to"]),
    ("Tools that run in the browser", &["tools"]),
    ("Reference", &["reference"]),
    ("Comparisons", &["compare"]),
    ("Symbol library", &["symbols"]),
    ("Example projects", &["examples"]),
];

/// The pages of the end of the list, which a reader short of room may skip.
const OPTIONAL: [&str; 2] = ["news", "videos"];

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&")
}

/// The entry of the English page `file` (`docs/tutorial/index.html`) with the content
/// `html`; none for the page of another language and for a page the list leaves out.
pub fn entry(file: &str, html: &str) -> Option<Entry> {
    let folder = file.strip_suffix("index.html")?;
    let mut parts = folder.split('/').filter(|part| !part.is_empty());
    let (first, below) = (parts.next(), parts.next());
    let listed = match (first, below) {
        (Some("symbols"), Some(below)) => crate::symbols::is_category(below),
        (Some("news"), Some(_)) => false,
        _ => parts.next().is_none(),
    };
    let title = html.split_once("<title")?.1.split_once('>')?.1.split("</title>").next()?;
    let description = html.split_once("<meta name=\"description\" content=\"")?.1.split('"').next()?;
    // The name of the project is the heading of the file.
    let title = unescape(title);
    let title = [" – OpenECraft documentation", " – OpenECraft", " | OpenECraft"].iter().find_map(|end| title.strip_suffix(end)).unwrap_or(&title).trim().to_string();
    (listed && !title.is_empty()).then(|| Entry { folder: folder.to_string(), title, description: unescape(description) })
}

fn line(entry: &Entry) -> String {
    format!("- [{}](./{}): {}\n", entry.title.replace(['[', ']'], ""), entry.folder, entry.description)
}

/// The file: `entries` are the pages of the site, `repository` is the address of the
/// source code (empty: not named).
pub fn text(entries: &[Entry], repository: &str) -> String {
    let top = |entry: &Entry| entry.folder.split('/').next().unwrap_or_default().to_string();
    // The overview of a folder comes before the pages below it.
    let of = |folders: &[&str]| -> Vec<&Entry> {
        let below = |folder: &&str| {
            let mut pages: Vec<&Entry> = entries.iter().filter(|e| top(e) == **folder).collect();
            pages.sort_by_key(|e| e.folder.matches('/').count() > 1);
            pages
        };
        folders.iter().flat_map(below).collect()
    };
    let mut out = String::from("# OpenECraft\n\n");
    if let Some(main) = entries.iter().find(|e| e.folder.is_empty()) {
        out.push_str(&format!("> {}\n\n", main.description));
    }
    out.push_str(
        "OpenECraft is free and open source (MIT or Apache-2.0). It runs on Windows, Linux and macOS and, as a web editor, in the browser with nothing to install. \
         The main page of this site runs the editor's own code as live demos. Every page listed here is written in English; most exist in further languages as files beside it (`de.html`, `fr.html`, `es.html`, ...).\n\n",
    );
    out.push_str("## Start here\n\n- [Main page with live demos](./): schematic editor, panel layout in 3D, project check, reports, console and exports, all in the browser\n");
    if let Some(compare) = entries.iter().find(|e| e.folder == "compare/") {
        out.push_str(&line(compare));
    }
    for (heading, folders) in GROUPS {
        let pages: Vec<&Entry> = of(folders).into_iter().filter(|e| e.folder != "compare/").collect();
        if !pages.is_empty() {
            out.push_str(&format!("\n## {heading}\n\n"));
            pages.into_iter().for_each(|e| out.push_str(&line(e)));
        }
    }
    let optional = of(&OPTIONAL);
    if !optional.is_empty() || !repository.is_empty() {
        out.push_str("\n## Optional\n\n");
        optional.into_iter().for_each(|e| out.push_str(&line(e)));
        if !repository.is_empty() {
            out.push_str(&format!("- [Source code]({repository}): the repository with the code, the issues and the releases to download\n"));
        }
    }
    out
}

/// The folders of the pages written by hand, which the list reads from the built page.
pub fn written_by_hand() -> Vec<&'static str> {
    BAR.iter().map(|(target, _)| *target).filter(|target| !target.starts_with('#')).collect()
}

#[cfg(test)]
mod tests;
