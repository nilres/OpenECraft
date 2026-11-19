//! 3D view of the panel layout: the current enclosure or its whole line-up with walls,
//! door, mounting plate, rails, ducts, busbars, devices and the routed wiring; moving
//! the camera ([`super::nav3d`]) and picking. What it shows comes from the [`Scene3d`];
//! it is drawn by the graphics card ([`super::gpu3d`]) or projected here and painted far
//! to near.

use openecraft_i18n::{tr, tr_fmt};
use super::gpu3d::{Camera, Frame};
use super::nav3d;
use crate::commands::Cmd;
use crate::gestures::{MouseAction, Role};
use super::routing_ui::RouteRef;
use super::scene3d::{Scene3d, EQUIPMENT, ON_TOP};
use super::{PanelUi, RoofTarget, Tool, DOOR_OPEN_ANGLE, OUTER_WALLS};
use openecraft_core::geom::p;
use openecraft_core::model::Project;
use openecraft_core::panel::{self, ItemKind, SitePose};
use openecraft_core::parts::vec3::{self, dot, sub, Orbit, V3};
use openecraft_core::analysis::Analysis;
use openecraft_ui::view_cube::{self, Action, ViewCube};
use egui::{vec2, Color32, PointerButton, Pos2, Rect, Sense, Stroke, Vec2};

impl PanelUi {
    /// Opens or closes the door in the 3D view.
    pub fn door_button(&mut self, ui: &mut egui::Ui) {
        let text = if self.door_open { tr("Close door") } else { tr("Open door") };
        if ui.button(text).on_hover_text(tr("Swing the enclosure door open or shut (door devices and their wires move with it)")).clicked() {
            self.door_open = !self.door_open;
        }
    }

    /// 3D view of the current enclosure without the toolbar (wiring assistant).
    pub fn view_3d_current(&mut self, ui: &mut egui::Ui, prj: &mut Project, an: &Analysis) {
        if let Some(ei) = self.enclosure_index(prj) {
            self.view_3d(ui, prj, ei, an);
        }
    }

    /// Enclosures shown in the 3D view with their poses in view coordinates (x right,
    /// y up, z towards the door): all of them at their places in the room, a room with
    /// the cabinets standing in it, or the current one or its whole line-up from the
    /// left with the first one's bottom at y 0.
    /// The 3D view shows all of the plant: all enclosures, or the site is shown.
    pub(super) fn whole_site(&self) -> bool {
        self.show_site || self.site
    }

    /// The level whose rooms the 3D view shows side by side (its id): the level clicked
    /// in the navigator while the 3D view is shown.
    pub(super) fn level_3d(&self) -> Option<u64> {
        (self.site && self.site_plan.level != 0).then_some(self.site_plan.level)
    }

    pub(super) fn shown_3d(&self, prj: &Project, ei: usize) -> Vec<(usize, SitePose)> {
        if let Some(level) = self.level_3d() {
            // All rooms of the level at their places, with the cabinets standing in them.
            let poses = panel::site_poses(prj);
            return panel::buildings::rooms_on(&prj.layout, level).into_iter().flat_map(|r| std::iter::once(r).chain(panel::cabinets_in(&prj.layout, r))).map(|k| (k, poses[k])).collect();
        }
        if self.whole_site() {
            return panel::site_poses(prj).into_iter().enumerate().collect();
        }
        if prj.layout.enclosures[ei].is_room() {
            let poses = panel::site_poses(prj);
            let room = poses[ei];
            let local = |p: SitePose| SitePose { at: room.local(p.at), angle: p.angle - room.angle };
            // The whole building: its rooms with the cabinets in them.
            let rooms = match panel::buildings::building_of(&prj.layout, ei).filter(|_| self.show_building) {
                Some(bi) => {
                    let mut v = vec![ei];
                    v.extend(panel::buildings::building_rooms(&prj.layout, bi).into_iter().filter(|&r| r != ei));
                    v
                }
                None => vec![ei],
            };
            return rooms.into_iter().flat_map(|r| std::iter::once(r).chain(panel::cabinets_in(&prj.layout, r))).map(|k| (k, local(poses[k]))).collect();
        }
        let lu = if self.show_lineup { panel::lineup_of(&prj.layout, ei) } else { vec![ei] };
        let pos = panel::positions(&prj.layout);
        let first = lu.first().copied().unwrap_or(ei);
        let base = pos[first];
        let bottom = base[1] + prj.layout.enclosures[first].height;
        lu.into_iter()
            .map(|k| {
                let o = pos[k];
                (k, SitePose { at: [o[0] - base[0], bottom - o[1] - prj.layout.enclosures[k].height, o[2] - base[2]], angle: 0.0 })
            })
            .collect()
    }

    /// The pose of the room the 3D view is drawn in the coordinates of (the plant
    /// points `q` are at `local(q)`), `None` when it is drawn in the plant's.
    pub(super) fn view_frame(&self, prj: &Project, ei: usize) -> Option<SitePose> {
        (!self.whole_site() && prj.layout.enclosures.get(ei).is_some_and(|e| e.is_room())).then(|| panel::site_poses(prj)[ei])
    }

