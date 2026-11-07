use super::*;
use crate::model::{SymbolInst, TextItem, Wire};

fn page_with_coils() -> (Project, BTreeSet<u64>) {
    let mut prj = Project::default();
    let mut sel = BTreeSet::new();
    for (i, (x, y)) in [(40.0, 60.0), (100.0, 80.0), (120.0, 52.0)].into_iter().enumerate() {
        let id = prj.new_id();
        prj.pages[0].symbols.push(SymbolInst { id, sym: "coil".into(), pos: p(x, y), tag: format!("-K{}", i + 1), ..Default::default() });
        sel.insert(id);
    }
    (prj, sel)
}

#[test]
fn align_moves_items_and_their_wires() {
    let (mut prj, sel) = page_with_coils();
    let symbols = prj.symbol_table();
    // A wire ending on the top connection point of -K2.
    let k2 = prj.pages[0].symbols[1].clone();
    let top = symbols.get("coil").unwrap().pin_world(0, k2.pos, 0, false);
    let id = prj.new_id();
    prj.pages[0].wires.push(Wire { id, a: top + p(0.0, -20.0), b: top, ..Default::default() });
    assert_eq!(align(&mut prj, 0, &sel, Align::Top), 2);
    let ys: Vec<f32> = prj.pages[0].symbols.iter().map(|s| s.pos.y).collect();
    assert_eq!(ys, [52.0, 52.0, 52.0]);
    let moved = prj.pages[0].symbols[1].pos;
    let new_top = symbols.get("coil").unwrap().pin_world(0, moved, 0, false);
    assert!(prj.pages[0].wires[0].b.same(new_top), "the wire end follows");

    assert_eq!(align(&mut prj, 0, &sel, Align::Left), 2);
    assert!(prj.pages[0].symbols.iter().all(|s| s.pos.x == 40.0));
    let w = &prj.pages[0].wires[0];
    assert_eq!(w.a.x, w.b.x, "the wire stays vertical");
    assert_eq!(align(&mut prj, 0, &sel, Align::Left), 0, "nothing left to do");
    let one: BTreeSet<u64> = sel.iter().take(1).copied().collect();
    assert_eq!(align(&mut prj, 0, &one, Align::Right), 0);
    assert_eq!(Align::from_code("Middle"), Some(Align::Middle));
    assert_eq!(Align::from_code("diagonal"), None);
}

#[test]
fn distribute_spaces_items_evenly() {
    let (mut prj, sel) = page_with_coils();
    assert_eq!(distribute(&mut prj, 0, &sel, true), 1);
    let xs: Vec<f32> = prj.pages[0].symbols.iter().map(|s| s.pos.x).collect();
    assert_eq!(xs, [40.0, 80.0, 120.0]);
    // Texts are moved without snapping.
    let id = prj.new_id();
    prj.pages[0].texts.push(TextItem { id, pos: p(10.0, 10.0), text: "a".into(), ..Default::default() });
    let mut both = sel.clone();
    both.insert(id);
    assert_eq!(align(&mut prj, 0, &both, Align::Bottom), 3, "the lowest item stays");
}

#[test]
fn multiple_copies_get_new_tags() {
    let (mut prj, sel) = page_with_coils();
    let first: BTreeSet<u64> = sel.iter().take(1).copied().collect();
    let ids = multiple_copy(&mut prj, 0, &first, 3, p(0.0, 40.0));
    assert_eq!(ids.len(), 3);
    let copies: Vec<&SymbolInst> = prj.pages[0].symbols.iter().filter(|s| ids.contains(&s.id)).collect();
    let ys: Vec<f32> = copies.iter().map(|s| s.pos.y).collect();
    assert_eq!(ys, [100.0, 140.0, 180.0]);
    let tags: BTreeSet<&str> = copies.iter().map(|s| s.tag.as_str()).collect();
    assert_eq!(tags.len(), 3, "every copy is a device of its own");
    assert!(!tags.contains("-K1"));
    assert!(multiple_copy(&mut prj, 0, &BTreeSet::new(), 2, p(10.0, 0.0)).is_empty());
}

#[test]
fn typed_points() {
    assert_eq!(parse_point("40 80", None), Some(p(40.0, 80.0)));
    assert_eq!(parse_point(" 40, 80 ", None), Some(p(40.0, 80.0)));
    assert_eq!(parse_point("@8 -4", Some(p(40.0, 80.0))), Some(p(48.0, 76.0)));
    assert_eq!(parse_point("@8 -4", None), None);
    assert_eq!(parse_point("12.5;3", None), Some(p(12.5, 3.0)));
    assert_eq!(parse_point("12", None), None);
    assert_eq!(parse_point("a b", None), None);
}
