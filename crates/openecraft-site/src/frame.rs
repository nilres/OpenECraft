//! The frame of a page of the site: head (title, description, link previews, structured
//! data, the markers of the public address), bar, footer and the links between the
//! pages in the language of the page. The pages written by hand (`website/index.html`,
//! `website/compare/…`) have the same frame; the tests of the website compare them.

use openecraft_i18n::page::file_name;
use openecraft_i18n::{tr, Lang};
use serde_json::{json, Value};

/// A file of the site: its path below the folder of the page and its content.
#[derive(Clone, Debug, PartialEq)]
pub struct File {
    pub path: String,
    pub bytes: Vec<u8>,
}

impl File {
    pub fn text(path: impl Into<String>, text: impl Into<String>) -> File {
        File { path: path.into(), bytes: text.into().into_bytes() }
    }
}

/// The languages a page is written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Langs {
    /// Every language of the catalog: `index.html`, `de.html`, …
    All,
    /// English only: the documentation and the release notes are not translated.
    English,
}

impl Langs {
    pub fn list(self) -> Vec<Lang> {
        match self {
            Langs::All => Lang::ALL.to_vec(),
            Langs::English => vec![Lang::En],
        }
    }
}

/// What a page is beyond its text.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Plain,
    /// Questions with their answers, named to search engines as such.
    Faq(Vec<(String, String)>),
    /// A tool that runs the module of the site (`tools.js`).
    Tool,
}

/// A page in one language, without its frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Page {
    /// Its folder below the site: `symbols/contacts`.
    pub folder: String,
    pub title: String,
    pub description: String,
    /// The content of `<main>`.
    pub main: String,
    pub kind: Kind,
    pub langs: Langs,
}

/// Where a page is and in which language: the addresses of its links.
#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub lang: Lang,
    /// Folders between the page and the site: 1 for `faq/`, 2 for `symbols/contacts/`.
    pub depth: usize,
}

impl Place {
    pub fn of(folder: &str, lang: Lang) -> Place {
        Place { lang, depth: folder.split('/').filter(|part| !part.is_empty()).count() }
    }

    /// The folder of the site, seen from the page.
    pub fn root(&self) -> String {
        "../".repeat(self.depth)
    }

    /// A page that exists in every language, by its folder below the site with what
    /// follows it (`compare/`, `#studio`, `?example=conveyor#studio`): the file of
    /// the language of this page.
    pub fn to(&self, target: &str) -> String {
        let at = target.find(['?', '#']).unwrap_or(target.len());
        let file = if self.lang == Lang::En { String::new() } else { file_name(self.lang) };
        format!("{}{}{file}{}", self.root(), &target[..at], &target[at..])
    }

    /// A file of the site, or a page that exists in English only.
    pub fn file(&self, target: &str) -> String {
        format!("{}{target}", self.root())
    }
}

/// The length of a description search engines show whole.
pub const DESCRIPTION: std::ops::Range<usize> = 81..170;

pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub fn attribute(text: &str) -> String {
    escape(text).replace('"', "&quot;")
}

/// `text` cut at a word to at most `max` characters, with an ellipsis when cut.
pub fn shortened(text: &str, max: usize) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= max {
        return text;
    }
    let cut: String = text.chars().take(max - 1).collect();
    // A word that does not fit whole is left out.
    let whole = text.chars().nth(max - 1).is_some_and(char::is_whitespace);
    let cut = if whole { cut.as_str() } else { cut.rsplit_once(' ').map_or(cut.as_str(), |(head, _)| head) };
    format!("{}…", cut.trim_end_matches([',', ';', ':', '.', ' ', '(']))
}

/// The longest title search results show whole.
pub const TITLE: usize = 70;

/// The title of a page: `full`, or, where that is longer than search results show,
/// the `name` of what the page is about with the name of the project.
pub fn title(full: String, name: &str) -> String {
    const PROJECT: &str = " – OpenECraft";
    if full.chars().count() <= TITLE {
        return full;
    }
    format!("{}{PROJECT}", shortened(name, TITLE - PROJECT.chars().count()))
}

