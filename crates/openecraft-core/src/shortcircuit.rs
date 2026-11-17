//! Short-circuit currents and protection coordination of the mains (IEC 60909-0,
//! IEC 60364-4-41, -4-43) for radial networks: the source from the short-circuit
//! currents at the supply point or from the transformer and the network upstream,
//! the cables from the supply to each protective device and on to the end of its
//! circuit, the contribution of asynchronous motors. Checked are the highest
//! short-circuit current and its peak at each device against the breaking and making
//! capacity (or the back-up protection of the device upstream), the lowest fault
//! current at the end of its circuit against the disconnection time of table 41.1
//! (second fault in IT systems), the energy let through against the short-circuit
//! withstand of the cables (k²S²) and the selectivity of each device with the one
//! upstream: from the manufacturer's tables, the pre-arcing energy of fuses or the
//! time-current curves.

use openecraft_i18n::{tr, tr_fmt};
use crate::electrical::{circuits, Circuit, Coordination, Earthing, Supply};
use crate::model::*;
use crate::netlist::{Connection, Netlist};
use crate::parts::FnKind;
use crate::protection::{Protection, NO_TRIP};
use crate::issue::{Issue, Severity};
use std::collections::{BTreeMap, BTreeSet};

/// Voltage factors of IEC 60909-0 for low voltage (tolerance ±10 %).
const C_MAX: f32 = 1.1;
const C_MIN: f32 = 0.95;
/// Reactance of a conductor, Ω/m.
const X_CABLE: f32 = 0.08e-3;
/// Locked-rotor to rated current and peak factor of low-voltage motors
/// (IEC 60909-0, 3.8.2: ILR / IrM = 5, RM / XM = 0.42).
const MOTOR_ILR: f32 = 5.0;
const KAPPA_MOTOR: f32 = 1.3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Z {
    r: f32,
    x: f32,
}

impl Z {
    fn abs(self) -> f32 {
        self.r.hypot(self.x)
    }

    fn add(self, o: Z) -> Z {
        Z { r: self.r + o.r, x: self.x + o.x }
    }

    fn scale(self, f: f32) -> Z {
        Z { r: self.r * f, x: self.x * f }
    }

    /// Impedance of magnitude `abs` with this R/X.
    fn with_rx(abs: f32, rx: f32) -> Z {
        let x = abs / (1.0 + rx * rx).sqrt();
        Z { r: x * rx, x }
    }

    /// Peak factor κ (IEC 60909-0, equation 55).
    fn kappa(self) -> f32 {
        if self.x <= 0.0 {
            return 1.02;
        }
        1.02 + 0.98 * (-3.0 * self.r / self.x).exp()
    }
}

/// Impedances of the supply at the incoming terminals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Source {
    /// Positive-sequence impedance (highest short-circuit current), Ω.
    z: Z,
    /// Line-to-PE loop impedance (lowest fault current), Ω.
    z_loop: Z,
    /// Highest three-phase short-circuit current, A.
    pub ik3: f32,
    /// Lowest line-to-PE fault current, A.
    pub ik1: f32,
}

impl Source {
    pub fn of(s: &Supply) -> Source {
        let (z, z_loop) = match &s.transformer {
            Some(t) if t.power > 0.0 && t.uk > 0.0 => {
                let un2 = s.voltage * s.voltage;
                let zq = if t.network > 0.0 { C_MAX * un2 / (t.network * 1e6) } else { 0.0 };
                let zq = Z { r: 0.0995 * zq, x: 0.995 * zq };
                let sr = t.power * 1000.0;
                let zt = t.uk / 100.0 * un2 / sr;
                let rt = (t.load_losses() * un2 / (sr * sr)).min(zt);
                let zt = Z { r: rt, x: (zt * zt - rt * rt).max(0.0).sqrt() };
                // Correction factor of the transformer (equation 12a).
                let kt = 0.95 * C_MAX / (1.0 + 0.6 * zt.x / (un2 / sr));
                // Zero sequence of a Dyn transformer about its positive sequence; the
                // network upstream has none.
                (zq.add(zt.scale(kt)), zq.add(zt).scale(2.0).add(zt).scale(1.0 / 3.0))
            }
            _ => {
                let rx = if s.rx > 0.0 { s.rx } else { 0.1 };
                let z = Z::with_rx(C_MAX * s.voltage / (3f32.sqrt() * s.ik_max.max(0.01) * 1000.0), rx);
                (z, if s.ik_min > 0.0 { Z::with_rx(C_MIN * s.u0() / (s.ik_min * 1000.0), rx) } else { z.scale(2.0) })
            }
        };
        let ik3 = match &s.transformer {
            Some(t) if t.power > 0.0 && t.uk > 0.0 => C_MAX * s.voltage / (3f32.sqrt() * z.abs()),
            _ => s.ik_max.max(0.01) * 1000.0,
        };
        Source { z, z_loop, ik3, ik1: C_MIN * s.u0() / z_loop.abs() }
    }
}

