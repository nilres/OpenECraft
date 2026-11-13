use super::*;
use crate::model::panel::{PvString, RoofShape};
use crate::panel::buildings::add_building;
use crate::panel::site_poses;

/// A building with a gable roof of 10 × 8 m at 45° and an array of 4 × 2 modules in its
/// front slope with one string of `series` modules, `parallel` times.
fn project(series: u32, parallel: u32) -> Project {
    let mut prj = Project::default();
    let bi = add_building(&mut prj, "+B1").unwrap();
    let mut r = Roof { name: "R1".into(), shape: RoofShape::Gable, width: 10_000.0, depth: 8000.0, eaves: 3000.0, pitch: 45.0, thickness: 300.0, ..Default::default() };
    let s = PvString { name: "S1".into(), series, parallel, inverter: "=PV-T1".into(), mppt: 1, ..Default::default() };
    r.solar.push(PvArray { id: 7, name: "PV1".into(), x: 5000.0, z: 6000.0, earthing: "=PV-PE1".into(), strings: vec![s], ..Default::default() });
    prj.layout.buildings[bi].roofs.push(r);
    prj
}

fn lines(prj: &Project) -> Vec<PvLine> {
    let b = &prj.layout.buildings[0];
    cabling(prj, &site_poses(prj), b, &b.roofs[0], &b.roofs[0].solar[0])
}

#[test]
fn strings_take_the_modules_row_by_row_back_and_forth() {
    let prj = project(4, 2);
    let r = &prj.layout.buildings[0].roofs[0];
    let g = geometry(r, &r.solar[0]).unwrap();
    assert_eq!(string_order(&g, &r.solar[0]), vec![0, 1, 2, 3, 7, 6, 5, 4]);
    assert_eq!(string_modules(&g, &r.solar[0]), vec![(0, vec![0, 1, 2, 3]), (0, vec![7, 6, 5, 4])]);
    // More modules in series than there are: as many as there are.
    let prj = project(20, 1);
    let r = &prj.layout.buildings[0].roofs[0];
    assert_eq!(string_modules(&g, &r.solar[0])[0].1.len(), 8);
}

#[test]
fn modules_are_linked_and_the_leads_end_under_the_roof_without_an_inverter() {
    let prj = project(4, 2);
    let all = lines(&prj);
    // Three links in each of the two strings, a plus and a minus lead each, one earthing
    // conductor.
    let count = |lead: Lead| all.iter().filter(|l| l.lead == lead).count();
    assert_eq!((count(Lead::Link), count(Lead::Plus), count(Lead::Minus), count(Lead::Earth)), (6, 2, 2, 1));
    let r = &prj.layout.buildings[0].roofs[0];
    let g = geometry(r, &r.solar[0]).unwrap();
    // The links run under the modules, above the covering.
    let cover = |q: V3| g.plane.at([q[0], q[2]]) + r.thickness * (1.0 + g.plane.slope() * g.plane.slope()).sqrt();
    for l in all.iter().filter(|l| l.lead == Lead::Link) {
        assert!(l.points.iter().all(|q| q[1] > cover(*q) + 10.0), "{:?}", l.points);
    }
    // No inverter in the layout: the leads go through the roof and end under it.
    for l in all.iter().filter(|l| l.lead != Lead::Link) {
        assert!(!l.reached);
        let end = *l.points.last().unwrap();
        assert!(end[1] < roofs::lowest(&roofs::planes(r), [end[0], end[2]]), "{end:?}");
    }
    // The earthing conductor through all four earthing points of the rails.
    let earth = all.iter().find(|l| l.lead == Lead::Earth).unwrap();
    assert_eq!(g.earth.len(), 4);
    assert!(g.earth.iter().all(|e| earth.points.contains(e)));
}

