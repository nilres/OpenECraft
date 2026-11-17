//! The electrical installation of a building: socket outlets, switches and lights in
//! the rooms ([`kind_of`]), placed on the floor plan ([`place`]), and the final circuits
//! that feed them ([`FinalCircuit`]): a protective device of a distribution board in the
//! schematic, a cable and the items it runs to one after the other.
//!
//! **Cable length** ([`runs`]): the cable leaves the distribution board at its top and
//! runs to each item in turn in the horizontal installation zone under the ceiling
//! (DIN 18015-3: [`ZONE`] below it), straight along x and z of the plant (along the
//! walls, not across rooms), up and down in the vertical zones above and below the
//! items, up or down between levels; each item adds [`BOX_RESERVE`] for its box. An
//! estimate for planning before the cable is laid.
//!
//! **Voltage drop**: ΔU = k · I · L · ρ · cos φ / A with k = 2 (single-phase) or √3
//! (three-phase), ρ of copper at its operating temperature
//! ([`crate::electrical::Conditions`]), I the rated current of the protective device
//! (the highest current it lets the cable carry) or the load given, against the limit
//! of the rule sets ([`crate::rules::limits`]).
//!
//! **Protective devices** ([`breakers`]): those of the mains in the schematic
//! ([`crate::electrical::circuits`]); a spare one feeds nothing yet: no other device on
//! the nets behind it and no final circuit.

use crate::electrical;
use crate::geom::p;
use crate::issue::{Issue, Severity};
use crate::model::panel::FinalCircuit;
use crate::model::Project;
use crate::netlist::Netlist;
use crate::panel::{self, Face, ItemKind, PanelItem, SitePose};
use crate::parts::Part;
use openecraft_i18n::{tr, tr_fmt};

/// Distance of the horizontal installation zone below the ceiling, mm (DIN 18015-3:
/// 150 … 450 mm).
pub const ZONE: f32 = 300.0;
/// Cable kept in the box of each item, m.
pub const BOX_RESERVE: f32 = 0.3;
/// Farthest a click may lie from a wall to put a wall item on it, mm.
pub const WALL_REACH: f32 = 800.0;
/// Cable of a new circuit.
pub const DEFAULT_CABLE: &str = "NYM-J 3x1.5";
pub const DEFAULT_CABLE_3P: &str = "NYM-J 5x2.5";

/// What an item of a room is for the installation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Socket,
    Switch,
    Light,
}

impl Kind {
    pub fn title(self) -> &'static str {
        match self {
            Kind::Socket => tr("Socket outlet"),
            Kind::Switch => tr("Switch"),
            Kind::Light => tr("Light"),
        }
    }
}

