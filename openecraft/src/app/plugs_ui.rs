//! Plugs window: the pin-out of a plug on the mating faces of its pin and socket side,
//! the pins in plug order with both sides, the pin count of the plug parts and unused
//! pins.

use openecraft_ui::icons::{self, Icon};
use openecraft_i18n::{tr, tr_fmt};
use super::App;
use crate::editor::{EguiDraw, Palette};
use openecraft_core::model::ReportKind;
use openecraft_core::pinout::{self, Pinout, Side};
use openecraft_core::plugs;
use openecraft_core::symbols::PinKind;
use egui::{pos2, vec2, RichText};

#[derive(Default)]
pub struct PlugsUi {
    pub open: bool,
    plug: String,
    /// Pin designation under the pointer (face or table), drawn emphasised.
    hover: Option<String>,
}

/// Width of a face in drawing millimetres (as in the SVG export).
const FACE_MM: f32 = 60.0;

fn kind_text(kinds: &[PinKind]) -> String {
    let mut v: Vec<&str> = Vec::new();
    for k in kinds {
        let t = match k {
            PinKind::Pair => tr("pin and socket"),
            PinKind::Male => tr("male"),
            PinKind::Female => tr("female"),
        };
        if !v.contains(&t) {
            v.push(t);
        }
    }
    v.join(", ")
}

/// Details of a pin seen from one side, for the tooltip.
fn pin_details(ui: &mut egui::Ui, po: &Pinout, i: usize, side: Side) {
    let fp = &po.pins[i];
    ui.label(RichText::new(tr_fmt!("{}: pin {}", po.plug, fp.label)).strong());
    if !fp.used() {
        ui.weak(tr("spare"));
        return;
    }
    let end = fp.end(side);
    egui::Grid::new("pin_details").num_columns(2).spacing([8.0, 2.0]).show(ui, |ui| {
        for (k, v) in [(tr("Potential"), &fp.potential), (side.title(), &end.targets), (tr("Wire colour"), &end.color), (tr("Cable core"), &end.core), (tr("Function text"), &fp.function_text)] {
            if !v.is_empty() {
                ui.weak(k);
                ui.label(v.as_str());
                ui.end_row();
            }
        }
    });
}

/// Draws the face of one side; returns the pin under the pointer and whether it was clicked.
fn face_view(ui: &mut egui::Ui, po: &Pinout, side: Side, width: f32, highlight: Option<usize>) -> (Option<usize>, bool) {
    let (fw, fh) = po.face.outline.size();
    let h_mm = (FACE_MM * fh / fw.max(1.0)).clamp(FACE_MM * 0.35, FACE_MM);
    let zoom = width / (FACE_MM + 8.0);
    let size = vec2(width, (h_mm + 8.0) * zoom);
    let (resp, painter) = ui.allocate_painter(size, egui::Sense::click());
    let pal = Palette::new(ui.visuals().dark_mode);
    painter.rect_filled(resp.rect, 4.0, pal.paper);
    let scale = pinout::fit_scale(po, FACE_MM, h_mm);
    let centre = openecraft_core::geom::p((FACE_MM + 8.0) / 2.0, (h_mm + 8.0) / 2.0);
    let mut d = EguiDraw { painter: &painter, origin: resp.rect.min, zoom, pal: &pal, force: None, mirror: None, textures: None, links: Vec::new() };
    pinout::draw_face(&mut d, po, side, centre, scale, highlight);
    let hovered = resp.hover_pos().and_then(|pos| {
        let q = pos2((pos.x - resp.rect.min.x) / zoom, (pos.y - resp.rect.min.y) / zoom);
        pinout::pin_points(po, side, centre, scale).into_iter().find(|(_, c, r)| (q.x - c.x).hypot(q.y - c.y) <= r * 1.25).map(|(i, _, _)| i)
    });
    let clicked = resp.clicked();
    if let Some(i) = hovered {
        resp.on_hover_ui(|ui| pin_details(ui, po, i, side));
    }
    (hovered, clicked)
}