#[test]
fn the_leads_run_down_onto_the_inverter_and_the_busbar() {
    let mut prj = project(8, 1);
    // A room under the roof with the inverter and the busbar on its wall.
    let level = prj.layout.buildings[0].levels[0].id;
    let ri = crate::panel::buildings::add_room(&mut prj, "+AT1", Some(level)).unwrap();
    {
        let e = &mut prj.layout.enclosures[ri];
        (e.width, e.depth, e.height) = (10_000.0, 8000.0, 2700.0);
        e.site = Some(crate::panel::SitePlace::default());
    }
    for (tag, part) in [("=PV-T1", "GEN.INV-PV-10KW"), ("=PV-PE1", "GEN.MET-BAR-10")] {
        let mut d = crate::model::DeviceData::default();
        d.parts.push(crate::parts::PartRef { number: part.into(), count: 1, total: false });
        prj.devices.insert(tag.into(), d);
        if let Some(p) = crate::parts::catalog().iter().find(|p| p.number == part) {
            prj.parts.push(p.clone());
        }
    }
    let h = prj.layout.enclosures[ri].height;
    crate::panel::place_device_on(&mut prj, ri, "=PV-T1", crate::geom::p(3000.0, h - 2000.0), crate::model::panel::Face::Wall(0)).unwrap();
    crate::panel::place_device_on(&mut prj, ri, "=PV-PE1", crate::geom::p(3000.0, h - 1200.0), crate::model::panel::Face::Wall(0)).unwrap();
    let poses = site_poses(&prj);
    let all = lines(&prj);
    assert!(!all.iter().any(|l| l.lead == Lead::Main), "no main cable without a generator junction box");
    for (lead, tag) in [(Lead::Plus, "=PV-T1"), (Lead::Minus, "=PV-T1"), (Lead::Earth, "=PV-PE1")] {
        let l = all.iter().find(|l| l.lead == lead).unwrap();
        assert!(l.reached && l.to == tag);
        let (_, (lo, hi)) = placed_device(&prj, &poses, tag).unwrap();
        let end = *l.points.last().unwrap();
        // On the middle of the device's top (the plus and minus lead side by side).
        assert!((end[1] - hi[1]).abs() < 0.5 && end[0] > lo[0] - 50.0 && end[0] < hi[0] + 50.0, "{tag}: {end:?} {lo:?} {hi:?}");
        // Under the roof all the way from the cable entry down.
        let r = &prj.layout.buildings[0].roofs[0];
        let entry = l.points.iter().position(|q| q[1] < roofs::lowest(&roofs::planes(r), [q[0], q[2]])).unwrap();
        assert!(l.points[entry..].iter().all(|q| roofs::underside_at(r, [q[0], q[2]]).is_none_or(|_| q[1] < roofs::lowest(&roofs::planes(r), [q[0], q[2]]))));
    }
}

#[test]
fn strings_run_to_the_generator_junction_box_and_a_main_cable_on_to_the_inverter() {
    let mut prj = project(4, 2);
    let level = prj.layout.buildings[0].levels[0].id;
    let ri = crate::panel::buildings::add_room(&mut prj, "+AT1", Some(level)).unwrap();
    {
        let e = &mut prj.layout.enclosures[ri];
        (e.width, e.depth, e.height) = (10_000.0, 8000.0, 2700.0);
        e.site = Some(crate::panel::SitePlace::default());
    }
    for (tag, part) in [("=PV-T1", "GEN.INV-PV-10KW"), ("=PV-U1", "GEN.GAK-2S")] {
        let mut d = crate::model::DeviceData::default();
        d.parts.push(crate::parts::PartRef { number: part.into(), count: 1, total: false });
        prj.devices.insert(tag.into(), d);
        prj.parts.push(crate::parts::catalog().iter().find(|p| p.number == part).unwrap().clone());
    }
    let h = prj.layout.enclosures[ri].height;
    crate::panel::place_device_on(&mut prj, ri, "=PV-T1", crate::geom::p(3000.0, h - 2000.0), crate::model::panel::Face::Wall(0)).unwrap();
    crate::panel::place_device_on(&mut prj, ri, "=PV-U1", crate::geom::p(4000.0, h - 1800.0), crate::model::panel::Face::Wall(0)).unwrap();
    prj.layout.buildings[0].roofs[0].solar[0].combiner = "=PV-U1".into();
    let poses = site_poses(&prj);
    let all = lines(&prj);
    // Both strings' leads end on the generator junction box, side by side.
    let leads: Vec<&PvLine> = all.iter().filter(|l| matches!(l.lead, Lead::Plus | Lead::Minus)).collect();
    assert_eq!(leads.len(), 4);
    assert!(leads.iter().all(|l| l.to == "=PV-U1" && l.reached));
    let ends: Vec<V3> = leads.iter().map(|l| *l.points.last().unwrap()).collect();
    for (i, a) in ends.iter().enumerate() {
        assert!(ends[i + 1..].iter().all(|b| crate::parts::vec3::dist(*a, *b) > 1.0), "apart: {ends:?}");
    }
    // One main cable from its top to the inverter.
    let main: Vec<&PvLine> = all.iter().filter(|l| l.lead == Lead::Main).collect();
    assert_eq!(main.len(), 1);
    let (_, (_, inv_hi)) = placed_device(&prj, &poses, "=PV-T1").unwrap();
    let (_, (_, gak_hi)) = placed_device(&prj, &poses, "=PV-U1").unwrap();
    assert!((main[0].points[0][1] - gak_hi[1]).abs() < 0.5 && (main[0].points.last().unwrap()[1] - inv_hi[1]).abs() < 0.5);
    assert_eq!(main[0].to, "=PV-T1");
    assert_eq!(junction_boxes(&prj), vec!["=PV-U1".to_string()]);
}
