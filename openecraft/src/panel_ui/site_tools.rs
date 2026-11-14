//! Tools of the site plan: drawing the boundary of the plot corner by corner, placing
//! the reference point, measuring from one point to another and dragging the corners
//! of the boundary ([`openecraft_core::panel::plot`]), and the edit mode in which the
//! buildings are dragged to their places and turned. The points snap to the corners of
//! the boundary, the rooms and the shafts and to the reference point.

use super::site_plan::{self, Frame};
use super::{PanelUi, View};
use egui::{vec2, Key, Pos2, Rect, Response, Stroke};
use openecraft_core::model::Project;
use openecraft_core::panel::{self, plot, GeoReference};
use openecraft_i18n::{tr, tr_fmt};

/// What a click in the site plan does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SiteTool {
    /// A click on a room shows it; the corners of the boundary are dragged.
    #[default]
    Select,
    /// Draws the boundary of the plot corner by corner.
    Boundary,
    /// Places the reference point.
    Reference,
    /// Measures from one point to another.
    Measure,
    /// Edit mode: the buildings are dragged to their places and turned.
    Move,
    /// Places earth rods of a building where clicked.
    EarthRod,
    /// Places air-termination rods on the roofs of a building where clicked.
    AirRod,
    /// Places freestanding PV arrays on the plot where clicked.
    PvArray,
}

/// A measure written out: distance, how far east and north, the direction.
pub(crate) fn measure_text(m: &plot::Measure) -> String {
    tr_fmt!("{} m: {} m east, {} m north, {}° from north", format!("{:.3}", m.distance / 1000.0), format!("{:.3}", m.east), format!("{:.3}", m.north), format!("{:.1}", m.bearing))
}

impl PanelUi {
    /// Chooses a tool of the site plan (Layout space › Site); the site plan is shown.
    /// Chosen again, it stops.
    pub fn choose_site_tool(&mut self, tool: SiteTool) {
        let again = self.site && self.view == View::Plan && self.site_tool == tool;
        if !self.site {
            self.show_site_plan(None);
        }
        self.view = View::Plan;
        self.site_points.clear();
        self.site_drag = None;
        self.site_move = None;
        self.site_hover = None;
        self.site_tool = if again { SiteTool::Select } else { tool };
        self.status.clear();
    }

    /// A click with the tool of the site plan at `q` (the plant, mm); `close`: on the
    /// first corner of the boundary drawn.
    pub(super) fn site_click(&mut self, prj: &mut Project, q: [f32; 2], close: bool) {
        match self.site_tool {
            SiteTool::Select | SiteTool::Move => {}
            SiteTool::Boundary if close && self.site_points.len() >= 3 => self.finish_boundary(prj),
            SiteTool::Boundary => self.site_points.push(q),
            SiteTool::Reference => {
                let r = prj.layout.plot.reference.clone().unwrap_or_default();
                match plot::set_reference(prj, Some(GeoReference { x: q[0], z: q[1], ..r })) {
                    Ok(()) => {
                        self.status = tr_fmt!("Reference point placed at x {} m, z {} m; enter its coordinates in the Buildings pane", format!("{:.3}", q[0] / 1000.0), format!("{:.3}", q[1] / 1000.0));
                        self.site_tool = SiteTool::Select;
                    }
                    Err(e) => self.status = e,
                }
            }
            SiteTool::EarthRod | SiteTool::AirRod => self.place_rod(prj, q),
            SiteTool::PvArray => match panel::solar::add_ground_array(prj, Some([q[0].round(), q[1].round()])) {
                Ok(id) => {
                    self.select_array(prj, id);
                    let name = prj.layout.plot.solar.iter().find(|a| a.id == id).map(|a| a.name.clone()).unwrap_or_default();
                    self.status = tr_fmt!("Freestanding PV array {} placed; click for the next one · Esc: stop", name);
                }
                Err(e) => self.status = e,
            },
            SiteTool::Measure => {
                if self.site_points.len() >= 2 {
                    self.site_points.clear();
                }
                self.site_points.push(q);
                if let [a, b] = self.site_points[..] {
                    self.status = measure_text(&plot::measure(&prj.layout, a, b));
                }
            }
        }
    }

