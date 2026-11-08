//! The electrical side of PV arrays ([`PvString`]): modules in series form a string,
//! strings of the same kind run in parallel, each on an MPP tracker of an inverter or
//! charge controller (a device of the project).
//!
//! The values of a string follow from the module's data sheet values at STC and its
//! temperature coefficients at the design temperatures of the layout ([`PvDesign`]):
//! the highest voltage is the open-circuit voltage at the lowest temperature, the lowest
//! the voltage at maximum power at the highest temperature (with the coefficient of the
//! power, which mostly falls with the voltage), the mean that voltage at the mean
//! temperature. They are checked against the PV input data of the inverter: the
//! greatest DC voltage, the MPP voltage range, the number of trackers and the input and
//! short-circuit current per tracker.

use crate::issue::{Issue, Severity};
use crate::model::panel::{Building, PvArray, PvDesign, PvString, Roof};
use crate::model::Project;
use crate::parts::Part;
use openecraft_i18n::tr;
use std::collections::BTreeMap;

/// Temperature coefficients of modules without their own, per K.
pub const TC_VOC: f32 = -0.0028;
pub const TC_ISC: f32 = 0.0005;
pub const TC_PMAX: f32 = -0.0035;
/// Strings on one tracker whose voltages differ by more than this share.
pub const MISMATCH: f32 = 0.05;

/// The electrical data of a module at STC (V, A, W) with its temperature coefficients
/// (per K).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModuleData {
    pub pmax: f32,
    pub voc: f32,
    pub isc: f32,
    pub vmp: f32,
    pub imp: f32,
    pub tc_voc: f32,
    pub tc_isc: f32,
    pub tc_pmax: f32,
    /// Greatest system voltage, V.
    pub max_system: Option<f32>,
}

/// The first number of a text ("-0.29 %/K", "39.13 V", "13.9 A (Isc)").
fn number(text: &str) -> Option<f32> {
    let start = text.find(|c: char| c.is_ascii_digit() || c == '-' || c == '+')?;
    let rest = &text[start..];
    let end = rest.char_indices().skip(1).find(|&(_, c)| !(c.is_ascii_digit() || c == '.')).map_or(rest.len(), |(i, _)| i);
    rest[..end].trim_start_matches('+').parse().ok()
}

/// The numbers of a range ("65 - 450 V", "200 – 1000 V").
fn range(text: &str) -> Option<(f32, f32)> {
    let nums: Vec<f32> = text.split(|c: char| !(c.is_ascii_digit() || c == '.')).filter_map(|t| t.parse().ok()).collect();
    match nums.as_slice() {
        [a, b, ..] => Some((*a, *b)),
        _ => None,
    }
}

fn tech(p: &Part, key: &str) -> Option<f32> {
    p.tech_value(key).and_then(number)
}

fn find_part<'a>(prj: &'a Project, number: &str) -> Option<&'a Part> {
    prj.parts.iter().chain(crate::parts::catalog()).find(|p| p.number == number)
}

/// The electrical data of a module part, `None` without its voltages and currents.
pub fn module_data(prj: &Project, part: &str) -> Option<ModuleData> {
    let p = find_part(prj, part)?;
    let voc = tech(p, "Open-circuit voltage")?;
    let vmp = tech(p, "Voltage at maximum power")?;
    let imp = tech(p, "Current at maximum power")?;
    let isc = tech(p, "Short-circuit current").or_else(|| number(&p.current))?;
    let pmax = tech(p, "Maximum power").or_else(|| number(&p.power)).unwrap_or(vmp * imp);
    let tc = |key: &str, default: f32| tech(p, key).map_or(default, |v| v / 100.0);
    Some(ModuleData {
        pmax,
        voc,
        isc,
        vmp,
        imp,
        tc_voc: tc("Temperature coefficient of Voc", TC_VOC),
        tc_isc: tc("Temperature coefficient of Isc", TC_ISC),
        tc_pmax: tc("Temperature coefficient of Pmax", TC_PMAX),
        max_system: tech(p, "Max. system voltage"),
    })
}

/// The PV input of an inverter or charge controller (its part's technical data
/// "Max. PV input voltage", "MPPT voltage range", "MPP trackers", "Max. input current per
/// MPPT", "Max. short-circuit current per MPPT").
#[derive(Clone, Debug, PartialEq)]
pub struct PvInput {
    pub part: String,
    pub max_voltage: Option<f32>,
    pub mppt_range: Option<(f32, f32)>,
    pub trackers: u32,
    pub max_current: Option<f32>,
    pub max_isc: Option<f32>,
}

/// The PV input of a part, `None` for a part without one.
pub fn part_input(p: &Part) -> Option<PvInput> {
    let max_voltage = tech(p, "Max. PV input voltage");
    let mppt_range = p.tech_value("MPPT voltage range").and_then(range);
    let trackers = tech(p, "MPP trackers").map(|n| n.max(1.0) as u32);
    if max_voltage.is_none() && mppt_range.is_none() && trackers.is_none() {
        return None;
    }
    Some(PvInput {
        part: p.number.clone(),
        max_voltage,
        mppt_range,
        trackers: trackers.unwrap_or(1),
        max_current: tech(p, "Max. input current per MPPT"),
        max_isc: tech(p, "Max. short-circuit current per MPPT"),
    })
}

