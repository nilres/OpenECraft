//! A board designed in KiCad as a device of the project. Its netlist names the
//! connectors and the net at each of their pins: the board becomes a device box with a
//! connection point per connector pin (`J1.1`), the nets written beside them, so that
//! the board is wired like any other device. The outline of the board file gives the
//! size of its part for the panel layout.

use super::net::{Component, Netlist};
use crate::device_boxes;
use crate::geom::{p, GRID, P};
use crate::model::{natural_cmp, DeviceBoxRole, PageKind, Part, PartRef, Project, TextItem};
use crate::sexpr::{self, Sexpr};
use openecraft_i18n::{tr, tr_fmt};
use serde_json::json;

/// Reference prefixes of connectors in KiCad's libraries and common practice.
const CONNECTOR_PREFIXES: [&str; 7] = ["J", "P", "X", "CN", "CON", "TB", "XS"];
/// Reference prefixes of what KiCad keeps with its connectors but nothing is wired to:
/// test points and jumpers.
const NO_CONNECTOR_PREFIXES: [&str; 2] = ["TP", "JP"];
/// Distance of the connection points along an edge.
const PITCH: f32 = 3.0 * GRID;
/// Height of a box: half of it for the net names of each edge.
const BOX_HEIGHT: f32 = 12.0 * GRID;
/// From the top of a box to the top of the next one.
const BOX_STEP: f32 = 22.0 * GRID;
const TEXT_SIZE: f32 = 2.5;

/// Size of a board in mm.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Outline {
    pub width: f32,
    pub height: f32,
    pub thickness: f32,
}

/// The size of a board (`.kicad_pcb`): the extent of the graphics on its `Edge.Cuts`
/// layer and the thickness of the stack-up.
pub fn outline(pcb: &str) -> Result<Outline, String> {
    let root = sexpr::parse(pcb)?;
    if root.name() != "kicad_pcb" {
        return Err(tr("Not a KiCad board file (.kicad_pcb)").into());
    }
    let (mut min, mut max) = ([f32::MAX; 2], [f32::MIN; 2]);
    let mut extend = |x: f32, y: f32| {
        min = [min[0].min(x), min[1].min(y)];
        max = [max[0].max(x), max[1].max(y)];
    };
    let point = |e: &Sexpr| Some((e.num(0)?, e.num(1)?));
    for g in root.args().iter().filter(|g| g.name().starts_with("gr_") && g.value("layer") == Some("Edge.Cuts")) {
        if g.name() == "gr_circle" {
            if let (Some(c), Some(e)) = (g.child("center").and_then(point), g.child("end").and_then(point)) {
                let r = ((e.0 - c.0).powi(2) + (e.1 - c.1).powi(2)).sqrt();
                extend(c.0 - r, c.1 - r);
                extend(c.0 + r, c.1 + r);
            }
            continue;
        }
        for q in g.args().iter().filter(|a| matches!(a.name(), "start" | "end" | "mid")).filter_map(point) {
            extend(q.0, q.1);
        }
        for q in g.child("pts").into_iter().flat_map(|pts| pts.children("xy")).filter_map(point) {
            extend(q.0, q.1);
        }
    }
    if min[0] > max[0] {
        return Err(tr("The board has no outline on the layer Edge.Cuts").into());
    }
    let thickness = root.child("general").and_then(|g| g.child("thickness")).and_then(|t| t.num(0)).unwrap_or(1.6);
    Ok(Outline { width: max[0] - min[0], height: max[1] - min[1], thickness })
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Options {
    /// Device tag of the board; empty: the next free one.
    pub tag: String,
    /// References (`J1`) or reference prefixes (`J`) of the components whose pins become
    /// connection points, separated by commas or spaces; empty: the connectors.
    pub connectors: String,
    /// Index of the page the new page follows.
    pub after: usize,
    /// Size of the board for its part.
    pub outline: Option<Outline>,
}

/// What [`import`] added.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Import {
    /// Index of the first page with the device.
    pub page: usize,
    pub pages: usize,
    pub tag: String,
    /// References of the connectors taken over.
    pub connectors: Vec<String>,
    pub points: usize,
    /// Number of the part made for the board.
    pub part: String,
}

