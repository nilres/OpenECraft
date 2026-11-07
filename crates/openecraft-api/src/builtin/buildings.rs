//! Building commands: buildings of the plant with their levels and shafts, rooms on the
//! levels, and links (cable trenches, conduits, cable bridges) between the building
//! entries of different buildings ([`openecraft_core::panel::buildings`]).

use crate::builtin::common::{int_in, num_in, NO_LIMIT, NO_MAX};
use crate::params::{optional, param, Args, Kind};
use crate::{Info, Plugin, Registry};
use super::{add_commands, info, Spec};
use openecraft_core::model::Project;
use openecraft_core::panel::buildings::{self as bld, LEVEL_HEIGHT, SHAFT_SIZE};
use openecraft_core::panel::{self, LinkKind, Opening, SitePlace};
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde_json::{json, Value};

pub struct Buildings;

impl Plugin for Buildings {
    fn info(&self) -> Info {
        info("buildings", tr_const!("Buildings"), tr_const!("Buildings of the plant with levels and shafts, rooms on the levels and links between buildings"))
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        add_commands(reg, COMMANDS)
    }
}

const BUILDING: &str = tr_const!("Building (location, e.g. +B1)");
const LEVEL: &str = tr_const!("Level: its name or number (1 = the lowest)");
const SHAFT: &str = tr_const!("Shaft: its name");
const LINK: &str = tr_const!("Link between buildings: its name");
const KIND: &str = tr_const!("trench, conduit or bridge");

