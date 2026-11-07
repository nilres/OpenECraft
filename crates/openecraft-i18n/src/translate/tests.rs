use super::*;
use crate::scan;
use crate::AREAS;

const TIDY: &str = "run `cargo run -p openecraft-i18n --example tidy`";

#[test]
fn lang_codes_round_trip() {
    for l in Lang::ALL {
        assert_eq!(Lang::from_code(l.code()), Some(l));
    }
    assert_eq!(Lang::from_code("de_DE.UTF-8"), Some(Lang::De));
    assert_eq!(Lang::from_code("fr-CA"), Some(Lang::Fr));
    assert_eq!(Lang::from_code("pt-BR"), Some(Lang::Pt));
    assert_eq!(Lang::from_code("zh_CN.UTF-8"), Some(Lang::Zh));
    assert_eq!(Lang::from_code("tlh"), None);
}

#[test]
fn every_further_language_has_a_catalog() {
    for l in Lang::ALL.into_iter().filter(|l| !l.is_required()) {
        assert!(!catalog_translations(l).is_empty(), "no translations for {}", l.code());
        assert_eq!(catalog::further_files(l.code()).count(), AREAS.len(), "{}: one file per area", l.code());
    }
    assert!(catalog::further("de").next().is_none(), "the required languages are in the main catalog");
}

#[test]
fn further_languages_translate_and_fall_back_to_english() {
    assert_eq!(translate("no such text", Lang::Ja), "no such text");
    for l in Lang::ALL.into_iter().filter(|l| !l.is_required()) {
        let (en, t) = catalog_translations(l)[0];
        assert_eq!(translate(en, l), t);
    }
}

/// `{}` / `{0}` … placeholders and named template fields (`{page}`) of a text.
fn placeholders(s: &str) -> (usize, Vec<&str>) {
    let mut positional = 0;
    let mut named = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find('{') {
        let Some(end) = rest[start..].find('}') else { break };
        let key = &rest[start + 1..start + end];
        if key.is_empty() || key.chars().all(|c| c.is_ascii_digit()) {
            positional += 1;
        } else if !key.contains(char::is_whitespace) {
            named.push(key);
        }
        rest = &rest[start + end + 1..];
    }
    named.sort_unstable();
    (positional, named)
}

/// The English texts of an area's `required.tsv`.
fn area_keys(area: &str) -> std::collections::HashSet<&'static str> {
    catalog::required_files().filter(|(a, _)| *a == area).flat_map(|(_, f)| catalog::lines(f)).filter_map(catalog::parse_required).map(|e| e.0).collect()
}

#[test]
fn further_catalogs_are_consistent() {
    for l in Lang::ALL.into_iter().filter(|l| !l.is_required()) {
        let code = l.code();
        let bad = catalog::further(code).filter(|e| e.is_none()).count();
        assert_eq!(bad, 0, "{bad} malformed lines in the catalog/*/{code}.tsv files (need English<TAB>translation)");
        let mut seen = std::collections::HashSet::new();
        for (area, file) in catalog::further_files(code) {
            let known = area_keys(area);
            for (en, t) in catalog::lines(file).filter_map(catalog::parse_further) {
                assert!(seen.insert(en), "{code}: duplicate entry: {en}");
                assert!(known.contains(en), "catalog/{area}/{code}.tsv: not in catalog/{area}/required.tsv: {en} ({TIDY})");
                assert!(!t.trim().is_empty(), "{code}: empty translation: {en}");
                assert_eq!(placeholders(en), placeholders(t), "{code}: placeholders differ: {en}");
                assert_eq!(en.starts_with(' '), t.starts_with(' '), "{code}: leading space differs: {en}");
                assert_eq!(en.ends_with(' '), t.ends_with(' '), "{code}: trailing space differs: {en}");
            }
        }
    }
}

#[test]
fn placeholders_are_counted_and_named() {
    assert_eq!(placeholders("{} of {1} on {page}"), (2, vec!["page"]));
    assert_eq!(placeholders("{name, description}"), (0, vec![]));
}

#[test]
fn translates_and_falls_back_to_english() {
    assert_eq!(translate("Cancel", Lang::En), "Cancel");
    assert_eq!(translate("Cancel", Lang::De), "Abbrechen");
    assert_eq!(translate("Cancel", Lang::Fr), "Annuler");
    assert_eq!(translate("Cancel", Lang::Es), "Cancelar");
    assert_eq!(translate("no such text", Lang::De), "no such text");
}

#[test]
fn output_language_is_set_for_this_thread_only() {
    let ui = lang();
    let inner = with_lang(Lang::De, || {
        let other = std::thread::spawn(lang).join().unwrap();
        (tr("Open"), with_lang(Lang::Fr, lang), lang(), other)
    });
    assert_eq!(inner.0, translate("Open", Lang::De));
    assert_eq!((inner.1, inner.2), (Lang::Fr, Lang::De));
    assert_eq!(inner.3, ui);
    assert_eq!(lang(), ui);
}

