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

/// Every block of structured data of a page.
fn all_structured(html: &str) -> Vec<Value> {
    html.split("<script type=\"application/ld+json\">").skip(1).map(|s| serde_json::from_str(s.split("</script>").next().unwrap()).expect("JSON-LD is JSON")).collect()
}

/// The way to a page as its structured data names it: address and name of every step.
fn way(html: &str) -> Vec<(String, String)> {
    let data = all_structured(html);
    let list = data.iter().find(|block| block["@type"] == "BreadcrumbList").expect("a way to the page");
    let items = list["itemListElement"].as_array().unwrap();
    assert!(items.iter().enumerate().all(|(i, item)| item["@type"] == "ListItem" && item["position"] == i + 1));
    items.iter().map(|item| (item["item"].as_str().unwrap().to_string(), item["name"].as_str().unwrap().to_string())).collect()
}

fn step(address: &str, name: &str) -> (String, String) {
    (address.to_string(), name.to_string())
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
    // A file is the same from every language.
    assert_eq!(Place::of("faq", Lang::Fr).file("site.css"), "../site.css");
    // A page that is not written in the language of this one is linked in English:
    // the documentation and the symbols have four languages, the news one.
    let korean = Place::of("faq", Lang::Ko);
    assert_eq!(german.to("docs/tutorial/#9-berichte"), "../../docs/tutorial/de.html#9-berichte");
    assert_eq!(korean.to("docs/tutorial/"), "../docs/tutorial/");
    assert_eq!(german.to("news/"), "../../news/");
    assert_eq!(korean.to("symbols/contacts/"), "../symbols/contacts/ko.html");
    assert_eq!(korean.to("symbols/no-contact/"), "../symbols/no-contact/");
    assert_eq!(german.to("symbols/no-contact/"), "../../symbols/no-contact/de.html");
    assert_eq!(korean.to("symbols/"), "../symbols/ko.html");
}

