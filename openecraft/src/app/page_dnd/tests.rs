use super::*;

/// Cover page 1, section 1 with pages 1.1, 1.2, subsection 1.1 with page 1.1.1 and
/// section 2.
fn project() -> (Project, u64, u64, u64) {
    let mut prj = Project::default();
    let a = sections::add(&mut prj, 0, "Supply");
    let b = sections::add(&mut prj, a, "Main feed");
    let c = sections::add(&mut prj, 0, "Controls");
    for s in [a, a, b] {
        openecraft_core::ops::add_page_in(&mut prj, PageKind::Schematic, s);
    }
    (prj, a, b, c)
}

fn probe(dragged: Dragged, prj: &Project, y: f32) -> Probe {
    let blocked = match dragged {
        Dragged::Section(id) => sections::subtree(prj, id),
        Dragged::Page(_) => Vec::new(),
    };
    Probe { enabled: true, dragged: Some(dragged), pointer: Some(egui::pos2(10.0, y)), blocked, ..Probe::off() }
}

fn row(top: f32) -> egui::Rect {
    egui::Rect::from_min_max(egui::pos2(0.0, top), egui::pos2(100.0, top + 20.0))
}

#[test]
fn collapsed_layout_groups_keep_drop_markers_below_the_visible_rows() {
    let (prj, _, _, _) = project();
    let mut over = probe(Dragged::Page(0), &prj, 10.0);
    over.group_row(row(0.0));
    over.below(None);
    assert_eq!(over.target, None);
    let mut below = probe(Dragged::Page(0), &prj, 30.0);
    below.group_row(row(0.0));
    below.below(None);
    assert_eq!(below.target, Some((DropAt::Page { section: 0, before: None }, Mark::Line { y: 20.0, left: 0.0 })));
}

#[test]
fn rows_are_split_into_zones() {
    assert_eq!(zone(0.0, 20.0, 4.0, false), Zone::Above);
    assert_eq!(zone(0.0, 20.0, 12.0, false), Zone::Below);
    assert_eq!(zone(0.0, 20.0, 4.0, true), Zone::Above);
    assert_eq!(zone(0.0, 20.0, 10.0, true), Zone::Into);
    assert_eq!(zone(0.0, 20.0, 18.0, true), Zone::Below);
}

#[test]
fn a_page_drops_between_pages_and_onto_sections() {
    let (prj, a, _, c) = project();
    assert_eq!(prj.pages.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["1", "1.1", "1.2", "1.1.1"]);
    // The lower half of page 1.1 is before page 1.2.
    let mut p = probe(Dragged::Page(0), &prj, 35.0);
    p.page(&prj, row(20.0), 1);
    assert_eq!(p.target, Some((DropAt::Page { section: a, before: Some(2) }, Mark::Line { y: 40.0, left: 0.0 })));
    // The lower half of page 1.2, the last of its section: after the section's pages.
    let mut p = probe(Dragged::Page(0), &prj, 55.0);
    p.page(&prj, row(40.0), 2);
    assert_eq!(p.target.map(|t| t.0), Some(DropAt::Page { section: a, before: None }));
    // A section header takes the page whole.
    let mut p = probe(Dragged::Page(0), &prj, 62.0);
    p.section_at(&prj, row(60.0), 80.0, c);
    assert_eq!(p.target, Some((DropAt::Page { section: c, before: None }, Mark::Into(row(60.0)))));
    assert_eq!(p.over_section, Some((c, egui::Id::NULL)));
    // Below the rows: the end of the top level.
    let mut p = probe(Dragged::Page(1), &prj, 120.0);
    p.page(&prj, row(0.0), 0);
    p.below(None);
    assert_eq!(p.target, Some((DropAt::Page { section: 0, before: None }, Mark::Line { y: 20.0, left: 0.0 })));
}

#[test]
fn a_section_drops_beside_or_into_a_section_but_not_into_itself() {
    let (prj, a, b, c) = project();
    let mut p = probe(Dragged::Section(c), &prj, 2.0);
    p.section_at(&prj, row(0.0), 80.0, a);
    assert_eq!(p.target, Some((DropAt::Section { parent: 0, before: Some(a) }, Mark::Line { y: 0.0, left: 0.0 })));
    let mut p = probe(Dragged::Section(c), &prj, 10.0);
    p.section_at(&prj, row(0.0), 80.0, a);
    assert_eq!(p.target.map(|t| t.0), Some(DropAt::Section { parent: a, before: None }));
    // Below an open section: after its whole block.
    let mut p = probe(Dragged::Section(b), &prj, 18.0);
    p.section_at(&prj, row(0.0), 80.0, a);
    assert_eq!(p.target, Some((DropAt::Section { parent: 0, before: Some(c) }, Mark::Line { y: 80.0, left: 0.0 })));
    // Section 1 cannot go into itself or into its subsection.
    for (id, y) in [(a, 10.0), (b, 10.0), (b, 2.0)] {
        let mut p = probe(Dragged::Section(a), &prj, y);
        p.section_at(&prj, row(0.0), 20.0, id);
        assert!(p.refused && p.target.is_none());
        p.below(None);
        assert!(p.target.is_none());
    }
}

#[test]
fn drops_that_change_nothing_are_recognised() {
    let (prj, a, b, c) = project();
    assert!(unchanged(&prj, Dragged::Page(1), DropAt::Page { section: a, before: Some(1) }));
    assert!(unchanged(&prj, Dragged::Page(1), DropAt::Page { section: a, before: Some(2) }));
    assert!(!unchanged(&prj, Dragged::Page(1), DropAt::Page { section: a, before: None }));
    assert!(unchanged(&prj, Dragged::Page(2), DropAt::Page { section: a, before: None }));
    assert!(!unchanged(&prj, Dragged::Page(2), DropAt::Page { section: b, before: None }));
    assert!(unchanged(&prj, Dragged::Section(a), DropAt::Section { parent: 0, before: Some(c) }));
    assert!(unchanged(&prj, Dragged::Section(c), DropAt::Section { parent: 0, before: None }));
    assert!(!unchanged(&prj, Dragged::Section(c), DropAt::Section { parent: 0, before: Some(a) }));
    assert!(unchanged(&prj, Dragged::Section(b), DropAt::Section { parent: a, before: None }));
}