/// Longest disconnection time of a final circuit (IEC 60364-4-41, table 41.1, AC) by
/// the line-to-earth voltage.
pub fn final_disconnection(u0: f32, tt: bool) -> f32 {
    let band = [132.0, 253.0, 440.0].iter().filter(|b| u0 > **b).count();
    if tt {
        [0.3, 0.2, 0.07, 0.04][band]
    } else {
        [0.8, 0.4, 0.2, 0.1][band]
    }
}

/// Short-circuit data of a circuit.
#[derive(Clone, Debug, PartialEq)]
pub struct Fault {
    pub device: String,
    pub target: (usize, u64),
    pub protection: Protection,
    /// Highest short-circuit current at the device, A.
    pub ik_max: f32,
    /// Peak short-circuit current at the device, A.
    pub ip: f32,
    /// Part of `ik_max` fed by motors, A.
    pub ik_motors: f32,
    /// Cables from the supply to the device.
    pub feeders: Vec<String>,
    /// Cable at the end of the circuit (highest impedance), empty: none.
    pub cable: String,
    pub cross_section: f32,
    /// Highest short-circuit current at the end of the circuit, A.
    pub ik_max_end: f32,
    /// Lowest fault current at the end of the circuit (line to PE; IT system: the
    /// second fault), A.
    pub ik_min_end: f32,
    /// IT system: the disconnection is checked for the second fault.
    pub second_fault: bool,
    /// The device upstream protects this one where its breaking capacity is too low
    /// (manufacturer's table).
    pub backup: Option<String>,
    /// Longest permitted disconnection time, s.
    pub limit: Option<f32>,
    /// Current that disconnects within the limit, A.
    pub trip_current: f32,
    /// Longest disconnection time at the lowest fault current, s.
    pub time: f32,
    /// Rated residual current of a residual current device protecting the circuit, mA.
    pub residual: Option<f32>,
    /// TT system without a residual current device: the earth fault loop is unknown.
    pub needs_rcd: bool,
    /// Energy let through and what the end cable withstands (k²S²), A²s.
    pub energy: Option<(f32, f32)>,
}

impl Fault {
    /// Breaking capacity at least the short-circuit current, or back-up protection
    /// by the device upstream (`None`: no data).
    pub fn breaks(&self) -> Option<bool> {
        if self.backup.is_some() {
            return Some(true);
        }
        self.protection.breaking.map(|b| b * 1000.0 + 0.5 >= self.ik_max)
    }

    /// Making capacity at least the peak short-circuit current (`None`: no data).
    pub fn makes(&self) -> Option<bool> {
        if self.backup.is_some() {
            return Some(true);
        }
        self.protection.making.map(|m| m * 1000.0 + 0.5 >= self.ip)
    }

    pub fn disconnects(&self) -> bool {
        !self.needs_rcd && (self.limit.is_none() || self.residual.is_some() || self.limit.is_some_and(|l| self.time <= l))
    }

    pub fn withstands(&self) -> bool {
        self.energy.is_none_or(|(e, k)| e <= k)
    }

    /// All checks pass.
    pub fn is_ok(&self) -> bool {
        self.breaks() != Some(false) && self.makes() != Some(false) && self.disconnects() && self.withstands()
    }
}

/// Cable runs in series: Σ length / mm² and Σ length, m.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Run {
    per_mm2: f32,
    length: f32,
}

impl Run {
    fn of(cb: &Cable, mm2: f32) -> Run {
        Run { per_mm2: cb.length / mm2, length: cb.length }
    }

    fn add(self, o: Run) -> Run {
        Run { per_mm2: self.per_mm2 + o.per_mm2, length: self.length + o.length }
    }

    /// Impedance of one conductor at the resistivity `rho`.
    fn z(self, rho: f32) -> Z {
        Z { r: rho * self.per_mm2, x: X_CABLE * self.length }
    }
}

