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
    assert_eq!((it.kind, it.face, it.x, it.y, it.w, it.h), (ItemKind::MountingPlate, Face::Plate, 250.0, 0.0, 250.0, 1050.0));
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
fn a_distribution_panel_brings_a_mounting_rail_for_each_row() {
    let (mut prj, ei) = sj_project();
    let fv17 = part(&prj, FV17).clone();
    let rows = [150.0, 275.0, 400.0, 525.0, 650.0, 775.0, 900.0];
    assert_eq!(panel_rails(&fv17, 1050.0), rows);
    let slot = places(&prj, ei, &fv17)[1].clone();
    let id = put(&mut prj, &PartsDb::default(), ei, FV17, &slot, p(0.0, 0.0)).unwrap();
    let e = &prj.layout.enclosures[ei];
    let panel = e.item(id).unwrap().clone();
    let rails: Vec<PanelItem> = e.items.iter().filter(|i| i.kind == ItemKind::Rail).cloned().collect();
    assert_eq!(rails.len(), 7);
    for (r, y) in rails.iter().zip(rows) {
        assert_eq!((r.part.as_str(), r.x, r.w, r.center().y, r.mounted_on, r.bom_excluded), (crate::panel::RAIL_PART, 260.0, 230.0, y, id, true));
        // So deep that a modular device on it ends at the front of the panel.
        assert!((r.z + r.d + MODULAR_DEPTH - fv17.depth).abs() < 0.01, "{}", r.z);
    }
    assert!((panel.z + panel.d - rails[0].z).abs() < 0.01);
    // The rails are not billed, the panel is.
    let bom = crate::panel::layout_material(&prj);
    assert!(bom.iter().any(|l| l.0 == FV17) && !bom.iter().any(|l| l.0 == crate::panel::RAIL_PART), "{bom:?}");
    assert!(crate::panel::cut_pieces(&prj).is_empty());
    // A device dropped onto a row goes onto its rail and moves with the panel.
    let mut dev = PanelItem { id: prj.new_id(), kind: ItemKind::Device, tag: "-F1".into(), w: 18.0, h: 90.0, d: 70.0, x: 300.0, y: 405.0 - 45.0, ..Default::default() };
    crate::panel::snap_to_rail(&prj.layout.enclosures[ei], &mut dev);
    assert_eq!(dev.mounted_on, rails[2].id);
    let dev_id = dev.id;
    prj.layout.enclosures[ei].items.push(dev);
    let carried = crate::panel::mounted_items(&prj.layout.enclosures[ei], &[id]);
    assert_eq!(carried.len(), 9, "the panel, its rails and the device");
    assert!(carried.contains(&dev_id));
}

/// Bounds (min, max) of the triangles of a colour.
fn painted_bounds(tris: &[(Tri, Option<[u8; 3]>)], color: [u8; 3]) -> Option<([f32; 3], [f32; 3])> {
    let pts: Vec<[f32; 3]> = tris.iter().filter(|t| t.1 == Some(color)).flat_map(|t| t.0).collect();
    (!pts.is_empty()).then(|| ([0, 1, 2].map(|k| pts.iter().map(|v| v[k]).fold(f32::INFINITY, f32::min)), [0, 1, 2].map(|k| pts.iter().map(|v| v[k]).fold(f32::NEG_INFINITY, f32::max))))
}

