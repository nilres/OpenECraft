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

fn report_of(prj: &Project) -> Report {
    let an = Analysis::build(prj);
    Report::of(prj, &an.netlist, &an.connections)
}

fn circuits_of(prj: &Project) -> Vec<Circuit> {
    report_of(prj).circuits
}

fn by<'a>(cs: &'a [Circuit], tag: &str) -> &'a Circuit {
    cs.iter().find(|c| c.circuit == tag).unwrap()
}

fn rcd_part(k: &mut Kit) -> &mut Part {
    k.prj.parts.iter_mut().find(|p| p.number.contains("Q1")).unwrap()
}

/// The results that pass in -F1 (B16 behind a 30 mA RCD).
fn passing(prj: &mut Project) {
    for (f, v) in [(Field::Continuity, "0,2"), (Field::InsulationVoltage, "500"), (Field::Insulation, "> 500"), (Field::LoopImpedance, "0.85"), (Field::Polarity, "OK"), (Field::Function, "OK"), (Field::RcdTime, "24")] {
        set(prj, "-F1", f, v).unwrap();
    }
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
    assert_eq!(f1.required(), [Field::Continuity, Field::InsulationVoltage, Field::Insulation, Field::LoopImpedance, Field::Polarity, Field::Function, Field::RcdTime]);
    assert!(!f1.fields().contains(&Field::PhaseSequence), "one phase");
    assert_eq!(f1.limit(Field::RcdTime).map(|l| (l.min, l.max)), Some((None, Some(300.0))));
    assert_eq!(f1.limit(Field::RcdCurrent).map(|l| l.text()), Some("15 – 30".into()));
    assert_eq!(f1.limit(Field::InsulationVoltage).map(|l| l.text()), Some("≥ 500".into()));
    assert!(by(&cs, "-F3").fields().iter().all(|f| !f.is_rcd()), "no RCD tests without an RCD");
}

/// Data the project does not give are missing, not assumed (no C characteristic, no
/// 30 mA, no limit resting on them).
#[test]
fn missing_data_are_marked_not_assumed() {
    let mut k = installation();
    rcd_part(&mut k).tech_data.clear();
    let cs = circuits_of(&k.prj);
    let f1 = by(&cs, "-F1");
    assert_eq!(f1.rcd, Some(Rcd { device: "-Q1".into(), kind: None, rated: None }));
    assert!(f1.limit(Field::RcdCurrent).is_none());
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
    let t = circuit_table(&cs);
    let row = |tag: &str| &t.rows[cs.iter().position(|c| c.circuit == tag).unwrap()];
    assert_eq!(row("-F2")[1], missing);
    assert_eq!(row("-F2")[4], missing);
    assert_eq!(row("-F3")[7], missing);
}

#[test]
fn measurements_are_checked_against_the_limits() {
    let mut k = installation();
    passing(&mut k.prj);
    set(&mut k.prj, "-F1", Field::RcdTime, "").unwrap();
    assert_eq!(by(&circuits_of(&k.prj), "-F1").verdict, Verdict::Incomplete, "the RCD test is missing");
    set(&mut k.prj, "-F1", Field::RcdTime, "24").unwrap();
    set(&mut k.prj, "-F1", Field::RcdCurrent, "21 mA").unwrap();
    let f1 = by(&circuits_of(&k.prj), "-F1").clone();
    assert_eq!((f1.verdict, f1.failed.as_slice()), (Verdict::Passed, &[][..]));
    set(&mut k.prj, "-F1", Field::InsulationVoltage, "250").unwrap();
    set(&mut k.prj, "-F1", Field::Insulation, "0.4").unwrap();
    set(&mut k.prj, "-F1", Field::RcdTime, "320").unwrap();
    set(&mut k.prj, "-F1", Field::RcdTime5, "45").unwrap();
    set(&mut k.prj, "-F1", Field::RcdVoltage, "51").unwrap();
    set(&mut k.prj, "-F1", Field::RcdCurrent, "12").unwrap();
    let f1 = by(&circuits_of(&k.prj), "-F1").clone();
    assert_eq!(f1.verdict, Verdict::Failed);
    assert_eq!(f1.failed, [Field::InsulationVoltage, Field::Insulation, Field::RcdCurrent, Field::RcdTime, Field::RcdTime5, Field::RcdVoltage]);
    // The loop impedance against the breaker alone (-F3 has no RCD, a B characteristic now).
    k.prj.parts.iter_mut().find(|p| p.number.contains("F3")).unwrap().tech_data = "Characteristic: B".into();
    set(&mut k.prj, "-F3", Field::LoopImpedance, "2.5").unwrap();
    let f3 = by(&circuits_of(&k.prj), "-F3").clone();
    assert_eq!(f3.failed, [Field::LoopImpedance]);
    set(&mut k.prj, "-F3", Field::LoopImpedance, "1.5").unwrap();
    assert_eq!(by(&circuits_of(&k.prj), "-F3").verdict, Verdict::Incomplete);
}

