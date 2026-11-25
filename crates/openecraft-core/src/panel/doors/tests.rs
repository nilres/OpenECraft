use super::*;

fn cabinet(width: f32, leaves: &str) -> Enclosure {
    let mut e = Enclosure { width, ..Default::default() };
    assert!(set(&mut e, leaves));
    e
}

#[test]
fn a_double_door_has_two_leaves_hinged_at_the_outer_edges() {
    let e = cabinet(800.0, "275 + 525");
    let all = leaves(&e);
    assert_eq!(all, [DoorLeaf { x: 0.0, w: 275.0, hinge: DoorHinge::Left }, DoorLeaf { x: 275.0, w: 525.0, hinge: DoorHinge::Right }]);
    assert_eq!((all[0].hinge_x(), all[1].hinge_x()), (0.0, 800.0));
    assert_eq!((leaf_index(&e, 100.0), leaf_index(&e, 300.0), leaf_index(&e, 900.0)), (0, 1, 1));
    assert_eq!(joints(&e), [275.0]);
    assert_eq!(text(&e), "275 + 525");
    // A device over the joint cannot be mounted; one beside it can.
    let over = PanelItem { face: Face::Door, x: 250.0, w: 50.0, ..Default::default() };
    let beside = PanelItem { x: 280.0, ..over.clone() };
    assert!(crosses_joint(&e, &over) && !crosses_joint(&e, &beside));
    assert_eq!((leaf_of(&e, &over), leaf_of(&e, &beside)), (1, 1));
    // One leaf: the whole front, hinged as the enclosure says.
    let mut one = Enclosure { width: 600.0, door_hinge: DoorHinge::Right, ..Default::default() };
    assert_eq!(leaves(&one), [DoorLeaf { x: 0.0, w: 600.0, hinge: DoorHinge::Right }]);
    assert!(!crosses_joint(&one, &over) && text(&one).is_empty());
    split(&mut one, Some(300.0));
    assert_eq!(text(&one), "300 + 300");
    split(&mut one, None);
    assert!(one.door_leaves.is_empty());
}

#[test]
fn door_leaves_are_read_from_part_data() {
    // Equal leaves, widths scaled to the front, hinges given.
    assert_eq!(parse("2", 1050.0).unwrap().iter().map(|l| (l.x, l.w)).collect::<Vec<_>>(), [(0.0, 525.0), (525.0, 525.0)]);
    let scaled = parse("1 + 3", 800.0).unwrap();
    assert_eq!((scaled[0].w, scaled[1].x), (200.0, 200.0));
    let hinged = parse("400 right + 400 left", 800.0).unwrap();
    assert_eq!(hinged.iter().map(|l| l.hinge).collect::<Vec<_>>(), [DoorHinge::Right, DoorHinge::Left]);
    assert_eq!(text(&Enclosure { width: 800.0, door_leaves: hinged, ..Default::default() }), "400 right + 400 left");
    for bad in ["", "1", "abc", "275 + x", "275 up + 525"] {
        assert!(parse(bad, 800.0).is_none(), "{bad}");
    }
    // A wider enclosure keeps the shares of its leaves.
    let mut e = cabinet(800.0, "200 + 600");
    e.width = 1200.0;
    fit_width(&mut e, 800.0);
    assert_eq!(text(&e), "300 + 900");
}

#[test]
fn the_double_door_of_a_catalog_cabinet_has_its_leaves_everywhere() {
    use crate::model::Project;
    use crate::panel::{components, new_enclosure, validate, wiring_end, CheckKind, Role};
    // ComfortLine A of 3 field widths (800 mm): a leaf of one field and one of two.
    let mut prj = Project { parts: crate::parts::bundled::load("striebel_john").unwrap().parts, ..Default::default() };
    let mut e = new_enclosure(&mut prj, "SJ.2CPX052032R9999", "+Z1");
    assert_eq!(text(&e), "275 + 525");
    // Two field widths: one leaf.
    assert!(new_enclosure(&mut prj, "SJ.2CPX052036R9999", "+Z2").door_leaves.is_empty());
    // The univers cabinet of Hager with five fields: a leaf of two fields and one of three.
    let mut hager = Project { parts: crate::parts::bundled::load("hager").unwrap().parts, ..Default::default() };
    assert_eq!(text(&new_enclosure(&mut hager, "HAG.FR25S2", "+HV")), "525 + 775");
    // A sheet for each leaf of the door.
    let door = components(&e).into_iter().find(|c| c.role == Role::Door).unwrap();
    assert_eq!(door.boxes.iter().map(|b| (b.0[0], b.1[0])).collect::<Vec<_>>(), [(0.0, 274.5), (275.5, 800.0)]);
    // The wires of a device on each leaf run over the hinge of its leaf, onto that side of
    // the plate.
    let (pw, _) = e.plate_size();
    let device = |x: f32| PanelItem { id: x as u64, face: Face::Door, kind: crate::panel::ItemKind::Device, x, y: 300.0, w: 40.0, h: 40.0, d: 30.0, ..Default::default() };
    let (left, right) = (device(100.0), device(600.0));
    assert_eq!(wiring_end(&e, &left, None).0.pos.x, 0.0);
    assert_eq!(wiring_end(&e, &right, None).0.pos.x, pw);
    // Over the joint: a warning.
    e.items = vec![left, right, device(260.0)];
    let found: Vec<_> = validate(&prj, &e).into_iter().filter(|f| f.kind == CheckKind::Outside).collect();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].items, [260]);
}
