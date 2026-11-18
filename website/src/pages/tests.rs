use super::*;
use openecraft_i18n::Lang;
use serde_json::Value;
use std::collections::BTreeSet;

fn read(path: &str) -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("{path}: {e}")).replace("\r\n", "\n")
}

fn page(folder: &str) -> String {
    read(&format!("{folder}/index.html"))
}

/// What stands between each `open` and the next `close`.
fn between<'a>(text: &'a str, open: &str, close: char) -> Vec<&'a str> {
    text.split(open).skip(1).filter_map(|rest| rest.split(close).next()).collect()
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&")
}

/// The `content` of the first `<meta` with `name`.
fn meta(html: &str, name: &str) -> String {
    let tag = between(html, &format!("<meta {name} content=\""), '"');
    unescape(tag.first().unwrap_or_else(|| panic!("no meta {name}")))
}

fn title(html: &str) -> String {
    unescape(between(html, "<title data-t>", '<').first().expect("a title"))
}

/// The English texts a page shows and names in its head.
fn texts(html: &str) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = between(html, " data-t>", '<').into_iter().map(unescape).collect();
    out.insert(meta(html, "name=\"description\""));
    out
}

#[test]
fn the_table_holds_the_texts_of_the_pages() {
    let pages: BTreeSet<String> = PAGES.iter().flat_map(|p| texts(&page(p))).collect();
    let table: BTreeSet<String> = TEXTS.iter().map(|t| (*t).to_string()).collect();
    assert_eq!(table.len(), TEXTS.len(), "a text is listed twice");
    let missing: Vec<&String> = pages.difference(&table).collect();
    assert!(missing.is_empty(), "texts of the pages missing in TEXTS: {missing:#?}");
    let unused: Vec<&String> = table.difference(&pages).collect();
    assert!(unused.is_empty(), "texts in TEXTS no page shows: {unused:#?}");
    for text in &pages {
        assert!(!text.contains('\n') && text.trim() == text && !text.is_empty(), "{text:?}");
    }
}

#[test]
fn every_page_has_a_title_and_a_description_of_its_own() {
    let main = read("index.html");
    let mut titles = BTreeSet::from([title(&main)]);
    let mut descriptions = BTreeSet::from([meta(&main, "name=\"description\"")]);
    for folder in PAGES {
        let html = page(folder);
        let (title, description) = (title(&html), meta(&html, "name=\"description\""));
        assert!(title.contains("OpenECraft") && title.len() <= 70, "{folder}: {title}");
        assert!(description.len() > 80 && description.len() < 170, "{folder}: a description search engines show whole");
        assert_eq!(meta(&html, "property=\"og:title\""), title, "{folder}");
        assert_eq!(meta(&html, "property=\"og:description\""), description, "{folder}");
        assert_eq!(meta(&html, "name=\"twitter:card\""), "summary_large_image");
        assert!(titles.insert(title) && descriptions.insert(description), "{folder}: title and description are its own");
        assert!(html.starts_with("<!DOCTYPE html>\n<html lang=\"en\">\n"), "{folder}");
        assert_eq!(html.matches("<h1").count(), 1, "{folder}");
        // The tags with the address come from website-site.sh, never from the page itself.
        for tag in ["rel=\"canonical\"", "og:url", "og:image\"", "twitter:image", "hreflang=\"x-default\""] {
            assert!(!html.contains(tag), "{folder}: {tag} needs the public address");
        }
        assert!(html.contains("    <!-- site address (installer/docker/website-site.sh) -->\n    <!-- /site address -->\n"), "{folder}");
    }
}

#[test]
fn every_page_describes_itself_as_structured_data() {
    for folder in PAGES {
        let html = page(folder);
        let json = between(&html, "<script type=\"application/ld+json\">", '<')[0];
        let data: Value = serde_json::from_str(json).unwrap_or_else(|e| panic!("{folder}: {e}"));
        assert_eq!(data["@context"], "https://schema.org");
        assert_eq!(data["@type"], "WebPage");
        assert_eq!(data["name"].as_str(), Some(title(&html).as_str()), "{folder}");
        assert_eq!(data["description"].as_str(), Some(meta(&html, "name=\"description\"").as_str()), "{folder}");
        assert_eq!(data["about"]["@type"], "SoftwareApplication");
        assert_eq!(data["about"]["name"], "OpenECraft");
        assert_eq!(data["about"]["offers"]["price"], "0");
        assert_eq!(data["audience"].is_null(), folder == "compare", "{folder}: the pages for an audience name it");
    }
}

