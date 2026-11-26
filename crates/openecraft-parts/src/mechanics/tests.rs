use super::*;

#[test]
fn part_numbers_of_components() {
    let ts = system("baying").unwrap();
    assert_eq!(body_number(ts, 800.0, 2000.0, 600.0), "GEN.TS-800X2000X600");
    assert_eq!(side_number(ts, 2000.0, 600.0), "GEN.TS-SP-2000X600");
    assert_eq!(rear_number(ts, 800.0, 2000.0), "GEN.TS-RW-800X2000");
    assert_eq!(plinth_number(800.0, 600.0, 100.0), "GEN.PL-800X600-100");
    assert_eq!(gland_plate_number(600.0, 210.0), "GEN.GP-600X210");
    assert!(system("nonsense").is_none());
}

#[test]
fn racks_have_their_units() {
    let rk = system(RACK_SYSTEM).unwrap();
    let units: Vec<u32> = rk.sizes.iter().map(|s| rack_units(s.1)).collect();
    assert_eq!(units, [12, 24, 42, 42, 42, 47]);
    assert!(rk.sizes.iter().all(|s| s.0 - 2.0 * rk.frame >= RACK_WIDTH), "the panels fit between the posts");
    assert_eq!(rack_rail_number(42), "GEN.RK-RAIL-42U");
    assert_eq!(rack_units(80.0), 0);
}

#[test]
fn fasteners() {
    assert_eq!(thread(" m5 "), Some("M5"));
    assert_eq!(thread("M7"), None);
    assert_eq!(screw_number("M5"), "GEN.SCREW-M5X12");
    assert_eq!(rivet_nut_number("M4"), "GEN.RIVNUT-M4");
}

#[test]
fn systems_have_sizes() {
    for s in SYSTEMS {
        assert!(!s.sizes.is_empty(), "{}", s.id);
        assert!(s.wall > 0.0 && s.flange >= s.wall, "{}", s.id);
        assert_eq!(s.bayable, s.frame > 0.0, "{}", s.id);
    }
}

#[test]
fn top_hat_rails_have_flanges_walls_and_a_slotted_web() {
    let inside = |b: &[([f32; 3], [f32; 3])], len: f32, w: f32, d: f32| b.iter().all(|(lo, hi)| (0..3).all(|k| lo[k] >= 0.0 && lo[k] < hi[k]) && hi[0] <= len && hi[1] <= w && hi[2] <= d);
    let rail = rail_boxes(RailProfile::TopHat { perforated: true }, 200.0, 35.0, 7.5);
    assert!(inside(&rail, 200.0, 35.0, 7.5));
    // The flanges in front over the whole width, 4 mm each; the web 27 mm wide at the back.
    let front: Vec<_> = rail.iter().filter(|(_, hi)| hi[2] == 7.5).collect();
    assert_eq!(front.iter().map(|(lo, hi)| hi[1] - lo[1]).sum::<f32>(), 2.0 * (4.0 + 1.0));
    let back: Vec<_> = rail.iter().filter(|(lo, _)| lo[2] == 0.0).collect();
    let (w0, w1) = back.iter().fold((f32::MAX, f32::MIN), |(a, b), (lo, hi)| (a.min(lo[1]), b.max(hi[1])));
    assert_eq!((w0, w1), (4.0, 31.0));
    // Slots 25 mm apart in the middle of the web: nothing of the web at their middles.
    let slots = rail_slots(200.0);
    assert_eq!(slots, [12.5, 37.5, 62.5, 87.5, 112.5, 137.5, 162.5, 187.5]);
    for s in &slots {
        assert!(!back.iter().any(|(lo, hi)| lo[0] < *s && *s < hi[0] && lo[1] < 17.5 && 17.5 < hi[1]), "slot at {s}");
    }
    assert!(back.iter().any(|(lo, hi)| lo[0] < 25.0 && 25.0 < hi[0] && lo[1] < 17.5 && 17.5 < hi[1]), "a bridge between the slots");
    // Not perforated: the web is one piece.
    let plain = rail_boxes(RailProfile::TopHat { perforated: false }, 200.0, 15.0, 5.5);
    assert!(inside(&plain, 200.0, 15.0, 5.5));
    assert_eq!(plain.iter().filter(|(lo, _)| lo[2] == 0.0).count(), 1);
    // A G rail: the high side reaches the front, the low one does not.
    let g = rail_boxes(RailProfile::G, 100.0, 32.0, 15.0);
    assert!(inside(&g, 100.0, 32.0, 15.0));
    assert!(g.iter().any(|(lo, hi)| hi[2] == 15.0 && lo[1] == 0.0) && !g.iter().any(|(_, hi)| hi[2] == 15.0 && hi[1] == 32.0));
}

#[test]
fn rail_profiles_by_part() {
    let part = |number: &str, description: &str| crate::Part { number: number.into(), description: description.into(), ..Default::default() };
    assert_eq!(RailProfile::of(&part("GEN.RAIL-TS35-1M", "Mounting rail TS 35 x 7.5, perforated, 1 m")), RailProfile::TopHat { perforated: true });
    assert_eq!(RailProfile::of(&part("GEN.RAIL-TS15-1M", "Mounting rail TS 15 x 5.5, 1 m")), RailProfile::TopHat { perforated: false });
    assert_eq!(RailProfile::of(&part("GEN.RAIL-G32-1M", "Mounting rail G 32, 1 m")), RailProfile::G);
}

#[test]
fn the_system_of_an_enclosure_comes_from_its_description() {
    assert_eq!(system_from_text("VX Baying enclosure system, WHD: 800x2000x600 mm"), Some("baying"));
    assert_eq!(system_from_text("TS Bayed enclosure system"), Some("baying"));
    assert_eq!(system_from_text("AX Compact enclosure, WHD: 500x500x210 mm"), Some("compact"));
    assert_eq!(system_from_text("KX Terminal box, WHD: 200x200x120 mm"), Some("terminal_box"));
    assert_eq!(system_from_text("Network rack 19-inch, 42 U"), Some(RACK_SYSTEM));
    assert_eq!(system_from_text("Enclosure, polycarbonate, hinged cover"), Some("plastic"));
    assert_eq!(system_from_text("Wall cabinet ComfortLine A"), None);
    assert!(system_from_text("VX Baying enclosure system").and_then(system).is_some());
}
