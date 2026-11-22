//! Level 8: the pump station (level 6) built into a line-up of two floor-standing
//! enclosures: +CAB1 for the infeed, pump 1 and the PLC, +CAB2 for the frequency
//! converter of pump 2. Busbar system, mounting rails and cable ducts, door devices
//! (fault lamps, the analog voltmeter of the supply and ammeter of pump 1, the digital
//! ammeter of pump 2) and handles, filter fan and outlet filter, light, PE bar, gland plate with cable glands,
//! a cable passage between the enclosures, the climate calculation and the reports of
//! the panel layout (duct fill, climate, 3D routing).

use super::*;
use crate::model::climate::{Climate, Installation};
use crate::panel::{self, Face, ItemKind};

pub(super) const CAB1: &str = "GEN.TS-800X2000X500";
pub(super) const CAB2: &str = "GEN.TS-600X2000X500";

pub(super) fn project() -> Project {
    let mut prj = super::plant::project();
    prj.info.name = "Pump station cabinets".into();
    prj.info.description = "Two-bay line-up\nwith climate and routing".into();
    move_pump2_to_cab2(&mut prj);
    let db = PartsDb::default();
    motor_cables(&mut prj, &db);
    meters(&mut prj, &db);
    for part in [CAB1, CAB2, panel::RAIL_PART, panel::DUCT_PART, panel::DOOR_DUCT_PART, panel::BUSBAR_PART] {
        crate::parts::ensure_in_project(&mut prj, &db, part);
    }
    for part in ["GEN.FAN-FILTER-120", "GEN.FAN-OUTLET-120", "GEN.GLAND-M20", "GEN.LIGHT-LED-350", "GEN.PE-BAR-10", "GEN.HANDLE-SWING", "GEN.CLAMP-RAIL-500"] {
        crate::parts::ensure_in_project(&mut prj, &db, part);
    }
    cab1(&mut prj);
    cab2(&mut prj);
    // +CAB2 stands right of +CAB1; the wires between them run through a passage.
    panel::bay_right_of(&mut prj, 1, 0);
    // In front of the cable ducts, clear of the 40 mm frame of the side walls.
    panel::add_passage(&mut prj, 0, 300.0, 120.0, panel::PASSAGE_DIAMETER);
    prj.layout.enclosures[0].climate =
        Climate { installation: Installation::EndFreeStanding, ambient_max: 35.0, ambient_min: 5.0, inside_max: 40.0, inside_min: 5.0, ..Default::default() };
    prj.layout.enclosures[1].climate =
        Climate { installation: Installation::EndFreeStanding, ambient_max: 35.0, ambient_min: 5.0, inside_max: 40.0, inside_min: 5.0, ..Default::default() };

    let reports = prj.sections.iter().find(|s| s.description == "Reports").map(|s| s.id).unwrap_or(0);
    let layout = crate::sections::add(&mut prj, 0, "Panel layout");
    let ids: Vec<(u64, String)> = prj.layout.enclosures.iter().map(|e| (e.id, e.name.clone())).collect();
    for (n, (id, name)) in ids.iter().enumerate() {
        page_in(&mut prj, layout, &(n + 1).to_string(), &format!("Mounting layout {name}"), PageKind::Mounting(*id));
    }
    for (n, desc, kind) in [
        ("7", "Cable overview", ReportKind::CableOverview),
        ("8", "Duct fill levels", ReportKind::DuctFill),
        ("9", "Enclosure climate", ReportKind::Climate),
        ("10", "Panel legend", ReportKind::PanelLegend),
        ("11", "3D routing", ReportKind::RoutingList),
    ] {
        page_in(&mut prj, reports, n, desc, PageKind::Report(kind.into()));
    }
    crate::sections::sort_pages(&mut prj);
    prj
}

/// A page at the end of a section.
fn page_in(prj: &mut Project, section: u64, n: &str, desc: &str, kind: PageKind) {
    let name = format!("{}.{n}", crate::sections::full_number(prj, section));
    let i = add_page(prj, &name, desc, kind);
    prj.pages[i].section = section;
}