/// The PV input of the device `tag`: of its first part with one.
pub fn pv_input(prj: &Project, tag: &str) -> Option<PvInput> {
    let d = prj.devices.get(tag)?;
    d.parts.iter().filter_map(|r| find_part(prj, &r.number)).find_map(part_input)
}

/// The devices with a PV input (inverters and charge controllers), by tag.
pub fn inverters(prj: &Project) -> Vec<String> {
    prj.devices.keys().filter(|t| pv_input(prj, t).is_some()).cloned().collect()
}

/// The values of a string at the design temperatures: voltages in V, currents in A (of
/// all its strings in parallel), power in W.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StringValues {
    /// Modules in all its strings.
    pub modules: u32,
    /// Open-circuit voltage at the lowest temperature and at STC.
    pub voc_max: f32,
    pub voc: f32,
    /// Voltage at maximum power at the highest, the mean and the lowest temperature.
    pub vmp_min: f32,
    pub vmp_mean: f32,
    pub vmp_max: f32,
    /// Current at maximum power at STC, short-circuit current at the highest temperature.
    pub imp: f32,
    pub isc_max: f32,
    /// Power at STC and at the highest temperature.
    pub power: f32,
    pub power_hot: f32,
}

pub fn values(m: &ModuleData, d: &PvDesign, s: &PvString) -> StringValues {
    let (n, p) = (s.series.max(1) as f32, s.parallel.max(1) as f32);
    let at = |v: f32, tc: f32, t: f32| v * (1.0 + tc * (t - 25.0));
    StringValues {
        modules: s.series.max(1) * s.parallel.max(1),
        voc_max: n * at(m.voc, m.tc_voc, d.t_min),
        voc: n * m.voc,
        vmp_min: n * at(m.vmp, m.tc_pmax, d.t_max),
        vmp_mean: n * at(m.vmp, m.tc_pmax, d.t_mean),
        vmp_max: n * at(m.vmp, m.tc_pmax, d.t_min),
        imp: p * m.imp,
        isc_max: p * at(m.isc, m.tc_isc, d.t_max),
        power: n * p * m.pmax,
        power_hot: n * p * at(m.pmax, m.tc_pmax, d.t_max),
    }
}

/// A string where it is, with its values (`None` without module data).
pub struct StringRow<'a> {
    pub building: &'a Building,
    pub roof: &'a Roof,
    pub array: &'a PvArray,
    pub string: &'a PvString,
    pub values: Option<StringValues>,
}

/// All strings of the PV arrays of the plant.
pub fn all_strings(prj: &Project) -> Vec<StringRow<'_>> {
    let d = &prj.layout.pv_design;
    let mut out = Vec::new();
    for b in &prj.layout.buildings {
        for r in &b.roofs {
            for a in &r.solar {
                let m = module_data(prj, &a.module);
                for s in &a.strings {
                    out.push(StringRow { building: b, roof: r, array: a, string: s, values: m.map(|m| values(&m, d, s)) });
                }
            }
        }
    }
    out
}

/// Modules of an array in its strings.
pub fn connected(a: &PvArray) -> u32 {
    a.strings.iter().map(|s| s.series.max(1) * s.parallel.max(1)).sum()
}

/// A string with its numbers in range.
pub fn checked_string(mut s: PvString) -> PvString {
    s.series = s.series.clamp(1, 200);
    s.parallel = s.parallel.clamp(1, 50);
    s.mppt = s.mppt.clamp(1, 64);
    s
}

/// A new string on PV array `ai` of roof `k` of building `bi`: the modules no string
/// takes yet in series, on the first inverter of the project; its id.
pub fn add_string(prj: &mut Project, bi: usize, k: usize, ai: usize) -> Result<u64, String> {
    let inverter = inverters(prj).into_iter().next().unwrap_or_default();
    let id = prj.new_id().max(1);
    let a = prj.layout.buildings.get_mut(bi).and_then(|b| b.roofs.get_mut(k)).and_then(|r| r.solar.get_mut(ai)).ok_or_else(|| tr("No such PV array").to_string())?;
    let free = super::module_count(a).saturating_sub(connected(a));
    let n = a.strings.len() + 1;
    let name = (n..).map(|k| format!("S{k}")).find(|c| !a.strings.iter().any(|s| &s.name == c)).unwrap_or_default();
    let mppt = a.strings.iter().filter(|s| s.inverter == inverter).map(|s| s.mppt + 1).max().unwrap_or(1);
    a.strings.push(checked_string(PvString { id, name, series: free.max(1), parallel: 1, inverter, mppt }));
    Ok(id)
}

fn v(x: f32) -> String {
    format!("{x:.0}")
}

fn amps(x: f32) -> String {
    format!("{x:.1}")
}

