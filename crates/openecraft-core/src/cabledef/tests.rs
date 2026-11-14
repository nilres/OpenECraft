use super::*;
use crate::geom::p;

fn project() -> Project {
    let mut prj = Project::default();
    for (i, x) in [20.0, 28.0, 36.0].into_iter().enumerate() {
        prj.pages[0].wires.push(Wire { id: 10 + i as u64, a: p(x, 20.0), b: p(x, 80.0), ..Default::default() });
    }
    prj.fix_next_id();
    prj
}

fn five_wires() -> Project {
    let mut prj = project();
    for (id, x) in [(13, 44.), (14, 52.)] {
        prj.pages[0].wires.push(Wire { id, a: p(x, 20.), b: p(x, 80.), ..Default::default() });
    }
    prj.cables.push(Cable { id: 90, tag: "-W1".into(), cores: "BN BK GY GNYE".into(), cross_section: "1.5".into(), ..Default::default() });
    prj.fix_next_id();
    prj
}

#[test]
fn a_rejected_new_cable_keeps_a_correctable_definition_without_inventing_a_cable() {
    for target in ["", "-Unknown"] {
        let mut prj = project();
        prj.pages[0].wires[0].cable = "-Other".into();
        prj.pages[0].wires[0].core = "BN".into();
        let before = prj.clone();
        let id = add(&mut prj, 0, p(16., 50.), p(40., 50.), target);
        assert_eq!(prj.cables, before.cables);
        assert_eq!(prj.pages[0].wires, before.pages[0].wires);
        assert_eq!(prj.pages[0].cable_defs[0].cable, target);
        assert_eq!(problems(&prj)[0].target, Some((0, id)));
        assert_eq!(Project::from_json(&prj.to_json()).unwrap(), prj);
        // The user can select a real cable and explicitly clear the conflict.
        prj.cables.push(Cable { id: 90, tag: "-W1".into(), cores: "1 2 3".into(), ..Default::default() });
        prj.pages[0].cable_defs[0].cable = "-W1".into();
        prj.pages[0].wires[0].cable.clear();
        prj.pages[0].wires[0].core.clear();
        assert_eq!(apply(&mut prj, 0, id), 3);
        assert!(problems(&prj).is_empty());
    }
}

#[test]
fn an_imported_core_without_a_cable_is_a_conflict_and_is_never_overwritten() {
    let mut prj = project();
    prj.pages[0].wires[0].core = "Imported".into();
    let wires = prj.pages[0].wires.clone();
    let preview = plan(&prj, 0, p(16., 50.), p(40., 50.), "");
    assert_eq!(preview.conflicts, [10]);
    assert!(!preview.can_apply());
    let id = add(&mut prj, 0, p(16., 50.), p(40., 50.), "");
    assert_eq!(apply(&mut prj, 0, id), 0);
    assert!(prj.cables.is_empty());
    assert_eq!(prj.pages[0].wires, wires);
}

#[test]
fn an_imported_unknown_cable_reference_does_not_create_an_unrelated_unused_cable() {
    let mut prj = project();
    prj.pages[0].wires[0].cable = "-Unknown".into();
    let wires = prj.pages[0].wires.clone();
    let preview = plan(&prj, 0, p(16., 50.), p(40., 50.), "-Unknown");
    assert_eq!(preview.conflicts, [10]);
    let id = add(&mut prj, 0, p(16., 50.), p(40., 50.), "-Unknown");
    assert!(prj.cables.is_empty());
    assert_eq!(prj.pages[0].wires, wires);
    assert_eq!(prj.pages[0].cable_defs[0].id, id);
    assert_eq!(prj.pages[0].cable_defs[0].cable, "-Unknown");
}

#[test]
fn indexed_previews_follow_other_pages_edits_and_undo() {
    let mut prj = five_wires();
    let mut other = Page::new(100, "2", "", PageKind::Schematic);
    other.wires.push(Wire { id: 101, cable: "-W1".into(), core: "BN".into(), ..Default::default() });
    prj.pages.push(other);
    let mut history = crate::history::History::new(&prj);
    let check = |prj: &Project| {
        let idx = crate::index::Index::build(prj);
        for b in [p(32., 50.), p(56., 50.)] {
            assert_eq!(plan_with_uses(prj, 0, p(16., 50.), b, "-W1", idx.cable_cores.get("-W1").unwrap()), plan(prj, 0, p(16., 50.), b, "-W1"));
        }
        plan_with_uses(prj, 0, p(16., 50.), p(32., 50.), "-W1", idx.cable_cores.get("-W1").unwrap()).free_cores
    };
    assert!(!check(&prj).contains(&"BN".into()));
    prj.pages[1].wires[0].core = "BK".into();
    history.track(&prj, true);
    assert!(!check(&prj).contains(&"BK".into()));
    history.undo(&mut prj);
    assert!(!check(&prj).contains(&"BN".into()));
    history.redo(&mut prj);
    assert!(!check(&prj).contains(&"BK".into()));
}

