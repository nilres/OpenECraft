//! Installation commands: socket outlets, switches, lights and heating elements put
//! into the rooms for a protective device of the schematic (they go into the schematic
//! with a cable), devices of the schematic placed in the rooms, and the final circuits
//! read from the schematic with their cable lengths (3D routing) and voltage drops
//! ([`openecraft_core::installation`]).

use crate::builtin::common::num_in;
use crate::params::{optional, param, Args, Kind};
use crate::{Info, Plugin, Registry};
use super::{add_commands, info, Spec};
use openecraft_core::analysis::Analysis;
use openecraft_core::installation as inst;
use openecraft_core::model::Project;
use openecraft_core::netlist::Netlist;
use openecraft_core::panel;
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde_json::{json, Value};

pub struct Installation;

impl Plugin for Installation {
    fn info(&self) -> Info {
        info("installation", tr_const!("Installation"), tr_const!("Socket outlets, switches, lights and heating elements in the rooms and in the schematic, the final circuits of the schematic with their cable lengths and voltage drops"))
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        add_commands(reg, COMMANDS)
    }
}

const BREAKER: &str = tr_const!("Device tag of its protective device (circuit breaker, RCBO, fuse) in the schematic");
const ROOM: &str = tr_const!("Room (location, e.g. +ROOM1)");
const X: &str = tr_const!("x in the plant in mm");
const Z: &str = tr_const!("z in the plant (towards the front) in mm");

pub static COMMANDS: &[Spec] = &[
    Spec {
        name: "plan_symbols",
        help: tr_const!("Lists the symbols of installation plans (IEC 60617-11) a part can be shown with: key and name"),
        params: &[],
        run: plan_symbols,
    },
    Spec {
        name: "set_plan_symbol",
        help: tr_const!("Sets the symbol of a part on installation plans (a key of plan_symbols or another symbol of the library); a part of a catalog is copied into the project first"),
        params: &[
            param("part", Kind::Str, tr_const!("Part number")),
            optional("symbol", Kind::Str, tr_const!("Symbol key, e.g. inst_thermostat; empty: the symbol of its kind")),
        ],
        run: set_plan_symbol,
    },
    Spec {
        name: "place_installation",
        help: tr_const!("Puts a socket outlet, switch, light, heating element, junction box or device connection box for a protective device into a room at a point of the plant (a ceiling light on the ceiling there, anything else on the nearest wall at its mounting height; outside the room on the outside of its nearest wall) and into the schematic: a new device on the installation page of the circuit with a cable from the device before it or from the device given; without a protective device only into the room (data and antenna outlets, smoke alarms)"),
        params: &[
            optional("breaker", Kind::Str, BREAKER),
            param("room", Kind::Str, ROOM),
            param("part", Kind::Str, tr_const!("Part number, e.g. GEN.SOCKET-F-UP, GEN.SW-ONOFF-UP, GEN.LAMP-CEILING-LED, GEN.JBOX-UP, GEN.SOCKET-CEE-16-5P")),
            param("x", Kind::Num, X),
            param("z", Kind::Num, Z),
            optional("from", Kind::Str, tr_const!("Device or junction box of the circuit the cable comes from (default: the device before it)")),
            optional("tag", Kind::Str, tr_const!("Device tag (default: the room, the letter code of the kind and the next number)")),
            optional("height", Kind::Num, tr_const!("Mounting height above the floor in mm (default: that of the part)")),
            optional("cable", Kind::Str, tr_const!("Cable tag (default: the next free -W)")),
            optional("cable_type", Kind::Str, tr_const!("Cable type (default: NYM-J for its cores and cross-section)")),
            optional("cable_part", Kind::Str, tr_const!("Part number of the cable")),
            optional("cores", Kind::Int, tr_const!("Number of cores of the cable (default: 3, three-phase 5)")),
            optional("cross_section", Kind::Num, tr_const!("Cross-section of the cable in mm² (default: for the protective device)")),
            optional("length", Kind::Num, tr_const!("Known length of the cable in m (taken instead of the 3D route)")),
        ],
        run: place_installation,
    },
    Spec {
        name: "place_circuit_device",
        help: tr_const!("Places a socket outlet, switch, light or heating element of a circuit of the schematic in a room at a point of the plant like place_installation (one placed before moves), or makes an item placed without a device tag that device"),
        params: &[
            param("device", Kind::Str, tr_const!("Device tag in the schematic, e.g. +ROOM1-XD1")),
            optional("room", Kind::Str, ROOM),
            optional("x", Kind::Num, X),
            optional("z", Kind::Num, Z),
            optional("part", Kind::Str, tr_const!("Part for a device without one (default: the generic part of its kind)")),
            optional("item", Kind::Int, tr_const!("Id of a socket outlet, switch, light or heating element placed without a device tag to become the device (instead of room, x and z)")),
        ],
        run: place_circuit_device,
    },
    Spec {
        name: "list_breakers",
        help: tr_const!("Lists the protective devices of the mains for final circuits (up to 32 A): rated current, characteristic, phases, residual current protection and whether spare"),
        params: &[optional("spare", Kind::Bool, tr_const!("Only the spare ones (feeding nothing yet)"))],
        run: list_breakers,
    },
    Spec {
        name: "list_circuits",
        help: tr_const!("Lists the final circuits of the schematic with the socket outlets, switches, lights and heating elements behind their protective devices (kind, part, room), their connections and cables with the lengths of their 3D routes (estimated where not routed), currents and voltage drops"),
        params: &[
            optional("breaker", Kind::Str, tr_const!("Only the circuit of this protective device")),
            optional("unplaced", Kind::Bool, tr_const!("Only the devices not placed yet")),
            optional("estimate", Kind::Bool, tr_const!("Estimate the lengths along the installation zones instead of routing in 3D (faster)")),
        ],
        run: list_circuits,
    },
    Spec {
        name: "write_circuit_cables",
        help: tr_const!("Writes the lengths of the cables of the final circuits (of their 3D routes, estimated where not routed) into the cable list"),
        params: &[],
        run: write_circuit_cables,
    },
];

