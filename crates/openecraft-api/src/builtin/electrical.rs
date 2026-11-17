//! Electrical calculations and verification: supply and installation conditions,
//! short circuit and selectivity, loads, EMC, safety functions, rule sets and the
//! inspection and test report of the installation.

use crate::builtin::common::{int_in, num_in, NO_MAX};
use crate::params::{optional, Args, Kind};
use crate::{Info, Plugin, Registry};
use super::{add_commands, info, Spec};
use openecraft_core::analysis::Analysis;
use openecraft_core::electrical::{Coordination, Earthing, Insulation, Material, Transformer};
use openecraft_core::shortcircuit::Basis;
use openecraft_core::model::Project;
use openecraft_core::rules::{self, RuleSet};
use openecraft_core::safety::{self, SafetyFunction};
use openecraft_core::inspection::{self, Field};
use openecraft_core::{emc, loads, shortcircuit};
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde_json::{json, Value};

pub struct Electrical;

impl Plugin for Electrical {
    fn info(&self) -> Info {
        info("electrical", tr_const!("Electrical engineering"), tr_const!("Short circuit, load, EMC and safety calculations, rule sets and project verification"))
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        add_commands(reg, COMMANDS)
    }
}

pub static COMMANDS: &[Spec] = &[
    Spec {
        name: "electrical_settings",
        help: tr_const!("Supply (voltage, earthing system, short-circuit currents) and installation conditions of the conductors; changes the given values"),
        params: &[
            optional("voltage", Kind::Num, tr_const!("Line-to-line voltage in V")),
            optional("frequency", Kind::Num, tr_const!("Frequency in Hz")),
            optional("earthing", Kind::Str, tr_const!("tn_s, tn_c, tn_c_s, tt or it")),
            optional("ik_max", Kind::Num, tr_const!("Highest three-phase short-circuit current at the supply in kA")),
            optional("ik_min", Kind::Num, tr_const!("Lowest line-to-earth fault current at the supply in kA (0: from ik_max)")),
            optional("rated_current", Kind::Num, tr_const!("Rated current of the supply in A (0: not limited)")),
            optional("rx", Kind::Num, tr_const!("R/X ratio of the supply impedance")),
            optional("transformer_kva", Kind::Num, tr_const!("Rated power of the transformer feeding the supply in kVA (0: none, the short-circuit currents are given)")),
            optional("transformer_uk", Kind::Num, tr_const!("Short-circuit voltage of the transformer in %")),
            optional("transformer_losses", Kind::Num, tr_const!("Load losses of the transformer in W (0: typical)")),
            optional("network_mva", Kind::Num, tr_const!("Short-circuit power of the network upstream of the transformer in MVA (0: infinite)")),
            optional("motors", Kind::Bool, tr_const!("Asynchronous motors add to the short-circuit current")),
            optional("neutral", Kind::Bool, tr_const!("IT system with distributed neutral")),
            optional("ambient", Kind::Num, tr_const!("Ambient temperature of the conductors in °C")),
            optional("grouped", Kind::Int, tr_const!("Loaded circuits bunched together")),
            optional("material", Kind::Str, tr_const!("copper or aluminium")),
            optional("insulation", Kind::Str, tr_const!("pvc or xlpe")),
            optional("diversity", Kind::Num, tr_const!("Diversity factor of the loads (0: IEC 61439-1, table 101)")),
        ],
        run: electrical_settings,
    },
    Spec {
        name: "short_circuit",
        help: tr_const!("Short-circuit currents, breaking capacities, disconnection times, short-circuit withstand of the cables and selectivity of the protective devices"),
        params: &[],
        run: short_circuit,
    },
    Spec {
        name: "coordination_table",
        help: tr_const!("Selectivity and back-up protection of device pairs from the manufacturers' tables, by part number; adds, replaces or removes an entry"),
        params: &[
            optional("upstream", Kind::Str, tr_const!("Part number of the upstream device")),
            optional("downstream", Kind::Str, tr_const!("Part number of the downstream device")),
            optional("selectivity_ka", Kind::Num, tr_const!("Selective up to this short-circuit current in kA (0: not given)")),
            optional("backup_ka", Kind::Num, tr_const!("Back-up protection up to this short-circuit current in kA (0: not given)")),
            optional("remove", Kind::Bool, tr_const!("Remove the entry of upstream and downstream")),
        ],
        run: coordination_table,
    },
    Spec {
        name: "load_calculation",
        help: tr_const!("Loads of the mains, load of each protective device, demand with the diversity factor, phase balance and power supply budgets"),
        params: &[],
        run: load_calculation,
    },
    Spec {
        name: "emc_check",
        help: tr_const!("EMC classes of the cables (drive output, analogue, power, signal, shield) and the EMC problems"),
        params: &[],
        run: emc_check,
    },
    Spec {
        name: "safety_functions",
        help: tr_const!("Safety functions (ISO 13849-1) with the performance level reached; adds, replaces or removes one"),
        params: &[
            optional("set", Kind::Object, tr_const!("Safety function to add or replace (by name): {name, description, required: A…E, subsystems: [{name, devices, channel2, category: B|One|Two|Three|Four, dc, operations, mttfd, ccf}]}")),
            optional("remove", Kind::Str, tr_const!("Name of the safety function to remove")),
        ],
        run: safety_functions,
    },
    Spec {
        name: "rule_sets",
        help: tr_const!("Rule libraries of standards and customer rule sets with the limits in effect; enables, disables, adds or removes them"),
        params: &[
            optional("enable", Kind::Str, tr_const!("Library id or customer rule set name to verify against")),
            optional("disable", Kind::Str, tr_const!("Library id or customer rule set name to no longer verify against")),
            optional("add", Kind::Object, tr_const!("Customer rule set to add or replace (by name), as exported")),
            optional("remove", Kind::Str, tr_const!("Name of the customer rule set to remove")),
        ],
        run: rule_sets,
    },
    Spec {
        name: "inspection_report",
        help: tr_const!("Inspection and test report of the installation (IEC 60364-6): the data of the report, the visual inspection, every circuit of the mains with what the schematic gives (description, cables, protective device, residual current device, limits; null: missing in the project), the results measured and the verdicts; sets the data of the report and the results of a circuit"),
        params: &[
            optional("circuit", Kind::Str, tr_const!("Device tag of the protective device whose results to set")),
            optional("results", Kind::Object, tr_const!("Results to set: continuity (Ω), insulation_voltage (V), insulation (MΩ), loop_impedance (Ω), rcd_current (mA), rcd_time (ms), rcd_time_5 (ms), rcd_voltage (V), function (ok or failed), tester, date, instrument, remark; an empty text clears")),
            optional("header", Kind::Object, tr_const!("Data of the report to set: client, site, installer, reason, inspector, instruments, ze (Ω), ik (kA), earth_electrode (Ω), bonding (Ω), next_inspection, signed_by, signed_on and the visual inspection as check.<item> (ok, failed or n/a); an empty text clears")),
        ],
        run: inspection_report,
    },
    Spec {
        name: "verify_project",
        help: tr_const!("Verifies the project against the enabled rule libraries and customer rule sets"),
        params: &[],
        run: verify_project,
    },
];

