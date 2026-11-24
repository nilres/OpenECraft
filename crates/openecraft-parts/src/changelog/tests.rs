use super::*;

const CHANGELOG: &str = "# Changelog\n\nThe notable changes.\n\n## [1.0.0] - unreleased\n\n- **Not out yet:** nothing to tell.\n\n## [0.2.1] - 2026-12-08\n\n- **Fix:** a small one.\n\n## [0.2.0] - 2026-12-01\n\n- **Symbol pages:** every symbol has a page ([website](docs/website.md)).\n- **Faster & <better>:** see the [tutorial](docs/tutorial.md#9-reports).\n\n## [0.1.0] - 2026-11-04\n\n- **First release:** the editor.\n";

#[test]
fn the_versions_are_read_from_the_changelog() {
    let versions = versions(CHANGELOG);
    assert_eq!(versions.iter().map(|r| r.version.as_str()).collect::<Vec<_>>(), ["1.0.0", "0.2.1", "0.2.0", "0.1.0"]);
    assert_eq!(versions[0].date, None);
    assert_eq!(versions[2].date.as_deref(), Some("2026-12-01"));
    assert_eq!(versions[2].headlines(), ["Symbol pages", "Faster & <better>"]);
    assert!(versions[2].text.starts_with("- **Symbol pages:**") && versions[2].text.ends_with("#9-reports)."));
    assert_eq!(super::versions(&CHANGELOG.replace('\n', "\r\n")), versions);
    let releases = releases(CHANGELOG);
    assert_eq!(releases.len(), 3, "the version being worked on is left out");
    assert_eq!(releases[0].version, "0.2.1");
}

#[test]
fn versions_compare_by_their_numbers() {
    assert_eq!(number("0.1.3"), Some((0, 1, 3)));
    assert_eq!(number("v1.10.0-rc.1+build"), Some((1, 10, 0)));
    assert_eq!(number("0.1"), None);
    assert_eq!(number("0.1.2.3"), None);
    assert!(number("0.1.10") > number("0.1.9"));
}

#[test]
fn the_history_ends_with_the_version_of_the_program() {
    let names = |h: &[Release]| h.iter().map(|r| r.version.clone()).collect::<Vec<_>>();
    assert_eq!(names(&history(CHANGELOG, "0.2.0")), ["0.2.0", "0.1.0"]);
    // The version being worked on is part of its own program.
    assert_eq!(names(&history(CHANGELOG, "1.0.0")), ["1.0.0", "0.2.1", "0.2.0", "0.1.0"]);
    assert_eq!(names(&history(CHANGELOG, "0.2.5")), ["0.2.1", "0.2.0", "0.1.0"]);
}

#[test]
fn what_is_new_are_the_versions_since_the_one_used_last() {
    let history = history(CHANGELOG, "0.2.1");
    assert_eq!(new_since(&history, Some("0.1.0"), "0.2.1"), [0, 1]);
    assert_eq!(new_since(&history, Some("0.2.0"), "0.2.1"), [0]);
    // The same version, or an older one started again: nothing new.
    assert!(new_since(&history, Some("0.2.1"), "0.2.1").is_empty());
    assert!(new_since(&history, Some("1.0.0"), "0.2.1").is_empty());
    // A version that did not remember it: only the version of the program.
    assert_eq!(new_since(&history, None, "0.2.1"), [0]);
    assert!(new_since(&history, None, "0.3.0").is_empty());
}

#[test]
fn the_versions_are_grouped_by_series() {
    let history = versions(CHANGELOG);
    assert_eq!(series(&history), [("1.0".to_string(), vec![0]), ("0.2".to_string(), vec![1, 2]), ("0.1".to_string(), vec![3])]);
}

#[test]
fn the_changelog_of_the_repository_has_the_version_of_the_workspace() {
    let changelog = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../CHANGELOG.md")).unwrap();
    let history = history(&changelog, env!("CARGO_PKG_VERSION"));
    assert_eq!(history[0].version, env!("CARGO_PKG_VERSION"), "CHANGELOG.md needs a section of the version");
    assert!(history.iter().any(|r| r.version == "0.1.0"));
    for release in &history {
        assert!(number(&release.version).is_some() && !release.headlines().is_empty(), "{}", release.version);
    }
}
