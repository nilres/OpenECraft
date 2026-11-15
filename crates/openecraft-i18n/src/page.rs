//! A static web page in the languages of the catalog. The page is written in English
//! and marks its texts (`data-t` on an element, `data-t-placeholder`, `data-t-alt`);
//! its script translates them in the browser. [`translated`] writes the same page in
//! a language as a file of its own, which a search engine reads without running the
//! script (`examples/pages.rs`, the website: `docs/website.md`). The English text
//! stays in the page as the value of `data-t`, so the script still finds it.

use crate::{translation, Lang};

/// The file of the page in a language: `index.html` in English, `de.html` beside it.
pub fn file_name(lang: Lang) -> String {
    match lang {
        Lang::En => "index.html".to_string(),
        _ => format!("{}.html", lang.code()),
    }
}

/// The `content` of these tags is translated too: what search results and link
/// previews show.
const META: [&str; 3] = ["<meta name=\"description\" content=\"", "<meta property=\"og:title\" content=\"", "<meta property=\"og:description\" content=\""];

/// The English page `html` in `lang`; a text the catalog lacks stays English.
pub fn translated(html: &str, lang: Lang) -> String {
    if lang == Lang::En {
        return html.to_string();
    }
    let find = |escaped: &str| translation(&unescape(escaped), lang);
    let mut out = html.replacen("<html lang=\"en\"", &format!("<html lang=\"{}\"", lang.code()), 1);
    out = rewrite(&out, " data-t>", '<', |text| find(text.trim()).map(|t| format!(" data-t=\"{}\">{}", attribute(&unescape(text.trim())), escape(t))));
    out = rewrite(&out, " data-t-placeholder=\"", '"', |text| find(text).map(|t| format!(" placeholder=\"{}\" data-t-placeholder=\"{text}", attribute(t))));
    for meta in META {
        out = rewrite(&out, meta, '"', |text| find(text).map(|t| format!("{meta}{}", attribute(t))));
    }
    // alt="…" data-t-alt="…": the text shown and its English.
    let keys: Vec<String> = out.split(" data-t-alt=\"").skip(1).filter_map(|rest| rest.split('"').next()).map(String::from).collect();
    for key in keys {
        if let Some(t) = find(&key) {
            out = out.replace(&format!(" alt=\"{key}\" data-t-alt=\""), &format!(" alt=\"{}\" data-t-alt=\"", attribute(t)));
        }
    }
    out
}

/// `html` with what stands between each `open` and the next `close` replaced, `open`
/// included, where `with` has something for it.
fn rewrite(html: &str, open: &str, close: char, with: impl Fn(&str) -> Option<String>) -> String {
    let mut out = String::with_capacity(html.len() + html.len() / 4);
    let mut rest = html;
    while let Some(at) = rest.find(open) {
        let after = &rest[at + open.len()..];
        let Some(end) = after.find(close) else { break };
        out.push_str(&rest[..at]);
        match with(&after[..end]) {
            Some(new) => out.push_str(&new),
            None => out.push_str(&rest[at..at + open.len() + end]),
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&")
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn attribute(text: &str) -> String {
    escape(text).replace('"', "&quot;")
}

#[cfg(test)]
mod tests;
