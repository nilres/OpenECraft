//! Detection of walls, doors, windows and rooms in a floor plan on paper: a scan or a
//! picture of a drawing whose walls are filled, shaded or hatched (the usual way of
//! drawing cut walls), at right angles to the edges of the image.
//!
//! 1. **Ink:** the pixels clearly darker than the paper ([`Gray::paper`]).
//! 2. **Walls:** the ink with everything thinner than the thinnest wall taken away
//!    (morphological opening): text, dimension lines, door swings and the lines of
//!    windows go, the filled walls stay. Of what is left only the large connected
//!    parts are walls (stray blots and thick letters are not).
//! 3. **Openings:** gaps in a wall: along a row (or column), a stretch of free pixels
//!    between two wall pixels as long as a door or window, where at least one of the
//!    two ends is the end of a wall running along the row (its run across the row no
//!    thicker than a wall). Such stretches in neighbouring rows make one opening when it
//!    is as wide as about the wall is thick. The openings are closed with wall.
//! 4. **Rooms:** the free areas enclosed by the walls with their openings closed
//!    (those reaching the border of the image are outside). The outline of each is made
//!    of straight walls at right angles: cut along the lines where its border runs for
//!    some length, the cells of the cuts mostly inside the room make up its floor plan.
//! 5. **Doors and windows:** an opening between two rooms is a door; one between a room
//!    and the outside a window when lines run through it along the wall (glass and
//!    frame), else a door. A quarter circle of ink drawn from one end of a door into a
//!    room is its swing: the door is hinged at that end and opens into that room.
//!
//! Everything is measured in pixels of the image, from its top left corner (x to the
//! right, y down); [`Settings::mm_per_px`] gives the sizes of the settings in pixels.

use crate::raster::{Gray, Mask};

/// What the detection takes for a wall, an opening and a room, mm (at the scale of the
/// drawing).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Size of a pixel in the building, mm.
    pub mm_per_px: f32,
    /// Thinnest and thickest wall.
    pub min_wall: f32,
    pub max_wall: f32,
    /// Narrowest and widest door or window.
    pub min_opening: f32,
    pub max_opening: f32,
    /// Smallest room, mm².
    pub min_room: f32,
    /// Shortest step of a room's outline.
    pub min_step: f32,
    /// How much darker than the paper round it a pixel of ink is at least.
    pub contrast: u8,
}

impl Settings {
    pub fn new(mm_per_px: f32) -> Settings {
        Settings { mm_per_px, min_wall: 90.0, max_wall: 700.0, min_opening: 500.0, max_opening: 5000.0, min_room: 1.0e6, min_step: 150.0, contrast: INK_CONTRAST }
    }

    fn px(&self, mm: f32) -> f32 {
        mm / self.mm_per_px.max(1e-3)
    }
}

/// How much darker than the paper ink is at least by default (paper varies by one or
/// two levels, faint shading by fifteen).
pub const INK_CONTRAST: u8 = 10;

/// Widest opening between the end of a wall and a wall across it (a door beside a
/// corner), mm.
pub const MAX_DOOR: f32 = 1500.0;

/// A door or a window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GapKind {
    Door,
    Window,
}

/// An opening in a wall: the rectangle of the gap (pixels, `hi` excluded), whether the
/// wall runs along x, the rooms on its two sides (the side of smaller and of larger
/// coordinates across the wall; `None` outside) and, for a door with a swing drawn,
/// the side it opens to and whether it is hinged at its end of smaller coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gap {
    pub lo: [f32; 2],
    pub hi: [f32; 2],
    pub along_x: bool,
    pub kind: GapKind,
    pub sides: [Option<usize>; 2],
    pub swing: Option<(usize, bool)>,
}

impl Gap {
    /// Its width along the wall and the thickness of the wall, pixels.
    pub fn size(&self) -> (f32, f32) {
        let (dx, dy) = (self.hi[0] - self.lo[0], self.hi[1] - self.lo[1]);
        if self.along_x {
            (dx, dy)
        } else {
            (dy, dx)
        }
    }
}

