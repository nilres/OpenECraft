//! Single-line and multi-line engineering. A device is planned in the single-line
//! diagram (overview symbols) and engineered in the multi-line schematic; both
//! representations share the device tag. Each single-line symbol stands for a
//! multi-line power symbol, so the schematic can be generated from the single-line
//! diagram and the single-line diagram from the schematic, and the check finds
//! devices and connections that differ between both.

use openecraft_i18n::{tr, tr_fmt};
use crate::fluid::Problem;
use crate::geom::{on_segment_interior, p, P, GRID};
use crate::index::{DeviceEntry, Index, Loc};
use crate::model::*;
use crate::netlist::Netlist;
use crate::issue::Severity;
use crate::structure::Dt;
use crate::symbols::{Category, SymbolDef, SymbolRef};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Single-line symbol, the multi-line power symbol generated for it and a further
/// function placed with it (the contactor's coil).
const MAPPING: [(&str, &str, Option<&str>); 8] = [
    ("sl_cb", "cb3", None),
    ("sl_disc", "disc3", None),
    ("sl_mps", "mps3", None),
    ("sl_fuse", "fuse3", None),
    ("sl_ol", "ol3", None),
    ("sl_contactor", "main3", Some("coil")),
    ("sl_motor", "motor3", None),
    ("sl_trafo", "trafo", None),
];

/// Further multi-line power symbols and their single-line symbol: a single-line circuit
/// breaker stands for any number of poles.
const ALSO: [(&str, &str); 4] = [("cb1", "sl_cb"), ("cb2", "sl_cb"), ("cb4", "sl_cb"), ("main4", "sl_contactor")];

/// The multi-line symbol of a single-line circuit breaker by the poles of the device's
/// part (`cb3` without one).
fn breaker_symbol(prj: &Project, tag: &str) -> Option<&'static str> {
    use openecraft_parts::FnKind;
    let numbers: Vec<&str> = prj.devices.get(tag).map(|d| d.parts.iter().map(|r| r.number.as_str()).collect()).unwrap_or_default();
    let generic = openecraft_parts::default_catalog();
    let parts: Vec<&openecraft_parts::Part> = numbers.into_iter().filter_map(|n| prj.parts.iter().chain(generic.iter()).find(|p| p.number == n)).collect();
    parts.iter().flat_map(|p| p.functions.iter()).find_map(|f| match f.kind {
        FnKind::Breaker1 => Some("cb1"),
        FnKind::Breaker2 => Some("cb2"),
        FnKind::Breaker3 => Some("cb3"),
        FnKind::Breaker4 => Some("cb4"),
        _ => None,
    })
}

/// Multi-line power symbol generated for a single-line symbol.
pub fn multi_of(single: &str) -> Option<&'static str> {
    MAPPING.iter().find(|m| m.0 == single).map(|m| m.1)
}

/// Single-line symbol that shows a multi-line power symbol.
pub fn single_of(multi: &str) -> Option<&'static str> {
    MAPPING.iter().find(|m| m.1 == multi).map(|m| m.0).or_else(|| ALSO.iter().find(|m| m.0 == multi).map(|m| m.1))
}

fn is_single_line(prj: &Project, sym: &str) -> bool {
    prj.symbol(sym).is_some_and(|d| d.category == Category::SingleLine)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// Only in the single-line diagram.
    Planned,
    /// In both representations.
    Engineered,
    /// Only in the schematic.
    NotInSingleLine,
    /// The single-line symbol does not fit the multi-line power symbol.
    Mismatch,
}

impl Status {
    pub fn title(self) -> &'static str {
        match self {
            Status::Planned => tr("Planned"),
            Status::Engineered => tr("Engineered"),
            Status::NotInSingleLine => tr("Not in the single-line diagram"),
            Status::Mismatch => tr("Does not match"),
        }
    }
}

/// A device of the single-line diagram or a power device of the schematic.
#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    pub tag: String,
    /// Single-line symbol (key) and its location.
    pub single: Option<(String, (usize, u64))>,
    /// Multi-line power symbol (key) and its location.
    pub multi: Option<(String, (usize, u64))>,
    pub status: Status,
}

fn sym_at<'a>(prj: &'a Project, l: &Loc) -> Option<&'a SymbolInst> {
    prj.pages[l.page].symbols.iter().find(|s| s.id == l.sym_id)
}

