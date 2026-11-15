use super::*;

const PAGE: &str = "<html lang=\"en\">\n<head>\n<title data-t>Open</title>\n<meta name=\"description\" content=\"Close\" />\n</head>\n<body>\n<a href=\"#x\" data-t>Close</a> <b data-t>No such text &amp; more</b>\n<input data-t-placeholder=\"Open\" />\n<img src=\"a.png\" alt=\"Close\" data-t-alt=\"Close\" />\n</body>\n</html>\n";

#[test]
fn the_english_page_stays_as_it_is() {
    assert_eq!(translated(PAGE, Lang::En), PAGE);
    assert_eq!(file_name(Lang::En), "index.html");
    assert_eq!(file_name(Lang::De), "de.html");
    let names: std::collections::BTreeSet<String> = Lang::ALL.into_iter().map(file_name).collect();
    assert_eq!(names.len(), Lang::ALL.len());
}

#[test]
fn texts_are_translated_and_keep_their_english() {
    let german = translated(PAGE, Lang::De);
    let open = translation("Open", Lang::De).unwrap();
    assert!(german.starts_with("<html lang=\"de\">") && open != "Open");
    assert!(german.contains(&format!("<title data-t=\"Open\">{open}</title>")), "{german}");
    assert!(german.contains("<a href=\"#x\" data-t=\"Close\">Schließen</a>"));
    assert!(german.contains("<meta name=\"description\" content=\"Schließen\" />"));
    assert!(german.contains(&format!("<input placeholder=\"{open}\" data-t-placeholder=\"Open\" />")));
    assert!(german.contains("alt=\"Schließen\" data-t-alt=\"Close\""));
    // A text the catalog lacks stays, markup around the texts too.
    assert!(german.contains("<b data-t>No such text &amp; more</b>"));
    assert_eq!(german.lines().count(), PAGE.lines().count());
}

#[test]
fn translations_are_written_as_html() {
    let page = "<p data-t>Wires &amp; cables</p><p data-t>a \"b\"</p>";
    let out = rewrite(page, " data-t>", '<', |text| Some(format!(" data-t=\"{}\">{}", attribute(&unescape(text)), escape("x < y & \"z\""))));
    assert_eq!(out, "<p data-t=\"Wires &amp; cables\">x &lt; y &amp; \"z\"</p><p data-t=\"a &quot;b&quot;\">x &lt; y &amp; \"z\"</p>");
    assert_eq!(unescape("&amp;lt;"), "&lt;");
}

#[test]
fn every_language_gets_a_page_of_its_own() {
    for lang in Lang::ALL {
        let page = translated(PAGE, lang);
        assert!(page.contains(&format!("<html lang=\"{}\"", lang.code())));
        assert_eq!(page.matches("<").count(), PAGE.matches("<").count(), "{}", lang.code());
    }
}
