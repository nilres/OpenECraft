use super::*;
use openecraft_i18n::Lang;
use serde_json::Value;

fn read(path: &str) -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The value of the attribute `name` of the first tag of `html` that contains `tag`.
fn attribute<'a>(html: &'a str, tag: &str, name: &str) -> &'a str {
    let at = html.find(tag).unwrap_or_else(|| panic!("no {tag}"));
    let rest = &html[at..html[at..].find('>').unwrap() + at];
    let value = rest.split(&format!("{name}=\"")).nth(1).unwrap_or_else(|| panic!("{tag} has no {name}"));
    value.split('"').next().unwrap()
}

#[test]
fn search_engines_and_previews_get_the_same_title_and_description() {
    let html = read("index.html");
    let title = html.split("<title data-t>").nth(1).and_then(|t| t.split("</title>").next()).unwrap();
    let description = attribute(&html, "<meta name=\"description\"", "content");
    assert!(description.len() > 80 && description.len() < 170, "a description search engines show whole");
    assert_eq!(attribute(&html, "property=\"og:title\"", "content"), title);
    assert_eq!(attribute(&html, "property=\"og:description\"", "content"), description);
    assert_eq!(attribute(&html, "name=\"twitter:card\"", "content"), "summary_large_image");
    assert_eq!(html.matches("<h1").count(), 1);
    // The tags with the address come from website-site.sh, never from the page itself.
    for tag in ["rel=\"canonical\"", "og:url", "og:image\"", "twitter:image"] {
        assert!(!html.contains(tag), "{tag} needs the public address");
    }
    let (open, close) = (html.find("<!-- site address (installer/docker/website-site.sh) -->").unwrap(), html.find("<!-- /site address -->").unwrap());
    assert!(open < close && html[open..close].lines().count() == 2, "nothing between the markers");
}

#[test]
fn the_structured_data_describes_the_application() {
    let html = read("index.html");
    let json = html.split("<script type=\"application/ld+json\">").nth(1).and_then(|s| s.split("</script>").next()).unwrap();
    let data: Value = serde_json::from_str(json).expect("JSON-LD is JSON");
    assert_eq!(data["@type"], "SoftwareApplication");
    assert_eq!(data["name"], "OpenECraft");
    assert_eq!(data["offers"]["price"], "0");
    assert_eq!(data["description"].as_str(), Some(attribute(&html, "<meta name=\"description\"", "content")));
    let languages: Vec<&str> = data["inLanguage"].as_array().unwrap().iter().map(|l| l.as_str().unwrap()).collect();
    assert_eq!(languages.len(), Lang::ALL.len());
    for (tag, lang) in languages.iter().zip(Lang::ALL) {
        assert!(tag.split('-').next() == Some(lang.code()), "{tag} is not {}", lang.code());
    }
}

#[test]
fn the_gallery_has_small_pictures_with_texts() {
    let html = read("index.html");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(html.contains("rel=\"copy-dir\" href=\"images\""));
    let shown: Vec<&str> = html.split("src=\"images/").skip(1).filter_map(|rest| rest.split(".webp\"").next()).collect();
    assert_eq!(shown, GALLERY);
    for name in GALLERY {
        let tag = format!("src=\"images/{name}.webp\"");
        let alt = attribute(&html, &tag, "alt");
        assert!(alt.starts_with("Screenshot of ") && attribute(&html, &tag, "data-t-alt") == alt, "{name}: alt text");
        assert_eq!((attribute(&html, &tag, "width"), attribute(&html, &tag, "height")), ("1080", "608"));
        let picture = std::fs::read(dir.join(format!("images/{name}.webp"))).unwrap();
        assert!(picture.starts_with(b"RIFF") && &picture[8..12] == b"WEBP", "{name}.webp");
        assert!(picture.len() < 120_000, "{name}.webp: {} bytes", picture.len());
        assert!(dir.join(format!("../docs/images/{name}.png")).exists(), "{name}: the screenshot of the documentation");
    }
    let preview = std::fs::read(dir.join("social.jpg")).unwrap();
    assert!(preview.starts_with(&[0xff, 0xd8]) && html.contains("rel=\"copy-file\" href=\"social.jpg\""));
    let size = preview.windows(2).position(|w| w == [0xff, 0xc0] || w == [0xff, 0xc2]).map(|at| &preview[at + 5..at + 9]).unwrap();
    assert_eq!(size, [2, 118, 4, 176], "the preview is 1200 × 630");
}