/// Above 500 V the insulation is measured at 1000 V.
#[test]
fn the_insulation_test_voltage_follows_the_supply() {
    let mut k = installation();
    assert_eq!(insulation_voltage(&k.prj), 500.0);
    k.prj.electrical.supply.voltage = 690.0;
    assert_eq!(insulation_voltage(&k.prj), 1000.0);
    set(&mut k.prj, "-F1", Field::InsulationVoltage, "500").unwrap();
    assert_eq!(by(&circuits_of(&k.prj), "-F1").failed, [Field::InsulationVoltage]);
}

/// Selective RCDs trip later: 130 – 500 ms at IΔn, 50 – 150 ms at 5 × IΔn.
#[test]
fn selective_rcds_have_their_own_times() {
    let mut k = installation();
    rcd_part(&mut k).tech_data = "Rated residual current: 300 mA\nResidual current type: A S".into();
    set(&mut k.prj, "-F1", Field::RcdTime, "350").unwrap();
    set(&mut k.prj, "-F1", Field::RcdTime5, "100").unwrap();
    let f1 = by(&circuits_of(&k.prj), "-F1").clone();
    assert!(f1.rcd.as_ref().unwrap().selective());
    assert!(f1.failed.is_empty(), "{:?}", f1.failed);
    set(&mut k.prj, "-F1", Field::RcdTime, "90").unwrap();
    assert_eq!(by(&circuits_of(&k.prj), "-F1").failed, [Field::RcdTime], "too fast for type S");
    for (kind, s) in [("AS", true), ("B+ S", true), ("S", true), ("A", false), ("AC", false), ("F", false)] {
        assert_eq!(Rcd { device: String::new(), kind: Some(kind.into()), rated: None }.selective(), s, "{kind}");
    }
}

/// TT: RA × IΔn ≤ 50 V, not Zs × IΔn ≤ U0; the earth electrode is checked in the header.
#[test]
fn tt_systems_limit_the_earth_electrode_to_the_touch_voltage() {
    let mut k = installation();
    k.prj.electrical.supply.earthing = Earthing::Tt;
    let r = report_of(&k.prj);
    let f1 = by(&r.circuits, "-F1");
    assert!((f1.zs_max.unwrap() - 50.0 / 0.03).abs() < 0.5, "{:?}", f1.zs_max);
    assert!((r.limit("earth_electrode").unwrap().max.unwrap() - 50.0 / 0.03).abs() < 0.5);
    set_header(&mut k.prj, "earth_electrode", "1800").unwrap();
    let r = report_of(&k.prj);
    assert_eq!((r.verdict, r.failed.as_slice()), (Verdict::Failed, &["earth_electrode".to_string()][..]));
    k.prj.electrical.supply.earthing = Earthing::TnS;
    assert_eq!(report_of(&k.prj).limit("earth_electrode"), None);
}

