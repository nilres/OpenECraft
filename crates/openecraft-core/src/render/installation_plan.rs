//! Installation plan pages: the rooms of one level of a building seen from above with
//! their walls, doors and windows, the installation devices with their symbols (IEC
//! 60617-11) and tags, the connections of the circuits and a legend of the circuits,
//! at a drawing scale (that of the page or the largest that fits), centred on the rooms
//! chosen and cut off at the edge of the drawing area.

use super::*;
use crate::installation::{self, Placed};
use crate::model::panel::Opening;

/// Size of a symbol (radius, mm on the page).
const SYMBOL: f32 = 2.4;
/// Width of a column of the legend of the circuits at its text height of 2.2 mm.
const LEGEND: f32 = 62.0;

/// Text height in mm on the page of a font size in pt as printed (the exports print a
/// text of height `h` mm at `h × 1.25` mm em size).
pub fn pt_to_mm(pt: f32) -> f32 {
    pt.max(1.0) * 25.4 / 72.0 / 1.25
}
const THIN: f32 = 0.18;
/// Least thickness of a wall on the page (mm).
const WALL: f32 = 0.5;

/// Colours of the circuits (by their index), shared with the site plan on screen.
pub const CIRCUIT_COLORS: [[u8; 3]; 8] = [[220, 70, 60], [40, 130, 220], [40, 160, 80], [200, 130, 20], [150, 80, 200], [20, 160, 170], [200, 60, 150], [120, 120, 40]];

/// The colour of the circuit with index `k`.
pub fn circuit_color(k: usize) -> [u8; 3] {
    CIRCUIT_COLORS[k % CIRCUIT_COLORS.len()]
}

/// Maps the plant (x, z, mm) onto the page: `lo` of the plant at `origin`, 1 : `n`.
struct Map {
    lo: [f32; 2],
    origin: P,
    n: f32,
}

impl Map {
    fn at(&self, q: [f32; 2]) -> P {
        p(self.origin.x + (q[0] - self.lo[0]) / self.n, self.origin.y + (q[1] - self.lo[1]) / self.n)
    }
}

/// A point of a room (x, z of the room) in the plant.
fn plant(pose: &panel::SitePose, q: [f32; 2]) -> [f32; 2] {
    let v = pose.apply([q[0], 0.0, q[1]]);
    [v[0], v[2]]
}

