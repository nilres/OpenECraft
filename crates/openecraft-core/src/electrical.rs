//! Electrical design data of a project: the supply (voltage, earthing system,
//! short-circuit currents), the installation conditions of the conductors, the
//! safety functions and the rule libraries / customer rule sets the project is
//! verified against. Also the circuits of the mains: the protective devices fed
//! from the supply, the nets behind them and the device upstream of each.

use openecraft_i18n::tr;
use crate::model::*;
use crate::netlist::Netlist;
use crate::parts::{kind_of_symbol, FnKind};
use crate::potentials::PotentialKind;
use crate::protection::Protection;
use crate::sizing::{designation_number, is_protective, passes_current, protected_nets};
use std::collections::{BTreeMap, BTreeSet};
pub use crate::model::electrical::{Conditions, Coordination, Earthing, Electrical, Insulation, Material, Supply, Transformer};

impl Earthing {
    pub const ALL: [Earthing; 5] = [Earthing::TnS, Earthing::TnC, Earthing::TnCS, Earthing::Tt, Earthing::It];

    pub fn title(self) -> &'static str {
        match self {
            Earthing::TnS => "TN-S",
            Earthing::TnC => "TN-C",
            Earthing::TnCS => "TN-C-S",
            Earthing::Tt => "TT",
            Earthing::It => "IT",
        }
    }
}

impl Default for Supply {
    fn default() -> Self {
        Supply {
            voltage: 400.0,
            frequency: 50.0,
            earthing: Earthing::TnS,
            ik_max: 10.0,
            ik_min: 0.0,
            rated_current: 0.0,
            rx: 0.1,
            transformer: None,
            motors: true,
            neutral: false,
        }
    }
}

impl Default for Transformer {
    fn default() -> Self {
        Transformer { power: 630.0, uk: 6.0, losses: 0.0, network: 500.0 }
    }
}

impl Transformer {
    /// Load losses: as given, otherwise about 1.1 % of the rated power (oil-immersed
    /// distribution transformers of EN 50588-1).
    pub fn load_losses(&self) -> f32 {
        if self.losses > 0.0 {
            self.losses
        } else {
            self.power * 11.0
        }
    }
}

impl Supply {
    /// Line-to-earth voltage.
    pub fn u0(&self) -> f32 {
        self.voltage / 3f32.sqrt()
    }
}

impl Material {
    pub const ALL: [Material; 2] = [Material::Copper, Material::Aluminium];

    pub fn title(self) -> &'static str {
        match self {
            Material::Copper => tr("Copper"),
            Material::Aluminium => tr("Aluminium"),
        }
    }

    /// Resistivity at 20 °C in Ω·mm²/m.
    pub fn rho20(self) -> f32 {
        match self {
            Material::Copper => 0.018,
            Material::Aluminium => 0.0288,
        }
    }
}

impl Insulation {
    pub const ALL: [Insulation; 2] = [Insulation::Pvc, Insulation::Xlpe];

    pub fn title(self) -> &'static str {
        match self {
            Insulation::Pvc => tr("PVC (70 °C)"),
            Insulation::Xlpe => tr("XLPE / EPR (90 °C)"),
        }
    }

    /// Highest conductor temperature in operation, °C.
    pub fn max_temperature(self) -> f32 {
        match self {
            Insulation::Pvc => 70.0,
            Insulation::Xlpe => 90.0,
        }
    }
}

impl Default for Conditions {
    fn default() -> Self {
        Conditions { ambient: 40.0, grouped: 1, material: Material::Copper, insulation: Insulation::Pvc }
    }
}

/// Reduction factors for bunched circuits (IEC 60364-5-52, table B.52.17, item 1).
const GROUPING: [(u32, f32); 12] =
    [(1, 1.0), (2, 0.8), (3, 0.7), (4, 0.65), (5, 0.6), (6, 0.57), (7, 0.54), (8, 0.52), (9, 0.5), (12, 0.45), (16, 0.41), (20, 0.38)];

impl Conditions {
    /// Factor applied to the current-carrying capacity of the table.
    pub fn factor(&self) -> f32 {
        let tmax = self.insulation.max_temperature();
        let temp = ((tmax - self.ambient).max(0.0) / (tmax - 40.0)).sqrt();
        let insulation = match self.insulation {
            Insulation::Pvc => 1.0,
            Insulation::Xlpe => 1.2,
        };
        let material = match self.material {
            Material::Copper => 1.0,
            Material::Aluminium => 0.78,
        };
        let group = GROUPING.iter().rev().find(|g| self.grouped.max(1) >= g.0).map_or(1.0, |g| g.1);
        temp * insulation * material * group
    }

    /// Resistivity in operation (voltage drop, lowest fault current): 1.25 × ρ20
    /// (IEC 60364-5-52, G.52.1), Ω·mm²/m.
    pub fn rho_hot(&self) -> f32 {
        self.material.rho20() * 1.25
    }