const EARTHING: [(&str, Earthing); 5] = [("tn_s", Earthing::TnS), ("tn_c", Earthing::TnC), ("tn_c_s", Earthing::TnCS), ("tt", Earthing::Tt), ("it", Earthing::It)];

fn pick<T: Copy>(table: &[(&str, T)], value: &str) -> Option<T> {
    table.iter().find(|x| x.0 == value).map(|x| x.1)
}

fn electrical_settings(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let el = &mut prj.electrical;
    for (name, field) in [
        ("voltage", &mut el.supply.voltage),
        ("frequency", &mut el.supply.frequency),
        ("ik_max", &mut el.supply.ik_max),
        ("ik_min", &mut el.supply.ik_min),
        ("rated_current", &mut el.supply.rated_current),
        ("ambient", &mut el.conditions.ambient),
        ("diversity", &mut el.diversity),
    ] {
        if let Some(v) = a.num(name) {
            *field = v;
        }
    }
    if let Some(v) = a.str("earthing") {
        el.supply.earthing = pick(&EARTHING, v).ok_or_else(|| tr_fmt!("Unknown earthing system: {}", v))?;
    }
    if let Some(v) = a.str("material") {
        el.conditions.material = pick(&[("copper", Material::Copper), ("aluminium", Material::Aluminium)], v).ok_or_else(|| tr_fmt!("Unknown material: {}", v))?;
    }
    if let Some(v) = a.str("insulation") {
        el.conditions.insulation = pick(&[("pvc", Insulation::Pvc), ("xlpe", Insulation::Xlpe)], v).ok_or_else(|| tr_fmt!("Unknown insulation: {}", v))?;
    }
    if let Some(v) = int_in(a, "grouped", 1, u32::MAX.into())? {
        el.conditions.grouped = v as u32;
    }
    if let Some(v) = num_in(a, "rx", 0.0, NO_MAX)? {
        el.supply.rx = v;
    }
    if let Some(v) = a.num("transformer_kva") {
        el.supply.transformer = (v > 0.0).then(|| Transformer { power: v, ..el.supply.transformer.clone().unwrap_or_default() });
    }
    for name in ["transformer_uk", "transformer_losses", "network_mva"] {
        let Some(v) = num_in(a, name, 0.0, NO_MAX)? else { continue };
        let t = el.supply.transformer.as_mut().ok_or_else(|| tr_fmt!("No transformer: set {} first", "transformer_kva"))?;
        let field = match name {
            "transformer_uk" => &mut t.uk,
            "transformer_losses" => &mut t.losses,
            _ => &mut t.network,
        };
        *field = v;
    }
    for (name, field) in [("motors", &mut el.supply.motors), ("neutral", &mut el.supply.neutral)] {
        if let Some(v) = a.value(name).and_then(Value::as_bool) {
            *field = v;
        }
    }
    let el = &prj.electrical;
    let src = shortcircuit::Source::of(&el.supply);
    Ok(json!({
        "voltage": el.supply.voltage,
        "frequency": el.supply.frequency,
        "earthing": EARTHING.iter().find(|x| x.1 == el.supply.earthing).map(|x| x.0),
        "ik_max_ka": el.supply.ik_max,
        "ik_min_ka": el.supply.ik_min,
        "rx": el.supply.rx,
        "transformer": el.supply.transformer.as_ref().map(|t| json!({ "kva": t.power, "uk_percent": t.uk, "losses_w": t.load_losses(), "network_mva": t.network })),
        "motors": el.supply.motors,
        "neutral": el.supply.neutral,
        "supply_ik3_ka": src.ik3 / 1000.0,
        "supply_ik1_ka": src.ik1 / 1000.0,
        "rated_current_a": el.supply.rated_current,
        "ambient": el.conditions.ambient,
        "grouped": el.conditions.grouped,
        "material": if el.conditions.material == Material::Copper { "copper" } else { "aluminium" },
        "insulation": if el.conditions.insulation == Insulation::Pvc { "pvc" } else { "xlpe" },
        "capacity_factor": el.conditions.factor(),
        "diversity": el.diversity,
    }))
}

