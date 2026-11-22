//! The tools of the site (`tools/…`, `tools.js`): single functions of OpenECraft that
//! a visitor uses on a value or a file of their own, in the browser. Each is the code
//! of the library crates: the voltage drop of the installation circuits, the sizing
//! of conductors, the short-circuit currents of the mains, the fill of cable ducts,
//! the DXF and DWG import with the SVG and PDF exports, the import of parts files.
//! No UI code; the page shows the JSON answers and nothing is sent anywhere.

use openecraft_core::dwg;
use openecraft_core::dxf_import::DxfOptions;
use openecraft_core::electrical::{Conditions, Insulation, Supply, Transformer};
use openecraft_core::installation;
use openecraft_core::model::electrical::Material;
use openecraft_core::model::Project;
use openecraft_core::panel::duct_lanes;
use openecraft_core::shortcircuit::Source;
use openecraft_core::{harness, pdf, sizing, svg};
use openecraft_i18n::{decimal, tr, tr_fmt};
use serde_json::{json, Value};

/// The names of the tools on the pages.
#[cfg(test)]
pub const TOOLS: [&str; 6] = ["voltage_drop", "cable_size", "short_circuit", "duct_fill", "drawing", "parts"];

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
        "cable_size" => cable_size(arg),
        "short_circuit" => short_circuit(arg),
        "duct_fill" => duct_fill(arg),
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

/// A value that may be left out or 0: a number of wires, a length.
fn amount(values: &Value, name: &str, label: &str) -> Result<f32, String> {
    let value = &values[name];
    if value.is_null() || value.as_str().is_some_and(|text| text.trim().is_empty()) {
        return Ok(0.0);
    }
    let number = value.as_f64().or_else(|| value.as_str().and_then(|text| text.trim().replace(',', ".").parse().ok()));
    number.filter(|n| n.is_finite() && *n >= 0.0).map(|n| n as f32).ok_or_else(|| tr_fmt!("{}: a number from 0 is needed", label))
}

fn material(values: &Value) -> Material {
    if values["material"] == "aluminium" {
        Material::Aluminium
    } else {
        Material::Copper
    }
}

fn mm2(section: f32) -> String {
    let text = if section.fract() == 0.0 { decimal(section as f64, 0) } else { decimal(section as f64, 2).trim_end_matches('0').to_string() };
    format!("{text} mm²")
}

/// The cross-section of a conductor for a current and a length: the smallest of the
/// table of the electrical sizing that carries the current under the installation
/// conditions ([`sizing::cross_section_for`]) and keeps the voltage drop within the
/// limit ([`installation::drop`]).
fn cable_size(arg: &str) -> Result<Value, String> {
    let values: Value = serde_json::from_str(arg).map_err(|e| e.to_string())?;
    let current = number(&values, "current", tr("Current"))?;
    let length = number(&values, "length", tr("Length"))?;
    let voltage = number(&values, "voltage", tr("Voltage"))?;
    let limit = number(&values, "limit", tr("Limit"))?;
    let cos_phi = number(&values, "cos_phi", "cos φ")?.min(1.0);
    let ambient = amount(&values, "ambient", tr("Ambient temperature"))?;
    let grouped = number(&values, "grouped", tr("Circuits bunched together"))?.round().max(1.0) as u32;
    let three_phase = values["phases"] == "3";
    let mut project = Project::default();
    project.electrical.conditions = Conditions { ambient, grouped, material: material(&values), insulation: if values["insulation"] == "xlpe" { Insulation::Xlpe } else { Insulation::Pvc } };
    let drop = |section: f32| installation::drop(&project, current, length, section, cos_phi, voltage, three_phase).1;
    let carries = sizing::cross_section_for(&project, current);
    let keeps = sizing::sections().find(|section| drop(*section) <= limit);
    let fits = carries.zip(keeps).map(|(a, b)| a.max(b));
    let none = || tr("none in the table").to_string();
    let rows: Vec<Value> = sizing::sections()
        .map(|section| {
            let capacity = sizing::capacity(&project, section).unwrap_or(0.0);
            let percent = drop(section);
            let verdict = match (capacity + 0.01 >= current, percent <= limit) {
                (true, true) => tr("fits"),
                (false, _) => tr("current too high"),
                (true, false) => tr("voltage drop too high"),
            };
            json!([mm2(section), format!("{} A", decimal(capacity as f64, 1)), format!("{} %", decimal(percent as f64, 2)), verdict])
        })
        .collect();
    Ok(json!({
        "ok": fits.is_some(),
        "summary": match fits {
            Some(section) => tr_fmt!("{} carries {} A and keeps the voltage drop within {} %.", mm2(section), decimal(current as f64, 1), decimal(limit as f64, 1)),
            None => tr("No cross-section of the table fits: the current or the length is too high for one conductor.").to_string(),
        },
        "facts": [
            [tr("Smallest cross-section for the current"), carries.map_or_else(none, mm2)],
            [tr("Smallest cross-section within the limit"), keeps.map_or_else(none, mm2)],
            [tr("Correction factor of the installation conditions"), decimal(project.electrical.conditions.factor() as f64, 2)],
        ],
        "headers": [tr("Cross-section"), tr("Current-carrying capacity"), tr("Voltage drop in percent"), tr("Result")],
        "rows": rows,
    }))
}

