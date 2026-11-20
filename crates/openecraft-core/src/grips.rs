//! Grips of a selected item, dragged to change its size or shape: the corners and
//! edge middles of rectangles, ellipses, structure boxes and device boxes, the corners
//! of images (their aspect ratio is kept), the ends and corner points of lines,
//! polylines, polygons and dimensions, the radius of circles and the ends of arcs.

use crate::device_boxes;
use crate::geom::{p, P, R, GRID};
use crate::model::*;

/// A point of an item that can be dragged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grip {
    /// A point of the outline of a rectangular item: `x` and `y` are -1 (left, top),
    /// 0 (middle) or 1 (right, bottom).
    Frame(i8, i8),
    /// End or corner point: 0 = `a`, 1 = `b` of lines and dimensions, the start of an
    /// arc (1), corner `i` of polylines and polygons.
    Point(usize),
    /// A point on a circle (right, bottom, left, top): its radius.
    Radius(u8),
    /// The end of an arc: its angle.
    ArcEnd,
}

/// Smallest width and height a grip gives an item, in mm.
pub const MIN_SIZE: f32 = 1.0;

/// The item as it was when its grip was taken.
#[derive(Clone, Debug, PartialEq)]
enum Item {
    Shape(Shape),
    Image(R),
    StructureBox(R),
    DeviceBox(R),
    Dimension(P, P),
}

/// A grip being dragged: the item and its grip at the start of the drag.
#[derive(Clone, Debug, PartialEq)]
pub struct Grab {
    pub id: u64,
    pub grip: Grip,
    from: Item,
}

fn item(prj: &Project, page: &Page, id: u64) -> Option<Item> {
    if let Some(s) = page.shapes.iter().find(|s| s.id == id) {
        return Some(Item::Shape(s.clone()));
    }
    if let Some(i) = page.images.iter().find(|i| i.id == id) {
        return Some(Item::Image(R::from_points(i.a, i.b)));
    }
    if let Some(b) = page.structure_boxes.iter().find(|b| b.id == id) {
        return Some(Item::StructureBox(R::from_points(b.a, b.b)));
    }
    if let Some(d) = page.dimensions.iter().find(|d| d.id == id) {
        return Some(Item::Dimension(d.a, d.b));
    }
    let s = page.symbols.iter().find(|s| s.id == id && device_boxes::is_box(s))?;
    device_boxes::rect(prj, s).map(Item::DeviceBox)
}

/// The grips of a rectangle; only its corners without `edges`.
fn frame(r: R, edges: bool) -> Vec<(Grip, P)> {
    let at = |v: i8, lo: f32, hi: f32| match v {
        -1 => lo,
        0 => (lo + hi) / 2.0,
        _ => hi,
    };
    let mut v = Vec::new();
    for y in -1..=1i8 {
        for x in -1..=1i8 {
            if (x, y) != (0, 0) && (edges || (x != 0 && y != 0)) {
                v.push((Grip::Frame(x, y), p(at(x, r.min.x, r.max.x), at(y, r.min.y, r.max.y))));
            }
        }
    }
    v
}

/// Angle of `q` seen from `c` in degrees, counter-clockwise on the page (y down).
fn angle(c: P, q: P) -> f32 {
    (-(q.y - c.y)).atan2(q.x - c.x).to_degrees()
}

/// Point at `deg` degrees and distance `r` from `c`.
fn polar(c: P, r: f32, deg: f32) -> P {
    let t = deg.to_radians();
    c + p(r * t.cos(), -r * t.sin())
}

/// `deg` as a sweep in the direction of `like`: (0, 360] or [-360, 0).
fn sweep_like(deg: f32, like: f32) -> f32 {
    let v = deg.rem_euclid(360.0);
    if like < 0.0 {
        if v < 0.5 { -360.0 } else { v - 360.0 }
    } else if v < 0.5 {
        360.0
    } else {
        v
    }
}

fn shape_grips(s: &Shape) -> Vec<(Grip, P)> {
    match s.kind {
        ShapeKind::Line => vec![(Grip::Point(0), s.a), (Grip::Point(1), s.b)],
        ShapeKind::Rect | ShapeKind::Ellipse => frame(R::from_points(s.a, s.b), true),
        ShapeKind::Circle => {
            let r = s.a.dist(s.b);
            [p(r, 0.0), p(0.0, r), p(-r, 0.0), p(0.0, -r)].into_iter().enumerate().map(|(i, d)| (Grip::Radius(i as u8), s.a + d)).collect()
        }
        ShapeKind::Arc => {
            let r = s.a.dist(s.b);
            vec![(Grip::Point(1), s.b), (Grip::ArcEnd, polar(s.a, r, angle(s.a, s.b) + s.sweep))]
        }
        ShapeKind::Polyline | ShapeKind::Polygon => s.pts.iter().enumerate().map(|(i, q)| (Grip::Point(i), *q)).collect(),
    }
}

/// The grips of item `id` on page `page_i` with their places; none for items
/// without grips (symbols, wires, texts, …).
pub fn grips(prj: &Project, page_i: usize, id: u64) -> Vec<(Grip, P)> {
    match item(prj, &prj.pages[page_i], id) {
        Some(Item::Shape(s)) => shape_grips(&s),
        Some(Item::Image(r)) => frame(r, false),
        Some(Item::StructureBox(r) | Item::DeviceBox(r)) => frame(r, true),
        Some(Item::Dimension(a, b)) => vec![(Grip::Point(0), a), (Grip::Point(1), b)],
        None => Vec::new(),
    }
}