/// Cables with a core on one of the nets, with their cross-section (mm²).
fn cables_at<'a>(prj: &'a Project, nl: &Netlist, conns: &[Connection], nets: &BTreeSet<usize>) -> Vec<(&'a Cable, f32)> {
    let mut tags: Vec<&str> = conns
        .iter()
        .filter(|k| !k.cable.is_empty())
        .filter(|k| [k.a, k.b].iter().any(|p| nl.pin_net.get(&(prj.pages[p.page].symbols[p.sym].id, p.pin)).is_some_and(|n| nets.contains(n))))
        .map(|k| k.cable.as_str())
        .collect();
    tags.sort();
    tags.dedup();
    tags.into_iter()
        .filter_map(|t| prj.cable(t))
        .filter_map(|cb| crate::production::cross_section_mm2(&cb.cross_section).filter(|a| *a > 0.0 && cb.length > 0.0).map(|a| (cb, a)))
        .collect()
}

/// The cable with the highest impedance.
fn longest<'a>(cables: &[(&'a Cable, f32)]) -> Option<(&'a Cable, f32)> {
    cables.iter().copied().max_by(|a, b| (a.0.length / a.1).total_cmp(&(b.0.length / b.1)))
}

/// Cables from the supply to each protective device: the cable of the circuit
/// upstream that reaches its line side, and so on up to the supply.
fn feeders<'a>(prj: &'a Project, nl: &Netlist, conns: &[Connection], cs: &[Circuit]) -> Vec<(Run, Vec<&'a Cable>)> {
    let behind: Vec<Vec<(&Cable, f32)>> = cs.iter().map(|c| cables_at(prj, nl, conns, &c.nets)).collect();
    let own: Vec<Option<(&Cable, f32)>> = cs
        .iter()
        .map(|c| {
            let up = &behind[c.parent?];
            let line: Vec<(&Cable, f32)> = cables_at(prj, nl, conns, &c.line).into_iter().filter(|x| up.iter().any(|u| u.0.tag == x.0.tag)).collect();
            longest(&line)
        })
        .collect();
    (0..cs.len())
        .map(|i| {
            let (mut run, mut tags) = (Run::default(), Vec::new());
            let (mut at, mut depth) = (i, 0);
            while let (Some(up), true) = (cs[at].parent, depth < cs.len()) {
                if let Some((cb, mm2)) = own[at] {
                    run = run.add(Run::of(cb, mm2));
                    tags.insert(0, cb);
                }
                at = up;
                depth += 1;
            }
            (run, tags)
        })
        .collect()
}

/// A cable on the way from the supply to a protective device: its tag, length (m),
/// cross-section (mm²), the protective device upstream feeding through it and the
/// number of phases it carries.
#[derive(Clone, Debug, PartialEq)]
pub struct FeederCable {
    pub tag: String,
    pub length: f32,
    pub mm2: f32,
    pub device: String,
    pub phases: usize,
}

/// Reactance of a cable per metre, Ω (IEC 60909: 0.08 mΩ/m).
pub const CABLE_REACTANCE: f32 = X_CABLE;

/// The cables from the supply to each protective device of the mains (by its tag),
/// the one nearest the supply first.
pub fn feeder_cables(prj: &Project, nl: &Netlist, conns: &[Connection]) -> std::collections::HashMap<String, Vec<FeederCable>> {
    let cs = crate::electrical::circuits(prj, nl);
    let behind: Vec<Vec<(&Cable, f32)>> = cs.iter().map(|c| cables_at(prj, nl, conns, &c.nets)).collect();
    let mut out = std::collections::HashMap::new();
    for i in 0..cs.len() {
        let (mut at, mut depth, mut chain) = (i, 0, Vec::new());
        while let (Some(up), true) = (cs[at].parent, depth < cs.len()) {
            let line: Vec<(&Cable, f32)> = cables_at(prj, nl, conns, &cs[at].line).into_iter().filter(|x| behind[up].iter().any(|u| u.0.tag == x.0.tag)).collect();
            if let Some((cb, mm2)) = longest(&line) {
                chain.insert(0, FeederCable { tag: cb.tag.clone(), length: cb.length, mm2, device: cs[up].device.clone(), phases: cs[up].phases.len() });
            }
            at = up;
            depth += 1;
        }
        out.insert(cs[i].device.clone(), chain);
    }
    out
}

