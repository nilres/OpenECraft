//! Graphical models of parts: a 2D front view (for mounting layouts) and a 3D mesh
//! (for the 3D panel view), with importers for SVG, STL and OBJ.
//!
//! Units are millimetres.
//! - 2D: origin at the top left corner of the front view, x to the right, y down.
//! - 3D: x to the right, y up, z towards the viewer (out of the mounting plate).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Largest mesh accepted by the importers.
pub const MAX_TRIANGLES: usize = 200_000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Prim2d {
    Line { a: [f32; 2], b: [f32; 2] },
    Rect { min: [f32; 2], max: [f32; 2], #[serde(default)] fill: bool },
    Circle { c: [f32; 2], r: f32, #[serde(default)] fill: bool },
    Polyline { pts: Vec<[f32; 2]>, #[serde(default)] closed: bool, #[serde(default)] fill: bool },
    Text { pos: [f32; 2], text: String, size: f32 },
}

/// Front view of a part.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Model2d {
    pub width: f32,
    pub height: f32,
    pub prims: Vec<Prim2d>,
}

/// Triangle mesh of a part.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mesh3d {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    /// Colour (red, green, blue) of each triangle; triangles without one (all of an
    /// imported mesh) have the colour the view gives the device.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub colors: Vec<[u8; 3]>,
}

/// Colour of the triangles of a painted mesh that were not painted.
pub const UNPAINTED: [u8; 3] = [200, 204, 210];

/// Colour of the door of an enclosure's model (with what is mounted on it: hinge
/// leaves, lock, seal): the panel layout opens the triangles of this colour with the
/// door.
pub const DOOR_PAINT: [u8; 3] = [190, 194, 200];

impl Mesh3d {
    /// Axis aligned bounding box (min, max).
    pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for v in &self.vertices {
            for k in 0..3 {
                min[k] = min[k].min(v[k]);
                max[k] = max[k].max(v[k]);
            }
        }
        if self.vertices.is_empty() {
            ([0.0; 3], [0.0; 3])
        } else {
            (min, max)
        }
    }

    /// Width, height, depth.
    pub fn size(&self) -> [f32; 3] {
        let (min, max) = self.bounds();
        [max[0] - min[0], max[1] - min[1], max[2] - min[2]]
    }

    /// Moves the mesh so that its bounding box starts at the origin.
    pub fn normalize(&mut self) {
        let (min, _) = self.bounds();
        for v in &mut self.vertices {
            for k in 0..3 {
                v[k] -= min[k];
            }
        }
    }

    pub fn scale(&mut self, f: f32) {
        for v in &mut self.vertices {
            for c in v.iter_mut() {
                *c *= f;
            }
        }
        self.normalize();
    }

    /// Rotates by 90° around an axis (0 = x, 1 = y, 2 = z), right handed.
    pub fn rotate90(&mut self, axis: usize) {
        for v in &mut self.vertices {
            let [x, y, z] = *v;
            *v = match axis {
                0 => [x, -z, y],
                1 => [z, y, -x],
                _ => [-y, x, z],
            };
        }
        self.normalize();
    }

    /// Colour of triangle `i`, if the mesh is painted.
    pub fn color(&self, i: usize) -> Option<[u8; 3]> {
        self.colors.get(i).copied()
    }

    /// Paints the triangles from `from` on with `c` (the ones before without a colour
    /// get [`UNPAINTED`]).
    pub fn paint(&mut self, from: usize, c: [u8; 3]) {
        let from = from.min(self.triangles.len());
        if self.colors.len() < from {
            self.colors.resize(from, UNPAINTED);
        }
        self.colors.truncate(from);
        self.colors.resize(self.triangles.len(), c);
    }

    /// A box painted with `c`.
    pub fn add_painted_box(&mut self, min: [f32; 3], max: [f32; 3], c: [u8; 3]) {
        let from = self.triangles.len();
        self.add_box(min, max);
        self.paint(from, c);
    }

    /// A closed frustum along `axis` (0 = x, 1 = y, 2 = z) from the centre `base` over
    /// `len`, with the radius `r0` at the base and `r1` at the other end, `n` sides,
    /// painted with `c` (outward facing, counter clockwise triangles).
    #[allow(clippy::too_many_arguments)]
    pub fn add_frustum(&mut self, base: [f32; 3], axis: usize, r0: f32, r1: f32, len: f32, n: usize, c: [u8; 3]) {
        let from = self.triangles.len();
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        let first = self.vertices.len() as u32;
        let n = n.max(3);
        for (r, along) in [(r0, 0.0), (r1, len)] {
            for k in 0..n {
                let t = k as f32 / n as f32 * std::f32::consts::TAU;
                let mut q = base;
                q[u] += r * t.cos();
                q[v] += r * t.sin();
                q[axis] += along;
                self.vertices.push(q);
            }
        }
        let mut c1 = base;
        c1[axis] += len;
        self.vertices.push(base);
        self.vertices.push(c1);
        let (bottom, top, n32) = (first, first + n as u32, n as u32);
        let (cb, ct) = (first + 2 * n32, first + 2 * n32 + 1);
        for k in 0..n32 {
            let k1 = (k + 1) % n32;
            self.triangles.push([bottom + k, bottom + k1, top + k1]);
            self.triangles.push([bottom + k, top + k1, top + k]);
            if r0 > 0.0 {
                self.triangles.push([cb, bottom + k1, bottom + k]);
            }
            if r1 > 0.0 {
                self.triangles.push([ct, top + k, top + k1]);
            }
        }
        self.paint(from, c);
    }

    /// A closed cylinder (see [`Mesh3d::add_frustum`]).
    #[allow(clippy::too_many_arguments)]
    pub fn add_cylinder(&mut self, base: [f32; 3], axis: usize, r: f32, len: f32, n: usize, c: [u8; 3]) {
        self.add_frustum(base, axis, r, r, len, n, c);
    }

    /// Adds an axis aligned box (outward facing, counter clockwise triangles).
    pub fn add_box(&mut self, min: [f32; 3], max: [f32; 3]) {
        let base = self.vertices.len() as u32;
        for i in 0..8u32 {
            self.vertices.push([
                if i & 4 == 0 { min[0] } else { max[0] },
                if i & 2 == 0 { min[1] } else { max[1] },
                if i & 1 == 0 { min[2] } else { max[2] },
            ]);
        }
        // Faces as quads of corner indices (bits: x y z), counter clockwise from outside.
        let quads = [[0, 1, 3, 2], [4, 6, 7, 5], [0, 4, 5, 1], [2, 3, 7, 6], [0, 2, 6, 4], [1, 5, 7, 3]];
        for q in quads {
            self.triangles.push([base + q[0], base + q[1], base + q[2]]);
            self.triangles.push([base + q[0], base + q[2], base + q[3]]);
        }
    }
}