#[test]
fn placeholders_are_filled_in_order_or_by_index() {
    assert_eq!(fill("Saved {} as {}", &[&"a", &3]), "Saved a as 3");
    assert_eq!(fill("{1} von {0}", &[&"x", &"y"]), "y von x");
    assert_eq!(fill("keeps {unknown} and {}", &[&1]), "keeps {unknown} and 1");
}

#[test]
fn catalog_is_complete_and_consistent() {
    let mut seen = std::collections::HashSet::new();
    for (en, de, fr, es) in catalog::all() {
        assert!(seen.insert(en), "duplicate catalog entry: {en}");
        let count = |s: &str| s.matches("{}").count() + s.matches("{0}").count() + s.matches("{1}").count() + s.matches("{2}").count();
        for (name, t) in [("German", de), ("French", fr), ("Spanish", es)] {
            assert!(!t.trim().is_empty(), "{name} translation missing: {en}");
            assert_eq!(count(en), count(t), "{name} placeholders differ: {en}");
            assert_eq!(en.starts_with(' '), t.starts_with(' '), "{name} leading space differs: {en}");
            assert_eq!(en.ends_with(' '), t.ends_with(' '), "{name} trailing space differs: {en}");
        }
    }
}

#[test]
fn catalog_lines_are_well_formed() {
    let bad = catalog::parse().filter(|e| e.is_none()).count();
    assert_eq!(bad, 0, "{bad} malformed catalog lines (need exactly 4 tab separated columns)");
}

/// Every file is sorted by its (raw) English text, so that branches adding texts
/// seldom touch the same lines.
#[test]
fn catalog_files_are_sorted_by_english_text() {
    let further = Lang::ALL.into_iter().filter(|l| !l.is_required()).flat_map(|l| catalog::further_files(l.code()).map(move |(a, f)| (a, l.code(), f)));
    for (area, name, file) in catalog::required_files().map(|(a, f)| (a, "required", f)).chain(further) {
        let keys: Vec<&str> = catalog::lines(file).map(|l| l.split('\t').next().unwrap()).collect();
        for pair in keys.windows(2) {
            assert!(pair[0] < pair[1], "catalog/{area}/{name}.tsv is not sorted at {:?} > {:?} ({TIDY})", pair[0], pair[1]);
        }
    }
}

/// A text is in the file of the area whose crates use it, in `common/` when crates of
/// several areas do; texts the scan cannot see stay where they are.
#[test]
fn texts_are_in_the_file_of_their_area() {
    let used = scan::used_texts(&scan::workspace_root());
    let mut misplaced = Vec::new();
    for (area, file) in catalog::required_files() {
        for (en, ..) in catalog::lines(file).filter_map(catalog::parse_required) {
            if let Some(to) = used.get(en).and_then(|u| scan::area_for(&u.areas)) {
                if to != area {
                    misplaced.push(format!("catalog/{area}/required.tsv -> catalog/{to}/: {en:?}"));
                }
            }
        }
    }
    assert!(misplaced.is_empty(), "texts in the file of another area ({TIDY}):\n{}", misplaced.join("\n"));
}

/// Every text the applications translate must be in the catalog in all languages.
#[test]
fn all_texts_used_in_the_sources_are_in_the_catalog() {
    let known: std::collections::HashSet<&str> = catalog::all().map(|e| e.0).collect();
    let used = scan::used_texts(&scan::workspace_root());
    assert!(used.len() > 500, "source scan found only {} texts – scanner broken?", used.len());
    let missing: Vec<String> = used
        .iter()
        .filter(|(text, _)| !known.contains(text.as_str()))
        .map(|(text, u)| format!("{} -> {text:?} (catalog/{}/required.tsv)", u.files[0].file_name().unwrap().to_string_lossy(), scan::area_for(&u.areas).unwrap()))
        .collect();
    assert!(missing.is_empty(), "texts without catalog entry (add them to catalog/<area>/required.tsv, English, German, French, Spanish):\n{}", missing.join("\n"));
}

#[test]
fn decimals_follow_the_language() {
    assert_eq!(with_lang(Lang::De, || decimal(12.5, 2)), "12,50");
    assert_eq!(with_lang(Lang::En, || decimal(12.5, 2)), "12.50");
    assert_eq!(with_lang(Lang::Ja, || decimal(3.0, 1)), "3.0");
}

#[test]
fn file_errors_in_the_language_of_the_program() {
    let e = std::io::Error::from(std::io::ErrorKind::NotFound);
    assert_eq!(with_lang(Lang::En, || io_error(&e)), "file or folder not found");
    assert_eq!(with_lang(Lang::De, || io_error(&e)), "Datei oder Ordner nicht gefunden");
}