/// First single-line representation of a device.
fn single_loc<'a>(prj: &'a Project, dev: &DeviceEntry) -> Option<(&'a SymbolInst, Loc)> {
    dev.overviews.iter().find_map(|l| sym_at(prj, l).filter(|s| is_single_line(prj, &s.sym)).map(|s| (s, *l)))
}

/// First multi-line power symbol of a device.
fn power_loc<'a>(prj: &'a Project, dev: &DeviceEntry) -> Option<(&'a SymbolInst, Loc)> {
    dev.mains.iter().chain(&dev.auxs).find_map(|l| sym_at(prj, l).filter(|s| single_of(&s.sym).is_some()).map(|s| (s, *l)))
}

fn engineered(dev: &DeviceEntry) -> bool {
    !dev.mains.is_empty() || !dev.auxs.is_empty() || !dev.others.is_empty()
}

/// Devices with a single-line symbol or a multi-line power symbol, in tag order.
pub fn devices(prj: &Project, idx: &Index) -> Vec<Device> {
    let mut v = Vec::new();
    for tag in idx.sorted_tags() {
        let dev = &idx.devices[tag];
        let single = single_loc(prj, dev);
        let multi = power_loc(prj, dev);
        let status = match (&single, &multi) {
            (None, None) => continue,
            (None, Some(_)) => Status::NotInSingleLine,
            (Some((s, _)), Some((m, _))) if single_of(&m.sym) != Some(s.sym.as_str()) => Status::Mismatch,
            (Some(_), _) if engineered(dev) => Status::Engineered,
            (Some(_), _) => Status::Planned,
        };
        let at = |x: Option<(&SymbolInst, Loc)>| x.map(|(s, l)| (s.sym.clone(), (l.page, l.sym_id)));
        v.push(Device { tag: tag.clone(), single: at(single), multi: at(multi), status });
    }
    v
}

/// Union-find over the points of one page.
#[derive(Default)]
struct Points {
    parent: Vec<usize>,
    at: HashMap<(i64, i64), usize>,
}

impl Points {
    fn node(&mut self, q: P) -> usize {
        let n = self.parent.len();
        let i = *self.at.entry(q.key()).or_insert(n);
        if i == n {
            self.parent.push(n);
        }
        i
    }
    fn find(&mut self, mut i: usize) -> usize {
        while self.parent[i] != i {
            self.parent[i] = self.parent[self.parent[i]];
            i = self.parent[i];
        }
        i
    }
    fn union(&mut self, a: usize, b: usize) {
        let (a, b) = (self.find(a), self.find(b));
        self.parent[a] = b;
    }
}

/// Connected pins of the single-line symbols of a page: groups of (symbol id, pin).
fn single_line_nets(prj: &Project, page: &Page) -> Vec<Vec<(u64, usize)>> {
    let mut pts = Points::default();
    let ends: Vec<P> = page.wires.iter().flat_map(|w| [w.a, w.b]).collect();
    for w in &page.wires {
        let (a, b) = (pts.node(w.a), pts.node(w.b));
        pts.union(a, b);
        for q in ends.iter().filter(|q| on_segment_interior(**q, w.a, w.b)) {
            let n = pts.node(*q);
            pts.union(a, n);
        }
    }
    let mut groups: BTreeMap<usize, Vec<(u64, usize)>> = BTreeMap::new();
    for s in page.symbols.iter().filter(|s| is_single_line(prj, &s.sym)) {
        let def = prj.symbol(&s.sym).unwrap();
        for i in 0..def.pins.len() {
            let q = def.pin_world(i, s.pos, s.rot, s.mirror);
            let n = pts.node(q);
            let root = pts.find(n);
            groups.entry(root).or_default().push((s.id, i));
        }
    }
    groups.into_values().filter(|g| g.len() > 1).collect()
}

