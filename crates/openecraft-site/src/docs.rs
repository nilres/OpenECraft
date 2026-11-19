//! The documentation of the repository as pages of the site: the tutorial, the
//! installation guide, the command line, the plugins, the servers and the feature
//! documents, rendered from their Markdown. In English, as they are written. Links
//! between the documents lead to their pages, links to other files to the repository.

use crate::frame::{escape, files, hero, shortened, File, Kind, Langs, Page, Place, DESCRIPTION};
use crate::markdown::{render, resolved, Rendered, Target};
use openecraft_i18n::Lang;
use std::path::Path;

/// The folder of the pages below the site.
pub const FOLDER: &str = "docs";

/// The documents on the site, by their file in the repository, in the order of the
/// overview. The folder of each is the name of its file.
pub const DOCUMENTS: [&str; 22] = [
    "docs/tutorial.md",
    "docs/installation.md",
    "docs/user-interface.md",
    "docs/features/schematic.md",
    "docs/features/symbols.md",
    "docs/features/connections.md",
    "docs/features/terminals-plugs-plc.md",
    "docs/features/macros.md",
    "docs/features/parts.md",
    "docs/features/reports-and-checks.md",
    "docs/features/calculations.md",
    "docs/features/panel-layout.md",
    "docs/features/workshop.md",
    "docs/features/fluid-and-process.md",
    "docs/features/harnesses.md",
    "docs/features/data-exchange.md",
    "docs/features/revisions-and-review.md",
    "docs/command-line.md",
    "docs/plugins.md",
    "docs/parts-server.md",
    "docs/project-server.md",
    "docs/access-control.md",
];

/// The folder of a document below [`FOLDER`]: the name of its file.
pub fn slug(file: &str) -> &str {
    file.rsplit('/').next().unwrap_or(file).trim_end_matches(".md")
}

/// Where the link `address` of the file `from` of the repository leads, seen from a
/// page whose way to the site is `root`. `webp` names the screenshots the site has as
/// small pictures (`website/images/<name>.webp`).
pub fn target(from: &str, address: &str, root: &str, webp: &[String]) -> Target {
    if address.starts_with('#') || address.contains("://") || address.starts_with("mailto:") {
        return Target::Address(address.to_string());
    }
    let (path, section) = address.split_once('#').map_or((address, String::new()), |(path, section)| (path, format!("#{section}")));
    let file = resolved(from, path);
    if DOCUMENTS.contains(&file.as_str()) {
        return Target::Address(format!("{root}{FOLDER}/{}/{section}", slug(&file)));
    }
    if file == "CHANGELOG.md" {
        return Target::Address(format!("{root}news/"));
    }
    if let Some(name) = file.strip_prefix("docs/images/") {
        let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
        return Target::Address(if webp.iter().any(|w| w == stem) { format!("{root}images/{stem}.webp") } else { format!("{root}{FOLDER}/images/{name}") });
    }
    Target::Repository(format!("{file}{section}"))
}

/// The list of the documents: every page has it below its text, the overview as cards.
fn list(documents: &[(&str, Rendered)], place: Place, current: Option<&str>) -> String {
    let mut out = String::from("    <div class=\"cards\">\n");
    for (file, document) in documents.iter().filter(|(file, _)| Some(*file) != current) {
        out.push_str(&format!(
            "        <article class=\"card\"><h3>{}</h3><p>{}</p><a class=\"button\" href=\"{}\">Read</a></article>\n",
            escape(&document.title),
            escape(&shortened(&document.summary, 220)),
            place.file(&format!("{FOLDER}/{}/", slug(file)))
        ));
    }
    out.push_str("    </div>\n");
    out
}

fn description(document: &Rendered) -> String {
    let text = shortened(&document.summary, DESCRIPTION.end - 1);
    if text.chars().count() >= DESCRIPTION.start {
        return text;
    }
    shortened(&format!("{}: {text} From the documentation of OpenECraft, the free open source electrical CAE.", document.title), DESCRIPTION.end - 1)
}

/// The pages of the documentation, with the pictures the site has no small copy of.
/// `root` is the repository.
pub fn site(root: &Path, webp: &[String]) -> Result<Vec<File>, String> {
    let read = |file: &str| std::fs::read_to_string(root.join(file)).map(|text| text.replace("\r\n", "\n")).map_err(|e| format!("{file}: {e}"));
    let page_root = Place::of(&format!("{FOLDER}/x"), Lang::En).root();
    let mut documents: Vec<(&str, Rendered)> = Vec::new();
    for file in DOCUMENTS {
        documents.push((file, render(&read(file)?, &|address| target(file, address, &page_root, webp))));
    }
    let mut out = Vec::new();
    for (file, document) in &documents {
        let folder = format!("{FOLDER}/{}", slug(file));
        let place = Place::of(&folder, Lang::En);
        let source = format!("<a data-link=\"repository\" data-path=\"/blob/master/{file}\" href=\"#\">This document in the repository</a>");
        let main = format!(
            "<article class=\"band doc\">\n    <p class=\"eyebrow\"><a href=\"{}\">Documentation</a></p>\n{}    <p class=\"fine\">{source}</p>\n</article>\n\n<section class=\"band\">\n    <h2>More documentation</h2>\n{}</section>\n",
            place.file(&format!("{FOLDER}/")),
            document.html,
            list(&documents, place, Some(file))
        );
        let page = Page { folder, title: crate::frame::title(format!("{} – OpenECraft documentation", document.title), &document.title), description: description(document), main, kind: Kind::Plain, langs: Langs::English };
        out.extend(files(Langs::English, |_| page.clone()));
        // A screenshot without a small copy comes along as it is.
        for picture in &document.pictures {
            if let Some(name) = picture.strip_prefix(&format!("{page_root}{FOLDER}/images/")) {
                let bytes = std::fs::read(root.join("docs/images").join(name)).map_err(|e| format!("docs/images/{name}: {e}"))?;
                out.push(File { path: format!("{FOLDER}/images/{name}"), bytes });
            }
        }
    }
    let place = Place::of(FOLDER, Lang::En);
    let actions = "<a class=\"button\" data-link=\"documentation\" href=\"#\">All documents in the repository</a>";
    let mut main = hero(
        "Documentation",
        "How OpenECraft works",
        "The tutorial from the first schematic page to the bill of materials, the installation guide, every feature in detail, and the command line, the MCP server and the plugins for those who automate.",
        actions,
    );
    main.push_str(&format!("\n<section class=\"band\">\n    <h2>Documents</h2>\n{}</section>\n", list(&documents, place, None)));
    let overview = Page {
        folder: FOLDER.to_string(),
        title: "Documentation – OpenECraft".to_string(),
        description: "The documentation of OpenECraft, the open source electrical CAE: tutorial, installation, schematics, panel layout, reports, command line, MCP server and plugins.".to_string(),
        main,
        kind: Kind::Plain,
        langs: Langs::English,
    };
    out.extend(files(Langs::English, |_| overview.clone()));
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out.dedup_by(|a, b| a.path == b.path);
    Ok(out)
}

#[cfg(test)]
mod tests;