#[test]
fn values_are_checked_when_entered() {
    let mut prj = Project::default();
    assert_eq!(set(&mut prj, "-F1", Field::Insulation, ""), Ok(false), "clearing nothing adds no record");
    assert!(prj.inspection.is_empty());
    assert!(set(&mut prj, "-F1", Field::Insulation, "many").is_err());
    assert!(set(&mut prj, "-F1", Field::Function, "maybe").is_err());
    assert!(set(&mut prj, "-F1", Field::Function, "n/a").is_err(), "a function test is done");
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
    assert_eq!(Field::from_key("RCD_TIME_5"), Some(Field::RcdTime5));
    assert!(set_header(&mut prj, "ze", "low").is_err());
    assert!(set_header(&mut prj, "check.protection", "perhaps").is_err());
    assert!(set_header(&mut prj, "check.rocket", "ok").is_err());
    assert!(set_header(&mut prj, "colour", "red").is_err());
    assert_eq!(set_header(&mut prj, "check.surge", "Not applicable"), Ok(true));
    assert_eq!(prj.inspection.header.checks["surge"], NOT_APPLICABLE);
    assert_eq!(set_header(&mut prj, "Installation site", "Hall 3"), Ok(true), "by its title too");
    assert_eq!(prj.inspection.header.site, "Hall 3");
}

/// The report passes when every circuit passes, the visual inspection is answered
/// and an inspector named; a change after signing takes the signature away.
#[test]
fn the_report_has_a_verdict_and_a_signature() {
    let mut k = installation();
    assert_eq!(report_of(&k.prj).verdict, Verdict::NotTested);
    for c in ["-F2", "-F3"] {
        k.prj.devices.remove(c);
    }
    k.prj.pages[0].symbols.retain(|s| s.tag != "-F2" && s.tag != "-F3");
    passing(&mut k.prj);
    assert_eq!(report_of(&k.prj).verdict, Verdict::Incomplete, "the visual inspection is missing");
    for (key, _) in CHECKS {
        set_header(&mut k.prj, &format!("check.{key}"), if key == "surge" { "n/a" } else { "ok" }).unwrap();
    }
    assert_eq!(report_of(&k.prj).verdict, Verdict::Incomplete, "no inspector");
    set_header(&mut k.prj, "inspector", "A. Smith").unwrap();
    assert_eq!(report_of(&k.prj).verdict, Verdict::Passed);
    // Signed together with a change: the signature comes last and stays.
    set_header_all(&mut k.prj, [("signed_by", "A. Smith"), ("signed_on", "2026-11-17"), ("site", "Hall 3")]).unwrap();
    assert_eq!((k.prj.inspection.header.signed_by.as_str(), k.prj.inspection.header.site.as_str()), ("A. Smith", "Hall 3"));
    set(&mut k.prj, "-F1", Field::Remark, "Checked again").unwrap();
    assert!(k.prj.inspection.header.signed_by.is_empty() && k.prj.inspection.header.signed_on.is_empty());
    set_header(&mut k.prj, "check.fire", "Not OK").unwrap();
    let r = report_of(&k.prj);
    assert_eq!((r.verdict, r.failed.as_slice()), (Verdict::Failed, &["check.fire".to_string()][..]));
    set_header(&mut k.prj, "check.fire", "OK").unwrap();
    // An own item takes part like the others.
    set_header(&mut k.prj, "own.1.title", "Emergency lighting").unwrap();
    assert_eq!(report_of(&k.prj).verdict, Verdict::Incomplete, "the own item is not answered");
    set_header(&mut k.prj, "own.1", "Not OK").unwrap();
    let r = report_of(&k.prj);
    assert_eq!((r.failed.as_slice(), r.checks.last().map(|c| c.title.as_str())), (&["own.1".to_string()][..], Some("Emergency lighting")));
    let t = summary_table(&r);
    assert_eq!(t.rows.last().unwrap()[1], Verdict::Failed.title());
    assert!(t.rows.iter().any(|row| row[0] == tr("Inspector") && row[1] == "A. Smith"));
}