/// Devices connected in the single-line diagram: (tag, tag, location of the first).
pub fn single_line_links(prj: &Project) -> Vec<(String, String, (usize, u64))> {
    let mut seen = BTreeSet::new();
    let mut v = Vec::new();
    for (pi, page) in prj.pages.iter().enumerate().filter(|(_, pg)| matches!(pg.kind, PageKind::Schematic | PageKind::Overview)) {
        let tag_of = |id: u64| page.symbols.iter().find(|s| s.id == id).map(|s| s.tag.clone()).unwrap_or_default();
        for net in single_line_nets(prj, page) {
            for (i, a) in net.iter().enumerate() {
                for b in &net[i + 1..] {
                    let (ta, tb) = (tag_of(a.0), tag_of(b.0));
                    if ta.is_empty() || tb.is_empty() || ta == tb {
                        continue;
                    }
                    let key = if ta < tb { (ta.clone(), tb.clone()) } else { (tb.clone(), ta.clone()) };
                    if seen.insert(key) {
                        v.push((ta, tb, (pi, a.0)));
                    }
                }
            }
        }
    }
    v
}

/// Nets of the pins of all power symbols of a device.
fn power_nets(prj: &Project, nl: &Netlist, dev: &DeviceEntry) -> BTreeSet<usize> {
    let mut v = BTreeSet::new();
    for l in dev.mains.iter().chain(&dev.auxs) {
        let Some(s) = sym_at(prj, l).filter(|s| single_of(&s.sym).is_some()) else { continue };
        let n = prj.symbol(&s.sym).map(|d| d.pins.len()).unwrap_or(0);
        v.extend((0..n).filter_map(|i| nl.pin_net.get(&(s.id, i)).copied()));
    }
    v
}

/// Function text shown by the main function (else the power symbol) of a device.
fn text_loc(prj: &Project, dev: &DeviceEntry) -> Option<Loc> {
    dev.mains.first().copied().or_else(|| power_loc(prj, dev).map(|(_, l)| l))
}

/// Differences between the single-line diagram and the schematic.
pub fn problems(prj: &Project, idx: &Index, nl: &Netlist) -> Vec<Problem> {
    let mut v = Vec::new();
    let devices = devices(prj, idx);
    let uses_single_line = devices.iter().any(|d| d.single.is_some());
    for d in &devices {
        let name = |key: &str| prj.symbol(key).map(|s| tr(s.name)).unwrap_or_default();
        match (d.status, &d.single, &d.multi) {
            (Status::Planned, Some((_, at)), _) => {
                v.push((Severity::Info, tr_fmt!("{}: planned in the single-line diagram, not in the schematic yet", d.tag), Some(*at)));
            }
            (Status::NotInSingleLine, _, Some((_, at))) if uses_single_line => {
                v.push((Severity::Info, tr_fmt!("{}: not in the single-line diagram", d.tag), Some(*at)));
            }
            (Status::Mismatch, Some((s, at)), Some((m, _))) => {
                v.push((Severity::Warning, tr_fmt!("{}: single-line symbol '{}' does not match the schematic ('{}')", d.tag, name(s), name(m)), Some(*at)));
            }
            _ => {}
        }
        if let (Some((_, at)), Some(text)) = (&d.single, text_loc(prj, &idx.devices[&d.tag])) {
            let single = prj.pages[at.0].symbols.iter().find(|s| s.id == at.1).map(|s| s.function_text.as_str()).unwrap_or("");
            let multi = sym_at(prj, &text).map(|s| s.function_text.as_str()).unwrap_or("");
            if !single.is_empty() && !multi.is_empty() && single != multi {
                v.push((Severity::Info, tr_fmt!("{}: function texts differ in the single-line diagram and the schematic", d.tag), Some(*at)));
            }
        }
    }
    for (a, b, at) in single_line_links(prj) {
        let (Some(da), Some(db)) = (idx.devices.get(&a), idx.devices.get(&b)) else { continue };
        let (na, nb) = (power_nets(prj, nl, da), power_nets(prj, nl, db));
        if !na.is_empty() && !nb.is_empty() && na.is_disjoint(&nb) {
            v.push((Severity::Warning, tr_fmt!("{} – {}: connected in the single-line diagram but not in the schematic", a, b), Some(at)));
        }
    }
    v
}

/// Copies function texts between the single-line symbols and the schematic where
/// one side is empty. Returns the number of texts copied.
pub fn synchronize(prj: &mut Project) -> usize {
    let idx = Index::build(prj);
    let mut copies = Vec::new();
    for d in devices(prj, &idx) {
        let (Some((_, at)), Some(text)) = (d.single, text_loc(prj, &idx.devices[&d.tag])) else { continue };
        let single = prj.pages[at.0].symbols.iter().find(|s| s.id == at.1).map(|s| s.function_text.clone()).unwrap_or_default();
        let multi = sym_at(prj, &text).map(|s| s.function_text.clone()).unwrap_or_default();
        match (single.is_empty(), multi.is_empty()) {
            (true, false) => copies.push((at, multi)),
            (false, true) => copies.push(((text.page, text.sym_id), single)),
            _ => {}
        }
    }
    for ((page, id), text) in &copies {
        if let Some(s) = prj.pages[*page].symbols.iter_mut().find(|s| s.id == *id) {
            s.function_text = text.clone();
        }
    }
    copies.len()
}