/// Takes grip `grip` of item `id` to drag it; none if the item has no such grip.
pub fn grab(prj: &Project, page_i: usize, id: u64, grip: Grip) -> Option<Grab> {
    let from = item(prj, &prj.pages[page_i], id)?;
    grips(prj, page_i, id).iter().any(|(g, _)| *g == grip).then_some(Grab { id, grip, from })
}

/// The rectangle `r` with the frame grip `x`, `y` moved to `to`; the opposite edges
/// stay, the size stays at least `min`.
pub fn reframe(r: R, (x, y): (i8, i8), to: P, min: f32) -> R {
    let (mut lo, mut hi) = (r.min, r.max);
    match x {
        -1 => lo.x = to.x.min(hi.x - min),
        1 => hi.x = to.x.max(lo.x + min),
        _ => {}
    }
    match y {
        -1 => lo.y = to.y.min(hi.y - min),
        1 => hi.y = to.y.max(lo.y + min),
        _ => {}
    }
    R { min: lo, max: hi }
}

/// The rectangle `r` with its corner `x`, `y` moved towards `to`, keeping its aspect
/// ratio: the opposite corner stays, the larger change of width or height wins.
pub fn reframe_keeping_aspect(r: R, (x, y): (i8, i8), to: P, min: f32) -> R {
    if x == 0 || y == 0 || r.width() <= 0.0 || r.height() <= 0.0 {
        return reframe(r, (x, y), to, min);
    }
    let anchor = p(if x < 0 { r.max.x } else { r.min.x }, if y < 0 { r.max.y } else { r.min.y });
    let ratio = r.height() / r.width();
    let w = ((to.x - anchor.x) * x as f32).max(min);
    let h = ((to.y - anchor.y) * y as f32).max(min);
    let (w, h) = if h / ratio > w { (h / ratio, h) } else { (w, w * ratio) };
    R::from_points(anchor, anchor + p(w * x as f32, h * y as f32))
}

/// Moves the grip of `g` to `to` (page coordinates): the item takes the shape it
/// had at the start of the drag, changed by the grip.
pub fn drag(prj: &mut Project, page_i: usize, g: &Grab, to: P) -> Result<(), String> {
    match (&g.from, g.grip) {
        (Item::Shape(s0), grip) => {
            let mut s = s0.clone();
            match (s.kind, grip) {
                (ShapeKind::Line, Grip::Point(0)) => s.a = to,
                (ShapeKind::Line, Grip::Point(_)) => s.b = to,
                (ShapeKind::Polyline | ShapeKind::Polygon, Grip::Point(i)) if i < s.pts.len() => s.pts[i] = to,
                (ShapeKind::Rect | ShapeKind::Ellipse, Grip::Frame(x, y)) => {
                    let r = reframe(R::from_points(s.a, s.b), (x, y), to, MIN_SIZE);
                    (s.a, s.b) = (r.min, r.max);
                }
                (ShapeKind::Circle, Grip::Radius(_)) => {
                    let r = s.a.dist(to).max(MIN_SIZE / 2.0);
                    s.b = polar(s.a, r, angle(s.a, s.b));
                }
                (ShapeKind::Arc, Grip::Point(1)) if !to.same(s.a) => {
                    // The start moves, the end stays.
                    let end = angle(s.a, s0.b) + s0.sweep;
                    s.b = to;
                    s.sweep = sweep_like(end - angle(s.a, to), s0.sweep);
                }
                (ShapeKind::Arc, Grip::ArcEnd) if !to.same(s.a) => s.sweep = sweep_like(angle(s.a, to) - angle(s.a, s.b), s0.sweep),
                _ => return Ok(()),
            }
            if let Some(t) = prj.pages[page_i].shapes.iter_mut().find(|t| t.id == g.id) {
                (t.a, t.b, t.sweep) = (s.a, s.b, s.sweep);
                t.pts = s.pts;
            }
        }
        (Item::Image(r), Grip::Frame(x, y)) => {
            let r = reframe_keeping_aspect(*r, (x, y), to, MIN_SIZE);
            if let Some(i) = prj.pages[page_i].images.iter_mut().find(|i| i.id == g.id) {
                (i.a, i.b) = (r.min, r.max);
            }
        }
        (Item::StructureBox(r), Grip::Frame(x, y)) => {
            let r = reframe(*r, (x, y), to, MIN_SIZE);
            if let Some(b) = prj.pages[page_i].structure_boxes.iter_mut().find(|b| b.id == g.id) {
                (b.a, b.b) = (r.min, r.max);
            }
        }
        (Item::DeviceBox(r), Grip::Frame(x, y)) => {
            let r = reframe(*r, (x, y), to, GRID);
            let now = prj.pages[page_i].symbols.iter().find(|s| s.id == g.id).and_then(|s| device_boxes::rect(prj, s));
            let snapped = R::from_points(r.min.snap(GRID), r.max.snap(GRID));
            if now != Some(snapped) {
                device_boxes::resize(prj, page_i, g.id, r)?;
            }
        }
        (Item::Dimension(..), Grip::Point(i)) => {
            if let Some(d) = prj.pages[page_i].dimensions.iter_mut().find(|d| d.id == g.id) {
                if i == 0 { d.a = to } else { d.b = to }
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests;