/// The letters a reference starts with (`J` of `J12`).
fn prefix(reference: &str) -> &str {
    let end = reference.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(reference.len());
    &reference[..end]
}

fn is_connector(c: &Component, chosen: &[String]) -> bool {
    if !chosen.is_empty() {
        return chosen.iter().any(|x| x.eq_ignore_ascii_case(&c.reference) || x.eq_ignore_ascii_case(prefix(&c.reference)));
    }
    let lib = |s: &str| s.starts_with("Connector") || s.starts_with("TerminalBlock");
    let prefix = prefix(&c.reference).to_ascii_uppercase();
    CONNECTOR_PREFIXES.contains(&prefix.as_str()) || !NO_CONNECTOR_PREFIXES.contains(&prefix.as_str()) && (lib(&c.library) || lib(&c.footprint))
}

/// A connection point of the board: designation and the text beside it.
struct Point {
    designation: String,
    text: String,
}

/// The connection points in rows that fit on an edge of a box: connector after
/// connector with a gap between them (`None`).
fn rows(connectors: &[(String, Vec<Point>)], per_row: usize) -> Vec<Vec<Option<&Point>>> {
    let mut rows: Vec<Vec<Option<&Point>>> = vec![Vec::new()];
    for (_, points) in connectors {
        let used = rows.last().map_or(0, Vec::len);
        if used > 0 {
            // A connector that fits on an edge is not split; a longer one goes on where
            // there is room for a gap and a point.
            let room = if points.len() <= per_row { used + 1 + points.len() <= per_row } else { used + 2 <= per_row };
            match rows.last_mut() {
                Some(row) if room => row.push(None),
                _ => rows.push(Vec::new()),
            }
        }
        for pt in points {
            match rows.last_mut() {
                Some(row) if row.len() < per_row => row.push(Some(pt)),
                _ => rows.push(vec![Some(pt)]),
            }
        }
    }
    rows
}

/// A part number for the board that no part of the project has yet.
fn part_number(prj: &Project, name: &str) -> String {
    let base = format!("PCB.{}", super::plain_name(name));
    let mut number = base.clone();
    let mut n = 2;
    while prj.parts.iter().any(|x| x.number == number) {
        number = format!("{base}-{n}");
        n += 1;
    }
    number
}