/// The pages of the bar, as on the pages written by hand (`website/src/pages.rs`).
const BAR: [(&str, &str); 7] = [
    ("#features", "Features"),
    ("#studio", "Live demo"),
    ("panel-builders/", "Panel builders"),
    ("electricians/", "Electricians"),
    ("education/", "Schools"),
    ("compare/", "Comparison"),
    ("#get", "Get it"),
];

/// The sections of the site every footer links: folder, text, whether the page exists
/// in every language. The texts are translated where the footer is written.
pub const SECTIONS: [(&str, &str, bool); 7] = [
    ("symbols/", "Symbols", true),
    ("examples/", "Examples", true),
    ("tools/", "Tools", true),
    ("how-to/", "How-to", true),
    ("docs/", "Documentation", false),
    ("news/", "News", false),
    ("faq/", "FAQ", true),
];

/// The documents the footer links, in the site (`docs`).
pub const DOCUMENTS: [(&str, &str); 3] = [("docs/tutorial/", "Tutorial"), ("docs/plugins/", "Plugins"), ("docs/command-line/", "Command line and MCP")];

fn bar_text(text: &str) -> &'static str {
    match text {
        "Features" => tr("Features"),
        "Live demo" => tr("Live demo"),
        "Panel builders" => tr("Panel builders"),
        "Electricians" => tr("Electricians"),
        "Schools" => tr("Schools"),
        "Comparison" => tr("Comparison"),
        _ => tr("Get it"),
    }
}

/// The text of a section of [`SECTIONS`] or a document of [`DOCUMENTS`] in the
/// language of the page.
pub fn section_text(text: &str) -> &'static str {
    match text {
        "Symbols" => tr("Symbols"),
        "Examples" => tr("Examples"),
        "Tools" => tr("Tools"),
        "How-to" => tr("How-to"),
        "Documentation" => tr("Documentation"),
        "News" => tr("News"),
        "FAQ" => tr("FAQ"),
        "Tutorial" => tr("Tutorial"),
        "Plugins" => tr("Plugins"),
        _ => tr("Command line and MCP"),
    }
}

/// What the page tells search engines about itself (JSON-LD).
fn structured(page: &Page, lang: Lang) -> Value {
    let about = json!({
        "@type": "SoftwareApplication",
        "name": "OpenECraft",
        "applicationCategory": "DesignApplication",
        "applicationSubCategory": "Electrical CAE",
        "operatingSystem": "Windows, Linux, macOS, web browser",
        "isAccessibleForFree": true,
        "offers": { "@type": "Offer", "price": "0", "priceCurrency": "EUR" }
    });
    let mut data = json!({
        "@context": "https://schema.org",
        "@type": "WebPage",
        "name": page.title,
        "description": page.description,
        "inLanguage": lang.code(),
        "about": about
    });
    if let Kind::Faq(questions) = &page.kind {
        data["@type"] = json!("FAQPage");
        data["mainEntity"] = questions.iter().map(|(q, a)| json!({ "@type": "Question", "name": q, "acceptedAnswer": { "@type": "Answer", "text": a } })).collect();
    }
    data
}

