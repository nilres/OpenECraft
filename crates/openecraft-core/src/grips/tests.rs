use super::*;

fn shape(prj: &mut Project, kind: ShapeKind, a: P, b: P) -> u64 {
    let id = prj.new_id();
    prj.pages[0].shapes.push(Shape { id, kind, a, b, ..Default::default() });
    id
}

fn shape_of(prj: &Project, id: u64) -> &Shape {
    prj.pages[0].shapes.iter().find(|s| s.id == id).unwrap()
}

/// Drags grip `grip` of item `id` on the first page to `to`.
fn pull(prj: &mut Project, id: u64, grip: Grip, to: P) -> Result<(), String> {
    let g = grab(prj, 0, id, grip).unwrap();
    drag(prj, 0, &g, to)
}

fn close(a: P, b: P) -> bool {
    a.dist(b) < 0.01
}

#[test]
fn a_rectangle_has_grips_at_its_corners_and_edges() {
    let mut prj = Project::default();
    let id = shape(&mut prj, ShapeKind::Rect, p(30.0, 20.0), p(10.0, 40.0));
    let g = grips(&prj, 0, id);
    assert_eq!(g.len(), 8);
    assert!(g.contains(&(Grip::Frame(-1, -1), p(10.0, 20.0))));
    assert!(g.contains(&(Grip::Frame(0, 1), p(20.0, 40.0))));
    assert!(g.contains(&(Grip::Frame(1, 0), p(30.0, 30.0))));
    // Texts, symbols and wires have none.
    assert!(grips(&prj, 0, 999).is_empty());
}

#[test]
fn a_corner_grip_moves_two_edges_and_an_edge_grip_one() {
    let mut prj = Project::default();
    let id = shape(&mut prj, ShapeKind::Rect, p(10.0, 20.0), p(30.0, 40.0));
    let corner = grab(&prj, 0, id, Grip::Frame(1, 1)).unwrap();
    drag(&mut prj, 0, &corner, p(50.0, 45.0)).unwrap();
    assert_eq!((shape_of(&prj, id).a, shape_of(&prj, id).b), (p(10.0, 20.0), p(50.0, 45.0)));
    // Every step starts from the shape at the start of the drag.
    drag(&mut prj, 0, &corner, p(40.0, 30.0)).unwrap();
    assert_eq!(shape_of(&prj, id).b, p(40.0, 30.0));
    let left = grab(&prj, 0, id, Grip::Frame(-1, 0)).unwrap();
    drag(&mut prj, 0, &left, p(0.0, 99.0)).unwrap();
    assert_eq!((shape_of(&prj, id).a, shape_of(&prj, id).b), (p(0.0, 20.0), p(40.0, 30.0)));
    // An edge dragged over the opposite one stops short of it.
    drag(&mut prj, 0, &left, p(60.0, 0.0)).unwrap();
    assert_eq!(shape_of(&prj, id).a, p(40.0 - MIN_SIZE, 20.0));
}

#[test]
fn lines_polygons_and_dimensions_move_their_points() {
    let mut prj = Project::default();
    let line = shape(&mut prj, ShapeKind::Line, p(0.0, 0.0), p(10.0, 0.0));
    pull(&mut prj, line, Grip::Point(1), p(10.0, 10.0)).unwrap();
    assert_eq!(shape_of(&prj, line).b, p(10.0, 10.0));
    let poly = shape(&mut prj, ShapeKind::Polygon, p(0.0, 0.0), p(0.0, 0.0));
    prj.pages[0].shapes.last_mut().unwrap().pts = vec![p(0.0, 0.0), p(10.0, 0.0), p(5.0, 8.0)];
    assert_eq!(grips(&prj, 0, poly).len(), 3);
    pull(&mut prj, poly, Grip::Point(2), p(5.0, 20.0)).unwrap();
    assert_eq!(shape_of(&prj, poly).pts[2], p(5.0, 20.0));
    assert!(grab(&prj, 0, poly, Grip::Point(3)).is_none());
    let id = prj.new_id();
    prj.pages[0].dimensions.push(Dimension { id, a: p(0.0, 50.0), b: p(20.0, 50.0), ..Default::default() });
    pull(&mut prj, id, Grip::Point(0), p(-10.0, 50.0)).unwrap();
    assert_eq!(prj.pages[0].dimensions[0].a, p(-10.0, 50.0));
}

