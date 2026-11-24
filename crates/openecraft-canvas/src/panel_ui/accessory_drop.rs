//! Accessories dragged from the parts management (or the data portal) into the 2D or
//! 3D view of an enclosure: the places they fit are highlighted ([`fit::places`]: the
//! free fields of a meter cabinet for its distribution panels, the faces for other
//! accessories of the enclosure, the devices whose part they fit), dropped onto one
//! they go there.

use super::new_enclosure::find_part;
use super::PanelUi;
use egui::{Color32, Pos2, Shape, Stroke};
use openecraft_core::geom::{p, P};
use openecraft_core::model::Project;
use openecraft_core::panel::fit::{self, Place};
use openecraft_core::panel::{self, Enclosure, Face, PanelItem, SitePose};
use openecraft_core::parts::vec3::V3;
use openecraft_core::parts::{Part, PartsDb};
use openecraft_i18n::tr_fmt;
use openecraft_ui::part_editor::PartDrag;

/// The part being dragged: its number and the part when a parts list of the editor has
/// it (looked up once per drag).
pub(super) type Dragged = Option<(String, Option<Part>)>;

/// Colour of the places an accessory fits.
fn fit_color() -> Color32 {
    crate::style::ok()
}

impl PanelUi {
    /// Follows the part dragged in the application (call before the views are drawn).
    pub(super) fn track_dragged_part(&mut self, ctx: &egui::Context, prj: &Project, db: &PartsDb) {
        match egui::DragAndDrop::payload::<PartDrag>(ctx) {
            None => self.dragged = None,
            Some(d) if self.dragged.as_ref().is_some_and(|x| x.0 == d.number) => {}
            Some(d) => self.dragged = Some((d.number.clone(), find_part(prj, db, &d.number).cloned())),
        }
    }

    /// The part dragged over a view, while a part is being dragged.
    fn dragged_now(&self, ctx: &egui::Context) -> Option<(String, Option<Part>)> {
        let d = egui::DragAndDrop::payload::<PartDrag>(ctx)?;
        self.dragged.clone().filter(|x| x.0 == d.number)
    }

    /// What the status says while a part is dragged over enclosure `ei`; `full`: it is an
    /// accessory of the enclosure, but no place is left for it.
    fn drag_status(&self, e: &Enclosure, number: &str, places: &[Place], hit: Option<&Place>, face: Option<Face>, full: bool) -> String {
        match hit {
            Some(Place::Device(_, tag)) => tr_fmt!("Drop {} onto {} to add it to the device", number, tag),
            Some(_) => tr_fmt!("Drop {} here", number),
            None if places.is_empty() && full => tr_fmt!("No free place is left for {} in this enclosure", number),
            None if places.is_empty() => tr_fmt!("{} fits neither this enclosure nor a device in it", number),
            None if face.is_some_and(|f| !places.iter().any(|pl| pl.face(e) == Some(f))) => {
                let mut faces: Vec<String> = Vec::new();
                for f in places.iter().filter_map(|pl| pl.face(e)) {
                    let title = e.face_title(f);
                    if !faces.contains(&title) {
                        faces.push(title);
                    }
                }
                tr_fmt!("{} fits on another face of this enclosure: {}", number, faces.join(", "))
            }
            None => tr_fmt!("Drop {} onto one of the highlighted places", number),
        }
    }

    /// Puts the dropped accessory at a place of enclosure `ei` and says so.
    fn drop_accessory(&mut self, prj: &mut Project, ei: usize, part: &Part, place: &Place, at: P) {
        if !prj.parts.iter().any(|p| p.number == part.number) {
            prj.parts.push(part.clone());
        }
        let number = part.number.as_str();
        self.status = match fit::put(prj, &PartsDb::default(), ei, number, place, at) {
            Ok(_) if matches!(place, Place::Device(..)) => {
                let Place::Device(_, tag) = place else { unreachable!() };
                tr_fmt!("{} added to device {}", number, tag)
            }
            Ok(id) => {
                self.select_item(id);
                tr_fmt!("{} placed in {}", number, prj.layout.enclosures[ei].name)
            }
            Err(e) => e,
        };
    }

