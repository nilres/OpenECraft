use super::*;
use crate::geom::p;
use crate::panel::{collisions, PanelItem};
use crate::analysis::Analysis;
use crate::sample::demo_project;

#[test]
fn explicit_rotations_turn_asymmetric_drilling_without_scaling_hole_spacing() {
    let part=Part {number:"ROTATED".into(),width:20.,height:40.,tech_data:"Mounting holes: M5: 3,7".into(),..Default::default()};
    for (angle,expected) in [(0,p(103.,207.)),(90,p(133.,203.)),(180,p(117.,233.)),(270,p(107.,217.))] {
        let mut dev=item(1,ItemKind::Device,100.,200.,80.,80.);
        dev.part=part.number.clone();dev.rotation=crate::panel::PanelRotation::from_degrees(angle);
        let holes=item_holes(&dev,std::slice::from_ref(&part));
        assert_eq!(holes.len(),1);assert_eq!(holes[0].pos,expected);assert_eq!(holes[0].thread,"M5");
    }
}

fn item(id: u64, kind: ItemKind, x: f32, y: f32, w: f32, h: f32) -> PanelItem {
    PanelItem { id, kind, x, y, w, h, d: 40.0, ..Default::default() }
}

#[test]
fn drill_diameters() {
    assert_eq!(drill_diameter("M5"), Some(4.2));
    assert_eq!(drill_diameter("m4"), Some(3.3));
    assert_eq!(drill_diameter("Ø4.5"), Some(4.5));
    assert_eq!(drill_diameter("6,6"), Some(6.6));
    assert_eq!(drill_diameter("M7"), None);
}

#[test]
fn holes_along_profiles() {
    let v = line_positions(1000.0, 12.5, 200.0);
    assert_eq!(v.len(), 6);
    assert_eq!((v[0], v[5]), (12.5, 987.5));
    assert!(v.windows(2).all(|w| w[1] - w[0] <= 200.0 + 1e-3));
    assert_eq!(line_positions(20.0, 12.5, 200.0), vec![10.0]);
    // A horizontal rail: M5 holes on its centre line.
    let rail = item(1, ItemKind::Rail, 10.0, 100.0, 400.0, 35.0);
    let hs = item_holes(&rail, &[]);
    assert!(hs.iter().all(|h| h.pos.y == 117.5 && h.thread == "M5" && h.diameter == 4.2));
    assert_eq!(hs.first().unwrap().pos.x, 22.5);
    // Busbar systems: a pair of holder holes per position.
    let bb = item(2, ItemKind::Busbar, 0.0, 300.0, 500.0, 75.0);
    let hs = item_holes(&bb, &[]);
    assert_eq!(hs.len(), 2 * line_positions(500.0, 25.0, 250.0).len());
    assert!(hs.iter().all(|h| h.thread == "M6"));
    let e = Enclosure { items: vec![bb], ..Default::default() };
    assert_eq!(busbar_holders(&e), 3);
}

#[test]
fn device_holes_from_part_data() {
    let mut part = Part { number: "T".into(), width: 78.0, height: 84.0, ..Default::default() };
    // Devices on rails need no holes, devices on the plate two by default.
    let mut dev = PanelItem { id: 1, kind: ItemKind::Device, part: "T".into(), x: 100.0, y: 50.0, w: 78.0, h: 84.0, z: 7.5, ..Default::default() };
    assert!(item_holes(&dev, std::slice::from_ref(&part)).is_empty());
    dev.z = 0.0;
    let hs = item_holes(&dev, std::slice::from_ref(&part));
    assert_eq!(hs.iter().map(|h| h.pos).collect::<Vec<_>>(), vec![p(105.0, 55.0), p(173.0, 129.0)]);
    part.tech_data = "Mounting holes: M5: 9,12; 69,72".into();
    let hs = item_holes(&dev, std::slice::from_ref(&part));
    assert_eq!(hs.iter().map(|h| h.pos).collect::<Vec<_>>(), vec![p(109.0, 62.0), p(169.0, 122.0)]);
    assert!(hs.iter().all(|h| h.thread == "M5"));
    // Turned by 90°: the pattern turns with the device.
    dev.w = 84.0;
    dev.h = 78.0;
    let hs = item_holes(&dev, std::slice::from_ref(&part));
    assert_eq!(hs.iter().map(|h| h.pos).collect::<Vec<_>>(), vec![p(100.0 + 84.0 - 12.0, 59.0), p(100.0 + 84.0 - 72.0, 119.0)]);
    for h in &hs {
        assert!(dev.rect().contains(h.pos));
    }
}