/// Locked-rotor current of a motor (`Locked rotor current: 7.2 x In` or in A).
fn locked_rotor(prj: &Project, tag: &str, rated: f32) -> f32 {
    crate::sizing::device_parts(prj, tag)
        .iter()
        .find_map(|p| {
            let v = p.tech_value("Locked rotor current")?;
            crate::electrical::value_with_unit(v, "A").filter(|a| *a > rated).or_else(|| first_value(v).map(|m| m * rated))
        })
        .unwrap_or(MOTOR_ILR * rated)
}

fn first_value(text: &str) -> Option<f32> {
    let start = text.find(|c: char| c.is_ascii_digit())?;
    text[start..].chars().take_while(|c| c.is_ascii_digit() || matches!(c, '.' | ',')).collect::<String>().replace(',', ".").parse().ok()
}

/// Short-circuit current fed by the asynchronous motors on the mains (IEC 60909-0,
/// 3.8.2: neglected up to 1 % of the current from the supply), A.
fn motor_current(prj: &Project, nl: &Netlist, ik3: f32) -> f32 {
    if !prj.electrical.supply.motors {
        return 0.0;
    }
    let motors: Vec<(f32, f32)> = crate::loads::calculate(prj, nl)
        .loads
        .iter()
        .filter(|l| matches!(l.kind, FnKind::Motor3 | FnKind::SoftStarter))
        .map(|l| (l.current, locked_rotor(prj, &l.device, l.current)))
        .collect();
    let rated: f32 = motors.iter().map(|m| m.0).sum();
    if rated <= 0.01 * ik3 {
        return 0.0;
    }
    C_MAX * motors.iter().map(|m| m.1).sum::<f32>()
}

/// Entry of the manufacturer's tables for two devices.
pub fn coordination<'a>(table: &'a [Coordination], up: &Protection, down: &Protection) -> Option<&'a Coordination> {
    (!up.part.is_empty() && !down.part.is_empty()).then(|| table.iter().find(|c| c.upstream.trim() == up.part && c.downstream.trim() == down.part)).flatten()
}

/// Short-circuit data of every circuit of the mains.
pub fn faults(prj: &Project, nl: &Netlist, conns: &[Connection]) -> Vec<Fault> {
    let el = &prj.electrical;
    let s = &el.supply;
    let limits = crate::rules::limits(prj);
    let src = Source::of(s);
    let rho20 = el.conditions.material.rho20();
    let rho_hot = el.conditions.rho_hot();
    let k = el.conditions.k();
    let motors = motor_current(prj, nl, src.ik3);
    let cs = circuits(prj, nl);
    let feeds = feeders(prj, nl, conns, &cs);
    let ik_at = |z: Z| src.ik3 * src.z.abs() / z.abs();
    cs.iter()
        .zip(&feeds)
        .map(|(c, (feed, feed_cables))| {
            let end = longest(&cables_at(prj, nl, conns, &c.nets));
            let (cable, mm2) = end.map(|(cb, a)| (cb.tag.clone(), a)).unwrap_or_default();
            let end_run = end.map(|(cb, a)| Run::of(cb, a)).unwrap_or_default();
            let z_device = src.z.add(feed.z(rho20));
            let from_supply = ik_at(z_device);
            let ik_max = from_supply + motors;
            let ip = 2f32.sqrt() * (z_device.kappa() * from_supply + KAPPA_MOTOR * motors);
            let ik_max_end = ik_at(z_device.add(end_run.z(rho20))) + motors;
            // Phase and protective conductor of the cables (same cross-section).
            let cable_loop = feed.add(end_run).z(rho_hot).scale(2.0);
            let second_fault = s.earthing == Earthing::It;
            let ik_min_end = if second_fault {
                // Two faults on different phases in circuits alike (411.6.4).
                let u = if s.neutral { s.u0() } else { s.voltage };
                C_MIN * u / cable_loop.add(src.z).scale(2.0).abs()
            } else {
                C_MIN * s.u0() / src.z_loop.add(cable_loop).abs()
            };
            let tt = s.earthing == Earthing::Tt;
            let limit = if c.is_final() {
                limits.disconnect_final.min(final_disconnection(s.u0(), tt))
            } else if tt {
                limits.disconnect_distribution.min(1.0)
            } else {
                limits.disconnect_distribution
            };
            let p = &c.protection;
            let trip_current = p.current_for(limit);
            let time = p.trip_time(ik_min_end).1;
            // The cable must withstand what the device lets through at the lowest and
            // the highest fault current.
            let energy = (mm2 > 0.0)
                .then(|| [ik_min_end, ik_max_end].iter().filter_map(|&i| p.energy(i)).reduce(f32::max))
                .flatten()
                .map(|e| (e, (k * mm2).powi(2)));
            let backup = c
                .parent
                .map(|u| &cs[u])
                .filter(|u| coordination(&el.coordination, &u.protection, p).is_some_and(|x| x.backup * 1000.0 + 0.5 >= ik_max))
                .map(|u| u.device.clone());
            Fault {
                device: c.device.clone(),
                target: (c.page, c.symbol),
                protection: c.protection.clone(),
                ik_max,
                ip,
                ik_motors: motors,
                feeders: feed_cables.iter().map(|cb| cb.tag.clone()).collect(),
                cable,
                cross_section: mm2,
                ik_max_end,
                ik_min_end,
                second_fault,
                backup,
                limit: Some(limit),
                trip_current,
                time,
                residual: c.residual,
                needs_rcd: tt && c.residual.is_none(),
                energy,
            }
        })
        .collect()
}

