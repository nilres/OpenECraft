use super::*;

/// A unit square in z = 0 as EPLAN writes meshes: two triangles of one face, its
/// neighbours and its outline, then the point table.
const SQUARE: &str = "#,1,1{20,2,16,1>{}{}9,4,0/0/0,1/0/0,1/1/0,0/1/0,0,1,84,2,16842751,*,2,0,3,1,4,2,5,0,3,2,5,3,6,2,4294967295,4294967295,4294967295,4294967295,4294967295,4294967295,4294967295,4294967295,4294967295,4294967295,4294967295,4294967295,4,7,2,0,1,8,2,1,2,9,2,2,3,10,2,3,0,1,5,0/0/0,0,0,0}";

/// A macro with a housing (Y up, placed with its height along z like EPLAN's
/// enclosures), a mounting surface on its front, a door on that surface and a
/// restricted placing area on the surface.
fn macro_file() -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8" ?>
<EplanPxfRoot Name="Enclosure" Type="WindowMacro">
 <MacroVariant MacroFuncType="10" VariantId="0">
  <O179 A1="179/1" A2482="0/0/0" A2483="1/0/0" A2484="0/1/0"><P11 P20100_1="RIT.1"/></O179>
  <O161 A1="161/2"><Stream Id="62491">{SQUARE}</Stream></O161>
  <O180 A1="180/3" R9="161/2" R2481="179/1" A2482="0/0/0" A2483="1/0/0" A2484="0/0/1"><P11 P20468="Rittal\Items\1_3D.ema"/></O180>
  <O170 A1="170/4" R2481="180/3" A2482="10/20/30" A2483="1/0/0" A2484="0/1/0" A2486="10039" A2601="600/500"><P11 P20045="??_??@Front outside;"/></O170>
  <O180 A1="180/5" R9="161/2" R2481="170/4" A2482="5/5/0" A2483="1/0/0" A2484="0/1/0"><P2605 P22001="RIT.2"/></O180>
  <O175 A1="175/6" R9="161/7" R2481="170/4" A2482="1/2/0" A2483="1/0/0" A2484="0/1/0" A2488="571" A2606="11/11"/>
  <O175 A1="175/8" R2481="170/4" A2482="0/0/0" A2483="1/0/0" A2484="0/1/0" A2488="560" A2606="11/11"/>
 </MacroVariant>
</EplanPxfRoot>"#
    )
}

#[test]
fn meshes_are_read() {
    let m = mesh(SQUARE).unwrap();
    assert_eq!(m.vertices, [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]]);
    assert_eq!(m.triangles, [[0, 1, 2], [0, 2, 3]]);
    // Boxes of restricted areas and damaged streams are no meshes.
    assert!(mesh("#,1,1{21,5,16,1>{}{15,0,16,0,0,0}{10,0,16{3,0,16,0/0/-1.51,0,0,1,0}4,600,1.52}0}").is_none());
    assert!(mesh(&SQUARE.replace(",2,5,3,6,", ",2,5,9,6,")).is_none(), "vertex out of range");
    assert!(mesh(&SQUARE[..SQUARE.len() / 2]).is_none());
}

#[test]
fn objects_are_placed_in_their_parents() {
    let layout = decode(&macro_file()).unwrap();
    assert_eq!(layout.part, "RIT.1");
    assert_eq!(layout.bodies.len(), 2);
    let (housing, door) = (&layout.bodies[0], &layout.bodies[1]);
    assert!(housing.main && !door.main);
    assert_eq!((housing.part.as_str(), housing.source.as_str()), ("", r"Rittal\Items\1_3D.ema"));
    // The housing's y is the macro's z, its z (x × y) the macro's -y.
    assert_eq!(housing.mesh.vertices[2], [1.0, 0.0, 1.0]);
    let s = &layout.surfaces[0];
    assert_eq!((s.kind, s.name.as_str(), s.size), (10039, "Front outside", [600.0, 500.0]));
    assert_eq!((s.origin, s.x, s.y), ([10.0, -30.0, 20.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]));
    // The door: 5 / 5 on the surface.
    assert_eq!(door.part, "RIT.2");
    assert_eq!(door.mesh.vertices[0], [15.0, -30.0, 25.0]);
    assert_eq!(layout.areas, [Area { kind: AreaKind::Placing, surface: 10039, origin: [11.0, -30.0, 22.0], x: [1.0, 0.0, 0.0], y: [0.0, 0.0, 1.0], size: [11.0, 11.0] }]);
    assert_eq!(layout.mesh().triangles.len(), 4);
    assert_eq!(layout.main_bounds(), Some(([0.0, 0.0, 0.0], [1.0, 0.0, 1.0])));
}

#[test]
fn other_macros_have_no_layout() {
    assert!(decode(&macro_file().replace("MacroFuncType=\"10\"", "MacroFuncType=\"1\"")).is_none());
    assert!(decode("not xml").is_none());
}

/// The macro of a Rittal configurator package; it is not part of the repository
/// (`OPENECRAFT_RITTAL_EMA=<path to Configurated_Enclosure_….ema>`).
#[test]
fn rittal_macro() {
    let Ok(path) = std::env::var("OPENECRAFT_RITTAL_EMA") else { return };
    let layout = decode(&crate::decode_text(&std::fs::read(path).unwrap())).unwrap();
    let placing = layout.areas.iter().filter(|a| a.kind == AreaKind::Placing).count();
    eprintln!("{} bodies, {} triangles, {} surfaces, {} areas ({placing} placing), main {:?}", layout.bodies.len(), layout.mesh().triangles.len(), layout.surfaces.len(), layout.areas.len(), layout.main_bounds());
    assert_eq!((layout.bodies.len(), layout.surfaces.len(), layout.areas.len(), placing), (17, 20, 181, 105));
    let (min, max) = layout.main_bounds().unwrap();
    for (got, want) in min.iter().chain(&max).zip([-300.0, -170.1, 0.0, 300.0, 175.0, 600.0]) {
        assert!((got - want).abs() < 0.01, "{min:?} {max:?}");
    }
}
