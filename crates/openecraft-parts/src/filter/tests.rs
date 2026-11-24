use super::*;

fn part(number: &str, manufacturer: &str, supplier: &str, top_group: TopGroup, group: &str) -> Part {
    Part { number: number.into(), manufacturer: manufacturer.into(), supplier: supplier.into(), top_group, group: group.into(), ..Default::default() }
}

fn sample() -> Vec<Part> {
    vec![
        part("ABB.K1", "ABB", "Wholesale", TopGroup::Electrical, "Relays, contactors"),
        part("SIE.K2", "Siemens", "Wholesale", TopGroup::Electrical, "Relays, contactors"),
        part("SIE.Q1", "Siemens", "Direct", TopGroup::Electrical, "Safety devices"),
        part("FES.V1", "Festo", "", TopGroup::Fluid, "Valves"),
    ]
}

fn numbers(parts: &[Part], f: &PartFilter) -> Vec<String> {
    parts.iter().filter(|p| f.matches(p)).map(|p| p.number.clone()).collect()
}

#[test]
fn empty_filter_matches_everything() {
    let parts = sample();
    let f = PartFilter::default();
    assert!(!f.is_active());
    assert_eq!(numbers(&parts, &f).len(), parts.len());
}

#[test]
fn criteria_are_combined() {
    let parts = sample();
    let mut f = PartFilter { manufacturer: Some("Siemens".into()), ..Default::default() };
    assert_eq!(numbers(&parts, &f), ["SIE.K2", "SIE.Q1"]);
    f.group = Some("Safety devices".into());
    assert_eq!(numbers(&parts, &f), ["SIE.Q1"]);
    f.text = "k2".into();
    assert!(numbers(&parts, &f).is_empty());

    let f = PartFilter { supplier: Some("Wholesale".into()), text: "abb".into(), ..Default::default() };
    assert_eq!(numbers(&parts, &f), ["ABB.K1"]);
    let f = PartFilter { top_group: Some(TopGroup::Fluid), ..Default::default() };
    assert_eq!(numbers(&parts, &f), ["FES.V1"]);
}

#[test]
fn reset_keeps_grouping() {
    let mut f = PartFilter { text: "x".into(), manufacturer: Some("ABB".into()), grouping: Grouping::Manufacturer, ..Default::default() };
    assert!(f.is_active());
    f.reset();
    assert!(!f.is_active());
    assert_eq!(f.grouping, Grouping::Manufacturer);
}

#[test]
fn distinct_values_are_sorted_and_unique() {
    let parts = sample();
    assert_eq!(distinct_values(&parts, |p| &p.manufacturer), ["ABB", "Festo", "Siemens"]);
    assert_eq!(distinct_values(&parts, |p| &p.supplier), ["Direct", "Wholesale"]);
}

#[test]
fn filter_for_wall_mountable_parts() {
    let wall = Part { number: "VIC.MPPT".into(), mounting_data: crate::MountingData { wall_mount: true, ..Default::default() }, ..Default::default() };
    let rail = Part { number: "GEN.MCB".into(), ..Default::default() };
    let f = PartFilter { wall_mount: true, ..Default::default() };
    assert!(f.is_active());
    assert!(f.matches(&wall));
    assert!(!f.matches(&rail));
}

#[test]
fn filter_for_floor_standing_parts() {
    let pump = Part { number: "GEN.PUMP".into(), mounting_data: crate::MountingData { floor_mount: true, ..Default::default() }, ..Default::default() };
    let rail = Part { number: "GEN.MCB".into(), ..Default::default() };
    let f = PartFilter { floor_mount: true, ..Default::default() };
    assert!(f.is_active());
    assert!(f.matches(&pump));
    assert!(!f.matches(&rail));
}

fn enclosure(number: &str, manufacturer: &str, (w, h, d): (f32, f32, f32), tech: &str) -> Part {
    Part { number: number.into(), manufacturer: manufacturer.into(), subgroup: "Enclosure".into(), width: w, height: h, depth: d, tech_data: tech.into(), ..Default::default() }
}

