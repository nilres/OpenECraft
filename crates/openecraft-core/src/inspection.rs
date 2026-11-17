//! Inspection and test report of the installation (initial verification, IEC 60364-6 /
//! DIN VDE 0100-600): the circuits of the mains with what the schematic says about
//! them (description, cables, protective device, residual current device) and the
//! results measured on site, checked against the limits. Data the project does not
//! give is marked missing, never assumed: a limit that would rest on assumed data is
//! not given either.

use crate::electrical::Earthing;
use crate::model::*;
use crate::netlist::{Connection, Netlist};
use crate::table::Table;
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub use crate::model::inspection::{Inspection, TestResult};

/// Lowest insulation resistance of a circuit up to 500 V (test voltage 500 V DC), MΩ
/// (IEC 60364-6, table 6.1).
pub const MIN_INSULATION: f32 = 1.0;
/// Longest tripping time of a residual current device at its rated residual current,
/// ms (general type, IEC 61008-1 / IEC 61009-1).
pub const MAX_RCD_TIME: f32 = 300.0;
/// The measured loop impedance may reach this share of U0 / Ia: the conductors warm
/// up in use (IEC 60364-6, annex C).
pub const LOOP_FACTOR: f32 = 2.0 / 3.0;

impl Inspection {
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// The results of the circuit with the protective device `circuit`.
    pub fn result(&self, circuit: &str) -> Option<&TestResult> {
        self.results.iter().find(|r| r.circuit == circuit)
    }
}

/// Results of both; of a circuit in both the record with more edits (on a tie
/// `theirs`). Order of `theirs`, then the new records of `mine`.
pub fn merge(theirs: &Inspection, mine: &Inspection) -> Inspection {
    let mut results: Vec<TestResult> = theirs
        .results
        .iter()
        .map(|t| match mine.result(&t.circuit) {
            Some(m) if m.rev > t.rev => m.clone(),
            _ => t.clone(),
        })
        .collect();
    results.extend(mine.results.iter().filter(|m| theirs.result(&m.circuit).is_none()).cloned());
    Inspection { results }
}

/// What is entered per circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    Continuity,
    Insulation,
    LoopImpedance,
    RcdCurrent,
    RcdTime,
    Function,
    Tester,
    Date,
    Instrument,
    Remark,
}

impl Field {
    pub const ALL: [Field; 10] =
        [Field::Continuity, Field::Insulation, Field::LoopImpedance, Field::RcdCurrent, Field::RcdTime, Field::Function, Field::Tester, Field::Date, Field::Instrument, Field::Remark];

    /// Name in files, on the command line and in requests.
    pub fn key(self) -> &'static str {
        match self {
            Field::Continuity => "continuity",
            Field::Insulation => "insulation",
            Field::LoopImpedance => "loop_impedance",
            Field::RcdCurrent => "rcd_current",
            Field::RcdTime => "rcd_time",
            Field::Function => "function",
            Field::Tester => "tester",
            Field::Date => "date",
            Field::Instrument => "instrument",
            Field::Remark => "remark",
        }
    }

    /// English title (column name, translated where shown).
    pub fn title_key(self) -> &'static str {
        match self {
            Field::Continuity => tr_const!("Protective conductor (Ω)"),
            Field::Insulation => tr_const!("Insulation resistance (MΩ)"),
            Field::LoopImpedance => tr_const!("Loop impedance Zs (Ω)"),
            Field::RcdCurrent => tr_const!("RCD trip current (mA)"),
            Field::RcdTime => tr_const!("RCD trip time (ms)"),
            Field::Function => tr_const!("Function test"),
            Field::Tester => tr_const!("Tested by"),
            Field::Date => tr_const!("Date"),
            Field::Instrument => tr_const!("Test instrument"),
            Field::Remark => tr_const!("Remark"),
        }
    }

    pub fn title(self) -> &'static str {
        tr(self.title_key())
    }

    /// By its key or its English title (case-insensitive).
    pub fn from_key(name: &str) -> Option<Field> {
        let name = name.trim();
        Field::ALL.into_iter().find(|f| f.key().eq_ignore_ascii_case(name) || f.title_key().eq_ignore_ascii_case(name))
    }

    /// A measured value (a number, `>` or `<` before it allowed).
    pub fn is_measurement(self) -> bool {
        matches!(self, Field::Continuity | Field::Insulation | Field::LoopImpedance | Field::RcdCurrent | Field::RcdTime)
    }

    pub fn get(self, r: &TestResult) -> &str {
        match self {
            Field::Continuity => &r.continuity,
            Field::Insulation => &r.insulation,
            Field::LoopImpedance => &r.loop_impedance,
            Field::RcdCurrent => &r.rcd_current,
            Field::RcdTime => &r.rcd_time,
            Field::Function => &r.function,
            Field::Tester => &r.tester,
            Field::Date => &r.date,
            Field::Instrument => &r.instrument,
            Field::Remark => &r.remark,
        }
    }

    fn slot(self, r: &mut TestResult) -> &mut String {
        match self {
            Field::Continuity => &mut r.continuity,
            Field::Insulation => &mut r.insulation,
            Field::LoopImpedance => &mut r.loop_impedance,
            Field::RcdCurrent => &mut r.rcd_current,
            Field::RcdTime => &mut r.rcd_time,
            Field::Function => &mut r.function,
            Field::Tester => &mut r.tester,
            Field::Date => &mut r.date,
            Field::Instrument => &mut r.instrument,
            Field::Remark => &mut r.remark,
        }
    }

    /// The value as shown: a translated function test, the others as entered.
    pub fn display(self, r: &TestResult) -> String {
        match (self, self.get(r)) {
            (Field::Function, FUNCTION_OK) => tr("OK").into(),
            (Field::Function, FUNCTION_FAILED) => tr("Not OK").into(),
            (_, v) => v.to_string(),
        }
    }
}

