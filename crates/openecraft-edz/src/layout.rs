//! EPLAN Pro Panel 3D macros (window macros of the layout space, `MacroFuncType 10`):
//! the placed bodies with their meshes, the mounting surfaces and the restricted areas.
//!
//! Every object is placed in its parent (`R2481`) by an origin (`A2482`), an x axis
//! (`A2483`) and a y axis (`A2484`); its z axis is x × y. `O180` is a body, its mesh the
//! stream `62491` of its group (`R9`); `O170` a mounting surface of a body (kind
//! `A2486`, size `A2601`); `O175` a restricted area on a surface (size `A2606`, layer
//! `A2488`: 571 placing, 572 cut-out). A mesh (`Mesh3dX`, `#,1,1{20,2,…}`) lists its
//! vertices (`x/y/z`) and then its faces: id, colour, material and triangles of
//! (vertex, point) index pairs, followed by neighbours and outline edges.

use openecraft_parts::xml::{self, Element};
use openecraft_parts::{Mesh3d, MAX_TRIANGLES};
use std::collections::HashMap;

type V3 = [f64; 3];

/// A body of the macro.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Body {
    /// Part number (`RIT.398818`), empty for bodies of the part around them.
    pub part: String,
    /// Macro the body was taken from (`Rittal\Items\398818_3D.ema`).
    pub source: String,
    /// Placed in the macro itself (the housing of an enclosure), not on a mounting
    /// surface of another body.
    pub main: bool,
    /// The mesh in the frame of the macro, mm.
    pub mesh: Mesh3d,
}

/// A mounting surface, in the frame of the macro.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Surface {
    /// EPLAN kind of the surface (`10002` front of the mounting plate, …).
    pub kind: u32,
    /// Its name (`Front inside`), may be empty.
    pub name: String,
    /// Corner at the origin, the directions of its sides and their lengths (mm).
    pub origin: [f32; 3],
    pub x: [f32; 3],
    pub y: [f32; 3],
    pub size: [f32; 2],
}

/// What a restricted area keeps free.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AreaKind {
    /// No devices are placed there.
    Placing,
    /// No cutouts are made there.
    Cutout,
}

/// A restricted area on a mounting surface, in the frame of the macro.
#[derive(Clone, Debug, PartialEq)]
pub struct Area {
    pub kind: AreaKind,
    /// Kind of the mounting surface it lies on.
    pub surface: u32,
    pub origin: [f32; 3],
    pub x: [f32; 3],
    pub y: [f32; 3],
    pub size: [f32; 2],
}

/// The 3D content of a layout space macro.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    /// Part number of the macro's main part (`RIT.398580`), may be empty.
    pub part: String,
    pub bodies: Vec<Body>,
    pub surfaces: Vec<Surface>,
    pub areas: Vec<Area>,
    /// Bodies whose mesh is in a form not read (another kind of stream).
    pub unread: usize,
}

impl Layout {
    /// All bodies in one mesh; bodies beyond [`MAX_TRIANGLES`] are left out.
    pub fn mesh(&self) -> Mesh3d {
        let mut out = Mesh3d::default();
        for body in &self.bodies {
            if out.triangles.len() + body.mesh.triangles.len() > MAX_TRIANGLES {
                continue;
            }
            let base = out.vertices.len() as u32;
            out.vertices.extend(&body.mesh.vertices);
            out.triangles.extend(body.mesh.triangles.iter().map(|t| t.map(|k| k + base)));
        }
        out
    }

    /// Bounds of the bodies placed in the macro itself (the housing of an enclosure
    /// without the door and the parts mounted on it).
    pub fn main_bounds(&self) -> Option<([f32; 3], [f32; 3])> {
        let mut points = self.bodies.iter().filter(|b| b.main).flat_map(|b| &b.mesh.vertices).peekable();
        points.peek()?;
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for v in points {
            for k in 0..3 {
                min[k] = min[k].min(v[k]);
                max[k] = max[k].max(v[k]);
            }
        }
        Some((min, max))
    }
}

fn v3(s: &str) -> Option<V3> {
    let mut it = s.split('/').map(|n| n.trim().parse::<f64>().ok());
    let v = [it.next()??, it.next()??, it.next()??];
    (it.next().is_none() && v.iter().all(|c| c.is_finite())).then_some(v)
}

fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn f32s(v: V3) -> [f32; 3] {
    v.map(|c| c as f32)
}

/// Placement of an object in its parent.
#[derive(Clone, Copy)]
struct Frame {
    origin: V3,
    x: V3,
    y: V3,
    z: V3,
}

impl Frame {
    fn of(el: &Element) -> Option<Frame> {
        let x = v3(el.attribute("A2483")?)?;
        let y = v3(el.attribute("A2484")?)?;
        Some(Frame { origin: v3(el.attribute("A2482")?)?, x, y, z: cross(x, y) })
    }

    fn point(&self, p: V3) -> V3 {
        std::array::from_fn(|k| self.origin[k] + p[0] * self.x[k] + p[1] * self.y[k] + p[2] * self.z[k])
    }

    fn direction(&self, d: V3) -> V3 {
        std::array::from_fn(|k| d[0] * self.x[k] + d[1] * self.y[k] + d[2] * self.z[k])
    }
}

/// The objects of the macro by id, and the frames that place a point of an object in
/// the macro (its own and those of its parents, innermost first).
struct Objects<'a> {
    by_id: HashMap<&'a str, &'a Element>,
}

impl<'a> Objects<'a> {
    fn frames(&self, el: &Element) -> Vec<Frame> {
        let mut out = Vec::new();
        let mut at = Some(el);
        // Placements nest a few levels; a cycle in a damaged file ends the walk.
        while let Some(el) = at.filter(|_| out.len() < 32) {
            let Some(frame) = Frame::of(el) else { break };
            out.push(frame);
            at = el.attribute("R2481").and_then(|id| self.by_id.get(id).copied());
        }
        out
    }
}

fn place(frames: &[Frame], p: V3) -> V3 {
    frames.iter().fold(p, |p, f| f.point(p))
}

fn turn(frames: &[Frame], d: V3) -> V3 {
    frames.iter().fold(d, |d, f| f.direction(d))
}

fn size2(s: &str) -> Option<[f32; 2]> {
    let (w, h) = s.split_once('/')?;
    Some([w.trim().parse().ok()?, h.trim().parse().ok()?])
}

fn property<'a>(el: &'a Element, name: &str) -> Option<&'a str> {
    el.children.iter().find_map(|c| c.attribute(name)).filter(|v| !v.is_empty())
}

/// Text of a multi-language EPLAN value (`??_??@Front inside;`, `en_US@…;de_DE@…;`).
fn text(value: &str) -> String {
    let parts: Vec<&str> = value.split(';').filter(|s| !s.is_empty()).collect();
    let pick = parts.iter().find(|s| s.starts_with("en_US@")).or_else(|| parts.first()).copied().unwrap_or_default();
    pick.split_once('@').map_or(pick, |(_, t)| t).to_string()
}

/// Reads a mesh stream (`Mesh3dX`); `None` for other objects (boxes of restricted areas).
pub fn mesh(stream: &str) -> Option<Mesh3d> {
    let body = stream.trim().strip_prefix("#,1,1{20,")?;
    // Class 20, version, flags, object id, then two empty sub-objects.
    let body = &body[body.find("{}{}")? + 4..];
    let body = body.strip_suffix('}')?;
    let mut tokens = body.split(',');
    let mut next = || tokens.next().map(str::trim);
    let int = |t: Option<&str>| t.and_then(|t| t.parse::<i64>().ok());
    let _ids = int(next())?;
    let count = usize::try_from(int(next())?).ok()?;
    let mut vertices = Vec::with_capacity(count.min(1 << 20));
    for _ in 0..count {
        vertices.push(f32s(v3(next()?)?));
    }
    let _textures = int(next())?;
    let faces = int(next())?;
    let mut triangles = Vec::new();
    for _ in 0..faces {
        // Kind, id, colour, material.
        for _ in 0..4 {
            next()?;
        }
        let n = int(next())?;
        for _ in 0..n {
            let mut corner = [0u32; 3];
            for c in &mut corner {
                *c = u32::try_from(int(next())?).ok().filter(|&k| (k as usize) < count)?;
                next()?;
            }
            triangles.push(corner);
        }
        // Neighbours: six per triangle.
        let n = int(next())?;
        for _ in 0..6 * n {
            next()?;
        }
        // Outline edges: id, number of points, points.
        let edges = int(next())?;
        for _ in 0..edges {
            next()?;
            let points = int(next())?;
            for _ in 0..points {
                next()?;
            }
        }
    }
    (!triangles.is_empty()).then_some(Mesh3d { vertices, triangles, colors: Vec::new() })
}