/// Two users enter results at the same time: each circuit and the header keep the
/// record with more edits, new records of both stay.
#[test]
fn results_merge_record_by_record() {
    let mut base = Project::default();
    set(&mut base, "-F1", Field::Insulation, "300").unwrap();
    let (mut theirs, mut mine) = (base.clone(), base.clone());
    set(&mut theirs, "-F2", Field::Insulation, "200").unwrap();
    set_header(&mut theirs, "client", "ACME").unwrap();
    set(&mut mine, "-F1", Field::LoopImpedance, "0.5").unwrap();
    set(&mut mine, "-F3", Field::Insulation, "100").unwrap();
    let merged = merge(&theirs.inspection, &mine.inspection);
    let circuits: Vec<&str> = merged.results.iter().map(|r| r.circuit.as_str()).collect();
    assert_eq!(circuits, ["-F1", "-F2", "-F3"]);
    assert_eq!(merged.result("-F1").unwrap().loop_impedance, "0.5");
    assert_eq!(merged.header.client, "ACME");
    assert_eq!(merge(&mine.inspection, &theirs.inspection).result("-F1").unwrap().loop_impedance, "0.5");
}

#[test]
fn the_report_lists_the_results() {
    let mut k = installation();
    set(&mut k.prj, "-F1", Field::Tester, "A. Smith").unwrap();
    set(&mut k.prj, "-F1", Field::Function, "ok").unwrap();
    let cs = circuits_of(&k.prj);
    let t = circuit_table(&cs);
    assert_eq!(t.rows.len(), 3);
    assert_eq!(t.header_keys()[0], "Circuit");
    let col = |name: &str| t.header_keys().iter().position(|h| *h == name).unwrap();
    let f1 = &t.rows[0];
    assert_eq!(&f1[..4], ["-F1", "Heating", "L1", "-W1 NYM-J 3x2.5"]);
    assert_eq!(f1[5], "B 16 A");
    assert_eq!(f1[6], "-Q1 A 30 mA");
    assert_eq!(f1[col("Function test")], tr("OK"));
    assert_eq!(f1[col("Result")], Verdict::Incomplete.title());
    assert_eq!(f1[col("Tested by")], "A. Smith");
    assert_eq!(t.rows[2][col("RCD trip time (ms)")], "–", "-F3 has no RCD");
    assert_eq!(t.targets[0], Some(cs[0].target));
}

/// Polarity for every circuit, the phase sequence for three-phase ones, the voltage
/// drop against the limit of the rule sets.
#[test]
fn polarity_phase_sequence_and_voltage_drop() {
    let mut k = Kit::new();
    let pots: Vec<u64> = ["L1", "L2", "L3"].iter().enumerate().map(|(i, l)| k.sym(&format!("pot_{}", l.to_lowercase()), 100.0 + 8.0 * i as f32, 20.0, "", l)).collect();
    let q1 = k.sym("mps3", 100.0, 40.0, "-Q1", "");
    for (i, p) in pots.iter().enumerate() {
        k.wire((*p, ""), (q1, ["1", "3", "5"][i]));
    }
    k.part("-Q1", FnKind::MotorProtection, "1 2 3 4 5 6", |p| p.current = "10 A".into());
    let q1 = by(&circuits_of(&k.prj), "-Q1").clone();
    assert!(q1.multi_phase() && q1.required().contains(&Field::PhaseSequence));
    assert_eq!(q1.limit(Field::VoltageDrop).and_then(|l| l.max), Some(crate::sizing::VOLTAGE_DROP_LIMIT));
    set(&mut k.prj, "-Q1", Field::PhaseSequence, "Not OK").unwrap();
    set(&mut k.prj, "-Q1", Field::Polarity, "OK").unwrap();
    set(&mut k.prj, "-Q1", Field::VoltageDrop, "6.5").unwrap();
    assert_eq!(by(&circuits_of(&k.prj), "-Q1").failed, [Field::VoltageDrop, Field::PhaseSequence]);
    assert!(set(&mut k.prj, "-Q1", Field::Polarity, "n/a").is_err(), "polarity is tested");
}

