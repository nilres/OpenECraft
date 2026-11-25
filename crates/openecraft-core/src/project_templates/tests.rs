use super::*;

#[test]
fn built_in_templates() {
    for (name, _) in BUILTIN {
        let t = builtin(name).unwrap_or_else(|| panic!("{name}"));
        assert!(!t.pages.is_empty());
        assert!(!t.potentials.is_empty(), "IEC 60204-1 potentials");
    }
    assert!(builtin("Nothing").is_none());
    let m = builtin("Machine").unwrap();
    assert_eq!(m.pages.iter().filter(|p| p.kind == PageKind::Schematic).count(), 3);
    assert!(m.pages.iter().any(|p| p.kind == PageKind::Cover));
    assert!(m.structures.entry(crate::structure::Aspect::Function, "POWER").is_some(), "identifiers of the structure template");
    let ids: std::collections::BTreeSet<u64> = m.pages.iter().map(|p| p.id).collect();
    assert_eq!(ids.len(), m.pages.len(), "unique page ids");
}

#[test]
fn templates_drop_what_belongs_to_one_project() {
    let mut prj = crate::sample::demo_project();
    prj.pages[1].locked = true;
    prj.wiring_marks.push(Default::default());
    prj.check_settings.accepted.push(Default::default());
    prj.check_settings.disabled.push("emc".into());
    let t = as_template(&prj);
    assert!(t.wiring_marks.is_empty() && t.check_settings.accepted.is_empty() && t.revisions.is_empty());
    assert_eq!(t.check_settings.disabled, ["emc"], "settings stay");
    assert!(t.pages.iter().all(|p| !p.locked));
    let new = from_template(&t, "Line 2", "2026-10-25");
    assert_eq!((new.info.name.as_str(), new.info.date.as_str()), ("Line 2", "2026-10-25"));
    assert_eq!(new.pages.len(), prj.pages.len());
    assert_eq!(new.potentials, prj.potentials);
}

