//! Harness interface for machines and tools: the complete harness data as one JSON
//! document (format [`FORMAT`], version [`FORMAT_VERSION`]; connectors with cavities
//! and end treatment, wires with lengths, cut lengths and ends, the 3D topology with
//! protections, fixings and further parts), the wire processing list for cutting,
//! stripping and crimping machines (CSV, in processing order) and the bundle in 3D
//! (glTF, JT). The field names are stable; new fields may be added within a version.

use crate::analysis::Analysis;
use crate::export3d::{EnclosureScene, Solid};
use crate::harness::topology::{self, Topology};
use crate::harness::{self, Harness, HarnessWire};
use crate::model::{natural_cmp, Project};
use openecraft_parts::vec3::{add, box_triangles, cross, dist, norm, sub, Tri, V3};
use serde_json::{json, Value};

pub const FORMAT: &str = "openecraft.harness";
pub const FORMAT_VERSION: u32 = 1;

fn r1(v: f32) -> f64 {
    (f64::from(v) * 10.0).round() / 10.0
}

fn r3(v: f32) -> f64 {
    (f64::from(v) * 1000.0).round() / 1000.0
}

fn point(q: V3) -> Value {
    json!([r1(q[0]), r1(q[1]), r1(q[2])])
}

/// The harnesses named (all if `tag` is empty or missing).
pub fn selected<'a>(prj: &'a Project, tag: Option<&str>) -> Result<Vec<&'a Harness>, String> {
    match tag.map(str::trim).filter(|t| !t.is_empty()) {
        Some(tag) => prj.harnesses.iter().find(|h| h.tag == tag).map(|h| vec![h]).ok_or_else(|| openecraft_i18n::tr_fmt!("No harness {}", tag)),
        None => Ok(prj.harnesses.iter().collect()),
    }
}

/// Where the length of a wire comes from.
fn length_source(w: &HarnessWire, t: &Topology, i: usize) -> &'static str {
    if w.length <= 0.0 {
        "unknown"
    } else if w.routed {
        "panel"
    } else if t.source == topology::Source::Stored && t.wire_length(i).is_some_and(|l| (l - w.length).abs() < 0.01) {
        "topology"
    } else {
        "field"
    }
}

/// Cut length: the length rounded up to the cut step of wires.
pub fn cut_length(prj: &Project, length: f32) -> f32 {
    let step = prj.layout.routing.cut_wire;
    if step > 0.0 && length > 0.0 { (length / step).ceil() * step } else { length }
}

