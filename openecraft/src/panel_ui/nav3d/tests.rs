use super::*;
use super::super::gpu3d::Camera;
use egui::{vec2, Key, Pos2};
use openecraft_core::parts::vec3::dot;

const F: f32 = 1000.0;

fn cam() -> Orbit {
    Orbit { yaw: 0.55, pitch: 0.3, dist: 4000.0, target: [300.0, 900.0, 200.0] }
}

fn camera(o: &Orbit) -> Camera {
    let (eye, right, up, fwd) = o.basis();
    Camera { eye, right, up, fwd, f: F }
}

fn close(a: V3, b: V3, tol: f32) -> bool {
    length(sub(a, b)) < tol
}

#[test]
fn a_pan_keeps_the_point_grabbed_under_the_pointer() {
    let mut o = cam();
    let at = vec2(120.0, -80.0);
    let grabbed = camera(&o).unproject(at, o.dist);
    pan(&mut o, vec2(35.0, 20.0), F);
    let (q, _) = camera(&o).project(Pos2::ZERO, grabbed).unwrap();
    assert!((q.to_vec2() - (at + vec2(35.0, 20.0))).length() < 0.01, "{q:?}");
}

#[test]
fn zooming_keeps_the_point_under_the_pointer() {
    let mut o = cam();
    let at = vec2(-200.0, 150.0);
    let under = camera(&o).unproject(at, o.dist);
    zoom_at(&mut o, 1.5, at, F);
    assert!((o.dist - 4000.0 / 1.5).abs() < 0.1);
    let (q, _) = camera(&o).project(Pos2::ZERO, under).unwrap();
    assert!((q.to_vec2() - at).length() < 0.01, "{q:?}");
    zoom_at(&mut o, 1.0 / 1.5, at, F);
    assert!(close(o.target, cam().target, 0.1), "zooming back out returns: {:?}", o.target);
}

#[test]
fn zooming_in_past_the_nearest_distance_walks_on() {
    let mut o = Orbit { dist: MIN_DIST, ..cam() };
    let (eye, _, _, fwd) = o.basis();
    zoom_at(&mut o, 1.2, Vec2::ZERO, F);
    assert_eq!(o.dist, MIN_DIST);
    let (moved, ..) = o.basis();
    let step = sub(moved, eye);
    assert!((length(step) - WALK * 1.2f32.ln()).abs() < 0.5, "{step:?}");
    assert!(close(norm(step), fwd, 1e-3), "straight ahead at the centre of the view");
    // Off the centre it walks towards the pointer.
    let before = o;
    zoom_at(&mut o, 1.2, vec2(300.0, 0.0), F);
    let (_, right, ..) = before.basis();
    assert!(dot(sub(o.target, before.target), right) > 0.0);
}

#[test]
fn walking_stays_level_and_follows_the_view() {
    let mut o = Orbit { pitch: 1.2, ..cam() };
    let (_, right, _, fwd) = o.basis();
    walk(&mut o, 1000.0, 0.0, 0.0);
    let d = sub(o.target, cam().target);
    assert!(d[1].abs() < 1e-3, "level: {d:?}");
    assert!((length(d) - 1000.0).abs() < 0.1);
    let level = norm([fwd[0], 0.0, fwd[2]]);
    assert!(close(norm(d), level, 1e-3), "{d:?} {level:?}");
    let start = o.target;
    walk(&mut o, 0.0, 500.0, 200.0);
    let d = sub(o.target, start);
    assert!(close(d, add(scale(right, 500.0), [0.0, 200.0, 0.0]), 0.1), "{d:?}");
}

