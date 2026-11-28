//! Integration tools: exchange with ERP, PDM, PLM, ECAD, MCAD, procurement and wiring
//! machines through the connectors of the integration layer, with profiles (column
//! mapping), a preview of imported changes and conflict detection.

use super::{optional, param, Args, Kind, Tool};
use crate::session::Session;
use openecraft_core::integration::{self, Change, CONNECTORS};
use openecraft_core::analysis::Analysis;
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use openecraft_api::builtin::integration::{connector, mapping};

pub static TOOLS: &[Tool] = &[
    Tool {
        name: "integration_connectors",
        help: tr_const!("Lists the connectors of the integration layer (ERP, PDM, PLM, ECAD, MCAD, procurement, wiring machine) with the fields they exchange, and the profiles of the project"),
        params: &[],
        needs_project: false,
        run: connectors,
    },
    Tool {
        name: "integration_export",
        help: tr_const!("Exports the project for another system with a connector, or as the neutral exchange file (connector exchange); later imports of that connector detect conflicts against this export"),
        params: &[
            param("connector", Kind::Str, tr_const!("erp, pdm, plm, ecad, mcad, procurement, wiring_machine, kicad, freecad or exchange")),
            optional("path", Kind::Str, tr_const!("Output file; without it the content is returned")),
            optional("date", Kind::Str, tr_const!("Date for the exchange log")),
        ],
        needs_project: true,
        run: export,
    },
    Tool {
        name: "integration_import",
        help: tr_const!("Reads a file of another system (CSV, JSON, a KiCad netlist or a FreeCAD document) with a connector and lists the changes; with apply they are taken over (conflicts only with overwrite)"),
        params: &[
            param("connector", Kind::Str, tr_const!("erp, pdm, plm, ecad, mcad, procurement, wiring_machine, kicad or freecad")),
            optional("path", Kind::Str, tr_const!("CSV or JSON file, KiCad netlist (.net) or FreeCAD document (.FCStd)")),
            optional("text", Kind::Str, tr_const!("File content instead of a file")),
            optional("profile", Kind::Str, tr_const!("Profile whose column mapping is used")),
            optional("mapping", Kind::Object, tr_const!("Column mapping {field code: column name} (added to the profile's)")),
            optional("apply", Kind::Bool, tr_const!("Take the changes over (otherwise only a preview)")),
            optional("overwrite", Kind::Bool, tr_const!("Also take over values changed on both sides since the last export")),
            optional("date", Kind::Str, tr_const!("Date for the exchange log")),
        ],
        needs_project: true,
        run: import,
    },
    Tool {
        name: "import_kicad",
        help: tr_const!("Reads a file of KiCad: the netlist of a board (.net) adds the board as a device (as import_kicad_board), a symbol library (.kicad_sym) adds its symbols to the project (as import_kicad_symbols)"),
        params: &[
            param("path", Kind::Str, tr_const!("KiCad netlist (.net) or symbol library (.kicad_sym)")),
            optional("board", Kind::Str, tr_const!("KiCad board file (.kicad_pcb) of the netlist: its outline gives the size of the part")),
            optional("tag", Kind::Str, tr_const!("Device tag of the board (default: the next free one)")),
            optional("connectors", Kind::Str, tr_const!("References (J1) or reference prefixes (J) of the components whose pins become connection points, separated by commas (default: the connectors)")),
            optional("after", Kind::Str, tr_const!("Insert after this page (default: at the end)")),
            optional("library", Kind::Str, tr_const!("Library the symbols are listed under (default: as the library is named in KiCad)")),
        ],
        needs_project: true,
        run: import_kicad,
    },
];

fn connectors(s: &mut Session, _: &Args) -> Result<Value, String> {
    let list: Vec<Value> = CONNECTORS
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "system": c.system.title(),
                "title": tr(c.title),
                "help": tr(c.help),
                "extension": c.extension,
                "target": c.target.title(),
                "imports": c.accepts.iter().map(|f| f.code()).collect::<Vec<_>>(),
            })
        })
        .collect();
    let profiles = s.project().map(|p| p.integration.profiles.clone()).unwrap_or_default();
    Ok(json!({
        "connectors": list,
        "fields": integration::Field::ALL.iter().map(|f| json!({ "code": f.code(), "title": f.title() })).collect::<Vec<_>>(),
        "profiles": profiles,
    }))
}

