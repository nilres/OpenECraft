//! The installation of a level in the Buildings pane: socket outlets, switches, lights
//! and heating elements put into the rooms with clicks in the site plan for a protective
//! device of the schematic (they go into the schematic with their cables), the devices
//! of the circuits of the schematic placed there one after the other, and the circuits
//! of the schematic with their cables, lengths (3D routing) and voltage drops, read only
//! (`openecraft_core::installation`).

use crate::panel_ui::site_plan;
use egui::{vec2, RichText};
use openecraft_core::analysis::Analysis;
use openecraft_core::installation as inst;
use openecraft_core::model::Project;
use openecraft_i18n::{tr, tr_fmt};

#[derive(Default)]
pub struct InstallationUi {
    /// The part to put in (its number).
    part: String,
    /// The protective device the devices are put in for and whose circuit is shown
    /// (its tag).
    breaker: String,
    /// The device of the circuit the cable of the next device comes from (empty: the
    /// one before it).
    from: String,
    /// Clicks in the site plan put the part in.
    placing: bool,
    /// The device of the schematic clicks place (its tag).
    device: Option<String>,
    /// The circuits of the schematic as they are this frame.
    circuits: Vec<inst::Circuit>,
    status: String,
}

/// The parts of the installation to choose from: those of the project, the generic
/// ones of the catalog, then those of the bundled manufacturer catalogs (number, kind,
/// description).
fn choices(prj: &Project) -> Vec<(String, inst::Kind, String)> {
    let mut out: Vec<(String, inst::Kind, String)> = Vec::new();
    for p in prj.parts.iter().chain(openecraft_core::parts::catalog()).chain(inst::bundled_parts()) {
        if let Some(k) = inst::kind_of(p) {
            if !out.iter().any(|c| c.0 == p.number) {
                out.push((p.number.clone(), k, p.description.clone()));
            }
        }
    }
    out
}

impl InstallationUi {
    /// Clicks in the site plan are for this.
    pub fn picking(&self) -> bool {
        self.placing || self.device.is_some()
    }

    /// The circuit shown.
    fn circuit(&self) -> Option<&inst::Circuit> {
        self.circuits.iter().find(|c| c.breaker == self.breaker)
    }

    /// The circuits of the schematic, with the 3D routing of `an`.
    fn refresh(&mut self, prj: &Project, an: &Analysis) {
        self.circuits = inst::circuits(prj, &an.netlist, &an.connections, Some(&|k: &str| an.routing.routed_length(k)));
    }