pub(super) fn draw_page(d: &mut dyn Draw, prj: &Project, an: &Analysis, page: &Page, level: u64, frame: &crate::frame::PlotFrame) {
    let a = frame.area();
    let layout = &prj.layout;
    let Some((building, lv)) = layout.buildings.iter().find_map(|b| b.levels.iter().find(|l| l.id == level).map(|l| (b, l))) else {
        let msg = tr("Level not found - choose another one in the page properties");
        d.text(a.min + p(15.0, 25.0), msg, 4.0, Anchor::Left, VAnchor::Middle, Ink::Error);
        return;
    };
    let rooms = panel::buildings::rooms_on(layout, level);
    if rooms.is_empty() {
        d.text(a.min + p(15.0, 25.0), tr("No rooms on this level"), 4.0, Anchor::Left, VAnchor::Middle, Ink::Text);
        return;
    }
    let poses = panel::site_poses(prj);
    let view = &page.plan;
    // The rooms the page is about (all of the level when none of them is chosen).
    let chosen: Vec<usize> = rooms.iter().copied().filter(|&ri| view.rooms.contains(&layout.enclosures[ri].name)).collect();
    let about = if chosen.is_empty() { rooms.clone() } else { chosen };
    // Their extent in the plant with their walls.
    let mut pts: Vec<[f32; 2]> = Vec::new();
    for &ri in &about {
        let e = &layout.enclosures[ri];
        let t = e.wall();
        for w in panel::walls(e) {
            for q in [w.a, w.b] {
                pts.push(plant(&poses[ri], [q[0] - w.normal[0] * t, q[1] - w.normal[1] * t]));
            }
        }
    }
    let (lo, hi) = panel::bounds(&pts);
    let circuits: Vec<installation::Circuit> = installation::circuits(prj, &an.netlist, &an.connections, None);
    let items: Vec<Placed> = installation::items(prj, &poses).into_iter().filter(|i| i.level == level).collect();
    let circuit_of = |item: u64| circuits.iter().position(|c| c.has_item(item));
    let shown: Vec<usize> = (0..circuits.len()).filter(|k| items.iter().any(|i| about.contains(&i.room) && circuits[*k].has_item(i.item))).collect();
    // The drawing area, beside or above / below the legend.
    let legend = view.legend && !shown.is_empty();
    let ls = pt_to_mm(view.legend_size) / 2.2;
    let lw = LEGEND * ls;
    let (mut x0, mut y0, mut x1, mut y1) = (a.min.x + 10.0, a.min.y + 10.0, a.max.x - 10.0, a.max.y - 16.0);
    let band = |cols: usize| 4.0 * ls + legend_rows(view, &circuits, &shown, cols, lw).iter().sum::<f32>() + 4.0;
    let cols = (((a.max.x - a.min.x) - 16.0) / lw).floor().max(1.0) as usize;
    // A band above or below keeps at least half of the area for the plan; what does not
    // fit into it is noted in the legend.
    let band_max = ((a.max.y - a.min.y) - 26.0) / 2.0;
    let lbox = if !legend {
        None
    } else {
        Some(match view.legend_at {
            LegendAt::Right => {
                x1 = a.max.x - lw - 2.0;
                (p(x1 + 4.0, a.min.y + 10.0), 1, a.max.y - 12.0)
            }
            LegendAt::Left => {
                x0 = a.min.x + lw + 6.0;
                (p(a.min.x + 4.0, a.min.y + 10.0), 1, a.max.y - 12.0)
            }
            LegendAt::Top => {
                y0 = a.min.y + 6.0 + band(cols).min(band_max);
                (p(a.min.x + 8.0, a.min.y + 6.0), cols, y0 - 2.0)
            }
            LegendAt::Bottom => {
                y1 = a.max.y - 14.0 - band(cols).min(band_max);
                (p(a.min.x + 8.0, y1 + 4.0), cols, a.max.y - 12.0)
            }
        })
    };
    let real = ((hi[0] - lo[0]).max(1.0), (hi[1] - lo[1]).max(1.0));
    let fixed = page.scale > 1.0;
    let n = if fixed { page.scale } else { fit_scale(real, (x1 - x0, y1 - y0)) };
    let origin = p(x0 + ((x1 - x0) - real.0 / n) / 2.0, y0 + ((y1 - y0) - real.1 / n) / 2.0);
    let map = Map { lo, origin, n };
    // What lies outside the drawing area (other rooms, a large scale) is cut off.
    let mut clip = Clip { d: &mut *d, lo: p(x0, y0), hi: p(x1, y1) };
    let d: &mut dyn Draw = &mut clip;
    for &ri in &rooms {
        draw_room(d, prj, ri, &poses[ri], &map);
    }
    for &ri in &rooms {
        for ci in panel::cabinets_in(layout, ri) {
            cabinet(d, prj, ri, ci, &poses[ri], &map, (pt_to_mm(view.tag_size), view.cabinet_scale.clamp(10.0, 1000.0) / 100.0));
        }
    }
    if view.room_names || view.room_descriptions {
        let rs = pt_to_mm(view.room_size);
        for &ri in &rooms {
            let e = &layout.enclosures[ri];
            let description = room_description(prj, ri).filter(|_| view.room_descriptions);
            let lines: Vec<&str> = [view.room_names.then_some(e.name.as_str()), description].into_iter().flatten().collect();
            if lines.is_empty() {
                continue;
            }
            let outline: Vec<[f32; 2]> = panel::outline(e).into_iter().map(|q| plant(&poses[ri], q)).collect();
            let c = outline.iter().fold([0.0, 0.0], |s, q| [s[0] + q[0], s[1] + q[1]]).map(|v| v / outline.len() as f32);
            let mut at = map.at(c);
            // Above a symbol in the middle of the room (a ceiling light).
            let tall = rs * 1.3 * lines.len() as f32;
            if items.iter().any(|i| { let q = map.at(i.at); (q.x - at.x).abs() < SYMBOL + rs && (q.y - at.y).abs() < SYMBOL + tall / 2.0 + rs }) {
                at.y -= SYMBOL + tall / 2.0 + rs / 2.0;
            }
            for (k, text) in lines.iter().enumerate() {
                let y = at.y + (k as f32 - (lines.len() - 1) as f32 / 2.0) * rs * 1.3;
                d.text(p(at.x, y), text, rs, Anchor::Center, VAnchor::Middle, Ink::Text);
            }
        }
    }
    let ink_of = |k: Option<usize>| match k {
        Some(k) if view.circuit_colors => Ink::Color(circuit_color(k)),
        _ => Ink::Line,
    };
    if view.connections {
        for (k, c) in circuits.iter().enumerate().filter(|(k, _)| shown.contains(k)) {
            for l in &c.legs {
                let on_level = |tag: &str| items.iter().any(|i| i.tag == tag);
                if !on_level(&l.from) && !on_level(&l.to) {
                    continue;
                }
                let (Some(f), Some(t)) = (installation::plan_point(prj, &poses, &l.from), installation::plan_point(prj, &poses, &l.to)) else { continue };
                let (f, t) = (map.at(f), map.at(t));
                // Along the walls: first along x, then along y.
                let corner = p(t.x, f.y);
                d.line(f, corner, THIN, ink_of(Some(k)), true);
                d.line(corner, t, THIN, ink_of(Some(k)), true);
            }
        }
    }
    let sym = SYMBOL * view.symbol_scale.clamp(10.0, 1000.0) / 100.0;
    for i in &items {
        let k = circuit_of(i.item);
        let c = map.at(i.at);
        let up = i.facing.map(|f| p(-f[0], -f[1]));
        // Devices of circuits the legend does not list (in neighbouring rooms) stay black.
        symbol_sized(d, prj, &i.symbol, c, up, ink_of(k.filter(|k| shown.contains(k))), sym);
        if view.tags && !i.tag.is_empty() {
            let into = up.map_or(p(0.0, 1.0), |u| p(-u.x, -u.y));
            let reach = if up.is_some() { 2.0 * sym + 1.8 } else { sym + 1.8 };
            let at = p(c.x + into.x * reach, c.y + into.y * reach);
            let h = if into.x > 0.5 { Anchor::Left } else if into.x < -0.5 { Anchor::Right } else { Anchor::Center };
            let tag = view.tag_parts.apply(&i.tag);
            d.text(at, &tag, pt_to_mm(view.tag_size), h, VAnchor::Middle, Ink::Text);
        }
        if let Some(page) = k.and_then(|k| circuits[k].page) {
            d.link(p(c.x - sym, c.y - sym), p(c.x + sym, c.y + sym), page);
        }
    }
    let d = clip.d;
    if let Some((at, cols, bottom)) = lbox {
        draw_legend(d, view, &circuits, &shown, (at, cols, lw), bottom);
    }
    let n_text = crate::graphics::scale_text(n);
    let scale = if fixed { tr_fmt!("Scale {}", n_text) } else { tr_fmt!("Scale {} (fitted)", n_text) };
    let caption = format!("{} {}  -  {}", building.name, lv.name, scale);
    d.text(p(a.min.x + 8.0, a.max.y - 7.0), &caption, 3.0, Anchor::Left, VAnchor::Middle, Ink::Text);
}

