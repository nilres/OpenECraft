//! Buildings pane: the buildings of the plant with their levels, the rooms on them,
//! their shafts, stairs and roofs (with roof windows and PV arrays), the links between
//! buildings and a site plan of all of them ([`openecraft_core::panel::buildings`],
//! [`openecraft_core::panel::stairs`], [`openecraft_core::panel::roofs`],
//! [`openecraft_core::panel::solar`]).

use super::App;
use egui::{pos2, vec2, Color32, Pos2, RichText, Sense, Stroke};
use openecraft_core::model::Project;
use openecraft_core::panel::{self, buildings as bld, roofs, solar, stairs, LinkKind, Opening, PvMounting, RoofShape, StairKind, Turn};
use openecraft_i18n::{tr, tr_fmt};
use openecraft_ui::icons::{self, Icon};

#[derive(Default)]
pub struct BuildingsUi {
    pub open: bool,
    /// Index of the building shown.
    sel: usize,
    /// Level shown in the site plan (id), 0 for all levels.
    level: u64,
    /// The building entries of a new link.
    link: (u64, u64),
    kind: LinkKind,
    status: String,
    /// Kind of a new stair and shape of a new roof.
    stair_kind: StairKind,
    roof_shape: RoofShape,
}

/// Corners in the floor plan.
type Outline = Vec<[f32; 2]>;

/// Glass of roof windows in the site plan.
const GLASS: Color32 = Color32::from_rgba_premultiplied(60, 100, 140, 120);
/// PV modules in the site plan.
const PV_MODULE: Color32 = Color32::from_rgba_premultiplied(20, 30, 70, 200);

/// A part to choose: its number, description and size (width, height, depth).
struct Choice {
    number: String,
    description: String,
    subgroup: String,
    size: [f32; 3],
}

/// The parts of the project and of the built-in catalog of one of `subgroups`.
fn choices(prj: &Project, subgroups: &[&str]) -> Vec<Choice> {
    let mut out: Vec<Choice> = Vec::new();
    for p in prj.parts.iter().chain(openecraft_core::parts::catalog()) {
        if subgroups.contains(&p.subgroup.as_str()) && !out.iter().any(|c| c.number == p.number) {
            out.push(Choice { number: p.number.clone(), description: p.description.clone(), subgroup: p.subgroup.clone(), size: [p.width, p.height, p.depth] });
        }
    }
    out
}