/// Collects triangles and merges identical vertices (STL / OBJ / STEP import).
pub struct MeshBuilder {
    mesh: Mesh3d,
    index: HashMap<(i64, i64, i64), u32>,
}

impl Default for MeshBuilder {
    fn default() -> Self {
        MeshBuilder::new()
    }
}

impl MeshBuilder {
    pub fn new() -> MeshBuilder {
        MeshBuilder { mesh: Mesh3d::default(), index: HashMap::new() }
    }

    fn vertex(&mut self, v: [f32; 3]) -> u32 {
        let key = ((v[0] * 1000.0).round() as i64, (v[1] * 1000.0).round() as i64, (v[2] * 1000.0).round() as i64);
        if let Some(&i) = self.index.get(&key) {
            return i;
        }
        let i = self.mesh.vertices.len() as u32;
        self.mesh.vertices.push(v);
        self.index.insert(key, i);
        i
    }

    /// Adds a triangle; an error once the mesh has [`MAX_TRIANGLES`] triangles.
    pub fn triangle(&mut self, a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Result<(), String> {
        if self.mesh.triangles.len() >= MAX_TRIANGLES {
            return Err(format!("model has more than {MAX_TRIANGLES} triangles"));
        }
        let t = [self.vertex(a), self.vertex(b), self.vertex(c)];
        if t[0] != t[1] && t[1] != t[2] && t[0] != t[2] {
            self.mesh.triangles.push(t);
        }
        Ok(())
    }

    /// The mesh, moved to the origin; an error without triangles.
    pub fn finish(mut self) -> Result<Mesh3d, String> {
        if self.mesh.triangles.is_empty() {
            return Err("no triangles found".into());
        }
        if self.mesh.vertices.iter().flatten().any(|c| !c.is_finite()) {
            return Err("invalid coordinates".into());
        }
        self.mesh.normalize();
        Ok(self.mesh)
    }
}

/// Reads an STL file (ASCII or binary).
pub fn parse_stl(bytes: &[u8]) -> Result<Mesh3d, String> {
    let is_ascii = bytes.starts_with(b"solid") && {
        let head = String::from_utf8_lossy(&bytes[..bytes.len().min(1024)]).to_lowercase();
        head.contains("facet") || head.contains("endsolid")
    };
    let mut b = MeshBuilder::new();
    if is_ascii {
        let text = String::from_utf8_lossy(bytes);
        let mut verts: Vec<[f32; 3]> = Vec::new();
        for line in text.lines() {
            let mut it = line.split_whitespace();
            match it.next() {
                Some("vertex") => {
                    let c: Vec<f32> = it.filter_map(|x| x.parse().ok()).collect();
                    if c.len() != 3 {
                        return Err(format!("invalid vertex line: {line}"));
                    }
                    verts.push([c[0], c[1], c[2]]);
                }
                Some("endloop") => {
                    // Facets are usually triangles; larger loops are fanned.
                    for i in 1..verts.len().saturating_sub(1) {
                        b.triangle(verts[0], verts[i], verts[i + 1])?;
                    }
                    verts.clear();
                }
                _ => {}
            }
        }
    } else {
        if bytes.len() < 84 {
            return Err("file too short for a binary STL".into());
        }
        let n = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]) as usize;
        if bytes.len() < 84 + n * 50 {
            return Err("binary STL is truncated".into());
        }
        let f = |o: usize| f32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
        for t in 0..n {
            let o = 84 + t * 50 + 12; // skip the normal
            let v = |k: usize| [f(o + k * 12), f(o + k * 12 + 4), f(o + k * 12 + 8)];
            b.triangle(v(0), v(1), v(2))?;
        }
    }
    b.finish()
}

