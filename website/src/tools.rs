//! The tools of the site (`tools/…`, `tools.js`): single functions of OpenECraft that
//! a visitor uses on a value or a file of their own, in the browser. Each is the code
//! of the library crates: the voltage drop of the installation circuits, the DXF and
//! DWG import with the SVG and PDF exports, the import of parts files. No UI code;
//! the page shows the JSON answers and nothing is sent anywhere.

use openecraft_core::dwg;
use openecraft_core::dxf_import::DxfOptions;
use openecraft_core::installation;
use openecraft_core::model::electrical::Material;
use openecraft_core::model::Project;
use openecraft_core::{pdf, svg};
use openecraft_i18n::{decimal, tr, tr_fmt};
use serde_json::{json, Value};

/// The names of the tools on the pages.
#[cfg(test)]
pub const TOOLS: [&str; 3] = ["voltage_drop", "drawing", "parts"];

/// The files a tool writes from the file it was given: name, ending.
pub const FILES: [(&str, &str); 6] = [("drawing_svg", "svg"), ("drawing_pdf", "pdf"), ("drawing_dxf", "dxf"), ("drawing_dwg", "dwg"), ("parts_csv", "csv"), ("parts_json", "json")];

/// The nominal cross-sections the voltage drop is shown for, in mm².
pub const SECTIONS: [f32; 15] = [1.5, 2.5, 4.0, 6.0, 10.0, 16.0, 25.0, 35.0, 50.0, 70.0, 95.0, 120.0, 150.0, 185.0, 240.0];

/// At most this many parts of a file are shown.
pub const PART_ROWS: usize = 500;

/// A tool by its name, with its values as JSON and the file it works on. The answer
/// is JSON; an error is `{"error": "…"}`.
pub fn call(name: &str, arg: &str, bytes: &[u8]) -> Value {
    let result = match name {
        "voltage_drop" => voltage_drop(arg),
        "drawing" => drawing(bytes),
        "parts" => parts(bytes),
        other => Err(tr_fmt!("Unknown command: {}", other)),
    };
    result.unwrap_or_else(|e| json!({ "error": e }))
}

fn number(values: &Value, name: &str, label: &str) -> Result<f32, String> {
    let value = &values[name];
    let number = value.as_f64().or_else(|| value.as_str().and_then(|text| text.trim().replace(',', ".").parse().ok()));
    number.filter(|n| n.is_finite() && *n > 0.0).map(|n| n as f32).ok_or_else(|| tr_fmt!("{}: a number above 0 is needed", label))
}

/// The voltage drop of a cable ([`installation::drop`]) for the cross-section asked
/// for and for the standard ones, with the smallest that keeps the limit.
fn voltage_drop(arg: &str) -> Result<Value, String> {
    let values: Value = serde_json::from_str(arg).map_err(|e| e.to_string())?;
    let current = number(&values, "current", tr("Current"))?;
    let length = number(&values, "length", tr("Length"))?;
    let section = number(&values, "section", tr("Cross-section"))?;
    let voltage = number(&values, "voltage", tr("Voltage"))?;
    let limit = number(&values, "limit", tr("Limit"))?;
    let cos_phi = number(&values, "cos_phi", "cos φ")?.min(1.0);
    let three_phase = values["phases"] == "3";
    let mut project = Project::default();
    project.electrical.conditions.material = if values["material"] == "aluminium" { Material::Aluminium } else { Material::Copper };
    let drop = |mm2: f32| installation::drop(&project, current, length, mm2, cos_phi, voltage, three_phase);
    let (volts, percent) = drop(section);
    let fits = SECTIONS.into_iter().find(|mm2| drop(*mm2).1 <= limit);
    let verdict = match percent <= limit {
        true => tr_fmt!("{} % is within the limit of {} %.", decimal(percent as f64, 2), decimal(limit as f64, 1)),
        false => tr_fmt!("{} % is over the limit of {} %.", decimal(percent as f64, 2), decimal(limit as f64, 1)),
    };
    let rows: Vec<Value> = SECTIONS
        .into_iter()
        .map(|mm2| {
            let (v, pct) = drop(mm2);
            json!([format!("{} mm²", decimal(mm2 as f64, 1)), format!("{} V", decimal(v as f64, 2)), format!("{} %", decimal(pct as f64, 2)), if pct <= limit { tr("within the limit") } else { tr("over the limit") }])
        })
        .collect();
    Ok(json!({
        "ok": percent <= limit,
        "summary": verdict,
        "facts": [
            [tr("Voltage drop"), format!("{} V", decimal(volts as f64, 2))],
            [tr("Voltage drop in percent"), format!("{} %", decimal(percent as f64, 2))],
            [tr("Smallest cross-section within the limit"), fits.map_or_else(|| tr("none up to 240 mm²").to_string(), |mm2| format!("{} mm²", decimal(mm2 as f64, 1)))],
        ],
        "headers": [tr("Cross-section"), tr("Voltage drop"), tr("Voltage drop in percent"), tr("Result")],
        "rows": rows,
    }))
}

