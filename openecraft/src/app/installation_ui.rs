//! The installation of a level in the Buildings pane: socket outlets, switches and
//! lights put into the rooms with clicks in the site plan, and the final circuits that
//! feed them from the protective devices of a distribution board, wired with clicks on
//! their items, with cable lengths and voltage drops (`openecraft_core::installation`).

use crate::panel_ui::site_plan::{self, SitePlan};
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
    status: String,
}

/// The parts of the installation to choose from: those of the project, then the
/// generic ones of the catalog (number, kind, description).
fn choices(prj: &Project) -> Vec<(String, inst::Kind, String)> {
    let mut out: Vec<(String, inst::Kind, String)> = Vec::new();
    for p in prj.parts.iter().chain(openecraft_core::parts::catalog()) {
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
        self.placing || self.wiring.is_some()
    }

    /// Placing items and the final circuits.
    #[allow(clippy::too_many_lines)]
    pub fn section(&mut self, ui: &mut egui::Ui, prj: &mut Project, an: &Analysis) {
        let nl = &an.netlist;
        ui.label(RichText::new(tr("Installation")).strong());
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

    /// The installation over the site plan drawn in `size` and the clicks on it.
    pub fn overlay(&mut self, ui: &egui::Ui, prj: &mut Project, resp: &egui::Response, size: egui::Rect, bi: usize, level: u64) {
        let plan = SitePlan { building: Some(bi), level, ..Default::default() };
        site_plan::paint_installation(ui, size, prj, &plan, self.wiring.or(self.selected));
        if !self.picking() {
            return;
        }
        let color = ui.visuals().selection.stroke.color;
        let hint = if self.placing { tr("Click in a room to place the part") } else { tr("Click the items of the circuit in the order its cable runs") };
        ui.painter_at(size).text(size.left_top() + vec2(8.0, 8.0), egui::Align2::LEFT_TOP, hint, egui::FontId::proportional(12.0), color);
        let Some(p) = resp.interact_pointer_pos().filter(|_| resp.clicked()) else { return };
        let frame = site_plan::frame(size, prj, &plan);
        if self.placing {
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