/// A room: its outline (corners in pixels, at right angles) and its area in pixels.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Region {
    pub outline: Vec<[f32; 2]>,
    pub area: f32,
}

/// What was found in a floor plan.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scan {
    /// The pixels of the walls (openings not closed).
    pub walls: Mask,
    /// The straight pieces of the walls.
    pub bands: Vec<Band>,
    pub openings: Vec<Gap>,
    pub rooms: Vec<Region>,
    /// The ink: pixels clearly darker than the paper round them (lines of one pixel
    /// too).
    pub ink: Mask,
}

/// Finds the walls, openings and rooms of the floor plan `img`.
pub fn scan(img: &Gray, s: &Settings) -> Scan {
    // Walls from the image without specks; lines (glass, swings) from all of it.
    let smooth = img.median();
    let paper = smooth.background(s.px(s.max_wall).ceil() as usize + 2);
    let walls = wall_mask(&Mask::ink(&smooth, &paper, s.contrast), s);
    let ink = Mask::ink(img, &paper, s.contrast);
    let bands = bands(&walls, s);
    let mut gaps = openings(&walls, &bands, s);
    // Openings joining what is joined anyway inside the building (a blot in a room, a
    // wall ending in it) are none.
    let (label, parts) = free_parts(&walls, &gaps);
    gaps.retain(|g| {
        let [a, b] = sides(&label, img.w, img.h, g, s);
        a != b || a == 0 || parts[a as usize - 1].touches_border(img.w, img.h)
    });
    let (label, parts) = free_parts(&walls, &gaps);
    let min_area = s.min_room / (s.mm_per_px * s.mm_per_px);
    // Free parts enclosed by walls and large enough: the rooms (their label → index).
    let mut index = vec![None; parts.len() + 1];
    let mut rooms = Vec::new();
    for (k, c) in parts.iter().enumerate() {
        if c.touches_border(img.w, img.h) || (c.area as f32) < min_area {
            continue;
        }
        let outline = outline(&label, img.w, k as u32 + 1, c, s);
        if outline.len() >= 4 {
            index[k + 1] = Some(rooms.len());
            rooms.push(Region { area: polygon_area(&outline), outline });
        }
    }
    gaps.retain_mut(|g| {
        g.sides = sides(&label, img.w, img.h, g, s).map(|l| index[l as usize]);
        if g.sides == [None, None] {
            return false;
        }
        g.kind = if g.sides.contains(&None) && glazed(&ink, &walls, g) { GapKind::Window } else { GapKind::Door };
        if g.kind == GapKind::Door {
            g.swing = swing(&ink, &walls, g);
        }
        true
    });
    Scan { walls, bands, openings: gaps, rooms, ink }
}

/// The free parts of the plan with the openings closed: a label per pixel (0 for
/// walls) and the parts.
fn free_parts(walls: &Mask, gaps: &[Gap]) -> (Vec<u32>, Vec<crate::raster::Component>) {
    let mut closed = walls.clone();
    for g in gaps {
        closed.fill([(g.lo[0] as usize).saturating_sub(1), (g.lo[1] as usize).saturating_sub(1)], [g.hi[0] as usize + 1, g.hi[1] as usize + 1]);
    }
    closed.inverted().components()
}

/// The free parts (labels of [`free_parts`]) half a wall away from the gap on its two
/// sides: of three places along it, what most of them see.
fn sides(label: &[u32], w: usize, h: usize, g: &Gap, s: &Settings) -> [u32; 2] {
    let (len, thick) = g.size();
    let reach = thick / 2.0 + s.px(s.min_wall).max(3.0);
    let c = [(g.lo[0] + g.hi[0]) / 2.0, (g.lo[1] + g.hi[1]) / 2.0];
    let at = |x: f32, y: f32| {
        let (x, y) = (x.round(), y.round());
        if x < 0.0 || y < 0.0 || x >= w as f32 || y >= h as f32 {
            0
        } else {
            label[y as usize * w + x as usize]
        }
    };
    [-1.0, 1.0].map(|sign: f32| {
        let mut seen: Vec<u32> = [-0.3, 0.0, 0.3]
            .iter()
            .map(|t| if g.along_x { at(c[0] + t * len, c[1] + sign * reach) } else { at(c[0] + sign * reach, c[1] + t * len) })
            .collect();
        seen.sort_unstable();
        seen[1]
    })
}