#[test]
fn the_pages_load_nothing_from_elsewhere() {
    let settings: BTreeSet<&str> = crate::links::SETTINGS.iter().map(|s| s.0).collect();
    for folder in PAGES {
        let html = page(folder);
        let policy = between(&html, "http-equiv=\"Content-Security-Policy\" content=\"", '"')[0];
        assert!(html.find("Content-Security-Policy").unwrap() < html.find("<script").unwrap(), "{folder}: the policy comes before the scripts");
        assert!(policy.starts_with("default-src 'self';") && policy.contains("object-src 'none'"), "{folder}: {policy}");
        assert!(!policy.contains("http") && !policy.contains('*') && !policy.contains("unsafe"), "{folder}: {policy}");
        assert!(!html.contains("href=\"http") && !html.contains("src=\"http"), "{folder}: addresses belong into config.js");
        // The settings, the script and the style sheet of the main page, beside it.
        let (config, script) = (html.find("<script src=\"../config.js\"></script>").unwrap(), html.find("<script src=\"../site.js\" defer></script>").unwrap());
        assert!(config < script, "{folder}");
        assert!(html.contains("<link rel=\"stylesheet\" href=\"../site.css\" />") && html.contains("<link rel=\"icon\" href=\"../icon.svg\""), "{folder}");
        let used: BTreeSet<&str> = between(&html, "data-link=\"", '"').into_iter().collect();
        assert!(used.is_subset(&settings), "{folder}: {used:?}");
    }
    // The build copies the pages and the style sheet as they are.
    let main = read("index.html");
    assert!(main.contains("<link data-trunk rel=\"copy-file\" href=\"site.css\" />"));
    for folder in PAGES {
        assert!(main.contains(&format!("<link data-trunk rel=\"copy-dir\" href=\"{folder}\" />")), "index.html copies {folder}");
    }
}

#[test]
fn the_pages_link_each_other_and_the_main_page() {
    let main = read("index.html");
    for folder in PAGES {
        assert!(main.contains(&format!(" data-page href=\"{folder}/\" data-t>")), "index.html links {folder}");
        let html = page(folder);
        assert!(html.contains("class=\"brand\" data-page href=\"../\""), "{folder}: the main page");
        for other in PAGES {
            let own = if other == folder { " aria-current=\"page\"" } else { "" };
            assert!(html.contains(&format!("<a data-page href=\"../{other}/\"{own} data-t>")), "{folder} links {other}");
        }
        // To the dialogs of the main page, in the language of the page (site.js).
        for name in ["licenses", "privacy", "legal"] {
            assert!(html.contains(&format!("<a id=\"{name}-open\" data-page href=\"../#{name}\"")), "{folder}: {name}");
        }
        assert!(html.contains("<a id=\"legal-open\" data-page href=\"../#legal\" hidden data-t>"), "{folder}: the link waits for a name");
        assert!(html.contains("data-page href=\"../#studio\""), "{folder}: the live demo");
    }
    let script = read("site.js");
    assert!(script.contains("$$(\"a[data-page]\")") && script.contains("pages(code);") && script.contains("pages(pageLang);"));
    assert!(script.contains("target.tagName === \"DIALOG\""), "a dialog of the main page opens without the module");
}

#[test]
fn every_page_lists_every_language() {
    let expected: Vec<String> = Lang::ALL
        .into_iter()
        .map(|lang| {
            let file = if lang == Lang::En { "./".to_string() } else { openecraft_i18n::page::file_name(lang) };
            let code = lang.code();
            format!("<a href=\"{file}\" hreflang=\"{code}\" lang=\"{code}\">{}</a>", lang.native_name())
        })
        .collect();
    for (name, html) in PAGES.iter().map(|p| (*p, page(p))).chain([("index.html", read("index.html"))]) {
        let nav = html.split("<nav class=\"languages\" aria-label=\"Language\">\n").nth(1).and_then(|rest| rest.split("\n    </nav>").next()).unwrap();
        let links: Vec<&str> = nav.lines().map(str::trim).collect();
        assert_eq!(links, expected, "{name}");
    }
    let script = read("site.js");
    assert!(script.contains("$$(\".languages a\")") && script.contains("location.href = select.value"));
}

#[test]
fn the_build_writes_every_page_in_every_language() {
    use openecraft_i18n::page::translated;
    let trunk = read("Trunk.toml");
    assert!(trunk.contains("command = \"sh\"\ncommand_arguments = [\"pages.sh\"]"));
    assert!(trunk.contains("command_arguments = [\"-ExecutionPolicy\", \"Bypass\", \"-File\", \"pages.ps1\"]"));
    let scripts = [read("pages.sh"), read("pages.ps1")];
    for script in &scripts {
        assert!(script.contains("cargo run --release --quiet -p openecraft-i18n --example pages -- "));
    }
    assert!(scripts[0].contains(&format!("for page in {}; do", PAGES.join(" "))));
    assert!(scripts[1].contains(&format!("foreach ($page in '{}')", PAGES.join("', '"))));
    for folder in PAGES {
        let html = page(folder);
        let description = meta(&html, "name=\"description\"");
        for lang in Lang::REQUIRED.into_iter().filter(|l| *l != Lang::En) {
            let out = translated(&html, lang);
            assert!(out.contains(&format!("<html lang=\"{}\">", lang.code())), "{folder}: {}", lang.code());
            let left: Vec<&str> = between(&out, " data-t>", '<');
            assert!(left.is_empty(), "{folder}, {}: not translated: {left:?}", lang.code());
            let translation = meta(&out, "name=\"description\"");
            assert!(translation != description && meta(&out, "property=\"og:description\"") == translation, "{folder}, {}", lang.code());
            assert_ne!(meta(&out, "property=\"og:title\""), title(&html), "{folder}, {}", lang.code());
            assert_eq!(out.lines().count(), html.lines().count());
        }
    }
    // A folder without its slash is sent to it: the links of a page are relative to it.
    let nginx = read("../installer/docker/website.conf");
    assert!(nginx.contains(&format!("location ~ ^/({})$ {{\n        absolute_redirect off;\n        return 301 $uri/;", PAGES.join("|"))));
}
