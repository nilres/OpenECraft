use super::*;

#[test]
fn every_library_symbol_has_a_definition() {
    let prj = Project::default();
    for sym in symbols::library() {
        let d = default_for_symbol(&prj, sym.key);
        assert!(!d.title().is_empty());
        // Symbols of a known function kind get a definition of that kind.
        if let Some(kind) = crate::parts::kind_of_symbol(sym.key) {
            assert_eq!(d.kind, kind, "{}", sym.key);
        }
    }
    assert_eq!(default_for_symbol(&prj, "pot_l1").key, "fn.potential");
    assert_eq!(default_for_symbol(&prj, "box2").key, "fn.black_box");
    assert_eq!(default_for_symbol(&prj, "no-such-symbol").key, "fn.generic");
}

#[test]
fn builtin_definitions_are_consistent() {
    let mut keys = std::collections::HashSet::new();
    for d in builtin() {
        assert!(keys.insert(d.key.as_str()), "duplicate {}", d.key);
        assert!(d.symbol.is_empty() || crate::symbols::builtin(&d.symbol).is_some(), "{}", d.key);
    }
    for c in FnCategory::ALL {
        assert!(!c.title().is_empty());
        assert!(!c.groups().is_empty());
    }
}

#[test]
fn project_definitions_are_assigned_to_symbols() {
    let mut prj = Project::default();
    prj.pages[0].symbols.push(SymbolInst { id: 7, sym: "coil".into(), tag: "-K1".into(), pins: vec!["A1".into(), "A2".into()], ..Default::default() });
    let mut d = derive(&prj, builtin_def("fn.coil").unwrap());
    assert_eq!(d.key, "user.coil");
    d.name = "Coupling relay 24 V".into();
    d.pins = "X1 X2".into();
    d.scheme = "Part data".into();
    d.defaults.insert("device.coil_voltage".into(), "24".into());
    define(&mut prj, d.clone()).unwrap();
    assert_eq!(derive(&prj, &d).key, "user.coil_2");
    assert!(define(&mut prj, FunctionDef { key: "fn.coil".into(), ..d.clone() }).is_err());
    assert!(define(&mut prj, FunctionDef { symbol: "nope".into(), ..d.clone() }).is_err());
    let mut bad = d.clone();
    bad.defaults.insert("no.such".into(), "1".into());
    assert!(define(&mut prj, bad).is_err());

    assign(&mut prj, 7, "user.coil").unwrap();
    let s = &prj.pages[0].symbols[0];
    assert_eq!(s.function_def, "user.coil");
    assert_eq!(s.pins, vec!["X1", "X2"]);
    assert_eq!(s.scheme, "Part data");
    assert_eq!(s.props.len(), 3);
    assert_eq!(of_symbol(&prj, s).unwrap().title(), "Coupling relay 24 V");
    assert!(fitting(&prj, s).iter().any(|f| f.key == "fn.timer_on"));
    assert!(!fitting(&prj, s).iter().any(|f| f.key == "fn.motor3"));
    assert_eq!(usage(&prj)["user.coil"], 1);

    // The symbol's own built-in definition is stored as none.
    assign(&mut prj, 7, "fn.coil").unwrap();
    assert!(prj.pages[0].symbols[0].function_def.is_empty());
    assign(&mut prj, 7, "user.coil").unwrap();
    assert_eq!(remove(&mut prj, "user.coil"), 1);
    assert_eq!(of_symbol(&prj, &prj.pages[0].symbols[0]).unwrap().key, "fn.coil");
    assert!(assign(&mut prj, 7, "user.coil").is_err());
    assert!(assign(&mut prj, 99, "fn.coil").is_err());
}

#[test]
fn semiconductor_definitions_are_not_passive_components() {
    let category = |key: &str| builtin_def(key).unwrap().category;
    for key in ["fn.diode", "fn.led", "fn.zener_diode", "fn.photodiode", "fn.transistor", "fn.thyristor", "fn.optocoupler", "fn.bridge_rectifier"] {
        assert_eq!(category(key), FnCategory::Semiconductors, "{key}");
    }
    for key in ["fn.resistor", "fn.capacitor", "fn.inductor", "fn.varistor", "fn.ntc"] {
        assert_eq!(category(key), FnCategory::Passive, "{key}");
    }
}