/// The text of the legend line of circuit `c`: its protective device and function text.
fn legend_text(c: &installation::Circuit) -> String {
    if c.text.is_empty() { c.breaker.clone() } else { format!("{}  {}", c.breaker, c.text) }
}

/// The lines of the legend text of each circuit shown, wrapped at blanks so that they
/// stay inside a column `width` wide (the line in the circuit's colour before them).
fn legend_lines(view: &PlanView, circuits: &[installation::Circuit], shown: &[usize], width: f32) -> Vec<Vec<String>> {
    let size = pt_to_mm(view.legend_size);
    let room = width - 8.0 * size / 2.2 - 4.0;
    shown.iter().map(|&k| wrap_printed(&legend_text(&circuits[k]), size, room)).collect()
}

/// Width of a text as printed (Helvetica metrics at the scale of the PDF and SVG
/// exports), a little more than [`crate::text::est_width`] for capitals and wide
/// letters.
fn printed_width(text: &str, size: f32) -> f32 {
    text.chars().map(|c| if (' '..='~').contains(&c) { f32::from(crate::text::HELVETICA[c as usize - 32]) } else { 600.0 }).sum::<f32>() * size * 1.3 / 1000.0
}

/// `text` wrapped at blanks into lines no wider than `width` mm as printed.
fn wrap_printed(text: &str, size: f32, width: f32) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in text.split(' ').flat_map(|w| split_wide(w, size, width)) {
        let longer = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
        if cur.is_empty() || printed_width(&longer, size) <= width {
            cur = longer;
        } else {
            out.push(std::mem::replace(&mut cur, word.to_string()));
        }
    }
    out.push(cur);
    out
}