pub static COMMANDS: &[Spec] = &[
    Spec {
        name: "list_buildings",
        help: tr_const!("Lists the buildings with their levels, the rooms on them, their shafts and the rooms these join, and the links between buildings"),
        params: &[optional("building", Kind::Str, BUILDING)],
        run: list_buildings,
    },
    Spec {
        name: "add_building",
        help: tr_const!("Adds a building with one level at ground level; without x and z it goes beside what stands in the plant already"),
        params: &[
            param("name", Kind::Str, BUILDING),
            optional("x", Kind::Num, tr_const!("Origin of the building in the plant, x in mm")),
            optional("z", Kind::Num, tr_const!("Origin of the building in the plant, z (towards the front) in mm")),
            optional("angle", Kind::Num, tr_const!("Turn about the vertical in degrees, counter-clockwise seen from above")),
            optional("description", Kind::Str, tr_const!("Description")),
        ],
        run: add_building,
    },
    Spec {
        name: "set_building",
        help: tr_const!("Moves, turns, renames or removes a building; its rooms move with it, removed they stay where they stand"),
        params: &[
            param("building", Kind::Str, BUILDING),
            optional("name", Kind::Str, tr_const!("New name")),
            optional("x", Kind::Num, tr_const!("Origin of the building in the plant, x in mm")),
            optional("z", Kind::Num, tr_const!("Origin of the building in the plant, z (towards the front) in mm")),
            optional("angle", Kind::Num, tr_const!("Turn about the vertical in degrees, counter-clockwise seen from above")),
            optional("description", Kind::Str, tr_const!("Description")),
            optional("remove", Kind::Bool, tr_const!("Remove the building")),
        ],
        run: set_building,
    },
    Spec {
        name: "add_level",
        help: tr_const!("Adds a level to a building: by default on top of the highest one, 3000 mm high; a negative elevation makes a basement"),
        params: &[
            param("building", Kind::Str, BUILDING),
            optional("name", Kind::Str, tr_const!("Name (default L0, L1, … above and B1, B2, … below the ground)")),
            optional("elevation", Kind::Num, tr_const!("Height of its floor above the ground level of the building in mm")),
            optional("height", Kind::Num, tr_const!("Height from its floor to the floor of the level above in mm")),
        ],
        run: add_level,
    },
    Spec {
        name: "set_level",
        help: tr_const!("Renames a level, changes its elevation or height (the rooms on it move with it) or removes it (its rooms stay where they stand)"),
        params: &[
            param("building", Kind::Str, BUILDING),
            param("level", Kind::Str, LEVEL),
            optional("name", Kind::Str, tr_const!("New name")),
            optional("elevation", Kind::Num, tr_const!("Height of its floor above the ground level of the building in mm")),
            optional("height", Kind::Num, tr_const!("Height from its floor to the floor of the level above in mm")),
            optional("remove", Kind::Bool, tr_const!("Remove the level")),
        ],
        run: set_level,
    },
    Spec {
        name: "place_on_level",
        help: tr_const!("Puts a room on a level of a building at a place of the building's floor plan (rear left corner of the room's outline); without x and z next to the other rooms of the level; without building out of its building"),
        params: &[
            param("room", Kind::Str, tr_const!("Room (location, e.g. +ROOM1)")),
            optional("building", Kind::Str, tr_const!("Building; empty to take the room out of its building")),
            optional("level", Kind::Str, LEVEL),
            optional("x", Kind::Num, tr_const!("x in the floor plan of the building in mm")),
            optional("z", Kind::Num, tr_const!("z in the floor plan of the building (towards the front) in mm")),
            optional("angle", Kind::Num, tr_const!("Turn about the vertical in degrees, counter-clockwise seen from above")),
        ],
        run: place_on_level,
    },
    Spec {
        name: "add_shaft",
        help: tr_const!("Adds a shaft to a building from one level up to another: cables, tubes and pipes run through it into the rooms above and below that it passes"),
        params: &[
            param("building", Kind::Str, BUILDING),
            param("x", Kind::Num, tr_const!("Rear left corner in the floor plan of the building, x in mm")),
            param("z", Kind::Num, tr_const!("Rear left corner in the floor plan of the building, z (towards the front) in mm")),
            optional("width", Kind::Num, tr_const!("Width in mm (default 600)")),
            optional("depth", Kind::Num, tr_const!("Depth in mm (default 600)")),
            optional("bottom", Kind::Str, tr_const!("Lowest level (default the lowest of the building)")),
            optional("top", Kind::Str, tr_const!("Highest level (default the highest of the building)")),
            optional("name", Kind::Str, tr_const!("Name (default S1, S2, …)")),
        ],
        run: add_shaft,
    },
    Spec {
        name: "set_shaft",
        help: tr_const!("Moves or resizes a shaft, changes the levels it runs through, renames or removes it"),
        params: &[
            param("building", Kind::Str, BUILDING),
            param("shaft", Kind::Str, SHAFT),
            optional("x", Kind::Num, tr_const!("Rear left corner in the floor plan of the building, x in mm")),
            optional("z", Kind::Num, tr_const!("Rear left corner in the floor plan of the building, z (towards the front) in mm")),
            optional("width", Kind::Num, tr_const!("Width in mm")),
            optional("depth", Kind::Num, tr_const!("Depth in mm")),
            optional("bottom", Kind::Str, tr_const!("Lowest level")),
            optional("top", Kind::Str, tr_const!("Highest level")),
            optional("name", Kind::Str, tr_const!("New name")),
            optional("remove", Kind::Bool, tr_const!("Remove the shaft")),
        ],
        run: set_shaft,
    },
    Spec {
        name: "add_site_link",
        help: tr_const!("Links a building entry of one room to a building entry of a room in another building: a cable trench, a conduit or a cable bridge, along which cables, tubes and pipes are routed from building to building"),
        params: &[
            param("from", Kind::Str, tr_const!("Room with the first building entry")),
            param("to", Kind::Str, tr_const!("Room with the second building entry")),
            optional("kind", Kind::Str, KIND),
            optional("from_entry", Kind::Int, tr_const!("Id of the building entry in the first room (default its first one)")),
            optional("to_entry", Kind::Int, tr_const!("Id of the building entry in the second room (default its first one)")),
            optional("points", Kind::Points, tr_const!("Corners of the way in the plant (x, z in mm)")),
            optional("length", Kind::Num, tr_const!("Length in mm (default along the way)")),
            optional("name", Kind::Str, tr_const!("Name (default SL1, SL2, …)")),
        ],
        run: add_site_link,
    },
    Spec {
        name: "set_site_link",
        help: tr_const!("Changes the kind, the corners or the length of a link between buildings, renames or removes it"),
        params: &[
            param("link", Kind::Str, LINK),
            optional("kind", Kind::Str, KIND),
            optional("points", Kind::Points, tr_const!("Corners of the way in the plant (x, z in mm)")),
            optional("length", Kind::Num, tr_const!("Length in mm; 0 along the way")),
            optional("name", Kind::Str, tr_const!("New name")),
            optional("remove", Kind::Bool, tr_const!("Remove the link")),
        ],
        run: set_site_link,
    },
];

