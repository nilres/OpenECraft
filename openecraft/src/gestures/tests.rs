use super::*;
use egui::{pos2, vec2, Context, Pos2, RawInput, Rect, Sense};

#[test]
fn defaults_have_no_conflicts_and_ids_are_unique() {
    assert_eq!(Gestures::default().conflicts(), Vec::new());
    let mut ids: Vec<&str> = MouseAction::ALL.iter().map(|a| a.id()).chain(Role::ALL.iter().map(|r| r.id())).collect();
    let n = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), n);
    for id in ids {
        assert!(!title(id).is_empty(), "{id}");
    }
    for a in MouseAction::ALL {
        assert!(a.defaults().iter().all(|b| a.accepts(b)), "{a:?}");
    }
}

#[test]
fn bindings_are_stored_and_read_back() {
    let mut g = Gestures::default();
    g.set_mouse(MouseAction::PanView, vec![Binding::new(Some(Mod::Alt), Button::Primary)]);
    g.set_modifier(Role::OpenLink, None);
    g.set_modifier(Role::AddToSelection, Some(Mod::Ctrl));
    let stored = g.to_storage();
    assert_eq!(stored["pan_view"], vec!["Alt+left".to_string()]);
    assert_eq!(stored["open_link"], Vec::<String>::new());
    assert_eq!(Gestures::from_storage(&stored), g);
    // Setting the defaults again removes the change.
    g.set_mouse(MouseAction::PanView, MouseAction::PanView.defaults());
    g.set_modifier(Role::OpenLink, Role::OpenLink.default());
    assert!(g.mouse_is_default(MouseAction::PanView) && g.modifier_is_default(Role::OpenLink));
    g.reset_modifiers();
    assert_eq!(g, Gestures::default());
}

#[test]
fn unknown_stored_entries_are_ignored() {
    let stored: BTreeMap<String, Vec<String>> = [
        ("teleport".to_string(), vec!["left".to_string()]),
        ("pan_view".to_string(), vec!["Hyper+left".to_string(), "Shift+middle".to_string(), "thumb".to_string()]),
        ("rotate_back".to_string(), vec!["Bogus".to_string()]),
    ]
    .into();
    let g = Gestures::from_storage(&stored);
    assert_eq!(g.mouse(MouseAction::PanView), vec![Binding::new(Some(Mod::Shift), Button::Middle)]);
    assert!(g.modifier_is_default(Role::RotateBack));
}

#[test]
fn the_plain_left_button_neither_pans_nor_cancels_the_flat_views() {
    let mut g = Gestures::default();
    let left = Binding::new(None, Button::Primary);
    g.add_mouse(MouseAction::PanView, left);
    g.add_mouse(MouseAction::CancelTool, left);
    g.add_mouse(MouseAction::RotateWhileMoving, Binding::new(Some(Mod::Shift), Button::Primary));
    assert_eq!(g, Gestures::default());
    // With a modifier it may pan; twice the same is kept once.
    g.add_mouse(MouseAction::PanView, Binding::new(Some(Mod::Alt), Button::Primary));
    g.add_mouse(MouseAction::PanView, Binding::new(Some(Mod::Alt), Button::Primary));
    assert_eq!(g.mouse(MouseAction::PanView).len(), 3);
}

#[test]
fn conflicts_are_found_where_both_act_at_once() {
    let mut g = Gestures::default();
    // Orbit and pan of the 3D view with the same drag.
    g.set_mouse(MouseAction::Orbit3d, vec![Binding::new(None, Button::Middle)]);
    // A drawing tool and the select tool both double-click: no conflict.
    g.set_mouse(MouseAction::OpenProperties, vec![Binding::new(None, Button::Primary)]);
    // Two click modifiers alike, the wheel modifier alike to one of them: only the first.
    g.set_modifier(Role::OpenLink, Some(Mod::Shift));
    g.set_modifier(Role::WheelZoom, Some(Mod::Alt));
    let c = g.conflicts();
    assert_eq!(c.len(), 2, "{c:?}");
    assert_eq!(c[0], (Clash::Mouse(Kind::Drag, Binding::new(None, Button::Middle)), vec!["orbit_3d", "pan_3d"]));
    assert_eq!(c[1], (Clash::Modifier(Mod::Shift), vec!["add_to_selection", "open_link"]));
    // The pan of the flat views does not clash with the 3D view.
    g.reset_mouse();
    g.set_mouse(MouseAction::PanView, vec![Binding::new(Some(Mod::Shift), Button::Primary)]);
    assert_eq!(g.conflicts().len(), 1);
}

