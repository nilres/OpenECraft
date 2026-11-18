//! Result tools: project check, reports, wire numbering and export.

use super::common::*;
use super::{optional, param, Args, Kind, Tool};
use crate::session::Session;
use openecraft_api::params::{self, text};
use openecraft_core::reports::self;
use openecraft_core::analysis::Analysis;
use openecraft_core::issue::Severity;
use openecraft_core::translation;
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde_json::{json, Map, Value};

pub static TOOLS: &[Tool] = &[
    Tool {
        name: "check",
        help: tr_const!("Checks the project (errors, warnings, hints)"),
        params: &[],
        needs_project: true,
        run: check,
    },
    Tool {
        name: "list_checks",
        help: tr_const!("Lists the checks of the project check (built-in and of plugins) and whether they run"),
        params: &[],
        needs_project: true,
        run: list_checks,
    },
    Tool {
        name: "list_reports",
        help: tr_const!("Lists the reports (built-in and of plugins): name for report and title"),
        params: &[optional("search", Kind::Str, tr_const!("Words to find in the titles, names and search terms (standards such as VDE 0100-600) of the reports"))],
        needs_project: false,
        run: list_reports,
    },
    Tool {
        name: "report",
        help: tr_const!("Generates a report as table"),
        params: &[
            param("kind", Kind::Str, tr_const!("Report name as shown by list_reports")),
            optional("format", Kind::Str, tr_const!("json (default), csv or xlsx (Excel, needs path)")),
            optional("scheme", Kind::Str, tr_const!("Report scheme: its filter, sort order and columns")),
            optional("path", Kind::Str, tr_const!("File to write (xlsx)")),
        ],
        needs_project: true,
        run: report,
    },
    Tool {
        name: "list_exports",
        help: tr_const!("Lists the exports (built-in and of plugins): name for export, title, file extension and parameters"),
        params: &[],
        needs_project: false,
        run: list_exports,
    },
    Tool {
        name: "export",
        help: tr_const!("Exports the project in a format of list_exports; binary formats (dwg, jt) need an output file"),
        params: &[
            param("format", Kind::Str, tr_const!("Export name as shown by list_exports")),
            optional("language", Kind::Str, tr_const!("Language of the project texts and reports (a project language, default the source language)")),
            optional("path", Kind::Str, tr_const!("Output file; without it the content is returned")),
            optional("page", Kind::Str, tr_const!("Page (svg: required, dxf, dwg: default all pages)")),
            optional("options", Kind::Str, tr_const!("Further parameters of the export as name=value, separated by commas (see list_exports), e.g. harness=-WH1")),
        ],
        needs_project: true,
        run: export,
    },
];

fn check(s: &mut Session, _: &Args) -> Result<Value, String> {
    let prj = s.project()?;
    let mut an = Analysis::build(prj);
    an.issues = s.plugins.check(prj, &an);
    an.issues.sort_by_key(|i| (i.accepted, i.severity));
    let issues: Vec<Value> = an
        .issues
        .iter()
        .map(|i| {
            json!({
                "severity": i.severity.key(),
                "message": i.message(),
                "check": i.check,
                "code": i.code(),
                "fingerprint": i.fingerprint(),
                "accepted": i.accepted,
                "key": i.key(),
                "args": i.args(),
                "page": i.target.map(|(pg, _)| prj.pages[pg].name.clone()),
                "id": i.target.map(|(_, id)| id).filter(|id| *id != 0),
            })
        })
        .collect();
    let count = |sev| an.issues.iter().filter(|i| i.severity == sev && !i.accepted).count();
    let accepted = an.issues.iter().filter(|i| i.accepted).count();
    Ok(json!({ "errors": count(Severity::Error), "warnings": count(Severity::Warning), "accepted": accepted, "issues": issues }))
}

fn list_checks(s: &mut Session, _: &Args) -> Result<Value, String> {
    let prj = s.project()?;
    let list: Vec<Value> = s
        .plugins
        .checks()
        .map(|c| json!({ "name": c.name, "title": text(&c.title), "enabled": openecraft_core::checks::is_enabled(prj, &c.name) }))
        .collect();
    Ok(Value::Array(list))
}

fn list_reports(s: &mut Session, a: &Args) -> Result<Value, String> {
    let query = a.str("search").unwrap_or("");
    Ok(Value::Array(
        s.plugins
            .reports()
            .map(|r| (r, text(&r.title).to_string()))
            .filter(|(r, t)| r.matches(t, query))
            .map(|(r, t)| json!({ "name": r.name, "title": t, "keywords": r.keywords }))
            .collect(),
    ))
}