    /// The buildings whose floor slabs, stairs and roofs the 3D view shows: all of them
    /// with all enclosures, the room's own with its building.
    pub(super) fn buildings_3d(&self, prj: &Project, ei: usize) -> Vec<usize> {
        if let Some(level) = self.level_3d() {
            return panel::buildings::find_level(&prj.layout, level).map(|(bi, _)| bi).into_iter().collect();
        }
        if self.whole_site() {
            return (0..prj.layout.buildings.len()).collect();
        }
        panel::buildings::building_of(&prj.layout, ei).filter(|_| self.show_building).into_iter().collect()
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn view_3d(&mut self, ui: &mut egui::Ui, prj: &mut Project, ei: usize, an: &Analysis) {
        let shown = self.shown_3d(prj, ei);
        let dark = ui.visuals().dark_mode;
        let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = resp.rect;
        let dt = ui.input(|i| i.stable_dt).min(0.1);
        let target = if self.door_open { DOOR_OPEN_ANGLE } else { 0.0 };
        if (self.door_angle - target).abs() > 1e-3 {
            let step = dt * 2.5;
            self.door_angle += (target - self.door_angle).clamp(-step, step);
            ui.ctx().request_repaint();
        }
        // A camera set elsewhere (fit, focus) ends the glide to a view.
        if self.cam_goal.is_some_and(|(_, left)| self.cam != Some(left)) {
            self.cam_goal = None;
        }
        let fitted = default_cam(prj, &shown);
        let cam = self.cam.get_or_insert(fitted);
        let f = rect.height() * 1.3;
        let mut moved = false;
        // Orbit and pan by the gestures chosen (left button; right or middle button or
        // Shift); zoom at the pointer. Two fingers on a touch screen pan and zoom; the
        // first one does not orbit then.
        let mods = ui.input(|i| i.modifiers);
        let gestures = crate::gestures::get(ui.ctx());
        let gesture = openecraft_ui::nav::read(ui, &resp);
        let drag = if gesture.touch { None } else { gestures.drag_of(&[MouseAction::Pan3d, MouseAction::Orbit3d], &resp, mods) };
        match drag {
            Some(MouseAction::Pan3d) => nav3d::pan(cam, resp.drag_delta(), f),
            Some(_) => nav3d::orbit(cam, resp.drag_delta()),
            None => {}
        }
        moved |= drag.is_some();
        moved |= nav3d::gesture(cam, &gesture, rect.center(), f);
        // Keyboard: moving through the view, Home fits it.
        let mut fit = false;
        let mut delete = false;
        if (self.keys_active || resp.hovered()) && !ui.ctx().egui_wants_keyboard_input() {
            let sc = crate::shortcuts::get(ui.ctx());
            let ((dir, fast), home, del) = ui.input(|i| (nav3d::held(i, &sc, gestures.modifier(Role::Faster3d)), sc.cmd_pressed(i, Cmd::FitPage), sc.cmd_pressed(i, Cmd::Delete)));
            delete = del;
            if dir.iter().any(|d| *d != 0.0) {
                nav3d::keys(cam, dir, fast, dt);
                moved = true;
                ui.ctx().request_repaint();
            }
            fit = home;
        }
        let mut goal = self.cam_goal.filter(|_| !moved).map(|g| g.0);
        if fit {
            goal = Some(Orbit { yaw: cam.yaw, pitch: cam.pitch, ..fitted });
        }
        if let Some(g) = goal {
            if nav3d::glide(cam, &g, dt) {
                goal = None;
            } else {
                ui.ctx().request_repaint();
            }
        }
        let cam = *cam;
        self.cam_goal = goal.map(|g| (g, cam));
        if delete && !self.selection.is_empty() {
            self.delete_selected_3d(prj);
        }
        let (eye, right, up, fwd) = cam.basis();
        let camera = Camera { eye, right, up, fwd, f };
        let project_view = |q: V3| camera.project(rect.center(), q);
        let background = if dark { Color32::from_rgb(24, 26, 30) } else { Color32::from_rgb(214, 218, 224) };
        painter.rect_filled(rect, 0.0, background);

        let scene = self.scene_3d(ui.ctx(), prj, an, ei, &shown);
        let opacity = self.wall_opacity.clamp(0.0, 1.0);
        let hide_front = self.hide_front_walls;
        let picking = resp.clicked_by(PointerButton::Primary);
        let menu_pick = resp.secondary_clicked();
        let focusing = gestures.double_clicked(MouseAction::Focus3d, &resp, mods);
        // A part of the installation menu goes where a wall or the ceiling of a room shown
        // is clicked.
        let rooms: Vec<usize> = shown.iter().map(|s| s.0).filter(|&k| prj.layout.enclosures[k].is_room()).collect();
        let placing = match &self.tool {
            Tool::Accessory(part) if !rooms.is_empty() => Some(part.clone()),
            _ => None,
        };
        let hover = resp.hover_pos().filter(|_| placing.is_some());
        let prims = if self.gpu_3d && !self.gpu.failed() {
            painter.add(self.gpu.shape(ui.ctx(), rect, scene.clone(), Frame { camera, opacity, hide_front, background }));
            // The faces under the pointer are only needed for a click, or for the part
            // shown where it would go.
            if picking || focusing || menu_pick || hover.is_some() { project(&scene, &camera, rect, opacity, hide_front) } else { Vec::new() }
        } else {
            let prims = project(&scene, &camera, rect, opacity, hide_front);
            paint(&painter, &prims);
            prims
        };
        // The front edges of the enclosures, the plot boundary and the reference point.
        for ((a, b), stroke) in scene.outlines.iter().map(|l| (l, Stroke::new(1.5, Color32::from_rgb(110, 115, 125)))).chain(scene.plot_lines.iter().map(|l| (l, Stroke::new(2.0, crate::style::ok())))) {
            if let (Some(pa), Some(pb)) = (project_view(*a), project_view(*b)) {
                painter.line_segment([pa.0, pb.0], stroke);
            }
        }
        for label in &scene.labels {
            if let Some((q, z)) = project_view(label.at) {
                let size = (label.size * 0.25 * camera.f / z).clamp(0.0, 18.0);
                if label.warning {
                    // Readable from anywhere: on a pale box, in the colour of errors.
                    let galley = painter.layout_no_wrap(label.tag.clone(), egui::FontId::proportional(size.max(11.0)), Color32::from_rgb(170, 30, 30));
                    let rect = egui::Rect::from_center_size(q, galley.size()).expand(4.0);
                    painter.rect(rect, 3.0, Color32::from_rgba_unmultiplied(255, 245, 240, 230), Stroke::new(1.0, Color32::from_rgb(200, 60, 60)), egui::StrokeKind::Outside);
                    painter.galley(rect.shrink(4.0).min, galley, Color32::from_rgb(170, 30, 30));
                } else if size >= 7.0 && dot(sub(eye, label.at), label.normal) > 0.0 {
                    painter.text(q, egui::Align2::CENTER_CENTER, &label.tag, egui::FontId::proportional(size), Color32::from_rgb(20, 30, 50));
                }
            }
        }
        for q in scene.branches.iter().filter_map(|q| project_view(*q)) {
            painter.circle(q.0, 3.5, Color32::WHITE, Stroke::new(1.5, Color32::from_rgb(60, 60, 60)));
        }
        // While the 3D routing is searched again after a change, the routes shown are
        // those from before it.
        if an.routes_pending {
            let text = tr("Routing the cables and wires again… the routes shown are from before the last change");
            painter.text(rect.left_top() + vec2(10.0, 10.0), egui::Align2::LEFT_TOP, text, egui::FontId::proportional(13.0), crate::style::warning());
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
        }
        // While the wires, cables and tubes are laid out on another thread: how far it is.
        if let Some(done) = self.layouts.progress() {
            let width = 300.0;
            let at = Rect::from_min_size(Pos2::new(rect.center().x - width / 2.0, rect.top() + 12.0), vec2(width, 60.0));
            let mut boxed = ui.new_child(egui::UiBuilder::new().max_rect(at));
            egui::Frame::popup(boxed.style()).show(&mut boxed, |ui| {
                ui.set_width(width - 20.0);
                ui.label(tr("Laying out the wires, cables and tubes…"));
                ui.add(egui::ProgressBar::new(done).show_percentage().animate(true));
            });
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
        }
        let mut view = self.view_cube(ui, rect, resp.id, cam, fitted);
        view_cube::axes(&painter, rect.left_bottom() + vec2(34.0, -58.0), cam.yaw, cam.pitch, 20.0);
        // Double-click: the camera turns to what is under the pointer and closes in.
        if let Some(ptr) = resp.interact_pointer_pos().filter(|_| focusing) {
            if let Some(z) = depth_under(&prims, ptr, opacity < 0.99) {
                view = Some(nav3d::look_at(&cam, camera.unproject(ptr - rect.center(), z)));
            }
        }
        if let Some(view) = view {
            self.cam_goal = Some((view, cam));
            ui.ctx().request_repaint();
        }
        // Where the pointer meets the inside of a wall or the ceiling of a room shown
        // (seen through or not), otherwise the nearest solid face under it.
        let frame = self.view_frame(prj, ei);
        let target = |prj: &Project, ptr: Pos2| -> Option<Target> {
            let far = camera.unproject(ptr - rect.center(), 1000.0);
            let dir = vec3::norm(sub(far, eye));
            let plant = panel::site_poses(prj);
            let mut best: Option<(f32, Target)> = None;
            for &(k, pose) in shown.iter().filter(|s| rooms.contains(&s.0)) {
                let (le, lf) = (pose.local(eye), pose.local(vec3::add(eye, dir)));
                let ld = sub(lf, le);
                if let Some((q, t)) = openecraft_core::installation::inside_hit(&prj.layout.enclosures[k], le, ld) {
                    if best.as_ref().is_none_or(|b| t < b.0) {
                        let w = plant[k].apply([q[0], 0.0, q[2]]);
                        best = Some((t, Target { room: Some(k), at: [w[0], w[2]], height: Some(q[1]) }));
                    }
                }
            }
            best.map(|b| b.1).or_else(|| {
                let v = camera.unproject(ptr - rect.center(), solid_depth_under(&prims, ptr)?);
                let q = frame.map_or(v, |pose| pose.apply(v));
                Some(Target { room: None, at: [q[0], q[2]], height: None })
            })
        };
        if let (Some(part), Some(ptr)) = (&placing, resp.interact_pointer_pos().filter(|_| picking)) {
            let t = target(prj, ptr);
            self.place_at(prj, ei, &rooms, part, t);
            self.ghost = None;
            return;
        }
        // The part where it would go, see-through.
        if let (Some(part), Some(ptr)) = (&placing, hover) {
            if let Some(t) = target(prj, ptr) {
                match self.ghost_3d(prj, ei, &rooms, part, &t) {
                    Ok((k, it)) => {
                        if let Some(pose) = shown.iter().find(|s| s.0 == k).map(|s| s.1) {
                            draw_ghost(&painter, prj, k, &it, &pose, &project_view);
                        }
                    }
                    Err(e) => {
                        painter.text(ptr + vec2(14.0, 14.0), egui::Align2::LEFT_TOP, e, egui::FontId::proportional(12.0), crate::style::error());
                    }
                }
            }
        }
        // A right click selects what is under the pointer for the context menu.
        if let Some(ptr) = resp.interact_pointer_pos().filter(|_| menu_pick && placing.is_none()) {
            self.pick_3d(prj, an, &scene, &prims, &project_view, ptr);
        }
        if placing.is_none() {
            egui::Popup::context_menu(&resp).show(|ui| self.menu_3d(ui, prj));
        }
        // Picking: items of a neighbour in the line-up make it the current enclosure.
        if let Some(ptr) = resp.interact_pointer_pos().filter(|_| picking) {
            if self.pick_3d(prj, an, &scene, &prims, &project_view, ptr) {
                return;
            }
        }
        let help = self.help_3d(ui, &resp, placing.is_some());
        painter.text(
            rect.left_bottom() + vec2(10.0, -10.0),
            egui::Align2::LEFT_BOTTOM,
            help,
            egui::FontId::proportional(12.0),
            if dark { Color32::GRAY } else { Color32::DARK_GRAY },
        );
        if self.selected_wire.is_none() && placing.is_none() {
            let e = &prj.layout.enclosures[self.enclosure.min(prj.layout.enclosures.len() - 1)];
            // A room as in the list: width × depth × height; an enclosure width × height × depth.
            self.status = if e.is_room() {
                tr_fmt!("{}  room {} × {} × {}", e.name, format!("{:.0}", e.width), format!("{:.0}", e.depth), format!("{:.0}", e.height))
            } else {
                format!("{}  {:.0} × {:.0} × {:.0} mm", e.name, e.width, e.height, e.depth)
            };
        }
    }

    /// The help line of the 3D view; Esc stops placing a part.
    fn help_3d(&mut self, ui: &egui::Ui, resp: &egui::Response, placing: bool) -> &'static str {
        if !placing {
            return tr("Drag: orbit · right/middle or Shift+drag: pan · wheel: zoom at the pointer · double-click: look there · W A S D / arrows, Q E: move · click: select");
        }
        if resp.hovered() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.set_tool(Tool::Select);
            self.status.clear();
        }
        tr("Click on a wall or the ceiling: place the part · Esc: stop · drag: orbit · wheel: zoom")
    }

    /// Places `part` where a wall or the ceiling was clicked: at `v` of the view (none:
    /// nothing solid was clicked), in the room of `rooms` there (the current one, `ei`,
    /// where none takes the point).
    pub(super) fn place_at(&mut self, prj: &mut Project, ei: usize, rooms: &[usize], part: &str, t: Option<Target>) {
        let Some(t) = t else {
            self.status = tr("Click on a wall or the ceiling of the room").to_string();
            return;
        };
        let Some(room) = t.room.or_else(|| openecraft_core::installation::room_at(prj, rooms, t.at)).or_else(|| rooms.contains(&ei).then_some(ei)) else {
            self.status = tr("Click on a wall or the ceiling of the room").to_string();
            return;
        };
        let height = t.height.and_then(|h| openecraft_core::installation::height_for(prj, part, h));
        self.place_in_room(prj, room, part, t.at, height);
    }

    /// Where `part` would go for point `at` of the plant (see [`Self::place_at`]): the
    /// room and the item, or why it cannot go there; kept while the pointer stays.
    fn ghost_3d(&mut self, prj: &Project, ei: usize, rooms: &[usize], part: &str, t: &Target) -> Result<(usize, panel::PanelItem), String> {
        let at = t.at;
        let height = t.height.and_then(|h| openecraft_core::installation::height_for(prj, part, h));
        let key = (part.to_string(), [(at[0] / 5.0).round() as i64, (at[1] / 5.0).round() as i64, height.map_or(-1, |h| (h / 5.0).round() as i64)]);
        if let Some((_, r)) = self.ghost.as_ref().filter(|(k, _)| *k == key) {
            return r.clone();
        }
        let r = match t.room.or_else(|| openecraft_core::installation::room_at(prj, rooms, at)).or_else(|| rooms.contains(&ei).then_some(ei)) {
            Some(room) => openecraft_core::installation::preview(prj, room, part, at, height).map(|it| (room, it)),
            None => Err(tr("Click on a wall or the ceiling of the room").to_string()),
        };
        self.ghost = Some((key, r.clone()));
        r
    }

    /// The context menu of the 3D view: what was right-clicked.
    fn menu_3d(&mut self, ui: &mut egui::Ui, prj: &mut Project) {
        let any = !self.selection.is_empty();
        if ui.add_enabled(any, egui::Button::new(tr("Properties"))).clicked() {
            self.open_properties = true;
            ui.close();
        }
        let delete = ui.add_enabled(any, egui::Button::new(tr("Delete")).shortcut_text("Del")).on_hover_text(tr("Devices go back to the devices to place"));
        if delete.clicked() {
            self.delete_selected_3d(prj);
            ui.close();
        }
    }

    /// Deletes the items selected in the 3D view (of the current enclosure).
    pub(super) fn delete_selected_3d(&mut self, prj: &mut Project) {
        // Taken first: finding the current enclosure may switch the face and clear it.
        let ids: Vec<u64> = self.selection.iter().copied().collect();
        if let Some(k) = self.enclosure_index(prj) {
            let n = panel::remove_items(&mut prj.layout.enclosures[k], &ids);
            self.selection.clear();
            self.status = tr_fmt!("{} items deleted", n);
        }
    }

    /// The view cube in the top right corner of `rect`: looking from a side, an edge or a
    /// corner, quarter turns, the starting view and fitting (turns start from where a
    /// glide is heading); gives the view to glide to. A drag on it orbits at once.
    fn view_cube(&mut self, ui: &mut egui::Ui, rect: Rect, id: egui::Id, cam: Orbit, fitted: Orbit) -> Option<Orbit> {
        let cube = ViewCube::full();
        let centre = rect.right_top() + vec2(-cube.size().x / 2.0 - 6.0, cube.size().y / 2.0 + 6.0);
        let aim = self.cam_goal.map_or(cam, |(g, _)| g);
        match cube.show(ui, id.with("view_cube"), centre, cam.yaw, cam.pitch)? {
            Action::Look(d) => {
                let (yaw, pitch) = vec3::view_from(d, aim.yaw);
                Some(Orbit { yaw, pitch: pitch.clamp(-nav3d::MAX_PITCH, nav3d::MAX_PITCH), ..aim })
            }
            Action::Turn(q) => Some(Orbit { yaw: vec3::quarter_turn(aim.yaw, q), ..aim }),
            Action::Orbit(dl) => {
                if let Some(c) = self.cam.as_mut() {
                    nav3d::orbit(c, dl);
                }
                self.cam_goal = None;
                ui.ctx().request_repaint();
                None
            }
            Action::Home => {
                let (yaw, pitch) = nav3d::ISO;
                Some(Orbit { yaw, pitch, ..fitted })
            }
            Action::Fit => Some(Orbit { yaw: aim.yaw, pitch: aim.pitch, ..fitted }),
        }
    }

    /// Selects what was clicked at `ptr`: a device, a wire, a cable or tube, another item.
    /// True when the status shows what was picked.
    fn pick_3d(&mut self, prj: &Project, an: &Analysis, scene: &Scene3d, prims: &[Prim], project_view: &dyn Fn(V3) -> Option<(Pos2, f32)>, ptr: Pos2) -> bool {
        // Faces are in drawing order: the last one under the pointer is visible. A PV array
        // (on the roof, drawn with it) nearer to the eye than that one is in front of it.
        let under = |prim: &Prim| match prim {
            Prim::Face { pts, z, n, item, pv, .. } if (item.is_some() || pv.is_some()) && point_in_convex(ptr, &pts[..*n as usize]) => Some((*item, *pv, depth_at(ptr, &pts[..*n as usize], z))),
            _ => None,
        };
        let last = prims.iter().rev().find_map(under);
        let nearest_pv = prims.iter().filter_map(under).filter(|t| t.1.is_some()).min_by(|a, b| a.2.total_cmp(&b.2));
        let top = match (last, nearest_pv) {
            (Some(l), Some(p)) if p.2 < l.2 => Some(p),
            (l, p) => l.or(p),
        };
        let array = top.and_then(|t| t.1);
        let hit = top.and_then(|(item, _, z)| item.map(|i| (i, z)));
        let on_device = hit.and_then(|((k, id), _)| prj.layout.enclosures[k].item(id)).map(|i| i.kind == ItemKind::Device).unwrap_or(false);
        // Wires, cables and tubes in front of a device or a PV array clicked are picked,
        // those behind it not.
        let device_depth = top.filter(|_| on_device || array.is_some()).map(|t| t.2);
        let hit = hit.map(|(item, _)| item);
        let mut candidates: Vec<(usize, f32)> = Vec::new();
        let mut near_route: Option<(&str, f32)> = None;
        for (a, b, target) in &scene.picks {
            let (Some(pa), Some(pb)) = (project_view(*a), project_view(*b)) else { continue };
            let d = openecraft_core::geom::dist_to_segment(p(ptr.x, ptr.y), p(pa.0.x, pa.0.y), p(pb.0.x, pb.0.y));
            if d > 6.0 {
                continue;
            }
            let ab = pb.0 - pa.0;
            let t = if ab.length_sq() > 0.0 { ((ptr - pa.0).dot(ab) / ab.length_sq()).clamp(0.0, 1.0) } else { 0.0 };
            if device_depth.is_some_and(|dz| pa.1 + (pb.1 - pa.1) * t > dz - 1.0) {
                continue;
            }
            match target {
                RouteRef::Wire(wi) => match candidates.iter_mut().find(|c| c.0 == *wi) {
                    Some(c) => c.1 = c.1.min(d),
                    None => candidates.push((*wi, d)),
                },
                RouteRef::Item(key) if near_route.is_none_or(|(_, n)| d < n) => near_route = Some((key, d)),
                _ => {}
            }
        }
        candidates.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        let candidates: Vec<usize> = candidates.into_iter().map(|c| c.0).collect();
        // Earth electrodes, conductors and rods of the lightning protection near the
        // pointer and not behind what was clicked.
        let mut near_earthing: Option<((usize, panel::lightning::Owner), f32)> = None;
        for (a, b, target) in &scene.earthing_picks {
            let (Some(pa), Some(pb)) = (project_view(*a), project_view(*b)) else { continue };
            let d = openecraft_core::geom::dist_to_segment(p(ptr.x, ptr.y), p(pa.0.x, pa.0.y), p(pb.0.x, pb.0.y));
            let ab = pb.0 - pa.0;
            let t = if ab.length_sq() > 0.0 { ((ptr - pa.0).dot(ab) / ab.length_sq()).clamp(0.0, 1.0) } else { 0.0 };
            if d > 6.0 || device_depth.is_some_and(|dz| pa.1 + (pb.1 - pa.1) * t > dz - 1.0) {
                continue;
            }
            if near_earthing.is_none_or(|(_, n)| d < n) {
                near_earthing = Some((*target, d));
            }
        }
        match (hit, near_route) {
            _ if !candidates.is_empty() => {
                self.pick_wire(an, &candidates);
                return true;
            }
            // Cables and tubes of the 3D routing.
            (_, Some((key, _))) => {
                self.select_route(an, key);
                return true;
            }
            (_, None) if near_earthing.is_some() => {
                if let Some((target, _)) = near_earthing {
                    self.select_earthing(prj, target);
                }
                return true;
            }
            (Some((k, id)), None) => {
                self.switch_enclosure(k);
                self.select_item(id);
            }
            (None, None) if array.is_some() => {
                self.select_array(prj, array.unwrap_or_default());
                return true;
            }
            (None, None) => match roof_at(scene, project_view, ptr) {
                Some(target) => {
                    self.select_roof(prj, target);
                    return true;
                }
                None => self.deselect(),
            },
        }
        false
    }
}

