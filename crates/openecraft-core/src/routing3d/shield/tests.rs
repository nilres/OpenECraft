use super::*;
use crate::parts::vec3::V3;
use crate::routing3d::{End, ShieldEnds, Status};

fn item(key: &str, class: Class, diameter: f32, points: Vec<V3>) -> RoutedItem {
    RoutedItem {
        kind: RouteKind::Cable,
        key: key.into(),
        label: key.into(),
        class,
        ends: [End::default(), End::default()],
        enclosures: vec![0],
        points,
        radii: Vec::new(),
        legs: Vec::new(),
        length: 0.0,
        cut: 0.0,
        diameter,
        bend_radius: 0.0,
        free: 0.0,
        bends: 0,
        shielded: false,
        status: Status::Routed,
        wire: None,
        laying: crate::routing3d::Laying::Surface,
    }
}

#[test]
fn shield_detected_from_type_and_bus() {
    let c = |ty: &str| Cable { cable_type: ty.into(), ..Default::default() };
    assert!(is_shielded(&c("LiYCY 4x0.5")));
    assert!(is_shielded(&c("ÖLFLEX CLASSIC 110 CY 4G1.5")));
    assert!(is_shielded(&c("NYM (St) 3x1.5")));
    assert!(!is_shielded(&c("H07RN-F 4G1.5")));
    assert!(is_shielded(&Cable { bus: Some(crate::devdesc::BusKind::Profinet), ..c("") }));
    assert!(!is_shielded(&Cable { shielded: Some(false), ..c("LiYCY") }));
}

#[test]
fn terminations_and_problems() {
    let mut prj = Project::default();
    let e = crate::panel::new_enclosure(&mut prj, "", "+A");
    prj.layout.enclosures.push(e);
    prj.cables.push(Cable { tag: "-W5".into(), bus: Some(crate::devdesc::BusKind::Profinet), ..Default::default() });
    let device = |tag: &str, x: f32| End { tag: tag.into(), enclosure: Some(0), point: Some([x, 400.0, 100.0]), item: Some(1), ..Default::default() };
    let mut it = item("-W5", Class::Signal, 7.0, vec![[100.0, 400.0, 100.0], [500.0, 400.0, 100.0]]);
    it.shielded = true;
    // The device ends are no placed devices of the project: give them ids that are not
    // devices either way (field ends are cable glands).
    it.ends = [device("-A1", 100.0), End { tag: "-A9".into(), ..Default::default() }];
    let power = item("L1", Class::Power, 4.0, vec![]);
    let bundle = Bundle { items: vec![0, 1], points: vec![], length: 200.0, diameter: 9.0, classes: vec![Class::Signal, Class::Power] };
    let (infos, problems) = analyse(&prj, &[it.clone(), power], std::slice::from_ref(&bundle));
    // No placed device -A1: its end counts as entering from the field.
    assert_eq!(infos[0].ends[1].termination, Termination::Field);
    assert!(problems.iter().any(|p| p.message.contains("power conductors over 200 mm")));
    // A device end without a clamp rail: pigtail, a problem for bus cables.
    it.ends[0].item = None;
    let (infos, problems) = analyse(&prj, &[it.clone()], &[]);
    assert_eq!(infos[0].ends[0].termination, Termination::Pigtail(prj.layout.routing.pigtail));
    assert!(problems.iter().any(|p| p.message.contains("pigtail at -A1")));
    // With a shield clamp rail in the enclosure.
    prj.layout.enclosures[0].items.push(crate::panel::PanelItem { id: 9, kind: crate::panel::ItemKind::Accessory, part: "GEN.CLAMP-RAIL-500".into(), x: 0.0, y: 700.0, w: 500.0, h: 30.0, d: 15.0, ..Default::default() });
    let (infos, problems) = analyse(&prj, &[it.clone()], &[]);
    assert!(matches!(infos[0].ends[0].termination, Termination::Clamp { rail: 9, .. }));
    assert!(problems.is_empty(), "{problems:?}");
    // Bus cable shield on one end only.
    prj.cables[0].shield_ends = ShieldEnds::First;
    let (infos, problems) = analyse(&prj, &[it], &[]);
    assert_eq!(infos[0].ends[1].termination, Termination::Open);
    assert!(problems.iter().any(|p| p.message.contains("not connected at both ends")));
}
