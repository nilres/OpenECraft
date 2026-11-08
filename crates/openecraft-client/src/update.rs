//! Releases of OpenECraft on GitHub: the latest release, the comparison of versions,
//! the file of a release for a kind of installation and the check of a download
//! against the `SHA256SUMS` of the release (`docs/releasing.md`). Only the requests and
//! the decoding of their answers; the editor downloads and installs in the background.

use crate::{check, decode_found, Call};
use openecraft_i18n::tr_fmt;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::fmt;
use std::time::Duration;

/// The repository whose releases are looked at.
pub const REPOSITORY: &str = "nilres/OpenECraft";

/// The file of a release with the SHA-256 of all its other files.
pub const SUMS: &str = "SHA256SUMS";

/// Waiting for the answer of the check: a slow network does not hold up the update.
pub const CHECK_TIMEOUT: Duration = Duration::from_secs(20);

/// Waiting for a download (the installers are tens of megabytes).
pub const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// A release as the GitHub API returns it (only the fields used).
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Release {
    #[serde(default)]
    pub tag_name: String,
    /// The page of the release.
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

/// A file of a release.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    #[serde(default)]
    pub size: u64,
}

impl Release {
    pub fn version(&self) -> Option<Version> {
        Version::parse(&self.tag_name)
    }

    /// A published release with a version above `current`.
    pub fn is_newer_than(&self, current: &Version) -> bool {
        !self.draft && !self.prerelease && self.version().is_some_and(|v| v > *current)
    }

    pub fn asset(&self, name: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.name == name)
    }
}

/// A version `major.minor.patch` with an optional pre-release (`0.2.0-rc.1`); a tag
/// may start with `v`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: String,
}

impl Version {
    pub fn parse(text: &str) -> Option<Version> {
        let text = text.trim();
        let text = text.strip_prefix('v').unwrap_or(text);
        // Build metadata (`+…`) does not count.
        let text = text.split_once('+').map_or(text, |(v, _)| v);
        let (core, pre) = text.split_once('-').unwrap_or((text, ""));
        let mut parts = core.split('.').map(|p| p.parse::<u64>().ok());
        let (major, minor, patch) = (parts.next()??, parts.next()??, parts.next()??);
        if parts.next().is_some() {
            return None;
        }
        Some(Version { major, minor, patch, pre: pre.to_string() })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre)?;
        }
        Ok(())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch)).then_with(|| match (self.pre.is_empty(), other.pre.is_empty()) {
            (true, true) => Ordering::Equal,
            // A pre-release comes before its version.
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => compare_pre(&self.pre, &other.pre),
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Pre-releases by their dot separated parts: numbers by value and before words.
fn compare_pre(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.split('.'), b.split('.'));
    loop {
        match (a.next(), b.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let o = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(x), Ok(y)) => x.cmp(&y),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => x.cmp(y),
                };
                if o != Ordering::Equal {
                    return o;
                }
            }
        }
    }
}

/// The file of a release that updates an installation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Package {
    /// `OpenECraft-<version>-setup.exe` (Inno Setup).
    WindowsSetup,
    /// `OpenECraft-<version>-<arch>.AppImage`.
    AppImage,
    /// `OpenECraft-<version>-macos.dmg` with the app bundles.
    MacDiskImage,
}

impl Package {
    /// The name of the file in the release of `version` (`arch` as `uname -m`).
    pub fn file_name(self, version: &Version, arch: &str) -> String {
        match self {
            Package::WindowsSetup => format!("OpenECraft-{version}-setup.exe"),
            Package::AppImage => format!("OpenECraft-{version}-{arch}.AppImage"),
            Package::MacDiskImage => format!("OpenECraft-{version}-macos.dmg"),
        }
    }
}

/// GET the latest published release (no drafts, no pre-releases); `None` if the
/// repository has none.
pub fn latest(repository: &str) -> Call<Option<Release>> {
    let mut req = ehttp::Request::get(format!("https://api.github.com/repos/{repository}/releases/latest"));
    req.headers.insert("Accept", "application/vnd.github+json");
    req.headers.insert("User-Agent", "OpenECraft");
    req.timeout = Some(CHECK_TIMEOUT);
    Call::new(req, decode_found)
}

/// GET a file of a release (GitHub redirects to its storage).
pub fn download(asset: &Asset) -> Call<Vec<u8>> {
    let mut req = ehttp::Request::get(&asset.browser_download_url);
    req.headers.insert("Accept", "application/octet-stream");
    req.headers.insert("User-Agent", "OpenECraft");
    req.timeout = Some(DOWNLOAD_TIMEOUT);
    Call::new(req, |r| check(r).map(|r| r.bytes))
}

/// The SHA-256 of `file` in the text of `SHA256SUMS` (lines `<hash>  <file>`, binary
/// mode `<hash> *<file>`).
pub fn checksum<'a>(sums: &'a str, file: &str) -> Option<&'a str> {
    sums.lines().find_map(|line| {
        let (hash, name) = line.trim().split_once(char::is_whitespace)?;
        let name = name.trim_start();
        (name.strip_prefix('*').unwrap_or(name) == file).then_some(hash)
    })
}

/// The SHA-256 of `bytes` in hex.
pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Checks a download against `SHA256SUMS`.
pub fn verify(bytes: &[u8], sums: &str, file: &str) -> Result<(), String> {
    let expected = checksum(sums, file).ok_or_else(|| tr_fmt!("{} has no checksum in SHA256SUMS", file))?;
    if sha256(bytes).eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(tr_fmt!("The checksum of {} does not match: the download is damaged", file))
    }
}

#[cfg(test)]
mod tests;
