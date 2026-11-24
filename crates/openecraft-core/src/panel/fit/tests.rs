use super::*;
use crate::panel::{new_enclosure, PanelItem};
use crate::parts::{bundled, PartsDb};

/// The Striebel & John wall cabinet of 3 field widths and 7 rows, its distribution
/// panel of one field (FV17) and of two fields.
const CABINET: &str = "SJ.2CPX052032R9999";
const FV17: &str = "SJ.2CPX054002R9999";

fn sj_project() -> (Project, usize) {
    let mut prj = Project { parts: bundled::load("striebel_john").unwrap().parts, ..Default::default() };
    let e = new_enclosure(&mut prj, CABINET, "+Z1");
    prj.layout.enclosures.push(e);
    (prj, 0)
}

fn part<'a>(prj: &'a Project, number: &str) -> &'a Part {
    prj.parts.iter().find(|p| p.number == number).unwrap()
}

fn slots(places: &[Place]) -> Vec<(f32, f32, f32, f32)> {
    places.iter().filter_map(|pl| if let Place::Slot(r) = pl { Some((r.min.x, r.min.y, r.width(), r.height())) } else { None }).collect()
}

#[test]
fn a_distribution_panel_takes_the_free_fields_of_its_cabinet() {
    let (mut prj, ei) = sj_project();
    let e = &prj.layout.enclosures[ei];
    assert_eq!(field_grid(&prj, e), Some((3, 7, 250.0, 150.0)));
    let fv17 = part(&prj, FV17).clone();
    assert_eq!(slots(&places(&prj, ei, &fv17)), [(0.0, 0.0, 250.0, 1050.0), (250.0, 0.0, 250.0, 1050.0), (500.0, 0.0, 250.0, 1050.0)]);
    // Into the middle field: the other two stay free.
    let db = PartsDb::default();
    let middle = places(&prj, ei, &fv17)[1].clone();
    let id = put(&mut prj, &db, ei, FV17, &middle, p(0.0, 0.0)).unwrap();
    let it = prj.layout.enclosures[ei].item(id).unwrap();
    assert_eq!((it.kind, it.face, it.x, it.y, it.w, it.h), (ItemKind::Accessory, Face::Plate, 250.0, 0.0, 250.0, 1050.0));
    assert_eq!(slots(&places(&prj, ei, &fv17)), [(0.0, 0.0, 250.0, 1050.0), (500.0, 0.0, 250.0, 1050.0)]);
    // A panel of two fields no longer fits anywhere.
    let two = prj.parts.iter().find(|p| crate::parts::count_value(p, "Fields") == Some(2) && crate::parts::count_value(p, "Rows") == Some(7) && crate::parts::accessory::fits(p, part(&prj, CABINET))).unwrap().clone();
    assert!(slots(&places(&prj, ei, &two)).is_empty());
    // The place under the pointer.
    let found = places(&prj, ei, &fv17);
    let e = &prj.layout.enclosures[ei];
    assert_eq!(place_at(e, &found, Face::Plate, p(600.0, 500.0)), Some(&found[1]));
    assert_eq!(place_at(e, &found, Face::Plate, p(300.0, 500.0)), None);
    assert_eq!(place_at(e, &found, Face::Door, p(600.0, 500.0)), None);
    // All fields taken: no place left, though it is the cabinet's accessory.
    for pl in found {
        put(&mut prj, &db, ei, FV17, &pl, p(0.0, 0.0)).unwrap();
    }
    assert!(places(&prj, ei, &fv17).is_empty());
    assert!(fits_enclosure(&prj, ei, &fv17));
    assert!(!fits_enclosure(&prj, ei, crate::parts::catalog().iter().find(|p| p.number == "GEN.GLAND-M20").unwrap()));
}

#[test]
fn parts_of_other_cabinets_and_other_parts_fit_nowhere() {
    let (prj, ei) = sj_project();
    // A panel of five rows is made for the cabinets of five rows.
    let five = prj.parts.iter().find(|p| p.subgroup == "Distribution panel" && crate::parts::count_value(p, "Rows") == Some(5)).unwrap();
    assert!(places(&prj, ei, five).is_empty());
    let unrelated = crate::parts::catalog().iter().find(|p| p.number == "GEN.GLAND-M20").unwrap();
    assert!(places(&prj, ei, unrelated).is_empty());
}

#[test]
fn an_enclosure_accessory_without_fields_goes_onto_its_faces() {
    let mut prj = Project::default();
    let gland = crate::parts::catalog().iter().find(|p| p.number == "GEN.GLAND-M20").unwrap().clone();
    let mut e = new_enclosure(&mut prj, crate::panel::ENCLOSURE_PART, "+CAB1");
    // The enclosure lists the gland as its accessory.
    let mut body = crate::parts::catalog().iter().find(|p| p.number == crate::panel::ENCLOSURE_PART).unwrap().clone();
    body.accessories.push(gland.number.clone());
    prj.parts.push(body);
    e.part = crate::panel::ENCLOSURE_PART.into();
    prj.layout.enclosures.push(e);
    let found = places(&prj, 0, &gland);
    let faces: Vec<Face> = allowed_faces(&gland).into_iter().filter(|f| prj.layout.enclosures[0].faces().contains(f)).collect();
    assert_eq!(found, faces.into_iter().map(Place::Face).collect::<Vec<_>>());
    let face = found[0].face(&prj.layout.enclosures[0]).unwrap();
    let id = put(&mut prj, &PartsDb::default(), 0, &gland.number, &found[0], p(200.0, 100.0)).unwrap();
    let it = prj.layout.enclosures[0].item(id).unwrap();
    assert_eq!((it.face, it.part.as_str()), (face, "GEN.GLAND-M20"));
    assert!((it.x + it.w / 2.0 - 200.0).abs() <= 5.0, "{}", it.x);
}

#[test]
fn an_accessory_of_a_device_goes_onto_the_placed_device() {
    let mut prj = Project::default();
    let db = PartsDb { parts: bundled::default_parts() };
    crate::parts::add_to_device(&mut prj, &db, "-F1", "HAG.CCA216D", 1);
    crate::parts::add_to_device(&mut prj, &db, "-F2", "HAG.MBN106", 1);
    let mut e = new_enclosure(&mut prj, crate::panel::ENCLOSURE_PART, "+CAB1");
    for (id, tag, part) in [(1, "-F1", "HAG.CCA216D"), (2, "-F2", "HAG.MBN106")] {
        e.items.push(PanelItem { id, kind: ItemKind::Device, tag: tag.into(), part: part.into(), w: 36.0, h: 90.0, x: 100.0 * id as f32, ..Default::default() });
    }
    prj.layout.enclosures.push(e);
    let contact = db.parts.iter().find(|p| p.number == "HAG.CZ001").unwrap().clone();
    let found = places(&prj, 0, &contact);
    assert_eq!(found, [Place::Device(1, "-F1".into())]);
    assert_eq!(place_at(&prj.layout.enclosures[0], &found, Face::Plate, p(110.0, 40.0)), Some(&found[0]));
    assert_eq!(put(&mut prj, &db, 0, "HAG.CZ001", &found[0], p(0.0, 0.0)), Ok(0));
    assert!(prj.devices["-F1"].parts.iter().any(|r| r.number == "HAG.CZ001"));
}