/// The walls in the ink: what is at least as thick as the thinnest wall, in parts that
/// are large (at least a metre and a fifth of the largest part) and away from the
/// border of the image (the edges of a scanned sheet).
pub fn wall_mask(ink: &Mask, s: &Settings) -> Mask {
    let r = ((s.px(s.min_wall) - 1.0) / 2.0).floor().max(1.0) as usize;
    let thick = ink.opened(r);
    let (label, parts) = thick.components();
    let inner = |c: &crate::raster::Component| !c.near_border(ink.w, ink.h, 2 * r + 2);
    let largest = parts.iter().filter(|c| inner(c)).map(|c| c.span()).max().unwrap_or(0) as f32;
    let keep: Vec<bool> = parts.iter().map(|c| inner(c) && c.span() as f32 >= s.px(1000.0).max(largest / 5.0)).collect();
    Mask { w: ink.w, h: ink.h, on: label.iter().map(|&l| l != 0 && keep[l as usize - 1]).collect() }
}

/// A straight piece of wall: its rectangle (pixels, `hi` excluded) and whether it runs
/// along x.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    pub lo: [usize; 2],
    pub hi: [usize; 2],
    pub along_x: bool,
}

/// The runs of a band being found: its first and last line and the mean of the start
/// and end of its thin runs.
#[derive(Clone, Copy, Debug, Default)]
struct BandRuns {
    first: usize,
    last: usize,
    /// The last line with a thin run (the band ends there, not on the far side of a
    /// wall it meets).
    last_thin: usize,
    sum_a: f32,
    sum_b: f32,
    thin: u32,
}

impl BandRuns {
    fn add(&mut self, a: f32, b: f32) {
        self.sum_a += a;
        self.sum_b += b;
        self.thin += 1;
    }

    fn reference(&self) -> (f32, f32) {
        let n = self.thin.max(1) as f32;
        (self.sum_a / n, self.sum_b / n)
    }
}

