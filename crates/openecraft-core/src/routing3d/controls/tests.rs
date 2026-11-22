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
    // A second point before the first along the route goes first; given on a corner of
    // the route it goes back along the stretch by the bend radius (no room for the
    // arcs on the corner).
    add_control(&mut prj, &an.routing, "-W1", w1.points[1]).unwrap();
    let pts = control_points(&prj, "-W1");
    assert_eq!(pts.len(), 2);
    let off = dist(pts[0], w1.points[1]);
    assert!(off > w1.bend_radius.min(dist(w1.points[0], w1.points[1]) / 2.0) - 1.0 && dist(pts[1], mid) < 1.0, "{:?} {off}", pts);
    assert!(w1.points.windows(2).take(2).any(|s| grid::dist_to_segment(pts[0], s[0], s[1]) < 0.5), "on the route: {:?}", pts[0]);
    assert_eq!(remove_control(&mut prj, "-W1", Some(1)).unwrap(), 1);
    assert_eq!(control_points(&prj, "-W1").len(), 1);
}

/// A wire of the demo laid along the cable ducts (not to the door) and the middle of
/// the longest stretch of its route there.
fn duct_wire(an: &Analysis) -> (String, V3) {
    let it = an
        .routing
        .items
        .iter()
        .filter(|i| i.kind == RouteKind::Wire && i.routed())
        .filter(|i| i.wire.is_some_and(|wi| an.wiring[wi].door == (None, None) && !crate::routing3d::wire_in_3d(&demo_project().layout, &an.wiring[wi])))
        .max_by(|a, b| a.length.total_cmp(&b.length))
        .expect("a wire in the ducts");
    let s = it.points.windows(2).max_by(|a, b| dist(a[0], a[1]).total_cmp(&dist(b[0], b[1]))).unwrap();
    (it.key.clone(), [(s[0][0] + s[1][0]) / 2.0, (s[0][1] + s[1][1]) / 2.0, (s[0][2] + s[1][2]) / 2.0])
}

#[test]
fn a_wire_with_control_points_is_led_through_them_in_3d() {
    let mut prj = demo_project();
    let an = Analysis::checked(&prj);
    let (key, mid) = duct_wire(&an);
    assert_eq!(add_control(&mut prj, &an.routing, &key, mid).unwrap(), 0);
    let wire = |an: &Analysis| an.wiring.iter().find(|w| crate::routing3d::wire_key(w) == key).unwrap().clone();
    // Routed in 3D from its connection points through the control point.
    let an = Analysis::checked(&prj);
    assert!(crate::routing3d::wire_in_3d(&prj.layout, &wire(&an)));
    let it = an.routing.item(&key).unwrap();
    assert!(control_errors(&an).is_empty(), "{:?}", control_errors(&an));
    assert_eq!(it.controls.len(), 1, "{key}");
    assert!(dist(it.controls[0], mid) <= prj.layout.routing.grid, "{:?} {:?}", it.controls, mid);
    assert!(it.points.windows(2).any(|s| grid::dist_to_segment(it.controls[0], s[0], s[1]) < 0.01));
    // Moved into a device it is an error, the wire is laid without it.
    let t1 = prj.layout.enclosures[0].items.iter().find(|i| i.tag == "-T1").unwrap().clone();
    let (lo, hi) = crate::panel::item_box(&prj.layout.enclosures[0], &t1);
    move_control(&mut prj, &key, 0, [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0]).unwrap();
    let an = Analysis::checked(&prj);
    assert!(control_errors(&an).iter().any(|m| m.contains("Control point 1 of Wire") && m.contains("lies inside something")), "{:?}", control_errors(&an));
    let it = an.routing.item(&key).unwrap();
    assert!(it.routed() && it.controls.is_empty());
    // Without control points it is laid along the ducts again.
    remove_control(&mut prj, &key, None).unwrap();
    assert!(!crate::routing3d::wire_in_3d(&prj.layout, &wire(&Analysis::checked(&prj))));
}