/// The data of one harness.
pub fn harness_data(prj: &Project, an: &Analysis, h: &Harness) -> Value {
    let ws = harness::wires(prj, an, &h.tag);
    let cons = harness::connectors(&ws);
    let t = topology::build(prj, an, h, &ws);
    let plugs = crate::plugs::plug_tags(prj);
    let end = |dev: &str, pin: &str, cs: &str| {
        let (terminal, seal, strip) = harness::end_parts(prj, h, dev, pin, cs);
        json!({ "device": dev, "pin": pin, "terminal": terminal, "seal": seal, "strip_mm": r1(strip) })
    };
    let connectors: Vec<Value> = cons
        .iter()
        .map(|c| {
            let device_end = harness::end_of(h, &c.device, "").filter(|e| e.pin.is_empty());
            let pins: Vec<Value> = c
                .pins
                .iter()
                .map(|(pin, number, other)| {
                    let cs = ws.iter().find(|w| w.number == *number).map(|w| w.cross_section.as_str()).unwrap_or("");
                    let (terminal, seal, strip) = harness::end_parts(prj, h, &c.device, pin, cs);
                    json!({ "pin": pin, "wire": number, "to": other, "terminal": terminal, "seal": seal, "strip_mm": r1(strip) })
                })
                .collect();
            json!({
                "device": c.device,
                "part": prj.devices.get(&c.device).and_then(|d| d.parts.first()).map(|r| r.number.clone()).unwrap_or_default(),
                "description": prj.devices.get(&c.device).map(|d| d.description.clone()).unwrap_or_default(),
                "plug": plugs.contains(&c.device),
                "usage": device_end.map(|e| e.usage.clone()).unwrap_or_default(),
                "cavity_plug": device_end.map(|e| e.cavity_plug.clone()).unwrap_or_default(),
                "node": t.node_of(&c.device).map(|n| t.nodes[n].id.clone()),
                "pins": pins,
            })
        })
        .collect();
    let wires: Vec<Value> = ws
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let segments: Vec<&str> = t.routes[i].iter().flatten().map(|&s| t.segments[s].id.as_str()).collect();
            json!({
                "number": w.number,
                "net": w.net,
                "color": w.color,
                "cross_section": w.cross_section,
                "cross_section_mm2": crate::production::cross_section_mm2(&w.cross_section).map(r3),
                "outer_diameter_mm": r3(harness::outer_diameter(&w.cross_section)),
                "cable": w.cable,
                "core": w.core,
                "length_mm": r1(w.length),
                "length_source": length_source(w, &t, i),
                "dmu_length_mm": t.wire_length(i).map(r1),
                "cut_length_mm": r1(cut_length(prj, w.length)),
                "from": end(&w.from.0, &w.from.1, &w.cross_section),
                "to": end(&w.to.0, &w.to.1, &w.cross_section),
                "segments": segments,
            })
        })
        .collect();
    let nodes: Vec<Value> = t.nodes.iter().map(|n| json!({ "id": n.id, "position": point(n.pos), "connectors": n.connectors, "bend_radius_mm": r1(n.bend_radius) })).collect();
    let segments: Vec<Value> = t
        .segments
        .iter()
        .map(|s| {
            json!({
                "id": s.id, "start": t.nodes[s.start].id, "end": t.nodes[s.end].id, "length_mm": r1(s.length), "physical_length_mm": r1(s.physical),
                "diameter_mm": r1(s.diameter), "points": s.points.iter().map(|q| point(*q)).collect::<Vec<_>>(),
                "wires": s.wires.iter().map(|&w| ws[w].number.clone()).collect::<Vec<_>>(),
            })
        })
        .collect();
    let protections: Vec<Value> = h.protections.iter().map(|p| json!({ "id": p.id, "kind": p.kind.key(), "part": p.part, "segment": p.segment, "start": r3(p.start), "end": r3(p.end) })).collect();
    let fixings: Vec<Value> = h.fixings.iter().map(|f| json!({ "id": f.id, "kind": f.kind.key(), "part": f.part, "segment": f.segment, "location": r3(f.location) })).collect();
    let parts: Vec<Value> = h.parts.iter().map(|p| json!({ "kind": p.kind.key(), "part": p.part, "count": p.count, "description": p.description, "node": p.node })).collect();
    let total: f32 = ws.iter().map(|w| w.length).sum();
    json!({
        "tag": h.tag,
        "description": h.description,
        "part": h.part,
        "field_length_m": r3(h.length),
        "wire_count": ws.len(),
        "wire_length_mm": r1(total),
        "bundle_diameter_mm": r1(if ws.is_empty() { 0.0 } else { harness::bundle_diameter(&ws) }),
        "connectors": connectors,
        "wires": wires,
        "topology": { "source": t.source.key(), "length_mm": r1(t.length()), "nodes": nodes, "segments": segments },
        "protections": protections,
        "fixings": fixings,
        "parts": parts,
    })
}

/// The harness interface document of the harnesses named (all if `tag` is empty).
pub fn document(prj: &Project, an: &Analysis, tag: Option<&str>) -> Result<Value, String> {
    let list: Vec<Value> = selected(prj, tag)?.into_iter().map(|h| harness_data(prj, an, h)).collect();
    Ok(json!({
        "format": FORMAT,
        "version": FORMAT_VERSION,
        "units": { "length": "mm", "coordinates": "mm, panel layout: x right, y down, z from the rear wall" },
        "project": { "name": prj.info.name, "revision": prj.info.revision, "company": prj.info.company, "date": prj.info.date },
        "harnesses": list,
    }))
}