#[test]
fn the_keep_connections_modifier_clashes_only_at_the_start_of_a_move() {
    let mut g = Gestures::default();
    assert!(g.held(Role::ReverseKeepConnections, Modifiers::ALT));
    // Alt for an assembly item is read in the panel layout, not at a move on the page.
    assert_eq!(g.conflicts(), Vec::new());
    g.set_modifier(Role::ReverseKeepConnections, Some(Mod::Shift));
    assert_eq!(g.conflicts(), vec![(Clash::Modifier(Mod::Shift), vec!["add_to_selection", "reverse_keep_connections"])]);
    g.set_modifier(Role::ReverseKeepConnections, Some(Mod::Ctrl));
    assert_eq!(g.conflicts(), Vec::new(), "Ctrl opens links on a click, not at a move");
}

#[test]
fn modifier_roles_can_be_switched_off() {
    let mut g = Gestures::default();
    assert!(g.held(Role::AddToSelection, Modifiers::SHIFT));
    assert!(!g.held(Role::AddToSelection, Modifiers::CTRL));
    assert!(g.held(Role::OpenLink, Modifiers::COMMAND), "Cmd on macOS");
    g.set_modifier(Role::OpenLink, None);
    assert!(!g.held(Role::OpenLink, Modifiers::CTRL));
    g.set_modifier(Role::WheelSideways, Some(Mod::Alt));
    g.set_modifier(Role::WheelZoom, None);
    let nav = g.nav(nav::Settings { invert: true, ..Default::default() });
    assert_eq!((nav.sideways, nav.zoom, nav.invert), (Some(Mod::Alt), None, true));
}

fn button(button: PointerButton, pressed: bool, at: Pos2, modifiers: Modifiers) -> Event {
    Event::PointerButton { pos: at, button, pressed, modifiers }
}

/// Runs frames over a view filling the screen; gives what `f` saw in each.
fn frames<T>(frames: Vec<(Vec<Event>, Modifiers)>, f: impl Fn(&Response, &[Event], Modifiers) -> T) -> Vec<T> {
    let ctx = Context::default();
    let mut out = Vec::new();
    for (n, (events, modifiers)) in frames.into_iter().enumerate() {
        let events = std::iter::once(Event::ModifiersChanged(modifiers)).chain(events).collect();
        let input = RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))), time: Some(n as f64 * 0.1), events, ..Default::default() };
        let res = ctx.run_ui(input, |ui| {
            let (_, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
            let events = ui.input(|i| i.events.clone());
            let m = ui.input(|i| i.modifiers);
            out.push(f(&resp, &events, m));
        });
        res.drop_without_applying_deltas();
    }
    out
}

/// A drag with `b` and the modifiers `m` from (100, 100) to (200, 100).
fn drag(b: PointerButton, m: Modifiers) -> Vec<(Vec<Event>, Modifiers)> {
    let (a, z) = (pos2(100.0, 100.0), pos2(200.0, 100.0));
    vec![
        (vec![Event::PointerMoved(a)], m),
        (vec![button(b, true, a, m)], m),
        (vec![Event::PointerMoved(pos2(150.0, 100.0))], m),
        (vec![Event::PointerMoved(z)], m),
        (vec![button(b, false, z, m)], m),
    ]
}

