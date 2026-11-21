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
    // Every symbol leads to its own page, where the page has the language.
    assert!(english.main.contains(&format!("<h3><a href=\"../../symbols/{}/\">{}</a></h3>", symbol_slug(symbols[0]), symbols[0].name)));
    assert!(german.main.contains(&format!("<h3><a href=\"../../symbols/{}/de.html\">{}</a></h3>", symbol_slug(symbols[0]), openecraft_i18n::translate(symbols[0].name, Lang::De))));
    let korean = openecraft_i18n::with_lang(Lang::Ko, || category_page(Category::Contacts, Lang::Ko));
    assert!(korean.main.contains(&format!("<a class=\"symbol-art\" href=\"../../symbols/{}/\">", symbol_slug(symbols[0]))));
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
    // The overview and the categories in every language; of every symbol two drawings
    // and a page in the required languages.
    assert_eq!(files.len(), (categories().len() + 1) * Lang::ALL.len() + library().len() * (2 + Lang::REQUIRED.len()));
    for path in ["symbols/index.html", "symbols/de.html", "symbols/contacts/index.html", "symbols/contacts/ko.html", "symbols/art/no.svg", "symbols/art/no.dxf", "symbols/no-contact/index.html", "symbols/no-contact/es.html"] {
        assert!(paths.contains(path), "{path}");
    }
    assert!(!paths.contains("symbols/no-contact/ko.html"));
}

#[test]
fn every_symbol_has_a_folder_of_its_own() {
    let folders: BTreeSet<&str> = library().iter().map(symbol_slug).collect();
    assert_eq!(folders.len(), library().len(), "two symbols share a folder");
    for folder in &folders {
        // What the web server takes as the folder of a page (website.conf).
        assert!(!folder.is_empty() && folder.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'), "{folder}");
        assert!(*folder != "art" && !is_category(folder), "{folder}: taken");
        assert_eq!(Langs::of(&format!("{FOLDER}/{folder}/")), Langs::Required);
    }
    assert_eq!(symbol_slug(library().iter().find(|def| def.key == "no").unwrap()), "no-contact");
    assert!(is_category("contacts") && !is_category("no-contact"));
    for category in categories() {
        assert_eq!(Langs::of(&format!("{FOLDER}/{}/", slug(category))), Langs::All);
    }
}

#[test]
fn a_symbol_has_a_page_with_its_drawing_to_download() {
    let def = library().iter().find(|def| def.key == "no").unwrap();
    let english = openecraft_i18n::with_lang(Lang::En, || symbol_page(def, Lang::En));
    assert_eq!(english.folder, "symbols/no-contact");
    assert_eq!(english.langs, Langs::Required);
    assert_eq!(english.title, "NO contact – electrical symbol, SVG and DXF | OpenECraft");
    assert_eq!(english.main.matches("<h1").count(), 1);
    assert!(english.main.contains("<h1>NO contact</h1>"));
    assert!(english.main.contains("href=\"../../symbols/art/no.svg\" download>") && english.main.contains("href=\"../../symbols/art/no.dxf\" download>"));
    assert!(english.main.contains("<a href=\"../../symbols/contacts/\">Contacts</a>") && english.main.contains("<dt>Identifying letter</dt><dd>K</dd>"));
    assert!(english.main.contains(&format!("<dt>Connection points</dt><dd>{}", def.pins.len())));
    // The other symbols of the category, each with its page.
    let others = of(Category::Contacts).len() - 1;
    assert_eq!(english.main.split("class=\"symbol-links\"").nth(1).unwrap().matches("<a href=\"../../symbols/").count(), others);
    assert!(!english.main.contains("href=\"../../symbols/no-contact/\""));
    let german = openecraft_i18n::with_lang(Lang::De, || symbol_page(def, Lang::De));
    assert!(german.main.contains("href=\"../../symbols/contacts/de.html\"") && german.main.contains("href=\"../../de.html#studio\""));
    assert_ne!(german.title, english.title);
    assert_ne!(german.description, english.description);
    // The DXF drawing is the symbol in millimetres.
    let dxf = dxf(def);
    assert!(dxf.starts_with("0\nSECTION\n2\nHEADER\n") && dxf.contains("\n0\nLINE\n") && dxf.ends_with("0\nEOF\n"));
    assert_eq!(dxf_file(def), "art/no.dxf");
}

#[test]
fn every_symbol_page_has_a_title_and_a_description_search_engines_show_whole() {
    for lang in Lang::REQUIRED {
        openecraft_i18n::with_lang(lang, || {
            let mut titles = BTreeSet::new();
            for def in library() {
                let page = symbol_page(def, lang);
                assert!(page.title.chars().count() <= 70, "{}: {}", lang.code(), page.title);
                assert!(DESCRIPTION.contains(&page.description.chars().count()), "{}: {}", lang.code(), page.description);
                assert!(titles.insert(page.title.clone()), "{}: {} twice", lang.code(), page.title);
            }
        });
    }
}