/// Adds the board of a KiCad netlist as a device: new schematic pages after
/// `opts.after` with its device box, a connection point for every pin of its
/// connectors and the nets as texts, and a part for it with the list of the board's
/// components.
pub fn import(prj: &mut Project, netlist: &Netlist, opts: &Options) -> Result<Import, String> {
    let chosen: Vec<String> = opts.connectors.split([',', ' ', ';']).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
    let mut connectors: Vec<(String, Vec<Point>)> = Vec::new();
    for c in netlist.components.iter().filter(|c| is_connector(c, &chosen)) {
        let mut pins: Vec<(&str, String)> = Vec::new();
        for net in &netlist.nets {
            for n in net.nodes.iter().filter(|n| n.reference == c.reference) {
                // The net tells what the pin is for; a pin that connects nothing its name.
                let text = if net.unconnected() { n.function.clone() } else { net.name.trim_start_matches('/').to_string() };
                pins.push((n.pin.as_str(), text));
            }
        }
        pins.sort_by(|a, b| natural_cmp(a.0, b.0));
        pins.dedup_by(|a, b| a.0 == b.0);
        if !pins.is_empty() {
            connectors.push((c.reference.clone(), pins.into_iter().map(|(pin, text)| Point { designation: format!("{}.{}", c.reference, pin), text }).collect()));
        }
    }
    connectors.sort_by(|a, b| natural_cmp(&a.0, &b.0));
    if connectors.is_empty() {
        return Err(if chosen.is_empty() {
            tr("The netlist has no connectors (references J, P, X, CN, TB): name the components whose pins are the connections of the board").into()
        } else {
            tr_fmt!("The netlist has no component {} with connected pins", opts.connectors.trim())
        });
    }
    let name = Some(netlist.name()).filter(|n| !n.is_empty()).unwrap_or_else(|| "KiCad".to_string());
    let description = tr_fmt!("Printed circuit board {}", name);
    let area = crate::frame::project_default(prj).area();
    let left = ((area.min.x + 2.0 * GRID) / GRID).ceil() * GRID;
    let top = ((area.min.y + 4.0 * GRID) / GRID).ceil() * GRID;
    let per_row = (((area.max.x - left - 2.0 * GRID) / PITCH).floor() as usize).saturating_sub(1).max(4);
    let per_page = ((((area.max.y - top - BOX_HEIGHT) / BOX_STEP).floor() as usize) + 1).max(1);
    let rows = rows(&connectors, per_row);
    let mut out = Import { page: opts.after + 1, connectors: connectors.iter().map(|c| c.0.clone()).collect(), ..Default::default() };
    let mut tag = opts.tag.trim().to_string();
    let mut page = opts.after.min(prj.pages.len().saturating_sub(1));
    for (b, edges) in rows.chunks(2).enumerate() {
        if b % per_page == 0 {
            page = crate::ops::add_page(prj, PageKind::Schematic, (page + 1).min(prj.pages.len()));
            prj.pages[page].description = description.clone();
            if out.pages == 0 {
                out.page = page;
            }
            out.pages += 1;
        }
        let y = top + (b % per_page) as f32 * BOX_STEP;
        let slots = edges.iter().map(Vec::len).max().unwrap_or(1);
        let corner = p(left, y);
        let id = device_boxes::add_box(prj, page, corner, corner + p((slots + 1) as f32 * PITCH, BOX_HEIGHT), &tag, if b == 0 { &name } else { "" })?;
        if b == 0 {
            // The tag the box got: the one given, completed, or the next free one.
            tag = prj.pages[page].symbols.iter().find(|s| s.id == id).map(|s| s.tag.clone()).unwrap_or_default();
            out.tag = tag.clone();
        } else {
            device_boxes::set_role(prj, page, id, DeviceBoxRole::Extension)?;
        }
        for (edge, row) in edges.iter().enumerate() {
            for (i, pt) in row.iter().enumerate().filter_map(|(i, pt)| Some((i, (*pt)?))) {
                let x = left + (i + 1) as f32 * PITCH;
                let pos: P = p(x, if edge == 0 { y } else { y + BOX_HEIGHT });
                let point = device_boxes::place_point(prj, page, pos, if edge == 0 { 0 } else { 2 }, &tag, &pt.designation)?;
                out.points += 1;
                if pt.text.trim().is_empty() {
                    continue;
                }
                if let Some(s) = prj.pages[page].symbols.iter_mut().find(|s| s.id == point) {
                    s.function_text = pt.text.clone();
                }
                // The net inside the box, read from bottom to top, in the half of its edge.
                let id = prj.new_id();
                let base = y + if edge == 0 { BOX_HEIGHT / 2.0 - 1.0 } else { BOX_HEIGHT - 1.0 };
                prj.pages[page].texts.push(TextItem { id, pos: p(x + 1.0, base), text: pt.text.clone(), size: TEXT_SIZE, angle: 90.0, ..Default::default() });
            }
        }
    }
    // The part of the board, with what is on it for the documentation.
    let number = part_number(prj, &name);
    let components: Vec<serde_json::Value> = netlist.components.iter().map(|c| json!({ "reference": c.reference, "value": c.value, "footprint": c.footprint })).collect();
    let size = opts.outline.unwrap_or_default();
    prj.parts.push(Part {
        number: number.clone(),
        type_number: name.clone(),
        manufacturer: netlist.company.trim().to_string(),
        description: description.clone(),
        note: if netlist.revision.trim().is_empty() { String::new() } else { tr_fmt!("Revision {}", netlist.revision.trim()) },
        width: size.width,
        height: size.height,
        // The components stand on the board.
        depth: if size.thickness > 0.0 { size.thickness + 15.0 } else { 0.0 },
        source_data: Some(openecraft_parts::Shared::new(json!({ "kicad": { "source": netlist.source, "revision": netlist.revision, "components": components } }))),
        ..Default::default()
    });
    let device = prj.devices.entry(tag).or_default();
    if device.description.is_empty() {
        device.description = description;
    }
    device.parts.push(PartRef { number: number.clone(), count: 1, total: false });
    out.part = number;
    Ok(out)
}