fn stream(group: &Element) -> Option<&str> {
    group.all("Stream").find(|s| s.attribute("Id") == Some("62491")).map(|s| s.text.as_str())
}

/// Reads the 3D content of a layout space macro; `None` for other macros.
pub fn decode(text_of_file: &str) -> Option<Layout> {
    let root = xml::parse(text_of_file).ok()?;
    let variant = root.all("MacroVariant").find(|v| v.attribute("MacroFuncType") == Some("10"))?;
    let objects = Objects { by_id: variant.children.iter().filter_map(|o| Some((o.attribute("A1")?, o))).collect() };
    let mut layout = Layout::default();
    if let Some(main) = variant.all("O179").next() {
        layout.part = property(main, "P20100_1").or_else(|| property(main, "P22001")).unwrap_or_default().to_string();
    }
    for el in &variant.children {
        match el.name.as_str() {
            "O180" => {
                let Some(stream) = el.attribute("R9").and_then(|g| objects.by_id.get(g)).and_then(|g| stream(g)) else { continue };
                let Some(mesh) = mesh(stream) else {
                    layout.unread += 1;
                    continue;
                };
                let frames = objects.frames(el);
                let vertices = mesh.vertices.iter().map(|v| f32s(place(&frames, v.map(f64::from)))).collect();
                let parent = el.attribute("R2481").and_then(|id| objects.by_id.get(id));
                layout.bodies.push(Body {
                    part: property(el, "P22001").unwrap_or_default().to_string(),
                    source: property(el, "P20468").unwrap_or_default().to_string(),
                    main: parent.is_none_or(|p| p.name == "O179"),
                    mesh: Mesh3d { vertices, ..mesh },
                });
            }
            "O170" => {
                let (Some(size), Some(kind)) = (el.attribute("A2601").and_then(size2), el.attribute("A2486").and_then(|k| k.parse().ok())) else { continue };
                let frames = objects.frames(el);
                layout.surfaces.push(Surface {
                    kind,
                    name: property(el, "P20045").map(text).unwrap_or_default(),
                    origin: f32s(place(&frames, [0.0; 3])),
                    x: f32s(turn(&frames, [1.0, 0.0, 0.0])),
                    y: f32s(turn(&frames, [0.0, 1.0, 0.0])),
                    size,
                });
            }
            "O175" => {
                let kind = match el.attribute("A2488") {
                    Some("571") => AreaKind::Placing,
                    Some("572") => AreaKind::Cutout,
                    _ => continue,
                };
                let Some(size) = el.attribute("A2606").and_then(size2) else { continue };
                let surface = el.attribute("R2481").and_then(|s| objects.by_id.get(s)).and_then(|s| s.attribute("A2486")).and_then(|k| k.parse().ok()).unwrap_or_default();
                let frames = objects.frames(el);
                layout.areas.push(Area {
                    kind,
                    surface,
                    origin: f32s(place(&frames, [0.0; 3])),
                    x: f32s(turn(&frames, [1.0, 0.0, 0.0])),
                    y: f32s(turn(&frames, [0.0, 1.0, 0.0])),
                    size,
                });
            }
            _ => {}
        }
    }
    (!layout.bodies.is_empty() || !layout.surfaces.is_empty() || layout.unread > 0).then_some(layout)
}

/// The 3D model of a part from its 3D macro, at the origin (devices: x across the
/// front, y up, z towards the front, as EPLAN's device macros are); `Err` for a
/// macro whose bodies could not be read, `Ok(None)` for other macros.
pub fn model(text_of_file: &str) -> Result<Option<Mesh3d>, String> {
    let Some(layout) = decode(text_of_file) else { return Ok(None) };
    if layout.bodies.is_empty() {
        return if layout.unread > 0 { Err(unread(layout.unread)) } else { Ok(None) };
    }
    let mut mesh = layout.mesh();
    mesh.normalize();
    Ok(Some(mesh))
}

/// Diagnostic of a macro with bodies in a form not read.
pub fn unread(n: usize) -> String {
    format!("{n} bodies of the 3D macro have no mesh this version reads")
}

#[cfg(test)]
mod tests;