#[test]
fn nothing_is_mounted_over_a_cutout() {
    let mut cut = item(1, ItemKind::Cutout, 100.0, 100.0, 50.0, 50.0);
    cut.z = -3.0;
    cut.d = 3.0;
    let on_plate = PanelItem { z: 0.0, ..item(2, ItemKind::Duct, 120.0, 0.0, 40.0, 300.0) };
    let above = PanelItem { z: 7.5, ..item(3, ItemKind::Device, 100.0, 100.0, 45.0, 70.0) };
    let e = Enclosure { items: vec![cut, on_plate, above], ..Default::default() };
    assert_eq!(collisions(&e, &[]), vec![(1, 2), (2, 3)]);
}

#[test]
fn nc_export() {
    let mut prj = demo_project();
    let e = &mut prj.layout.enclosures[0];
    let id = 9999;
    e.items.push(PanelItem { id, kind: ItemKind::Cutout, x: 400.0, y: 600.0, w: 22.5, h: 22.5, round: true, ..Default::default() });
    let (_, ph) = e.plate_size();
    let csv = nc_csv(&prj, 0, Face::Plate);
    let lines: Vec<&str> = csv.lines().collect();
    let n_holes = holes(&prj.layout.enclosures[0], &prj.parts).len();
    assert!(n_holes > 0);
    assert_eq!(lines.len(), 1 + n_holes + 1);
    // Coordinates from the bottom left corner of the plate.
    let cut = lines.last().unwrap();
    assert!(cut.contains("Cutout round"), "{cut}");
    assert!(cut.contains(&format!(";{:.1};", ph - 611.25)), "{cut}");
    let dxf = nc_dxf(&prj, 0, Face::Plate);
    assert!(dxf.contains("\nHOLE_D4.2\n") || dxf.contains("\nHOLE_D3.3\n"));
    assert!(dxf.contains("\nCUTOUT\n"));
    assert!(dxf.contains("\nPLATE\n"));
    assert!(dxf.ends_with("EOF\n"));
    assert_eq!(dxf.matches("0\nCIRCLE\n").count(), n_holes + holes(&prj.layout.enclosures[0], &prj.parts).iter().filter(|h| !h.thread.is_empty()).count() + 1);
}

#[test]
fn wire_end_treatment() {
    assert_eq!(cross_section_mm2("1.5 mm²"), Some(1.5));
    assert_eq!(cross_section_mm2("2,5mm2"), Some(2.5));
    assert_eq!(cross_section_mm2("AWG 14"), None);
    let t = end_treatment("1.5 mm²", true).unwrap();
    assert_eq!((t.strip, t.ferrule.unwrap().color, t.ferrule.unwrap().part), (8.0, "black", "GEN.FER-1.5"));
    assert_eq!(end_treatment("6 mm²", false).unwrap(), EndTreatment { kind: Termination::Bare, strip: 10.0, ferrule: None, crimp: None });
    // Unusual cross-sections get a stripping length but no ferrule.
    assert_eq!(end_treatment("1.2", true).unwrap().ferrule, None);
    assert!(end_treatment("", true).is_none());
    // Every ferrule is in the catalog.
    let catalog = crate::parts::catalog();
    for f in FERRULES {
        assert!(catalog.iter().any(|p| p.number == f.part), "{}", f.part);
    }
}

#[test]
fn wire_ends_seen_in_3d() {
    let outer = crate::harness::outer_diameter("1.5 mm²");
    // With a ferrule: its metal sleeve at the tip, then its collar (black for 1.5 mm²)
    // over the insulation.
    let p = end_treatment("1.5 mm²", true).unwrap().pieces("1.5 mm²", outer);
    assert_eq!(p.len(), 2);
    assert_eq!((p[0].length, p[0].color), (8.0, SLEEVE_RGB));
    assert!(p[0].diameter < outer && p[1].diameter > outer, "{p:?}");
    assert_eq!((p[1].length, p[1].color), (6.0, [35, 35, 38]));
    // Every ferrule colour is known.
    assert!(FERRULES.iter().all(|f| f.rgb() != [128, 128, 128]));
    // Without one: the bare copper as long as it is stripped, thinner than the insulation.
    let p = end_treatment("1.5 mm²", false).unwrap().pieces("1.5 mm²", outer);
    assert_eq!(p.len(), 1);
    assert_eq!((p[0].length, p[0].color), (8.0, COPPER_RGB));
    assert!(p[0].diameter > 1.3 && p[0].diameter < outer, "{p:?}");
}