/// Strings that do not fit their modules or inverters: more or fewer modules than the
/// array has, modules without electrical data, voltages above the system voltage of the
/// modules or the greatest input voltage of the inverter, outside the MPP range of its
/// tracker, trackers it does not have, currents above what a tracker takes and strings of
/// different voltages on one tracker.
pub fn problems(prj: &Project) -> Vec<Issue> {
    let mut out = Vec::new();
    let d = &prj.layout.pv_design;
    for b in &prj.layout.buildings {
        for a in b.roofs.iter().flat_map(|r| &r.solar).filter(|a| !a.strings.is_empty()) {
            let (n, have) = (connected(a), super::module_count(a));
            if n > have {
                out.push(crate::issue!(Severity::Error, None, "{}: the strings of PV array {} connect {} modules, the array has {}", b.name, a.name, n, have));
            } else if n < have {
                out.push(crate::issue!(Severity::Warning, None, "{}: {} modules of PV array {} are not connected to a string", b.name, have - n, a.name));
            }
            if module_data(prj, &a.module).is_none() {
                out.push(crate::issue!(Severity::Warning, None, "{}: the module {} of PV array {} has no electrical data", b.name, a.module, a.name));
            }
        }
    }
    // Each string against its modules and its inverter, then each tracker.
    let mut trackers: BTreeMap<(String, u32), Vec<(String, StringValues)>> = BTreeMap::new();
    for row in all_strings(prj) {
        let (b, a, s) = (row.building, row.array, row.string);
        let Some(val) = row.values else { continue };
        let max_system = module_data(prj, &a.module).and_then(|m| m.max_system);
        if let Some(max) = max_system.filter(|m| val.voc_max > *m) {
            out.push(crate::issue!(Severity::Error, None, "{}: string {} of PV array {} reaches {} V at {} °C, more than the {} V system voltage of its modules", b.name, s.name, a.name, v(val.voc_max), v(d.t_min), v(max)));
        }
        if s.inverter.is_empty() {
            out.push(crate::issue!(Severity::Warning, None, "{}: string {} of PV array {} is connected to no inverter", b.name, s.name, a.name));
            continue;
        }
        let Some(input) = pv_input(prj, &s.inverter) else {
            out.push(crate::issue!(Severity::Warning, None, "{}: string {} of PV array {}: {} is no device with PV input data", b.name, s.name, a.name, s.inverter));
            continue;
        };
        if s.mppt > input.trackers {
            out.push(crate::issue!(Severity::Error, None, "{}: string {} of PV array {}: {} has only {} MPP trackers", b.name, s.name, a.name, s.inverter, input.trackers));
        }
        if let Some(max) = input.max_voltage.filter(|m| val.voc_max > *m) {
            out.push(crate::issue!(Severity::Error, None, "{}: string {} of PV array {} reaches {} V at {} °C, more than the {} V {} takes", b.name, s.name, a.name, v(val.voc_max), v(d.t_min), v(max), s.inverter));
        }
        if let Some((lo, hi)) = input.mppt_range {
            if val.vmp_min < lo {
                out.push(crate::issue!(Severity::Warning, None, "{}: string {} of PV array {} drops to {} V at {} °C, below the MPP range of {} ({} - {} V)", b.name, s.name, a.name, v(val.vmp_min), v(d.t_max), s.inverter, v(lo), v(hi)));
            }
            if val.vmp_max > hi {
                out.push(crate::issue!(Severity::Warning, None, "{}: string {} of PV array {} reaches {} V at maximum power at {} °C, above the MPP range of {} ({} - {} V)", b.name, s.name, a.name, v(val.vmp_max), v(d.t_min), s.inverter, v(lo), v(hi)));
            }
        }
        trackers.entry((s.inverter.clone(), s.mppt)).or_default().push((format!("{} {}", a.name, s.name), val));
    }
    for ((tag, mppt), strings) in &trackers {
        let Some(input) = pv_input(prj, tag) else { continue };
        let isc: f32 = strings.iter().map(|(_, v)| v.isc_max).sum();
        let imp: f32 = strings.iter().map(|(_, v)| v.imp).sum();
        if let Some(max) = input.max_isc.filter(|m| isc > *m + 0.05) {
            out.push(crate::issue!(Severity::Error, None, "{}, MPP tracker {}: the strings reach a short-circuit current of {} A, more than the {} A it takes", tag, mppt, amps(isc), amps(max)));
        }
        if let Some(max) = input.max_current.filter(|m| imp > *m + 0.05) {
            out.push(crate::issue!(Severity::Warning, None, "{}, MPP tracker {}: the strings deliver {} A, more than the {} A it uses (the power is limited)", tag, mppt, amps(imp), amps(max)));
        }
        let (lo, hi) = strings.iter().fold((f32::MAX, f32::MIN), |(l, h), (_, v)| (l.min(v.vmp_mean), h.max(v.vmp_mean)));
        if strings.len() > 1 && hi > lo * (1.0 + MISMATCH) {
            out.push(crate::issue!(Severity::Warning, None, "{}, MPP tracker {}: its strings differ in voltage ({} V and {} V)", tag, mppt, v(lo), v(hi)));
        }
    }
    out
}

#[cfg(test)]
mod tests;