/// Stored values of the function test.
pub const FUNCTION_OK: &str = "ok";
pub const FUNCTION_FAILED: &str = "failed";

/// The number of a measured value: decimal comma or point, a unit after it and `>` or
/// `<` before it (instruments show `> 500`) allowed.
pub fn number(text: &str) -> Option<f32> {
    let t = text.trim().trim_start_matches(['>', '<', '=']).trim_start();
    let num: String = t.chars().take_while(|c| c.is_ascii_digit() || matches!(c, '.' | ',')).collect();
    num.replace(',', ".").parse::<f32>().ok().filter(|v| v.is_finite() && *v >= 0.0)
}

/// The stored form of an entered value; an error for a measurement that is no number
/// and a function test that is neither passed nor failed.
fn checked(field: Field, value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    if field.is_measurement() {
        return number(value).map(|_| value.to_string()).ok_or_else(|| tr_fmt!("Not a number: {}", value));
    }
    if field == Field::Function {
        let is = |words: &[&str]| words.iter().any(|w| value.eq_ignore_ascii_case(w));
        if is(&[FUNCTION_OK, "OK", "pass", "passed", "yes", tr("OK"), Verdict::Passed.title()]) {
            return Ok(FUNCTION_OK.into());
        }
        if is(&[FUNCTION_FAILED, "Not OK", "fail", "no", tr("Not OK"), Verdict::Failed.title()]) {
            return Ok(FUNCTION_FAILED.into());
        }
        return Err(tr_fmt!("The function test is OK or Not OK, not {}", value));
    }
    Ok(value.replace("\\n", "\n"))
}

/// Sets a value of the results of the circuit `circuit` ("" clears it). Returns whether
/// something changed.
pub fn set(prj: &mut Project, circuit: &str, field: Field, value: &str) -> Result<bool, String> {
    let value = checked(field, value)?;
    let results = &mut prj.inspection.results;
    let i = match results.iter().position(|r| r.circuit == circuit) {
        Some(i) => i,
        None if value.is_empty() => return Ok(false),
        None => {
            results.push(TestResult { circuit: circuit.to_string(), ..Default::default() });
            results.len() - 1
        }
    };
    let slot = field.slot(&mut results[i]);
    if *slot == value {
        return Ok(false);
    }
    *slot = value;
    results[i].rev += 1;
    Ok(true)
}

/// Where a circuit stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Nothing measured yet.
    NotTested,
    /// Measured in part, nothing out of the limits.
    Incomplete,
    Passed,
    Failed,
}

impl Verdict {
    pub fn title(self) -> &'static str {
        match self {
            Verdict::NotTested => tr("Not tested"),
            Verdict::Incomplete => tr("Incomplete"),
            Verdict::Passed => tr("Passed"),
            Verdict::Failed => tr("Not passed"),
        }
    }
}

/// The residual current device protecting a circuit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rcd {
    pub device: String,
    /// Type (AC, A, F, B); `None`: not given.
    pub kind: Option<String>,
    /// Rated residual current, mA; `None`: not given.
    pub rated: Option<f32>,
}

/// A circuit of the report: what the project says about it and what was measured.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Circuit {
    /// Device tag of the protective device (the circuit number).
    pub circuit: String,
    /// Page index and id of the protective symbol.
    pub target: (usize, u64),
    pub description: Option<String>,
    pub phases: Vec<String>,
    /// Cables of the circuit (tag and type).
    pub cables: Vec<String>,
    pub cross_section: Option<String>,
    pub characteristic: Option<String>,
    /// Rated current of the protective device, A.
    pub rated: Option<f32>,
    pub rcd: Option<Rcd>,
    /// Highest loop impedance measured that passes, Ω; `None`: the data are missing.
    pub zs_max: Option<f32>,
    pub result: TestResult,
    pub verdict: Verdict,
    /// Values out of the limits.
    pub failed: Vec<Field>,
}