/// The devices of pump 2 in the cabinet move to the second enclosure +CAB2.
/// Meters in the doors: the supply voltage behind its own MCB -F3 on the infeed page
/// (analog), the currents of the pumps in their L1 before their protective devices
/// (analog for pump 1, digital for pump 2).
fn meters(prj: &mut Project, db: &PartsDb) {
    let page = |prj: &Project, desc: &str| prj.pages.iter().position(|p| p.description == desc).expect(desc);
    let supply = page(prj, "Infeed and 24 V supply");
    let mut g = Pg { prj, page: supply };
    g.sym("cb1", 120.0, 80.0, 0, "-F3", "", "Measuring").w(120.0, 24.0, 120.0, 76.0).w(120.0, 84.0, 120.0, 92.0);
    g.sym("voltmeter", 120.0, 96.0, 0, "-P1", "", "Supply voltage");
    g.path(&[(120.0, 100.0), (120.0, 108.0), (132.0, 108.0), (132.0, 48.0)]);
    for desc in ["Power pump 1", "Power pump 2"] {
        let pi = page(prj, desc);
        let feed = |w: &Wire| w.a.x == 60.0 && w.b.x == 60.0 && w.a.y.min(w.b.y) == 24.0 && w.a.y.max(w.b.y) == 76.0;
        prj.pages[pi].wires.retain(|w| !feed(w));
        let mut g = Pg { prj, page: pi };
        g.sym("ammeter", 60.0, 68.0, 0, "-P2", "", "Current").w(60.0, 24.0, 60.0, 64.0).w(60.0, 72.0, 60.0, 76.0);
    }
    assign_parts(prj, db, &[("=SUP+CAB1-F3", &["GEN.MCB-1P-C6"][..]), ("=SUP+CAB1-P1", &["GEN.VOLT-96-500V"]), ("=P1+CAB1-P2", &["GEN.AMM-72-5A"]), ("=P2+CAB2-P2", &["GEN.DAM-72-5A"])]);
    number_wires(prj);
}

fn move_pump2_to_cab2(prj: &mut Project) {
    prj.structures.define(crate::structure::Aspect::Location, "CAB2", "Control cabinet, converter bay");
    for p in prj.pages.iter_mut().filter(|p| p.function == "P2") {
        p.location = "CAB2".into();
    }
    let tags: Vec<String> = prj.all_tags().into_iter().filter(|t| t.starts_with("=P2+CAB1-")).collect();
    for old in tags {
        let new = old.replace("+CAB1-", "+CAB2-");
        crate::ops::rename_device(prj, &old, &new);
    }
}

/// Motor cables from the terminals -X1 to the pumps in the field.
fn motor_cables(prj: &mut Project, db: &PartsDb) {
    let part = db.get("GEN.CBL-4G1.5").expect("cable part").clone();
    crate::parts::ensure_in_project(prj, db, &part.number);
    for (tag, page, pe_x) in [("=P1-W1", "3.1", 84.0), ("=P2-W1", "4.1", 92.0)] {
        let id = prj.new_id();
        prj.cables.push(Cable {
            id,
            tag: tag.into(),
            cable_type: part.type_number.clone(),
            cores: part.tech_value("Cores").unwrap_or_default().into(),
            cross_section: part.tech_value("Cross-section").unwrap_or_default().into(),
            length: 25.0,
            part: part.number.clone(),
            description: "Pump motor".into(),
            ..Default::default()
        });
        let pi = prj.pages.iter().position(|p| p.name == page).expect("power page");
        let cores = [(60.0, "BN"), (68.0, "BK"), (76.0, "GY"), (pe_x, "GNYE")];
        for w in prj.pages[pi].wires.iter_mut().filter(|w| w.a.y == 144.0) {
            if let Some((_, core)) = cores.iter().find(|c| c.0 == w.a.x) {
                w.cable = tag.into();
                w.core = core.to_string();
            }
        }
    }
}

/// Devices side by side on a rail at height `rail_y`, from `x` with a gap.
fn on_rail(prj: &mut Project, enc: usize, rail_y: f32, x: f32, tags: &[&str]) {
    let mut x = x;
    for tag in tags {
        let (w, h) = panel::device_size(prj, tag).map(|d| (d.w, d.h)).unwrap_or((40.0, 40.0));
        panel::place_device(prj, enc, tag, crate::geom::p(x, rail_y - h / 2.0));
        x += panel::snap(w + 10.0);
    }
}

fn profile(prj: &mut Project, enc: usize, kind: ItemKind, part: &str, a: (f32, f32), b: (f32, f32)) {
    let item = panel::new_profile(prj, kind, part, crate::geom::p(a.0, a.1), crate::geom::p(b.0, b.1));
    prj.layout.enclosures[enc].items.push(item);
}