    /// Factor k of the adiabatic short-circuit withstand (IEC 60364-4-43, table 43A).
    pub fn k(&self) -> f32 {
        match (self.material, self.insulation) {
            (Material::Copper, Insulation::Pvc) => 115.0,
            (Material::Copper, Insulation::Xlpe) => 143.0,
            (Material::Aluminium, Insulation::Pvc) => 76.0,
            (Material::Aluminium, Insulation::Xlpe) => 94.0,
        }
    }
}

impl Electrical {
    pub fn is_default(&self) -> bool {
        *self == Electrical::default()
    }
}

/// Kind of the potential of a net: as defined, otherwise from its name (L1, N, PE …).
pub fn net_kind(prj: &Project, name: &str) -> PotentialKind {
    if let Some(d) = prj.potential(name) {
        return d.kind;
    }
    match name {
        "L" | "L1" | "L2" | "L3" | "L1'" | "L2'" | "L3'" => PotentialKind::L,
        "N" | "N'" => PotentialKind::N,
        "PE" | "PEN" => PotentialKind::Pe,
        _ => PotentialKind::Other,
    }
}

/// Pin of the other side of a pass-through or protective device (1 ↔ 2, 3 ↔ 4 …).
fn partner_pin(s: &SymbolInst, pin: usize) -> Option<usize> {
    let d = designation_number(s, pin)?;
    let partner = if d % 2 == 1 { d + 1 } else { d - 1 };
    (0..s.pins.len()).find(|&j| designation_number(s, j) == Some(partner))
}

/// Nets of the mains: the phase potentials and every net reached from them through
/// protective devices, main contacts and other pass-through devices, with the phase
/// potential each comes from.
pub fn mains(prj: &Project, nl: &Netlist) -> BTreeMap<usize, String> {
    let mut out: BTreeMap<usize, String> = BTreeMap::new();
    let mut todo: Vec<(usize, String)> =
        nl.nets.iter().enumerate().filter(|(_, n)| n.named && net_kind(prj, &n.name) == PotentialKind::L).map(|(i, n)| (i, n.name.clone())).collect();
    while let Some((n, phase)) = todo.pop() {
        if out.contains_key(&n) {
            continue;
        }
        out.insert(n, phase.clone());
        for p in &nl.nets[n].pins {
            let s = &prj.pages[p.page].symbols[p.sym];
            if !passes_current(prj, s) && !is_protective(&s.sym) {
                continue;
            }
            if let Some(&m) = partner_pin(s, p.pin).and_then(|j| nl.pin_net.get(&(s.id, j))) {
                todo.push((m, phase.clone()));
            }
        }
    }
    out
}

/// A protective device of the mains and the circuit behind it.
#[derive(Clone, Debug)]
pub struct Circuit {
    pub device: String,
    pub page: usize,
    /// Id of the protective symbol.
    pub symbol: u64,
    pub protection: Protection,
    /// Nets on the line side of the device.
    pub line: BTreeSet<usize>,
    /// Nets behind the device ([`protected_nets`]).
    pub nets: BTreeSet<usize>,
    /// Phases the circuit is fed from.
    pub phases: BTreeSet<String>,
    /// Index of the circuit upstream.
    pub parent: Option<usize>,
    /// A residual current device protects the circuit, with this rated residual
    /// current in mA.
    pub residual: Option<f32>,
}

impl Circuit {
    /// Final circuits (up to 32 A) must disconnect faster than distribution circuits.
    pub fn is_final(&self) -> bool {
        self.protection.rated <= 32.0
    }
}

/// A protective device fed from the mains, whether its parts give protection data or
/// not.
#[derive(Clone, Debug)]
pub struct Feed {
    pub device: String,
    pub page: usize,
    /// Id of the protective symbol.
    pub symbol: u64,
    /// Nets on the line side of the device.
    pub line: BTreeSet<usize>,
    /// Nets behind the device ([`protected_nets`]).
    pub nets: BTreeSet<usize>,
    /// Phases the device is fed from.
    pub phases: BTreeSet<String>,
    /// Index of the device upstream (among the devices of the same list).
    pub parent: Option<usize>,
    /// Device tag of the residual current device protecting it, its own or that of a
    /// device upstream.
    pub rcd: Option<String>,
}

/// Rated residual current of a residual current device (mA) as its data states it:
/// the property, `Rated residual current` of the tech data or a value in mA in the
/// description of its parts.
pub fn stated_residual_current(prj: &Project, tag: &str) -> Option<f32> {
    let stored = crate::properties::stored(prj, &crate::properties::Object::Device(tag.into()), "device.residual_current");
    if let Some(v) = stored.and_then(|v| v.replace(',', ".").trim().trim_end_matches("mA").trim().parse::<f32>().ok()) {
        return Some(v);
    }
    crate::sizing::device_parts(prj, tag).iter().find_map(|p| {
        let text = p.tech_value("Rated residual current").map(String::from).unwrap_or_else(|| p.description.clone());
        let i = text.find("mA")?;
        let num: String = text[..i].trim_end().chars().rev().take_while(|c| c.is_ascii_digit() || *c == '.').collect::<Vec<_>>().into_iter().rev().collect();
        num.parse().ok()
    })
}

