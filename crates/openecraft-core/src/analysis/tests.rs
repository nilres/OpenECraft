use super::*;
use crate::geom::p;
use crate::model::PageKind;

/// Nets as sorted lists of pin designations, independent of the net order.
fn nets(prj: &Project, nl: &Netlist) -> Vec<(String, Vec<String>, usize)> {
    let mut v: Vec<_> = nl
        .nets
        .iter()
        .map(|n| {
            let mut pins: Vec<String> = n.pins.iter().map(|&pin| crate::netlist::pin_designation(prj, pin)).collect();
            pins.sort();
            (n.name.clone(), pins, n.wires.len())
        })
        .collect();
    v.sort();
    v
}

fn schematic_pages(prj: &Project) -> usize {
    prj.pages.iter().filter(|p| p.kind == PageKind::Schematic).count()
}

/// Builds the analysis and tells whether the 3D routing was searched for it.
fn build_searching(prj: &Project, cache: &mut Cache) -> (Analysis, bool) {
    let searched = std::sync::atomic::AtomicBool::new(false);
    let an = Analysis::build_progress(prj, cache, &|p| {
        if p.step == Step::Routing && p.total > 0 {
            searched.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    });
    (an, searched.into_inner())
}

#[test]
fn a_routing_searched_elsewhere_is_taken_instead_of_a_search() {
    let example = |key: &str| (crate::sample::EXAMPLES.iter().find(|e| e.key == key).unwrap().build)();
    let prj = example("conveyor");
    let mut elsewhere = Cache::default();
    assert!(elsewhere.routing_searched().is_none());
    let (first, searched) = build_searching(&prj, &mut elsewhere);
    assert!(searched && !first.routing.items.is_empty());
    let text = elsewhere.routing_searched().unwrap();

    // Another build of the project as it was read from its file takes it.
    let read: Project = serde_json::from_str(&serde_json::to_string(&prj).unwrap()).unwrap();
    let mut cache = Cache::default();
    cache.adopt_routing(&text).unwrap();
    let (taken, searched) = build_searching(&read, &mut cache);
    assert!(!searched);
    assert_eq!(taken.routing, first.routing);
    assert_eq!(taken.wiring, first.wiring);
    // It is kept like a routing searched here.
    assert!(!build_searching(&read, &mut cache).1);

    // One of another wiring or language is not taken, nor is text that is none.
    let mut other = Cache::default();
    other.adopt_routing(&text).unwrap();
    assert!(openecraft_i18n::with_lang(openecraft_i18n::Lang::De, || build_searching(&prj, &mut other).1));
    let mut other = Cache::default();
    other.adopt_routing(&text).unwrap();
    assert!(build_searching(&example("cabinet_lineup"), &mut other).1);
    assert!(Cache::default().adopt_routing("{}").is_err());

    // One without translated text is taken in every language, and stays right there.
    let any = Cache::routing_for_any_language(&text).unwrap();
    assert_ne!(any, text);
    assert!(Cache::routing_for_any_language("{}").is_err());
    openecraft_i18n::with_lang(openecraft_i18n::Lang::De, || {
        let mut cache = Cache::default();
        cache.adopt_routing(&any).unwrap();
        let (taken, searched) = build_searching(&prj, &mut cache);
        assert!(!searched);
        assert_eq!(taken.routing, Analysis::build(&prj).routing);
        assert_eq!(Cache::routing_for_any_language(&cache.routing_searched().unwrap()).unwrap(), any);
    });
}

#[test]
fn a_second_build_scans_only_the_changed_page() {
    let mut prj = crate::sample::demo_project();
    let mut cache = Cache::default();
    let first = Analysis::build_with(&prj, &mut cache);
    assert_eq!(cache.scanned(), 2 * schematic_pages(&prj));
    assert_eq!(nets(&prj, &first.netlist), nets(&prj, &Analysis::build(&prj).netlist));

    let page = prj.pages.iter().position(|p| p.kind == PageKind::Schematic && !p.wires.is_empty()).unwrap();
    let wire = prj.pages[page].wires.remove(0);
    let again = Analysis::build_with(&prj, &mut cache);
    assert_eq!(cache.scanned(), 2);
    let fresh = Analysis::build(&prj);
    assert_eq!(nets(&prj, &again.netlist), nets(&prj, &fresh.netlist));
    assert_eq!(nets(&prj, &again.open), nets(&prj, &fresh.open));
    assert_eq!(again.netlist.unconnected.len(), fresh.netlist.unconnected.len());
    assert!(!again.netlist.wire_net.contains_key(&wire.id));
    assert_eq!(again.connections.len(), fresh.connections.len());

    Analysis::build_with(&prj, &mut cache);
    assert_eq!(cache.scanned(), 0);
}

#[test]
fn a_moved_symbol_or_a_new_page_is_scanned_again() {
    let mut prj = crate::sample::demo_project();
    let mut cache = Cache::default();
    Analysis::build_with(&prj, &mut cache);
    let page = prj.pages.iter().position(|p| p.kind == PageKind::Schematic && !p.symbols.is_empty()).unwrap();
    prj.pages[page].symbols[0].pos = prj.pages[page].symbols[0].pos + p(4.0, 0.0);
    let mut copy = prj.pages[page].clone();
    copy.id = prj.new_id();
    for s in &mut copy.symbols {
        s.id = prj.new_id();
    }
    for w in &mut copy.wires {
        w.id = prj.new_id();
    }
    prj.pages.push(copy);
    let an = Analysis::build_with(&prj, &mut cache);
    assert_eq!(cache.scanned(), 4);
    assert_eq!(nets(&prj, &an.netlist), nets(&prj, &Analysis::build(&prj).netlist));
}

#[test]
fn changed_user_symbols_scan_every_page_again() {
    let mut prj = crate::sample::demo_project();
    let mut cache = Cache::default();
    Analysis::build_with(&prj, &mut cache);
    prj.symbols.push(crate::model::symlib::UserSymbol { key: "user:x".into(), name: "X".into(), ..Default::default() });
    Analysis::build_with(&prj, &mut cache);
    assert_eq!(cache.scanned(), 2 * schematic_pages(&prj));
}

#[test]
fn par_map_keeps_the_order_and_the_language() {
    let items: Vec<usize> = (0..100).collect();
    assert_eq!(par_map(&items, |i| i * 2), items.iter().map(|i| i * 2).collect::<Vec<_>>());
    let langs = openecraft_i18n::with_lang(openecraft_i18n::Lang::De, || par_map(&items, |_| openecraft_i18n::lang()));
    assert!(langs.iter().all(|l| *l == openecraft_i18n::Lang::De));
    let (a, b) = openecraft_i18n::with_lang(openecraft_i18n::Lang::Fr, || join(openecraft_i18n::lang, openecraft_i18n::lang));
    assert_eq!((a, b), (openecraft_i18n::Lang::Fr, openecraft_i18n::Lang::Fr));
}

#[test]
fn a_build_tells_its_steps_and_the_routes() {
    let prj = crate::sample::example("pump_room").unwrap();
    let seen = Mutex::new(Vec::new());
    let an = Analysis::build_progress(&prj, &mut Cache::default(), &|p| seen.lock().unwrap().push(p));
    let seen = seen.into_inner().unwrap();
    let mut steps: Vec<Step> = seen.iter().map(|p| p.step).collect();
    steps.dedup();
    assert_eq!(steps, Step::ALL[..5]);
    let routes: Vec<&Progress> = seen.iter().filter(|p| p.step == Step::Routing && p.total > 0).collect();
    assert!(!routes.is_empty());
    assert!(routes.windows(2).all(|w| w[0].done < w[1].done && w[1].done < w[1].total));
    assert!(!an.routing.items.is_empty());
}
