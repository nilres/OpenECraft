use super::*;
use crate::sample::demo_project;

#[test]
fn settings_go_from_one_project_to_another() {
    let mut a = demo_project();
    a.wire_colors = true;
    a.wire_settings.jump = crate::model::JumpStyle::Bridge;
    a.wire_settings.reroute = true;
    a.marking.wire_material = "SHOP.WIRE".into();
    a.check_settings.disabled.push("cables".into());
    let json = export(&a);
    let mut b = crate::model::Project::default();
    let taken = apply(&mut b, &json).unwrap();
    assert_eq!(taken.len(), 11);
    assert_eq!(b.potentials, a.potentials);
    assert_eq!(b.wire_numbering, a.wire_numbering);
    assert!(b.wire_colors);
    assert_eq!(b.wire_settings, a.wire_settings);
    assert_eq!(b.marking.wire_material, "SHOP.WIRE");
    assert_eq!(b.check_settings.disabled, vec!["cables".to_string()]);
}

#[test]
fn partial_schemes_change_only_their_parts() {
    let mut b = demo_project();
    let potentials = b.potentials.clone();
    let taken = apply(&mut b, r#"{ "wire_colors": true }"#).unwrap();
    assert_eq!(taken, vec!["Wire colours"]);
    assert!(b.wire_colors);
    assert_eq!(b.potentials, potentials);
    assert!(apply(&mut b, "[]").is_err());
}

#[test]
fn layers_go_to_another_project_with_their_settings() {
    let a = Project {
        layers: vec![
            Layer { name: "Background".into(), visible: true, locked: true, print: false, color: Some([128, 128, 128]) },
            Layer { name: "Texts".into(), visible: false, locked: false, print: true, color: None },
        ],
        ..Default::default()
    };
    let json = export_layers(&a);
    assert!(!json.contains("wire_numbering"));
    let mut b = Project {
        layers: vec![
            Layer { name: "Texts".into(), visible: true, locked: true, print: false, color: None },
            Layer { name: "Own".into(), visible: true, locked: false, print: true, color: None },
        ],
        ..Default::default()
    };
    assert_eq!(apply(&mut b, &json).unwrap(), vec!["Layers"]);
    // Layers of the file take its settings, the project's own stay.
    let names: Vec<&str> = b.layers.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(names, ["Texts", "Own", "Background"]);
    assert_eq!(b.layers[0], a.layers[1]);
    assert_eq!(b.layers[2], a.layers[0]);
    // The whole scheme carries them too.
    let mut c = Project::default();
    apply(&mut c, &export(&a)).unwrap();
    assert_eq!(c.layers, a.layers);
}