#[test]
fn five_crossings_need_five_cores_and_never_partially_assign_in_either_direction() {
    for (a, b) in [(p(16., 50.), p(56., 50.)), (p(56., 50.), p(16., 50.))] {
        let mut prj = five_wires();
        let before = prj.clone();
        let preview = plan(&prj, 0, a, b, "-W1");
        assert_eq!(prj, before, "preview is read-only");
        assert_eq!((preview.crossed.len(), preview.free_cores.len(), preview.missing.len(), preview.unassigned()), (5, 4, 1, 5));
        assert!(!preview.can_apply());
        let id = add(&mut prj, 0, a, b, "-W1");
        assert_eq!(prj.pages[0].wires, before.pages[0].wires);
        assert_eq!(prj.cables, before.cables, "never invent a larger physical cable");
        assert_eq!(apply(&mut prj, 0, id), 0);
        assert_eq!(label(&prj, &prj.pages[0], &prj.pages[0].cable_defs[0]), ["-W1", "4x1.5"]);
        assert_eq!(problems(&prj).len(), 1);
        assert_eq!(problems(&prj)[0].target, Some((0, id)));
        prj.cables[0].cores = "BN BK GY BU GNYE".into();
        assert_eq!(apply(&mut prj, 0, id), 5);
        let actual: Vec<&str> = preview.crossed.iter().map(|id| prj.pages[0].wires.iter().find(|w| w.id == *id).unwrap().core.as_str()).collect();
        assert_eq!(actual, ["BN", "BK", "GY", "BU", "GNYE"]);
        assert!(problems(&prj).is_empty());
        let assigned = prj.clone();
        assert_eq!(apply(&mut prj, 0, id), 5);
        assert_eq!(prj, assigned, "reapplying keeps existing core order and positions");
        assert_eq!(Project::from_json(&prj.to_json()).unwrap(), prj);
    }
}

#[test]
fn other_pages_and_other_cables_keep_their_assignments_and_label_positions() {
    let mut prj = five_wires();
    prj.cables[0].cores = "BN BK GY BU GNYE".into();
    prj.pages[0].wires[0].cable = "-W1".into();
    prj.pages[0].wires[0].core = "BN".into();
    prj.pages[0].wires[0].core_offset = Some(p(3., 4.));
    prj.pages[0].wires[1].cable = "-Other".into();
    prj.pages[0].wires[1].core = "1".into();
    let mut other = Page::new(100, "2", "", PageKind::Schematic);
    other.wires.push(Wire { id: 101, a: p(20., 20.), b: p(20., 80.), cable: "-W1".into(), core: "BK".into(), ..Default::default() });
    prj.pages.push(other);
    let before = prj.clone();
    let id = add(&mut prj, 0, p(16., 50.), p(56., 50.), "-W1");
    assert_eq!(apply(&mut prj, 0, id), 1);
    assert_eq!(prj.pages[0].wires, before.pages[0].wires);
    assert_eq!(prj.pages[1], before.pages[1]);
    let preview = plan(&prj, 0, p(16., 50.), p(56., 50.), "-W1");
    assert_eq!(preview.conflicts, [11]);
    assert_eq!(preview.free_cores, ["GY", "BU", "GNYE"]);
    // An explicit correction resolves both the conflict and the shortage.
    prj.pages[0].wires[1].cable.clear(); prj.pages[0].wires[1].core.clear();
    prj.cables[0].cores.push_str(" RD");
    assert_eq!(apply(&mut prj, 0, id), 5);
    assert_eq!(prj.pages[0].wires[0], before.pages[0].wires[0]);
    assert_eq!(prj.pages[1], before.pages[1]);
    assert!(prj.pages[0].wires.iter().all(|w| w.core != "BK"));
}

