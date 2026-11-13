//! Enclosure packages of configurators (the Rittal configurator): a ZIP archive with
//! a bill of materials (`bill-of-material.csv`: number; quantity; unit; description;
//! EAN, without header), the configuration id (`ConfigurationId.txt`), EPLAN Pro Panel
//! data (a 3D window macro `.ema` and an EPLAN project `.zw1`), a STEP model and PDF
//! documents. The parts come from the bill of materials. The first one is the
//! configured product: it gets the files, the preview of the macro as its picture, the
//! bodies of the macro ([`crate::layout`]) or else the STEP model without its
//! restricted areas as its 3D model, the place of its mounting plate (part data
//! `Mounting origin`) and its restricted areas on the faces of its enclosures
//! (`mounting_data.restricted`); the others are its components.

use crate::archive::{text, Edz, EdzFile};
use openecraft_i18n::tr;
use openecraft_parts::portal::AttachmentKind;
use openecraft_parts::{Component, Mesh3d, Part, RestrictedArea, Restriction, TopGroup};
use serde_json::json;

/// Reads a STEP model without the products whose name the filter accepts
/// (`openecraft_step::parse_step_without`).
pub type StepReader = fn(&[u8], &dyn Fn(&str) -> bool) -> Result<Mesh3d, String>;

/// The bytes are a ZIP archive.
pub fn is_zip(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK\x03\x04")
}

/// Products of a configurator's STEP model that are areas kept free of cutouts or
/// devices, not bodies ("Restricted cut-out area 1/56", "Restricted placing area").
pub fn is_restricted_area(name: &str) -> bool {
    let name = name.to_lowercase();
    name.contains("restricted") && name.contains("area")
}

/// A line of the bill of materials.
#[derive(Clone, Debug, PartialEq)]
struct Row {
    number: String,
    quantity: f32,
    unit: String,
    description: String,
    ean: String,
}

fn rows(csv: &str) -> Vec<Row> {
    csv.trim_start_matches('\u{feff}')
        .lines()
        .filter_map(|line| {
            let field: Vec<&str> = line.split(';').map(|f| f.trim().trim_matches('"').trim()).collect();
            let number = *field.first()?;
            // A header line has no number.
            if !number.chars().any(|c| c.is_ascii_digit()) {
                return None;
            }
            let get = |k: usize| field.get(k).copied().unwrap_or_default().to_string();
            Some(Row {
                number: number.to_string(),
                quantity: get(1).replace(',', ".").parse().unwrap_or(1.0),
                unit: get(2),
                description: get(3),
                ean: get(4),
            })
        })
        .collect()
}

/// Width, height and depth from a description like `…, WHD: 600x600x350 mm, …`.
fn whd(description: &str) -> Option<[f32; 3]> {
    let at = description.find("WHD:")? + 4;
    let sizes: Vec<f32> = description[at..]
        .trim_start()
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == 'x' || c == 'X'))
        .next()?
        .split(['x', 'X'])
        .map(|s| s.parse().ok())
        .collect::<Option<_>>()?;
    match sizes[..] {
        [w, h, d] => Some([w, h, d]),
        _ => None,
    }
}

/// Value of the first attribute `name` in the XML text.
fn attribute<'a>(xml: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let at = xml.find(&key)? + key.len();
    xml[at..].split('"').next()
}

/// Manufacturer prefix of the part numbers (`RIT` of `RIT.398580`) and the
/// manufacturer's name (the folder of its macros, `Rittal\Items\…`) in the macro.
fn manufacturer(ema: &str) -> (Option<String>, Option<String>) {
    let prefix = attribute(ema, "P22001").or_else(|| attribute(ema, "P20100_1")).and_then(|n| n.split_once('.')).map(|(p, _)| p.to_string()).filter(|p| !p.is_empty());
    let name = attribute(ema, "P20468").filter(|path| !path.contains(':') && !path.starts_with('$')).and_then(|path| path.split('\\').next()).map(str::to_string).filter(|n| !n.is_empty());
    (prefix, name)
}