#[test]
fn the_page_has_an_icon_for_every_browser() {
    let html = read("index.html");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(html.contains("<link rel=\"icon\" href=\"icon.svg\" type=\"image/svg+xml\" />"));
    assert!(html.contains("rel=\"copy-file\" href=\"../assets/icon.svg\""));
    // iOS takes a PNG for the home screen: opaque, as it shows transparent corners black.
    assert!(html.contains("<link rel=\"apple-touch-icon\" href=\"apple-touch-icon.png\" />"));
    assert!(html.contains("rel=\"copy-file\" href=\"apple-touch-icon.png\""));
    let icon = std::fs::read(dir.join("apple-touch-icon.png")).unwrap();
    assert!(icon.starts_with(b"\x89PNG\r\n\x1a\n") && &icon[12..16] == b"IHDR");
    assert_eq!(icon[16..24], [0, 0, 0, 180, 0, 0, 0, 180], "180 × 180 pixels");
    assert_eq!(icon[25], 2, "RGB without transparency");
}

#[test]
fn the_image_sends_the_big_files_compressed_by_its_build() {
    let dockerfile = read("../installer/docker/Dockerfile");
    let nginx = read("../installer/docker/website.conf");
    assert!(nginx.contains("\n    gzip_static on;"), "website.conf sends file.gz");
    let build = dockerfile.split("AS website-build").nth(1).and_then(|rest| rest.split("\nFROM ").next()).unwrap();
    assert!(build.contains("-exec gzip -9 -k {} +") && build.contains("-name '*.wasm'"), "the build of the image compresses");
    // The files written at every start have no copy that would be sent in their place.
    assert!(build.contains("! -name config.js") && !build.contains("'*.html'") && !build.contains("'*.txt'"));
}

#[test]
fn the_address_is_set_in_the_image_and_by_the_script() {
    let script = read("../installer/docker/website-site.sh");
    let dockerfile = read("../installer/docker/Dockerfile");
    let nginx = read("../installer/docker/website.conf");
    let compose = read("../installer/docker/compose.yaml");
    let docs = read("../docs/website.md");
    assert!(script.contains(&format!("url=${{{SITE_URL}:-}}")) && !script.contains('\r'));
    assert!(script.contains("<!-- site address /") && script.contains("<!-- \\/site address -->"));
    assert!(dockerfile.contains(&format!("{SITE_URL}=")) && dockerfile.contains("installer/docker/website-site.sh /docker-entrypoint.d/"));
    assert!(compose.contains(&format!("{SITE_URL}: ${{{SITE_URL}:-}}")));
    assert!(docs.contains(&format!("`{SITE_URL}`")));
    for file in ["index.html", "robots.txt", "sitemap.xml"] {
        assert!(nginx.contains(&format!("location = /{file} {{\n        alias /tmp/openecraft/{file};")) || nginx.contains(&format!("location = /{file} {{\r\n        alias /tmp/openecraft/{file};")), "website.conf: {file}");
    }
    // try_files would hand out the page as built: / has to go through /index.html.
    assert!(nginx.replace("\r\n", "\n").contains("location = / {\n        rewrite ^ /index.html last;"), "website.conf: /");
    // Without an address the page has the robots.txt the script writes then.
    assert_eq!(read("robots.txt").replace("\r\n", "\n"), "User-agent: *\nAllow: /\n");
    assert!(script.contains("printf 'User-agent: *\\nAllow: /\\n'"));
    assert!(read("index.html").contains("rel=\"copy-file\" href=\"robots.txt\""));
}