/// The straight pieces of wall in a mask: along x, where the runs of the columns are
/// as thin as a wall and keep their place over at least twice their length (and
/// [`Settings::min_step`]); along y the same with the rows.
pub fn bands(walls: &Mask, s: &Settings) -> Vec<Band> {
    let (min_t, max_t) = (s.px(s.min_wall) * 0.7, s.px(s.max_wall));
    let mut out = Vec::new();
    for along_x in [true, false] {
        // Runs across the band, line by line along it: (start, end) per line.
        let (n, lines) = if along_x { (walls.h, walls.w) } else { (walls.w, walls.h) };
        let at = |line: usize, k: usize| if along_x { walls.on[k * walls.w + line] } else { walls.on[line * walls.w + k] };
        // A run as thin as a wall starts a band or continues one at about its place; a
        // run reaching over a band (a wall meeting it, a blot on its side) continues it
        // too. A band survives a few lines without either.
        let skip = 3;
        let mut open: Vec<BandRuns> = Vec::new();
        let mut done = Vec::new();
        for line in 0..lines {
            let mut k = 0;
            let mut runs = Vec::new();
            while k < n {
                if !at(line, k) {
                    k += 1;
                    continue;
                }
                let start = k;
                while k < n && at(line, k) {
                    k += 1;
                }
                runs.push((start, k));
            }
            let mut used = vec![false; open.len()];
            let mut started = Vec::new();
            for (a, b) in runs {
                let len = (b - a) as f32;
                let thin = len >= min_t && len <= max_t;
                let mut continued = false;
                for (g, used) in open.iter_mut().zip(used.iter_mut()) {
                    if *used || g.last >= line {
                        continue;
                    }
                    let (ra, rb) = g.reference();
                    let tol = (0.3 * (rb - ra)).max(2.0);
                    let (a, b) = (a as f32, b as f32);
                    let same = (a - ra).abs() <= tol && (b - rb).abs() <= tol;
                    let over = a <= ra + tol && b >= rb - tol;
                    if (thin && same && !continued) || (!thin && over) {
                        g.last = line;
                        if thin {
                            g.last_thin = line;
                            g.add(a, b);
                        }
                        *used = true;
                        continued = true;
                    }
                }
                if thin && !continued {
                    let mut g = BandRuns { first: line, last: line, last_thin: line, ..Default::default() };
                    g.add(a as f32, b as f32);
                    started.push(g);
                }
            }
            let (keep, gone): (Vec<_>, Vec<_>) = open.into_iter().partition(|g| g.last + skip >= line);
            done.extend(gone);
            open = keep;
            open.extend(started);
        }
        done.extend(open);
        for g in done {
            let (a, b) = g.reference();
            let (len, thick) = ((g.last_thin + 1 - g.first) as f32, b - a);
            if len >= (2.0 * thick).max(s.px(s.min_step)) && g.thin as f32 >= thick {
                let (a, b) = (a.round() as usize, b.round() as usize);
                let (lo, hi) = if along_x { ([g.first, a], [g.last_thin + 1, b]) } else { ([a, g.first], [b, g.last_thin + 1]) };
                out.push(Band { lo, hi, along_x });
            }
        }
    }
    out
}

