use super::*;
use std::collections::BTreeSet;

#[test]
fn every_category_has_a_folder_of_its_own() {
    let folders: BTreeSet<&str> = Category::ALL.into_iter().map(slug).collect();
    assert_eq!(folders.len(), Category::ALL.len());
    for folder in folders {
        assert!(!folder.is_empty() && folder.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'), "{folder}");
        assert_ne!(folder, "art", "the folder of the drawings");
    }
    // Every symbol of the library is on a page.
    let shown: usize = categories().into_iter().map(|c| of(c).len()).sum();
    assert_eq!(shown, library().len());
    assert!(categories().len() > 15);
}

#[test]
fn every_symbol_has_a_drawing() {
    let files: BTreeSet<String> = library().iter().map(drawing_file).collect();
    assert_eq!(files.len(), library().len(), "two symbols share a file");
    for file in &files {
        assert!(file.starts_with("art/") && file.ends_with(".svg") && file[4..file.len() - 4].chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'), "{file}");
    }
    let contact = library().iter().find(|def| def.key == "no").unwrap();
    let svg = drawing(contact);
    assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\"") && svg.contains("<line") && svg.ends_with("</svg>\n"));
    // The drawing is the size of the symbol with white around it.
    let width = contact.bounds.max.x - contact.bounds.min.x + 2.0 * MARGIN;
    assert!(svg.contains(&format!("viewBox=\"0 0 {width:.2} ")), "{}", svg.lines().next().unwrap());
}

#[test]
fn a_category_shows_its_symbols_in_the_language_of_the_page() {
    let english = openecraft_i18n::with_lang(Lang::En, || category_page(Category::Contacts, Lang::En));
    let symbols = of(Category::Contacts);
    assert_eq!(english.folder, "symbols/contacts");
    assert_eq!(english.title, "Contacts – schematic symbols | OpenECraft");
    assert_eq!(english.main.matches("<article class=\"card symbol\"").count(), symbols.len());
    assert!(english.main.contains("id=\"no\"") && english.main.contains("src=\"../../symbols/art/no.svg\""));
    assert!(english.main.contains(&format!("<h2>{} symbols</h2>", symbols.len())));
    assert_eq!(english.main.matches("<h1").count(), 1);
    assert!(english.description.chars().count() < DESCRIPTION.end && english.description.starts_with(&format!("{} electrical symbols", symbols.len())));
    // The other categories, in the language of the page.
    assert_eq!(english.main.matches("<a class=\"button\" href=\"../../symbols/").count(), categories().len() - 1);
    let german = openecraft_i18n::with_lang(Lang::De, || category_page(Category::Contacts, Lang::De));
    assert_eq!(german.title, "Kontakte – Schaltplansymbole | OpenECraft");
    assert!(german.main.contains("href=\"../../symbols/coils-relays/de.html\"") && german.main.contains("href=\"../../de.html#studio\""));
    assert!(german.main.contains(&format!("<h3>{}</h3>", openecraft_i18n::translate(symbols[0].name, Lang::De))));
    assert_ne!(german.description, english.description);
}

#[test]
fn every_page_has_a_title_and_a_description_search_engines_show_whole() {
    let mut titles = BTreeSet::new();
    for lang in Lang::REQUIRED {
        openecraft_i18n::with_lang(lang, || {
            for page in categories().into_iter().map(|c| category_page(c, lang)).chain([overview(lang)]) {
                assert!(page.title.chars().count() <= 70, "{}: {}", lang.code(), page.title);
                assert!(DESCRIPTION.contains(&page.description.chars().count()), "{}: {}", lang.code(), page.description);
                assert!(titles.insert((lang.code(), page.title.clone())), "{}: {} twice", lang.code(), page.title);
            }
        });
    }
}

#[test]
fn the_overview_lists_every_category() {
    let page = openecraft_i18n::with_lang(Lang::En, || overview(Lang::En));
    assert_eq!(page.folder, "symbols");
    for category in categories() {
        assert!(page.main.contains(&format!("href=\"../symbols/{}/\"", slug(category))), "{}", slug(category));
    }
    assert!(page.main.contains(&format!("The {} symbols that come with OpenECraft", library().len())));
}

#[test]
fn the_site_has_the_pages_and_the_drawings() {
    let files = site();
    let paths: BTreeSet<&str> = files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths.len(), files.len());
    assert_eq!(files.len(), (categories().len() + 1) * Lang::ALL.len() + library().len());
    for path in ["symbols/index.html", "symbols/de.html", "symbols/contacts/index.html", "symbols/contacts/ko.html", "symbols/art/no.svg"] {
        assert!(paths.contains(path), "{path}");
    }
}
