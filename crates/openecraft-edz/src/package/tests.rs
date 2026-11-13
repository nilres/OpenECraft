use super::*;
use openecraft_core::zip::ZipWriter;

/// A PNG of 2 × 2 px as the preview of a macro.
fn png() -> Vec<u8> {
    let mut out = Vec::new();
    image::RgbImage::from_pixel(2, 2, image::Rgb([200, 30, 30])).write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png).unwrap();
    out
}

/// A 3D window macro as EPLAN Pro Panel writes it: the preview and a placed body.
fn ema() -> String {
    let png = png();
    let bytes: Vec<String> = png.iter().map(u8::to_string).collect();
    format!(
        r#"<?xml version="1.0" encoding="utf-8" ?>
<EplanPxfRoot Name="Configurated_Enclosure_1_1360000" Type="WindowMacro">
 <VariantPreview MacroFuncType="10" VariantId="0" Index="0"><![CDATA[#,1,1{{1,2,16,1,1,{},{}}}]]></VariantPreview>
 <MacroVariant MacroFuncType="10" VariantId="0">
  <O180 A1="180/1"><P11 P20468="Rittal\Items\398818_3D.ema"/><P2605 P22001="RIT.398818" P22010="$(MD_MACROS)\Rittal\Items\398818_3D.ema"/></O180>
 </MacroVariant>
</EplanPxfRoot>"#,
        png.len(),
        bytes.join(",")
    )
}

const BOM: &str = "\u{feff}Number;Quantity;Unit;Description;EAN\n1360000;1;Pc(s.);AX Compact enclosure, WHD: 600x600x350 mm, sheet steel;4028177813168\n2500100;2;Pc(s.);SZ Wall mounting bracket;4028177000001\n";

/// The 3D macro of an enclosure of 600 × 600 × 350 mm lying on its back (height along
/// z, front towards -y), with the front of its mounting plate (550 × 575 mm, 20 mm in
/// front of the rear) and a restricted cut-out area on it.
fn layout_ema() -> String {
    let housing = "#,1,1{20,2,16,1>{}{}9,4,-300/-175/0,300/-175/0,300/175/600,-300/175/600,0,1,84,2,16842751,*,2,0,3,1,4,2,5,0,3,2,5,3,6,2,0,0,0,0,0,0,0,0,0,0,0,0,0}";
    format!(
        r#"<?xml version="1.0" encoding="utf-8" ?>
<EplanPxfRoot Name="Configurated_Enclosure_1_1360000" Type="WindowMacro">
 <MacroVariant MacroFuncType="10" VariantId="0">
  <O179 A1="179/1" A2482="0/0/0" A2483="1/0/0" A2484="0/1/0"><P11 P20100_1="RIT.398580"/><P2605 P22001="RIT.398580"/></O179>
  <O161 A1="161/2"><Stream Id="62491">{housing}</Stream></O161>
  <O180 A1="180/3" R9="161/2" R2481="179/1" A2482="0/0/0" A2483="1/0/0" A2484="0/1/0"><P11 P20468="Rittal\Items\398580_3D.ema"/></O180>
  <O170 A1="170/4" R2481="180/3" A2482="-275/155/12.5" A2483="1/0/0" A2484="0/0/1" A2486="10002" A2601="550/575"/>
  <O175 A1="175/5" R2481="170/4" A2482="0/0/0" A2483="1/0/0" A2484="0/1/0" A2488="572" A2606="10/20"/>
 </MacroVariant>
</EplanPxfRoot>"#
    )
}

/// A package like the one of the Rittal configurator (paths with backslashes).
fn package() -> Vec<u8> {
    package_with(&ema())
}

fn package_with(ema: &str) -> Vec<u8> {
    let mut zip = ZipWriter::new("2026-11-11");
    zip.add("CSV_article_list\\bill-of-material.csv", BOM.as_bytes());
    zip.add("ConfigurationId.txt", b"R136000AX01T9K6KQ0");
    zip.add("EPLAN_Pro_Panel_data\\Configurated_Enclosure_1_1360000.ema", ema.as_bytes());
    zip.add("EPLAN_Pro_Panel_data\\EplanProPanelProject.zw1", b"7z");
    zip.add("EPLAN_Pro_Panel_data\\ReadMe.txt", b"Attention!");
    zip.add("Product_description\\Product_Description.pdf", b"%PDF-1.7");
    zip.add("Step_3Dpdf\\3D.pdf", b"%PDF-1.7");
    zip.add("Step_3Dpdf\\3D_Step.stp", b"ISO-10303-21;");
    zip.finish()
}

/// A STEP reader returning the enclosure lying on its back (z up, depth along y), as
/// CAD models are; it checks that the restricted areas are left out.
fn step(_: &[u8], skip: &dyn Fn(&str) -> bool) -> Result<Mesh3d, String> {
    assert!(skip("S1:Restricted placing area 3/71") && skip("S1:Restricted cut-out area (26361)"));
    assert!(!skip("S1:Door (3523)") && !skip("S1:Housings (1)"));
    Ok(Mesh3d { vertices: vec![[-300.0, -175.0, 0.0], [300.0, 175.0, 600.0], [0.0, 0.0, 600.0]], triangles: vec![[0, 1, 2]], colors: Vec::new() })
}

#[test]
fn the_bill_of_materials_gives_the_parts() {
    let edz = read(&package(), None).unwrap();
    assert_eq!(edz.parts.len(), 2);
    let p = &edz.parts[0];
    assert_eq!((p.number.as_str(), p.type_number.as_str(), p.order_number.as_str(), p.manufacturer.as_str()), ("RIT.1360000", "1360000", "1360000", "Rittal"));
    assert_eq!(p.description, "AX Compact enclosure, WHD: 600x600x350 mm, sheet steel");
    assert_eq!((p.width, p.height, p.depth), (600.0, 600.0, 350.0));
    assert_eq!((p.top_group, p.group.as_str(), p.subgroup.as_str(), p.quantity_unit.as_str()), (TopGroup::Mechanics, "Mounting", "Enclosure", "pcs"));
    assert_eq!(p.tech_value("Enclosure system"), Some("compact"));
    assert_eq!(p.tech_value("GTIN"), Some("4028177813168"));
    assert_eq!(p.tech_value("Configuration"), Some("R136000AX01T9K6KQ0"));
    assert_eq!(p.components, vec![Component { number: "RIT.2500100".into(), quantity: 2.0 }]);
    assert!(!p.picture.is_empty(), "the preview of the macro");
    assert!(p.model_3d.is_none(), "no STEP reader");
    assert_eq!(p.source_data.as_ref().unwrap()["configuration"], "R136000AX01T9K6KQ0");
    let accessory = &edz.parts[1];
    assert_eq!((accessory.number.as_str(), accessory.subgroup.as_str(), accessory.width), ("RIT.2500100", "", 0.0));
}

#[test]
fn the_files_belong_to_the_configured_part() {
    let edz = read(&package(), None).unwrap();
    let files: Vec<(&str, &str, AttachmentKind)> = edz.files.iter().map(|f| (f.part.as_str(), f.name.as_str(), f.kind)).collect();
    assert_eq!(
        files,
        [
            ("RIT.1360000", "Configurated_Enclosure_1_1360000.ema", AttachmentKind::Other),
            ("RIT.1360000", "EplanProPanelProject.zw1", AttachmentKind::Other),
            ("RIT.1360000", "Product_Description.pdf", AttachmentKind::Datasheet),
            ("RIT.1360000", "3D.pdf", AttachmentKind::Model3d),
            ("RIT.1360000", "3D_Step.stp", AttachmentKind::Model3d),
        ]
    );
}

#[test]
fn the_step_model_stands_upright() {
    let edz = read(&package(), Some(step)).unwrap();
    let mesh = edz.parts[0].model_3d.as_ref().unwrap();
    assert_eq!(mesh.size(), [600.0, 600.0, 350.0]);
    // The front (y = -175) faces +z.
    assert_eq!(mesh.vertices[0], [0.0, 0.0, 350.0]);
}

#[test]
fn the_macro_gives_model_plate_and_restricted_areas() {
    // No STEP reader is needed (the command line has none).
    let edz = read(&package_with(&layout_ema()), None).unwrap();
    let p = &edz.parts[0];
    assert_eq!(p.model_3d.as_ref().unwrap().size(), [600.0, 600.0, 350.0]);
    assert_eq!(p.tech_value("Mounting origin"), Some("25/12.5/20"));
    assert_eq!(p.tech_value("Mounting size"), Some("550 x 575"));
    // Without a surface of the plate's kind: the largest one facing the front.
    let other = read(&package_with(&layout_ema().replace("A2486=\"10002\"", "A2486=\"10099\"")), None).unwrap();
    assert_eq!(other.parts[0].tech_value("Mounting origin"), Some("25/12.5/20"));
    // The area 10 × 20 mm at the lower left corner of the plate, in plate coordinates
    // (y down from the plate's top).
    assert_eq!(p.mounting_data.restricted, [RestrictedArea { kind: Restriction::Cutout, face: "plate".into(), x: 0.0, y: 555.0, w: 10.0, h: 20.0 }]);
    // The macro wins over the STEP model.
    let edz = read(&package_with(&layout_ema()), Some(step)).unwrap();
    assert_eq!(edz.parts[0].model_3d.as_ref().unwrap().triangles.len(), 2);
}

#[test]
fn an_unreadable_model_keeps_the_part() {
    fn broken(_: &[u8], _: &dyn Fn(&str) -> bool) -> Result<Mesh3d, String> {
        Err("a circle without range".into())
    }
    let edz = read(&package(), Some(broken)).unwrap();
    let p = &edz.parts[0];
    assert!(p.model_3d.is_none());
    assert_eq!(p.source_data.as_ref().unwrap()["model_error"], "a circle without range");
    assert!(edz.files.iter().any(|f| f.name == "3D_Step.stp"));
}

#[test]
fn archives_without_bill_of_materials_are_refused() {
    let mut zip = ZipWriter::new("2026-11-11");
    zip.add("readme.txt", b"no parts");
    assert!(read(&zip.finish(), None).is_err());
    let mut zip = ZipWriter::new("2026-11-11");
    zip.add("bill-of-material.csv", b"Number;Quantity\n");
    assert!(read(&zip.finish(), None).is_err());
    assert!(is_zip(&package()) && !is_zip(b"7z\xBC\xAF\x27\x1C"));
}

#[test]
fn sizes_come_from_the_description() {
    assert_eq!(whd("AX Compact enclosure, WHD: 600x600x350 mm, sheet steel"), Some([600.0, 600.0, 350.0]));
    assert_eq!(whd("VX, WHD: 1200 x 2000"), None);
    assert_eq!(whd("Mounting plate"), None);
}

/// A package of the Rittal configurator; it is not part of the repository
/// (`OPENECRAFT_RITTAL_ZIP=<path to the .zip>`).
#[test]
fn rittal_package() {
    let Ok(path) = std::env::var("OPENECRAFT_RITTAL_ZIP") else { return };
    let edz = read(&std::fs::read(path).unwrap(), Some(openecraft_step::parse_step_without)).unwrap();
    let p = &edz.parts[0];
    eprintln!("{} {} {:?} picture {} files {:?}", p.number, p.manufacturer, p.model_3d.as_ref().map(|m| (m.triangles.len(), m.size())), p.picture.len(), edz.files.iter().map(|f| &f.name).collect::<Vec<_>>());
    assert_eq!(p.manufacturer, "Rittal");
    assert!(!p.picture.is_empty());
    // The bodies of the macro, with the door's hinges and lock in front of the housing.
    let size = p.model_3d.as_ref().unwrap().size();
    assert!((size[1] - 600.0).abs() < 1.0 && (350.0..360.0).contains(&size[2]), "{size:?}");
    assert_eq!(p.tech_value("Mounting origin"), Some("25/12.5/20"));
    // All areas but the three on the frame around the door, on all faces.
    let areas = &p.mounting_data.restricted;
    let faces: std::collections::BTreeMap<(&str, &str), usize> = areas.iter().fold(Default::default(), |mut m, a| {
        *m.entry((a.face.as_str(), a.kind.key())).or_default() += 1;
        m
    });
    eprintln!("{} areas: {faces:?}", areas.len());
    assert_eq!(areas.len(), 178);
    assert!(openecraft_parts::ENCLOSURE_FACES.iter().all(|f| areas.iter().any(|a| a.face == *f)), "{faces:?}");
    // Each within its face of an enclosure of the part.
    let mut prj = openecraft_core::model::Project::default();
    prj.parts.push(p.clone());
    let e = openecraft_core::panel::new_enclosure(&mut prj, &p.number, "+A1");
    for a in areas {
        let (w, h) = e.face_size(openecraft_core::panel::Face::from_key(&a.face).unwrap());
        assert!(a.x >= -1.0 && a.y >= -1.0 && a.x + a.w <= w + 1.0 && a.y + a.h <= h + 1.0, "{} on {w} x {h}", a.text());
    }
}
