use super::*;
use openecraft_core::sample;

#[test]
fn pages_are_drawn_once_and_unknown_ones_not_at_all() {
    let prj = sample::demo_project();
    let id = prj.pages[1].id;
    let v = Viewed::new(prj, 3);
    let svg = v.page_svg(id).unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(Arc::ptr_eq(&svg, &v.page_svg(id).unwrap()), "drawn once");
    assert!(v.page_svg(u64::MAX).is_none());
    assert_eq!(v.view(Lang::En).pages.len(), v.project.pages.len());
}

#[test]
fn views_follow_the_revision_and_keep_the_last_used() {
    let mut views = Views::default();
    let view = Arc::new(Viewed::new(Project::default(), 1));
    views.put("a", view.clone());
    assert!(views.get("a", 1).is_some());
    assert!(views.get("a", 2).is_none(), "another revision is analysed again");
    for i in 0..KEPT {
        views.put(&format!("p{i}"), view.clone());
        // "a" stays in use.
        assert!(views.get("a", 1).is_some());
    }
    assert_eq!(views.kept.len(), KEPT);
    assert!(views.get("a", 1).is_some());
    assert!(views.get("p0", 1).is_none(), "the one used longest ago is dropped");
    views.remove("a");
    assert!(views.get("a", 1).is_none());
}
