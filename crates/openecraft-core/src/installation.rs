//! The electrical installation of a building: socket outlets, switches, lights and
//! heating elements in the rooms ([`kind_of`]) and the final circuits that feed them,
//! all of it from the schematic.
//!
//! **Circuits** ([`circuits`]): each protective device of the mains up to 32 A with the
//! socket outlets, switches, lights and heating elements drawn behind it (through
//! switches, contacts and terminals), its connections and cables. A device of the
//! schematic is placed in a room of the floor plan ([`place_device`]) or taken from what
//! was placed there ([`link_device`]); a device placed in a room for a protective
//! device goes into the schematic ([`add_device`]): onto the installation page of its
//! circuit, a schematic page drawn for it, with a cable from the device before it (from
//! the distribution board for the first one). The list of the circuits is read only:
//! what it shows comes from the schematic.
//!
//! **Cable length** ([`Leg`]): each connection of the phase conductors of a circuit
//! between two devices, as a cable (one cable once) or a single wire, has the length of
//! its 3D route ([`crate::routing3d`]). Until it is routed it is estimated: the cable
//! leaves the distribution board at its top and runs to the device in the horizontal
//! installation zone under the ceiling (DIN 18015-3: [`ZONE`] below it), straight along
//! x and z of the plant (along the walls, not across rooms), up and down in the vertical
//! zones above and below the devices, up or down between levels; each device adds
//! [`BOX_RESERVE`] for its box.
//!
//! **Voltage drop** from the supply to the farthest device: ΔU = k · I · L · (ρ / A ·
//! cos φ + X' · sin φ) per cable with k = 2 (single-phase) or √3 (three-phase), ρ of
//! copper at its operating temperature ([`crate::electrical::Conditions`]), X' the
//! reactance per metre ([`crate::shortcircuit::CABLE_REACTANCE`]), cos φ
//! [`DEFAULT_COS_PHI`]. The circuit's cables carry the design current of its protective
//! device (load calculation, [`crate::loads`]) or its rated current; the feeder cables
//! from the supply to the distribution board ([`crate::shortcircuit::feeder_cables`])
//! the design current of the protective device upstream or its rated current. The sum
//! is checked against the limit of the rule sets ([`crate::rules::limits`]).
//!
//! **Protective devices** ([`breakers`]): those of the mains in the schematic
//! ([`crate::electrical::circuits`]); a spare one feeds nothing yet: no other device on
//! the nets behind it.

use crate::electrical;
use crate::geom::{p, P, GRID};
use crate::issue::{Issue, Severity};
use crate::model::{Page, PageKind, Project, SymbolInst, Wire};
use crate::netlist::{Connection, Netlist, PinRef};
use crate::panel::{self, Face, ItemKind, PanelItem, SitePose};
use crate::parts::Part;
use crate::shortcircuit::FeederCable;
use crate::symbols::SymbolRef;
use openecraft_i18n::{tr, tr_fmt};
use std::collections::{BTreeSet, HashMap};

/// Distance of the horizontal installation zone below the ceiling, mm (DIN 18015-3:
/// 150 … 450 mm).
pub const ZONE: f32 = 300.0;
/// Cable kept in the box of each item, m.
pub const BOX_RESERVE: f32 = 0.3;
/// Farthest a click may lie from a wall to put a wall item on it, mm.
pub const WALL_REACH: f32 = 800.0;
/// Type of the cables of new devices, with the cross-section in mm².
pub const CABLE_TYPE: &str = "NYM-J 3x";
/// Power factor of the loads for the voltage drop.
pub const DEFAULT_COS_PHI: f32 = 0.95;
/// The length of the 3D route of a cable (by its tag) or of a single wire (by `from|to`,
/// the designations of its connection points), mm, where it is routed (see
/// `crate::routing3d::Routing::routed_length`).
pub type Routes<'a> = Option<&'a dyn Fn(&str) -> Option<f32>>;

/// What an item of a room is for the installation: a device of the mains (socket
/// outlets, switches, lights, heating elements, junction boxes, device connection boxes
/// for fixed equipment) or one without a circuit (data and antenna outlets, smoke
/// alarms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Socket,
    Switch,
    Light,
    Heater,
    Junction,
    Connection,
    Data,
    Aerial,
    Smoke,
}

impl Kind {
    pub const ALL: [Kind; 9] = [Kind::Socket, Kind::Switch, Kind::Light, Kind::Heater, Kind::Junction, Kind::Connection, Kind::Data, Kind::Aerial, Kind::Smoke];

    pub fn title(self) -> &'static str {
        match self {
            Kind::Socket => tr("Socket outlet"),
            Kind::Switch => tr("Switch"),
            Kind::Light => tr("Luminaire"),
            Kind::Heater => tr("Heating element"),
            Kind::Junction => tr("Junction box"),
            Kind::Connection => tr("Device connection box"),
            Kind::Data => tr("Data outlet"),
            Kind::Aerial => tr("Antenna outlet"),
            Kind::Smoke => tr("Smoke alarm"),
        }
    }

    /// Key of the kind in the commands' answers.
    pub fn key(self) -> &'static str {
        match self {
            Kind::Socket => "socket",
            Kind::Switch => "switch",
            Kind::Light => "light",
            Kind::Heater => "heater",
            Kind::Junction => "junction",
            Kind::Connection => "connection",
            Kind::Data => "data",
            Kind::Aerial => "aerial",
            Kind::Smoke => "smoke",
        }
    }

    /// The generic part a device of the schematic without a part is placed with.
    pub fn default_part(self) -> &'static str {
        match self {
            Kind::Socket => "GEN.SOCKET-F-UP",
            Kind::Switch => "GEN.SW-ONOFF-UP",
            Kind::Light => "GEN.LAMP-CEILING-LED",
            Kind::Heater => "GEN.HEATER-PANEL-1000",
            Kind::Junction => "GEN.JBOX-UP",
            Kind::Connection => "GEN.CONN-BOX-UP",
            Kind::Data => "GEN.DATA-RJ45-2X-UP",
            Kind::Aerial => "GEN.TV-SAT-UP",
            Kind::Smoke => "GEN.SMOKE-ALARM",
        }
    }

    /// A device of the mains, in a circuit of the schematic (not a data or antenna
    /// outlet or a smoke alarm).
    pub fn mains(self) -> bool {
        !matches!(self, Kind::Data | Kind::Aerial | Kind::Smoke)
    }
}

/// What a part is for the installation: a part for the walls or the ceiling of a room
/// by its subgroup (socket outlets, switches, push buttons, dimmers, lights, room
/// heaters) or its functions; `None` for anything else (devices of cabinets: motor
/// protection switches, rail sockets, enclosure heaters).
pub fn kind_of(part: &Part) -> Option<Kind> {
    if !part.wall_mountable() && !on_ceiling(part) {
        return None;
    }
    let sub = part.subgroup.trim().to_lowercase();
    for (name, kind) in [
        ("junction box", Kind::Junction),
        ("device connection box", Kind::Connection),
        ("data outlet", Kind::Data),
        ("antenna outlet", Kind::Aerial),
        ("smoke alarm", Kind::Smoke),
    ] {
        if sub == name {
            return Some(kind);
        }
    }
    if sub.contains("socket outlet") {
        return Some(Kind::Socket);
    }
    if ["switch", "push button", "blind switch", "dimmer", "glass push button"].contains(&sub.as_str()) {
        return Some(Kind::Switch);
    }
    if ["luminaire", "light", "lamp"].iter().any(|k| sub.contains(k)) {
        return Some(Kind::Light);
    }
    if ["heater", "heating panel", "radiator"].iter().any(|k| sub.contains(k)) {
        return Some(Kind::Heater);
    }
    let has = |k: crate::parts::FnKind| part.functions.iter().any(|f| f.kind == k);
    if has(crate::parts::FnKind::SocketOutlet) {
        Some(Kind::Socket)
    } else {
        has(crate::parts::FnKind::Heater).then_some(Kind::Heater)
    }
}

/// The part goes on the ceiling (lights made for it).
pub fn on_ceiling(part: &Part) -> bool {
    part.tech_value("Ceiling mounting").is_some_and(|v| v.trim().eq_ignore_ascii_case("yes")) || part.mounting.to_lowercase().contains("ceiling")
}

/// An item of the installation where it is: its room (index), item, kind, place in
/// the plant (x, z, mm), height above the floor of its room and height of the floor
/// above the ground (mm), the room's height.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    pub room: usize,
    pub item: u64,
    pub part: String,
    /// Its device tag (empty: not in the schematic).
    pub tag: String,
    pub kind: Kind,
    pub at: [f32; 2],
    pub height: f32,
    pub floor: f32,
    pub ceiling: f32,
    pub level: u64,
}

/// Height of the floor of a room above the ground (the level it is on).
fn floor_of(prj: &Project, ri: usize) -> f32 {
    let e = &prj.layout.enclosures[ri];
    panel::buildings::level_by_id(&prj.layout, e.level).map_or(0.0, |l| l.elevation)
}

/// Where an item of room `ri` is: x, z of the plant and its height above the floor.
fn spot(prj: &Project, poses: &[SitePose], ri: usize, it: &PanelItem) -> Option<([f32; 2], f32)> {
    let e = &prj.layout.enclosures[ri];
    let (lo, hi) = panel::item_box(e, it);
    let c = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
    let q = poses.get(ri)?.apply([c[0], 0.0, c[2]]);
    // Room coordinates: y down from the ceiling.
    Some(([q[0], q[2]], (e.height - c[1]).clamp(0.0, e.height)))
}

/// Where an item of a room is (see [`Placed`]); `None` for items that are no part of
/// the installation.
pub fn placed(prj: &Project, poses: &[SitePose], ri: usize, it: &PanelItem) -> Option<Placed> {
    if !matches!(it.kind, ItemKind::Device | ItemKind::Accessory) {
        return None;
    }
    let kind = panel::part_by_number(prj, &it.part).and_then(kind_of)?;
    let e = &prj.layout.enclosures[ri];
    let (at, height) = spot(prj, poses, ri, it)?;
    Some(Placed { room: ri, item: it.id, part: it.part.clone(), tag: it.tag.clone(), kind, at, height, floor: floor_of(prj, ri), ceiling: e.height, level: e.level })
}

/// Every item of the installation in the rooms.
pub fn items(prj: &Project, poses: &[SitePose]) -> Vec<Placed> {
    panel::rooms(&prj.layout).into_iter().flat_map(|ri| prj.layout.enclosures[ri].items.iter().filter_map(move |it| placed(prj, poses, ri, it))).collect()
}

/// The parts of the bundled manufacturer catalogs that are parts of the installation
/// (read once).
pub fn bundled_parts() -> &'static [Part] {
    static PARTS: std::sync::OnceLock<Vec<Part>> = std::sync::OnceLock::new();
    PARTS.get_or_init(|| crate::parts::bundled::all().unwrap_or_default().into_iter().flat_map(|b| b.parts).filter(|p| kind_of(p).is_some()).collect())
}

/// Part `number` of the project, the generic catalog or a bundled manufacturer catalog.
pub fn find_part<'a>(prj: &'a Project, number: &str) -> Option<&'a Part> {
    panel::part_by_number(prj, number).or_else(|| bundled_parts().iter().find(|p| p.number == number))
}

/// Part `number` of the project or the generic catalog, or of a bundled manufacturer
/// catalog, which then goes into the project.
pub fn take_part(prj: &mut Project, number: &str) -> Option<Part> {
    if let Some(p) = panel::part_by_number(prj, number) {
        return Some(p.clone());
    }
    let p = bundled_parts().iter().find(|p| p.number == number)?.clone();
    prj.parts.push(p.clone());
    Some(p)
}

