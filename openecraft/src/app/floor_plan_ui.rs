//! Floor plans on paper under the levels of a building, in the Buildings pane: a scan
//! or drawing (PDF, picture) put under a level, its scale (or measured on two points of
//! a known distance), place and area, the walls, doors, windows and rooms found in it
//! and the rooms made of them (`openecraft_core::panel::floorplan`).

use crate::panel_ui::site_plan;
use egui::{vec2, Pos2, RichText, Sense, Stroke};
use openecraft_core::model::Project;
use openecraft_core::panel::{buildings as bld, floorplan, FloorPlan};
use openecraft_core::wallscan::{GapKind, Scan};
use openecraft_i18n::{tr, tr_fmt};
use openecraft_scan::Sheet;
use openecraft_ui::icons::{self, Icon};

/// What clicks in the site plan pick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Pick {
    #[default]
    None,
    /// Two points of a known distance: the scale.
    Calibrate,
    /// Two corners of the part the rooms are found in.
    Area,
}

#[derive(Default)]
pub struct FloorPlanUi {
    /// The pages of a file read for a level (its id), one to choose.
    pending: Option<(u64, String, Vec<Sheet>)>,
    page: usize,
    /// What was found in the floor plan of a level (its id) as it was then.
    found: Option<(u64, FloorPlan, Scan)>,
    pick: Pick,
    /// The points picked (pixels of the picture).
    points: Vec<[f32; 2]>,
    /// Distance of the two points of a measure, mm.
    distance: f32,
    status: String,
}

fn level_name(prj: &Project, id: u64) -> String {
    bld::level_by_id(&prj.layout, id).map(|l| l.name.clone()).unwrap_or_default()
}

impl FloorPlanUi {
    /// The sheets of a file read for level `level`: a single one goes under it, of
    /// several the page is chosen in the pane. Gives the message to show.
    pub fn read(&mut self, prj: &mut Project, level: u64, name: String, mut sheets: Vec<Sheet>) -> String {
        if sheets.len() == 1 {
            let sheet = sheets.remove(0);
            return self.put(prj, level, &name, &sheet);
        }
        let n = sheets.len();
        self.pending = Some((level, name.clone(), sheets));
        self.page = 0;
        tr_fmt!("{} has {} pages: choose one in the Buildings pane", name, n)
    }

    fn put(&mut self, prj: &mut Project, level: u64, name: &str, sheet: &Sheet) -> String {
        // A plan put again keeps its scale, place and turn.
        let mut plan = floorplan::new_plan(name, &sheet.img, sheet.paper);
        if let Some(old) = floorplan::plan_of(&prj.layout, level) {
            (plan.scale, plan.x, plan.z, plan.angle, plan.opacity) = (old.scale, old.x, old.z, old.angle, old.opacity);
        }
        self.found = None;
        self.pick = Pick::None;
        match floorplan::set_plan(prj, level, Some(plan)) {
            Ok(()) => tr_fmt!("Floor plan {} put under level {}", name, level_name(prj, level)),
            Err(e) => e,
        }
    }