/// A DXF or DWG drawing on a page of a project of its own.
fn imported(bytes: &[u8]) -> Result<(Project, openecraft_core::dxf_import::DxfImport), String> {
    if bytes.is_empty() {
        return Err(tr("Choose a file first").to_string());
    }
    let mut project = Project::default();
    let import = dwg::import_drawing(&mut project, bytes, "", 0, DxfOptions::default())?;
    Ok((project, import))
}

/// The drawing as the SVG export draws its page, with what the import read.
fn drawing(bytes: &[u8]) -> Result<Value, String> {
    let (project, import) = imported(bytes)?;
    let size = (import.source_max.x - import.source_min.x, import.source_max.y - import.source_min.y);
    let mut facts = vec![
        json!([tr("Format"), if dwg::is_dwg(bytes) { "DWG" } else { "DXF" }]),
        json!([tr("Lines"), import.lines]),
        json!([tr("Texts"), import.texts]),
        json!([tr("Size of the drawing"), format!("{} × {} mm", decimal(size.0 as f64, 1), decimal(size.1 as f64, 1))]),
        json!([tr("Scale on the page"), decimal(import.scale as f64, 3)]),
    ];
    if !import.skipped.is_empty() {
        facts.push(json!([tr("Not imported"), import.skipped.join(", ")]));
    }
    Ok(json!({ "svg": svg::page_svg(&project, import.page), "facts": facts, "files": if dwg::is_dwg(bytes) { ["drawing_svg", "drawing_pdf", "drawing_dxf"] } else { ["drawing_svg", "drawing_pdf", "drawing_dwg"] } }))
}

/// The parts of a parts file (EDZ, EPLAN XML, BMEcat, CSV, JSON) as a table.
fn parts(bytes: &[u8]) -> Result<Value, String> {
    if bytes.is_empty() {
        return Err(tr("Choose a file first").to_string());
    }
    let parts = openecraft_edz::import(bytes)?;
    let size = |part: &openecraft_parts::Part| match part.width > 0.0 || part.height > 0.0 || part.depth > 0.0 {
        true => format!("{} × {} × {}", decimal(part.width as f64, 1), decimal(part.height as f64, 1), decimal(part.depth as f64, 1)),
        false => String::new(),
    };
    let rows: Vec<Value> = parts.iter().take(PART_ROWS).map(|p| json!([p.number, p.manufacturer, p.type_number, p.order_number, p.description, p.group, size(p), p.functions.len()])).collect();
    Ok(json!({
        "summary": if parts.len() > PART_ROWS { tr_fmt!("The first {} of {} parts", PART_ROWS, parts.len()) } else { tr_fmt!("{} parts", parts.len()) },
        "headers": [tr("Part number"), tr("Manufacturer"), tr("Type number"), tr("Order number"), tr("Description"), tr("Product group"), tr("Width × height × depth (mm)"), tr("Functions")],
        "rows": rows,
        "files": ["parts_csv", "parts_json"],
    }))
}

/// A file a tool writes from the file it was given ([`FILES`]).
pub fn file(name: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
    let known = FILES.iter().any(|file| file.0 == name);
    match name {
        "drawing_svg" => imported(bytes).map(|(project, import)| svg::page_svg(&project, import.page).into_bytes()),
        "drawing_pdf" => imported(bytes).map(|(project, _)| pdf::document(&project).into_bytes()),
        "drawing_dxf" => dwg::to_dxf(bytes).map(String::into_bytes),
        "drawing_dwg" => dwg::from_dxf(&String::from_utf8_lossy(bytes)),
        "parts_csv" => openecraft_edz::import(bytes).map(|parts| openecraft_parts::to_csv(&parts).into_bytes()),
        "parts_json" => openecraft_edz::import(bytes).and_then(|parts| serde_json::to_vec_pretty(&parts).map_err(|e| e.to_string())),
        _ if known => Err(tr_fmt!("Unknown export: {}", name)),
        other => Err(tr_fmt!("Unknown export: {}", other)),
    }
}

#[cfg(test)]
mod tests;