/// Reads a Wavefront OBJ file (vertices and faces; polygons are fanned).
pub fn parse_obj(text: &str) -> Result<Mesh3d, String> {
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut b = MeshBuilder::new();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("v") => {
                let c: Vec<f32> = it.take(3).filter_map(|x| x.parse().ok()).collect();
                if c.len() != 3 {
                    return Err(format!("invalid vertex line: {line}"));
                }
                verts.push([c[0], c[1], c[2]]);
            }
            Some("f") => {
                let idx: Result<Vec<usize>, String> = it
                    .map(|tok| {
                        let i: i64 = tok.split('/').next().unwrap_or("").parse().map_err(|_| format!("invalid face: {line}"))?;
                        let i = if i < 0 { verts.len() as i64 + i } else { i - 1 };
                        if i < 0 || i as usize >= verts.len() {
                            return Err(format!("face index out of range: {line}"));
                        }
                        Ok(i as usize)
                    })
                    .collect();
                let idx = idx?;
                for i in 1..idx.len().saturating_sub(1) {
                    b.triangle(verts[idx[0]], verts[idx[i]], verts[idx[i + 1]])?;
                }
            }
            _ => {}
        }
    }
    b.finish()
}

/// Reads a 3D model by file name extension (`.stl`, `.obj` or `.ifc`).
pub fn parse_mesh(name: &str, bytes: &[u8]) -> Result<Mesh3d, String> {
    let lower = name.to_lowercase();
    if lower.ends_with(".obj") {
        parse_obj(&String::from_utf8_lossy(bytes))
    } else if lower.ends_with(".stl") {
        parse_stl(bytes)
    } else if lower.ends_with(".ifc") {
        crate::ifc::parse_ifc(bytes)
    } else {
        Err("unsupported 3D format (use STL, OBJ or IFC)".into())
    }
}