#[test]
fn a_distribution_panel_is_a_frame_on_brackets_behind_a_cover() {
    let (mut prj, ei) = sj_project();
    let slot = places(&prj, ei, part(&prj, FV17))[0].clone();
    let id = put(&mut prj, &PartsDb::default(), ei, FV17, &slot, p(0.0, 0.0)).unwrap();
    let panel = prj.layout.enclosures[ei].item(id).unwrap().clone();
    assert!(is_insert(&prj, &panel));
    let tris = insert_mesh(&prj, &panel).unwrap();
    // The frame where the rails sit, the brackets back to the rear wall.
    let (lo, hi) = painted_bounds(&tris, FRAME_STEEL).unwrap();
    assert_eq!((lo[0], lo[1], hi[0], hi[1]), (0.0, 0.0, 250.0, 1050.0));
    let e = &prj.layout.enclosures[ei];
    let wall = e.wall() - e.mounting_z();
    assert!(wall < 0.0 && (lo[2] - wall).abs() < 0.01 && (hi[2] - (panel.z + panel.d)).abs() < 0.01, "{lo:?} {hi:?}");
    // The cover at the front of the panel, a slot for each row of devices.
    let (lo, hi) = painted_bounds(&tris, COVER_PLASTIC).unwrap();
    assert!((hi[2] - 160.0).abs() < 0.01 && lo[2] < 160.0 - COVER - 1.0, "{lo:?} {hi:?}");
    let covered = |x: f32, y: f32| tris.iter().filter(|t| t.1 == Some(COVER_PLASTIC)).any(|t| {
        let (a, b) = ([0, 1].map(|k| t.0.iter().map(|v| v[k]).fold(f32::INFINITY, f32::min)), [0, 1].map(|k| t.0.iter().map(|v| v[k]).fold(f32::NEG_INFINITY, f32::max)));
        t.0.iter().all(|v| v[2] > 160.0 - COVER - 0.01) && a[0] <= x && x <= b[0] && a[1] <= y && y <= b[1]
    });
    assert!(!covered(125.0, 150.0) && !covered(125.0, 900.0), "slots at the rails");
    assert!(covered(125.0, 210.0) && covered(125.0, 50.0) && covered(5.0, 150.0));
    // Without its cover.
    prj.layout.enclosures[ei].items.iter_mut().find(|i| i.id == id).unwrap().uncovered = true;
    let bare = crate::panel::device_mesh_painted(&prj, prj.layout.enclosures[ei].item(id).unwrap()).unwrap();
    assert!(painted_bounds(&bare, COVER_PLASTIC).is_none() && painted_bounds(&bare, FRAME_STEEL).is_some());
    // Other panels of the grid (a meter panel) too, the rails' own item is none.
    let meter = prj.parts.iter().find(|p| p.subgroup == "Meter panel" && crate::parts::accessory::fits(p, part(&prj, CABINET)) && crate::parts::count_value(p, "Fields") == Some(1)).unwrap().number.clone();
    let slot = places(&prj, ei, part(&prj, &meter))[0].clone();
    let id = put(&mut prj, &PartsDb::default(), ei, &meter, &slot, p(0.0, 0.0)).unwrap();
    let e = &prj.layout.enclosures[ei];
    let tris = insert_mesh(&prj, e.item(id).unwrap()).unwrap();
    assert!(painted_bounds(&tris, FRAME_STEEL).is_some_and(|(lo, _)| (lo[2] - wall).abs() < 0.01));
    assert!(e.items.iter().filter(|i| i.kind == ItemKind::Rail).all(|r| insert_mesh(&prj, r).is_none()));
}