fn write_or_return(a: &Args, content: String) -> Result<Value, String> {
    match a.str("path") {
        Some(path) => {
            std::fs::write(path, &content).map_err(|e| tr_fmt!("Cannot write {}: {}", path, openecraft_i18n::io_error(&e)))?;
            Ok(json!({ "path": path, "bytes": content.len() }))
        }
        None => Ok(Value::String(content)),
    }
}

fn export(s: &mut Session, a: &Args) -> Result<Value, String> {
    let prj = s.project_mut()?;
    let an = Analysis::with_builtin(prj);
    if a.req("connector") == "exchange" {
        return write_or_return(a, integration::exchange(prj, &an));
    }
    let c = connector(a)?;
    let text = integration::export(prj, &an, c, a.str("date").unwrap_or_default());
    write_or_return(a, text)
}

pub(super) fn change_json(c: &Change) -> Value {
    json!({
        "target": format!("{:?}", c.target).to_lowercase(),
        "key": c.key.replace('|', " – "),
        "field": c.field.code(),
        "old": c.old,
        "new": c.new,
        "conflict": c.conflict,
    })
}

fn import(s: &mut Session, a: &Args) -> Result<Value, String> {
    let c = connector(a)?;
    let text = match (a.str("text"), a.str("path")) {
        (Some(t), _) => t.to_string(),
        (None, Some(path)) => integration::decode(c, &std::fs::read(path).map_err(|e| tr_fmt!("Cannot read {}: {}", path, openecraft_i18n::io_error(&e)))?)?,
        (None, None) => return Err(tr_fmt!("Missing parameter: {}", "path")),
    };
    let prj = s.project_mut()?;
    let mut map = match a.str("profile") {
        Some(name) => prj.integration.profile(name).ok_or_else(|| tr_fmt!("No profile named {}", name))?.mapping.clone(),
        None => BTreeMap::new(),
    };
    map.extend(mapping(a));
    let an = Analysis::with_builtin(prj);
    let preview = integration::import(prj, &an, c, &text, &map)?;
    let applied = if a.flag("apply") { Some(integration::apply(prj, &an, c, &preview.changes, a.flag("overwrite"), a.str("date").unwrap_or_default())) } else { None };
    Ok(json!({
        "records": preview.records,
        "changes": preview.changes.iter().map(change_json).collect::<Vec<_>>(),
        "conflicts": preview.conflicts(),
        "unmatched": preview.unmatched,
        "applied": applied,
    }))
}

/// A file of KiCad by its content: the commands of the integration plugin take the
/// content, this tool the file.
fn import_kicad(s: &mut Session, a: &Args) -> Result<Value, String> {
    let read = |path: &str| std::fs::read_to_string(path).map_err(|e| tr_fmt!("Cannot read {}: {}", path, openecraft_i18n::io_error(&e)));
    let path = a.req("path");
    let text = read(path)?;
    let mut args = a.0.clone();
    args.remove("path");
    let prj = s.project_mut()?;
    if openecraft_core::kicad::is_kind(&text, "kicad_symbol_lib") {
        let file = path.rsplit(['/', '\\']).next().unwrap_or(path);
        args.entry("library").or_insert_with(|| json!(file.rsplit_once('.').map_or(file, |x| x.0)));
        args.insert("symbols".into(), json!(text));
        return openecraft_api::builtin::integration::import_kicad_symbols(prj, &Args(&args));
    }
    if let Some(board) = args.remove("board") {
        args.insert("pcb".into(), json!(read(board.as_str().unwrap_or_default())?));
    }
    args.insert("netlist".into(), json!(text));
    openecraft_api::builtin::integration::import_kicad_board(prj, &Args(&args))
}

#[cfg(test)]
mod tests;
