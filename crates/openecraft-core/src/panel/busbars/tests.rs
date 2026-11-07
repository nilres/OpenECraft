use super::*;
use crate::sample::example;

fn bar_with(prj: &Project, tag: &str) -> u64 {
    let (_, it) = crate::panel::locate(prj, tag).unwrap_or_else(|| panic!("{tag}"));
    let e = prj.layout.enclosures.iter().find(|e| e.items.iter().any(|i| i.id == it.id)).unwrap();
    e.items.iter().find(|b| b.kind == ItemKind::Busbar && b.y <= it.y + it.h && it.y <= b.y + b.h && b.x <= it.x && it.x <= b.x + b.w).unwrap().id
}

#[test]
fn each_bar_carries_one_conductor() {
    let prj = example("industrial_main_distribution").unwrap();
    let nl = Netlist::build(&prj, true);
    let uses = uses(&prj, &nl);
    // The supply busbar: the phases of both supply cables and the isolation strip.
    let supply = uses.iter().find(|u| u.busbar == bar_with(&prj, "=GRID+HV1-X01")).unwrap();
    assert_eq!((supply.bars, supply.nets.len()), (3, 3), "{supply:?}");
    assert!(supply.devices.contains(&"=MS+HV1-F10".to_string()));
    // The transformers of the metering sit in it: one conductor through each.
    assert!(supply.devices.contains(&"=MS+HV1-T1".to_string()));
    // The strips behind the coupling switch have a busbar of their own.
    let pv = uses.iter().find(|u| u.busbar == bar_with(&prj, "=PV+HV1-Q41")).unwrap();
    assert!(!pv.devices.contains(&"=MS+HV1-Q10".to_string()), "{pv:?}");
    // The measured strips and the heat pumps behind the transformers of the power
    // analyser: the conductors through their transformers.
    let dist = uses.iter().find(|u| u.busbar == bar_with(&prj, "=DIST+HV2-Q21")).unwrap();
    assert_eq!((dist.devices.len(), dist.nets.len()), (7, 3), "{dist:?}");
    assert!(dist.devices.contains(&"=EMS+HV2-T5".to_string()));
    assert!(problems(&prj, &nl).is_empty());
}

#[test]
fn a_pen_terminal_on_the_phase_bars_is_a_short_circuit() {
    let mut prj = example("industrial_main_distribution").unwrap();
    // The PEN terminals moved onto the busbar of the phases.
    let (ei, x0) = crate::panel::locate(&prj, "=GRID+HV1-X02").map(|(ei, it)| (ei, it.clone())).unwrap();
    let x1 = prj.layout.enclosures[ei].items.iter_mut().find(|i| i.tag == "=GRID+HV1-X1").unwrap();
    (x1.x, x1.y) = (x0.x + x0.w + 20.0, x0.y + 20.0);
    let nl = Netlist::build(&prj, true);
    let issues = problems(&prj, &nl);
    assert_eq!(issues.len(), 1, "{issues:?}");
    assert!(issues[0].message().contains("3 bars joins 4 conductors"), "{}", issues[0].message());
}

#[test]
fn the_bars_are_drawn_apart() {
    let prj = example("industrial_main_distribution").unwrap();
    let id = bar_with(&prj, "=DIST+HV2-Q21");
    let it = prj.layout.enclosures.iter().flat_map(|e| &e.items).find(|i| i.id == id).unwrap();
    let tris = mesh(&prj, it);
    // Three copper bars across the system (y), apart from each other.
    let mut ys: Vec<f32> = tris.iter().filter(|t| t.1 == Some([200, 120, 60])).flat_map(|t| t.0.iter().map(|v| v[1])).collect();
    ys.sort_by(f32::total_cmp);
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.01);
    assert_eq!(ys.len(), 6, "{ys:?}");
    assert!(ys.windows(2).skip(1).step_by(2).all(|w| w[1] - w[0] > 50.0), "gaps between the bars: {ys:?}");
}

