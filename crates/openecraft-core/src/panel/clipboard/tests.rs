use super::*;
use crate::geom::p;

fn item(id: u64, kind: ItemKind, x: f32, y: f32, w: f32, h: f32) -> PanelItem {
    PanelItem { id, kind, part: format!("PART.{id}"), x, y, w, h, d: 10.0, ..Default::default() }
}

/// A rail with a fan mounted on it and a device, a duct of an assembly, an item on
/// the door.
fn project() -> Project {
    let mut prj = Project { next_id: 100, ..Default::default() };
    let mut e = Enclosure { width: 600.0, height: 800.0, depth: 250.0, ..Default::default() };
    e.items.push(item(1, ItemKind::Rail, 100.0, 200.0, 300.0, 35.0));
    e.items.push(PanelItem { mounted_on: 1, ..item(2, ItemKind::Accessory, 150.0, 190.0, 40.0, 60.0) });
    e.items.push(PanelItem { tag: "-K1".into(), mounted_on: 1, ..item(3, ItemKind::Device, 250.0, 180.0, 18.0, 80.0) });
    e.items.push(PanelItem { group: 7, ..item(4, ItemKind::Duct, 50.0, 100.0, 500.0, 60.0) });
    e.items.push(PanelItem { face: Face::Door, ..item(5, ItemKind::Accessory, 10.0, 10.0, 20.0, 20.0) });
    e.groups.push(ItemGroup { id: 7, name: "Duct frame".into(), source: String::new() });
    prj.layout.enclosures.push(e);
    prj
}

#[test]
fn copies_the_items_with_their_mountings_without_devices() {
    let prj = project();
    let e = &prj.layout.enclosures[0];
    let clip = copy_items(e, &[1, 3]);
    assert_eq!(clip.devices, 1);
    let kinds: Vec<ItemKind> = clip.items.iter().map(|i| i.kind).collect();
    assert_eq!(kinds, [ItemKind::Rail, ItemKind::Accessory]);
    // Relative to the top left corner of both, the fan mounted on the rail.
    assert_eq!((clip.items[0].x, clip.items[0].y), (0.0, 10.0));
    assert_eq!((clip.items[1].x, clip.items[1].y), (50.0, 0.0));
    assert_eq!((clip.x, clip.y, clip.w, clip.h), (100.0, 190.0, 300.0, 60.0));
    assert_eq!(clip.items[1].mounted_on, clip.items[0].id);
    // Only the face of the first item.
    let clip = copy_items(e, &[4, 5]);
    assert_eq!(clip.items.len(), 1);
    assert_eq!(clip.groups[0].name, "Duct frame");
    assert_eq!(clip.items[0].group, clip.groups[0].id);
    assert!(copy_items(e, &[3]).is_empty());
}

#[test]
fn pastes_new_items_onto_the_face() {
    let mut prj = project();
    let clip = copy_items(&prj.layout.enclosures[0], &[1, 4]);
    let ids = paste_items(&mut prj, 0, &clip, Face::Door, p(20.0, 30.0));
    assert_eq!(ids.len(), 3);
    let e = &prj.layout.enclosures[0];
    assert_eq!(e.items.len(), 8);
    let new: Vec<&PanelItem> = ids.iter().filter_map(|id| e.item(*id)).collect();
    assert!(new.iter().all(|i| i.face == Face::Door && ![1, 2, 3, 4, 5].contains(&i.id)));
    let duct = new.iter().find(|i| i.kind == ItemKind::Duct).unwrap();
    assert_eq!((duct.x, duct.y), (20.0, 30.0));
    let rail = new.iter().find(|i| i.kind == ItemKind::Rail).unwrap();
    assert_eq!((rail.x, rail.y), (70.0, 130.0));
    let fan = new.iter().find(|i| i.kind == ItemKind::Accessory).unwrap();
    assert_eq!(fan.mounted_on, rail.id);
    // A new assembly of the same name.
    assert!(duct.group != 0 && duct.group != 7);
    assert!(e.groups.iter().any(|g| g.id == duct.group && g.name == "Duct frame"));
    assert!(paste_items(&mut prj, 3, &clip, Face::Plate, p(0.0, 0.0)).is_empty());
}

#[test]
fn clip_text_round_trip() {
    let prj = project();
    let clip = copy_items(&prj.layout.enclosures[0], &[1]);
    assert_eq!(PanelClip::from_text(&clip.to_text()), Some(clip));
    assert_eq!(PanelClip::from_text("{}"), None);
    assert_eq!(PanelClip::from_text("-K1"), None);
}