fn enclosures() -> Vec<Part> {
    vec![
        enclosure("SJ.A37", "Striebel & John", (800.0, 1100.0, 215.0), "Fields: 3\nRows: 7"),
        enclosure("SJ.A15", "Striebel & John", (300.0, 800.0, 215.0), "Fields: 1\nRows: 5"),
        enclosure("SJ.A27", "Striebel & John", (550.0, 1100.0, 215.0), "Fields: 2\nRows: 7"),
        enclosure("GEN.CE-600X760X210", "", (600.0, 760.0, 210.0), "Enclosure system: compact"),
        enclosure("GEN.TS-600X2000X500", "", (600.0, 2000.0, 500.0), "Enclosure system: baying"),
        Part { number: "GEN.GP-600X500".into(), subgroup: "Gland plate".into(), width: 600.0, height: 500.0, ..Default::default() },
    ]
}

fn found(f: &EnclosureFilter) -> Vec<String> {
    f.apply(&enclosures()).into_iter().map(|p| p.number.clone()).collect()
}

#[test]
fn enclosure_search_offers_only_enclosure_bodies() {
    let f = EnclosureFilter::default();
    assert!(!f.is_active());
    assert_eq!(found(&f), ["GEN.CE-600X760X210", "GEN.TS-600X2000X500", "SJ.A15", "SJ.A27", "SJ.A37"]);
}

#[test]
fn enclosure_search_by_rows_fields_and_size() {
    let f = EnclosureFilter { rows: Some(7), ..Default::default() };
    assert!(f.is_active());
    assert_eq!(found(&f), ["SJ.A27", "SJ.A37"]);
    assert_eq!(found(&EnclosureFilter { rows: Some(7), fields: Some(3), ..Default::default() }), ["SJ.A37"]);
    let narrow = EnclosureFilter { width: SizeRange { min: 0.0, max: 600.0 }, height: SizeRange { min: 1000.0, max: 0.0 }, ..Default::default() };
    assert_eq!(found(&narrow), ["GEN.TS-600X2000X500", "SJ.A27"]);
    assert_eq!(found(&EnclosureFilter { depth: SizeRange { min: 300.0, max: 0.0 }, ..Default::default() }), ["GEN.TS-600X2000X500"]);
    let parts = enclosures();
    assert_eq!(distinct_counts(&parts, "Rows"), [5, 7]);
    assert_eq!(distinct_counts(&parts, "Fields"), [1, 2, 3]);
}

#[test]
fn enclosure_search_by_text_system_maker_and_mounting() {
    assert_eq!(found(&EnclosureFilter { text: "a3".into(), ..Default::default() }), ["SJ.A37"]);
    assert_eq!(found(&EnclosureFilter { system: "baying".into(), ..Default::default() }), ["GEN.TS-600X2000X500"]);
    assert_eq!(found(&EnclosureFilter { manufacturer: "Striebel & John".into(), ..Default::default() }).len(), 3);
    assert_eq!(found(&EnclosureFilter { mounting: EnclosureMounting::Floor, ..Default::default() }), ["GEN.TS-600X2000X500"]);
    assert_eq!(found(&EnclosureFilter { mounting: EnclosureMounting::Wall, ..Default::default() }).len(), 4);
    let mut f = EnclosureFilter { text: "x".into(), rows: Some(5), sort: EnclosureSort::Height, ..Default::default() };
    f.reset();
    assert_eq!(f, EnclosureFilter { sort: EnclosureSort::Height, ..Default::default() });
}

#[test]
fn enclosure_search_sorts_and_keeps_each_part_once() {
    let mut parts = enclosures();
    parts.push(enclosure("SJ.A15", "Striebel & John", (999.0, 999.0, 999.0), ""));
    let f = EnclosureFilter { sort: EnclosureSort::Size, ..Default::default() };
    let sizes: Vec<(String, f32)> = f.apply(&parts).into_iter().map(|p| (p.number.clone(), p.width)).collect();
    assert_eq!(sizes[0], ("SJ.A15".to_string(), 300.0));
    assert_eq!(sizes.len(), 5);
    let f = EnclosureFilter { sort: EnclosureSort::Height, ..Default::default() };
    assert_eq!(f.apply(&parts).last().map(|p| p.number.as_str()), Some("GEN.TS-600X2000X500"));
}
