use super::*;
use super::super::scene3d::{SceneFace, SceneWire};
use openecraft_core::parts::vec3::Orbit;

fn camera() -> Camera {
    let orbit = Orbit { yaw: 0.55, pitch: 0.3, dist: 4000.0, target: [300.0, 900.0, 200.0] };
    let (eye, right, up, fwd) = orbit.basis();
    Camera { eye, right, up, fwd, f: 800.0 * 1.3 }
}

#[test]
fn the_matrix_projects_as_the_processor_does() {
    let cam = camera();
    let rect = Rect::from_min_size(Pos2::new(40.0, 60.0), egui::vec2(1200.0, 800.0));
    let m = cam.matrix(rect.size(), 50.0, 20000.0);
    for q in [[0.0, 0.0, 0.0], [600.0, 1800.0, 400.0], [-900.0, 300.0, 2500.0], [300.0, 900.0, 200.0]] {
        let clip: [f32; 4] = std::array::from_fn(|r| (0..4).map(|c| m[c * 4 + r] * [q[0], q[1], q[2], 1.0][c]).sum());
        let (screen, z) = cam.project(rect.center(), q).unwrap();
        let ndc = [clip[0] / clip[3], clip[1] / clip[3], clip[2] / clip[3]];
        let on_screen = rect.center() + egui::vec2(ndc[0] * rect.width() / 2.0, -ndc[1] * rect.height() / 2.0);
        assert!((on_screen - screen).length() < 0.05, "{q:?} {on_screen:?} {screen:?}");
        assert!((clip[3] - z).abs() < 0.01, "depth {} {z}", clip[3]);
        assert!(ndc[2] > -1.0 && ndc[2] < 1.0, "{q:?} {}", ndc[2]);
    }
}

#[test]
fn unprojecting_reverses_the_projection() {
    let cam = camera();
    for q in [[0.0, 0.0, 0.0], [600.0, 1800.0, 400.0], [-900.0, 300.0, 2500.0]] {
        let (screen, z) = cam.project(Pos2::ZERO, q).unwrap();
        let back = cam.unproject(screen.to_vec2(), z);
        assert!((0..3).all(|k| (back[k] - q[k]).abs() < 0.05), "{q:?} {back:?}");
    }
}

#[test]
fn the_depths_reach_from_the_scene_to_its_farthest_corner() {
    let bounds = ([0.0, 0.0, 0.0], [1000.0, 2000.0, 500.0]);
    let (near, far) = depth_range(bounds, [500.0, 1000.0, 3500.0]);
    assert!((near - 2700.0).abs() < 1.0, "{near}");
    let farthest = (500.0f32 * 500.0 + 1000.0 * 1000.0 + 3500.0 * 3500.0).sqrt();
    assert!(far > farthest && far < farthest * 1.05, "{far} {farthest}");
    // Inside a room: from the nearest depth drawn.
    assert_eq!(depth_range(bounds, [500.0, 1000.0, 250.0]).0, NEAR);
}

fn side(normal: V3, hideable: bool) -> SceneSide {
    SceneSide { normal, at: [0.0; 3], inside: 0, hideable, decal: false, glass: false }
}

#[test]
fn walls_seen_from_the_outside_are_translucent_or_left_out() {
    let (outside, inside) = ([0.0, 0.0, 1000.0], [0.0, 0.0, -1000.0]);
    let wall = side([0.0, 0.0, 1.0], true);
    assert_eq!(look(&wall, outside, false, 0.6), Look::Translucent);
    assert_eq!(look(&wall, outside, false, 1.0), Look::Opaque);
    assert_eq!(look(&wall, outside, false, 0.0), Look::Hidden);
    assert_eq!(look(&wall, outside, true, 0.6), Look::Hidden);
    assert_eq!(look(&wall, inside, true, 0.6), Look::Opaque);
    // A door is not one of the walls in front.
    assert_eq!(look(&side([0.0, 0.0, 1.0], false), outside, true, 0.6), Look::Translucent);
}

fn face(n: u8, side: Option<usize>, obj: Option<usize>, decal: bool) -> SceneFace {
    SceneFace { pts: [[0.0; 3], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]], n, normal: [0.0, 0.0, 1.0], color: Color32::RED, layer: 1, side, obj, item: None, outline: n == 4, decal }
}

fn line(stripe: Option<Color32>) -> SceneLine {
    SceneLine { a: [0.0; 3], b: [100.0, 0.0, 0.0], color: Color32::BLUE, stripe, least: 1.5, diameter: 2.4, tube: false }
}

#[test]
fn the_scene_goes_to_the_graphics_card_in_parts() {
    let scene = Scene3d {
        faces: vec![face(4, None, None, false), face(3, Some(1), None, false), face(4, None, Some(0), false), face(3, None, None, true), face(3, None, Some(1), false)],
        sides: vec![side([0.0, 0.0, 1.0], true), side([0.0, 0.0, 1.0], true)],
        objects: vec![([0.0; 3], [1.0; 3]); 2],
        ducts: vec![true, false],
        wires: vec![SceneWire { line: line(None), inside: Some(0) }, SceneWire { line: line(Some(Color32::YELLOW)), inside: None }],
        lines: vec![line(None)],
        ..Default::default()
    };
    let g = geometry(&scene);
    let s = &g.spans;
    // Quads are two triangles; the device triangle stays with the base.
    assert_eq!((s.base.count, s.decal.count, s.ducts.count), (9, 3, 6));
    assert_eq!(s.sides, vec![Span { first: 18, count: 0 }, Span { first: 18, count: 3 }]);
    assert_eq!(g.faces.len(), 21 * FACE_STRIDE as usize);
    assert_eq!((s.wires.count, s.stripes.count, s.top.count), (12, 6, 6));
    assert_eq!((s.stripes.first, s.top.first), (12, 18));
    assert_eq!(g.lines.len(), 24 * LINE_STRIDE as usize);
    // Each vertex of a face: position, normal and colour.
    let at = |i: usize| f32::from_ne_bytes(g.faces[i * 4..i * 4 + 4].try_into().unwrap());
    assert_eq!([at(3), at(4), at(5)], [0.0, 0.0, 1.0]);
    assert_eq!(&g.faces[24..28], &Color32::RED.to_array());
}

#[test]
fn tubes_are_drawn_as_faces_instead_of_their_lines() {
    let tube = SceneLine { tube: true, ..line(Some(Color32::YELLOW)) };
    let scene = Scene3d { tubes: vec![face(3, None, None, false)], wires: vec![SceneWire { line: tube.clone(), inside: None }, SceneWire { line: line(None), inside: None }], lines: vec![tube], ..Default::default() };
    let g = geometry(&scene);
    let s = &g.spans;
    assert_eq!(s.base.count, 3);
    assert_eq!((s.wires.count, s.stripes.count, s.top.count), (6, 0, 6));
    // A selected tube stays on top as a thin line (no diameter).
    let diameter = |vertex: i32| {
        let i = vertex as usize * LINE_STRIDE as usize + 40;
        f32::from_ne_bytes(g.lines[i..i + 4].try_into().unwrap())
    };
    assert_eq!((diameter(s.wires.first), diameter(s.top.first)), (2.4, 0.0));
}