fn finite(v: f32) -> Value {
    if v.is_finite() {
        json!(v)
    } else {
        Value::Null
    }
}

/// A value of an object parameter as text (numbers too; null clears).
fn value_text(key: &str, value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Null => Ok(String::new()),
        _ => Err(tr_fmt!("Unknown field: {}", key)),
    }
}

fn inspection_report(prj: &mut Project, a: &Args) -> Result<Value, String> {
    // All values or none.
    let mut changed = prj.clone();
    if let Some(results) = a.value("results") {
        let circuit = a.str("circuit").ok_or_else(|| tr("Name the circuit whose results to set").to_string())?;
        let results = results.as_object().ok_or_else(|| tr("The results are an object of fields and values").to_string())?;
        let an = Analysis::build(prj);
        if !inspection::circuits(prj, &an.netlist, &an.connections).iter().any(|c| c.circuit == circuit) {
            return Err(tr_fmt!("No circuit {}", circuit));
        }
        for (key, value) in results {
            let field = Field::from_key(key).ok_or_else(|| tr_fmt!("Unknown field: {}", key))?;
            inspection::set(&mut changed, circuit, field, &value_text(key, value)?)?;
        }
    }
    if let Some(header) = a.value("header") {
        let header = header.as_object().ok_or_else(|| tr("The results are an object of fields and values").to_string())?;
        let values = header.iter().map(|(k, v)| value_text(k, v).map(|v| (k.as_str(), v))).collect::<Result<Vec<_>, _>>()?;
        inspection::set_header_all(&mut changed, values.iter().map(|(k, v)| (*k, v.as_str())))?;
    }
    prj.inspection = changed.inspection;
    let an = Analysis::build(prj);
    serde_json::to_value(inspection::Report::of(prj, &an.netlist, &an.connections)).map_err(|e| e.to_string())
}

