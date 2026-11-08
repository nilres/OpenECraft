//! Project archives and version conversion. An archive (`.oecz`) is a ZIP file with
//! the project, a manifest, the embedded documents and images as plain files and
//! optionally a PDF of all pages, so it can be read without OpenECraft. Opening a
//! project file of an older format converts it and reports what was converted.

use crate::zip::{read_zip, ZipWriter};
#[cfg(test)]
use crate::zip::{crc32, dos_date};
use openecraft_i18n::{tr, tr_fmt};
use crate::model::{Project, FORMAT};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const ARCHIVE_EXT: &str = "oecz";
const PROJECT_FILE: &str = "project.oec";
const MANIFEST_FILE: &str = "manifest.json";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArchiveOptions {
    pub comment: String,
    /// Archive date (`YYYY-MM-DD`).
    pub date: String,
    /// Adds a PDF of all pages.
    pub pdf: bool,
    /// Leaves out stored baselines and wiring marks.
    pub compact: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Manifest {
    pub generator: String,
    pub format: u32,
    pub name: String,
    pub description: String,
    pub revision: String,
    pub archived: String,
    pub comment: String,
    pub pages: usize,
    pub files: Vec<String>,
}

/// What opening a file converted.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Conversion {
    /// Format of the file.
    pub from: u32,
    pub steps: Vec<String>,
}

impl Conversion {
    pub fn converted(&self) -> bool {
        self.from < FORMAT
    }

    /// Written by a newer version: data this one does not know is lost on saving.
    pub fn newer(&self) -> bool {
        self.from > FORMAT
    }

    /// Note for the user, if there is something to say.
    pub fn message(&self) -> Option<String> {
        if self.newer() {
            Some(tr_fmt!("The file was written by a newer version (format {}): data this version does not know is lost when saving", self.from))
        } else if self.converted() {
            Some(tr_fmt!("Converted from file format {} to {}: {}", self.from, FORMAT, self.steps.join("; ")))
        } else {
            None
        }
    }
}

/// Reads a project file of any format version.
pub fn convert(text: &str) -> Result<(Project, Conversion), String> {
    let v: Value = serde_json::from_str(text).map_err(|e| tr_fmt!("Not a valid project file: {}", e))?;
    let from = v.get("format").and_then(Value::as_u64).unwrap_or(0) as u32;
    let mut steps = Vec::new();
    if from < 1 {
        let parts = v["devices"].as_object().map(|d| d.values().filter(|d| d["part"].as_str().is_some_and(|p| !p.is_empty())).count()).unwrap_or(0);
        if parts > 0 {
            steps.push(tr_fmt!("{} single part assignments converted to part lists", parts));
        }
        let props = v["pages"]
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|p| p["symbols"].as_array().into_iter().flatten())
            .flat_map(|s| s["props"].as_array().into_iter().flatten())
            .filter(|pp| pp.get("kind").is_some())
            .count();
        if props > 0 {
            steps.push(tr_fmt!("{} placed properties converted to property keys", props));
        }
        steps.push(tr("file format version added").into());
    }
    if from < 2 {
        // Report pages, blocks and forms named the built-in kinds by their variant
        // names; since format 2 they hold report names (also those of plugins).
        let kinds = v["pages"].as_array().into_iter().flatten().filter(|p| p["kind"].get("Report").is_some()).count()
            + v["report_blocks"].as_array().map(|b| b.len()).unwrap_or(0)
            + v["forms"]["schemes"].as_array().map(|s| s.len()).unwrap_or(0)
            + v["forms"]["reports"].as_object().map(|r| r.len()).unwrap_or(0);
        if kinds > 0 {
            steps.push(tr_fmt!("{} report references converted to report names", kinds));
        }
    }
    let prj = Project::from_json(text)?;
    Ok((prj, Conversion { from, steps }))
}

pub fn is_archive(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK\x03\x04")
}

/// Reads a project file or an archive.
pub fn read_project(bytes: &[u8]) -> Result<(Project, Conversion), String> {
    if is_archive(bytes) {
        let (prj, _, conv) = restore(bytes)?;
        Ok((prj, conv))
    } else {
        convert(&String::from_utf8_lossy(bytes))
    }
}

