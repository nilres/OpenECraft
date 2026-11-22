//! Graphics of drawing pages: layers, outlines of shapes, dimensions and images.
//! Electrical items (symbols, wires) are on no layer.

use crate::geom::{dist_to_segment, p, P, R};
use crate::model::*;
pub use crate::model::graphics::Layer;

impl Default for Layer {
    fn default() -> Self {
        Layer { name: String::new(), visible: true, locked: false, print: true, color: None }
    }
}

impl Layer {
    pub fn new(name: &str) -> Layer {
        Layer { name: name.to_string(), ..Default::default() }
    }
}

pub fn layer<'a>(prj: &'a Project, name: &str) -> Option<&'a Layer> {
    prj.layers.iter().find(|l| l.name == name)
}

/// Items of the layer are drawn (on screen, or in exports with `export`).
pub fn shown(prj: &Project, name: &str, export: bool) -> bool {
    layer(prj, name).is_none_or(|l| l.visible && (!export || l.print))
}

pub fn locked(prj: &Project, name: &str) -> bool {
    layer(prj, name).is_some_and(|l| l.locked)
}

pub fn color(prj: &Project, name: &str) -> Option<[u8; 3]> {
    layer(prj, name).and_then(|l| l.color)
}

/// Layer of a graphic item; None for symbols, wires and the other electrical items.
pub fn item_layer(page: &Page, id: u64) -> Option<&str> {
    if let Some(s) = page.shapes.iter().find(|s| s.id == id) {
        return Some(&s.layer);
    }
    if let Some(t) = page.texts.iter().find(|t| t.id == id) {
        return Some(&t.layer);
    }
    if let Some(d) = page.dimensions.iter().find(|d| d.id == id) {
        return Some(&d.layer);
    }
    page.images.iter().find(|i| i.id == id).map(|i| i.layer.as_str())
}

/// All layers of the project: the defined ones followed by names only used by items.
pub fn layer_names(prj: &Project) -> Vec<String> {
    let mut v: Vec<String> = prj.layers.iter().map(|l| l.name.clone()).collect();
    for pg in &prj.pages {
        let used = pg
            .shapes
            .iter()
            .map(|s| &s.layer)
            .chain(pg.texts.iter().map(|t| &t.layer))
            .chain(pg.dimensions.iter().map(|d| &d.layer))
            .chain(pg.images.iter().map(|i| &i.layer));
        for l in used {
            if !l.is_empty() && !v.contains(l) {
                v.push(l.clone());
            }
        }
    }
    v
}

fn for_each_layer_field(prj: &mut Project, mut f: impl FnMut(&mut String)) {
    for pg in &mut prj.pages {
        pg.shapes.iter_mut().for_each(|s| f(&mut s.layer));
        pg.texts.iter_mut().for_each(|t| f(&mut t.layer));
        pg.dimensions.iter_mut().for_each(|d| f(&mut d.layer));
        pg.images.iter_mut().for_each(|i| f(&mut i.layer));
    }
}

/// Why the layer `old` cannot be renamed to `new` (None: it can, or nothing changes);
/// `names` are the names of all layers. English, translated where shown.
pub fn layer_rename_problem(names: &[String], old: &str, new: &str) -> Option<&'static str> {
    if new == old {
        None
    } else if new.trim().is_empty() {
        Some(openecraft_i18n::tr_const!("A layer needs a name"))
    } else if names.iter().any(|n| n == new) {
        Some(openecraft_i18n::tr_const!("Name already used"))
    } else {
        None
    }
}

/// Renames a layer together with the layer of its items.
pub fn rename_layer(prj: &mut Project, old: &str, new: &str) {
    if let Some(l) = prj.layers.iter_mut().find(|l| l.name == old) {
        l.name = new.to_string();
    }
    for_each_layer_field(prj, |l| {
        if l == old {
            *l = new.to_string();
        }
    });
}

/// Removes a layer; its items move to the default layer.
pub fn remove_layer(prj: &mut Project, name: &str) {
    prj.layers.retain(|l| l.name != name);
    for_each_layer_field(prj, |l| {
        if l == name {
            l.clear();
        }
    });
}