fn report(s: &mut Session, a: &Args) -> Result<Value, String> {
    let id = report_id(&s.plugins, a.req("kind"))?;
    let prj = s.project()?;
    let mut an = Analysis::build(prj);
    an.set_reports(s.plugins.reports().cloned());
    let title = s.plugins.report(id.as_str()).map(|r| text(&r.title).to_string()).unwrap_or_else(|| id.title());
    let mut table = an.report(prj, id).map(|t| (*t).clone()).unwrap_or_default();
    if let Some(name) = a.str("scheme") {
        let scheme = prj.forms.schemes.iter().find(|s| s.kind == id && s.name == name).ok_or_else(|| tr_fmt!("Unknown report scheme: {}", name))?;
        let t = openecraft_core::report_blocks::scheme_table(&table, Some(&scheme.form));
        let cols = openecraft_core::forms::report_columns(Some(&scheme.form), &t.headers);
        table = reports::Table {
            headers: cols.iter().map(|&c| t.headers[c].clone()).collect(),
            widths: cols.iter().map(|&c| t.widths[c]).collect(),
            rows: t.rows.iter().map(|r| cols.iter().map(|&c| r.get(c).cloned().unwrap_or_default()).collect()).collect(),
            targets: t.targets,
        };
    }
    match a.str("format").unwrap_or("json") {
        "csv" => Ok(Value::String(table.to_csv())),
        "json" => Ok(json!({ "report": id.as_str(), "title": title, "headers": table.header_texts(), "rows": table.rows })),
        "xlsx" => {
            let path = a.str("path").ok_or_else(|| tr("The format xlsx needs a path").to_string())?;
            let bytes = openecraft_core::xlsx::write(&[(title, table.sheet_rows())]);
            std::fs::write(path, &bytes).map_err(|e| tr_fmt!("Cannot write {}: {}", path, openecraft_i18n::io_error(&e)))?;
            Ok(json!({ "path": path, "rows": table.rows.len() }))
        }
        other => Err(tr_fmt!("Unknown format: {}", other)),
    }
}

fn list_exports(s: &mut Session, _: &Args) -> Result<Value, String> {
    let list: Vec<Value> = s
        .plugins
        .exporters()
        .map(|e| json!({ "name": e.name, "title": text(&e.title), "extension": e.extension, "binary": e.binary, "params": e.params.iter().map(|p| p.name.as_ref()).collect::<Vec<_>>() }))
        .collect();
    Ok(Value::Array(list))
}

/// `name=value` pairs separated by commas; a part without `=` continues the value
/// before it, so page lists keep their commas (`pages=1-3, 5`).
fn split_options(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    for part in text.split(',').filter(|p| !p.trim().is_empty()) {
        let pair = part.split_once('=').filter(|(k, _)| !k.trim().is_empty() && !k.trim().contains(char::is_whitespace));
        match (pair, out.last_mut()) {
            (Some((k, v)), _) => out.push((k.trim().to_string(), v.to_string())),
            (None, Some((_, v))) => {
                v.push(',');
                v.push_str(part);
            }
            (None, None) => return Err(tr_fmt!("Option {} needs the form name=value", part.trim())),
        }
    }
    Ok(out.into_iter().map(|(k, v)| (k, v.trim().to_string())).collect())
}

/// Runs an exporter of the registry: `page` and the file name of `path` are passed
/// on if the exporter takes them; the texts are generated in `language`.
fn export(s: &mut Session, a: &Args) -> Result<Value, String> {
    let out = export_content(s, a, a.str("path"))?;
    match a.str("path") {
        Some(path) => {
            let content = out.into_bytes();
            std::fs::write(path, &content).map_err(|e| tr_fmt!("Cannot write {}: {}", path, openecraft_i18n::io_error(&e)))?;
            Ok(json!({ "path": path, "bytes": content.len() }))
        }
        None => out.text().map(Value::String).ok_or_else(|| tr("This format needs an output file (path)").to_string()),
    }
}

/// Runs the exporter `format` with `language`, `page` and `options`; `file` is the
/// name of the file it goes to.
pub(super) fn export_content(s: &Session, a: &Args, file: Option<&str>) -> Result<openecraft_api::Export, String> {
    let prj = s.project()?;
    let name = a.req("format");
    let e = s.plugins.exporter(name).cloned().ok_or_else(|| tr_fmt!("Unknown format: {}", name))?;
    let lang = match a.str("language") {
        Some(l) => {
            let code = translation::normalize_code(l).filter(|c| prj.languages.all().contains(c));
            code.ok_or_else(|| tr_fmt!("{} is not a language of the project", l))?
        }
        None => prj.languages.source.clone(),
    };
    let takes = |n: &str| e.params.iter().any(|p| p.name == n);
    let mut args = Map::new();
    if let Some(page) = a.str("page").filter(|_| takes("page")) {
        args.insert("page".into(), page.into());
    }
    if let Some(path) = file.filter(|_| takes("file")) {
        args.insert("file".into(), path.rsplit(['/', '\\']).next().unwrap_or(path).into());
    }
    for (k, v) in split_options(a.str("options").unwrap_or_default())? {
        let p = e.params.iter().find(|p| p.name == k).ok_or_else(|| tr_fmt!("Unknown parameter: {}", k))?;
        let value = p.kind.parse(&v).ok_or_else(|| tr_fmt!("Parameter {} has the wrong type", k))?;
        args.insert(k, value);
    }
    params::check(&e.params, &args)?;
    translation::with_language(prj, &lang, |prj| {
        let mut an = Analysis::build(prj);
        an.set_reports(s.plugins.reports().cloned());
        (e.run)(prj, &an, &Args(&args))
    })
}

#[cfg(test)]
mod tests;