/// Where a click in the 3D view puts a part: the room whose wall or ceiling it met (none:
/// the room the point lies in), the point of the plant (x, z) and its height above the
/// room's floor (none: not on a wall or ceiling).
pub(super) struct Target {
    pub(super) room: Option<usize>,
    pub(super) at: [f32; 2],
    pub(super) height: Option<f32>,
}

/// Part `it` of enclosure `k` (shown at `pose`) drawn see-through on top: its model, or
/// its box where its part has none.
fn draw_ghost(painter: &egui::Painter, prj: &Project, k: usize, it: &panel::PanelItem, pose: &SitePose, project_view: &dyn Fn(V3) -> Option<(Pos2, f32)>) {
    let e = &prj.layout.enclosures[k];
    let model = openecraft_core::installation::find_part(prj, &it.part).and_then(|p| p.model_3d.clone());
    let tris = match model {
        Some(m) => panel::place_triangles(e, it, panel::place_mesh(&m, it)),
        None => {
            let (a, b) = panel::item_box(e, it);
            let c = |i: usize| [if i & 1 == 0 { a[0] } else { b[0] }, if i & 2 == 0 { a[1] } else { b[1] }, if i & 4 == 0 { a[2] } else { b[2] }];
            [[0, 1, 3, 2], [4, 5, 7, 6], [0, 1, 5, 4], [2, 3, 7, 6], [0, 2, 6, 4], [1, 3, 7, 5]].iter().flat_map(|f| [[c(f[0]), c(f[1]), c(f[2])], [c(f[0]), c(f[2]), c(f[3])]]).collect()
        }
    };
    // One mesh without feathered edges (no seams between its triangles), only the faces
    // turned to the eye (no layers of it over each other).
    let fill = Color32::from_rgba_unmultiplied(70, 150, 255, 90);
    let mut mesh = egui::Mesh::default();
    for t in tris {
        let Some(pts) = t.iter().map(|q| project_view(pose.apply([q[0], e.height - q[1], q[2]])).map(|p| p.0)).collect::<Option<Vec<Pos2>>>() else { continue };
        let area = (pts[1] - pts[0]).x * (pts[2] - pts[0]).y - (pts[1] - pts[0]).y * (pts[2] - pts[0]).x;
        if area.abs() < 0.01 {
            continue;
        }
        let i = mesh.vertices.len() as u32;
        for q in pts {
            mesh.colored_vertex(q, fill);
        }
        mesh.add_triangle(i, i + 1, i + 2);
    }
    // Drawn twice over, the faces of both windings: keep those of the winding with the
    // larger area seen (the front of a closed model).
    let (mut front, mut back) = (egui::Mesh::default(), egui::Mesh::default());
    for t in mesh.indices.chunks(3) {
        let v = |k: usize| mesh.vertices[t[k] as usize].pos;
        let area = (v(1) - v(0)).x * (v(2) - v(0)).y - (v(1) - v(0)).y * (v(2) - v(0)).x;
        let m = if area > 0.0 { &mut front } else { &mut back };
        let i = m.vertices.len() as u32;
        for k in 0..3 {
            m.colored_vertex(v(k), fill);
        }
        m.add_triangle(i, i + 1, i + 2);
    }
    let area = |m: &egui::Mesh| m.indices.chunks(3).map(|t| {
        let v = |k: usize| m.vertices[t[k] as usize].pos;
        ((v(1) - v(0)).x * (v(2) - v(0)).y - (v(1) - v(0)).y * (v(2) - v(0)).x).abs()
    }).sum::<f32>();
    let shown = if area(&front) >= area(&back) { front } else { back };
    painter.add(egui::Shape::mesh(shown));
}

