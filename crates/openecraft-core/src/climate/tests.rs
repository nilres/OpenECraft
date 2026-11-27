use super::*;
use crate::sample::demo_project;

fn enclosure() -> Enclosure {
    Enclosure { width: 600.0, height: 800.0, depth: 250.0, ..Default::default() }
}

#[test]
fn effective_surface_by_installation() {
    let s = |i: Installation| i.surface(0.6, 0.8, 0.25);
    assert!((s(Installation::WallMounted) - 1.242).abs() < 1e-4);
    assert!((s(Installation::FreeStanding) - (1.8 * 0.8 * 0.85 + 1.4 * 0.6 * 0.25)).abs() < 1e-4);
    // Every neighbour or wall takes surface away.
    assert!(s(Installation::FreeStanding) > s(Installation::WallMounted));
    assert!(s(Installation::MiddleWallMounted) > s(Installation::MiddleWallMountedCovered));
}

#[test]
fn loss_comes_from_the_parts_of_the_placed_devices() {
    let prj = demo_project();
    let e = &prj.layout.enclosures[0];
    let (loss, without) = enclosure_loss(&prj, e);
    assert!(loss > 10.0, "{loss}");
    // Devices without data are named (terminal strips have none).
    assert!(without.iter().all(|t| prj.devices.contains_key(t)));
    let r = calculate(&prj, e);
    assert!(r.inside > e.climate.ambient_max && r.inside < e.climate.inside_max);
    assert_eq!(r.cooling, Cooling::None);
}

#[test]
fn fan_cooling_unit_and_heating() {
    let prj = Project::default();
    let mut e = enclosure();
    // ka = 5.5 W/K · 1.242 m² = 6.831 W/K; 10 K reserve gives off 68 W.
    e.climate.extra_loss = 50.0;
    assert_eq!(calculate(&prj, &e).cooling, Cooling::None);
    e.climate.extra_loss = 168.31;
    match calculate(&prj, &e).cooling {
        Cooling::Fan(v) => assert!((v - 31.0).abs() < 0.1, "{v}"),
        c => panic!("{c:?}"),
    }
    // Hot surroundings: only a cooling unit helps.
    e.climate.ambient_max = 33.0;
    match calculate(&prj, &e).cooling {
        Cooling::CoolingUnit(q) => assert!((q - (168.31 - 6.831 * 2.0)).abs() < 0.1, "{q}"),
        c => panic!("{c:?}"),
    }
    assert_eq!(calculate(&prj, &e).heating, 0.0);
    e.climate.ambient_min = -15.0;
    assert!((calculate(&prj, &e).heating - 6.831 * 20.0).abs() < 0.1);
}

#[test]
fn power_loss_is_read_from_the_tech_data() {
    let mut p = crate::parts::Part { tech_data: "Clearance: 25 mm\nPower loss: 7,5 W".into(), ..Default::default() };
    assert_eq!(power_loss(&p), Some(7.5));
    p.tech_data.clear();
    assert_eq!(power_loss(&p), None);
}

#[test]
fn too_warm_enclosures_are_reported() {
    let mut prj = demo_project();
    prj.layout.enclosures[0].climate.extra_loss = 500.0;
    let an = crate::analysis::Analysis::checked(&prj);
    assert!(an.issues.iter().any(|i| i.message().contains("needed")));
    let t = &an.report(&prj, crate::model::ReportKind::Climate).unwrap();
    assert_eq!(t.rows.len(), prj.layout.enclosures.len());
}

#[test]
fn altitude_and_sealed_enclosures() {
    let prj = Project::default();
    let mut e = enclosure();
    e.climate.extra_loss = 168.31;
    e.climate.altitude = 2000.0;
    // The air at 2000 m has about 82 % of the density at sea level.
    assert!((air_density(2000.0) - 0.8216).abs() < 1e-3, "{}", air_density(2000.0));
    match calculate(&prj, &e).cooling {
        Cooling::Fan(v) => assert!((v - 31.0 / air_density(2000.0)).abs() < 0.1, "{v}"),
        c => panic!("{c:?}"),
    }
    // A sealed enclosure gets a heat exchanger of 100 W / 10 K.
    e.climate.sealed = true;
    match calculate(&prj, &e).cooling {
        Cooling::HeatExchanger(q) => assert!((q - 10.0).abs() < 0.01, "{q}"),
        c => panic!("{c:?}"),
    }
    assert_eq!(Cooling::HeatExchanger(10.0).text(), "heat exchanger 10.0 W/K");
    e.climate.ambient_max = 33.0;
    assert!(matches!(calculate(&prj, &e).cooling, Cooling::CoolingUnit(_)));
}

#[test]
fn the_heater_keeps_the_dew_point() {
    assert!((dew_point(25.0, 60.0) - 16.7).abs() < 0.1, "{}", dew_point(25.0, 60.0));
    assert!((dew_point(20.0, 100.0) - 20.0).abs() < 0.01);
    let prj = Project::default();
    let mut e = enclosure();
    e.climate.ambient_min = 10.0;
    assert_eq!(calculate(&prj, &e).heating, 0.0);
    // 25 °C at 60 %: the inside must stay above 16.7 °C when it is 10 °C outside.
    e.climate.humidity = 60.0;
    let r = calculate(&prj, &e);
    assert!((r.dew_point.unwrap() - 16.7).abs() < 0.1);
    assert!((r.heating - 6.831 * (r.dew_point.unwrap() - 10.0)).abs() < 0.1, "{}", r.heating);
}

#[test]
fn an_open_assembly_gives_its_heat_to_the_room() {
    let prj = Project::default();
    let mut e = Enclosure { kind: crate::panel::EnclosureKind::Assembly, ..enclosure() };
    e.climate.extra_loss = 500.0;
    e.climate.ambient_min = -15.0;
    let r = calculate(&prj, &e);
    assert!(r.open);
    assert_eq!((r.inside, r.cooling, r.heating, r.loss), (e.climate.ambient_max, Cooling::None, 0.0, 500.0));
}