fn building_arg(prj: &Project, a: &Args, name: &str) -> Result<usize, String> {
    let key = a.req(name);
    bld::building_index(&prj.layout, key).ok_or_else(|| tr_fmt!("No building named {}", key))
}

fn level_arg(prj: &Project, bi: usize, key: &str) -> Result<u64, String> {
    let b = &prj.layout.buildings[bi];
    bld::level_index(b, key).map(|li| b.levels[li].id).ok_or_else(|| tr_fmt!("No level {} in {}", key, b.name))
}

fn room_arg(prj: &Project, name: &str) -> Result<usize, String> {
    let i = prj.layout.enclosures.iter().position(|e| e.name == name.trim()).ok_or_else(|| tr_fmt!("No enclosure named {}", name))?;
    if !prj.layout.enclosures[i].is_room() {
        return Err(tr_fmt!("{} is not a room", name));
    }
    Ok(i)
}

fn kind_arg(a: &Args) -> Result<Option<LinkKind>, String> {
    a.str("kind").map(|k| LinkKind::from_key(k).ok_or_else(|| tr_fmt!("Unknown kind: {}", k))).transpose()
}

fn level_name(prj: &Project, id: u64) -> String {
    bld::level_by_id(&prj.layout, id).map(|l| l.name.clone()).unwrap_or_default()
}

fn building_json(prj: &Project, bi: usize) -> Value {
    let layout = &prj.layout;
    let b = &layout.buildings[bi];
    let poses = panel::site_poses(prj);
    let name = |ri: usize| layout.enclosures[ri].name.clone();
    let levels: Vec<Value> = b
        .levels
        .iter()
        .map(|l| {
            let rooms: Vec<Value> = bld::rooms_on(layout, l.id)
                .into_iter()
                .map(|ri| {
                    let e = &layout.enclosures[ri];
                    let p = e.site.unwrap_or_default();
                    json!({ "room": e.name, "x": p.x, "z": p.z, "angle": p.angle, "height": e.height, "placed": e.site.is_some() })
                })
                .collect();
            json!({ "name": l.name, "elevation": l.elevation, "height": l.height, "rooms": rooms })
        })
        .collect();
    let shafts: Vec<Value> = b
        .shafts
        .iter()
        .map(|s| {
            let rooms: Vec<String> = bld::shaft_rooms(prj, &poses, bi, s).into_iter().map(name).collect();
            json!({ "name": s.name, "x": s.x, "z": s.z, "width": s.width, "depth": s.depth, "bottom": level_name(prj, s.bottom), "top": level_name(prj, s.top), "rooms": rooms })
        })
        .collect();
    json!({
        "name": b.name,
        "description": b.description,
        "x": b.site.x,
        "z": b.site.z,
        "angle": b.site.angle,
        "levels": levels,
        "shafts": shafts,
    })
}

fn link_json(prj: &Project, link: &openecraft_core::panel::SiteLink) -> Value {
    let layout = &prj.layout;
    let poses = panel::site_poses(prj);
    let end = |id: u64| bld::entry_room(layout, id).map(|(ri, _)| layout.enclosures[ri].name.clone());
    json!({
        "name": link.name,
        "kind": link.kind.key(),
        "from": end(link.a),
        "from_entry": link.a,
        "to": end(link.b),
        "to_entry": link.b,
        "points": link.points,
        "length": bld::site_link_length(prj, &poses, link),
    })
}

fn list_buildings(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let list: Vec<usize> = match a.str("building") {
        Some(_) => vec![building_arg(prj, a, "building")?],
        None => (0..prj.layout.buildings.len()).collect(),
    };
    let links: Vec<Value> = prj.layout.site_links.iter().map(|l| link_json(prj, l)).collect();
    let problems: Vec<String> = bld::problems(prj).iter().map(|i| i.message()).collect();
    Ok(json!({ "buildings": list.into_iter().map(|bi| building_json(prj, bi)).collect::<Vec<_>>(), "links": links, "problems": problems }))
}

