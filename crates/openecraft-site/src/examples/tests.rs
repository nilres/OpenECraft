use super::*;
use std::collections::BTreeSet;

fn example(key: &str) -> &'static Example {
    EXAMPLES.iter().find(|e| e.key == key).unwrap()
}

#[test]
fn every_example_has_a_folder_of_its_own() {
    let folders: BTreeSet<String> = EXAMPLES.iter().map(slug).collect();
    assert_eq!(folders.len(), EXAMPLES.len());
    for folder in folders {
        assert!(folder.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'), "{folder}");
    }
    assert_eq!(plain_title("4 · Conveyor with PLC"), "Conveyor with PLC");
    assert_eq!(plain_title("Conveyor"), "Conveyor");
}

#[test]
fn an_example_shows_its_sheets_its_pages_and_its_link_into_the_demo() {
    let lamp = example("lamp_circuit");
    let project = (lamp.build)();
    let english = openecraft_i18n::with_lang(Lang::En, || page(lamp, Lang::En));
    assert_eq!(english.folder, "examples/lamp-circuit");
    assert_eq!(english.title, "First circuit: push buttons and lamps – example project | OpenECraft");
    assert!(english.main.contains("<h1>First circuit: push buttons and lamps</h1>"));
    assert!(english.main.contains("href=\"../../?example=lamp_circuit#studio\"") && english.main.contains("href=\"lamp-circuit.pdf\" download"));
    assert_eq!(english.main.matches("<img loading=\"lazy\" src=\"page-").count(), project.pages.len().min(SHEETS));
    assert_eq!(english.main.matches("<tr><th scope=\"row\">").count(), project.pages.len());
    assert_eq!(english.main.matches("<a class=\"button\" href=\"../../examples/").count(), EXAMPLES.len() - 1, "the other examples");
    assert_eq!(english.main.matches("<h1").count(), 1);
    let german = openecraft_i18n::with_lang(Lang::De, || page(lamp, Lang::De));
    assert!(german.main.contains("href=\"../../de.html?example=lamp_circuit#studio\"") && german.main.contains("href=\"../../examples/conveyor/de.html\""));
    assert!(german.title.ends_with("– Beispielprojekt | OpenECraft") && german.description != english.description);
}

#[test]
fn every_page_has_a_title_and_a_description_search_engines_show_whole() {
    let mut titles = BTreeSet::new();
    for lang in Lang::REQUIRED {
        openecraft_i18n::with_lang(lang, || {
            for page in EXAMPLES.iter().map(|e| page(e, lang)).chain([overview(lang)]) {
                assert!(page.title.chars().count() <= 70, "{}: {}", lang.code(), page.title);
                assert!(DESCRIPTION.contains(&page.description.chars().count()), "{}: {}", lang.code(), page.description);
                assert!(titles.insert((lang.code(), page.title.clone())), "{}: {} twice", lang.code(), page.title);
            }
        });
    }
}

#[test]
fn the_overview_lists_every_example() {
    let page = openecraft_i18n::with_lang(Lang::En, || overview(Lang::En));
    for example in &EXAMPLES {
        assert!(page.main.contains(&format!("href=\"../examples/{}/\"", slug(example))), "{}", example.key);
    }
    assert!(page.main.contains(&format!("{} projects that come with OpenECraft", EXAMPLES.len())));
}

#[test]
fn the_sheets_and_the_pdf_are_drawn_by_the_exports() {
    let plugins = Plugins::new(Vec::<String>::new());
    let lamp = example("lamp_circuit");
    let files = drawings(lamp, &plugins);
    let project = (lamp.build)();
    assert_eq!(files.len(), project.pages.len().min(SHEETS) + 1);
    assert_eq!(files[0].path, "examples/lamp-circuit/page-1.svg");
    assert!(files[0].bytes.starts_with(b"<svg "));
    let pdf = files.last().unwrap();
    assert_eq!(pdf.path, "examples/lamp-circuit/lamp-circuit.pdf");
    assert!(pdf.bytes.starts_with(b"%PDF"));
}
