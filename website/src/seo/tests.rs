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

/// Every block of structured data of a page.
fn structured(html: &str) -> Vec<Value> {
    html.split("<script type=\"application/ld+json\">").skip(1).map(|s| serde_json::from_str(s.split("</script>").next().unwrap()).expect("JSON-LD is JSON")).collect()
}

/// A setting of `config.js`: the address between the quotes after `name: `.
fn setting(name: &str) -> String {
    let config = read("config.js");
    config.split(&format!("\n    {name}: \"")).nth(1).and_then(|rest| rest.split('"').next()).unwrap_or_else(|| panic!("config.js: {name}")).to_string()
}

#[test]
fn the_structured_data_names_the_site_its_pictures_and_where_the_project_is() {
    let html = read("index.html");
    let blocks = structured(&html);
    assert_eq!(blocks.len(), 2);
    let (application, site) = (&blocks[0], &blocks[1]);
    // The pictures of the gallery, with the texts they have on the page.
    let shots = application["screenshot"].as_array().unwrap();
    assert_eq!(shots.len(), GALLERY.len());
    for (shot, name) in shots.iter().zip(GALLERY) {
        assert_eq!(shot["@type"], "ImageObject");
        assert_eq!(shot["contentUrl"], format!("./images/{name}.webp"));
        assert_eq!(shot["name"].as_str(), Some(attribute(&html, &format!("src=\"images/{name}.webp\""), "alt")));
    }
    assert_eq!(application["image"], "./social.jpg");
    assert_eq!(application["license"], "./LICENSE-MIT");
    assert!(read("../installer/web/licenses.sh").contains("LICENSE-MIT"));
    // Where the source and the downloads are: the repository the page links.
    let repository = setting("repository");
    assert_eq!(application["sameAs"], serde_json::json!([repository]));
    assert_eq!(application["downloadUrl"], format!("{repository}/releases/latest"));
    // The site, whose name search results show, and the project behind it.
    assert_eq!(site["@type"], "WebSite");
    assert_eq!((&site["name"], &site["url"], &application["url"]), (&Value::from("OpenECraft"), &Value::from("./"), &Value::from("./")));
    assert_eq!(site["publisher"]["@type"], "Organization");
    assert_eq!(site["publisher"]["logo"], "./apple-touch-icon.png");
    assert_eq!(site["publisher"]["sameAs"], serde_json::json!([repository, setting("videos")]));
    // An address of the site starts at its folder and is the value of a name on a line
    // of its own, which is where website-site.sh puts the public address.
    let data: String = html.split("<script type=\"application/ld+json\">").skip(1).filter_map(|s| s.split("</script>").next()).collect();
    assert_eq!(data.matches("\"./").count(), data.matches("\": \"./").count());
    assert!(data.lines().all(|line| line.matches("\": \"./").count() <= 1));
    let script = read("../installer/docker/website-site.sh");
    assert!(script.contains("if (json != \"\") $0 = replaced($0, value site, value json)") && script.contains("value = \"\\\": \\\"\""));
    assert!(script.contains("index($0, \"<script type=\\\"application/ld+json\\\">\") { data = 1 }"));
    // In another language the texts of the data are in that language too.
    let german = structured(&openecraft_i18n::page::translated(&html, Lang::De));
    assert_eq!(german[0]["description"].as_str(), Some(openecraft_i18n::translate(DESCRIPTION, Lang::De)));
    assert_eq!(german[0]["name"], "OpenECraft");
    assert_ne!(german[0]["screenshot"][0]["name"], application["screenshot"][0]["name"]);
    assert_eq!(german[0]["screenshot"][0]["contentUrl"], "./images/schematic.webp");
}

