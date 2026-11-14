use super::*;
use egui::{pos2, Event, PointerButton, RawInput};
use openecraft_parts::vec3::view_from;
use std::f32::consts::FRAC_PI_2;

const C: Pos2 = pos2(100.0, 100.0);

#[test]
fn from_the_front_only_the_front_face_is_seen() {
    let proj = Projection::new(0.0, 0.0, C, 30.0);
    let regions = proj.regions();
    assert_eq!(regions.len(), 9);
    assert_eq!(region_at(&regions, C), Some([0, 0, 1]));
    assert_eq!(region_at(&regions, C + vec2(0.0, -27.0)), Some([0, 1, 1]), "the top edge");
    assert_eq!(region_at(&regions, C + vec2(27.0, 27.0)), Some([1, -1, 1]), "the bottom right corner");
    assert_eq!(region_at(&regions, C + vec2(40.0, 0.0)), None, "beside the cube");
}

#[test]
fn from_a_corner_three_faces_meet_in_the_middle() {
    let (yaw, pitch) = view_from([1, 1, 1], 0.0);
    let proj = Projection::new(yaw, pitch, C, 30.0);
    let regions = proj.regions();
    assert_eq!(regions.len(), 27);
    assert_eq!(region_at(&regions, C + vec2(0.5, 0.5)), Some([1, 1, 1]));
    // Every face, edge and corner of the visible half is there.
    let mut dirs: Vec<[i8; 3]> = regions.iter().map(|r| r.dir).collect();
    dirs.sort();
    dirs.dedup();
    assert_eq!(dirs.len(), 19);
    // The front face lies lower left.
    let front = regions.iter().find(|r| r.dir == [0, 0, 1]).unwrap();
    let mid = front.pts.iter().fold(Vec2::ZERO, |s, p| s + p.to_vec2()) / 4.0;
    assert!(mid.x < C.x && mid.y > C.y, "{mid:?}");
}

#[test]
fn every_region_looks_from_its_direction() {
    for (yaw, pitch) in [(0.0, 0.0), (0.55, 0.3), (-2.0, -0.8), (FRAC_PI_2, 1.2)] {
        let proj = Projection::new(yaw, pitch, C, 30.0);
        for r in proj.regions() {
            let mid = r.pts.iter().fold(Vec2::ZERO, |s, p| s + p.to_vec2()) / 4.0;
            assert_eq!(region_at(&proj.regions(), mid.to_pos2()), Some(r.dir));
        }
    }
}

/// Runs frames of a full cube at `C` seen from the front with the pointer `events`, one
/// frame each; gives what the cube asked for.
fn run(events: Vec<Vec<Event>>) -> Vec<Option<Action>> {
    let ctx = egui::Context::default();
    let mut out = Vec::new();
    for ev in events.into_iter().chain([vec![], vec![]]) {
        let input = RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))), events: ev, ..Default::default() };
        ctx.run_ui(input, |ui| out.push(ViewCube::full().show(ui, ui.scope_id().with("cube"), C, 0.0, 0.0))).textures_delta.clear();
    }
    out
}

fn press(at: Pos2, pressed: bool) -> Event {
    Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: Default::default() }
}

#[test]
fn clicking_a_corner_looks_from_it() {
    let at = C + vec2(-24.0, -24.0);
    let out = run(vec![vec![Event::PointerMoved(at)], vec![press(at, true)], vec![press(at, false)]]);
    assert!(out.contains(&Some(Action::Look([-1, 1, 1]))), "{out:?}");
}

#[test]
fn dragging_the_cube_orbits() {
    let at = C;
    let out = run(vec![vec![Event::PointerMoved(at)], vec![press(at, true)], vec![Event::PointerMoved(at + vec2(20.0, 0.0))], vec![Event::PointerMoved(at + vec2(40.0, 0.0))], vec![press(at + vec2(40.0, 0.0), false)]]);
    assert!(out.iter().any(|a| matches!(a, Some(Action::Orbit(d)) if d.x > 0.0)), "{out:?}");
    assert!(!out.iter().any(|a| matches!(a, Some(Action::Look(..)))), "a drag is no click: {out:?}");
}

#[test]
fn the_buttons_turn_go_home_and_fit() {
    let cube = ViewCube::full();
    let o = cube.button_offset();
    let click = |at: Pos2| run(vec![vec![Event::PointerMoved(at)], vec![press(at, true)], vec![press(at, false)]]);
    assert!(click(C + vec2(-o, -o)).contains(&Some(Action::Turn(1))), "turn left");
    assert!(click(C + vec2(o, -o)).contains(&Some(Action::Turn(-1))), "turn right");
    assert!(click(C + vec2(-o, o)).contains(&Some(Action::Home)));
    assert!(click(C + vec2(o, o)).contains(&Some(Action::Fit)));
    assert!(cube.size().x / 2.0 >= o + 9.0, "the buttons lie in the space taken");
}

#[test]
fn labels_are_laid_on_their_faces() {
    let ctx = egui::Context::default();
    ctx.run_ui(RawInput::default(), |ui| {
        let Shape::Mesh(mesh) = face_label(ui, "Front", C, vec2(30.0, 0.0), vec2(0.0, 30.0), Color32::WHITE) else { panic!("a mesh") };
        assert!(!mesh.vertices.is_empty());
        // Within the face (half a side of 30 points), around its middle.
        assert!(mesh.vertices.iter().all(|v| (v.pos - C).abs().max_elem() < 30.0));
        assert!(mesh.vertices.iter().all(|v| (0.0..=1.0).contains(&v.uv.x) && (0.0..=1.0).contains(&v.uv.y)));
        // A face seen at a slant: the text slants with it.
        let Shape::Mesh(slanted) = face_label(ui, "Front", C, vec2(20.0, 10.0), vec2(0.0, 30.0), Color32::WHITE) else { panic!("a mesh") };
        let right = slanted.vertices.iter().max_by(|a, b| a.pos.x.total_cmp(&b.pos.x)).unwrap();
        assert!(right.pos.y > C.y - 5.0, "{:?}", right.pos);
    })
    .textures_delta
    .clear();
}

#[test]
fn the_axes_are_drawn() {
    let ctx = egui::Context::default();
    let mut out = ctx.run_ui(RawInput::default(), |ui| axes(ui.painter(), C, 0.55, 0.3, 20.0));
    out.textures_delta.clear();
    assert!(out.shapes.len() >= 7, "three lines, three letters and a dot");
}