/// The PV arrays of a roof: place, modules, mounting and mounting system of each.
fn pv_arrays(ui: &mut egui::Ui, r: &mut panel::Roof, k: usize, modules: &[Choice], systems: &[Choice], powers: &[String]) {
    let mut gone = None;
    for (a_k, a) in r.solar.iter_mut().enumerate() {
        let id = (k, a_k);
        ui.horizontal_wrapped(|ui| {
            ui.add_space(14.0);
            ui.add(egui::TextEdit::singleline(&mut a.name).desired_width(40.0));
            egui::ComboBox::from_id_salt(("bldpvmount", id)).selected_text(tr(a.mounting.title())).show_ui(ui, |ui| {
                for m in PvMounting::ALL {
                    ui.selectable_value(&mut a.mounting, m, tr(m.title()));
                }
            });
            egui::ComboBox::from_id_salt(("bldpvmodule", id)).selected_text(a.module.as_str()).width(170.0).show_ui(ui, |ui| {
                for c in modules {
                    if ui.selectable_label(a.module == c.number, c.number.as_str()).on_hover_text(c.description.as_str()).clicked() {
                        a.module = c.number.clone();
                        if c.size[0] > 0.0 && c.size[1] > 0.0 {
                            (a.module_width, a.module_length) = (c.size[0].min(c.size[1]), c.size[0].max(c.size[1]));
                        }
                        if c.size[2] > 0.0 {
                            a.module_height = c.size[2];
                        }
                    }
                }
            })
            .response
            .on_hover_text(tr("Module part; its size comes from the part"));
            ui.label(powers.get(a_k).map_or("", String::as_str));
            if icons::button(ui, Icon::Delete, tr("Delete PV array")).clicked() {
                gone = Some(a_k);
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.add_space(28.0);
            ui.add(egui::DragValue::new(&mut a.x).speed(10.0).prefix("x ").suffix(" mm"));
            ui.add(egui::DragValue::new(&mut a.z).speed(10.0).prefix("z ").suffix(" mm")).on_hover_text(tr("Middle of the PV array in the floor plan of the building"));
            ui.add(egui::DragValue::new(&mut a.columns).range(1..=200).suffix(" ×")).on_hover_text(tr("Modules side by side in a row"));
            ui.add(egui::DragValue::new(&mut a.rows).range(1..=100)).on_hover_text(tr("Rows of modules (east-west: rows of pairs)"));
            ui.checkbox(&mut a.landscape, tr("landscape")).on_hover_text(tr("The long side of the modules along the rows"));
        });
        ui.horizontal_wrapped(|ui| {
            ui.add_space(28.0);
            if a.mounting.is_tilted() {
                ui.label(tr("Tilt"));
                ui.add(egui::DragValue::new(&mut a.tilt).speed(0.5).range(0.0..=60.0).suffix(" °"));
                ui.label(tr("Facing"));
                ui.add(egui::DragValue::new(&mut a.azimuth).speed(1.0).suffix(" °")).on_hover_text(tr("Direction the modules face, counter-clockwise seen from above (0: towards the front)"));
                ui.label(tr("Row distance"));
                ui.add(egui::DragValue::new(&mut a.row_gap).speed(10.0).range(0.0..=20_000.0).suffix(" mm")).on_hover_text(tr("From row to row; 0: free of shade in winter"));
                ui.label(tr("Ballast"));
                ui.add(egui::DragValue::new(&mut a.ballast).speed(0.5).range(0.0..=500.0).suffix(" kg")).on_hover_text(tr("Ballast per module"));
            } else {
                ui.label(tr("Hook distance"));
                ui.add(egui::DragValue::new(&mut a.span).speed(10.0).range(300.0..=3000.0).suffix(" mm")).on_hover_text(tr("Greatest distance of the roof hooks along a rail"));
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.add_space(28.0);
            ui.label(tr("Mounting system"));
            let main = if a.mounting.is_tilted() { solar::Role::Base } else { solar::Role::Rail };
            let shown = if a.system.is_empty() { tr("Generic").to_string() } else { a.system.clone() };
            egui::ComboBox::from_id_salt(("bldpvsystem", id)).selected_text(shown).width(150.0).show_ui(ui, |ui| {
                ui.selectable_value(&mut a.system, String::new(), tr("Generic"));
                for c in systems.iter().filter(|c| c.subgroup == main.subgroup()) {
                    ui.selectable_value(&mut a.system, c.number.clone(), c.number.as_str()).on_hover_text(c.description.as_str());
                }
            })
            .response
            .on_hover_text(tr("The rail or mounting base; its accessories are the hooks, clamps and ballast"));
            ui.label(tr("Device"));
            ui.add(egui::TextEdit::singleline(&mut a.device).desired_width(90.0)).on_hover_text(tr("Device (PV generator) the modules belong to; its parts then count them in the bill of materials"));
        });
        *a = solar::checked_array(a.clone());
    }
    if let Some(a_k) = gone {
        r.solar.remove(a_k);
    }
}

/// A free name for a new building: `+B1`, `+B2`, …
fn free_building_name(prj: &Project) -> String {
    let taken = |n: &str| prj.layout.buildings.iter().any(|b| b.name == n) || prj.layout.enclosures.iter().any(|e| e.name == n);
    (1..).map(|n| format!("+B{n}")).find(|n| !taken(n)).unwrap_or_default()
}

/// The building entries of all rooms: (item id, "room – wall n").
fn entry_choices(prj: &Project) -> Vec<(u64, String)> {
    let layout = &prj.layout;
    panel::rooms(layout)
        .into_iter()
        .flat_map(|ri| {
            let e = &layout.enclosures[ri];
            e.items.iter().filter(|i| i.opening == Opening::Entry).map(move |i| (i.id, format!("{}  {}", e.name, e.face_title(i.face))))
        })
        .collect()
}

fn entry_label(choices: &[(u64, String)], id: u64) -> String {
    choices.iter().find(|c| c.0 == id).map_or_else(|| tr("(none)").to_string(), |c| c.1.clone())
}

impl App {
    #[allow(clippy::too_many_lines)]
    pub(super) fn buildings_pane(&mut self, ui: &mut egui::Ui) {
        let (prj, st) = (&mut self.session.prj, &mut self.buildings_ui);
        ui.horizontal_wrapped(|ui| {
            if icons::text_button(ui, Icon::Plus, tr("New building")).on_hover_text(tr("A building with one level at ground level, beside what stands in the plant")).clicked() {
                let name = free_building_name(prj);
                match bld::add_building(prj, &name) {
                    Ok(bi) => {
                        st.sel = bi;
                        st.status.clear();
                    }
                    Err(e) => st.status = e,
                }
            }
            ui.weak(tr("Rooms stand on the levels of a building; shafts join rooms on different levels, links join the building entries of different buildings."));
        });
        if !st.status.is_empty() {
            ui.colored_label(crate::style::warning(), st.status.as_str());
        }
        let problems = bld::problems(prj);
        if !problems.is_empty() {
            egui::CollapsingHeader::new(RichText::new(tr_fmt!("{} problems", problems.len())).color(crate::style::warning())).id_salt("bldproblems").show(ui, |ui| {
                for p in &problems {
                    ui.label(p.message());
                }
            });
        }
        if prj.layout.buildings.is_empty() {
            ui.weak(tr("No building yet: create one with New building, then put rooms on its levels."));
            return;
        }
        st.sel = st.sel.min(prj.layout.buildings.len() - 1);
        ui.horizontal_wrapped(|ui| {
            for (bi, b) in prj.layout.buildings.iter().enumerate() {
                if ui.selectable_label(st.sel == bi, b.name.as_str()).clicked() {
                    st.sel = bi;
                    st.level = 0;
                }
            }
        });
        ui.separator();
        egui::ScrollArea::vertical().id_salt("bldscroll").auto_shrink([false, false]).show(ui, |ui| {
            let bi = st.sel;
            building_section(ui, prj, st, bi);
            if st.sel >= prj.layout.buildings.len() {
                return;
            }
            ui.separator();
            levels_section(ui, prj, st, bi);
            ui.separator();
            shafts_section(ui, prj, bi);
            ui.separator();
            stairs_section(ui, prj, st, bi);
            ui.separator();
            roofs_section(ui, prj, st, bi);
            ui.separator();
            links_section(ui, prj, st);
            ui.separator();
            site_plan(ui, prj, st);
        });
    }
}

fn building_section(ui: &mut egui::Ui, prj: &mut Project, st: &mut BuildingsUi, bi: usize) {
    let mut remove = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new(tr("Building")).strong());
        if icons::button(ui, Icon::Delete, tr("Delete building; its rooms stay where they stand")).clicked() {
            remove = true;
        }
    });
    if remove {
        bld::remove_building(prj, bi);
        st.sel = bi.saturating_sub(1);
        return;
    }
    let b = &mut prj.layout.buildings[bi];
    egui::Grid::new("bldprops").num_columns(2).spacing([10.0, 5.0]).show(ui, |ui| {
        ui.label(tr("Location"));
        ui.text_edit_singleline(&mut b.name);
        ui.end_row();
        ui.label(tr("Description"));
        ui.text_edit_singleline(&mut b.description);
        ui.end_row();
        ui.label(tr("Place in the plant"));
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut b.site.x).speed(50.0).prefix("x ").suffix(" mm"));
            ui.add(egui::DragValue::new(&mut b.site.z).speed(50.0).prefix("z ").suffix(" mm"));
            ui.add(egui::DragValue::new(&mut b.site.angle).speed(1.0).range(-360.0..=360.0).suffix(" °"));
        })
        .response
        .on_hover_text(tr("Origin of the building's floor plan in the plant; its rooms move with it"));
        ui.end_row();
    });
}

