//! KiCad symbol libraries (`.kicad_sym`). Symbols here have their connection points on
//! a 4 mm grid with y downwards, KiCad's on a 1.27 mm (50 mil) grid with y upwards, by
//! its convention on every second line of it. Graphics and connection points are
//! scaled by the ratio of the grids, so that pins stay on the grid of the other tool
//! and wires attach: 4 mm here are 2.54 mm in a library written for KiCad ([`SCALE`]),
//! 1.27 mm of a library of KiCad are 4 mm here ([`KICAD_SCALE`]). Texts keep a
//! readable size instead: the usual 2.5 mm here are the usual 1.27 mm there.

use crate::geom::{p, P};
use crate::model::symlib::{UserPin, UserSymbol};
use crate::model::Project;
use crate::sexpr::{self, number, quote, Sexpr};
use crate::symbols::{self, Anchor, FieldKind, Func, Prim, SymbolDef, SymbolRef};
use crate::symlib;
use openecraft_i18n::tr;
use serde_json::json;
use std::collections::BTreeSet;
use std::fmt::Write;

/// Millimetres in a library written for KiCad for a millimetre here: the 4 mm grid is
/// KiCad's 100 mil grid, the size its own symbols have.
pub const SCALE: f32 = 2.54 / crate::geom::GRID;
/// Millimetres in a library of KiCad for a millimetre here: every line of KiCad's
/// 50 mil grid is a line of the 4 mm grid, whatever line a pin is on.
pub const KICAD_SCALE: f32 = 1.27 / crate::geom::GRID;
/// Name this module writes as the generator of a library: its own libraries are read
/// back at the scale they were written with.
const GENERATOR: &str = "openecraft";
/// KiCad's default text size.
const FONT: f32 = 1.27;
/// The text size here that stands for KiCad's default.
const TEXT: f32 = 2.5;
/// Segments of a full circle when an arc or a curve becomes a polyline.
const SEGMENTS: f32 = 32.0;

fn xy(q: P) -> String {
    format!("{} {}", number(q.x * SCALE), number(-q.y * SCALE))
}

fn stroke(width: f32, dashed: bool) -> String {
    format!("(stroke (width {}) (type {}))", number(width * SCALE), if dashed { "dash" } else { "default" })
}

fn fill(filled: bool) -> &'static str {
    if filled { "(fill (type outline))" } else { "(fill (type none))" }
}

fn polyline(out: &mut String, pts: &[P], closed: bool, filled: bool, width: f32, dashed: bool) {
    if pts.len() < 2 {
        return;
    }
    out.push_str("      (polyline (pts");
    for q in pts.iter().chain(pts.first().filter(|_| closed)) {
        let _ = write!(out, " (xy {})", xy(*q));
    }
    let _ = writeln!(out, ") {} {})", stroke(width, dashed), fill(filled));
}

fn font(size: f32) -> String {
    format!("(font (size {0} {0}))", number(size))
}

fn justify(anchor: Anchor) -> &'static str {
    match anchor {
        Anchor::Left => " (justify left)",
        Anchor::Right => " (justify right)",
        Anchor::Center => "",
    }
}

