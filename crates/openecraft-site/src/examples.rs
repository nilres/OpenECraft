//! The example projects as pages: what each shows, its first sheets as the SVG export
//! draws them, the list of its pages, its PDF to download and a link that opens it in
//! the live demo. The texts are those of the examples in the language of the page;
//! the sheets and the PDF are drawn once, in English.

use crate::frame::{attribute, escape, files, hero, shortened, File, Kind, Langs, Page, Place, DESCRIPTION};
use openecraft_api::{params, Plugins};
use openecraft_core::analysis::Analysis;
use openecraft_core::model::Project;
use openecraft_core::sample::{Example, EXAMPLES};
use openecraft_core::svg::page_svg_with;
use openecraft_i18n::{tr, tr_fmt, Lang};

/// The folder of the pages below the site.
pub const FOLDER: &str = "examples";

/// At most this many sheets of an example are shown as pictures.
pub const SHEETS: usize = 6;

/// The title of an example without its number in the gallery of the editor
/// ("4 · Conveyor …").
pub fn plain_title(title: &str) -> &str {
    let rest = title.trim_start_matches(|c: char| c.is_ascii_digit());
    match rest.len() < title.len() {
        true => rest.trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '·' | '.' | ':' | '-' | '–')),
        false => title,
    }
}

/// The folder of an example below [`FOLDER`]: its key with dashes.
pub fn slug(example: &Example) -> String {
    example.key.replace('_', "-")
}

/// The address that opens an example in the live demo of the main page.
pub fn demo(example: &Example) -> String {
    format!("?example={}#studio", example.key)
}

/// How many sheets of a project the page shows.
fn sheets(project: &Project) -> usize {
    project.pages.len().min(SHEETS)
}

/// The page of an example in `lang`.
pub fn page(example: &Example, lang: Lang) -> Page {
    let folder = format!("{FOLDER}/{}", slug(example));
    let place = Place::of(&folder, lang);
    let project = (example.build)();
    let title = plain_title(tr(example.title));
    let actions = format!(
        "<a class=\"button primary\" href=\"{}\">{}</a><a class=\"button\" href=\"{}.pdf\" download>{}</a><a class=\"button ghost\" data-link=\"editor\" href=\"#\">{}</a>",
        place.to(&demo(example)),
        escape(tr("Open in the live demo")),
        slug(example),
        escape(tr("Download the PDF")),
        escape(tr("Open the web editor"))
    );
    let mut main = hero(&tr_fmt!("Example project, level {}", example.level), title, tr(example.description), &actions);
    main.push_str(&format!("\n<section class=\"band\">\n    <h2>{}</h2>\n    <p class=\"sub\">{}</p>\n    <div class=\"gallery sheets\">\n", escape(tr("Sheets")), escape(tr("Drawn by the SVG export of OpenECraft, as the project comes."))));
    for (i, sheet) in project.pages.iter().take(sheets(&project)).enumerate() {
        let caption = if sheet.description.is_empty() { sheet.name.clone() } else { format!("{} · {}", sheet.name, sheet.description) };
        main.push_str(&format!(
            "        <figure><a href=\"page-{n}.svg\"><img loading=\"lazy\" src=\"page-{n}.svg\" alt=\"{}\" /></a><figcaption>{}</figcaption></figure>\n",
            attribute(&tr_fmt!("Sheet {} of the example: {}", sheet.name, sheet.description)),
            escape(&caption),
            n = i + 1
        ));
    }
    main.push_str(&format!(
        "    </div>\n</section>\n\n<section class=\"band\">\n    <h2>{}</h2>\n    <div class=\"compare-scroll\">\n    <table class=\"compare pages\">\n        <thead><tr><th scope=\"col\">{}</th><th scope=\"col\">{}</th></tr></thead>\n        <tbody>\n",
        escape(&tr_fmt!("{} pages", project.pages.len())),
        escape(tr("Page")),
        escape(tr("Description"))
    ));
    for sheet in &project.pages {
        main.push_str(&format!("            <tr><th scope=\"row\">{}</th><td>{}</td></tr>\n", escape(&sheet.name), escape(&sheet.description)));
    }
    main.push_str("        </tbody>\n    </table>\n    </div>\n</section>\n\n");
    main.push_str(&others(Some(example), place));
    Page {
        title: crate::frame::title(tr_fmt!("{} – example project | OpenECraft", title), title),
        description: shortened(&format!("{title}: {}", tr(example.description)), DESCRIPTION.end - 1),
        folder,
        main,
        kind: Kind::Plain,
        langs: Langs::All,
    }
}

