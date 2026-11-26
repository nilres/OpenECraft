//! New enclosures: the dialog that searches the enclosure parts (text, enclosure system,
//! manufacturer, mounting, size, field widths and rows) and enclosures made of parts
//! dropped into the navigator from the parts management.

use super::{bundled_parts, PanelUi};
use egui::RichText;
use openecraft_core::model::Project;
use openecraft_core::panel;
use openecraft_core::parts::{self, EnclosureFilter, EnclosureMounting, EnclosureSort, Part, PartsDb, SizeRange};
use openecraft_i18n::{tr, tr_fmt};
use openecraft_ui::icons::{self, Icon};

/// The dialog of a new enclosure: its location, the search of the enclosure parts, the
/// part chosen (empty: a custom size) and the room it is to stand in.
pub(super) struct NewEnclosure {
    pub(super) name: String,
    pub(super) part: String,
    pub(super) filter: EnclosureFilter,
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) depth: f32,
    /// The room the enclosure goes into (index).
    pub(super) room: Option<usize>,
}

/// A part dropped into the navigator that none of the parts lists of the editor has:
/// the application imports it from the parts server, then hands it to
/// [`PanelUi::drop_part`].
#[derive(Clone, Debug, PartialEq)]
pub struct PartDrop {
    pub number: String,
    /// The room it was dropped onto (index).
    pub room: Option<usize>,
}

/// Widths of the columns of the list of enclosures, points.
const COLUMNS: [f32; 5] = [190.0, 300.0, 120.0, 50.0, 50.0];

/// The parts an enclosure can be made of: the parts database, the project, the example
/// catalog and the catalogs shipped with OpenECraft.
fn sources<'a>(prj: &'a Project, db: &'a PartsDb) -> impl Iterator<Item = &'a Part> {
    db.parts.iter().chain(prj.parts.iter()).chain(parts::catalog()).chain(bundled_parts())
}

pub(super) fn find_part<'a>(prj: &'a Project, db: &'a PartsDb, number: &str) -> Option<&'a Part> {
    sources(prj, db).find(|p| p.number == number)
}

/// Puts a part into the project with its accessories; a part of a shipped catalog
/// brings those along from there. False when no parts list has it.
fn adopt_part(prj: &mut Project, db: &PartsDb, number: &str) -> bool {
    parts::ensure_in_project(prj, db, number);
    if prj.parts.iter().any(|p| p.number == number) {
        return true;
    }
    let Some(p) = bundled_parts().iter().find(|p| p.number == number) else { return false };
    for a in &p.accessories {
        let known = prj.parts.iter().any(|x| &x.number == a) || db.get(a).is_some() || parts::catalog().iter().any(|x| &x.number == a);
        if let (false, Some(acc)) = (known, bundled_parts().iter().find(|x| &x.number == a)) {
            prj.parts.push(acc.clone());
        }
    }
    prj.parts.push(p.clone());
    true
}

/// The value of a whole-number part data key (`Fields`, `Rows`), "–" without one.
fn count_text(p: &Part, key: &str) -> String {
    parts::count_value(p, key).map_or_else(|| "–".to_string(), |n| n.to_string())
}

impl PanelUi {
    /// Opens the dialog for a new enclosure, standing in room `room` (index) if given.
    pub fn new_enclosure_dialog_in(&mut self, prj: &Project, db: &PartsDb, room: Option<usize>) {
        let part = if db.get(panel::ENCLOSURE_PART).is_some() { panel::ENCLOSURE_PART } else { "" };
        let mut ne = NewEnclosure {
            name: panel::free_name(&prj.layout, "+CAB"),
            part: part.into(),
            filter: self.new_enclosure.as_ref().map(|n| n.filter.clone()).or_else(|| self.enclosure_filter.clone()).unwrap_or_default(),
            width: 600.0,
            height: 800.0,
            depth: 250.0,
            room: room.filter(|&r| prj.layout.enclosures.get(r).is_some_and(|e| e.is_room())),
        };
        apply_part_size(&mut ne, prj, db);
        self.new_enclosure = Some(ne);
    }

    /// Opens the dialog for a new enclosure.
    pub fn new_enclosure_dialog(&mut self, prj: &Project, db: &PartsDb) {
        self.new_enclosure_dialog_in(prj, db, None);
    }