fn short_circuit(prj: &mut Project, _: &Args) -> Result<Value, String> {
    let an = Analysis::build(prj);
    let faults: Vec<Value> = shortcircuit::faults(prj, &an.netlist, &an.connections)
        .iter()
        .map(|f| {
            json!({
                "device": f.device,
                "characteristic": f.protection.characteristic.title(),
                "rated_a": f.protection.rated,
                "breaking_capacity_ka": f.protection.breaking,
                "making_capacity_ka": f.protection.making,
                "ik_max_a": f.ik_max,
                "ip_a": f.ip,
                "motors_a": f.ik_motors,
                "feeders": f.feeders,
                "cable": f.cable,
                "ik_max_end_a": f.ik_max_end,
                "ik_min_end_a": f.ik_min_end,
                "second_fault": f.second_fault,
                "backup_by": f.backup,
                "disconnection_s": finite(f.time),
                "permitted_s": f.limit,
                "trip_current_a": finite(f.trip_current),
                "residual_ma": f.residual,
                "energy_a2s": f.energy.map(|e| e.0),
                "withstand_a2s": f.energy.map(|e| e.1),
                "breaks": f.breaks(),
                "makes": f.makes(),
                "disconnects": f.disconnects(),
                "withstands": f.withstands(),
            })
        })
        .collect();
    let selectivity: Vec<Value> = shortcircuit::selectivity(prj, &an.netlist, &an.connections)
        .iter()
        .map(|x| {
            let basis = match x.basis {
                Basis::Table => "table",
                Basis::FuseRatio => "fuse_ratio",
                Basis::Energy => "pre_arcing_energy",
                Basis::Curves => "curves",
            };
            json!({ "upstream": x.upstream, "downstream": x.downstream, "selective_up_to_a": x.limit, "ik_max_a": x.ik, "total": x.is_total(), "basis": basis })
        })
        .collect();
    let src = shortcircuit::Source::of(&prj.electrical.supply);
    Ok(json!({ "supply_ik3_a": src.ik3, "supply_ik1_a": src.ik1, "devices": faults, "selectivity": selectivity }))
}