/// The preview picture (PNG or JPEG) of the macro: a stream of bytes
/// `#,1,1{1,2,16,1,1,<length>,<byte>,…}`.
fn preview(ema: &str) -> Option<Vec<u8>> {
    let at = ema.find("<VariantPreview")?;
    let stream = &ema[at..];
    let stream = &stream[stream.find('{')? + 1..stream.find('}')?];
    let numbers: Vec<&str> = stream.split(',').collect();
    let start = (1..numbers.len()).find(|&k| matches!(numbers[k..], ["137", "80", "78", "71", ..] | ["255", "216", "255", ..]))?;
    let len: usize = numbers[start - 1].parse().ok()?;
    numbers.get(start..start + len)?.iter().map(|n| n.parse().ok()).collect()
}

/// Millimetres with at most one decimal.
fn mm(v: f32) -> String {
    let r = (v * 10.0).round() / 10.0;
    format!("{}", if r == 0.0 { 0.0 } else { r })
}

/// Kind of the mounting surface on the front of the mounting plate.
const PLATE_FRONT: u32 = 10002;

/// Largest distance of a restricted area from the plane of the face it is put on, mm
/// (sheets and the door are thinner; the front frame around the door is farther).
const ON_FACE: f32 = 3.0;

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The macro of an enclosure in the coordinates of the panel layout's enclosures: x to
/// the right from the left, y down from the top and z towards the front from the rear
/// of its housing.
struct Housing<'a, F: Fn([f32; 3]) -> [f32; 3]> {
    layout: &'a crate::layout::Layout,
    turn: &'a F,
    min: [f32; 3],
    top: f32,
}

impl<'a, F: Fn([f32; 3]) -> [f32; 3]> Housing<'a, F> {
    fn of(layout: &'a crate::layout::Layout, turn: &'a F) -> Option<Self> {
        let (a, b) = layout.main_bounds()?;
        let (a, b) = (turn(a), turn(b));
        Some(Housing { layout, turn, min: std::array::from_fn(|k| a[k].min(b[k])), top: a[1].max(b[1]) })
    }

    fn point(&self, v: [f32; 3]) -> [f32; 3] {
        let t = (self.turn)(v);
        [t[0] - self.min[0], self.top - t[1], t[2] - self.min[2]]
    }

    /// The four corners of a rectangle on a surface.
    fn corners(&self, origin: [f32; 3], x: [f32; 3], y: [f32; 3], size: [f32; 2]) -> [[f32; 3]; 4] {
        let at = |u: f32, v: f32| self.point(std::array::from_fn(|k| origin[k] + u * x[k] + v * y[k]));
        [at(0.0, 0.0), at(size[0], 0.0), at(size[0], size[1]), at(0.0, size[1])]
    }

    /// The top left corner of the front of the mounting plate
    /// (`Enclosure::mounting_origin`).
    fn mounting_origin(&self) -> Option<[f32; 3]> {
        let s = self.layout.surfaces.iter().find(|s| s.kind == PLATE_FRONT)?;
        let c = self.corners(s.origin, s.x, s.y, s.size);
        Some(std::array::from_fn(|k| c.iter().map(|p| p[k]).fold(f32::INFINITY, f32::min)))
    }