/// A projected face or line of one frame.
pub(super) enum Prim {
    /// `item`: enclosure and item id for picking; `z`: depth of the corners.
    /// `pv`: the PV array (id) for picking.
    Face { pts: [Pos2; 4], z: [f32; 4], n: u8, color: Color32, item: Option<(usize, u64)>, outline: bool, pv: Option<u64> },
    Line { a: Pos2, b: Pos2, color: Color32, width: f32 },
}

/// Width on the screen of a line of `diameter` mm at depth `z` (at least `least` px)
/// with the focal length `f`.
pub(super) fn line_px(diameter: f32, z: f32, least: f32, f: f32) -> f32 {
    if diameter > 0.0 { (diameter * f / z).clamp(least, 24.0) } else { least }
}

/// Cells per side of the [`ScreenGrid`].
const GRID: usize = 24;

/// The objects by the cells of a grid over the view that their rectangles on the
/// screen touch: the ones that may hide a wire there.
struct ScreenGrid {
    origin: Pos2,
    cell: Vec2,
    cells: Vec<Vec<usize>>,
    /// Objects with a corner behind the eye (no rectangle): candidates everywhere.
    everywhere: Vec<usize>,
}

impl ScreenGrid {
    fn new(view: Rect, rects: &[Option<Rect>]) -> ScreenGrid {
        let mut grid = ScreenGrid { origin: view.min, cell: view.size() / GRID as f32, cells: vec![Vec::new(); GRID * GRID], everywhere: Vec::new() };
        for (i, r) in rects.iter().enumerate() {
            match r {
                Some(r) => {
                    let ((x0, y0), (x1, y1)) = (grid.cell_of(r.min), grid.cell_of(r.max));
                    for y in y0..=y1 {
                        for x in x0..=x1 {
                            grid.cells[y * GRID + x].push(i);
                        }
                    }
                }
                None => grid.everywhere.push(i),
            }
        }
        grid
    }

