//! Panel layout tools: enclosure systems and their components, accessories on the
//! plate, the door and the walls, fastening, parametric placement (anchors, patterns,
//! aligning, rails), assemblies and 3D macros; floor plans on paper under the levels of
//! buildings.

use super::{Args, Tool};
use crate::session::Session;
use openecraft_api::params::{optional, param, Kind};
use openecraft_core::panel::{self, buildings as bld, floorplan};
use openecraft_i18n::{tr_const, tr_fmt};
use serde_json::{json, Value};

pub static TOOLS: &[Tool] = &[
    Tool {
        name: "enclosure_systems",
        help: tr_const!("Lists the enclosure systems with their standard sizes, wall thickness, plinth and whether they can be bayed"),
        params: &[],
        needs_project: false,
        run: enclosure_systems,
    },
    Tool {
        name: "import_floor_plan",
        help: tr_const!("Puts a floor plan on paper under a level of a building: a picture (PNG, JPEG, …) or a page of a PDF file (a scan or a drawing); its rooms are then found with detect_floor_plan and made with trace_floor_plan"),
        params: &[
            param("path", Kind::Str, tr_const!("Picture or PDF file")),
            param("building", Kind::Str, tr_const!("Building (location, e.g. +B1)")),
            param("level", Kind::Str, tr_const!("Level: its name or number (1 = the lowest)")),
            optional("page", Kind::Int, tr_const!("Page of a PDF file, from 1 (default 1)")),
            optional("scale", Kind::Num, tr_const!("Scale of the drawing: 100 for 1:100")),
            optional("dpi", Kind::Num, tr_const!("Resolution of the scan in dots per inch")),
        ],
        needs_project: true,
        run: import_floor_plan,
    },
];

fn import_floor_plan(s: &mut Session, a: &Args) -> Result<Value, String> {
    let path = a.req("path");
    let bytes = std::fs::read(path).map_err(|e| tr_fmt!("Cannot read {}: {}", path, openecraft_i18n::io_error(&e)))?;
    let name = std::path::Path::new(path).file_name().map_or_else(|| path.to_string(), |n| n.to_string_lossy().into_owned());
    let sheets = openecraft_scan::read(&name, &bytes)?;
    let page = a.int("page").unwrap_or(1);
    let sheet = usize::try_from(page - 1).ok().and_then(|k| sheets.get(k)).ok_or_else(|| tr_fmt!("The file has {} pages", sheets.len()))?;
    let prj = s.project_mut()?;
    let key = a.req("building");
    let bi = bld::building_index(&prj.layout, key).ok_or_else(|| tr_fmt!("No building named {}", key))?;
    let b = &prj.layout.buildings[bi];
    let level = a.req("level");
    let id = bld::level_index(b, level).map(|li| b.levels[li].id).ok_or_else(|| tr_fmt!("No level {} in {}", level, b.name))?;
    let mut plan = floorplan::new_plan(&name, &sheet.img, a.num("dpi").map_or(sheet.paper, floorplan::paper_of_dpi));
    if let Some(v) = a.num("scale") {
        plan.scale = v;
    }
    let out = json!({ "pages": sheets.len(), "pixels": plan.px, "dpi": (254.0 / plan.paper).round() / 10.0, "scale": plan.scale, "mm_per_pixel": plan.mm_per_px() });
    floorplan::set_plan(prj, id, Some(plan))?;
    Ok(out)
}

fn enclosure_systems(_: &mut Session, _: &Args) -> Result<Value, String> {
    Ok(Value::Array(
        panel::SYSTEMS
            .iter()
            .map(|s| {
                json!({
                    "system": s.id(),
                    "title": s.title(),
                    "sizes": s.data.sizes.iter().map(|x| [x.0, x.1, x.2]).collect::<Vec<_>>(),
                    "wall_mm": s.data.wall,
                    "flange_mm": s.data.flange,
                    "plinth_mm": s.data.plinth,
                    "bayable": s.data.bayable,
                    "protection": s.data.ip,
                })
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests;