#[allow(clippy::too_many_lines)]
fn levels_section(ui: &mut egui::Ui, prj: &mut Project, st: &mut BuildingsUi, bi: usize) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(tr("Levels")).strong());
        if icons::text_button(ui, Icon::Plus, tr("Level on top")).clicked() {
            if let Err(e) = bld::add_level(prj, bi, None, None, None) {
                st.status = e;
            }
        }
        if icons::text_button(ui, Icon::Plus, tr("Basement")).on_hover_text(tr("A level below the lowest one")).clicked() {
            let low = prj.layout.buildings[bi].levels.first().map_or(0.0, |l| l.elevation);
            if let Err(e) = bld::add_level(prj, bi, None, Some(low - bld::LEVEL_HEIGHT), None) {
                st.status = e;
            }
        }
    });
    let free_rooms: Vec<(usize, String)> = panel::rooms(&prj.layout).into_iter().filter(|&r| prj.layout.enclosures[r].level == 0).map(|r| (r, prj.layout.enclosures[r].name.clone())).collect();
    let mut change: Option<(u64, Option<f32>, Option<f32>)> = None;
    let mut remove = None;
    let mut new_room = None;
    let mut put: Option<(usize, u64)> = None;
    let mut take_out = None;
    let levels = prj.layout.buildings[bi].levels.clone();
    egui::Grid::new("bldlevels").num_columns(5).striped(true).spacing([10.0, 4.0]).show(ui, |ui| {
        ui.label(RichText::new(tr("Name")).weak());
        ui.label(RichText::new(tr("Floor above ground")).weak());
        ui.label(RichText::new(tr("Height")).weak());
        ui.label(RichText::new(tr("Rooms")).weak());
        ui.label("");
        ui.end_row();
        for (k, l) in levels.iter().enumerate().rev() {
            // The name is edited in place (designations, not checked while typing).
            ui.add(egui::TextEdit::singleline(&mut prj.layout.buildings[bi].levels[k].name).desired_width(60.0));
            let mut elevation = l.elevation;
            if ui.add(egui::DragValue::new(&mut elevation).speed(50.0).suffix(" mm")).changed() {
                change = Some((l.id, Some(elevation), None));
            }
            let mut height = l.height;
            if ui.add(egui::DragValue::new(&mut height).speed(50.0).range(1000.0..=50_000.0).suffix(" mm")).changed() {
                change = Some((l.id, None, Some(height)));
            }
            ui.horizontal(|ui| {
                for r in bld::rooms_on(&prj.layout, l.id) {
                    ui.label(prj.layout.enclosures[r].name.as_str());
                    if icons::button(ui, Icon::Close, tr("Take the room out of the building; it stays where it stands")).clicked() {
                        take_out = Some(r);
                    }
                }
                if icons::button(ui, Icon::Wall, tr("New room on this level")).clicked() {
                    new_room = Some(l.id);
                }
                if !free_rooms.is_empty() {
                    egui::ComboBox::from_id_salt(("bldput", l.id)).selected_text(tr("Put a room here")).width(140.0).show_ui(ui, |ui| {
                        for (r, n) in &free_rooms {
                            if ui.selectable_label(false, n.as_str()).clicked() {
                                put = Some((*r, l.id));
                            }
                        }
                    });
                }
            });
            if icons::button(ui, Icon::Delete, tr("Delete level; its rooms stay where they stand")).clicked() {
                remove = Some(l.id);
            }
            ui.end_row();
        }
    });
    let result = if let Some((id, elevation, height)) = change {
        bld::set_level(prj, bi, id, None, elevation, height)
    } else if let Some(id) = remove {
        bld::remove_level(prj, bi, id)
    } else if let Some(level) = new_room {
        let name = panel::free_name(&prj.layout, "+ROOM");
        bld::add_room(prj, &name, Some(level)).map(|_| ())
    } else if let Some((r, level)) = put {
        bld::set_room_level(prj, r, Some(level), None)
    } else if let Some(r) = take_out {
        bld::set_room_level(prj, r, None, None)
    } else {
        Ok(())
    };
    match result {
        Ok(()) if change.is_some() || remove.is_some() || new_room.is_some() || put.is_some() || take_out.is_some() => st.status.clear(),
        Ok(()) => {}
        Err(e) => st.status = e,
    }
    ui.weak(tr("A room on a level stands at its place in the building's floor plan (Layout space › room properties); without one next to the other rooms of the level."));
}