/// What the selectivity of two devices is based on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Basis {
    /// Manufacturer's selectivity table.
    Table,
    /// Fuses with a rated current ratio of at least 1.6.
    FuseRatio,
    /// The energy let through downstream stays below the pre-arcing energy of the
    /// fuse upstream.
    Energy,
    /// Time-current bands.
    Curves,
}

impl Basis {
    pub fn title(self) -> &'static str {
        match self {
            Basis::Table => tr("Manufacturer's table"),
            Basis::FuseRatio => tr("Fuse ratio 1.6"),
            Basis::Energy => tr("Pre-arcing energy"),
            Basis::Curves => tr("Time-current curves"),
        }
    }
}

/// Selectivity of a protective device with the one upstream.
#[derive(Clone, Debug, PartialEq)]
pub struct Selectivity {
    pub upstream: String,
    pub downstream: String,
    pub target: (usize, u64),
    /// The devices are selective up to this current, A.
    pub limit: f32,
    /// Highest short-circuit current at the downstream device, A.
    pub ik: f32,
    pub basis: Basis,
}

impl Selectivity {
    pub fn is_total(&self) -> bool {
        self.limit >= self.ik
    }

    /// Not even overloads of the downstream device are selective.
    pub fn is_none(&self, down: &Protection) -> bool {
        self.limit < down.rated * 2.0
    }
}

/// The downstream device trips before the upstream device at the current `i`.
fn selective_at(up: &Protection, down: &Protection, i: f32) -> bool {
    let (up_min, _) = up.trip_time(i);
    let (_, down_max) = down.trip_time(i);
    (up_min == NO_TRIP && down_max == NO_TRIP) || up_min > down_max
}

/// Current up to which two devices are selective (`ik` when they are throughout).
pub fn selectivity_limit(up: &Protection, down: &Protection, ik: f32) -> f32 {
    // Fuses with a rated current ratio of 1.6 are selective (IEC 60269-1, gG).
    if up.characteristic.is_fuse() && down.characteristic.is_fuse() && up.rated >= 1.6 * down.rated - 1e-3 {
        return ik;
    }
    let start = down.rated * 1.05;
    if ik <= start {
        return ik;
    }
    let steps = 400;
    let f = (ik / start).powf(1.0 / steps as f32);
    let mut i = start;
    let mut last = start;
    for _ in 0..=steps {
        if !selective_at(up, down, i) {
            return last;
        }
        last = i;
        i *= f;
    }
    ik
}

/// Selectivity of two devices up to the short-circuit current `ik`: the
/// manufacturer's table, fuses by their ratio, the pre-arcing energy of a fuse
/// upstream, otherwise the time-current curves.
pub fn coordinate(table: &[Coordination], up: &Protection, down: &Protection, ik: f32) -> (f32, Basis) {
    if let Some(c) = coordination(table, up, down).filter(|c| c.selectivity > 0.0) {
        return ((c.selectivity * 1000.0).min(ik), Basis::Table);
    }
    let limit = selectivity_limit(up, down, ik);
    if up.characteristic.is_fuse() && down.characteristic.is_fuse() && limit >= ik && ik > down.rated * 1.05 {
        return (limit, Basis::FuseRatio);
    }
    if limit < ik && up.pre_arcing.is_some_and(|pa| down.energy(ik).is_some_and(|e| e <= pa)) {
        return (ik, Basis::Energy);
    }
    (limit, Basis::Curves)
}