impl Circuit {
    /// The measurements this circuit needs.
    pub fn required(&self) -> Vec<Field> {
        let mut out = vec![Field::Continuity, Field::Insulation, Field::LoopImpedance, Field::Function];
        if self.rcd.is_some() {
            out.push(Field::RcdTime);
        }
        out
    }

    /// The values out of the limits.
    fn evaluate(&self) -> Vec<Field> {
        let r = &self.result;
        let value = |f: Field| number(f.get(r));
        let mut out = Vec::new();
        if value(Field::Insulation).is_some_and(|v| v < MIN_INSULATION) {
            out.push(Field::Insulation);
        }
        if let (Some(v), Some(max)) = (value(Field::LoopImpedance), self.zs_max) {
            if v > max {
                out.push(Field::LoopImpedance);
            }
        }
        if let (Some(v), Some(rated)) = (value(Field::RcdCurrent), self.rcd.as_ref().and_then(|d| d.rated)) {
            // Between half the rated residual current and the rated residual current.
            if v > rated || v < rated / 2.0 {
                out.push(Field::RcdCurrent);
            }
        }
        if value(Field::RcdTime).is_some_and(|v| v > MAX_RCD_TIME) {
            out.push(Field::RcdTime);
        }
        if r.function == FUNCTION_FAILED {
            out.push(Field::Function);
        }
        out
    }

    fn verdict(&self) -> Verdict {
        let r = &self.result;
        let given = |f: Field| if f.is_measurement() { number(f.get(r)).is_some() } else { !f.get(r).is_empty() };
        if !self.failed.is_empty() {
            Verdict::Failed
        } else if !Field::ALL.iter().any(|f| (f.is_measurement() || *f == Field::Function) && given(*f)) {
            Verdict::NotTested
        } else if self.required().into_iter().all(given) {
            Verdict::Passed
        } else {
            Verdict::Incomplete
        }
    }

    /// The protective device as written in the report: characteristic and rated
    /// current, each marked missing where the project does not give it.
    pub fn device_text(&self) -> String {
        format!("{} {}", self.characteristic.clone().unwrap_or_else(|| tr("missing").into()), self.rated.map_or_else(|| tr("missing").to_string(), |a| format!("{} A", fmt(a))))
    }

    /// The residual current device as written in the report ("–": none).
    pub fn rcd_text(&self) -> String {
        let Some(d) = &self.rcd else { return "–".into() };
        let missing = || tr("missing").to_string();
        format!("{} {} {}", d.device, d.kind.clone().unwrap_or_else(missing), d.rated.map_or_else(missing, |i| format!("{} mA", fmt(i))))
    }
}

/// A number without needless decimals.
fn fmt(v: f32) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// A missing value as written in the report.
fn or_missing(v: Option<String>) -> String {
    v.unwrap_or_else(|| tr("missing").into())
}

/// Type of a residual current device: the property, then the tech data of its parts.
fn rcd_kind(prj: &Project, tag: &str) -> Option<String> {
    let v = crate::properties::get(prj, &crate::properties::Object::Device(tag.into()), "device.rcd_type");
    if !v.trim().is_empty() {
        return Some(v.trim().to_string());
    }
    crate::sizing::device_parts(prj, tag).iter().find_map(|p| ["Residual current type", "RCD type"].iter().find_map(|k| p.tech_value(k)).map(String::from)).filter(|v| !v.is_empty())
}

/// What the circuit is for: the description of the device, else a function text.
fn description(prj: &Project, tag: &str) -> Option<String> {
    let d = prj.devices.get(tag).map(|d| d.description.trim()).filter(|d| !d.is_empty());
    d.map(String::from).or_else(|| {
        prj.pages.iter().flat_map(|p| p.symbols.iter()).filter(|s| s.tag == tag).map(|s| s.function_text.trim().replace('\n', " ")).find(|t| !t.is_empty())
    })
}