/// Result of a transformation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Generated {
    /// Index of the new page.
    pub page: usize,
    pub symbols: usize,
    pub wires: usize,
    /// Devices left out (already in the target representation or without mapping).
    pub skipped: Vec<String>,
}

/// Pins of a symbol on its upper (supply) or lower side, from left to right.
fn side_pins(def: &SymbolDef, upper: bool) -> Vec<usize> {
    let ys = def.pins.iter().map(|q| q.pos.y);
    let y = if upper { ys.fold(f32::MAX, f32::min) } else { ys.fold(f32::MIN, f32::max) };
    let mut v: Vec<usize> = (0..def.pins.len()).filter(|&i| (def.pins[i].pos.y - y).abs() < 0.01).collect();
    v.sort_by(|a, b| def.pins[*a].pos.x.total_cmp(&def.pins[*b].pos.x));
    v
}

fn new_inst(prj: &mut Project, def: &SymbolDef, pos: P, rot: u8, mirror: bool, tag: &str, function_text: &str) -> SymbolInst {
    SymbolInst {
        id: prj.new_id(),
        sym: def.key.to_string(),
        pos,
        rot,
        mirror,
        tag: tag.to_string(),
        function_text: function_text.to_string(),
        pins: def.pins.iter().map(|x| x.label.to_string()).collect(),
        ..Default::default()
    }
}

fn push_wire(prj: &mut Project, page: usize, a: P, b: P) -> usize {
    if a.same(b) {
        return 0;
    }
    let id = prj.new_id();
    prj.pages[page].wires.push(Wire { id, a, b, ..Default::default() });
    1
}

/// Parallel conductors from pins `a` to pins `b`: straight when aligned, else with a
/// step in between whose rows are staggered so no conductor ends on another one.
fn conductors(prj: &mut Project, page: usize, a: &[P], b: &[P]) -> usize {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0;
    }
    let mut count = 0;
    let mid = ((a[0].y + b[0].y) / 2.0 / GRID).round() * GRID;
    let down = if b[0].y >= a[0].y { 1.0 } else { -1.0 };
    let right = b[0].x > a[0].x;
    let centre = ((n - 1) / 2) as f32;
    for i in 0..n {
        let (pa, pb) = (a[i], b[i]);
        if (pa.x - pb.x).abs() < 0.01 {
            count += push_wire(prj, page, pa, pb);
            continue;
        }
        let rank = if right { (n - 1 - i) as f32 } else { i as f32 };
        let y = mid + down * GRID * (rank - centre);
        count += push_wire(prj, page, pa, p(pa.x, y));
        count += push_wire(prj, page, p(pa.x, y), p(pb.x, y));
        count += push_wire(prj, page, p(pb.x, y), pb);
    }
    count
}