#[test]
fn duplicate_core_names_do_not_create_extra_free_cores() {
    let mut prj = five_wires();
    prj.cables[0].cores = "BN BN BK GY GNYE".into();
    let preview = plan(&prj, 0, p(16., 50.), p(56., 50.), "-W1");
    assert_eq!(preview.free_cores.len(), 4);
    assert_eq!(preview.missing.len(), 1);
    add(&mut prj, 0, p(16., 50.), p(56., 50.), "-W1");
    assert!(prj.pages[0].wires.iter().all(|w| w.cable.is_empty()));
}

#[test]
fn fresh_cable_and_corrected_assignment_survive_undo_and_redo() {
    let mut prj = five_wires();
    let before = prj.clone();
    let mut history = crate::history::History::new(&prj);
    let id = add(&mut prj, 0, p(56., 50.), p(16., 50.), "");
    let tag = prj.pages[0].cable_defs.iter().find(|d| d.id == id).unwrap().cable.clone();
    assert_eq!(prj.cable(&tag).unwrap().cores, "1 2 3 4 5");
    assert!(prj.pages[0].wires.iter().all(|w| w.cable == tag));
    let after = prj.clone();
    history.track(&prj, true); history.undo(&mut prj); assert_eq!(prj, before);
    history.redo(&mut prj); assert_eq!(prj, after);
}

#[test]
fn crossed_wires_in_order() {
    let prj = project();
    assert_eq!(crossed_wires(&prj.pages[0], p(40.0, 50.0), p(16.0, 50.0)), vec![12, 11, 10]);
    assert!(crossed_wires(&prj.pages[0], p(40.0, 90.0), p(16.0, 90.0)).is_empty());
}

#[test]
fn new_cable_gets_numbered_cores() {
    let mut prj = project();
    let id = add(&mut prj, 0, p(16.0, 50.0), p(32.0, 50.0), "");
    assert_eq!(prj.cables.len(), 1);
    assert_eq!(prj.cables[0].cores, "1 2");
    let tag = prj.cables[0].tag.clone();
    assert_eq!(prj.pages[0].cable_defs[0].id, id);
    let cores: Vec<(&str, &str)> = prj.pages[0].wires.iter().map(|w| (w.cable.as_str(), w.core.as_str())).collect();
    assert_eq!(cores, vec![(tag.as_str(), "1"), (tag.as_str(), "2"), ("", "")]);
    assert!(problems(&prj).is_empty());
}

#[test]
fn existing_cable_keeps_used_cores() {
    let mut prj = project();
    prj.cables.push(Cable { id: 90, tag: "-W5".into(), cores: "BN BK GY".into(), ..Default::default() });
    prj.pages[0].wires[1].cable = "-W5".into();
    prj.pages[0].wires[1].core = "BN".into();
    add(&mut prj, 0, p(16.0, 50.0), p(40.0, 50.0), "-W5");
    let cores: Vec<&str> = prj.pages[0].wires.iter().map(|w| w.core.as_str()).collect();
    assert_eq!(cores, vec!["BK", "BN", "GY"]);
}

#[test]
fn problems_unknown_cable_and_nothing_crossed() {
    let mut prj = project();
    prj.pages[0].cable_defs.push(CableDef { id: 1, a: p(0.0, 0.0), b: p(4.0, 0.0), cable: "-W9".into(), shield: false });
    prj.cables.push(Cable { id: 90, tag: "-W5".into(), ..Default::default() });
    prj.pages[0].cable_defs.push(CableDef { id: 2, a: p(0.0, 0.0), b: p(4.0, 0.0), cable: "-W5".into(), shield: false });
    let issues = problems(&prj);
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0].severity, Severity::Error);
}

#[test]
fn numbered_core_list() {
    assert_eq!(numbered_cores(3, true), "1 2 3 GNYE");
    assert_eq!(numbered_cores(2, false), "1 2");
}

#[test]
fn cores_sit_where_the_cable_definition_line_crosses() {
    let mut prj = project();
    add(&mut prj, 0, p(16.0, 50.0), p(40.0, 35.0), "");
    let page = &prj.pages[0];
    for w in &page.wires {
        let t = core_position(page, w);
        let y = w.a.y + (w.b.y - w.a.y) * t;
        let on_line = 50.0 + (w.a.x - 16.0) * (35.0 - 50.0) / 24.0;
        assert!((y - on_line).abs() < 1e-3, "{} {y} {on_line}", w.id);
    }
    let other = Wire { id: 99, a: p(60.0, 20.0), b: p(60.0, 80.0), cable: page.wires[0].cable.clone(), ..Default::default() };
    assert_eq!(core_position(page, &other), 0.3, "not crossed: at 30 %");
}