/// The circuits of the schematic: with the 3D routing, or estimated.
fn circuits(prj: &Project, estimate: bool) -> Vec<inst::Circuit> {
    if estimate {
        let (nl, open) = (Netlist::build(prj, true), Netlist::build(prj, false));
        let conns = openecraft_core::netlist::connections(prj, &nl, &open);
        return inst::circuits(prj, &nl, &conns, None);
    }
    let an = Analysis::build(prj);
    inst::circuits(prj, &an.netlist, &an.connections, Some(&|k: &str| an.routing.routed_length(k)))
}

fn circuit_json(prj: &Project, supply: &inst::Supply, c: &inst::Circuit, unplaced: bool) -> Value {
    let r2 = |v: f32| (v * 100.0).round() / 100.0;
    let run = inst::run(prj, supply, c);
    let devices: Vec<Value> = c
        .devices
        .iter()
        .filter(|d| !unplaced || d.placed.is_none())
        .map(|d| {
            json!({
                "device": d.tag, "kind": d.kind.key(), "text": d.text, "part": d.part.clone().unwrap_or_else(|| d.kind.default_part().to_string()), "own_part": d.part.is_some(),
                "room": d.placed.map(|(ri, _)| prj.layout.enclosures[ri].name.clone()), "item": d.placed.map(|p| p.1),
            })
        })
        .collect();
    let legs: Vec<Value> = c
        .legs
        .iter()
        .map(|l| json!({ "from": l.from, "to": l.to, "cable": l.cable, "cable_type": l.cable_type, "cross_section": l.mm2, "length": l.length.map(r2), "routed": l.routed }))
        .collect();
    json!({
        "breaker": c.breaker, "text": c.text, "rated": c.rated, "characteristic": c.characteristic, "three_phase": c.three_phase,
        "board": c.board.map(|ei| prj.layout.enclosures[ei].name.clone()), "page": c.page.map(|k| prj.pages[k].name.clone()),
        "placed": c.placed(), "devices": devices, "connections": legs,
        "length": r2(run.length), "routed": run.routed, "current": run.current.map(r2), "voltage": r2(run.voltage),
        "drop_v": run.drop.map(|d| r2(d.0)), "drop_percent": run.drop.map(|d| r2(d.1)), "feeder_drop_percent": run.upstream.map(r2),
        "total_drop_percent": run.total().map(r2), "ampacity": run.ampacity,
    })
}