/// Selectivity of every protective device with the device upstream.
pub fn selectivity(prj: &Project, nl: &Netlist, conns: &[Connection]) -> Vec<Selectivity> {
    let cs = circuits(prj, nl);
    let ik: BTreeMap<String, f32> = faults(prj, nl, conns).into_iter().map(|f| (f.device, f.ik_max)).collect();
    cs.iter()
        .filter_map(|c| {
            let up = &cs[c.parent?];
            let ik = ik.get(&c.device).copied().unwrap_or(0.0);
            let (limit, basis) = coordinate(&prj.electrical.coordination, &up.protection, &c.protection, ik);
            Some(Selectivity { upstream: up.device.clone(), downstream: c.device.clone(), target: (c.page, c.symbol), limit, ik, basis })
        })
        .collect()
}

fn ka(a: f32) -> String {
    format!("{:.2}", a / 1000.0)
}

fn amps(a: f32) -> String {
    if a.is_finite() {
        format!("{a:.0}")
    } else {
        "–".into()
    }
}

/// Check messages: breaking capacity, disconnection, withstand, selectivity.
pub fn problems(prj: &Project, nl: &Netlist, conns: &[Connection]) -> Vec<Issue> {
    let limits = crate::rules::limits(prj);
    let mut out = Vec::new();
    let faults = faults(prj, nl, conns);
    let mut without = 0;
    for f in &faults {
        match f.breaks() {
            Some(false) => out.push(crate::issue!(Severity::Error, Some(f.target), "{}: breaking capacity {} kA below the short-circuit current {} kA", f.device, format!("{}", f.protection.breaking.unwrap_or(0.0)), ka(f.ik_max))),
            None => without += 1,
            _ => {}
        }
        if f.makes() == Some(false) {
            out.push(crate::issue!(Severity::Error, Some(f.target), "{}: making capacity {} kA below the peak short-circuit current {} kA", f.device, format!("{}", f.protection.making.unwrap_or(0.0)), ka(f.ip)));
        }
        if f.needs_rcd {
            out.push(crate::issue!(Severity::Error, Some(f.target), "{}: a TT system needs a residual current device", f.device));
        } else if !f.disconnects() {
            let at = if f.cable.is_empty() { f.device.clone() } else { f.cable.clone() };
            let limit = format!("{}", f.limit.unwrap_or(0.0));
            out.push(if f.second_fault {
                crate::issue!(Severity::Error, Some(f.target), "{}: second fault current {} A at {} does not disconnect within {} s (needs {} A)", f.device, amps(f.ik_min_end), at, limit, amps(f.trip_current))
            } else {
                crate::issue!(Severity::Error, Some(f.target), "{}: fault current {} A at {} does not disconnect within {} s (needs {} A)", f.device, amps(f.ik_min_end), at, limit, amps(f.trip_current))
            });
        }
        if let Some((e, k)) = f.energy.filter(|_| !f.withstands()) {
            out.push(crate::issue!(Severity::Error, Some(f.target), "{}: lets through {} A²s, cable {} withstands {} A²s", f.device, format!("{e:.0}"), f.cable, format!("{k:.0}")));
        }
    }
    if without > 0 {
        out.push(crate::issue!(Severity::Info, None, "{} protective devices of the mains have no breaking capacity data", without));
    }
    let downs: BTreeMap<&str, &Protection> = faults.iter().map(|f| (f.device.as_str(), &f.protection)).collect();
    for s in selectivity(prj, nl, conns) {
        if s.is_total() {
            continue;
        }
        let none = downs.get(s.downstream.as_str()).is_some_and(|p| s.is_none(p));
        let severity = if none || limits.total_selectivity { Severity::Warning } else { Severity::Info };
        let message = if none {
            tr_fmt!("{} and {} are not selective", s.upstream, s.downstream)
        } else {
            tr_fmt!("{} is selective to {} only up to {} A (short-circuit current {} A)", s.downstream, s.upstream, amps(s.limit), amps(s.ik))
        };
        out.push(Issue::plain(severity, message, Some(s.target)));
    }
    out
}

#[cfg(test)]
mod tests;
