//! The release notes as pages of the site and as a feed: every released version of
//! `CHANGELOG.md` gets a page with its changes, the overview lists them, and
//! `news/feed.xml` (Atom) tells feed readers of a new release. In English, as the
//! changelog is written (read by `openecraft_parts::changelog`). The version being
//! worked on (`unreleased`) is left out.

use crate::docs::target;
use crate::frame::{attribute, escape, files, hero, shortened, File, Kind, Langs, Page, Place, DESCRIPTION};
use crate::markdown::{render, Target};
use openecraft_i18n::Lang;

/// The folder of the pages below the site.
pub const FOLDER: &str = "news";

/// The file of the feed below the site.
pub const FEED: &str = "news/feed.xml";

pub use openecraft_parts::changelog::{releases, Release};

/// The folder of a release below [`FOLDER`]: `v0-1-2`.
pub fn slug(release: &Release) -> String {
    format!("v{}", release.version.replace('.', "-"))
}

/// The date of a release: `2026-11-15`.
fn date(release: &Release) -> &str {
    release.date.as_deref().unwrap_or_default()
}

/// The changes of a release as HTML for a page whose way to the site is `root`; a
/// link to a file of the repository leads to `repository` (none: nowhere).
fn changes(release: &Release, root: &str, repository: Option<&str>) -> String {
    render(&release.text, &|address| match (target("CHANGELOG.md", address, root, &[]), repository) {
        (Target::Repository(path), Some(repository)) => Target::Address(format!("{}/blob/master/{path}", repository.trim_end_matches('/'))),
        (other, _) => other,
    })
    .html
}

fn description(release: &Release) -> String {
    shortened(&format!("What is new in OpenECraft {} of {}: {}", release.version, date(release), release.headlines().join(", ")), DESCRIPTION.end - 1)
}

/// The feed of the releases (Atom). Its addresses are relative to the folder of the
/// news; `website-site.sh` names the public address of that folder in it (`xml:base`).
pub fn feed(releases: &[Release], repository: &str) -> String {
    let updated = |release: &Release| format!("{}T00:00:00Z", date(release));
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    out.push_str("  <title>OpenECraft releases</title>\n  <subtitle>What is new in OpenECraft, the open source electrical CAE</subtitle>\n  <id>tag:openecraft.org,2026:releases</id>\n");
    out.push_str(&format!("  <updated>{}</updated>\n", releases.first().map(updated).unwrap_or_else(|| "2026-11-04T00:00:00Z".to_string())));
    out.push_str("  <link rel=\"alternate\" type=\"text/html\" href=\"./\" />\n  <link rel=\"self\" type=\"application/atom+xml\" href=\"feed.xml\" />\n  <author><name>OpenECraft</name></author>\n");
    let repository = (!repository.is_empty()).then_some(repository);
    for release in releases {
        out.push_str(&format!(
            "  <entry>\n    <title>OpenECraft {version}</title>\n    <id>tag:openecraft.org,2026:release-{version}</id>\n    <updated>{}</updated>\n    <link rel=\"alternate\" type=\"text/html\" href=\"{}/\" />\n    <summary>{}</summary>\n    <content type=\"html\">{}</content>\n  </entry>\n",
            updated(release),
            slug(release),
            escape(&description(release)),
            escape(&changes(release, "../", repository)),
            version = attribute(&release.version)
        ));
    }
    out.push_str("</feed>\n");
    out
}

/// The pages of the releases and the feed. `repository` is the address of the source
/// repository for the feed, whose readers have no settings of the site.
pub fn site(changelog: &str, repository: &str) -> Vec<File> {
    let releases = releases(changelog);
    let mut out = Vec::new();
    for release in &releases {
        let folder = format!("{FOLDER}/{}", slug(release));
        let place = Place::of(&folder, Lang::En);
        let main = format!(
            "<article class=\"band doc\">\n    <p class=\"eyebrow\"><a href=\"{}\">Release notes</a></p>\n    <h1>OpenECraft {}</h1>\n    <p class=\"sub\">Released on {}. <a data-link=\"downloads\" href=\"#\">Download the latest release</a></p>\n{}</article>\n",
            place.file(&format!("{FOLDER}/")),
            escape(&release.version),
            escape(date(release)),
            changes(release, &place.root(), None)
        );
        let page = Page { folder, title: format!("OpenECraft {} – release notes", release.version), description: description(release), main, kind: Kind::Plain, langs: Langs::English };
        out.extend(files(Langs::English, |_| page.clone()));
    }
    let place = Place::of(FOLDER, Lang::En);
    let actions = format!("<a class=\"button primary\" href=\"{}\">Subscribe to the feed</a><a class=\"button\" data-link=\"downloads\" href=\"#\">Download the latest release</a>", place.file(FEED));
    let mut main = hero("News", "What is new in OpenECraft", "The changes of every release. The feed tells your feed reader of a new one: no account, no newsletter, nothing stored about you.", &actions);
    main.push_str("\n<section class=\"band\">\n    <h2>Releases</h2>\n    <div class=\"cards releases\">\n");
    for release in &releases {
        let headlines: String = release.headlines().iter().take(12).map(|h| format!("<li>{}</li>", escape(h))).collect();
        main.push_str(&format!(
            "        <article class=\"card\"><h3>OpenECraft {}</h3><p>{}</p><ul>{headlines}</ul><a class=\"button\" href=\"{}\">Read the release notes</a></article>\n",
            escape(&release.version),
            escape(date(release)),
            place.file(&format!("{FOLDER}/{}/", slug(release)))
        ));
    }
    main.push_str("    </div>\n</section>\n");
    let overview = Page {
        folder: FOLDER.to_string(),
        title: "News and release notes – OpenECraft".to_string(),
        description: "What is new in OpenECraft, the open source electrical CAE: the changes of every release, with a feed that tells your feed reader of the next one.".to_string(),
        main,
        kind: Kind::Plain,
        langs: Langs::English,
    };
    out.extend(files(Langs::English, |_| overview.clone()));
    out.push(File::text(FEED, feed(&releases, repository)));
    out
}

#[cfg(test)]
mod tests;
