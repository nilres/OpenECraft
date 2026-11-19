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
    let script = script(Some(&names), true, true, 120, 7);
    assert!(script.contains("\"module\":[\"a.js\",\"a_bg.wasm\"]") && script.contains("\"tour\":\"tour.webm\"") && script.contains("\"poster\":\"tour.jpg\""));
    assert!(script.contains("\"stars\":120") && script.contains("\"contributors\":7"));
    assert!(script.contains("document.currentScript.src") && script.contains("window.OPENECRAFT_BUILD = build;"));
    // Nothing known: no module, no recording, no numbers.
    let empty = super::script(None, false, true, 0, 0);
    assert!(empty.contains("\"module\":[]") && empty.contains("\"tour\":\"\"") && empty.contains("\"poster\":\"\"") && empty.contains("\"stars\":0"));
    assert!(!script.contains("http"));
}

#[test]
fn numbers_of_the_build_are_digits() {
    assert_eq!(number(Some(" 42\n")), 42);
    assert_eq!(number(Some("many")), 0);
    assert_eq!(number(Some("-3")), 0);
    assert_eq!(number(None), 0);
}
