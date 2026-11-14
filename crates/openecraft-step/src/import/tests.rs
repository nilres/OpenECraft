use super::*;

/// A cuboid of 10 × 20 × 30 mm from the origin, written by monstertruck.
const CUBOID_MM: &str = include_str!("cuboid_mm.step");
/// The same cuboid in metres (`SI_UNIT($,.METRE.)`).
const CUBOID_M: &str = include_str!("cuboid_m.step");

fn assert_size(mesh: &Mesh3d, expected: [f32; 3]) {
    let size = mesh.size();
    for k in 0..3 {
        assert!((size[k] - expected[k]).abs() < 0.01, "size {size:?}, expected {expected:?}");
    }
}

#[test]
fn single_solid_in_millimetres() {
    let mesh = parse_step(CUBOID_MM.as_bytes()).unwrap();
    assert_size(&mesh, [10.0, 20.0, 30.0]);
    assert!(mesh.triangles.len() >= 12, "six faces, at least two triangles each");
    // Moved to the origin like every imported model.
    assert_eq!(mesh.bounds().0, [0.0, 0.0, 0.0]);
}

#[test]
fn lengths_are_converted_to_millimetres() {
    assert_size(&parse_step(CUBOID_M.as_bytes()).unwrap(), [10.0, 20.0, 30.0]);
}

/// The cuboid file plus a top product that uses its part (`#5`, shape
/// representation `#10`) once per offset: the AP214 assembly structure with
/// `NEXT_ASSEMBLY_USAGE_OCCURRENCE` and `ITEM_DEFINED_TRANSFORMATION`.
fn assembly_file(part: &str, offsets: &[f64]) -> String {
    let mut entities = vec![
        "#1000 = PRODUCT('top','top','',(#8));".to_string(),
        "#1001 = PRODUCT_DEFINITION_FORMATION('','',#1000);".into(),
        "#1002 = PRODUCT_DEFINITION('design','',#1001,#9);".into(),
        "#1003 = PRODUCT_DEFINITION_SHAPE('','',#1002);".into(),
        "#1006 = CARTESIAN_POINT('',(0.,0.,0.));".into(),
        "#1007 = DIRECTION('',(0.,0.,1.));".into(),
        "#1008 = DIRECTION('',(1.,0.,0.));".into(),
        "#1009 = AXIS2_PLACEMENT_3D('',#1006,#1007,#1008);".into(),
    ];
    let mut placements = Vec::new();
    for (i, x) in offsets.iter().enumerate() {
        let n = 1100 + 10 * i;
        placements.push(format!("#{}", n + 1));
        entities.extend([
            format!("#{n} = CARTESIAN_POINT('',({x:.1},0.,0.));"),
            format!("#{} = AXIS2_PLACEMENT_3D('',#{n},#1007,#1008);", n + 1),
            format!("#{} = NEXT_ASSEMBLY_USAGE_OCCURRENCE('{i}','block','',#1002,#5,$);", n + 2),
            format!("#{} = PRODUCT_DEFINITION_SHAPE('','',#{});", n + 3, n + 2),
            format!("#{} = ITEM_DEFINED_TRANSFORMATION('','',#1009,#{});", n + 4, n + 1),
            format!(
                "#{} = ( REPRESENTATION_RELATIONSHIP('','',#10,#1004) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#{}) SHAPE_REPRESENTATION_RELATIONSHIP() );",
                n + 5,
                n + 4
            ),
            format!("#{} = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#{},#{});", n + 6, n + 5, n + 3),
        ]);
    }
    entities.push(format!("#1004 = SHAPE_REPRESENTATION('top',(#1009,{}),#11);", placements.join(",")));
    entities.push("#1005 = SHAPE_DEFINITION_REPRESENTATION(#1003,#1004);".into());
    let end = part.rfind("ENDSEC;").unwrap();
    format!("{}{}\n{}", &part[..end], entities.join("\n"), &part[end..])
}

#[test]
fn assemblies_place_their_parts() {
    // One part used twice, 50 mm apart.
    let mesh = parse_step(assembly_file(CUBOID_MM, &[0.0, 50.0]).as_bytes()).unwrap();
    assert_size(&mesh, [60.0, 20.0, 30.0]);
    // Used once: only the part.
    assert_size(&parse_step(assembly_file(CUBOID_MM, &[50.0]).as_bytes()).unwrap(), [10.0, 20.0, 30.0]);
}