/// IT: the second fault at U / 2 (no neutral) or U0 / 2, the insulation monitoring
/// device and the first fault current against UL / RA.
#[test]
fn it_systems_test_the_monitoring_and_the_first_fault() {
    let mut k = installation();
    rcd_part(&mut k).tech_data.clear();
    k.prj.electrical.supply.earthing = Earthing::It;
    let r = report_of(&k.prj);
    assert!(r.it && r.header_fields().contains(&HeaderField::Imd));
    // B16: Ia = 80 A, U = 400 V: ⅔ × 200 V / 80 A.
    let f1 = by(&r.circuits, "-F1");
    assert!((f1.zs_max.unwrap() - LOOP_FACTOR * 200.0 / 80.0).abs() < 0.01, "{:?}", f1.zs_max);
    k.prj.electrical.supply.neutral = true;
    let u0 = k.prj.electrical.supply.u0();
    assert!((by(&circuits_of(&k.prj), "-F1").zs_max.unwrap() - LOOP_FACTOR * u0 / 2.0 / 80.0).abs() < 0.01);
    set_header(&mut k.prj, "earth_electrode", "10").unwrap();
    set_header(&mut k.prj, "first_fault", "6").unwrap();
    set_header(&mut k.prj, "imd", "failed").unwrap();
    let r = report_of(&k.prj);
    assert_eq!(r.limit("first_fault").and_then(|l| l.max), Some(5.0));
    assert_eq!(r.failed, ["first_fault", "imd"]);
    k.prj.electrical.supply.earthing = Earthing::TnS;
    let r = report_of(&k.prj);
    assert!(!r.header_fields().contains(&HeaderField::Imd) && r.failed.is_empty(), "no IT tests in TN systems");
}

/// Floors and walls of non-conducting locations: at least 50 kΩ up to 500 V.
#[test]
fn insulating_floors_and_walls() {
    let mut k = installation();
    assert_eq!(report_of(&k.prj).limit("floor_insulation").and_then(|l| l.min), Some(50.0));
    set_header(&mut k.prj, "floor_insulation", "40").unwrap();
    assert_eq!(report_of(&k.prj).failed, ["floor_insulation"]);
    k.prj.electrical.supply.voltage = 690.0;
    assert_eq!(report_of(&k.prj).limit("floor_insulation").and_then(|l| l.min), Some(100.0));
}

/// Power supplies and transformers are the sources of SELV, PELV and separated
/// circuits: the measure decides the limits, SELV and PELV also the highest voltage.
#[test]
fn sources_of_selv_pelv_and_separation() {
    let mut k = installation();
    k.sym("psu", 300.0, 40.0, "-T1", "");
    k.part("-T1", FnKind::PowerSupply, "L N + -", |p| {
        p.voltage = "230 V / 24 V DC".into();
        p.current = "5 A".into();
    });
    k.sym("trafo", 360.0, 40.0, "-T2", "");
    k.part("-T2", FnKind::Transformer, "1 2 3 4", |p| p.voltage = "400 V / 230 V".into());
    let ss = sources(&k.prj);
    assert_eq!(ss.iter().map(|s| (s.device.as_str(), s.voltage, s.transformer)).collect::<Vec<_>>(), [("-T1", Some(24.0), false), ("-T2", Some(230.0), true)]);
    assert_eq!(ss[0].verdict, Verdict::NotTested);
    set_source(&mut k.prj, "-T1", SourceField::Measure, "SELV").unwrap();
    set_source(&mut k.prj, "-T1", SourceField::InsulationVoltage, "250").unwrap();
    set_source(&mut k.prj, "-T1", SourceField::Insulation, "0.8").unwrap();
    set_source(&mut k.prj, "-T2", SourceField::Measure, "pelv").unwrap();
    let ss = sources(&k.prj);
    assert_eq!((ss[0].verdict, ss[0].result.measure.as_str()), (Verdict::Passed, "selv"));
    assert_eq!((ss[1].verdict, ss[1].failed.as_slice()), (Verdict::Failed, &["measure".to_string()][..]), "230 V is no PELV");
    set_source(&mut k.prj, "-T2", SourceField::Measure, "Electrical separation").unwrap();
    set_source(&mut k.prj, "-T2", SourceField::InsulationVoltage, "250").unwrap();
    set_source(&mut k.prj, "-T2", SourceField::Insulation, "0.8").unwrap();
    assert_eq!(sources(&k.prj)[1].failed, ["insulation_voltage", "insulation"], "separation is tested like the mains");
    set_source(&mut k.prj, "-T2", SourceField::Measure, "none").unwrap();
    assert_eq!(sources(&k.prj)[1].verdict, Verdict::Passed, "no protective measure of its own");
    assert!(set_source(&mut k.prj, "-T2", SourceField::Measure, "FELV").is_err());
    let r = report_of(&k.prj);
    assert_eq!(r.sources.len(), 2);
    assert_eq!(source_table(&r.sources).rows[0][3], tr("SELV"));
}