/// The page in its frame, as the file of `lang`.
pub fn html(page: &Page, lang: Lang) -> String {
    let place = Place::of(&page.folder, lang);
    let root = place.root();
    let (title, description) = (attribute(&page.title), attribute(&page.description));
    // A tool runs the WebAssembly of the site; no other page may.
    let scripts = if page.kind == Kind::Tool { "script-src 'self' 'wasm-unsafe-eval'" } else { "script-src 'self'" };
    // "</" would end the script element.
    let data = serde_json::to_string_pretty(&structured(page, lang)).unwrap_or_default().replace("</", "<\\/");
    let mut out = String::with_capacity(page.main.len() + 8000);
    out.push_str(&format!(
        "<!DOCTYPE html>\n<html lang=\"{code}\">\n<head>\n    <meta charset=\"utf-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n    \
         <!-- Written by the build of the site (crates/openecraft-site, docs/website.md). It loads nothing beyond its own address. -->\n    \
         <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self'; {scripts}; style-src 'self'; img-src 'self' data: blob:; object-src 'none'; base-uri 'self'; form-action 'self'\" />\n    \
         <title>{title}</title>\n    <meta name=\"description\" content=\"{description}\" />\n    <meta name=\"theme-color\" content=\"#070b14\" />\n    \
         <meta property=\"og:type\" content=\"website\" />\n    <meta property=\"og:site_name\" content=\"OpenECraft\" />\n    \
         <meta property=\"og:title\" content=\"{title}\" />\n    <meta property=\"og:description\" content=\"{description}\" />\n    \
         <meta name=\"twitter:card\" content=\"summary_large_image\" />\n    \
         <!-- site address (installer/docker/website-site.sh) -->\n    <!-- /site address -->\n    \
         <script type=\"application/ld+json\">\n{data}\n    </script>\n    \
         <link rel=\"stylesheet\" href=\"{root}site.css\" />\n    <link rel=\"icon\" href=\"{root}icon.svg\" type=\"image/svg+xml\" />\n    \
         <link rel=\"apple-touch-icon\" href=\"{root}apple-touch-icon.png\" />\n",
        code = lang.code()
    ));
    if page.folder.starts_with("news") {
        out.push_str(&format!("    <link rel=\"alternate\" type=\"application/atom+xml\" title=\"OpenECraft\" href=\"{root}news/feed.xml\" />\n"));
    }
    out.push_str(&format!("    <script src=\"{root}config.js\"></script>\n    <script src=\"{root}site.js\" defer></script>\n"));
    if page.kind == Kind::Tool {
        out.push_str(&format!("    <script src=\"{root}build.js\"></script>\n    <script src=\"{root}tools.js\" defer></script>\n"));
    }
    out.push_str("</head>\n<body>\n<div class=\"backdrop\" aria-hidden=\"true\"></div>\n\n<header class=\"nav\">\n");
    out.push_str(&format!("    <a class=\"brand\" href=\"{}\"><img src=\"{root}icon.svg\" alt=\"\" width=\"30\" height=\"30\" /><span>OpenECraft</span></a>\n", place.to("")));
    out.push_str(&format!(
        "    <button class=\"nav-toggle\" id=\"nav-toggle\" aria-expanded=\"false\" aria-controls=\"nav-links\" hidden><span class=\"bars\" aria-hidden=\"true\"></span><span class=\"sr\">{}</span></button>\n    <nav id=\"nav-links\">\n",
        escape(tr("Menu"))
    ));
    for (target, text) in BAR {
        let current = if !target.starts_with('#') && page.folder.starts_with(target.trim_end_matches('/')) { " aria-current=\"page\"" } else { "" };
        out.push_str(&format!("        <a href=\"{}\"{current}>{}</a>\n", place.to(target), escape(bar_text(text))));
    }
    let source = escape(tr("Source code"));
    out.push_str(&format!("        <a class=\"menu-only\" data-link=\"repository\" href=\"#\">{source}</a>\n    </nav>\n    <div class=\"nav-right\">\n"));
    out.push_str("        <select id=\"language\" aria-label=\"Language\" hidden></select>\n");
    out.push_str(&format!("        <a class=\"button ghost\" data-link=\"repository\" href=\"#\">{source}</a>\n    </div>\n</header>\n\n<main id=\"top\">\n\n"));
    out.push_str(page.main.trim_end());
    out.push_str("\n\n</main>\n\n");
    out.push_str(&footer(page, place));
    out.push_str("</body>\n</html>\n");
    out
}