/// Puts a socket outlet, switch or light (part `part`) into room `ri` at point `at` of
/// the plant (x, z, mm): a light made for the ceiling on the ceiling there, anything
/// else on the wall nearest to the point (within [`WALL_REACH`]) at its mounting
/// height; a point outside the room on the outside of its nearest wall (devices outside
/// the building: an outdoor socket, a light on the facade). Gives the item's id.
pub fn place(prj: &mut Project, ri: usize, part: &str, at: [f32; 2]) -> Result<u64, String> {
    place_at(prj, ri, part, at, None)
}

/// [`place`] at mounting height `height` (mm above the floor; `None`: that of the part).
/// A junction box of a maker that is an enclosure ([`is_box_enclosure`]) hangs on the
/// wall as an enclosure of the room with a wire connector inside; the connector's id is
/// given then.
pub fn place_at(prj: &mut Project, ri: usize, part: &str, at: [f32; 2], height: Option<f32>) -> Result<u64, String> {
    let id = mount(prj, ri, part, at, height)?;
    if find_part(prj, part).is_some_and(is_box_enclosure) {
        return into_box(prj, ri, id);
    }
    Ok(id)
}

/// Whether part `p` is a junction box with room for wire connectors inside, placed as an
/// enclosure (part data `Junction box enclosure: yes`).
pub fn is_box_enclosure(p: &Part) -> bool {
    p.tech_value("Junction box enclosure").is_some_and(|v| v.trim() == "yes")
}

/// The wire connector put into a junction box placed as an enclosure.
pub const BOX_CONNECTOR: &str = "WAGO.221-415";

/// Part `number` of a bundled manufacturer catalog (any part, read once).
fn vendor_part(number: &str) -> Option<Part> {
    static PARTS: std::sync::OnceLock<Vec<Part>> = std::sync::OnceLock::new();
    PARTS.get_or_init(|| crate::parts::bundled::all().unwrap_or_default().into_iter().flat_map(|b| b.parts).collect()).iter().find(|p| p.number == number).cloned()
}

/// Turns item `id` of room `ri` (a junction box on a wall) into an enclosure of the room
/// hanging there, its front into the room, with a wire connector inside; gives the
/// connector's id.
fn into_box(prj: &mut Project, ri: usize, id: u64) -> Result<u64, String> {
    let e = prj.layout.enclosures[ri].clone();
    let it = e.item(id).cloned().ok_or_else(|| tr_fmt!("No enclosure {}", ri))?;
    let wall = panel::side(&e, it.face).ok_or_else(|| tr("No wall near the point").to_string())?;
    prj.layout.enclosures[ri].items.retain(|i| i.id != id);
    let (start, right, n) = (wall.start(), wall.right(), wall.normal);
    let along = it.x + it.w / 2.0;
    let centre = [start[0] + right[0] * along, start[1] + right[1] * along];
    // Its front faces the room: at angle 0 the front faces +z, at 90 +x; its x axis
    // runs to the right of one looking at its front.
    let angle = n[0].atan2(n[1]);
    let x_axis = [angle.cos(), -angle.sin()];
    let room_id = if e.id == 0 { let id = prj.new_id(); prj.layout.enclosures[ri].id = id; id } else { e.id };
    let name = panel::free_name(&prj.layout, "+JB");
    let mut b = panel::new_enclosure(prj, &it.part, &name);
    let rear_left = [centre[0] - x_axis[0] * b.width / 2.0, centre[1] - x_axis[1] * b.width / 2.0];
    let middle = e.height - (it.y + it.h / 2.0);
    b.room = room_id;
    b.site = Some(panel::SitePlace { x: rear_left[0], z: rear_left[1], angle: angle.to_degrees() });
    b.elevation = Some((middle - b.height / 2.0).max(0.0));
    prj.layout.enclosures.push(b);
    let k = prj.layout.enclosures.len() - 1;
    if !prj.parts.iter().any(|p| p.number == BOX_CONNECTOR) {
        if let Some(c) = vendor_part(BOX_CONNECTOR) {
            prj.parts.push(c);
        }
    }
    let (pw, ph) = prj.layout.enclosures[k].face_size(Face::Plate);
    let [cw, ch, _] = panel::part_by_number(prj, BOX_CONNECTOR).and_then(|p| p.size()).unwrap_or([30.0, 19.0, 9.0]);
    panel::place_accessory(prj, k, BOX_CONNECTOR, Face::Plate, p(((pw - cw) / 2.0).max(0.0), ((ph - ch) / 2.0).max(0.0))).ok_or_else(|| tr_fmt!("No enclosure {}", k))
}

/// The room, the item and the part of an item of the installation: an item of a room,
/// or the wire connector in a junction box hanging in a room (the box's part then).
fn installed(prj: &Project, item: u64) -> Option<(usize, PanelItem, String)> {
    let layout = &prj.layout;
    let k = layout.enclosures.iter().position(|e| e.item(item).is_some())?;
    let it = layout.enclosures[k].item(item)?.clone();
    if layout.enclosures[k].is_room() {
        let part = it.part.clone();
        return Some((k, it, part));
    }
    let e = &layout.enclosures[k];
    let ri = panel::room_of(layout, k)?;
    find_part(prj, &e.part).filter(|p| is_box_enclosure(p))?;
    Some((ri, it, e.part.clone()))
}

/// [`place_at`] for an item of the room itself.
fn mount(prj: &mut Project, ri: usize, part: &str, at: [f32; 2], height: Option<f32>) -> Result<u64, String> {
    let e = prj.layout.enclosures.get(ri).ok_or_else(|| tr_fmt!("No enclosure {}", ri))?;
    if !e.is_room() {
        return Err(tr_fmt!("{} is not a room", e.name));
    }
    let pd = take_part(prj, part).ok_or_else(|| tr_fmt!("Unknown device or part: {}", part))?;
    let e = &prj.layout.enclosures[ri];
    let poses = panel::site_poses(prj);
    let local = poses[ri].local([at[0], 0.0, at[1]]);
    let q = [local[0], local[2]];
    let [w, h, _] = pd.size().unwrap_or([80.0, 80.0, 20.0]);
    let outside = beyond_walls(e, q);
    let (face, point) = if on_ceiling(&pd) && !outside {
        let (lo, hi) = panel::bounds(&panel::outline(e));
        let x = (q[0] - lo[0] - w / 2.0).clamp(0.0, (hi[0] - lo[0] - w).max(0.0));
        let z = (q[1] - lo[1] - h / 2.0).clamp(0.0, (hi[1] - lo[1] - h).max(0.0));
        (Face::Roof, p(x, z))
    } else {
        // The nearest wall (or side of an inner wall) and the place along it.
        let faces: Vec<Face> = panel::room_faces(e).into_iter().filter(|f| if outside { matches!(f, Face::Outside(_)) } else { matches!(f, Face::Wall(_) | Face::Inner(..)) }).collect();
        let near = faces
            .into_iter()
            .filter_map(|f| {
                let wall = panel::side(e, f)?;
                let (start, right) = (wall.start(), wall.right());
                let along = (q[0] - start[0]) * right[0] + (q[1] - start[1]) * right[1];
                let off = (q[0] - start[0]) * wall.normal[0] + (q[1] - start[1]) * wall.normal[1];
                (along >= -50.0 && along <= wall.length() + 50.0 && off > -50.0).then_some((f, along.clamp(0.0, wall.length()), off.abs(), wall.length()))
            })
            .min_by(|a, b| a.2.total_cmp(&b.2));
        let Some((face, along, _, len)) = near.filter(|n| n.2 <= WALL_REACH) else {
            return Err(tr("No wall near the point").to_string());
        };
        (face, p((along - w / 2.0).clamp(0.0, (len - w).max(0.0)), 0.0))
    };
    let id = panel::mount_on_wall(prj, ri, part, face, point)?;
    if let Some(height) = height.or_else(|| panel::tech_number(&pd, "Mounting height")).filter(|_| face != Face::Roof) {
        let e = prj.layout.enclosures[ri].clone();
        if let Some(it) = prj.layout.enclosures[ri].items.iter_mut().find(|i| i.id == id) {
            panel::at_mounting_height(&e, it, height);
        }
    }
    Ok(id)
}

/// The room among `rooms` that point `at` of the plant (x, z, mm) lies in, its walls
/// included (a click on a wall in the 3D view); the first one that takes it.
pub fn room_at(prj: &Project, rooms: &[usize], at: [f32; 2]) -> Option<usize> {
    let poses = panel::site_poses(prj);
    rooms.iter().copied().filter(|&r| prj.layout.enclosures.get(r).is_some_and(|e| e.is_room())).find(|&r| {
        let q = poses[r].local([at[0], 0.0, at[1]]);
        !beyond_walls(&prj.layout.enclosures[r], [q[0], q[2]])
    })
}

/// Where [`place`] would put part `part` in room `ri` for point `at` of the plant,
/// without changing the project: the item it would add (id 0).
/// `height`: mm above the floor (`None`: the part's mounting height).
pub fn preview(prj: &Project, ri: usize, part: &str, at: [f32; 2], height: Option<f32>) -> Result<PanelItem, String> {
    let mut scratch = Project { layout: prj.layout.clone(), parts: prj.parts.clone(), next_id: prj.next_id, ..Default::default() };
    let id = mount(&mut scratch, ri, part, at, height)?;
    let it = scratch.layout.enclosures[ri].item(id).cloned().ok_or_else(|| tr_fmt!("No enclosure {}", ri))?;
    Ok(PanelItem { id: 0, ..it })
}

/// How far from its mounting height a point clicked on a wall may be for a part to go at
/// that height (in its installation zone); farther off it goes where it was clicked, mm.
pub const SNAP_HEIGHT: f32 = 200.0;

/// The height part `part` goes at for a point clicked `clicked` mm above the floor of a
/// room: its mounting height near it ([`SNAP_HEIGHT`]), otherwise the point's (`None`:
/// its mounting height).
pub fn height_for(prj: &Project, part: &str, clicked: f32) -> Option<f32> {
    let pd = find_part(prj, part)?;
    if on_ceiling(pd) {
        return None;
    }
    match panel::tech_number(pd, "Mounting height") {
        Some(h) if (clicked - h).abs() <= SNAP_HEIGHT => None,
        _ => Some(clicked.max(0.0)),
    }
}

/// Where the ray from `eye` along `dir` (room coordinates: x, z of its floor plan, y up
/// from its floor) first meets the inside of a wall or the ceiling of room `e`: the
/// point and how far along the ray.
pub fn inside_hit(e: &panel::Enclosure, eye: [f32; 3], dir: [f32; 3]) -> Option<([f32; 3], f32)> {
    let mut best: Option<([f32; 3], f32)> = None;
    let mut take = |q: [f32; 3], t: f32| {
        if t > 1.0 && best.is_none_or(|b| t < b.1) {
            best = Some((q, t));
        }
    };
    for f in panel::room_faces(e) {
        if !matches!(f, Face::Wall(_) | Face::Inner(..)) {
            continue;
        }
        let Some(w) = panel::side(e, f) else { continue };
        let n = w.normal;
        let facing = dir[0] * n[0] + dir[2] * n[1];
        if facing >= -1e-6 {
            continue; // seen from behind
        }
        let t = -((eye[0] - w.a[0]) * n[0] + (eye[2] - w.a[1]) * n[1]) / facing;
        let q = [eye[0] + dir[0] * t, eye[1] + dir[1] * t, eye[2] + dir[2] * t];
        let (start, right) = (w.start(), w.right());
        let along = (q[0] - start[0]) * right[0] + (q[2] - start[1]) * right[1];
        if along >= 0.0 && along <= w.length() && q[1] >= 0.0 && q[1] <= e.height {
            take(q, t);
        }
    }
    if dir[1] > 1e-6 {
        let t = (e.height - eye[1]) / dir[1];
        let q = [eye[0] + dir[0] * t, e.height, eye[2] + dir[2] * t];
        if panel::inside_outline(&panel::outline(e), [q[0], q[2]]) {
            take(q, t);
        }
    }
    best
}

