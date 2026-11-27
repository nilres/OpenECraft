//! Engineering calculations: enclosure climate and electrical sizing.

use crate::builtin::common::{num_in, NO_MAX};
use crate::params::{optional, Args, Kind};
use crate::{Info, Plugin, Registry};
use super::{add_commands, info, Spec};
use openecraft_core::analysis::Analysis;
use openecraft_core::climate::{self, Cooling, Installation, Material};
use openecraft_core::model::Project;
use openecraft_core::netlist::Netlist;
use openecraft_core::sizing;
use openecraft_i18n::{tr_const, tr_fmt};
use serde_json::{json, Value};

pub struct Engineering;

impl Plugin for Engineering {
    fn info(&self) -> Info {
        info("engineering", tr_const!("Engineering"), tr_const!("Enclosure climate and electrical sizing"))
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        add_commands(reg, COMMANDS)
    }
}

pub static COMMANDS: &[Spec] = &[
    Spec {
        name: "enclosure_climate",
        help: tr_const!("Climate of the enclosures (IEC/TR 60890): power loss, inside temperature, fan or cooling unit and heating needed; optionally changes the settings of one"),
        params: &[
            optional("enclosure", Kind::Str, tr_const!("Enclosure (location, e.g. +CAB1) whose settings change")),
            optional("installation", Kind::Str, tr_const!("free_standing, wall_mounted, end_free_standing, end_wall_mounted, middle_free_standing, middle_wall_mounted or middle_wall_mounted_covered")),
            optional("material", Kind::Str, tr_const!("sheet_steel, stainless_steel, aluminium or plastic")),
            optional("ambient_max", Kind::Num, tr_const!("Highest ambient temperature in °C")),
            optional("ambient_min", Kind::Num, tr_const!("Lowest ambient temperature in °C")),
            optional("inside_max", Kind::Num, tr_const!("Highest inside temperature in °C")),
            optional("inside_min", Kind::Num, tr_const!("Lowest inside temperature in °C")),
            optional("extra_loss", Kind::Num, tr_const!("Additional power loss in W")),
            optional("altitude", Kind::Num, tr_const!("Altitude of the site in m")),
            optional("sealed", Kind::Bool, tr_const!("Keep the degree of protection: heat exchanger or cooling unit instead of filter fans")),
            optional("humidity", Kind::Num, tr_const!("Relative humidity of the ambient air in % (0: no condensation check)")),
        ],
        run: enclosure_climate,
    },
    Spec {
        name: "electrical_sizing",
        help: tr_const!("Wires too thin for their protective device and the voltage drop of the cables (IEC 60204-1)"),
        params: &[],
        run: electrical_sizing,
    },
    Spec {
        name: "size_wires",
        help: tr_const!("Gives the wires behind protective devices that are too thin for the device the smallest cross-section that carries its rated current (IEC 60204-1); wires of cables keep their cable"),
        params: &[],
        run: size_wires,
    },
];

const INSTALLATIONS: [(&str, Installation); 7] = [
    ("free_standing", Installation::FreeStanding),
    ("wall_mounted", Installation::WallMounted),
    ("end_free_standing", Installation::EndFreeStanding),
    ("end_wall_mounted", Installation::EndWallMounted),
    ("middle_free_standing", Installation::MiddleFreeStanding),
    ("middle_wall_mounted", Installation::MiddleWallMounted),
    ("middle_wall_mounted_covered", Installation::MiddleWallMountedCovered),
];

const MATERIALS: [(&str, Material); 4] =
    [("sheet_steel", Material::SheetSteel), ("stainless_steel", Material::StainlessSteel), ("aluminium", Material::Aluminium), ("plastic", Material::Plastic)];