    /// The dialog of a new enclosure (shown while it is open).
    pub(super) fn new_enclosure_window(&mut self, ctx: &egui::Context, prj: &mut Project, db: &PartsDb) {
        let Some(ne) = &mut self.new_enclosure else { return };
        let mut all: Vec<&Part> = Vec::new();
        for p in sources(prj, db) {
            if parts::is_enclosure(p) && !all.iter().any(|x| x.number == p.number) {
                all.push(p);
            }
        }
        let found = ne.filter.apply(all.iter().copied());
        let rooms = panel::rooms(&prj.layout);
        let mut open = true;
        let mut create = false;
        let mut cancel = false;
        let before = ne.part.clone();
        egui::Window::new(tr("New enclosure"))
            .open(&mut open)
            .resizable(true)
            .collapsible(false)
            .default_width(COLUMNS.iter().sum::<f32>() + 40.0)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| {
                egui::Grid::new("newenc").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.label(tr("Location"));
                    ui.text_edit_singleline(&mut ne.name);
                    ui.end_row();
                    if !rooms.is_empty() {
                        ui.label(tr("Room"));
                        let name = |r: usize| prj.layout.enclosures[r].name.clone();
                        let title = ne.room.map_or_else(|| tr("(none)").to_string(), name);
                        egui::ComboBox::from_id_salt("newencroom").selected_text(title).width(220.0).show_ui(ui, |ui| {
                            ui.selectable_value(&mut ne.room, None, tr("(none)"));
                            for &r in &rooms {
                                ui.selectable_value(&mut ne.room, Some(r), name(r));
                            }
                        });
                        ui.end_row();
                    }
                });
                ui.separator();
                filters(ui, &mut ne.filter, &all, found.len());
                ui.separator();
                if parts_list(ui, ne, &found) {
                    create = true;
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(tr("Width × height × depth"));
                    ui.add_enabled_ui(ne.part.is_empty(), |ui| {
                        ui.add(egui::DragValue::new(&mut ne.width).speed(5.0).range(100.0..=5000.0));
                        ui.add(egui::DragValue::new(&mut ne.height).speed(5.0).range(100.0..=5000.0));
                        ui.add(egui::DragValue::new(&mut ne.depth).speed(5.0).range(60.0..=2000.0).suffix(" mm"));
                    });
                });
                ui.horizontal(|ui| {
                    if icons::text_button(ui, Icon::Check, tr("Create")).clicked() {
                        create = true;
                    }
                    if ui.button(tr("Cancel")).clicked() {
                        cancel = true;
                    }
                    let chosen = match find_part(prj, db, &ne.part) {
                        Some(p) => format!("{} – {}", p.number, p.description),
                        None => tr("(custom size)").to_string(),
                    };
                    ui.add(egui::Label::new(RichText::new(chosen).weak()).truncate());
                });
            });
        if ne.part != before {
            apply_part_size(ne, prj, db);
        }
        // The search stays for the next new enclosure.
        self.enclosure_filter = Some(ne.filter.clone());
        if create {
            let ne = self.new_enclosure.take().unwrap();
            self.create_enclosure(prj, db, ne);
        } else if !open || cancel {
            self.new_enclosure = None;
        }
    }

    /// Adds the enclosure of the dialog and shows it.
    fn create_enclosure(&mut self, prj: &mut Project, db: &PartsDb, ne: NewEnclosure) -> usize {
        if !ne.part.is_empty() {
            adopt_part(prj, db, &ne.part);
        }
        let mut e = panel::new_enclosure(prj, &ne.part, &ne.name);
        if ne.part.is_empty() {
            e.width = ne.width;
            e.height = ne.height;
            e.depth = ne.depth;
            if !ne.filter.system.is_empty() {
                panel::apply_system(&mut e, &ne.filter.system, None);
            }
        }
        prj.layout.enclosures.push(e);
        let ei = prj.layout.enclosures.len() - 1;
        let room = ne.room.filter(|&r| prj.layout.enclosures.get(r).is_some_and(|e| e.is_room()));
        self.status = match room.map(|r| panel::set_room(prj, ei, Some(r), None)) {
            Some(Err(err)) => err,
            Some(Ok(())) => tr_fmt!("Enclosure {} added in room {}", prj.layout.enclosures[ei].name, prj.layout.enclosures[room.unwrap()].name),
            None => tr_fmt!("Enclosure {} added", prj.layout.enclosures[ei].name),
        };
        self.open_cabinet(ei);
        ei
    }

    /// A part dropped into the navigator: an enclosure of the part, in room `room`
    /// (index) if it was dropped onto one. Its index, or why not.
    pub fn drop_part(&mut self, prj: &mut Project, db: &PartsDb, number: &str, room: Option<usize>) -> Result<usize, String> {
        let Some(p) = find_part(prj, db, number) else {
            return Err(tr_fmt!("{} is neither in the parts database nor on the parts server", number));
        };
        if !parts::is_enclosure(p) {
            return Err(tr_fmt!("{} is not an enclosure: only enclosures can be dropped here", number));
        }
        let ne = NewEnclosure {
            name: panel::free_name(&prj.layout, "+CAB"),
            part: number.to_string(),
            filter: EnclosureFilter::default(),
            width: p.width,
            height: p.height,
            depth: p.depth,
            room,
        };
        Ok(self.create_enclosure(prj, db, ne))
    }

    /// A part dropped into the navigator: made into an enclosure when a parts list of
    /// the editor has it, otherwise asked of the application (`fetch_part`).
    pub(super) fn part_dropped(&mut self, prj: &mut Project, db: &PartsDb, number: &str, room: Option<usize>) {
        if find_part(prj, db, number).is_none() {
            self.fetch_part = Some(PartDrop { number: number.to_string(), room });
            return;
        }
        if let Err(e) = self.drop_part(prj, db, number, room) {
            self.status = e;
        }
    }
}