/// A word broken by characters into pieces no wider than `width` mm as printed (a
/// single piece when it fits).
fn split_wide(word: &str, size: f32, width: f32) -> Vec<String> {
    let mut out = vec![String::new()];
    for c in word.chars() {
        let cur = out.last_mut().expect("never empty");
        if !cur.is_empty() && printed_width(&format!("{cur}{c}"), size) > width {
            out.push(c.to_string());
        } else {
            cur.push(c);
        }
    }
    out
}

/// The heights of the rows of the legend laid out in `cols` columns (row by row): a
/// row is as tall as its entry with the most lines.
fn legend_rows(view: &PlanView, circuits: &[installation::Circuit], shown: &[usize], cols: usize, width: f32) -> Vec<f32> {
    let ls = pt_to_mm(view.legend_size) / 2.2;
    let lines = legend_lines(view, circuits, shown, width);
    lines.chunks(cols.max(1)).map(|r| 5.0 * ls + 3.0 * ls * (r.iter().map(Vec::len).max().unwrap_or(1) - 1) as f32).collect()
}

/// The legend of the circuits `shown` from `at` in `cols` columns `width` apart, not
/// below `bottom`: a line in the colour of each circuit, its device and text (wrapped
/// inside its column, so that it does not run over the frame), a link to its
/// installation page.
fn draw_legend(d: &mut dyn Draw, view: &PlanView, circuits: &[installation::Circuit], shown: &[usize], (at, cols, width): (P, usize, f32), bottom: f32) {
    let size = pt_to_mm(view.legend_size);
    let ls = size / 2.2;
    let cols = cols.max(1);
    d.text(p(at.x, at.y + 2.0 * ls), tr("Circuits"), 3.0 * ls, Anchor::Left, VAnchor::Middle, Ink::Text);
    let lines = legend_lines(view, circuits, shown, width);
    let rows = legend_rows(view, circuits, shown, cols, width);
    let mut top = at.y + 4.0 * ls + 2.5 * ls;
    for (r, h) in rows.iter().enumerate() {
        for col in 0..cols {
            let n = r * cols + col;
            let Some(&k) = shown.get(n) else { break };
            let (x, y) = (at.x + col as f32 * width, top + 2.5 * ls);
            // The last line left notes the circuits that do not fit.
            let last = n + 1 == shown.len();
            if y + 3.0 * ls * (lines[n].len() - 1) as f32 > bottom - if last { 0.0 } else { 3.0 * ls } {
                let more = shown.len() - n;
                d.text(p(x, y.min(bottom - ls)), &tr_fmt!("{} more circuits do not fit", more), size, Anchor::Left, VAnchor::Middle, Ink::Text);
                return;
            }
            let ink = if view.circuit_colors { Ink::Color(circuit_color(k)) } else { Ink::Line };
            d.line(p(x, y), p(x + 6.0 * ls, y), 0.5, ink, false);
            for (i, line) in lines[n].iter().enumerate() {
                d.text(p(x + 8.0 * ls, y + 3.0 * ls * i as f32), line, size, Anchor::Left, VAnchor::Middle, Ink::Text);
            }
            if let Some(page) = circuits[k].page {
                d.link(p(x, top), p(x + width - 2.0, top + h), page);
            }
        }
        top += h;
    }
}