/// The symbol `def` as a symbol of a KiCad library named `name`.
fn symbol(out: &mut String, def: &SymbolDef, name: &str) {
    let letter: String = def.letter.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    // A grid line away from the symbol: KiCad writes the pin numbers beside the pins.
    let (top, bottom) = (def.bounds.min.y - crate::geom::GRID, def.bounds.max.y + crate::geom::GRID);
    // The tag where the symbol shows it, otherwise above the symbol.
    let (tag_at, tag_anchor) = def
        .prims
        .iter()
        .find_map(|x| match x {
            Prim::Field { kind: FieldKind::Tag, pos, anchor, .. } => Some((*pos, *anchor)),
            _ => None,
        })
        .unwrap_or((p(0.0, top), Anchor::Center));
    // A potential is a power symbol in KiCad: its value names the net, its pin shows
    // no number.
    let power = def.func == Func::Potential;
    let _ = writeln!(out, "  (symbol {}{} (in_bom yes) (on_board yes)", quote(name), if power { " (power) (pin_numbers hide) (pin_names (offset 0) hide)" } else { "" });
    let property = |out: &mut String, id: usize, key: &str, value: &str, at: P, anchor: Anchor, shown: bool| {
        let _ = writeln!(out, "    (property {} {} (id {id}) (at {} 0) (effects {}{}{}))", quote(key), quote(value), xy(at), font(FONT), justify(anchor), if shown { "" } else { " hide" });
    };
    if power {
        property(out, 0, "Reference", "#PWR", p(0.0, top), Anchor::Center, false);
        property(out, 1, "Value", if def.default_label.is_empty() { def.name } else { def.default_label }, p(0.0, bottom), Anchor::Center, true);
    } else {
        property(out, 0, "Reference", if letter.is_empty() { "A" } else { &letter }, tag_at, tag_anchor, true);
        property(out, 1, "Value", def.name, p(0.0, bottom), Anchor::Center, true);
    }
    property(out, 2, "Footprint", "", p(0.0, 0.0), Anchor::Center, false);
    property(out, 3, "Datasheet", "", p(0.0, 0.0), Anchor::Center, false);
    property(out, 4, "ki_description", def.name, p(0.0, 0.0), Anchor::Center, false);
    let _ = writeln!(out, "    (symbol {}", quote(&format!("{name}_0_1")));
    for prim in &def.prims {
        match prim {
            Prim::Line { a, b, dashed } => polyline(out, &[*a, *b], false, false, 0.0, *dashed),
            Prim::Poly { pts, closed, fill } => polyline(out, pts, *closed, *fill, 0.0, false),
            Prim::Path { pts, closed, fill, width, dashed, .. } => polyline(out, pts, *closed, *fill, *width, *dashed),
            Prim::Circle { c, r, fill: filled, .. } => {
                let _ = writeln!(out, "      (circle (center {}) (radius {}) {} {})", xy(*c), number(r * SCALE), stroke(0.0, false), fill(*filled));
            }
            Prim::Text { pos, text, size, anchor, angle, .. } if !text.trim().is_empty() => {
                // Libraries of this version give the angle in tenths of a degree.
                let _ = writeln!(out, "      (text {} (at {} {}) (effects {}{}))", quote(text), xy(*pos), number(angle.unwrap_or(0.0) * 10.0), font((size / TEXT * FONT).max(0.8)), justify(*anchor));
            }
            _ => {}
        }
    }
    out.push_str("    )\n");
    let _ = writeln!(out, "    (symbol {}", quote(&format!("{name}_1_1")));
    let c = p((def.bounds.min.x + def.bounds.max.x) / 2.0, (def.bounds.min.y + def.bounds.max.y) / 2.0);
    for (i, pin) in def.pins.iter().enumerate().filter(|(_, pin)| !pin.internal) {
        // The pin has no length (the symbol draws its own leads); its direction, towards
        // the symbol, places the number.
        let d = pin.pos - c;
        let angle = if d.x.abs() > d.y.abs() {
            if d.x < 0.0 { 0 } else { 180 }
        } else if d.y < 0.0 {
            270
        } else {
            90
        };
        let label = if pin.label.trim().is_empty() { (i + 1).to_string() } else { pin.label.trim().to_string() };
        let _ = writeln!(
            out,
            "      (pin {} line (at {} {angle}) (length 0) (name \"~\" (effects {})) (number {} (effects {})))",
            if power { "power_in" } else { "passive" },
            xy(pin.pos),
            font(FONT),
            quote(&label),
            font(FONT)
        );
    }
    out.push_str("    )\n  )\n");
}

/// A KiCad symbol library with the symbols `defs`, named by their keys.
pub fn export(defs: &[&SymbolDef]) -> String {
    let mut out = String::from("(kicad_symbol_lib (version 20211014) (generator ");
    out.push_str(GENERATOR);
    out.push_str(")\n");
    let mut names = BTreeSet::new();
    for def in defs {
        let mut name = super::plain_name(def.key.trim_start_matches(symlib::PREFIX));
        while !names.insert(name.clone()) {
            name.push('_');
        }
        symbol(&mut out, def, &name);
    }
    out.push_str(")\n");
    out
}

