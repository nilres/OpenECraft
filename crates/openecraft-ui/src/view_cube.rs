//! View cube of the 3D views: shows the direction looked from and turns the camera. A
//! click on one of its faces, edges or corners looks from there, a drag orbits; the
//! buttons beside it turn the view by a quarter, go home or fit. [`axes`] draws the
//! axis triad.

use crate::icons::{self, Icon};
use egui::epaint::Vertex;
use egui::{vec2, Color32, CursorIcon, FontId, Id, Mesh, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};
use openecraft_i18n::{tr, tr_const};
use openecraft_parts::vec3::{dot, Orbit, V3};

/// What was asked for with the cube.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    /// Look from this face, edge or corner (see [`openecraft_parts::vec3::view_from`]).
    Look([i8; 3]),
    /// Turn by this many quarter turns: positive to the left, as the cube's top is
    /// dragged (see [`openecraft_parts::vec3::quarter_turn`]).
    Turn(i32),
    /// Orbit as by a drag of this many points over the view.
    Orbit(Vec2),
    /// The starting view, everything shown.
    Home,
    /// Everything shown from the direction looked in.
    Fit,
}

/// A face of the cube: outward normal, the directions its label is written in (right,
/// down) and the English label.
struct Face {
    n: [i8; 3],
    u: [i8; 3],
    v: [i8; 3],
    label: &'static str,
}

const FACES: [Face; 6] = [
    Face { n: [0, 0, 1], u: [1, 0, 0], v: [0, -1, 0], label: tr_const!("Front") },
    Face { n: [0, 0, -1], u: [-1, 0, 0], v: [0, -1, 0], label: tr_const!("Rear") },
    Face { n: [1, 0, 0], u: [0, 0, -1], v: [0, -1, 0], label: tr_const!("Right") },
    Face { n: [-1, 0, 0], u: [0, 0, 1], v: [0, -1, 0], label: tr_const!("Left") },
    Face { n: [0, 1, 0], u: [1, 0, 0], v: [0, 0, 1], label: tr_const!("Top") },
    Face { n: [0, -1, 0], u: [1, 0, 0], v: [0, 0, -1], label: tr_const!("Bottom") },
];

/// Width of the bands of the edges and corners on a face, in halves of a side.
const BAND: f32 = 0.42;

/// Faces turned less towards the eye than this are left out.
const EDGE_ON: f32 = 0.01;

fn f3(d: [i8; 3]) -> V3 {
    d.map(f32::from)
}

/// The cube as seen by a camera at `yaw`, `pitch`, drawn at `centre` with sides of
/// `2 * half` points.
#[derive(Clone, Copy, Debug)]
pub struct Projection {
    centre: Pos2,
    half: f32,
    eye: V3,
    right: V3,
    up: V3,
}

impl Projection {
    pub fn new(yaw: f32, pitch: f32, centre: Pos2, half: f32) -> Self {
        let (eye, right, up, _) = Orbit { yaw, pitch, dist: 1.0, target: [0.0; 3] }.basis();
        Projection { centre, half, eye, right, up }
    }

    /// A point of the cube (each coordinate -1 … 1) on screen.
    pub fn at(&self, q: V3) -> Pos2 {
        self.centre + self.dir(q)
    }

    /// A direction of the cube on screen (points).
    fn dir(&self, q: V3) -> Vec2 {
        vec2(dot(q, self.right), -dot(q, self.up)) * self.half
    }

    /// How much a face with the outward normal `n` is turned towards the eye: 0 edge-on,
    /// 1 straight.
    fn facing(&self, n: [i8; 3]) -> f32 {
        dot(f3(n), self.eye)
    }

    /// The regions of the faces turned towards the eye: the direction each one looks
    /// from (a face, an edge or a corner), its outline and the face it lies on.
    pub fn regions(&self) -> Vec<Region> {
        let g = [-1.0, -1.0 + BAND, 1.0 - BAND, 1.0];
        let mut out = Vec::new();
        for (k, face) in FACES.iter().enumerate().filter(|(_, f)| self.facing(f.n) > EDGE_ON) {
            let (n, u, v) = (f3(face.n), f3(face.u), f3(face.v));
            let point = |a: f32, b: f32| self.at([0, 1, 2].map(|i| n[i] + u[i] * a + v[i] * b));
            for i in 0..3 {
                for j in 0..3 {
                    let dir = [0, 1, 2].map(|c| face.n[c] + face.u[c] * (i as i8 - 1) + face.v[c] * (j as i8 - 1));
                    let pts = [point(g[i], g[j]), point(g[i + 1], g[j]), point(g[i + 1], g[j + 1]), point(g[i], g[j + 1])];
                    out.push(Region { dir, pts, face: k });
                }
            }
        }
        out
    }
}

