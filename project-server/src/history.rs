//! History of the ended sessions: one JSON line per session appended to
//! `sessions.jsonl` in the data directory. The last [`KEPT`] are kept in memory and
//! in the file, which is written anew when it has grown to twice as many lines.

use openecraft_core::collab::SessionRecord;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const FILE: &str = "sessions.jsonl";
/// Ended sessions that are kept.
pub const KEPT: usize = 5000;

pub struct History {
    path: Option<PathBuf>,
    /// Ended sessions kept ([`KEPT`]).
    limit: usize,
    /// Oldest first.
    records: VecDeque<SessionRecord>,
    /// Lines in the file.
    lines: usize,
}

impl History {
    /// The history of a data directory (empty if it has none). Lines that cannot be
    /// read are skipped.
    pub fn open(dir: &Path) -> io::Result<History> {
        History::open_keeping(dir, KEPT)
    }

    fn open_keeping(dir: &Path, limit: usize) -> io::Result<History> {
        let path = dir.join(FILE);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e),
        };
        let mut history = History { path: Some(path), limit, ..Default::default() };
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            history.lines += 1;
            match serde_json::from_str(line) {
                Ok(r) => history.keep(r),
                Err(e) => log::warn!("{}: skipping a line: {e}", FILE),
            }
        }
        Ok(history)
    }

    fn keep(&mut self, record: SessionRecord) {
        self.records.push_back(record);
        while self.records.len() > self.limit {
            self.records.pop_front();
        }
    }

    /// Adds an ended session; the file is written as well as possible (a failure is
    /// logged, the session ends all the same).
    pub fn add(&mut self, record: SessionRecord) {
        let line = serde_json::to_string(&record).unwrap_or_default();
        self.keep(record);
        let Some(path) = self.path.clone() else { return };
        let written = if self.lines + 1 > 2 * self.limit { self.rewrite(&path) } else { append(&path, &line).map(|_| self.lines + 1) };
        match written {
            Ok(lines) => self.lines = lines,
            Err(e) => log::warn!("cannot write {}: {e}", path.display()),
        }
    }

    /// Writes the kept sessions only; returns the lines of the file.
    fn rewrite(&self, path: &Path) -> io::Result<usize> {
        let mut text = String::new();
        for r in &self.records {
            text.push_str(&serde_json::to_string(r).unwrap_or_default());
            text.push('\n');
        }
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, path)?;
        Ok(self.records.len())
    }

    /// The last `limit` ended sessions, newest first.
    pub fn recent(&self, limit: usize) -> Vec<SessionRecord> {
        self.records.iter().rev().take(limit).cloned().collect()
    }
}

impl Default for History {
    /// A history in memory only (tests).
    fn default() -> Self {
        History { path: None, limit: KEPT, records: VecDeque::new(), lines: 0 }
    }
}

fn append(path: &Path, line: &str) -> io::Result<()> {
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")
}

#[cfg(test)]
mod tests;