/// The openings in the walls: from each end of a straight piece of wall, the gap along
/// it to the next wall (as long as a door or window, about the same in the rows across
/// the middle of the piece). Their sides and kind are not set yet.
fn openings(walls: &Mask, bands: &[Band], s: &Settings) -> Vec<Gap> {
    let (min_len, max_len) = (s.px(s.min_opening), s.px(s.max_opening));
    let mut out: Vec<Gap> = Vec::new();
    // Runs along y and along x: across a wall along x and along y.
    let runs = [walls.runs(true), walls.runs(false)];
    for (i, b) in bands.iter().enumerate() {
        let (axis, cross) = if b.along_x { (0, 1) } else { (1, 0) };
        let (c0, c1) = (b.lo[cross], b.hi[cross]);
        let t = c1 - c0;
        let (m0, m1) = (c0 + t / 5, (c1 - t / 5).max(c0 + t / 5 + 1));
        let size = if axis == 0 { walls.w } else { walls.h };
        let pixel = |k: isize, c: isize| {
            let mut q = [0isize; 2];
            q[axis] = k;
            q[cross] = c;
            walls.at(q[0], q[1])
        };
        for dir in [-1isize, 1] {
            let end = if dir < 0 { b.lo[axis] as isize } else { b.hi[axis] as isize - 1 };
            // A wall ending there, not where it meets another one (a piece across it at
            // its end).
            let joined = bands.iter().enumerate().any(|(j, o)| {
                let (a0, a1) = (o.lo[axis] as isize, o.hi[axis] as isize);
                let tol = (t / 2).max(3);
                j != i && o.along_x != b.along_x && a0 - 6 <= end && end <= a1 + 5 && o.lo[cross] <= c1 + tol && o.hi[cross] + tol >= c0
            });
            if joined {
                continue;
            }
            let mut found: Vec<(isize, usize)> = Vec::new();
            for c in m0..m1 {
                let inside = |k: isize| k >= 0 && (k as usize) < size;
                let get = |k: isize| {
                    let mut q = [0isize; 2];
                    q[axis] = k;
                    q[cross] = c as isize;
                    walls.at(q[0], q[1])
                };
                // From inside the piece to where its wall ends in this row, then on to the
                // next wall.
                let mut k = end - 2 * dir;
                while inside(k) && get(k) {
                    k += dir;
                }
                // The wall goes on beyond the end of the piece: no gap here.
                if (k - end) * dir > t as isize + 4 {
                    continue;
                }
                let at = |len: usize| k + dir * len as isize;
                let mut len = 0;
                while inside(at(len)) && !get(at(len)) && len as f32 <= max_len {
                    len += 1;
                }
                if inside(at(len)) && get(at(len)) && len as f32 >= min_len {
                    found.push((k, len));
                }
            }
            // The gap ends where most rows meet the next wall (blots on its face end some
            // rows earlier, a row passing beside the next wall later).
            let need = ((m1 - m0) * 6).div_ceil(10).max(1);
            if found.len() < need {
                continue;
            }
            let mut starts: Vec<isize> = found.iter().map(|f| f.0).collect();
            let mut lens: Vec<usize> = found.iter().map(|f| f.1).collect();
            starts.sort_unstable();
            lens.sort_unstable();
            let (f, len) = (starts[starts.len() / 2], lens[need - 1] as isize);
            // Ending at a wall across it (a door beside a corner): no wider than a door;
            // only between two pieces of the same wall as wide as a window.
            let hit = f + dir * len;
            let mid = (c0 + c1) as isize / 2;
            let run = |k: isize| {
                let mut q = [0isize; 2];
                q[axis] = k;
                q[cross] = mid;
                if pixel(k, mid) { runs[usize::from(b.along_x)][q[1] as usize * walls.w + q[0] as usize] } else { u32::MAX }
            };
            let thick = (2..=(t as isize / 2).max(2)).map(|j| run(hit + dir * j)).min().unwrap_or(u32::MAX);
            // Wider than a door: a window between two pieces of the same wall (the next
            // piece starts where the gap ends).
            let piece = bands.iter().any(|o| {
                let overlap = o.hi[cross].min(c1) as isize - o.lo[cross].max(c0) as isize;
                let near = if dir > 0 { o.lo[axis] as isize } else { o.hi[axis] as isize - 1 };
                o.along_x == b.along_x && 2 * overlap >= t.min(o.hi[cross] - o.lo[cross]) as isize && (near - hit).abs() <= t as isize + 2
            });
            let across = thick as f32 > 1.5 * t as f32 + 4.0;
            if len as f32 > s.px(MAX_DOOR) && (across || !piece) {
                continue;
            }
            let (a, z) = if dir > 0 { (f, f + len) } else { (f - len + 1, f + 1) };
            let mut lo = [0.0; 2];
            let mut hi = [0.0; 2];
            lo[axis] = a as f32;
            hi[axis] = z as f32;
            lo[cross] = c0 as f32;
            hi[cross] = c1 as f32;
            out.push(Gap { lo, hi, along_x: b.along_x, kind: GapKind::Door, sides: [None, None], swing: None });
        }
    }
    // The same gap found from both of its ends, or along both directions (a square
    // hole: the one across the thinner wall).
    let overlap = |a: &Gap, b: &Gap| a.lo[0] < b.hi[0] && b.lo[0] < a.hi[0] && a.lo[1] < b.hi[1] && b.lo[1] < a.hi[1];
    let mut keep = vec![true; out.len()];
    for i in 0..out.len() {
        for j in i + 1..out.len() {
            if keep[i] && keep[j] && overlap(&out[i], &out[j]) {
                let drop = if out[i].size().1 <= out[j].size().1 { j } else { i };
                keep[drop] = false;
            }
        }
    }
    out.into_iter().zip(keep).filter(|(_, k)| *k).map(|(g, _)| g).collect()
}