/// Takes the size of the part chosen.
fn apply_part_size(ne: &mut NewEnclosure, prj: &Project, db: &PartsDb) {
    if let Some(p) = find_part(prj, db, &ne.part).filter(|p| p.width > 0.0 && p.height > 0.0) {
        ne.width = p.width;
        ne.height = p.height;
        if p.depth > 0.0 {
            ne.depth = p.depth;
        }
    }
}

/// The search line and the filters of the enclosures.
fn filters(ui: &mut egui::Ui, f: &mut EnclosureFilter, all: &[&Part], found: usize) {
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut f.text).hint_text(tr("Search…")).desired_width(260.0));
        ui.weak(tr_fmt!("{} of {} enclosures", found, all.len()));
        if f.is_active() && icons::text_button(ui, Icon::Close, tr("Clear filters")).clicked() {
            f.reset();
        }
    });
    egui::Grid::new("newencfilter").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
        ui.label(tr("Enclosure system"));
        let title = panel::SYSTEMS.iter().find(|s| s.id() == f.system).map(|s| s.title()).unwrap_or(tr("(all)"));
        egui::ComboBox::from_id_salt("newencsys").selected_text(title).width(200.0).show_ui(ui, |ui| {
            ui.selectable_value(&mut f.system, String::new(), tr("(all)"));
            for s in &panel::SYSTEMS {
                ui.selectable_value(&mut f.system, s.id().to_string(), s.title());
            }
        });
        ui.label(tr("Manufacturer"));
        let makers = parts::distinct_values(all.iter().copied(), |p| p.manufacturer.as_str());
        let title = if f.manufacturer.is_empty() { tr("(all)").to_string() } else { f.manufacturer.clone() };
        egui::ComboBox::from_id_salt("newencmaker").selected_text(title).width(160.0).show_ui(ui, |ui| {
            ui.selectable_value(&mut f.manufacturer, String::new(), tr("(all)"));
            for m in makers {
                ui.selectable_value(&mut f.manufacturer, m.clone(), m);
            }
        });
        ui.end_row();
        ui.label(tr("Mounting"));
        ui.horizontal(|ui| {
            ui.selectable_value(&mut f.mounting, EnclosureMounting::Any, tr("(all)"));
            ui.selectable_value(&mut f.mounting, EnclosureMounting::Wall, tr("Wall"));
            ui.selectable_value(&mut f.mounting, EnclosureMounting::Floor, tr("Floor"));
        });
        ui.label(tr("Sort by"));
        egui::ComboBox::from_id_salt("newencsort").selected_text(sort_title(f.sort)).width(160.0).show_ui(ui, |ui| {
            for s in EnclosureSort::ALL {
                ui.selectable_value(&mut f.sort, s, sort_title(s));
            }
        });
        ui.end_row();
        ui.label(tr("Width"));
        range(ui, &mut f.width);
        ui.label(tr("Height"));
        range(ui, &mut f.height);
        ui.end_row();
        ui.label(tr("Depth"));
        range(ui, &mut f.depth);
        ui.label(tr("Fields / rows"));
        ui.horizontal(|ui| {
            count(ui, "newencfields", &mut f.fields, &parts::distinct_counts(all.iter().copied(), "Fields"));
            count(ui, "newencrows", &mut f.rows, &parts::distinct_counts(all.iter().copied(), "Rows"));
        })
        .response
        .on_hover_text(tr("Field widths and rows of meter and distribution cabinets"));
        ui.end_row();
    });
}

