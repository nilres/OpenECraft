//! Floor plans on paper under the levels of a building ([`FloorPlan`]): a scan or a
//! picture of the drawing, put to scale and in place in the building's floor plan, to
//! draw the rooms on or to find them in ([`crate::wallscan`]). [`trace`] makes rooms of
//! what was found on the level, with their doors and windows.
//!
//! A pixel of the picture lies in the building's floor plan at its top left corner
//! (`x`, `z`) plus its place in the picture (x to the right, y down as z) times
//! [`FloorPlan::mm_per_px`], turned by `angle` like a room.

use super::buildings::{find_level, room_height_on, set_room_level};
use super::rooms::{add_opening, new_room, set_opening, set_outline, walls, DOOR_SIZE, WINDOW_SIZE};
use super::{bounds, free_name, Face, Layout, Level, Opening, SitePlace, Swing};
use crate::model::document::Document;
use crate::model::panel::FloorPlan;
use crate::model::Project;
use crate::raster::Gray;
use crate::wallscan::{self, GapKind, Scan, Settings};
use openecraft_i18n::{tr, tr_fmt};

/// Scale of a new floor plan: 1:100.
pub const DEFAULT_SCALE: f32 = 100.0;
/// How strongly a new floor plan is shown.
pub const DEFAULT_OPACITY: f32 = 0.5;
/// Resolution of a picture that tells none, dots per inch.
pub const DEFAULT_DPI: f32 = 150.0;
/// Longest side of a picture kept in the project, pixels (larger ones are shrunk).
pub const MAX_SIDE: usize = 7000;

/// Size of a pixel on paper scanned at `dpi`, mm.
pub fn paper_of_dpi(dpi: f32) -> f32 {
    25.4 / dpi.max(1.0)
}

/// A floor plan of the picture `img` (named `name`), each pixel `paper` mm on the
/// paper, at 1:100 at the origin of the building; pictures larger than [`MAX_SIDE`]
/// are shrunk.
pub fn new_plan(name: &str, img: &Gray, paper: f32) -> FloorPlan {
    let f = img.w.max(img.h).div_ceil(MAX_SIDE).max(1);
    let img = img.shrunk(f);
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s);
    FloorPlan {
        image: Document::new(&format!("{stem}.png"), &img.to_png()),
        px: [img.w as u32, img.h as u32],
        paper: paper * f as f32,
        scale: DEFAULT_SCALE,
        opacity: DEFAULT_OPACITY,
        ..Default::default()
    }
}

/// The picture of a floor plan in grey.
pub fn image(plan: &FloorPlan) -> Result<Gray, String> {
    Gray::from_png(&plan.image.bytes()?)
}

/// The floor plan under level `id`.
pub fn plan_of(layout: &Layout, id: u64) -> Option<&FloorPlan> {
    let (bi, li) = find_level(layout, id)?;
    layout.buildings[bi].levels[li].plan.as_ref()
}

fn level_mut(prj: &mut Project, id: u64) -> Result<&mut Level, String> {
    let (bi, li) = find_level(&prj.layout, id).ok_or_else(|| tr("No such level").to_string())?;
    Ok(&mut prj.layout.buildings[bi].levels[li])
}

/// Puts a floor plan under level `id` (`None`: removes it). Checks its scale and
/// place.
pub fn set_plan(prj: &mut Project, id: u64, plan: Option<FloorPlan>) -> Result<(), String> {
    if let Some(p) = &plan {
        check(p)?;
    }
    level_mut(prj, id)?.plan = plan;
    Ok(())
}

/// Changes the floor plan under level `id`.
pub fn edit_plan(prj: &mut Project, id: u64, change: impl FnOnce(&mut FloorPlan)) -> Result<(), String> {
    let level = level_mut(prj, id)?;
    let mut p = level.plan.clone().ok_or_else(|| tr_fmt!("Level {} has no floor plan", level.name))?;
    change(&mut p);
    check(&p)?;
    level.plan = Some(p);
    Ok(())
}