#[test]
fn the_sitemap_names_the_days_the_pages_changed() {
    use openecraft_site::changes::FILE;
    assert_eq!(FILE, "pages.tsv");
    // The build compares its pages with the file of the site that is online.
    let fetch = read("../installer/web/pages.sh");
    assert!(fetch.contains("url=${OPENECRAFT_PAGES_URL-}") && fetch.contains(&format!("website/{FILE}")) && !fetch.contains('\r'));
    let dockerfile = read("../installer/docker/Dockerfile");
    let build = dockerfile.split("AS website-build").nth(1).and_then(|rest| rest.split("\nFROM ").next()).unwrap();
    assert!(build.contains("ARG OPENECRAFT_PAGES_URL=") && build.contains("OPENECRAFT_PAGES_URL=\"$OPENECRAFT_PAGES_URL\" sh installer/web/pages.sh"));
    assert!(build.find("installer/web/pages.sh").unwrap() < build.find("trunk build").unwrap());
    for workflow in ["ci.yml", "docker-image.yml"] {
        let text = read(&format!("../.github/workflows/{workflow}"));
        assert_eq!(text.matches(&format!("OPENECRAFT_PAGES_URL=${{{{ vars.OPENECRAFT_SITE_URL || 'https://openecraft.org/' }}}}{FILE}")).count(), 2, "{workflow}");
    }
    assert!(read("../.gitignore").replace("\r\n", "\n").contains(&format!("\n/website/{FILE}\n")));
    assert!(read("../crates/openecraft-site/examples/site.rs").contains("website.join(changes::FILE)"));
    // The sitemap takes the days from the file, which is no page for a search engine.
    let script = read("../installer/docker/website-site.sh");
    assert!(script.contains(&format!("SITE_DAYS=\"$page/{FILE}\"")) && script.contains("\"<lastmod>\" day[$1] \"</lastmod>\""));
    assert!(script.contains("printf '%s.html\\t%s.html\\n' \"$lang\" \"$lang\"") && script.contains("<loc>%s%s</loc>"));
    let nginx = read("../installer/docker/website.conf").replace("\r\n", "\n");
    let block = nginx.split(&format!("location = /{FILE} {{")).nth(1).and_then(|rest| rest.split("\n    }").next()).expect("website.conf: pages.tsv");
    assert!(block.contains("add_header X-Robots-Tag noindex always;"));
}

#[test]
fn search_engines_are_told_of_changed_pages_only_with_a_key() {
    let script = read("../installer/docker/website-indexnow.sh");
    assert!(!script.contains('\r') && script.contains("key=${OPENECRAFT_INDEXNOW_KEY:-}") && script.contains(&format!("url=${{{SITE_URL}:-}}")));
    assert!(script.contains("'') exit 0 ;;") && script.contains("\"$page/pages.tsv\"") && script.contains("https://api.indexnow.org/indexnow"));
    assert!(script.contains("$3 != \"\" && $3 == built") && script.contains("\\\"keyLocation\\\":\\\"%sindexnow.txt\\\""));
    let dockerfile = read("../installer/docker/Dockerfile");
    assert!(dockerfile.contains("OPENECRAFT_INDEXNOW_KEY= \\") && dockerfile.contains("installer/docker/website-indexnow.sh /docker-entrypoint.d/44-openecraft-website-indexnow.sh"));
    assert!(read("../installer/docker/compose.yaml").contains("OPENECRAFT_INDEXNOW_KEY: ${OPENECRAFT_INDEXNOW_KEY:-}"));
    let nginx = read("../installer/docker/website.conf").replace("\r\n", "\n");
    assert!(nginx.contains("location = /indexnow.txt {\n        alias /tmp/openecraft/indexnow.txt;"));
    let docs = read("../docs/website.md");
    assert!(docs.contains("`OPENECRAFT_INDEXNOW_KEY`") && docs.contains("`OPENECRAFT_PAGES_URL`"));
    assert!(read("../installer/docker/check-website.sh").contains("OPENECRAFT_INDEXNOW_KEY="));
}

#[test]
fn an_address_without_a_page_gets_one_in_the_language_of_the_browser() {
    use openecraft_site::missing::file_name;
    let nginx = read("../installer/docker/website.conf").replace("\r\n", "\n");
    let codes: Vec<&str> = Lang::ALL.into_iter().filter(|lang| *lang != Lang::En).map(Lang::code).collect();
    assert!(nginx.contains(&format!("map $http_accept_language $missing {{\n    default {};\n    \"~*^(?<code>{})\\b\" 404.$code.html;\n}}", file_name(Lang::En), codes.join("|"))), "website.conf: the languages");
    assert_eq!(file_name(Lang::De), "404.de.html");
    assert!(nginx.contains("\n    error_page 404 /$missing;\n"));
    assert!(nginx.contains("location ~ ^/(404(\\.[a-z][a-z])?\\.html)$ {\n        internal;\n        alias /tmp/openecraft/$1;"));
    // The page is asked for at any depth: the script names the folder of the site.
    let script = read("../installer/docker/website-site.sh");
    assert!(script.contains("for file in \"$page\"/404.html \"$page\"/404.[a-z][a-z].html; do") && script.contains("SITE_URL=${esc:-/} awk"));
    assert!(script.contains("\"<base href=\\\"\" ENVIRON[\"SITE_URL\"] \"\\\" />\""));
    // A <base> of the site itself is allowed by the policy of the pages.
    let page = String::from_utf8(openecraft_site::missing::site().remove(0).bytes).unwrap();
    assert!(page.contains("base-uri 'self'"));
}