fn place_of(a: &Args, old: SitePlace) -> SitePlace {
    SitePlace { x: a.num("x").unwrap_or(old.x), z: a.num("z").unwrap_or(old.z), angle: a.num("angle").unwrap_or(old.angle) }
}

fn add_building(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let bi = bld::add_building(prj, a.req("name"))?;
    let b = &mut prj.layout.buildings[bi];
    b.site = place_of(a, b.site);
    if let Some(d) = a.str("description") {
        b.description = d.into();
    }
    Ok(building_json(prj, bi))
}

fn set_building(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let bi = building_arg(prj, a, "building")?;
    if a.flag("remove") {
        let name = prj.layout.buildings[bi].name.clone();
        bld::remove_building(prj, bi);
        return Ok(json!({ "removed": name }));
    }
    if let Some(n) = a.str("name").map(str::trim).filter(|n| !n.is_empty()) {
        if prj.layout.buildings.iter().enumerate().any(|(k, b)| k != bi && b.name == n) || prj.layout.enclosures.iter().any(|e| e.name == n) {
            return Err(tr_fmt!("{} exists already", n));
        }
        prj.layout.buildings[bi].name = n.into();
    }
    let b = &mut prj.layout.buildings[bi];
    b.site = place_of(a, b.site);
    if let Some(d) = a.str("description") {
        b.description = d.into();
    }
    Ok(building_json(prj, bi))
}

fn add_level(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let bi = building_arg(prj, a, "building")?;
    bld::add_level(prj, bi, a.str("name"), a.num("elevation"), a.num("height"))?;
    Ok(building_json(prj, bi))
}

fn set_level(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let bi = building_arg(prj, a, "building")?;
    let id = level_arg(prj, bi, a.req("level"))?;
    if a.flag("remove") {
        bld::remove_level(prj, bi, id)?;
    } else {
        bld::set_level(prj, bi, id, a.str("name"), a.num("elevation"), a.num("height"))?;
    }
    Ok(building_json(prj, bi))
}

fn place_on_level(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let ri = room_arg(prj, a.req("room"))?;
    let Some(bi) = a.str("building").filter(|b| !b.trim().is_empty()).map(|_| building_arg(prj, a, "building")).transpose()? else {
        bld::set_room_level(prj, ri, None, None)?;
        return Ok(json!({ "room": prj.layout.enclosures[ri].name, "building": Value::Null }));
    };
    let level = match a.str("level") {
        Some(l) => level_arg(prj, bi, l)?,
        None => prj.layout.buildings[bi].levels.iter().find(|l| l.elevation.abs() < 0.5).or(prj.layout.buildings[bi].levels.first()).map(|l| l.id).ok_or_else(|| tr("No such level").to_string())?,
    };
    let place = (a.num("x").is_some() || a.num("z").is_some() || a.num("angle").is_some()).then(|| place_of(a, SitePlace::default()));
    bld::set_room_level(prj, ri, Some(level), place)?;
    Ok(building_json(prj, bi))
}

/// The shaft of building `bi` named by `shaft`.
fn shaft_arg(prj: &Project, bi: usize, a: &Args) -> Result<usize, String> {
    let key = a.req("shaft").trim();
    let b = &prj.layout.buildings[bi];
    b.shafts.iter().position(|s| s.name.eq_ignore_ascii_case(key)).ok_or_else(|| tr_fmt!("No shaft {} in {}", key, b.name))
}

fn add_shaft(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let bi = building_arg(prj, a, "building")?;
    let b = &prj.layout.buildings[bi];
    let (first, last) = (b.levels.first().map_or(0, |l| l.id), b.levels.last().map_or(0, |l| l.id));
    let bottom = a.str("bottom").map(|l| level_arg(prj, bi, l)).transpose()?.unwrap_or(first);
    let top = a.str("top").map(|l| level_arg(prj, bi, l)).transpose()?.unwrap_or(last);
    let size = |n: &str| num_in(a, n, 100.0, NO_MAX).map(|v| v.unwrap_or(SHAFT_SIZE));
    let (x, z) = (a.num("x").unwrap_or_default(), a.num("z").unwrap_or_default());
    bld::add_shaft(prj, bi, a.str("name"), x, z, size("width")?, size("depth")?, bottom, top)?;
    Ok(building_json(prj, bi))
}

