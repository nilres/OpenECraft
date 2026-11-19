use super::*;
use crate::analysis::Analysis;
use crate::model::Cable;

/// Two of the four cores of the demo's motor cable as a cable -W2 of its own: a second
/// cable out of +CAB1, which has one gland.
fn split(prj: &mut Project) {
    let part = prj.parts.iter().find(|p| p.number == "GEN.CBL-4G1.5").unwrap().clone();
    let id = prj.new_id();
    prj.cables.push(Cable { id, tag: "-W2".into(), cable_type: part.type_number.clone(), cores: part.tech_value("Cores").unwrap_or_default().into(), cross_section: part.tech_value("Cross-section").unwrap_or_default().into(), length: 15.0, part: part.number.clone(), ..Default::default() });
    let page = prj.pages.iter().position(|p| p.name == "2").unwrap();
    let wires: std::collections::BTreeSet<u64> = prj.pages[page].wires.iter().filter(|w| w.cable == "-W1").map(|w| w.id).take(2).collect();
    assert_eq!(crate::ops::assign_cores(prj, page, &wires, "-W2"), 2);
}

#[test]
fn a_new_cable_out_of_an_enclosure_gets_a_gland_of_its_size() {
    let before = crate::sample::demo_project();
    assert!(before.layout.auto_glands, "on by default");
    let mut prj = before.clone();
    split(&mut prj);
    let added = follow(&mut prj, &before);
    assert_eq!(added.len(), 1);
    let e = &prj.layout.enclosures[0];
    let g = e.item(added[0]).unwrap();
    assert_eq!(g.face, Face::Bottom);
    // Of the size of the cable, clear of the gland there was.
    let d = cable_diameter(prj.cables.iter().find(|c| c.tag == "-W2").unwrap());
    let (lo, hi) = panel::clamping_range(&prj, &g.part).unwrap();
    assert!(d >= lo && d <= hi, "{d} in {lo} – {hi}");
    let other = e.items.iter().find(|i| i.face == Face::Bottom && i.id != g.id).unwrap();
    assert!(g.x >= other.x + other.w || g.x + g.w <= other.x || g.y >= other.y + other.h || g.y + g.h <= other.y);
    // Both cables are routed, each through a gland.
    let an = Analysis::checked(&prj);
    assert!(["-W1", "-W2"].iter().all(|k| an.routing.item(k).unwrap().routed()), "{:?}", an.routing.problems);
    // Nothing more without a change; a gland removed by hand stays removed while only
    // the layout changes, and comes back with the next change of the cables.
    let same = prj.clone();
    assert!(follow(&mut prj, &same).is_empty());
    let mut removed = prj.clone();
    removed.layout.enclosures[0].items.retain(|i| i.id != added[0]);
    assert!(follow(&mut removed, &prj).is_empty());
    let kept = removed.clone();
    removed.cables[0].length += 1.0;
    assert_eq!(follow(&mut removed, &kept).len(), 1);
}

#[test]
fn without_the_option_glands_are_added_only_on_request() {
    let mut before = crate::sample::demo_project();
    before.layout.auto_glands = false;
    let mut prj = before.clone();
    split(&mut prj);
    assert!(follow(&mut prj, &before).is_empty());
    assert_eq!(add_missing(&mut prj).len(), 1);
    assert!(add_missing(&mut prj).is_empty());
    // Stored only where it is off.
    let text = serde_json::to_string(&before.layout).unwrap();
    assert!(text.contains("\"auto_glands\":false"));
    assert!(!serde_json::to_string(&crate::panel::Layout::default()).unwrap().contains("auto_glands"));
    let back: crate::panel::Layout = serde_json::from_str("{}").unwrap();
    assert!(back.auto_glands);
}

#[test]
fn glands_by_the_size_of_the_cable() {
    let prj = Project::default();
    assert_eq!(gland_for(&prj, 7.0).as_deref(), Some("GEN.GLAND-M16"));
    assert_eq!(gland_for(&prj, 15.0).as_deref(), Some("GEN.GLAND-M25"));
    assert_eq!(gland_for(&prj, 70.0).as_deref(), Some("GEN.GLAND-M63"));
}