    /// The cell of a point, the nearest one for a point outside the view.
    fn cell_of(&self, q: Pos2) -> (usize, usize) {
        let c = |v: f32, o: f32, s: f32| (((v - o) / s.max(1e-3)).floor().max(0.0) as usize).min(GRID - 1);
        (c(q.x, self.origin.x, self.cell.x), c(q.y, self.origin.y, self.cell.y))
    }

    /// The objects whose rectangles may touch `r` into `out`, each once: `seen` marks
    /// them with `stamp`, new for each call.
    fn candidates(&self, r: Rect, seen: &mut [u32], stamp: u32, out: &mut Vec<usize>) {
        out.clear();
        out.extend(self.everywhere.iter().copied());
        let ((x0, y0), (x1, y1)) = (self.cell_of(r.min), self.cell_of(r.max));
        for y in y0..=y1 {
            for x in x0..=x1 {
                for &i in &self.cells[y * GRID + x] {
                    if seen[i] != stamp {
                        seen[i] = stamp;
                        out.push(i);
                    }
                }
            }
        }
    }
}

/// The scene projected for the camera, in painting order: layer by layer, far to near,
/// the equipment object by object in the order of their boxes (walls and plate are
/// always behind the equipment when looking through the open door).
pub(super) fn project(scene: &Scene3d, camera: &Camera, rect: Rect, opacity: f32, hide_front: bool) -> Vec<Prim> {
    let eye = camera.eye;
    let project_view = |q: V3| camera.project(rect.center(), q);
    let order = vec3::paint_order(&scene.objects, eye);
    let mut rank = vec![0usize; scene.objects.len()];
    for (r, &i) in order.iter().enumerate() {
        rank[i] = r;
    }
    // Sort key: the layer; the object and its part (none first; 0 wires it may hide,
    // 1 its faces, 2 the wires in a duct, 3 wires nothing hides, after the last object);
    // far to near.
    let key = |layer: u8, obj: Option<(usize, u8)>, z: f32| -> u64 {
        let obj = obj.map_or(0, |(i, part)| 1 + rank[i] as u64 * 4 + part as u64);
        (layer as u64) << 56 | obj << 32 | (!z.to_bits()) as u64
    };
    let mut prims: Vec<Prim> = Vec::new();
    let mut keys: Vec<(u64, u32)> = Vec::new();
    let mut push = |prims: &mut Vec<Prim>, k: u64, prim: Prim| {
        keys.push((k, prims.len() as u32));
        prims.push(prim);
    };
    for face in &scene.faces {
        let layer = match face.side {
            Some(s) => match scene.sides[s].layer(eye, hide_front) {
                Some(layer) => layer,
                None => continue,
            },
            None => face.layer,
        };
        if dot(face.normal, sub(eye, face.pts[0])) <= 0.0 {
            continue; // back face
        }
        let n = face.n as usize;
        let (mut pts, mut zs) = ([Pos2::ZERO; 4], [0.0; 4]);
        let mut seen = true;
        for (i, corner) in face.pts[..n].iter().enumerate() {
            match project_view(*corner) {
                Some((q, z)) => (pts[i], zs[i]) = (q, z),
                None => seen = false,
            }
        }
        let depth: f32 = zs[..n].iter().sum();
        if !seen {
            continue;
        }
        let glass = face.side.is_some_and(|s| scene.sides[s].glass);
        let color = if glass { face.color.gamma_multiply(super::scene3d::GLASS_ALPHA) } else if layer == OUTER_WALLS { face.color.gamma_multiply(opacity) } else { face.color };
        let k = key(layer, face.obj.map(|o| (o, 1)), depth / n as f32);
        push(&mut prims, k, Prim::Face { pts, z: zs, n: face.n, color, item: face.item, outline: face.outline, pv: face.pv });
    }
    for l in &scene.lines {
        if let (Some(pa), Some(pb)) = (project_view(l.a), project_view(l.b)) {
            let z = (pa.1 + pb.1) / 2.0;
            let width = line_px(l.diameter, z, l.least, camera.f);
            push(&mut prims, key(ON_TOP, None, z), Prim::Line { a: pa.0, b: pb.0, color: l.color, width });
        }
    }
    // Wires over the ducts that cover them on the screen (the one they run in and its
    // neighbours), under the devices. Screen rectangles of the objects; `None` when a
    // corner is behind the eye.
    let rects: Vec<Option<Rect>> = scene
        .objects
        .iter()
        .map(|(lo, hi)| {
            let corners = (0..8).map(|c| project_view([if c & 1 == 0 { lo[0] } else { hi[0] }, if c & 2 == 0 { lo[1] } else { hi[1] }, if c & 4 == 0 { lo[2] } else { hi[2] }]));
            corners.map(|p| p.map(|x| x.0)).collect::<Option<Vec<Pos2>>>().map(|pts| Rect::from_points(&pts))
        })
        .collect();
    let grid = ScreenGrid::new(rect, &rects);
    let mut seen = vec![0u32; scene.objects.len()];
    let mut near = Vec::new();
    for (stamp, w) in (1u32..).zip(&scene.wires) {
        let l = &w.line;
        let (Some(pa), Some(pb)) = (project_view(l.a), project_view(l.b)) else { continue };
        let piece = ([0, 1, 2].map(|k| l.a[k].min(l.b[k])), [0, 1, 2].map(|k| l.a[k].max(l.b[k])));
        let z = (pa.1 + pb.1) / 2.0;
        let width = line_px(l.diameter, z, l.least, camera.f);
        let on_screen = Rect::from_two_pos(pa.0, pb.0).expand(width);
        grid.candidates(on_screen, &mut seen, stamp, &mut near);
        let hiders = near
            .iter()
            .filter(|&&i| rects[i].is_none_or(|r| r.intersects(on_screen)) && vec3::may_hide(&scene.objects[i], &piece, eye))
            .map(|&i| (i, scene.ducts[i]))
            .chain(w.inside.map(|d| (d, true)));
        let obj = match vec3::wire_place(hiders, &rank) {
            Some((i, true)) => Some((i, 2)),
            Some((i, false)) => Some((i, 0)),
            None => order.last().map(|&i| (i, 3)),
        };
        push(&mut prims, key(EQUIPMENT, obj, z), Prim::Line { a: pa.0, b: pb.0, color: l.color, width });
        // A two-colour wire (green-yellow) with its second colour along the middle.
        if let Some(stripe) = l.stripe {
            push(&mut prims, key(EQUIPMENT, obj, z), Prim::Line { a: pa.0, b: pb.0, color: stripe, width: width * 0.45 });
        }
    }
    // Equal keys keep the order they were made in.
    keys.sort_unstable();
    let mut slots: Vec<Option<Prim>> = prims.into_iter().map(Some).collect();
    keys.into_iter().filter_map(|(_, i)| slots[i as usize].take()).collect()
}

