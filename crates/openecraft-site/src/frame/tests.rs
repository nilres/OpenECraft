use super::*;
use serde_json::Value;

fn page(kind: Kind, langs: Langs) -> Page {
    Page {
        folder: "symbols/contacts".to_string(),
        title: "Contacts & \"more\" – OpenECraft".to_string(),
        description: "A <description> of the page.".to_string(),
        main: "<section class=\"band\"><h1>Contacts</h1></section>".to_string(),
        kind,
        langs,
    }
}

fn structured(html: &str) -> Value {
    let json = html.split("<script type=\"application/ld+json\">").nth(1).and_then(|s| s.split("</script>").next()).unwrap();
    serde_json::from_str(json).expect("JSON-LD is JSON")
}

#[test]
fn links_lead_to_the_page_in_the_language() {
    let english = Place::of("symbols/contacts", Lang::En);
    let german = Place::of("symbols/contacts/", Lang::De);
    assert_eq!((english.depth, german.depth), (2, 2));
    assert_eq!(english.to("compare/"), "../../compare/");
    assert_eq!(german.to("compare/"), "../../compare/de.html");
    assert_eq!(english.to("#studio"), "../../#studio");
    assert_eq!(german.to("#studio"), "../../de.html#studio");
    assert_eq!(german.to("?example=conveyor#studio"), "../../de.html?example=conveyor#studio");
    assert_eq!(german.to(""), "../../de.html");
    // A file, or a page in English only, is the same from every language.
    assert_eq!(german.file("docs/tutorial/"), "../../docs/tutorial/");
    assert_eq!(Place::of("faq", Lang::Fr).file("site.css"), "../site.css");
}

#[test]
fn the_frame_names_the_page_to_search_engines() {
    let html = html(&page(Kind::Plain, Langs::All), Lang::En);
    assert!(html.starts_with("<!DOCTYPE html>\n<html lang=\"en\">\n"));
    assert!(html.contains("<title>Contacts &amp; &quot;more&quot; – OpenECraft</title>"));
    assert!(html.contains("<meta name=\"description\" content=\"A &lt;description&gt; of the page.\" />"));
    assert!(html.contains("<meta property=\"og:title\" content=\"Contacts &amp; &quot;more&quot; – OpenECraft\" />"));
    assert!(html.contains("<meta property=\"og:description\" content=\"A &lt;description&gt; of the page.\" />"));
    assert!(html.contains("<meta name=\"twitter:card\" content=\"summary_large_image\" />"));
    // The tags with the address come from website-site.sh, between its markers.
    assert!(html.contains("    <!-- site address (installer/docker/website-site.sh) -->\n    <!-- /site address -->\n"));
    for tag in ["rel=\"canonical\"", "og:url", "og:image\"", "hreflang=\"x-default\""] {
        assert!(!html.contains(tag), "{tag} needs the public address");
    }
    let data = structured(&html);
    assert_eq!(data["@type"], "WebPage");
    assert_eq!(data["name"], "Contacts & \"more\" – OpenECraft");
    assert_eq!(data["description"], "A <description> of the page.");
    assert_eq!(data["inLanguage"], "en");
    assert_eq!(data["about"]["@type"], "SoftwareApplication");
    assert_eq!(data["about"]["offers"]["price"], "0");
    assert_eq!(html.matches("<h1").count(), 1);
    assert!(html.ends_with("</body>\n</html>\n"));
}

#[test]
fn the_frame_loads_nothing_from_elsewhere() {
    let plain = html(&page(Kind::Plain, Langs::All), Lang::En);
    let policy = plain.split("http-equiv=\"Content-Security-Policy\" content=\"").nth(1).and_then(|rest| rest.split('"').next()).unwrap();
    assert!(policy.starts_with("default-src 'self'; script-src 'self';") && policy.contains("object-src 'none'"));
    assert!(!policy.contains("http") && !policy.contains('*') && !policy.contains("unsafe"), "{policy}");
    assert!(plain.find("Content-Security-Policy").unwrap() < plain.find("<script").unwrap());
    assert!(!plain.contains("href=\"http") && !plain.contains("src=\"http"));
    assert!(plain.contains("<link rel=\"stylesheet\" href=\"../../site.css\" />") && plain.contains("<script src=\"../../config.js\"></script>\n    <script src=\"../../site.js\" defer></script>"));
    assert!(!plain.contains("tools.js") && !plain.contains("build.js"));
    // Only a tool runs the WebAssembly of the site.
    let tool = html(&page(Kind::Tool, Langs::All), Lang::En);
    assert!(tool.contains("script-src 'self' 'wasm-unsafe-eval';") && !tool.contains("'unsafe-inline'") && !tool.contains("'unsafe-eval'"));
    assert!(tool.contains("<script src=\"../../build.js\"></script>\n    <script src=\"../../tools.js\" defer></script>"));
}

