//! The pages written as content files (`website/content/<folder>.html`, a folder of
//! pages with its own as `index.html`): questions and answers, how-tos, tools and the
//! comparisons with single products. A file starts with lines `name: value` (`title`,
//! `description`, `kind`: `page`, `faq` or `tool`), then an empty line and the content
//! of `<main>`, written in English with its texts marked (`data-t`, as on the pages
//! written whole). The build puts it into the frame of the site in every language.
//!
//! Links to the site start with `@/` for a page that exists in every language
//! (`@/compare/`, `@/#studio`) and with `~/` for a file or a page that is not written
//! in every language (`~/docs/tutorial/`, `~/news/`): such a page is linked in the
//! language of the content where it has that language. A page of the kind `faq` names its questions to search
//! engines: `<details><summary data-t>…</summary><p data-t>…</p></details>`.

use crate::frame::{files, File, Kind, Langs, Page, Place};
use openecraft_i18n::page::translated;
use openecraft_i18n::{translation, Lang};
use std::collections::BTreeSet;
use std::path::Path;

/// A content file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Its folder below the site: `how-to/wire-list`.
    pub folder: String,
    pub title: String,
    pub description: String,
    /// `page`, `faq` or `tool`.
    pub kind: String,
    pub body: String,
}

/// What stands between each `open` and the next `close`.
fn between<'a>(text: &'a str, open: &str, close: &str) -> Vec<&'a str> {
    text.split(open).skip(1).filter_map(|rest| rest.split(close).next()).collect()
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&")
}

/// Reads a content file of `folder`.
pub fn parse(folder: &str, text: &str) -> Result<Content, String> {
    let text = text.replace("\r\n", "\n");
    let (head, body) = text.split_once("\n\n").ok_or_else(|| format!("{folder}: no empty line after the head"))?;
    let mut content = Content { folder: folder.to_string(), title: String::new(), description: String::new(), kind: "page".to_string(), body: body.trim().to_string() };
    for line in head.lines() {
        let (name, value) = line.split_once(": ").ok_or_else(|| format!("{folder}: not `name: value`: {line}"))?;
        match name {
            "title" => content.title = value.trim().to_string(),
            "description" => content.description = value.trim().to_string(),
            "kind" if ["page", "faq", "tool"].contains(&value.trim()) => content.kind = value.trim().to_string(),
            _ => return Err(format!("{folder}: unknown line of the head: {line}")),
        }
    }
    if content.title.is_empty() || content.description.is_empty() {
        return Err(format!("{folder}: a page needs a title and a description"));
    }
    Ok(content)
}

/// The content files of a folder (`website/content`), by the folder of their pages.
pub fn read(dir: &Path) -> Result<Vec<Content>, String> {
    fn walk(dir: &Path, below: &str, out: &mut Vec<Content>) -> Result<(), String> {
        let mut entries: Vec<_> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.filter_map(Result::ok).map(|e| e.path()).collect();
        entries.sort();
        for path in entries {
            let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or_default().to_string();
            if path.is_dir() {
                walk(&path, &format!("{below}{name}/"), out)?;
            } else if path.extension().is_some_and(|e| e == "html") {
                let folder = if name == "index" { below.trim_end_matches('/').to_string() } else { format!("{below}{name}") };
                let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                out.push(parse(&folder, &text)?);
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(dir, "", &mut out)?;
    Ok(out)
}

impl Content {
    /// The English texts of the page: what the catalog has to translate.
    pub fn texts(&self) -> BTreeSet<String> {
        let mut out: BTreeSet<String> = between(&self.body, " data-t>", "<").into_iter().map(|t| unescape(t.trim())).collect();
        out.extend(between(&self.body, " data-t-placeholder=\"", "\"").into_iter().map(unescape));
        out.insert(self.title.clone());
        out.insert(self.description.clone());
        out
    }

    /// The questions of the page with their answers, as the body in a language has them.
    fn questions(body: &str) -> Vec<(String, String)> {
        let text = |element: &str| element.split_once('>').map(|(_, text)| unescape(text.trim())).unwrap_or_default();
        between(body, "<details", "</details>")
            .into_iter()
            .filter_map(|qa| {
                let question = between(qa, "<summary", "</summary>").first().map(|q| text(q))?;
                let answer: Vec<String> = between(qa, "<p", "</p>").into_iter().map(text).collect();
                Some((question, answer.join(" ")))
            })
            .collect()
    }

    /// The page in `lang`: its texts from the catalog, its links to the files of the
    /// language.
    pub fn page(&self, lang: Lang) -> Page {
        let place = Place::of(&self.folder, lang);
        let text = |english: &str| translation(english, lang).unwrap_or(english).to_string();
        let body = links(&translated(&self.body, lang), place);
        let kind = match self.kind.as_str() {
            "faq" => Kind::Faq(Content::questions(&body)),
            "tool" => Kind::Tool,
            _ => Kind::Plain,
        };
        Page { folder: self.folder.clone(), title: text(&self.title), description: text(&self.description), main: body, kind, langs: Langs::All }
    }
}

/// `html` with its links to the site (`="@/…"`, `="~/…"`) as addresses seen from `place`.
pub fn links(html: &str, place: Place) -> String {
    let mut out = String::with_capacity(html.len() + 256);
    let mut rest = html;
    while let Some(at) = rest.find("=\"@/").into_iter().chain(rest.find("=\"~/")).min() {
        let translated = rest[at..].starts_with("=\"@/");
        let after = &rest[at + 4..];
        let end = after.find('"').unwrap_or(after.len());
        out.push_str(&rest[..at + 2]);
        let target = &after[..end];
        let page = target[..target.find(['?', '#']).unwrap_or(target.len())].ends_with('/');
        out.push_str(&if translated || page { place.to(target) } else { place.file(target) });
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// The pages of the content files in every language.
pub fn site(contents: &[Content]) -> Vec<File> {
    contents.iter().flat_map(|content| files(Langs::All, |lang| content.page(lang))).collect()
}

#[cfg(test)]
mod tests;
