use super::*;
use crate::parts::{FnKind, Part};
use crate::analysis::Analysis;
use crate::electrical::Transformer;
use crate::testkit::Kit;

/// L1 → -F1 → cable -W1 (one core) → heater -E1.
fn feeder(protection: (FnKind, &str, &str), cross_section: &str, length: f32) -> Kit {
    let mut k = Kit::new();
    let l1 = k.sym("pot_l1", 100.0, 20.0, "", "L1");
    let f1 = k.sym("cb1", 100.0, 40.0, "-F1", "");
    let e1 = k.sym("heater", 100.0, 80.0, "-E1", "");
    k.wire((l1, ""), (f1, "1"));
    k.core((f1, "2"), (e1, "1"), "-W1", "1");
    k.cable("-W1", "1 2 GNYE", cross_section, length);
    let (kind, current, tech) = protection;
    k.part("-F1", kind, "1 2", |p| {
        p.current = current.into();
        p.tech_data = tech.into();
    });
    k
}

fn faults_of(k: &Kit) -> Vec<Fault> {
    let an = Analysis::checked(&k.prj);
    faults(&k.prj, &an.netlist, &an.connections)
}

fn messages(k: &Kit) -> Vec<String> {
    Analysis::checked(&k.prj).issues.into_iter().map(|i| i.message()).collect()
}

#[test]
fn short_circuit_currents_at_the_device_and_the_end_of_the_cable() {
    let k = feeder((FnKind::Breaker1, "16 A", "Characteristic: C\nBreaking capacity: 10 kA"), "2.5 mm²", 0.0);
    let f = &faults_of(&k)[0];
    assert_eq!(f.ik_max, 10000.0);
    // Without cable: the loop impedance of the supply (2 × Zsupply).
    let z = C_MAX * 400.0 / (3f32.sqrt() * 10000.0);
    assert!((f.ik_min_end - C_MIN * 400.0 / 3f32.sqrt() / (2.0 * z)).abs() < 1.0, "{}", f.ik_min_end);
    assert_eq!(f.breaks(), Some(true));
    assert!(f.disconnects() && f.withstands());

    let k = feeder((FnKind::Breaker1, "16 A", "Characteristic: C\nBreaking capacity: 10 kA"), "2.5 mm²", 50.0);
    let f = &faults_of(&k)[0];
    assert_eq!((f.cable.as_str(), f.cross_section), ("-W1", 2.5));
    // 2 × 50 m of 2.5 mm² at 0.0225 Ω·mm²/m: 0.9 Ω.
    assert!(f.ik_min_end < 260.0 && f.ik_min_end > 200.0, "{}", f.ik_min_end);
    assert!(f.ik_max_end > f.ik_min_end && f.ik_max_end < 10000.0);
    assert_eq!(f.limit, Some(0.4));
    assert!((f.trip_current - 160.0).abs() < 1.0);
    assert!(f.disconnects());
}

#[test]
fn breaking_capacity_below_the_short_circuit_current() {
    let mut k = feeder((FnKind::Breaker1, "16 A", "Breaking capacity: 6 kA"), "2.5 mm²", 10.0);
    assert!(messages(&k).iter().any(|m| m == "-F1: breaking capacity 6 kA below the short-circuit current 10.00 kA"));
    k.prj.electrical.supply.ik_max = 6.0;
    assert!(!messages(&k).iter().any(|m| m.contains("breaking capacity")));
    k.prj.parts[0].tech_data.clear();
    assert!(messages(&k).iter().any(|m| m == "1 protective devices of the mains have no breaking capacity data"));
}

