use super::*;
use openecraft_core::sample;

#[test]
fn pages_are_grouped_by_structure_in_their_order() {
    let mut prj = sample::demo_project();
    prj.pages[1].location = "CAB1".into();
    prj.pages[2].location = "CAB1".into();
    let all: Vec<usize> = (0..prj.pages.len()).collect();
    let groups = structure_groups(&prj, &all);
    // Pages without structure stay together, wherever they are.
    assert_eq!(groups[0].0, "");
    assert_eq!(groups[0].1[0], 0);
    assert_eq!(groups[1], ("+CAB1".to_string(), vec![1, 2]));
    assert_eq!(groups.len(), 2);
    assert_eq!(groups.iter().map(|g| g.1.len()).sum::<usize>(), prj.pages.len());
    assert_eq!(structure_title(&prj, "+CAB1"), "+CAB1");
}

#[test]
fn devices_are_grouped_by_structure() {
    let tags = ["=A1-K1", "-K2", "=A1-K3"].map(String::from);
    let groups = device_groups(&tags);
    assert_eq!(groups, vec![(String::new(), vec![&tags[1]]), ("=A1".to_string(), vec![&tags[0], &tags[2]])]);
    assert_eq!(device_title("=A1-K1", true), "-K1");
    assert_eq!(device_title("=A1-K1", false), "=A1-K1");
}

#[test]
fn places_name_the_symbol_and_its_reference() {
    let prj = sample::demo_project();
    let idx = Index::build(&prj);
    let (tag, dev) = idx.devices.iter().find(|(_, d)| !d.mains.is_empty() && !d.auxs.is_empty()).expect("a device with main and auxiliary functions");
    let list = places(&prj, dev);
    assert_eq!(list.len(), dev.mains.len() + dev.auxs.len() + dev.others.len(), "{tag}");
    assert_eq!(list[0].kind, PlaceKind::Main);
    assert!(list.iter().any(|p| p.kind == PlaceKind::Aux));
    let text = list[0].text(|t| t);
    assert!(text.starts_with(&list[0].reference) && text.contains("Main") && text.contains(list[0].symbol), "{text}");
    let de = list[0].text(|t| translate(t, Lang::De));
    assert!(de.contains(translate("Main", Lang::De)), "{de}");
}

#[test]
fn the_view_has_every_page_section_and_device() {
    let prj = sample::example("building_services").expect("example");
    let idx = Index::build(&prj);
    let view = project_view(&prj, &idx, Lang::En);
    assert_eq!(view.title, prj.info.name);
    assert_eq!(view.pages.len(), prj.pages.len());
    assert_eq!(view.pages[0].icon, page_icon(prj.pages[0].kind));
    assert_eq!(view.sections.len(), prj.sections.len());
    assert!(view.pages.iter().any(|p| p.section != 0));
    assert!(view.pages.iter().all(|p| p.section == 0 || view.sections.iter().any(|s| s.id == p.section)));
    let devices: usize = view.devices.iter().map(|g| g.devices.len()).sum();
    assert_eq!(devices, idx.devices.len());
    let ids: Vec<u64> = view.pages.iter().map(|p| p.id).collect();
    assert!(view.devices.iter().flat_map(|g| &g.devices).flat_map(|d| &d.places).all(|p| ids.contains(&p.page)));
    let json = serde_json::to_string(&view).unwrap();
    assert_eq!(serde_json::from_str::<ProjectView>(&json).unwrap(), view);
}
