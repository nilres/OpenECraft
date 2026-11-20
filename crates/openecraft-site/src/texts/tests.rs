use super::*;
use openecraft_i18n::{translation, Lang};
use std::collections::BTreeSet;

#[test]
fn the_table_holds_the_texts_of_the_content_files() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../website/content");
    let pages: BTreeSet<String> = crate::content::read(&dir).unwrap().iter().flat_map(|c| c.texts()).collect();
    let table: BTreeSet<String> = TEXTS.iter().map(|t| (*t).to_string()).collect();
    assert_eq!(table.len(), TEXTS.len(), "a text is listed twice");
    let missing: Vec<&String> = pages.difference(&table).collect();
    assert!(missing.is_empty(), "texts of the content files missing in TEXTS: {missing:#?}");
    let unused: Vec<&String> = table.difference(&pages).collect();
    assert!(unused.is_empty(), "texts in TEXTS no content file shows: {unused:#?}");
    assert!(TEXTS.len() > 150);
}

#[test]
fn the_texts_are_translated_into_the_required_languages() {
    for lang in Lang::REQUIRED.into_iter().filter(|l| *l != Lang::En) {
        let same = TEXTS.iter().filter(|text| translation(text, lang).is_none_or(|t| t == **text)).count();
        assert!(same * 20 < TEXTS.len(), "{}: {same} of {} texts are not translated", lang.code(), TEXTS.len());
    }
}
