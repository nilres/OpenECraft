use super::*;

#[test]
fn pins_on_grid_and_keys_unique() {
    let lib = library();
    let mut keys: Vec<_> = lib.iter().map(|d| d.key).collect();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), lib.len());
    // The Insert center lists symbols by name: two alike could not be told apart.
    let mut names: Vec<_> = lib.iter().map(|d| d.name).collect();
    names.sort();
    let twice: Vec<_> = names.windows(2).filter(|w| w[0] == w[1]).map(|w| w[0]).collect();
    assert!(twice.is_empty(), "symbol names used twice: {twice:?}");
    for d in lib {
        for pin in &d.pins {
            assert_eq!(pin.pos.x % crate::geom::GRID, 0.0, "{} pin x off grid", d.key);
            assert_eq!(pin.pos.y % crate::geom::GRID, 0.0, "{} pin y off grid", d.key);
        }
    }
}

#[test]
fn symbol_names_are_translated() {
    let known: std::collections::HashSet<&str> = openecraft_i18n::catalog_entries().into_iter().map(|e| e[0]).collect();
    let missing: Vec<_> = library().iter().map(|d| d.name).chain(Category::ALL.iter().map(|c| c.title())).filter(|n| !known.contains(n)).collect();
    assert!(missing.is_empty(), "missing in the translation catalog: {missing:#?}");
}

#[test]
fn extra_libraries_are_complete() {
    for cat in [Category::BlackBox, Category::SingleLine, Category::Nfpa, Category::Fluid, Category::Process, Category::Installation, Category::Gb, Category::Gost, Category::Jis] {
        let n = library().iter().filter(|d| d.category == cat).count();
        assert!(n >= 4, "{} has only {n} symbols", cat.title());
    }
    // Every function a symbol can have has a title (symbol editor).
    assert_eq!(Func::ALL.iter().map(|f| f.title()).collect::<std::collections::BTreeSet<_>>().len(), Func::ALL.len());
    for d in library().iter().filter(|d| d.category == Category::SingleLine) {
        assert_eq!(d.func, Func::Overview, "{}", d.key);
    }
}

#[test]
fn overview_symbols_refer_to_the_main_function_but_stay_out_of_the_netlist() {
    use crate::model::{Page, PageKind, Project, SymbolInst};
    let mut prj = Project { pages: vec![Page::new(1, "1", "", PageKind::Schematic), Page::new(2, "2", "", PageKind::Schematic)], next_id: 10, ..Default::default() };
    let motor = builtin("motor3").unwrap();
    let overview = builtin("sl_motor").unwrap();
    let inst = |id, sym: &str, x| SymbolInst { id, sym: sym.into(), pos: p(x, 60.0), tag: "-M1".into(), ..Default::default() };
    let mut m = inst(2, "motor3", 100.0);
    m.pins = motor.pins.iter().map(|x| x.label.to_string()).collect();
    prj.pages[0].symbols.push(m);
    prj.pages[1].symbols.push(inst(3, "sl_motor", 60.0));
    assert_eq!(overview.pins.len(), 1);
    let idx = crate::index::Index::build(&prj);
    assert_eq!(idx.devices["-M1"].mains.len(), 1);
    assert_eq!(idx.devices["-M1"].overviews.len(), 1);
    let nl = crate::netlist::Netlist::build(&prj, true);
    // Only the four motor pins are open, the overview has no connection points.
    assert_eq!(nl.unconnected.len(), 4);
    assert!(nl.unconnected.iter().all(|(page, _, _)| *page == 0));
    // The overview shows the cross reference to the motor on page 1.
    let svg = crate::svg::page_svg(&prj, 1);
    assert!(svg.contains("/1.3"), "cross reference missing");
}

#[test]
fn semiconductors_are_not_passive_components() {
    let category = |key: &str| library().iter().find(|d| d.key == key).unwrap().category;
    for key in ["diode", "led", "zener", "photodiode", "npn", "pnp", "nmos", "thyristor", "triac", "optocoupler", "bridge_rect"] {
        assert_eq!(category(key), Category::Semiconductors, "{key}");
    }
    for key in ["res", "cap", "cap_pol", "inductor", "potentiometer", "varistor", "ntc", "ldr"] {
        assert_eq!(category(key), Category::Passive, "{key}");
    }
}