#[test]
fn the_rails_of_other_panels() {
    let prj = Project { parts: bundled::load("striebel_john").unwrap().parts, ..Default::default() };
    let get = |t: &str| prj.parts.iter().find(|p| p.type_number == t).unwrap();
    // ABB's FV17: seven rails 125 mm apart; a connection point zone above five rails;
    // the busbars of the FV17S above five; none in a panel of meters and connections.
    assert_eq!(panel_rails(get("FV17"), 1050.0), [150.0, 275.0, 400.0, 525.0, 650.0, 775.0, 900.0]);
    assert_eq!(panel_rails(get("FV17A2"), 1050.0), [400.0, 525.0, 650.0, 775.0, 900.0]);
    assert_eq!(panel_rails(get("FV17S"), 1050.0).len(), 5);
    assert_eq!(panel_rails(get("FV19A2"), 1350.0).len(), 7);
    assert!(panel_rails(get("FV17A2R5"), 1050.0).is_empty());
    // Rows 125 mm apart, the rails 35 x 15 mm.
    let narrow = get("1V3A");
    assert_eq!(panel_rails(narrow, 1050.0), [150.0, 275.0, 400.0, 525.0, 650.0, 775.0, 900.0]);
    assert_eq!(rail_part(narrow), "GEN.RAIL-TS35X15-1M");
    // Blind panels, meter panels and the cabinets carry none.
    for t in ["1B3A", "FD17R", "A37"] {
        assert!(panel_rails(get(t), 1050.0).is_empty(), "{t}");
    }
    // Part data of any other panel.
    let mut other = Part { number: "X.PANEL".into(), tech_data: "Device rows: 2".into(), ..Default::default() };
    assert_eq!(panel_rails(&other, 300.0), [75.0, 225.0]);
    other.tech_data = "Device rows: 2\nRows: 3".into();
    assert_eq!(panel_rails(&other, 300.0), [150.0, 250.0]);
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

/// The insulating frame CZ3 that joins two cabinets at their sides, and the smaller
/// frame CZ5.
const CZ3: &str = "SJ.2CPX052461R9999";
const CZ5: &str = "SJ.2CPX052463R9999";

fn openings_of(places: &[Place]) -> Vec<(Face, f32, f32, f32, f32)> {
    places.iter().filter_map(|pl| if let Place::Opening(f, r) = pl { Some((*f, r.min.x, r.min.y, r.width(), r.height())) } else { None }).collect()
}

#[test]
fn a_joining_frame_goes_only_into_the_openings_of_the_side_walls() {
    let (mut prj, ei) = sj_project();
    let cz3 = part(&prj, CZ3).clone();
    // The cabinet of 7 rows, 1100 x 800 x 215 mm: an opening 202 x 85 mm 86 mm above the
    // bottom and one 86 mm below the top, 20 mm from the back, in both side walls.
    let found = places(&prj, ei, &cz3);
    assert_eq!(
        openings_of(&found),
        [(Face::Left, 20.0, 812.0, 85.0, 202.0), (Face::Right, 110.0, 812.0, 85.0, 202.0), (Face::Left, 20.0, 86.0, 85.0, 202.0), (Face::Right, 110.0, 86.0, 85.0, 202.0)]
    );
    assert_eq!(found.len(), 4, "nowhere else");
    // The small openings take the small frame.
    assert_eq!(openings_of(&places(&prj, ei, part(&prj, CZ5))).len(), 4);
    // Dropped: the frame over the opening, outside the wall.
    let id = put(&mut prj, &PartsDb::default(), ei, CZ3, &found[0], p(0.0, 0.0)).unwrap();
    let it = prj.layout.enclosures[ei].item(id).unwrap().clone();
    assert_eq!((it.face, it.x, it.y, it.w, it.h, it.d, it.z), (Face::Left, 5.0, 797.0, 115.0, 232.0, 50.0, -50.0));
    assert_eq!(openings_of(&places(&prj, ei, &cz3)).len(), 3, "taken");
    // Reaching over the edge of the wall there is no fault.
    let e = &prj.layout.enclosures[ei];
    assert!(at_opening(&prj, e, &it));
    assert!(!crate::panel::validate(&prj, e).iter().any(|f| f.kind == crate::panel::CheckKind::Outside), "{:?}", crate::panel::validate(&prj, e));
    // Placed by hand on a side wall: into the nearest free opening there.
    let at = crate::panel::place_accessory(&mut prj, ei, CZ3, Face::Right, p(60.0, 700.0)).unwrap();
    let it = prj.layout.enclosures[ei].item(at).unwrap();
    assert_eq!((it.face, it.x, it.y), (Face::Right, 95.0, 797.0));
    // A cabinet without openings in its data: anywhere on the faces the frame may go.
    let mut prj = Project { parts: bundled::load("striebel_john").unwrap().parts, ..Default::default() };
    prj.parts.iter_mut().find(|p| p.number == CABINET).unwrap().set_tech_value("Side openings", None);
    let e = new_enclosure(&mut prj, CABINET, "+Z1");
    prj.layout.enclosures.push(e);
    assert_eq!(places(&prj, 0, &cz3), [Place::Face(Face::Left), Place::Face(Face::Right)]);
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

const CZC4: &str = "SJ.2CPX052466R9999";
const CZC14: &str = "SJ.2CPX052467R9999";

#[test]
fn a_joining_frame_is_a_collar_through_its_opening_with_a_flange() {
    let (mut prj, ei) = sj_project();
    let found = places(&prj, ei, part(&prj, CZ3));
    let id = put(&mut prj, &PartsDb::default(), ei, CZ3, &found[0], p(0.0, 0.0)).unwrap();
    let it = prj.layout.enclosures[ei].item(id).unwrap().clone();
    let tris = frame_mesh(&prj, &it).unwrap();
    // The collar fills the opening (20..105 from the back, 812..1014 from the top) less
    // 1 mm, the flange the whole frame, all within the item.
    let (lo, hi) = painted_bounds(&tris, FRAME_PLASTIC).unwrap();
    assert_eq!((lo[0], lo[1], hi[0], hi[1]), (it.x, it.y, it.x + it.w, it.y + it.h));
    assert!((lo[2] - it.z).abs() < 0.01 && hi[2].abs() < 0.01, "{lo:?} {hi:?}");
    let through = |x: f32, y: f32| !tris.iter().any(|t| {
        let (a, b) = ([0, 1].map(|k| t.0.iter().map(|v| v[k]).fold(f32::INFINITY, f32::min)), [0, 1].map(|k| t.0.iter().map(|v| v[k]).fold(f32::NEG_INFINITY, f32::max)));
        a[0] < x && x < b[0] && a[1] < y && y < b[1]
    });
    assert!(through(62.5, 913.0), "open in the middle");
    assert!(!through(22.0, 913.0) && !through(10.0, 913.0), "collar and flange");
    assert!(painted_bounds(&tris, COPPER).is_none(), "no busbars");
    // The busbar connectors: five bars, or five pairs, through the frame.
    for (number, bars) in [(CZC4, 5), (CZC14, 10)] {
        let slot = places(&prj, ei, part(&prj, number))[0].clone();
        let id = put(&mut prj, &PartsDb::default(), ei, number, &slot, p(0.0, 0.0)).unwrap();
        let it = prj.layout.enclosures[ei].item(id).unwrap().clone();
        let tris = crate::panel::device_mesh_painted(&prj, &it).unwrap();
        assert_eq!(tris.iter().filter(|t| t.1 == Some(COPPER)).count(), bars * 12, "{number}");
    }
    // A frame elsewhere keeps its box.
    let mut loose = it.clone();
    loose.y = 400.0;
    assert!(frame_mesh(&prj, &loose).is_none());
    assert_eq!(busbar_size("Cu 2 x 12 x 5 mm"), Some((2, 12.0, 5.0)));
    assert_eq!(busbar_size("Cu 12 x 5 mm"), Some((1, 12.0, 5.0)));
}

#[test]
fn an_enclosure_is_attached_at_the_side_of_its_joining_frame() {
    let (mut prj, ei) = sj_project();
    for name in ["+Z2", "+Z3"] {
        let e = new_enclosure(&mut prj, CABINET, name);
        prj.layout.enclosures.push(e);
    }
    let room = crate::panel::buildings::add_room(&mut prj, "+R1", None).unwrap();
    crate::panel::set_room(&mut prj, ei, Some(room), None).unwrap();
    let found = places(&prj, ei, part(&prj, CZ3));
    let right = found.iter().find(|pl| pl.face(&prj.layout.enclosures[ei]) == Some(Face::Right)).unwrap().clone();
    let left = found.iter().find(|pl| pl.face(&prj.layout.enclosures[ei]) == Some(Face::Left)).unwrap().clone();
    let r = put(&mut prj, &PartsDb::default(), ei, CZ3, &right, p(0.0, 0.0)).unwrap();
    let l = put(&mut prj, &PartsDb::default(), ei, CZ3, &left, p(0.0, 0.0)).unwrap();
    assert_eq!((joining_side(&prj, ei, r), joining_side(&prj, ei, l)), (Some(Face::Right), Some(Face::Left)));
    attach_at_frame(&mut prj, ei, r, 1).unwrap();
    assert_eq!(crate::panel::lineup_of(&prj.layout, ei), [0, 1]);
    // On the left it becomes the left end and stands in the room of the line-up.
    attach_at_frame(&mut prj, ei, l, 2).unwrap();
    assert_eq!(crate::panel::lineup_of(&prj.layout, ei), [2, 0, 1]);
    assert_eq!([0, 1, 2].map(|i| crate::panel::room_of(&prj.layout, i)), [Some(room); 3]);
    assert!(attach_at_frame(&mut prj, ei, r, ei).is_err(), "not to itself");
    assert!(attach_at_frame(&mut prj, ei, r, room).is_err(), "no room");
}

#[test]
fn the_covers_of_all_panels_are_taken_off_and_fitted_at_once() {
    let (mut prj, ei) = sj_project();
    for _ in 0..2 {
        let slot = places(&prj, ei, part(&prj, FV17))[0].clone();
        put(&mut prj, &PartsDb::default(), ei, FV17, &slot, p(0.0, 0.0)).unwrap();
    }
    assert_eq!(covers(&prj, None), (2, 0));
    assert_eq!(set_covers(&mut prj, Some(ei), false), 2);
    assert_eq!((covers(&prj, Some(ei)), set_covers(&mut prj, None, false)), ((0, 2), 0));
    assert_eq!(set_covers(&mut prj, None, true), 2);
    assert_eq!(covers(&prj, None), (2, 0));
}