#[test]
fn ferrules_in_wire_list_and_bom() {
    let mut prj = demo_project();
    for w in prj.pages.iter_mut().flat_map(|pg| pg.wires.iter_mut()).filter(|w| w.cable.is_empty()) {
        w.cross_section = "0.75 mm²".into();
    }
    let an = Analysis::checked(&prj);
    assert!(!an.wiring.is_empty());
    let list = crate::reports::production_wire_list(&prj, &an);
    let col = list.headers.iter().position(|h| *h == "Terminal parts").unwrap();
    assert!(list.rows.iter().any(|r| r[col] == "GEN.FER-0.75"));
    // Routed (panel) wires come first.
    let len = list.headers.iter().position(|h| *h == "Length (mm)").unwrap();
    let first_unrouted = list.rows.iter().position(|r| r[len].is_empty()).unwrap_or(list.rows.len());
    assert!(list.rows[first_unrouted..].iter().all(|r| r[len].is_empty()));
    let bom = &an.report(&prj, crate::model::ReportKind::BillOfMaterials).unwrap();
    let row = bom.rows.iter().find(|r| r[3] == "GEN.FER-0.75").expect("ferrules in the BOM");
    assert_eq!(row[1], (2 * an.wiring.iter().filter(|w| w.cross_section == "0.75 mm²").count()).to_string());
    assert!(!row[6].contains("not in project"), "described from the catalog");
    prj.layout.ferrules = false;
    let an = Analysis::checked(&prj);
    assert!(!an.report(&prj, crate::model::ReportKind::BillOfMaterials).unwrap().rows.iter().any(|r| r[3] == "GEN.FER-0.75"));
}

#[test]
fn door_cutouts_and_nc_data() {
    let prj = demo_project();
    let e = &prj.layout.enclosures[0];
    let door = openings(e, &prj.parts, Face::Door);
    // One cutout per door device: Ø22.5 for command and signal devices.
    assert_eq!(door.len(), 6);
    assert!(door.iter().all(|o| o.round && o.w == 22.5));
    // The door has no drilling pattern of the plate and vice versa.
    assert!(openings(e, &prj.parts, Face::Plate).is_empty());
    let csv = nc_csv(&prj, 0, Face::Door);
    assert_eq!(csv.lines().count(), 1 + 6);
    assert!(csv.lines().skip(1).all(|l| l.contains(";Door;") && l.contains("Cutout round")));
    let dxf = nc_dxf(&prj, 0, Face::Door);
    assert!(dxf.contains("\nDOOR\n") && !dxf.contains("HOLE_D"));
    assert_eq!(dxf.matches("0\nCIRCLE\n").count(), 6);
    // Part data overrides the default: rectangular cutout.
    let part = Part { number: "D".into(), tech_data: "Door cutout: 92 x 68 mm".into(), ..Default::default() };
    let it = item(1, ItemKind::Device, 0.0, 0.0, 96.0, 72.0);
    assert_eq!(door_cutout(Some(&part), &it), (false, 92.0, 68.0));
    let part = Part { number: "D".into(), tech_data: "Door cutout: Ø30.5".into(), ..Default::default() };
    assert_eq!(door_cutout(Some(&part), &it), (true, 30.5, 30.5));
    assert_eq!(door_cutout(None, &it), (false, 92.0, 68.0));
}

#[test]
fn bars_sit_at_their_centre_distance() {
    let parts = crate::parts::catalog();
    // 185 mm apart in the middle of the 400 mm wide system.
    assert_eq!(bar_centres(parts, "GEN.BB-3X30X10-185", 400.0), vec![15.0, 200.0, 385.0]);
    assert_eq!(bar_centres(parts, "GEN.BB-3X30X10-60", 150.0), vec![15.0, 75.0, 135.0]);
    // Without a centre distance, or one the system is too narrow for: evenly spaced.
    assert_eq!(bar_centres(parts, "GEN.BB-3X30X10", 90.0), vec![15.0, 45.0, 75.0]);
    assert_eq!(bar_centres(parts, "GEN.BB-3X30X10-185", 300.0), vec![50.0, 150.0, 250.0]);
}