/// Paints the projected primitives in their order.
fn paint(painter: &egui::Painter, prims: &[Prim]) {
    // Mesh triangles in a row go into one mesh: without feathering they show no seams.
    let mut mesh = egui::Mesh::default();
    let flush = |mesh: &mut egui::Mesh| {
        if !mesh.is_empty() {
            painter.add(egui::Shape::mesh(std::mem::take(mesh)));
        }
    };
    for prim in prims {
        match *prim {
            Prim::Face { pts, n, color, outline, .. } => {
                let pts = &pts[..n as usize];
                if outline {
                    flush(&mut mesh);
                    painter.add(openecraft_ui::faces::face(pts.to_vec(), color, Stroke::new(0.6, color.gamma_multiply(0.6))));
                } else {
                    let first = mesh.vertices.len() as u32;
                    for &q in pts {
                        mesh.colored_vertex(q, color);
                    }
                    for i in 1..pts.len() as u32 - 1 {
                        mesh.add_triangle(first, first + i, first + i + 1);
                    }
                }
            }
            Prim::Line { a, b, color, width } => {
                flush(&mut mesh);
                painter.line_segment([a, b], Stroke::new(width, color));
            }
        }
    }
    flush(&mut mesh);
}

/// Camera looking at the enclosures shown (see [`PanelUi::shown_3d`]).
pub(super) fn default_cam(prj: &Project, shown: &[(usize, SitePose)]) -> Orbit {
    let (mut min, mut max) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for &(k, pose) in shown {
        let e = &prj.layout.enclosures[k];
        for c in 0..8 {
            let q = pose.apply([if c & 1 == 0 { 0.0 } else { e.width }, if c & 2 == 0 { 0.0 } else { e.height }, if c & 4 == 0 { 0.0 } else { e.depth }]);
            for i in 0..3 {
                min[i] = min[i].min(q[i]);
                max[i] = max[i].max(q[i]);
            }
        }
    }
    if shown.is_empty() {
        (min, max) = ([0.0; 3], [600.0, 800.0, 250.0]);
    }
    let target = [0, 1, 2].map(|i| (min[i] + max[i]) / 2.0);
    let size = (max[0] - min[0]).max(max[1] - min[1]).max(max[2] - min[2]);
    let (yaw, pitch) = nav3d::ISO;
    Orbit { yaw, pitch, dist: size * 1.9, target }
}

