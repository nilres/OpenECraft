use cadmpeg_ir::products::{AssemblyGraph, Occurrence, OccurrenceParent, ProductDefinition, PrototypeReference};
use cadmpeg_ir::topology::{Body, BodyKind};
use cadmpeg_ir::transform::Transform;
use cadmpeg_ir::geometry::CurveGeometry;
use cadmpeg_ir::math::{Point3 as IrPoint, Vector3 as IrVector};
use cadmpeg_ir::CadIr;
use monstertruck_io::cadmpeg::{to_bodies, ImportedBody, ImportedSheet};
use monstertruck_meshing::prelude::*;
use monstertruck_modeling::cgmath::Transform as _;
use monstertruck_topology::compress::CompressedShell;
use openecraft_parts::{Mesh3d, MeshBuilder, MAX_TRIANGLES};
use std::collections::{HashMap, HashSet};

/// Chord tolerance of the tessellation relative to the size of a shell; coarser
/// steps are tried when the mesh gets too large.
const TOLERANCES: [f64; 4] = [0.002, 0.006, 0.02, 0.06];

/// The shells of a converted body.
pub(crate) fn body_shells(body: ImportedBody) -> Vec<ImportedSheet> {
    match body {
        ImportedBody::Solid(solid) => solid.boundaries,
        ImportedBody::Sheet(sheet) => vec![sheet],
    }
}

fn matrix(t: &Transform) -> Matrix4 {
    let r = &t.rows;
    Matrix4::new(
        r[0][0], r[1][0], r[2][0], r[3][0], r[0][1], r[1][1], r[2][1], r[3][1], r[0][2], r[1][2], r[2][2], r[3][2], r[0][3], r[1][3], r[2][3], r[3][3],
    )
}

/// The number of a STEP entity reference (`#123`, also at the end of an id).
fn entity(reference: &str) -> Option<u64> {
    reference.rsplit_once('#').and_then(|(_, n)| n.parse().ok())
}

/// The solids and surface models of each product (by the entity number of its
/// `PRODUCT`) as the file links them: product definition, its shape, the shape's
/// representation and the representations related to that one without a placement
/// (`SHAPE_REPRESENTATION_RELATIONSHIP`, as CADENAS and CATIA write them).
fn representation_links(text: &str) -> HashMap<u64, Vec<u64>> {
    // Each simple entity: its type and the entities it refers to, in order.
    let mut entities: HashMap<u64, (String, Vec<u64>)> = HashMap::new();
    let (mut start, mut quoted) = (0, false);
    for (i, c) in text.char_indices() {
        match c {
            '\'' => quoted = !quoted,
            ';' if !quoted => {
                let statement = text[start..i].trim();
                start = i + 1;
                let Some((id, body)) = statement.strip_prefix('#').and_then(|s| s.split_once('=')) else { continue };
                let (Ok(id), body) = (id.trim().parse::<u64>(), body.trim()) else { continue };
                let kind: String = body.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
                let refs = body.split('#').skip(1).filter_map(|r| r.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()).collect();
                entities.insert(id, (kind, refs));
            }
            _ => {}
        }
    }
    let of_kind = |kind: &'static str| entities.iter().filter(move |(_, (k, _))| k == kind).map(|(id, (_, refs))| (*id, refs));
    let first_ref = |kind: &'static str| -> HashMap<u64, u64> { of_kind(kind).filter_map(|(id, refs)| Some((*refs.first()?, id))).collect() };
    // Product → its formation → definition → shape → representations.
    let formations: HashMap<u64, u64> = entities.iter().filter(|(_, (k, _))| k.starts_with("PRODUCT_DEFINITION_FORMATION")).filter_map(|(id, (_, refs))| Some((*refs.last()?, *id))).collect();
    let (definitions, shapes) = (first_ref("PRODUCT_DEFINITION"), first_ref("PRODUCT_DEFINITION_SHAPE"));
    let mut representations: HashMap<u64, Vec<u64>> = HashMap::new();
    for (_, refs) in of_kind("SHAPE_DEFINITION_REPRESENTATION") {
        if let [shape, rep, ..] = refs[..] {
            representations.entry(shape).or_default().push(rep);
        }
    }
    let mut related: HashMap<u64, Vec<u64>> = HashMap::new();
    for (_, refs) in of_kind("SHAPE_REPRESENTATION_RELATIONSHIP") {
        if let [a, b, ..] = refs[..] {
            related.entry(a).or_default().push(b);
            related.entry(b).or_default().push(a);
        }
    }
    let mut out = HashMap::new();
    for (&product, &formation) in &formations {
        let Some(shape) = definitions.get(&formation).and_then(|d| shapes.get(d)) else { continue };
        let mut reps: Vec<u64> = representations.get(shape).cloned().unwrap_or_default();
        let mut i = 0;
        while i < reps.len() {
            for &r in related.get(&reps[i]).into_iter().flatten() {
                if !reps.contains(&r) {
                    reps.push(r);
                }
            }
            i += 1;
        }
        // The items of a representation; the last reference is its context.
        let items: Vec<u64> = reps.iter().filter_map(|r| entities.get(r)).flat_map(|(_, refs)| refs.split_last().map_or(&[][..], |(_, items)| items)).copied().collect();
        out.insert(product, items);
    }
    out
}