/// Whether point `q` of room `e` (its coordinates) lies outside it beyond half the
/// thickness of its walls: a point on a wall (clicked on its surface in the 3D view)
/// belongs to the room.
fn beyond_walls(e: &panel::Enclosure, q: [f32; 2]) -> bool {
    let pts = panel::outline(e);
    if panel::inside_outline(&pts, q) {
        return false;
    }
    let dist = |a: [f32; 2], b: [f32; 2]| {
        let (d, v) = ([b[0] - a[0], b[1] - a[1]], [q[0] - a[0], q[1] - a[1]]);
        let len = d[0] * d[0] + d[1] * d[1];
        let t = if len > 0.0 { ((v[0] * d[0] + v[1] * d[1]) / len).clamp(0.0, 1.0) } else { 0.0 };
        ((v[0] - t * d[0]).powi(2) + (v[1] - t * d[1]).powi(2)).sqrt()
    };
    let near = (0..pts.len()).map(|k| dist(pts[k], pts[(k + 1) % pts.len()])).fold(f32::INFINITY, f32::min);
    near > (e.wall() / 2.0).max(20.0)
}

/// The cable that feeds device `tag` of an installation and the device it comes from
/// (`cable.source`, `cable.target`).
pub fn feed_of(prj: &Project, tag: &str) -> Option<(String, String)> {
    prj.cables
        .iter()
        .find(|c| c.values.get("cable.target").is_some_and(|t| t == tag))
        .map(|c| (c.tag.clone(), c.values.get("cable.source").cloned().unwrap_or_default()))
}

/// A point of a cable run: x, z of the plant, height above its floor, its floor above
/// the ground and the height of the installation zone above that floor (mm).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunPoint {
    pub at: [f32; 2],
    pub height: f32,
    pub floor: f32,
    pub zone: f32,
}

/// Length of the cable from `a` to `b` in the installation zones, mm.
pub fn leg(a: &RunPoint, b: &RunPoint) -> f32 {
    let plan = (a.at[0] - b.at[0]).abs() + (a.at[1] - b.at[1]).abs();
    plan + (a.zone - a.height).abs() + (b.zone - b.height).abs() + ((a.floor + a.zone) - (b.floor + b.zone)).abs()
}

/// The enclosure (distribution board) protective device `tag` is placed in; `None` when
/// it is placed nowhere or in a room.
pub fn board_of(prj: &Project, tag: &str) -> Option<usize> {
    let layout = &prj.layout;
    layout.find_device(tag).map(|(ei, _)| ei).filter(|&ei| !layout.enclosures[ei].is_room())
}

/// The start of a cable at a distribution board: the middle of its top.
fn board_point(prj: &Project, poses: &[SitePose], ei: usize) -> RunPoint {
    let layout = &prj.layout;
    let e = &layout.enclosures[ei];
    let q = poses[ei].apply([e.width / 2.0, 0.0, e.depth / 2.0]);
    let room = panel::room_of(layout, ei);
    let (floor, ceiling) = room.map_or((0.0, e.height + ZONE), |r| (floor_of(prj, r), layout.enclosures[r].height));
    let height = panel::floor_height(e).unwrap_or(0.0) + e.height;
    RunPoint { at: [q[0], q[2]], height, floor, zone: (ceiling - ZONE).max(height) }
}

/// Where device `tag` is placed, as a point of a cable run, and the cabinet it is in
/// (`None` in a room).
fn point_of(prj: &Project, poses: &[SitePose], tag: &str) -> Option<(RunPoint, Option<usize>)> {
    let (ei, it) = prj.layout.find_device(tag)?;
    let e = &prj.layout.enclosures[ei];
    if !e.is_room() {
        return Some((board_point(prj, poses, ei), Some(ei)));
    }
    let (at, height) = spot(prj, poses, ei, it)?;
    Some((RunPoint { at, height, floor: floor_of(prj, ei), zone: (e.height - ZONE).max(0.0) }, None))
}

/// Where device `tag` is placed, x and z of the plant (the middle of the top of its
/// cabinet for a device in a cabinet).
pub fn plan_point(prj: &Project, poses: &[SitePose], tag: &str) -> Option<[f32; 2]> {
    point_of(prj, poses, tag).map(|p| p.0.at)
}

/// What the voltage drops upstream of the distribution boards come from: the feeder
/// cables of each protective device of the mains and the design currents of the
/// protective devices (both by tag).
#[derive(Clone, Debug, Default)]
pub struct Supply {
    pub feeders: HashMap<String, Vec<FeederCable>>,
    pub currents: HashMap<String, f32>,
}

impl Supply {
    pub fn of(prj: &Project, nl: &Netlist, conns: &[Connection]) -> Supply {
        let currents = crate::loads::calculate(prj, nl).feeders.into_iter().filter(|f| f.current > 0.0).map(|f| (f.device, f.current)).collect();
        Supply { feeders: crate::shortcircuit::feeder_cables(prj, nl, conns), currents }
    }
}

/// Voltage of a circuit: line to line when three-phase, line to neutral otherwise
/// (from the supply, 400 V without one).
pub fn voltage(prj: &Project, three_phase: bool) -> f32 {
    let u = if prj.electrical.supply.voltage > 0.0 { prj.electrical.supply.voltage } else { 400.0 };
    if three_phase {
        u
    } else {
        u / 3f32.sqrt()
    }
}

/// Voltage drop of a cable: ΔU in V and in % of `u` (see the module documentation).
pub fn drop(prj: &Project, current: f32, length_m: f32, mm2: f32, cos_phi: f32, u: f32, three_phase: bool) -> (f32, f32) {
    let k = if three_phase { 3f32.sqrt() } else { 2.0 };
    let sin_phi = (1.0 - cos_phi * cos_phi).max(0.0).sqrt();
    let per_m = prj.electrical.conditions.rho_hot() / mm2.max(1e-3) * cos_phi + crate::shortcircuit::CABLE_REACTANCE * sin_phi;
    let du = k * current * length_m * per_m;
    (du, du / u.max(1.0) * 100.0)
}

/// The feeder cables from the supply to the board of a circuit: those of its protective
/// device, or of another one placed in the same board.
fn feeder_of<'a>(prj: &Project, supply: &'a Supply, breaker: &str, board: Option<usize>) -> Option<&'a Vec<FeederCable>> {
    supply.feeders.get(breaker).filter(|f| !f.is_empty()).or_else(|| {
        let board = board?;
        let mut tags: Vec<&String> = supply.feeders.keys().collect();
        tags.sort();
        tags.into_iter().find(|t| prj.layout.find_device(t).is_some_and(|(ei, _)| ei == board)).and_then(|t| supply.feeders.get(t)).filter(|f| !f.is_empty())
    })
}

/// Voltage drop of the feeder cables up to a circuit's board, %.
fn upstream(prj: &Project, supply: &Supply, breaker: &str, board: Option<usize>) -> Option<f32> {
    let chain = feeder_of(prj, supply, breaker, board)?;
    let mut total = 0.0;
    for cable in chain {
        let i = supply.currents.get(&cable.device).copied().or_else(|| crate::protection::of_device(prj, &cable.device).map(|p| p.rated))?;
        let three = cable.phases >= 3;
        total += drop(prj, i, cable.length, cable.mm2, DEFAULT_COS_PHI, voltage(prj, three), three).1;
    }
    Some(total)
}

/// A protective device of the mains for final circuits: its tag, function text, rated
/// current, characteristic, phases, the residual current device protecting it (mA) and
/// whether it is spare.
#[derive(Clone, Debug, PartialEq)]
pub struct Breaker {
    pub tag: String,
    pub text: String,
    pub rated: f32,
    pub characteristic: String,
    pub phases: usize,
    pub residual: Option<f32>,
    pub spare: bool,
}

impl Breaker {
    /// Its tag and function text.
    pub fn title(&self) -> String {
        format!("{} {}", self.tag, self.text).trim().to_string()
    }
}

/// The function text of device `tag` (of the first of its symbols with one).
fn text_of(prj: &Project, tag: &str) -> String {
    prj.pages.iter().flat_map(|pg| pg.symbols.iter()).find(|s| s.tag == tag && !s.function_text.is_empty()).map(|s| s.function_text.clone()).unwrap_or_default()
}

/// The protective devices of the mains (in schematic order) for final circuits (up to
/// 32 A).
pub fn breakers(prj: &Project, nl: &Netlist) -> Vec<Breaker> {
    electrical::circuits(prj, nl)
        .into_iter()
        .filter(|c| c.is_final())
        .map(|c| {
            // Spare: no other device on the nets behind it.
            let fed = c.nets.iter().any(|&n| {
                nl.nets.get(n).is_some_and(|net| net.pins.iter().any(|pr| prj.pages.get(pr.page).and_then(|pg| pg.symbols.get(pr.sym)).is_some_and(|s| !s.tag.is_empty() && s.tag != c.device)))
            });
            Breaker {
                spare: !fed,
                text: text_of(prj, &c.device),
                characteristic: c.protection.characteristic.title().to_string(),
                rated: c.protection.rated,
                phases: c.phases.len(),
                residual: c.residual,
                tag: c.device,
            }
        })
        .collect()
}

/// A socket outlet, switch, light or heating element of a circuit: its tag, kind,
/// function text, its own part for walls or ceilings (none: placed with the part of its
/// kind, [`Kind::default_part`]) and where it is placed (room, item).
#[derive(Clone, Debug, PartialEq)]
pub struct SchematicDevice {
    pub tag: String,
    pub kind: Kind,
    pub text: String,
    pub part: Option<String>,
    pub placed: Option<(usize, u64)>,
}

/// A connection of the phase conductors of a circuit between two devices (tags): its
/// cable (empty: a single wire), the cable's type, the cross-section (mm²) and the
/// length (m) of its 3D route (`routed`) or estimated along the installation zones;
/// `None` while one of its devices is not placed.
#[derive(Clone, Debug, PartialEq)]
pub struct Leg {
    pub from: String,
    pub to: String,
    pub cable: String,
    pub cable_type: String,
    pub mm2: f32,
    pub length: Option<f32>,
    pub routed: bool,
}

/// A final circuit of the schematic (a protective device of the mains up to 32 A) with
/// the socket outlets, switches, lights and heating elements behind it, in schematic
/// order, the distribution board its protective device is placed in, the connections
/// between them and its installation page (index).
#[derive(Clone, Debug, PartialEq)]
pub struct Circuit {
    pub breaker: String,
    pub text: String,
    pub rated: f32,
    pub characteristic: String,
    pub three_phase: bool,
    pub board: Option<usize>,
    pub devices: Vec<SchematicDevice>,
    pub legs: Vec<Leg>,
    pub page: Option<usize>,
}

impl Circuit {
    /// Its protective device and function text.
    pub fn title(&self) -> String {
        format!("{} {}", self.breaker, self.text).trim().to_string()
    }

    pub fn placed(&self) -> usize {
        self.devices.iter().filter(|d| d.placed.is_some()).count()
    }

    pub fn device(&self, tag: &str) -> Option<&SchematicDevice> {
        self.devices.iter().find(|d| d.tag == tag)
    }

    /// The device placed as item `id`.
    pub fn has_item(&self, id: u64) -> bool {
        self.devices.iter().any(|d| d.placed.is_some_and(|p| p.1 == id))
    }

    /// The cables of its connections, in order.
    pub fn cables(&self) -> Vec<&str> {
        self.legs.iter().map(|l| l.cable.as_str()).filter(|c| !c.is_empty()).collect()
    }
}

