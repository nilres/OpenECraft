//! Copy and paste of panel items: rails, ducts, busbars, cutouts and accessories
//! copied with the items mounted on them, relative to their common top left corner,
//! and pasted onto any face of any enclosure. Devices are left out (a device is placed
//! once); assemblies and mountings among the copied items are kept.

use super::{mounted_items, Enclosure, Face, ItemGroup, ItemKind, PanelItem};
use crate::geom::P;
use crate::model::Project;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Marks the text of a clip on the system clipboard.
const CLIP_MAGIC: &str = "openecraft-panel-clip";

/// Copied panel items: ids and groups numbered from 1 within the clip, positions
/// relative to the top left corner of all of them.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelClip {
    magic: String,
    /// Where the copied items were: their top left corner on their face (mm).
    pub x: f32,
    pub y: f32,
    /// Width and height of the copied items together (mm).
    pub w: f32,
    pub h: f32,
    pub items: Vec<PanelItem>,
    pub groups: Vec<ItemGroup>,
    /// Devices of the selection that were left out.
    pub devices: usize,
}

impl PanelClip {
    pub fn to_text(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_text(s: &str) -> Option<PanelClip> {
        let c: PanelClip = serde_json::from_str(s).ok()?;
        (c.magic == CLIP_MAGIC).then_some(c)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Copies items of an enclosure with the items mounted on them: those on the face of
/// the first one, without the devices.
pub fn copy_items(e: &Enclosure, ids: &[u64]) -> PanelClip {
    let devices = e.items.iter().filter(|i| ids.contains(&i.id) && i.kind == ItemKind::Device).count();
    let face = e.items.iter().find(|i| ids.contains(&i.id) && i.kind != ItemKind::Device).map(|i| i.face);
    let all = mounted_items(e, ids);
    let items: Vec<&PanelItem> = e.items.iter().filter(|i| all.contains(&i.id) && i.kind != ItemKind::Device && Some(i.face) == face).collect();
    let mut clip = PanelClip { magic: CLIP_MAGIC.into(), devices, ..Default::default() };
    if items.is_empty() {
        return clip;
    }
    let min_x = items.iter().map(|i| i.x).fold(f32::INFINITY, f32::min);
    let min_y = items.iter().map(|i| i.y).fold(f32::INFINITY, f32::min);
    (clip.x, clip.y) = (min_x, min_y);
    clip.w = items.iter().map(|i| i.x + i.w).fold(f32::NEG_INFINITY, f32::max) - min_x;
    clip.h = items.iter().map(|i| i.y + i.h).fold(f32::NEG_INFINITY, f32::max) - min_y;
    let local: HashMap<u64, u64> = items.iter().zip(1..).map(|(i, n)| (i.id, n)).collect();
    let mut groups: HashMap<u64, u64> = HashMap::new();
    for i in items {
        let group = match i.group {
            0 => 0,
            g => *groups.entry(g).or_insert_with(|| {
                let id = clip.groups.len() as u64 + 1;
                let name = e.groups.iter().find(|x| x.id == g).map(|x| x.name.clone()).unwrap_or_default();
                clip.groups.push(ItemGroup { id, name, source: String::new() });
                id
            }),
        };
        let mut it = PanelItem { id: local[&i.id], x: i.x - min_x, y: i.y - min_y, anchor: Default::default(), group, ..i.clone() };
        if it.mounted_on != 0 {
            it.mounted_on = local.get(&it.mounted_on).copied().unwrap_or(0);
            it.mounting_explicit = true;
        }
        clip.items.push(it);
    }
    clip
}

/// Pastes a clip into an enclosure with its top left corner at `at` on `face`: new
/// ids, new assemblies. Returns the ids of the new items.
pub fn paste_items(prj: &mut Project, enc: usize, clip: &PanelClip, face: Face, at: P) -> Vec<u64> {
    if enc >= prj.layout.enclosures.len() {
        return Vec::new();
    }
    let mapping: HashMap<u64, u64> = clip.items.iter().map(|i| (i.id, prj.new_id())).collect();
    let groups: HashMap<u64, u64> = clip.groups.iter().map(|g| (g.id, prj.new_id())).collect();
    let e = &mut prj.layout.enclosures[enc];
    for g in &clip.groups {
        e.groups.push(ItemGroup { id: groups[&g.id], ..g.clone() });
    }
    let mut ids = Vec::new();
    for src in &clip.items {
        let id = mapping[&src.id];
        let mut it = PanelItem { id, x: src.x + at.x, y: src.y + at.y, face, group: groups.get(&src.group).copied().unwrap_or(0), ..src.clone() };
        if it.mounted_on != 0 {
            it.mounted_on = mapping.get(&it.mounted_on).copied().unwrap_or(0);
        }
        e.items.push(it);
        ids.push(id);
    }
    ids
}

#[cfg(test)]
mod tests;