/// Columns of the wire processing list (machine readable, not translated).
pub const PROCESSING_COLUMNS: [&str; 18] = [
    "sequence", "harness", "wire", "cable", "core", "color", "cross_section_mm2", "length_mm", "cut_length_mm", "from", "strip_from_mm", "terminal_from", "seal_from", "to", "strip_to_mm", "terminal_to", "seal_to", "marking",
];

/// A row of the processing list with its sort keys: cross-section, colour, terminals
/// and seals of both ends, cut length.
type ProcessingRow = (f32, String, (String, String, String, String), f32, Vec<String>);

/// The wire processing list: one row per wire for cutting, stripping and crimping
/// machines, in processing order (cross-section, colour, terminals, longest first).
pub fn processing_list(prj: &Project, an: &Analysis, tag: Option<&str>) -> Result<String, String> {
    let mut rows: Vec<ProcessingRow> = Vec::new();
    for h in selected(prj, tag)? {
        for w in harness::wires(prj, an, &h.tag) {
            let a = harness::end_parts(prj, h, &w.from.0, &w.from.1, &w.cross_section);
            let b = harness::end_parts(prj, h, &w.to.0, &w.to.1, &w.cross_section);
            let cs = crate::production::cross_section_mm2(&w.cross_section).unwrap_or(0.0);
            let cut = cut_length(prj, w.length);
            let fields = vec![
                h.tag.clone(),
                w.number.clone(),
                w.cable.clone(),
                w.core.clone(),
                w.color.clone(),
                format!("{}", r3(cs)),
                format!("{}", r1(w.length)),
                format!("{}", r1(cut)),
                harness::joined(&w.from),
                format!("{}", r1(a.2)),
                a.0.clone(),
                a.1.clone(),
                harness::joined(&w.to),
                format!("{}", r1(b.2)),
                b.0.clone(),
                b.1.clone(),
                w.number.clone(),
            ];
            rows.push((cs, w.color.clone(), (a.0, a.1, b.0, b.1), cut, fields));
        }
    }
    rows.sort_by(|x, y| x.0.total_cmp(&y.0).then_with(|| natural_cmp(&x.1, &y.1)).then_with(|| x.2.cmp(&y.2)).then(y.3.total_cmp(&x.3)).then_with(|| natural_cmp(&x.4[1], &y.4[1])));
    let mut out = PROCESSING_COLUMNS.join(";") + "\n";
    for (i, row) in rows.iter().enumerate() {
        out.push_str(&(i + 1).to_string());
        for f in &row.4 {
            out.push(';');
            out.push_str(&openecraft_parts::csv_escape(f));
        }
        out.push('\n');
    }
    Ok(out)
}

// ---------------------------------------------------------------------- 3D

/// Layout coordinates (y down) to the view of the 3D exports (y up).
fn view(q: V3) -> V3 {
    [q[0], -q[1], q[2]]
}

/// A tube of radius `r` along a polyline (octagonal prisms).
fn tube(points: &[V3], r: f32) -> Vec<Tri> {
    let mut out = Vec::new();
    for w in points.windows(2) {
        let (a, b) = (view(w[0]), view(w[1]));
        if dist(a, b) < 1e-3 {
            continue;
        }
        let d = norm(sub(b, a));
        let helper = if d[1].abs() > 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
        let u = norm(cross(d, helper));
        let v = cross(d, u);
        let ring = |c: V3| -> Vec<V3> {
            (0..8)
                .map(|k| {
                    let a = std::f32::consts::TAU * k as f32 / 8.0;
                    add(c, add(u.map(|x| x * r * a.cos()), v.map(|x| x * r * a.sin())))
                })
                .collect()
        };
        let (ra, rb) = (ring(a), ring(b));
        for k in 0..8 {
            let n = (k + 1) % 8;
            out.push([ra[k], ra[n], rb[n]]);
            out.push([ra[k], rb[n], rb[k]]);
        }
    }
    out
}