/// Depth of what is seen at `q`: the nearest face, or with `items_first` (walls seen
/// through) the nearest face of an item if there is one.
fn depth_under(prims: &[Prim], q: Pos2, items_first: bool) -> Option<f32> {
    let faces = || {
        prims.iter().rev().filter_map(|prim| match prim {
            Prim::Face { pts, z, n, item, .. } if point_in_convex(q, &pts[..*n as usize]) => Some((item.is_some(), depth_at(q, &pts[..*n as usize], z))),
            _ => None,
        })
    };
    faces().find(|f| f.0 && items_first).or_else(|| faces().next()).map(|f| f.1)
}

/// Depth of the nearest solid face at `q` (not one seen through: the outer walls in
/// front, glass), or of the nearest face.
fn solid_depth_under(prims: &[Prim], q: Pos2) -> Option<f32> {
    let faces = || {
        prims.iter().filter_map(|prim| match prim {
            Prim::Face { pts, z, n, color, .. } if point_in_convex(q, &pts[..*n as usize]) => Some((color.a() == 255, depth_at(q, &pts[..*n as usize], z))),
            _ => None,
        })
    };
    let nearest = |solid: bool| faces().filter(|f| f.0 || !solid).map(|f| f.1).reduce(f32::min);
    nearest(true).or_else(|| nearest(false))
}

