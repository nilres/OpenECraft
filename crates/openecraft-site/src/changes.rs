//! When the pages of the site last changed: `pages.tsv` beside the main page, a line
//! for every page with its file, a hash of what it says and the day that changed
//! last. A build has no memory of the one before, so it takes the days from the file
//! of the site that is online (`website/pages.tsv`, fetched by
//! `installer/web/pages.sh`): a page that says the same keeps its day, a changed or
//! new one gets the day of the build. `website-site.sh` writes the days into the
//! sitemap (`<lastmod>`) and `website-indexnow.sh` tells search engines of the pages
//! of the day of the build. A day nobody knows is left empty rather than made up: a
//! search engine trusts the days of a sitemap only as long as they are true.

use std::collections::BTreeMap;

/// The file below the site.
pub const FILE: &str = "pages.tsv";

/// The mark of a page in its head: the place of the tags with the public address.
const PAGE: &str = "<!-- site address ";

/// Whether `file` (`index.html`, `compare/de.html`) with the content `html` is a page
/// of the sitemap: the page of a language, one or two folders deep at most, with the
/// place for its address.
pub fn is_page(file: &str, html: &str) -> bool {
    let mut parts: Vec<&str> = file.split('/').collect();
    let name = parts.pop().unwrap_or_default();
    let language = name.strip_suffix(".html").is_some_and(|code| code == "index" || (code.len() == 2 && code.bytes().all(|b| b.is_ascii_lowercase())));
    language && parts.len() <= 2 && html.contains(PAGE)
}

/// What stands between `open` and the next `close`.
fn between<'a>(html: &'a str, open: &str, close: &str) -> &'a str {
    html.split_once(open).and_then(|(_, rest)| rest.split_once(close)).map_or("", |(text, _)| text)
}

/// A hash of what a page says: its title, its description and its content. The frame
/// around them (bar, footer, the names of the scripts) changes with every build of the
/// site without the page saying anything new.
pub fn fingerprint(html: &str) -> String {
    let title = between(html, "<title", "</title>");
    let description = between(html, "<meta name=\"description\" content=\"", "\"");
    let main = between(html, "<main", "</main>");
    // FNV-1a, 64 bit: the same on every machine and in every version of the compiler.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for part in [title, description, main] {
        for byte in part.bytes().filter(|b| *b != b'\r').chain([0]) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("{hash:016x}")
}

/// The day of `seconds` since 1970 (UTC): `2026-11-28`.
pub fn day(seconds: u64) -> String {
    // Days to a date of the proleptic Gregorian calendar, in eras of 400 years that
    // start on the first of March.
    let days = (seconds / 86_400) as i64 + 719_468;
    let (era, of_era) = (days.div_euclid(146_097), days.rem_euclid(146_097));
    let year_of_era = (of_era - of_era / 1_460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month = (5 * of_year + 2) / 153;
    let (day, month) = (of_year - (153 * month + 2) / 5 + 1, if month < 10 { month + 3 } else { month - 9 });
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

fn is_day(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10 && bytes.iter().enumerate().all(|(i, b)| if i == 4 || i == 7 { *b == b'-' } else { b.is_ascii_digit() })
}

/// The lines of a file: page, hash and day (empty: not known).
fn lines(file: &str) -> BTreeMap<&str, (&str, &str)> {
    file.lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let mut fields = line.trim_end_matches('\r').split('\t');
            let (page, hash, day) = (fields.next()?, fields.next()?, fields.next().unwrap_or_default());
            Some((page, (hash, if is_day(day) { day } else { "" })))
        })
        .collect()
}

/// The file of a build on `today`: `pages` are the pages of the site (file and
/// content), `before` is the file of the site as it is online (empty: none).
///
/// A page that says what it said keeps its day, a changed one and one that is new to
/// a site with a file get `today`. Without a file before, no page has a day: the
/// first build cannot know when its pages changed.
pub fn file(pages: &[(String, String)], before: &str, today: &str) -> String {
    let before = lines(before);
    let mut out = format!("# The pages of the site: file, hash of its content, day of its last change (docs/website.md).\n# built {today}\n");
    let mut pages: Vec<(&str, String)> = pages.iter().map(|(file, html)| (file.as_str(), fingerprint(html))).collect();
    pages.sort();
    for (page, hash) in pages {
        let day = match before.get(page) {
            Some((old, day)) if *old == hash => day,
            Some(_) => today,
            None if before.is_empty() => "",
            None => today,
        };
        out.push_str(&format!("{page}\t{hash}\t{day}\n"));
    }
    out
}

/// The pages of `file` that changed on the day it was built.
pub fn changed(file: &str) -> Vec<&str> {
    let built = file.lines().find_map(|line| line.strip_prefix("# built ")).unwrap_or_default().trim();
    lines(file).into_iter().filter(|(_, (_, day))| !day.is_empty() && *day == built).map(|(page, _)| page).collect()
}

#[cfg(test)]
mod tests;