/// The symbols of a project as a KiCad library: those placed on its pages and its
/// user symbols, or with `all` the whole built-in library as well.
pub fn export_project(prj: &Project, all: bool) -> String {
    let mut keys: Vec<String> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut add = |key: &str| {
        if seen.insert(key.to_string()) {
            keys.push(key.to_string());
        }
    };
    if all {
        symbols::library().iter().for_each(|d| add(d.key));
    }
    prj.pages.iter().flat_map(|pg| &pg.symbols).for_each(|s| add(symlib::scale_of(&s.sym).0));
    prj.symbols.iter().for_each(|u| add(&u.key));
    let refs: Vec<SymbolRef> = keys.iter().filter_map(|k| prj.symbol(k)).collect();
    let defs: Vec<&SymbolDef> = refs.iter().map(|r| &**r).filter(|d| d.key != symbols::DEVICE_BOX).collect();
    export(&defs)
}

/// A point of a KiCad symbol in the coordinates of a symbol here; `k`: millimetres of
/// the library for a millimetre here.
fn local(k: f32, x: f32, y: f32) -> P {
    p(x / k, -y / k)
}

fn point(k: f32, e: &Sexpr) -> Option<P> {
    Some(local(k, e.num(0)?, e.num(1)?))
}

/// The points of an arc through three points.
fn arc(a: P, m: P, b: P) -> Vec<P> {
    let d = 2.0 * (a.x * (m.y - b.y) + m.x * (b.y - a.y) + b.x * (a.y - m.y));
    if d.abs() < 1e-6 {
        return vec![a, b];
    }
    let sq = |q: P| q.x * q.x + q.y * q.y;
    let c = p((sq(a) * (m.y - b.y) + sq(m) * (b.y - a.y) + sq(b) * (a.y - m.y)) / d, (sq(a) * (b.x - m.x) + sq(m) * (a.x - b.x) + sq(b) * (m.x - a.x)) / d);
    let r = c.dist(a);
    let angle = |q: P| (q.y - c.y).atan2(q.x - c.x);
    let turn = std::f32::consts::TAU;
    let (a0, am, a1) = (angle(a), angle(m), angle(b));
    // The sweep from the start to the end that passes the middle point.
    let ccw = (a1 - a0).rem_euclid(turn);
    let sweep = if (am - a0).rem_euclid(turn) <= ccw { ccw } else { ccw - turn };
    let n = ((sweep.abs() / turn * SEGMENTS).ceil() as usize).max(2);
    (0..=n).map(|i| {
        let t = a0 + sweep * i as f32 / n as f32;
        p(c.x + r * t.cos(), c.y + r * t.sin())
    }).collect()
}

/// The points of a cubic Bézier curve.
fn bezier(c: &[P]) -> Vec<P> {
    let [a, b, d, e] = [c[0], c[1], c[2], c[3]];
    (0..=16)
        .map(|i| {
            let t = i as f32 / 16.0;
            let u = 1.0 - t;
            a * (u * u * u) + b * (3.0 * u * u * t) + d * (3.0 * u * t * t) + e * (t * t * t)
        })
        .collect()
}

fn poly(pts: Vec<P>, filled: bool) -> Option<Prim> {
    match pts.len() {
        0 | 1 => None,
        2 => Some(Prim::Line { a: pts[0], b: pts[1], dashed: false }),
        _ => {
            let closed = pts[0].dist(pts[pts.len() - 1]) < 1e-3;
            let pts = if closed { pts[..pts.len() - 1].to_vec() } else { pts };
            Some(Prim::Poly { pts, closed, fill: filled && closed })
        }
    }
}

/// Size of a text here for a font size of KiCad.
fn text_size(e: &Sexpr) -> f32 {
    let size = e.child("effects").and_then(|x| x.child("font")).and_then(|f| f.child("size")).and_then(|s| s.num(0)).unwrap_or(FONT);
    (size / FONT * TEXT * 100.0).round() / 100.0
}

