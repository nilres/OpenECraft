//! The documentation of the repository as pages of the site: the tutorial, the
//! installation guide, the command line, the plugins, the servers and the feature
//! documents, rendered from their Markdown. In English, as they are written, and in
//! German, French and Spanish from their translations (`docs/de/…`, `docs/fr/…`,
//! `docs/es/…`, the same files one folder deeper). Links between the documents lead
//! to their pages in the language of the document, links to other files to the
//! repository.

use crate::frame::{escape, files, hero, shortened, File, Kind, Langs, Page, Place, DESCRIPTION};
use crate::markdown::{render, resolved, Rendered, Target};
use openecraft_i18n::page::file_name;
use openecraft_i18n::{tr, tr_fmt, Lang};
use std::collections::BTreeMap;
use std::path::Path;

/// The folder of the pages below the site.
pub const FOLDER: &str = "docs";

/// The languages the documents are written in.
pub const LANGS: Langs = Langs::Required;

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

/// The file of a document in `lang`: `docs/tutorial.md` in German is
/// `docs/de/tutorial.md`.
pub fn source(file: &str, lang: Lang) -> String {
    match lang {
        Lang::En => file.to_string(),
        lang => format!("docs/{}/{}", lang.code(), file.trim_start_matches("docs/")),
    }
}

/// The language of a file of the documentation and its file in English.
fn original(file: &str) -> (Lang, String) {
    for lang in LANGS.list().into_iter().filter(|lang| *lang != Lang::En) {
        if let Some(rest) = file.strip_prefix(&format!("docs/{}/", lang.code())) {
            return (lang, format!("docs/{rest}"));
        }
    }
    (Lang::En, file.to_string())
}

/// Where the link `address` of the file `from` of the repository leads, seen from a
/// page whose way to the site is `root`. A translated document (`docs/de/…`) links
/// the pages of its language. `webp` names the screenshots the site has as small
/// pictures (`website/images/<name>.webp`).
pub fn target(from: &str, address: &str, root: &str, webp: &[String]) -> Target {
    if address.starts_with('#') || address.contains("://") || address.starts_with("mailto:") {
        return Target::Address(address.to_string());
    }
    let (path, section) = address.split_once('#').map_or((address, String::new()), |(path, section)| (path, format!("#{section}")));
    let (lang, _) = original(from);
    // A translation names the other documents as the original does, beside itself.
    let file = match original(&resolved(from, path)) {
        (of, file) if of == lang || of == Lang::En => file,
        (_, _) => resolved(from, path),
    };
    if DOCUMENTS.contains(&file.as_str()) {
        let page = if lang == Lang::En { String::new() } else { file_name(lang) };
        return Target::Address(format!("{root}{FOLDER}/{}/{page}{section}", slug(&file)));
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
            "        <article class=\"card\"><h3>{}</h3><p>{}</p><a class=\"button\" href=\"{}\">{}</a></article>\n",
            escape(&document.title),
            escape(&shortened(&document.summary, 220)),
            place.to(&format!("{FOLDER}/{}/", slug(file))),
            escape(tr("Read"))
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
    shortened(&tr_fmt!("{}: {} From the documentation of OpenECraft, the free open source electrical CAE.", document.title, text), DESCRIPTION.end - 1)
}

/// The page of the document `file` in the language of `place`.
fn document_page(file: &str, document: &Rendered, documents: &[(&str, Rendered)], lang: Lang) -> Page {
    let folder = format!("{FOLDER}/{}", slug(file));
    let place = Place::of(&folder, lang);
    let source = format!("<a data-link=\"repository\" data-path=\"/blob/master/{}\" href=\"#\">{}</a>", source(file, lang), escape(tr("This document in the repository")));
    let main = format!(
        "<article class=\"band doc\">\n    <p class=\"eyebrow\"><a href=\"{}\">{}</a></p>\n{}    <p class=\"fine\">{source}</p>\n</article>\n\n<section class=\"band\">\n    <h2>{}</h2>\n{}</section>\n",
        place.to(&format!("{FOLDER}/")),
        escape(tr("Documentation")),
        document.html,
        escape(tr("More documentation")),
        list(documents, place, Some(file))
    );
    Page { folder, title: crate::frame::title(tr_fmt!("{} – OpenECraft documentation", document.title), &document.title), description: description(document), main, kind: Kind::Plain, langs: LANGS }
}

/// The overview of the documents in the language of the page.
fn overview(documents: &[(&str, Rendered)], lang: Lang) -> Page {
    let place = Place::of(FOLDER, lang);
    let actions = format!("<a class=\"button\" data-link=\"documentation\" href=\"#\">{}</a>", escape(tr("All documents in the repository")));
    let mut main = hero(
        tr("Documentation"),
        tr("How OpenECraft works"),
        tr("The tutorial from the first schematic page to the bill of materials, the installation guide, every feature in detail, and the command line, the MCP server and the plugins for those who automate."),
        &actions,
    );
    main.push_str(&format!("\n<section class=\"band\">\n    <h2>{}</h2>\n{}</section>\n", escape(tr("Documents")), list(documents, place, None)));
    Page {
        folder: FOLDER.to_string(),
        title: tr("Documentation – OpenECraft").to_string(),
        description: tr("The documentation of OpenECraft, the open source electrical CAE: tutorial, installation, schematics, panel layout, reports, command line, MCP server and plugins.").to_string(),
        main,
        kind: Kind::Plain,
        langs: LANGS,
    }
}

/// The pages of the documentation in its languages, with the pictures the site has no
/// small copy of. `root` is the repository.
pub fn site(root: &Path, webp: &[String]) -> Result<Vec<File>, String> {
    let read = |file: &str| std::fs::read_to_string(root.join(file)).map(|text| text.replace("\r\n", "\n")).map_err(|e| format!("{file}: {e}"));
    let page_root = Place::of(&format!("{FOLDER}/x"), Lang::En).root();
    let mut documents: BTreeMap<&str, Vec<(&str, Rendered)>> = BTreeMap::new();
    for lang in LANGS.list() {
        for file in DOCUMENTS {
            let from = source(file, lang);
            documents.entry(lang.code()).or_default().push((file, render(&read(&from)?, &|address| target(&from, address, &page_root, webp))));
        }
    }
    let of = |lang: Lang| documents[lang.code()].as_slice();
    let mut out = Vec::new();
    for (i, file) in DOCUMENTS.into_iter().enumerate() {
        out.extend(files(LANGS, |lang| document_page(file, &of(lang)[i].1, of(lang), lang)));
        // A screenshot without a small copy comes along as it is.
        for picture in LANGS.list().into_iter().flat_map(|lang| &of(lang)[i].1.pictures) {
            if let Some(name) = picture.strip_prefix(&format!("{page_root}{FOLDER}/images/")) {
                let bytes = std::fs::read(root.join("docs/images").join(name)).map_err(|e| format!("docs/images/{name}: {e}"))?;
                out.push(File { path: format!("{FOLDER}/images/{name}"), bytes });
            }
        }
    }
    out.extend(files(LANGS, |lang| overview(of(lang), lang)));
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out.dedup_by(|a, b| a.path == b.path);
    Ok(out)
}

#[cfg(test)]
mod tests;
