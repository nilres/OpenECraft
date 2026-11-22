//! Grips of a selected item, dragged to change its size or shape: the corners and
//! edge middles of rectangles, ellipses, structure boxes and device boxes, the corners
//! of images (their aspect ratio is kept), the ends and corner points of lines,
//! polylines, polygons and dimensions, the radius of circles and the ends of arcs.
//! Shapes, images and texts are also turned to any angle about their middle.

use crate::device_boxes;
use crate::geom::{p, P, R, GRID};
use crate::graphics::{self, turn_about};
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
    /// Its rectangle before the angle, and the angle.
    Image(R, f32),
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
        return Some(Item::Image(R::from_points(i.a, i.b), i.angle));
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
        ShapeKind::Rect | ShapeKind::Ellipse => turned(frame(R::from_points(s.a, s.b), true), R::from_points(s.a, s.b), s.angle),
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
        Some(Item::Image(r, angle)) => turned(frame(r, false), r, angle),
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

/// Grips of a rectangle `r` turned by `angle` about its middle.
fn turned(grips: Vec<(Grip, P)>, r: R, angle: f32) -> Vec<(Grip, P)> {
    grips.into_iter().map(|(g, q)| (g, turn_about(r.center(), q, angle))).collect()
}

/// [`reframe`] (or [`reframe_keeping_aspect`]) of the rectangle `r` turned by `angle`
/// about its middle: the edges that do not move stay where they are on the page. The
/// result is the rectangle before the angle.
pub fn reframe_turned(r: R, angle: f32, grip: (i8, i8), to: P, min: f32, keep_aspect: bool) -> R {
    let c = r.center();
    let local = R { min: r.min - c, max: r.max - c };
    let to = (to - c).turned(-angle);
    let n = if keep_aspect { reframe_keeping_aspect(local, grip, to, min) } else { reframe(local, grip, to, min) };
    let mid = c + n.center().turned(angle);
    let half = p(n.width() / 2.0, n.height() / 2.0);
    R { min: mid - half, max: mid + half }
}

impl Grab {
    /// The angle of the frame of a turned rectangle, ellipse or image, else 0.
    pub fn angle(&self) -> f32 {
        match &self.from {
            Item::Shape(s) if matches!(s.kind, ShapeKind::Rect | ShapeKind::Ellipse) => s.angle,
            Item::Image(_, angle) => *angle,
            _ => 0.0,
        }
    }
}