fn graphic(k: f32, g: &Sexpr) -> Option<Prim> {
    let point = |e: &Sexpr| point(k, e);
    let filled = g.child("fill").and_then(|f| f.value("type")).is_some_and(|t| t != "none");
    match g.name() {
        "polyline" => poly(g.child("pts")?.children("xy").filter_map(point).collect(), filled),
        "rectangle" => {
            let (a, b) = (point(g.child("start")?)?, point(g.child("end")?)?);
            Some(Prim::Poly { pts: vec![a, p(b.x, a.y), b, p(a.x, b.y)], closed: true, fill: filled })
        }
        "circle" => Some(Prim::Circle { c: point(g.child("center")?)?, r: g.child("radius")?.num(0)? / k, fill: filled, color: None }),
        "arc" => Some(Prim::Poly { pts: arc(point(g.child("start")?)?, point(g.child("mid")?)?, point(g.child("end")?)?), closed: false, fill: false }),
        "bezier" => {
            let c: Vec<P> = g.child("pts")?.children("xy").filter_map(point).collect();
            (c.len() == 4).then(|| Prim::Poly { pts: bezier(&c), closed: false, fill: false })
        }
        "text" => {
            let at = g.child("at")?;
            // Older libraries give the angle in tenths of a degree.
            let angle = at.num(2).map(|a| if a.abs() > 360.0 { a / 10.0 } else { a }).filter(|a| *a != 0.0);
            let justify = g.child("effects").and_then(|x| x.child("justify"));
            let anchor = match justify {
                Some(j) if j.flag("left") => Anchor::Left,
                Some(j) if j.flag("right") => Anchor::Right,
                _ => Anchor::Center,
            };
            Some(Prim::Text { pos: point(at)?, text: g.arg(0)?.to_string(), size: text_size(g), anchor, angle, color: None })
        }
        _ => None,
    }
}

/// Unit and body style of a sub-symbol from its name (`LM358_2_1`: unit 2, style 1).
fn unit_of(sub: &str) -> (u32, u32) {
    let mut parts = sub.rsplit('_');
    let style = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let unit = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (unit, style)
}

/// Graphics and pins of one unit of a symbol: its own and the shared ones (unit 0).
fn unit(u: &mut UserSymbol, sym: &Sexpr, wanted: u32, names_shown: bool, name_offset: f32, k: f32) {
    for sub in sym.children("symbol") {
        let (n, style) = unit_of(sub.arg(0).unwrap_or_default());
        if (n != 0 && n != wanted) || style > 1 {
            continue;
        }
        for g in sub.args() {
            if g.name() != "pin" {
                u.prims.extend(graphic(k, g));
                continue;
            }
            let Some(at) = g.child("at") else { continue };
            let (Some(x), Some(y)) = (at.num(0), at.num(1)) else { continue };
            if g.flag("hide") {
                continue;
            }
            let angle = at.num(2).unwrap_or(0.0);
            let length = g.child("length").and_then(|l| l.num(0)).unwrap_or(0.0);
            let (dx, dy) = (angle.to_radians().cos(), angle.to_radians().sin());
            let (pos, end) = (local(k, x, y), local(k, x + dx * length, y + dy * length));
            if length > 0.0 {
                u.prims.push(Prim::Line { a: pos, b: end, dashed: false });
            }
            let name = g.child("name").and_then(|n| n.arg(0)).unwrap_or_default();
            if names_shown && !name.is_empty() && name != "~" {
                // The name inside the symbol, where the pin ends.
                let inside = local(k, x + dx * (length + name_offset), y + dy * (length + name_offset));
                let size = g.child("name").map_or(TEXT, text_size);
                let (anchor, angle) = match angle.rem_euclid(360.0).round() as i32 {
                    0 => (Anchor::Left, None),
                    180 => (Anchor::Right, None),
                    90 => (Anchor::Left, Some(90.0)),
                    _ => (Anchor::Right, Some(90.0)),
                };
                // KiCad centres the text on the pin; here the position is the base line.
                let pos = if angle.is_none() { inside + p(0.0, size * 0.35) } else { inside + p(size * 0.35, 0.0) };
                u.prims.push(Prim::Text { pos, text: name.replace(['{', '}'], "").replace('~', ""), size, anchor, angle, color: None });
            }
            // Wires attach on the grid: a pin KiCad has between its grid lines gets a lead
            // to the next one.
            let grid = |v: f32| (v / crate::geom::GRID).round() * crate::geom::GRID;
            let at = p(grid(pos.x), grid(pos.y));
            if (at.x - pos.x).abs() > 0.01 || (at.y - pos.y).abs() > 0.01 {
                u.prims.push(Prim::Line { a: pos, b: at, dashed: false });
            }
            // Pins KiCad stacks on one point are one connection point, with the first number.
            if !u.pins.iter().any(|b| b.pos.same(at)) {
                u.pins.push(UserPin { pos: at, label: g.child("number").and_then(|n| n.arg(0)).unwrap_or_default().to_string(), internal: false });
            }
        }
    }
}

fn slug(s: &str) -> String {
    s.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>().split('_').filter(|w| !w.is_empty()).collect::<Vec<_>>().join("_")
}

