//! The installation of a level in the Buildings pane: socket outlets, switches, lights
//! and heating elements put into the rooms with clicks in the site plan, the devices of
//! the circuits of the schematic placed there one after the other, and the final
//! circuits that feed them from the protective devices of a distribution board, wired
//! with clicks on their items, with cable lengths and voltage drops
//! (`openecraft_core::installation`).

use crate::panel_ui::site_plan;
use egui::{vec2, RichText};
use openecraft_core::installation as inst;
use openecraft_core::model::Project;
use openecraft_core::analysis::Analysis;
use openecraft_i18n::{tr, tr_fmt};
use openecraft_ui::icons::{self, Icon};

#[derive(Default)]
pub struct InstallationUi {
    /// The part to place (its number).
    part: String,
    /// Clicks in the site plan place the part.
    placing: bool,
    /// Clicks on items put them into this circuit (its id) or take them out.
    wiring: Option<u64>,
    /// The circuit highlighted (its id).
    selected: Option<u64>,
    /// The circuit of the schematic chosen (its protective device) and the device of it
    /// clicks place (its tag).
    breaker: String,
    device: Option<String>,
    /// The chosen circuit of the schematic as it is this frame.
    schematic: Option<inst::SchematicCircuit>,
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
        self.placing || self.wiring.is_some() || self.device.is_some()
    }

    /// The circuits of the schematic: one chosen, its devices listed with where they
    /// are; a click on one places it with the next clicks in the site plan.
    fn schematic_section(&mut self, ui: &mut egui::Ui, prj: &Project, nl: &openecraft_core::netlist::Netlist) {
        let list = inst::schematic_circuits(prj, nl);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr("Circuits of the schematic")).strong());
            if list.is_empty() {
                ui.weak(tr("No socket outlets, switches, lights or heating elements behind the protective devices of the schematic yet."));
                return;
            }
            if !list.iter().any(|c| c.breaker == self.breaker) {
                self.breaker = list.iter().find(|c| c.placed() < c.devices.len()).unwrap_or(&list[0]).breaker.clone();
            }
            let label = |c: &inst::SchematicCircuit| {
                let name = if c.text.is_empty() { c.breaker.clone() } else { format!("{} {}", c.breaker, c.text) };
                tr_fmt!("{} – {} of {} placed", name, c.placed(), c.devices.len())
            };
            let shown = list.iter().find(|c| c.breaker == self.breaker).map(label).unwrap_or_default();
            egui::ComboBox::from_id_salt("instscheme").selected_text(shown).width(300.0).show_ui(ui, |ui| {
                for c in &list {
                    if ui.selectable_value(&mut self.breaker, c.breaker.clone(), label(c)).on_hover_text(format!("{} {} {:.0} A", c.breaker, c.characteristic, c.rated)).clicked() {
                        self.device = None;
                    }
                }
            });
            if let Some(next) = list.iter().find(|c| c.breaker == self.breaker).and_then(|c| c.devices.iter().find(|d| d.placed.is_none())) {
                if ui.button(tr("Place all")).on_hover_text(tr("Places the devices not placed yet one after the other: a click in the site plan below for each")).clicked() {
                    self.device = Some(next.tag.clone());
                    self.placing = false;
                    self.wiring = None;
                }
            }
        });
        self.schematic = list.into_iter().find(|c| c.breaker == self.breaker);
        let Some(sc) = &self.schematic else {
            self.device = None;
            return;
        };
        if let (Some(id), true) = (sc.circuit, self.device.is_some()) {
            self.selected = Some(id);
        }
        egui::Grid::new("instschemedev").num_columns(4).striped(true).spacing([8.0, 2.0]).show(ui, |ui| {
            for d in &sc.devices {
                let on = self.device.as_deref() == Some(d.tag.as_str());
                let tip = tr("Click in a room of the site plan below to place it there (again: to move it); a click on a socket outlet, switch, light or heating element placed without a device tag makes it this device");
                if ui.selectable_label(on, d.tag.as_str()).on_hover_text(tip).clicked() {
                    self.device = if on { None } else { Some(d.tag.clone()) };
                    self.placing = false;
                    self.wiring = None;
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

    /// Placing items and the final circuits.
    #[allow(clippy::too_many_lines)]
    pub fn section(&mut self, ui: &mut egui::Ui, prj: &mut Project, an: &Analysis) {
        let nl = &an.netlist;
        ui.label(RichText::new(tr("Electrical installation")).strong());
        self.schematic_section(ui, prj, nl);
        ui.add_space(4.0);
        let parts = choices(prj);
        if self.part.is_empty() {
            self.part = parts.first().map(|p| p.0.clone()).unwrap_or_default();
        }
        ui.horizontal_wrapped(|ui| {
            let shown = parts.iter().find(|p| p.0 == self.part).map_or_else(|| self.part.clone(), |p| format!("{} – {}", p.1.title(), p.0));
            egui::ComboBox::from_id_salt("instpart").selected_text(shown).width(260.0).show_ui(ui, |ui| {
                for (number, kind, desc) in &parts {
                    ui.selectable_value(&mut self.part, number.clone(), format!("{} – {}", kind.title(), number)).on_hover_text(desc.as_str());
                }
            });
            if ui.selectable_label(self.placing, tr("Place")).on_hover_text(tr("Click in the rooms of the site plan below: a ceiling light goes where clicked, anything else on the nearest wall")).clicked() {
                self.placing = !self.placing;
                self.wiring = None;
                self.device = None;
            }
        });
        let breakers = inst::breakers(prj, nl);
        let runs = inst::runs(prj, &inst::Supply::of(prj, nl, &an.connections));
        let limit = openecraft_core::rules::limits(prj).voltage_drop;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr("Circuits")).strong());
            if icons::text_button(ui, Icon::Plus, tr("New circuit")).on_hover_text(tr("A final circuit with the first spare protective device")).clicked() {
                let spare = breakers.iter().find(|b| b.spare).map(|b| b.tag.clone()).unwrap_or_default();
                match inst::add_circuit(prj, "", &spare, false) {
                    Ok(id) => {
                        self.selected = Some(id);
                        self.wiring = Some(id);
                        self.placing = false;
                        self.device = None;
                    }
                    Err(e) => self.status = e,
                }
            }
            if !prj.layout.circuits.is_empty() && ui.button(tr("Write cable lengths")).on_hover_text(tr("Writes the cable of every circuit into the cable list with its type, cross-section and length")).clicked() {
                self.status = tr_fmt!("{} cables written", inst::write_cables(prj));
            }
            if !prj.layout.circuits.is_empty() && ui.button(tr("Single-line page")).on_hover_text(tr("Gives the items device tags and draws the circuits as single-line diagrams on overview pages (made again each time)")).clicked() {
                self.status = match inst::overview(prj) {
                    Ok(pages) => tr_fmt!("Single-line diagram of the circuits on page {}", pages.iter().map(|&k| prj.pages[k].name.clone()).collect::<Vec<_>>().join(", ")),
                    Err(e) => e,
                };
            }
        });
        if prj.layout.circuits.is_empty() {
            ui.weak(tr("No circuit yet: a circuit feeds socket outlets and lights from a protective device of the distribution board (single-line or schematic)."));
        }
        let mut remove = None;
        let mut planned = None;
        egui::Grid::new("instcircuits").num_columns(8).striped(true).spacing([8.0, 4.0]).show(ui, |ui| {
            if !prj.layout.circuits.is_empty() {
                for h in [tr("Name"), tr("Protective device"), tr("Cross-section"), tr("Items"), tr("Length"), tr("Voltage drop"), tr("From the supply"), ""] {
                    ui.label(RichText::new(h).weak());
                }
                ui.end_row();
            }
            for (k, r) in runs.iter().enumerate() {
                let c = &mut prj.layout.circuits[k];
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 4.0, site_plan::circuit_color(k));
                    if ui.add(egui::TextEdit::singleline(&mut c.name).desired_width(110.0)).gained_focus() {
                        self.selected = Some(c.id);
                    }
                });
                let shown = if c.breaker.is_empty() { tr("None").to_string() } else { c.breaker.clone() };
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt(("instbreaker", c.id)).selected_text(shown).width(110.0).show_ui(ui, |ui| {
                        ui.selectable_value(&mut c.breaker, String::new(), tr("None"));
                        for b in &breakers {
                            let text = if b.spare { tr_fmt!("{} {} {} A (spare)", b.tag, b.characteristic, format!("{:.0}", b.rated)) } else { format!("{} {} {:.0} A", b.tag, b.characteristic, b.rated) };
                            ui.selectable_value(&mut c.breaker, b.tag.clone(), text);
                        }
                    });
                    if icons::button(ui, Icon::Plus, tr("A new planned circuit breaker of 16 A for the circuit, drawn in the single-line diagram")).clicked() {
                        planned = Some(k);
                    }
                });
                let mut mm2 = inst::cross_section(c);
                if ui.add(egui::DragValue::new(&mut mm2).range(0.5..=50.0).speed(0.1).suffix(" mm²")).changed() {
                    c.cross_section = mm2;
                }
                ui.label(r.items.len().to_string());
                ui.label(format!("{:.1} m", r.length));
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
                ui.horizontal(|ui| {
                    let wiring = self.wiring == Some(c.id);
                    if ui.selectable_label(wiring, tr("Wire")).on_hover_text(tr("Click socket outlets, switches and lights in the site plan below in the order the cable runs to them; a click on one of the circuit takes it out")).clicked() {
                        self.wiring = if wiring { None } else { Some(c.id) };
                        self.selected = Some(c.id);
                        self.placing = false;
                        self.device = None;
                    }
                    if icons::button(ui, Icon::Delete, tr("Remove the circuit; its items stay")).clicked() {
                        remove = Some(k);
                    }
                });
                ui.end_row();
            }
        });
        if let Some(k) = planned {
            match inst::new_breaker(prj, k, 16.0) {
                Ok(tag) => self.status = tr_fmt!("{} planned: draw the single-line page, then make it in the schematic from it", tag),
                Err(e) => self.status = e,
            }
        }
        if let Some(k) = remove {
            let id = prj.layout.circuits.remove(k).id;
            if self.wiring == Some(id) {
                self.wiring = None;
            }
        }
        if !self.status.is_empty() {
            ui.weak(self.status.as_str());
        }
    }

    /// A click at `p` for device `tag` of the schematic: an item placed there without a
    /// device tag becomes the device, otherwise the device is placed where clicked
    /// (with the part chosen when it has none and that part is of its kind); then on to
    /// the next device not placed yet.
    fn place_device(&mut self, prj: &mut Project, sc: &inst::SchematicCircuit, tag: &str, plan: &site_plan::SitePlan, frame: &site_plan::Frame, p: egui::Pos2) {
        let free = site_plan::installation_at(prj, plan, frame, p).filter(|id| prj.layout.enclosures.iter().filter_map(|e| e.item(*id)).any(|it| it.tag.is_empty()));
        let done = match (free, site_plan::room_at(prj, plan, frame, p)) {
            (Some(item), _) => inst::link_device(prj, sc, tag, item),
            (None, Some(ri)) => {
                let chosen = choices(prj).iter().any(|c| c.0 == self.part && sc.device(tag).is_some_and(|d| d.kind == c.1));
                inst::place_device(prj, sc, tag, ri, if chosen { self.part.as_str() } else { "" }, frame.plant(p)).map(|r| r.1)
            }
            (None, None) => Err(tr("Click in a room").to_string()),
        };
        match done {
            Ok(ci) => {
                self.status.clear();
                self.selected = Some(prj.layout.circuits[ci].id);
                let at = sc.devices.iter().position(|d| d.tag == tag).unwrap_or(0);
                self.device = sc.devices.iter().skip(at + 1).chain(sc.devices.iter().take(at)).find(|d| d.placed.is_none() && d.tag != tag).map(|d| d.tag.clone());
            }
            Err(e) => self.status = e,
        }
    }

    /// The installation over the site plan drawn in `size` and the clicks on it.
    pub fn overlay(&mut self, ui: &egui::Ui, prj: &mut Project, resp: &egui::Response, size: egui::Rect, bi: usize, level: u64) {
        let plan = site_plan::level_plan(prj, bi, level);
        site_plan::paint_installation(ui, size, prj, &plan, self.wiring.or(self.selected));
        if !self.picking() {
            return;
        }
        let color = ui.visuals().selection.stroke.color;
        let hint = match &self.device {
            Some(tag) => tr_fmt!("Click in a room to place {}", tag),
            None if self.placing => tr("Click in a room to place the part").to_string(),
            None => tr("Click the items of the circuit in the order its cable runs").to_string(),
        };
        ui.painter_at(size).text(size.left_top() + vec2(8.0, 8.0), egui::Align2::LEFT_TOP, hint, egui::FontId::proportional(12.0), color);
        let Some(p) = resp.interact_pointer_pos().filter(|_| resp.clicked()) else { return };
        let frame = site_plan::frame(size, prj, &plan);
        if let (Some(tag), Some(sc)) = (self.device.clone(), self.schematic.clone()) {
            self.place_device(prj, &sc, &tag, &plan, &frame, p);
        } else if self.placing {
            let Some(ri) = site_plan::room_at(prj, &plan, &frame, p) else {
                self.status = tr("Click in a room").to_string();
                return;
            };
            match inst::place(prj, ri, &self.part, frame.plant(p)) {
                Ok(_) => self.status.clear(),
                Err(e) => self.status = e,
            }
        } else if let Some(id) = self.wiring {
            let Some(item) = site_plan::installation_at(prj, &plan, &frame, p) else { return };
            let Some(ci) = prj.layout.circuits.iter().position(|c| c.id == id) else { return };
            if prj.layout.circuits[ci].items.contains(&item) {
                prj.layout.circuits[ci].items.retain(|i| *i != item);
            } else if let Err(e) = inst::connect(prj, ci, item) {
                self.status = e;
            }
        }
    }
}