/// The walls of a room with gaps for its doors and windows, the leaves of the doors
/// and the panes of the windows, and its inner walls.
fn draw_room(d: &mut dyn Draw, prj: &Project, ri: usize, pose: &panel::SitePose, map: &Map) {
    let e = &prj.layout.enclosures[ri];
    // Rooms carry thin walls of their own; on paper a wall is at least a line pair.
    let t = e.wall().max(WALL * map.n);
    let to_page = |q: [f32; 2]| map.at(plant(pose, q));
    let walls = panel::walls(e);
    for (n, w) in walls.iter().enumerate() {
        let (s, r, nm) = (w.start(), w.right(), w.normal);
        let along = |u: f32, into: f32| to_page([s[0] + r[0] * u + nm[0] * into, s[1] + r[1] * u + nm[1] * into]);
        let len = w.length();
        let mut gaps: Vec<&PanelItem> = e.items.iter().filter(|it| it.face == Face::Wall(n as u16) && matches!(it.opening, Opening::Door | Opening::Window)).collect();
        gaps.sort_by(|a, b| a.x.total_cmp(&b.x));
        // The pieces of wall between the openings, at the corners as far as the
        // thickness of the walls.
        let mut from = -t;
        for g in &gaps {
            band(d, &along, from, g.x.max(from), t);
            from = g.x + g.w;
        }
        band(d, &along, from, len + t, t);
        for g in &gaps {
            let (u0, u1) = (g.x, g.x + g.w);
            match g.opening {
                Opening::Window => {
                    for into in [0.0, -t / 2.0, -t] {
                        d.line(along(u0, into), along(u1, into), THIN, Ink::Line, false);
                    }
                }
                _ => door(d, &along, g, t),
            }
        }
    }
    for iw in &e.inner_walls {
        let c = panel::inner_footprint(iw).map(to_page);
        d.poly(&c, true, THIN, Ink::Fill, true);
        d.poly(&c, true, THIN, Ink::Line, false);
    }
}

/// A cabinet or distribution board standing in a room: its footprint as the symbol of a
/// distribution board (IEC 60617-11: a box, the half behind the diagonal filled) with its
/// name.
fn cabinet(d: &mut dyn Draw, prj: &Project, ri: usize, ci: usize, pose: &panel::SitePose, map: &Map, (size, scale): (f32, f32)) {
    let c = panel::footprint(prj, ri, ci).map(|q| map.at(plant(pose, q)));
    // Enlarged or reduced about its middle (`scale` 1: its footprint).
    let m = p((c[0].x + c[2].x) / 2.0, (c[0].y + c[2].y) / 2.0);
    let c = c.map(|q| p(m.x + (q.x - m.x) * scale, m.y + (q.y - m.y) * scale));
    d.poly(&c, true, 0.35, Ink::Line, false);
    d.poly(&[c[0], c[1], c[2]], true, THIN, Ink::Line, true);
    let mid = p((c[0].x + c[2].x) / 2.0, (c[0].y + c[2].y) / 2.0);
    let below = c.iter().map(|q| q.y).fold(f32::MIN, f32::max);
    d.text(p(mid.x, below + size + 0.6), &prj.layout.enclosures[ci].name, size, Anchor::Center, VAnchor::Middle, Ink::Text);
}