/// Lines of ink run through the gap along the wall (a window: glass, frame, sill).
fn glazed(dark: &Mask, walls: &Mask, g: &Gap) -> bool {
    let (len, thick) = g.size();
    if len < 4.0 || thick < 2.0 {
        return false;
    }
    // Along the gap (its ends left out), the share of places with ink across it.
    let (lo, hi) = (g.lo.map(|v| v as usize), g.hi.map(|v| v as usize));
    let margin = (len * 0.1) as usize;
    let mut with = 0;
    let mut all = 0;
    let (a, b) = if g.along_x { (lo[0] + margin, hi[0].saturating_sub(margin)) } else { (lo[1] + margin, hi[1].saturating_sub(margin)) };
    let (c, d) = if g.along_x { (lo[1], hi[1]) } else { (lo[0], hi[0]) };
    for k in a..b {
        all += 1;
        let ink = (c..d).filter(|&m| {
            let (x, y) = if g.along_x { (k, m) } else { (m, k) };
            dark.get(x, y) && !walls.get(x, y)
        });
        if ink.count() > 0 {
            with += 1;
        }
    }
    all > 0 && with * 10 >= all * 7
}

/// The swing of a door drawn as a quarter circle: the side it opens to and whether its
/// hinge is at the end of smaller coordinates along the wall.
fn swing(dark: &Mask, walls: &Mask, g: &Gap) -> Option<(usize, bool)> {
    let (len, _) = g.size();
    let mut best: Option<((usize, bool), f32)> = None;
    for side in 0..2 {
        g.sides[side]?;
        for hinge_lo in [true, false] {
            // The hinge on the face of the wall on that side, the leaf closed along the
            // wall, the quarter circle into the room.
            let along = if hinge_lo { 0.0 } else { len };
            let dir_along = if hinge_lo { 1.0 } else { -1.0 };
            let dir_across = if side == 0 { -1.0 } else { 1.0 };
            let (axis, cross) = if g.along_x { (0, 1) } else { (1, 0) };
            let face = if side == 0 { g.lo[cross] } else { g.hi[cross] };
            let mut centre = [0.0; 2];
            centre[axis] = g.lo[axis] + along;
            centre[cross] = face;
            let steps = 16;
            let hits = (1..steps)
                .filter(|&i| {
                    let t = std::f32::consts::FRAC_PI_2 * i as f32 / steps as f32;
                    let mut q = [0.0; 2];
                    q[axis] = centre[axis] + dir_along * len * t.cos();
                    q[cross] = centre[cross] + dir_across * len * t.sin();
                    (-2..=2).any(|dy: i32| (-2..=2).any(|dx: i32| {
                        let (x, y) = (q[0].round() as isize + dx as isize, q[1].round() as isize + dy as isize);
                        dark.at(x, y) && !walls.at(x, y)
                    }))
                })
                .count();
            let score = hits as f32 / (steps - 1) as f32;
            if score >= 0.7 && best.is_none_or(|b| score > b.1) {
                best = Some(((side, hinge_lo), score));
            }
        }
    }
    best.map(|b| b.0)
}

/// Positions of the lines along which the border of a room runs for at least
/// `min_len` pixels: from the count of border pixels per line, its peaks (no two
/// closer than `apart`), each at the mean place of its border pixels.
fn cuts(count: &[u32], min_len: f32, apart: usize) -> Vec<f32> {
    let n = count.len();
    let window = 2usize;
    let sum = |k: usize| -> u32 { count[k.saturating_sub(window)..(k + window + 1).min(n)].iter().sum() };
    let sums: Vec<u32> = (0..n).map(sum).collect();
    let mut peaks: Vec<usize> = (0..n).filter(|&k| sums[k] as f32 >= min_len).collect();
    // Strongest first, none within `apart` of a stronger one.
    peaks.sort_by(|a, b| sums[*b].cmp(&sums[*a]).then(a.cmp(b)));
    let mut kept: Vec<usize> = Vec::new();
    for k in peaks {
        if kept.iter().all(|&j| j.abs_diff(k) > apart) {
            kept.push(k);
        }
    }
    let mut out: Vec<f32> = kept
        .into_iter()
        .map(|k| {
            let (lo, hi) = (k.saturating_sub(window), (k + window + 1).min(n));
            let total: u32 = count[lo..hi].iter().sum();
            let at: f32 = (lo..hi).map(|j| j as f32 * count[j] as f32).sum();
            if total == 0 {
                k as f32
            } else {
                at / total as f32
            }
        })
        .collect();
    out.sort_by(f32::total_cmp);
    out
}