fn room_arg(prj: &Project, a: &Args) -> Result<usize, String> {
    let room = a.str("room").map(str::trim).unwrap_or("");
    prj.layout.enclosures.iter().position(|e| e.is_room() && e.name == room).ok_or_else(|| tr_fmt!("No room named {}", room))
}

/// Where device `tag` is placed: its item.
fn placed_json(prj: &Project, tag: &str) -> Result<Value, String> {
    let (ri, it) = prj.layout.find_device(tag).ok_or_else(|| tr_fmt!("No socket outlet, switch or light {}", tag))?;
    let e = &prj.layout.enclosures[ri];
    Ok(json!({ "id": it.id, "device": tag, "part": it.part, "room": e.name, "face": panel::room_face_key(e, it.face), "x": it.x, "y": it.y }))
}

fn place_installation(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let ri = room_arg(prj, a)?;
    let at = [a.num("x").unwrap_or(0.0), a.num("z").unwrap_or(0.0)];
    let text = |k: &str| a.str(k).map(str::trim).filter(|v| !v.is_empty()).map(String::from);
    let cores = match a.int("cores") {
        Some(n) if !(1..=60).contains(&n) => return Err(tr("The number of cores must be from 1 to 60").to_string()),
        n => n.map(|n| n as usize),
    };
    let opts = inst::Options {
        from: text("from"),
        tag: text("tag"),
        cable: text("cable"),
        cable_type: text("cable_type"),
        cable_part: text("cable_part"),
        cores,
        mm2: num_in(a, "cross_section", 0.1, 1000.0)?,
        length: num_in(a, "length", 0.0, 100_000.0)?,
        height: num_in(a, "height", 0.0, 100_000.0)?,
    };
    let part = a.req("part").trim();
    let Some(breaker) = text("breaker") else {
        // Only into the room, with the device tag given.
        let id = inst::place_at(prj, ri, part, at, opts.height)?;
        if let Some(tag) = &opts.tag {
            inst::name_item(prj, id, tag).inspect_err(|_| prj.layout.enclosures[ri].items.retain(|i| i.id != id))?;
        }
        let e = &prj.layout.enclosures[ri];
        let it = e.item(id).ok_or_else(|| tr_fmt!("No socket outlet, switch or light {}", id))?;
        let device = if it.tag.is_empty() { Value::Null } else { json!(it.tag) };
        return Ok(json!({ "id": id, "device": device, "part": it.part, "room": e.name, "face": panel::room_face_key(e, it.face), "x": it.x, "y": it.y }));
    };
    let tag = inst::add_device(prj, &breaker, part, ri, at, &opts)?;
    let mut out = placed_json(prj, &tag)?;
    let page = prj.pages.iter().find(|pg| prj.layout.installation_pages.contains(&pg.id) && pg.symbols.iter().any(|s| s.tag == tag)).map(|pg| pg.name.clone());
    let cable = prj.cables.iter().find(|c| c.values.get("cable.target").is_some_and(|t| *t == tag));
    out["page"] = json!(page);
    out["cable"] = json!(cable.map(|c| c.tag.clone()));
    out["from"] = json!(cable.and_then(|c| c.values.get("cable.source").cloned()));
    Ok(out)
}

