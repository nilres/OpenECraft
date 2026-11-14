use super::*;

fn five(vertical: bool, reverse: bool) -> Project {
    let mut prj = Project::default();
    for i in 0..5 {
        let x = 80. + i as f32 * 12.;
        let (a, b) = if vertical { (p(x, 60.), p(x, 140.)) } else { (p(60., x), p(160., x)) };
        prj.pages[0].wires.push(Wire { id: 20 + i, a, b, ..Default::default() });
    }
    prj.fix_next_id();
    let (a, b) = if vertical { (p(70., 100.), p(140., 100.)) } else { (p(100., 70.), p(100., 140.)) };
    crate::cabledef::add(&mut prj, 0, if reverse { b } else { a }, if reverse { a } else { b }, "");
    let names = ["BN", "BK", "GY", "BU", "GNYE"];
    for (w, name) in prj.pages[0].wires.iter_mut().zip(names) { w.core = name.into(); }
    prj
}

#[test]
fn five_core_texts_clear_wires_and_definitions_in_both_orientations_and_directions() {
    for vertical in [true, false] {
        for reverse in [true, false] {
            let prj = five(vertical, reverse);
            let original = prj.clone();
            let labels = core_labels(&prj, 0);
            assert_eq!(labels.len(), 6);
            assert!(labels.iter().all(|l| !l.conflict), "{labels:?}");
            for label in &labels {
                assert!(!blocked(&prj.pages[0], label.bounds, &[]));
                for other in labels.iter().filter(|l| l.wire != label.wire || l.which != label.which) { assert!(!label.bounds.overlaps(&other.bounds)); }
            }
            assert_eq!(prj, original, "rendering and hit testing never mutate");
            let mut far = prj.clone();
            far.pages[0].texts.push(TextItem { id: 800, pos: p(300., 250.), text: "Far away".into(), ..Default::default() });
            assert_eq!(labels.iter().map(|l| l.pos).collect::<Vec<_>>(), core_labels(&far, 0).iter().map(|l| l.pos).collect::<Vec<_>>());
        }
    }
}

#[test]
fn alignment_is_explicit_page_local_serializable_and_undoable() {
    let mut prj = five(true, false);
    prj.pages[0].wires[2].core_offset = Some(p(3., -10.));
    prj.pages.push(prj.pages[0].clone());
    let original = prj.clone();
    let id = prj.pages[0].cable_defs[0].id;
    let mut history = crate::history::History::new(&prj);
    let preview = preview_core_labels(&prj, 0, id, false, 2.);
    assert_eq!(preview.len(), 5);
    assert_eq!(prj, original);
    assert_eq!(align_core_labels(&mut prj, 0, id, false, 2.), 5);
    assert_eq!(prj.pages[1], original.pages[1]);
    for (w, old) in prj.pages[0].wires.iter().zip(&original.pages[0].wires) {
        assert_eq!((w.a,w.b,&w.cable,&w.core,w.cable_offset), (old.a,old.b,&old.cable,&old.core,old.cable_offset));
    }
    assert_eq!(Project::from_json(&prj.to_json()).unwrap(), prj);
    let after = prj.clone();
    history.track(&prj, true); history.undo(&mut prj); assert_eq!(prj, original);
    history.redo(&mut prj); assert_eq!(prj, after);
    let positions: Vec<_> = core_labels(&prj, 0).iter().map(|l| (l.wire,l.which,l.pos)).collect();
    prj.pages[0].texts.push(TextItem { id: 801, pos: p(85., 90.), text: "nearby edit".into(), ..Default::default() });
    for (wire, which, pos) in positions.into_iter().filter(|(_,which,_)| *which == CoreText::Core) {
        assert_eq!(core_labels(&prj, 0).iter().find(|l| l.wire == wire && l.which == which).unwrap().pos, pos, "manual positions stay put");
    }
}

#[test]
fn crowded_area_reports_conflicts_and_refuses_to_apply() {
    let mut prj = five(true, false);
    for i in 0..45 {
        let y = 78. + i as f32;
        prj.pages[0].wires.push(Wire { id: 1000 + i, a: p(60.,y), b: p(160.,y), ..Default::default() });
    }
    let id = prj.pages[0].cable_defs[0].id;
    let original = prj.clone();
    assert!(core_labels(&prj, 0).iter().any(|l| l.conflict));
    assert!(preview_core_labels(&prj, 0, id, true, 0.8).iter().any(|l| l.conflict));
    assert_eq!(align_core_labels(&mut prj, 0, id, true, 0.8), 0);
    assert_eq!(prj, original);
}

#[test]
fn automatic_layout_avoids_nearby_symbol_and_text_bounds() {
    let mut prj = five(true, false);
    let first = core_labels(&prj, 0).into_iter().find(|l| l.which == CoreText::Core).unwrap();
    prj.pages[0].symbols.push(SymbolInst { id: 900, sym: "coil".into(), pos: first.bounds.center(), tag: "-K1".into(), ..Default::default() });
    prj.pages[0].texts.push(TextItem { id: 901, pos: p(94.,98.), text: "obstacle".into(), ..Default::default() });
    let occupied = obstacles(&prj, &Index::build(&prj), 0);
    for label in core_labels(&prj, 0) { assert!(!occupied.iter().any(|r| r.overlaps(&label.bounds)), "{label:?}"); }
}

#[test]
fn released_pages_keep_data_and_manual_positions_while_automatic_rendering_uses_the_new_layout() {
    let mut prj = five(false, false);
    prj.pages[0].wires[2].core_offset = Some(p(3., -6.));
    let open = core_labels(&prj, 0);
    prj.pages[0].locked = true;
    let before = prj.clone();
    let labels = core_labels_with_index(&prj, &Index::build(&prj), 0);
    assert_eq!(labels.iter().map(|l| l.pos).collect::<Vec<_>>(), open.iter().map(|l| l.pos).collect::<Vec<_>>());
    let w = &prj.pages[0].wires[2];
    let manual = labels.iter().find(|l| l.wire == w.id && l.which == CoreText::Core).unwrap();
    assert_eq!(manual.pos, wire_text_place(w, crate::cabledef::core_position(&prj.pages[0], w), true).0 + w.core_offset.unwrap());
    assert_eq!(prj, before);
}

#[test]
fn shared_index_lays_out_a_synthetic_hundred_page_project_without_mutating_it() {
    let mut prj = five(true, false);
    let template = prj.pages[0].clone();
    for i in 1..100 {
        let mut page = template.clone();
        page.id = 1000 + i * 100;
        page.name = (i + 1).to_string();
        for w in &mut page.wires { w.id += i * 100; }
        for d in &mut page.cable_defs { d.id += i * 100; }
        prj.pages.push(page);
    }
    let original = prj.clone();
    let idx = Index::build(&prj);
    let start = std::time::Instant::now();
    let mut count = 0;
    for pi in 0..prj.pages.len() {
        let labels = core_labels_with_index(&prj, &idx, pi);
        assert_eq!(labels.len(), 6);
        assert!(labels.iter().all(|l| !l.conflict));
        count += labels.len();
        for w in &prj.pages[pi].wires { assert!(core_text_bounds(&labels, w, CoreText::Core).is_some()); }
    }
    eprintln!("100 synthetic pages, {count} labels, one shared index: {:?}", start.elapsed());
    assert_eq!(prj, original);
}
