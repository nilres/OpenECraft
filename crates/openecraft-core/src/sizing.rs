//! Electrical sizing: current-carrying capacity of the wires behind protective
//! devices (IEC 60204-1, table 6, installation method B2, copper, PVC, 40 °C,
//! corrected for the installation conditions of the project) and the voltage drop of
//! cables to their loads. Rated currents come from the parts (`Current` field, e.g.
//! "16 A" or the setting range "2.5 - 4 A"); fuse bases and fuse switch disconnectors
//! protect at the rated current of the fuse links put into them ([`FUSE_LINK`]).

use openecraft_i18n::tr_fmt;
use crate::model::*;
use crate::netlist::{Connection, Netlist};
use crate::parts::{kind_of_symbol, FnKind, Part};
use crate::potentials::props_of;
use crate::issue::{Issue, Severity};
use crate::symbols::{ContactKind, Func};
use std::collections::{BTreeSet, HashSet};

/// Current-carrying capacity (A) per cross-section (mm²), IEC 60204-1 table 6, B2.
const AMPACITY: [(f32, f32); 14] = [
    (0.75, 7.6),
    (1.0, 10.4),
    (1.5, 13.5),
    (2.5, 18.3),
    (4.0, 25.0),
    (6.0, 32.0),
    (10.0, 44.0),
    (16.0, 60.0),
    (25.0, 77.0),
    (35.0, 97.0),
    (50.0, 117.0),
    (70.0, 149.0),
    (95.0, 180.0),
    (120.0, 208.0),
];

/// Highest voltage drop from the supply to a load (IEC 60204-1, 13.5): 5 %.
pub const VOLTAGE_DROP_LIMIT: f32 = 5.0;

/// Current-carrying capacity of a copper cross-section; `None` above the table.
/// Smaller sections than the table are scaled down from 0.75 mm².
pub fn ampacity(mm2: f32) -> Option<f32> {
    if mm2 <= 0.0 {
        return None;
    }
    if mm2 < AMPACITY[0].0 {
        return Some(AMPACITY[0].1 * mm2 / AMPACITY[0].0);
    }
    AMPACITY.iter().rev().find(|(cs, _)| mm2 >= *cs - 0.001).filter(|_| mm2 <= AMPACITY[AMPACITY.len() - 1].0 + 0.001).map(|x| x.1)
}

/// Current-carrying capacity under the installation conditions of the project.
pub fn capacity(prj: &Project, mm2: f32) -> Option<f32> {
    ampacity(mm2).map(|i| i * prj.electrical.conditions.factor())
}

/// Rated current from a current text: the largest value given in A ("2.5 - 4 A" → 4,
/// "9 A (AC-3)" → 9).
pub fn parse_current(text: &str) -> Option<f32> {
    let chars: Vec<char> = text.chars().collect();
    let mut best: Option<f32> = None;
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() && (i == 0 || !chars[i - 1].is_alphanumeric()) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == ',') {
                i += 1;
            }
            let num: String = chars[start..i].iter().collect();
            let mut j = i;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            let unit = j < chars.len() && chars[j] == 'A' && (j + 1 == chars.len() || !chars[j + 1].is_alphabetic());
            if let (true, Ok(v)) = (unit, num.replace(',', ".").parse::<f32>()) {
                best = Some(best.map_or(v, |b: f32| b.max(v)));
            }
        } else {
            i += 1;
        }
    }
    best
}

/// First voltage value of a text ("400 V AC" → 400, "230 V AC / 24 V DC" → 230).
pub fn parse_voltage(text: &str) -> Option<f32> {
    let start = text.find(|c: char| c.is_ascii_digit())?;
    let num: String = text[start..].chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',').collect();
    let rest = text[start + num.len()..].trim_start();
    if !rest.starts_with('V') {
        return None;
    }
    num.replace(',', ".").parse().ok()
}

pub(crate) fn part<'a>(prj: &'a Project, number: &str) -> Option<&'a Part> {
    prj.parts.iter().chain(crate::parts::catalog()).find(|p| p.number == number)
}

pub(crate) fn device_parts<'a>(prj: &'a Project, tag: &str) -> Vec<&'a Part> {
    prj.devices.get(tag).map(|d| d.parts.iter().filter_map(|r| part(prj, &r.number)).collect()).unwrap_or_default()
}

