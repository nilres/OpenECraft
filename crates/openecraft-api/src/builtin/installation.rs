//! Installation commands: socket outlets, switches and lights placed in the rooms by a
//! point of the floor plan, the final circuits feeding them from the protective
//! devices of a distribution board, their cable lengths and voltage drops, spare
//! protective devices ([`openecraft_core::installation`]).

use crate::params::{optional, param, Args, Kind};
use crate::{Info, Plugin, Registry};
use super::{add_commands, info, Spec};
use openecraft_core::installation as inst;
use openecraft_core::model::Project;
use openecraft_core::netlist::Netlist;
use openecraft_core::panel;
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde_json::{json, Value};

pub struct Installation;

impl Plugin for Installation {
    fn info(&self) -> Info {
        info("installation", tr_const!("Installation"), tr_const!("Socket outlets, switches and lights in the rooms, the final circuits from the protective devices of the distribution boards with their cable lengths and voltage drops"))
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        add_commands(reg, COMMANDS)
    }
}

const CIRCUIT: &str = tr_const!("Circuit: its name");
const BREAKER: &str = tr_const!("Device tag of its protective device (circuit breaker, RCBO, fuse) in the schematic");

pub static COMMANDS: &[Spec] = &[
    Spec {
        name: "place_installation",
        help: tr_const!("Puts a socket outlet, switch or light into a room at a point of the plant: a ceiling light on the ceiling there, anything else on the nearest wall at its mounting height"),
        params: &[
            param("room", Kind::Str, tr_const!("Room (location, e.g. +ROOM1)")),
            param("part", Kind::Str, tr_const!("Part number, e.g. GEN.SOCKET-F-UP, GEN.SW-ONOFF-UP, GEN.LAMP-CEILING-LED")),
            param("x", Kind::Num, tr_const!("x in the plant in mm")),
            param("z", Kind::Num, tr_const!("z in the plant (towards the front) in mm")),
            optional("circuit", Kind::Str, tr_const!("Circuit to add it to (its name)")),
        ],
        run: place_installation,
    },
    Spec {
        name: "list_breakers",
        help: tr_const!("Lists the protective devices of the mains for final circuits (up to 32 A): rated current, characteristic, phases, residual current protection, whether spare and the circuit using them"),
        params: &[optional("spare", Kind::Bool, tr_const!("Only the spare ones (feeding nothing yet)"))],
        run: list_breakers,
    },
    Spec {
        name: "add_circuit",
        help: tr_const!("Adds a final circuit: a protective device and the socket outlets, switches and lights its cable runs to"),
        params: &[
            optional("name", Kind::Str, tr_const!("Name (default Circuit 1, Circuit 2, …)")),
            optional("breaker", Kind::Str, BREAKER),
            optional("three_phase", Kind::Bool, tr_const!("Fed from three phases (five-core cable, 2.5 mm²)")),
            optional("items", Kind::Ids, tr_const!("Ids of its socket outlets, switches and lights in the order the cable runs to them")),
        ],
        run: add_circuit,
    },
    Spec {
        name: "set_circuit",
        help: tr_const!("Changes a final circuit: its name, protective device, distribution board, cable, load, items (added at its end or removed), or removes it"),
        params: &[
            param("circuit", Kind::Str, CIRCUIT),
            optional("name", Kind::Str, tr_const!("New name")),
            optional("breaker", Kind::Str, BREAKER),
            optional("board", Kind::Str, tr_const!("Distribution board its cable starts at (enclosure location; empty: where its protective device is placed)")),
            optional("cable_type", Kind::Str, tr_const!("Type of its cable, e.g. NYM-J 3x1.5")),
            optional("cross_section", Kind::Num, tr_const!("Cross-section of its conductors in mm²")),
            optional("cable", Kind::Str, tr_const!("Tag of its cable in the cable list")),
            optional("three_phase", Kind::Bool, tr_const!("Fed from three phases")),
            optional("load", Kind::Num, tr_const!("Load for the voltage drop in W (0: the rated current of its protective device)")),
            optional("cos_phi", Kind::Num, tr_const!("Power factor of the load")),
            optional("add", Kind::Ids, tr_const!("Ids of items to add at its end")),
            optional("remove", Kind::Ids, tr_const!("Ids of items to take out")),
            optional("remove_circuit", Kind::Bool, tr_const!("Remove the circuit (its items stay)")),
        ],
        run: set_circuit,
    },
    Spec {
        name: "list_circuits",
        help: tr_const!("Lists the final circuits with their items, cable lengths (along the installation zones), currents and voltage drops"),
        params: &[optional("circuit", Kind::Str, CIRCUIT)],
        run: list_circuits,
    },
    Spec {
        name: "write_circuit_cables",
        help: tr_const!("Writes the cable of every final circuit into the cable list: type, cross-section and length (a new cable for a circuit without one)"),
        params: &[],
        run: write_circuit_cables,
    },
];

fn circuit_arg(prj: &Project, name: &str) -> Result<usize, String> {
    prj.layout.circuits.iter().position(|c| c.name == name.trim()).ok_or_else(|| tr_fmt!("No circuit named {}", name))
}

