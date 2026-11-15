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
fn the_settings_are_the_same_everywhere() {
    let config = read("config.js");
    let script = read("site.js");
    let nginx = read("../installer/docker/website.conf");
    let dockerfile = read("../installer/docker/Dockerfile");
    let compose = read("../installer/docker/compose.yaml");
    let docs = read("../docs/website.md");
    for (name, variable) in SETTINGS {
        assert!(config.contains(&format!("\n    {name}: \"")), "config.js: {name}");
        assert!(config.contains(variable), "config.js names {variable}");
        assert!(script.contains(&format!("(\"{name}\")")), "site.js: {name}");
        assert!(nginx.contains(&format!("{name}: \"${{{variable}}}\"")), "website.conf: {name}");
        assert!(dockerfile.contains(&format!("{variable}=")), "Dockerfile: {variable}");
        assert!(compose.contains(&format!("{variable}: ${{{variable}:-")), "compose.yaml: {variable}");
        assert!(docs.contains(&format!("| `{name}` | `{variable}` |")), "docs/website.md: {name}");
    }
    assert_eq!(config.lines().filter(|l| l.starts_with("    ") && !l.trim_start().starts_with("//")).count(), SETTINGS.len(), "a setting of config.js is not listed");
}