#[test]
fn the_mesh_is_wound_like_a_placed_model() {
    // In plate coordinates (y down) a placed model is wound inside out: the front faces
    // of the bars (at the system's depth) point into the plate, like those of a box of
    // a terminal without a model. The 3D view turns them right side out with its y.
    let prj = example("industrial_main_distribution").unwrap();
    let id = bar_with(&prj, "=GRID+HV1-X01");
    let bar = prj.layout.enclosures.iter().flat_map(|e| &e.items).find(|i| i.id == id).unwrap();
    let normal = |t: &Tri| crate::parts::vec3::norm(crate::parts::vec3::cross(crate::parts::vec3::sub(t[1], t[0]), crate::parts::vec3::sub(t[2], t[0])));
    let painted = mesh(&prj, bar);
    let front: Vec<&Tri> = painted.iter().filter(|t| t.1 == Some(COPPER) && t.0.iter().all(|v| (v[2] - bar.z - bar.d).abs() < 0.01)).map(|t| &t.0).collect();
    assert_eq!(front.len(), 6);
    assert!(front.iter().all(|t| normal(t)[2] < -0.99), "{:?}", front.iter().map(|t| normal(t)).collect::<Vec<_>>());
    let (_, x01) = crate::panel::locate(&prj, "=GRID+HV1-X01").unwrap();
    let tris = crate::panel::device_mesh(&prj, x01).unwrap();
    let front: Vec<&Tri> = tris.iter().filter(|t| t.iter().all(|v| (v[2] - x01.z - x01.d).abs() < 0.01)).collect();
    assert!(!front.is_empty() && front.iter().all(|t| normal(t)[2] < -0.99));
}

#[test]
fn the_holders_carry_the_bars() {
    let prj = example("industrial_main_distribution").unwrap();
    let id = bar_with(&prj, "=DIST+HV2-Q21");
    let it = prj.layout.enclosures.iter().flat_map(|e| &e.items).find(|i| i.id == id).unwrap();
    let ys = |color: [u8; 3]| {
        let v: Vec<f32> = mesh(&prj, it).iter().filter(|t| t.1 == Some(color)).flat_map(|t| t.0.iter().map(|v| v[1])).collect();
        (v.iter().copied().fold(f32::MAX, f32::min), v.iter().copied().fold(f32::MIN, f32::max))
    };
    let ((b0, b1), (h0, h1)) = (ys(COPPER), ys(HOLDER));
    assert!(h0 < b0 && b1 < h1, "bars {b0}..{b1}, holders {h0}..{h1}");
    assert!(h0 >= it.y - 0.01 && h1 <= it.y + it.h + 0.01, "holders {h0}..{h1} in {}..{}", it.y, it.y + it.h);
    // In the 3D exports the bars and their holders are solids of their own colour.
    let e = prj.layout.enclosures.iter().find(|e| e.items.iter().any(|i| i.id == id)).unwrap();
    let solids = crate::export3d::enclosure_solids(&prj, e);
    let colour = |name: &str| solids.iter().filter(|s| s.name == name).map(|s| s.color).collect::<std::collections::BTreeSet<_>>();
    assert_eq!((colour("Busbar"), colour("Busbar holder")), ([COPPER].into(), [HOLDER].into()));
}

#[test]
fn the_holders_stand_beside_the_devices_in_the_bars() {
    let prj = example("industrial_main_distribution").unwrap();
    let id = bar_with(&prj, "=MS+HV1-T1");
    let e = prj.layout.enclosures.iter().find(|e| e.items.iter().any(|i| i.id == id)).unwrap();
    let bar = e.items.iter().find(|i| i.id == id).unwrap();
    // The transformers of the metering sit in the bars (the strips and the terminals of
    // the supply cables reach across the whole system: mounted on it, over the
    // holders) on a drilling position: that holder moves beside them, the others stay
    // at theirs.
    let on_bars: Vec<&PanelItem> = e.items.iter().filter(|d| d.kind == ItemKind::Device && d.x < bar.x + bar.w && bar.x < d.x + d.w && d.y < bar.y + bar.h && bar.y < d.y + d.h && d.h < bar.h - 1.0).collect();
    assert_eq!(on_bars.iter().map(|d| d.tag.as_str()).collect::<Vec<_>>(), ["=MS+HV1-T1"]);
    let holes = crate::production::line_positions(bar.w, crate::production::BUSBAR_HOLES.0, crate::production::BUSBAR_HOLES.1);
    let places = holder_places(&prj, bar);
    assert_eq!(places.len(), holes.len(), "{places:?} for {holes:?}");
    assert!(places.iter().zip(&holes).any(|(a, t)| (a - t).abs() > 10.0) && places.iter().zip(&holes).any(|(a, t)| (a - t).abs() < 0.01), "{places:?} for {holes:?}");
    for a in &places {
        let (x0, x1) = (bar.x + a - HOLDER_WIDTH / 2.0, bar.x + a + HOLDER_WIDTH / 2.0);
        assert!(on_bars.iter().all(|d| x1 <= d.x + 0.01 || x0 >= d.x + d.w - 0.01), "a holder at {x0}..{x1} under {:?}", on_bars.iter().map(|d| (&d.tag, d.x, d.x + d.w)).collect::<Vec<_>>());
    }
    // Drawn there, and nowhere else.
    let xs: Vec<f32> = mesh(&prj, bar).iter().filter(|t| t.1 == Some(HOLDER)).flat_map(|t| t.0.iter().map(|v| v[0])).collect();
    assert!(xs.iter().all(|x| places.iter().any(|a| (x - (bar.x + a)).abs() <= HOLDER_WIDTH / 2.0 + 0.01)), "{xs:?}");
}