/// Number of items on a layer (all pages).
pub fn layer_count(prj: &Project, name: &str) -> usize {
    prj.pages
        .iter()
        .map(|pg| {
            pg.shapes.iter().filter(|s| s.layer == name).count()
                + pg.texts.iter().filter(|t| t.layer == name).count()
                + pg.dimensions.iter().filter(|d| d.layer == name).count()
                + pg.images.iter().filter(|i| i.layer == name).count()
        })
        .sum()
}

// ---------------------------------------------------------------- shapes

const ARC_STEPS: usize = 72;

fn ellipse_pts(c: P, rx: f32, ry: f32, start: f32, sweep: f32, steps: usize) -> Vec<P> {
    (0..=steps)
        .map(|i| {
            let t = (start + sweep * i as f32 / steps as f32).to_radians();
            p(c.x + rx * t.cos(), c.y - ry * t.sin())
        })
        .collect()
}

/// `q` turned by `deg` degrees counter-clockwise about `c`.
pub fn turn_about(c: P, q: P, deg: f32) -> P {
    c + (q - c).turned(deg)
}

/// Outline of a shape as a polyline and whether it is closed.
pub fn outline(s: &Shape) -> (Vec<P>, bool) {
    let (pts, closed) = flat_outline(s);
    if s.angle == 0.0 || !matches!(s.kind, ShapeKind::Rect | ShapeKind::Ellipse) {
        return (pts, closed);
    }
    let c = R::from_points(s.a, s.b).center();
    (pts.into_iter().map(|q| turn_about(c, q, s.angle)).collect(), closed)
}

/// Outline of a shape before its angle.
fn flat_outline(s: &Shape) -> (Vec<P>, bool) {
    let (a, b) = (s.a, s.b);
    match s.kind {
        ShapeKind::Line => (vec![a, b], false),
        ShapeKind::Rect => (vec![a, p(b.x, a.y), b, p(a.x, b.y)], true),
        ShapeKind::Circle => {
            let r = a.dist(b);
            let mut v = ellipse_pts(a, r, r, 0.0, 360.0, ARC_STEPS);
            v.pop();
            (v, true)
        }
        ShapeKind::Ellipse => {
            let r = R::from_points(a, b);
            let mut v = ellipse_pts(r.center(), r.width() / 2.0, r.height() / 2.0, 0.0, 360.0, ARC_STEPS);
            v.pop();
            (v, true)
        }
        ShapeKind::Arc => {
            let r = a.dist(b);
            let start = (-(b.y - a.y)).atan2(b.x - a.x).to_degrees();
            let steps = ((s.sweep.abs() / 5.0).ceil() as usize).max(2);
            (ellipse_pts(a, r, r, start, s.sweep, steps), false)
        }
        ShapeKind::Polyline => (s.pts.clone(), false),
        ShapeKind::Polygon => (s.pts.clone(), true),
    }
}

/// Shapes that enclose an area (can be filled).
pub fn closed(kind: ShapeKind) -> bool {
    matches!(kind, ShapeKind::Rect | ShapeKind::Circle | ShapeKind::Ellipse | ShapeKind::Polygon)
}

pub fn shape_bounds(s: &Shape) -> R {
    let mut r = R::nothing();
    for q in outline(s).0 {
        r.extend(q);
    }
    r
}

fn inside(pts: &[P], q: P) -> bool {
    let mut odd = false;
    let n = pts.len();
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + n - 1) % n]);
        if (a.y > q.y) != (b.y > q.y) && q.x < (b.x - a.x) * (q.y - a.y) / (b.y - a.y) + a.x {
            odd = !odd;
        }
    }
    odd
}

pub fn shape_hit(s: &Shape, q: P, tol: f32) -> bool {
    let (pts, is_closed) = outline(s);
    let edge = |i: usize| (pts[i], pts[(i + 1) % pts.len()]);
    let edges = if is_closed { pts.len() } else { pts.len().saturating_sub(1) };
    (0..edges).any(|i| {
        let (a, b) = edge(i);
        dist_to_segment(q, a, b) <= tol
    }) || (s.fill && is_closed && inside(&pts, q))
}

/// Applies `f` to every point of a shape.
pub fn map_shape(s: &mut Shape, f: impl Fn(P) -> P) {
    s.a = f(s.a);
    s.b = f(s.b);
    for q in &mut s.pts {
        *q = f(*q);
    }
}

