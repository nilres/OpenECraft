//! Stairs of a building ([`Stair`]): from the floor of one level up to the floor of
//! another, straight, turned at landings or winding round a newel. Their steps are
//! worked out from the levels: as many risers as give a rise near [`RISE`], each step
//! as deep as the stair's going. The steps and landings come out as prisms in the
//! building's floor plan ([`geometry`]) for the floor plans and the 3D view.

use super::buildings::find_level;
use crate::issue::{Issue, Severity};
use crate::model::panel::{Building, Stair, StairKind, Turn};
use crate::model::Project;
use openecraft_i18n::{tr, tr_const};

/// Rise of a step aimed at, mm.
pub const RISE: f32 = 175.0;
/// Rises outside this range are reported, mm.
pub const RISE_RANGE: (f32, f32) = (140.0, 200.0);
/// Step rule: twice the rise and the going together, mm.
pub const STEP_RULE: (f32, f32) = (590.0, 650.0);
/// Radius of the newel of a spiral stair, mm.
pub const NEWEL: f32 = 100.0;
/// Gap between the two flights of a stair with a half landing (stairwell), mm.
pub const WELL: f32 = 100.0;
/// Thickness of a landing, mm.
pub const LANDING_SLAB: f32 = 200.0;

impl StairKind {
    pub const ALL: [StairKind; 5] = [StairKind::Straight, StairKind::QuarterLanding, StairKind::TwoQuarterLandings, StairKind::HalfLanding, StairKind::Spiral];

    pub fn key(&self) -> &'static str {
        match self {
            StairKind::Straight => "straight",
            StairKind::QuarterLanding => "quarter_landing",
            StairKind::TwoQuarterLandings => "two_quarter_landings",
            StairKind::HalfLanding => "half_landing",
            StairKind::Spiral => "spiral",
        }
    }

    pub fn from_key(key: &str) -> Option<StairKind> {
        Self::ALL.into_iter().find(|k| k.key().eq_ignore_ascii_case(key.trim()))
    }

    /// Untranslated title (`tr` where shown).
    pub fn title(&self) -> &'static str {
        match self {
            StairKind::Straight => tr_const!("Straight"),
            StairKind::QuarterLanding => tr_const!("Turned once (quarter landing)"),
            StairKind::TwoQuarterLandings => tr_const!("Turned twice (two quarter landings)"),
            StairKind::HalfLanding => tr_const!("Half landing (two flights)"),
            StairKind::Spiral => tr_const!("Spiral"),
        }
    }
}

impl Turn {
    pub fn key(&self) -> &'static str {
        match self {
            Turn::Left => "left",
            Turn::Right => "right",
        }
    }

    pub fn from_key(key: &str) -> Option<Turn> {
        [Turn::Left, Turn::Right].into_iter().find(|k| k.key().eq_ignore_ascii_case(key.trim()))
    }

    /// Untranslated title (`tr` where shown).
    pub fn title(&self) -> &'static str {
        match self {
            Turn::Left => tr_const!("Left"),
            Turn::Right => tr_const!("Right"),
        }
    }
}

/// A step, a landing or the newel: a prism from `bottom` up to `top` (heights above the
/// building's ground level) over a convex outline in the building's floor plan.
#[derive(Clone, Debug, PartialEq)]
pub struct StairPiece {
    pub outline: Vec<[f32; 2]>,
    pub bottom: f32,
    pub top: f32,
    pub landing: bool,
}

/// The steps of a stair worked out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StairGeometry {
    pub pieces: Vec<StairPiece>,
    /// The walking line from the foot up to the head (floor plan).
    pub walk: Vec<[f32; 2]>,
    /// Number of risers and their height, mm.
    pub risers: usize,
    pub rise: f32,
    /// Heights of the floors it starts on and leads up to.
    pub from: f32,
    pub to: f32,
}

/// Floor heights (above the building's ground level) of the levels a stair joins.
fn floors(b: &Building, s: &Stair) -> Option<(f32, f32)> {
    let at = |id: u64| b.levels.iter().find(|l| l.id == id).map(|l| l.elevation);
    Some((at(s.bottom)?, at(s.top)?))
}