#[test]
fn the_languages_of_a_page_follow_its_folder() {
    assert_eq!(Langs::of(""), Langs::All);
    assert_eq!(Langs::of("compare/"), Langs::All);
    assert_eq!(Langs::of("reference/ip-ratings/"), Langs::All);
    assert_eq!(Langs::of("symbols/"), Langs::All);
    assert_eq!(Langs::of("symbols/contacts"), Langs::All);
    assert_eq!(Langs::of("symbols/no-contact/"), Langs::Required);
    assert_eq!(Langs::of("docs/"), Langs::Required);
    assert_eq!(Langs::of("docs/tutorial/"), Langs::Required);
    assert_eq!(Langs::of("news/0-1-0/"), Langs::English);
    assert_eq!(Langs::Required.list(), Lang::REQUIRED.to_vec());
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
fn the_structured_data_names_the_site_and_the_way_to_the_page() {
    let english = html(&page(Kind::Plain, Langs::All), Lang::En);
    let data = structured(&english);
    // The addresses of the site start at its folder, where website-site.sh puts the
    // public address.
    assert_eq!(data["isPartOf"], serde_json::json!({ "@type": "WebSite", "name": "OpenECraft", "url": "../../" }));
    assert_eq!(data["about"]["url"], "../../");
    assert!(data["primaryImageOfPage"].is_null());
    assert_eq!(way(&english), [step("../../", "OpenECraft"), step("../../symbols/", "Symbols"), step("../../symbols/contacts/", "Contacts")]);
    // In the language of the page, to the pages of that language.
    let german = openecraft_i18n::with_lang(Lang::De, || html(&page(Kind::Plain, Langs::All), Lang::De));
    assert_eq!(way(&german), [step("../../de.html", "OpenECraft"), step("../../symbols/de.html", "Symbole"), step("../../symbols/contacts/de.html", "Contacts")]);
    // A page right below the main one, and one below a page written by hand.
    let main = "<h1 data-t=\"Questions\">Questions &amp; <em>answers</em></h1>".to_string();
    let faq = html(&Page { folder: "faq".into(), main: main.clone(), ..page(Kind::Plain, Langs::All) }, Lang::En);
    assert_eq!(way(&faq), [step("../", "OpenECraft"), step("../faq/", "FAQ")]);
    assert_eq!(structured(&faq)["isPartOf"]["url"], "../");
    let product = html(&Page { folder: "compare/eplan-alternative".into(), main, ..page(Kind::Plain, Langs::All) }, Lang::En);
    assert_eq!(way(&product), [step("../../", "OpenECraft"), step("../../compare/", "Comparison"), step("../../compare/eplan-alternative/", "Questions & answers")]);
    // Every address of the site is the value of a name, on a line of its own.
    let blocks: String = english.split("<script type=\"application/ld+json\">").skip(1).filter_map(|s| s.split("</script>").next()).collect();
    assert_eq!(blocks.matches("\"../../").count(), blocks.matches("\": \"../../").count());
    assert!(blocks.lines().all(|line| line.matches("\": \"../../").count() <= 1));
    assert_eq!(english.matches("</script>").count(), english.matches("<script").count());
}

#[test]
fn the_page_of_a_symbol_names_its_category_and_its_drawing() {
    let def = crate::symbols::by_folder("no-contact").expect("the NO contact");
    let english = html(&crate::symbols::symbol_page(def, Lang::En), Lang::En);
    assert_eq!(way(&english), [step("../../", "OpenECraft"), step("../../symbols/", "Symbols"), step("../../symbols/contacts/", "Contacts"), step("../../symbols/no-contact/", "NO contact")]);
    let image = &structured(&english)["primaryImageOfPage"];
    assert_eq!(image["@type"], "ImageObject");
    assert_eq!(image["name"], "Symbol: NO contact");
    assert_eq!(image["contentUrl"], format!("../../symbols/{}", crate::symbols::drawing_file(def)));
    // What makes a picture one that may be used: its license and where it is from.
    assert_eq!(image["license"], "../../LICENSE-MIT");
    assert_eq!(image["acquireLicensePage"], "../../symbols/no-contact/");
    assert_eq!(image["creditText"], "OpenECraft");
    let german = openecraft_i18n::with_lang(Lang::De, || html(&crate::symbols::symbol_page(def, Lang::De), Lang::De));
    assert_eq!(structured(&german)["primaryImageOfPage"]["name"], "Symbol: Schließer");
    assert_eq!(structured(&german)["primaryImageOfPage"]["acquireLicensePage"], "../../symbols/no-contact/de.html");
    assert_eq!(way(&german)[2], step("../../symbols/contacts/de.html", "Kontakte"));
    // The page of a category has no drawing of its own.
    assert!(crate::symbols::by_folder("contacts").is_none());
}

#[test]
fn the_page_for_a_missing_address_is_kept_out_of_the_index() {
    let missing = html(&Page { folder: String::new(), ..page(Kind::Missing, Langs::All) }, Lang::En);
    assert!(missing.contains("<meta name=\"robots\" content=\"noindex\" />") && !missing.contains("application/ld+json"));
    assert!(missing.contains("<link rel=\"stylesheet\" href=\"site.css\" />"));
    let plain = html(&page(Kind::Plain, Langs::All), Lang::En);
    assert!(!plain.contains("name=\"robots\""));
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
    // The documentation has a German page, the news are in English only.
    assert!(german.contains("<a href=\"../../docs/de.html\">Dokumentation</a>") && german.contains("<a href=\"../../docs/tutorial/de.html\">Tutorial</a>"));
    assert!(german.contains("<a href=\"../../news/\">Neuigkeiten</a>"));
    assert!(german.contains("<a id=\"privacy-open\" href=\"../../de.html#privacy\">") && german.contains("<a id=\"legal-open\" href=\"../../de.html#legal\" hidden>"));
    assert_eq!(structured(&german)["inLanguage"], "de");
    // Every language of the page, the one shown marked.
    let nav = german.split("<nav class=\"languages\" aria-label=\"Language\">\n").nth(1).and_then(|rest| rest.split("\n    </nav>").next()).unwrap();
    let links: Vec<&str> = nav.lines().map(str::trim).collect();
    assert_eq!(links.len(), Lang::ALL.len());
    assert_eq!(links[0], "<a href=\"./\" hreflang=\"en\" lang=\"en\">English</a>");
    assert_eq!(links[1], "<a href=\"de.html\" hreflang=\"de\" lang=\"de\" aria-current=\"page\">Deutsch</a>");
    // A page in four languages names those, a page in English only no other language.
    let four = html(&page(Kind::Plain, Langs::Required), Lang::En);
    let nav = four.split("<nav class=\"languages\" aria-label=\"Language\">\n").nth(1).and_then(|rest| rest.split("\n    </nav>").next()).unwrap();
    assert_eq!(nav.lines().count(), Lang::REQUIRED.len());
    assert!(nav.contains("hreflang=\"es\"") && !nav.contains("hreflang=\"ko\""));
    let english = html(&page(Kind::Plain, Langs::English), Lang::En);
    assert!(!english.contains("class=\"languages\""));
    // In a language the documentation is not written in, the footer links the English one.
    let korean = openecraft_i18n::with_lang(Lang::Ko, || html(&page(Kind::Plain, Langs::All), Lang::Ko));
    assert!(korean.contains("<a href=\"../../docs/\">") && korean.contains("<a href=\"../../symbols/ko.html\""));
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
        for (_, text) in SECTIONS {
            // Two are the same word in German.
            assert!(section_text(text) != text || ["FAQ", "Videos"].contains(&text), "{text}");
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