fn file_name(text: &str) -> String {
    let s: String = text.chars().map(|c| if c.is_alphanumeric() || "-_. ".contains(c) { c } else { '_' }).collect();
    let s = s.trim().to_string();
    if s.is_empty() {
        "_".into()
    } else {
        s
    }
}

/// Writes the project into an archive.
pub fn archive(prj: &Project, opts: &ArchiveOptions) -> Vec<u8> {
    let mut prj = prj.clone();
    prj.format = FORMAT;
    if opts.compact {
        prj.baselines.clear();
        prj.wiring_marks.clear();
    }
    let mut zip = ZipWriter::new(&opts.date);
    zip.add(PROJECT_FILE, prj.to_json().as_bytes());
    let mut files = vec![PROJECT_FILE.to_string()];
    let mut used = std::collections::BTreeSet::new();
    // A file name used before gets a number in front, in the folder `files/`.
    let mut unique = |name: String| {
        let mut n = format!("files/{name}");
        let mut i = 2;
        while !used.insert(n.clone()) {
            n = format!("files/{i} {name}");
            i += 1;
        }
        n
    };
    for pg in &prj.pages {
        let docs = pg.document.iter().chain(pg.images.iter().map(|i| &i.file));
        for doc in docs {
            let Ok(bytes) = doc.bytes() else { continue };
            let name = unique(format!("{} {}", file_name(&pg.name), file_name(&doc.name)));
            zip.add(&name, &bytes);
            files.push(name);
        }
    }
    if opts.pdf {
        let name = "project.pdf".to_string();
        zip.add(&name, crate::pdf::document(&prj).as_bytes());
        files.push(name);
    }
    let manifest = Manifest {
        generator: format!("OpenECraft {}", env!("CARGO_PKG_VERSION")),
        format: FORMAT,
        name: prj.info.name.clone(),
        description: prj.info.description.clone(),
        revision: prj.info.revision.clone(),
        archived: opts.date.clone(),
        comment: opts.comment.clone(),
        pages: prj.pages.len(),
        files,
    };
    zip.add(MANIFEST_FILE, serde_json::to_string_pretty(&manifest).unwrap_or_default().as_bytes());
    zip.finish()
}

/// Reads the project of an archive.
pub fn restore(bytes: &[u8]) -> Result<(Project, Manifest, Conversion), String> {
    let entries = read_zip(bytes)?;
    let get = |name: &str| entries.iter().find(|(n, _)| n == name).map(|(_, b)| b);
    let text = get(PROJECT_FILE).ok_or_else(|| tr("The archive contains no project").to_string())?;
    let manifest = get(MANIFEST_FILE).and_then(|m| serde_json::from_slice(m).ok()).unwrap_or_default();
    let (prj, conv) = convert(&String::from_utf8_lossy(text))?;
    Ok((prj, manifest, conv))
}

/// Manifest of an archive (without reading the project).
pub fn manifest(bytes: &[u8]) -> Result<Manifest, String> {
    let entries = read_zip(bytes)?;
    let m = entries.iter().find(|(n, _)| n == MANIFEST_FILE).ok_or_else(|| tr("The archive has no manifest").to_string())?;
    serde_json::from_slice(&m.1).map_err(|e| e.to_string())
}

/// Prefix of a packed project (see [`pack`]).
const PACKED: &str = "oec-deflate-base64:";

/// The project file compressed (deflate) and as base64 text: about a tenth of the
/// JSON, for stores with a size limit such as the browser's local storage.
pub fn pack(prj: &Project) -> String {
    let json = prj.to_json();
    format!("{PACKED}{}", crate::document::base64_encode(&miniz_oxide::deflate::compress_to_vec(json.as_bytes(), 6)))
}

/// A project packed by [`pack`].
pub fn unpack(text: &str) -> Result<Project, String> {
    let data = text.strip_prefix(PACKED).ok_or_else(|| tr("Not a packed project").to_string())?;
    let bytes = crate::document::base64_decode(data).ok_or_else(|| tr("Not a packed project").to_string())?;
    let json = miniz_oxide::inflate::decompress_to_vec_with_limit(&bytes, crate::zip::MAX_UNPACKED).map_err(|_| tr("Not a packed project").to_string())?;
    Project::from_json(&String::from_utf8_lossy(&json))
}

#[cfg(test)]
mod tests;