/// A piece of wall from `u0` to `u1` along it, `t` thick behind its face.
fn band(d: &mut dyn Draw, along: &dyn Fn(f32, f32) -> P, u0: f32, u1: f32, t: f32) {
    if u1 - u0 < 0.5 {
        return;
    }
    let c = [along(u0, 0.0), along(u1, 0.0), along(u1, -t), along(u0, -t)];
    d.poly(&c, true, THIN, Ink::Fill, true);
    d.poly(&c, true, THIN, Ink::Line, false);
}

/// The leaf of a door: hinged ones as a line with the quarter circle of their swing
/// (into the room or out of it), sliding ones as a line in front of the wall.
fn door(d: &mut dyn Draw, along: &dyn Fn(f32, f32) -> P, g: &PanelItem, t: f32) {
    let (u0, u1) = (g.x, g.x + g.w);
    let slide = g.swing.slide();
    if slide != 0.0 {
        let off = if slide < 0.0 { -g.w } else { g.w };
        d.line(along(u0 + off, 60.0), along(u1 + off, 60.0), 0.35, Ink::Line, false);
        return;
    }
    let (hinge, free) = if g.swing.right() { (u1, u0) } else { (u0, u1) };
    let (face, sign) = if g.swing.outward() { (-t, -1.0) } else { (0.0, 1.0) };
    let width = g.w;
    let leaf = along(hinge, face + sign * width);
    d.line(along(hinge, face), leaf, 0.35, Ink::Line, false);
    let arc: Vec<P> = (0..=12)
        .map(|k| {
            let a = std::f32::consts::FRAC_PI_2 * k as f32 / 12.0;
            let u = hinge + (free - hinge) * a.sin();
            along(u, face + sign * width * a.cos())
        })
        .collect();
    d.poly(&arc, false, THIN, Ink::Line, false);
}

/// The usual drawing scales a fitted page chooses from, fine enough to fill the page.
const FIT_SCALES: [f32; 29] = [
    1.0, 2.0, 2.5, 5.0, 7.5, 10.0, 12.5, 15.0, 20.0, 25.0, 30.0, 40.0, 50.0, 60.0, 75.0, 100.0, 125.0, 150.0, 200.0, 250.0, 300.0, 400.0, 500.0, 750.0, 1000.0, 1250.0, 1500.0, 2000.0, 2500.0,
];

/// The description of room `ri`: that of its location identifier (`+KIT` → `Kitchen`)
/// in the structure management.
pub fn room_description(prj: &Project, ri: usize) -> Option<&str> {
    let id = room_location(&prj.layout.enclosures.get(ri)?.name);
    prj.structures.entry(crate::structure::Aspect::Location, &id).map(|e| e.description.as_str()).filter(|d| !d.trim().is_empty())
}

/// The location identifier a room is named by: `KIT` of `+KIT` (or of `KIT`).
pub fn room_location(name: &str) -> String {
    let dt = crate::structure::Dt::parse(name);
    let id = dt.get(crate::structure::Aspect::Location);
    if id.is_empty() { name.trim().trim_start_matches('+').to_string() } else { id.to_string() }
}

/// The largest usual drawing scale at which `real` (mm) fits into `space` (mm).
fn fit_scale(real: (f32, f32), space: (f32, f32)) -> f32 {
    if !(space.0 > 0.0 && space.1 > 0.0) {
        return FIT_SCALES[FIT_SCALES.len() - 1];
    }
    let needed = (real.0 / space.0).max(real.1 / space.1);
    FIT_SCALES
        .into_iter()
        .find(|n| *n >= needed)
        .unwrap_or(needed.ceil())
}

/// Draws into another [`Draw`] only what lies inside `lo`..`hi`: lines and outlines
/// are cut at the edge, filled areas clipped, texts and links outside left out.
struct Clip<'a> {
    d: &'a mut dyn Draw,
    lo: P,
    hi: P,
}