/// The symbol passes the circuit on to the devices behind it: a switch, push button,
/// contact, terminal or junction box (all its pins one circuit).
fn switches_through(s: &SymbolInst) -> bool {
    use crate::parts::FnKind as F;
    matches!(
        crate::parts::kind_of_symbol(&s.sym),
        Some(F::No | F::Nc | F::Changeover | F::PushbuttonNo | F::PushbuttonNc | F::SwitchNo | F::SwitchNc | F::Terminal | F::NoOnDelay | F::NoOffDelay | F::Sensor)
    ) || ["inst_switch", "inst_switch2", "inst_dimmer", "inst_push", "inst_junction", "inst_motion"].contains(&s.sym.as_str())
}

/// The devices on the nets behind protective device `c` of the schematic: through
/// pass-through devices (contacts of contactors, residual current devices),
/// switches, contacts and terminals, not through further protective devices or loads
/// and never into a potential of the supply. Tags in schematic order, and the nets.
fn behind(prj: &Project, nl: &Netlist, c: &electrical::Circuit) -> (Vec<String>, BTreeSet<usize>) {
    let mut todo: Vec<usize> = c.nets.iter().copied().collect();
    let mut seen = BTreeSet::new();
    let mut nets = BTreeSet::new();
    let mut found: HashMap<String, (usize, usize)> = HashMap::new();
    while let Some(n) = todo.pop() {
        let Some(net) = nl.nets.get(n) else { continue };
        if !seen.insert(n) || (net.named && !c.nets.contains(&n) && electrical::is_supply_potential(prj, &net.name)) {
            continue;
        }
        nets.insert(n);
        for pr in &net.pins {
            let Some(pg) = prj.pages.get(pr.page).filter(|pg| pg.kind == PageKind::Schematic) else { continue };
            let Some(s) = pg.symbols.get(pr.sym) else { continue };
            if s.tag.is_empty() || s.tag == c.device {
                continue;
            }
            let at = found.entry(s.tag.clone()).or_insert((pr.page, pr.sym));
            *at = (*at).min((pr.page, pr.sym));
            if crate::sizing::is_protective(&s.sym) {
                continue;
            }
            let through: Vec<usize> = if crate::sizing::passes_current(prj, s) {
                crate::sizing::designation_number(s, pr.pin)
                    .map(|d| if d % 2 == 1 { d + 1 } else { d - 1 })
                    .and_then(|partner| (0..s.pins.len()).find(|&j| crate::sizing::designation_number(s, j) == Some(partner)))
                    .into_iter()
                    .collect()
            } else if switches_through(s) {
                (0..s.pins.len()).filter(|&j| j != pr.pin).collect()
            } else {
                Vec::new()
            };
            todo.extend(through.into_iter().filter_map(|j| nl.pin_net.get(&(s.id, j)).copied()));
        }
    }
    let mut tags: Vec<(String, (usize, usize))> = found.into_iter().collect();
    tags.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    (tags.into_iter().map(|t| t.0).collect(), nets)
}

/// What a device of the schematic is for the installation, with its part: by its parts
/// (one for walls or ceilings), or, without parts, by its symbols (socket outlets,
/// heating elements, lamps, installation symbols, switches and contacts of an `S`
/// device). `None` for devices with other parts (devices of cabinets).
fn device_kind(prj: &Project, tag: &str) -> Option<(Kind, Option<String>)> {
    use crate::parts::FnKind as F;
    let refs = prj.devices.get(tag).map(|d| d.parts.clone()).unwrap_or_default();
    if !refs.is_empty() {
        return refs.iter().find_map(|r| find_part(prj, &r.number).and_then(kind_of).map(|k| (k, Some(r.number.clone()))));
    }
    let letters = tag.rsplit('-').next().unwrap_or("");
    let kinds: Vec<Kind> = prj
        .pages
        .iter()
        .filter(|pg| pg.kind == PageKind::Schematic)
        .flat_map(|pg| pg.symbols.iter())
        .filter(|s| s.tag == tag)
        .filter_map(|s| match (s.sym.as_str(), crate::parts::kind_of_symbol(&s.sym)) {
            ("inst_socket2" | "socket3", _) | (_, Some(F::SocketOutlet)) => Some(Kind::Socket),
            ("jbox" | "jbox3" | "inst_junction", _) => Some(Kind::Junction),
            ("conn_box" | "conn_box3", _) => Some(Kind::Connection),
            ("heater3", _) | (_, Some(F::Heater)) => Some(Kind::Heater),
            ("lamp" | "luminaire" | "inst_lamp" | "inst_luminaire", _) => Some(Kind::Light),
            ("inst_switch" | "inst_switch2" | "inst_dimmer" | "inst_push", _) => Some(Kind::Switch),
            (_, Some(F::No | F::Nc | F::Changeover | F::PushbuttonNo | F::SwitchNo)) if letters.starts_with('S') => Some(Kind::Switch),
            _ => None,
        })
        .collect();
    // A load before the switches that feed it.
    Kind::ALL.into_iter().filter(|k| *k != Kind::Switch).chain([Kind::Switch]).find(|k| kinds.contains(k)).map(|k| (k, None))
}

/// Where a device is placed in a room (room index, item id); `None` when it is not
/// placed or placed in a cabinet.
fn placed_in_room(prj: &Project, tag: &str) -> Option<(usize, u64)> {
    prj.layout.find_device(tag).filter(|(ei, _)| prj.layout.enclosures[*ei].is_room()).map(|(ei, it)| (ei, it.id))
}

/// A cross-section in mm² from a text such as "1.5 mm²".
fn mm2_of(text: &str) -> Option<f32> {
    let num: String = text.trim().chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',').collect();
    num.replace(',', ".").parse::<f32>().ok().filter(|v| *v > 0.0)
}

/// The connections of the phase conductors on the nets `nets` of the circuit of
/// `breaker` between two devices (a cable once), with their lengths: from `routing`
/// where routed, estimated otherwise.
fn legs(prj: &Project, nl: &Netlist, conns: &[Connection], routes: Routes, poses: &[SitePose], breaker: &str, nets: &BTreeSet<usize>) -> Vec<Leg> {
    let sym = |p: PinRef| &prj.pages[p.page].symbols[p.sym];
    let mut out: Vec<Leg> = Vec::new();
    for c in conns {
        if !nl.pin_net.get(&(sym(c.a).id, c.a.pin)).is_some_and(|n| nets.contains(n)) {
            continue;
        }
        let (from, to) = (sym(c.a).tag.clone(), sym(c.b).tag.clone());
        if from.is_empty() || to.is_empty() || from == to {
            continue;
        }
        let known = if c.cable.is_empty() {
            out.iter().any(|l| l.cable.is_empty() && ((l.from == from && l.to == to) || (l.from == to && l.to == from)))
        } else {
            out.iter().any(|l| l.cable == c.cable)
        };
        if known {
            continue;
        }
        let cable = prj.cables.iter().find(|x| !c.cable.is_empty() && x.tag == c.cable);
        let mm2 = cable.and_then(|x| mm2_of(&x.cross_section)).or_else(|| mm2_of(&c.cross_section)).unwrap_or(1.5);
        let routed = routes
            .and_then(|r| {
                if c.cable.is_empty() {
                    let (a, b) = (crate::netlist::pin_designation(prj, c.a), crate::netlist::pin_designation(prj, c.b));
                    r(&format!("{a}|{b}")).or_else(|| r(&format!("{b}|{a}")))
                } else {
                    r(&c.cable)
                }
            })
            .map(|mm| mm / 1000.0);
        let estimate = || {
            let ((pa, ea), (pb, eb)) = (point_of(prj, poses, &from)?, point_of(prj, poses, &to)?);
            // Wired inside the distribution board.
            if ea.is_some() && ea == eb {
                return Some(0.0);
            }
            Some(leg(&pa, &pb) / 1000.0 + BOX_RESERVE)
        };
        let known = cable.filter(|x| x.length > 0.0 && x.values.get(FIXED_LENGTH).is_some_and(|v| v == "true")).map(|x| x.length);
        let length = known.or(routed).or_else(estimate);
        out.push(Leg { cable_type: cable.map(|x| x.cable_type.clone()).unwrap_or_default(), cable: c.cable.clone(), mm2, length, routed: routed.is_some(), from, to });
    }
    orient(&mut out, breaker);
    out
}

/// Turns the connections to run away from protective device `from`.
fn orient(legs: &mut [Leg], from: &str) {
    let mut reached: BTreeSet<String> = BTreeSet::from([from.to_string()]);
    loop {
        let mut grew = false;
        for l in legs.iter_mut() {
            match (reached.contains(&l.from), reached.contains(&l.to)) {
                (false, true) => std::mem::swap(&mut l.from, &mut l.to),
                (true, false) => {}
                _ => continue,
            }
            reached.insert(l.to.clone());
            grew = true;
        }
        if !grew {
            break;
        }
    }
}

/// The cross-section for a current of `need` A: the thinnest of the usual ones that
/// carries it (1.5 mm² while the capacity is not known).
fn size_for(prj: &Project, need: f32) -> f32 {
    [1.5, 2.5, 4.0, 6.0, 10.0, 16.0].into_iter().find(|a| crate::sizing::capacity(prj, *a).is_none_or(|c| c + 0.01 >= need)).unwrap_or(16.0)
}

/// The final circuits of the schematic with socket outlets, switches, lights or heating
/// elements behind them (see [`Circuit`]), in schematic order; the lengths of their
/// connections from `routes` where they are routed.
pub fn circuits(prj: &Project, nl: &Netlist, conns: &[Connection], routes: Routes) -> Vec<Circuit> {
    let poses = panel::site_poses(prj);
    let finals = finals(prj, nl);
    electrical::circuits(prj, nl)
        .into_iter()
        .filter(|c| c.is_final())
        .filter_map(|c| {
            let (tags, nets) = behind(prj, nl, &c);
            let devices: Vec<SchematicDevice> = tags
                .into_iter()
                .filter(|tag| prj.layout.find_device(tag).is_none_or(|(ei, _)| prj.layout.enclosures[ei].is_room()))
                .filter_map(|tag| {
                    let (kind, part) = device_kind(prj, &tag)?;
                    Some(SchematicDevice { text: text_of(prj, &tag), kind, part, placed: placed_in_room(prj, &tag), tag })
                })
                .collect();
            if devices.is_empty() {
                return None;
            }
            Some(Circuit {
                text: text_of(prj, &c.device),
                rated: c.protection.rated,
                characteristic: c.protection.characteristic.title().to_string(),
                three_phase: c.phases.len() >= 3,
                board: board_of(prj, &c.device),
                legs: legs(prj, nl, conns, routes, &poses, &c.device, &nets),
                page: page_of(prj, &finals, &c.device),
                breaker: c.device,
                devices,
            })
        })
        .collect()
}

/// The circuit device `tag` is in.
pub fn circuit_of<'a>(list: &'a [Circuit], tag: &str) -> Option<&'a Circuit> {
    list.iter().find(|c| c.device(tag).is_some())
}

/// A final circuit worked out: the length of its connections (m), whether all of them
/// are routed, the current (A), voltage (V), voltage drop to its farthest device (V, %)
/// and from the supply to the distribution board (%), the protective device's rated
/// current, the thinnest cross-section (mm²) and its current-carrying capacity (A).
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub length: f32,
    pub routed: bool,
    pub rated: Option<f32>,
    pub current: Option<f32>,
    pub voltage: f32,
    pub drop: Option<(f32, f32)>,
    pub upstream: Option<f32>,
    pub mm2: Option<f32>,
    pub ampacity: Option<f32>,
}

impl Run {
    /// Voltage drop from the supply to the farthest device, %.
    pub fn total(&self) -> Option<f32> {
        self.drop.map(|d| d.1 + self.upstream.unwrap_or(0.0))
    }
}