/// Own items: added with the next number, renamed, removed with an empty text.
#[test]
fn own_items_of_the_visual_inspection() {
    let mut prj = Project::default();
    assert!(set_header(&mut prj, "own.2.title", "Gap").is_err(), "numbers follow on");
    assert_eq!(set_header(&mut prj, "own.1.title", "Emergency lighting"), Ok(true));
    assert_eq!(set_header(&mut prj, "own.2.title", "Lightning protection"), Ok(true));
    assert_eq!(set_header(&mut prj, "own.2", "n/a"), Ok(true));
    assert!(set_header(&mut prj, "own.3", "ok").is_err());
    assert_eq!(set_header(&mut prj, "own.1.title", ""), Ok(true));
    let own = &prj.inspection.header.own_checks;
    assert_eq!((own.len(), own[0].title.as_str(), own[0].answer.as_str()), (1, "Lightning protection", NOT_APPLICABLE));
}

/// Limits of another edition: set per project, back to the default when cleared.
#[test]
fn limits_are_set_per_project() {
    let mut k = installation();
    set(&mut k.prj, "-F1", Field::Insulation, "0.8").unwrap();
    assert_eq!(by(&circuits_of(&k.prj), "-F1").failed, [Field::Insulation]);
    assert_eq!(set_header(&mut k.prj, "limit.min_insulation", "0,5"), Ok(true));
    assert!(by(&circuits_of(&k.prj), "-F1").failed.is_empty());
    let r = report_of(&k.prj);
    let s = r.settings.iter().find(|s| s.key == "min_insulation").unwrap();
    assert_eq!((s.value, s.default), (0.5, 1.0));
    assert!(set_header(&mut k.prj, "limit.min_insulation", "lots").is_err());
    assert!(set_header(&mut k.prj, "limit.rocket", "1").is_err());
    assert_eq!(set_header(&mut k.prj, "limit.min_insulation", "1"), Ok(true), "the default is no own value");
    assert!(k.prj.inspection.header.limits.is_empty());
    set_header(&mut k.prj, "limit.rcd_time", "200").unwrap();
    assert_eq!(by(&circuits_of(&k.prj), "-F1").limit(Field::RcdTime).and_then(|l| l.max), Some(200.0));
    assert_eq!(limit_text(LOOP_FACTOR), "0.667");
}

/// Insulation between each pair of conductors, all against the same limit.
#[test]
fn insulation_between_the_conductors() {
    let mut k = installation();
    set(&mut k.prj, "-F1", Field::InsulationNPe, "0.6").unwrap();
    set(&mut k.prj, "-F1", Field::InsulationLN, "> 500").unwrap();
    assert_eq!(by(&circuits_of(&k.prj), "-F1").failed, [Field::InsulationNPe]);
    assert!(!by(&circuits_of(&k.prj), "-F1").required().contains(&Field::InsulationNPe), "with the live conductors connected together L–PE is enough");
}