fn coordination_table(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let table = &mut prj.electrical.coordination;
    match (a.str("upstream"), a.str("downstream")) {
        (Some(up), Some(down)) => {
            let at = table.iter().position(|c| c.upstream == up && c.downstream == down);
            if a.flag("remove") {
                table.remove(at.ok_or_else(|| tr_fmt!("No entry for {} and {}", up, down))?);
            } else {
                let i = at.unwrap_or_else(|| {
                    table.push(Coordination { upstream: up.into(), downstream: down.into(), ..Default::default() });
                    table.len() - 1
                });
                if let Some(v) = num_in(a, "selectivity_ka", 0.0, NO_MAX)? {
                    table[i].selectivity = v;
                }
                if let Some(v) = num_in(a, "backup_ka", 0.0, NO_MAX)? {
                    table[i].backup = v;
                }
            }
        }
        (None, None) => {}
        _ => return Err(tr_fmt!("{} and {} are needed together", "upstream", "downstream")),
    }
    Ok(Value::Array(prj.electrical.coordination.iter().map(|c| json!({ "upstream": c.upstream, "downstream": c.downstream, "selectivity_ka": c.selectivity, "backup_ka": c.backup })).collect()))
}

fn load_calculation(prj: &mut Project, _: &Args) -> Result<Value, String> {
    let c = loads::calculate(prj, &Analysis::build(prj).netlist);
    let loads: Vec<Value> = c
        .loads
        .iter()
        .map(|l| json!({ "device": l.device, "kind": l.kind.title(), "feeder": l.feeder, "phases": l.phases, "power_kw": l.power, "apparent_kva": l.apparent, "current_a": l.current }))
        .collect();
    let feeders: Vec<Value> = c.feeders.iter().map(|f| json!({ "device": f.device, "rated_a": f.rated, "loads": f.loads, "current_a": f.current })).collect();
    let budgets: Vec<Value> = c
        .budgets
        .iter()
        .map(|b| json!({ "device": b.device, "voltage_v": b.voltage, "rated_a": b.rated, "current_a": b.current, "loads": b.loads.iter().map(|(t, i)| json!({ "device": t, "current_a": i })).collect::<Vec<_>>(), "without_data": b.without_data }))
        .collect();
    Ok(json!({
        "loads": loads,
        "feeders": feeders,
        "power_supplies": budgets,
        "installed_kw": c.installed.0,
        "installed_kva": c.installed.1,
        "diversity": c.diversity,
        "demand_kva": c.demand,
        "demand_a": c.demand_current,
        "phases_a": c.phases,
        "unbalance_percent": c.unbalance,
    }))
}

fn emc_check(prj: &mut Project, _: &Args) -> Result<Value, String> {
    let an = Analysis::build(prj);
    let cables: Vec<Value> = emc::classes(prj, &an.netlist, &an.connections)
        .iter()
        .map(|c| json!({ "cable": c.cable, "drive": c.drive, "analog": c.analog, "power": c.power, "signal": c.signal, "shielded": c.shielded, "shield_both_ends": c.both_ends }))
        .collect();
    let problems: Vec<String> = emc::problems(prj, &an.netlist, &an.connections).into_iter().map(|i| i.message()).collect();
    Ok(json!({ "cables": cables, "problems": problems }))
}

fn safety_functions(prj: &mut Project, a: &Args) -> Result<Value, String> {
    if let Some(v) = a.value("set") {
        let f: SafetyFunction = serde_json::from_value(v.clone()).map_err(|e| tr_fmt!("Not a valid safety function: {}", e))?;
        let list = &mut prj.electrical.safety;
        match list.iter_mut().find(|x| x.name == f.name) {
            Some(x) => *x = f,
            None => list.push(f),
        }
    }
    if let Some(name) = a.str("remove") {
        let list = &mut prj.electrical.safety;
        let n = list.len();
        list.retain(|x| x.name != name);
        if list.len() == n {
            return Err(tr_fmt!("No safety function {}", name));
        }
    }
    let out: Vec<Value> = prj
        .electrical
        .safety
        .iter()
        .map(|f| {
            let e = safety::evaluate(prj, f);
            let subs: Vec<Value> = e
                .subsystems
                .iter()
                .map(|r| json!({ "name": r.name, "mttfd_years": r.mttfd, "dc_avg": r.dc, "pfh": r.pfh, "pl": r.pl.map(|p| p.title()), "problems": r.problems, "notes": r.notes }))
                .collect();
            json!({
                "name": f.name,
                "required": f.required.title(),
                "achieved": e.achieved.map(|p| p.title()),
                "pfh": e.pfh,
                "sil": e.achieved.and_then(|p| p.sil()),
                "met": e.is_met(),
                "subsystems": subs,
                "definition": f,
            })
        })
        .collect();
    Ok(Value::Array(out))
}