/// Circuit `c` worked out (see [`Run`]).
pub fn run(prj: &Project, supply: &Supply, c: &Circuit) -> Run {
    let u = voltage(prj, c.three_phase);
    let rated = Some(c.rated).filter(|r| *r > 0.0);
    let current = supply.currents.get(&c.breaker).copied().or(rated);
    let length = c.legs.iter().filter_map(|l| l.length).sum();
    // The drop at each device: along the connections from the protective device.
    let worst = current.filter(|_| c.board.is_some()).and_then(|i| {
        let mut at: HashMap<&str, f32> = HashMap::from([(c.breaker.as_str(), 0.0)]);
        loop {
            let mut grew = false;
            for l in &c.legs {
                let Some(len) = l.length else { continue };
                let du = drop(prj, i, len, l.mm2, DEFAULT_COS_PHI, u, c.three_phase).0;
                for (x, y) in [(&l.from, &l.to), (&l.to, &l.from)] {
                    let v = at.get(x.as_str()).copied();
                    if let (Some(v), false) = (v, at.contains_key(y.as_str())) {
                        at.insert(y.as_str(), v + du);
                        grew = true;
                    }
                }
            }
            if !grew {
                break;
            }
        }
        c.devices.iter().filter_map(|d| at.get(d.tag.as_str()).copied()).reduce(f32::max)
    });
    let mm2 = c.legs.iter().map(|l| l.mm2).reduce(f32::min);
    Run {
        length,
        routed: !c.legs.is_empty() && c.legs.iter().all(|l| l.routed),
        rated,
        current,
        voltage: u,
        drop: worst.map(|du| (du, du / u.max(1.0) * 100.0)),
        upstream: upstream(prj, supply, &c.breaker, c.board),
        ampacity: mm2.and_then(|a| crate::sizing::capacity(prj, a)),
        mm2,
    }
}

/// The check: circuits whose protective device is in no distribution board of the
/// layout, cables thinner than their protective device asks, voltage drops above the
/// limit, devices not placed in a room (info); items of the installation that are not
/// in the schematic (info).
pub fn problems(prj: &Project, nl: &Netlist, conns: &[Connection], routes: Routes) -> Vec<Issue> {
    let mut out = Vec::new();
    let list = circuits(prj, nl, conns, routes);
    let limit = crate::rules::limits(prj).voltage_drop;
    let supply = if list.iter().any(|c| c.placed() > 0) { Supply::of(prj, nl, conns) } else { Supply::default() };
    for c in list.iter().filter(|c| c.placed() > 0) {
        let r = run(prj, &supply, c);
        let name = c.title();
        if c.board.is_none() {
            out.push(crate::issue!(Severity::Warning, None, "Circuit {}: protective device {} is in no distribution board of the layout", name, c.breaker));
        }
        if let (Some(rated), Some(cap), Some(mm2)) = (r.rated, r.ampacity, r.mm2) {
            if cap < rated {
                out.push(crate::issue!(Severity::Error, None, "Circuit {}: {} mm² carry {} A, less than the {} A of its protective device", name, mm2, format!("{cap:.0}"), format!("{rated:.0}")));
            }
        }
        if let Some(pct) = r.total() {
            if pct > limit {
                out.push(crate::issue!(Severity::Warning, None, "Circuit {}: voltage drop {} % over the limit of {} % ({} m)", name, format!("{pct:.2}"), format!("{limit:.1}"), format!("{:.1}", r.length)));
            }
        }
        let missing = c.devices.len() - c.placed();
        if missing > 0 {
            out.push(crate::issue!(Severity::Info, None, "Circuit {}: {} of its devices are placed in no room", name, missing));
        }
    }
    let poses = panel::site_poses(prj);
    let free = items(prj, &poses).into_iter().filter(|p| p.tag.is_empty()).count();
    if free > 0 {
        out.push(crate::issue!(Severity::Info, None, "{} socket outlets, switches, lights and heating elements are not in the schematic", free));
    }
    out
}

/// Writes the lengths of the cables of the circuits into the cable list (rounded up to
/// the half metre). Gives how many cables were written.
pub fn write_cables(prj: &mut Project, list: &[Circuit]) -> usize {
    let mut n = 0;
    for l in list.iter().flat_map(|c| &c.legs) {
        let (Some(length), false) = (l.length, l.cable.is_empty()) else { continue };
        if let Some(cable) = prj.cables.iter_mut().find(|x| x.tag == l.cable && x.values.get(FIXED_LENGTH).is_none_or(|v| v != "true")) {
            cable.length = (length * 2.0).ceil() / 2.0;
            n += 1;
        }
    }
    n
}

/// Device `tag` became item `id`: the device gets the item's part when it has none and
/// an earlier place of it goes.
fn adopt(prj: &mut Project, tag: &str, id: u64, old: Option<(usize, u64)>) -> Result<(), String> {
    let ri = prj.layout.enclosures.iter().position(|e| e.item(id).is_some()).ok_or_else(|| tr_fmt!("No socket outlet, switch or light {}", id))?;
    let part = prj.layout.enclosures[ri].item(id).map(|it| it.part.clone()).unwrap_or_default();
    if let Some(it) = prj.layout.enclosures[ri].items.iter_mut().find(|i| i.id == id) {
        it.kind = ItemKind::Device;
        it.tag = tag.to_string();
    }
    if !prj.parts.iter().any(|x| x.number == part) {
        if let Some(x) = find_part(prj, &part).cloned() {
            prj.parts.push(x);
        }
    }
    if prj.devices.get(tag).is_none_or(|d| d.parts.is_empty()) {
        let description = find_part(prj, &part).map(|x| x.description.clone()).unwrap_or_default();
        let data = prj.devices.entry(tag.to_string()).or_default();
        data.parts.push(crate::parts::PartRef { number: part, ..Default::default() });
        if data.description.is_empty() {
            data.description = description;
        }
    }
    if let Some((oi, old_id)) = old.filter(|o| o.1 != id) {
        prj.layout.enclosures[oi].items.retain(|i| i.id != old_id);
    }
    Ok(())
}

/// Places device `tag` of circuit `sc` into room `ri` at point `at` of the plant like
/// [`place`]: with its own part, or `part` (empty: the generic one of its kind), which
/// then becomes its part. A device placed before moves there. Gives the item's id.
pub fn place_device(prj: &mut Project, sc: &Circuit, tag: &str, ri: usize, part: &str, at: [f32; 2]) -> Result<u64, String> {
    let d = sc.device(tag).ok_or_else(|| tr_fmt!("{} is no socket outlet, switch, light or heating element of circuit {}", tag, sc.breaker))?;
    let number = match &d.part {
        Some(p) => p.clone(),
        None if !part.trim().is_empty() => {
            let pd = find_part(prj, part.trim()).ok_or_else(|| tr_fmt!("Unknown device or part: {}", part))?;
            if kind_of(pd) != Some(d.kind) {
                return Err(tr_fmt!("{} does not fit {}: choose a part of its kind", part, tag));
            }
            part.trim().to_string()
        }
        None => d.kind.default_part().to_string(),
    };
    let old = placed_in_room(prj, tag);
    let id = place(prj, ri, &number, at)?;
    adopt(prj, tag, id, old)?;
    Ok(id)
}

/// Makes item `item` (a socket outlet, switch, light or heating element placed
/// without a device tag, of the same kind) device `tag` of circuit `sc`: what was
/// placed before becomes the device of the schematic.
pub fn link_device(prj: &mut Project, sc: &Circuit, tag: &str, item: u64) -> Result<(), String> {
    let d = sc.device(tag).ok_or_else(|| tr_fmt!("{} is no socket outlet, switch, light or heating element of circuit {}", tag, sc.breaker))?;
    let poses = panel::site_poses(prj);
    let p = items(prj, &poses).into_iter().find(|p| p.item == item).ok_or_else(|| tr_fmt!("No socket outlet, switch or light {}", item))?;
    if !p.tag.is_empty() && p.tag != tag {
        return Err(tr_fmt!("{} is device {} already", item, p.tag));
    }
    if p.kind != d.kind || d.part.as_ref().is_some_and(|own| *own != p.part) {
        return Err(tr_fmt!("{} does not fit {}: choose a part of its kind", p.part, tag));
    }
    adopt(prj, tag, item, placed_in_room(prj, tag))
}

/// Letter code of the device tags of a kind of item (IEC 81346-2).
pub fn code(kind: Kind) -> &'static str {
    match kind {
        Kind::Socket => "XD",
        Kind::Switch => "SF",
        Kind::Light => "EA",
        Kind::Heater => "EB",
        Kind::Junction | Kind::Connection => "XD",
        Kind::Data | Kind::Aerial => "XF",
        Kind::Smoke => "BF",
    }
}

/// Item `item` (placed in a room without a device tag, see [`place_at`]) becomes device
/// `tag` with its part, without a place in the schematic (a data or antenna outlet, a
/// smoke alarm).
pub fn name_item(prj: &mut Project, item: u64, tag: &str) -> Result<(), String> {
    let tag = tag.trim();
    if tag_used(prj, tag) {
        return Err(tr_fmt!("{} is used already", tag));
    }
    adopt(prj, tag, item, None)
}

/// Device tag `t` is used: a device, a symbol or an item of the layout has it.
fn tag_used(prj: &Project, t: &str) -> bool {
    prj.devices.contains_key(t) || prj.pages.iter().any(|pg| pg.symbols.iter().any(|s| s.tag == t)) || prj.layout.find_device(t).is_some()
}

/// Cable tag `t` is used.
fn cable_used(prj: &Project, t: &str) -> bool {
    prj.cables.iter().any(|c| c.tag == t) || prj.pages.iter().any(|pg| pg.wires.iter().any(|w| w.cable == t))
}

/// A device tag not used yet: `prefix` and the first free number.
fn free_tag(prj: &Project, prefix: &str) -> String {
    (1..).map(|n| format!("{prefix}{n}")).find(|t| !tag_used(prj, t)).unwrap_or_default()
}

/// A cable tag not used yet: `-W` and the first free number.
fn free_cable(prj: &Project) -> String {
    (1..).map(|n| format!("-W{n}")).find(|t| !cable_used(prj, t)).unwrap_or_default()
}

// ------------------------------------------------------------- installation pages

/// Width of a column of the installation page: one device, mm (of a page with the
/// five conductors of a three-phase circuit, `wide`).
fn column(wide: bool) -> f32 {
    if wide { 12.0 * GRID } else { 8.0 * GRID }
}

/// The labels of the circuit of protective device `breaker` on its installation page:
/// the interruption points of its phase conductors (also at the protective device; `L`,
/// or `L1` … `L3` of a three-phase circuit) and the potentials of its neutral and
/// protective conductors, named after the short tag of the protective device unless
/// another one of `finals` (the protective devices of the final circuits) has it too.
fn labels(finals: &[String], breaker: &str, three: bool) -> Vec<String> {
    let short = format!("-{}", breaker.rsplit('-').next().unwrap_or(breaker));
    let unique = !finals.iter().any(|t| t != breaker && t.ends_with(&short));
    let name = if unique { short } else { breaker.to_string() };
    let phases: &[&str] = if three { &["L1", "L2", "L3"] } else { &["L"] };
    phases.iter().chain(&["N", "PE"]).map(|c| format!("{c}{name}")).collect()
}

/// The protective devices of the final circuits.
fn finals(prj: &Project, nl: &Netlist) -> Vec<String> {
    electrical::circuits(prj, nl).into_iter().filter(|c| c.is_final()).map(|c| c.device).collect()
}

/// The installation page of the circuit of protective device `breaker` (index): the
/// first one where it has several.
fn page_of(prj: &Project, finals: &[String], breaker: &str) -> Option<usize> {
    pages_of(prj, finals, breaker).first().copied()
}