impl Clip<'_> {
    fn inside(&self, q: P) -> bool {
        q.x >= self.lo.x && q.x <= self.hi.x && q.y >= self.lo.y && q.y <= self.hi.y
    }

    /// The part of the segment `a`..`b` inside (Liang-Barsky).
    fn segment(&self, a: P, b: P) -> Option<(P, P)> {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let (mut t0, mut t1) = (0.0f32, 1.0f32);
        for (pp, q) in [(-dx, a.x - self.lo.x), (dx, self.hi.x - a.x), (-dy, a.y - self.lo.y), (dy, self.hi.y - a.y)] {
            if pp == 0.0 {
                if q < 0.0 {
                    return None;
                }
            } else {
                let r = q / pp;
                if pp < 0.0 {
                    t0 = t0.max(r);
                } else {
                    t1 = t1.min(r);
                }
            }
        }
        (t0 <= t1).then(|| (p(a.x + t0 * dx, a.y + t0 * dy), p(a.x + t1 * dx, a.y + t1 * dy)))
    }

    /// Polygon `pts` clipped to the area (Sutherland-Hodgman).
    fn polygon(&self, pts: &[P]) -> Vec<P> {
        let (lo, hi) = (self.lo, self.hi);
        // Edge `e` (left, right, top, bottom): whether `q` is on the inner side, and
        // where the segment `a`..`b` crosses it.
        let keep = |e: usize, q: P| [q.x >= lo.x, q.x <= hi.x, q.y >= lo.y, q.y <= hi.y][e];
        let cut = |e: usize, a: P, b: P| {
            let t = match e {
                0 => (lo.x - a.x) / (b.x - a.x),
                1 => (hi.x - a.x) / (b.x - a.x),
                2 => (lo.y - a.y) / (b.y - a.y),
                _ => (hi.y - a.y) / (b.y - a.y),
            };
            p(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y))
        };
        let mut out = pts.to_vec();
        for e in 0..4 {
            let src = std::mem::take(&mut out);
            for (k, &b) in src.iter().enumerate() {
                let a = src[(k + src.len() - 1) % src.len()];
                match (keep(e, a), keep(e, b)) {
                    (true, true) => out.push(b),
                    (true, false) => out.push(cut(e, a, b)),
                    (false, true) => out.extend([cut(e, a, b), b]),
                    (false, false) => {}
                }
            }
        }
        out
    }
}

impl Draw for Clip<'_> {
    fn line(&mut self, a: P, b: P, width: f32, ink: Ink, dashed: bool) {
        if let Some((a, b)) = self.segment(a, b) {
            self.d.line(a, b, width, ink, dashed);
        }
    }
    fn poly(&mut self, pts: &[P], closed: bool, width: f32, ink: Ink, fill: bool) {
        if pts.iter().all(|q| self.inside(*q)) {
            self.d.poly(pts, closed, width, ink, fill);
        } else if fill {
            let c = self.polygon(pts);
            if c.len() >= 3 {
                self.d.poly(&c, true, width, ink, true);
            }
        } else {
            let n = if closed { pts.len() } else { pts.len().saturating_sub(1) };
            for k in 0..n {
                self.line(pts[k], pts[(k + 1) % pts.len()], width, ink, false);
            }
        }
    }
    fn circle(&mut self, c: P, r: f32, width: f32, ink: Ink, fill: bool) {
        if self.inside(p(c.x - r, c.y - r)) && self.inside(p(c.x + r, c.y + r)) {
            self.d.circle(c, r, width, ink, fill);
        } else if c.x + r >= self.lo.x && c.x - r <= self.hi.x && c.y + r >= self.lo.y && c.y - r <= self.hi.y {
            let pts: Vec<P> = (0..48).map(|k| k as f32 * std::f32::consts::TAU / 48.0).map(|t| p(c.x + r * t.cos(), c.y + r * t.sin())).collect();
            self.poly(&pts, true, width, ink, fill);
        }
    }
    fn text(&mut self, pos: P, text: &str, size: f32, h: Anchor, v: VAnchor, ink: Ink) {
        if self.inside(pos) {
            self.d.text(pos, text, size, h, v, ink);
        }
    }
    fn link(&mut self, min: P, max: P, page: usize) {
        let (a, b) = (p(min.x.max(self.lo.x), min.y.max(self.lo.y)), p(max.x.min(self.hi.x), max.y.min(self.hi.y)));
        if a.x < b.x && a.y < b.y {
            self.d.link(a, b, page);
        }
    }
    fn layer(&mut self, name: Option<&str>) {
        self.d.layer(name);
    }
}

