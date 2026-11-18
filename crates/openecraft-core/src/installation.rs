//! The electrical installation of a building: socket outlets, switches, lights and
//! heating elements in the rooms ([`kind_of`]), placed on the floor plan ([`place`]),
//! and the final circuits that feed them ([`FinalCircuit`]): a protective device of a
//! distribution board in the schematic, a cable and the items it runs to one after the
//! other.
//!
//! **Circuits of the schematic** ([`schematic_circuits`]): the devices drawn behind a
//! protective device of the mains (through switches, contacts and terminals) that are
//! socket outlets, switches, lights or heating elements are placed on the floor plan
//! ([`place_device`]) or taken from what was placed there ([`link_device`]); each goes
//! into the final circuit of its protective device, made when there is none.
//!
//! **Cable length** ([`runs`]): the cable leaves the distribution board at its top and
//! runs to each item in turn in the horizontal installation zone under the ceiling
//! (DIN 18015-3: [`ZONE`] below it), straight along x and z of the plant (along the
//! walls, not across rooms), up and down in the vertical zones above and below the
//! items, up or down between levels; each item adds [`BOX_RESERVE`] for its box. An
//! estimate for planning before the cable is laid.
//!
//! **Voltage drop** from the supply to the last item: ΔU = k · I · L · (ρ / A · cos φ +
//! X' · sin φ) per cable with k = 2 (single-phase) or √3 (three-phase), ρ of copper at
//! its operating temperature ([`crate::electrical::Conditions`]), X' the reactance per
//! metre ([`crate::shortcircuit::CABLE_REACTANCE`]). The final circuit's cable carries
//! the rated current of its protective device (the highest current it lets the cable
//! carry) or the load given; the feeder cables from the supply to the distribution
//! board ([`crate::shortcircuit::feeder_cables`]) the design current of the protective
//! device upstream (load calculation, [`crate::loads`]) or its rated current. The sum
//! is checked against the limit of the rule sets ([`crate::rules::limits`]).
//!
//! **Devices** ([`link_devices`], [`overview`]): the items of the circuits get device
//! tags (socket outlets `XD`, switches `SF`, lights `EA` in their room) and a generated
//! overview page shows each circuit as a single-line diagram: its protective device,
//! its cable and its items with their installation plan symbols. A circuit without a
//! protective device gets a planned one ([`new_breaker`]), which the single-line
//! diagram turns into the schematic ([`crate::singleline::to_multiline`]).
//!
//! **Protective devices** ([`breakers`]): those of the mains in the schematic
//! ([`crate::electrical::circuits`]); a spare one feeds nothing yet: no other device on
//! the nets behind it and no final circuit.

use crate::electrical;
use crate::geom::{p, P, GRID};
use crate::shortcircuit::FeederCable;
use std::collections::HashMap;
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
/// Power factor of a new circuit.
pub const DEFAULT_COS_PHI: f32 = 0.95;

/// What an item of a room is for the installation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Socket,
    Switch,
    Light,
    Heater,
}

impl Kind {
    pub const ALL: [Kind; 4] = [Kind::Socket, Kind::Switch, Kind::Light, Kind::Heater];

    pub fn title(self) -> &'static str {
        match self {
            Kind::Socket => tr("Socket outlet"),
            Kind::Switch => tr("Switch"),
            Kind::Light => tr("Luminaire"),
            Kind::Heater => tr("Heating element"),
        }
    }

    /// Key of the kind in the commands' answers.
    pub fn key(self) -> &'static str {
        match self {
            Kind::Socket => "socket",
            Kind::Switch => "switch",
            Kind::Light => "light",
            Kind::Heater => "heater",
        }
    }

    /// The generic part a device of the schematic without a part is placed with.
    pub fn default_part(self) -> &'static str {
        match self {
            Kind::Socket => "GEN.SOCKET-F-UP",
            Kind::Switch => "GEN.SW-ONOFF-UP",
            Kind::Light => "GEN.LAMP-CEILING-LED",
            Kind::Heater => "GEN.HEATER-PANEL-1000",
        }
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

