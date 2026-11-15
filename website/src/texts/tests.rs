use super::*;
use std::collections::BTreeSet;

/// The texts a file names between `open` and the next `close`.
fn between<'a>(text: &'a str, open: &str, close: char) -> Vec<&'a str> {
    text.split(open).skip(1).filter_map(|rest| rest.split(close).next()).collect()
}

/// The English texts of the page and its script.
fn page_texts() -> BTreeSet<String> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let html = std::fs::read_to_string(dir.join("index.html")).unwrap();
    let js = std::fs::read_to_string(dir.join("site.js")).unwrap();
    let mut out: BTreeSet<String> = BTreeSet::new();
    out.extend(between(&html, " data-t>", '<').into_iter().map(|t| t.replace("&amp;", "&")));
    out.extend(between(&html, "data-t-placeholder=\"", '"').into_iter().map(String::from));
    out.extend(between(&html, "data-t-alt=\"", '"').into_iter().map(String::from));
    // `t("…")`, not `act("…")` or `print("…")`.
    for before in [' ', '(', '?', ':', '='] {
        out.extend(between(&js, &format!("{before}t(\""), '"').into_iter().map(String::from));
    }
    out
}

#[test]
fn the_table_holds_the_texts_of_the_page() {
    let page = page_texts();
    let table: BTreeSet<String> = TEXTS.iter().map(|t| (*t).to_string()).collect();
    assert_eq!(table.len(), TEXTS.len(), "a text is listed twice");
    let missing: Vec<&String> = page.difference(&table).collect();
    assert!(missing.is_empty(), "texts of the page missing in TEXTS: {missing:#?}");
    let unused: Vec<&String> = table.difference(&page).collect();
    assert!(unused.is_empty(), "texts in TEXTS the page does not show: {unused:#?}");
    assert!(page.len() > 40);
}

#[test]
fn texts_of_the_page_are_single_lines() {
    for text in page_texts() {
        assert!(!text.contains('\n') && text.trim() == text && !text.is_empty(), "{text:?}");
    }
}

#[test]
fn the_page_gets_its_texts_translated() {
    let german = translations(Lang::De);
    let german = german.as_object().unwrap();
    assert_eq!(german.len(), TEXTS.len());
    let same = TEXTS.iter().filter(|t| german[**t] == **t).count();
    assert!(same * 5 < TEXTS.len(), "most texts differ in German ({same} of {} are the same)", TEXTS.len());
}
