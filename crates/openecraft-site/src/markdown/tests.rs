use super::*;

fn plain(address: &str) -> Target {
    Target::Address(address.to_string())
}

#[test]
fn headings_get_the_anchors_of_github() {
    assert_eq!(anchor("Web (WebAssembly)"), "web-webassembly");
    assert_eq!(anchor("Docker images of the servers"), "docker-images-of-the-servers");
    assert_eq!(anchor("1. A new project"), "1-a-new-project");
    assert_eq!(anchor("Legal notice, disclaimer and privacy notice"), "legal-notice-disclaimer-and-privacy-notice");
    let out = render("# Title\n\nFirst paragraph\nof two lines.\n\n## Part `one`\n\nText.\n\n## Part `one`\n", &plain);
    assert_eq!(out.title, "Title");
    assert_eq!(out.summary, "First paragraph of two lines.");
    assert!(out.html.contains("<h1 id=\"title\">Title</h1>"));
    assert!(out.html.contains("<h2 id=\"part-one\">Part <code>one</code></h2>"));
    assert!(out.html.contains("<h2 id=\"part-one-1\">"), "a second heading of the same text: {}", out.html);
}

#[test]
fn a_page_has_one_first_heading() {
    let out = render("# Title\n\nText.\n\n# Another\n", &plain);
    assert_eq!(out.html.matches("<h1").count(), 1);
    assert!(out.html.contains("<h2 id=\"another\">Another</h2>"));
}

#[test]
fn paths_are_resolved_against_the_document() {
    assert_eq!(resolved("docs/features/parts.md", "../tutorial.md"), "docs/tutorial.md");
    assert_eq!(resolved("docs/tutorial.md", "features/parts.md"), "docs/features/parts.md");
    assert_eq!(resolved("docs/tutorial.md", "./images/a.png"), "docs/images/a.png");
    assert_eq!(resolved("CHANGELOG.md", "docs/releasing.md"), "docs/releasing.md");
    assert_eq!(resolved("docs/tutorial.md", "../crates/openecraft-api/src/builtin.rs"), "crates/openecraft-api/src/builtin.rs");
}

#[test]
fn links_lead_where_the_site_has_their_target() {
    let link = |address: &str| match address {
        "plugins.md" => Target::Address("../plugins/".to_string()),
        "images/a.png" => Target::Address("../../images/a.webp".to_string()),
        "../Cargo.toml" => Target::Repository("Cargo.toml".to_string()),
        other => Target::Address(other.to_string()),
    };
    let out = render("See [plugins](plugins.md), [the manifest](../Cargo.toml), [a site](https://example.org/a?b=1&c=2) and [below](#part).\n\n![A picture](images/a.png)\n", &link);
    assert!(out.html.contains("<a href=\"../plugins/\">plugins</a>"));
    assert!(out.html.contains("<a data-link=\"repository\" data-path=\"/blob/master/Cargo.toml\" href=\"#\">the manifest</a>"));
    assert!(out.html.contains("<a href=\"https://example.org/a?b=1&amp;c=2\" rel=\"noopener\">a site</a>"));
    assert!(out.html.contains("<a href=\"#part\">below</a>"));
    assert!(out.html.contains("<img loading=\"lazy\" src=\"../../images/a.webp\" alt=\"A picture\""));
    assert_eq!(out.pictures, ["../../images/a.webp"]);
}

#[test]
fn tables_scroll_and_markup_of_a_document_is_text() {
    let out = render("| a | b |\n| --- | --- |\n| 1 | 2 |\n\nPress <kbd>F1</kbd>.\n\n<script>alert(1)</script>\n", &plain);
    assert!(out.html.contains("<div class=\"table-scroll\"><table>") && out.html.contains("</table></div>"));
    assert!(out.html.contains("&lt;kbd&gt;F1&lt;/kbd&gt;") && !out.html.contains("<kbd>"));
    assert!(!out.html.contains("<script>") && out.html.contains("&lt;script&gt;"));
}