    /// An earth rod or an air-termination rod at `q` (the plant) for the building there,
    /// otherwise the building of the earthing menu.
    fn place_rod(&mut self, prj: &mut Project, q: [f32; 2]) {
        let poses = panel::site_poses(prj);
        let Some(bi) = panel::buildings::building_at(prj, &poses, q).or(self.earthing_building(prj)) else {
            self.status = tr("Choose the building.").to_string();
            return;
        };
        let local = panel::buildings::building_pose(&prj.layout.buildings[bi]).local([q[0], 0.0, q[1]]);
        let at = [local[0].round(), local[2].round()];
        let name = prj.layout.buildings[bi].name.clone();
        if self.site_tool == SiteTool::AirRod {
            let b = &prj.layout.buildings[bi];
            if !b.roofs.iter().any(|r| panel::roofs::covering_at(r, at).is_some()) {
                self.status = tr_fmt!("Click on a roof of building {}", name);
                return;
            }
            match panel::lightning::add_rod(prj, bi, Some(at)) {
                Ok(_) => self.status = tr_fmt!("Air-termination rod placed on building {}; click for the next one · Esc: stop", name),
                Err(e) => self.status = e,
            }
        } else {
            match panel::earthing::add_electrode(prj, bi, panel::ElectrodeKind::Rod, Some(at)) {
                Ok(_) => self.status = tr_fmt!("Earth rod placed at building {}; click for the next one · Esc: stop", name),
                Err(e) => self.status = e,
            }
        }
        self.earthing_target = Some(bi);
    }

    /// Closes the boundary drawn: it becomes the plot's (or stays to be corrected).
    pub(super) fn finish_boundary(&mut self, prj: &mut Project) {
        let pts = std::mem::take(&mut self.site_points);
        match plot::set_outline(prj, pts.clone()) {
            Ok(()) => {
                let area = format!("{:.1}", plot::area(&prj.layout.plot.outline));
                let outside: Vec<String> = plot::outside(prj)
                    .into_iter()
                    .map(|s| match s {
                        plot::Standing::Room(i) | plot::Standing::Cabinet(i) => prj.layout.enclosures[i].name.clone(),
                        plot::Standing::Roof(bi, k) => format!("{} {}", prj.layout.buildings[bi].name, prj.layout.buildings[bi].roofs[k].name),
                        plot::Standing::PvArray(k) => prj.layout.plot.solar[k].name.clone(),
                    })
                    .collect();
                self.status = if outside.is_empty() { tr_fmt!("Plot boundary drawn: {} m²", area) } else { tr_fmt!("Plot boundary drawn: {} m²; outside it: {}", area, outside.join(", ")) };
                self.site_tool = SiteTool::Select;
            }
            Err(e) => {
                self.status = e;
                self.site_points = pts;
            }
        }
    }

    /// Keys of the site tools: Enter closes the boundary, Backspace takes back the last
    /// point, Escape drops the points or stops the tool. Enter and Backspace stand for
    /// the keys of finishing a polyline and of Delete.
    pub(super) fn site_key(&mut self, prj: &mut Project, key: Key) {
        match key {
            Key::Enter if self.site_tool == SiteTool::Boundary && self.site_points.len() >= 3 => self.finish_boundary(prj),
            Key::Backspace => {
                self.site_points.pop();
            }
            Key::Escape if !self.site_points.is_empty() => self.site_points.clear(),
            Key::Escape => self.site_tool = SiteTool::Select,
            _ => {}
        }
    }