/// The outline of the room labelled `id` (see the module documentation), corners in
/// pixels; empty when it has none.
fn outline(label: &[u32], w: usize, id: u32, c: &crate::raster::Component, s: &Settings) -> Vec<[f32; 2]> {
    let (x0, y0, x1, y1) = (c.lo[0], c.lo[1], c.hi[0], c.hi[1]);
    let (cw, ch) = (x1 - x0, y1 - y0);
    let inside = |x: isize, y: isize| x >= 0 && y >= 0 && (x as usize) < cw && (y as usize) < ch && label[(y as usize + y0) * w + x as usize + x0] == id;
    // Border pixels per column and per row (between a pixel and the one before it).
    let mut cols = vec![0u32; cw + 1];
    let mut rows = vec![0u32; ch + 1];
    for y in 0..=ch as isize {
        for x in 0..=cw as isize {
            let here = inside(x, y);
            if here != inside(x - 1, y) {
                cols[x as usize] += 1;
            }
            if here != inside(x, y - 1) {
                rows[y as usize] += 1;
            }
        }
    }
    let min_len = s.px(s.min_step).max(4.0);
    // No two cuts closer than half a step (the walls of the outline get at least that
    // long).
    let apart = (s.px(s.min_step) / 2.0).max(2.0) as usize;
    let mut xs = cuts(&cols, min_len, apart);
    let mut ys = cuts(&rows, min_len, apart);
    for (v, end) in [(&mut xs, cw as f32), (&mut ys, ch as f32)] {
        if v.first().is_none_or(|f| *f > apart as f32) {
            v.insert(0, 0.0);
        }
        if v.last().is_none_or(|l| *l < end - apart as f32) {
            v.push(end);
        }
    }
    // Cells mostly inside the room (summed area table of the room's pixels).
    let mut sat = vec![0u32; (cw + 1) * (ch + 1)];
    for y in 0..ch {
        let mut row = 0;
        for x in 0..cw {
            row += u32::from(inside(x as isize, y as isize));
            sat[(y + 1) * (cw + 1) + x + 1] = sat[y * (cw + 1) + x + 1] + row;
        }
    }
    let area = |xa: usize, ya: usize, xb: usize, yb: usize| -> u32 { sat[yb * (cw + 1) + xb] + sat[ya * (cw + 1) + xa] - sat[ya * (cw + 1) + xb] - sat[yb * (cw + 1) + xa] };
    let (nx, ny) = (xs.len() - 1, ys.len() - 1);
    // How much of each cell lies in the room.
    let mut share = vec![0.0f32; nx * ny];
    for j in 0..ny {
        for i in 0..nx {
            let (xa, xb) = (xs[i].round() as usize, (xs[i + 1].round() as usize).min(cw));
            let (ya, yb) = (ys[j].round() as usize, (ys[j + 1].round() as usize).min(ch));
            if xb > xa && yb > ya {
                share[j * nx + i] = area(xa, ya, xb, yb) as f32 / ((xb - xa) * (yb - ya)) as f32;
            }
        }
    }
    let mut cell: Vec<bool> = share.iter().map(|&v| v >= 0.5).collect();
    // Cells meeting only at a corner would make walls touch: the fuller of the two
    // others joins them.
    for _ in 0..nx * ny {
        let pinch = (0..ny.saturating_sub(1)).flat_map(|j| (0..nx.saturating_sub(1)).map(move |i| (i, j))).find_map(|(i, j)| {
            let [a, b, c, d] = [j * nx + i, j * nx + i + 1, (j + 1) * nx + i, (j + 1) * nx + i + 1];
            match (cell[a], cell[b], cell[c], cell[d]) {
                (true, false, false, true) => Some(if share[b] >= share[c] { b } else { c }),
                (false, true, true, false) => Some(if share[a] >= share[d] { a } else { d }),
                _ => None,
            }
        });
        match pinch {
            Some(k) => cell[k] = true,
            None => break,
        }
    }
    let corners = trace(&cell, nx, ny);
    corners.into_iter().map(|[i, j]| [x0 as f32 + xs[i], y0 as f32 + ys[j]]).collect()
}