/// Depth at `q` of a convex face with the corners `pts` at the depths `z`.
/// The roof whose top is nearest to the eye at `ptr`, with the point there in the
/// building's floor plan.
fn roof_at(scene: &Scene3d, project_view: &dyn Fn(V3) -> Option<(Pos2, f32)>, ptr: Pos2) -> Option<RoofTarget> {
    let mut best: Option<(f32, RoofTarget)> = None;
    for r in &scene.roof_picks {
        let Some(seen) = r.pts.iter().map(|&q| project_view(q)).collect::<Option<Vec<(Pos2, f32)>>>() else { continue };
        // The triangles of the face from its first corner: the barycentric weights of the
        // pointer, corrected for the perspective.
        for i in 1..seen.len().saturating_sub(1) {
            let ((a, za), (b, zb), (c, zc)) = (seen[0], seen[i], seen[i + 1]);
            let det = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
            if det.abs() < 1e-6 {
                continue;
            }
            let u = ((ptr.x - a.x) * (c.y - a.y) - (c.x - a.x) * (ptr.y - a.y)) / det;
            let v = ((b.x - a.x) * (ptr.y - a.y) - (ptr.x - a.x) * (b.y - a.y)) / det;
            if u < 0.0 || v < 0.0 || u + v > 1.0 {
                continue;
            }
            let w = [(1.0 - u - v) / za, u / zb, v / zc];
            let sum = w[0] + w[1] + w[2];
            let depth = 1.0 / sum;
            let at: [f32; 3] = std::array::from_fn(|k| (w[0] * r.plan[0][k] + w[1] * r.plan[i][k] + w[2] * r.plan[i + 1][k]) / sum);
            if best.is_none_or(|(d, _)| depth < d) {
                best = Some((depth, RoofTarget { building: r.building, roof: r.roof, at: [at[0].round(), at[2].round()] }));
            }
            break;
        }
    }
    best.map(|b| b.1)
}

fn depth_at(q: Pos2, pts: &[Pos2], z: &[f32; 4]) -> f32 {
    for i in 1..pts.len().saturating_sub(1) {
        let (a, b, c) = (pts[0], pts[i], pts[i + 1]);
        let det = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
        if det.abs() < 1e-6 {
            continue;
        }
        let u = ((q.x - a.x) * (c.y - a.y) - (c.x - a.x) * (q.y - a.y)) / det;
        let v = ((b.x - a.x) * (q.y - a.y) - (q.x - a.x) * (b.y - a.y)) / det;
        if u >= -1e-3 && v >= -1e-3 && u + v <= 1.0 + 1e-3 {
            return z[0] + u * (z[i] - z[0]) + v * (z[i + 1] - z[0]);
        }
    }
    z[..pts.len()].iter().sum::<f32>() / pts.len().max(1) as f32
}

pub(super) fn point_in_convex(q: Pos2, pts: &[Pos2]) -> bool {
    let mut sign = 0.0f32;
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        let c = (b.x - a.x) * (q.y - a.y) - (b.y - a.y) * (q.x - a.x);
        if c.abs() < 1e-6 {
            continue;
        }
        if sign == 0.0 {
            sign = c.signum();
        } else if c.signum() != sign {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests;
