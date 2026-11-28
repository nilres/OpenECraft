use super::*;

fn html(title: &str, description: &str) -> String {
    format!("<html><head>\n<title>{title}</title>\n<meta name=\"description\" content=\"{description}\" />\n</head><body></body></html>")
}

#[test]
fn an_entry_is_an_english_page_with_its_title_and_description() {
    let page = entry("docs/tutorial/index.html", &html("Getting started &amp; more – OpenECraft", "From the first page to the &quot;bill&quot; of materials.")).unwrap();
    assert_eq!(page, Entry { folder: "docs/tutorial/".into(), title: "Getting started & more".into(), description: "From the first page to the \"bill\" of materials.".into() });
    // The pages written by hand mark their title for the script.
    let main = entry("index.html", "<title data-t>OpenECraft – open source electrical CAE</title><meta name=\"description\" content=\"D\" />").unwrap();
    assert_eq!((main.folder.as_str(), main.title.as_str()), ("", "OpenECraft – open source electrical CAE"));
    assert_eq!(entry("symbols/contacts/index.html", &html("Contacts – schematic symbols | OpenECraft", "D")).unwrap().title, "Contacts – schematic symbols");
    assert_eq!(entry("docs/plugins/index.html", &html("Plugins – OpenECraft documentation", "D")).unwrap().title, "Plugins");
    // Not the page of another language, of a single symbol or of a single release.
    assert_eq!(entry("docs/tutorial/de.html", &html("T", "D")), None);
    assert_eq!(entry("symbols/no-contact/index.html", &html("T", "D")), None);
    assert_eq!(entry("news/v0-1-2/index.html", &html("T", "D")), None);
    assert!(entry("news/index.html", &html("T", "D")).is_some());
    assert_eq!(entry("faq/index.html", "<html></html>"), None);
}

#[test]
fn the_file_lists_the_pages_under_their_headings() {
    let page = |folder: &str, title: &str| Entry { folder: folder.into(), title: title.into(), description: format!("About {title}.") };
    let entries = [
        page("how-to/wire-list/", "Wire list"),
        page("", "OpenECraft"),
        page("docs/tutorial/", "Tutorial"),
        page("docs/", "Documentation [all]"),
        page("faq/", "FAQ"),
        page("compare/", "Comparison"),
        page("compare/eplan-alternative/", "EPLAN alternative"),
        page("news/", "News"),
        page("unknown/", "Unknown"),
    ];
    let text = text(&entries, "https://git.example/openecraft");
    assert!(text.starts_with("# OpenECraft\n\n> About OpenECraft.\n\nOpenECraft is free and open source"));
    // The overview of a folder first, the comparison at the start and not twice.
    let docs = text.split("\n## Documentation\n\n").nth(1).and_then(|rest| rest.split("\n## ").next()).unwrap();
    assert_eq!(docs, "- [Documentation all](./docs/): About Documentation [all].\n- [Tutorial](./docs/tutorial/): About Tutorial.\n- [FAQ](./faq/): About FAQ.\n");
    assert!(text.contains("\n## Start here\n\n- [Main page with live demos](./): ") && text.contains("\n- [Comparison](./compare/): About Comparison.\n\n## "));
    assert_eq!(text.matches("(./compare/)").count(), 1);
    assert!(text.contains("\n## Comparisons\n\n- [EPLAN alternative](./compare/eplan-alternative/): "));
    assert!(text.contains("\n## Guides\n\n- [Wire list](./how-to/wire-list/): "));
    // A heading without pages is left out, and so is a page without a heading.
    assert!(!text.contains("## Tools") && !text.contains("Unknown"));
    assert!(text.ends_with("\n## Optional\n\n- [News](./news/): About News.\n- [Source code](https://git.example/openecraft): the repository with the code, the issues and the releases to download\n"));
    // Every address of the site starts where website-site.sh puts the public one.
    assert_eq!(text.matches("](").count(), text.matches("](./").count() + 1);
    assert!(!super::text(&entries[..1], "").contains("## Optional"));
}

#[test]
fn the_headings_cover_the_sections_of_the_site() {
    let named: Vec<&str> = GROUPS.iter().flat_map(|group| group.1.iter().copied()).chain(OPTIONAL).collect();
    for (folder, _) in crate::frame::SECTIONS {
        assert!(named.contains(&folder.trim_end_matches('/')), "{folder} has no heading in llms.txt");
    }
    for folder in written_by_hand() {
        assert!(named.contains(&folder.trim_end_matches('/')), "{folder} has no heading in llms.txt");
    }
    assert_eq!(written_by_hand(), ["panel-builders/", "electricians/", "education/", "compare/"]);
}
