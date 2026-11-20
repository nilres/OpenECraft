use super::*;
use crate::frame::DESCRIPTION;

fn contents() -> Vec<Content> {
    read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../website/content")).unwrap()
}

const FILE: &str = "title: A page – OpenECraft\ndescription: What the page is about.\nkind: faq\n\n<section class=\"band\">\n    <h1 data-t>Heading &amp; more</h1>\n    <details><summary data-t>Is it free?</summary><p data-t>Yes.</p><p data-t>For everyone.</p></details>\n    <a href=\"@/compare/\" data-t>Comparison</a> <a href=\"@/?example=conveyor&amp;page=2#studio\">demo</a> <a href=\"~/docs/tutorial/\">Tutorial</a>\n    <input data-t-placeholder=\"Search\" />\n</section>\n";

#[test]
fn a_content_file_has_a_head_and_a_body() {
    let content = parse("how-to/page", FILE).unwrap();
    assert_eq!((content.folder.as_str(), content.title.as_str(), content.kind.as_str()), ("how-to/page", "A page – OpenECraft", "faq"));
    assert_eq!(content.description, "What the page is about.");
    assert!(content.body.starts_with("<section class=\"band\">") && content.body.ends_with("</section>"));
    assert_eq!(parse("x", &FILE.replace('\n', "\r\n")).unwrap().body, content.body);
    let texts: Vec<String> = content.texts().into_iter().collect();
    assert_eq!(texts, ["A page – OpenECraft", "Comparison", "For everyone.", "Heading & more", "Is it free?", "Search", "What the page is about.", "Yes."]);
    // What a file must have.
    assert!(parse("x", "title: T\n\n<p></p>").is_err(), "no description");
    assert!(parse("x", "title: T\ndescription: D\nkind: other\n\n<p></p>").is_err(), "an unknown kind");
    assert!(parse("x", "title: T\ndescription: D").is_err(), "no body");
}

#[test]
fn links_lead_to_the_files_of_the_language() {
    let place = Place::of("how-to/page", Lang::De);
    let html = links("<a href=\"@/compare/\">a</a><a href=\"@/#studio\">b</a><a href=\"~/docs/\">c</a><img src=\"~/images/a.webp\" /><a href=\"@/?example=x&amp;page=2#studio\">d</a>", place);
    assert_eq!(html, "<a href=\"../../compare/de.html\">a</a><a href=\"../../de.html#studio\">b</a><a href=\"../../docs/\">c</a><img src=\"../../images/a.webp\" /><a href=\"../../de.html?example=x&amp;page=2#studio\">d</a>");
    assert_eq!(links("<p>a@/b ~/c</p>", place), "<p>a@/b ~/c</p>", "only addresses");
}

#[test]
fn a_page_is_written_in_the_language_with_its_questions() {
    let content = parse("faq", FILE).unwrap();
    let english = content.page(Lang::En);
    assert_eq!(english.kind, Kind::Faq(vec![("Is it free?".to_string(), "Yes. For everyone.".to_string())]));
    assert!(english.main.contains("href=\"../compare/\"") && english.main.contains("href=\"../docs/tutorial/\""));
    let german = content.page(Lang::De);
    assert!(german.main.contains("href=\"../compare/de.html\" data-t=\"Comparison\">Vergleich</a>"));
    assert!(german.main.contains("href=\"../docs/tutorial/\""), "the documentation is in English only");
    assert_eq!(german.folder, "faq");
}