/// What a part is for the installation: by its subgroup (socket outlets, switches,
/// push buttons, dimmers, lights) or its functions; `None` for anything else.
pub fn kind_of(part: &Part) -> Option<Kind> {
    let sub = part.subgroup.to_lowercase();
    if sub.contains("socket") {
        return Some(Kind::Socket);
    }
    if ["switch", "push button", "dimmer"].iter().any(|k| sub.contains(k)) {
        return Some(Kind::Switch);
    }
    if ["luminaire", "light", "lamp"].iter().any(|k| sub.contains(k)) {
        return Some(Kind::Light);
    }
    part.functions.iter().any(|f| f.kind == crate::parts::FnKind::SocketOutlet).then_some(Kind::Socket)
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

/// Where an item of a room is (see [`Placed`]); `None` for items that are no part of
/// the installation.
pub fn placed(prj: &Project, poses: &[SitePose], ri: usize, it: &PanelItem) -> Option<Placed> {
    if !matches!(it.kind, ItemKind::Device | ItemKind::Accessory) {
        return None;
    }
    let kind = panel::part_by_number(prj, &it.part).and_then(kind_of)?;
    let e = &prj.layout.enclosures[ri];
    let (lo, hi) = panel::item_box(e, it);
    let c = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
    let q = poses.get(ri)?.apply([c[0], 0.0, c[2]]);
    // Room coordinates: y down from the ceiling.
    let height = (e.height - c[1]).clamp(0.0, e.height);
    Some(Placed { room: ri, item: it.id, part: it.part.clone(), kind, at: [q[0], q[2]], height, floor: floor_of(prj, ri), ceiling: e.height, level: e.level })
}

/// Every item of the installation in the rooms.
pub fn items(prj: &Project, poses: &[SitePose]) -> Vec<Placed> {
    panel::rooms(&prj.layout).into_iter().flat_map(|ri| prj.layout.enclosures[ri].items.iter().filter_map(move |it| placed(prj, poses, ri, it))).collect()
}

/// Puts a socket outlet, switch or light (part `part`) into room `ri` at point `at` of
/// the plant (x, z, mm): a light made for the ceiling on the ceiling there, anything
/// else on the wall nearest to the point (within [`WALL_REACH`]) at its mounting
/// height. Gives the item's id.
pub fn place(prj: &mut Project, ri: usize, part: &str, at: [f32; 2]) -> Result<u64, String> {
    let e = prj.layout.enclosures.get(ri).ok_or_else(|| tr_fmt!("No enclosure {}", ri))?;
    if !e.is_room() {
        return Err(tr_fmt!("{} is not a room", e.name));
    }
    let pd = panel::part_by_number(prj, part).cloned().ok_or_else(|| tr_fmt!("Unknown device or part: {}", part))?;
    let poses = panel::site_poses(prj);
    let local = poses[ri].local([at[0], 0.0, at[1]]);
    let q = [local[0], local[2]];
    let [w, h, _] = pd.size().unwrap_or([80.0, 80.0, 20.0]);
    let (face, point) = if on_ceiling(&pd) {
        let (lo, hi) = panel::bounds(&panel::outline(e));
        if !panel::inside_outline(&panel::outline(e), q) {
            return Err(tr("The point is not inside the room").to_string());
        }
        let x = (q[0] - lo[0] - w / 2.0).clamp(0.0, (hi[0] - lo[0] - w).max(0.0));
        let z = (q[1] - lo[1] - h / 2.0).clamp(0.0, (hi[1] - lo[1] - h).max(0.0));
        (Face::Roof, p(x, z))
    } else {
        // The nearest wall (or side of an inner wall) and the place along it.
        let faces: Vec<Face> = panel::room_faces(e).into_iter().filter(|f| matches!(f, Face::Wall(_) | Face::Inner(..))).collect();
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
    if let Some(height) = panel::tech_number(&pd, "Mounting height") {
        let e = prj.layout.enclosures[ri].clone();
        if let Some(it) = prj.layout.enclosures[ri].items.iter_mut().find(|i| i.id == id) {
            panel::at_mounting_height(&e, it, height);
        }
    }
    Ok(id)
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

/// The enclosure a circuit's cable starts at: its board, or the one its protective
/// device is placed in.
pub fn board_of(prj: &Project, c: &FinalCircuit) -> Option<usize> {
    let layout = &prj.layout;
    if c.board != 0 {
        return layout.enclosures.iter().position(|e| e.id == c.board);
    }
    if c.breaker.is_empty() {
        return None;
    }
    layout.find_device(&c.breaker).map(|(ei, _)| ei).filter(|&ei| !layout.enclosures[ei].is_room())
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

/// A final circuit worked out: its cable run (from the board through its items), its
/// length (m), current (A), voltage (V), voltage drop (V, %) and the protective device's
/// rated current and the cable's current-carrying capacity (A).
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub circuit: u64,
    pub points: Vec<RunPoint>,
    /// The items found (ids), in order.
    pub items: Vec<u64>,
    /// Whether the run starts at a distribution board.
    pub from_board: bool,
    pub length: f32,
    pub rated: Option<f32>,
    pub current: Option<f32>,
    pub voltage: f32,
    pub drop: Option<(f32, f32)>,
    pub ampacity: Option<f32>,
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
    let du = k * current * length_m * prj.electrical.conditions.rho_hot() * cos_phi / mm2.max(1e-3);
    (du, du / u.max(1.0) * 100.0)
}

/// The run of one circuit.
pub fn run(prj: &Project, poses: &[SitePose], all: &[Placed], c: &FinalCircuit) -> Run {
    let mut points = Vec::new();
    let board = board_of(prj, c);
    if let Some(ei) = board {
        points.push(board_point(prj, poses, ei));
    }
    let mut found = Vec::new();
    for id in &c.items {
        if let Some(p) = all.iter().find(|p| p.item == *id) {
            points.push(RunPoint { at: p.at, height: p.height, floor: p.floor, zone: (p.ceiling - ZONE).max(0.0) });
            found.push(*id);
        }
    }
    let length = points.windows(2).map(|w| leg(&w[0], &w[1])).sum::<f32>() / 1000.0 + BOX_RESERVE * found.len() as f32;
    let rated = crate::protection::of_device(prj, &c.breaker).map(|p| p.rated).filter(|_| !c.breaker.is_empty());
    let u = voltage(prj, c.three_phase);
    let cos_phi = if c.cos_phi > 0.0 { c.cos_phi.min(1.0) } else { 1.0 };
    let current = if c.load > 0.0 { Some(c.load / (u * cos_phi * if c.three_phase { 3f32.sqrt() } else { 1.0 })) } else { rated };
    let mm2 = cross_section(c);
    let drop = current.filter(|_| board.is_some() && !found.is_empty()).map(|i| drop(prj, i, length, mm2, cos_phi, u, c.three_phase));
    Run { circuit: c.id, points, items: found, from_board: board.is_some(), length, rated, current, voltage: u, drop, ampacity: crate::sizing::capacity(prj, mm2) }
}

/// Cross-section of a circuit's conductors: given, or 1.5 mm².
pub fn cross_section(c: &FinalCircuit) -> f32 {
    if c.cross_section > 0.0 {
        c.cross_section
    } else {
        1.5
    }
}

/// The runs of all circuits.
pub fn runs(prj: &Project) -> Vec<Run> {
    let poses = panel::site_poses(prj);
    let all = items(prj, &poses);
    prj.layout.circuits.iter().map(|c| run(prj, &poses, &all, c)).collect()
}

/// A protective device of the mains for final circuits: its tag, rated current,
/// characteristic, phases, the residual current device protecting it (mA), whether it
/// is spare, and the final circuit using it.
#[derive(Clone, Debug, PartialEq)]
pub struct Breaker {
    pub tag: String,
    pub rated: f32,
    pub characteristic: String,
    pub phases: usize,
    pub residual: Option<f32>,
    pub spare: bool,
    pub circuit: Option<String>,
}

/// The protective devices of the mains (in schematic order) for final circuits (up to
/// 32 A).
pub fn breakers(prj: &Project, nl: &Netlist) -> Vec<Breaker> {
    electrical::circuits(prj, nl)
        .into_iter()
        .filter(|c| c.is_final())
        .map(|c| {
            let circuit = prj.layout.circuits.iter().find(|f| f.breaker == c.device).map(|f| f.name.clone());
            // Spare: no other device on the nets behind it.
            let fed = c.nets.iter().any(|&n| {
                nl.nets.get(n).is_some_and(|net| net.pins.iter().any(|pr| prj.pages.get(pr.page).and_then(|pg| pg.symbols.get(pr.sym)).is_some_and(|s| !s.tag.is_empty() && s.tag != c.device)))
            });
            Breaker {
                spare: circuit.is_none() && !fed,
                characteristic: c.protection.characteristic.title().to_string(),
                tag: c.device,
                rated: c.protection.rated,
                phases: c.phases.len(),
                residual: c.residual,
                circuit,
            }
        })
        .collect()
}

/// The check: circuits without protective device or board, unknown or shared
/// protective devices, cables thinner than their protective device asks, voltage
/// drops above the limit, items gone; items of the installation in no circuit (info).
pub fn problems(prj: &Project, nl: &Netlist) -> Vec<Issue> {
    let mut out = Vec::new();
    if prj.layout.circuits.is_empty() {
        return out;
    }
    let known: Vec<Breaker> = breakers(prj, nl);
    let limit = crate::rules::limits(prj).voltage_drop;
    let runs = runs(prj);
    for (c, r) in prj.layout.circuits.iter().zip(&runs) {
        let name = &c.name;
        if c.breaker.is_empty() {
            out.push(crate::issue!(Severity::Warning, None, "Circuit {} has no protective device", name));
        } else if !known.iter().any(|b| b.tag == c.breaker) && crate::protection::of_device(prj, &c.breaker).is_none() {
            out.push(crate::issue!(Severity::Error, None, "Circuit {}: no protective device {} in the schematic", name, c.breaker));
        } else if prj.layout.circuits.iter().filter(|o| o.breaker == c.breaker).count() > 1 {
            out.push(crate::issue!(Severity::Warning, None, "Circuit {}: protective device {} feeds other circuits too", name, c.breaker));
        }
        if !c.breaker.is_empty() && !r.from_board {
            out.push(crate::issue!(Severity::Warning, None, "Circuit {}: protective device {} is in no distribution board of the layout", name, c.breaker));
        }
        if r.items.len() < c.items.len() {
            out.push(crate::issue!(Severity::Warning, None, "Circuit {}: {} of its items are gone", name, c.items.len() - r.items.len()));
        }
        if let (Some(rated), Some(cap)) = (r.rated, r.ampacity) {
            if cap < rated {
                out.push(crate::issue!(Severity::Error, None, "Circuit {}: {} mm² carry {} A, less than the {} A of its protective device", name, cross_section(c), format!("{cap:.0}"), format!("{rated:.0}")));
            }
        }
        if let Some((_, pct)) = r.drop {
            if pct > limit {
                out.push(crate::issue!(Severity::Warning, None, "Circuit {}: voltage drop {} % over the limit of {} % ({} m)", name, format!("{pct:.2}"), format!("{limit:.1}"), format!("{:.1}", r.length)));
            }
        }
    }
    let poses = panel::site_poses(prj);
    let used: std::collections::HashSet<u64> = prj.layout.circuits.iter().flat_map(|c| c.items.iter().copied()).collect();
    let free: Vec<Placed> = items(prj, &poses).into_iter().filter(|p| p.kind != Kind::Switch && !used.contains(&p.item)).collect();
    if !free.is_empty() {
        out.push(crate::issue!(Severity::Info, None, "{} socket outlets and lights are in no circuit", free.len()));
    }
    out
}

/// Adds a final circuit (single-phase, 1.5 mm² unless three-phase) and gives its id.
pub fn add_circuit(prj: &mut Project, name: &str, breaker: &str, three_phase: bool) -> Result<u64, String> {
    let name = if name.trim().is_empty() { free_name(prj) } else { name.trim().to_string() };
    if prj.layout.circuits.iter().any(|c| c.name == name) {
        return Err(tr_fmt!("Circuit {} exists already", name));
    }
    let id = prj.new_id();
    prj.layout.circuits.push(FinalCircuit {
        id,
        name,
        breaker: breaker.trim().to_string(),
        cable_type: if three_phase { DEFAULT_CABLE_3P } else { DEFAULT_CABLE }.into(),
        cross_section: if three_phase { 2.5 } else { 1.5 },
        three_phase,
        cos_phi: 1.0,
        ..Default::default()
    });
    Ok(id)
}

/// A name for a new circuit: "Circuit n".
pub fn free_name(prj: &Project) -> String {
    (1..).map(|n| tr_fmt!("Circuit {}", n)).find(|n| !prj.layout.circuits.iter().any(|c| &c.name == n)).unwrap_or_default()
}

/// The circuit of an item (its index), if it is in one.
pub fn circuit_of(prj: &Project, item: u64) -> Option<usize> {
    prj.layout.circuits.iter().position(|c| c.items.contains(&item))
}

/// Puts item `item` into circuit `ci` at its end (out of the circuit it was in).
pub fn connect(prj: &mut Project, ci: usize, item: u64) -> Result<(), String> {
    let poses = panel::site_poses(prj);
    if !items(prj, &poses).iter().any(|p| p.item == item) {
        return Err(tr_fmt!("No socket outlet, switch or light {}", item));
    }
    if ci >= prj.layout.circuits.len() {
        return Err(tr("No such circuit").to_string());
    }
    for c in &mut prj.layout.circuits {
        c.items.retain(|&i| i != item);
    }
    prj.layout.circuits[ci].items.push(item);
    Ok(())
}

/// Writes each circuit's cable into the cable list: its type, cross-section and length
/// (rounded up to the half metre); a circuit without a cable tag gets a new cable
/// `-W<n>`. Gives how many cables were written.
pub fn write_cables(prj: &mut Project) -> usize {
    let runs = runs(prj);
    let mut n = 0;
    for (ci, r) in runs.iter().enumerate() {
        if r.items.is_empty() {
            continue;
        }
        let c = prj.layout.circuits[ci].clone();
        let tag = if c.cable.is_empty() {
            let tag = (1..).map(|k| format!("-W{k}")).find(|t| !prj.cables.iter().any(|x| &x.tag == t) && !prj.layout.circuits.iter().any(|x| &x.cable == t)).unwrap_or_default();
            prj.layout.circuits[ci].cable = tag.clone();
            tag
        } else {
            c.cable.clone()
        };
        let pos = match prj.cables.iter().position(|x| x.tag == tag) {
            Some(k) => k,
            None => {
                prj.cables.push(crate::model::Cable { tag: tag.clone(), ..Default::default() });
                prj.cables.len() - 1
            }
        };
        let cable = &mut prj.cables[pos];
        cable.length = (r.length * 2.0).ceil() / 2.0;
        cable.cable_type = c.cable_type.clone();
        cable.cross_section = format!("{} mm²", cross_section(&c));
        cable.cores = if c.three_phase { "5".into() } else { "3".into() };
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests;