#[test]
fn long_cables_do_not_disconnect_in_time() {
    let mut k = feeder((FnKind::Breaker1, "16 A", "Characteristic: C\nBreaking capacity: 10 kA"), "1.5 mm²", 300.0);
    let f = &faults_of(&k)[0];
    assert!(!f.disconnects() && f.ik_min_end < 160.0);
    assert!(messages(&k).iter().any(|m| m.starts_with("-F1: fault current") && m.contains("at -W1 does not disconnect within 0.4 s (needs 160 A)")));
    // A residual current device disconnects.
    let l1 = k.prj.pages[0].symbols[0].id;
    let q1 = k.sym("rcd4", 60.0, 40.0, "-Q1", "");
    k.wire((l1, ""), (q1, "1"));
    let f1 = k.prj.pages[0].symbols[1].id;
    k.prj.pages[0].wires.remove(0);
    k.wire((q1, "2"), (f1, "1"));
    k.part("-Q1", FnKind::ResidualCurrent, "1 3 5 N 2 4 6 N'", |p| p.current = "40 A".into());
    let f = &faults_of(&k)[0];
    assert_eq!(f.residual, Some(30.0));
    assert!(f.disconnects());
    // IT system: the second fault is checked like a TN system.
    k.prj.electrical.supply.earthing = Earthing::It;
    let f = &faults_of(&k)[0];
    assert!(f.second_fault && f.limit == Some(0.4));
}

#[test]
fn second_fault_in_it_systems() {
    let mut k = feeder((FnKind::Breaker1, "16 A", "Characteristic: C\nBreaking capacity: 10 kA"), "2.5 mm²", 50.0);
    k.prj.electrical.supply.earthing = Earthing::It;
    let f = &faults_of(&k)[0];
    // U / (2 Zs) without neutral: twice the cable loop and the source in series.
    let zs = 2.0 * 0.0225 * 50.0 / 2.5;
    assert!(f.ik_min_end < C_MIN * 400.0 / (2.0 * zs) && f.ik_min_end > 0.9 * C_MIN * 400.0 / (2.0 * zs), "{}", f.ik_min_end);
    assert!(f.disconnects());
    // With a distributed neutral at U0 the current falls below the trip current.
    k.prj.electrical.supply.neutral = true;
    let f = &faults_of(&k)[0];
    assert!(!f.disconnects() && f.ik_min_end < 160.0);
    assert!(messages(&k).iter().any(|m| m.starts_with("-F1: second fault current") && m.ends_with("within 0.4 s (needs 160 A)")));
}

#[test]
fn disconnection_times_by_voltage() {
    assert_eq!([120.0, 230.0, 400.0, 600.0].map(|u| final_disconnection(u, false)), [0.8, 0.4, 0.2, 0.1]);
    assert_eq!([120.0, 230.0, 400.0, 600.0].map(|u| final_disconnection(u, true)), [0.3, 0.2, 0.07, 0.04]);
    let mut k = feeder((FnKind::Breaker1, "16 A", "Characteristic: C\nBreaking capacity: 10 kA"), "2.5 mm²", 10.0);
    k.prj.electrical.supply.voltage = 690.0;
    assert!((faults_of(&k)[0].limit.unwrap() - 0.2).abs() < 1e-6);
}

#[test]
fn transformer_and_network_as_source() {
    let mut s = Supply::default();
    let src = Source::of(&s);
    assert_eq!(src.ik3, 10000.0);
    s.transformer = Some(Transformer { power: 630.0, uk: 6.0, losses: 6500.0, network: 500.0 });
    let src = Source::of(&s);
    // About In / uk = 909 A / 0.06 with c max and the network.
    assert!(src.ik3 > 15000.0 && src.ik3 < 17500.0, "{}", src.ik3);
    assert!(src.ik1 < src.ik3 && src.ik1 > 0.8 * src.ik3, "{}", src.ik1);
    // An infinite network gives more, a weak one less.
    let ik = |network: f32| {
        let mut s = s.clone();
        s.transformer.as_mut().unwrap().network = network;
        Source::of(&s).ik3
    };
    assert!(ik(0.0) > src.ik3 && ik(50.0) < src.ik3);
    let mut k = feeder((FnKind::Breaker1, "16 A", "Characteristic: C\nBreaking capacity: 10 kA"), "2.5 mm²", 10.0);
    k.prj.electrical.supply = s;
    assert!(messages(&k).iter().any(|m| m.starts_with("-F1: breaking capacity 10 kA below the short-circuit current 1")));
}