pub(crate) const PROTECTIVE: [FnKind; 7] =
    [FnKind::Breaker1, FnKind::Breaker2, FnKind::Breaker3, FnKind::Breaker4, FnKind::MotorProtection, FnKind::Fuse, FnKind::Fuse3];

pub(crate) fn is_protective(key: &str) -> bool {
    kind_of_symbol(key).is_some_and(|k| PROTECTIVE.contains(&k))
}

/// The current passes through these from one designation to the next (1 → 2).
pub(crate) fn passes_current(prj: &Project, s: &SymbolInst) -> bool {
    matches!(
        kind_of_symbol(&s.sym),
        Some(FnKind::MainContacts3 | FnKind::MainContacts4 | FnKind::MainContact | FnKind::Overload | FnKind::Disconnector3 | FnKind::ResidualCurrent)
    )
        || prj.symbol(&s.sym).is_some_and(|d| d.func == Func::Aux(ContactKind::MainNo))
        || (is_protective(&s.sym) && isolates_only(prj, &s.tag))
}

/// Subgroup of the fuse links (NH, D, cylindrical) put into fuse bases and fuse switch
/// disconnectors.
pub const FUSE_LINK: &str = "Fuse link";

/// Subgroup of the solid links put into fuse bases and fuse switch disconnectors in
/// place of fuse links: the device isolates (the isolation of a transformer metering),
/// it does not protect.
pub const SOLID_LINK: &str = "Solid link";

/// A fuse base or fuse switch disconnector fitted with solid links.
pub fn isolates_only(prj: &Project, tag: &str) -> bool {
    !tag.is_empty() && device_parts(prj, tag).iter().any(|p| p.subgroup == SOLID_LINK)
}

/// The fuse link of a device (the one with the highest rated current), if it has one.
pub fn fuse_link<'a>(prj: &'a Project, tag: &str) -> Option<&'a Part> {
    let current = |p: &Part| parse_current(&p.current).unwrap_or(0.0);
    device_parts(prj, tag).into_iter().filter(|p| p.subgroup == FUSE_LINK && parse_current(&p.current).is_some()).max_by(|a, b| current(a).total_cmp(&current(b)))
}

/// Rated current of a protective device: that of its fuse links, otherwise the largest
/// rated current of its parts that have a protective function.
pub fn protective_current(prj: &Project, tag: &str) -> Option<f32> {
    if isolates_only(prj, tag) {
        return None;
    }
    if let Some(link) = fuse_link(prj, tag) {
        return parse_current(&link.current);
    }
    device_parts(prj, tag)
        .into_iter()
        .filter(|p| p.functions.iter().any(|f| PROTECTIVE.contains(&f.kind)))
        .filter_map(|p| parse_current(&p.current))
        .reduce(f32::max)
}

/// Smallest cross-section of the table that carries `current` under the installation
/// conditions of the project (mm²); `None` above the table.
pub fn cross_section_for(prj: &Project, current: f32) -> Option<f32> {
    AMPACITY.iter().map(|x| x.0).find(|&a| capacity(prj, a).is_some_and(|iz| iz + 0.01 >= current))
}

/// Gives the wires behind protective devices that carry less than the device lets
/// through the smallest cross-section that carries it (wires of cables keep the
/// cross-section of their cable). Returns the number of wires changed.
pub fn size_wires(prj: &mut Project) -> usize {
    let (nl, open) = (Netlist::build(prj, true), Netlist::build(prj, false));
    let mut needed: std::collections::HashMap<usize, f32> = std::collections::HashMap::new();
    for o in overloads(prj, &nl, &open) {
        if let Some(&n) = open.wire_net.get(&o.target.1) {
            let rated = needed.entry(n).or_insert(0.0);
            *rated = rated.max(o.rated);
        }
    }
    let mut changed = 0;
    for (n, rated) in needed {
        let Some(mm2) = cross_section_for(prj, rated) else { continue };
        let cs = format!("{mm2} mm²");
        for &(pi, wid) in &open.nets[n].wires {
            if let Some(w) = prj.pages[pi].wires.iter_mut().find(|w| w.id == wid && w.cable.is_empty()) {
                w.cross_section = cs.clone();
                changed += 1;
            }
        }
    }
    changed
}

pub(crate) fn designation_number(s: &SymbolInst, pin: usize) -> Option<u32> {
    s.pins.get(pin)?.trim().parse().ok()
}