/// A key no symbol of the library has yet: names that differ only in their punctuation
/// (`D_Bridge_+AA-`, `D_Bridge_-AA+`) get a number.
fn unique(key: String, used: &mut std::collections::HashSet<String>) -> String {
    let (mut out, mut n) = (key.clone(), 1);
    while !used.insert(out.clone()) {
        n += 1;
        out = format!("{key}_{n}");
    }
    out
}

/// The symbols of a KiCad symbol library as user symbols of the library `library`
/// (empty: `KiCad`). A symbol of several units gives a symbol per unit (`LM358 A`,
/// `LM358 B`), a power symbol a potential. The keys are made from the names, so that
/// reading the library again replaces the symbols instead of adding them twice.
pub fn import(text: &str, library: &str) -> Result<Vec<UserSymbol>, String> {
    let root = sexpr::parse(text)?;
    if root.name() != "kicad_symbol_lib" {
        return Err(tr("Not a KiCad symbol library (.kicad_sym)").into());
    }
    let library = if library.trim().is_empty() { "KiCad" } else { library.trim() };
    let k = if root.value("generator") == Some(GENERATOR) { SCALE } else { KICAD_SCALE };
    let all: Vec<&Sexpr> = root.children("symbol").collect();
    let property = |sym: &Sexpr, key: &str| sym.children("property").find(|x| x.arg(0) == Some(key)).and_then(|x| x.arg(1)).unwrap_or_default().to_string();
    let mut out = Vec::new();
    let mut used = std::collections::HashSet::new();
    for sym in &all {
        let Some(name) = sym.arg(0) else { continue };
        // A derived symbol has the graphics of its parent and properties of its own.
        let body = sym.value("extends").and_then(|parent| all.iter().find(|s| s.arg(0) == Some(parent))).unwrap_or(sym);
        let mut units: Vec<u32> = body.children("symbol").map(|s| unit_of(s.arg(0).unwrap_or_default()).0).filter(|u| *u > 0).collect();
        units.sort_unstable();
        units.dedup();
        if units.is_empty() {
            units.push(1);
        }
        let names = body.child("pin_names");
        let names_shown = !names.is_some_and(|n| n.flag("hide"));
        let name_offset = names.and_then(|n| n.child("offset")).and_then(|o| o.num(0)).unwrap_or(0.508);
        let reference = property(sym, "Reference");
        let power = body.flag("power") || reference.starts_with('#');
        for &n in &units {
            let letter_of_unit = char::from_u32('A' as u32 + (n - 1) % 26).unwrap_or('A');
            let (suffix, key_suffix) = if units.len() > 1 { (format!(" {letter_of_unit}"), format!("_{n}")) } else { Default::default() };
            let description = [property(sym, "Description"), property(sym, "ki_description")].into_iter().find(|d| !d.is_empty()).unwrap_or_default();
            let mut u = UserSymbol {
                key: unique(format!("{}kicad_{}_{}{key_suffix}", symlib::PREFIX, slug(library), slug(name)), &mut used),
                name: format!("{name}{suffix}"),
                library: library.to_string(),
                letter: reference.chars().filter(|c| c.is_ascii_alphabetic()).collect(),
                show_pin_labels: !body.child("pin_numbers").is_some_and(|x| x.flag("hide")),
                source_data: Some(openecraft_parts::Shared::new(json!({ "kicad": {
                    "symbol": name,
                    "unit": n,
                    "value": property(sym, "Value"),
                    "footprint": property(sym, "Footprint"),
                    "datasheet": property(sym, "Datasheet"),
                    "description": description,
                } }))),
                ..Default::default()
            };
            unit(&mut u, body, n, names_shown, name_offset, k);
            if u.prims.is_empty() && u.pins.is_empty() {
                continue;
            }
            if power && u.pins.len() == 1 {
                u.func = Func::Potential;
                u.default_label = property(sym, "Value");
                u.show_pin_labels = false;
            }
            if u.pins.is_empty() {
                // A mounting hole, a logo, a unit with hidden supply pins only.
                u.func = Func::Graphic;
            }
            if u.letter.is_empty() {
                u.letter = "A".into();
            }
            out.push(u);
        }
    }
    if out.is_empty() {
        return Err(tr("The KiCad symbol library has no symbols").into());
    }
    Ok(out)
}
