use super::*;

#[test]
fn the_module_is_the_one_trunk_names_in_the_page() {
    let index = "<head>\n<script>window.OPENECRAFT_MODULE = [\"./openecraft-website-0123abcd.js\", \"./openecraft-website-0123abcd_bg.wasm\"];</script>\n</head>";
    assert_eq!(module(index), Some(["openecraft-website-0123abcd.js".to_string(), "openecraft-website-0123abcd_bg.wasm".to_string()]));
    assert_eq!(module("<head></head>"), None);
    assert_eq!(module("window.OPENECRAFT_MODULE = [\"a.txt\", \"b.wasm\"];"), None);
    // The pattern of Trunk.toml this reads.
    let trunk = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../website/Trunk.toml")).unwrap();
    assert!(trunk.contains("window.OPENECRAFT_MODULE = [\\\"{base}{js}\\\", \\\"{base}{wasm}\\\"];"));
    assert!(trunk.contains("\"openecraft-site\", \"--example\", \"site\""));
}

#[test]
fn the_script_says_what_the_build_knows() {
    let names = ["a.js".to_string(), "a_bg.wasm".to_string()];
    let script = script(Some(&names), true, true);
    assert!(script.contains("\"module\":[\"a.js\",\"a_bg.wasm\"]") && script.contains("\"tour\":\"tour.webm\"") && script.contains("\"poster\":\"tour.jpg\""));
    assert!(script.contains("document.currentScript.src") && script.contains("window.OPENECRAFT_BUILD = build;"));
    // Nothing known: no module, no recording.
    let empty = super::script(None, false, true);
    assert!(empty.contains("\"module\":[]") && empty.contains("\"tour\":\"\"") && empty.contains("\"poster\":\"\""));
    assert!(!script.contains("http"));
}

#[test]
fn the_numbers_are_a_script_the_image_of_the_site_writes_too() {
    assert_eq!(numbers(120, 7), "window.OPENECRAFT_NUMBERS = { stars: 120, contributors: 7 };\n");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |path: &str| std::fs::read_to_string(root.join(path)).unwrap().replace("\r\n", "\n");
    // The script of the image writes the same line, from the same variables.
    let script = read("installer/docker/website-numbers.sh");
    assert!(script.contains("'window.OPENECRAFT_NUMBERS = { stars: %s, contributors: %s };\\n'"));
    assert!(NUMBERS.iter().all(|name| script.contains(&format!("${{{name}:-}}"))));
    // The image has the script and a baseline, nginx answers with its file.
    let docker = read("installer/docker/Dockerfile");
    assert!(docker.contains("installer/docker/website-numbers.sh /docker-entrypoint.d/"));
    assert!(NUMBERS.iter().all(|name| docker.contains(&format!("ARG {name}=0"))));
    assert!(read("installer/docker/website.conf").contains(&format!("location = /{NUMBERS_FILE} {{\n        alias /tmp/openecraft/{NUMBERS_FILE};")));
    // The main page loads it and site.js shows it.
    assert!(read("website/index.html").contains(&format!("<script src=\"{NUMBERS_FILE}\"></script>")));
    assert!(read("website/site.js").contains("window.OPENECRAFT_NUMBERS"));
}

#[test]
fn the_videos_are_a_script_the_image_of_the_site_fills() {
    assert_eq!(videos(), "window.OPENECRAFT_VIDEOS = [];\n");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |path: &str| std::fs::read_to_string(root.join(path)).unwrap().replace("\r\n", "\n");
    // The script of the image starts with the same line and fills it from the feed
    // of the channel, four times a day.
    let script = read("installer/docker/website-videos.sh");
    assert!(script.contains("printf 'window.OPENECRAFT_VIDEOS = [];\\n'") && script.contains("print \"window.OPENECRAFT_VIDEOS = [\""));
    assert!(script.contains("${OPENECRAFT_VIDEOS_CHANNEL:-}") && script.contains("interval=${OPENECRAFT_VIDEOS_INTERVAL:-21600}"));
    assert!(!script.contains('\r'), "website-videos.sh runs on Linux");
    let docker = read("installer/docker/Dockerfile");
    assert!(docker.contains("installer/docker/website-videos.sh /docker-entrypoint.d/"));
    assert!(docker.contains("OPENECRAFT_VIDEOS_CHANNEL=") && docker.contains("OPENECRAFT_VIDEOS_INTERVAL=21600"));
    // nginx answers with the list and with the pictures the script fetched.
    let nginx = read("installer/docker/website.conf");
    assert!(nginx.contains(&format!("location = /{VIDEOS_FILE} {{\n        alias /tmp/openecraft/{VIDEOS_FILE};")));
    assert!(nginx.contains("location ~ ^/videos/([A-Za-z0-9_-]+\\.jpg)$ {\n        alias /tmp/openecraft/videos/$1;"));
    // The page of the videos loads the list before the script that shows it; no other
    // page does, and the script names no address of its own.
    let page = crate::content::parse("videos", &read("website/content/videos.html")).unwrap().page(openecraft_i18n::Lang::En);
    assert_eq!(page.kind, crate::frame::Kind::Videos);
    let html = crate::frame::html(&page, openecraft_i18n::Lang::En);
    let (list, script) = (html.find("<script src=\"../videos.js\"></script>").unwrap(), html.find("<script src=\"../site.js\" defer></script>").unwrap());
    assert!(list < script);
    assert!(html.contains("data-videos") && html.contains("<template id=\"video\">") && html.contains("data-link=\"videos\""));
    let plain = crate::frame::html(&crate::frame::Page { kind: crate::frame::Kind::Plain, ..page }, openecraft_i18n::Lang::En);
    assert!(!plain.contains("videos.js"));
    assert!(read("website/site.js").contains("window.OPENECRAFT_VIDEOS"));
}

#[test]
fn numbers_of_the_build_are_digits() {
    assert_eq!(number(Some(" 42\n")), 42);
    assert_eq!(number(Some("many")), 0);
    assert_eq!(number(Some("-3")), 0);
    assert_eq!(number(None), 0);
}