/// Nets (of the netlist with bridged terminals) behind a protective device symbol:
/// from its load side (even designations) through main contacts and overload relays.
pub fn protected_nets(prj: &Project, nl: &Netlist, page: usize, sym: usize) -> BTreeSet<usize> {
    let s = &prj.pages[page].symbols[sym];
    let mut todo: Vec<usize> =
        (0..s.pins.len()).filter(|&i| designation_number(s, i).is_some_and(|d| d % 2 == 0)).filter_map(|i| nl.pin_net.get(&(s.id, i)).copied()).collect();
    let mut seen = BTreeSet::new();
    while let Some(n) = todo.pop() {
        if !seen.insert(n) {
            continue;
        }
        for p in &nl.nets[n].pins {
            let other = &prj.pages[p.page].symbols[p.sym];
            if other.id == s.id || !passes_current(prj, other) {
                continue;
            }
            let Some(d) = designation_number(other, p.pin) else { continue };
            let partner = if d % 2 == 1 { d + 1 } else { d - 1 };
            if let Some(j) = (0..other.pins.len()).find(|&j| designation_number(other, j) == Some(partner)) {
                if let Some(&m) = nl.pin_net.get(&(other.id, j)) {
                    todo.push(m);
                }
            }
        }
    }
    seen
}

/// A wire or cable whose cross-section carries less than its protective device lets through.
#[derive(Clone, Debug, PartialEq)]
pub struct Overload {
    pub device: String,
    pub rated: f32,
    pub cross_section: String,
    pub ampacity: f32,
    pub target: (usize, u64),
}

/// Cross-section of a connection: resolved (wire, potential, default), cable cores
/// from their cable.
fn cross_section_of(prj: &Project, nl: &Netlist, net: &crate::netlist::Net) -> String {
    let props = props_of(prj, nl, net);
    if !props.cross_section.is_empty() {
        return props.cross_section;
    }
    crate::wirenum::wires_of(prj, net).iter().find_map(|w| prj.cable(&w.cable)).map(|c| c.cross_section.clone()).unwrap_or_default()
}

/// Connections behind protective devices whose cross-section is too small for the
/// rated current of the device.
pub fn overloads(prj: &Project, nl: &Netlist, open: &Netlist) -> Vec<Overload> {
    let mut out = Vec::new();
    for (pi, pg) in prj.pages.iter().enumerate().filter(|(_, p)| p.kind == PageKind::Schematic) {
        for (si, s) in pg.symbols.iter().enumerate() {
            if !is_protective(&s.sym) || s.tag.is_empty() {
                continue;
            }
            let Some(rated) = protective_current(prj, &s.tag) else { continue };
            let mut checked: HashSet<usize> = HashSet::new();
            for n in protected_nets(prj, nl, pi, si) {
                for (wp, wid) in &nl.nets[n].wires {
                    let Some(&on) = open.wire_net.get(wid) else { continue };
                    if !checked.insert(on) {
                        continue;
                    }
                    let cs = cross_section_of(prj, nl, &open.nets[on]);
                    let Some(iz) = crate::production::cross_section_mm2(&cs).and_then(|a| capacity(prj, a)) else { continue };
                    if iz + 0.01 < rated {
                        out.push(Overload { device: s.tag.clone(), rated, cross_section: cs, ampacity: iz, target: (*wp, *wid) });
                    }
                }
            }
        }
    }
    out
}

/// Voltage drop of a cable to its load.
#[derive(Clone, Debug, PartialEq)]
pub struct CableDrop {
    pub cable: String,
    pub length: f32,
    pub cross_section: String,
    /// The load at the cable's ends with the highest rated current.
    pub load: String,
    pub current: Option<f32>,
    pub voltage: Option<f32>,
    pub three_phase: bool,
    /// Voltage drop in V and in % of the voltage.
    pub drop: Option<(f32, f32)>,
    pub ampacity: Option<f32>,
}