/// A part of a face that looks from `dir` when clicked.
#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    pub dir: [i8; 3],
    pub pts: [Pos2; 4],
    face: usize,
}

/// The direction of the region at `p`, if any.
pub fn region_at(regions: &[Region], p: Pos2) -> Option<[i8; 3]> {
    regions.iter().find(|r| inside(p, &r.pts)).map(|r| r.dir)
}

/// Whether `p` lies in the convex outline `pts` (either winding).
fn inside(p: Pos2, pts: &[Pos2]) -> bool {
    let side = |i: usize| {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        (b - a).x * (p - a).y - (b - a).y * (p - a).x
    };
    let s: Vec<f32> = (0..pts.len()).map(side).collect();
    s.iter().all(|v| *v >= 0.0) || s.iter().all(|v| *v <= 0.0)
}

/// The view cube widget.
#[derive(Clone, Copy, Debug)]
pub struct ViewCube {
    /// Half a side of the cube (points).
    pub half: f32,
    /// The names of the sides on the faces.
    pub labels: bool,
    /// The buttons beside the cube: turn left and right, home and (with `fit`) fit.
    pub buttons: bool,
    pub fit: bool,
}

impl ViewCube {
    /// The cube with labels and buttons of the large 3D views.
    pub fn full() -> Self {
        ViewCube { half: 26.0, labels: true, buttons: true, fit: true }
    }

    /// A small cube without labels and buttons (previews).
    pub fn compact() -> Self {
        ViewCube { half: 10.0, labels: false, buttons: false, fit: false }
    }

    /// Offset of the buttons from the centre, along both axes.
    fn button_offset(&self) -> f32 {
        self.half * 1.75 + 2.0
    }

    /// The space the cube takes with its buttons, centred on it.
    pub fn size(&self) -> Vec2 {
        let r = if self.buttons { self.button_offset() + 11.0 } else { self.half * 1.8 };
        Vec2::splat(2.0 * r)
    }

    /// Draws the cube at `centre` for a camera at `yaw`, `pitch`; gives what was asked
    /// for. `id` must be unique among the cubes shown.
    pub fn show(&self, ui: &mut Ui, id: Id, centre: Pos2, yaw: f32, pitch: f32) -> Option<Action> {
        let proj = Projection::new(yaw, pitch, centre, self.half);
        let regions = proj.regions();
        let area = Rect::from_center_size(centre, Vec2::splat(self.half * 3.6));
        let resp = ui.interact(area, id, Sense::click_and_drag());
        let dragging = resp.dragged();
        let hover = resp.hover_pos().filter(|_| !dragging).and_then(|p| region_at(&regions, p));
        let mut action = None;
        if dragging {
            action = Some(Action::Orbit(resp.drag_delta()));
        } else if resp.clicked() {
            action = resp.interact_pointer_pos().and_then(|p| region_at(&regions, p)).map(Action::Look);
        }
        if hover.is_some() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        let active = resp.hovered() || dragging;
        self.paint(ui, &proj, &regions, hover, active);
        resp.on_hover_text(tr("Click a face, edge or corner to look from there; drag to orbit"));
        if self.buttons {
            let o = self.button_offset();
            let mut buttons = vec![
                (Icon::TurnLeft, vec2(-o, -o), tr("Turn the view 90° to the left"), Action::Turn(1)),
                (Icon::TurnRight, vec2(o, -o), tr("Turn the view 90° to the right"), Action::Turn(-1)),
                (Icon::Home, vec2(-o, o), tr("Isometric view, everything shown"), Action::Home),
            ];
            if self.fit {
                buttons.push((Icon::Fit, vec2(o, o), tr("Show everything (Home)"), Action::Fit));
            }
            for (icon, at, tip, act) in buttons {
                let rect = Rect::from_center_size(centre + at, Vec2::splat(ui.spacing().interact_size.y));
                if ui.put(rect, |ui: &mut Ui| icons::button(ui, icon, tip)).clicked() {
                    action = Some(act);
                }
            }
        }
        action
    }