fn footer(page: &Page, place: Place) -> String {
    let root = place.root();
    let mut out = format!(
        "<footer>\n    <div><img src=\"{root}icon.svg\" alt=\"\" width=\"22\" height=\"22\" /> <b>OpenECraft</b> · <span>{}</span></div>\n    <div class=\"links\">\n",
        escape(tr("Open source under the MIT and Apache-2.0 licenses."))
    );
    for (target, text) in DOCUMENTS {
        out.push_str(&format!("        <a href=\"{}\">{}</a>\n", place.file(target), escape(section_text(text))));
    }
    out.push_str(&format!("        <a data-link=\"issues\" href=\"#\">{}</a>\n", escape(tr("Report a problem"))));
    // The dialogs of the main page, in the language of this one.
    out.push_str(&format!("        <a id=\"licenses-open\" href=\"{}\">{}</a>\n", place.to("#licenses"), escape(tr("Licenses"))));
    out.push_str(&format!("        <a id=\"privacy-open\" href=\"{}\">{}</a>\n", place.to("#privacy"), escape(tr("Privacy"))));
    out.push_str(&format!("        <a id=\"legal-open\" href=\"{}\" hidden>{}</a>\n    </div>\n    <div class=\"links\">\n", place.to("#legal"), escape(tr("Legal notice"))));
    for (target, text) in BAR.iter().filter(|(target, _)| !target.starts_with('#')) {
        out.push_str(&format!("        <a href=\"{}\">{}</a>\n", place.to(target), escape(bar_text(text))));
    }
    out.push_str("    </div>\n    <div class=\"links\">\n");
    for (target, text, translated) in SECTIONS {
        let address = if translated { place.to(target) } else { place.file(target) };
        out.push_str(&format!("        <a href=\"{address}\">{}</a>\n", escape(section_text(text))));
    }
    out.push_str("    </div>\n");
    if page.langs == Langs::All {
        out.push_str("    <nav class=\"languages\" aria-label=\"Language\">\n");
        for lang in Lang::ALL {
            let file = if lang == Lang::En { "./".to_string() } else { file_name(lang) };
            let code = lang.code();
            let current = if lang == place.lang { " aria-current=\"page\"" } else { "" };
            out.push_str(&format!("        <a href=\"{file}\" hreflang=\"{code}\" lang=\"{code}\"{current}>{}</a>\n", lang.native_name()));
        }
        out.push_str("    </nav>\n");
    }
    out.push_str(&format!(
        "    <p class=\"fine\"><span>{}</span> <span>{}</span></p>\n    <p class=\"fine\">{}</p>\n</footer>\n",
        escape(tr("EPLAN, TIA Portal and other names are trademarks of their owners. OpenECraft is an independent project.")),
        escape(tr("It is not affiliated with, endorsed by or sponsored by the owners of these names.")),
        escape(tr("OpenECraft comes without warranty. Its calculations and checks support the planning; a qualified person has to verify the results against the standards and regulations that apply."))
    ));
    out
}

/// The first screen of a page: what it is about, with the buttons below.
pub fn hero(eyebrow: &str, heading: &str, lead: &str, actions: &str) -> String {
    let actions = if actions.is_empty() { String::new() } else { format!("\n        <div class=\"actions\">{actions}</div>") };
    format!(
        "<section class=\"hero page-hero\">\n    <div class=\"hero-text\">\n        <p class=\"eyebrow\">{}</p>\n        <h1>{}</h1>\n        <p class=\"lead\">{}</p>{actions}\n    </div>\n</section>\n",
        escape(eyebrow),
        escape(heading),
        escape(lead)
    )
}

/// The files of a page: one for every language it has, each written by `page` in
/// that language.
pub fn files(langs: Langs, page: impl Fn(Lang) -> Page) -> Vec<File> {
    langs
        .list()
        .into_iter()
        .map(|lang| {
            let page = openecraft_i18n::with_lang(lang, || page(lang));
            let text = openecraft_i18n::with_lang(lang, || html(&page, lang));
            File::text(format!("{}/{}", page.folder, file_name(lang)), text)
        })
        .collect()
}

#[cfg(test)]
mod tests;