/// All shells of the solids and surface models with their placement: once per use
/// in the assembly structure; bodies of no product where they are. Uses of a product
/// whose name `skip` accepts are left out with everything below them. `links` gives a
/// product the solids the decoder left without one ([`representation_links`]).
fn placed_shells(ir: &CadIr, links: &HashMap<u64, Vec<u64>>, skip: &dyn Fn(&str) -> bool) -> Result<Vec<(ImportedSheet, Matrix4)>, String> {
    let mut uses: HashMap<_, Vec<Matrix4>> = HashMap::new();
    let mut left_out = HashSet::new();
    let owned: HashSet<_> = ir.model.product_definitions.iter().flat_map(|p| &p.bodies).collect();
    let bodies_of = |p: &ProductDefinition| -> Vec<_> {
        if !p.bodies.is_empty() {
            return p.bodies.clone();
        }
        let items = p.native_ref.as_deref().and_then(entity).and_then(|n| links.get(&n)).map_or(&[][..], Vec::as_slice);
        ir.model.bodies.iter().filter(|b| !owned.contains(&b.id) && entity(b.id.as_str()).is_some_and(|n| items.contains(&n))).map(|b| b.id.clone()).collect()
    };
    if let Ok(graph) = AssemblyGraph::new(&ir.model.occurrences) {
        let definition = |o: &Occurrence| match &o.prototype {
            PrototypeReference::Local { definition } => ir.model.product_definitions.iter().find(|p| &p.id == definition),
            _ => None,
        };
        let named = |o: &Occurrence| {
            let product = definition(o).into_iter().flat_map(|p| [&p.label, &p.source_name, &p.part_number]);
            [&o.name].into_iter().chain(product).flatten().any(|name| skip(name))
        };
        let skipped = |o: &Occurrence| {
            let mut at = Some(o);
            while let Some(o) = at {
                if named(o) {
                    return true;
                }
                at = match &o.parent {
                    OccurrenceParent::Occurrence { occurrence } => graph.occurrence(occurrence),
                    OccurrenceParent::Root => None,
                };
            }
            false
        };
        for occurrence in &ir.model.occurrences {
            let Some(product) = definition(occurrence) else { continue };
            let bodies = bodies_of(product);
            if skipped(occurrence) {
                left_out.extend(bodies);
                continue;
            }
            let placement = graph.resolved_transform(&occurrence.id).map_or(Matrix4::identity(), |t| matrix(&t));
            for body in bodies {
                uses.entry(body).or_default().push(placement);
            }
        }
    }
    let mut out = Vec::new();
    for (body, imported) in convert(ir, &|b| uses.contains_key(&b.id) || !left_out.contains(&b.id))? {
        let placements = uses.get(&body.id).cloned().unwrap_or_else(|| vec![Matrix4::identity()]);
        for shell in body_shells(imported) {
            out.extend(placements.iter().map(|&m| (shell.clone(), m)));
        }
    }
    Ok(out)
}

