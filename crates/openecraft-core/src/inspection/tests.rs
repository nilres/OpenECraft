use super::*;
use crate::analysis::Analysis;
use crate::parts::FnKind;
use crate::testkit::Kit;

/// L1 → -Q1 (RCD, 30 mA type A) → -F1 (B16) → cable -W1 → heater -E1; -F2 beside it
/// on L1 without a part, -F3 (16 A, no characteristic) without an RCD.
fn installation() -> Kit {
    let mut k = Kit::new();
    let l1 = k.sym("pot_l1", 100.0, 20.0, "", "L1");
    let q1 = k.sym("rcd4", 100.0, 40.0, "-Q1", "");
    let f1 = k.sym("cb1", 100.0, 70.0, "-F1", "");
    let e1 = k.sym("heater", 100.0, 100.0, "-E1", "");
    k.wire((l1, ""), (q1, "1"));
    k.wire((q1, "2"), (f1, "1"));
    k.core((f1, "2"), (e1, "1"), "-W1", "1");
    k.cable("-W1", "1 2 GNYE", "2.5 mm²", 20.0);
    k.prj.cables[0].cable_type = "NYM-J 3x2.5".into();
    let f2 = k.sym("cb1", 160.0, 40.0, "-F2", "");
    k.wire((l1, ""), (f2, "1"));
    let f3 = k.sym("cb1", 220.0, 40.0, "-F3", "");
    k.wire((l1, ""), (f3, "1"));
    k.part("-Q1", FnKind::ResidualCurrent, "1 3 5 N 2 4 6 N'", |p| {
        p.current = "40 A".into();
        p.tech_data = "Rated residual current: 30 mA\nResidual current type: A".into();
    });
    k.part("-F1", FnKind::Breaker1, "1 2", |p| {
        p.current = "16 A".into();
        p.type_number = "B16".into();
    });
    k.part("-F3", FnKind::Breaker1, "1 2", |p| p.current = "16 A".into());
    k.prj.devices.get_mut("-F1").unwrap().description = "Heating".into();
    k
}

fn circuits_of(prj: &Project) -> Vec<Circuit> {
    let an = Analysis::build(prj);
    circuits(prj, &an.netlist, &an.connections)
}

fn by<'a>(cs: &'a [Circuit], tag: &str) -> &'a Circuit {
    cs.iter().find(|c| c.circuit == tag).unwrap()
}

#[test]
fn circuits_are_filled_in_from_the_schematic() {
    let k = installation();
    let cs = circuits_of(&k.prj);
    assert_eq!(cs.iter().map(|c| c.circuit.as_str()).collect::<Vec<_>>(), ["-F1", "-F2", "-F3"]);
    let f1 = by(&cs, "-F1");
    assert_eq!(f1.description.as_deref(), Some("Heating"));
    assert_eq!(f1.phases, ["L1"]);
    assert_eq!(f1.cables, ["-W1 NYM-J 3x2.5"]);
    assert_eq!(f1.cross_section.as_deref(), Some("2.5 mm²"));
    assert_eq!((f1.characteristic.as_deref(), f1.rated), (Some("B"), Some(16.0)));
    assert_eq!(f1.rcd, Some(Rcd { device: "-Q1".into(), kind: Some("A".into()), rated: Some(30.0) }));
    // Disconnection by the residual current device: Zs × IΔn ≤ U0.
    let u0 = k.prj.electrical.supply.u0();
    assert!((f1.zs_max.unwrap() - u0 / 0.03).abs() < 0.5);
    assert_eq!(f1.verdict, Verdict::NotTested);
    assert_eq!(f1.required(), [Field::Continuity, Field::Insulation, Field::LoopImpedance, Field::Function, Field::RcdTime]);
}

/// Data the project does not give are missing, not assumed (no C characteristic, no
/// 30 mA, no limit resting on them).
#[test]
fn missing_data_are_marked_not_assumed() {
    let mut k = installation();
    k.prj.parts.iter_mut().find(|p| p.number.contains("Q1")).unwrap().tech_data.clear();
    let cs = circuits_of(&k.prj);
    let f1 = by(&cs, "-F1");
    assert_eq!(f1.rcd, Some(Rcd { device: "-Q1".into(), kind: None, rated: None }));
    // The breaker's own limit remains: U0 / Ia with Ia = 5 × 16 A (B), two thirds of it.
    let u0 = k.prj.electrical.supply.u0();
    assert!((f1.zs_max.unwrap() - LOOP_FACTOR * u0 / 80.0).abs() < 0.01);
    let f2 = by(&cs, "-F2");
    assert_eq!((f2.characteristic.as_ref(), f2.rated, f2.zs_max, f2.description.as_ref(), f2.cross_section.as_ref()), (None, None, None, None, None));
    let f3 = by(&cs, "-F3");
    assert_eq!((f3.characteristic.as_ref(), f3.rated, f3.zs_max), (None, Some(16.0), None));
    let missing = tr("missing");
    assert_eq!(f3.device_text(), format!("{missing} 16 A"));
    assert_eq!(f1.rcd_text(), format!("-Q1 {missing} {missing}"));
    assert_eq!(f3.rcd_text(), "–");
    let t = report(&cs);
    let row = |tag: &str| &t.rows[cs.iter().position(|c| c.circuit == tag).unwrap()];
    assert_eq!(row("-F2")[1], missing);
    assert_eq!(row("-F2")[4], missing);
    assert_eq!(row("-F3")[10], missing);
}