/// The examples as cards, without `current`.
fn others(current: Option<&Example>, place: Place) -> String {
    let heading = if current.is_some() { tr("More examples") } else { tr("Example projects, simplest first") };
    let mut out = format!("<section class=\"band\">\n    <h2>{}</h2>\n    <div class=\"cards\">\n", escape(heading));
    for example in EXAMPLES.iter().filter(|e| current.is_none_or(|c| c.key != e.key)) {
        out.push_str(&format!(
            "        <article class=\"card\"><h3>{}</h3><p>{}</p><a class=\"button\" href=\"{}\">{}</a></article>\n",
            escape(tr(example.title)),
            escape(tr(example.description)),
            place.to(&format!("{FOLDER}/{}/", slug(example))),
            escape(tr("See the example"))
        ));
    }
    out.push_str("    </div>\n</section>\n");
    out
}

/// The overview of the examples in `lang`.
pub fn overview(lang: Lang) -> Page {
    let place = Place::of(FOLDER, lang);
    let lead = tr_fmt!(
        "{} projects that come with OpenECraft, from a lamp circuit to a whole building. Each has its sheets, its PDF and a link that opens it in the live demo.",
        EXAMPLES.len()
    );
    let actions = format!("<a class=\"button primary\" href=\"{}\">{}</a><a class=\"button\" data-link=\"editor\" href=\"#\">{}</a>", place.to("#studio"), escape(tr("Try the live demo")), escape(tr("Open the web editor")));
    let mut main = hero(tr("Examples"), tr("Example projects to open, read and change"), &lead, &actions);
    main.push('\n');
    main.push_str(&others(None, place));
    Page {
        folder: FOLDER.to_string(),
        title: tr("Example electrical projects with schematics – OpenECraft").to_string(),
        description: tr("Example projects of OpenECraft, the free electrical CAE: motor starters, PLC, control cabinets and buildings, with their schematics as pictures and PDF.").to_string(),
        main,
        kind: Kind::Plain,
        langs: Langs::All,
    }
}

/// The first sheets of an example as SVG and its PDF, in English: the files beside
/// its page.
pub fn drawings(example: &Example, plugins: &Plugins) -> Vec<File> {
    openecraft_i18n::with_lang(Lang::En, || {
        let folder = format!("{FOLDER}/{}", slug(example));
        let project = (example.build)();
        let analysis = Analysis::with_builtin(&project);
        let mut out: Vec<File> = (0..sheets(&project)).map(|i| File::text(format!("{folder}/page-{}.svg", i + 1), page_svg_with(&project, &analysis, i))).collect();
        let pdf = plugins.exporter("pdf").and_then(|exporter| (exporter.run)(&project, &analysis, &params::Args(&serde_json::Map::new())).ok());
        if let Some(pdf) = pdf {
            out.push(File { path: format!("{folder}/{}.pdf", slug(example)), bytes: pdf.into_bytes() });
        }
        out
    })
}

/// The pages of the examples in every language, with their sheets and PDFs.
pub fn site() -> Vec<File> {
    let plugins = Plugins::new(Vec::<String>::new());
    let mut out = files(Langs::All, overview);
    for example in &EXAMPLES {
        out.extend(files(Langs::All, |lang| page(example, lang)));
        out.extend(drawings(example, &plugins));
    }
    out
}

#[cfg(test)]
mod tests;