/// [`drag`] on a grid of `grid` mm: the grip goes to the nearest grid point, the grip
/// of a turned frame (rectangle, ellipse, image) gives it a width and height of whole
/// grid steps along its own sides instead.
pub fn drag_on_grid(prj: &mut Project, page_i: usize, g: &Grab, to: P, grid: f32) -> Result<(), String> {
    let frame = match (&g.from, g.grip) {
        (Item::Shape(s), Grip::Frame(x, y)) => Some((R::from_points(s.a, s.b), (x, y))),
        (Item::Image(r, _), Grip::Frame(x, y)) => Some((*r, (x, y))),
        _ => None,
    };
    let Some((r, (x, y))) = frame.filter(|_| g.angle() != 0.0 && grid > 0.0) else {
        return drag(prj, page_i, g, to.snap(grid));
    };
    let c = r.center();
    let mut local = (to - c).turned(-g.angle());
    // Measured from the edge that stays.
    let step = |v: f32, from: f32| from + ((v - from) / grid).round() * grid;
    match x {
        -1 => local.x = step(local.x, r.max.x - c.x),
        1 => local.x = step(local.x, r.min.x - c.x),
        _ => {}
    }
    match y {
        -1 => local.y = step(local.y, r.max.y - c.y),
        1 => local.y = step(local.y, r.min.y - c.y),
        _ => {}
    }
    drag(prj, page_i, g, c + local.turned(g.angle()))
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
                    let r = reframe_turned(R::from_points(s.a, s.b), s.angle, (x, y), to, MIN_SIZE, false);
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
        (Item::Image(r, angle), Grip::Frame(x, y)) => {
            let r = reframe_turned(*r, *angle, (x, y), to, MIN_SIZE, true);
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

/// An item being turned: the middle it turns about and the item as it was.
#[derive(Clone, Debug, PartialEq)]
pub struct Turning {
    pub id: u64,
    pub pivot: P,
    from: Turned,
}

#[derive(Clone, Debug, PartialEq)]
enum Turned {
    Shape(Shape),
    Image(f32),
    Text(f32),
}

/// The middle item `id` turns about and the point on its outline the turning handle
/// grows from (towards the top of the item); none for items that do not turn
/// (symbols, wires, boxes, dimensions, circles).
pub fn turn_handle(page: &Page, id: u64) -> Option<(P, P)> {
    if let Some(s) = page.shapes.iter().find(|s| s.id == id) {
        return match s.kind {
            ShapeKind::Circle => None,
            ShapeKind::Rect | ShapeKind::Ellipse => {
                let r = R::from_points(s.a, s.b);
                Some((r.center(), turn_about(r.center(), p(r.center().x, r.min.y), s.angle)))
            }
            _ => {
                let r = graphics::shape_bounds(s);
                r.is_valid().then(|| (r.center(), p(r.center().x, r.min.y)))
            }
        };
    }
    if let Some(i) = page.images.iter().find(|i| i.id == id) {
        let r = R::from_points(i.a, i.b);
        return Some((r.center(), turn_about(r.center(), p(r.center().x, r.min.y), i.angle)));
    }
    let t = page.texts.iter().find(|t| t.id == id)?;
    Some((t.pos, t.pos + p(0.0, -1.5 * t.size.max(1.0)).turned(t.angle)))
}

/// Starts turning item `id`; none for items that do not turn.
pub fn start_turn(page: &Page, id: u64) -> Option<Turning> {
    let (pivot, _) = turn_handle(page, id)?;
    let from = if let Some(s) = page.shapes.iter().find(|s| s.id == id) {
        Turned::Shape(s.clone())
    } else if let Some(i) = page.images.iter().find(|i| i.id == id) {
        Turned::Image(i.angle)
    } else {
        Turned::Text(page.texts.iter().find(|t| t.id == id)?.angle)
    };
    Some(Turning { id, pivot, from })
}

/// The angle of the frame grips of item `id`: that of a turned rectangle, ellipse or
/// image, else 0.
pub fn frame_angle(page: &Page, id: u64) -> f32 {
    if let Some(s) = page.shapes.iter().find(|s| s.id == id && matches!(s.kind, ShapeKind::Rect | ShapeKind::Ellipse)) {
        return s.angle;
    }
    page.images.iter().find(|i| i.id == id).map_or(0.0, |i| i.angle)
}

/// An angle in degrees from 0 to below 360.
pub fn normal_angle(deg: f32) -> f32 {
    let a = deg.rem_euclid(360.0);
    if (a - 360.0).abs() < 1e-3 || a.abs() < 1e-3 { 0.0 } else { a }
}

/// Turns the item of `t` by `deg` degrees counter-clockwise from where it was at the
/// start: rectangles, ellipses, images and texts take an angle, the other shapes turn
/// their points about its middle.
pub fn turn(page: &mut Page, t: &Turning, deg: f32) {
    match &t.from {
        Turned::Shape(s0) => {
            let Some(s) = page.shapes.iter_mut().find(|s| s.id == t.id) else { return };
            if matches!(s0.kind, ShapeKind::Rect | ShapeKind::Ellipse) {
                s.angle = normal_angle(s0.angle + deg);
            } else {
                let mut n = s0.clone();
                graphics::map_shape(&mut n, |q| turn_about(t.pivot, q, deg));
                (s.a, s.b) = (n.a, n.b);
                s.pts = n.pts;
            }
        }
        Turned::Image(a0) => {
            if let Some(i) = page.images.iter_mut().find(|i| i.id == t.id) {
                i.angle = normal_angle(a0 + deg);
            }
        }
        Turned::Text(a0) => {
            if let Some(x) = page.texts.iter_mut().find(|x| x.id == t.id) {
                x.angle = normal_angle(a0 + deg);
            }
        }
    }
}

#[cfg(test)]
mod tests;