fn shafts_section(ui: &mut egui::Ui, prj: &mut Project, bi: usize) {
    let poses = panel::site_poses(prj);
    ui.horizontal(|ui| {
        ui.label(RichText::new(tr("Shafts")).strong());
        if icons::text_button(ui, Icon::Plus, tr("New shaft")).on_hover_text(tr("Through all levels; move it to its place in the building's floor plan")).clicked() {
            let b = &prj.layout.buildings[bi];
            let (lo, hi) = (b.levels.first().map_or(0, |l| l.id), b.levels.last().map_or(0, |l| l.id));
            let _ = bld::add_shaft(prj, bi, None, 0.0, 0.0, bld::SHAFT_SIZE, bld::SHAFT_SIZE, lo, hi);
        }
    });
    let b = &prj.layout.buildings[bi];
    if b.shafts.is_empty() {
        ui.weak(tr("No shaft yet: cables, tubes and pipes run through shafts from level to level."));
        return;
    }
    let joins: Vec<String> = b.shafts.iter().map(|s| bld::shaft_rooms(prj, &poses, bi, s).into_iter().map(|r| prj.layout.enclosures[r].name.clone()).collect::<Vec<_>>().join(", ")).collect();
    let levels: Vec<(u64, String)> = b.levels.iter().map(|l| (l.id, l.name.clone())).collect();
    let level_name = |id: u64| levels.iter().find(|l| l.0 == id).map_or_else(String::new, |l| l.1.clone());
    let mut remove = None;
    let b = &mut prj.layout.buildings[bi];
    egui::Grid::new("bldshafts").num_columns(6).striped(true).spacing([8.0, 4.0]).show(ui, |ui| {
        for t in [tr("Name"), tr("Place"), tr("Size"), tr("Levels"), tr("Joins"), ""] {
            ui.label(RichText::new(t).weak());
        }
        ui.end_row();
        for (k, s) in b.shafts.iter_mut().enumerate() {
            ui.add(egui::TextEdit::singleline(&mut s.name).desired_width(50.0));
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut s.x).speed(10.0).prefix("x ").suffix(" mm"));
                ui.add(egui::DragValue::new(&mut s.z).speed(10.0).prefix("z ").suffix(" mm"));
            });
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut s.width).speed(10.0).range(100.0..=20_000.0));
                ui.add(egui::DragValue::new(&mut s.depth).speed(10.0).range(100.0..=20_000.0).suffix(" mm"));
            });
            ui.horizontal(|ui| {
                for (salt, value) in [("bot", &mut s.bottom), ("top", &mut s.top)] {
                    egui::ComboBox::from_id_salt(("bldshaft", salt, k)).selected_text(level_name(*value)).width(60.0).show_ui(ui, |ui| {
                        for (id, n) in &levels {
                            ui.selectable_value(value, *id, n.as_str());
                        }
                    });
                }
            });
            if joins[k].is_empty() {
                ui.colored_label(crate::style::warning(), tr("no rooms"));
            } else {
                ui.label(joins[k].as_str());
            }
            if icons::button(ui, Icon::Delete, tr("Delete shaft")).clicked() {
                remove = Some(k);
            }
            ui.end_row();
        }
    });
    if let Some(k) = remove {
        b.shafts.remove(k);
    }
    // Levels chosen from the top down are turned round.
    let shafts: Vec<_> = b.shafts.clone();
    for (k, s) in shafts.into_iter().enumerate() {
        if let Ok(s) = bld::checked_shaft(b, s) {
            b.shafts[k] = s;
        }
    }
}

/// A choice of a level of building `bi` by its id.
fn level_combo(ui: &mut egui::Ui, salt: impl std::hash::Hash + std::fmt::Debug, levels: &[(u64, String)], value: &mut u64) {
    let name = levels.iter().find(|l| l.0 == *value).map_or_else(String::new, |l| l.1.clone());
    egui::ComboBox::from_id_salt(salt).selected_text(name).width(60.0).show_ui(ui, |ui| {
        for (id, n) in levels {
            ui.selectable_value(value, *id, n.as_str());
        }
    });
}

fn stairs_section(ui: &mut egui::Ui, prj: &mut Project, st: &mut BuildingsUi, bi: usize) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(tr("Stairs")).strong());
        egui::ComboBox::from_id_salt("bldstairkind").selected_text(tr(st.stair_kind.title())).show_ui(ui, |ui| {
            for kind in StairKind::ALL {
                ui.selectable_value(&mut st.stair_kind, kind, tr(kind.title()));
            }
        });
        if icons::text_button(ui, Icon::Plus, tr("New stair")).on_hover_text(tr("From the lowest level up to the next one; move it to its place in the building's floor plan")).clicked() {
            let bottom = prj.layout.buildings[bi].levels.first().map_or(0, |l| l.id);
            match stairs::add_stair(prj, bi, st.stair_kind, 0.0, 0.0, bottom, None) {
                Ok(_) => st.status.clear(),
                Err(e) => st.status = e,
            }
        }
    });
    let b = &prj.layout.buildings[bi];
    if b.stairs.is_empty() {
        ui.weak(tr("No stair yet: stairs lead from the floor of one level up to the floor of another."));
        return;
    }
    let levels: Vec<(u64, String)> = b.levels.iter().map(|l| (l.id, l.name.clone())).collect();
    let steps: Vec<String> = b.stairs.iter().map(|s| stairs::geometry(b, s).map_or_else(|| tr("no valid levels").to_string(), |g| tr_fmt!("{} × {} mm", g.risers, format!("{:.0}", g.rise)))).collect();
    let mut remove = None;
    let b = &mut prj.layout.buildings[bi];
    egui::Grid::new("bldstairs").num_columns(7).striped(true).spacing([8.0, 4.0]).show(ui, |ui| {
        for t in [tr("Name"), tr("Kind"), tr("Foot"), tr("Width / going"), tr("Levels"), tr("Risers"), ""] {
            ui.label(RichText::new(t).weak());
        }
        ui.end_row();
        for (k, s) in b.stairs.iter_mut().enumerate() {
            ui.add(egui::TextEdit::singleline(&mut s.name).desired_width(40.0));
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt(("bldstair", k)).selected_text(tr(s.kind.title())).width(120.0).show_ui(ui, |ui| {
                    for kind in StairKind::ALL {
                        ui.selectable_value(&mut s.kind, kind, tr(kind.title()));
                    }
                });
                if s.kind != StairKind::Straight {
                    egui::ComboBox::from_id_salt(("bldstairturn", k)).selected_text(tr(s.turn.title())).width(60.0).show_ui(ui, |ui| {
                        for turn in [Turn::Left, Turn::Right] {
                            ui.selectable_value(&mut s.turn, turn, tr(turn.title()));
                        }
                    });
                }
            });
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut s.x).speed(10.0).prefix("x ").suffix(" mm"));
                ui.add(egui::DragValue::new(&mut s.z).speed(10.0).prefix("z ").suffix(" mm"));
                ui.add(egui::DragValue::new(&mut s.angle).speed(1.0).range(-360.0..=360.0).suffix(" °"));
            })
            .response
            .on_hover_text(tr("Middle of the front edge of the lowest step in the building's floor plan, and the direction of the first flight"));
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut s.width).speed(10.0).range(500.0..=5000.0));
                ui.add(egui::DragValue::new(&mut s.tread).speed(1.0).range(150.0..=500.0).suffix(" mm"));
            });
            ui.horizontal(|ui| {
                level_combo(ui, ("bldstairbot", k), &levels, &mut s.bottom);
                level_combo(ui, ("bldstairtop", k), &levels, &mut s.top);
            });
            ui.label(steps[k].as_str());
            if icons::button(ui, Icon::Delete, tr("Delete stair")).clicked() {
                remove = Some(k);
            }
            ui.end_row();
        }
    });
    if let Some(k) = remove {
        b.stairs.remove(k);
    }
}