#[test]
fn the_held_modifier_picks_the_drag_of_the_3d_view() {
    let g = Gestures::default();
    let which = |b, m| frames(drag(b, m), |resp, _, m| g.drag_of(&[MouseAction::Orbit3d, MouseAction::Pan3d], resp, m)).into_iter().flatten().collect::<Vec<_>>();
    let orbit = which(PointerButton::Primary, Modifiers::NONE);
    assert!(!orbit.is_empty() && orbit.iter().all(|a| *a == MouseAction::Orbit3d), "{orbit:?}");
    let pan = which(PointerButton::Primary, Modifiers::SHIFT);
    assert!(!pan.is_empty() && pan.iter().all(|a| *a == MouseAction::Pan3d), "{pan:?}");
    let middle = which(PointerButton::Middle, Modifiers::NONE);
    assert!(!middle.is_empty() && middle.iter().all(|a| *a == MouseAction::Pan3d), "{middle:?}");
}

#[test]
fn a_changed_binding_pans_and_the_old_one_does_not() {
    let mut g = Gestures::default();
    g.set_mouse(MouseAction::PanView, vec![Binding::new(Some(Mod::Alt), Button::Primary)]);
    let pans = |b, m| frames(drag(b, m), |resp, _, m| g.dragged(MouseAction::PanView, resp, m)).into_iter().any(|d| d);
    assert!(pans(PointerButton::Primary, Modifiers::ALT));
    assert!(!pans(PointerButton::Primary, Modifiers::NONE));
    assert!(!pans(PointerButton::Middle, Modifiers::NONE));
}

#[test]
fn clicks_and_presses_follow_their_bindings() {
    let at = pos2(100.0, 100.0);
    let click = |b, m| vec![(vec![Event::PointerMoved(at)], m), (vec![button(b, true, at, m)], m), (vec![button(b, false, at, m)], m)];
    let mut g = Gestures::default();
    let cancels = |g: &Gestures, b, m| frames(click(b, m), |resp, _, m| g.clicked(MouseAction::CancelTool, resp, m)).into_iter().any(|c| c);
    assert!(cancels(&g, PointerButton::Secondary, Modifiers::NONE));
    g.set_mouse(MouseAction::CancelTool, vec![Binding::new(None, Button::Middle)]);
    assert!(!cancels(&g, PointerButton::Secondary, Modifiers::NONE));
    assert!(cancels(&g, PointerButton::Middle, Modifiers::SHIFT), "no modifier: any modifier");
    let presses = frames(click(PointerButton::Secondary, Modifiers::SHIFT), |_, events, _| Gestures::default().presses(MouseAction::RotateWhileMoving, events));
    assert_eq!(presses.concat(), vec![Modifiers::SHIFT]);
}

#[test]
fn a_recording_takes_the_gesture_of_the_kind_asked_for() {
    let got = frames(drag(PointerButton::Middle, Modifiers::CTRL), |resp, events, m| recorded(Kind::Drag, resp, events, m));
    assert_eq!(got.into_iter().flatten().collect::<Vec<_>>(), vec![Binding::new(Some(Mod::Ctrl), Button::Middle)]);
    // A drag is no click.
    let clicks = frames(drag(PointerButton::Middle, Modifiers::NONE), |resp, events, m| recorded(Kind::Click, resp, events, m));
    assert!(clicks.iter().all(Option::is_none));
    // A press while moving keeps no modifier.
    let press = frames(drag(PointerButton::Secondary, Modifiers::SHIFT), |resp, events, m| recorded(Kind::Press, resp, events, m));
    assert_eq!(press.into_iter().flatten().collect::<Vec<_>>(), vec![Binding::new(None, Button::Secondary)]);
}

#[test]
fn bindings_print_their_modifier_and_button() {
    assert_eq!(Binding::new(Some(Mod::Shift), Button::Middle).text(), "Shift+Middle button");
    assert_eq!(Clash::Mouse(Kind::Drag, Binding::new(None, Button::Primary)).text(), "Left button (Drag)");
    for b in [Binding::new(None, Button::Secondary), Binding::new(Some(Mod::Alt), Button::Primary)] {
        assert_eq!(Binding::parse(&b.id()), Some(b));
    }
}

#[test]
fn the_gestures_reach_the_views_through_the_context() {
    let ctx = Context::default();
    assert_eq!(*get(&ctx), Gestures::default());
    let mut g = Gestures::default();
    g.set_modifier(Role::Faster3d, Some(Mod::Alt));
    set(&ctx, &g);
    assert_eq!(*get(&ctx), g);
}