/// The parts of the bundled manufacturer catalogs that are parts of the installation
/// (read once).
pub fn bundled_parts() -> &'static [Part] {
    static PARTS: std::sync::OnceLock<Vec<Part>> = std::sync::OnceLock::new();
    PARTS.get_or_init(|| crate::parts::bundled::all().unwrap_or_default().into_iter().flat_map(|b| b.parts).filter(|p| kind_of(p).is_some()).collect())
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
/// height. Gives the item's id.
pub fn place(prj: &mut Project, ri: usize, part: &str, at: [f32; 2]) -> Result<u64, String> {
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
    /// Voltage drop from the supply to the distribution board, %.
    pub upstream: Option<f32>,
    pub ampacity: Option<f32>,
}

impl Run {
    /// Voltage drop from the supply to the last item, %.
    pub fn total(&self) -> Option<f32> {
        self.drop.map(|d| d.1 + self.upstream.unwrap_or(0.0))
    }
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
    pub fn of(prj: &Project, nl: &Netlist, conns: &[crate::netlist::Connection]) -> Supply {
        let currents = crate::loads::calculate(prj, nl).feeders.into_iter().filter(|f| f.current > 0.0).map(|f| (f.device, f.current)).collect();
        Supply { feeders: crate::shortcircuit::feeder_cables(prj, nl, conns), currents }
    }

    /// Of the project as it is (netlist and connections built).
    pub fn build(prj: &Project) -> Supply {
        let (nl, open) = (Netlist::build(prj, true), Netlist::build(prj, false));
        let conns = crate::netlist::connections(prj, &nl, &open);
        Supply::of(prj, &nl, &conns)
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
fn feeder_of<'a>(prj: &Project, supply: &'a Supply, c: &FinalCircuit, board: Option<usize>) -> Option<&'a Vec<FeederCable>> {
    supply.feeders.get(&c.breaker).filter(|f| !f.is_empty()).or_else(|| {
        let board = board?;
        let mut tags: Vec<&String> = supply.feeders.keys().collect();
        tags.sort();
        tags.into_iter().find(|t| prj.layout.find_device(t).is_some_and(|(ei, _)| ei == board)).and_then(|t| supply.feeders.get(t)).filter(|f| !f.is_empty())
    })
}

/// Voltage drop of the feeder cables up to a circuit's board, %.
fn upstream(prj: &Project, supply: &Supply, c: &FinalCircuit, board: Option<usize>, cos_phi: f32) -> Option<f32> {
    let chain = feeder_of(prj, supply, c, board)?;
    let mut total = 0.0;
    for cable in chain {
        let i = supply.currents.get(&cable.device).copied().or_else(|| crate::protection::of_device(prj, &cable.device).map(|p| p.rated))?;
        let three = cable.phases >= 3;
        total += drop(prj, i, cable.length, cable.mm2, cos_phi, voltage(prj, three), three).1;
    }
    Some(total)
}