#[test]
fn peak_current_against_the_making_capacity() {
    let mut k = feeder((FnKind::Breaker1, "16 A", "Characteristic: C\nBreaking capacity: 10 kA\nMaking capacity: 20 kA"), "2.5 mm²", 0.0);
    let f = &faults_of(&k)[0];
    // R/X 0.1: κ = 1.02 + 0.98 e^-0.3.
    let kappa = 1.02 + 0.98 * (-0.3f32).exp();
    assert!((f.ip - kappa * 2f32.sqrt() * 10000.0).abs() < 1.0, "{}", f.ip);
    assert_eq!(f.makes(), Some(false));
    assert!(messages(&k).iter().any(|m| m == "-F1: making capacity 20 kA below the peak short-circuit current 24.69 kA"));
    // A resistive supply has a smaller peak.
    k.prj.electrical.supply.rx = 1.0;
    assert_eq!(faults_of(&k)[0].makes(), Some(true));
}

/// L1 → -F0 → cable -W0 → -F1 → cable -W1 → heater -E1.
fn chain(w0: f32, w1: f32) -> Kit {
    let mut k = Kit::new();
    let l1 = k.sym("pot_l1", 100.0, 20.0, "", "L1");
    let f0 = k.sym("cb1", 100.0, 40.0, "-F0", "");
    let f1 = k.sym("cb1", 100.0, 70.0, "-F1", "");
    let e1 = k.sym("heater", 100.0, 100.0, "-E1", "");
    k.wire((l1, ""), (f0, "1"));
    k.core((f0, "2"), (f1, "1"), "-W0", "1");
    k.core((f1, "2"), (e1, "1"), "-W1", "1");
    k.cable("-W0", "1 2 GNYE", "10 mm²", w0);
    k.cable("-W1", "1 2 GNYE", "2.5 mm²", w1);
    k.part("-F0", FnKind::Breaker1, "1 2", |p| {
        p.current = "63 A".into();
        p.tech_data = "Characteristic: C\nBreaking capacity: 10 kA".into();
    });
    k.part("-F1", FnKind::Breaker1, "1 2", |p| {
        p.current = "16 A".into();
        p.tech_data = "Characteristic: C\nBreaking capacity: 6 kA".into();
    });
    k
}

#[test]
fn cables_upstream_lower_the_short_circuit_current() {
    let k = chain(20.0, 30.0);
    let fs = faults_of(&k);
    let (f0, f1) = (&fs[0], &fs[1]);
    assert_eq!((f0.device.as_str(), f0.feeders.len(), f0.cable.as_str()), ("-F0", 0, "-W0"));
    assert_eq!((f1.feeders.clone(), f1.cable.as_str()), (vec!["-W0".to_string()], "-W1"));
    assert_eq!(f0.ik_max, 10000.0);
    assert!(f1.ik_max < 0.7 * f0.ik_max, "{}", f1.ik_max);
    // The lowest fault current of -F1 takes both cables.
    let alone = feeder((FnKind::Breaker1, "16 A", "Characteristic: C"), "2.5 mm²", 30.0);
    assert!(f1.ik_min_end < faults_of(&alone)[0].ik_min_end);
    // 6 kA are enough behind 20 m of 10 mm²; directly at the supply they are not.
    assert_eq!(f1.breaks(), Some(true));
    assert_eq!(faults_of(&chain(0.0, 30.0))[1].breaks(), Some(false));
}

#[test]
fn coordination_tables_of_the_manufacturer() {
    let mut k = chain(0.0, 30.0);
    let (up, down) = (k.prj.parts[0].number.clone(), k.prj.parts[1].number.clone());
    k.prj.electrical.coordination.push(Coordination { upstream: up, downstream: down, selectivity: 3.0, backup: 10.0 });
    let fs = faults_of(&k);
    assert_eq!((fs[1].backup.as_deref(), fs[1].breaks()), (Some("-F0"), Some(true)));
    assert!(!messages(&k).iter().any(|m| m.contains("breaking capacity")));
    let an = Analysis::checked(&k.prj);
    let s = &selectivity(&k.prj, &an.netlist, &an.connections)[0];
    assert_eq!((s.limit, s.basis), (3000.0, Basis::Table));
    assert!(an.issues.iter().any(|i| i.message() == "-F1 is selective to -F0 only up to 3000 A (short-circuit current 10000 A)"));
    // Back-up only up to 8 kA does not cover 10 kA.
    k.prj.electrical.coordination[0].backup = 8.0;
    assert_eq!(faults_of(&k)[1].breaks(), Some(false));
}