#[test]
fn language_models_get_a_list_of_the_pages() {
    use openecraft_site::llms::FILE;
    assert_eq!(FILE, "llms.txt");
    let script = read("../installer/docker/website-site.sh");
    assert!(script.contains(&format!("\"$page/{FILE}\"")) && script.contains("replaced($0, \"](./\", \"](\" url)"));
    let nginx = read("../installer/docker/website.conf").replace("\r\n", "\n");
    assert!(nginx.contains(&format!("location = /{FILE} {{\n        alias /tmp/openecraft/{FILE};")));
    assert!(read("../crates/openecraft-site/examples/site.rs").contains("File::text(llms::FILE, llms::text(&entries, repository))"));
    // The pages written by hand are read from the built page: those of the bar.
    assert_eq!(openecraft_site::llms::written_by_hand().iter().map(|folder| folder.trim_end_matches('/')).collect::<Vec<_>>(), crate::pages::PAGES);
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
    // What is asked of a site without reading a page of it: /favicon.ico, in three
    // sizes, of the page and of the web editor.
    assert!(html.contains("rel=\"copy-file\" href=\"favicon.ico\""));
    assert!(read("../openecraft/index.html").contains("rel=\"copy-file\" href=\"../website/favicon.ico\""));
    let favicon = std::fs::read(dir.join("favicon.ico")).unwrap();
    assert_eq!(favicon[..6], [0, 0, 1, 0, 3, 0], "an icon file with three pictures");
    let sizes: Vec<u8> = (0..3).map(|i| favicon[6 + 16 * i]).collect();
    assert_eq!(sizes, [16, 32, 48]);
    assert!(read("apple-touch-icon.py").contains("favicon.ico"));
}

#[test]
fn the_image_sends_the_big_files_compressed_by_its_build() {
    let dockerfile = read("../installer/docker/Dockerfile");
    let nginx = read("../installer/docker/website.conf");
    assert!(nginx.contains("\n    gzip_static on;"), "website.conf sends file.gz");
    let build = dockerfile.split("AS website-build").nth(1).and_then(|rest| rest.split("\nFROM ").next()).unwrap();
    assert!(build.contains("sh installer/docker/compress.sh /src/website/dist /src/target/website-editor"), "the build of the image compresses");
    let script = read("../installer/docker/compress.sh");
    assert!(script.contains("gzip -9 -k") && script.contains("brotli -q 11 -k") && script.contains("-name '*.wasm'"));
    // The files written at every start have no copy that would be sent in their place.
    assert!(script.contains("! -name config.js") && !script.contains("'*.html'") && !script.contains("'*.txt'"));
    // Brotli copies are sent to who takes them, without a module of nginx.
    assert!(nginx.contains("rewrite ^ $uri.br last;") && nginx.contains("add_header Content-Encoding br;"));
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
    assert!(script.contains("tags index.html \"\"") && script.contains("printf '%s.html\\t%s.html\\n' \"$lang\" \"$lang\""));
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
fn the_page_fits_phones_and_tablets() {
    let html = read("index.html");
    let css = read("site.css");
    let script = read("site.js");
    assert!(html.contains("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />"));
    // A long line in a list must not widen its column beyond the screen.
    assert!(css.contains(".studio > *, .reports > *, .automation > * { min-width: 0; }"));
    // On a narrow screen the links are a menu, opened by a button the script shows.
    assert!(html.contains("id=\"nav-toggle\" aria-expanded=\"false\" aria-controls=\"nav-links\" hidden>") && html.contains("<nav id=\"nav-links\">"));
    assert!(css.contains(".nav.open nav { display: flex; }") && script.contains("$(\"#nav-toggle\")") && script.contains("toggle.hidden = false"));
    // What tells of the mouse has a text for fingers beside it, and every veil both.
    assert_eq!(html.matches("class=\"for-mouse\"").count(), 3);
    assert_eq!(html.matches("<span class=\"for-mouse\" data-t>").count(), html.matches("</span><span class=\"for-touch\" data-t>").count());
    assert_eq!(html.matches("<button class=\"veil\"><span class=\"for-mouse\"").count(), html.matches("class=\"veil\"").count());
    assert!(css.contains(".for-touch { display: none; }") && css.contains("@media (pointer: coarse)"));
    // Nothing moves under a finger that cannot hover.
    let hover = css.split("@media (hover: hover)").next().unwrap();
    assert!(!hover.lines().any(|line| line.contains(":hover") && line.contains("transform")), "a transform on :hover outside the hover query");
}

#[test]
fn the_log_of_the_image_has_no_whole_addresses() {
    let nginx = read("../installer/docker/website.conf");
    assert!(nginx.contains("map $remote_addr $visitor {") && nginx.contains("log_format visits '$visitor "));
    assert!(nginx.contains("access_log /var/log/nginx/access.log visits;"));
    let format = nginx.split("log_format visits").nth(1).and_then(|rest| rest.split(';').next()).unwrap();
    assert!(!format.contains("remote_addr") && !format.contains("forwarded"), "{format}");
}
