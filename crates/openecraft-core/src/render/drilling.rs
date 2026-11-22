//! Drilling pattern pages: the faces of an enclosure that have holes or cutouts, each at
//! its own scale (the first, the mounting plate, over the whole height of the page, the
//! others in a grid beside it), every hole and cutout with its letter, dimensioned from
//! the bottom left corner of its face as the NC data ([`crate::production::nc_csv`]),
//! and the legend of the kinds with their sizes, threads, operations and counts.

use super::*;
use crate::production::{dim, drilling, FaceDrilling};
use crate::text::TextStyle;

/// Height of the dimension and label texts, and the least distance between two
/// dimension texts side by side (mm on the page).
const TEXT: f32 = 1.8;
const STEP: f32 = 2.2;
/// Room around a face for its title (above), its dimensions (below and left) and the
/// next face (right), and for the legend at the right of the page.
const ABOVE: f32 = 8.0;
const BELOW: f32 = 15.0;
const LEFT: f32 = 15.0;
const RIGHT: f32 = 6.0;
const LEGEND: f32 = 78.0;

pub(super) fn draw_page(d: &mut dyn Draw, prj: &Project, id: u64, frame: &crate::frame::PlotFrame) {
    let a = frame.area();
    let Some(e) = prj.layout.enclosures.iter().find(|e| e.id == id) else {
        let msg = tr("Enclosure not found - choose another one in the page properties");
        d.text(a.min + p(15.0, 25.0), msg, 4.0, Anchor::Left, VAnchor::Middle, Ink::Error);
        return;
    };
    let faces = drilling(e, &prj.parts);
    if faces.is_empty() {
        d.text(a.min + p(15.0, 25.0), tr("No holes or cutouts in this enclosure"), 4.0, Anchor::Left, VAnchor::Middle, Ink::Text);
        return;
    }
    let area = R::from_points(p(a.min.x + 6.0, a.min.y + 6.0), p(a.max.x - LEGEND - 6.0, a.max.y - 12.0));
    for (f, (o, n)) in faces.iter().zip(layout(&faces, area)) {
        draw_face(d, f, o, n);
    }
    legend(d, e, &faces, p(a.max.x - LEGEND - 2.0, a.min.y + 14.0));
    let caption = tr_fmt!("{} {} - dimensions in mm from the bottom left corner of each face", e.name, e.part);
    d.text(p(a.min.x + 8.0, a.max.y - 7.0), &caption, 3.0, Anchor::Left, VAnchor::Middle, Ink::Text);
}

/// Where the faces go in `area` and their scales (1:n): the first over the whole height
/// at the largest scale that leaves at least half of the width, the others in the grid
/// of columns and rows beside it that draws them largest.
fn layout(faces: &[FaceDrilling], area: R) -> Vec<(P, f32)> {
    let cell = |(fw, fh): (f32, f32), (w, h): (f32, f32)| drawing_scale((fw, fh), ((w - LEFT - RIGHT).max(10.0), (h - ABOVE - BELOW).max(10.0)));
    let (w, h) = (area.max.x - area.min.x, area.max.y - area.min.y);
    let origin = |q: P| p(q.x + LEFT, q.y + ABOVE);
    if faces.len() == 1 {
        return vec![(origin(area.min), cell(faces[0].size, (w, h)))];
    }
    let first = cell(faces[0].size, (w / 2.0, h));
    let used = faces[0].size.0 / first + LEFT + RIGHT;
    let mut out = vec![(origin(area.min), first)];
    let rest = &faces[1..];
    let (left, width) = (area.min.x + used, w - used);
    let drawn = |cols: usize| {
        let rows = rest.len().div_ceil(cols);
        let size = (width / cols as f32, h / rows as f32);
        let scales: Vec<f32> = rest.iter().map(|f| cell(f.size, size)).collect();
        let area: f32 = rest.iter().zip(&scales).map(|(f, n)| f.size.0 * f.size.1 / (n * n)).sum();
        (area, cols, size, scales)
    };
    let (_, cols, size, scales) = (1..=rest.len()).map(drawn).max_by(|a, b| a.0.total_cmp(&b.0)).unwrap_or_else(|| drawn(1));
    for (i, n) in scales.into_iter().enumerate() {
        out.push((origin(p(left + (i % cols) as f32 * size.0, area.min.y + (i / cols) as f32 * size.1)), n));
    }
    out
}