/// Rated residual current of a residual current device (mA) for the calculations:
/// as stated ([`stated_residual_current`]), 30 mA otherwise.
fn residual_current(prj: &Project, tag: &str) -> f32 {
    stated_residual_current(prj, tag).unwrap_or(30.0)
}

/// The protective devices fed from the mains in schematic order, without the device
/// upstream and the residual current device ([`link`]).
fn gather(prj: &Project, nl: &Netlist) -> Vec<Feed> {
    let mains = mains(prj, nl);
    let mut out: Vec<Feed> = Vec::new();
    for (pi, pg) in prj.pages.iter().enumerate().filter(|(_, p)| p.kind == PageKind::Schematic) {
        for (si, s) in pg.symbols.iter().enumerate() {
            if !is_protective(&s.sym) || s.tag.is_empty() {
                continue;
            }
            let line: BTreeSet<usize> =
                (0..s.pins.len()).filter(|&i| designation_number(s, i).is_some_and(|d| d % 2 == 1)).filter_map(|i| nl.pin_net.get(&(s.id, i)).copied()).collect();
            let phases: BTreeSet<String> = line.iter().filter_map(|n| mains.get(n).cloned()).collect();
            if phases.is_empty() {
                continue;
            }
            let nets = protected_nets(prj, nl, pi, si);
            if let Some(c) = out.iter_mut().find(|c| c.device == s.tag) {
                c.line.extend(line);
                c.nets.extend(nets);
                c.phases.extend(phases);
                continue;
            }
            out.push(Feed { device: s.tag.clone(), page: pi, symbol: s.id, line, nets, phases, parent: None, rcd: None });
        }
    }
    out
}

/// Sets the device upstream of each feed and the residual current device protecting it.
fn link(prj: &Project, nl: &Netlist, feeds: &mut [Feed]) {
    let rcds: Vec<(String, BTreeSet<usize>)> = prj
        .pages
        .iter()
        .filter(|p| p.kind == PageKind::Schematic)
        .flat_map(|p| p.symbols.iter())
        .filter(|s| kind_of_symbol(&s.sym) == Some(FnKind::ResidualCurrent) && !s.tag.is_empty())
        .map(|s| {
            let load = (0..s.pins.len()).filter(|&i| designation_number(s, i).is_some_and(|d| d % 2 == 0));
            (s.tag.clone(), load.filter_map(|i| nl.pin_net.get(&(s.id, i)).copied()).collect())
        })
        .collect();
    for i in 0..feeds.len() {
        feeds[i].parent = (0..feeds.len()).find(|&j| j != i && !feeds[j].nets.is_disjoint(&feeds[i].line));
        let c = &feeds[i];
        feeds[i].rcd = rcds.iter().find(|(_, nets)| !nets.is_disjoint(&c.line) || !nets.is_disjoint(&c.nets)).map(|(tag, _)| tag.clone());
    }
    // A residual current device upstream protects the devices below it.
    for i in 0..feeds.len() {
        let mut p = feeds[i].parent;
        let mut depth = 0;
        while let (Some(j), None, true) = (p, &feeds[i].rcd, depth < feeds.len()) {
            feeds[i].rcd = feeds[j].rcd.clone();
            p = feeds[j].parent;
            depth += 1;
        }
    }
}

/// Every protective device fed from the mains, also those whose parts give no
/// protection data, with the device upstream and the residual current device.
pub fn feeds(prj: &Project, nl: &Netlist) -> Vec<Feed> {
    let mut out = gather(prj, nl);
    link(prj, nl, &mut out);
    out
}

/// The protective devices fed from the mains with their circuits, in schematic order.
pub fn circuits(prj: &Project, nl: &Netlist) -> Vec<Circuit> {
    let (mut kept, mut protections) = (Vec::new(), Vec::new());
    for f in gather(prj, nl) {
        if let Some(p) = crate::protection::of_device(prj, &f.device) {
            kept.push(f);
            protections.push(p);
        }
    }
    link(prj, nl, &mut kept);
    kept.into_iter()
        .zip(protections)
        .map(|(f, protection)| Circuit {
            residual: f.rcd.as_deref().map(|tag| residual_current(prj, tag)),
            device: f.device,
            page: f.page,
            symbol: f.symbol,
            protection,
            line: f.line,
            nets: f.nets,
            phases: f.phases,
            parent: f.parent,
        })
        .collect()
}

/// Value with a unit from a text: the number before `unit` ("0.75 kW" → 0.75 for "kW").
pub fn value_with_unit(text: &str, unit: &str) -> Option<f32> {
    let mut rest = text;
    while let Some(i) = rest.find(unit) {
        let before = rest[..i].trim_end();
        let after = &rest[i + unit.len()..];
        let num: String = before.chars().rev().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',').collect::<Vec<_>>().into_iter().rev().collect();
        let starts_word = before.len() == num.len() || !before[..before.len() - num.len()].ends_with(|c: char| c.is_alphanumeric());
        if !num.is_empty() && starts_word && !after.starts_with(|c: char| c.is_alphabetic()) {
            if let Ok(v) = num.replace(',', ".").parse() {
                return Some(v);
            }
        }
        rest = after;
    }
    None
}

#[cfg(test)]
mod tests;