fn check(p: &FloorPlan) -> Result<(), String> {
    if !(p.scale.is_finite() && p.scale > 0.0 && p.paper.is_finite() && p.paper > 0.0) {
        return Err(tr("The scale of the floor plan must be greater than 0").to_string());
    }
    if ![p.x, p.z, p.angle, p.opacity].iter().all(|v| v.is_finite()) {
        return Err(tr("Not a number").to_string());
    }
    if let Some([l, t, r, b]) = p.area {
        if l >= r || t >= b {
            return Err(tr("The area of the floor plan is empty").to_string());
        }
    }
    Ok(())
}

/// The point of the building's floor plan (x, z, mm) at pixel `q` of the picture.
pub fn to_building(plan: &FloorPlan, q: [f32; 2]) -> [f32; 2] {
    let m = plan.mm_per_px();
    let (x, z) = (q[0] * m, q[1] * m);
    let (s, c) = plan.angle.to_radians().sin_cos();
    [plan.x + x * c + z * s, plan.z - x * s + z * c]
}

/// The pixel of the picture at point `b` of the building's floor plan.
pub fn to_pixel(plan: &FloorPlan, b: [f32; 2]) -> [f32; 2] {
    let m = plan.mm_per_px().max(1e-6);
    let (dx, dz) = (b[0] - plan.x, b[1] - plan.z);
    let (s, c) = plan.angle.to_radians().sin_cos();
    [(dx * c - dz * s) / m, (dx * s + dz * c) / m]
}

/// The corners of the picture in the building's floor plan (top left, top right,
/// bottom right, bottom left).
pub fn corners(plan: &FloorPlan) -> [[f32; 2]; 4] {
    let [w, h] = plan.px.map(|v| v as f32);
    [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]].map(|q| to_building(plan, q))
}

/// The scale (1 : scale) at which the pixels `a` and `b` of the picture are `mm` apart
/// in the building.
pub fn calibrated(plan: &FloorPlan, a: [f32; 2], b: [f32; 2], mm: f32) -> Result<f32, String> {
    let px = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
    if px < 1.0 || !(mm.is_finite() && mm > 0.0) {
        return Err(tr("Pick two points apart and give their distance").to_string());
    }
    Ok(mm / (px * plan.paper))
}

/// The smallest room the detection keeps when the floor plan does not say, m².
pub const MIN_ROOM: f32 = 1.0;

/// What the detection takes for walls and rooms in this floor plan (its smallest room).
pub fn settings(plan: &FloorPlan) -> Settings {
    let mut s = Settings::new(plan.mm_per_px());
    s.min_room = plan.min_room.filter(|a| a.is_finite() && *a > 0.0).unwrap_or(MIN_ROOM) * 1.0e6;
    s
}

/// Finds the walls, doors, windows and rooms in the floor plan (in its area; pixels of
/// the whole picture).
pub fn detect(plan: &FloorPlan) -> Result<Scan, String> {
    let mut img = image(plan)?;
    if let Some([l, t, r, b]) = plan.area {
        // Outside the area only paper.
        let paper = img.paper();
        let (l, t, r, b) = (l as usize, t as usize, (r as usize).min(img.w), (b as usize).min(img.h));
        let (w, h) = (img.w, img.h);
        img.fill([0, 0], [w, t], paper);
        img.fill([0, b], [w, h], paper);
        img.fill([0, t], [l, b], paper);
        img.fill([r, t], [w, b], paper);
    }
    Ok(wallscan::scan(&img, &settings(plan)))
}

/// What [`trace`] made: the new rooms (indices) and how many doors and windows they
/// got; why rooms found could not be made.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Traced {
    pub rooms: Vec<usize>,
    pub doors: usize,
    pub windows: usize,
    pub skipped: Vec<String>,
}

/// Makes rooms on level `id` of what [`detect`] finds in its floor plan: their outlines
/// (the inside faces of the walls, mm rounded), as high as the level less its slab,
/// with the doors and windows. A door goes into the room it opens to (its swing
/// drawn) or the smaller of the two, a window into the room behind it.
pub fn trace(prj: &mut Project, id: u64) -> Result<Traced, String> {
    let plan = plan_of(&prj.layout, id).cloned().ok_or_else(|| tr("The level has no floor plan").to_string())?;
    let found = detect(&plan)?;
    trace_scan(prj, id, &plan, &found)
}

