use super::*;

fn text(file: &File) -> String {
    String::from_utf8(file.bytes.clone()).unwrap()
}

#[test]
fn every_language_has_a_page_beside_the_main_one() {
    let files = site();
    assert_eq!(files.len(), Lang::ALL.len());
    assert_eq!((files[0].path.as_str(), files[1].path.as_str()), ("404.html", "404.de.html"));
    assert!(files.iter().all(|file| !file.path.contains('/')));
    let german = text(&files[1]);
    assert!(german.starts_with("<!DOCTYPE html>\n<html lang=\"de\">\n"));
    assert!(german.contains("<h1>Seite nicht gefunden</h1>") && german.contains("<title>Seite nicht gefunden – OpenECraft</title>"));
    for lang in Lang::REQUIRED.into_iter().filter(|lang| *lang != Lang::En) {
        for english in ["Page not found", "Page not found – OpenECraft", "Error 404", "Start page"] {
            assert!(openecraft_i18n::translation(english, lang).is_some(), "{english} in {}", lang.code());
        }
    }
}

#[test]
fn the_page_is_no_page_of_the_index_and_leads_on() {
    let english = text(&site()[0]);
    assert!(english.contains("<meta name=\"robots\" content=\"noindex\" />"));
    // Nothing a search engine would take for a page of the site, but the place where
    // website-site.sh names the folder of the site.
    assert!(!english.contains("application/ld+json") && !english.contains("rel=\"canonical\""));
    assert!(english.contains("    <!-- site address (installer/docker/website-site.sh) -->\n    <!-- /site address -->\n"));
    assert!(english.find("<!-- site address ").unwrap() < english.find("<link rel=\"stylesheet\"").unwrap(), "the folder is named before the first address");
    assert_eq!(english.matches("<h1").count(), 1);
    // Its addresses are those of a page beside the main one.
    assert!(english.contains("<link rel=\"stylesheet\" href=\"site.css\" />") && english.contains("<script src=\"site.js\" defer></script>"));
    assert!(english.contains("<a class=\"button primary\" href=\"\">Start page</a>") && english.contains("<a class=\"button\" href=\"docs/\">Documentation</a>"));
    assert!(!english.contains("href=\"../") && !english.contains("src=\"../"));
    // In another language they lead to the pages of that language, where there are.
    let files = site();
    let (german, korean) = (text(&files[1]), text(files.iter().find(|file| file.path == "404.ko.html").unwrap()));
    assert!(german.contains("<a class=\"button primary\" href=\"de.html\">Startseite</a>") && german.contains("href=\"docs/de.html\">Dokumentation</a>"));
    assert!(korean.contains("href=\"ko.html\"") && korean.contains("href=\"docs/\"") && korean.contains("href=\"symbols/ko.html\""));
    // The languages of the footer are the main pages.
    assert!(german.contains("<a href=\"./\" hreflang=\"en\" lang=\"en\">English</a>") && german.contains("<a href=\"fr.html\" hreflang=\"fr\" lang=\"fr\">"));
    for (folder, _) in LEADS {
        assert!(crate::frame::SECTIONS.iter().any(|section| section.0 == folder), "{folder}");
    }
}