#[test]
fn a_wire_to_the_door_cannot_take_control_points() {
    let mut prj = demo_project();
    let an = Analysis::checked(&prj);
    let w = an.wiring.iter().find(|w| w.door != (None, None)).expect("a wire to the door");
    let key = crate::routing3d::wire_key(w);
    assert!(!crate::routing3d::wire_in_3d(&prj.layout, w));
    prj.layout.controls.push(RouteControl { key: key.clone(), points: vec![ControlPoint { enclosure: prj.layout.enclosures[0].id, at: [100.0, 100.0, 100.0] }] });
    let an = Analysis::checked(&prj);
    let errors: Vec<String> = an.issues.iter().filter(|i| i.severity == Severity::Error).map(|i| i.message()).collect();
    assert!(errors.iter().any(|m| m.contains("runs to the door: it cannot be led through its control points")), "{:?}", errors);
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

#[test]
fn a_control_point_is_passed_or_its_error_is_told_wherever_it_is_moved() {
    let mut prj = demo_project();
    let an = Analysis::checked(&prj);
    let (wire, mid) = duct_wire(&an);
    add_control(&mut prj, &an.routing, &wire, mid).unwrap();
    let w1 = an.routing.item("-W1").unwrap().points[2];
    add_control(&mut prj, &an.routing, "-W1", w1).unwrap();
    let e = &prj.layout.enclosures[0];
    let (w, h, d) = (e.width, e.height, e.depth);
    let mut silent = Vec::new();
    for (key, at) in [(wire.as_str(), 0usize), ("-W1", 0usize)] {
        for fx in [0.1, 0.3, 0.5, 0.7, 0.9] {
            for fy in [0.15, 0.5, 0.85] {
                for fz in [0.2, 0.5, 0.8] {
                    let q = [fx * w, fy * h, fz * d];
                    move_control(&mut prj, key, at, q).unwrap();
                    let an = Analysis::checked(&prj);
                    let it = an.routing.item(key).unwrap();
                    if it.controls.is_empty() && it.control_error.is_none() {
                        silent.push((key.to_string(), q));
                    }
                    if !it.controls.is_empty() {
                        assert!(it.control_error.is_none(), "{key} {q:?}: {:?}", it.control_error);
                    }
                }
            }
        }
    }
    assert!(silent.is_empty(), "laid without its control point and no error: {silent:?}");
}

#[test]
fn a_wire_with_control_points_runs_from_its_connection_points() {
    let mut prj = demo_project();
    let an = Analysis::checked(&prj);
    let (key, mid) = duct_wire(&an);
    let before = an.routing.item(&key).unwrap().clone();
    add_control(&mut prj, &an.routing, &key, mid).unwrap();
    let an = Analysis::checked(&prj);
    let it = an.routing.item(&key).unwrap();
    // Not from the plate behind its devices: from their connection points, where a wire
    // routed between points starts.
    let plate = crate::panel::positions(&prj.layout)[0][2] + crate::panel::PLATE_Z + 5.0;
    for (k, end) in it.ends.iter().enumerate() {
        let q = end.point.unwrap();
        assert!(q[2] > plate + 1.0, "end {k} of {key} on the plate: {q:?} (before {:?})", before.ends[k].point);
        let at = if k == 0 { it.points[0] } else { *it.points.last().unwrap() };
        assert!(dist(at, q) < 30.0, "{at:?} {q:?}");
    }
}

#[test]
fn a_long_detour_through_a_control_point_does_not_run_into_itself() {
    let mut prj = demo_project();
    let an = Analysis::checked(&prj);
    let (key, mid) = duct_wire(&an);
    add_control(&mut prj, &an.routing, &key, mid).unwrap();
    let e = &prj.layout.enclosures[0];
    // To the corners of the cabinet and back: the way back keeps clear of the way there
    // (it may cross it in a cable duct).
    for q in [[0.05 * e.width, 0.05 * e.height, 0.8 * e.depth], [0.95 * e.width, 0.95 * e.height, 0.2 * e.depth]] {
        move_control(&mut prj, &key, 0, q).unwrap();
        let an = Analysis::checked(&prj);
        let it = an.routing.item(&key).unwrap();
        assert!(it.controls.len() == 1 && it.control_error.is_none(), "{q:?}: {:?}", it.control_error);
    }
}

#[test]
fn a_changed_control_point_is_routed_at_once_among_the_other_routes() {
    let prj = demo_project();
    let mut cache = crate::analysis::Cache::default();
    let an = Analysis::build_with(&prj, &mut cache);
    let mut moved = prj.clone();
    add_control(&mut moved, &an.routing, "-W1", an.routing.item("-W1").unwrap().points[2]).unwrap();
    move_control(&mut moved, "-W1", 0, [300.0, 650.0, 150.0]).unwrap();
    // Until the whole routing is searched again: the route of -W1 searched now, the
    // others where they were.
    let quick = Analysis::build_kept(&moved, &mut cache.clone(), &an.routing);
    assert!(quick.routes_pending);
    assert_eq!(quick.routing.rerouted, vec!["-W1".to_string()]);
    let full = Analysis::build_with(&moved, &mut cache.clone());
    assert!(full.routing.rerouted.is_empty());
    let (q, f) = (quick.routing.item("-W1").unwrap(), full.routing.item("-W1").unwrap());
    assert_eq!((q.controls.len(), q.points.clone(), q.control_error.clone()), (f.controls.len(), f.points.clone(), f.control_error.clone()));
    assert_eq!(quick.routing.items.len(), an.routing.items.len());
    for it in quick.routing.items.iter().filter(|i| i.key != "-W1") {
        assert_eq!(Some(&it.points), an.routing.item(&it.key).map(|i| &i.points), "{}", it.key);
    }
    // More than control points changed: the routing before stands in for it.
    let mut other = moved.clone();
    other.layout.routing.grid += 5.0;
    let interim = Analysis::build_kept(&other, &mut cache.clone(), &an.routing);
    assert!(interim.routes_pending && interim.routing.rerouted.is_empty());
}