/// Converts the bodies `keep` accepts. A body `cadmpeg` cannot convert (e.g. a
/// circular edge without parameter range) fails the conversion of all, so they are
/// then converted one by one and those that fail are left out.
fn convert<'a>(ir: &'a CadIr, keep: &dyn Fn(&Body) -> bool) -> Result<Vec<(&'a Body, ImportedBody)>, String> {
    let bodies: Vec<&Body> = ir.model.bodies.iter().filter(|b| keep(b)).collect();
    let mut part = None;
    let all = if bodies.len() == ir.model.bodies.len() {
        to_bodies(ir, "STEP")
    } else {
        let ir = part.insert(ir.clone());
        ir.model.bodies = bodies.iter().map(|&b| b.clone()).collect();
        to_bodies(ir, "STEP")
    };
    let error = match all {
        Ok(converted) => return Ok(bodies.into_iter().zip(converted).collect()),
        Err(e) => format!("STEP file: {e}"),
    };
    let one = part.get_or_insert_with(|| ir.clone());
    let mut out = Vec::new();
    for body in bodies {
        one.model.bodies = vec![body.clone()];
        if let Some(converted) = to_bodies(one, "STEP").ok().and_then(|mut c| c.pop()) {
            out.push((body, converted));
        }
    }
    if out.is_empty() {
        return Err(error);
    }
    Ok(out)
}

/// STEP gives the arc of a circular edge by its vertices only, which `cadmpeg` does
/// not convert. Its edges run along their curve (it swaps the vertices of edges against
/// the sense of the curve), so the arc runs counter-clockwise about the axis from the
/// start to the end vertex; a closed edge goes once round.
fn fill_arc_ranges(ir: &mut CadIr) {
    let dot = |a: IrVector, b: IrVector| a.x * b.x + a.y * b.y + a.z * b.z;
    let scale = |a: IrVector, k: f64| IrVector { x: a.x * k, y: a.y * k, z: a.z * k };
    let minus = |a: IrVector, b: IrVector| IrVector { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z };
    let unit = |a: IrVector| scale(a, 1.0 / dot(a, a).sqrt());
    let points: HashMap<_, _> = ir.model.points.iter().map(|p| (&p.id, p.position)).collect();
    let vertices: HashMap<_, _> = ir.model.vertices.iter().filter_map(|v| Some((&v.id, *points.get(&v.point)?))).collect();
    let circles: HashMap<_, _> = ir
        .model
        .curves
        .iter()
        .filter_map(|c| match &c.geometry {
            CurveGeometry::Circle { center, axis, ref_direction, .. } => {
                let z = unit(*axis);
                let x = unit(minus(*ref_direction, scale(z, dot(*ref_direction, z))));
                let y = IrVector { x: z.y * x.z - z.z * x.y, y: z.z * x.x - z.x * x.z, z: z.x * x.y - z.y * x.x };
                Some((&c.id, (*center, x, y)))
            }
            _ => None,
        })
        .filter(|(_, (_, x, y))| [x.x, x.y, x.z, y.x, y.y, y.z].iter().all(|v| v.is_finite()))
        .collect();
    let mut ranges = Vec::new();
    for (k, edge) in ir.model.edges.iter().enumerate().filter(|(_, e)| e.param_range.is_none()) {
        let Some(&(c, x, y)) = edge.curve.as_ref().and_then(|id| circles.get(id)) else { continue };
        let (Some(&a), Some(&b)) = (vertices.get(&edge.start), vertices.get(&edge.end)) else { continue };
        let angle = |p: IrPoint| {
            let d = IrVector { x: p.x - c.x, y: p.y - c.y, z: p.z - c.z };
            dot(d, y).atan2(dot(d, x))
        };
        let (start, mut end) = (angle(a), angle(b));
        if edge.start == edge.end || end <= start + 1e-9 {
            end += std::f64::consts::TAU;
        }
        ranges.push((k, [start, end]));
    }
    for (k, range) in ranges {
        ir.model.edges[k].param_range = Some(range);
    }
}