    /// Putting devices in for a protective device, the devices of its circuit and the
    /// circuits of the schematic.
    pub fn section(&mut self, ui: &mut egui::Ui, prj: &mut Project, an: &Analysis) {
        let nl = &an.netlist;
        self.refresh(prj, an);
        ui.label(RichText::new(tr("Electrical installation")).strong());
        let breakers = inst::breakers(prj, nl);
        if breakers.is_empty() {
            ui.weak(tr("No protective devices of final circuits in the schematic yet: draw them in the schematic of the distribution board first."));
            self.placing = false;
            self.device = None;
        } else {
            if !breakers.iter().any(|b| b.tag == self.breaker) {
                self.breaker = breakers[0].tag.clone();
            }
            let parts = choices(prj);
            if self.part.is_empty() {
                self.part = parts.first().map(|p| p.0.clone()).unwrap_or_default();
            }
            ui.horizontal_wrapped(|ui| {
                ui.label(tr("Protective device"));
                let label = |b: &inst::Breaker| if b.spare { tr_fmt!("{} (spare)", b.title()) } else { b.title() };
                let shown = breakers.iter().find(|b| b.tag == self.breaker).map(label).unwrap_or_default();
                egui::ComboBox::from_id_salt("instbreaker").selected_text(shown).width(220.0).show_ui(ui, |ui| {
                    for b in &breakers {
                        if ui.selectable_value(&mut self.breaker, b.tag.clone(), label(b)).on_hover_text(format!("{} {} {:.0} A", b.tag, b.characteristic, b.rated)).clicked() {
                            self.device = None;
                        }
                    }
                });
                let shown = parts.iter().find(|p| p.0 == self.part).map_or_else(|| self.part.clone(), |p| format!("{} – {}", p.1.title(), p.0));
                egui::ComboBox::from_id_salt("instpart").selected_text(shown).width(240.0).show_ui(ui, |ui| {
                    for (number, kind, desc) in &parts {
                        ui.selectable_value(&mut self.part, number.clone(), format!("{} – {}", kind.title(), number)).on_hover_text(desc.as_str());
                    }
                });
                // Where the cable comes from: a device of the circuit (a junction box) or the
                // one before it.
                let devices: Vec<String> = self.circuit().map(|c| c.devices.iter().map(|d| d.tag.clone()).collect()).unwrap_or_default();
                if !self.from.is_empty() && self.from != self.breaker && !devices.contains(&self.from) {
                    self.from.clear();
                }
                let before = tr("the device before it");
                let shown = if self.from.is_empty() { before.to_string() } else { self.from.clone() };
                ui.label(tr("Cable from"));
                egui::ComboBox::from_id_salt("instfrom").selected_text(shown).width(150.0).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.from, String::new(), before);
                    for t in std::iter::once(&self.breaker).chain(&devices) {
                        ui.selectable_value(&mut self.from, t.clone(), t.as_str());
                    }
                })
                .response
                .on_hover_text(tr("The device or junction box of the circuit the cable of the next device comes from: a circuit branches there"));
                let tip = tr("Click in the rooms of the site plan below: the part goes into the room (a ceiling light where clicked, anything else on the nearest wall) and into the schematic, on the installation page of the protective device with a cable from the device before it");
                if ui.selectable_label(self.placing, tr("Put in")).on_hover_text(tip).clicked() {
                    self.placing = !self.placing;
                    self.device = None;
                }
            });
        }
        self.devices_section(ui, prj);
        ui.add_space(4.0);
        self.circuits_section(ui, prj, an);
        if !self.status.is_empty() {
            ui.weak(self.status.as_str());
        }
    }

    /// The devices of the circuit shown, with where they are placed; a click on one
    /// places it with the next clicks in the site plan.
    fn devices_section(&mut self, ui: &mut egui::Ui, prj: &Project) {
        let Some(sc) = self.circuit().cloned() else {
            self.device = None;
            return;
        };
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr_fmt!("Devices of {}", sc.title())).strong());
            if let Some(page) = sc.page {
                ui.weak(tr_fmt!("Installation page {}", prj.pages[page].name));
            }
            if let Some(next) = sc.devices.iter().find(|d| d.placed.is_none()) {
                if ui.button(tr("Place all")).on_hover_text(tr("Places the devices not placed yet one after the other: a click in the site plan below for each")).clicked() {
                    self.device = Some(next.tag.clone());
                    self.placing = false;
                }
            }
        });
        egui::Grid::new("instschemedev").num_columns(4).striped(true).spacing([8.0, 2.0]).show(ui, |ui| {
            for d in &sc.devices {
                let on = self.device.as_deref() == Some(d.tag.as_str());
                let tip = tr("Click in a room of the site plan below to place it there (again: to move it); a click on a socket outlet, switch, light or heating element placed without a device tag makes it this device");
                if ui.selectable_label(on, d.tag.as_str()).on_hover_text(tip).clicked() {
                    self.device = if on { None } else { Some(d.tag.clone()) };
                    self.placing = false;
                }
                ui.label(d.kind.title());
                match &d.part {
                    Some(p) => ui.label(p.as_str()),
                    None => ui.weak(d.kind.default_part()).on_hover_text(tr("No part yet: placed with this one, which becomes its part")),
                };
                match d.placed {
                    Some((ri, _)) => ui.label(prj.layout.enclosures[ri].name.as_str()),
                    None => ui.weak(tr("Not placed")),
                };
                ui.end_row();
            }
        });
        if self.device.as_ref().is_some_and(|t| sc.device(t).is_none()) {
            self.device = None;
        }
    }

    /// The circuits of the schematic, read only: what they show comes from the schematic
    /// and the 3D routing.
    fn circuits_section(&mut self, ui: &mut egui::Ui, prj: &mut Project, an: &Analysis) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr("Circuits of the schematic")).strong());
            let tip = tr("Writes the lengths of the cables of the circuits (of their 3D routes, estimated where not routed yet) into the cable list");
            if self.circuits.iter().any(|c| !c.cables().is_empty()) && ui.button(tr("Write cable lengths")).on_hover_text(tip).clicked() {
                self.status = tr_fmt!("{} cables written", inst::write_cables(prj, &self.circuits));
            }
        });
        if self.circuits.is_empty() {
            ui.weak(tr("No socket outlets, switches, lights or heating elements behind the protective devices of the schematic yet."));
            return;
        }
        ui.weak(tr("From the schematic: the devices behind each protective device, their connections and cables; the lengths of the 3D routes."));
        let supply = inst::Supply::of(prj, &an.netlist, &an.connections);
        let limit = openecraft_core::rules::limits(prj).voltage_drop;
        egui::Grid::new("instcircuits").num_columns(6).striped(true).spacing([8.0, 4.0]).show(ui, |ui| {
            for h in [tr("Circuit"), tr("Devices"), tr("Cables"), tr("Length"), tr("Voltage drop"), tr("From the supply")] {
                ui.label(RichText::new(h).weak());
            }
            ui.end_row();
            for (k, c) in self.circuits.iter().enumerate() {
                let r = inst::run(prj, &supply, c);
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 4.0, site_plan::circuit_color(k));
                    if ui.selectable_label(self.breaker == c.breaker, c.title()).on_hover_text(format!("{} {:.0} A", c.characteristic, c.rated)).clicked() {
                        self.breaker = c.breaker.clone();
                        self.device = None;
                    }
                });
                ui.label(format!("{}/{}", c.placed(), c.devices.len()));
                let cables = c.cables();
                if cables.is_empty() {
                    ui.weak("–");
                } else {
                    let types: Vec<String> = c.legs.iter().filter(|l| !l.cable.is_empty()).map(|l| format!("{} {} ({})", l.cable, l.cable_type, l.length.map_or_else(|| "–".to_string(), |m| format!("{m:.1} m")))).collect();
                    ui.label(cables.join(", ")).on_hover_text(types.join("\n"));
                }
                if r.routed {
                    ui.label(format!("{:.1} m", r.length)).on_hover_text(tr("From the 3D routing"));
                } else {
                    ui.label(format!("~{:.1} m", r.length)).on_hover_text(tr("Estimated along the installation zones: not all connections are routed in 3D yet"));
                }
                match r.drop {
                    Some((_, pct)) => ui.label(format!("{pct:.2} %")),
                    None => ui.weak("–"),
                };
                let feeder = r.upstream.map_or_else(|| tr("No feeder cables with lengths up to the board").to_string(), |u| tr_fmt!("Of it the feeder cables up to the board: {} %", format!("{u:.2}")));
                match r.total() {
                    Some(pct) if pct > limit => ui.colored_label(crate::style::warning(), format!("{pct:.2} %")).on_hover_text(feeder),
                    Some(pct) => ui.label(format!("{pct:.2} %")).on_hover_text(feeder),
                    None => ui.weak("–"),
                };
                ui.end_row();
            }
        });
    }

    /// A click at `p` for device `tag` of the schematic: an item placed there without a
    /// device tag becomes the device, otherwise the device is placed where clicked
    /// (with the part chosen when it has none and that part is of its kind); then on to
    /// the next device not placed yet.
    fn place_device(&mut self, prj: &mut Project, sc: &inst::Circuit, tag: &str, plan: &site_plan::SitePlan, frame: &site_plan::Frame, p: egui::Pos2) {
        let free = site_plan::installation_at(prj, plan, frame, p).filter(|id| prj.layout.enclosures.iter().filter_map(|e| e.item(*id)).any(|it| it.tag.is_empty()));
        let done = match (free, site_plan::room_at(prj, plan, frame, p)) {
            (Some(item), _) => inst::link_device(prj, sc, tag, item),
            (None, Some(ri)) => {
                let chosen = choices(prj).iter().any(|c| c.0 == self.part && sc.device(tag).is_some_and(|d| d.kind == c.1));
                inst::place_device(prj, sc, tag, ri, if chosen { self.part.as_str() } else { "" }, frame.plant(p)).map(|_| ())
            }
            (None, None) => Err(tr("Click in a room").to_string()),
        };
        match done {
            Ok(()) => {
                self.status.clear();
                let at = sc.devices.iter().position(|d| d.tag == tag).unwrap_or(0);
                self.device = sc.devices.iter().skip(at + 1).chain(sc.devices.iter().take(at)).find(|d| d.placed.is_none() && d.tag != tag).map(|d| d.tag.clone());
            }
            Err(e) => self.status = e,
        }
    }

    /// The installation over the site plan drawn in `size`, and the clicks on it.
    pub fn overlay(&mut self, ui: &egui::Ui, prj: &mut Project, resp: &egui::Response, size: egui::Rect, bi: usize, level: u64) {
        let plan = site_plan::level_plan(prj, bi, level);
        site_plan::paint_installation(ui, size, prj, &plan, &self.circuits, Some(self.breaker.as_str()));
        if !self.picking() {
            return;
        }
        let color = ui.visuals().selection.stroke.color;
        let hint = match &self.device {
            Some(tag) => tr_fmt!("Click in a room to place {}", tag),
            None => tr_fmt!("Click in a room to put the part in for {}", self.breaker),
        };
        ui.painter_at(size).text(size.left_top() + vec2(8.0, 8.0), egui::Align2::LEFT_TOP, hint, egui::FontId::proportional(12.0), color);
        let Some(p) = resp.interact_pointer_pos().filter(|_| resp.clicked()) else { return };
        let frame = site_plan::frame(size, prj, &plan);
        if let (Some(tag), Some(sc)) = (self.device.clone(), self.circuit().cloned()) {
            self.place_device(prj, &sc, &tag, &plan, &frame, p);
        } else if self.placing {
            let Some(ri) = site_plan::room_at(prj, &plan, &frame, p) else {
                self.status = tr("Click in a room").to_string();
                return;
            };
            // Data and antenna outlets and smoke alarms only into the room.
            if !inst::find_part(prj, &self.part).and_then(inst::kind_of).is_none_or(inst::Kind::mains) {
                self.status = match inst::place(prj, ri, &self.part, frame.plant(p)) {
                    Ok(_) => tr("Placed").to_string(),
                    Err(e) => e,
                };
                return;
            }
            let opts = inst::Options { from: Some(self.from.clone()).filter(|f| !f.is_empty()), ..Default::default() };
            self.status = match inst::add_device(prj, &self.breaker, &self.part, ri, frame.plant(p), &opts) {
                Ok(tag) => tr_fmt!("{} is in the schematic", tag),
                Err(e) => e,
            };
        }
    }

    /// The installation over the site plan while its clicks are for something else.
    pub fn paint(&mut self, ui: &egui::Ui, prj: &Project, an: &Analysis, size: egui::Rect, bi: usize, level: u64) {
        let plan = site_plan::level_plan(prj, bi, level);
        self.refresh(prj, an);
        site_plan::paint_installation(ui, size, prj, &plan, &self.circuits, None);
    }
}
