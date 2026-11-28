//! Integration commands: exchange with ERP, PDM, PLM, ECAD, MCAD, procurement and wiring
//! machines through the connectors of the integration layer, with profiles (column
//! mapping), a preview of imported changes and conflict detection; boards and symbols
//! of KiCad.

use crate::params::{optional, param, Args, Kind};
use crate::{Info, Plugin, Registry};
use super::{add_commands, info, Spec};
use openecraft_core::integration::{self, Profile};
use openecraft_core::kicad;
use openecraft_core::model::Project;
use openecraft_i18n::{tr_const, tr_fmt};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub struct Integration;

impl Plugin for Integration {
    fn info(&self) -> Info {
        info("integration", tr_const!("Integration"), tr_const!("Integration profiles and status"))
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        add_commands(reg, COMMANDS)
    }
}

pub static COMMANDS: &[Spec] = &[
    Spec {
        name: "set_integration_profile",
        help: tr_const!("Adds, replaces or removes a profile: a connector with the column names the other system uses"),
        params: &[
            param("name", Kind::Str, tr_const!("Profile name, e.g. SAP")),
            optional("connector", Kind::Str, tr_const!("Connector id (required unless removing)")),
            optional("mapping", Kind::Object, tr_const!("Column mapping {field code: column name}")),
            optional("remove", Kind::Bool, tr_const!("Remove the profile")),
        ],
        run: set_profile,
    },
    Spec {
        name: "integration_status",
        help: tr_const!("Shows the data of the other systems stored in the project (part links, documents) and the exchange log"),
        params: &[],
        run: status,
    },
    Spec {
        name: "import_kicad_board",
        help: tr_const!("Adds a board designed in KiCad as a device: a device box on a new schematic page with a connection point for every pin of its connectors (J1.1), named with the nets of the board, and a part for it"),
        params: &[
            param("netlist", Kind::Str, tr_const!("KiCad netlist of the board: the content of its .net export")),
            optional("pcb", Kind::Str, tr_const!("KiCad board: the content of its .kicad_pcb, whose outline gives the size of the part")),
            optional("tag", Kind::Str, tr_const!("Device tag of the board (default: the next free one)")),
            optional("connectors", Kind::Str, tr_const!("References (J1) or reference prefixes (J) of the components whose pins become connection points, separated by commas (default: the connectors)")),
            optional("after", Kind::Str, tr_const!("Insert after this page (default: at the end)")),
        ],
        run: import_kicad_board,
    },
    Spec {
        name: "import_kicad_symbols",
        help: tr_const!("Adds the symbols of a KiCad symbol library to the symbols of the project; symbols read before are replaced"),
        params: &[
            param("symbols", Kind::Str, tr_const!("KiCad symbol library: the content of a .kicad_sym")),
            optional("library", Kind::Str, tr_const!("Library the symbols are listed under (default: KiCad)")),
        ],
        run: import_kicad_symbols,
    },
];

pub fn connector(a: &Args) -> Result<&'static integration::Connector, String> {
    let id = a.req("connector");
    integration::connector(id).ok_or_else(|| tr_fmt!("Unknown connector: {}", id))
}

pub fn mapping(a: &Args) -> BTreeMap<String, String> {
    a.value("mapping").and_then(Value::as_object).map(|m| m.iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string()))).collect()).unwrap_or_default()
}

fn set_profile(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let name = a.req("name").trim().to_string();
    let profiles = &mut prj.integration.profiles;
    if a.flag("remove") {
        let before = profiles.len();
        profiles.retain(|p| p.name != name);
        return Ok(json!({ "removed": before - profiles.len() }));
    }
    let id = a.str("connector").ok_or_else(|| tr_fmt!("Missing parameter: {}", "connector"))?;
    integration::connector(id).ok_or_else(|| tr_fmt!("Unknown connector: {}", id))?;
    let map = mapping(a);
    if let Some(bad) = map.keys().find(|k| integration::Field::from_code(k).is_none()) {
        return Err(tr_fmt!("Unknown field: {}", bad));
    }
    let p = Profile { name: name.clone(), connector: id.to_string(), mapping: map };
    match profiles.iter_mut().find(|p| p.name == name) {
        Some(old) => *old = p,
        None => profiles.push(p),
    }
    Ok(json!({ "profiles": profiles.len() }))
}

fn status(prj: &mut Project, _: &Args) -> Result<Value, String> {
    let i = &prj.integration;
    Ok(json!({
        "items": i.items,
        "documents": i.documents,
        "project": i.project,
        "profiles": i.profiles,
        "log": i.log,
    }))
}

pub fn import_kicad_board(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let netlist = kicad::net::read(a.req("netlist"))?;
    let outline = a.str("pcb").filter(|t| !t.trim().is_empty()).map(kicad::board::outline).transpose()?;
    let after = match a.str("after") {
        Some(name) => super::common::page_index(prj, name)?,
        None => prj.pages.len().saturating_sub(1),
    };
    let opts = kicad::board::Options { tag: a.str("tag").unwrap_or_default().to_string(), connectors: a.str("connectors").unwrap_or_default().to_string(), after, outline };
    let r = kicad::board::import(prj, &netlist, &opts)?;
    let pages: Vec<&str> = prj.pages[r.page..r.page + r.pages].iter().map(|p| p.name.as_str()).collect();
    Ok(json!({ "tag": r.tag, "pages": pages, "connectors": r.connectors, "connection_points": r.points, "part": r.part }))
}

pub fn import_kicad_symbols(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let symbols = kicad::sym::import(a.req("symbols"), a.str("library").unwrap_or_default())?;
    let keys: Vec<String> = symbols.iter().map(|u| u.key.clone()).collect();
    // What the symbol editor would report: a symbol with a problem needs a look there.
    let problems: Vec<Value> = symbols.iter().map(|u| (u, openecraft_core::symlib::problems(u))).filter(|(_, p)| !p.is_empty()).map(|(u, p)| json!({ "key": u.key, "problems": p })).collect();
    let (added, replaced) = openecraft_core::symlib::merge(&mut prj.symbols, openecraft_core::symlib::SymbolLibrary { symbols });
    Ok(json!({ "added": added, "replaced": replaced, "keys": keys, "problems": problems }))
}

#[cfg(test)]
mod tests;