#[test]
fn measurements_are_checked_against_the_limits() {
    let mut k = installation();
    for (f, v) in [(Field::Continuity, "0,2"), (Field::Insulation, "> 500"), (Field::LoopImpedance, "0.85"), (Field::Function, "OK")] {
        assert!(set(&mut k.prj, "-F1", f, v).unwrap());
    }
    assert_eq!(by(&circuits_of(&k.prj), "-F1").verdict, Verdict::Incomplete, "the RCD test is missing");
    set(&mut k.prj, "-F1", Field::RcdTime, "24").unwrap();
    set(&mut k.prj, "-F1", Field::RcdCurrent, "21 mA").unwrap();
    let f1 = by(&circuits_of(&k.prj), "-F1").clone();
    assert_eq!((f1.verdict, f1.failed.as_slice()), (Verdict::Passed, &[][..]));
    set(&mut k.prj, "-F1", Field::Insulation, "0.4").unwrap();
    set(&mut k.prj, "-F1", Field::RcdTime, "320").unwrap();
    set(&mut k.prj, "-F1", Field::RcdCurrent, "12").unwrap();
    let f1 = by(&circuits_of(&k.prj), "-F1").clone();
    assert_eq!(f1.verdict, Verdict::Failed);
    assert_eq!(f1.failed, [Field::Insulation, Field::RcdCurrent, Field::RcdTime]);
    // The loop impedance against the breaker alone (-F3 has no RCD, a B characteristic now).
    k.prj.parts.iter_mut().find(|p| p.number.contains("F3")).unwrap().tech_data = "Characteristic: B".into();
    set(&mut k.prj, "-F3", Field::LoopImpedance, "2.5").unwrap();
    let f3 = by(&circuits_of(&k.prj), "-F3").clone();
    assert_eq!(f3.failed, [Field::LoopImpedance]);
    set(&mut k.prj, "-F3", Field::LoopImpedance, "1.5").unwrap();
    assert_eq!(by(&circuits_of(&k.prj), "-F3").verdict, Verdict::Incomplete);
}

#[test]
fn values_are_checked_when_entered() {
    let mut prj = Project::default();
    assert_eq!(set(&mut prj, "-F1", Field::Insulation, ""), Ok(false), "clearing nothing adds no record");
    assert!(prj.inspection.is_empty());
    assert!(set(&mut prj, "-F1", Field::Insulation, "many").is_err());
    assert!(set(&mut prj, "-F1", Field::Function, "maybe").is_err());
    assert_eq!(set(&mut prj, "-F1", Field::Function, "not ok"), Ok(true));
    assert_eq!(prj.inspection.result("-F1").unwrap().function, FUNCTION_FAILED);
    assert_eq!(set(&mut prj, "-F1", Field::Function, "failed"), Ok(false));
    assert_eq!(set(&mut prj, "-F1", Field::Remark, "Socket\\nloose"), Ok(true));
    let r = prj.inspection.result("-F1").unwrap();
    assert_eq!((r.remark.as_str(), r.rev), ("Socket\nloose", 2));
    assert_eq!(Field::Function.display(r), tr("Not OK"));
    assert_eq!(number(">500"), Some(500.0));
    assert_eq!(number("0,35 Ω"), Some(0.35));
    assert_eq!(number("-1"), None);
    assert_eq!(Field::from_key("Loop impedance Zs (Ω)"), Some(Field::LoopImpedance));
    assert_eq!(Field::from_key("RCD_TIME"), Some(Field::RcdTime));
}

/// Two users enter results at the same time: each circuit keeps the record with more
/// edits, new records of both stay.
#[test]
fn results_merge_record_by_record() {
    let mut base = Project::default();
    set(&mut base, "-F1", Field::Insulation, "300").unwrap();
    let (mut theirs, mut mine) = (base.clone(), base.clone());
    set(&mut theirs, "-F2", Field::Insulation, "200").unwrap();
    set(&mut mine, "-F1", Field::LoopImpedance, "0.5").unwrap();
    set(&mut mine, "-F3", Field::Insulation, "100").unwrap();
    let merged = merge(&theirs.inspection, &mine.inspection);
    let circuits: Vec<&str> = merged.results.iter().map(|r| r.circuit.as_str()).collect();
    assert_eq!(circuits, ["-F1", "-F2", "-F3"]);
    assert_eq!(merged.result("-F1").unwrap().loop_impedance, "0.5");
    assert_eq!(merge(&mine.inspection, &theirs.inspection).result("-F1").unwrap().loop_impedance, "0.5");
}

#[test]
fn the_report_lists_the_results() {
    let mut k = installation();
    set(&mut k.prj, "-F1", Field::Tester, "A. Smith").unwrap();
    set(&mut k.prj, "-F1", Field::Function, "ok").unwrap();
    let cs = circuits_of(&k.prj);
    let t = report(&cs);
    assert_eq!(t.rows.len(), 3);
    assert_eq!(t.header_keys()[0], "Circuit");
    let f1 = &t.rows[0];
    assert_eq!(&f1[..4], ["-F1", "Heating", "L1", "-W1 NYM-J 3x2.5"]);
    assert_eq!(f1[5], "B 16 A");
    assert_eq!(f1[6], "-Q1 A 30 mA");
    assert_eq!(f1[13], tr("OK"));
    assert_eq!(f1[14], Verdict::Incomplete.title());
    assert_eq!(f1[15], "A. Smith");
    assert_eq!(t.targets[0], Some(cs[0].target));
}