impl App {
    pub(super) fn plugs_pane(&mut self, ui: &mut egui::Ui) {
        let tags = plugs::plug_tags(&self.session.prj);
        if !tags.contains(&self.plugs_ui.plug) {
            self.plugs_ui.plug = tags.first().cloned().unwrap_or_default();
        }
        let plug = self.plugs_ui.plug.clone();
        let rows = plugs::rows(&self.session.prj, &plug, &self.session.analysis.netlist, &self.session.analysis.open);
        let po = pinout::pinout(&self.session.prj, &plug, &self.session.analysis.netlist, &self.session.analysis.open);
        let capacity = plugs::capacity(&self.session.prj, &plug);
        let used = rows.iter().filter(|r| !r.pins.is_empty()).count();
        let highlight = self.plugs_ui.hover.as_ref().and_then(|l| po.pins.iter().position(|p| &p.label == l));
        let (mut sort, mut moved, mut jump, mut diagram, mut hover) = (false, None, None, false, None);
        ui.scope(|ui| {
            if tags.is_empty() {
                ui.weak(tr("No plugs: place plug pins (symbol group Plugs) in a schematic first."));
                return;
            }
            ui.horizontal_wrapped(|ui| {
                ui.label(tr("Plug"));
                egui::ComboBox::from_id_salt("plug").selected_text(&plug).show_ui(ui, |ui| {
                    for t in &tags {
                        ui.selectable_value(&mut self.plugs_ui.plug, t.clone(), t);
                    }
                });
                ui.separator();
                sort = ui.button(tr("Sort by designation")).clicked();
                diagram = icons::text_button(ui, Icon::Report, tr("Plug diagram")).clicked();
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(match capacity {
                    Some(n) => tr_fmt!("{} of {} pins used", used, n),
                    None => tr_fmt!("{} pins used", used),
                });
                ui.separator();
                if po.parts.is_empty() {
                    ui.weak(tr("No parts: assign the housings in the properties of a pin (parts with the function Plug pin give the pin count)."));
                } else {
                    ui.label(tr_fmt!("Parts: {}", po.parts.iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(", ")));
                }
            });
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                egui::CollapsingHeader::new(RichText::new(tr("Pin-out")).strong()).id_salt("pinout").default_open(true).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        match &po.layout {
                            Some(l) => ui.label(tr_fmt!("Pin layout: {}", l.text())),
                            None => ui.weak(tr("No pin layout in the technical data of the part: the pins are shown in a row (Pin layout: circle 4, circle 7+1, row 8, zigzag 10, grid 2x8+PE, dsub 9).")),
                        };
                        if !po.coding.is_empty() {
                            ui.separator();
                            ui.label(tr_fmt!("Coding: {}", po.coding));
                        }
                    });
                    let width = ((ui.available_width() - 12.0) / 2.0).clamp(120.0, 320.0);
                    ui.horizontal_top(|ui| {
                        for side in [Side::Pin, Side::Socket] {
                            ui.vertical(|ui| {
                                ui.set_width(width);
                                let (h, clicked) = face_view(ui, &po, side, width, highlight);
                                if let Some(i) = h {
                                    hover = Some(po.pins[i].label.clone());
                                    if clicked {
                                        jump = po.pins[i].pins.first().map(|p| (p.page, p.id));
                                    }
                                }
                                ui.vertical_centered(|ui| ui.label(side.title()));
                            });
                        }
                    });
                    ui.weak(tr("Seen on the mating faces; the colours are those of the wires and cable cores at the pins."));
                });
                ui.separator();
                egui::Grid::new("plugpins").striped(true).num_columns(9).spacing([10.0, 4.0]).show(ui, |ui| {
                    for h in ["", tr("Pin"), tr("Symbols"), tr("Socket side"), tr("Pin side"), tr("Potential"), tr("Cable core"), tr("Function text"), tr("Location")] {
                        ui.label(RichText::new(h).strong());
                    }
                    ui.end_row();
                    let placed = rows.iter().filter(|r| !r.pins.is_empty()).count();
                    for (i, r) in rows.iter().enumerate() {
                        let emphasised = self.plugs_ui.hover.as_deref() == Some(r.label.as_str());
                        let label = if emphasised { RichText::new(&r.label).monospace().strong().underline() } else { RichText::new(&r.label).monospace() };
                        if r.pins.is_empty() {
                            ui.label("");
                            if ui.label(label.weak()).hovered() {
                                hover = Some(r.label.clone());
                            }
                            ui.label("");
                            ui.weak(tr("spare"));
                            ui.end_row();
                            continue;
                        }
                        ui.horizontal(|ui| {
                            if ui.add_enabled_ui(i > 0, |ui| icons::button(ui, Icon::ChevronUp, tr("Move up"))).inner.clicked() {
                                moved = Some((i, true));
                            }
                            if ui.add_enabled_ui(i + 1 < placed, |ui| icons::button(ui, Icon::ChevronDown, tr("Move down"))).inner.clicked() {
                                moved = Some((i, false));
                            }
                        });
                        if ui.label(label).hovered() {
                            hover = Some(r.label.clone());
                        }
                        ui.label(kind_text(&r.pins.iter().map(|p| p.kind).collect::<Vec<_>>()));
                        ui.label(&r.socket);
                        ui.label(&r.pin);
                        ui.label(&r.potential);
                        let cores: Vec<&str> = po.pins.iter().filter(|p| p.label == r.label).flat_map(|p| [p.pin.core.as_str(), p.socket.core.as_str()]).filter(|c| !c.is_empty()).collect();
                        ui.label(cores.join(", "));
                        ui.label(&r.function_text);
                        if ui.link(&r.location).clicked() {
                            jump = r.pins.first().map(|p| (p.page, p.id));
                        }
                        ui.end_row();
                    }
                });
            });
        });
        self.plugs_ui.hover = hover;
        if sort {
            plugs::sort(&mut self.session.prj, &plug);
        }
        if let Some((i, up)) = moved {
            plugs::move_pin(&mut self.session.prj, &plug, i, up);
        }
        if diagram {
            self.report_kind = ReportKind::PlugDiagram.into();
            self.show_reports = true;
        }
        if let Some((page, id)) = jump {
            self.goto(page, id);
        }
    }
}