fn sort_title(s: EnclosureSort) -> &'static str {
    match s {
        EnclosureSort::Number => tr("Part number"),
        EnclosureSort::Width => tr("Width"),
        EnclosureSort::Height => tr("Height"),
        EnclosureSort::Depth => tr("Depth"),
        EnclosureSort::Size => tr("Size"),
    }
}

/// From … to of a dimension, mm; 0 is any.
fn range(ui: &mut egui::Ui, r: &mut SizeRange) {
    let any = |v: f64, _| if v <= 0.0 { tr("any").to_string() } else { format!("{v:.0}") };
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut r.min).speed(10.0).range(0.0..=5000.0).custom_formatter(any)).on_hover_text(tr("At least, mm (0: any)"));
        ui.label("–");
        ui.add(egui::DragValue::new(&mut r.max).speed(10.0).range(0.0..=5000.0).custom_formatter(any)).on_hover_text(tr("At most, mm (0: any)"));
    });
}

/// Choice of a number of fields or rows among those of the enclosures.
fn count(ui: &mut egui::Ui, id: &str, value: &mut Option<u32>, choices: &[u32]) {
    let text = |v: Option<u32>| v.map_or_else(|| tr("any").to_string(), |n| n.to_string());
    ui.add_enabled_ui(!choices.is_empty() || value.is_some(), |ui| {
        egui::ComboBox::from_id_salt(id).selected_text(text(*value)).width(50.0).show_ui(ui, |ui| {
            ui.selectable_value(value, None, text(None));
            for &n in choices {
                ui.selectable_value(value, Some(n), text(Some(n)));
            }
        });
    });
}

/// The enclosures found, a custom size first; a click chooses one, a double click
/// creates the enclosure (true).
fn parts_list(ui: &mut egui::Ui, ne: &mut NewEnclosure, found: &[&Part]) -> bool {
    let cell = |ui: &mut egui::Ui, w: f32, text: RichText| {
        let size = egui::vec2(w, ui.spacing().interact_size.y);
        ui.allocate_ui_with_layout(size, egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_width(w);
            ui.add(egui::Label::new(text).truncate());
        });
    };
    ui.horizontal(|ui| {
        for (w, title) in COLUMNS.iter().zip([tr("Part"), tr("Description"), tr("Width × height × depth"), tr("Fields"), tr("Rows")]) {
            cell(ui, *w, RichText::new(title).strong());
        }
    });
    let mut create = false;
    let row_h = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
    // Of a fixed height, so that the window keeps its size while the filters change.
    egui::ScrollArea::vertical().id_salt("newencparts").max_height(row_h * 12.0).auto_shrink([false, false]).show_rows(ui, row_h, found.len() + 1, |ui, rows| {
        for i in rows {
            let p = i.checked_sub(1).map(|k| found[k]);
            let number = p.map_or("", |p| p.number.as_str());
            let r = ui
                .horizontal(|ui| {
                    let label = p.map_or_else(|| tr("(custom size)").to_string(), |p| p.number.clone());
                    let r = ui.add_sized([COLUMNS[0], ui.spacing().interact_size.y], egui::Button::selectable(ne.part == number, label).truncate());
                    if let Some(p) = p {
                        cell(ui, COLUMNS[1], RichText::new(&p.description));
                        cell(ui, COLUMNS[2], RichText::new(format!("{:.0} × {:.0} × {:.0}", p.width, p.height, p.depth)));
                        cell(ui, COLUMNS[3], RichText::new(count_text(p, "Fields")));
                        cell(ui, COLUMNS[4], RichText::new(count_text(p, "Rows")));
                    }
                    r
                })
                .inner;
            let r = match p {
                Some(p) => r.on_hover_text(format!("{}\n{}", p.description, p.manufacturer)),
                None => r,
            };
            if r.clicked() {
                ne.part = number.to_string();
            }
            if r.double_clicked() {
                ne.part = number.to_string();
                create = true;
            }
        }
    });
    create
}

#[cfg(test)]
mod tests;
