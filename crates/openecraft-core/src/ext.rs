//! Extension points: what a plugin or a built-in area adds to a [`Registry`]. The
//! front ends (editor, command line, MCP server) take commands, checks, reports and
//! exporters from the registries of `openecraft_api::Plugins`, so an extension is
//! available everywhere at once.

use crate::model::Project;
use crate::params::{Args, Param, Text};
use crate::analysis::Analysis;
use crate::issue::Issue;
use crate::table::Table;
use serde_json::Value;
use std::sync::Arc;

pub type CommandFn = dyn Fn(&mut Project, &Args) -> Result<Value, String> + Send + Sync;
pub type CheckFn = dyn Fn(&Project, &Analysis) -> Result<Vec<Issue>, String> + Send + Sync;
pub type ReportFn = dyn Fn(&Project, &Analysis) -> Result<Table, String> + Send + Sync;
pub type ExportFn = dyn Fn(&Project, &Analysis, &Args) -> Result<Export, String> + Send + Sync;

/// What an exporter produced: a text file or a binary one.
#[derive(Clone, Debug, PartialEq)]
pub enum Export {
    Text(String),
    Binary(Vec<u8>),
}

impl Export {
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Export::Text(t) => t.into_bytes(),
            Export::Binary(b) => b,
        }
    }

    /// The content of a text export; `None` for a binary one.
    pub fn text(self) -> Option<String> {
        match self {
            Export::Text(t) => Some(t),
            Export::Binary(_) => None,
        }
    }
}

/// A function on the project with typed parameters. Offered as command of the command
/// line, tool of the MCP server and entry of the editor's Plugins menu; changes of the
/// project become one undo step.
#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub help: Text,
    pub params: Vec<Param>,
    pub run: Arc<CommandFn>,
}

/// Adds its issues to the project check.
#[derive(Clone)]
pub struct Check {
    pub name: String,
    pub title: Text,
    pub run: Arc<CheckFn>,
}

/// A table generated from the project (editor: Reports, command line: `report`).
#[derive(Clone)]
pub struct Report {
    pub name: String,
    pub title: Text,
    /// Further search terms besides the title, not translated: the standards the report
    /// follows (`DIN VDE 0100-600`, `IEC 60364-6`), other names of it.
    pub keywords: Vec<String>,
    pub run: Arc<ReportFn>,
}

impl Report {
    /// Whether a search finds the report: every word of `query` is in its title as shown,
    /// its name or one of its keywords (all in one of them), ignoring case, spaces and
    /// punctuation, so that `VDE 0100 600` finds `DIN VDE 0100-600`.
    pub fn matches(&self, title: &str, query: &str) -> bool {
        let words: Vec<String> = query.split_whitespace().map(squeeze).filter(|w| !w.is_empty()).collect();
        words.is_empty()
            || [title, self.name.as_str()].into_iter().chain(self.keywords.iter().map(String::as_str)).map(squeeze).any(|t| words.iter().all(|w| t.contains(w.as_str())))
    }
}

/// Lower case letters and digits of a text, without spaces and punctuation.
fn squeeze(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// A file generated from the project (editor: File › Export, command line: `export`).
/// `params` are what the host may pass (by convention `page`, the name of one page,
/// and `file`, the name the file is written under).
#[derive(Clone)]
pub struct Exporter {
    pub name: String,
    pub title: Text,
    /// File extension without the dot.
    pub extension: String,
    /// The export is binary (it can only be written to a file).
    pub binary: bool,
    pub params: Vec<Param>,
    pub run: Arc<ExportFn>,
}

/// What one plugin adds. Names are snake_case and unique among all plugins (commands
/// also among the built-in commands of the front ends), see `openecraft_api::Plugins::add`.
#[derive(Clone, Default)]
pub struct Registry {
    pub commands: Vec<Command>,
    pub checks: Vec<Check>,
    pub reports: Vec<Report>,
    pub exporters: Vec<Exporter>,
}

impl Registry {
    pub fn command(&mut self, name: impl Into<String>, help: impl Into<Text>, params: Vec<Param>, run: impl Fn(&mut Project, &Args) -> Result<Value, String> + Send + Sync + 'static) -> &mut Self {
        self.commands.push(Command { name: name.into(), help: help.into(), params, run: Arc::new(run) });
        self
    }

    pub fn check(&mut self, name: impl Into<String>, title: impl Into<Text>, run: impl Fn(&Project, &Analysis) -> Result<Vec<Issue>, String> + Send + Sync + 'static) -> &mut Self {
        self.checks.push(Check { name: name.into(), title: title.into(), run: Arc::new(run) });
        self
    }

    pub fn report(&mut self, name: impl Into<String>, title: impl Into<Text>, run: impl Fn(&Project, &Analysis) -> Result<Table, String> + Send + Sync + 'static) -> &mut Self {
        self.reports.push(Report { name: name.into(), title: title.into(), keywords: Vec::new(), run: Arc::new(run) });
        self
    }

    /// Sets the search terms of the report added last ([`Report::keywords`]).
    pub fn keywords<S: Into<String>>(&mut self, words: impl IntoIterator<Item = S>) -> &mut Self {
        if let Some(r) = self.reports.last_mut() {
            r.keywords = words.into_iter().map(Into::into).collect();
        }
        self
    }

    /// A text export without parameters.
    pub fn exporter(&mut self, name: impl Into<String>, title: impl Into<Text>, extension: impl Into<String>, run: impl Fn(&Project) -> Result<String, String> + Send + Sync + 'static) -> &mut Self {
        self.exporter_with(name, title, extension, false, Vec::new(), move |prj, _, _| run(prj).map(Export::Text))
    }

    /// An export with parameters, binary if `binary`.
    pub fn exporter_with(
        &mut self,
        name: impl Into<String>,
        title: impl Into<Text>,
        extension: impl Into<String>,
        binary: bool,
        params: Vec<Param>,
        run: impl Fn(&Project, &Analysis, &Args) -> Result<Export, String> + Send + Sync + 'static,
    ) -> &mut Self {
        self.exporters.push(Exporter { name: name.into(), title: title.into(), extension: extension.into(), binary, params, run: Arc::new(run) });
        self
    }

    /// Names of everything registered, with the kind (for the uniqueness check).
    pub fn names(&self) -> impl Iterator<Item = (&'static str, &str)> {
        let c = self.commands.iter().map(|x| ("command", x.name.as_str()));
        let k = self.checks.iter().map(|x| ("check", x.name.as_str()));
        let r = self.reports.iter().map(|x| ("report", x.name.as_str()));
        let e = self.exporters.iter().map(|x| ("exporter", x.name.as_str()));
        c.chain(k).chain(r).chain(e)
    }
}

#[cfg(test)]
mod tests;