/// One face at `o` (its top left corner on the page) at the scale 1:`n`: its outline, the
/// holes and cutouts with their letters and the ordinate dimensions of their middles
/// below it (x) and left of it (y).
fn draw_face(d: &mut dyn Draw, f: &FaceDrilling, o: P, n: f32) {
    let k = 1.0 / n;
    let (fw, fh) = f.size;
    let at = |q: P| p(o.x + q.x * k, o.y + q.y * k);
    d.rect(o, at(p(fw, fh)), 0.35, Ink::Frame, false);
    let title = if f.face == Face::Door { tr("Door (front view)") } else { f.face.title() };
    d.text(p(o.x + fw * k / 2.0, o.y - 2.0), &format!("{title} (1:{n})"), 3.0, Anchor::Center, VAnchor::Bottom, Ink::Text);
    for x in &f.features {
        let c = at(x.pos);
        let kind = &f.kinds[x.kind];
        let r = match &kind.hole {
            Some(h) => {
                let r = (h.diameter / 2.0 * k).max(0.4);
                d.circle(c, r, 0.25, Ink::Line, false);
                // A tapped hole shows its thread as a wider thin circle.
                if !h.thread.is_empty() {
                    let nominal: f32 = h.thread[1..].parse().unwrap_or(h.diameter);
                    d.circle(c, (nominal / 2.0 * k).max(r + 0.2), 0.13, Ink::Line, false);
                }
                r
            }
            None => {
                let pts: Vec<P> = x.outline.iter().map(|q| at(*q)).collect();
                d.poly(&pts, true, 0.25, Ink::Line, false);
                let b = R::from_points(pts.iter().copied().fold(c, |m, q| p(m.x.min(q.x), m.y.min(q.y))), pts.iter().copied().fold(c, |m, q| p(m.x.max(q.x), m.y.max(q.y))));
                (b.max.x - b.min.x).min(b.max.y - b.min.y) / 2.0
            }
        };
        let arm = r + 0.8;
        d.line(p(c.x - arm, c.y), p(c.x + arm, c.y), 0.13, Ink::Frame, false);
        d.line(p(c.x, c.y - arm), p(c.x, c.y + arm), 0.13, Ink::Frame, false);
        d.text(p(c.x + r * 0.7 + 0.3, c.y - r * 0.7 - 0.3), &kind.label, 1.6, Anchor::Left, VAnchor::Bottom, Ink::Text);
    }
    // Ordinate dimensions: the zero at the corner, then every distinct x and y of the
    // middles; texts that would touch move on, joined to their place by a bent leader.
    let bottom = o.y + fh * k;
    let distinct = |mut v: Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v.dedup_by(|a, b| (*a - *b).abs() < 0.05);
        v
    };
    let xs = distinct(std::iter::once(0.0).chain(f.features.iter().map(|x| x.x)).collect());
    // Side by side over the width of the face and the room right of it.
    let step = STEP.min((fw * k + RIGHT) / xs.len() as f32).max(1.2);
    let mut last = f32::MIN;
    for v in xs {
        let px = o.x + v * k;
        let lx = px.max(last + step);
        last = lx;
        d.line(p(px, bottom + 0.8), p(px, bottom + 3.0), 0.13, Ink::Frame, false);
        d.line(p(px, bottom + 3.0), p(lx, bottom + 5.0), 0.13, Ink::Frame, false);
        let style = TextStyle { angle: 90.0, ..TextStyle::plain(TEXT.min(0.85 * step), Anchor::Right, VAnchor::Middle) };
        d.styled_text(p(lx, bottom + 5.5), &dim(v), &style, Ink::Text);
    }
    let ys = distinct(std::iter::once(0.0).chain(f.features.iter().map(|x| x.y)).collect());
    let mut last = f32::MAX;
    for v in ys {
        let py = bottom - v * k;
        let ly = py.min(last - STEP);
        last = ly;
        d.line(p(o.x - 0.8, py), p(o.x - 3.0, py), 0.13, Ink::Frame, false);
        d.line(p(o.x - 3.0, py), p(o.x - 5.0, ly), 0.13, Ink::Frame, false);
        d.text(p(o.x - 5.5, ly), &dim(v), TEXT, Anchor::Right, VAnchor::Middle, Ink::Text);
    }
}

/// The kinds of holes and cutouts of every face: letter, size, thread, operation and
/// count.
fn legend(d: &mut dyn Draw, e: &Enclosure, faces: &[FaceDrilling], o: P) {
    let mut y = o.y;
    d.text(p(o.x, y), &tr_fmt!("Holes and cutouts {}", e.name), 3.0, Anchor::Left, VAnchor::Bottom, Ink::Text);
    y += 3.0;
    for f in faces {
        y += 3.5;
        let title = if f.face == Face::Door { tr("Door (front view)") } else { f.face.title() };
        d.text(p(o.x, y), title, 2.5, Anchor::Left, VAnchor::Bottom, Ink::Text);
        d.line(p(o.x, y + 0.6), p(o.x + LEGEND - 4.0, y + 0.6), 0.13, Ink::Frame, false);
        for kind in &f.kinds {
            y += 3.0;
            d.text(p(o.x + 1.0, y), &kind.label, TEXT, Anchor::Left, VAnchor::Bottom, Ink::Text);
            d.text(p(o.x + 7.0, y), &kind.size(), TEXT, Anchor::Left, VAnchor::Bottom, Ink::Text);
            d.text(p(o.x + 24.0, y), &kind.thread(), TEXT, Anchor::Left, VAnchor::Bottom, Ink::Text);
            d.text(p(o.x + 33.0, y), kind.operation(), TEXT, Anchor::Left, VAnchor::Bottom, Ink::Text);
            d.text(p(o.x + LEGEND - 4.0, y), &format!("{}", kind.count), TEXT, Anchor::Right, VAnchor::Bottom, Ink::Text);
        }
    }
}

#[cfg(test)]
mod tests;