/// Duct frame around the plate (`w` × 1950 mm) with horizontal ducts between the rails.
fn ducts_and_rails(prj: &mut Project, enc: usize, w: f32, rails: &[f32], ducts: &[f32]) {
    let d = panel::DUCT_PART;
    profile(prj, enc, ItemKind::Duct, d, (0.0, 20.0), (w, 20.0));
    profile(prj, enc, ItemKind::Duct, d, (0.0, 1930.0), (w, 1930.0));
    profile(prj, enc, ItemKind::Duct, d, (20.0, 40.0), (20.0, 1910.0));
    profile(prj, enc, ItemKind::Duct, d, (w - 20.0, 40.0), (w - 20.0, 1910.0));
    for &y in ducts {
        profile(prj, enc, ItemKind::Duct, d, (40.0, y), (w - 40.0, y));
    }
    for &y in rails {
        profile(prj, enc, ItemKind::Rail, panel::RAIL_PART, (60.0, y), (w - 60.0, y));
    }
}

fn enclosure(prj: &mut Project, part: &str, name: &str) -> usize {
    let mut e = panel::new_enclosure(prj, part, name);
    e.build.gland_plate = true;
    prj.layout.enclosures.push(e);
    prj.layout.enclosures.len() - 1
}

fn door_device(prj: &mut Project, enc: usize, tag: &str, x: f32, y: f32) {
    panel::place_device_on(prj, enc, tag, crate::geom::p(x, y), Face::Door);
}

/// Infeed, pump 1 and the PLC.
fn cab1(prj: &mut Project) {
    let e = enclosure(prj, CAB1, "+CAB1");
    ducts_and_rails(prj, e, 750.0, &[380.0, 720.0, 1050.0, 1400.0], &[230.0, 560.0, 900.0, 1230.0, 1580.0]);
    // Busbar system above the first duct (bars and holders in the bill of materials).
    profile(prj, e, ItemKind::Busbar, panel::BUSBAR_PART, (60.0, 120.0), (690.0, 120.0));
    on_rail(prj, e, 380.0, 60.0, &["=SUP+CAB1-F1", "=SUP+CAB1-F3", "=SUP+CAB1-T1", "=P1+CAB1-Q1", "=P1+CAB1-K1"]);
    on_rail(prj, e, 720.0, 60.0, &["=PLC+CAB1-A1", "=PLC+CAB1-A2", "=PLC+CAB1-A3"]);
    on_rail(prj, e, 1050.0, 60.0, &["=P1+CAB1-X1"]);
    // PE bar and strain relief above the gland plate.
    panel::place_accessory(prj, e, "GEN.PE-BAR-10", Face::Plate, crate::geom::p(80.0, 1700.0));
    panel::place_accessory(prj, e, "GEN.CLAMP-RAIL-500", Face::Plate, crate::geom::p(80.0, 1800.0));
    // Door: fault lamp, handle; the filter fan blows in at the bottom of the left side,
    // the outlet filter lets the air out at its top.
    door_device(prj, e, "=SUP+CAB1-P1", 200.0, 640.0);
    door_device(prj, e, "=P1+CAB1-P2", 380.0, 652.0);
    door_device(prj, e, "=P1+CAB1-P1", 380.0, 900.0);
    panel::place_accessory(prj, e, "GEN.HANDLE-SWING", Face::Door, crate::geom::p(690.0, 900.0));
    panel::place_accessory(prj, e, "GEN.FAN-FILTER-120", Face::Left, crate::geom::p(160.0, 1600.0));
    panel::place_accessory(prj, e, "GEN.FAN-OUTLET-120", Face::Left, crate::geom::p(160.0, 200.0));
    panel::place_accessory(prj, e, "GEN.LIGHT-LED-350", Face::Roof, crate::geom::p(220.0, 260.0));
    panel::place_accessory(prj, e, "GEN.GLAND-M20", Face::Bottom, crate::geom::p(120.0, 300.0));
}

/// Pump 2 with its frequency converter.
fn cab2(prj: &mut Project) {
    let e = enclosure(prj, CAB2, "+CAB2");
    // The frequency converter needs 100 mm free above and below (ventilation).
    ducts_and_rails(prj, e, 550.0, &[380.0, 830.0, 1280.0], &[230.0, 560.0, 1100.0, 1430.0]);
    on_rail(prj, e, 380.0, 60.0, &["=P2+CAB2-F1"]);
    on_rail(prj, e, 830.0, 60.0, &["=P2+CAB2-T1"]);
    on_rail(prj, e, 1280.0, 60.0, &["=P2+CAB2-X1"]);
    door_device(prj, e, "=P2+CAB2-P2", 280.0, 652.0);
    door_device(prj, e, "=P2+CAB2-P1", 280.0, 900.0);
    panel::place_accessory(prj, e, "GEN.HANDLE-SWING", Face::Door, crate::geom::p(490.0, 900.0));
    panel::place_accessory(prj, e, "GEN.GLAND-M20", Face::Bottom, crate::geom::p(120.0, 300.0));
}
