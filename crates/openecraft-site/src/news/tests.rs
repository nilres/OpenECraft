use super::*;

const CHANGELOG: &str = "# Changelog\n\nThe notable changes.\n\n## [0.3.0] - unreleased\n\n- **Not out yet:** nothing to tell.\n\n## [0.2.0] - 2026-12-01\n\n- **Symbol pages:** every symbol has a page ([website](docs/website.md)).\n- **Faster & <better>:** see the [tutorial](docs/tutorial.md#9-reports).\n\n## [0.1.0] - 2026-11-04\n\n- **First release:** the editor.\n";

#[test]
fn the_released_versions_are_read_from_the_changelog() {
    let releases = releases(CHANGELOG);
    assert_eq!(releases.len(), 2, "the version being worked on is left out");
    assert_eq!((releases[0].version.as_str(), releases[0].date.as_deref().unwrap()), ("0.2.0", "2026-12-01"));
    assert_eq!(slug(&releases[0]), "v0-2-0");
    assert_eq!(releases[0].headlines(), ["Symbol pages", "Faster & <better>"]);
    assert!(releases[0].text.starts_with("- **Symbol pages:**") && releases[0].text.ends_with("#9-reports)."));
    assert_eq!(releases[1].version, "0.1.0");
    assert_eq!(super::releases(&CHANGELOG.replace('\n', "\r\n")), releases);
}

#[test]
fn every_release_has_a_page_and_the_overview_lists_them() {
    let files = site(CHANGELOG, "https://example.org/repo/");
    let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["news/v0-2-0/index.html", "news/v0-1-0/index.html", "news/index.html", "news/feed.xml"]);
    let text = |i: usize| String::from_utf8(files[i].bytes.clone()).unwrap();
    let release = text(0);
    assert!(release.contains("<title>OpenECraft 0.2.0 – release notes</title>") && release.contains("<h1>OpenECraft 0.2.0</h1>"));
    assert_eq!(release.matches("<h1").count(), 1);
    // Links lead to the documentation of the site, or to the repository through the settings.
    assert!(release.contains("<a href=\"../../docs/tutorial/#9-reports\">tutorial</a>"));
    assert!(release.contains("<a data-link=\"repository\" data-path=\"/blob/master/docs/website.md\" href=\"#\">website</a>"));
    assert!(release.contains("Faster &amp; &lt;better&gt;"));
    assert!(release.contains("<link rel=\"alternate\" type=\"application/atom+xml\" title=\"OpenECraft\" href=\"../../news/feed.xml\" />"));
    assert!(!release.contains("Not out yet"));
    let overview = text(2);
    assert!(overview.contains("href=\"../news/v0-2-0/\"") && overview.contains("href=\"../news/v0-1-0/\"") && overview.contains("href=\"../news/feed.xml\""));
    assert!(overview.contains("<li>Symbol pages</li>") && !overview.contains("Not out yet"));
}

#[test]
fn the_feed_tells_of_every_release() {
    let feed = feed(&releases(CHANGELOG), "https://example.org/repo/");
    assert!(feed.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\">\n"));
    assert!(feed.contains("<updated>2026-12-01T00:00:00Z</updated>") && feed.ends_with("</feed>\n"));
    assert_eq!(feed.matches("<entry>").count(), 2);
    assert!(feed.contains("<id>tag:openecraft.org,2026:release-0.2.0</id>") && feed.contains("<link rel=\"alternate\" type=\"text/html\" href=\"v0-2-0/\" />"));
    // The changes as escaped HTML; links relative to the folder of the news, files of
    // the repository by its address, as a feed reader has no settings of the site.
    assert!(feed.contains("&lt;a href=\"../docs/tutorial/#9-reports\"&gt;tutorial&lt;/a&gt;"));
    assert!(feed.contains("&lt;a href=\"https://example.org/repo/blob/master/docs/website.md\" rel=\"noopener\"&gt;website&lt;/a&gt;"));
    assert!(feed.contains("Faster &amp;amp; &amp;lt;better&amp;gt;") && !feed.contains("data-link"));
    assert!(feed.contains("<summary>What is new in OpenECraft 0.2.0 of 2026-12-01: Symbol pages, Faster &amp; &lt;better&gt;</summary>"));
    // Every element that is opened is closed.
    for element in ["feed", "entry", "title", "id", "updated", "summary", "content"] {
        assert_eq!(feed.matches(&format!("<{element}")).count(), feed.matches(&format!("</{element}>")).count(), "{element}");
    }
}

#[test]
fn the_changelog_of_the_repository_has_releases() {
    let changelog = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../CHANGELOG.md")).unwrap();
    let releases = releases(&changelog);
    assert!(releases.len() >= 3 && releases.iter().any(|r| r.version == "0.1.0"));
    for release in &releases {
        assert!(!release.headlines().is_empty(), "{}", release.version);
        let description = description(release);
        assert!(DESCRIPTION.contains(&description.chars().count()), "{description}");
    }
    let files = site(&changelog, "");
    assert_eq!(files.len(), releases.len() + 2);
    for file in files.iter().filter(|f| f.path.ends_with(".html")) {
        let html = String::from_utf8_lossy(&file.bytes);
        assert_eq!(html.matches("<h1").count(), 1, "{}", file.path);
        let left: Vec<&str> = html.split("href=\"").skip(1).filter_map(|rest| rest.split('"').next()).filter(|a| !a.contains("://") && a.contains(".md")).collect();
        assert!(left.is_empty(), "{}: links to Markdown files: {left:?}", file.path);
    }
}
