use super::*;
use std::collections::BTreeSet;

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn webp() -> Vec<String> {
    vec!["schematic".to_string(), "panel-3d".to_string()]
}

#[test]
fn every_document_has_a_folder_of_its_own() {
    let folders: BTreeSet<&str> = DOCUMENTS.iter().map(|file| slug(file)).collect();
    assert_eq!(folders.len(), DOCUMENTS.len());
    assert_eq!(slug("docs/features/panel-layout.md"), "panel-layout");
    for file in DOCUMENTS {
        assert!(root().join(file).is_file(), "{file}");
        assert!(slug(file).chars().all(|c| c.is_ascii_lowercase() || c == '-') && slug(file) != "images", "{file}");
    }
    // The documents the footer of every page links.
    for (target, _) in crate::frame::DOCUMENTS {
        assert!(DOCUMENTS.iter().any(|file| format!("{FOLDER}/{}/", slug(file)) == target), "{target}");
    }
}

#[test]
fn links_of_a_document_lead_to_the_site_or_the_repository() {
    let to = |from: &str, address: &str| target(from, address, "../../", &webp());
    let page = |address: &str| Target::Address(address.to_string());
    assert_eq!(to("docs/tutorial.md", "plugins.md"), page("../../docs/plugins/"));
    assert_eq!(to("docs/tutorial.md", "installation.md#web-webassembly"), page("../../docs/installation/#web-webassembly"));
    assert_eq!(to("docs/features/parts.md", "../tutorial.md"), page("../../docs/tutorial/"));
    assert_eq!(to("CHANGELOG.md", "docs/features/panel-layout.md#floor-plans-on-paper"), page("../../docs/panel-layout/#floor-plans-on-paper"));
    assert_eq!(to("docs/releasing.md", "../CHANGELOG.md"), page("../../news/"));
    assert_eq!(to("docs/tutorial.md", "#reports"), page("#reports"));
    assert_eq!(to("docs/tutorial.md", "https://example.org/"), page("https://example.org/"));
    // A screenshot the site has as a small picture, and one it has not.
    assert_eq!(to("docs/features/schematic.md", "../images/schematic.png"), page("../../images/schematic.webp"));
    assert_eq!(to("docs/features/panel-layout.md", "../images/room.png"), page("../../docs/images/room.png"));
    // Anything else is a file of the repository.
    assert_eq!(to("docs/tutorial.md", "architecture.md"), Target::Repository("docs/architecture.md".to_string()));
    assert_eq!(to("docs/plugins.md", "../crates/openecraft-api/src/builtin.rs#L10"), Target::Repository("crates/openecraft-api/src/builtin.rs#L10".to_string()));
}

#[test]
fn the_documents_of_the_repository_become_pages() {
    let files = site(&root(), &webp()).unwrap();
    let paths: BTreeSet<&str> = files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths.len(), files.len());
    let text = |path: &str| String::from_utf8(files.iter().find(|f| f.path == path).unwrap_or_else(|| panic!("{path}")).bytes.clone()).unwrap();
    let mut titles = BTreeSet::new();
    for file in DOCUMENTS {
        let html = text(&format!("docs/{}/index.html", slug(file)));
        assert_eq!(html.matches("<h1").count(), 1, "{file}");
        let left: Vec<&str> = html.split("href=\"").skip(1).filter_map(|rest| rest.split('"').next()).filter(|a| !a.contains("://") && a.contains(".md")).collect();
        assert!(left.is_empty(), "{file}: links to Markdown files: {left:?}");
        assert!(html.contains(&format!("data-path=\"/blob/master/{file}\"")), "{file}: its source");
        let title = html.split("<title>").nth(1).and_then(|t| t.split("</title>").next()).unwrap().to_string();
        assert!(title.chars().count() <= 70 && title.len() > 20, "{file}: {title}");
        assert!(titles.insert(title), "{file}: its title twice");
        let description = html.split("<meta name=\"description\" content=\"").nth(1).and_then(|t| t.split('"').next()).unwrap();
        assert!(DESCRIPTION.contains(&description.chars().count()), "{file}: {description}");
        assert!(!html.contains("class=\"languages\""), "{file}: in English only");
    }
    let tutorial = text("docs/tutorial/index.html");
    assert!(tutorial.contains("<h1 id=\"getting-started-from-the-first-page-to-the-bill-of-materials\">") && tutorial.contains("<a href=\"../../docs/plugins/\">plugins.md</a>"));
    assert!(tutorial.contains("<h2 id=\"9-reports\">9. Reports</h2>"));
    // The overview lists every document.
    let overview = text("docs/index.html");
    for file in DOCUMENTS {
        assert!(overview.contains(&format!("href=\"../docs/{}/\"", slug(file))), "{file}");
    }
    assert!(overview.contains("data-link=\"documentation\""));
    // Screenshots: the small picture of the site, or the one of the documentation.
    assert!(text("docs/schematic/index.html").contains("src=\"../../images/schematic.webp\""));
    assert!(text("docs/panel-layout/index.html").contains("src=\"../../docs/images/room.png\""));
    assert!(paths.contains("docs/images/room.png") && !paths.contains("docs/images/schematic.png"));
}