    /// A part dragged over the 2D view of enclosure `ei`: the places it fits on the face
    /// shown highlighted; dropped onto one it goes there. `at` is the pointer in face
    /// coordinates, `to_screen` maps face coordinates to the view.
    pub(super) fn accessory_drop_2d(&mut self, ui: &egui::Ui, resp: &egui::Response, prj: &mut Project, ei: usize, at: Option<P>, to_screen: &dyn Fn(P) -> Pos2) {
        let Some((number, part)) = self.dragged_now(ui.ctx()) else { return };
        if !resp.contains_pointer() {
            return;
        }
        let Some(part) = part else {
            self.status = tr_fmt!("{} is not in the parts database yet: import it, then drag it here", number);
            return;
        };
        let places = fit::places(prj, ei, &part);
        let e = &prj.layout.enclosures[ei];
        let hit = at.and_then(|q| fit::place_at(e, &places, self.face, q)).cloned();
        let painter = ui.painter_at(resp.rect);
        for pl in places.iter().filter(|pl| pl.face(e) == Some(self.face)) {
            let Some(r) = pl.rect(e) else { continue };
            let screen = egui::Rect::from_two_pos(to_screen(r.min), to_screen(r.max));
            let strong = hit.as_ref() == Some(pl);
            painter.rect_filled(screen, 2.0, fit_color().gamma_multiply(if strong { 0.45 } else { 0.18 }));
            painter.rect_stroke(screen, 2.0, Stroke::new(if strong { 2.5 } else { 1.2 }, fit_color()), egui::StrokeKind::Inside);
        }
        let full = places.is_empty() && fit::fits_enclosure(prj, ei, &part);
        self.status = self.drag_status(e, &number, &places, hit.as_ref(), Some(self.face), full);
        ui.ctx().set_cursor_icon(if hit.is_some() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::NoDrop });
        if resp.dnd_release_payload::<PartDrag>().is_some() {
            if let (Some(pl), Some(q)) = (hit, at) {
                self.drop_accessory(prj, ei, &part, &pl, q);
            }
        }
    }

    /// A part dragged over the 3D view of enclosure `ei` (shown at `pose`): the places it
    /// fits highlighted on every face; dropped onto one it goes there.
    pub(super) fn accessory_drop_3d(&mut self, ui: &egui::Ui, resp: &egui::Response, prj: &mut Project, ei: usize, pose: &SitePose, project_view: &dyn Fn(V3) -> Option<(Pos2, f32)>) {
        let Some((number, part)) = self.dragged_now(ui.ctx()) else { return };
        if !resp.contains_pointer() {
            return;
        }
        let Some(part) = part else {
            self.status = tr_fmt!("{} is not in the parts database yet: import it, then drag it here", number);
            return;
        };
        let places = fit::places(prj, ei, &part);
        let e = &prj.layout.enclosures[ei];
        let outlines: Vec<(usize, Vec<Pos2>)> = places.iter().enumerate().filter_map(|(k, pl)| Some((k, outline_3d(e, pl, pose, project_view)?))).collect();
        let ptr = ui.input(|i| i.pointer.hover_pos());
        // The smallest outline under the pointer: a device before the face it is on.
        let hit = ptr.and_then(|q| outlines.iter().filter(|(_, o)| inside(o, q)).min_by(|a, b| area(&a.1).total_cmp(&area(&b.1))).map(|(k, _)| *k));
        let painter = ui.painter_at(resp.rect);
        for (k, o) in &outlines {
            let strong = hit == Some(*k);
            painter.add(Shape::convex_polygon(o.clone(), fit_color().gamma_multiply(if strong { 0.45 } else { 0.18 }), Stroke::new(if strong { 2.5 } else { 1.2 }, fit_color())));
        }
        let full = places.is_empty() && fit::fits_enclosure(prj, ei, &part);
        self.status = self.drag_status(e, &number, &places, hit.map(|k| &places[k]), None, full);
        ui.ctx().set_cursor_icon(if hit.is_some() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::NoDrop });
        // The status beside the pointer: the 3D view shows other things in the status bar.
        if let Some(q) = ptr {
            let galley = painter.layout(self.status.clone(), egui::TextStyle::Small.resolve(ui.style()), ui.visuals().text_color(), 320.0);
            let frame = egui::Rect::from_min_size(q + egui::vec2(16.0, 14.0), galley.size()).expand(4.0);
            painter.rect(frame, 3.0, ui.visuals().extreme_bg_color, Stroke::new(1.0, fit_color()), egui::StrokeKind::Outside);
            painter.galley(frame.min + egui::vec2(4.0, 4.0), galley, ui.visuals().text_color());
        }
        if resp.dnd_release_payload::<PartDrag>().is_some() {
            if let (Some(k), Some(q)) = (hit, ptr) {
                let at = point_on_face(e, &places[k], pose, project_view, q);
                let place = places[k].clone();
                self.drop_accessory(prj, ei, &part, &place, at);
            }
        }
    }
}

