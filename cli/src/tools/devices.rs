//! Device tools: list, rename, assign parts.

use super::{optional, param, Args, Kind, Tool};
use crate::session::Session;
use openecraft_core::parts::{self, PartsDb};
use openecraft_i18n::{tr_const, tr_fmt};
use serde_json::{json, Value};

pub static TOOLS: &[Tool] = &[
    Tool {
        name: "assign_part",
        help: tr_const!("Assigns a part to a device (from the project, the built-in catalog or the parts server)"),
        params: &[
            param("tag", Kind::Str, tr_const!("Device tag")),
            param("number", Kind::Str, tr_const!("Part number")),
            optional("count", Kind::Int, tr_const!("Quantity (default 1)")),
            optional("replace", Kind::Bool, tr_const!("Remove the parts assigned so far")),
        ],
        needs_project: true,
        run: assign_part,
    },
];

fn assign_part(s: &mut Session, a: &Args) -> Result<Value, String> {
    let (tag, number) = (a.req("tag"), a.req("number"));
    let prj = s.project()?;
    if !prj.all_tags().iter().any(|t| t == tag) {
        return Err(tr_fmt!("No device {}", tag));
    }
    let mut db = PartsDb::default();
    let known = parts::lookup(prj, Some(&db), number).cloned();
    let part = match known {
        Some(p) => p,
        None => match s.server.get(number) {
            Ok(Some(p)) => p,
            found => {
                let similar: Vec<&str> = parts::similar(s.project().ok(), number, 8).into_iter().map(|p| p.number.as_str()).collect();
                return Err(match (found, similar.is_empty()) {
                    (_, false) => tr_fmt!("Unknown part: {}; similar parts of the project and the built-in catalog: {}", number, similar.join(", ")),
                    (Err(e), true) => tr_fmt!("Unknown part {} (parts server: {})", number, e),
                    (Ok(_), true) => tr_fmt!("Unknown part: {}", number),
                });
            }
        },
    };
    db.upsert(part);
    let prj = s.project_mut()?;
    if a.flag("replace") {
        prj.devices.entry(tag.to_string()).or_default().parts.clear();
    }
    parts::add_to_device(prj, &db, tag, number, a.int("count").unwrap_or(1).max(1) as u32);
    let assigned: Vec<Value> = prj.devices[tag].parts.iter().map(|r| json!({ "number": r.number, "count": r.count })).collect();
    let designated = parts::apply_designations(prj, Some(&db), tag);
    Ok(json!({ "tag": tag, "parts": assigned, "designations_applied": designated }))
}

#[cfg(test)]
mod tests;