fn rule_sets(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let library = rules::library();
    let el = &mut prj.electrical;
    for (name, on) in [(a.str("enable"), true), (a.str("disable"), false)] {
        let Some(name) = name else { continue };
        if library.iter().any(|r| r.id == name) {
            el.standards.retain(|x| x != name);
            if on {
                el.standards.push(name.into());
            }
        } else if let Some(r) = el.rule_sets.iter_mut().find(|r| r.name == name) {
            r.active = on;
        } else {
            return Err(tr_fmt!("No rule library or rule set {}", name));
        }
    }
    if let Some(v) = a.value("add") {
        let r: RuleSet = serde_json::from_value(v.clone()).map_err(|e| tr_fmt!("Not a valid rule set file: {}", e))?;
        match el.rule_sets.iter_mut().find(|x| x.name == r.name) {
            Some(x) => *x = r,
            None => el.rule_sets.push(r),
        }
    }
    if let Some(name) = a.str("remove") {
        let n = el.rule_sets.len();
        el.rule_sets.retain(|x| x.name != name);
        if el.rule_sets.len() == n {
            return Err(tr_fmt!("No rule library or rule set {}", name));
        }
    }
    let l = rules::limits(prj);
    let libs: Vec<Value> = library
        .iter()
        .map(|r| json!({ "id": r.id, "name": r.name, "description": r.description, "source": r.source, "enabled": prj.electrical.standards.contains(&r.id), "checks": r.checks.len() }))
        .collect();
    let custom: Vec<Value> = prj.electrical.rule_sets.iter().map(|r| json!({ "name": r.name, "source": r.source, "active": r.active, "checks": r.checks.len() })).collect();
    Ok(json!({
        "libraries": libs,
        "rule_sets": custom,
        "limits": {
            "voltage_drop_percent": l.voltage_drop,
            "disconnect_final_s": l.disconnect_final,
            "disconnect_distribution_s": l.disconnect_distribution,
            "min_cross_section_power_mm2": l.min_cross_section_power,
            "min_cross_section_control_mm2": l.min_cross_section_control,
            "load_reserve_percent": l.load_reserve,
            "max_unbalance_percent": l.max_unbalance,
            "total_selectivity": l.total_selectivity,
            "shield_drive_cables": l.shield_drive_cables,
            "shield_analog_cables": l.shield_analog_cables,
            "surge_protection": l.surge_protection,
        },
    }))
}

fn verify_project(prj: &mut Project, _: &Args) -> Result<Value, String> {
    let an = Analysis::build(prj);
    let results = rules::verify(prj, &an.connections);
    let failed = results.iter().filter(|r| !r.failed.is_empty()).count();
    let checks: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "rule_set": r.set,
                "rule": r.check.message,
                "level": format!("{:?}", r.check.level).to_lowercase(),
                "checked": r.checked,
                "violations": r.failed.iter().map(|i| &i.label).collect::<Vec<_>>(),
            })
        })
        .collect();
    let messages: Vec<String> = rules::problems(prj, &an.netlist, &an.connections).into_iter().map(|i| i.message()).collect();
    Ok(json!({ "checks": checks, "failed_checks": failed, "problems": messages }))
}

#[cfg(test)]
mod tests;