/// The box of a place in enclosure coordinates: a slot or a face flat on its face, a
/// device its own box.
fn place_box(e: &Enclosure, place: &Place) -> Option<(V3, V3)> {
    if let Place::Device(id, _) = place {
        return e.item(*id).map(|it| panel::item_box(e, it));
    }
    let (face, r) = (place.face(e)?, place.rect(e)?);
    let it = PanelItem { face, x: r.min.x, y: r.min.y, w: r.width(), h: r.height(), d: 0.0, ..Default::default() };
    Some(panel::item_box(e, &it))
}

/// The outline of a place in the 3D view: the hull of its box's corners on the screen.
fn outline_3d(e: &Enclosure, place: &Place, pose: &SitePose, project_view: &dyn Fn(V3) -> Option<(Pos2, f32)>) -> Option<Vec<Pos2>> {
    let (a, b) = place_box(e, place)?;
    let corners: Option<Vec<Pos2>> = (0..8)
        .map(|c| {
            let q = [if c & 1 == 0 { a[0] } else { b[0] }, if c & 2 == 0 { a[1] } else { b[1] }, if c & 4 == 0 { a[2] } else { b[2] }];
            project_view(pose.apply([q[0], e.height - q[1], q[2]])).map(|s| s.0)
        })
        .collect();
    let hull = convex_hull(corners?);
    (hull.len() >= 3).then_some(hull)
}

/// Where on its face the pointer `q` is over a place in the 3D view (face coordinates):
/// the nearest of a grid of points over it on the screen.
fn point_on_face(e: &Enclosure, place: &Place, pose: &SitePose, project_view: &dyn Fn(V3) -> Option<(Pos2, f32)>, q: Pos2) -> P {
    let (Some(face), Some(r)) = (place.face(e), place.rect(e)) else { return p(0.0, 0.0) };
    const N: usize = 24;
    let mut best = (f32::INFINITY, r.center());
    for i in 0..=N {
        for j in 0..=N {
            let at = p(r.min.x + r.width() * i as f32 / N as f32, r.min.y + r.height() * j as f32 / N as f32);
            let it = PanelItem { face, x: at.x, y: at.y, ..Default::default() };
            let (lo, _) = panel::item_box(e, &it);
            if let Some((s, _)) = project_view(pose.apply([lo[0], e.height - lo[1], lo[2]])) {
                let d = s.distance(q);
                if d < best.0 {
                    best = (d, at);
                }
            }
        }
    }
    best.1
}

/// The convex hull of points, counter-clockwise (monotone chain).
pub(super) fn convex_hull(mut pts: Vec<Pos2>) -> Vec<Pos2> {
    pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    pts.dedup_by(|a, b| a.distance(*b) < 0.01);
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: Pos2, a: Pos2, b: Pos2| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let mut hull: Vec<Pos2> = Vec::new();
    for pass in 0..2 {
        let start = hull.len();
        let iter: Box<dyn Iterator<Item = &Pos2>> = if pass == 0 { Box::new(pts.iter()) } else { Box::new(pts.iter().rev()) };
        for &q in iter {
            while hull.len() >= start + 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], q) <= 0.0 {
                hull.pop();
            }
            hull.push(q);
        }
        hull.pop();
    }
    hull
}

/// True if `q` lies inside the convex polygon `poly`.
pub(super) fn inside(poly: &[Pos2], q: Pos2) -> bool {
    let n = poly.len();
    let side = |k: usize| {
        let (a, b) = (poly[k], poly[(k + 1) % n]);
        (b.x - a.x) * (q.y - a.y) - (b.y - a.y) * (q.x - a.x)
    };
    n >= 3 && ((0..n).all(|k| side(k) >= 0.0) || (0..n).all(|k| side(k) <= 0.0))
}

fn area(poly: &[Pos2]) -> f32 {
    let n = poly.len();
    ((0..n).map(|k| poly[k].x * poly[(k + 1) % n].y - poly[(k + 1) % n].x * poly[k].y).sum::<f32>() / 2.0).abs()
}

#[cfg(test)]
mod tests;
