use super::*;
use crate::testkit::Kit;

#[test]
fn installation_conditions_correct_the_capacity() {
    let mut c = Conditions::default();
    assert!((c.factor() - 1.0).abs() < 1e-6);
    c.ambient = 50.0;
    assert!((c.factor() - (20f32 / 30.0).sqrt()).abs() < 1e-5);
    c.ambient = 40.0;
    c.grouped = 3;
    assert!((c.factor() - 0.7).abs() < 1e-6);
    c.grouped = 10;
    assert!((c.factor() - 0.5).abs() < 1e-6);
    c.grouped = 1;
    c.material = Material::Aluminium;
    assert!((c.factor() - 0.78).abs() < 1e-6);
    assert!((c.rho_hot() - 0.036).abs() < 1e-6);
    assert_eq!(c.k(), 76.0);
    assert!((Conditions::default().rho_hot() - 0.0225).abs() < 1e-6);
}

#[test]
fn values_with_units() {
    assert_eq!(value_with_unit("0.75 kW", "kW"), Some(0.75));
    assert_eq!(value_with_unit("60 W", "W"), Some(60.0));
    assert_eq!(value_with_unit("0.75 kW", "W"), None);
    assert_eq!(value_with_unit("1.5 kVA", "VA"), None);
    assert_eq!(value_with_unit("Breaking capacity 6,5 kA", "kA"), Some(6.5));
    assert_eq!(value_with_unit("30 mA, type A", "mA"), Some(30.0));
    assert_eq!(value_with_unit("4 kW / 400 V", "W"), None);
}

#[test]
fn potential_kinds_from_definitions_and_names() {
    let prj = Project::new_default();
    assert_eq!(net_kind(&prj, "L2"), PotentialKind::L);
    assert_eq!(net_kind(&prj, "+24V"), PotentialKind::DcPlus);
    let bare = Project::default();
    assert_eq!(net_kind(&bare, "L3"), PotentialKind::L);
    assert_eq!(net_kind(&bare, "PEN"), PotentialKind::Pe);
    assert_eq!(net_kind(&bare, "X"), PotentialKind::Other);
}

/// L1 → -F0 (C32) → -F1 (C16) → heater; a breaker on +24V is no circuit of the mains.
#[test]
fn circuits_of_the_mains_with_the_device_upstream() {
    let mut k = Kit::new();
    let l1 = k.sym("pot_l1", 100.0, 20.0, "", "L1");
    let f0 = k.sym("cb1", 100.0, 40.0, "-F0", "");
    let f1 = k.sym("cb1", 100.0, 60.0, "-F1", "");
    let e1 = k.sym("heater", 100.0, 80.0, "-E1", "");
    k.wire((l1, ""), (f0, "1"));
    k.wire((f0, "2"), (f1, "1"));
    k.wire((f1, "2"), (e1, "1"));
    let p24 = k.sym("pot_24", 200.0, 20.0, "", "+24V");
    let f9 = k.sym("cb1", 200.0, 40.0, "-F9", "");
    k.wire((p24, ""), (f9, "1"));
    k.part("-F0", FnKind::Breaker1, "1 2", |p| p.current = "32 A".into());
    k.part("-F1", FnKind::Breaker1, "1 2", |p| p.current = "16 A".into());
    k.part("-F9", FnKind::Breaker1, "1 2", |p| p.current = "2 A".into());
    let nl = Netlist::build(&k.prj, true);
    let m = mains(&k.prj, &nl);
    assert!(m.values().all(|p| p == "L1"));
    let cs = circuits(&k.prj, &nl);
    assert_eq!(cs.iter().map(|c| c.device.as_str()).collect::<Vec<_>>(), ["-F0", "-F1"]);
    assert_eq!((cs[0].parent, cs[1].parent), (None, Some(0)));
    assert_eq!(cs[1].phases.iter().collect::<Vec<_>>(), ["L1"]);
    assert!(cs[1].is_final() && cs[1].residual.is_none());
    assert_eq!(cs[0].protection.rated, 32.0);
}

/// A residual current device protects the breakers behind it, not those beside it.
#[test]
fn residual_current_devices_protect_the_circuits_behind_them() {
    let mut k = Kit::new();
    let l1 = k.sym("pot_l1", 100.0, 20.0, "", "L1");
    let q1 = k.sym("rcd4", 100.0, 40.0, "-Q1", "");
    let f1 = k.sym("cb1", 100.0, 60.0, "-F1", "");
    let f2 = k.sym("cb1", 60.0, 40.0, "-F2", "");
    k.wire((l1, ""), (q1, "1"));
    k.wire((q1, "2"), (f1, "1"));
    k.wire((l1, ""), (f2, "1"));
    k.part("-Q1", FnKind::ResidualCurrent, "1 3 5 N 2 4 6 N'", |p| {
        p.current = "40 A".into();
        p.tech_data = "Rated residual current: 100 mA".into();
    });
    k.part("-F1", FnKind::Breaker1, "1 2", |p| p.current = "16 A".into());
    k.part("-F2", FnKind::Breaker1, "1 2", |p| p.current = "16 A".into());
    let cs = circuits(&k.prj, &Netlist::build(&k.prj, true));
    let by = |t: &str| cs.iter().find(|c| c.device == t).unwrap();
    assert_eq!(by("-F1").residual, Some(100.0));
    assert_eq!(by("-F2").residual, None);
}

/// A fuse feeding a breaker directly and another through a residual current device:
/// only the one behind the residual current device is protected by it, not the fuse
/// nor the breaker beside it. An RCD right after a breaker that takes all its load
/// protects it.
#[test]
fn a_residual_current_device_behind_a_feeder_protects_only_what_is_behind_it() {
    let mut k = Kit::new();
    let l1 = k.sym("pot_l1", 40.0, 20.0, "", "L1");
    let f1 = k.sym("fuse", 40.0, 40.0, "-F1", "");
    let f2 = k.sym("cb1", 40.0, 80.0, "-F2", "");
    let f3 = k.sym("rcd4", 80.0, 80.0, "-F3", "");
    let f4 = k.sym("cb1", 80.0, 120.0, "-F4", "");
    let f5 = k.sym("cb1", 160.0, 40.0, "-F5", "");
    let f6 = k.sym("rcd4", 160.0, 80.0, "-F6", "");
    k.wire((l1, ""), (f1, "1"));
    k.wire((f1, "2"), (f2, "1"));
    k.wire((f1, "2"), (f3, "1"));
    k.wire((f3, "2"), (f4, "1"));
    k.wire((l1, ""), (f5, "1"));
    k.wire((f5, "2"), (f6, "1"));
    for t in ["-F3", "-F6"] {
        k.part(t, FnKind::ResidualCurrent, "1 3 5 N 2 4 6 N'", |p| p.current = "40 A".into());
    }
    k.part("-F1", FnKind::Fuse, "1 2", |p| p.current = "35 A".into());
    for t in ["-F2", "-F4", "-F5"] {
        k.part(t, FnKind::Breaker1, "1 2", |p| p.current = "16 A".into());
    }
    let cs = circuits(&k.prj, &Netlist::build(&k.prj, true));
    let by = |t: &str| cs.iter().find(|c| c.device == t).unwrap_or_else(|| panic!("{t}"));
    assert_eq!(by("-F1").residual, None);
    assert_eq!(by("-F2").residual, None);
    assert_eq!(by("-F4").residual, Some(30.0));
    assert_eq!(by("-F5").residual, Some(30.0));
}