// ---------------------------------------------------------------------- SVG

/// Attributes of an SVG element.
fn attributes(tag: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let chars: Vec<char> = tag.chars().collect();
    let mut i = 0;
    // Skip the element name
    while i < chars.len() && !chars[i].is_whitespace() {
        i += 1;
    }
    while i < chars.len() {
        while i < chars.len() && (chars[i].is_whitespace() || chars[i] == '/') {
            i += 1;
        }
        let start = i;
        while i < chars.len() && chars[i] != '=' && !chars[i].is_whitespace() {
            i += 1;
        }
        let name: String = chars[start..i].iter().collect();
        while i < chars.len() && (chars[i].is_whitespace() || chars[i] == '=') {
            i += 1;
        }
        if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
            let q = chars[i];
            i += 1;
            let vs = i;
            while i < chars.len() && chars[i] != q {
                i += 1;
            }
            map.insert(name, chars[vs..i].iter().collect());
            i += 1;
        } else if !name.is_empty() {
            map.insert(name, String::new());
        }
    }
    map
}

fn numbers(s: &str) -> Vec<f32> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let flush = |cur: &mut String, out: &mut Vec<f32>| {
        if let Ok(v) = cur.parse() {
            out.push(v);
        }
        cur.clear();
    };
    for c in s.chars() {
        match c {
            '0'..='9' | '.' | 'e' | 'E' => {
                // A second '.' starts a new number ("1.5.5" = 1.5 .5)
                if c == '.' && cur.contains('.') && !cur.contains(['e', 'E']) {
                    flush(&mut cur, &mut out);
                }
                cur.push(c);
            }
            '-' | '+' if !(cur.ends_with('e') || cur.ends_with('E')) => {
                flush(&mut cur, &mut out);
                cur.push(c);
            }
            '-' | '+' => cur.push(c),
            _ => flush(&mut cur, &mut out),
        }
    }
    flush(&mut cur, &mut out);
    out
}

/// Length in mm ("10mm", "1cm", "2in", "30" = user units).
fn length_mm(s: &str) -> Option<(f32, bool)> {
    let s = s.trim();
    let (num, unit) = s.split_at(s.find(|c: char| c.is_alphabetic() || c == '%').unwrap_or(s.len()));
    let v: f32 = num.trim().parse().ok()?;
    match unit.trim() {
        "mm" => Some((v, true)),
        "cm" => Some((v * 10.0, true)),
        "in" => Some((v * 25.4, true)),
        "pt" => Some((v * 25.4 / 72.0, true)),
        "px" | "" => Some((v, false)),
        _ => None,
    }
}