/// The installation pages of the circuit of protective device `breaker` (indexes, in
/// their order): the first and those that continue it.
fn pages_of(prj: &Project, finals: &[String], breaker: &str) -> Vec<usize> {
    let ls = [labels(finals, breaker, false), labels(finals, breaker, true)];
    (0..prj.pages.len()).filter(|&k| {
        let pg = &prj.pages[k];
        prj.layout.installation_pages.contains(&pg.id) && pg.symbols.iter().any(|s| s.sym == "intr" && ls.iter().any(|l| s.label == l[0]))
    }).collect()
}

/// A page after installation page `k` that continues it (a full one): its structure,
/// labels and title. Gives its index.
fn continuation(prj: &mut Project, k: usize) -> usize {
    let src = prj.pages[k].clone();
    let id = prj.new_id();
    let mut page = Page::new(id, &crate::ops::next_page_name(prj, k + 1), &src.description, PageKind::Schematic);
    page.section = src.section;
    page.function = src.function.clone();
    page.installation = src.installation.clone();
    page.location = src.location.clone();
    page.product = src.product.clone();
    page.values = src.values.clone();
    let at = crate::ops::insert_page(prj, k + 1, page);
    prj.layout.installation_pages.push(id);
    at
}

/// How a device goes into the schematic: the device or junction box its cable comes
/// from (default: the one before it, see [`into_schematic`]), its device tag, the tag,
/// type, part, number of cores, cross-section (mm²) and known length (m) of its cable
/// and the mounting height of a wall item (mm above the floor); what is not given is
/// chosen.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Options {
    pub from: Option<String>,
    pub tag: Option<String>,
    pub cable: Option<String>,
    pub cable_type: Option<String>,
    pub cable_part: Option<String>,
    pub cores: Option<usize>,
    pub mm2: Option<f32>,
    pub length: Option<f32>,
    pub height: Option<f32>,
}

/// A device of an installation page: its tag, symbol, connection point designations,
/// function text, cable (from the device before it) and the device that cable comes
/// from (empty: not stored, a page drawn before).
#[derive(Clone, Debug)]
struct Drawn {
    tag: String,
    sym: String,
    pins: Vec<String>,
    text: String,
    kind: Kind,
    cable: String,
    from: String,
}

/// The device has three phase conductors: connection points L2 or L3.
fn three_phase(pins: &[String]) -> bool {
    pins.iter().any(|l| l.eq_ignore_ascii_case("L2") || l.eq_ignore_ascii_case("L3"))
}

/// The symbol of a new device of kind `kind` with part `part`: one of the symbols of the
/// kind of the part's function with as many connection points, a symbol of the
/// installation's kind otherwise; and the designations of its connection points (the
/// part's where they fit).
fn symbol_for(prj: &Project, kind: Kind, part: Option<&Part>) -> Option<(SymbolRef, Vec<String>)> {
    let f = part.and_then(|p| p.functions.iter().find(|f| f.kind != crate::parts::FnKind::Terminal));
    let pins = f.map(|f| f.pin_list()).unwrap_or_default();
    let three = three_phase(&pins.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    let prefs: &[&str] = match (kind, three) {
        (Kind::Socket, false) => &["socket"],
        (Kind::Socket, true) => &["socket3"],
        (Kind::Light, _) => &["luminaire", "lamp"],
        (Kind::Heater, false) => &["heater"],
        (Kind::Heater, true) => &["heater3"],
        (Kind::Switch, _) => &["sel", "co_man", "pb_no"],
        (Kind::Junction, false) => &["jbox"],
        (Kind::Junction, true) => &["jbox3"],
        (Kind::Connection, false) => &["conn_box"],
        (Kind::Connection, true) => &["conn_box3"],
        (Kind::Data | Kind::Aerial | Kind::Smoke, _) => &[],
    };
    let fits = |d: &SymbolRef, own: bool| (pins.is_empty() || d.pins.len() == pins.len()) && (!own || f.is_none_or(|f| crate::parts::kind_of_symbol(d.key) == Some(f.kind)));
    let defs: Vec<SymbolRef> = prefs.iter().filter_map(|k| prj.symbol(k)).collect();
    let def = defs.iter().find(|d| fits(d, true)).or_else(|| defs.iter().find(|d| fits(d, false))).or(defs.first())?.clone();
    let labels = if pins.len() == def.pins.len() { pins.iter().map(|s| s.to_string()).collect() } else { def.pins.iter().map(|x| x.label.to_string()).collect() };
    Some((def, labels))
}

/// The kind of a device drawn on an installation page, by its symbol.
fn drawn_kind(sym: &str) -> Kind {
    match sym {
        "socket" | "socket3" => Kind::Socket,
        "luminaire" | "lamp" => Kind::Light,
        "heater" | "heater3" => Kind::Heater,
        "jbox" | "jbox3" => Kind::Junction,
        "conn_box" | "conn_box3" => Kind::Connection,
        _ => Kind::Switch,
    }
}

/// Puts a socket outlet, switch, light, heating element, junction box or device
/// connection box (part `part`) for protective device `breaker` into room `ri` at point
/// `at` of the plant like [`place`] (at the mounting height of `opts`) and into the
/// schematic ([`into_schematic`]). Gives the device tag; nothing is placed when it
/// cannot go into the schematic.
pub fn add_device(prj: &mut Project, breaker: &str, part: &str, ri: usize, at: [f32; 2], opts: &Options) -> Result<String, String> {
    match find_part(prj, part).and_then(kind_of) {
        None => return Err(tr_fmt!("{} is no socket outlet, switch, light or heating element", part)),
        Some(k) if !k.mains() => return Err(tr_fmt!("{} is not connected to the mains: place it without a protective device", part)),
        Some(_) => {}
    }
    let id = place_at(prj, ri, part, at, opts.height)?;
    into_schematic(prj, breaker, id, opts).inspect_err(|_| {
        prj.layout.enclosures[ri].items.retain(|i| i.id != id);
    })
}

/// The cores of a new cable of `n` cores: brown, black, grey, blue and green-yellow as
/// many as it has (2: brown and blue), numbered with green-yellow from six.
fn core_names(n: usize) -> Vec<String> {
    let colours: &[&str] = match n {
        0 | 1 => &["BN"],
        2 => &["BN", "BU"],
        3 => &["BN", "BU", "GNYE"],
        4 => &["BN", "BK", "BU", "GNYE"],
        5 => &["BN", "BK", "GY", "BU", "GNYE"],
        _ => return (1..n).map(|k| k.to_string()).chain(["GNYE".to_string()]).collect(),
    };
    colours.iter().map(|c| c.to_string()).collect()
}

/// The core of a cable with the cores `cores` for conductor `k` of an installation page
/// with `n` conductors (the phases, then the neutral and the protective conductor).
fn core_of(cores: &[String], n: usize, k: usize) -> String {
    let has = |c: &str| cores.iter().any(|x| x == c).then(|| c.to_string());
    let at = |i: usize| cores.get(i).or(cores.last()).cloned().unwrap_or_default();
    if k + 1 == n {
        return has("GNYE").unwrap_or_else(|| at(k));
    }
    if k + 2 == n {
        return has("BU").unwrap_or_else(|| at(k));
    }
    ["BN", "BK", "GY"].get(k).and_then(|c| has(c)).unwrap_or_else(|| at(k))
}

/// Puts item `item` (a socket outlet, switch, light, heating element, junction box or
/// device connection box placed in a room without a device tag) for protective device
/// `breaker` into the schematic: a new device `<room>-XD1` (`-SF`, `-EA`, `-EB`, or the
/// tag of `opts`) with its part, drawn on the installation page of the circuit (made with
/// the first device, after the page of the protective device, which gets interruption
/// points to it) with a new cable from the device `opts` names, or from the device before
/// it (a light from the first switch of its room, after the lights behind it), or from
/// the protective device; the cable's source and target stored with it (`cable.source`,
/// `cable.target`), which the connections follow. A three-phase device needs a
/// three-phase circuit. Gives the device tag.
pub fn into_schematic(prj: &mut Project, breaker: &str, item: u64, opts: &Options) -> Result<String, String> {
    let nl = Netlist::build(prj, true);
    let b = electrical::circuits(prj, &nl).into_iter().find(|c| c.is_final() && c.device == breaker).ok_or_else(|| tr_fmt!("{} is no protective device of a final circuit in the schematic", breaker))?;
    let (ri, it, part) = installed(prj, item).ok_or_else(|| tr_fmt!("No socket outlet, switch or light {}", item))?;
    if !it.tag.is_empty() {
        return Err(tr_fmt!("{} is device {} already", item, it.tag));
    }
    let kind = find_part(prj, &part).and_then(kind_of).ok_or_else(|| tr_fmt!("{} is no socket outlet, switch, light or heating element", part))?;
    if !kind.mains() {
        return Err(tr_fmt!("{} is not connected to the mains: place it without a protective device", part));
    }
    let room = prj.layout.enclosures[ri].name.clone();
    let finals = finals(prj, &nl);
    let (def, pins) = symbol_for(prj, kind, find_part(prj, &part)).ok_or_else(|| tr("Unknown symbol").to_string())?;
    let pages = pages_of(prj, &finals, breaker);
    let first = match pages.first() {
        Some(&k) => k,
        None => new_page(prj, &b, &labels(&finals, breaker, b.phases.len() >= 3))?,
    };
    let ls = page_labels(prj, first);
    let n = ls.len();
    if three_phase(&pins) && n < 5 {
        return Err(tr_fmt!("{} has three phases: it needs a three-phase protective device", part));
    }
    // The devices of the circuit on all its pages; a full last page is continued on a
    // new one after it.
    let pages = if pages.is_empty() { vec![first] } else { pages };
    let everyone: Vec<Drawn> = pages.iter().flat_map(|&k| drawn_on(prj, k)).collect();
    let last = *pages.last().unwrap_or(&first);
    let page = if page_columns(prj, last, n > 3) < drawn_on(prj, last).len() + 1 { continuation(prj, last) } else { last };
    let mut drawn = drawn_on(prj, page);
    let from_all = sources(prj, &everyone, breaker);
    let from = match opts.from.as_deref().map(str::trim).filter(|f| !f.is_empty()) {
        Some(f) if f == breaker || everyone.iter().any(|d| d.tag == f) => f.to_string(),
        Some(f) => return Err(tr_fmt!("{} is neither the protective device nor a device of the circuit of {}", f, breaker)),
        None => default_source(prj, &everyone, &from_all, breaker, kind, ri),
    };
    let tag = match opts.tag.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        Some(t) if tag_used(prj, t) => return Err(tr_fmt!("{} is used already", t)),
        Some(t) => t.to_string(),
        None => free_tag(prj, &format!("{}-{}", room, code(kind))),
    };
    let cable = match opts.cable.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        Some(c) if cable_used(prj, c) => return Err(tr_fmt!("{} is used already", c)),
        Some(c) => c.to_string(),
        None => free_cable(prj),
    };
    let count = opts.cores.unwrap_or(n);
    if count < n {
        return Err(tr_fmt!("The cable of {} needs at least {} cores", tag, n));
    }
    adopt(prj, &tag, item, None)?;
    // In a junction box hanging in the room: a wire connector per conductor.
    if prj.layout.enclosures[ri].item(item).is_none() {
        if let Some(d) = prj.devices.get_mut(&tag) {
            for r in d.parts.iter_mut().filter(|r| r.number == BOX_CONNECTOR) {
                r.count = pins.len().max(1) as u32;
            }
        }
    }
    let cores = core_names(count);
    let id = prj.new_id();
    let mut c = crate::model::Cable { id, tag: cable.clone(), cores: cores.join(" "), part: opts.cable_part.clone().unwrap_or_default(), ..Default::default() };
    if let Some(mm2) = opts.mm2 {
        c.cross_section = format!("{mm2} mm²");
        c.cable_type = format!("NYM-J {count}x{mm2}");
    }
    if let Some(t) = opts.cable_type.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        c.cable_type = t.to_string();
    }
    if let Some(l) = opts.length.filter(|l| *l > 0.0) {
        c.length = l;
        c.values.insert(FIXED_LENGTH.into(), "true".into());
    }
    let sized = opts.mm2.is_some() || opts.cable_type.is_some();
    prj.cables.push(c);
    drawn.push(Drawn { tag: tag.clone(), sym: def.key.to_string(), pins, text: String::new(), kind, cable, from });
    draw(prj, page, &drawn, breaker, &ls);
    // The cables carry the protective device's rated current and the highest rated
    // current of the devices of the circuit (those whose size was given keep it).
    let need = behind(prj, &nl, &b)
        .0
        .iter()
        .chain(std::iter::once(&tag))
        .filter_map(|t| prj.devices.get(t))
        .flat_map(|d| d.parts.iter())
        .filter_map(|r| find_part(prj, &r.number))
        .filter_map(|x| crate::sizing::parse_current(&x.current))
        .fold(b.protection.rated, f32::max);
    let mm2 = size_for(prj, need);
    let last = drawn.last().map(|d| d.cable.clone()).unwrap_or_default();
    let ours: Vec<&str> = drawn.iter().map(|d| d.cable.as_str()).filter(|c| !(sized && *c == last)).collect();
    for c in prj.cables.iter_mut().filter(|c| ours.contains(&c.tag.as_str())) {
        let generated = c.cable_type.is_empty() || c.cable_type.starts_with("NYM-J ");
        if generated && mm2_of(&c.cross_section).is_none_or(|a| a < mm2) {
            c.cross_section = format!("{mm2} mm²");
            c.cable_type = format!("NYM-J {}x{mm2}", c.core_list().len());
        }
    }
    Ok(tag)
}