#[test]
fn circles_change_their_radius_and_arcs_their_ends() {
    let mut prj = Project::default();
    let c = shape(&mut prj, ShapeKind::Circle, p(50.0, 50.0), p(60.0, 50.0));
    let g = grips(&prj, 0, c);
    assert_eq!(g.len(), 4);
    assert!(close(g[3].1, p(50.0, 40.0)));
    pull(&mut prj, c, Grip::Radius(1), p(50.0, 75.0)).unwrap();
    assert!((shape_of(&prj, c).a.dist(shape_of(&prj, c).b) - 25.0).abs() < 0.01);
    // A quarter arc from the right to the top of its centre.
    let arc = shape(&mut prj, ShapeKind::Arc, p(0.0, 0.0), p(10.0, 0.0));
    prj.pages[0].shapes.last_mut().unwrap().sweep = 90.0;
    let end = grips(&prj, 0, arc)[1].1;
    assert!(close(end, p(0.0, -10.0)));
    // Its end goes to the left: half a circle.
    pull(&mut prj, arc, Grip::ArcEnd, p(-10.0, 0.0)).unwrap();
    assert!((shape_of(&prj, arc).sweep - 180.0).abs() < 0.01);
    // Its start goes to the bottom, the end stays on the left.
    pull(&mut prj, arc, Grip::Point(1), p(0.0, 10.0)).unwrap();
    let s = shape_of(&prj, arc);
    assert!((s.sweep - 270.0).abs() < 0.01, "{}", s.sweep);
    assert!(close(grips(&prj, 0, arc)[1].1, p(-10.0, 0.0)));
}

#[test]
fn images_keep_their_aspect_ratio_and_have_corner_grips_only() {
    let mut prj = Project::default();
    let id = prj.new_id();
    prj.pages[0].images.push(ImageItem { id, a: p(10.0, 10.0), b: p(50.0, 30.0), px: [400, 200], ..Default::default() });
    assert_eq!(grips(&prj, 0, id).len(), 4);
    assert!(grab(&prj, 0, id, Grip::Frame(1, 0)).is_none());
    // The bottom right corner pulled to the right: twice as wide, twice as high.
    pull(&mut prj, id, Grip::Frame(1, 1), p(90.0, 20.0)).unwrap();
    let i = &prj.pages[0].images[0];
    assert_eq!((i.a, i.b), (p(10.0, 10.0), p(90.0, 50.0)));
    // The top left corner pulled up: the bottom right corner stays.
    pull(&mut prj, id, Grip::Frame(-1, -1), p(70.0, 0.0)).unwrap();
    let i = &prj.pages[0].images[0];
    assert_eq!((i.a, i.b), (p(-10.0, 0.0), p(90.0, 50.0)));
}

#[test]
fn structure_boxes_and_device_boxes_are_resized() {
    let mut prj = Project::default();
    let sb_id = prj.new_id();
    prj.pages[0].structure_boxes.push(structure::StructureBox { id: sb_id, a: p(100.0, 100.0), b: p(200.0, 150.0), ..Default::default() });
    pull(&mut prj, sb_id, Grip::Frame(0, 1), p(0.0, 180.0)).unwrap();
    let b = &prj.pages[0].structure_boxes[0];
    assert_eq!((b.a, b.b), (p(100.0, 100.0), p(200.0, 180.0)));
    // The connection points on the moved edge of a device box go with it.
    let dbox = device_boxes::add_box(&mut prj, 0, p(40.0, 40.0), p(80.0, 64.0), "", "").unwrap();
    let pt = device_boxes::place_point(&mut prj, 0, p(80.0, 48.0), 1, "", "").unwrap();
    let g = grab(&prj, 0, dbox, Grip::Frame(1, 0)).unwrap();
    drag(&mut prj, 0, &g, p(97.0, 0.0)).unwrap();
    let s = prj.pages[0].symbols.iter().find(|s| s.id == dbox).unwrap();
    assert_eq!(device_boxes::rect(&prj, s), Some(R::from_points(p(40.0, 40.0), p(96.0, 64.0))));
    assert_eq!(prj.pages[0].symbols.iter().find(|s| s.id == pt).unwrap().pos, p(96.0, 48.0));
}

#[test]
fn rectangles_turn_about_their_middle() {
    let mut prj = Project::default();
    let id = shape(&mut prj, ShapeKind::Rect, p(0.0, 0.0), p(20.0, 10.0));
    assert_eq!(turn_handle(&prj.pages[0], id), Some((p(10.0, 5.0), p(10.0, 0.0))));
    let t = start_turn(&prj.pages[0], id).unwrap();
    turn(&mut prj.pages[0], &t, 90.0);
    let s = shape_of(&prj, id);
    assert_eq!((s.a, s.b, s.angle), (p(0.0, 0.0), p(20.0, 10.0), 90.0));
    // Counter-clockwise: the long sides stand upright now, the top looks left.
    let r = graphics::shape_bounds(s);
    assert!(close(r.min, p(5.0, -5.0)) && close(r.max, p(15.0, 15.0)), "{r:?}");
    assert!(close(turn_handle(&prj.pages[0], id).unwrap().1, p(5.0, 5.0)));
    // Every step starts from the shape at the start of the turn.
    turn(&mut prj.pages[0], &t, -30.0);
    assert_eq!(shape_of(&prj, id).angle, 330.0);
    // Circles do not turn.
    let c = shape(&mut prj, ShapeKind::Circle, p(50.0, 50.0), p(60.0, 50.0));
    assert!(start_turn(&prj.pages[0], c).is_none());
}