/// The circuits of the mains: every protective device fed from the supply, in
/// schematic order, with the results of `prj.inspection`. `nl`: potentials, `conns`:
/// connections of the analysis.
pub fn circuits(prj: &Project, nl: &Netlist, conns: &[Connection]) -> Vec<Circuit> {
    let supply = &prj.electrical.supply;
    let u0 = supply.u0();
    let tt = supply.earthing == Earthing::Tt;
    let faults = crate::shortcircuit::faults(prj, nl, conns);
    crate::electrical::feeds(prj, nl)
        .into_iter()
        .map(|f| {
            let touches = |k: &&Connection| [k.a, k.b].iter().any(|p| nl.pin_net.get(&(prj.pages[p.page].symbols[p.sym].id, p.pin)).is_some_and(|n| f.nets.contains(n)));
            let mine: Vec<&Connection> = conns.iter().filter(touches).collect();
            let tags: BTreeSet<&str> = mine.iter().map(|k| k.cable.as_str()).filter(|c| !c.is_empty()).collect();
            let cables: Vec<&Cable> = tags.iter().filter_map(|t| prj.cable(t)).collect();
            let cable_texts = tags.iter().map(|t| match prj.cable(t).map(|c| c.cable_type.trim()).filter(|c| !c.is_empty()) {
                Some(ty) => format!("{t} {ty}"),
                None => t.to_string(),
            });
            let sections: BTreeSet<String> = if cables.is_empty() {
                mine.iter().map(|k| k.cross_section.trim().to_string()).filter(|c| !c.is_empty()).collect()
            } else {
                cables.iter().map(|c| c.cross_section.trim().to_string()).filter(|c| !c.is_empty()).collect()
            };
            let protection = crate::protection::of_device(prj, &f.device);
            let characteristic = crate::protection::stated_characteristic(prj, &f.device);
            let rcd = f.rcd.as_ref().map(|tag| Rcd { device: tag.clone(), kind: rcd_kind(prj, tag), rated: crate::electrical::stated_residual_current(prj, tag) });
            // Disconnection by the overcurrent device within the time of the standard,
            // or by the residual current device (Zs × IΔn ≤ U0).
            let by_device = faults
                .iter()
                .find(|x| x.device == f.device)
                .filter(|_| characteristic.is_some() && !tt)
                .map(|x| x.trip_current)
                .filter(|ia| ia.is_finite() && *ia > 0.0)
                .map(|ia| LOOP_FACTOR * u0 / ia);
            let by_rcd = rcd.as_ref().and_then(|d| d.rated).map(|i| u0 / (i / 1000.0));
            let zs_max = match (by_device, by_rcd) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            };
            let mut c = Circuit {
                description: description(prj, &f.device),
                target: (f.page, f.symbol),
                phases: f.phases.into_iter().collect(),
                cables: cable_texts.collect(),
                cross_section: (!sections.is_empty()).then(|| sections.into_iter().collect::<Vec<_>>().join(", ")),
                characteristic: characteristic.map(|c| c.title().to_string()),
                rated: protection.map(|p| p.rated),
                rcd,
                zs_max,
                result: prj.inspection.result(&f.device).cloned().unwrap_or_else(|| TestResult { circuit: f.device.clone(), ..Default::default() }),
                circuit: f.device,
                verdict: Verdict::NotTested,
                failed: Vec::new(),
            };
            c.failed = c.evaluate();
            c.verdict = c.verdict();
            c
        })
        .collect()
}

/// The inspection and test report: one row per circuit, the missing data marked.
pub fn report(circuits: &[Circuit]) -> Table {
    let mut t = Table::with_widths(&[
        (tr_const!("Circuit"), 0.7),
        (tr_const!("Description"), 1.4),
        (tr_const!("Phases"), 0.6),
        (tr_const!("Cable"), 1.2),
        (tr_const!("Cross-section"), 0.7),
        (tr_const!("Protective device"), 0.8),
        (tr_const!("RCD"), 1.0),
        (Field::Continuity.title_key(), 0.7),
        (Field::Insulation.title_key(), 0.7),
        (Field::LoopImpedance.title_key(), 0.7),
        (tr_const!("Zs max (Ω)"), 0.6),
        (Field::RcdCurrent.title_key(), 0.6),
        (Field::RcdTime.title_key(), 0.6),
        (Field::Function.title_key(), 0.6),
        (tr_const!("Result"), 0.7),
        (Field::Tester.title_key(), 0.8),
        (Field::Date.title_key(), 0.7),
        (Field::Instrument.title_key(), 0.9),
        (Field::Remark.title_key(), 1.4),
    ]);
    for c in circuits {
        let r = &c.result;
        let mut row = vec![
            c.circuit.clone(),
            or_missing(c.description.clone()),
            c.phases.join(" "),
            if c.cables.is_empty() { "–".into() } else { c.cables.join(", ") },
            or_missing(c.cross_section.clone()),
            c.device_text(),
            c.rcd_text(),
        ];
        row.extend([Field::Continuity, Field::Insulation, Field::LoopImpedance].map(|f| f.display(r)));
        row.push(or_missing(c.zs_max.map(|v| format!("{v:.2}"))));
        row.extend([Field::RcdCurrent, Field::RcdTime, Field::Function].map(|f| f.display(r)));
        row.push(c.verdict.title().into());
        row.extend([Field::Tester, Field::Date, Field::Instrument, Field::Remark].map(|f| f.display(r).replace('\n', " ")));
        t.push(row, Some(c.target));
    }
    t
}

#[cfg(test)]
mod tests;