#[test]
fn the_pages_of_the_site_are_complete() {
    let contents = contents();
    let folders: Vec<&str> = contents.iter().map(|c| c.folder.as_str()).collect();
    for folder in ["faq", "how-to", "how-to/wire-list", "tools", "tools/voltage-drop-calculator", "compare/eplan-alternative"] {
        assert!(folders.contains(&folder), "{folder}");
    }
    // The pages the links of the content files may lead to.
    let site: Vec<String> = folders.iter().map(|f| format!("{f}/")).chain(["", "compare/", "panel-builders/", "electricians/", "education/", "symbols/", "examples/"].map(String::from)).collect();
    let mut titles = std::collections::BTreeSet::new();
    for content in &contents {
        let name = &content.folder;
        assert!(name.split('/').count() <= 2 && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '/'), "{name}");
        assert!(content.title.contains("OpenECraft") && content.title.chars().count() <= 70, "{name}: {}", content.title);
        assert!(DESCRIPTION.contains(&content.description.chars().count()), "{name}: {} characters", content.description.chars().count());
        assert!(titles.insert(&content.title), "{name}: its title twice");
        assert_eq!(content.body.matches("<h1").count(), 1, "{name}");
        assert!(!content.body.contains("href=\"http") && !content.body.contains("src=\"http") && !content.body.contains("<script"), "{name}");
        for text in content.texts() {
            assert!(!text.contains('\n') && text.trim() == text && !text.is_empty(), "{name}: {text:?}");
        }
        for target in between(&content.body, "=\"@/", "\"") {
            let folder = &target[..target.find(['?', '#']).unwrap_or(target.len())];
            assert!(site.iter().any(|page| page == folder), "{name}: no page {folder}");
        }
        for target in between(&content.body, "href=\"~/", "\"") {
            assert!(target.starts_with("docs/") || target.starts_with("news/"), "{name}: {target}");
            let document = target.trim_start_matches("docs/").trim_end_matches('/');
            assert!(document.is_empty() || target.starts_with("news/") || crate::docs::DOCUMENTS.iter().any(|file| crate::docs::slug(file) == document), "{name}: no document {target}");
        }
        // A tool names the function of the module it runs, and only a tool does.
        assert_eq!(content.kind == "tool", content.body.contains(" data-tool=\""), "{name}");
        if content.kind == "tool" {
            for part in ["class=\"tool-form\"", "class=\"tool-wait\" hidden", "class=\"tool-failed\" hidden", "class=\"tool-error\" hidden", "class=\"tool-out\" hidden"] {
                assert!(content.body.contains(part), "{name}: {part}");
            }
        }
    }
    // The sections of the footer that are content files.
    for (folder, _, _) in crate::frame::SECTIONS {
        let generated = ["symbols/", "examples/", "docs/", "news/"].contains(&folder);
        assert!(generated || site.iter().any(|page| page == folder), "{folder}");
    }
}

#[test]
fn the_questions_are_answered_in_every_required_language() {
    let faq = contents().into_iter().find(|c| c.folder == "faq").unwrap();
    assert_eq!(faq.kind, "faq");
    let Kind::Faq(english) = faq.page(Lang::En).kind else { panic!("questions") };
    assert!(english.len() >= 8 && english.iter().all(|(q, a)| q.ends_with('?') && a.len() > 30));
    for lang in Lang::REQUIRED.into_iter().filter(|l| *l != Lang::En) {
        let page = faq.page(lang);
        let Kind::Faq(questions) = &page.kind else { panic!("questions") };
        assert_eq!(questions.len(), english.len());
        assert!(questions.iter().zip(&english).all(|(t, e)| t.0 != e.0 && t.1 != e.1), "{}", lang.code());
        assert_ne!(page.title, faq.title);
        let left = between(&page.main, " data-t>", "<");
        assert!(left.is_empty(), "{}: not translated: {left:?}", lang.code());
    }
}

#[test]
fn every_page_is_written_in_every_language() {
    let contents = contents();
    let files = site(&contents);
    assert_eq!(files.len(), contents.len() * Lang::ALL.len());
    assert!(files.iter().any(|f| f.path == "faq/index.html") && files.iter().any(|f| f.path == "how-to/wire-list/ja.html"));
    let german = String::from_utf8(files.iter().find(|f| f.path == "tools/voltage-drop-calculator/de.html").unwrap().bytes.clone()).unwrap();
    assert!(german.contains("<html lang=\"de\">") && german.contains("data-tool=\"voltage_drop\"") && german.contains("'wasm-unsafe-eval'"));
    assert!(!german.contains("=\"@/") && !german.contains("=\"~/"));
}