#[test]
fn turned_rectangles_are_resized_along_their_sides() {
    let mut prj = Project::default();
    let id = shape(&mut prj, ShapeKind::Rect, p(0.0, 0.0), p(20.0, 10.0));
    prj.pages[0].shapes[0].angle = 90.0;
    let g = grips(&prj, 0, id);
    let right = g.iter().find(|(g, _)| *g == Grip::Frame(1, 0)).unwrap().1;
    let left = g.iter().find(|(g, _)| *g == Grip::Frame(-1, 0)).unwrap().1;
    assert!(close(right, p(10.0, -5.0)) && close(left, p(10.0, 15.0)));
    // The right side (now on top) is pulled up by 6 mm: 6 mm longer, the left side stays.
    pull(&mut prj, id, Grip::Frame(1, 0), p(10.0, -11.0)).unwrap();
    let s = shape_of(&prj, id);
    assert!(close(s.b - s.a, p(26.0, 10.0)));
    let g = grips(&prj, 0, id);
    assert!(close(g.iter().find(|(g, _)| *g == Grip::Frame(-1, 0)).unwrap().1, left));
    assert!(close(g.iter().find(|(g, _)| *g == Grip::Frame(1, 0)).unwrap().1, p(10.0, -11.0)));
}

#[test]
fn lines_texts_and_images_turn() {
    let mut prj = Project::default();
    let line = shape(&mut prj, ShapeKind::Line, p(0.0, 0.0), p(10.0, 0.0));
    let t = start_turn(&prj.pages[0], line).unwrap();
    turn(&mut prj.pages[0], &t, 90.0);
    let s = shape_of(&prj, line);
    assert!(close(s.a, p(5.0, 5.0)) && close(s.b, p(5.0, -5.0)) && s.angle == 0.0);
    let text = prj.new_id();
    prj.pages[0].texts.push(TextItem { id: text, pos: p(30.0, 30.0), text: "A".into(), size: 3.0, angle: 30.0, ..Default::default() });
    let t = start_turn(&prj.pages[0], text).unwrap();
    assert_eq!(t.pivot, p(30.0, 30.0));
    turn(&mut prj.pages[0], &t, 340.0);
    assert_eq!(prj.pages[0].texts[0].angle, 10.0);
    // An image turned by a quarter: its corners, what it covers and where it is hit.
    let id = prj.new_id();
    prj.pages[0].images.push(ImageItem { id, a: p(0.0, 0.0), b: p(40.0, 20.0), px: [400, 200], ..Default::default() });
    let t = start_turn(&prj.pages[0], id).unwrap();
    turn(&mut prj.pages[0], &t, 90.0);
    let img = &prj.pages[0].images[0];
    assert!(close(graphics::image_corners(img)[0], p(10.0, 30.0)));
    let r = graphics::image_bounds(img);
    assert!(close(r.min, p(10.0, -10.0)) && close(r.max, p(30.0, 30.0)));
    assert!(graphics::image_hit(img, p(20.0, -8.0)) && !graphics::image_hit(img, p(2.0, 10.0)));
    // Its corner grips keep the aspect ratio along its turned sides.
    pull(&mut prj, id, Grip::Frame(1, 1), p(30.0, -30.0)).unwrap();
    let img = &prj.pages[0].images[0];
    assert!(close(img.b - img.a, p(60.0, 30.0)), "{:?}", img.b - img.a);
    assert!(close(graphics::image_corners(img)[0], p(10.0, 30.0)));
}

#[test]
fn a_quarter_turn_keeps_the_size_of_images() {
    let mut prj = Project::default();
    let id = prj.new_id();
    prj.pages[0].images.push(ImageItem { id, a: p(0.0, 0.0), b: p(40.0, 20.0), px: [400, 200], ..Default::default() });
    let symbols = prj.symbol_table();
    crate::ops::rotate(&symbols, &mut prj.pages[0], &std::collections::BTreeSet::from([id]));
    let img = &prj.pages[0].images[0];
    assert!(close(img.b - img.a, p(40.0, 20.0)));
    assert_eq!(img.angle, 270.0);
    // Clockwise on the page: it stands upright.
    let r = graphics::image_bounds(img);
    assert!(close(p(r.width(), r.height()), p(20.0, 40.0)));
}

#[test]
fn turned_frames_change_their_size_in_grid_steps() {
    let mut prj = Project::default();
    // A size off the grid comes onto it.
    let id = shape(&mut prj, ShapeKind::Rect, p(0.0, 0.0), p(20.3, 12.9));
    prj.pages[0].shapes[0].angle = 30.0;
    let g = grab(&prj, 0, id, Grip::Frame(1, 1)).unwrap();
    drag_on_grid(&mut prj, 0, &g, p(31.3, 9.7), 4.0).unwrap();
    let s = shape_of(&prj, id);
    let size = s.b - s.a;
    assert!(close(size, size.snap(4.0)) && size.x > 20.0, "{size:?}");
    // Without an angle the grip goes to the grid point.
    let r = shape(&mut prj, ShapeKind::Rect, p(0.0, 0.0), p(20.0, 12.0));
    let g = grab(&prj, 0, r, Grip::Frame(1, 1)).unwrap();
    drag_on_grid(&mut prj, 0, &g, p(30.9, 17.1), 4.0).unwrap();
    assert_eq!(shape_of(&prj, r).b, p(32.0, 16.0));
}