/// Path data → polylines (curves are approximated by their end points).
fn path_polylines(d: &str) -> Vec<(Vec<[f32; 2]>, bool)> {
    let mut out = Vec::new();
    let mut cur: Vec<[f32; 2]> = Vec::new();
    let (mut x, mut y) = (0.0f32, 0.0f32);
    let mut start = [0.0f32, 0.0f32];
    // Split into commands with their numbers
    let mut cmds: Vec<(char, Vec<f32>)> = Vec::new();
    let mut buf = String::new();
    let mut cmd: Option<char> = None;
    for c in d.chars() {
        if c.is_ascii_alphabetic() && c != 'e' && c != 'E' {
            if let Some(k) = cmd {
                cmds.push((k, numbers(&buf)));
            }
            cmd = Some(c);
            buf.clear();
        } else {
            buf.push(c);
        }
    }
    if let Some(k) = cmd {
        cmds.push((k, numbers(&buf)));
    }
    let finish = |cur: &mut Vec<[f32; 2]>, closed: bool, out: &mut Vec<(Vec<[f32; 2]>, bool)>| {
        if cur.len() >= 2 {
            out.push((std::mem::take(cur), closed));
        } else {
            cur.clear();
        }
    };
    for (k, n) in cmds {
        let rel = k.is_ascii_lowercase();
        let (ox, oy) = if rel { (x, y) } else { (0.0, 0.0) };
        // number of values per segment
        let step = match k.to_ascii_uppercase() {
            'M' | 'L' | 'T' => 2,
            'H' | 'V' => 1,
            'S' | 'Q' => 4,
            'C' => 6,
            'A' => 7,
            _ => 0,
        };
        match k.to_ascii_uppercase() {
            'Z' => {
                x = start[0];
                y = start[1];
                finish(&mut cur, true, &mut out);
            }
            _ if step == 0 => {}
            _ => {
                for (seg, chunk) in n.chunks(step).enumerate() {
                    if chunk.len() < step {
                        break;
                    }
                    let (ox, oy) = if rel { (x, y) } else { (ox, oy) };
                    match k.to_ascii_uppercase() {
                        'H' => x = chunk[0] + ox,
                        'V' => y = chunk[0] + oy,
                        _ => {
                            x = chunk[step - 2] + ox;
                            y = chunk[step - 1] + oy;
                        }
                    }
                    if k.eq_ignore_ascii_case(&'M') && seg == 0 {
                        finish(&mut cur, false, &mut out);
                        start = [x, y];
                    }
                    cur.push([x, y]);
                }
            }
        }
    }
    finish(&mut cur, false, &mut out);
    out
}