/// A stair's steps and landings in the building's floor plan; `None` when its levels
/// are missing or its head is not above its foot.
pub fn geometry(b: &Building, s: &Stair) -> Option<StairGeometry> {
    let (from, to) = floors(b, s)?;
    if to - from < 1.0 {
        return None;
    }
    let risers = ((to - from) / RISE).round().max(2.0) as usize;
    let rise = (to - from) / risers as f32;
    // The treads from the first one up (the last riser leads onto the upper floor):
    // steps and landings, at the heights of 1 … risers - 1 risers.
    let levels = risers - 1;
    let (w, a) = (s.width.max(300.0), s.tread.max(100.0));
    let landings = match s.kind {
        StairKind::Straight | StairKind::Spiral => 0,
        StairKind::QuarterLanding | StairKind::HalfLanding => 1,
        StairKind::TwoQuarterLandings => 2,
    }
    .min(levels);
    // Steps per flight, the lower flights get the odd ones.
    let flights = landings + 1;
    let steps = levels - landings;
    let per: Vec<usize> = (0..flights).map(|k| steps / flights + usize::from(k < steps % flights)).collect();
    // Local coordinates: u up the first flight from the middle of its foot, v to the
    // left; a right turn is the mirror image.
    let mut pieces: Vec<(Vec<[f32; 2]>, usize, bool)> = Vec::new();
    let mut walk: Vec<[f32; 2]> = vec![[0.0, 0.0]];
    let add = |c: [f32; 2], k: f32, d: [f32; 2]| [c[0] + d[0] * k, c[1] + d[1] * k];
    let mut level = 1;
    if s.kind == StairKind::Spiral {
        let r_walk = NEWEL + w / 2.0;
        let centre = [0.0, r_walk];
        let step = a / r_walk;
        let start = (-1.0f32).atan2(0.0);
        let at = |r: f32, t: f32| [centre[0] + r * t.cos(), centre[1] + r * t.sin()];
        for k in 0..levels {
            let (t0, t1) = (start + step * k as f32, start + step * (k + 1) as f32);
            let tm = (t0 + t1) / 2.0;
            pieces.push((vec![at(NEWEL, t0), at(NEWEL + w, t0), at(NEWEL + w, tm), at(NEWEL + w, t1), at(NEWEL, t1)], k + 1, false));
            walk.push(at(r_walk, t1));
        }
        let newel: Vec<[f32; 2]> = (0..8).map(|k| at(NEWEL, std::f32::consts::TAU * k as f32 / 8.0)).collect();
        let mut g = place(s, pieces, walk, risers, rise, from, to);
        g.pieces.push(StairPiece { outline: newel.into_iter().map(|q| to_plan(s, q)).collect(), bottom: from, top: to, landing: false });
        return Some(g);
    }
    let (mut c, mut d) = ([0.0f32, 0.0], [1.0f32, 0.0]);
    for (f, &n) in per.iter().enumerate() {
        let left = [-d[1], d[0]];
        for _ in 0..n {
            let e = add(c, a, d);
            pieces.push((vec![add(c, -w / 2.0, left), add(e, -w / 2.0, left), add(e, w / 2.0, left), add(c, w / 2.0, left)], level, false));
            c = e;
            level += 1;
        }
        walk.push(c);
        if f + 1 == flights {
            break;
        }
        let far = add(c, w, d);
        if s.kind == StairKind::HalfLanding {
            let reach = w * 1.5 + WELL;
            pieces.push((vec![add(c, -w / 2.0, left), add(far, -w / 2.0, left), add(far, reach, left), add(c, reach, left)], level, true));
            walk.push(add(add(c, w / 2.0, d), (w + WELL) / 2.0, left));
            c = add(c, w + WELL, left);
            d = [-d[0], -d[1]];
        } else {
            pieces.push((vec![add(c, -w / 2.0, left), add(far, -w / 2.0, left), add(far, w / 2.0, left), add(c, w / 2.0, left)], level, true));
            c = add(add(c, w / 2.0, d), w / 2.0, left);
            walk.push(c);
            d = left;
        }
        level += 1;
    }
    Some(place(s, pieces, walk, risers, rise, from, to))
}

/// A point of the stair's local coordinates in the building's floor plan.
fn to_plan(s: &Stair, q: [f32; 2]) -> [f32; 2] {
    let v = if s.turn == Turn::Right { -q[1] } else { q[1] };
    let (sin, cos) = s.angle.to_radians().sin_cos();
    // Counter-clockwise seen from above: the left of +x is towards -z.
    let (x, y) = (q[0] * cos - v * sin, q[0] * sin + v * cos);
    [s.x + x, s.z - y]
}

fn place(s: &Stair, pieces: Vec<(Vec<[f32; 2]>, usize, bool)>, walk: Vec<[f32; 2]>, risers: usize, rise: f32, from: f32, to: f32) -> StairGeometry {
    let pieces = pieces
        .into_iter()
        .map(|(o, k, landing)| {
            let top = from + rise * k as f32;
            StairPiece { outline: o.into_iter().map(|q| to_plan(s, q)).collect(), bottom: if landing { top - LANDING_SLAB } else { top - rise }, top, landing }
        })
        .collect();
    StairGeometry { pieces, walk: walk.into_iter().map(|q| to_plan(s, q)).collect(), risers, rise, from, to }
}