/// The run of one circuit.
pub fn run(prj: &Project, poses: &[SitePose], all: &[Placed], supply: &Supply, c: &FinalCircuit) -> Run {
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
    let upstream = upstream(prj, supply, c, board, cos_phi);
    Run { circuit: c.id, points, items: found, from_board: board.is_some(), length, rated, current, voltage: u, drop, upstream, ampacity: crate::sizing::capacity(prj, mm2) }
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
pub fn runs(prj: &Project, supply: &Supply) -> Vec<Run> {
    let poses = panel::site_poses(prj);
    let all = items(prj, &poses);
    prj.layout.circuits.iter().map(|c| run(prj, &poses, &all, supply, c)).collect()
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
pub fn problems(prj: &Project, nl: &Netlist, conns: &[crate::netlist::Connection]) -> Vec<Issue> {
    let mut out = Vec::new();
    if prj.layout.circuits.is_empty() {
        return out;
    }
    let known: Vec<Breaker> = breakers(prj, nl);
    let limit = crate::rules::limits(prj).voltage_drop;
    let runs = runs(prj, &Supply::of(prj, nl, conns));
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
        if let Some(pct) = r.total() {
            if pct > limit {
                out.push(crate::issue!(Severity::Warning, None, "Circuit {}: voltage drop {} % over the limit of {} % ({} m)", name, format!("{pct:.2}"), format!("{limit:.1}"), format!("{:.1}", r.length)));
            }
        }
        if !c.breaker.is_empty() && !known.iter().any(|b| b.tag == c.breaker) && crate::protection::of_device(prj, &c.breaker).is_some() {
            out.push(crate::issue!(Severity::Info, None, "Circuit {}: protective device {} is planned only: generate it into the schematic from the single-line diagram and connect it to the supply", name, c.breaker));
        }
    }
    let poses = panel::site_poses(prj);
    let used: std::collections::HashSet<u64> = prj.layout.circuits.iter().flat_map(|c| c.items.iter().copied()).collect();
    let free: Vec<Placed> = items(prj, &poses).into_iter().filter(|p| p.kind != Kind::Switch && !used.contains(&p.item)).collect();
    if !free.is_empty() {
        out.push(crate::issue!(Severity::Info, None, "{} socket outlets, lights and heating elements are in no circuit", free.len()));
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
        cos_phi: DEFAULT_COS_PHI,
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

/// A socket outlet, switch, light or heating element of a circuit of the schematic: its
/// tag, kind, function text, its own part for walls or ceilings (none: placed with the
/// part of its kind, [`Kind::default_part`]) and where it is placed (room, item).
#[derive(Clone, Debug, PartialEq)]
pub struct SchematicDevice {
    pub tag: String,
    pub kind: Kind,
    pub text: String,
    pub part: Option<String>,
    pub placed: Option<(usize, u64)>,
}

/// A final circuit of the schematic (a protective device of the mains up to 32 A) with
/// the socket outlets, switches, lights and heating elements behind it, in schematic
/// order, and the final circuit of the installation using its protective device.
#[derive(Clone, Debug, PartialEq)]
pub struct SchematicCircuit {
    pub breaker: String,
    pub text: String,
    pub rated: f32,
    pub characteristic: String,
    pub three_phase: bool,
    pub devices: Vec<SchematicDevice>,
    pub circuit: Option<u64>,
}

impl SchematicCircuit {
    pub fn placed(&self) -> usize {
        self.devices.iter().filter(|d| d.placed.is_some()).count()
    }

    pub fn device(&self, tag: &str) -> Option<&SchematicDevice> {
        self.devices.iter().find(|d| d.tag == tag)
    }
}

/// The symbol passes the circuit on to the devices behind it: a switch, push button,
/// contact, terminal or junction box (all its pins one circuit).
fn switches_through(s: &crate::model::SymbolInst) -> bool {
    use crate::parts::FnKind as F;
    matches!(
        crate::parts::kind_of_symbol(&s.sym),
        Some(F::No | F::Nc | F::Changeover | F::PushbuttonNo | F::PushbuttonNc | F::SwitchNo | F::SwitchNc | F::Terminal | F::NoOnDelay | F::NoOffDelay | F::Sensor)
    ) || ["inst_switch", "inst_switch2", "inst_dimmer", "inst_push", "inst_junction", "inst_motion"].contains(&s.sym.as_str())
}

/// The devices on the nets behind protective device `c` of the schematic: through
/// pass-through devices (contacts of contactors, residual current devices),
/// switches, contacts and terminals, not through further protective devices or loads
/// and never into a potential of the supply. Tags in schematic order.
fn devices_behind(prj: &Project, nl: &Netlist, c: &electrical::Circuit) -> Vec<String> {
    use crate::model::PageKind;
    let mut todo: Vec<usize> = c.nets.iter().copied().collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut found: HashMap<String, (usize, usize)> = HashMap::new();
    while let Some(n) = todo.pop() {
        let Some(net) = nl.nets.get(n) else { continue };
        if !seen.insert(n) || (net.named && !c.nets.contains(&n) && electrical::is_supply_potential(prj, &net.name)) {
            continue;
        }
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
    tags.into_iter().map(|t| t.0).collect()
}

/// What a device of the schematic is for the installation, with its part: by its parts
/// (one for walls or ceilings), or, without parts, by its symbols (socket outlets,
/// heating elements, lamps, installation symbols, switches and contacts of an `S`
/// device). `None` for devices with other parts (devices of cabinets).
fn device_kind(prj: &Project, tag: &str) -> Option<(Kind, Option<String>)> {
    use crate::parts::FnKind as F;
    let refs = prj.devices.get(tag).map(|d| d.parts.clone()).unwrap_or_default();
    if !refs.is_empty() {
        return refs.iter().find_map(|r| panel::part_by_number(prj, &r.number).and_then(kind_of).map(|k| (k, Some(r.number.clone()))));
    }
    let letters = tag.rsplit('-').next().unwrap_or("");
    let kinds: Vec<Kind> = prj
        .pages
        .iter()
        .filter(|pg| pg.kind == crate::model::PageKind::Schematic)
        .flat_map(|pg| pg.symbols.iter())
        .filter(|s| s.tag == tag)
        .filter_map(|s| match (s.sym.as_str(), crate::parts::kind_of_symbol(&s.sym)) {
            ("inst_socket2", _) | (_, Some(F::SocketOutlet)) => Some(Kind::Socket),
            (_, Some(F::Heater)) => Some(Kind::Heater),
            ("lamp" | "inst_lamp" | "inst_luminaire", _) => Some(Kind::Light),
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

/// The final circuits of the schematic with socket outlets, switches, lights or heating
/// elements behind them (see [`SchematicCircuit`]), in schematic order.
pub fn schematic_circuits(prj: &Project, nl: &Netlist) -> Vec<SchematicCircuit> {
    let text_of = |tag: &str| prj.pages.iter().flat_map(|pg| pg.symbols.iter()).find(|s| s.tag == tag && !s.function_text.is_empty()).map(|s| s.function_text.clone()).unwrap_or_default();
    electrical::circuits(prj, nl)
        .into_iter()
        .filter(|c| c.is_final())
        .filter_map(|c| {
            let devices: Vec<SchematicDevice> = devices_behind(prj, nl, &c)
                .into_iter()
                .filter(|tag| prj.layout.find_device(tag).is_none_or(|(ei, _)| prj.layout.enclosures[ei].is_room()))
                .filter_map(|tag| {
                    let (kind, part) = device_kind(prj, &tag)?;
                    Some(SchematicDevice { text: text_of(&tag), kind, part, placed: placed_in_room(prj, &tag), tag })
                })
                .collect();
            if devices.is_empty() {
                return None;
            }
            Some(SchematicCircuit {
                text: text_of(&c.device),
                rated: c.protection.rated,
                characteristic: c.protection.characteristic.title().to_string(),
                three_phase: c.phases.len() >= 3,
                circuit: prj.layout.circuits.iter().find(|f| f.breaker == c.device).map(|f| f.id),
                breaker: c.device,
                devices,
            })
        })
        .collect()
}

/// The schematic circuit device `tag` is in.
pub fn schematic_circuit_of<'a>(list: &'a [SchematicCircuit], tag: &str) -> Option<&'a SchematicCircuit> {
    list.iter().find(|c| c.device(tag).is_some())
}

/// The final circuit of the installation for schematic circuit `sc` (its index): the
/// one using its protective device, or a new one named after the protective device's
/// function text (or its tag).
pub fn circuit_for(prj: &mut Project, sc: &SchematicCircuit) -> Result<usize, String> {
    if let Some(ci) = prj.layout.circuits.iter().position(|c| c.breaker == sc.breaker) {
        return Ok(ci);
    }
    let name = [sc.text.trim(), sc.breaker.as_str()].into_iter().find(|n| !n.is_empty() && !prj.layout.circuits.iter().any(|c| c.name == *n)).unwrap_or("").to_string();
    let id = add_circuit(prj, &name, &sc.breaker, sc.three_phase)?;
    Ok(prj.layout.circuits.iter().position(|c| c.id == id).unwrap_or(0))
}

/// Device `tag` of circuit `sc` became item `id`: the device gets the item's part when
/// it has none, an earlier place of it goes (its place in a circuit taken by the new
/// one) and the item is in the final circuit of the protective device. Gives the index
/// of that circuit.
fn adopt(prj: &mut Project, sc: &SchematicCircuit, tag: &str, id: u64, old: Option<(usize, u64)>) -> Result<usize, String> {
    let ri = prj.layout.enclosures.iter().position(|e| e.item(id).is_some()).ok_or_else(|| tr_fmt!("No socket outlet, switch or light {}", id))?;
    let part = prj.layout.enclosures[ri].item(id).map(|it| it.part.clone()).unwrap_or_default();
    if let Some(it) = prj.layout.enclosures[ri].items.iter_mut().find(|i| i.id == id) {
        it.kind = ItemKind::Device;
        it.tag = tag.to_string();
    }
    if prj.devices.get(tag).is_none_or(|d| d.parts.is_empty()) {
        let description = panel::part_by_number(prj, &part).map(|x| x.description.clone()).unwrap_or_default();
        let data = prj.devices.entry(tag.to_string()).or_default();
        data.parts.push(crate::parts::PartRef { number: part, ..Default::default() });
        if data.description.is_empty() {
            data.description = description;
        }
    }
    if let Some((oi, old_id)) = old.filter(|o| o.1 != id) {
        prj.layout.enclosures[oi].items.retain(|i| i.id != old_id);
        for c in &mut prj.layout.circuits {
            for i in &mut c.items {
                if *i == old_id {
                    *i = id;
                }
            }
        }
    }
    let ci = circuit_for(prj, sc)?;
    if !prj.layout.circuits[ci].items.contains(&id) {
        for c in &mut prj.layout.circuits {
            c.items.retain(|&i| i != id);
        }
        prj.layout.circuits[ci].items.push(id);
    }
    Ok(ci)
}

/// Places device `tag` of schematic circuit `sc` into room `ri` at point `at` of the
/// plant like [`place`]: with its own part, or `part` (empty: the generic one of its
/// kind), which then becomes its part. A device placed before moves there. The item
/// goes into the final circuit of the circuit's protective device ([`circuit_for`]).
/// Gives the item's id and the index of the circuit.
pub fn place_device(prj: &mut Project, sc: &SchematicCircuit, tag: &str, ri: usize, part: &str, at: [f32; 2]) -> Result<(u64, usize), String> {
    let d = sc.device(tag).ok_or_else(|| tr_fmt!("{} is no socket outlet, switch, light or heating element of circuit {}", tag, sc.breaker))?;
    let number = match &d.part {
        Some(p) => p.clone(),
        None if !part.trim().is_empty() => {
            let pd = panel::part_by_number(prj, part.trim()).or_else(|| bundled_parts().iter().find(|p| p.number == part.trim())).ok_or_else(|| tr_fmt!("Unknown device or part: {}", part))?;
            if kind_of(pd) != Some(d.kind) {
                return Err(tr_fmt!("{} does not fit {}: choose a part of its kind", part, tag));
            }
            part.trim().to_string()
        }
        None => d.kind.default_part().to_string(),
    };
    let old = placed_in_room(prj, tag);
    let id = place(prj, ri, &number, at)?;
    let ci = adopt(prj, sc, tag, id, old)?;
    Ok((id, ci))
}

/// Makes item `item` (a socket outlet, switch, light or heating element placed
/// without a device tag, of the same kind) device `tag` of schematic circuit `sc`:
/// what was placed before becomes the device of the schematic. Gives the index of the
/// circuit it is in.
pub fn link_device(prj: &mut Project, sc: &SchematicCircuit, tag: &str, item: u64) -> Result<usize, String> {
    let d = sc.device(tag).ok_or_else(|| tr_fmt!("{} is no socket outlet, switch, light or heating element of circuit {}", tag, sc.breaker))?;
    let poses = panel::site_poses(prj);
    let p = items(prj, &poses).into_iter().find(|p| p.item == item).ok_or_else(|| tr_fmt!("No socket outlet, switch or light {}", item))?;
    let it = prj.layout.enclosures[p.room].item(item).cloned().unwrap_or_default();
    if !it.tag.is_empty() && it.tag != tag {
        return Err(tr_fmt!("{} is device {} already", item, it.tag));
    }
    if p.kind != d.kind || d.part.as_ref().is_some_and(|own| *own != it.part) {
        return Err(tr_fmt!("{} does not fit {}: choose a part of its kind", it.part, tag));
    }
    adopt(prj, sc, tag, item, placed_in_room(prj, tag))
}

/// Writes each circuit's cable into the cable list: its type, cross-section and length
/// (rounded up to the half metre); a circuit without a cable tag gets a new cable
/// `-W<n>`. Gives how many cables were written.
pub fn write_cables(prj: &mut Project) -> usize {
    let runs = runs(prj, &Supply::default());
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

/// Letter code of the device tags of a kind of item (IEC 81346-2).
pub fn code(kind: Kind) -> &'static str {
    match kind {
        Kind::Socket => "XD",
        Kind::Switch => "SF",
        Kind::Light => "EA",
        Kind::Heater => "EB",
    }
}

/// A device tag not used yet: `prefix` and the first free number.
fn free_tag(prj: &Project, prefix: &str) -> String {
    let used = |t: &str| prj.devices.contains_key(t) || prj.pages.iter().any(|pg| pg.symbols.iter().any(|s| s.tag == t)) || prj.layout.find_device(t).is_some();
    (1..).map(|n| format!("{prefix}{n}")).find(|t| !used(t)).unwrap_or_default()
}

/// Gives the items of the circuits without a device tag one (`<room>-XD1` for socket
/// outlets, `-SF1` for switches, `-EA1` for lights, `-EB1` for heating elements) with
/// their part. Gives how many.
pub fn link_devices(prj: &mut Project) -> usize {
    let poses = panel::site_poses(prj);
    let all = items(prj, &poses);
    let ids: Vec<u64> = prj.layout.circuits.iter().flat_map(|c| c.items.iter().copied()).collect();
    let mut n = 0;
    for id in ids {
        let Some(p) = all.iter().find(|p| p.item == id) else { continue };
        let e = &prj.layout.enclosures[p.room];
        if e.item(id).is_none_or(|it| !it.tag.is_empty()) {
            continue;
        }
        let tag = free_tag(prj, &format!("{}-{}", e.name, code(p.kind)));
        let description = panel::part_by_number(prj, &p.part).map(|x| x.description.clone()).unwrap_or_default();
        prj.devices.insert(tag.clone(), crate::model::DeviceData { parts: vec![crate::parts::PartRef { number: p.part.clone(), ..Default::default() }], description, ..Default::default() });
        if let Some(it) = prj.layout.enclosures[p.room].items.iter_mut().find(|i| i.id == id) {
            it.kind = ItemKind::Device;
            it.tag = tag;
        }
        n += 1;
    }
    n
}

/// Parts of planned protective devices: (part, rated current, three-phase).
pub const BREAKER_PARTS: [(&str, f32, bool); 4] = [("GEN.MCB-1P-B10", 10.0, false), ("GEN.MCB-1P-B16", 16.0, false), ("GEN.MCB-3P-B16", 16.0, true), ("GEN.MCB-4P-C32", 32.0, true)];

/// Gives circuit `ci` a new planned protective device: a miniature circuit breaker of
/// `rated` A (single- or three-phase like the circuit, [`BREAKER_PARTS`]), tag `-F<n>`,
/// starting in the board most other circuits start at; it is drawn in the single-line
/// diagram of the installation ([`overview`]). Gives its tag.
pub fn new_breaker(prj: &mut Project, ci: usize, rated: f32) -> Result<String, String> {
    let c = prj.layout.circuits.get(ci).ok_or_else(|| tr("No such circuit").to_string())?;
    let (part, _, _) = BREAKER_PARTS.iter().find(|b| b.1 == rated && b.2 == c.three_phase).ok_or_else(|| tr_fmt!("No circuit breaker of {} A", rated))?;
    let tag = free_tag(prj, "-F");
    let description = panel::part_by_number(prj, part).map(|x| x.description.clone()).unwrap_or_default();
    prj.devices.insert(tag.clone(), crate::model::DeviceData { parts: vec![crate::parts::PartRef { number: (*part).into(), ..Default::default() }], description, ..Default::default() });
    let mut boards: HashMap<usize, usize> = HashMap::new();
    for o in &prj.layout.circuits {
        if let Some(b) = board_of(prj, o) {
            *boards.entry(b).or_default() += 1;
        }
    }
    let board = boards.into_iter().max_by_key(|b| (b.1, std::cmp::Reverse(b.0))).map(|b| prj.layout.enclosures[b.0].id);
    let c = &mut prj.layout.circuits[ci];
    c.breaker = tag.clone();
    if c.board == 0 {
        c.board = board.unwrap_or(0);
    }
    Ok(tag)
}

/// Symbol of an item on the overview page.
fn symbol_of(kind: Kind) -> &'static str {
    match kind {
        Kind::Socket => "inst_socket2",
        Kind::Switch => "inst_switch",
        Kind::Light => "inst_lamp",
        Kind::Heater => "heater",
    }
}

/// Draws the circuits as single-line diagrams on overview pages, made again each time
/// (those made before are replaced): a column per circuit, its protective device at
/// the top (single-line circuit breaker), its cable down to its items (installation
/// plan symbols with their device tags and rooms). The items get their device tags
/// first ([`link_devices`]). Gives the indices of the pages.
pub fn overview(prj: &mut Project) -> Result<Vec<usize>, String> {
    if prj.layout.circuits.is_empty() {
        return Err(tr("No circuit yet").to_string());
    }
    link_devices(prj);
    let poses = panel::site_poses(prj);
    let all = items(prj, &poses);
    // The pages made before go; the new ones take the place of the first.
    let old = std::mem::take(&mut prj.layout.installation_pages);
    let mut at = prj.pages.len();
    for id in old {
        if let Some(k) = prj.pages.iter().position(|pg| pg.id == id) {
            at = at.min(k);
            prj.pages.remove(k);
        }
    }
    let at = at.min(prj.pages.len());
    let breaker = prj.symbol("sl_cb").ok_or_else(|| tr("Unknown symbol").to_string())?;
    let lower = (0..breaker.pins.len()).max_by(|a, b| breaker.pins[*a].pos.y.total_cmp(&breaker.pins[*b].pos.y)).unwrap_or(0);
    let (col_w, step, branch) = (9.0 * GRID, 5.0 * GRID, 3.0 * GRID);
    let mut pages = Vec::new();
    let mut page = usize::MAX;
    let mut x = f32::MAX;
    let mut border = crate::geom::R { min: p(0.0, 0.0), max: p(0.0, 0.0) };
    let circuits = prj.layout.circuits.clone();
    for c in &circuits {
        if page == usize::MAX || x + col_w > border.max.x - 2.0 * GRID {
            let id = prj.new_id();
            let k = at + pages.len();
            let mut pg = crate::model::Page::new(id, &crate::ops::next_page_name(prj, k), tr("Installation circuits"), crate::model::PageKind::Overview);
            pg.generated = true;
            page = crate::ops::insert_page(prj, k, pg);
            prj.layout.installation_pages.push(id);
            pages.push(page);
            border = crate::frame::of_page(prj, page).border();
            x = border.min.x + 4.0 * GRID;
        }
        let pos = p(x, border.min.y + 6.0 * GRID).snap(GRID);
        let bottom = border.max.y - 4.0 * GRID;
        // The protective device, or the start of the cable without one.
        let mut from = pos;
        if !c.breaker.is_empty() {
            let inst = new_symbol(prj, &breaker, pos, &c.breaker, &c.name);
            prj.pages[page].symbols.push(inst);
            from = breaker.pin_world(lower, pos, 0, false);
        }
        let count = c.items.len().max(1) as f32;
        let step = (((bottom - from.y) / count / GRID).floor() * GRID).clamp(2.0 * GRID, step);
        let mut last = from;
        for id in &c.items {
            let Some(item) = all.iter().find(|q| q.item == *id) else { continue };
            let Some(def) = prj.symbol(symbol_of(item.kind)) else { continue };
            let joint = p(from.x, last.y + step);
            let tag = prj.layout.enclosures[item.room].item(*id).map(|it| it.tag.clone()).unwrap_or_default();
            let room = prj.layout.enclosures[item.room].name.clone();
            // The symbol's connection point at the end of a branch to the right.
            let pin = def.pins.first().map_or(p(0.0, 0.0), |q| q.pos);
            let end = p(from.x + branch, joint.y);
            let inst = new_symbol(prj, &def, p(end.x - pin.x, end.y - pin.y), &tag, &room);
            prj.pages[page].symbols.push(inst);
            let cable = if last.same(from) { c.cable.clone() } else { String::new() };
            push_wire(prj, page, last, joint, &cable);
            push_wire(prj, page, joint, end, "");
            last = joint;
        }
        x += col_w;
    }
    Ok(pages)
}

fn new_symbol(prj: &mut Project, def: &crate::symbols::SymbolDef, pos: P, tag: &str, text: &str) -> crate::model::SymbolInst {
    crate::model::SymbolInst {
        id: prj.new_id(),
        sym: def.key.to_string(),
        pos,
        tag: tag.to_string(),
        function_text: text.to_string(),
        pins: def.pins.iter().map(|x| x.label.to_string()).collect(),
        ..Default::default()
    }
}

fn push_wire(prj: &mut Project, page: usize, a: P, b: P, cable: &str) {
    if a.same(b) {
        return;
    }
    let id = prj.new_id();
    prj.pages[page].wires.push(crate::model::Wire { id, a, b, cable: cable.to_string(), ..Default::default() });
}

#[cfg(test)]
mod tests;