/// Reads the drawing elements of an SVG file (`line`, `rect`, `circle`, `ellipse`,
/// `polyline`, `polygon`, `path`, `text`). Transforms are not applied and curves are
/// approximated by straight lines. Sizes in mm are taken from `width` / `height`,
/// otherwise one user unit is one millimetre.
pub fn parse_svg(text: &str) -> Result<Model2d, String> {
    let svg_start = text.find("<svg").ok_or("not an SVG file")?;
    let svg_end = text[svg_start..].find('>').ok_or("broken <svg> element")? + svg_start;
    let root = attributes(&text[svg_start + 1..svg_end]);
    let view: Vec<f32> = root.get("viewBox").map(|v| numbers(v)).unwrap_or_default();
    let w = root.get("width").and_then(|v| length_mm(v));
    let h = root.get("height").and_then(|v| length_mm(v));
    let (vx, vy, vw, vh) = if view.len() == 4 {
        (view[0], view[1], view[2], view[3])
    } else {
        (0.0, 0.0, w.map(|x| x.0).unwrap_or(0.0), h.map(|x| x.0).unwrap_or(0.0))
    };
    // Physical size if width/height carry units, else user units = mm
    let (mw, mh) = match (w, h) {
        (Some((w, true)), Some((h, true))) => (w, h),
        _ => (vw, vh),
    };
    let (sx, sy) = if vw > 0.0 && vh > 0.0 { (mw / vw, mh / vh) } else { (1.0, 1.0) };
    let tf = |x: f32, y: f32| [(x - vx) * sx, (y - vy) * sy];
    let filled = |a: &HashMap<String, String>| {
        let style = a.get("style").cloned().unwrap_or_default();
        let from_style = style.split(';').find_map(|kv| kv.trim().strip_prefix("fill:").map(|v| v.trim().to_string()));
        match from_style.or_else(|| a.get("fill").cloned()) {
            Some(f) => f != "none" && f != "transparent",
            None => false,
        }
    };
    let num = |a: &HashMap<String, String>, k: &str| a.get(k).and_then(|v| length_mm(v)).map(|x| x.0).unwrap_or(0.0);

    let mut prims = Vec::new();
    let mut rest = &text[svg_end + 1..];
    while let Some(lt) = rest.find('<') {
        let gt = rest[lt..].find('>').ok_or("unterminated element")? + lt;
        let tag = &rest[lt + 1..gt];
        let name = tag.split_whitespace().next().unwrap_or("").trim_end_matches('/');
        let a = attributes(tag);
        match name {
            "line" => prims.push(Prim2d::Line { a: tf(num(&a, "x1"), num(&a, "y1")), b: tf(num(&a, "x2"), num(&a, "y2")) }),
            "rect" => {
                let (x, y) = (num(&a, "x"), num(&a, "y"));
                prims.push(Prim2d::Rect { min: tf(x, y), max: tf(x + num(&a, "width"), y + num(&a, "height")), fill: filled(&a) });
            }
            "circle" => prims.push(Prim2d::Circle { c: tf(num(&a, "cx"), num(&a, "cy")), r: num(&a, "r") * sx, fill: filled(&a) }),
            "ellipse" => {
                let (cx, cy, rx, ry) = (num(&a, "cx"), num(&a, "cy"), num(&a, "rx"), num(&a, "ry"));
                let pts = (0..=24)
                    .map(|i| {
                        let t = i as f32 / 24.0 * std::f32::consts::TAU;
                        tf(cx + rx * t.cos(), cy + ry * t.sin())
                    })
                    .collect();
                prims.push(Prim2d::Polyline { pts, closed: true, fill: filled(&a) });
            }
            "polyline" | "polygon" => {
                let n = numbers(a.get("points").map(String::as_str).unwrap_or(""));
                let pts: Vec<[f32; 2]> = n.as_chunks::<2>().0.iter().map(|c| tf(c[0], c[1])).collect();
                if pts.len() >= 2 {
                    prims.push(Prim2d::Polyline { pts, closed: name == "polygon", fill: filled(&a) && name == "polygon" });
                }
            }
            "path" => {
                let fill = filled(&a);
                for (pts, closed) in path_polylines(a.get("d").map(String::as_str).unwrap_or("")) {
                    let pts = pts.into_iter().map(|p| tf(p[0], p[1])).collect();
                    prims.push(Prim2d::Polyline { pts, closed, fill: fill && closed });
                }
            }
            "text" => {
                let end = rest[gt..].find("</text>").map(|e| e + gt).unwrap_or(gt);
                let content: String = rest[gt + 1..end].split('<').next().unwrap_or("").trim().to_string();
                let size = a.get("font-size").and_then(|v| length_mm(v)).map(|x| x.0).unwrap_or(3.0) * sy;
                if !content.is_empty() {
                    prims.push(Prim2d::Text { pos: tf(num(&a, "x"), num(&a, "y")), text: content, size });
                }
            }
            _ => {}
        }
        rest = &rest[gt + 1..];
    }
    if prims.is_empty() {
        return Err("no drawing elements found".into());
    }
    let (width, height) = if mw > 0.0 && mh > 0.0 { (mw, mh) } else { bounds_2d(&prims) };
    Ok(Model2d { width, height, prims })
}

/// Extent of 2D primitives (max x, max y).
fn bounds_2d(prims: &[Prim2d]) -> (f32, f32) {
    let mut w: f32 = 0.0;
    let mut h: f32 = 0.0;
    let mut ext = |p: [f32; 2]| {
        w = w.max(p[0]);
        h = h.max(p[1]);
    };
    for p in prims {
        match p {
            Prim2d::Line { a, b } => {
                ext(*a);
                ext(*b);
            }
            Prim2d::Rect { min, max, .. } => {
                ext(*min);
                ext(*max);
            }
            Prim2d::Circle { c, r, .. } => ext([c[0] + r, c[1] + r]),
            Prim2d::Polyline { pts, .. } => pts.iter().for_each(|p| ext(*p)),
            Prim2d::Text { pos, .. } => ext(*pos),
        }
    }
    (w, h)
}

#[cfg(test)]
mod tests;