#[test]
fn fuses_upstream_select_by_their_pre_arcing_energy() {
    let fuse = protection("100 A", "Utilisation category: gG\nPre-arcing energy: 40000 A²s", FnKind::Fuse);
    let mcb = protection("16 A", "Characteristic: B\nLet-through energy: 1 kA: 3000; 6 kA: 20000", FnKind::Breaker1);
    assert_eq!(coordinate(&[], &fuse, &mcb, 6000.0), (6000.0, Basis::Energy));
    let (limit, basis) = coordinate(&[], &fuse, &mcb, 20000.0);
    assert!(limit < 20000.0 && basis == Basis::Curves, "{limit}");
    let f25 = protection("25 A", "", FnKind::Fuse);
    let f50 = protection("50 A", "", FnKind::Fuse);
    assert_eq!(coordinate(&[], &f50, &f25, 10000.0), (10000.0, Basis::FuseRatio));
}

#[test]
fn motors_feed_the_short_circuit() {
    let mut k = Kit::new();
    let pots: Vec<u64> = ["L1", "L2", "L3"].iter().enumerate().map(|(i, l)| k.sym(&format!("pot_{}", l.to_lowercase()), 100.0 + 8.0 * i as f32, 20.0, "", l)).collect();
    let q1 = k.sym("mps3", 100.0, 40.0, "-Q1", "");
    let m1 = k.sym("motor3", 100.0, 70.0, "-M1", "");
    for (i, (a, b)) in [("1", "2"), ("3", "4"), ("5", "6")].iter().enumerate() {
        k.wire((pots[i], ""), (q1, a));
        k.wire((q1, b), (m1, ["U1", "V1", "W1"][i]));
    }
    k.part("-Q1", FnKind::MotorProtection, "1 2 3 4 5 6", |p| p.current = "63 A".into());
    k.part("-M1", FnKind::Motor3, "U1 V1 W1 PE", |p| p.current = "60 A".into());
    k.prj.electrical.supply.ik_max = 1.0;
    let f = &faults_of(&k)[0];
    assert!((f.ik_motors - 1.1 * 5.0 * 60.0).abs() < 0.1, "{}", f.ik_motors);
    assert!((f.ik_max - 1000.0 - f.ik_motors).abs() < 0.1);
    k.prj.parts[1].tech_data = "Locked rotor current: 7 x In".into();
    assert!((faults_of(&k)[0].ik_motors - 1.1 * 7.0 * 60.0).abs() < 0.1);
    // Up to 1 % of the supply's current they are neglected.
    k.prj.electrical.supply.ik_max = 10.0;
    assert_eq!(faults_of(&k)[0].ik_motors, 0.0);
    k.prj.electrical.supply.ik_max = 1.0;
    k.prj.electrical.supply.motors = false;
    assert_eq!(faults_of(&k)[0].ik_motors, 0.0);
}

#[test]
fn cables_must_withstand_the_energy_let_through() {
    let k = feeder((FnKind::Fuse, "32 A", "Utilisation category: gG\nBreaking capacity: 100 kA"), "0.75 mm²", 18.0);
    let f = &faults_of(&k)[0];
    let (e, allowed) = f.energy.unwrap();
    assert_eq!(allowed, (115.0f32 * 0.75).powi(2));
    assert!(e > allowed && !f.withstands());
    assert!(messages(&k).iter().any(|m| m.starts_with("-F1: lets through") && m.ends_with(&format!("cable -W1 withstands {allowed:.0} A²s"))));
    let k = feeder((FnKind::Fuse, "32 A", "Utilisation category: gG\nBreaking capacity: 100 kA"), "6 mm²", 18.0);
    assert!(faults_of(&k)[0].withstands());
}

fn protection(current: &str, tech: &str, kind: FnKind) -> Protection {
    Protection::of_part(&Part { current: current.into(), tech_data: tech.into(), ..Default::default() }, kind).unwrap()
}