#[test]
fn keys_move_faster_with_shift_and_zoom() {
    let (mut slow, mut fast) = (cam(), cam());
    keys(&mut slow, [1.0, 0.0, 0.0, 0.0], false, 0.1);
    keys(&mut fast, [1.0, 0.0, 0.0, 0.0], true, 0.1);
    let (a, b) = (length(sub(slow.target, cam().target)), length(sub(fast.target, cam().target)));
    assert!(a > 100.0 && b > a * 2.0, "{a} {b}");
    let mut z = cam();
    keys(&mut z, [0.0, 0.0, 0.0, 1.0], false, 0.1);
    assert!(z.dist < cam().dist && z.target == cam().target);
}

#[test]
fn held_keys_give_the_directions() {
    let sc = Shortcuts::default();
    let shift = Some(Mod::Shift);
    let mut i = InputState::default();
    i.keys_down.extend([Key::W, Key::A, Key::E]);
    assert_eq!(held(&i, &sc, shift), ([1.0, -1.0, 1.0, 0.0], false));
    i.keys_down.insert(Key::ArrowDown);
    i.modifiers.shift = true;
    assert_eq!(held(&i, &sc, shift), ([0.0, -1.0, 1.0, 0.0], true), "forward and back cancel");
    i.modifiers.ctrl = true;
    i.modifiers.command = true;
    assert_eq!(held(&i, &sc, shift).0, [0.0; 4], "Ctrl+S, Ctrl+A are shortcuts");
    // Alt chosen for moving faster: held with Alt, the keys still move.
    i.modifiers = egui::Modifiers::ALT;
    assert_eq!(held(&i, &sc, Some(Mod::Alt)), ([0.0, -1.0, 1.0, 0.0], true));
    assert_eq!(held(&i, &sc, shift).0, [0.0; 4]);
}

#[test]
fn changed_keys_move_the_3d_view() {
    let mut sc = Shortcuts::default();
    sc.set(Action::FlyForward.id(), vec![crate::shortcuts::Shortcut::parse("I").unwrap()]);
    let mut i = InputState::default();
    i.keys_down.insert(Key::W);
    assert_eq!(held(&i, &sc, None).0, [0.0; 4]);
    i.keys_down.insert(Key::I);
    assert_eq!(held(&i, &sc, None).0, [1.0, 0.0, 0.0, 0.0]);
}

#[test]
fn the_camera_glides_to_a_view_the_short_way_round() {
    let mut o = Orbit { yaw: 3.0, ..cam() };
    let goal = Orbit { yaw: -3.0, pitch: 0.0, dist: 1000.0, target: [0.0; 3] };
    let mut steps = 0;
    while !glide(&mut o, &goal, 1.0 / 60.0) {
        assert!(o.yaw >= 3.0 - 1e-4, "turns through π, not back through 0: {}", o.yaw);
        steps += 1;
        assert!(steps < 300, "arrives");
    }
    assert_eq!(o, goal);
}

#[test]
fn looking_at_a_point_turns_to_it_and_closes_in() {
    let o = cam();
    let at = [1000.0, 200.0, -500.0];
    let (eye, ..) = o.basis();
    let v = look_at(&o, at);
    assert_eq!(v.target, at);
    assert!((v.dist - length(sub(eye, at)) * 0.6).abs() < 0.1);
    let (new_eye, _, _, fwd) = v.basis();
    assert!(close(fwd, norm(sub(at, eye)), 1e-3), "seen from the same direction");
    assert!(close(norm(sub(new_eye, eye)), norm(sub(at, eye)), 1e-3), "on the way to it");
}

#[test]
fn the_standard_views_look_from_their_sides() {
    let at = |s: Side| {
        let (yaw, pitch) = s.angles();
        let (eye, ..) = Orbit { yaw, pitch, dist: 1.0, target: [0.0; 3] }.basis();
        eye
    };
    assert!(at(Side::Front)[2] > 0.99);
    assert!(at(Side::Rear)[2] < -0.99);
    assert!(at(Side::Left)[0] < -0.99);
    assert!(at(Side::Right)[0] > 0.99);
    assert!(at(Side::Top)[1] > 0.99);
    assert!(Side::ALL.iter().all(|s| !s.title().is_empty()));
}
