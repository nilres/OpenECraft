use super::*;

fn page(title: &str, main: &str, script: &str) -> String {
    format!(
        "<html>\n<head>\n    <title>{title}</title>\n    <meta name=\"description\" content=\"About {title}.\" />\n    <!-- site address (installer/docker/website-site.sh) -->\n    <!-- /site address -->\n    <script src=\"{script}\"></script>\n</head>\n<body>\n<main id=\"top\">\n{main}\n</main>\n<footer>{script}</footer>\n</body>\n</html>\n"
    )
}

#[test]
fn the_pages_of_the_sitemap_are_those_of_a_language_with_a_place_for_the_address() {
    let html = page("Contacts", "<h1>Contacts</h1>", "a.js");
    for file in ["index.html", "de.html", "compare/index.html", "symbols/contacts/ko.html"] {
        assert!(is_page(file, &html), "{file}");
    }
    // Not the page for a missing address, a file three folders deep or one of another kind.
    for file in ["404.html", "404.de.html", "a/b/c/index.html", "site.js", "news/feed.xml", "THIRD-PARTY-LICENSES.html"] {
        assert!(!is_page(file, &html), "{file}");
    }
    assert!(!is_page("editor/index.html", "<html><head></head></html>"));
}

#[test]
fn the_hash_follows_what_the_page_says_and_not_its_frame() {
    let hash = fingerprint(&page("Contacts", "<h1>Contacts</h1>", "a.js"));
    assert_eq!(hash.len(), 16);
    assert_eq!(hash, fingerprint(&page("Contacts", "<h1>Contacts</h1>", "module-0123abcd.js")));
    assert_eq!(hash, fingerprint(&page("Contacts", "<h1>Contacts</h1>", "a.js").replace('\n', "\r\n")));
    assert_ne!(hash, fingerprint(&page("Contacts", "<h1>Contacts and coils</h1>", "a.js")));
    assert_ne!(hash, fingerprint(&page("Coils", "<h1>Contacts</h1>", "a.js")));
    // The same on every machine: the file of one build is read by the next.
    assert_eq!(fingerprint("<title>T</title><main>M</main>"), fingerprint("<title>T</title><main>M</main>"));
    assert_ne!(fingerprint("<title>TM</title><main></main>"), fingerprint("<title>T</title><main>M</main>"));
}

#[test]
fn a_day_is_written_as_a_sitemap_names_it() {
    assert_eq!(day(0), "1970-01-01");
    assert_eq!(day(86_399), "1970-01-01");
    assert_eq!(day(951_782_400), "2000-02-29");
    assert_eq!(day(951_868_800), "2000-03-01");
    assert_eq!(day(1_795_824_000), "2026-11-28");
    assert_eq!(day(1_798_761_599), "2026-12-31");
    assert_eq!(day(1_798_761_600), "2027-01-01");
}

#[test]
fn a_page_keeps_its_day_until_it_says_something_else() {
    let pages = |contacts: &str| vec![("symbols/index.html".to_string(), page("Symbols", "<h1>Symbols</h1>", "a.js")), ("index.html".to_string(), page("OpenECraft", contacts, "a.js"))];
    // The first build knows no day.
    let first = file(&pages("<h1>One</h1>"), "", "2026-11-20");
    let rows: Vec<&str> = first.lines().filter(|line| !line.starts_with('#')).collect();
    assert_eq!(rows.len(), 2);
    assert!(rows[0].starts_with("index.html\t") && rows[0].ends_with('\t') && rows[1].starts_with("symbols/index.html\t"), "{rows:?}");
    assert!(first.contains("\n# built 2026-11-20\n") && changed(&first).is_empty());
    // Nothing changed: nothing gets a day.
    let same = file(&pages("<h1>One</h1>"), &first, "2026-11-21");
    assert_eq!(same.replace("2026-11-21", "2026-11-20"), first);
    // A changed page gets the day of the build, and so does a new one.
    let mut later = pages("<h1>Two</h1>");
    later.push(("faq/index.html".to_string(), page("FAQ", "<h1>FAQ</h1>", "a.js")));
    let second = file(&later, &same, "2026-11-22");
    assert!(second.lines().any(|line| line.starts_with("index.html\t") && line.ends_with("\t2026-11-22")));
    assert!(second.lines().any(|line| line.starts_with("faq/index.html\t") && line.ends_with("\t2026-11-22")));
    assert!(second.lines().any(|line| line.starts_with("symbols/index.html\t") && line.ends_with('\t')));
    assert_eq!(changed(&second), ["faq/index.html", "index.html"]);
    // It keeps the day in the builds after, also with the line ends of Windows.
    let third = file(&later, &second.replace('\n', "\r\n"), "2026-11-30");
    assert_eq!(third.replace("# built 2026-11-30", "# built 2026-11-22"), second);
    assert!(changed(&third).is_empty());
    // A day that is none is not passed on.
    let odd = file(&pages("<h1>One</h1>"), &first.replace("\t\n", "\tyesterday\n"), "2026-11-21");
    assert!(!odd.contains("yesterday"));
}