/// The part of a polyline between the fractions `a` and `b` of its length.
fn sub_polyline(points: &[V3], a: f32, b: f32) -> Vec<V3> {
    let total = topology::polyline_length(points);
    let (a, b) = (a.clamp(0.0, 1.0) * total, b.clamp(0.0, 1.0) * total);
    let mut out = vec![topology::point_at(points, a / total.max(1e-6))];
    let mut run = 0.0;
    for w in points.windows(2) {
        run += dist(w[0], w[1]);
        if run > a && run < b {
            out.push(w[1]);
        }
    }
    out.push(topology::point_at(points, b / total.max(1e-6)));
    out
}

fn cube(at: V3, size: f32) -> Vec<Tri> {
    let c = view(at);
    let h = size / 2.0;
    box_triangles([c[0] - h, c[1] - h, c[2] - h], [c[0] + h, c[1] + h, c[2] + h])
}

/// The solids of a harness: the segments as tubes of their bundle diameter, the
/// protections over them, fixings, connectors and branch points.
pub fn solids(prj: &Project, an: &Analysis, h: &Harness) -> Vec<Solid> {
    let ws = harness::wires(prj, an, &h.tag);
    let t = topology::build(prj, an, h, &ws);
    let solid = |name: String, color: [u8; 3], triangles: Vec<Tri>| Solid { name, color, transparent: false, triangles, colors: Vec::new() };
    let mut out = Vec::new();
    for s in &t.segments {
        out.push(solid(s.id.clone(), [60, 60, 60], tube(&s.points, (s.diameter / 2.0).max(1.0))));
    }
    for p in &h.protections {
        let segs: Vec<usize> = if p.segment.is_empty() { (0..t.segments.len()).collect() } else { t.segment_index(&p.segment).into_iter().collect() };
        let color = match p.kind {
            harness::ProtectionKind::Tape => [20, 20, 20],
            harness::ProtectionKind::Tube | harness::ProtectionKind::HeatShrink => [30, 30, 30],
            harness::ProtectionKind::CorrugatedTube => [90, 90, 95],
            harness::ProtectionKind::Sleeve => [40, 40, 120],
        };
        let tris: Vec<Tri> = segs.iter().flat_map(|&si| tube(&sub_polyline(&t.segments[si].points, p.start.min(p.end), p.end.max(p.start)), t.segments[si].diameter / 2.0 + 1.5)).collect();
        out.push(solid(p.id.clone(), color, tris));
    }
    for f in &h.fixings {
        if let Some(si) = t.segment_index(&f.segment) {
            let s = &t.segments[si];
            out.push(solid(f.id.clone(), [235, 235, 225], cube(topology::point_at(&s.points, f.location), s.diameter + 6.0)));
        }
    }
    for n in &t.nodes {
        if n.connectors.is_empty() {
            let d = t.segments.iter().filter(|s| t.nodes[s.start].id == n.id || t.nodes[s.end].id == n.id).map(|s| s.diameter).fold(0.0, f32::max);
            out.push(solid(n.id.clone(), [60, 60, 60], cube(n.pos, d.max(2.0))));
        } else {
            out.push(solid(n.connectors.join(" "), [200, 170, 60], cube(n.pos, 20.0)));
        }
    }
    out
}

fn scenes(prj: &Project, an: &Analysis, tag: Option<&str>) -> Result<Vec<EnclosureScene>, String> {
    Ok(selected(prj, tag)?.into_iter().map(|h| EnclosureScene { name: h.tag.clone(), offset: [0.0; 3], solids: solids(prj, an, h) }).collect())
}

/// The harnesses in 3D as glTF.
pub fn gltf(prj: &Project, an: &Analysis, tag: Option<&str>) -> Result<String, String> {
    Ok(crate::export3d::gltf_of(&prj.info.name, &scenes(prj, an, tag)?))
}

/// The harnesses in 3D as JT.
pub fn jt(prj: &Project, an: &Analysis, tag: Option<&str>) -> Result<Vec<u8>, String> {
    Ok(crate::export3d::jt_of(if prj.info.name.is_empty() { "Harnesses" } else { &prj.info.name }, &scenes(prj, an, tag)?))
}

#[cfg(test)]
mod tests;