#[test]
fn skipped_products_are_left_out() {
    // The second use is a keep-out body.
    let file = assembly_file(CUBOID_MM, &[0.0, 50.0]).replacen("('1','block'", "('1','Restricted placing area'", 1);
    let skip = |name: &str| name.starts_with("Restricted");
    assert_size(&parse_step_without(file.as_bytes(), &skip).unwrap(), [10.0, 20.0, 30.0]);
    assert_size(&parse_step(file.as_bytes()).unwrap(), [60.0, 20.0, 30.0]);
    // Skipping the assembly leaves out everything below it.
    assert!(parse_step_without(file.as_bytes(), &|name| name == "top").map_or(true, |m| m.triangles.is_empty()));
}

/// Parameter ranges of circular edges from their vertices: a unit circle about z with
/// vertices at 0°, 90° and 180°.
#[test]
fn arcs_get_their_range_from_their_vertices() {
    use cadmpeg_ir::geometry::Curve;
    use cadmpeg_ir::ids::{CurveId, EdgeId, PointId, VertexId};
    use cadmpeg_ir::topology::{Edge, Point, Vertex};
    use std::f64::consts::{FRAC_PI_2, PI, TAU};
    let mut ir = CadIr::empty(Default::default());
    for (k, (x, y)) in [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)].into_iter().enumerate() {
        ir.model.points.push(Point { id: PointId(format!("p{k}")), position: IrPoint::new(x, y, 0.0), source_object: None });
        ir.model.vertices.push(Vertex { id: VertexId(format!("v{k}")), point: PointId(format!("p{k}")), tolerance: None });
    }
    ir.model.curves.push(Curve {
        id: CurveId("c".into()),
        geometry: CurveGeometry::Circle { center: IrPoint::new(0.0, 0.0, 0.0), axis: IrVector::new(0.0, 0.0, 2.0), ref_direction: IrVector::new(3.0, 0.0, 0.0), radius: 1.0 },
        source_object: None,
    });
    let edges = [(0, 1), (1, 0), (2, 2), (1, 2)];
    for (k, (start, end)) in edges.into_iter().enumerate() {
        let id = EdgeId(format!("e{k}"));
        ir.model.edges.push(Edge { id, curve: Some(CurveId("c".into())), start: VertexId(format!("v{start}")), end: VertexId(format!("v{end}")), param_range: None, tolerance: None });
    }
    fill_arc_ranges(&mut ir);
    let ranges: Vec<[f64; 2]> = ir.model.edges.iter().map(|e| e.param_range.unwrap()).collect();
    // Counter-clockwise from start to end; a closed edge once round.
    for (got, want) in ranges.iter().zip([[0.0, FRAC_PI_2], [FRAC_PI_2, TAU], [PI, PI + TAU], [FRAC_PI_2, PI]]) {
        assert!((got[0] - want[0]).abs() < 1e-9 && (got[1] - want[1]).abs() < 1e-9, "{got:?} != {want:?}");
    }
}

/// The STEP model of a Rittal configurator package without its restricted areas; the
/// file is not part of the repository (`OPENECRAFT_RITTAL_STEP=<path to 3D_Step.stp>`).
#[test]
fn rittal_step_without_restricted_areas() {
    let Ok(path) = std::env::var("OPENECRAFT_RITTAL_STEP") else { return };
    let bytes = std::fs::read(path).unwrap();
    let mesh = parse_step_without(&bytes, &|name| name.contains("Restricted")).unwrap();
    let all = parse_step(&bytes).unwrap();
    eprintln!("without restricted areas: {} triangles {:?}, all: {} triangles {:?}", mesh.triangles.len(), mesh.size(), all.triangles.len(), all.size());
    assert!(mesh.triangles.len() < all.triangles.len());
}

#[test]
fn files_without_geometry_are_refused() {
    assert!(parse_step(b"not a step file").is_err());
    let empty = "ISO-10303-21;\nHEADER;\nENDSEC;\nDATA;\nENDSEC;\nEND-ISO-10303-21;\n";
    assert!(parse_step(empty.as_bytes()).is_err());
}