fn roofs_section(ui: &mut egui::Ui, prj: &mut Project, st: &mut BuildingsUi, bi: usize) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(tr("Roofs")).strong());
        egui::ComboBox::from_id_salt("bldroofshape").selected_text(tr(st.roof_shape.title())).show_ui(ui, |ui| {
            for shape in RoofShape::ALL {
                ui.selectable_value(&mut st.roof_shape, shape, tr(shape.title()));
            }
        });
        if icons::text_button(ui, Icon::Plus, tr("New roof")).on_hover_text(tr("Over the rooms of the building with its eaves on top of the highest level")).clicked() {
            match roofs::add_roof(prj, bi, st.roof_shape) {
                Ok(_) => st.status.clear(),
                Err(e) => st.status = e,
            }
        }
    });
    if prj.layout.buildings[bi].roofs.is_empty() {
        ui.weak(tr("No roof yet: a roof lower than the ceilings of the rooms below it slopes their ceilings, also through several levels."));
        return;
    }
    // The rooms of the building under sloped ceilings, with their lowest and highest ceiling.
    let poses = panel::site_poses(prj);
    let sloped: Vec<String> = bld::building_rooms(&prj.layout, bi)
        .into_iter()
        .filter_map(|ri| {
            let e = &prj.layout.enclosures[ri];
            let c = roofs::ceiling_with(prj, &poses, ri)?;
            let (lo, hi) = c.range(&roofs::floor_polys(e));
            Some(tr_fmt!("{}: {} – {} m", e.name, format!("{:.2}", lo / 1000.0), format!("{:.2}", hi / 1000.0)))
        })
        .collect();
    let modules = choices(prj, &["PV module"]);
    let systems = choices(prj, &[solar::Role::Rail.subgroup(), solar::Role::Base.subgroup()]);
    let powers: Vec<Vec<String>> = prj.layout.buildings[bi]
        .roofs
        .iter()
        .map(|r| {
            r.solar
                .iter()
                .map(|a| match solar::peak_power(prj, a) {
                    Some(kw) => tr_fmt!("{} modules, {} kWp", solar::module_count(a), format!("{kw:.2}")),
                    None => tr_fmt!("{} modules", solar::module_count(a)),
                })
                .collect()
        })
        .collect();
    let mut remove = None;
    let mut add_window = None;
    let mut add_pv = None;
    let b = &mut prj.layout.buildings[bi];
    for (k, r) in b.roofs.iter_mut().enumerate() {
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::TextEdit::singleline(&mut r.name).desired_width(40.0));
                egui::ComboBox::from_id_salt(("bldroof", k)).selected_text(tr(r.shape.title())).show_ui(ui, |ui| {
                    for shape in RoofShape::ALL {
                        ui.selectable_value(&mut r.shape, shape, tr(shape.title()));
                    }
                });
                if matches!(r.shape, RoofShape::Gable | RoofShape::Shed) {
                    ui.checkbox(&mut r.along_z, tr("along z")).on_hover_text(tr("The ridge (or the slope of a shed roof) runs along z instead of x"));
                }
                if r.shape == RoofShape::Shed {
                    ui.checkbox(&mut r.flip, tr("low side at the rear")).on_hover_text(tr("The low eaves at the rear or left instead of the front or right"));
                }
                ui.label(tr_fmt!("ridge {} m", format!("{:.2}", roofs::ridge_height(r) / 1000.0)));
                if icons::button(ui, Icon::Delete, tr("Delete roof")).clicked() {
                    remove = Some(k);
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(tr("Covers"));
                ui.add(egui::DragValue::new(&mut r.x).speed(10.0).prefix("x ").suffix(" mm"));
                ui.add(egui::DragValue::new(&mut r.z).speed(10.0).prefix("z ").suffix(" mm"));
                ui.add(egui::DragValue::new(&mut r.width).speed(10.0).range(500.0..=200_000.0).prefix("w "));
                ui.add(egui::DragValue::new(&mut r.depth).speed(10.0).range(500.0..=200_000.0).prefix("d ").suffix(" mm"));
            })
            .response
            .on_hover_text(tr("Rear left corner and size of the rectangle it covers (the outer walls) in the building's floor plan"));
            ui.horizontal_wrapped(|ui| {
                ui.label(tr("Eaves"));
                ui.add(egui::DragValue::new(&mut r.eaves).speed(10.0).suffix(" mm")).on_hover_text(tr("Height of the underside of the roof over the walls above the ground level of the building; below the ceiling of a level it slopes the ceilings of the rooms there"));
                if r.shape != RoofShape::Flat {
                    ui.label(tr("Pitch"));
                    ui.add(egui::DragValue::new(&mut r.pitch).speed(0.5).range(1.0..=75.0).suffix(" °"));
                }
                ui.label(tr("Overhang"));
                ui.add(egui::DragValue::new(&mut r.overhang).speed(10.0).range(0.0..=3000.0).suffix(" mm"));
                ui.label(tr("Thickness"));
                ui.add(egui::DragValue::new(&mut r.thickness).speed(5.0).range(50.0..=1000.0).suffix(" mm"));
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(tr("Roof windows")).weak());
                if icons::text_button(ui, Icon::Plus, tr("Roof window")).on_hover_text(tr("Halfway up the front or right slope; move it to its place")).clicked() {
                    add_window = Some(k);
                }
            });
            let mut gone = None;
            for (w_k, w) in r.windows.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.add(egui::DragValue::new(&mut w.x).speed(10.0).prefix("x ").suffix(" mm"));
                    ui.add(egui::DragValue::new(&mut w.z).speed(10.0).prefix("z ").suffix(" mm"));
                    ui.add(egui::DragValue::new(&mut w.width).speed(5.0).range(200.0..=5000.0).prefix("w "));
                    ui.add(egui::DragValue::new(&mut w.length).speed(5.0).range(200.0..=5000.0).prefix("l ").suffix(" mm"))
                        .on_hover_text(tr("Width along the eaves and length up the slope"));
                    if icons::button(ui, Icon::Delete, tr("Delete roof window")).clicked() {
                        gone = Some(w_k);
                    }
                });
            }
            if let Some(w_k) = gone {
                r.windows.remove(w_k);
            }
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(tr("PV arrays")).weak());
                if icons::text_button(ui, Icon::Plus, tr("PV array")).on_hover_text(tr("In the middle of the front or right slope, on a flat roof tilted towards the front; move it to its place")).clicked() {
                    add_pv = Some(k);
                }
            });
            pv_arrays(ui, r, k, &modules, &systems, &powers[k]);
        });
        *r = roofs::checked_roof(r.clone());
    }
    if let Some(k) = remove {
        b.roofs.remove(k);
    }
    if let Some(k) = add_window {
        let _ = roofs::add_window(prj, bi, k, None);
    }
    if let Some(k) = add_pv {
        let _ = solar::add_array(prj, bi, k, None);
    }
    if !sloped.is_empty() {
        ui.label(RichText::new(tr("Rooms under sloped ceilings (lowest – highest ceiling)")).weak());
        for line in sloped {
            ui.label(line);
        }
    }
}