fn enclosure_climate(prj: &mut Project, a: &Args) -> Result<Value, String> {
    if let Some(name) = a.str("enclosure") {
        let e = prj.layout.enclosures.iter_mut().find(|e| e.name == name).ok_or_else(|| tr_fmt!("No enclosure {}", name))?;
        let c = &mut e.climate;
        if let Some(v) = a.str("installation") {
            c.installation = INSTALLATIONS.iter().find(|x| x.0 == v).map(|x| x.1).ok_or_else(|| tr_fmt!("Unknown installation: {}", v))?;
        }
        if let Some(v) = a.str("material") {
            c.material = MATERIALS.iter().find(|x| x.0 == v).map(|x| x.1).ok_or_else(|| tr_fmt!("Unknown material: {}", v))?;
        }
        num_in(a, "humidity", 0.0, 100.0)?;
        num_in(a, "extra_loss", 0.0, NO_MAX)?;
        for (name, field) in [
            ("ambient_max", &mut c.ambient_max),
            ("ambient_min", &mut c.ambient_min),
            ("inside_max", &mut c.inside_max),
            ("inside_min", &mut c.inside_min),
            ("extra_loss", &mut c.extra_loss),
            ("altitude", &mut c.altitude),
            ("humidity", &mut c.humidity),
        ] {
            if let Some(v) = a.num(name) {
                *field = v;
            }
        }
        if let Some(v) = a.value("sealed").and_then(Value::as_bool) {
            c.sealed = v;
        }
    }
    let out: Vec<Value> = prj
        .layout
        .enclosures
        .iter()
        .filter(|e| !e.is_room())
        .map(|e| {
            let r = climate::calculate(prj, e);
            let c = &e.climate;
            let (fan, exchanger, unit) = match r.cooling {
                Cooling::None => (None, None, None),
                Cooling::Fan(v) => (Some(v), None, None),
                Cooling::HeatExchanger(q) => (None, Some(q), None),
                Cooling::CoolingUnit(q) => (None, None, Some(q)),
            };
            json!({
                "enclosure": e.name,
                "installation": INSTALLATIONS.iter().find(|x| x.1 == c.installation).map(|x| x.0),
                "material": MATERIALS.iter().find(|x| x.1 == c.material).map(|x| x.0),
                "ambient": [c.ambient_min, c.ambient_max],
                "inside_allowed": [c.inside_min, c.inside_max],
                "open_assembly": r.open,
                "surface_m2": r.surface,
                "power_loss_w": r.loss,
                "inside_without_cooling": r.inside,
                "altitude_m": c.altitude,
                "sealed": c.sealed,
                "humidity_percent": c.humidity,
                "fan_m3_per_h": fan,
                "heat_exchanger_w_per_k": exchanger,
                "cooling_unit_w": unit,
                "heating_w": r.heating,
                "dew_point": r.dew_point,
                "devices_without_power_loss": r.without_data,
            })
        })
        .collect();
    Ok(Value::Array(out))
}

fn size_wires(prj: &mut Project, _: &Args) -> Result<Value, String> {
    let changed = sizing::size_wires(prj);
    Ok(json!({ "changed": changed }))
}

fn electrical_sizing(prj: &mut Project, _: &Args) -> Result<Value, String> {
    let (nl, open) = (Netlist::build(prj, true), Netlist::build(prj, false));
    let an = Analysis::build(prj);
    let overloads: Vec<Value> = sizing::overloads(prj, &nl, &open)
        .into_iter()
        .map(|o| json!({ "device": o.device, "rated_a": o.rated, "cross_section": o.cross_section, "capacity_a": o.ampacity, "page": prj.pages[o.target.0].name, "wire": o.target.1 }))
        .collect();
    let cables: Vec<Value> = sizing::cable_drops(prj, &an.connections)
        .into_iter()
        .map(|d| {
            json!({
                "cable": d.cable,
                "length_m": d.length,
                "cross_section": d.cross_section,
                "load": d.load,
                "current_a": d.current,
                "voltage_v": d.voltage,
                "three_phase": d.three_phase,
                "voltage_drop_v": d.drop.map(|x| x.0),
                "voltage_drop_percent": d.drop.map(|x| x.1),
                "capacity_a": d.ampacity,
            })
        })
        .collect();
    Ok(json!({ "too_thin": overloads, "cables": cables, "voltage_drop_limit_percent": openecraft_core::rules::limits(prj).voltage_drop }))
}

#[cfg(test)]
mod tests;