#[test]
fn terminations_are_chosen_per_wire_end() {
    let mut prj = demo_project();
    let an = Analysis::checked(&prj);
    let w = an.wiring.iter().find(|w| crate::production::cross_section_mm2(&w.cross_section).is_some()).expect("a routed wire").clone();
    let cs = w.cross_section.clone();
    // Without a choice: the layout setting.
    assert_eq!(termination_of(&prj, &w.from, &w.to), None);
    assert_eq!(end_at(&prj, &w.from, &w.to, &cs).unwrap().kind, Termination::Ferrule);
    // For the device, for the connection point, for the one wire: the nearest counts.
    let device = w.from.rsplit_once(':').unwrap().0.to_string();
    set_termination(&mut prj, &device, "", Some(Termination::Pin));
    assert_eq!(termination_of(&prj, &w.from, &w.to), Some(Termination::Pin));
    set_termination(&mut prj, &w.from, "", Some(Termination::Fork));
    assert_eq!(termination_of(&prj, &w.from, &w.to), Some(Termination::Fork));
    set_termination(&mut prj, &w.from, &w.to, Some(Termination::Ring));
    assert_eq!(termination_of(&prj, &w.from, &w.to), Some(Termination::Ring));
    assert_eq!(termination_of(&prj, &w.from, "-Z9:1"), Some(Termination::Fork), "another wire at the same point");
    assert_eq!(termination_of(&prj, &w.to, &w.from), None, "the other end keeps the setting");
    // Choosing again replaces, None goes back.
    set_termination(&mut prj, &w.from, &w.to, Some(Termination::Blade));
    assert_eq!(prj.layout.terminations.iter().filter(|t| t.at == w.from && t.to == w.to).count(), 1);
    set_termination(&mut prj, &w.from, &w.to, None);
    assert_eq!(termination_of(&prj, &w.from, &w.to), Some(Termination::Fork));
    // Renaming the device keeps them.
    crate::ops::rename_device(&mut prj, &device, "-Q77");
    let renamed = format!("-Q77:{}", w.from.rsplit_once(':').unwrap().1);
    assert_eq!(termination_of(&prj, &renamed, &w.to), Some(Termination::Fork));
    assert!(prj.layout.terminations.iter().any(|t| t.at == "-Q77"));
    // Keys round trip, the file keeps them.
    for t in Termination::ALL {
        assert_eq!(Termination::from_key(t.key()), Some(t));
    }
    assert_eq!(Termination::from_key("Twin ferrule"), Some(Termination::TwinFerrule));
    let json = serde_json::to_string(&prj.layout.terminations).unwrap();
    assert!(json.contains("\"fork\""), "{json}");
}

#[test]
fn crimp_terminals_by_cross_section() {
    let part = |cs: &str, kind| treatment(cs, kind).unwrap().part();
    assert_eq!(part("0.75 mm²", Termination::Ring), "GEN.RING-R");
    assert_eq!(part("1.5 mm²", Termination::Fork), "GEN.FORK-B");
    assert_eq!(part("6 mm²", Termination::Receptacle), "GEN.REC-Y");
    assert_eq!(part("16 mm²", Termination::Ring), "GEN.LUG-16");
    assert_eq!(part("1.5 mm²", Termination::TwinFerrule), "GEN.FER2-1.5");
    assert_eq!(part("1.5 mm²", Termination::Bare), "");
    // No flat plug for 16 mm²: stripped, no part, said so.
    let t = treatment("16 mm²", Termination::Blade).unwrap();
    assert_eq!((t.crimp, t.part().as_str()), (None, ""));
    assert!(t.describe().contains(Termination::Blade.title()));
    // The crimp barrel sets the stripping length.
    assert_eq!(treatment("2.5 mm²", Termination::Pin).unwrap().strip, 7.0);
    // Every part is in the catalog.
    let catalog = crate::parts::catalog();
    for cs in ["0.5", "1", "1.5", "2.5", "4", "6", "10", "16", "25", "35", "50", "70", "95", "120", "150", "185", "240"] {
        for kind in Termination::ALL {
            let Some(t) = treatment(&format!("{cs} mm²"), kind) else { continue };
            let p = t.part();
            assert!(p.is_empty() || catalog.iter().any(|c| c.number == p), "{p} for {cs} mm²");
        }
    }
}

