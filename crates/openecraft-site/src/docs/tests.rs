use super::*;
use openecraft_i18n::page::file_name;
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
        for lang in LANGS.list() {
            assert!(root().join(source(file, lang)).is_file(), "{}", source(file, lang));
        }
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
    // A translated document links the pages of its language; what is not translated
    // is the same file as for the original.
    assert_eq!(source("docs/features/parts.md", Lang::De), "docs/de/features/parts.md");
    assert_eq!(source("docs/tutorial.md", Lang::En), "docs/tutorial.md");
    assert_eq!(to("docs/de/tutorial.md", "plugins.md"), page("../../docs/plugins/de.html"));
    assert_eq!(to("docs/fr/features/parts.md", "../tutorial.md#9-rapports"), page("../../docs/tutorial/fr.html#9-rapports"));
    assert_eq!(to("docs/es/features/schematic.md", "../../images/schematic.png"), page("../../images/schematic.webp"));
    assert_eq!(to("docs/de/tutorial.md", "../architecture.md"), Target::Repository("docs/architecture.md".to_string()));
    assert_eq!(to("docs/de/plugins.md", "../../examples/plugins/page_notes"), Target::Repository("examples/plugins/page_notes".to_string()));
    assert_eq!(to("docs/de/tutorial.md", "#9-auswertungen"), page("#9-auswertungen"));
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
        // In English and in three more languages, each with its own text.
        assert_eq!(html.split("<nav class=\"languages\"").nth(1).unwrap().split("</nav>").next().unwrap().matches("hreflang=").count(), LANGS.list().len(), "{file}");
        for lang in LANGS.list().into_iter().filter(|lang| *lang != Lang::En) {
            let translated = text(&format!("docs/{}/{}", slug(file), file_name(lang)));
            assert!(translated.contains(&format!("<html lang=\"{}\">", lang.code())), "{file}");
            assert_eq!(translated.matches("<h1").count(), 1, "{file}, {}", lang.code());
            assert!(translated.contains(&format!("data-path=\"/blob/master/{}\"", source(file, lang))), "{file}: its source");
            let title = translated.split("<title>").nth(1).and_then(|t| t.split("</title>").next()).unwrap().to_string();
            assert!(title.chars().count() <= 70 && title.len() > 20, "{file}, {}: {title}", lang.code());
            assert!(titles.insert(format!("{} {title}", lang.code())), "{file}, {}: its title twice", lang.code());
            let description = translated.split("<meta name=\"description\" content=\"").nth(1).and_then(|t| t.split('"').next()).unwrap();
            assert!(DESCRIPTION.contains(&description.chars().count()), "{file}, {}: {description}", lang.code());
        }
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
    // The German tutorial with the German pages around it.
    let german = text("docs/tutorial/de.html");
    assert!(german.contains("<a href=\"../../docs/plugins/de.html\">") && german.contains("<a href=\"../../docs/de.html\">Dokumentation</a>"));
    assert!(!german.contains("Getting started: from the first page"));
    assert!(text("docs/de.html").contains("href=\"../docs/tutorial/de.html\""));
    assert!(text("docs/schematic/fr.html").contains("src=\"../../images/schematic.webp\""));
}

/// What a document is made of, without its words: the headings by level, the code
/// blocks with their content, the rows of the tables, the pictures and the links.
fn structure(markdown: &str) -> (Vec<usize>, Vec<String>, usize, usize, usize) {
    let (mut headings, mut code, mut rows, mut block) = (Vec::new(), Vec::new(), 0, None::<String>);
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            match block.take() {
                Some(text) => code.push(text),
                None => block = Some(String::new()),
            }
        } else if let Some(text) = &mut block {
            text.push_str(line.trim());
            text.push('\n');
        } else if line.starts_with('#') {
            headings.push(line.chars().take_while(|c| *c == '#').count());
        } else if line.starts_with('|') {
            rows += 1;
        }
    }
    (headings, code, rows, markdown.matches("![").count(), markdown.matches("](").count())
}

#[test]
fn a_translation_has_the_structure_of_its_document() {
    let read = |file: &str| std::fs::read_to_string(root().join(file)).unwrap().replace("\r\n", "\n");
    for file in DOCUMENTS {
        let english = structure(&read(file));
        for lang in LANGS.list().into_iter().filter(|lang| *lang != Lang::En) {
            let from = source(file, lang);
            let translated = structure(&read(&from));
            assert_eq!(translated.0, english.0, "{from}: the headings");
            assert_eq!(translated.1, english.1, "{from}: the code blocks");
            assert_eq!((translated.2, translated.3, translated.4), (english.2, english.3, english.4), "{from}: table rows, pictures, links");
        }
    }
}

#[test]
fn the_links_between_the_documents_reach_their_sections() {
    let files = site(&root(), &webp()).unwrap();
    let pages: std::collections::BTreeMap<&str, String> = files.iter().filter(|f| f.path.ends_with(".html")).map(|f| (f.path.as_str(), String::from_utf8_lossy(&f.bytes).to_string())).collect();
    let mut broken = Vec::new();
    for (path, html) in &pages {
        let own = path.rsplit('/').next().unwrap();
        for address in html.split("href=\"").skip(1).filter_map(|rest| rest.split('"').next()) {
            let Some((page, section)) = address.split_once('#') else { continue };
            if address.contains("://") || section.is_empty() {
                continue;
            }
            // A section of this page, or of another document in the same language.
            let target = match page.strip_prefix("../../") {
                None if page.is_empty() => path.to_string(),
                Some(rest) if rest.starts_with("docs/") => format!("{rest}{}", if rest.ends_with('/') { "index.html" } else { "" }),
                _ => continue,
            };
            let Some(other) = pages.get(target.as_str()) else {
                broken.push(format!("{path}: no page {address}"));
                continue;
            };
            assert_eq!(target.rsplit('/').next().unwrap(), own, "{path}: {address} is in another language");
            if !other.contains(&format!(" id=\"{section}\"")) {
                broken.push(format!("{path}: no section {address}"));
            }
        }
    }
    assert!(broken.is_empty(), "{broken:#?}");
}
