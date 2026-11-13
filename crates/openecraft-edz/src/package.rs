//! Enclosure packages of configurators (the Rittal configurator): a ZIP archive with
//! a bill of materials (`bill-of-material.csv`: number; quantity; unit; description;
//! EAN, without header), the configuration id (`ConfigurationId.txt`), EPLAN Pro Panel
//! data (a 3D window macro `.ema` and an EPLAN project `.zw1`), a STEP model and PDF
//! documents. The parts come from the bill of materials. The first one is the
//! configured product: it gets the files, the preview of the macro as its picture and
//! the STEP model without its restricted areas as its 3D model; the others are its
//! components.

use crate::archive::{text, Edz, EdzFile};
use openecraft_i18n::tr;
use openecraft_parts::portal::AttachmentKind;
use openecraft_parts::{Component, Mesh3d, Part, TopGroup};
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

/// Turns a model with the height along z (as CAD models of enclosures are) so that the
/// height is along y and the front, which faced -y, faces +z.
fn upright(mesh: &mut Mesh3d) {
    for v in &mut mesh.vertices {
        *v = [v[0], v[2], -v[1]];
    }
    mesh.normalize();
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

/// Reads an enclosure package; the 3D model is read with `step` (none: the STEP file
/// is only kept as a file of the part).
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
    // A model that cannot be read leaves the part without one; the file is kept.
    let mut model_error = None;
    if let (Some(read), Some((_, model))) = (step, files.iter().find(|(p, _)| lower(p).ends_with(".stp") || lower(p).ends_with(".step"))) {
        match read(model, &is_restricted_area) {
            Ok(mut mesh) => {
                let size = mesh.size();
                // Lying on its back: the depth along y.
                if first.height > 0.0 && (size[1] - first.depth).abs() < (size[1] - first.height).abs() {
                    upright(&mut mesh);
                }
                if first.width <= 0.0 {
                    [first.width, first.height, first.depth] = mesh.size();
                }
                first.model_3d = Some(mesh.into());
            }
            Err(e) => model_error = Some(e),
        }
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