fn set_shaft(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let bi = building_arg(prj, a, "building")?;
    let si = shaft_arg(prj, bi, a)?;
    if a.flag("remove") {
        prj.layout.buildings[bi].shafts.remove(si);
        return Ok(building_json(prj, bi));
    }
    let mut s = prj.layout.buildings[bi].shafts[si].clone();
    s.x = a.num("x").unwrap_or(s.x);
    s.z = a.num("z").unwrap_or(s.z);
    s.width = a.num("width").unwrap_or(s.width);
    s.depth = a.num("depth").unwrap_or(s.depth);
    if let Some(l) = a.str("bottom") {
        s.bottom = level_arg(prj, bi, l)?;
    }
    if let Some(l) = a.str("top") {
        s.top = level_arg(prj, bi, l)?;
    }
    if let Some(n) = a.str("name").map(str::trim).filter(|n| !n.is_empty()) {
        s.name = n.into();
    }
    let s = bld::checked_shaft(&prj.layout.buildings[bi], s)?;
    prj.layout.buildings[bi].shafts[si] = s;
    Ok(building_json(prj, bi))
}

/// A building entry of room `ri`: the one with id `entry`, otherwise its first one.
fn entry_of(prj: &Project, ri: usize, entry: Option<i64>) -> Result<u64, String> {
    let e = &prj.layout.enclosures[ri];
    let mut entries = e.items.iter().filter(|i| i.opening == Opening::Entry);
    let found = match entry {
        Some(id) => entries.find(|i| i.id as i64 == id),
        None => entries.next(),
    };
    found.map(|i| i.id).ok_or_else(|| tr_fmt!("{} has no such building entry", e.name))
}

fn add_site_link(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let (ra, rb) = (room_arg(prj, a.req("from"))?, room_arg(prj, a.req("to"))?);
    let ea = entry_of(prj, ra, int_in(a, "from_entry", 1, NO_LIMIT)?)?;
    let eb = entry_of(prj, rb, int_in(a, "to_entry", 1, NO_LIMIT)?)?;
    let kind = kind_arg(a)?.unwrap_or_default();
    let id = bld::add_site_link(prj, ea, eb, kind, a.points("points"), a.str("name"))?;
    if let Some(l) = num_in(a, "length", 0.0, NO_MAX)? {
        if let Some(link) = prj.layout.site_links.iter_mut().find(|k| k.id == id) {
            link.length = l;
        }
    }
    let link = prj.layout.site_links.iter().find(|k| k.id == id).cloned().unwrap_or_default();
    Ok(link_json(prj, &link))
}

fn set_site_link(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let key = a.req("link").trim().to_string();
    let k = prj.layout.site_links.iter().position(|l| l.name.eq_ignore_ascii_case(&key)).ok_or_else(|| tr_fmt!("No link named {}", key))?;
    if a.flag("remove") {
        let l = prj.layout.site_links.remove(k);
        return Ok(json!({ "removed": l.name }));
    }
    let kind = kind_arg(a)?;
    let length = num_in(a, "length", 0.0, NO_MAX)?;
    let link = &mut prj.layout.site_links[k];
    if let Some(kind) = kind {
        link.kind = kind;
    }
    if a.value("points").is_some() {
        link.points = a.points("points");
    }
    if let Some(l) = length {
        link.length = l;
    }
    if let Some(n) = a.str("name").map(str::trim).filter(|n| !n.is_empty()) {
        link.name = n.into();
    }
    let link = prj.layout.site_links[k].clone();
    Ok(link_json(prj, &link))
}

/// Default height of a new room on a level: the level's height less its ceiling slab.
pub fn room_height_on(prj: &Project, level: u64) -> f32 {
    bld::level_by_id(&prj.layout, level).map_or(LEVEL_HEIGHT, |l| l.height) - bld::SLAB
}

#[cfg(test)]
mod tests;