/// Property of a cable whose length is known (measured or given): it is taken instead
/// of the 3D route or the estimate and not written over.
pub const FIXED_LENGTH: &str = "cable.fixed_length";

/// Makes the installation page of circuit `c` (after the page of its protective device)
/// and puts the interruption points of the phase conductors (the first of `labels`) at
/// the outgoing connection points of the protective device, from the left. Gives the
/// page's index.
fn new_page(prj: &mut Project, c: &electrical::Circuit, labels: &[String]) -> Result<usize, String> {
    let at = prj.pages.get(c.page).ok_or_else(|| tr("No such page").to_string())?;
    let si = at.symbols.iter().position(|s| s.id == c.symbol).ok_or_else(|| tr("Unknown symbol").to_string())?;
    let s = at.symbols[si].clone();
    let def = prj.symbol(&s.sym).ok_or_else(|| tr("Unknown symbol").to_string())?;
    // The outgoing connection points: the lowest ones, from the left.
    let y = |i: usize| def.pins[i].pos.transform(s.rot, s.mirror).y;
    let low = (0..def.pins.len()).map(y).fold(f32::MIN, f32::max);
    let mut outs: Vec<usize> = (0..def.pins.len()).filter(|&i| (y(i) - low).abs() < 0.01).collect();
    outs.sort_by(|a, b| def.pins[*a].pos.transform(s.rot, s.mirror).x.total_cmp(&def.pins[*b].pos.transform(s.rot, s.mirror).x));
    let intr = prj.symbol("intr").ok_or_else(|| tr("Unknown symbol").to_string())?;
    let phases = labels.len().saturating_sub(2).max(1);
    let right = outs.iter().take(phases).map(|&o| def.pin_world(o, s.pos, s.rot, s.mirror).x).fold(f32::MIN, f32::max) + 3.0 * GRID;
    for (k, &out) in outs.iter().take(phases).enumerate() {
        let q = def.pin_world(out, s.pos, s.rot, s.mirror);
        // Below the connection point; beside a wire leaving it (several: each a step
        // lower, to the right of the device).
        let wired = prj.pages[c.page].wires.iter().any(|w| w.a.same(q) || w.b.same(q));
        let (end, rot) = if !wired {
            (p(q.x, q.y + 3.0 * GRID), 0)
        } else if phases == 1 {
            (p(q.x + 3.0 * GRID, q.y), 3)
        } else {
            (p(right, q.y + (k as f32 + 1.0) * GRID), 3)
        };
        if wired && phases > 1 {
            push_wire(prj, c.page, q, p(q.x, end.y), "", "", false);
            push_wire(prj, c.page, p(q.x, end.y), end, "", "", false);
        } else {
            push_wire(prj, c.page, q, end, "", "", false);
        }
        let mut point = new_symbol(prj, &intr, end, "", "");
        point.rot = rot;
        point.label = labels[k].clone();
        prj.pages[c.page].symbols.push(point);
    }
    let src = prj.pages[c.page].clone();
    let id = prj.new_id();
    let text = format!("{} {}", c.device, text_of(prj, &c.device)).trim().to_string();
    let mut page = Page::new(id, &crate::ops::next_page_name(prj, c.page + 1), &tr_fmt!("Installation of {}", text), PageKind::Schematic);
    // The structure of the page of the protective device.
    page.function = src.function.clone();
    page.installation = src.installation.clone();
    page.location = src.location.clone();
    page.product = src.product.clone();
    // Each device its own cable from the one before it.
    page.values.insert("page.cable_per_connection".into(), "true".into());
    // The labels the page is drawn with.
    page.values.insert(LABELS.into(), labels.join(" "));
    let k = crate::ops::insert_page(prj, c.page + 1, page);
    prj.layout.installation_pages.push(id);
    Ok(k)
}

/// Page value with the labels of the conductors of an installation page.
const LABELS: &str = "installation.labels";

/// The labels of the conductors of installation page `page` (see [`labels`]): stored
/// with it, or those of its interruption point and potentials (a page drawn before).
fn page_labels(prj: &Project, page: usize) -> Vec<String> {
    let pg = &prj.pages[page];
    if let Some(v) = pg.values.get(LABELS).filter(|v| !v.trim().is_empty()) {
        return v.split_whitespace().map(String::from).collect();
    }
    let intr: Vec<String> = pg.symbols.iter().filter(|s| s.sym == "intr").map(|s| s.label.clone()).collect();
    let pots: Vec<String> = pg.symbols.iter().filter(|s| s.sym == "pot").map(|s| s.label.clone()).collect();
    intr.into_iter().chain(pots).collect()
}

/// The devices drawn on installation page `page`, in their order, with the device each
/// one's cable comes from.
fn drawn_on(prj: &Project, page: usize) -> Vec<Drawn> {
    let pg = &prj.pages[page];
    let mut out: Vec<(f32, Drawn)> = Vec::new();
    for s in &pg.symbols {
        let Some(def) = prj.symbol(&s.sym).filter(|d| d.func.has_tag() && !s.tag.is_empty()) else { continue };
        let at: Vec<P> = (0..def.pins.len()).map(|i| def.pin_world(i, s.pos, s.rot, s.mirror)).collect();
        let cable = at.iter().find_map(|q| pg.wires.iter().find(|w| !w.cable.is_empty() && (w.a.same(*q) || w.b.same(*q)))).map(|w| w.cable.clone()).unwrap_or_default();
        let from = prj.cables.iter().find(|c| !cable.is_empty() && c.tag == cable).and_then(|c| c.values.get("cable.source")).cloned().unwrap_or_default();
        out.push((s.pos.x, Drawn { tag: s.tag.clone(), sym: s.sym.clone(), pins: s.pins.clone(), text: s.function_text.clone(), kind: drawn_kind(&s.sym), cable, from }));
    }
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out.into_iter().map(|d| d.1).collect()
}

/// The device the cable of each device of `drawn` comes from: as stored, for a page
/// drawn before the sources were stored as it was drawn (the device before it in its
/// row; the lights of a room behind its first switch, one after the other).
fn sources(prj: &Project, drawn: &[Drawn], breaker: &str) -> Vec<String> {
    let mut out: Vec<String> = drawn.iter().map(|d| d.from.clone()).collect();
    if out.iter().any(String::is_empty) {
        let idx = |t: &str| drawn.iter().position(|d| d.tag == t);
        let mut prev = breaker.to_string();
        for (dev, lights) in by_rooms(prj, drawn) {
            let Some(i) = idx(&dev.tag) else { continue };
            if out[i].is_empty() {
                out[i] = prev.clone();
            }
            let mut before = dev.tag.clone();
            for l in &lights {
                if let Some(j) = idx(&l.tag).filter(|&j| out[j].is_empty()) {
                    out[j] = before.clone();
                }
                before = l.tag.clone();
            }
            prev = dev.tag.clone();
        }
    }
    out.into_iter().map(|s| if s.is_empty() { breaker.to_string() } else { s }).collect()
}

/// A column of an installation page: a device on the phase conductor, and the lights
/// behind it when it is a switch.
type Column = (Drawn, Vec<Drawn>);

/// The columns of the devices of an installation page drawn before their sources were
/// stored: by room in the order of the rooms, in a room the devices on the phase
/// conductor, the lights of a room with a switch behind its first switch.
fn by_rooms(prj: &Project, drawn: &[Drawn]) -> Vec<Column> {
    let room = |tag: &str| prj.layout.find_device(tag).map(|(ei, _)| ei).filter(|&ei| prj.layout.enclosures[ei].is_room()).unwrap_or(usize::MAX);
    let mut rooms: Vec<usize> = drawn.iter().map(|d| room(&d.tag)).collect();
    rooms.sort();
    rooms.dedup();
    let mut out = Vec::new();
    for r in rooms {
        let here: Vec<&Drawn> = drawn.iter().filter(|d| room(&d.tag) == r).collect();
        let switch = here.iter().position(|d| d.kind == Kind::Switch);
        for d in &here {
            if d.kind == Kind::Light && switch.is_some() || d.kind == Kind::Switch {
                continue;
            }
            out.push(((*d).clone(), Vec::new()));
        }
        for (k, d) in here.iter().enumerate().filter(|(_, d)| d.kind == Kind::Switch) {
            let lights = if Some(k) == switch { here.iter().filter(|x| x.kind == Kind::Light).map(|x| (*x).clone()).collect() } else { Vec::new() };
            out.push(((*d).clone(), lights));
        }
    }
    out
}

/// The columns of the devices of an installation page (`from`: the device the cable of
/// each comes from): the tree of the cables from the protective device, each device
/// after the one its cable comes from (the devices fed from one device in the order they
/// were put in); the lights fed from a switch, or from a light behind one, behind the
/// switch.
fn columns(drawn: &[Drawn], from: &[String], breaker: &str) -> Vec<Column> {
    let idx = |t: &str| drawn.iter().position(|d| d.tag == t);
    let behind: Vec<bool> = (0..drawn.len())
        .map(|i| {
            let mut k = i;
            for _ in 0..drawn.len() {
                if drawn[k].kind != Kind::Light {
                    return false;
                }
                match idx(&from[k]) {
                    Some(j) if drawn[j].kind == Kind::Switch => return true,
                    Some(j) => k = j,
                    None => return false,
                }
            }
            false
        })
        .collect();
    let tree = Tree { drawn, from, behind: &behind };
    let mut seen = vec![false; drawn.len()];
    let mut out = Vec::new();
    tree.walk(breaker, &mut seen, &mut out);
    // Devices whose cable comes from nowhere on the page.
    for i in 0..drawn.len() {
        if !seen[i] && !behind[i] {
            seen[i] = true;
            out.push((drawn[i].clone(), Vec::new()));
            tree.walk(&drawn[i].tag, &mut seen, &mut out);
        }
    }
    for i in 0..drawn.len() {
        if !seen[i] {
            out.push((drawn[i].clone(), Vec::new()));
        }
    }
    out
}

/// The devices of an installation page with the devices their cables come from, and
/// which are lights behind a switch.
struct Tree<'a> {
    drawn: &'a [Drawn],
    from: &'a [String],
    behind: &'a [bool],
}