#[test]
fn terminals_seen_in_3d() {
    let outer = crate::harness::outer_diameter("1.5 mm²");
    let (tip, dir, up) = ([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]);
    for kind in Termination::ALL {
        let p = treatment("1.5 mm²", kind).unwrap().pieces("1.5 mm²", outer);
        assert!(!p.is_empty() && p.iter().all(|q| q.length > 0.0 && q.diameter > 0.0), "{kind:?}: {p:?}");
        // Flat pieces have their triangles, near the tip and as thick as they are.
        for q in p.iter().filter(|q| q.shape != EndShape::Round) {
            let tris = q.mesh(tip, dir, up);
            assert!(!tris.is_empty(), "{kind:?}");
            let reach = q.length + q.diameter;
            assert!(tris.iter().flatten().all(|v| v.iter().map(|c| c.abs()).fold(0.0, f32::max) <= reach), "{kind:?}");
            assert!(tris.iter().flatten().all(|v| v[2].abs() <= 1.6), "{kind:?} lies flat");
        }
    }
    // A ring terminal: the ring round the tip, then the blue insulated sleeve over the
    // insulation.
    let p = treatment("1.5 mm²", Termination::Ring).unwrap().pieces("1.5 mm²", outer);
    assert!(matches!(p[0].shape, EndShape::Ring { hole, .. } if (hole - 4.3).abs() < 1e-3));
    assert_eq!(p[1].color, [40, 90, 200]);
    assert!(p[1].diameter > outer);
    // The ring's triangles face outwards: those on top look up.
    let tris = p[0].mesh(tip, dir, up);
    let top: Vec<_> = tris.iter().filter(|t| t.iter().all(|v| (v[2] - 0.4).abs() < 1e-3)).collect();
    assert!(!top.is_empty());
    for t in top {
        let n = crate::parts::vec3::cross(crate::parts::vec3::sub(t[1], t[0]), crate::parts::vec3::sub(t[2], t[0]));
        assert!(n[2] > 0.0, "{t:?}");
    }
}

#[test]
fn terminals_in_the_bill_of_materials() {
    let mut prj = demo_project();
    for w in prj.pages.iter_mut().flat_map(|pg| pg.wires.iter_mut()).filter(|w| w.cable.is_empty()) {
        w.cross_section = "1.5 mm²".into();
    }
    let an = Analysis::checked(&prj);
    let ends = 2 * an.wiring.iter().filter(|w| w.cross_section == "1.5 mm²").count();
    // Every wire end at one device gets a ring terminal, one wire end a twin ferrule.
    let w = an.wiring.iter().find(|w| w.cross_section == "1.5 mm²").unwrap().clone();
    let device = w.from.rsplit_once(':').unwrap().0.to_string();
    set_termination(&mut prj, &device, "", Some(Termination::Ring));
    set_termination(&mut prj, &w.to, &w.from, Some(Termination::TwinFerrule));
    let rings: usize = an.wiring.iter().filter(|x| x.cross_section == "1.5 mm²").map(|x| [&x.from, &x.to].iter().filter(|e| e.starts_with(&format!("{device}:"))).count()).sum();
    let material = terminal_material(&prj, &an.wiring);
    let count = |p: &str| material.iter().find(|m| m.0 == p).map(|m| m.1).unwrap_or(0);
    assert_eq!(count("GEN.RING-B"), rings);
    assert_eq!(count("GEN.FER2-1.5"), 1, "a twin ferrule for a single end too");
    assert_eq!(count("GEN.FER-1.5") + rings + 1, ends);
    let an = Analysis::checked(&prj);
    let bom = an.report(&prj, crate::model::ReportKind::BillOfMaterials).unwrap();
    assert!(bom.rows.iter().any(|r| r[3] == "GEN.RING-B"));
    // The wire list shows both ends.
    let list = crate::reports::production_wire_list(&prj, &an);
    let col = |name: &str| list.headers.iter().position(|h| *h == name).unwrap();
    let row = list.rows.iter().find(|r| r[1] == w.from && r[2] == w.to || r[1] == w.to && r[2] == w.from).expect("the wire");
    let (from_end, to_end) = (row[col("Termination (from)")].clone(), row[col("Termination (to)")].clone());
    assert!(from_end.contains(Termination::Ring.title()) || to_end.contains(Termination::Ring.title()), "{row:?}");
    assert!(row[col("Terminal parts")].contains(" / "), "{row:?}");
}
