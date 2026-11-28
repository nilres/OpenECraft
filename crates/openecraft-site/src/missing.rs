//! The page for an address the site has nothing at (404): what happened and where to
//! go on, in the frame of the site and in every language (`404.html`, `404.de.html`,
//! …; the server picks the one of the browser, `installer/docker/website.conf`).
//!
//! It is shown for an address of any depth, so its links cannot be relative to where
//! it is asked for: it is written like a page beside the main one, and
//! `website-site.sh` names the folder of the site in its head (`<base>`).

use crate::frame::{escape, html, section_text, File, Kind, Langs, Page, Place};
use openecraft_i18n::{tr, Lang};

/// The file of the page in a language, beside the main page.
pub fn file_name(lang: Lang) -> String {
    match lang {
        Lang::En => "404.html".to_string(),
        _ => format!("404.{}.html", lang.code()),
    }
}

/// The sections the page leads on to.
const LEADS: [(&str, &str); 4] = [("docs/", "Documentation"), ("symbols/", "Symbols"), ("tools/", "Tools"), ("faq/", "FAQ")];

/// The page in `lang`.
pub fn page(lang: Lang) -> Page {
    let place = Place::of("", lang);
    let mut actions = format!("<a class=\"button primary\" href=\"{}\">{}</a>", place.to(""), escape(tr("Start page")));
    for (folder, text) in LEADS {
        actions.push_str(&format!("<a class=\"button\" href=\"{}\">{}</a>", place.to(folder), escape(section_text(text))));
    }
    let lead = tr("There is no page at this address. It may have moved, or the address has a typing error. These pages lead on.");
    Page {
        folder: String::new(),
        title: tr("Page not found – OpenECraft").to_string(),
        description: lead.to_string(),
        main: crate::frame::hero(tr("Error 404"), tr("Page not found"), lead, &actions),
        kind: Kind::Missing,
        langs: Langs::All,
    }
}

/// The page in every language.
pub fn site() -> Vec<File> {
    Lang::ALL.into_iter().map(|lang| File::text(file_name(lang), openecraft_i18n::with_lang(lang, || html(&page(lang), lang)))).collect()
}

#[cfg(test)]
mod tests;