impl Tree<'_> {
    /// The columns of the devices fed from `tag`, depth first.
    fn walk(&self, tag: &str, seen: &mut [bool], out: &mut Vec<Column>) {
        for i in 0..self.drawn.len() {
            if seen[i] || self.from[i] != tag || self.behind[i] {
                continue;
            }
            seen[i] = true;
            let mut lights = Vec::new();
            if self.drawn[i].kind == Kind::Switch {
                self.lights(&self.drawn[i].tag, seen, &mut lights);
            }
            out.push((self.drawn[i].clone(), lights.iter().map(|&j| self.drawn[j].clone()).collect()));
            self.walk(&self.drawn[i].tag, seen, out);
            for j in lights {
                self.walk(&self.drawn[j].tag, seen, out);
            }
        }
    }

    /// The lights behind a switch fed from `tag`, depth first.
    fn lights(&self, tag: &str, seen: &mut [bool], out: &mut Vec<usize>) {
        for i in 0..self.drawn.len() {
            if !seen[i] && self.behind[i] && self.from[i] == tag {
                seen[i] = true;
                out.push(i);
                self.lights(&self.drawn[i].tag, seen, out);
            }
        }
    }
}

/// The device the cable of a new device of kind `kind` in room `ri` comes from: a light
/// from the first switch of its room (after the lights behind it), anything else from
/// the last device of the page, or from the protective device.
fn default_source(prj: &Project, drawn: &[Drawn], from: &[String], breaker: &str, kind: Kind, ri: usize) -> String {
    let cols = columns(drawn, from, breaker);
    if kind == Kind::Light {
        let room = |t: &str| prj.layout.find_device(t).map(|(ei, _)| ei);
        if let Some((sw, lights)) = cols.iter().find(|(d, _)| d.kind == Kind::Switch && room(&d.tag) == Some(ri)) {
            return lights.last().map_or_else(|| sw.tag.clone(), |l| l.tag.clone());
        }
    }
    cols.last().map_or_else(|| breaker.to_string(), |(d, _)| d.tag.clone())
}

/// How many device columns fit on page `page` (`wide`: of a three-phase circuit).
fn page_columns(prj: &Project, page: usize, wide: bool) -> usize {
    let border = crate::frame::of_page(prj, page).border();
    (((border.max.x - border.min.x) - 10.0 * GRID) / column(wide)).floor().max(0.0) as usize
}

/// The connection points of a device symbol placed with its first upper one at `at`
/// on a page with `n` conductors: the symbol's position and rotation, the connection
/// point of each conductor (phases, neutral, protective conductor; a single-phase
/// device on the first phase) and the switched output of a switch.
fn pins_of(def: &SymbolRef, kind: Kind, at: P, n: usize) -> (P, u8, Vec<Option<P>>, Option<P>) {
    // A switch with more upper than lower connection points stands on its head.
    let rot = if kind == Kind::Switch && def.pins.iter().filter(|q| q.pos.y < 0.0).count() > 1 { 2 } else { 0 };
    let pos_of = |i: usize| def.pins[i].pos.transform(rot, false);
    let top = def.pins.iter().enumerate().map(|(i, _)| pos_of(i).y).fold(f32::MAX, f32::min);
    let first = (0..def.pins.len()).filter(|&i| (pos_of(i).y - top).abs() < 0.01).min_by(|a, b| pos_of(*a).x.total_cmp(&pos_of(*b).x)).unwrap_or(0);
    let pos = p(at.x - pos_of(first).x, at.y - pos_of(first).y);
    let world = |i: usize| pos + pos_of(i);
    let label = |l: &str| (0..def.pins.len()).find(|&i| def.pins[i].label.eq_ignore_ascii_case(l)).map(world);
    let lower: Vec<usize> = (0..def.pins.len()).filter(|&i| pos_of(i).y > top + 0.01).collect();
    let below = lower.iter().copied().min_by(|a, b| (pos_of(*a).x - pos_of(first).x).abs().total_cmp(&(pos_of(*b).x - pos_of(first).x).abs())).map(world);
    let mut at = vec![None; n];
    if kind == Kind::Switch {
        at[0] = Some(world(first));
        return (pos, rot, at, below);
    }
    at[0] = label("L1").or_else(|| label("L")).or(Some(world(first)));
    for (k, l) in ["L2", "L3"].iter().enumerate().take(n.saturating_sub(3)) {
        at[k + 1] = label(l);
    }
    if n >= 2 {
        at[n - 2] = label("N").or(below);
    }
    if n >= 3 {
        at[n - 1] = label("PE");
    }
    (pos, rot, at, None)
}

/// Draws installation page `page` again with the devices `drawn` (see [`columns`]) of
/// the circuit of `breaker` and the conductors `labels`: the interruption points of the
/// phase conductors and the potentials of the neutral and protective conductors at the
/// left, the conductors as lines along the top, each device below them in its column;
/// the lights behind a switch below its output. The wires to each device are the cores
/// of its cable, whose source and target are stored with it.
fn draw(prj: &mut Project, page: usize, drawn: &[Drawn], breaker: &str, labels: &[String]) {
    let from = sources(prj, drawn, breaker);
    for (d, f) in drawn.iter().zip(&from) {
        if let Some(c) = prj.cables.iter_mut().find(|c| !d.cable.is_empty() && c.tag == d.cable) {
            c.values.insert("cable.source".into(), f.clone());
            c.values.insert("cable.target".into(), d.tag.clone());
        }
    }
    let n = labels.len().max(3);
    let col = column(n > 3);
    let cols = columns(drawn, &from, breaker);
    prj.pages[page].symbols.clear();
    prj.pages[page].wires.clear();
    prj.pages[page].values.insert(LABELS.into(), labels.join(" "));
    let border = crate::frame::of_page(prj, page).border();
    let x0 = ((border.min.x + 4.0 * GRID) / GRID).round() * GRID;
    let y0 = ((border.min.y + 8.0 * GRID) / GRID).round() * GRID;
    let ys: Vec<f32> = (0..n).map(|k| y0 + 2.0 * GRID * k as f32).collect();
    let row = y0 + (2.0 * (n as f32 - 1.0) + 8.0) * GRID;
    // The starts: the interruption points and the potentials point to the left.
    for (k, label) in labels.iter().enumerate() {
        let key = if k + 2 < n { "intr" } else { "pot" };
        let Some(def) = prj.symbol(key) else { continue };
        let at = p(x0, ys[k]);
        let rot = if key == "intr" { 1 } else { 3 };
        let pin = def.pins.first().map_or(p(0.0, 0.0), |q| q.pos.transform(rot, false));
        let mut s = new_symbol(prj, &def, p(at.x - pin.x, at.y - pin.y), "", "");
        s.rot = rot;
        s.label = label.to_string();
        prj.pages[page].symbols.push(s);
    }
    // The lines of the conductors and their last points.
    let mut lines = Lines { page, ends: ys.iter().map(|y| p(x0, *y)).collect(), ys };
    let mut x = x0 + 4.0 * GRID;
    for (dev, lights) in &cols {
        let Some(def) = prj.symbol(&dev.sym) else { continue };
        let (pos, rot, at, out) = pins_of(&def, dev.kind, p(x, row), n);
        let mut s = new_symbol(prj, &def, pos, &dev.tag, &dev.text);
        s.rot = rot;
        s.pins = dev.pins.clone();
        prj.pages[page].symbols.push(s);
        let cores = cores_of(prj, &dev.cable);
        lines.conductors(prj, &at, x, &dev.cable, &cores);
        x += col;
        // The lights behind the switch: from its output along a line below it.
        let Some(o) = out else { continue };
        let line = ((o.y + 2.0 * GRID) / GRID).round() * GRID;
        let low = line + 3.0 * GRID;
        let mut last = o;
        let mut first = true;
        for d in lights {
            let Some(ld) = prj.symbol(&d.sym) else { continue };
            let (lpos, lrot, lat, _) = pins_of(&ld, d.kind, p(x, low), n);
            let mut s = new_symbol(prj, &ld, lpos, &d.tag, &d.text);
            s.rot = lrot;
            s.pins = d.pins.clone();
            prj.pages[page].symbols.push(s);
            let cores = cores_of(prj, &d.cable);
            let phase = core_of(&cores, n, 0);
            if let Some(q) = lat[0] {
                let j = p(q.x, line);
                if first {
                    push_wire(prj, page, last, p(last.x, line), &d.cable, &phase, true);
                    last = p(last.x, line);
                    first = false;
                }
                push_wire(prj, page, last, j, &d.cable, &phase, true);
                push_wire(prj, page, j, q, &d.cable, &phase, false);
                last = j;
            }
            let mut rest = lat.clone();
            rest[0] = None;
            lines.conductors(prj, &rest, x, &d.cable, &cores);
            x += col;
        }
    }
}

/// The cores of cable `tag` (those of a new single-phase cable when it has none).
fn cores_of(prj: &Project, tag: &str) -> Vec<String> {
    prj.cables.iter().find(|c| c.tag == tag).map(|c| c.core_list()).filter(|c| !c.is_empty()).unwrap_or_else(|| core_names(3))
}

/// The lines of the conductors along the top of an installation page: their heights
/// and last points.
struct Lines {
    page: usize,
    ys: Vec<f32>,
    ends: Vec<P>,
}

impl Lines {
    /// Joins connection point `q` to the line of conductor `k`: along the line, then
    /// down to it, as core `core` of `cable`.
    fn join(&mut self, prj: &mut Project, k: usize, q: P, cable: &str, core: &str) {
        let j = p(q.x, self.ys[k]);
        push_wire(prj, self.page, self.ends[k], j, cable, core, true);
        push_wire(prj, self.page, j, q, cable, core, false);
        self.ends[k] = j;
    }

    /// Joins the connection points `at` of a device in the column at `x` to the lines of
    /// their conductors: from above when they are upper connection points, one at the
    /// bottom (a neutral) along a step to the right of the device.
    fn conductors(&mut self, prj: &mut Project, at: &[Option<P>], x: f32, cable: &str, cores: &[String]) {
        let n = self.ys.len();
        for (k, q) in at.iter().enumerate() {
            let Some(q) = *q else { continue };
            let core = core_of(cores, n, k);
            if k == 0 || q.x > x + 0.01 {
                self.join(prj, k, q, cable, &core);
            } else {
                // At the bottom: down, to the right and up to the line.
                let (down, side) = (p(q.x, q.y + 2.0 * GRID), p(x + 3.0 * GRID, q.y + 2.0 * GRID));
                push_wire(prj, self.page, q, down, cable, &core, true);
                push_wire(prj, self.page, down, side, cable, &core, true);
                self.join(prj, k, side, cable, &core);
            }
        }
    }
}

fn new_symbol(prj: &mut Project, def: &crate::symbols::SymbolDef, pos: P, tag: &str, text: &str) -> SymbolInst {
    SymbolInst {
        id: prj.new_id(),
        sym: def.key.to_string(),
        pos,
        tag: tag.to_string(),
        function_text: text.to_string(),
        pins: def.pins.iter().map(|x| x.label.to_string()).collect(),
        ..Default::default()
    }
}

/// A wire from `a` to `b`, a core of `cable`; its labels hidden (`quiet`) on the lines
/// along the page, shown on the wire down to a device.
fn push_wire(prj: &mut Project, page: usize, a: P, b: P, cable: &str, core: &str, quiet: bool) {
    if a.same(b) {
        return;
    }
    let id = prj.new_id();
    let core = if cable.is_empty() { "" } else { core };
    prj.pages[page].wires.push(Wire { id, a, b, cable: cable.to_string(), core: core.to_string(), hide_labels: quiet, ..Default::default() });
}

#[cfg(test)]
mod tests;