    /// The restricted areas on the faces of an enclosure of the part (that of the
    /// panel layout), each put on the face whose plane it lies in; areas on no face
    /// (the frame around the door) are left out.
    fn areas(&self, enclosure: &openecraft_core::panel::Enclosure) -> Vec<RestrictedArea> {
        use openecraft_core::panel::{face_frame, Face};
        let frames: Vec<(Face, [[f32; 3]; 4])> = Face::ALL.into_iter().map(|f| (f, face_frame(enclosure, f))).collect();
        let r = |v: f32| (v * 100.0).round() / 100.0;
        let mut out = Vec::new();
        for a in &self.layout.areas {
            let c = self.corners(a.origin, a.x, a.y, a.size);
            let rel = |q: [f32; 3], o: [f32; 3]| [q[0] - o[0], q[1] - o[1], q[2] - o[2]];
            let (e1, e2) = (rel(c[1], c[0]), rel(c[3], c[0]));
            let normal = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            let len = dot(normal, normal).sqrt();
            if len <= 0.0 {
                continue;
            }
            let face = frames
                .iter()
                .filter(|(_, [_, _, _, n])| dot(normal, *n).abs() > 0.99 * len)
                .map(|(f, [o, u, w, n])| (c.iter().map(|q| dot(rel(*q, *o), *n).abs()).fold(0.0f32, f32::max), *f, *o, *u, *w))
                .min_by(|a, b| a.0.total_cmp(&b.0));
            let Some((_, face, o, u, w)) = face.filter(|f| f.0 <= ON_FACE) else { continue };
            let xs = c.map(|q| dot(rel(q, o), u));
            let ys = c.map(|q| dot(rel(q, o), w));
            let (x0, x1) = (xs.iter().copied().fold(f32::INFINITY, f32::min), xs.iter().copied().fold(f32::NEG_INFINITY, f32::max));
            let (y0, y1) = (ys.iter().copied().fold(f32::INFINITY, f32::min), ys.iter().copied().fold(f32::NEG_INFINITY, f32::max));
            let kind = match a.kind {
                crate::layout::AreaKind::Placing => Restriction::Placing,
                crate::layout::AreaKind::Cutout => Restriction::Cutout,
            };
            out.push(RestrictedArea { kind, face: face.key().to_string(), x: r(x0), y: r(y0), w: r(x1 - x0), h: r(y1 - y0) });
        }
        out
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Kind of a file of the package; `None` for files that are not kept.
fn kind(name: &str) -> Option<AttachmentKind> {
    let lower = name.to_lowercase();
    let ext = lower.rsplit('.').next().unwrap_or_default();
    match ext {
        "csv" | "txt" => None,
        // The EPLAN macro and project.
        "ema" | "zw1" => Some(AttachmentKind::Other),
        "pdf" if lower.starts_with("3d") => Some(AttachmentKind::Model3d),
        _ => Some(AttachmentKind::guess(name)),
    }
}

/// Reads an enclosure package; a macro without bodies leaves the 3D model to the STEP
/// file, read with `step` (none: the STEP file is only kept as a file of the part).
pub fn read(bytes: &[u8], step: Option<StepReader>) -> Result<Edz, String> {
    let files: Vec<(String, Vec<u8>)> = openecraft_core::zip::read_zip(bytes)?.into_iter().map(|(path, data)| (path.replace('\\', "/"), data)).collect();
    let lower = |path: &str| file_name(path).to_lowercase();
    let bom = files
        .iter()
        .find(|(p, _)| lower(p).ends_with(".csv") && lower(p).contains("bill"))
        .ok_or_else(|| tr("The ZIP archive is no enclosure package: it has no bill of materials (bill-of-material.csv)").to_string())?;
    let rows = rows(&text(&bom.1));
    if rows.is_empty() {
        return Err(tr("The bill of materials of the enclosure package lists no parts").to_string());
    }
    let ema = files.iter().find(|(p, _)| lower(p).ends_with(".ema")).map(|(_, d)| text(d));
    let (prefix, maker) = ema.as_deref().map(manufacturer).unwrap_or_default();
    let number = |n: &str| prefix.as_ref().map_or_else(|| n.to_string(), |p| format!("{p}.{n}"));
    let configuration = files.iter().find(|(p, _)| lower(p) == "configurationid.txt").map(|(_, d)| text(d).trim().to_string()).filter(|c| !c.is_empty());
    let part = |row: &Row| {
        let mut p = Part {
            number: number(&row.number),
            type_number: row.number.clone(),
            order_number: row.number.clone(),
            manufacturer: maker.clone().or_else(|| prefix.clone()).unwrap_or_default(),
            description: row.description.clone(),
            top_group: TopGroup::Mechanics,
            ..Part::default()
        };
        if row.unit.to_lowercase().starts_with("pc") {
            p.quantity_unit = "pcs".into();
        } else if !row.unit.is_empty() {
            p.quantity_unit.clone_from(&row.unit);
        }
        if let Some([w, h, d]) = whd(&row.description) {
            (p.width, p.height, p.depth) = (w, h, d);
        }
        let description = row.description.to_lowercase();
        let mut tech = Vec::new();
        if description.contains("enclosure") {
            p.group = "Mounting".into();
            p.subgroup = "Enclosure".into();
            if description.contains("compact enclosure") {
                tech.push("Enclosure system: compact".to_string());
            }
        }
        if !row.ean.is_empty() {
            tech.push(format!("GTIN: {}", row.ean));
        }
        p.tech_data = tech.join("\n");
        p
    };
    let mut parts: Vec<Part> = rows.iter().map(part).collect();
    let main_number = parts[0].number.clone();
    let first = &mut parts[0];
    first.components = rows[1..].iter().map(|r| Component { number: number(&r.number), quantity: r.quantity }).collect();
    if let Some(id) = &configuration {
        let sep = if first.tech_data.is_empty() { "" } else { "\n" };
        first.tech_data = format!("{}{sep}Configuration: {id}", first.tech_data);
    }
    if let Some(picture) = ema.as_deref().and_then(preview).and_then(|png| crate::picture::thumbnail(&png)) {
        first.picture = picture;
    }
    // The model: the bodies of the EPLAN macro, otherwise the STEP model. A model that
    // cannot be read leaves the part without one; the file is kept.
    let layout = ema.as_deref().and_then(crate::layout::decode).filter(|l| !l.bodies.is_empty());
    let step_file = files.iter().find(|(p, _)| lower(p).ends_with(".stp") || lower(p).ends_with(".step")).map(|(_, d)| d);
    let mut model_error = None;
    let mesh = match (&layout, step, step_file) {
        (Some(layout), ..) => Some(layout.mesh()),
        (None, Some(read), Some(model)) => read(model, &is_restricted_area).map_err(|e| model_error = Some(e)).ok(),
        _ => None,
    };
    if let Some(mut mesh) = mesh {
        let size = mesh.size();
        // Lying on its back (the height along z, the front towards -y): stood upright.
        let lying = first.height > 0.0 && (size[1] - first.depth).abs() < (size[1] - first.height).abs();
        let turn = |v: [f32; 3]| if lying { [v[0], v[2], -v[1]] } else { v };
        if let Some(housing) = layout.as_ref().and_then(|l| Housing::of(l, &turn)) {
            if let Some(origin) = housing.mounting_origin() {
                let sep = if first.tech_data.is_empty() { "" } else { "\n" };
                first.tech_data = format!("{}{sep}Mounting origin: {}", first.tech_data, origin.map(mm).join("/"));
            }
            // The faces of an enclosure of the part, as the panel layout makes it.
            let mut prj = openecraft_core::model::Project::default();
            prj.parts.push(first.clone());
            let enclosure = openecraft_core::panel::new_enclosure(&mut prj, &first.number, "");
            first.mounting_data.restricted = housing.areas(&enclosure);
        }
        for v in &mut mesh.vertices {
            *v = turn(*v);
        }
        mesh.normalize();
        if first.width <= 0.0 {
            [first.width, first.height, first.depth] = mesh.size();
        }
        first.model_3d = Some(mesh.into());
    }
    first.source_data = Some(
        json!({
            "source": "configurator package",
            "configuration": configuration,
            "bill_of_material": rows.iter().map(|r| json!({"number": r.number, "quantity": r.quantity, "unit": r.unit, "description": r.description, "ean": r.ean})).collect::<Vec<_>>(),
            "model_error": model_error,
        })
        .into(),
    );
    let mut out = Vec::new();
    for (path, data) in &files {
        let name = file_name(path);
        if let Some(kind) = kind(name).filter(|_| !out.iter().any(|f: &EdzFile| f.name == name)) {
            out.push(EdzFile { part: main_number.clone(), name: name.to_string(), kind, data: data.clone() });
        }
    }
    Ok(Edz { parts, files: out })
}

#[cfg(test)]
mod tests;