/// [`trace`] with what was found already.
pub fn trace_scan(prj: &mut Project, id: u64, plan: &FloorPlan, found: &Scan) -> Result<Traced, String> {
    if found.rooms.is_empty() {
        return Err(tr("No rooms found in the floor plan").to_string());
    }
    let m = plan.mm_per_px();
    let height = room_height_on(&prj.layout, id);
    let mut out = Traced::default();
    // Per room found: the new room and the pixel its outline is measured from.
    let mut made: Vec<Option<(usize, [f32; 2])>> = Vec::new();
    for r in &found.rooms {
        let (lo, _) = bounds(&r.outline);
        let pts: Vec<[f32; 2]> = r.outline.iter().map(|q| [((q[0] - lo[0]) * m).round(), ((q[1] - lo[1]) * m).round()]).collect();
        let name = free_name(&prj.layout, "+ROOM");
        let mut e = new_room(prj, &name, 1000.0, 1000.0, height);
        if let Err(why) = set_outline(&mut e, &pts) {
            out.skipped.push(why);
            made.push(None);
            continue;
        }
        prj.layout.enclosures.push(e);
        let ri = prj.layout.enclosures.len() - 1;
        let at = to_building(plan, lo);
        set_room_level(prj, ri, Some(id), Some(SitePlace { x: at[0], z: at[1], angle: plan.angle }))?;
        out.rooms.push(ri);
        made.push(Some((ri, lo)));
    }
    for g in &found.openings {
        let side = match g.swing {
            Some((side, _)) if g.kind == GapKind::Door => side,
            _ => match g.sides {
                [Some(a), Some(b)] => usize::from(found.rooms[b].area < found.rooms[a].area),
                [Some(_), None] => 0,
                _ => 1,
            },
        };
        let Some((ri, lo)) = g.sides[side].and_then(|k| made[k]) else { continue };
        let (axis, cross) = if g.along_x { (0, 1) } else { (1, 0) };
        // The face of the wall on the room's side and the ends of the gap, in the room.
        let face = (if side == 0 { g.lo[cross] } else { g.hi[cross] } - lo[cross]) * m;
        let (a, b) = ((g.lo[axis] - lo[axis]) * m, (g.hi[axis] - lo[axis]) * m);
        let mid = (a + b) / 2.0;
        let e = &prj.layout.enclosures[ri];
        let wall = walls(e).into_iter().enumerate().filter(|(_, w)| (w.a[cross] - w.b[cross]).abs() < 0.5 && w.a[axis].min(w.b[axis]) <= mid && mid <= w.a[axis].max(w.b[axis])).min_by(|x, y| (x.1.a[cross] - face).abs().total_cmp(&(y.1.a[cross] - face).abs()));
        let Some((wi, w)) = wall else { continue };
        if (w.a[cross] - face).abs() > (g.size().1 * m).max(200.0) {
            continue;
        }
        // Along the wall from its left end seen from inside.
        let (start, right) = (w.start(), w.right());
        let along = |v: f32| {
            let mut q = [0.0; 2];
            q[axis] = v;
            q[cross] = w.a[cross];
            (q[0] - start[0]) * right[0] + (q[1] - start[1]) * right[1]
        };
        let (xa, xb) = (along(a), along(b));
        let width = (b - a).abs().round();
        let (opening, height) = if g.kind == GapKind::Door { (Opening::Door, DOOR_SIZE.1) } else { (Opening::Window, WINDOW_SIZE.1) };
        let Ok(oid) = add_opening(prj, ri, opening, Face::Wall(wi as u16), Some(xa.min(xb).max(0.0)), Some((width, height)), None) else { continue };
        if g.kind == GapKind::Door {
            out.doors += 1;
            if let Some((_, hinge_lo)) = g.swing {
                // The hinge at the left end seen from inside or at the right one.
                let hinge = if hinge_lo { xa } else { xb };
                let swing = if hinge <= xa.min(xb) + 0.5 { Swing::LeftIn } else { Swing::RightIn };
                let _ = set_opening(prj, ri, oid, Some(swing), None, None, None);
            }
        } else {
            out.windows += 1;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
