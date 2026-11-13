use super::*;
use crate::panel::{layout_material, new_enclosure};
use crate::sample::demo_project;

fn baying() -> Enclosure {
    let mut e = Enclosure::default();
    apply_system(&mut e, "baying", Some((800.0, 2000.0, 600.0)));
    e
}

#[test]
fn monobloc_enclosure_is_one_part() {
    let mut e = Enclosure::default();
    apply_system(&mut e, "compact", Some((600.0, 760.0, 210.0)));
    assert_eq!(e.part, "GEN.CE-600X760X210");
    let parts: Vec<_> = components(&e).into_iter().filter(|c| !c.part.is_empty()).collect();
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].role, Role::Body);
    // Walls, door, plate still have their geometry.
    assert!(components(&e).iter().any(|c| c.role == Role::Wall(Face::Left) && !c.boxes.is_empty()));
    assert_eq!(e.wall(), 1.5);
}

#[test]
fn baying_system_components() {
    let mut e = baying();
    assert_eq!(e.build.plinth, 100.0);
    let cs = components(&e);
    let part = |r: Role| cs.iter().find(|c| c.role == r).map(|c| c.part.clone());
    assert_eq!(part(Role::Body).as_deref(), Some("GEN.TS-800X2000X600"));
    assert_eq!(part(Role::Wall(Face::Left)).as_deref(), Some("GEN.TS-SP-2000X600"));
    assert_eq!(part(Role::Wall(Face::Rear)).as_deref(), Some("GEN.TS-RW-800X2000"));
    assert_eq!(part(Role::Plinth).as_deref(), Some("GEN.PL-800X600-100"));
    // Frame: four posts and eight rails; plinth below the enclosure.
    assert_eq!(cs.iter().find(|c| c.role == Role::Body).unwrap().boxes.len(), 12);
    let plinth = cs.iter().find(|c| c.role == Role::Plinth).unwrap();
    assert!(plinth.boxes.iter().all(|b| b.0[1] >= e.height && b.1[1] <= e.height + 100.0));
    // Bayed to the right: no right side panel, one baying kit.
    e.build.right_open = true;
    let cs = components(&e);
    assert!(!cs.iter().any(|c| c.role == Role::Wall(Face::Right)));
    assert_eq!(cs.iter().find(|c| c.role == Role::BayingKit).map(|c| c.count), Some(1));
    // Every component part of the standard sizes is in the catalog.
    for c in cs.iter().filter(|c| !c.part.is_empty()) {
        assert!(crate::parts::catalog().iter().any(|p| p.number == c.part), "{}", c.part);
    }
}

#[test]
fn system_from_catalog_part() {
    let mut prj = demo_project();
    let e = new_enclosure(&mut prj, "GEN.TS-600X2000X500", "+CAB9");
    assert_eq!(e.build.system, "baying");
    assert_eq!((e.width, e.height, e.depth), (600.0, 2000.0, 500.0));
    assert_eq!(e.climate.material, Material::SheetSteel);
    let p = new_enclosure(&mut prj, "GEN.PC-300X400X180", "+B1");
    assert_eq!(p.climate.material, Material::Plastic);
    assert!(!p.build.left_open && p.build.plinth == 0.0);
}

#[test]
fn components_in_the_bill_of_materials() {
    let mut prj = demo_project();
    let mut e = baying();
    e.name = "+CAB2".into();
    e.build.gland_plate = true;
    e.build.left_open = true;
    prj.layout.enclosures.push(e);
    let bom = layout_material(&prj);
    let qty = |n: &str| bom.iter().find(|m| m.0 == n).map(|m| m.1);
    assert_eq!(qty("GEN.TS-800X2000X600"), Some(1.0));
    assert_eq!(qty("GEN.TS-SP-2000X600"), Some(1.0));
    assert_eq!(qty("GEN.GP-800X600"), Some(1.0));
    assert_eq!(qty(crate::parts::mechanics::BAYING_KIT), Some(1.0));
    // The plain enclosure of the demo keeps its part.
    assert_eq!(qty(crate::panel::ENCLOSURE_PART), Some(1.0));
}

