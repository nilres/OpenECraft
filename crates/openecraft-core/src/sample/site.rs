//! Level 15: the pump room of level 10 in a plant of two buildings. The main building
//! +B1 has the electrical room +ER1 on its ground level and the control room +CR1 on
//! the level above, joined by the shaft S1; the pump house +B2 holds pump motor 2 and
//! is linked to the main building by a cable trench between their building entries,
//! through which the motor cable of pump 2 is routed. Façade lights hang on the
//! outside walls, a weather station stands on the roof.

use super::*;
use crate::geom::p;
use crate::model::panel::{LinkKind, Opening, SitePlace};
use crate::panel::{self, buildings as bld, Face};
use crate::parts::{MountingData, Part};

/// Façade light (a project part made for wall mounting) and weather station (stands on
/// the roof).
pub(super) const FACADE_LIGHT: &str = "PRJ.FACADE-LIGHT";
pub(super) const WEATHER: &str = "PRJ.WEATHER-STATION";
/// The motor of pump 2, standing in the pump house.
pub(super) const MOTOR_2: &str = "=P2+FLD-M1";

pub(super) fn project() -> Project {
    let mut prj = super::pump_room::project();
    prj.info.name = "Pump station plant".into();
    prj.info.description = "Two buildings with levels,\na shaft and a cable trench".into();
    prj.structures.define(crate::structure::Aspect::Location, "CR1", "Control room");
    prj.structures.define(crate::structure::Aspect::Location, "PH1", "Pump house");
    prj.parts.push(Part {
        number: FACADE_LIGHT.into(),
        description: "Façade light LED 20 W, IP65, for wall mounting".into(),
        group: "Installation devices".into(),
        subgroup: "Light".into(),
        width: 260.0,
        height: 160.0,
        depth: 120.0,
        price: 64.0,
        mounting_data: MountingData { wall_mount: true, ..Default::default() },
        ..Default::default()
    });
    prj.parts.push(Part {
        number: WEATHER.into(),
        description: "Weather station on a mast: wind, temperature, rain".into(),
        group: "Sensors".into(),
        subgroup: "Weather station".into(),
        width: 400.0,
        height: 1500.0,
        depth: 400.0,
        price: 420.0,
        mounting: "Floor".into(),
        mounting_data: MountingData { floor_mount: true, ..Default::default() },
        ..Default::default()
    });
    let er = prj.layout.enclosures.iter().position(|e| e.name == "+ER1").expect("electrical room");

    // Main building at the origin of the plant: the electrical room on the ground
    // level (3.5 m to the floor above), the control room as large as the building above
    // it, the shaft in the battery corner behind the inner wall.
    let b1 = bld::add_building(&mut prj, "+B1").expect("plant");
    prj.layout.buildings[b1].description = "Main building".into();
    prj.layout.buildings[b1].site = SitePlace::default();
    let l0 = prj.layout.buildings[b1].levels[0].id;
    bld::set_level(&mut prj, b1, l0, None, None, Some(3500.0)).expect("plant");
    let l1 = bld::add_level(&mut prj, b1, None, None, None).expect("plant");
    bld::set_room_level(&mut prj, er, Some(l0), Some(SitePlace::default())).expect("plant");
    let control = panel::new_room(&mut prj, "+CR1", 6000.0, 4500.0, 2700.0);
    prj.layout.enclosures.push(control);
    let cr = prj.layout.enclosures.len() - 1;
    bld::set_room_level(&mut prj, cr, Some(l1), Some(SitePlace::default())).expect("plant");
    panel::add_opening(&mut prj, cr, Opening::Window, Face::Wall(2), None, None, None).expect("plant");
    bld::add_shaft(&mut prj, b1, Some("S1"), 5300.0, 100.0, 600.0, 600.0, l0, l1).expect("plant");

    // Pump house beside the main building, its door in the front wall.
    let b2 = bld::add_building(&mut prj, "+B2").expect("plant");
    prj.layout.buildings[b2].description = "Pump house".into();
    let level = prj.layout.buildings[b2].levels[0].id;
    let house = panel::new_room(&mut prj, "+PH1", 4000.0, 3000.0, 2700.0);
    prj.layout.enclosures.push(house);
    let ph = prj.layout.enclosures.len() - 1;
    bld::set_room_level(&mut prj, ph, Some(level), Some(SitePlace::default())).expect("plant");
    panel::add_opening(&mut prj, ph, Opening::Door, Face::Wall(2), Some(1500.0), None, None).expect("plant");

    // Building entries in the right wall of the electrical room (beside the door
    // swing) and the left wall of the pump house, the cable trench between them at
    // right angles.
    let a = panel::add_opening(&mut prj, er, Opening::Entry, Face::Wall(1), Some(3400.0), None, None).expect("plant");
    let b = panel::add_opening(&mut prj, ph, Opening::Entry, Face::Wall(3), Some(1400.0), None, None).expect("plant");
    let x = prj.layout.buildings[b2].site.x;
    let mid = (6000.0 + x) / 2.0;
    bld::add_site_link(&mut prj, a, b, LinkKind::Trench, vec![[mid, 3500.0], [mid, 1500.0]], Some("SL1")).expect("plant");

    // Pump motor 2 stands in the pump house now.
    prj.layout.enclosures[er].items.retain(|i| i.tag != MOTOR_2);
    panel::mount_on_wall(&mut prj, ph, MOTOR_2, Face::Bottom, p(1800.0, 1200.0)).expect("plant");

    // On the outside: a façade light on the rear wall of the main building and one
    // above the door of the pump house, the weather station on the roof.
    panel::mount_on_wall(&mut prj, er, FACADE_LIGHT, Face::Outside(0), p(2900.0, 300.0)).expect("plant");
    panel::mount_on_wall(&mut prj, ph, FACADE_LIGHT, Face::Outside(2), p(1870.0, 300.0)).expect("plant");
    panel::mount_on_wall(&mut prj, cr, WEATHER, Face::RoofTop, p(5000.0, 3500.0)).expect("plant");
    number_wires(&mut prj);
    prj
}
