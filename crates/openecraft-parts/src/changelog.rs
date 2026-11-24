//! The version history of OpenECraft as `CHANGELOG.md` writes it: every version with
//! its date and its changes, newest first, the series they belong to (`0.1`) and the
//! versions that are new to someone who last used an older one. Read by the editor
//! (What's new, Help › Version history) and by the release notes of the website.

/// A version of the changelog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    /// `2026-11-15`; none for the version being worked on (`unreleased`).
    pub date: Option<String>,
    /// Its changes, as Markdown.
    pub text: String,
}

impl Release {
    /// What its changes are about: the words in bold that start each of them.
    pub fn headlines(&self) -> Vec<String> {
        self.text.lines().filter_map(|line| line.strip_prefix("- **")).filter_map(|rest| rest.split("**").next()).map(|text| text.trim_end_matches(':').to_string()).collect()
    }

    /// The series it belongs to: `0.1` of `0.1.3`.
    pub fn series(&self) -> String {
        match number(&self.version) {
            Some((major, minor, _)) => format!("{major}.{minor}"),
            None => self.version.clone(),
        }
    }
}

/// The numbers of a version `major.minor.patch` (a leading `v`, a pre-release and build
/// metadata are left out), for comparing.
pub fn number(version: &str) -> Option<(u64, u64, u64)> {
    let version = version.trim();
    let version = version.strip_prefix('v').unwrap_or(version);
    let core = version.split(['-', '+']).next()?;
    let mut parts = core.split('.').map(|p| p.parse::<u64>().ok());
    let numbers = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(numbers)
}

/// Every version of a changelog, newest first as it lists them.
pub fn versions(changelog: &str) -> Vec<Release> {
    changelog
        .replace("\r\n", "\n")
        .split("\n## [")
        .skip(1)
        .filter_map(|section| {
            let (head, text) = section.split_once('\n').unwrap_or((section, ""));
            let (version, date) = head.split_once("] - ")?;
            let date = date.trim();
            let released = date.len() == 10 && date.chars().all(|c| c.is_ascii_digit() || c == '-');
            Some(Release { version: version.to_string(), date: released.then(|| date.to_string()), text: text.trim().to_string() })
        })
        .collect()
}

/// The released versions of a changelog (the version being worked on is left out).
pub fn releases(changelog: &str) -> Vec<Release> {
    versions(changelog).into_iter().filter(|r| r.date.is_some()).collect()
}

/// The history of the program of version `current`: the released versions up to it and
/// itself, also while it is being worked on; later versions are not in it.
pub fn history(changelog: &str, current: &str) -> Vec<Release> {
    let Some(now) = number(current) else { return releases(changelog) };
    versions(changelog).into_iter().filter(|r| number(&r.version).is_some_and(|n| n <= now)).collect()
}

/// The versions of `history` (newest first) that are new to someone who last used
/// version `seen` with the program now at `current`: those above `seen` up to
/// `current`. Without `seen` (a version that did not remember it) only `current`.
pub fn new_since(history: &[Release], seen: Option<&str>, current: &str) -> Vec<usize> {
    let Some(now) = number(current) else { return Vec::new() };
    let after = match seen {
        Some(seen) => match number(seen) {
            Some(n) => n,
            None => return Vec::new(),
        },
        None => return history.iter().position(|r| number(&r.version) == Some(now)).into_iter().collect(),
    };
    history.iter().enumerate().filter(|(_, r)| number(&r.version).is_some_and(|n| n > after && n <= now)).map(|(i, _)| i).collect()
}

/// The series of `history` (newest first) with the indices of their versions.
pub fn series(history: &[Release]) -> Vec<(String, Vec<usize>)> {
    let mut out: Vec<(String, Vec<usize>)> = Vec::new();
    for (i, release) in history.iter().enumerate() {
        let series = release.series();
        match out.iter_mut().find(|(s, _)| *s == series) {
            Some((_, versions)) => versions.push(i),
            None => out.push((series, vec![i])),
        }
    }
    out
}

#[cfg(test)]
mod tests;