/// Generates the multi-line schematic of the single-line symbols of page `from` on a
/// new schematic page after it: the power symbols with their conductors, contactor
/// coils in a row below. Devices that are in the schematic already are left out. The
/// layout is stretched, as the multi-line symbols are wider.
pub fn to_multiline(prj: &mut Project, from: usize) -> Result<Generated, String> {
    let idx = Index::build(prj);
    let src = prj.pages.get(from).ok_or_else(|| tr("No such page"))?.clone();
    let mut skipped = Vec::new();
    let mut todo: Vec<(&SymbolInst, &'static str, Option<&'static str>)> = Vec::new();
    for s in src.symbols.iter().filter(|s| is_single_line(prj, &s.sym) && !s.tag.is_empty()) {
        let Some(m) = MAPPING.iter().find(|m| m.0 == s.sym) else {
            skipped.push(s.tag.clone());
            continue;
        };
        if idx.devices.get(&s.tag).is_some_and(engineered) || todo.iter().any(|t| t.0.tag == s.tag) {
            skipped.push(s.tag.clone());
            continue;
        }
        let multi = if s.sym == "sl_cb" { breaker_symbol(prj, &s.tag).unwrap_or(m.1) } else { m.1 };
        todo.push((s, multi, m.2));
    }
    if todo.is_empty() {
        return Err(if src.symbols.iter().any(|s| is_single_line(prj, &s.sym)) {
            tr_fmt!("All devices of page {} are in the schematic already", src.name)
        } else {
            tr_fmt!("Page {} has no single-line symbols", src.name)
        });
    }
    let x0 = todo.iter().map(|t| t.0.pos.x).fold(f32::MAX, f32::min);
    let y0 = todo.iter().map(|t| t.0.pos.y).fold(f32::MAX, f32::min);
    let place = |q: P| p(x0 + (q.x - x0) * 2.0, y0 + (q.y - y0) * 1.5).snap(GRID);

    let id = prj.new_id();
    let at = (from + 1).min(prj.pages.len());
    let mut page = Page::new(id, &crate::ops::next_page_name(prj, at), &tr_fmt!("{} (multi-line)", src.description), PageKind::Schematic);
    Dt::of_page(&src).apply_to(&mut page);
    crate::ops::insert_page(prj, at, page);

    let mut out = Generated { page: at, skipped, ..Default::default() };
    // Single-line symbol id -> generated power symbol (def, position, rotation, mirror).
    let mut placed: HashMap<u64, (SymbolRef, P, u8, bool)> = HashMap::new();
    let mut coils = Vec::new();
    for (s, multi, extra) in &todo {
        let Some(def) = prj.symbol(multi) else { continue };
        let pos = place(s.pos);
        let inst = new_inst(prj, &def, pos, s.rot, s.mirror, &s.tag, &s.function_text);
        prj.pages[at].symbols.push(inst);
        placed.insert(s.id, (def, pos, s.rot, s.mirror));
        out.symbols += 1;
        if let Some(def) = extra.and_then(|k| prj.symbol(k)) {
            coils.push((def, s.tag.clone()));
        }
    }
    let bottom = placed.values().map(|(d, q, ..)| q.y + d.bounds.max.y).fold(y0, f32::max);
    for (i, (def, tag)) in coils.into_iter().enumerate() {
        let pos = p(x0 + i as f32 * 6.0 * GRID, bottom + 10.0 * GRID).snap(GRID);
        let inst = new_inst(prj, &def, pos, 0, false, &tag, "");
        prj.pages[at].symbols.push(inst);
        out.symbols += 1;
    }

    let symbols = prj.symbol_table();
    let single_def = |id: u64| src.symbols.iter().find(|s| s.id == id).and_then(|s| symbols.get(&s.sym));
    for net in single_line_nets(prj, &src) {
        let net: Vec<(u64, usize)> = net.into_iter().filter(|(id, _)| placed.contains_key(id)).collect();
        let Some((first, rest)) = net.split_first() else { continue };
        let pins = |(id, pin): (u64, usize)| -> Vec<P> {
            let (Some(sd), Some((def, pos, rot, mirror))) = (single_def(id), placed.get(&id)) else { return Vec::new() };
            let upper = sd.pins[pin].pos.y < 0.0;
            side_pins(def, upper).into_iter().map(|i| def.pin_world(i, *pos, *rot, *mirror)).collect()
        };
        let a = pins(*first);
        for b in rest {
            let b = pins(*b);
            out.wires += conductors(prj, at, &a, &b);
        }
    }
    Ok(out)
}