#[test]
fn packed_busbar_terminals_are_entered_from_the_front() {
    // Example 13: feed-in terminals 60 mm apart on the main busbar, 6 mm between them:
    // the points of the inner terminal facing its neighbours leave to the front; the
    // outer points keep their way. Example 14: 185 mm apart, every point keeps its way.
    use crate::parts::Exit;
    let exits = |key: &str, tag: &str| -> Vec<(String, Exit)> {
        let prj = example(key).unwrap();
        let (_, it) = crate::panel::locate(&prj, tag).unwrap();
        crate::panel::device_points(&prj, it).into_iter().map(|p| (p.terminal, p.exit)).collect()
    };
    let x0 = exits("pv_transformer_metering", "=GRID+HV-X0");
    assert_eq!(x0.iter().filter(|(t, _)| t == "L2").map(|(_, e)| *e).collect::<Vec<_>>(), [Exit::Front, Exit::Front], "{x0:?}");
    assert!(x0.iter().any(|(t, e)| t == "L1" && *e == Exit::Up) && x0.iter().any(|(t, e)| t == "L3" && *e == Exit::Down), "{x0:?}");
    let x01 = exits("industrial_main_distribution", "=GRID+HV1-X01");
    assert!(x01.iter().all(|(_, e)| matches!(e, Exit::Up | Exit::Down)), "{x01:?}");
}

#[test]
fn each_supply_terminal_sits_on_its_bar() {
    let prj = example("industrial_main_distribution").unwrap();
    let id = bar_with(&prj, "=GRID+HV1-X01");
    let bar = prj.layout.enclosures.iter().flat_map(|e| &e.items).find(|i| i.id == id).unwrap();
    let centres: Vec<f32> = crate::production::bar_centres(&prj.parts, &bar.part, bar.h).into_iter().map(|c| bar.y + c).collect();
    for tag in ["=GRID+HV1-X01", "=GRID+HV1-X02"] {
        let (_, it) = crate::panel::locate(&prj, tag).unwrap();
        let places = terminals_on_bars(&prj, it).unwrap_or_else(|| panic!("{tag} is not across the bars"));
        assert_eq!(places.iter().map(|p| p.2.as_str()).collect::<Vec<_>>(), ["1", "2", "3"]);
        for ((sub, _, _), c) in places.iter().zip(&centres) {
            assert!((sub.y + sub.h / 2.0 - c).abs() < 0.1, "{tag}: terminal at {} for the bar at {c}", sub.y);
        }
        // The cable comes in from below: the terminals upright, the points on them.
        let points = crate::panel::device_points(&prj, it);
        for (sub, _, label) in &places {
            let on = points.iter().filter(|p| &p.terminal == label).collect::<Vec<_>>();
            assert!(!on.is_empty(), "{tag}:{label}");
            assert!(on.iter().all(|p| p.pos[1] >= sub.y - 0.1 && p.pos[1] <= sub.y + sub.h + 0.1), "{tag}:{label} {on:?}");
        }
    }
    // A strip along the bars stays a strip.
    let (_, x1) = crate::panel::locate(&prj, "=GRID+HV1-X1").unwrap();
    assert!(terminals_on_bars(&prj, x1).is_none());
}