fn given(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn value(t: &Project, pairs: &[(&str, &str)], name: &str) -> String {
    values(&t.template, &given(pairs)).unwrap().into_iter().find(|(n, _)| n == name).map(|(_, v)| v).unwrap_or_default()
}

#[test]
fn bundled_templates_have_no_mistakes() {
    for t in bundled() {
        assert!(problems(&t).is_empty(), "{}: {:?}", name(&t), problems(&t));
        assert!(!t.template.name.is_empty());
        // Opened to be changed, the placeholders of its part numbers are no errors.
        let an = crate::analysis::Analysis::checked(&t);
        let errors: Vec<String> = an.issues.iter().filter(|i| i.severity == crate::issue::Severity::Error).map(|i| i.message()).collect();
        assert!(errors.is_empty(), "{}: {errors:#?}", name(&t));
    }
}

#[test]
fn direct_on_line_is_sized_from_the_motor_power() {
    let t = builtin("Motor starter direct on line").unwrap();
    let at = |kw: &str, n: &str| value(&t, &[("POWER_KW", kw)], n);
    assert_eq!(at("7.5", "CURRENT"), "14.6");
    assert_eq!(at("7.5", "PROTECTION"), "10 - 16 A");
    assert_eq!(at("7.5", "PROTECTION_PART"), "GEN.MPS-10-16A");
    assert_eq!(at("7.5", "CONTACTOR"), "15");
    assert_eq!(at("7.5", "CONTACTOR_PART"), "GEN.CT-18A-24DC");
    assert_eq!(at("7.5", "CROSS_SECTION"), "2.5");
    assert_eq!(at("0.75", "PROTECTION"), "1 - 1.6 A");
    assert_eq!(at("15", "CROSS_SECTION"), "6");
    assert_eq!(at("30", "PROTECTION_PART"), "", "no switch of the catalog");

    let prj = instantiate(&t, "Line 3", "2026-11-24", &given(&[("POWER_KW", "7.5"), ("MOTOR", "Conveyor 2")])).unwrap();
    assert_eq!(prj.info.name, "Line 3");
    assert!(prj.template.is_empty() && prj.pages.iter().all(|p| p.condition.is_empty()));
    let kinds: Vec<PageKind> = prj.pages.iter().map(|p| p.kind).collect();
    assert_eq!(kinds[..3], [PageKind::Cover, PageKind::Schematic, PageKind::Schematic]);
    assert_eq!(kinds[3..], [ReportKind::TerminalDiagram, ReportKind::ConnectionList, ReportKind::DeviceList, ReportKind::BillOfMaterials].map(|k| PageKind::Report(k.into())));
    assert_eq!(prj.pages[1].description, "Power circuit Conveyor 2");
    let motor = prj.pages[1].symbols.iter().find(|s| s.tag == "-M1").unwrap();
    assert_eq!(motor.function_text, "Conveyor 2\n7.5 kW");
    assert!(prj.pages[1].wires.iter().any(|w| w.cross_section == "2.5 mm²"));
    assert_eq!(prj.devices["-K1"].parts[0].number, "GEN.CT-18A-24DC");
    assert!(prj.parts.iter().any(|p| p.number == "GEN.MPS-10-16A"), "parts taken from the catalog");
    assert!(!prj.to_json().contains("{CURRENT}") && !prj.to_json().contains("{MOTOR}"), "every placeholder filled");

    let short = instantiate(&t, "", "", &given(&[("REPORTS", "no")])).unwrap();
    assert_eq!(short.pages.len(), 3, "the report pages left out");
    assert_eq!(short.info.name, tr("Conveyor"), "the project is named after the machine");
}

#[test]
fn started_projects_pass_the_checks() {
    for name in ["Motor starter direct on line", "Motor starter star-delta", "Motor starter reversing"] {
        let t = builtin(name).unwrap();
        for kw in ["0.37", "1.5", "4", "7.5", "11", "15"] {
            let prj = instantiate(&t, "", "", &given(&[("POWER_KW", kw)])).unwrap();
            let an = crate::analysis::Analysis::checked(&prj);
            let problems: Vec<String> =
                an.issues.iter().filter(|i| i.severity <= crate::issue::Severity::Warning).map(|i| format!("{:?}: {}", i.severity, i.message())).collect();
            assert!(problems.is_empty(), "{name} {kw} kW: {problems:#?}");
        }
    }
}

#[test]
fn wrong_values_are_refused() {
    let t = builtin("Motor starter reversing").unwrap();
    let err = |pairs: &[(&str, &str)]| instantiate(&t, "", "", &given(pairs)).unwrap_err();
    assert!(err(&[("SPEED", "3")]).contains("no parameter SPEED"));
    assert!(err(&[("POWER_KW", "much")]).contains("POWER_KW: much is not a number"));
    assert!(err(&[("REPORTS", "maybe")]).contains("REPORTS: maybe is not one of its choices"));
    assert!(err(&[("VOLTAGE", "0")]).contains("CURRENT: Division by zero"));
}

#[test]
fn templates_of_the_user_come_beside_the_bundled_ones() {
    let mut own = save_as(&builtin("Machine").unwrap(), "Machine", "Our machines");
    own.template.parameters.push(TemplateParameter { name: "LINE".into(), default: "1".into(), ..Default::default() });
    let other = save_as(&Project::new_default(), "Air compressor", "");
    let lib = library(vec![other, own]);
    assert_eq!(lib.len(), BUILTIN.len() + 1);
    assert_eq!((name(&lib[1].0).as_str(), lib[1].1), ("Machine", true), "in the place of the bundled one");
    assert_eq!(lib[1].0.template.description, "Our machines");
    assert!(!lib[0].1);
    assert_eq!((name(&lib[BUILTIN.len()].0).as_str(), lib[BUILTIN.len()].1), ("Air compressor", true));
    assert_eq!(find(&lib, "air COMPRESSOR").map(name).as_deref(), Some("Air compressor"));
    assert!(find(&lib, "Nothing").is_none());
    assert_eq!(file_name(" Line: 3/4? "), "Line_ 3_4_.oec");
    assert_eq!(file_name(".."), "template.oec");
}

#[test]
fn mistakes_of_a_definition_are_named() {
    let mut t = builtin("Motor starter direct on line").unwrap();
    t.template.parameters.push(TemplateParameter { name: "2ND".into(), ..Default::default() });
    t.template.parameters.push(TemplateParameter { name: "MOTOR".into(), ..Default::default() });
    t.pages[1].condition = "SIZE=big".into();
    let p = problems(&t);
    assert!(p.iter().any(|m| m.starts_with("2ND is no valid name")), "{p:?}");
    assert!(p.iter().any(|m| m == "MOTOR is defined twice"), "{p:?}");
    assert!(p.iter().any(|m| m.starts_with("Page 2: ") && m.contains("SIZE")), "{p:?}");
    let mut t = builtin("Motor starter direct on line").unwrap();
    t.template.formulas[0].formula = "POWER_KW *".into();
    assert!(problems(&t)[0].starts_with("CURRENT: "), "{:?}", problems(&t));
}

#[test]
fn templates_are_kept_in_a_folder() {
    let dir = std::env::temp_dir().join(format!("oec-templates-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    assert!(read_folder(&dir).0.is_empty(), "no folder yet");
    let t = save_as(&builtin("Motor starter direct on line").unwrap(), "Motor starter direct on line", "With our parts");
    let path = write_to_folder(&dir, &t, false).unwrap();
    assert_eq!(path.file_name().unwrap(), "Motor starter direct on line.oec");
    assert!(write_to_folder(&dir, &t, false).unwrap_err().contains("exists already"));
    write_to_folder(&dir, &t, true).unwrap();
    std::fs::write(dir.join("broken.oec"), "{").unwrap();
    let (found, errors) = read_folder(&dir);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].1.template, t.template, "parameters and formulas kept");
    assert!(errors[0].contains("broken.oec"), "{errors:?}");
    let lib = library(found.into_iter().map(|(_, t)| t).collect());
    assert_eq!(lib.len(), BUILTIN.len(), "in the place of the bundled one");
    assert_eq!(remove_from_folder(&dir, "motor starter direct on line").unwrap(), path);
    assert!(remove_from_folder(&dir, "Motor starter direct on line").unwrap_err().contains("No template"));
    let _ = std::fs::remove_dir_all(&dir);
}