/// Draws into another [`Draw`] scaled by `k` and moved to `o` (a symbol of the
/// library, drawn at its size on schematics, in the size of plan symbols).
struct Scaled<'a> {
    d: &'a mut dyn Draw,
    o: P,
    k: f32,
}

impl Scaled<'_> {
    fn at(&self, q: P) -> P {
        p(self.o.x + q.x * self.k, self.o.y + q.y * self.k)
    }
}

impl Draw for Scaled<'_> {
    fn line(&mut self, a: P, b: P, width: f32, ink: Ink, dashed: bool) {
        let (a, b) = (self.at(a), self.at(b));
        self.d.line(a, b, width * self.k, ink, dashed);
    }
    fn poly(&mut self, pts: &[P], closed: bool, width: f32, ink: Ink, fill: bool) {
        let pts: Vec<P> = pts.iter().map(|q| self.at(*q)).collect();
        self.d.poly(&pts, closed, width * self.k, ink, fill);
    }
    fn circle(&mut self, c: P, r: f32, width: f32, ink: Ink, fill: bool) {
        let c = self.at(c);
        self.d.circle(c, r * self.k, width * self.k, ink, fill);
    }
    fn text(&mut self, pos: P, text: &str, size: f32, h: Anchor, v: VAnchor, ink: Ink) {
        let pos = self.at(pos);
        self.d.text(pos, text, size * self.k, h, v, ink);
    }
}

/// Size of the symbols of the library on a plan: their 4 mm from the connection
/// point to the middle become [`SYMBOL`] mm.
const LIBRARY_TO_PLAN: f32 = SYMBOL / 4.0;

/// The symbol `key` of the library (installation plan symbols, IEC 60617-11) at `c`
/// on the wall side `up`: its connection point (the cable at its top) on `c`, turned
/// by the quarter turn nearest to `up`; on the ceiling (`up` = `None`) centred on `c`.
pub fn symbol(d: &mut dyn Draw, prj: &Project, key: &str, c: P, up: Option<P>, ink: Ink) {
    symbol_sized(d, prj, key, c, up, ink, SYMBOL);
}

/// [`symbol`] at the size `size` mm (the plan symbol's 2.4 mm by default).
pub fn symbol_sized(d: &mut dyn Draw, prj: &Project, key: &str, c: P, up: Option<P>, ink: Ink, size: f32) {
    let Some(def) = prj.symbol(key).or_else(|| prj.symbol("inst_junction")) else { return };
    let u = up.unwrap_or(p(0.0, -1.0));
    // Quarter turns clockwise that bring the top of the symbol (-y) to `u`.
    let rot = if u.y.abs() >= u.x.abs() { if u.y < 0.0 { 0 } else { 2 } } else if u.x > 0.0 { 1 } else { 3 };
    let k = LIBRARY_TO_PLAN * size / SYMBOL;
    let o = match up {
        Some(u) => p(c.x - u.x * 4.0 * k, c.y - u.y * 4.0 * k),
        None => c,
    };
    let inst = SymbolInst { sym: key.to_string(), pos: p(0.0, 0.0), rot, ..Default::default() };
    draw_symbol(&mut Scaled { d, o, k }, &def, &inst, ink);
}

#[cfg(test)]
mod tests;