/// Generates a single-line diagram of the schematic's power devices that are not in
/// a single-line diagram yet, on a new overview page after page `after`. The devices
/// are arranged as a tree from the supply (upper pins) to the loads.
pub fn to_singleline(prj: &mut Project, after: usize) -> Result<Generated, String> {
    let idx = Index::build(prj);
    let nl = Netlist::build(prj, true);
    let mut skipped = Vec::new();
    struct Node {
        tag: String,
        single: &'static str,
        text: String,
        /// Nets of the upper and lower pins of the power symbol.
        upper: BTreeSet<usize>,
        lower: BTreeSet<usize>,
    }
    let mut nodes: Vec<Node> = Vec::new();
    for tag in idx.sorted_tags() {
        let dev = &idx.devices[tag];
        let Some((s, _)) = power_loc(prj, dev) else { continue };
        if single_loc(prj, dev).is_some() {
            skipped.push(tag.clone());
            continue;
        }
        let (Some(single), Some(def)) = (single_of(&s.sym), prj.symbol(&s.sym)) else { continue };
        let nets = |upper: bool| side_pins(&def, upper).into_iter().filter_map(|i| nl.pin_net.get(&(s.id, i)).copied()).collect::<BTreeSet<_>>();
        let (upper, lower) = (nets(true), nets(false));
        let lower = if lower == upper { BTreeSet::new() } else { lower };
        let text = text_loc(prj, dev).and_then(|l| sym_at(prj, &l)).map(|s| s.function_text.clone()).unwrap_or_default();
        nodes.push(Node { tag: tag.clone(), single, text, upper, lower });
    }
    if nodes.is_empty() {
        return Err(tr("No devices to add to the single-line diagram").into());
    }
    // Tree from the supply: the parent of a device feeds its upper pins.
    let mut parent: Vec<Option<usize>> = vec![None; nodes.len()];
    for (b, node) in nodes.iter().enumerate() {
        parent[b] = (0..nodes.len()).find(|&a| a != b && !nodes[a].lower.is_disjoint(&node.upper));
    }
    // Break cycles: a device that reaches itself through its parents becomes a root.
    for b in 0..nodes.len() {
        let mut seen = HashSet::new();
        let mut c = b;
        while let Some(a) = parent[c] {
            if !seen.insert(c) || a == b {
                parent[b] = None;
                break;
            }
            c = a;
        }
    }
    let children = |a: usize, parent: &[Option<usize>]| (0..nodes.len()).filter(|&b| parent[b] == Some(a)).collect::<Vec<_>>();
    // Column and row of each device: leaves side by side, a parent above its first child.
    let mut cell: Vec<(f32, f32)> = vec![(0.0, 0.0); nodes.len()];
    let mut next_col = 0.0;
    fn layout(n: usize, depth: f32, kids: &dyn Fn(usize) -> Vec<usize>, cell: &mut [(f32, f32)], next_col: &mut f32) {
        let ch = kids(n);
        for &c in &ch {
            layout(c, depth + 1.0, kids, cell, next_col);
        }
        let col = match ch.first() {
            Some(&c) => cell[c].0,
            None => {
                *next_col += 1.0;
                *next_col - 1.0
            }
        };
        cell[n] = (col, depth);
    }
    let kids = |a: usize| children(a, &parent);
    for root in (0..nodes.len()).filter(|&n| parent[n].is_none()) {
        layout(root, 0.0, &kids, &mut cell, &mut next_col);
    }

    let id = prj.new_id();
    let at = (after + 1).min(prj.pages.len());
    let page = Page::new(id, &crate::ops::next_page_name(prj, at), tr("Single-line diagram"), PageKind::Overview);
    crate::ops::insert_page(prj, at, page);
    let mut out = Generated { page: at, skipped, ..Default::default() };
    let pos = |(col, row): (f32, f32)| p(10.0 * GRID + col * 8.0 * GRID, 10.0 * GRID + row * 9.0 * GRID);
    for (n, node) in nodes.iter().enumerate() {
        let Some(def) = prj.symbol(node.single) else { continue };
        let inst = new_inst(prj, &def, pos(cell[n]), 0, false, &node.tag, &node.text);
        prj.pages[at].symbols.push(inst);
        out.symbols += 1;
    }
    let mut drawn = HashSet::new();
    for b in 0..nodes.len() {
        let Some(a) = parent[b] else { continue };
        let (Some(da), Some(db)) = (prj.symbol(nodes[a].single), prj.symbol(nodes[b].single)) else { continue };
        let (Some(&ia), Some(&ib)) = (side_pins(&da, false).first(), side_pins(&db, true).first()) else { continue };
        let pa = da.pin_world(ia, pos(cell[a]), 0, false);
        let pb = db.pin_world(ib, pos(cell[b]), 0, false);
        if (pa.x - pb.x).abs() < 0.01 {
            out.wires += push_wire(prj, at, pa, pb);
            continue;
        }
        let y = pa.y + GRID;
        for (q, r) in [(pa, p(pa.x, y)), (p(pa.x, y), p(pb.x, y)), (p(pb.x, y), pb)] {
            if drawn.insert((q.key(), r.key())) {
                out.wires += push_wire(prj, at, q, r);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