    /// The floor plan of level `level` with its settings and actions; gives whether a
    /// file is to be opened for it.
    #[allow(clippy::too_many_lines)]
    pub fn section(&mut self, ui: &mut egui::Ui, prj: &mut Project, level: u64) -> bool {
        let mut open = false;
        ui.label(RichText::new(tr("Floor plan on paper")).strong());
        if let Some((l, name, sheets)) = &self.pending {
            if *l == level {
                let (name, n) = (name.clone(), sheets.len());
                let mut chosen = None;
                let mut cancel = false;
                ui.horizontal(|ui| {
                    ui.label(name.as_str());
                    let mut page = self.page + 1;
                    ui.add(egui::DragValue::new(&mut page).range(1..=n).prefix(format!("{} ", tr("Page"))));
                    self.page = page - 1;
                    ui.weak(tr_fmt!("of {}", n));
                    if ui.button(tr("Put under the level")).clicked() {
                        chosen = Some(self.page);
                    }
                    if ui.button(tr("Cancel")).clicked() {
                        cancel = true;
                    }
                });
                if let Some(k) = chosen {
                    if let Some((_, _, sheets)) = self.pending.take() {
                        self.status = self.put(prj, level, &name, &sheets[k]);
                    }
                } else if cancel {
                    self.pending = None;
                }
            }
        }
        let Some(fp) = floorplan::plan_of(&prj.layout, level).cloned() else {
            ui.horizontal_wrapped(|ui| {
                if icons::text_button(ui, Icon::FolderOpen, tr("Floor plan…")).clicked() {
                    open = true;
                }
                ui.weak(tr("A scan or drawing (PDF, PNG, JPEG) under the level: the rooms are drawn on it or found in it."));
            });
            return open;
        };
        let mut plan = fp.clone();
        egui::Grid::new("bldfloorplan").num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
            ui.label(tr("File"));
            ui.horizontal(|ui| {
                ui.label(plan.image.name.as_str());
                ui.weak(tr_fmt!("{} × {} pixels", plan.px[0], plan.px[1]));
                if icons::button(ui, Icon::FolderOpen, tr("Put another file under the level (scale and place stay)")).clicked() {
                    open = true;
                }
            });
            ui.end_row();
            ui.label(tr("Scale"));
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut plan.scale).range(1.0..=10_000.0).speed(1.0).prefix("1 : "));
                let measuring = self.pick == Pick::Calibrate;
                if ui.selectable_label(measuring, tr("Measure")).on_hover_text(tr("Click two points of a known distance in the site plan below, then give the distance")).clicked() {
                    self.pick = if measuring { Pick::None } else { Pick::Calibrate };
                    self.points.clear();
                }
                ui.weak(tr_fmt!("{} mm per pixel", format!("{:.2}", plan.mm_per_px())));
            });
            ui.end_row();
            ui.label(tr("Resolution"));
            let mut dpi = 25.4 / plan.paper;
            if ui.add(egui::DragValue::new(&mut dpi).range(10.0..=2400.0).speed(1.0).suffix(" dpi")).changed() {
                plan.paper = floorplan::paper_of_dpi(dpi);
            }
            ui.end_row();
            ui.label(tr("Top left corner"));
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut plan.x).speed(50.0).prefix("x ").suffix(" mm"));
                ui.add(egui::DragValue::new(&mut plan.z).speed(50.0).prefix("z ").suffix(" mm"));
            });
            ui.end_row();
            ui.label(tr("Turn"));
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut plan.angle).speed(0.5).suffix("°"));
                if icons::button(ui, Icon::Rotate, tr("Turn a quarter counter-clockwise")).clicked() {
                    plan.angle = (plan.angle + 90.0).rem_euclid(360.0);
                }
            });
            ui.end_row();
            ui.label(tr("Shown"));
            ui.add(egui::Slider::new(&mut plan.opacity, 0.0..=1.0).show_value(false));
            ui.end_row();
            ui.label(tr("Area"));
            ui.horizontal(|ui| {
                match plan.area {
                    Some([l, t, r, b]) => ui.label(format!("{l}, {t} … {r}, {b}")),
                    None => ui.weak(tr("All of the picture")),
                };
                let picking = self.pick == Pick::Area;
                if ui.selectable_label(picking, tr("Pick")).on_hover_text(tr("Click two corners of the part with the floor plan in the site plan below (leaving out title block, sections and other drawings)")).clicked() {
                    self.pick = if picking { Pick::None } else { Pick::Area };
                    self.points.clear();
                }
                if plan.area.is_some() && ui.button(tr("All")).clicked() {
                    plan.area = None;
                }
            });
            ui.end_row();
        });
        if self.pick == Pick::Calibrate && self.points.len() == 2 {
            ui.horizontal(|ui| {
                ui.label(tr("Distance of the two points"));
                ui.add(egui::DragValue::new(&mut self.distance).range(0.0..=1_000_000.0).speed(10.0).suffix(" mm"));
                if ui.button(tr("Set scale")).clicked() {
                    match floorplan::calibrated(&plan, self.points[0], self.points[1], self.distance) {
                        Ok(scale) => {
                            plan.scale = scale;
                            self.pick = Pick::None;
                            self.points.clear();
                        }
                        Err(e) => self.status = e,
                    }
                }
            });
        }
        if plan != fp {
            match floorplan::set_plan(prj, level, Some(plan.clone())) {
                Ok(()) => self.status.clear(),
                Err(e) => self.status = e,
            }
        }
        // Found in the plan as it is now.
        let current = self.found.as_ref().filter(|(l, p, _)| *l == level && *p == plan).map(|f| &f.2);
        let mut find = false;
        let mut make = false;
        let mut remove = false;
        ui.horizontal_wrapped(|ui| {
            if ui.button(tr("Find rooms")).on_hover_text(tr("Finds the walls, doors, windows and rooms in the floor plan and shows them in the site plan")).clicked() {
                find = true;
            }
            if ui.add_enabled(current.is_some_and(|f| !f.rooms.is_empty()), egui::Button::new(tr("Make rooms"))).on_hover_text(tr("Makes rooms on the level of the rooms found, with their doors and windows")).clicked() {
                make = true;
            }
            if icons::button(ui, Icon::Delete, tr("Remove the floor plan; the rooms stay")).clicked() {
                remove = true;
            }
            if let Some(f) = current {
                let doors = f.openings.iter().filter(|g| g.kind == GapKind::Door).count();
                ui.weak(tr_fmt!("Found: {} rooms, {} doors, {} windows", f.rooms.len(), doors, f.openings.len() - doors));
            }
        });
        if find {
            match floorplan::detect(&plan) {
                Ok(scan) => {
                    self.status = if scan.rooms.is_empty() { tr("No rooms found: check the scale and the area").to_string() } else { String::new() };
                    self.found = Some((level, plan.clone(), scan));
                }
                Err(e) => self.status = e,
            }
        }
        if make {
            if let Some((_, p, scan)) = self.found.take() {
                match floorplan::trace_scan(prj, level, &p, &scan) {
                    Ok(t) => {
                        self.status = tr_fmt!("{} rooms made with {} doors and {} windows", t.rooms.len(), t.doors, t.windows);
                        if !t.skipped.is_empty() {
                            self.status = format!("{} ({})", self.status, tr_fmt!("{} rooms not made: {}", t.skipped.len(), t.skipped.join("; ")));
                        }
                    }
                    Err(e) => self.status = e,
                }
            }
        }
        if remove {
            let _ = floorplan::set_plan(prj, level, None);
            self.found = None;
            self.pick = Pick::None;
        }
        if !self.status.is_empty() {
            ui.weak(self.status.as_str());
        }
        open
    }

    /// The site plan of building `bi` with level `level` highlighted, its floor plan
    /// under it, what was found in it and the points picked; clickable also when
    /// `click` (for the installation). Gives its response and rectangle.
    pub fn site_plan(&mut self, ui: &mut egui::Ui, prj: &Project, bi: usize, level: u64, click: bool) -> (egui::Response, egui::Rect) {
        let fp = floorplan::plan_of(&prj.layout, level).filter(|_| level != 0).cloned();
        let picking = fp.is_some() && self.pick != Pick::None;
        let (size, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 360.0), if picking || click { Sense::click() } else { Sense::hover() });
        let plan = site_plan::level_plan(prj, bi, level);
        site_plan::paint(ui, size, prj, &plan, None);
        let Some(fp) = fp else { return (resp, size) };
        if let Some((_, _, scan)) = self.found.as_ref().filter(|(l, p, _)| *l == level && *p == fp) {
            site_plan::paint_scan(ui, size, prj, &plan, bi, &fp, scan);
        }
        if !picking {
            return (resp, size);
        }
        let frame = site_plan::frame(size, prj, &plan);
        if resp.clicked() {
            if let Some(q) = resp.interact_pointer_pos().and_then(|p| site_plan::picture_point(prj, bi, &fp, &frame, p)) {
                let q = [q[0].clamp(0.0, fp.px[0] as f32), q[1].clamp(0.0, fp.px[1] as f32)];
                if self.points.len() >= 2 {
                    self.points.clear();
                }
                self.points.push(q);
            }
        }
        let pts: Vec<Pos2> = self.points.iter().map(|&q| site_plan::picture_on_screen(prj, bi, &fp, &frame, q)).collect();
        let color = ui.visuals().selection.stroke.color;
        let painter = ui.painter_at(size);
        for p in &pts {
            painter.circle_stroke(*p, 4.0, Stroke::new(2.0, color));
        }
        if let [a, b] = pts[..] {
            match self.pick {
                Pick::Calibrate => {
                    painter.line_segment([a, b], Stroke::new(1.5, color));
                }
                Pick::Area => {
                    painter.rect_stroke(egui::Rect::from_two_pos(a, b), 0.0, Stroke::new(1.5, color), egui::StrokeKind::Middle);
                }
                Pick::None => {}
            }
        }
        let hint = match (self.pick, self.points.len()) {
            (Pick::Calibrate, 2) => tr("Give the distance of the two points above"),
            (Pick::Calibrate, _) => tr("Click two points of a known distance (a dimension of the drawing)"),
            (_, _) => tr("Click two corners of the part with the floor plan"),
        };
        painter.text(size.left_top() + vec2(8.0, 8.0), egui::Align2::LEFT_TOP, hint, egui::FontId::proportional(12.0), color);
        (resp, size)
    }

    /// Clicks in the site plan pick no more points (another tab is shown).
    pub fn stop_picking(&mut self) {
        self.pick = Pick::None;
        self.points.clear();
    }

    /// The area picked, once both corners are: taken into the plan of `level`.
    pub fn take_area(&mut self, prj: &mut Project, level: u64) {
        if self.pick != Pick::Area || self.points.len() != 2 {
            return;
        }
        let [a, b] = [self.points[0], self.points[1]];
        let area = [a[0].min(b[0]), a[1].min(b[1]), a[0].max(b[0]), a[1].max(b[1])].map(|v| v.round() as u32);
        if let Err(e) = floorplan::edit_plan(prj, level, |p| p.area = Some(area)) {
            self.status = e;
        }
        self.pick = Pick::None;
        self.points.clear();
    }
}