/// The short-circuit currents at the supply point and at the end of a cable
/// ([`Source::of`], [`Source::behind`]): the source is a transformer or the
/// short-circuit current known at the supply point.
fn short_circuit(arg: &str) -> Result<Value, String> {
    let values: Value = serde_json::from_str(arg).map_err(|e| e.to_string())?;
    let mut supply = Supply { voltage: number(&values, "voltage", tr("Voltage"))?, ..Supply::default() };
    if values["source"] == "current" {
        supply.ik_max = number(&values, "ik", tr("Short-circuit current"))?;
    } else {
        supply.transformer = Some(Transformer { power: number(&values, "power", tr("Rated power"))?, uk: number(&values, "uk", tr("Short-circuit voltage"))?, ..Transformer::default() });
    }
    let length = amount(&values, "length", tr("Length"))?;
    let section = number(&values, "section", tr("Cross-section"))?;
    let conditions = Conditions { material: material(&values), ..Conditions::default() };
    let source = Source::of(&supply);
    let (at_supply, at_end) = (source.behind(&supply, &conditions, 0.0, section), source.behind(&supply, &conditions, length, section));
    let ka = |amps: f32| format!("{} kA", decimal(amps as f64 / 1000.0, 2));
    Ok(json!({
        "summary": tr_fmt!("{} at the supply point, {} at the end of the cable.", ka(at_supply.ik_max), ka(at_end.ik_max)),
        "headers": [tr("Place"), tr("Highest short-circuit current"), tr("Peak current"), tr("Lowest fault current (line to PE)")],
        "rows": [
            [tr("Supply point"), ka(at_supply.ik_max), ka(at_supply.ip), ka(at_supply.ik_min)],
            [tr("End of the cable"), ka(at_end.ik_max), ka(at_end.ip), ka(at_end.ik_min)],
        ],
        "facts": [
            [tr("Breaking capacity needed at the supply point"), ka(at_supply.ik_max)],
            [tr("Current the protective device has to trip at"), format!("{} A", decimal(at_end.ik_min as f64, 0))],
        ],
    }))
}

/// How full a cable duct or a conduit is: the cross sections of the wires with their
/// insulation ([`harness::outer_diameter`]) and of the cables against the cross
/// section inside, as the panel layout counts it for its ducts, and whether the wires
/// find room side by side in a duct ([`duct_lanes`]).
fn duct_fill(arg: &str) -> Result<Value, String> {
    let values: Value = serde_json::from_str(arg).map_err(|e| e.to_string())?;
    let round = values["shape"] == "conduit";
    let width = number(&values, "width", if round { tr("Inner diameter") } else { tr("Width") })?;
    let depth = if round { width } else { number(&values, "depth", tr("Depth"))? };
    let limit = number(&values, "limit", tr("Limit"))?;
    let mut diameters: Vec<f32> = Vec::new();
    for i in 1..=3 {
        let count = amount(&values, &format!("count{i}"), tr("Number of wires"))?.round() as usize;
        if count > 0 {
            let section = number(&values, &format!("section{i}"), tr("Cross-section"))?;
            diameters.extend(std::iter::repeat_n(harness::outer_diameter(&section.to_string()), count.min(10_000)));
        }
    }
    let cables = amount(&values, "cables", tr("Number of cables"))?.round() as usize;
    if cables > 0 {
        diameters.extend(std::iter::repeat_n(number(&values, "cable_diameter", tr("Outer diameter"))?, cables.min(10_000)));
    }
    if diameters.is_empty() {
        return Err(tr("Enter at least one wire or cable").to_string());
    }
    let quarter = std::f32::consts::PI / 4.0;
    let inside = if round { quarter * width * width } else { width * depth };
    let used: f32 = diameters.iter().map(|d| quarter * d * d).sum();
    let percent = used / inside * 100.0;
    let area = |value: f32| format!("{} mm²", decimal(value as f64, 0));
    let mut facts = vec![
        json!([tr("Fill level"), format!("{} %", decimal(percent as f64, 1))]),
        json!([tr("Cross section inside"), area(inside)]),
        json!([tr("Cross section of the wires and cables"), area(used)]),
        json!([tr("Wires and cables"), diameters.len()]),
    ];
    if !round {
        facts.push(json!([tr("Room side by side"), if duct_lanes(width, depth, &diameters).1 { tr("Yes") } else { tr("No") }]));
    }
    Ok(json!({
        "ok": percent <= limit,
        "summary": match percent <= limit {
            true => tr_fmt!("{} % is within the limit of {} %.", decimal(percent as f64, 1), decimal(limit as f64, 1)),
            false => tr_fmt!("{} % is over the limit of {} %.", decimal(percent as f64, 1), decimal(limit as f64, 1)),
        },
        "facts": facts,
    }))
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