fn list_breakers(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let nl = Netlist::build(prj, true);
    let list = inst::breakers(prj, &nl);
    Ok(Value::Array(
        list.into_iter()
            .filter(|b| !a.flag("spare") || b.spare)
            .map(|b| json!({ "breaker": b.tag, "text": b.text, "rated": b.rated, "characteristic": b.characteristic, "phases": b.phases, "residual_ma": b.residual, "spare": b.spare }))
            .collect(),
    ))
}

fn list_circuits(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let only = a.str("breaker").map(str::trim);
    let list = circuits(prj, a.flag("estimate"));
    let (nl, open) = (Netlist::build(prj, true), Netlist::build(prj, false));
    let supply = inst::Supply::of(prj, &nl, &openecraft_core::netlist::connections(prj, &nl, &open));
    let out: Vec<Value> = list.iter().filter(|c| only.is_none_or(|b| c.breaker == b)).map(|c| circuit_json(prj, &supply, c, a.flag("unplaced"))).collect();
    if let (Some(b), true) = (only, out.is_empty()) {
        return Err(tr_fmt!("No socket outlet, switch, light or heating element behind {} in the schematic", b));
    }
    Ok(Value::Array(out))
}

fn place_circuit_device(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let tag = a.req("device").trim().to_string();
    let list = circuits(prj, true);
    let sc = inst::circuit_of(&list, &tag).ok_or_else(|| tr_fmt!("{} is no socket outlet, switch, light or heating element of a circuit of the schematic", tag))?.clone();
    match a.int("item").and_then(|v| u64::try_from(v).ok()) {
        Some(item) => inst::link_device(prj, &sc, &tag, item)?,
        None => {
            let ri = room_arg(prj, a)?;
            inst::place_device(prj, &sc, &tag, ri, a.str("part").unwrap_or(""), [a.num("x").unwrap_or(0.0), a.num("z").unwrap_or(0.0)])?;
        }
    }
    let mut out = placed_json(prj, &tag)?;
    out["breaker"] = json!(sc.breaker);
    Ok(out)
}

fn write_circuit_cables(prj: &mut Project, _: &Args) -> Result<Value, String> {
    let list = circuits(prj, false);
    let n = inst::write_cables(prj, &list);
    let cables: Vec<Value> = list.iter().flat_map(|c| &c.legs).filter(|l| !l.cable.is_empty()).filter_map(|l| prj.cables.iter().find(|x| x.tag == l.cable)).map(|c| json!({ "cable": c.tag, "length": c.length, "cable_type": c.cable_type })).collect();
    Ok(json!({ "cables": n, "list": cables }))
}

fn plan_symbols(prj: &mut Project, _: &Args) -> Result<Value, String> {
    let list: Vec<Value> = openecraft_core::parts::PLAN_SYMBOLS.iter().filter(|(k, _)| prj.symbol(k).is_some()).map(|(k, n)| json!({ "key": k, "name": tr(n) })).collect();
    Ok(json!(list))
}

fn set_plan_symbol(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let number = a.req("part");
    let symbol = a.str("symbol").unwrap_or_default().trim().to_string();
    if !symbol.is_empty() && prj.symbol(&symbol).is_none() {
        return Err(tr_fmt!("Unknown symbol: {}", symbol));
    }
    inst::take_part(prj, number).ok_or_else(|| tr_fmt!("Unknown part: {}", number))?;
    let part = prj.parts.iter_mut().find(|p| p.number == number).ok_or_else(|| tr_fmt!("Unknown part: {}", number))?;
    part.plan_symbol = symbol;
    let part = part.clone();
    let shown = inst::kind_of(&part).map(|k| inst::plan_symbol(prj, &part, k));
    Ok(json!({ "part": part.number, "plan_symbol": part.plan_symbol, "shown": shown }))
}

#[cfg(test)]
mod tests;
