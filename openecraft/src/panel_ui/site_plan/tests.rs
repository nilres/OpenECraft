use super::*;

#[test]
fn points_inside_an_l_shaped_room() {
    let l = [pos2(0.0, 0.0), pos2(40.0, 0.0), pos2(40.0, 20.0), pos2(20.0, 20.0), pos2(20.0, 40.0), pos2(0.0, 40.0)];
    assert!(inside(&l, pos2(10.0, 30.0)));
    assert!(inside(&l, pos2(30.0, 10.0)));
    assert!(!inside(&l, pos2(30.0, 30.0)), "in the notch");
    assert!(!inside(&l, pos2(-5.0, 10.0)));
}
