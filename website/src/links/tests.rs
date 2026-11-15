use super::*;
use std::collections::BTreeSet;

fn read(path: &str) -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn every_link_of_the_page_has_a_setting() {
    let html = read("index.html");
    let used: BTreeSet<&str> = html.split("data-link=\"").skip(1).filter_map(|rest| rest.split('"').next()).collect();
    let settings: BTreeSet<&str> = SETTINGS.iter().map(|s| s.0).collect();
    assert_eq!(used, settings);
    // The page loads the settings before its script and has no address of its own.
    let (config, script) = (html.find("src=\"config.js\"").unwrap(), html.find("src=\"site.js\"").unwrap());
    assert!(config < script);
    assert!(html.contains("rel=\"copy-file\" href=\"config.js\""), "trunk copies the settings");
    assert!(!html.contains("href=\"http"), "addresses belong into config.js");
}

#[test]
fn every_field_of_the_legal_notice_has_a_place() {
    let html = read("index.html");
    let used: BTreeSet<&str> = html.split("data-legal=\"").skip(1).filter_map(|rest| rest.split('"').next()).collect();
    let fields: BTreeSet<&str> = LEGAL.iter().map(|s| s.0).collect();
    assert_eq!(used, fields);
    assert!(html.contains("id=\"legal-open\" href=\"#legal\" hidden"), "the link waits for a name");
}

#[test]
fn the_page_links_its_licenses_and_its_privacy_notice() {
    let html = read("index.html");
    let script = read("site.js");
    for name in ["licenses", "privacy", "legal"] {
        assert!(html.contains(&format!("id=\"{name}-open\" href=\"#{name}\"")) && html.contains(&format!("<dialog id=\"{name}\"")), "{name}");
        assert!(script.contains(&format!("\"{name}\"")), "site.js opens {name}");
    }
    let hooks = [read("../installer/web/licenses.sh"), read("../installer/web/licenses.ps1")];
    for file in LICENSES {
        assert!(html.contains(&format!("<a href=\"{file}\"")), "the page links {file}");
        assert!(hooks.iter().all(|hook| hook.contains(file)), "the build writes {file}");
    }
    assert!(read("Trunk.toml").contains("installer/web/licenses.sh"));
}

#[test]
fn the_page_loads_nothing_from_elsewhere() {
    let html = read("index.html");
    let policy = html.split("http-equiv=\"Content-Security-Policy\" content=\"").nth(1).and_then(|rest| rest.split('"').next()).expect("a policy");
    assert!(html.find("Content-Security-Policy").unwrap() < html.find("<script").unwrap(), "the policy comes before the scripts");
    assert!(policy.starts_with("default-src 'self';") && policy.contains("object-src 'none'") && policy.contains("'wasm-unsafe-eval'"));
    assert!(!policy.contains("http") && !policy.contains('*') && !policy.contains("'unsafe-eval'"), "no other host: {policy}");
    for file in ["site.js", "worker.js", "site.css"] {
        let text = read(file).replace("http://www.w3.org/", "");
        assert!(!text.contains("http://") && !text.contains("https://"), "{file} names an address");
    }
    // nginx drops the headers of the server in a block with headers of its own.
    let nginx = read("../installer/docker/website.conf");
    let blocks = nginx.matches("add_header Cache-Control").count();
    for header in HEADERS {
        assert_eq!(nginx.matches(&format!("add_header {header} always;")).count(), blocks, "website.conf: {header}");
    }
}

#[test]
fn the_settings_are_the_same_everywhere() {
    let config = read("config.js");
    let script = read("site.js");
    let writer = read("../installer/docker/website-config.sh");
    let nginx = read("../installer/docker/website.conf");
    let dockerfile = read("../installer/docker/Dockerfile");
    let compose = read("../installer/docker/compose.yaml");
    let docs = read("../docs/website.md");
    let all = SETTINGS.iter().map(|s| ("    ", "", s)).chain(LEGAL.iter().map(|s| ("        ", "legal.", s)));
    for (indent, prefix, (name, variable)) in all {
        assert!(config.contains(&format!("\n{indent}{name}: \"")), "config.js: {prefix}{name}");
        assert!(config.contains(variable), "config.js names {variable}");
        assert!(writer.contains(&format!("setting(\"{indent}\", \"{name}\", \"{variable}\")")), "website-config.sh: {variable}");
        assert!(dockerfile.contains(&format!("{variable}=")), "Dockerfile: {variable}");
        assert!(compose.contains(&format!("{variable}: ${{{variable}:-")), "compose.yaml: {variable}");
        assert!(docs.contains(&format!("| `{prefix}{name}` | `{variable}` |")), "docs/website.md: {prefix}{name}");
    }
    for (name, _) in SETTINGS {
        assert!(script.contains(&format!("(\"{name}\")")), "site.js: {name}");
    }
    let listed = |indent: &str| config.lines().filter(|l| l.starts_with(indent) && !l[indent.len()..].starts_with([' ', '/', '}'])).count();
    assert_eq!(listed("    "), SETTINGS.len() + 1, "a setting of config.js is not listed");
    assert_eq!(listed("        "), LEGAL.len(), "a field of the legal notice is not listed");
    assert!(nginx.contains("alias /tmp/openecraft/config.js;") && writer.contains("> /tmp/openecraft/config.js"));
    assert!(dockerfile.contains("installer/docker/website-config.sh /docker-entrypoint.d/"));
    assert!(!writer.contains('\r'), "website-config.sh runs on Linux");
}