    fn paint(&self, ui: &Ui, proj: &Projection, regions: &[Region], hover: Option<[i8; 3]>, active: bool) {
        let painter = ui.painter();
        let dark = ui.visuals().dark_mode;
        let base: [f32; 3] = if dark { [72.0, 76.0, 84.0] } else { [236.0, 238.0, 242.0] };
        let fade = if active { 1.0 } else { 0.8 };
        let selected = ui.visuals().selection.bg_fill;
        let outline = Stroke::new(1.0, if dark { Color32::from_gray(140) } else { Color32::from_gray(120) });
        let text = ui.visuals().strong_text_color();
        for r in regions {
            let k = proj.facing(FACES[r.face].n);
            let fill = if hover == Some(r.dir) {
                selected
            } else {
                let [cr, cg, cb] = base.map(|c| (c * (0.8 + 0.2 * k)) as u8);
                Color32::from_rgb(cr, cg, cb)
            };
            painter.add(Shape::convex_polygon(r.pts.to_vec(), fill.gamma_multiply(fade), Stroke::NONE));
        }
        for face in FACES.iter().filter(|f| proj.facing(f.n) > EDGE_ON) {
            let (n, u, v) = (f3(face.n), f3(face.u), f3(face.v));
            let corner = |a: f32, b: f32| proj.at([0, 1, 2].map(|i| n[i] + u[i] * a + v[i] * b));
            let pts = vec![corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0), corner(-1.0, 1.0)];
            painter.add(Shape::closed_line(pts, outline));
            // Labels fade out as their face turns away.
            let k = ((proj.facing(face.n) - 0.2) * 3.0).clamp(0.0, 1.0);
            if self.labels && k > 0.0 {
                let color = text.gamma_multiply(k * fade);
                painter.add(face_label(ui, tr(face.label), proj.at(n), proj.dir(u), proj.dir(v), color));
            }
        }
    }
}

/// `text` written on a face: centred at `at`, along `u` (one half side to the right on
/// screen) and `v` (one half side down), shrunk to fit.
fn face_label(ui: &Ui, text: &str, at: Pos2, u: Vec2, v: Vec2, color: Color32) -> Shape {
    let galley = ui.painter().layout_no_wrap(text.to_string(), FontId::proportional(14.0), color);
    let size = galley.size();
    // Points of the text per half side: at most 1.5 halves wide.
    let per_half = (size.x / 1.5).max(36.0);
    let [w, h] = ui.ctx().fonts(|f| f.font_image_size()).map(|n| n as f32);
    let mut mesh = Mesh::default();
    for row in &galley.rows {
        let base = mesh.vertices.len() as u32;
        mesh.indices.extend(row.visuals.mesh.indices.iter().map(|i| i + base));
        mesh.vertices.extend(row.visuals.mesh.vertices.iter().map(|vert| {
            let q = (row.pos.to_vec2() + vert.pos.to_vec2() - size / 2.0) / per_half;
            Vertex { pos: at + u * q.x + v * q.y, uv: egui::pos2(vert.uv.x / w, vert.uv.y / h), color }
        }));
    }
    Shape::mesh(mesh)
}

/// Colours and names of the axes x, y and z.
const AXES: [(V3, [u8; 3], &str); 3] = [([1.0, 0.0, 0.0], [215, 60, 55], "X"), ([0.0, 1.0, 0.0], [80, 165, 60], "Y"), ([0.0, 0.0, 1.0], [55, 120, 220], "Z")];

/// The axis triad: the axes x (red), y (green) and z (blue) from `origin`, `len` points
/// long, as seen by a camera at `yaw`, `pitch`; far ones drawn first.
pub fn axes(painter: &Painter, origin: Pos2, yaw: f32, pitch: f32, len: f32) {
    let proj = Projection::new(yaw, pitch, origin, len);
    let mut order: Vec<_> = AXES.iter().collect();
    order.sort_by(|a, b| dot(a.0, proj.eye).total_cmp(&dot(b.0, proj.eye)));
    for (axis, [r, g, b], name) in order {
        let color = Color32::from_rgb(*r, *g, *b);
        let tip = proj.at(*axis);
        painter.line_segment([origin, tip], Stroke::new(2.0, color));
        painter.text(origin + (tip - origin) * 1.4, egui::Align2::CENTER_CENTER, *name, FontId::proportional(11.0), color);
    }
    painter.circle_filled(origin, 2.0, Color32::from_gray(140));
}

#[cfg(test)]
mod tests;
