//! The translation catalog: one folder `catalog/<area>/` per area of [`AREAS`] (the
//! crate group that shows the texts, see `scan::area_of_member`; `common` for texts
//! shown by several). `required.tsv` holds tab separated
//! `English<TAB>Deutsch<TAB>Français<TAB>Español` lines; the English text is the lookup
//! key and must be identical to the literal passed to `tr` / `tr_fmt!` / `tr_const!`
//! (`\n` and `\"` are written as in Rust source). The further languages have one file
//! each, `<code>.tsv`, with `English<TAB>translation` lines; a text without a line
//! there is shown in English. Lines starting with `#` are comments.
//!
//! Every file is sorted by its English text and holds the texts of its area only, so
//! that branches adding texts rarely touch the same lines; `cargo run -p openecraft-i18n
//! --example tidy` sorts and places the entries, the tests check it.

pub type Entry = (&'static str, &'static str, &'static str, &'static str);

/// `with_areas!(m, args…)` expands to `m!(args…, [areas])`, so the area list of
/// [`AREAS`] is written once (the file names must be literals for `include_str!`).
macro_rules! with_areas {
    ($m:ident $(, $arg:literal)*) => {
        $m!($($arg,)* ["common", "editor", "core", "plugins", "cli", "ui", "parts"])
    };
}
macro_rules! area_names {
    ([$($area:literal),*]) => { [$($area),*] };
}
macro_rules! required_files {
    ([$($area:literal),*]) => { [$(($area, include_str!(concat!("catalog/", $area, "/required.tsv")))),*] };
}
macro_rules! further_files {
    ($code:literal, [$($area:literal),*]) => { ($code, [$(($area, include_str!(concat!("catalog/", $area, "/", $code, ".tsv")))),*]) };
}

/// `(area, file)` of the required languages.
const REQUIRED: [(&str, &str); AREAS.len()] = with_areas!(required_files);

/// The catalogs of the further languages by ISO code: `(area, file)` per area.
const FURTHER: [(&str, [(&str, &str); AREAS.len()]); 16] = [
    with_areas!(further_files, "pt"),
    with_areas!(further_files, "it"),
    with_areas!(further_files, "nl"),
    with_areas!(further_files, "pl"),
    with_areas!(further_files, "cs"),
    with_areas!(further_files, "ru"),
    with_areas!(further_files, "uk"),
    with_areas!(further_files, "tr"),
    with_areas!(further_files, "sv"),
    with_areas!(further_files, "da"),
    with_areas!(further_files, "fi"),
    with_areas!(further_files, "hu"),
    with_areas!(further_files, "ro"),
    with_areas!(further_files, "zh"),
    with_areas!(further_files, "ja"),
    with_areas!(further_files, "ko"),
];

/// The catalog areas: one folder `catalog/<area>/` each (`required.tsv` and one
/// `<code>.tsv` per further language).
pub const AREAS: [&str; 7] = with_areas!(area_names);

/// The entry lines of a file (comments and blank lines skipped).
pub fn lines(file: &'static str) -> impl Iterator<Item = &'static str> {
    file.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
}

/// `(area, contents)` of the `required.tsv` files.
pub fn required_files() -> impl Iterator<Item = (&'static str, &'static str)> {
    REQUIRED.iter().copied()
}

/// `(area, contents)` of the files of a further language (none for an unknown code).
pub fn further_files(code: &str) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
    FURTHER.iter().filter(move |(c, _)| *c == code).flat_map(|(_, files)| files.iter().copied())
}

/// The entry of a `required.tsv` line; `None` for a malformed one.
pub fn parse_required(line: &'static str) -> Option<Entry> {
    let mut cols = line.split('\t');
    let entry = (unescape(cols.next()?), unescape(cols.next()?), unescape(cols.next()?), unescape(cols.next()?));
    cols.next().is_none().then_some(entry)
}

/// The entry of a `<code>.tsv` line; `None` for a malformed one.
pub fn parse_further(line: &'static str) -> Option<(&'static str, &'static str)> {
    let mut cols = line.split('\t');
    let entry = (unescape(cols.next()?), unescape(cols.next()?));
    cols.next().is_none().then_some(entry)
}

/// All catalog entries; a malformed line yields `None` (reported by the tests).
pub fn parse() -> impl Iterator<Item = Option<Entry>> {
    required_files().flat_map(|(_, f)| lines(f)).map(parse_required)
}

pub fn all() -> impl Iterator<Item = Entry> {
    parse().flatten()
}

/// `(English, translation)` of a further language (none for an unknown code); a
/// malformed line yields `None`.
pub fn further(code: &str) -> impl Iterator<Item = Option<(&'static str, &'static str)>> + '_ {
    further_files(code).flat_map(|(_, f)| lines(f)).map(parse_further)
}

/// Resolves the Rust escapes used in the catalog (`\n`, `\t`, `\"`, `\\`).
fn unescape(s: &'static str) -> &'static str {
    if !s.contains('\\') {
        return s;
    }
    Box::leak(crate::scan::unescape(s).into_boxed_str())
}
