use super::*;
use crate::analysis::Analysis;
use crate::issue::Severity;
use crate::sample::demo_project;

/// The demo with a control point on its cable -W1 where it runs along the bottom duct.
fn with_control() -> (Project, V3) {
    let mut prj = demo_project();
    let an = Analysis::checked(&prj);
    let w1 = an.routing.item("-W1").unwrap();
    let q = [150.0, 712.5, 37.5];
    assert!(w1.points.windows(2).any(|s| grid::dist_to_segment(q, s[0], s[1]) < 1.0), "{:?}", w1.points);
    assert_eq!(add_control(&mut prj, &an.routing, "-W1", [150.0, 690.0, 40.0]).unwrap(), 0);
    (prj, q)
}

/// The errors of the analysis about control points.
fn control_errors(an: &Analysis) -> Vec<String> {
    an.issues.iter().filter(|i| i.severity == Severity::Error && i.message().contains("control point")).map(|i| i.message()).collect()
}

#[test]
fn a_control_point_is_added_on_the_route_and_passed() {
    let (prj, q) = with_control();
    // Put on the route where it runs nearest to the point given, relative to the cabinet.
    let c = &prj.layout.controls[0];
    assert_eq!((c.key.as_str(), c.points.len(), c.points[0].enclosure), ("-W1", 1, prj.layout.enclosures[0].id));
    assert!(dist(control_points(&prj, "-W1")[0], q) < 1.0, "{:?}", control_points(&prj, "-W1"));
    let an = Analysis::checked(&prj);
    let w1 = an.routing.item("-W1").unwrap();
    assert!(w1.routed() && control_errors(&an).is_empty(), "{:?}", control_errors(&an));
    assert_eq!(w1.controls.len(), 1);
    assert!(dist(w1.controls[0], q) <= prj.layout.routing.grid, "{:?}", w1.controls);
    assert!(w1.points.windows(2).any(|s| grid::dist_to_segment(w1.controls[0], s[0], s[1]) < 0.01));
    // The project file keeps it.
    let back: crate::panel::Layout = serde_json::from_str(&serde_json::to_string(&prj.layout).unwrap()).unwrap();
    assert_eq!(back.controls, prj.layout.controls);
    assert!(!serde_json::to_string(&crate::panel::Layout::default()).unwrap().contains("controls"));
}

#[test]
fn a_moved_control_point_leads_the_route_through_free_space() {
    let (mut prj, _) = with_control();
    let before = Analysis::checked(&prj).routing.item("-W1").unwrap().length;
    let to = [300.0, 650.0, 150.0];
    move_control(&mut prj, "-W1", 0, to).unwrap();
    let an = Analysis::checked(&prj);
    let w1 = an.routing.item("-W1").unwrap();
    assert!(control_errors(&an).is_empty(), "{:?}", control_errors(&an));
    assert_eq!(w1.controls.len(), 1);
    assert!(dist(w1.controls[0], to) <= prj.layout.routing.grid, "{:?}", w1.controls);
    assert!(w1.points.windows(2).any(|s| grid::dist_to_segment(w1.controls[0], s[0], s[1]) < 0.01));
    assert!(w1.length > before + 200.0, "{} {}", w1.length, before);
    // It keeps its bend radius: no bends closer than it.
    assert_eq!(grid::close_bends(&w1.points, 2.0 * w1.bend_radius), 0, "{:?}", w1.points);
    // Removed, the route is laid as before.
    assert_eq!(remove_control(&mut prj, "-W1", None).unwrap(), 1);
    assert!(prj.layout.controls.is_empty());
    let an = Analysis::checked(&prj);
    assert!((an.routing.item("-W1").unwrap().length - before).abs() < 0.5);
}