#[test]
fn selectivity_of_breakers_and_fuses() {
    let c32 = protection("32 A", "Characteristic: C", FnKind::Breaker1);
    let c16 = protection("16 A", "Characteristic: C", FnKind::Breaker1);
    // At most up to the lowest instantaneous trip of the upstream breaker (5 × 32 A).
    let limit = selectivity_limit(&c32, &c16, 10000.0);
    assert!(limit > 2.0 * 16.0 && limit <= 160.0, "{limit}");
    // Equal breakers are not selective.
    assert!(selectivity_limit(&c16, &c16, 10000.0) < 2.0 * 16.0);
    let (f50, f25, f32) = (protection("50 A", "", FnKind::Fuse), protection("25 A", "", FnKind::Fuse), protection("32 A", "", FnKind::Fuse));
    assert_eq!(selectivity_limit(&f50, &f25, 10000.0), 10000.0);
    assert!(selectivity_limit(&f32, &f25, 10000.0) < 10000.0);
    // Below the rated current of the downstream device there is nothing to select.
    assert_eq!(selectivity_limit(&c32, &c16, 10.0), 10.0);
}

#[test]
fn selectivity_with_the_device_upstream() {
    let mut k = Kit::new();
    let l1 = k.sym("pot_l1", 100.0, 20.0, "", "L1");
    let f0 = k.sym("cb1", 100.0, 40.0, "-F0", "");
    let f1 = k.sym("cb1", 100.0, 60.0, "-F1", "");
    k.wire((l1, ""), (f0, "1"));
    k.wire((f0, "2"), (f1, "1"));
    k.part("-F0", FnKind::Breaker1, "1 2", |p| {
        p.current = "32 A".into();
        p.tech_data = "Characteristic: C\nBreaking capacity: 10 kA".into();
    });
    k.part("-F1", FnKind::Breaker1, "1 2", |p| {
        p.current = "16 A".into();
        p.tech_data = "Characteristic: B\nBreaking capacity: 10 kA".into();
    });
    let an = Analysis::checked(&k.prj);
    let s = selectivity(&k.prj, &an.netlist, &an.connections);
    assert_eq!(s.len(), 1);
    assert_eq!((s[0].upstream.as_str(), s[0].downstream.as_str()), ("-F0", "-F1"));
    assert!(!s[0].is_total());
    let partial = |an: &Analysis| an.issues.iter().find(|i| i.message().starts_with("-F1 is selective to -F0 only up to")).map(|i| i.severity);
    assert_eq!(partial(&an), Some(Severity::Info));
    assert_eq!(an.report(&k.prj, ReportKind::Selectivity).unwrap().rows.len(), 1);
    // A customer rule set asks for total selectivity.
    k.prj.electrical.rule_sets.push(crate::rules::RuleSet {
        name: "Customer".into(),
        active: true,
        limits: crate::rules::LimitOverrides { total_selectivity: Some(true), ..Default::default() },
        ..Default::default()
    });
    assert_eq!(partial(&Analysis::checked(&k.prj)), Some(Severity::Warning));
}

#[test]
fn tt_systems_need_a_residual_current_device() {
    let mut k = feeder((FnKind::Breaker1, "16 A", "Characteristic: C\nBreaking capacity: 10 kA"), "2.5 mm²", 10.0);
    k.prj.electrical.supply.earthing = Earthing::Tt;
    let f = &faults_of(&k)[0];
    assert!(f.needs_rcd && !f.disconnects());
    assert!((f.limit.unwrap() - 0.2).abs() < 1e-6);
    assert!(messages(&k).contains(&"-F1: a TT system needs a residual current device".to_string()));
}

#[test]
fn the_currents_behind_a_cable_are_those_of_a_circuit() {
    // The same feeder as a point behind its cable, without a schematic.
    let k = feeder((FnKind::Breaker1, "16 A", "Characteristic: C\nBreaking capacity: 10 kA"), "2.5 mm²", 30.0);
    let f = &faults_of(&k)[0];
    let (s, conditions) = (&k.prj.electrical.supply, &k.prj.electrical.conditions);
    let source = Source::of(s);
    let at_supply = source.behind(s, conditions, 0.0, 2.5);
    assert!((at_supply.ik_max - source.ik3).abs() < 1.0 && (at_supply.ik_min - source.ik1).abs() < 1.0);
    let end = source.behind(s, conditions, 30.0, 2.5);
    assert!((end.ik_max - f.ik_max_end).abs() < 1.0, "{} / {}", end.ik_max, f.ik_max_end);
    assert!((end.ik_min - f.ik_min_end).abs() < 1.0, "{} / {}", end.ik_min, f.ik_min_end);
    assert!(end.ik_max < at_supply.ik_max && end.ik_min < end.ik_max && end.ip > end.ik_max);
}