// ---------------------------------------------------------------- scale

/// A drawing scale (real length per millimetre on the page) as it is written: `1 : 50`,
/// `2 : 1` for enlargements.
pub fn scale_text(scale: f32) -> String {
    let n = |v: f32| {
        let t = format!("{:.3}", v);
        t.trim_end_matches('0').trim_end_matches('.').to_string()
    };
    match scale {
        s if s.is_nan() || s <= 0.0 => "1 : 1".to_string(),
        s if s < 1.0 => format!("{} : 1", n(1.0 / s)),
        s => format!("1 : {}", n(s)),
    }
}

/// Common drawing scales (real length per millimetre): enlargements 10 : 1 to 2 : 1,
/// 1 : 1 and reductions to 1 : 1000.
pub const SCALES: [f32; 15] = [0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 25.0, 50.0, 100.0, 200.0, 250.0, 500.0, 1000.0];

/// The drawing scale of a page; 1 for a value that is no scale.
pub fn page_scale(page: &Page) -> f32 {
    if page.scale.is_finite() && page.scale > 0.0 { page.scale } else { 1.0 }
}

/// Sizes of a shape in mm on the page: width and height of its outline, the length of
/// lines and the length of the outline of polylines, polygons, circles and arcs.
pub fn shape_size(s: &Shape) -> (P, f32) {
    let r = shape_bounds(s);
    let (pts, closed) = outline(s);
    let mut len: f32 = pts.windows(2).map(|w| w[0].dist(w[1])).sum();
    if closed && pts.len() > 2 {
        len += pts[pts.len() - 1].dist(pts[0]);
    }
    if s.kind == ShapeKind::Circle {
        len = 2.0 * std::f32::consts::PI * s.a.dist(s.b);
    }
    (p(r.width(), r.height()), len)
}

// ---------------------------------------------------------------- dimensions

/// Measuring direction (unit vector).
pub fn dim_dir(d: &Dimension) -> P {
    match d.kind {
        DimKind::Horizontal => p(1.0, 0.0),
        DimKind::Vertical => p(0.0, 1.0),
        DimKind::Aligned => {
            let v = d.b - d.a;
            if v.len() < 1e-6 {
                p(1.0, 0.0)
            } else {
                v * (1.0 / v.len())
            }
        }
    }
}

/// Normal of the measuring direction; positive offsets lie on this side.
pub fn dim_normal(d: &Dimension) -> P {
    let u = dim_dir(d);
    p(u.y, -u.x)
}

fn dot(a: P, b: P) -> f32 {
    a.x * b.x + a.y * b.y
}

/// Measured length on the page (signed along the measuring direction).
pub fn dim_length(d: &Dimension) -> f32 {
    dot(d.b - d.a, dim_dir(d))
}

/// Offset of a dimension line through `q`.
pub fn dim_offset_through(d: &Dimension, q: P) -> f32 {
    dot(q - d.a, dim_normal(d))
}

/// Shown text: the value in real units or the text set by hand.
pub fn dim_text(d: &Dimension) -> String {
    if !d.text.is_empty() {
        return d.text.clone();
    }
    let v = format!("{:.*}", d.decimals as usize, dim_length(d).abs() * d.scale);
    if d.unit.is_empty() {
        v
    } else {
        format!("{v} {}", d.unit)
    }
}

/// Geometry of a dimension: dimension line, extension lines, arrow heads, text.
pub struct DimGeom {
    pub line: (P, P),
    pub ext: [(P, P); 2],
    pub arrows: [[P; 3]; 2],
    pub text_pos: P,
    /// The text goes left / right of the line (vertical dimension lines).
    pub text_side: Option<bool>,
    /// The text goes below the line.
    pub text_below: bool,
}

pub const ARROW: f32 = 2.5;