fn circuit_json(prj: &Project, ci: usize, run: &inst::Run, all: &[inst::Placed]) -> Value {
    let c = &prj.layout.circuits[ci];
    let r2 = |v: f32| (v * 100.0).round() / 100.0;
    let items: Vec<Value> = c
        .items
        .iter()
        .map(|id| match all.iter().find(|p| p.item == *id) {
            Some(p) => json!({ "id": id, "room": prj.layout.enclosures[p.room].name, "part": p.part, "kind": format!("{:?}", p.kind).to_lowercase() }),
            None => json!({ "id": id, "gone": true }),
        })
        .collect();
    let board = inst::board_of(prj, c).map(|ei| prj.layout.enclosures[ei].name.clone());
    json!({
        "name": c.name, "breaker": c.breaker, "board": board, "cable": c.cable, "cable_type": c.cable_type, "cross_section": inst::cross_section(c),
        "three_phase": c.three_phase, "load": c.load, "items": items,
        "length": r2(run.length), "rated": run.rated, "current": run.current.map(r2), "voltage": r2(run.voltage),
        "drop_v": run.drop.map(|d| r2(d.0)), "drop_percent": run.drop.map(|d| r2(d.1)), "ampacity": run.ampacity,
    })
}

fn circuits_json(prj: &Project, only: Option<usize>) -> Value {
    let poses = panel::site_poses(prj);
    let all = inst::items(prj, &poses);
    let runs = inst::runs(prj);
    Value::Array(runs.iter().enumerate().filter(|(ci, _)| only.is_none_or(|o| o == *ci)).map(|(ci, r)| circuit_json(prj, ci, r, &all)).collect())
}

fn place_installation(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let room = a.req("room").trim();
    let ri = prj.layout.enclosures.iter().position(|e| e.is_room() && e.name == room).ok_or_else(|| tr_fmt!("No room named {}", room))?;
    let ci = a.str("circuit").map(|c| circuit_arg(prj, c)).transpose()?;
    let id = inst::place(prj, ri, a.req("part"), [a.num("x").unwrap_or(0.0), a.num("z").unwrap_or(0.0)])?;
    if let Some(ci) = ci {
        inst::connect(prj, ci, id)?;
    }
    let it = prj.layout.enclosures[ri].item(id).cloned().unwrap_or_default();
    Ok(json!({ "id": id, "room": room, "face": panel::room_face_key(&prj.layout.enclosures[ri], it.face), "x": it.x, "y": it.y }))
}

fn list_breakers(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let nl = Netlist::build(prj, false);
    let list = inst::breakers(prj, &nl);
    Ok(Value::Array(
        list.into_iter()
            .filter(|b| !a.flag("spare") || b.spare)
            .map(|b| json!({ "breaker": b.tag, "rated": b.rated, "characteristic": b.characteristic, "phases": b.phases, "residual_ma": b.residual, "spare": b.spare, "circuit": b.circuit }))
            .collect(),
    ))
}

fn add_items(prj: &mut Project, ci: usize, ids: &[u64]) -> Result<(), String> {
    for id in ids {
        inst::connect(prj, ci, *id)?;
    }
    Ok(())
}

fn add_circuit(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let id = inst::add_circuit(prj, a.str("name").unwrap_or(""), a.str("breaker").unwrap_or(""), a.flag("three_phase"))?;
    let ci = prj.layout.circuits.iter().position(|c| c.id == id).unwrap_or(0);
    add_items(prj, ci, &a.ids("items"))?;
    Ok(circuits_json(prj, Some(ci)))
}

fn set_circuit(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let ci = circuit_arg(prj, a.req("circuit"))?;
    if a.flag("remove_circuit") {
        prj.layout.circuits.remove(ci);
        return Ok(circuits_json(prj, None));
    }
    if let Some(n) = a.str("name").map(str::trim).filter(|n| !n.is_empty()) {
        if prj.layout.circuits.iter().enumerate().any(|(k, c)| k != ci && c.name == n) {
            return Err(tr_fmt!("Circuit {} exists already", n));
        }
        prj.layout.circuits[ci].name = n.into();
    }
    let board = match a.str("board").map(str::trim) {
        Some("") => Some(0),
        Some(b) => Some(prj.layout.enclosures.iter().find(|e| !e.is_room() && e.name == b).map(|e| e.id).ok_or_else(|| tr_fmt!("No enclosure named {}", b))?),
        None => None,
    };
    if let Some(v) = a.num("cross_section") {
        if v <= 0.0 {
            return Err(tr("The cross-section must be greater than 0").to_string());
        }
    }
    let c = &mut prj.layout.circuits[ci];
    if let Some(v) = a.str("breaker") {
        c.breaker = v.trim().into();
    }
    if let Some(v) = board {
        c.board = v;
    }
    if let Some(v) = a.str("cable_type") {
        c.cable_type = v.trim().into();
    }
    if let Some(v) = a.num("cross_section") {
        c.cross_section = v;
    }
    if let Some(v) = a.str("cable") {
        c.cable = v.trim().into();
    }
    if a.value("three_phase").is_some() {
        c.three_phase = a.flag("three_phase");
    }
    if let Some(v) = a.num("load") {
        c.load = v.max(0.0);
    }
    if let Some(v) = a.num("cos_phi") {
        c.cos_phi = v.clamp(0.1, 1.0);
    }
    let remove = a.ids("remove");
    c.items.retain(|i| !remove.contains(i));
    add_items(prj, ci, &a.ids("add"))?;
    Ok(circuits_json(prj, Some(ci)))
}

fn list_circuits(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let only = a.str("circuit").map(|c| circuit_arg(prj, c)).transpose()?;
    Ok(circuits_json(prj, only))
}

fn write_circuit_cables(prj: &mut Project, _: &Args) -> Result<Value, String> {
    let n = inst::write_cables(prj);
    Ok(json!({ "cables": n, "circuits": circuits_json(prj, None) }))
}

#[cfg(test)]
mod tests;
