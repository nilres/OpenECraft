//! Markdown of the repository (the documentation, the changelog) as HTML of a page:
//! headings get the anchors GitHub gives them, so the links between the documents keep
//! working, and every link is led to where its target is on the site ([`Target`]).

use crate::frame::attribute;
use pulldown_cmark::{html, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::collections::BTreeMap;

/// Where a link of a document leads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// An address as it is: another site, a section of the same page, a page or a
    /// file of the site.
    Address(String),
    /// A file of the repository the site does not have: its path there. The page
    /// links it through the address of the repository (`config.js`).
    Repository(String),
}

/// A document as HTML.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rendered {
    pub html: String,
    /// The text of its first heading.
    pub title: String,
    /// The text of its first paragraph.
    pub summary: String,
    /// The pictures it shows, by the address the page has them at.
    pub pictures: Vec<String>,
}

/// The anchor GitHub gives a heading: lower case, spaces as dashes, without
/// punctuation.
pub fn anchor(heading: &str) -> String {
    heading.trim().to_lowercase().chars().filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_')).map(|c| if c == ' ' { '-' } else { c }).collect()
}

/// `path` relative to the folder of `from`, as a path of the repository: `..` and `.`
/// resolved (`docs/features/parts.md` + `../tutorial.md` = `docs/tutorial.md`).
pub fn resolved(from: &str, path: &str) -> String {
    let mut parts: Vec<&str> = from.split('/').collect();
    parts.pop();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    parts.join("/")
}

/// Renders `markdown`. `link` gets the address of every link and picture as the
/// document writes it and says where it leads.
pub fn render(markdown: &str, link: &dyn Fn(&str) -> Target) -> Rendered {
    let mut out = Rendered::default();
    let mut events: Vec<Event> = Parser::new_ext(markdown, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH).collect();
    // The text of every heading and of the first paragraph.
    let mut used: BTreeMap<String, usize> = BTreeMap::new();
    let mut open: Option<(usize, String, bool)> = None;
    let mut anchors: Vec<(usize, String)> = Vec::new();
    for (i, event) in events.iter().enumerate() {
        match event {
            Event::Start(Tag::Heading { .. }) => open = Some((i, String::new(), true)),
            Event::Start(Tag::Paragraph) if out.summary.is_empty() && open.is_none() => open = Some((i, String::new(), false)),
            Event::Text(text) | Event::Code(text) => {
                if let Some((_, collected, _)) = &mut open {
                    collected.push_str(text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some((_, collected, _)) = &mut open {
                    collected.push(' ');
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((at, text, true)) = open.take() {
                    if out.title.is_empty() {
                        out.title = text.clone();
                    }
                    // A second heading of the same text gets a number, as on GitHub.
                    let base = anchor(&text);
                    let count = used.entry(base.clone()).or_insert(0);
                    anchors.push((at, if *count == 0 { base.clone() } else { format!("{base}-{count}") }));
                    *count += 1;
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if let Some((_, text, false)) = open.take() {
                    out.summary = text;
                }
            }
            _ => {}
        }
    }
    for (at, id) in anchors {
        let mut second = false;
        if let Event::Start(Tag::Heading { id: slot, level, .. }) = &mut events[at] {
            *slot = Some(CowStr::from(id));
            // The page has one first heading: its own.
            second = *level == HeadingLevel::H1 && at > 0;
            if second {
                *level = HeadingLevel::H2;
            }
        }
        if let Some(end) = events[at..].iter_mut().find(|e| matches!(e, Event::End(TagEnd::Heading(_)))).filter(|_| second) {
            *end = Event::End(TagEnd::Heading(HeadingLevel::H2));
        }
    }
    // The links: a file of the repository becomes a link the page fills in.
    for event in &mut events {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => match link(dest_url) {
                Target::Address(address) => {
                    let outside = address.starts_with("http://") || address.starts_with("https://");
                    *event = Event::Html(CowStr::from(format!("<a href=\"{}\"{}>", attribute(&address), if outside { " rel=\"noopener\"" } else { "" })));
                }
                Target::Repository(path) => {
                    *event = Event::Html(CowStr::from(format!("<a data-link=\"repository\" data-path=\"/blob/master/{}\" href=\"#\">", attribute(&path))));
                }
            },
            Event::End(TagEnd::Link) => *event = Event::Html(CowStr::from("</a>")),
            Event::Start(Tag::Image { dest_url, .. }) => {
                if let Target::Address(address) = link(dest_url) {
                    out.pictures.push(address.clone());
                    *dest_url = CowStr::from(address);
                }
            }
            // HTML written into a document is shown as text.
            Event::Html(text) | Event::InlineHtml(text) => *event = Event::Text(text.clone()),
            _ => {}
        }
    }
    html::push_html(&mut out.html, events.into_iter());
    // A wide table scrolls in its own box, and pictures load when they are reached.
    out.html = out.html.replace("<table>", "<div class=\"table-scroll\"><table>").replace("</table>", "</table></div>").replace("<img src=", "<img loading=\"lazy\" src=");
    out
}

#[cfg(test)]
mod tests;