pub fn dim_geom(d: &Dimension) -> DimGeom {
    let u = dim_dir(d);
    let n = dim_normal(d);
    let len = dim_length(d);
    let p1 = d.a + n * d.offset;
    let p2 = p1 + u * len;
    let ext = |from: P, to: P| {
        let v = to - from;
        let l = v.len();
        if l < 1e-6 {
            return (from, to);
        }
        let s = v * (1.0 / l);
        (from + s * 1.0f32.min(l), to + s * 1.5)
    };
    let e2_from = d.b;
    let dir = if len < 0.0 { -u } else { u };
    let head = |tip: P, back: P| [tip, tip + back * ARROW + n * 0.6, tip + back * ARROW - n * 0.6];
    let mid = p1 + u * (len / 2.0);
    let text_side = (n.x.abs() > n.y.abs()).then_some(n.x > 0.0);
    let text_below = text_side.is_none() && n.y > 0.0;
    DimGeom {
        line: (p1, p2),
        ext: [ext(d.a, p1), ext(e2_from, p2)],
        arrows: [head(p1, dir), head(p2, -dir)],
        text_pos: mid + n * 1.2,
        text_side,
        text_below,
    }
}

pub fn dim_bounds(d: &Dimension) -> R {
    let g = dim_geom(d);
    let mut r = R::from_points(d.a, d.b);
    r.extend(g.line.0);
    r.extend(g.line.1);
    r.extend(g.text_pos);
    r
}

pub fn dim_hit(d: &Dimension, q: P, tol: f32) -> bool {
    let g = dim_geom(d);
    dist_to_segment(q, g.line.0, g.line.1) <= tol
        || g.ext.iter().any(|(a, b)| dist_to_segment(q, *a, *b) <= tol)
        || g.text_pos.dist(q) <= 2.5
}

// ---------------------------------------------------------------- images

/// Size in mm of an image `width` mm wide with the aspect ratio of its pixels.
pub fn image_size(px: [u32; 2], width: f32) -> P {
    if px[0] == 0 || px[1] == 0 {
        return p(width, width * 0.75);
    }
    p(width, width * px[1] as f32 / px[0] as f32)
}

/// Pixel size of a PNG or JPEG file read from its header.
pub fn image_pixels(bytes: &[u8]) -> Option<[u32; 2]> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") && bytes.len() >= 24 {
        let be = |i: usize| u32::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        return Some([be(16), be(20)]);
    }
    if bytes.starts_with(&[0xFF, 0xD8]) {
        let mut i = 2;
        while i + 9 < bytes.len() {
            if bytes[i] != 0xFF {
                i += 1;
                continue;
            }
            let marker = bytes[i + 1];
            let len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
            // Start of frame markers (not DHT, JPG, DAC).
            if (0xC0..=0xCF).contains(&marker) && !matches!(marker, 0xC4 | 0xC8 | 0xCC) {
                let h = u16::from_be_bytes([bytes[i + 5], bytes[i + 6]]) as u32;
                let w = u16::from_be_bytes([bytes[i + 7], bytes[i + 8]]) as u32;
                return Some([w, h]);
            }
            i += 2 + len;
        }
    }
    None
}

/// Adds an image `width` mm wide with its top-left corner at `at`. Returns its id.
pub fn add_image(prj: &mut Project, page: usize, at: P, name: &str, bytes: &[u8], width: f32, layer: &str) -> u64 {
    let px = image_pixels(bytes).unwrap_or([0, 0]);
    let id = prj.new_id();
    let size = image_size(px, width);
    prj.pages[page].images.push(ImageItem {
        id,
        a: at,
        b: at + size,
        file: crate::document::Document::new(name, bytes),
        px,
        layer: layer.to_string(),
        angle: 0.0,
    });
    id
}

/// Corners of an image on the page (top left, top right, bottom right, bottom left of
/// the picture), turned by its angle.
pub fn image_corners(img: &ImageItem) -> [P; 4] {
    let r = R::from_points(img.a, img.b);
    [r.min, p(r.max.x, r.min.y), r.max, p(r.min.x, r.max.y)].map(|q| turn_about(r.center(), q, img.angle))
}

/// What an image covers on the page.
pub fn image_bounds(img: &ImageItem) -> R {
    let mut r = R::nothing();
    for q in image_corners(img) {
        r.extend(q);
    }
    r
}

/// `q` lies on the (turned) image.
pub fn image_hit(img: &ImageItem, q: P) -> bool {
    let r = R::from_points(img.a, img.b);
    r.contains(turn_about(r.center(), q, -img.angle))
}

#[cfg(test)]
mod tests;