#[test]
fn every_language_has_a_page_search_engines_can_read() {
    use openecraft_i18n::page::{file_name, translated};
    let html = read("index.html");
    assert_eq!(attribute(&html, "<meta name=\"description\"", "content"), DESCRIPTION);
    for lang in Lang::REQUIRED.into_iter().filter(|l| *l != Lang::En) {
        let page = translated(&html, lang);
        let description = openecraft_i18n::translate(DESCRIPTION, lang);
        assert!(page.contains(&format!("<html lang=\"{}\">", lang.code())), "{}", lang.code());
        assert!(description != DESCRIPTION && attribute(&page, "<meta name=\"description\"", "content") == description);
        assert_eq!(attribute(&page, "property=\"og:description\"", "content"), description);
        assert!(!page.contains("<title data-t>") && page.contains("<title data-t=\"OpenECraft – open source electrical CAE\">"));
        // Every text of the page is in the language, and the script finds its English.
        let left: Vec<&str> = page.split(" data-t>").skip(1).filter_map(|rest| rest.split('<').next()).collect();
        assert!(left.is_empty(), "{}: not translated: {left:?}", lang.code());
        assert_eq!(page.matches(" data-t=\"").count(), html.matches(" data-t>").count());
        assert_eq!(page.matches(" placeholder=\"").count(), html.matches(" data-t-placeholder=\"").count());
        assert_eq!(page.lines().count(), html.lines().count());
    }
    // The build writes them, the script of the address and nginx know them.
    let names: Vec<String> = Lang::ALL.into_iter().map(file_name).collect();
    assert!(names.iter().skip(1).all(|n| n.len() == 7 && n.ends_with(".html")), "two letters: {names:?}");
    assert!(read("Trunk.toml").contains("\"openecraft-i18n\", \"--example\", \"pages\""));
    let script = read("../installer/docker/website-site.sh");
    assert!(script.contains("\"$page\"/[a-z][a-z].html") && script.contains("hreflang=\\\"x-default\\\""));
    assert!(script.contains("tags index.html \"\"") && script.contains("%s%s.html</loc>"));
    let nginx = read("../installer/docker/website.conf").replace("\r\n", "\n");
    assert!(nginx.contains("location ~ ^/([a-z][a-z]\\.html)$ {\n        alias /tmp/openecraft/$1;"), "website.conf: the pages of the languages");
    assert!(read("site.js").contains("document.documentElement.lang"), "the page of a language shows it");
}

#[test]
fn the_page_is_shown_before_its_module_is_loaded() {
    let trunk = read("Trunk.toml");
    let script = read("site.js");
    let html = read("index.html");
    // trunk names the module, site.js loads it: no script or preload of trunk's own.
    assert!(trunk.contains("pattern_script = \"<script>window.OPENECRAFT_MODULE = [\\\"{base}{js}\\\", \\\"{base}{wasm}\\\"];</script>\""));
    assert!(trunk.contains("pattern_preload = "));
    assert!(script.contains("window.OPENECRAFT_MODULE") && script.contains("import(moduleUrl)") && script.contains("module_or_path: wasmUrl"));
    assert!(!script.contains("wasmBindings") && !script.contains("TrunkApplicationStarted") && !script.contains("modulepreload"));
    assert!(script.contains("navigator.connection.saveData") && script.contains("requestIdleCallback"));
    // Until then the first screen has a picture, and nothing that needs the module.
    assert!(html.contains("id=\"hero-sheet\"><img src=\"sheet.svg\"") && html.contains("class=\"sheet-note\" hidden"));
    assert!(html.contains("<select id=\"language\" aria-label=\"Language\" hidden>"));
}

#[test]
fn the_log_of_the_image_has_no_whole_addresses() {
    let nginx = read("../installer/docker/website.conf");
    assert!(nginx.contains("map $remote_addr $visitor {") && nginx.contains("log_format visits '$visitor "));
    assert!(nginx.contains("access_log /var/log/nginx/access.log visits;"));
    let format = nginx.split("log_format visits").nth(1).and_then(|rest| rest.split(';').next()).unwrap();
    assert!(!format.contains("remote_addr") && !format.contains("forwarded"), "{format}");
}