/// The protective conductor against the resistance of the cable: a hint, no failure.
#[test]
fn the_protective_conductor_against_its_calculated_resistance() {
    let mut k = installation();
    // -W1: 20 m of 2.5 mm² copper.
    let f1 = by(&circuits_of(&k.prj), "-F1").clone();
    let r = f1.continuity_calculated.unwrap();
    assert!((r - k.prj.electrical.conditions.material.rho20() * 20.0 / 2.5).abs() < 1e-4, "{r}");
    set(&mut k.prj, "-F1", Field::Continuity, "0.18").unwrap();
    assert!(by(&circuits_of(&k.prj), "-F1").notes.is_empty());
    set(&mut k.prj, "-F1", Field::Continuity, "0.9").unwrap();
    let f1 = by(&circuits_of(&k.prj), "-F1").clone();
    assert_eq!((f1.notes.as_slice(), f1.failed.as_slice()), (&[Field::Continuity][..], &[][..]));
    assert_eq!(by(&circuits_of(&k.prj), "-F3").continuity_calculated, None, "no cable");
}

/// The fault current measured at the origin against the breaking capacities: a device
/// fed from the origin fails.
#[test]
fn breaking_capacities_against_the_measured_fault_current() {
    let mut k = installation();
    for p in k.prj.parts.iter_mut().filter(|p| p.number.contains("F1") || p.number.contains("F3")) {
        p.tech_data = format!("{}\nBreaking capacity: 6 kA", p.tech_data).trim().into();
    }
    set_header(&mut k.prj, "ik", "4.5").unwrap();
    let r = report_of(&k.prj);
    assert!(r.failed.is_empty() && r.notes.is_empty());
    set_header(&mut k.prj, "ik", "8").unwrap();
    let r = report_of(&k.prj);
    assert!(r.failed.contains(&"breaking.-F3".to_string()), "{:?}", r.failed);
    assert_eq!(r.verdict, Verdict::Failed);
    assert!(summary_table(&r).rows.iter().any(|row| row[0].contains("-F3") && row[1] == "6 kA"));
}

/// Own items changed together: numbered as loaded, a new one with its answer, a removed
/// one moving the others up.
#[test]
fn own_items_change_together() {
    let mut prj = Project::default();
    set_header_all(&mut prj, [("own.1.title", "A"), ("own.1", "ok"), ("own.2.title", "B")]).unwrap();
    set_header_all(&mut prj, [("own.1.title", ""), ("own.2", "failed"), ("own.3.title", "C"), ("own.3", "n/a")]).unwrap();
    let own: Vec<(&str, &str)> = prj.inspection.header.own_checks.iter().map(|c| (c.title.as_str(), c.answer.as_str())).collect();
    assert_eq!(own, [("B", FUNCTION_FAILED), ("C", NOT_APPLICABLE)]);
}

#[test]
fn the_forms_show_each_field_of_the_report_data_once() {
    // The editor and the web page show these groups; the signature is set by signing.
    let shown: Vec<HeaderField> = REPORT_FIELDS.into_iter().chain(ORIGIN_FIELDS).chain(IT_FIELDS).collect();
    for f in HeaderField::ALL {
        let signature = matches!(f, HeaderField::SignedBy | HeaderField::SignedOn);
        assert_eq!(shown.iter().filter(|x| **x == f).count(), usize::from(!signature), "{f:?}");
    }
    assert!(IT_FIELDS.iter().all(|f| f.is_it()) && !REPORT_FIELDS.iter().chain(&ORIGIN_FIELDS).any(|f| f.is_it()));
    assert!(ORIGIN_FIELDS.iter().all(|f| f.is_measurement()));
}

#[test]
fn the_workbook_has_the_summary_the_circuits_and_the_sources() {
    let k = installation();
    let r = report_of(&k.prj);
    let sheets = crate::xlsx::read(&workbook(&r)).unwrap();
    let names: Vec<&str> = sheets.iter().map(|s| s.0.as_str()).collect();
    assert_eq!(names, ["Summary", "Circuits", "SELV, PELV and separation"]);
    // A header row and a row per circuit.
    assert_eq!(sheets[1].1.len(), r.circuits.len() + 1);
}