#[test]
fn a_control_point_in_a_device_or_outside_is_an_error() {
    let (mut prj, _) = with_control();
    // Into the body of -T1 on the plate, then behind the door.
    let t1 = prj.layout.enclosures[0].items.iter().find(|i| i.tag == "-T1").unwrap().clone();
    let (lo, hi) = crate::panel::item_box(&prj.layout.enclosures[0], &t1);
    let inside = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
    for at in [inside, [300.0, 650.0, 400.0]] {
        move_control(&mut prj, "-W1", 0, at).unwrap();
        let an = Analysis::checked(&prj);
        let w1 = an.routing.item("-W1").unwrap();
        let errors = control_errors(&an);
        assert!(errors.iter().any(|m| m.contains("Control point 1 of Cable -W1 lies inside something")), "{:?}", errors);
        // Laid without it: still routed, but not through it.
        assert!(w1.routed() && w1.controls.is_empty());
    }
}

#[test]
fn control_points_close_together_or_off_the_way_are_errors() {
    let (mut prj, _) = with_control();
    // Two points a cell apart that turn the cable back on itself.
    let an = Analysis::checked(&prj);
    move_control(&mut prj, "-W1", 0, [300.0, 650.0, 150.0]).unwrap();
    let an2 = Analysis::checked(&prj);
    assert!(control_errors(&an2).is_empty());
    add_control(&mut prj, &an2.routing, "-W1", [300.0, 650.0, 150.0]).unwrap();
    let points = &mut prj.layout.controls[0].points;
    assert_eq!(points.len(), 2);
    let first = points[0].at;
    points[1].at = [first[0], first[1] + 25.0, first[2]];
    points[0].at = [first[0], first[1], first[2]];
    let an3 = Analysis::checked(&prj);
    let w1 = an3.routing.item("-W1").unwrap();
    assert!(!control_errors(&an3).is_empty() || w1.controls.len() == 2, "{:?}", control_errors(&an3));
    // A point of a cabinet the cable does not run through.
    let other = crate::panel::new_enclosure(&mut prj, crate::panel::ENCLOSURE_PART, "+CAB9");
    let id = other.id;
    prj.layout.enclosures.push(other);
    prj.layout.controls[0].points = vec![ControlPoint { enclosure: id, at: [100.0, 100.0, 100.0] }];
    let an4 = Analysis::checked(&prj);
    assert!(control_errors(&an4).iter().any(|m| m.contains("Control point 1 of Cable -W1 lies off its way")), "{:?}", control_errors(&an4));
    assert!(an4.routing.item("-W1").unwrap().routed());
    drop(an);
}

#[test]
fn editing_control_points_checks_the_route_and_numbers() {
    let mut prj = demo_project();
    let an = Analysis::checked(&prj);
    assert!(add_control(&mut prj, &an.routing, "-NONE", [0.0; 3]).is_err());
    assert!(move_control(&mut prj, "-W1", 0, [0.0; 3]).is_err());
    assert!(remove_control(&mut prj, "-W1", Some(0)).is_err());
    assert_eq!(remove_control(&mut prj, "-W1", None).unwrap(), 0);
    // A frozen route is released: the control point leads it.
    crate::routing3d::freeze(&mut prj, &an.routing, &["-W1".to_string()]);
    let w1 = an.routing.item("-W1").unwrap();
    let (a, b) = (w1.points[2], w1.points[3]);
    let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0];
    add_control(&mut prj, &an.routing, "-W1", mid).unwrap();
    assert!(prj.layout.frozen.iter().all(|f| f.key != "-W1"));
    // A second point before the first along the route goes first.
    add_control(&mut prj, &an.routing, "-W1", w1.points[1]).unwrap();
    let pts = control_points(&prj, "-W1");
    assert_eq!(pts.len(), 2);
    assert!(dist(pts[0], w1.points[1]) < 1.0 && dist(pts[1], mid) < 1.0, "{:?}", pts);
    assert_eq!(remove_control(&mut prj, "-W1", Some(1)).unwrap(), 1);
    assert_eq!(control_points(&prj, "-W1").len(), 1);
}

#[test]
fn every_failure_has_a_message() {
    for f in [Failure::Blocked(0), Failure::NoWay(1), Failure::Tight, Failure::Itself] {
        let m = message(f, RouteKind::Cable, "-W1", 30.0);
        assert!(m.contains("-W1"), "{m}");
    }
    assert_eq!(arrival(&[[0.0; 3], [0.0, 10.0, 0.5]]), [0.0, 1.0, 0.0]);
    assert_eq!(arrival(&[[0.0; 3]]), [0.0; 3]);
}