    /// The corner of the plot boundary at `p`.
    fn corner_at(prj: &Project, frame: &Frame, p: Pos2) -> Option<usize> {
        prj.layout.plot.outline.iter().position(|&q| frame.at(q).distance(p) <= 6.0)
    }

    /// What the tools of the site plan do with the pointer in `rect`; gives whether a
    /// corner of the boundary is dragged (the view is then not moved).
    pub(super) fn site_tools(&mut self, ui: &egui::Ui, rect: Rect, resp: &Response, prj: &mut Project) -> bool {
        let frame = site_plan::frame(rect, prj, &self.site_plan);
        if self.site_tool == SiteTool::Move {
            return self.move_buildings(ui, resp, prj, &frame);
        }
        if self.site_tool == SiteTool::Select {
            if resp.drag_started() {
                if let Some(k) = ui.input(|i| i.pointer.press_origin()).and_then(|p| Self::corner_at(prj, &frame, p)) {
                    self.site_drag = Some((k, prj.layout.plot.outline.clone()));
                }
            }
            if let (Some((k, _)), Some(p)) = (&self.site_drag, resp.interact_pointer_pos()) {
                let (q, _) = site_plan::snap(&site_plan::snap_points(prj, Some(*k)), &frame, p);
                if let Some(c) = prj.layout.plot.outline.get_mut(*k) {
                    *c = q;
                }
            }
            if resp.drag_stopped() {
                if let Some((_, old)) = self.site_drag.take() {
                    let moved = prj.layout.plot.outline.clone();
                    if let Err(e) = plot::set_outline(prj, moved) {
                        prj.layout.plot.outline = old;
                        self.status = e;
                    }
                }
            }
            if self.site_drag.is_some() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            } else if resp.hover_pos().is_some_and(|p| Self::corner_at(prj, &frame, p).is_some()) {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }
            return self.site_drag.is_some();
        }
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        if let Some(p) = resp.hover_pos().filter(|_| resp.clicked()) {
            let (q, _) = site_plan::snap(&site_plan::snap_points(prj, None), &frame, p);
            let close = self.site_tool == SiteTool::Boundary && self.site_points.first().is_some_and(|&f| frame.at(f).distance(p) <= 8.0);
            self.site_click(prj, q, close);
        }
        let gestures = crate::gestures::get(ui.ctx());
        let finish = gestures.double_clicked(crate::gestures::MouseAction::FinishByDoubleClick, resp, ui.input(|i| i.modifiers));
        if finish && self.site_tool == SiteTool::Boundary && self.site_points.len() >= 3 {
            self.finish_boundary(prj);
        }
        if (resp.hovered() || !self.site_points.is_empty()) && !ui.ctx().egui_wants_keyboard_input() {
            let sc = crate::shortcuts::get(ui.ctx());
            let keys = ui.input(|i| {
                [
                    (sc.action_pressed(i, crate::shortcuts::Action::FinishPolyline), Key::Enter),
                    (sc.cmd_pressed(i, crate::commands::Cmd::Delete), Key::Backspace),
                    (i.key_pressed(Key::Escape), Key::Escape),
                ]
            });
            for (pressed, key) in keys {
                if pressed {
                    self.site_key(prj, key);
                }
            }
        }
        false
    }

    /// The edit mode of the site plan: a building is dragged to its new place, R turns
    /// the one under the pointer (or dragged) by 90° about its middle, Escape stops.
    /// Gives whether a building is dragged (the view is then not moved).
    fn move_buildings(&mut self, ui: &egui::Ui, resp: &Response, prj: &mut Project, frame: &Frame) -> bool {
        let poses = panel::site_poses(prj);
        let hover = resp.hover_pos().and_then(|p| site_plan::building_at(prj, &poses, frame, p));
        if resp.drag_started() {
            if let Some((p, bi)) = ui.input(|i| i.pointer.press_origin()).and_then(|p| site_plan::building_at(prj, &poses, frame, p).map(|bi| (p, bi))) {
                self.site_move = Some((bi, frame.plant(p), prj.layout.buildings[bi].site));
            }
        }
        if let (Some((bi, from, place)), Some(p)) = (self.site_move, resp.interact_pointer_pos()) {
            let q = frame.plant(p);
            let site = panel::SitePlace { x: ((place.x + q[0] - from[0]) / 10.0).round() * 10.0, z: ((place.z + q[1] - from[1]) / 10.0).round() * 10.0, ..place };
            if panel::buildings::place_building(prj, bi, site).is_ok() {
                let b = &prj.layout.buildings[bi];
                self.status = tr_fmt!("Building {} at x {} m, z {} m", b.name, format!("{:.2}", b.site.x / 1000.0), format!("{:.2}", b.site.z / 1000.0));
            }
        }
        if resp.drag_stopped() {
            if let Some((bi, ..)) = self.site_move.take() {
                self.report_building(prj, bi);
            }
        }
        let target = self.site_move.map(|m| m.0).or(hover);
        if !ui.ctx().egui_wants_keyboard_input() && (resp.hovered() || self.site_move.is_some()) {
            let (turn, escape) = ui.input(|i| (i.key_pressed(Key::R), i.key_pressed(Key::Escape)));
            if let (true, Some(bi)) = (turn, target) {
                match panel::buildings::move_building(prj, bi, [0.0, 0.0], 90.0) {
                    Ok(()) => {
                        if let Some(m) = self.site_move.as_mut() {
                            // Dragged on from where it is now.
                            m.2 = prj.layout.buildings[bi].site;
                            m.1 = resp.interact_pointer_pos().map_or(m.1, |p| frame.plant(p));
                        }
                        self.report_building(prj, bi);
                    }
                    Err(e) => self.status = e,
                }
            }
            if escape && self.site_move.is_none() {
                self.site_tool = SiteTool::Select;
            }
        }
        if self.site_move.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if hover.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
        self.site_hover = target;
        self.site_move.is_some()
    }

    /// The status after a building was moved or turned: its place and angle, and what of
    /// it stands outside the plot boundary.
    fn report_building(&mut self, prj: &Project, bi: usize) {
        let Some(b) = prj.layout.buildings.get(bi) else { return };
        let outside = plot::outside(prj).into_iter().any(|s| match s {
            plot::Standing::Room(i) => panel::buildings::building_of(&prj.layout, i) == Some(bi),
            plot::Standing::Roof(b, _) => b == bi,
            plot::Standing::Cabinet(_) | plot::Standing::PvArray(_) => false,
        });
        let at = (format!("{:.2}", b.site.x / 1000.0), format!("{:.2}", b.site.z / 1000.0), format!("{:.0}", b.site.angle));
        self.status = if outside {
            tr_fmt!("Building {} at x {} m, z {} m, turned {}°: it stands outside the plot boundary", b.name, at.0, at.1, at.2)
        } else {
            tr_fmt!("Building {} at x {} m, z {} m, turned {}°", b.name, at.0, at.1, at.2)
        };
    }

    /// What the tools of the site plan show over it: the boundary being drawn, the
    /// measure, the building to move, the point snapped to and the coordinates under the
    /// pointer.
    pub(super) fn paint_site_tools(&self, ui: &egui::Ui, rect: Rect, prj: &Project, pointer: Option<Pos2>) {
        let frame = site_plan::frame(rect, prj, &self.site_plan);
        let painter = ui.painter_at(rect);
        let visuals = ui.visuals();
        let accent = visuals.selection.stroke.color;
        if let (SiteTool::Move, Some(bi)) = (self.site_tool, self.site_hover) {
            let poses = panel::site_poses(prj);
            for poly in panel::buildings::building_footprint(prj, &poses, bi) {
                let pts: Vec<Pos2> = poly.iter().map(|&q| frame.at(q)).collect();
                painter.add(egui::Shape::closed_line(pts, Stroke::new(2.5, accent)));
            }
            if let Some(b) = prj.layout.buildings.get(bi) {
                let at = frame.at([b.site.x, b.site.z]);
                painter.circle_stroke(at, 5.0, Stroke::new(1.5, accent));
                let text = tr_fmt!("{}: drag to move, R: turn by 90°", b.name);
                painter.text(at + vec2(8.0, 8.0), egui::Align2::LEFT_TOP, text, egui::FontId::proportional(11.0), accent);
            }
        }
        let at = pointer.filter(|p| rect.contains(*p)).map(|p| site_plan::snap(&site_plan::snap_points(prj, self.site_drag.as_ref().map(|d| d.0)), &frame, p));
        let drawing = self.site_tool != SiteTool::Select;
        let mut pts: Vec<Pos2> = self.site_points.iter().map(|&q| frame.at(q)).collect();
        if drawing {
            if let Some((q, _)) = at.filter(|_| self.site_tool != SiteTool::Measure || self.site_points.len() == 1) {
                pts.push(frame.at(q));
            }
        }
        match self.site_tool {
            SiteTool::Boundary if !pts.is_empty() => {
                painter.add(egui::Shape::line(pts.clone(), Stroke::new(2.0, crate::style::ok())));
                for p in &pts {
                    painter.circle_filled(*p, 3.0, crate::style::ok());
                }
                // The first corner closes it.
                if self.site_points.len() >= 3 {
                    painter.circle_stroke(pts[0], 7.0, Stroke::new(1.5, crate::style::ok()));
                }
                if let [.., a, b] = self.site_points.iter().chain(at.iter().map(|x| &x.0)).copied().collect::<Vec<_>>()[..] {
                    let len = (b[0] - a[0]).hypot(b[1] - a[1]);
                    painter.text(frame.at(b) + vec2(10.0, -10.0), egui::Align2::LEFT_BOTTOM, format!("{:.3} m", len / 1000.0), egui::FontId::proportional(11.0), accent);
                }
            }
            SiteTool::Measure if pts.len() >= 2 => {
                painter.line_segment([pts[0], pts[1]], Stroke::new(2.0, accent));
                for p in &pts[..2] {
                    painter.circle_stroke(*p, 4.0, Stroke::new(1.5, accent));
                }
                let q: Vec<[f32; 2]> = self.site_points.iter().copied().chain(at.iter().map(|x| x.0)).take(2).collect();
                let text = measure_text(&plot::measure(&prj.layout, q[0], q[1]));
                let galley = painter.layout_no_wrap(text, egui::FontId::proportional(11.0), visuals.text_color());
                let mid = pts[0] + (pts[1] - pts[0]) / 2.0 + vec2(8.0, -8.0);
                let back = Rect::from_min_size(mid - vec2(0.0, galley.size().y), galley.size()).expand(3.0);
                painter.rect_filled(back, 3.0, visuals.extreme_bg_color.gamma_multiply(0.9));
                painter.galley(back.min + vec2(3.0, 3.0), galley, visuals.text_color());
            }
            _ => {}
        }
        // The point snapped to, and its coordinates.
        if let Some((q, snapped)) = at {
            let p = frame.at(q);
            if snapped && (drawing || self.site_drag.is_some()) {
                painter.rect_stroke(Rect::from_center_size(p, vec2(10.0, 10.0)), 0.0, Stroke::new(1.5, accent), egui::StrokeKind::Middle);
            }
            let galley = painter.layout_no_wrap(site_plan::coordinates(prj, q), egui::FontId::monospace(11.0), visuals.weak_text_color());
            let back = Rect::from_min_size(rect.right_bottom() - galley.size() - vec2(10.0, 10.0), galley.size()).expand(3.0);
            painter.rect_filled(back, 3.0, visuals.extreme_bg_color);
            painter.galley(back.min + vec2(3.0, 3.0), galley, visuals.weak_text_color());
        }
    }
}

#[cfg(test)]
mod tests;
