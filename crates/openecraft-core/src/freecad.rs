//! Exchange with FreeCAD, the mechanical CAD. The panel layout is written as a macro
//! (Python) that builds it in FreeCAD: a part container per enclosure and an object per
//! component, rail, duct and device, named by the device tags, as solids the mechanical
//! design can measure and cut. Every device carries its tag, part, place and the
//! directions of the layout as properties, so that the document saved in FreeCAD
//! (`.FCStd`) tells where the devices were moved to ([`placements`]): the way back
//! through the integration layer.
//!
//! Part models come from FreeCAD as STEP files, like those of every other MCAD system.

use crate::export3d::{self, Solid};
use crate::model::Project;
use crate::panel::{self, Enclosure, ItemKind, PanelItem};
use crate::xml::{self, Element};
use openecraft_i18n::tr;
use std::collections::HashMap;
use std::fmt::Write;

fn text(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

/// A number of a coordinate list: three decimals, no trailing zeros.
fn coord(v: f32) -> String {
    let v = if v.abs() < 5e-4 { 0.0 } else { v };
    let s = format!("{v:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// A number Python reads as a float (`12.0`).
fn float(v: f32) -> String {
    let s = coord(v);
    if s.contains('.') { s } else { format!("{s}.0") }
}

/// A name FreeCAD takes for an object: letters, digits and `_`, not starting with a
/// digit.
fn object_name(s: &str) -> String {
    let t: String = s.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    let t = t.trim_matches('_');
    match t.chars().next() {
        None => "Item".into(),
        Some(c) if c.is_ascii_digit() => format!("N{t}"),
        _ => t.to_string(),
    }
}

/// View coordinates of an enclosure (x right, y up, z to the door) in FreeCAD's: z up.
fn z_up(v: [f32; 3]) -> [f32; 3] {
    [v[0], -v[2], v[1]]
}

/// The directions in FreeCAD in which a device moves when its x, y and z in the
/// layout grow: they depend on the face it is mounted on.
fn axes(e: &Enclosure, it: &PanelItem) -> [[f32; 3]; 3] {
    let center = |it: &PanelItem| {
        let (a, b) = panel::item_box(e, it);
        [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0]
    };
    let c0 = center(it);
    [0, 1, 2].map(|k| {
        let mut moved = it.clone();
        match k {
            0 => moved.x += 1.0,
            1 => moved.y += 1.0,
            _ => moved.z += 1.0,
        }
        let c = center(&moved);
        // Enclosure coordinates have y downwards.
        z_up([c[0] - c0[0], c0[1] - c[1], c[2] - c0[2]]).map(f32::round)
    })
}

fn vector(v: [f32; 3]) -> String {
    format!("({}, {}, {})", float(v[0]), float(v[1]), float(v[2]))
}

/// One object of the macro: the solid with its points listed once.
fn object(out: &mut String, solid: &Solid, name: &str, properties: &[(&str, String)]) {
    let mut index: HashMap<[i64; 3], usize> = HashMap::new();
    let mut points = String::new();
    let mut facets = String::new();
    for t in &solid.triangles {
        let ids = t.map(|v| {
            let q = z_up(v);
            let next = index.len();
            *index.entry(q.map(|c| (c * 1000.0).round() as i64)).or_insert_with(|| {
                let _ = write!(points, "({},{},{}),", coord(q[0]), coord(q[1]), coord(q[2]));
                next
            })
        });
        if ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2] {
            let _ = write!(facets, "({},{},{}),", ids[0], ids[1], ids[2]);
        }
    }
    if facets.is_empty() {
        return;
    }
    let [r, g, b] = solid.color.map(|c| format!("{:.3}", c as f32 / 255.0));
    let props: Vec<String> = properties.iter().map(|(k, v)| format!("({}, {v})", text(k))).collect();
    let _ = writeln!(
        out,
        "_add(group, {}, {}, ({r}, {g}, {b}), {}, [{points}], [{facets}], [{}])",
        text(&object_name(name)),
        text(name),
        if solid.transparent { 75 } else { 0 },
        props.join(", ")
    );
}

const HEADER: &str = r#"
# Run it in FreeCAD (Macro > Macros..., or open the file and press Ctrl+F6). It makes a
# document with a part container per enclosure and an object per item, named by the
# device tags. To take changed positions back: move the devices, save the document
# (.FCStd) and read that file in OpenECraft with the connector FreeCAD.

import FreeCAD as App
import Mesh
import Part

SOLIDS = True      # False: triangle meshes only (faster with large part models)
MAX_FACETS = 4000  # objects with more triangles stay meshes


def _solid(shell):
    try:
        solid = Part.makeSolid(shell)
        if solid.Volume < 0:
            solid.reverse()
        return solid if solid.isValid() else shell
    except Exception:
        return shell


def _shape(points, facets):
    mesh = Mesh.Mesh([App.Vector(*points[i]) for facet in facets for i in facet])
    if not SOLIDS or len(facets) > MAX_FACETS:
        return None, mesh
    try:
        shape = Part.Shape()
        shape.makeShapeFromMesh(mesh.Topology, 0.05)
        shape = shape.removeSplitter()
        # A part model is made of several closed bodies: each becomes a solid.
        shells = shape.Shells
        if shells and sum(len(shell.Faces) for shell in shells) == len(shape.Faces):
            solids = [_solid(shell) for shell in shells]
            shape = solids[0] if len(solids) == 1 else Part.makeCompound(solids)
        return shape, mesh
    except Exception:
        return None, mesh


def _add(group, name, label, color, transparency, points, facets, properties):
    shape, mesh = _shape(points, facets)
    if shape is None:
        obj = doc.addObject("Mesh::Feature", name)
        obj.Mesh = mesh
    else:
        obj = doc.addObject("Part::Feature", name)
        obj.Shape = shape
    obj.Label = label
    for key, value in properties:
        if isinstance(value, tuple):
            obj.addProperty("App::PropertyVector", key, "OpenECraft", "")
            value = App.Vector(*value)
        elif isinstance(value, float):
            obj.addProperty("App::PropertyFloat", key, "OpenECraft", "")
        else:
            obj.addProperty("App::PropertyString", key, "OpenECraft", "")
        setattr(obj, key, value)
    group.addObject(obj)
    if App.GuiUp:
        obj.ViewObject.ShapeColor = color
        obj.ViewObject.Transparency = transparency
    return obj


def _enclosure(name, label, x, y, z):
    part = doc.addObject("App::Part", name)
    part.Label = label
    part.Placement = App.Placement(App.Vector(x, y, z), App.Rotation())
    return part

"#;

const FOOTER: &str = r#"
doc.recompute()
if App.GuiUp:
    import FreeCADGui as Gui
    Gui.activeDocument().activeView().viewIsometric()
    Gui.SendMsgToActiveView("ViewFit")
"#;

/// The panel layout as a FreeCAD macro (`.FCMacro`): millimetres, z up, the enclosures
/// side by side as in the 3D view.
pub fn macro_text(prj: &Project) -> String {
    let name = if prj.info.name.trim().is_empty() { "Panel layout" } else { prj.info.name.trim() };
    let mut out = format!("# -*- coding: utf-8 -*-\n# Panel layout {} of OpenECraft {} for FreeCAD.\n", text(name), env!("CARGO_PKG_VERSION"));
    out.push_str(HEADER);
    let _ = writeln!(out, "doc = App.newDocument(\"OpenECraft\")\ndoc.Label = {}\n", text(name));
    for (n, ((es, items), e)) in export3d::scene_items(prj).iter().zip(&prj.layout.enclosures).enumerate() {
        let o = z_up(es.offset);
        let _ = writeln!(out, "group = _enclosure({}, {}, {}, {}, {})", text(&format!("Enclosure{}", n + 1)), text(&es.name), float(o[0]), float(o[1]), float(o[2]));
        for (solid, id) in es.solids.iter().zip(items) {
            let item = id.and_then(|id| e.items.iter().find(|i| i.id == id));
            let mut props: Vec<(&str, String)> = Vec::new();
            // A device placed whole can be moved in FreeCAD and read back by its tag.
            if let Some(it) = item.filter(|i| i.kind == ItemKind::Device && !i.tag.is_empty() && i.terminal.is_empty()) {
                let (a, b) = panel::item_box(e, it);
                let center = z_up([(a[0] + b[0]) / 2.0, e.height - (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0]);
                let [ax, ay, az] = axes(e, it);
                props = vec![
                    ("OEC_Tag", text(&it.tag)),
                    ("OEC_Part", text(&it.part)),
                    ("OEC_Enclosure", text(&e.name)),
                    ("OEC_Face", text(it.face.key())),
                    ("OEC_X", float(it.x)),
                    ("OEC_Y", float(it.y)),
                    ("OEC_Z", float(it.z)),
                    ("OEC_Center", vector(center)),
                    ("OEC_AxisX", vector(ax)),
                    ("OEC_AxisY", vector(ay)),
                    ("OEC_AxisZ", vector(az)),
                ];
            } else if let Some(it) = item.filter(|i| !i.part.is_empty()) {
                props.push(("OEC_Part", text(&it.part)));
            }
            object(&mut out, solid, &solid.name, &props);
        }
        out.push('\n');
    }
    out.push_str(FOOTER);
    out
}

/// The value of a property of an object in `Document.xml`: its first child element.
fn property<'a>(props: &'a Element, name: &str) -> Option<&'a Element> {
    props.children("Property").find(|p| p.attr("name") == Some(name)).and_then(|p| p.children.first())
}

fn attr(e: &Element, name: &str) -> Option<f32> {
    e.attr(name).and_then(|v| v.trim().parse().ok()).filter(|v: &f32| v.is_finite())
}

fn vector_of(props: &Element, name: &str) -> Option<[f32; 3]> {
    let v = property(props, name)?;
    Some([attr(v, "valueX")?, attr(v, "valueY")?, attr(v, "valueZ")?])
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// `v` turned by the quaternion (x, y, z, w).
fn turned(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let u = [q[0], q[1], q[2]];
    let (a, b) = (cross(u, v), cross(u, cross(u, v)));
    [0, 1, 2].map(|k| v[k] + 2.0 * q[3] * a[k] + 2.0 * b[k])
}

/// The devices of a macro written by [`macro_text`] with the places they were exported
/// at, as placement records; `None` for another text.
pub fn macro_placements(text: &str) -> Option<String> {
    if !text.starts_with("# -*- coding: utf-8 -*-\n# Panel layout ") {
        return None;
    }
    // What is written after a property name: a text or a number.
    let after = |line: &'_ str, key: &str| line.find(&format!("(\"{key}\", ")).map(|at| line[at + key.len() + 5..].to_string());
    let number = |line: &str, key: &str| -> Option<f32> {
        let rest = after(line, key)?;
        rest[..rest.find(')')?].trim().parse().ok()
    };
    let mut out = String::from("tag;x;y;z\n");
    for line in text.lines().filter(|l| l.starts_with("_add(")) {
        let Some(tag) = after(line, "OEC_Tag").and_then(|rest| serde_json::Deserializer::from_str(&rest).into_iter::<String>().next()?.ok()) else { continue };
        let (Some(x), Some(y), Some(z)) = (number(line, "OEC_X"), number(line, "OEC_Y"), number(line, "OEC_Z")) else { continue };
        let _ = writeln!(out, "{};{};{};{}", openecraft_parts::csv_escape(&tag), coord(x), coord(y), coord(z));
    }
    Some(out)
}

/// Is this a FreeCAD document (a ZIP archive with a `Document.xml`)?
pub fn is_document(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK") && bytes.windows(12).any(|w| w == b"Document.xml")
}

/// The places of the devices in a FreeCAD document made by the macro and saved again
/// (`.FCStd`), as the placement records of the integration layer (`tag;x;y;z`, CSV):
/// where a device was moved, its place in the layout moved along.
pub fn placements(bytes: &[u8]) -> Result<String, String> {
    let not_freecad = || tr("Not a FreeCAD document (.FCStd)").to_string();
    let files = crate::zip::read_zip(bytes).map_err(|_| not_freecad())?;
    let document = files.iter().find(|(name, _)| name == "Document.xml").ok_or_else(not_freecad)?;
    let root = xml::parse(&String::from_utf8_lossy(&document.1))?;
    let mut out = String::from("tag;x;y;z\n");
    let mut found = false;
    for props in root.child("ObjectData").into_iter().flat_map(|d| d.children("Object")).filter_map(|o| o.child("Properties")) {
        let Some(tag) = property(props, "OEC_Tag").and_then(|s| s.attr("value")).filter(|t| !t.is_empty()) else { continue };
        let number = |name: &str| property(props, name).and_then(|f| attr(f, "value"));
        let (Some(x), Some(y), Some(z)) = (number("OEC_X"), number("OEC_Y"), number("OEC_Z")) else { continue };
        // How far the middle of the device went: the object was made in place, so its
        // placement is the move (a turn about the origin moves the middle too).
        let mut delta = [0.0; 3];
        if let Some(pl) = property(props, "Placement") {
            let base = [attr(pl, "Px").unwrap_or(0.0), attr(pl, "Py").unwrap_or(0.0), attr(pl, "Pz").unwrap_or(0.0)];
            let q = [attr(pl, "Q0").unwrap_or(0.0), attr(pl, "Q1").unwrap_or(0.0), attr(pl, "Q2").unwrap_or(0.0), attr(pl, "Q3").unwrap_or(1.0)];
            let center = vector_of(props, "OEC_Center").unwrap_or_default();
            let moved = turned(q, center);
            delta = [0, 1, 2].map(|k| moved[k] + base[k] - center[k]);
        }
        let along = |axis: &str, old: f32| old + vector_of(props, axis).map_or(0.0, |a| dot(delta, a));
        let _ = writeln!(out, "{};{};{};{}", openecraft_parts::csv_escape(tag), coord(along("OEC_AxisX", x)), coord(along("OEC_AxisY", y)), coord(along("OEC_AxisZ", z)));
        found = true;
    }
    if !found {
        return Err(tr("The FreeCAD document has no devices of OpenECraft (objects with the property OEC_Tag)").into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