#[test]
fn the_frame_is_in_the_language_of_the_page() {
    let german = openecraft_i18n::with_lang(Lang::De, || html(&page(Kind::Plain, Langs::All), Lang::De));
    assert!(german.starts_with("<!DOCTYPE html>\n<html lang=\"de\">\n"));
    assert!(german.contains("<a href=\"../../compare/de.html\">Vergleich</a>") && german.contains("<a href=\"../../de.html#studio\">Live-Demo</a>"));
    assert!(german.contains("<a class=\"brand\" href=\"../../de.html\">"));
    assert!(german.contains("<a href=\"../../symbols/de.html\" aria-current=\"page\">") || german.contains("<a href=\"../../symbols/de.html\">Symbole</a>"));
    // The documentation and the news are in English only.
    assert!(german.contains("<a href=\"../../docs/\">Dokumentation</a>") && german.contains("<a href=\"../../docs/tutorial/\">Tutorial</a>"));
    assert!(german.contains("<a id=\"privacy-open\" href=\"../../de.html#privacy\">") && german.contains("<a id=\"legal-open\" href=\"../../de.html#legal\" hidden>"));
    assert_eq!(structured(&german)["inLanguage"], "de");
    // Every language of the page, the one shown marked.
    let nav = german.split("<nav class=\"languages\" aria-label=\"Language\">\n").nth(1).and_then(|rest| rest.split("\n    </nav>").next()).unwrap();
    let links: Vec<&str> = nav.lines().map(str::trim).collect();
    assert_eq!(links.len(), Lang::ALL.len());
    assert_eq!(links[0], "<a href=\"./\" hreflang=\"en\" lang=\"en\">English</a>");
    assert_eq!(links[1], "<a href=\"de.html\" hreflang=\"de\" lang=\"de\" aria-current=\"page\">Deutsch</a>");
    // A page in English only names no other language.
    let english = html(&page(Kind::Plain, Langs::English), Lang::En);
    assert!(!english.contains("class=\"languages\""));
}

#[test]
fn questions_are_named_as_such() {
    let questions = vec![("Is it free?".to_string(), "Yes, </script> and \"more\".".to_string())];
    let html = html(&page(Kind::Faq(questions), Langs::All), Lang::En);
    let data = structured(&html);
    assert_eq!(data["@type"], "FAQPage");
    assert_eq!(data["mainEntity"][0]["@type"], "Question");
    assert_eq!(data["mainEntity"][0]["name"], "Is it free?");
    assert_eq!(data["mainEntity"][0]["acceptedAnswer"]["text"], "Yes, </script> and \"more\".");
    assert_eq!(html.matches("</script>").count(), html.matches("<script").count(), "the answer does not end the script");
}

#[test]
fn every_language_gets_its_file() {
    let all = files(Langs::All, |lang| Page { title: lang.code().to_string(), ..page(Kind::Plain, Langs::All) });
    assert_eq!(all.len(), Lang::ALL.len());
    assert_eq!(all[0].path, "symbols/contacts/index.html");
    assert_eq!(all[1].path, "symbols/contacts/de.html");
    assert!(String::from_utf8_lossy(&all[1].bytes).contains("<title>de</title>"));
    assert!(String::from_utf8_lossy(&all[1].bytes).contains(">Vergleich</a>"), "the frame is in the language too");
    let english = files(Langs::English, |_| page(Kind::Plain, Langs::English));
    assert_eq!(english.len(), 1);
}

#[test]
fn a_long_text_is_cut_at_a_word() {
    assert_eq!(shortened("one two  three", 40), "one two three");
    assert_eq!(shortened("one two three four", 12), "one two…");
    assert_eq!(shortened("contacts, switches, lamps", 20), "contacts, switches…");
    assert!(shortened(&"word ".repeat(60), 169).chars().count() <= 169);
    assert!(DESCRIPTION.contains(&169) && !DESCRIPTION.contains(&170));
    // A title that is too long keeps the name of the project.
    assert_eq!(title("Contacts – schematic symbols | OpenECraft".to_string(), "Contacts"), "Contacts – schematic symbols | OpenECraft");
    let long = "Main low-voltage distribution of an industrial plant with everything";
    assert_eq!(title(format!("{long} – example project | OpenECraft"), long), "Main low-voltage distribution of an industrial plant… – OpenECraft");
    assert!(title("x".repeat(80), &"word ".repeat(30)).chars().count() <= TITLE);
}

#[test]
fn the_sections_and_the_bar_have_texts_in_the_required_languages() {
    openecraft_i18n::with_lang(Lang::De, || {
        for (_, text, _) in SECTIONS {
            assert!(section_text(text) != text || text == "FAQ", "{text}");
        }
        for (_, text) in BAR {
            assert_ne!(bar_text(text), text, "{text}");
        }
        assert_eq!(section_text("Command line and MCP"), "Kommandozeile und MCP");
    });
    for lang in Lang::REQUIRED {
        openecraft_i18n::with_lang(lang, || {
            for text in SECTIONS.iter().map(|s| s.1).chain(DOCUMENTS.iter().map(|d| d.1)) {
                assert!(openecraft_i18n::translation(text, lang).is_some() || lang == Lang::En, "{text} in {}", lang.code());
            }
        });
    }
}
