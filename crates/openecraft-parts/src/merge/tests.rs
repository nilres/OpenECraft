use super::*;
use crate::{bundled, count_value, Mesh3d, Shared};

const FV17: &str = "SJ.2CPX054002R9999";
const CABINET: &str = "SJ.2CPX052032R9999";

fn shipped() -> Vec<Part> {
    bundled::load("striebel_john").unwrap().parts
}

/// What an EDZ file of the data portal brings: another description, its own technical
/// data, a model, no fields and rows.
fn from_portal(number: &str) -> Part {
    Part {
        number: number.into(),
        order_number: "2CPX054002R9999".into(),
        manufacturer: "ABB Striebel & John GmbH".into(),
        description: "Verteilerfeld 1FB/7RH".into(),
        group: "Mechanics".into(),
        subgroup: "Housing accessory".into(),
        tech_data: "Colour: RAL 7035\nEAN: 4011617540021\nIP code: IP30".into(),
        width: 250.0,
        model_3d: Some(Shared::new(Mesh3d { vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], triangles: vec![[0, 1, 2]], ..Default::default() })),
        ..Default::default()
    }
}

#[test]
fn an_import_of_the_same_number_keeps_what_it_lacks() {
    let known = shipped();
    let old = known.iter().find(|p| p.number == FV17).unwrap();
    let out = merge_imported(vec![from_portal(FV17)], &known);
    assert_eq!((out.merged, out.renamed.len(), out.parts.len()), (1, 0, 1));
    let p = &out.parts[0];
    // The import's values win.
    assert_eq!((p.description.as_str(), p.manufacturer.as_str()), ("Verteilerfeld 1FB/7RH", "ABB Striebel & John GmbH"));
    assert!(p.model_3d.is_some());
    // What it lacks stays: the fields and rows, the module spaces, the size, the price.
    assert_eq!((count_value(p, "Fields"), count_value(p, "Rows"), count_value(p, "Module spaces")), (Some(1), Some(7), Some(84)));
    assert_eq!((p.height, p.depth, p.price, p.type_number.as_str()), (old.height, old.depth, old.price, "FV17"));
    // The classification stays; the technical data are merged.
    assert_eq!((p.group.as_str(), p.subgroup.as_str()), ("Mounting", "Distribution panel"));
    assert!(p.tech_data.contains("Colour: RAL 7035") && p.tech_data.contains("IP code: IP30"), "{}", p.tech_data);
    assert_eq!(p.tech_data.matches("EAN:").count(), 1);
}

#[test]
fn an_import_of_another_number_is_the_part_of_its_order_number() {
    let known = shipped();
    // The cabinet of the import lists the panel by the import's number.
    let mut cabinet = Part { number: "STR.A37".into(), order_number: "2CPX052032R9999".into(), manufacturer: "Striebel & John".into(), ..Default::default() };
    cabinet.accessories = vec!["STR.FV17".into()];
    let other = Part { number: "STR.OTHER".into(), order_number: "2CPX999999R9999".into(), ..Default::default() };
    let out = merge_imported(vec![cabinet, from_portal("STR.FV17"), other], &known);
    assert_eq!(out.renamed, [("STR.A37".to_string(), CABINET.to_string()), ("STR.FV17".to_string(), FV17.to_string())]);
    assert_eq!((out.merged, out.number_of("STR.FV17"), out.number_of("STR.OTHER")), (2, FV17, "STR.OTHER"));
    let cab = out.parts.iter().find(|p| p.number == CABINET).unwrap();
    // The cabinet keeps its accessories; the import's reference became the known number.
    assert!(cab.accessories.iter().filter(|a| *a == FV17).count() == 1 && cab.accessories.len() > 10, "{:?}", cab.accessories);
    assert_eq!(count_value(cab, "Fields"), Some(3));
    assert!(out.parts.iter().any(|p| p.number == "STR.OTHER"));
}

#[test]
fn order_numbers_and_manufacturers() {
    let a = Part { number: "A.1".into(), order_number: "123456".into(), manufacturer: "Acme".into(), ..Default::default() };
    let b = Part { number: "B.1".into(), order_number: "123 456".into(), manufacturer: "Bolt".into(), ..Default::default() };
    let known = [&a, &b];
    let probe = |order: &str, maker: &str| find_known(&Part { number: "X".into(), order_number: order.into(), manufacturer: maker.into(), ..Default::default() }, &known).map(|p| p.number.clone());
    // Two known parts of that order number: the one of the manufacturer.
    assert_eq!(probe("123-456", "BOLT GmbH"), Some("B.1".into()));
    assert_eq!(probe("123456", "Other"), None);
    // Too short to tell.
    assert_eq!(probe("12345", "Acme"), None);
    assert_eq!(find_known(&Part { number: "a.1".into(), ..Default::default() }, &known).map(|p| p.number.as_str()), Some("A.1"));
}

#[test]
fn technical_data_line_by_line() {
    assert_eq!(merge_lines("Fields: 1\nRows: 7\nEAN: 1", "EAN: 2\nColour: grey\nRows: 7"), "Fields: 1\nRows: 7\nEAN: 2\nColour: grey");
    assert_eq!(merge_lines("", "A: 1"), "A: 1");
    assert_eq!(merge_lines("A: 1\nfree text", ""), "A: 1\nfree text");
}
