use super::*;
use crate::analysis::Analysis;
use crate::integration;
use crate::sample::demo_project;
use std::collections::BTreeMap;

/// The devices of the layout the macro can move: (enclosure, item).
fn devices(prj: &Project) -> Vec<(&Enclosure, &PanelItem)> {
    prj.layout.enclosures.iter().flat_map(|e| e.items.iter().map(move |i| (e, i))).filter(|(_, i)| i.kind == ItemKind::Device && !i.tag.is_empty() && i.terminal.is_empty()).collect()
}

/// A document as FreeCAD saves it after the macro ran: the objects with the
/// properties the macro gave them and the placement `(base, quaternion)` of each.
fn document(prj: &Project, placement: impl Fn(&str) -> ([f32; 3], [f32; 4])) -> Vec<u8> {
    let mut objects = String::new();
    let mut data = String::new();
    for (n, (e, it)) in devices(prj).into_iter().enumerate() {
        let (a, b) = panel::item_box(e, it);
        let c = z_up([(a[0] + b[0]) / 2.0, e.height - (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0]);
        let [ax, ay, az] = axes(e, it);
        let (base, q) = placement(&it.tag);
        let vector = |name: &str, v: [f32; 3]| format!("<Property name=\"{name}\" type=\"App::PropertyVector\" group=\"OpenECraft\" doc=\"\" attr=\"0\" ro=\"0\" hide=\"0\"><PropertyVector valueX=\"{}\" valueY=\"{}\" valueZ=\"{}\"/></Property>\n", v[0], v[1], v[2]);
        let float = |name: &str, v: f32| format!("<Property name=\"{name}\" type=\"App::PropertyFloat\" group=\"OpenECraft\" doc=\"\" attr=\"0\" ro=\"0\" hide=\"0\"><Float value=\"{v}\"/></Property>\n");
        objects.push_str(&format!("<Object type=\"Part::Feature\" name=\"Obj{n}\" id=\"{}\" />\n", n + 1));
        data.push_str(&format!(
            "<Object name=\"Obj{n}\">\n<Properties Count=\"11\" TransientCount=\"0\">\n<Property name=\"Label\" type=\"App::PropertyString\" status=\"134217728\"><String value=\"{tag}\"/></Property>\n<Property name=\"OEC_Tag\" type=\"App::PropertyString\" group=\"OpenECraft\" doc=\"\" attr=\"0\" ro=\"0\" hide=\"0\"><String value=\"{tag}\"/></Property>\n{}{}{}{}{}{}{}<Property name=\"Placement\" type=\"App::PropertyPlacement\" status=\"8388608\"><PropertyPlacement Px=\"{}\" Py=\"{}\" Pz=\"{}\" Q0=\"{}\" Q1=\"{}\" Q2=\"{}\" Q3=\"{}\" A=\"0\" Ox=\"0\" Oy=\"0\" Oz=\"1\"/></Property>\n</Properties>\n</Object>\n",
            float("OEC_X", it.x),
            float("OEC_Y", it.y),
            float("OEC_Z", it.z),
            vector("OEC_Center", c),
            vector("OEC_AxisX", ax),
            vector("OEC_AxisY", ay),
            vector("OEC_AxisZ", az),
            base[0],
            base[1],
            base[2],
            q[0],
            q[1],
            q[2],
            q[3],
            tag = xml::esc(&it.tag),
        ));
    }
    // An object of the mechanical design that is none of the layout.
    objects.push_str("<Object type=\"Part::Box\" name=\"Bracket\" id=\"999\" />\n");
    data.push_str("<Object name=\"Bracket\">\n<Properties Count=\"1\" TransientCount=\"0\">\n<Property name=\"Length\" type=\"App::PropertyLength\"><Float value=\"10\"/></Property>\n</Properties>\n</Object>\n");
    let xml = format!(
        "<?xml version='1.0' encoding='utf-8'?>\n<!-- FreeCAD Document -->\n<Document SchemaVersion=\"4\" ProgramVersion=\"1.0.2R39319 (Git)\" FileVersion=\"1\">\n<Properties Count=\"1\" TransientCount=\"0\">\n<Property name=\"Label\" type=\"App::PropertyString\"><String value=\"Demo\"/></Property>\n</Properties>\n<Objects Count=\"1\" Dependencies=\"1\">\n{objects}</Objects>\n<ObjectData Count=\"1\">\n{data}</ObjectData>\n</Document>\n"
    );
    let mut zip = crate::zip::ZipWriter::new("2026-11-27");
    zip.add("Document.xml", xml.as_bytes());
    zip.add("GuiDocument.xml", b"<Document/>");
    zip.finish()
}

const STILL: ([f32; 3], [f32; 4]) = ([0.0; 3], [0.0, 0.0, 0.0, 1.0]);

#[test]
fn the_macro_builds_every_enclosure_and_names_every_device() {
    let prj = demo_project();
    let text = macro_text(&prj);
    assert!(text.starts_with("# -*- coding: utf-8 -*-\n# Panel layout "));
    assert!(text.contains("import FreeCAD as App") && text.contains("doc = App.newDocument(\"OpenECraft\")"));
    assert_eq!(text.matches("group = _enclosure(").count(), prj.layout.enclosures.len());
    let adds: Vec<&str> = text.lines().filter(|l| l.starts_with("_add(")).collect();
    for (_, it) in devices(&prj) {
        let tag = serde_json::to_string(&it.tag).unwrap();
        assert_eq!(adds.iter().filter(|l| l.contains(&format!("(\"OEC_Tag\", {tag})"))).count(), 1, "{}", it.tag);
    }
    // Rails and enclosure components are objects without a tag.
    assert!(adds.len() > devices(&prj).len());
    for line in &adds {
        // Balanced brackets; the facets refer to listed points.
        assert_eq!(line.matches('(').count(), line.matches(')').count(), "{line}");
        assert!(line.ends_with("])"), "{line}");
        let (points, rest) = line.split_once(", [").unwrap().1.split_once("], [").unwrap();
        let points = points.matches('(').count();
        let facets = rest.split("], [").next().unwrap();
        let highest = facets.split(|c: char| !c.is_ascii_digit()).filter_map(|n| n.parse::<usize>().ok()).max().unwrap();
        assert!(highest < points, "{line}");
    }
    // The header and footer are plain ASCII Python.
    assert!(HEADER.is_ascii() && FOOTER.is_ascii());
    assert!(!HEADER.contains('\t') && !FOOTER.contains('\t'));
}

#[test]
fn the_macro_itself_reads_back_as_the_exported_places() {
    let prj = demo_project();
    let csv = macro_placements(&macro_text(&prj)).unwrap();
    let rows = openecraft_parts::parse_csv(&csv);
    assert_eq!(rows[0], ["tag", "x", "y", "z"]);
    assert_eq!(rows.len() - 1, devices(&prj).len());
    for (_, it) in devices(&prj) {
        let row = rows.iter().find(|r| r[0] == it.tag).unwrap();
        assert_eq!(row[1].parse::<f32>().unwrap(), (it.x * 1000.0).round() / 1000.0, "{}", it.tag);
    }
    assert_eq!(macro_placements("tag;x\n-K1;1\n"), None);
}

#[test]
fn directions_follow_the_face_of_a_device() {
    let prj = demo_project();
    for (e, it) in devices(&prj) {
        let a = axes(e, it);
        // Unit directions along the axes of FreeCAD, at right angles to each other.
        for k in 0..3 {
            assert_eq!(dot(a[k], a[k]), 1.0, "{} {:?}", it.tag, a);
            assert_eq!(dot(a[k], a[(k + 1) % 3]), 0.0, "{} {:?}", it.tag, a);
        }
    }
    // On the mounting plate x runs to the right and y downwards.
    let (e, it) = devices(&prj).into_iter().find(|(_, i)| i.face == panel::Face::Plate).expect("a device on the mounting plate");
    let a = axes(e, it);
    assert_eq!((a[0], a[1]), ([1.0, 0.0, 0.0], [0.0, 0.0, -1.0]));
}

#[test]
fn a_saved_document_tells_where_devices_were_moved() {
    let prj = demo_project();
    let (e, it) = devices(&prj).into_iter().find(|(_, i)| i.face == panel::Face::Plate).unwrap();
    let (tag, a) = (it.tag.clone(), axes(e, it));
    // 30 mm along the layout's x and 12 mm against its y.
    let base = [0, 1, 2].map(|k| 30.0 * a[0][k] - 12.0 * a[1][k]);
    let bytes = document(&prj, |t| if t == tag { (base, STILL.1) } else { STILL });
    assert!(is_document(&bytes));
    let rows = openecraft_parts::parse_csv(&placements(&bytes).unwrap());
    assert_eq!(rows.len() - 1, devices(&prj).len());
    for (_, other) in devices(&prj) {
        let row = rows.iter().find(|r| r[0] == other.tag).unwrap();
        let at: Vec<f32> = row[1..].iter().map(|v| v.parse().unwrap()).collect();
        let expected = if other.tag == tag { [other.x + 30.0, other.y - 12.0, other.z] } else { [other.x, other.y, other.z] };
        for k in 0..3 {
            assert!((at[k] - expected[k]).abs() < 0.002, "{} {at:?} {expected:?}", other.tag);
        }
    }
}

#[test]
fn a_turn_moves_a_device_as_far_as_its_middle_went() {
    let prj = demo_project();
    let (e, it) = devices(&prj).into_iter().find(|(_, i)| i.face == panel::Face::Plate).unwrap();
    let tag = it.tag.clone();
    let (a, b) = panel::item_box(e, it);
    let c = z_up([(a[0] + b[0]) / 2.0, e.height - (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0]);
    // Half a turn about the vertical through the origin, then back to where it stood,
    // 20 mm further along x.
    let q = [0.0, 0.0, 1.0, 0.0];
    assert_eq!(turned(q, [1.0, 2.0, 3.0]), [-1.0, -2.0, 3.0]);
    let base = [2.0 * c[0] + 20.0, 2.0 * c[1], 0.0];
    let rows = openecraft_parts::parse_csv(&placements(&document(&prj, |t| if t == tag { (base, q) } else { STILL })).unwrap());
    let row = rows.iter().find(|r| r[0] == tag).unwrap();
    assert!((row[1].parse::<f32>().unwrap() - (it.x + 20.0)).abs() < 0.01, "{row:?}");
    assert!((row[2].parse::<f32>().unwrap() - it.y).abs() < 0.01, "{row:?}");
}

#[test]
fn other_files_are_refused() {
    assert!(!is_document(b"tag;x\n"));
    assert!(placements(b"tag;x\n").is_err());
    let mut zip = crate::zip::ZipWriter::new("2026-11-27");
    zip.add("Document.xml", b"<Document><ObjectData><Object name=\"Box\"><Properties/></Object></ObjectData></Document>");
    let empty = zip.finish();
    assert!(is_document(&empty));
    assert!(placements(&empty).is_err());
    let mut zip = crate::zip::ZipWriter::new("2026-11-27");
    zip.add("other.txt", b"x");
    assert!(placements(&zip.finish()).is_err());
}

#[test]
fn the_connector_takes_positions_over_from_freecad() {
    let mut prj = demo_project();
    let an = Analysis::checked(&prj);
    let c = integration::connector("freecad").unwrap();
    let exported = integration::export(&mut prj, &an, c, "2026-11-27");
    assert!(exported.contains("_add(group, "));
    let (e, it) = devices(&prj).into_iter().find(|(_, i)| i.face == panel::Face::Plate).unwrap();
    let (tag, x, a) = (it.tag.clone(), it.x, axes(e, it));
    // Nothing moved: nothing changes.
    let still = integration::decode(c, &document(&prj, |_| STILL)).unwrap();
    let p = integration::import(&prj, &an, c, &still, &BTreeMap::new()).unwrap();
    assert!(p.changes.is_empty() && p.unmatched.is_empty(), "{:?}", p.changes);
    assert_eq!(p.records, devices(&prj).len());
    // One device moved 40 mm along x.
    let moved = integration::decode(c, &document(&prj, |t| if t == tag { (a[0].map(|v| v * 40.0), STILL.1) } else { STILL })).unwrap();
    let p = integration::import(&prj, &an, c, &moved, &BTreeMap::new()).unwrap();
    assert_eq!(p.changes.len(), 1, "{:?}", p.changes);
    assert_eq!((p.changes[0].key.as_str(), p.changes[0].field, p.changes[0].conflict), (tag.as_str(), integration::Field::X, false));
    assert_eq!(integration::apply(&mut prj, &an, c, &p.changes, false, "2026-11-27"), 1);
    let now = devices(&prj).into_iter().find(|(_, i)| i.tag == tag).unwrap().1.x;
    assert_eq!(now, x + 40.0);
}