fn links_section(ui: &mut egui::Ui, prj: &mut Project, st: &mut BuildingsUi) {
    ui.label(RichText::new(tr("Links between buildings")).strong());
    let choices = entry_choices(prj);
    let poses = panel::site_poses(prj);
    let lengths: Vec<Option<f32>> = prj.layout.site_links.iter().map(|l| bld::site_link_length(prj, &poses, l)).collect();
    let mut remove = None;
    if !prj.layout.site_links.is_empty() {
        egui::Grid::new("bldlinks").num_columns(6).striped(true).spacing([8.0, 4.0]).show(ui, |ui| {
            for t in [tr("Name"), tr("Kind"), tr("From"), tr("To"), tr("Length"), ""] {
                ui.label(RichText::new(t).weak());
            }
            ui.end_row();
            for (k, l) in prj.layout.site_links.iter_mut().enumerate() {
                ui.add(egui::TextEdit::singleline(&mut l.name).desired_width(50.0));
                egui::ComboBox::from_id_salt(("bldlinkkind", k)).selected_text(l.kind.title()).show_ui(ui, |ui| {
                    for kind in LinkKind::ALL {
                        ui.selectable_value(&mut l.kind, kind, kind.title());
                    }
                });
                ui.label(entry_label(&choices, l.a));
                ui.label(entry_label(&choices, l.b));
                ui.horizontal(|ui| {
                    let mut given = l.length > 0.0;
                    if ui.checkbox(&mut given, tr("given")).on_hover_text(tr("Otherwise along its way from entry to entry")).changed() {
                        l.length = if given { lengths[k].unwrap_or(1000.0).round() } else { 0.0 };
                    }
                    if given {
                        ui.add(egui::DragValue::new(&mut l.length).speed(100.0).range(1.0..=10_000_000.0).suffix(" mm"));
                    } else {
                        ui.label(lengths[k].map_or_else(|| tr("unknown").to_string(), |v| format!("{:.1} m", v / 1000.0)));
                    }
                });
                if icons::button(ui, Icon::Delete, tr("Delete link")).clicked() {
                    remove = Some(k);
                }
                ui.end_row();
            }
        });
    }
    if let Some(k) = remove {
        prj.layout.site_links.remove(k);
    }
    if choices.len() < 2 {
        ui.weak(tr("A link joins two building entries: add building entries to the outer walls of rooms in different buildings (Layout space › Building entry)."));
        return;
    }
    ui.horizontal_wrapped(|ui| {
        for (salt, value) in [("from", &mut st.link.0), ("to", &mut st.link.1)] {
            egui::ComboBox::from_id_salt(("bldnewlink", salt)).selected_text(entry_label(&choices, *value)).width(180.0).show_ui(ui, |ui| {
                for (id, label) in &choices {
                    ui.selectable_value(value, *id, label.as_str());
                }
            });
        }
        egui::ComboBox::from_id_salt("bldnewlinkkind").selected_text(st.kind.title()).show_ui(ui, |ui| {
            for kind in LinkKind::ALL {
                ui.selectable_value(&mut st.kind, kind, kind.title());
            }
        });
        if icons::text_button(ui, Icon::Plus, tr("Link")).on_hover_text(tr("Cables, tubes and pipes are routed through it from one building to the other")).clicked() {
            match bld::add_site_link(prj, st.link.0, st.link.1, st.kind, Vec::new(), None) {
                Ok(_) => st.status.clear(),
                Err(e) => st.status = e,
            }
        }
    });
}