/// Lowest and highest corner of what a stair covers in the floor plan.
pub fn plan_bounds(g: &StairGeometry) -> ([f32; 2], [f32; 2]) {
    let pts = g.pieces.iter().flat_map(|p| p.outline.iter());
    pts.fold(([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]), |(lo, hi), q| ([lo[0].min(q[0]), lo[1].min(q[1])], [hi[0].max(q[0]), hi[1].max(q[1])]))
}

/// A new stair of building `bi` from level `bottom` up to level `top` (the next level
/// above `bottom` without one) with its foot at `x`, `z`; its id.
pub fn add_stair(prj: &mut Project, bi: usize, kind: StairKind, x: f32, z: f32, bottom: u64, top: Option<u64>) -> Result<u64, String> {
    let b = prj.layout.buildings.get(bi).ok_or_else(|| tr("No such building").to_string())?;
    let lo = b.levels.iter().position(|l| l.id == bottom).ok_or_else(|| tr("No such level").to_string())?;
    let top = match top {
        Some(t) => t,
        None => b.levels.get(lo + 1).map(|l| l.id).ok_or_else(|| tr("A stair needs a level above the one it starts on").to_string())?,
    };
    let n = b.stairs.len() + 1;
    let name = (n..).map(|k| format!("ST{k}")).find(|c| !b.stairs.iter().any(|s| &s.name == c)).unwrap_or_default();
    let id = prj.new_id().max(1);
    let s = Stair { id, name, kind, x, z, bottom, top, ..Default::default() };
    let s = checked_stair(&prj.layout.buildings[bi], s)?;
    prj.layout.buildings[bi].stairs.push(s);
    Ok(id)
}

/// A stair whose levels exist and whose head is above its foot, its sizes in range.
pub fn checked_stair(b: &Building, mut s: Stair) -> Result<Stair, String> {
    let (from, to) = floors(b, &s).ok_or_else(|| tr("No such level").to_string())?;
    if to <= from {
        return Err(tr("A stair leads up: its upper level must be above the one it starts on").to_string());
    }
    s.width = s.width.clamp(500.0, 5000.0);
    s.tread = s.tread.clamp(150.0, 500.0);
    if s.name.trim().is_empty() {
        s.name = format!("ST{}", b.stairs.len() + 1);
    }
    Ok(s)
}

/// Stairs whose levels are missing, whose steps are too high or too low and whose
/// steps break the step rule.
pub fn problems(prj: &Project) -> Vec<Issue> {
    let mut out = Vec::new();
    for b in &prj.layout.buildings {
        for s in &b.stairs {
            let Some(g) = geometry(b, s) else {
                out.push(crate::issue!(Severity::Error, None, "Stair {} of {}: its levels are missing or its upper level is not above the lower one", s.name, b.name));
                continue;
            };
            if g.rise < RISE_RANGE.0 || g.rise > RISE_RANGE.1 {
                out.push(crate::issue!(Severity::Warning, None, "Stair {} of {}: rise of {} mm per step, aim for 160 - 190 mm", s.name, b.name, format!("{:.0}", g.rise)));
            }
            let rule = 2.0 * g.rise + s.tread;
            if s.kind != StairKind::Spiral && !(STEP_RULE.0..=STEP_RULE.1).contains(&rule) {
                out.push(crate::issue!(Severity::Warning, None, "Stair {} of {}: twice the rise and the going make {} mm, the step rule asks for 590 - 650 mm", s.name, b.name, format!("{:.0}", rule)));
            }
        }
    }
    out
}

/// The stairs of the building room `ri` stands in that reach the room's level, in the
/// building's floor plan: (building, stair index, its geometry).
pub fn stairs_at_level(prj: &Project, level: u64) -> Vec<(usize, usize, StairGeometry)> {
    let Some((bi, li)) = find_level(&prj.layout, level) else { return Vec::new() };
    let b = &prj.layout.buildings[bi];
    let elevation = b.levels[li].elevation;
    b.stairs
        .iter()
        .enumerate()
        .filter_map(|(k, s)| geometry(b, s).filter(|g| g.from <= elevation + 0.5 && g.to >= elevation - 0.5).map(|g| (bi, k, g)))
        .collect()
}

#[cfg(test)]
mod tests;