/// Voltage drop and current-carrying capacity of every cable.
pub fn cable_drops(prj: &Project, conns: &[Connection]) -> Vec<CableDrop> {
    let mut out = Vec::new();
    for cable in &prj.cables {
        let on: Vec<&Connection> = conns.iter().filter(|c| c.cable == cable.tag).collect();
        // The load: the device at the cable's ends with the highest rated current
        // (terminals, plugs, junction boxes, protective devices, meters and current
        // transformers are no loads: the rating of a junction box is that of its
        // terminals).
        let mut load: Option<(String, f32, Option<f32>)> = None;
        let mut tags: Vec<&str> = Vec::new();
        for c in &on {
            for p in [c.a, c.b] {
                let s = &prj.pages[p.page].symbols[p.sym];
                let func = prj.symbol(&s.sym).map(|d| d.func);
                let measuring = matches!(kind_of_symbol(&s.sym), Some(FnKind::Meter | FnKind::CurrentTransformer));
                let junction = matches!(s.sym.as_str(), "jbox" | "jbox3" | "inst_junction");
                if func.is_some_and(|f| f.is_feedthrough()) || is_protective(&s.sym) || measuring || junction || tags.contains(&s.tag.as_str()) {
                    continue;
                }
                tags.push(&s.tag);
                // The parts providing the function at the cable (the motor operator on a
                // circuit breaker, not the breaker), all parts if none does.
                let parts = device_parts(prj, &s.tag);
                let kind = kind_of_symbol(&s.sym);
                let providing: Vec<&Part> = parts.iter().copied().filter(|p| kind.is_some_and(|k| p.functions.iter().any(|f| f.kind == k))).collect();
                for part in if providing.is_empty() { parts } else { providing } {
                    if let Some(i) = parse_current(&part.current) {
                        if load.as_ref().is_none_or(|l| i > l.1) {
                            load = Some((s.tag.clone(), i, parse_voltage(&part.voltage)));
                        }
                    }
                }
            }
        }
        let live: BTreeSet<&str> = on.iter().filter(|c| !crate::potentials::is_protective(prj, &c.net)).map(|c| c.core.as_str()).collect();
        let three_phase = live.len() >= 3;
        let voltage = load.as_ref().and_then(|l| l.2).or_else(|| on.iter().find_map(|c| prj.potential(&c.net).and_then(|d| parse_voltage(&d.voltage))));
        let mm2 = crate::production::cross_section_mm2(&cable.cross_section);
        let current = load.as_ref().map(|l| l.1);
        let drop = match (current, voltage, mm2) {
            (Some(i), Some(u), Some(a)) if cable.length > 0.0 && u > 0.0 => {
                let du = if three_phase { 3f32.sqrt() } else { 2.0 } * i * cable.length * prj.electrical.conditions.rho_hot() / a;
                Some((du, du / u * 100.0))
            }
            _ => None,
        };
        out.push(CableDrop {
            cable: cable.tag.clone(),
            length: cable.length,
            cross_section: cable.cross_section.clone(),
            load: load.as_ref().map(|l| l.0.clone()).unwrap_or_default(),
            current,
            voltage,
            three_phase,
            drop,
            ampacity: mm2.and_then(|a| capacity(prj, a)),
        });
    }
    out
}

/// Check messages: wires and cables too thin (errors), voltage drops above the limit.
pub fn problems(prj: &Project, nl: &Netlist, open: &Netlist, conns: &[Connection]) -> Vec<Issue> {
    let limits = crate::rules::limits(prj);
    let mut v = Vec::new();
    let mut issue = |error: bool, message: String, target| v.push(Issue::plain(if error { Severity::Error } else { Severity::Warning }, message, target));
    for o in overloads(prj, nl, open) {
        let message = tr_fmt!(
            "Cross-section {} carries {} A, less than the {} A of {}",
            o.cross_section,
            format!("{:.1}", o.ampacity),
            format!("{}", o.rated),
            o.device
        );
        issue(true, message, Some(o.target));
    }
    for d in cable_drops(prj, conns) {
        if let (Some(i), Some(iz)) = (d.current, d.ampacity) {
            if iz + 0.01 < i {
                issue(true, tr_fmt!("{}: {} carries {} A, the load {} needs {} A", d.cable, d.cross_section, format!("{iz:.1}"), d.load, format!("{i}")), None);
            }
        }
        if let Some((du, pct)) = d.drop.filter(|x| x.1 > limits.voltage_drop) {
            issue(false, tr_fmt!("{}: voltage drop {} V ({} %) above {} %", d.cable, format!("{du:.1}"), format!("{pct:.1}"), limits.voltage_drop), None);
        }
    }
    v
}

#[cfg(test)]
mod tests;