/// The site plan: the rooms of all buildings seen from above (those of the level shown
/// highlighted), the shafts, the stairs reaching the level shown, the roofs with their
/// windows and the links between the buildings.
fn site_plan(ui: &mut egui::Ui, prj: &Project, st: &mut BuildingsUi) {
    let layout = &prj.layout;
    let Some(b) = layout.buildings.get(st.sel) else { return };
    ui.horizontal(|ui| {
        ui.label(RichText::new(tr("Site plan")).strong());
        let shown = b.levels.iter().find(|l| l.id == st.level).map_or_else(|| tr("All levels").to_string(), |l| l.name.clone());
        egui::ComboBox::from_id_salt("bldplanlevel").selected_text(shown).show_ui(ui, |ui| {
            ui.selectable_value(&mut st.level, 0, tr("All levels"));
            for l in &b.levels {
                ui.selectable_value(&mut st.level, l.id, l.name.as_str());
            }
        });
    });
    let poses = panel::site_poses(prj);
    // Every room's outline in the plant (x, z).
    let rooms: Vec<(usize, Vec<[f32; 2]>)> = panel::rooms(layout)
        .into_iter()
        .map(|ri| {
            let pts = panel::outline(&layout.enclosures[ri])
                .into_iter()
                .map(|[x, z]| {
                    let q = poses[ri].apply([x, 0.0, z]);
                    [q[0], q[2]]
                })
                .collect();
            (ri, pts)
        })
        .collect();
    let links: Vec<Vec<[f32; 2]>> = layout.site_links.iter().filter_map(|l| bld::site_link_path(prj, &poses, l)).map(|p| p.into_iter().map(|q| [q[0], q[2]]).collect()).collect();
    let shafts: Vec<[[f32; 2]; 4]> = b.shafts.iter().map(|s| bld::shaft_corners(b, s)).collect();
    // The roofs (their slopes and windows) and stairs of every building in the plant.
    let to_plant = |b: &panel::Building, q: [f32; 2]| {
        let v = bld::building_pose(b).apply([q[0], 0.0, q[1]]);
        [v[0], v[2]]
    };
    let roof_slopes: Vec<(usize, Vec<[f32; 2]>)> = layout.buildings.iter().enumerate().flat_map(|(bi, b)| b.roofs.iter().flat_map(move |r| roofs::slopes(r).into_iter().map(move |p| (bi, p.into_iter().map(|q| to_plant(b, q)).collect())))).collect();
    let roof_windows: Vec<Vec<[f32; 2]>> = layout
        .buildings
        .iter()
        .flat_map(|b| b.roofs.iter().flat_map(move |r| r.windows.iter().filter_map(move |w| roofs::window_corners(r, w)).map(move |(c, _)| c.iter().map(|v| to_plant(b, [v[0], v[2]])).collect())))
        .collect();
    let pv_modules: Vec<Vec<[f32; 2]>> = layout
        .buildings
        .iter()
        .flat_map(|b| b.roofs.iter().flat_map(move |r| r.solar.iter().flat_map(move |a| solar::module_outlines(r, a)).map(move |m| m.iter().map(|&q| to_plant(b, q)).collect())))
        .collect();
    let stair_lines: Vec<(Vec<Outline>, Outline)> = b
        .stairs
        .iter()
        .filter_map(|s| stairs::geometry(b, s))
        .filter(|g| st.level == 0 || b.levels.iter().any(|l| l.id == st.level && g.from <= l.elevation + 0.5 && g.to >= l.elevation - 0.5))
        .map(|g| (g.pieces.iter().map(|p| p.outline.iter().map(|&q| to_plant(b, q)).collect()).collect(), g.walk.iter().map(|&q| to_plant(b, q)).collect()))
        .collect();
    let all: Vec<[f32; 2]> = rooms.iter().flat_map(|r| r.1.iter().copied()).chain(links.iter().flatten().copied()).chain(shafts.iter().flatten().copied()).chain(roof_slopes.iter().flat_map(|r| r.1.iter().copied())).collect();
    let (size, _) = ui.allocate_exact_size(vec2(ui.available_width(), 320.0), Sense::hover());
    let painter = ui.painter_at(size);
    let visuals = ui.visuals();
    painter.rect_filled(size, 2.0, visuals.extreme_bg_color);
    if all.is_empty() {
        painter.text(size.center(), egui::Align2::CENTER_CENTER, tr("No rooms in the plant yet"), egui::FontId::proportional(13.0), visuals.weak_text_color());
        return;
    }
    let (lo, hi) = panel::bounds(&all);
    let span = [(hi[0] - lo[0]).max(1000.0), (hi[1] - lo[1]).max(1000.0)];
    let inner = size.shrink(16.0);
    let scale = (inner.width() / span[0]).min(inner.height() / span[1]);
    let origin = inner.center() - vec2((lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0) * scale;
    let at = |q: [f32; 2]| -> Pos2 { origin + vec2(q[0], q[1]) * scale };
    let line = visuals.text_color();
    let weak = visuals.weak_text_color();
    let accent = visuals.selection.stroke.color;
    // Names already written: those of rooms above each other are written one below the
    // other.
    let mut labels: Vec<Pos2> = Vec::new();
    for (ri, pts) in &rooms {
        let e = &layout.enclosures[*ri];
        let in_building = bld::building_of(layout, *ri) == Some(st.sel);
        let on_level = in_building && (st.level == 0 || e.level == st.level);
        let pts: Vec<Pos2> = pts.iter().map(|&q| at(q)).collect();
        let (stroke, fill) = if on_level { (Stroke::new(1.5, line), visuals.selection.bg_fill.gamma_multiply(0.35)) } else { (Stroke::new(1.0, weak), Color32::TRANSPARENT) };
        if fill != Color32::TRANSPARENT {
            // Rooms at right angles: their cells are filled one by one.
            for (a, c) in panel::floor_cells(e) {
                let corners = [[a[0], a[2]], [c[0], a[2]], [c[0], c[2]], [a[0], c[2]]].map(|[x, z]| {
                    let q = poses[*ri].apply([x, 0.0, z]);
                    at([q[0], q[2]])
                });
                painter.add(egui::Shape::convex_polygon(corners.to_vec(), fill, Stroke::NONE));
            }
        }
        painter.add(egui::Shape::closed_line(pts.clone(), stroke));
        if on_level {
            let mut c = pts.iter().fold(Pos2::ZERO, |s, p| s + p.to_vec2()) / pts.len() as f32;
            while labels.iter().any(|l| (l.x - c.x).abs() < 40.0 && (l.y - c.y).abs() < 13.0) {
                c.y += 13.0;
            }
            labels.push(c);
            painter.text(c, egui::Align2::CENTER_CENTER, e.name.as_str(), egui::FontId::proportional(11.0), line);
        }
    }
    // The roofs: their eaves, ridges and hips dashed, the roof windows.
    for (bi, slope) in &roof_slopes {
        let pts: Vec<Pos2> = slope.iter().map(|&q| at(q)).collect();
        let color = if *bi == st.sel { line.gamma_multiply(0.7) } else { weak };
        for k in 0..pts.len() {
            painter.extend(egui::Shape::dashed_line(&[pts[k], pts[(k + 1) % pts.len()]], Stroke::new(1.0, color), 5.0, 3.0));
        }
    }
    for w in &roof_windows {
        let pts: Vec<Pos2> = w.iter().map(|&q| at(q)).collect();
        painter.add(egui::Shape::convex_polygon(pts, GLASS, Stroke::new(1.0, line)));
    }
    for m in &pv_modules {
        let pts: Vec<Pos2> = m.iter().map(|&q| at(q)).collect();
        painter.add(egui::Shape::convex_polygon(pts, PV_MODULE, Stroke::new(0.5, weak)));
    }
    // The stairs reaching the level shown: their steps and the walking line with its arrow.
    for (pieces, walk) in &stair_lines {
        for p in pieces {
            painter.add(egui::Shape::closed_line(p.iter().map(|&q| at(q)).collect(), Stroke::new(1.0, line)));
        }
        let pts: Vec<Pos2> = walk.iter().map(|&q| at(q)).collect();
        painter.add(egui::Shape::line(pts.clone(), Stroke::new(1.5, accent)));
        if let [.., a, b] = pts.as_slice() {
            let d = (*b - *a).normalized() * 7.0;
            painter.add(egui::Shape::convex_polygon(vec![*b, *b - d + d.rot90() * 0.5, *b - d - d.rot90() * 0.5], accent, Stroke::NONE));
        }
    }
    for s in &shafts {
        let pts: Vec<Pos2> = s.iter().map(|&q| at(q)).collect();
        painter.add(egui::Shape::closed_line(pts.clone(), Stroke::new(1.5, accent)));
        painter.line_segment([pts[0], pts[2]], Stroke::new(1.0, accent));
        painter.line_segment([pts[1], pts[3]], Stroke::new(1.0, accent));
    }
    for (path, l) in links.iter().zip(&layout.site_links) {
        let pts: Vec<Pos2> = path.iter().map(|&q| at(q)).collect();
        let color = crate::style::lock();
        for w in pts.windows(2) {
            painter.extend(egui::Shape::dashed_line(&[w[0], w[1]], Stroke::new(2.0, color), 6.0, 4.0));
        }
        if let Some(m) = pts.get(pts.len() / 2) {
            painter.text(*m + vec2(4.0, -4.0), egui::Align2::LEFT_BOTTOM, format!("{} ({})", l.name, l.kind.title()), egui::FontId::proportional(11.0), color);
        }
    }
    for (bi, b) in layout.buildings.iter().enumerate() {
        let corners: Vec<[f32; 2]> = rooms.iter().filter(|r| bld::building_of(layout, r.0) == Some(bi)).flat_map(|r| r.1.iter().copied()).collect();
        let place = if corners.is_empty() { [b.site.x, b.site.z] } else { panel::bounds(&corners).0 };
        let color = if bi == st.sel { accent } else { weak };
        painter.text(at(place) - vec2(0.0, 4.0), egui::Align2::LEFT_BOTTOM, b.name.as_str(), egui::FontId::proportional(13.0), color);
    }
    // Scale bar: 10 m.
    let bar = 10_000.0 * scale;
    if bar > 20.0 && bar < size.width() - 40.0 {
        let p0 = pos2(size.left() + 12.0, size.bottom() - 12.0);
        painter.line_segment([p0, p0 + vec2(bar, 0.0)], Stroke::new(1.5, weak));
        painter.text(p0 + vec2(bar / 2.0, -3.0), egui::Align2::CENTER_BOTTOM, "10 m", egui::FontId::proportional(10.0), weak);
    }
}