fn shell_size<C, S>(shell: &CompressedShell<Point3, C, S>) -> f64 {
    let bdd: BoundingBox<Point3> = shell.vertices.iter().collect();
    bdd.diameter()
}

fn tessellate<C, S>(shells: &[(CompressedShell<Point3, C, S>, Matrix4)], relative: f64, unit: f64) -> Result<Mesh3d, String>
where
    CompressedShell<Point3, C, S>: RobustMeshableShape,
    <CompressedShell<Point3, C, S> as RobustMeshableShape>::MeshedShape: MeshedShape,
{
    let mut b = MeshBuilder::new();
    for (shell, matrix) in shells {
        let tolerance = f64::max(shell_size(shell) * relative, TOLERANCE);
        let mesh = shell.robust_triangulation(tolerance).to_polygon();
        let pos: Vec<[f32; 3]> = mesh
            .positions()
            .iter()
            .map(|p| {
                let q = matrix.transform_point(*p);
                [(q.x * unit) as f32, (q.y * unit) as f32, (q.z * unit) as f32]
            })
            .collect();
        for face in mesh.face_iter() {
            for k in 1..face.len().saturating_sub(1) {
                b.triangle(pos[face[0].pos], pos[face[k].pos], pos[face[k + 1].pos])?;
            }
        }
    }
    b.finish()
}

/// Tessellates placed shells into a mesh in mm (`unit`: mm per model unit). Fine
/// details are tessellated more coarsely if the mesh would exceed [`MAX_TRIANGLES`].
pub(crate) fn mesh_shells<C, S>(shells: &[(CompressedShell<Point3, C, S>, Matrix4)], unit: f64) -> Result<Mesh3d, String>
where
    CompressedShell<Point3, C, S>: RobustMeshableShape,
    <CompressedShell<Point3, C, S> as RobustMeshableShape>::MeshedShape: MeshedShape,
{
    let mut last = String::new();
    for relative in TOLERANCES {
        match tessellate(shells, relative, unit) {
            Ok(mesh) => return Ok(mesh),
            Err(e) if e.contains(&MAX_TRIANGLES.to_string()) => last = e,
            Err(e) => return Err(e),
        }
    }
    Err(last)
}

/// Reads a STEP file into a mesh in mm. Curves (wire bodies) are left out.
pub fn parse_step(bytes: &[u8]) -> Result<Mesh3d, String> {
    parse_step_without(bytes, &|_| false)
}

/// Reads a STEP file into a mesh in mm without the products whose name `skip` accepts
/// (and everything they are assembled from), e.g. the keep-out bodies of an enclosure.
pub fn parse_step_without(bytes: &[u8], skip: &dyn Fn(&str) -> bool) -> Result<Mesh3d, String> {
    let mut ir = monstertruck_io::cadmpeg::step::decode_to_ir(&mut std::io::Cursor::new(bytes))
        .map_err(|e| format!("not a readable STEP file: {e}"))?;
    ir.model.bodies.retain(|b| matches!(b.kind, BodyKind::Solid | BodyKind::Sheet));
    if ir.model.bodies.is_empty() {
        return Err("the STEP file contains no solids or surface models".into());
    }
    fill_arc_ranges(&mut ir);
    // cadmpeg converts lengths to millimetres while decoding.
    let links = representation_links(&String::from_utf8_lossy(bytes));
    mesh_shells(&placed_shells(&ir, &links, skip)?, 1.0)
}

#[cfg(test)]
mod tests;