#[test]
fn resizing_follows_the_part_number() {
    let mut e = baying();
    e.width = 1200.0;
    sync_part(&mut e);
    assert_eq!(e.part, "GEN.TS-1200X2000X600");
    apply_system(&mut e, "", None);
    assert!(e.system().is_none());
    // A plain enclosure of the catalog becomes the system's body.
    let mut plain = Enclosure { part: crate::panel::ENCLOSURE_PART.into(), width: 600.0, height: 600.0, depth: 350.0, ..Default::default() };
    apply_system(&mut plain, "compact", None);
    assert_eq!(plain.part, "GEN.CE-600X600X350");
}

#[test]
fn a_makers_enclosure_keeps_its_part() {
    let mut prj = crate::model::Project::default();
    let mut part = crate::parts::Part { number: "RIT.1360000".into(), width: 600.0, height: 600.0, depth: 350.0, ..Default::default() };
    part.tech_data = "Enclosure system: compact".into();
    prj.parts.push(part);
    let mut e = new_enclosure(&mut prj, "RIT.1360000", "+A1");
    assert_eq!((e.part.as_str(), e.build.system.as_str()), ("RIT.1360000", "compact"));
    e.width = 800.0;
    sync_part(&mut e);
    assert_eq!(e.part, "RIT.1360000");
}

#[test]
fn a_plinth_delivered_with_the_enclosure_is_no_part_of_its_own() {
    let mut prj = crate::model::Project::default();
    let db = crate::parts::PartsDb { parts: crate::parts::bundled::default_parts() };
    crate::parts::ensure_in_project(&mut prj, &db, "HAG.FR25S2");
    let e = new_enclosure(&mut prj, "HAG.FR25S2", "+HV");
    assert_eq!((e.part.as_str(), e.width, e.height, e.depth), ("HAG.FR25S2", 1300.0, 1850.0, 275.0));
    assert!(e.build.plinth_included && e.build.plinth == 200.0);
    let cs = components(&e);
    let plinth = cs.iter().find(|c| c.role == Role::Plinth).unwrap();
    assert!(plinth.part.is_empty() && !plinth.boxes.is_empty());
    // A plinth set by hand stays a part of its own.
    let mut plain = new_enclosure(&mut prj, "GEN.ENC-600X800", "+CAB1");
    plain.build.plinth = 100.0;
    assert_eq!(components(&plain).iter().find(|c| c.role == Role::Plinth).unwrap().part, "GEN.PL-600X250-100");
}

/// A maker's enclosure part with a model of 600 × 600 × 350 mm: its left wall, the door
/// leaf in front and the mounting plate inside.
fn maker_project() -> crate::model::Project {
    use crate::parts::vec3::box_triangles;
    let mut prj = crate::model::Project::default();
    let mut part = crate::parts::Part { number: "RIT.1360000".into(), width: 600.0, height: 600.0, depth: 350.0, ..Default::default() };
    part.tech_data = "Enclosure system: compact".into();
    let mut mesh = crate::parts::Mesh3d::default();
    for tri in [box_triangles([0.0, 0.0, 0.0], [1.5, 600.0, 345.0]), box_triangles([2.0, 2.0, 334.0], [598.0, 598.0, 350.0]), box_triangles([25.0, 12.5, 17.5], [575.0, 587.5, 20.0])].concat() {
        let base = mesh.vertices.len() as u32;
        mesh.vertices.extend(tri);
        mesh.triangles.push([base, base + 1, base + 2]);
    }
    part.model_3d = Some(mesh.into());
    prj.parts.push(part);
    prj
}

#[test]
fn the_makers_model_goes_with_walls_door_and_inside() {
    let mut prj = maker_project();
    let e = new_enclosure(&mut prj, "RIT.1360000", "+A1");
    let model = maker_model(&prj, &e).unwrap();
    let count = |f: Option<Face>| model.iter().filter(|(g, _)| *g == f).count();
    // The wall's front edge lies in the front that opens with the door.
    assert_eq!((count(Some(Face::Left)), count(Some(Face::Door)), count(None)), (10, 14, 12));
    // The body is the maker's part, also in the bill of materials.
    assert_eq!(components(&e).iter().find(|c| c.role == Role::Body).map(|c| c.part.as_str()), Some("RIT.1360000"));
    // Not for catalog enclosures or a model of another size.
    let mut other = e.clone();
    other.width = 800.0;
    assert!(maker_model(&prj, &other).is_none());
    assert!(maker_model(&prj, &baying()).is_none());
}