/// The outer border of the largest group of cells that are on (4-connected), as grid
/// corners (`[i, j]`: column and row of the cuts), without corners in the middle of a
/// straight side.
fn trace(cell: &[bool], nx: usize, ny: usize) -> Vec<[usize; 2]> {
    let on = |i: isize, j: isize| i >= 0 && j >= 0 && (i as usize) < nx && (j as usize) < ny && cell[j as usize * nx + i as usize];
    // The largest group only.
    let mask = Mask { w: nx, h: ny, on: cell.to_vec() };
    let (label, parts) = mask.components();
    let Some(big) = (0..parts.len()).max_by_key(|&k| parts[k].area) else {
        return Vec::new();
    };
    let ours = |i: isize, j: isize| on(i, j) && label[j as usize * nx + i as usize] == big as u32 + 1;
    // Directed border edges with the room on their left (y down: clockwise on the
    // screen), keyed by their start corner.
    let mut next: std::collections::HashMap<[usize; 2], Vec<[usize; 2]>> = std::collections::HashMap::new();
    let mut start = None;
    for j in 0..ny as isize {
        for i in 0..nx as isize {
            if !ours(i, j) {
                continue;
            }
            let (iu, ju) = (i as usize, j as usize);
            let mut edge = |a: [usize; 2], b: [usize; 2]| {
                next.entry(a).or_default().push(b);
                start.get_or_insert(a);
            };
            if !ours(i, j - 1) {
                edge([iu + 1, ju], [iu, ju]);
            }
            if !ours(i - 1, j) {
                edge([iu, ju], [iu, ju + 1]);
            }
            if !ours(i, j + 1) {
                edge([iu, ju + 1], [iu + 1, ju + 1]);
            }
            if !ours(i + 1, j) {
                edge([iu + 1, ju + 1], [iu + 1, ju]);
            }
        }
    }
    // Follow the edges from the topmost leftmost corner, which lies on the outer border.
    let Some(first) = next.keys().min_by_key(|k| (k[1], k[0])).copied().or(start) else {
        return Vec::new();
    };
    let mut path = vec![first];
    let mut at = first;
    let mut dir: [isize; 2] = [0, 0];
    for _ in 0..4 * (nx + 1) * (ny + 1) {
        let Some(outs) = next.get_mut(&at) else { break };
        if outs.is_empty() {
            break;
        }
        // At a corner where two parts touch, turn the sharpest way to stay on the outside.
        let k = (0..outs.len())
            .max_by_key(|&k| {
                let d = [outs[k][0] as isize - at[0] as isize, outs[k][1] as isize - at[1] as isize];
                dir[0] * d[1] - dir[1] * d[0]
            })
            .unwrap_or(0);
        let b = outs.swap_remove(k);
        dir = [b[0] as isize - at[0] as isize, b[1] as isize - at[1] as isize];
        at = b;
        if at == first {
            break;
        }
        path.push(at);
    }
    // Corners only where the direction changes.
    let n = path.len();
    (0..n)
        .filter(|&k| {
            let (p, q, r) = (path[(k + n - 1) % n], path[k], path[(k + 1) % n]);
            !((p[0] == q[0] && q[0] == r[0]) || (p[1] == q[1] && q[1] == r[1]))
        })
        .map(|k| path[k])
        .collect()
}

/// Area of a polygon (absolute).
pub fn polygon_area(pts: &[[f32; 2]]) -> f32 {
    let n = pts.len();
    ((0..n).map(|k| pts[k][0] * pts[(k + 1) % n][1] - pts[(k + 1) % n][0] * pts[k][1]).sum::<f32>() / 2.0).abs()
}

#[cfg(test)]
mod tests;
