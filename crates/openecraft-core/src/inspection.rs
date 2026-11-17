//! Inspection and test report of the installation (initial verification, IEC 60364-6 /
//! DIN VDE 0100-600): the circuits of the mains with what the schematic says about
//! them (description, cables, protective device, residual current device) and the
//! results measured on site, checked against the limits; the data of the report
//! (client, site, inspector, measurements at the origin), the visual inspection
//! (6.4.2) and the signature. Data the project does not give is marked missing, never
//! assumed: a limit that would rest on assumed data is not given either.

use crate::electrical::Earthing;
use crate::model::*;
use crate::netlist::{Connection, Netlist};
use crate::table::Table;
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub use crate::model::inspection::{Header, Inspection, TestResult};

/// Lowest insulation resistance of a circuit of the mains, MΩ (IEC 60364-6, table 6.1;
/// up to 500 V at 500 V DC, above at 1000 V DC).
pub const MIN_INSULATION: f32 = 1.0;
/// Conventional touch voltage limit UL, V (AC, IEC 60364-4-41): highest contact voltage
/// at the rated residual current; TT: RA × IΔn ≤ UL.
pub const CONTACT_VOLTAGE: f32 = 50.0;
/// The measured loop impedance may reach this share of U0 / Ia: the conductors warm
/// up in use (IEC 60364-6, annex C).
pub const LOOP_FACTOR: f32 = 2.0 / 3.0;
/// Tripping times of residual current devices, ms (IEC 61008-1 / IEC 61009-1): at the
/// rated residual current and at five times of it, general and selective (type S).
const RCD_TIME: (Option<f32>, Option<f32>) = (None, Some(300.0));
const RCD_TIME_5: (Option<f32>, Option<f32>) = (None, Some(40.0));
const RCD_TIME_S: (Option<f32>, Option<f32>) = (Some(130.0), Some(500.0));
const RCD_TIME_5_S: (Option<f32>, Option<f32>) = (Some(50.0), Some(150.0));

impl Inspection {
    pub fn is_empty(&self) -> bool {
        self.results.is_empty() && self.header.is_empty()
    }

    /// The results of the circuit with the protective device `circuit`.
    pub fn result(&self, circuit: &str) -> Option<&TestResult> {
        self.results.iter().find(|r| r.circuit == circuit)
    }
}

/// Results of both; of a circuit in both (and of the header) the record with more
/// edits (on a tie `theirs`). Order of `theirs`, then the new records of `mine`.
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
    let header = if mine.header.rev > theirs.header.rev { mine.header.clone() } else { theirs.header.clone() };
    Inspection { header, results }
}

/// What is entered per circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    Continuity,
    InsulationVoltage,
    Insulation,
    LoopImpedance,
    RcdCurrent,
    RcdTime,
    #[serde(rename = "rcd_time_5")]
    RcdTime5,
    RcdVoltage,
    Function,
    Tester,
    Date,
    Instrument,
    Remark,
}

impl Field {
    pub const ALL: [Field; 13] = [
        Field::Continuity,
        Field::InsulationVoltage,
        Field::Insulation,
        Field::LoopImpedance,
        Field::RcdCurrent,
        Field::RcdTime,
        Field::RcdTime5,
        Field::RcdVoltage,
        Field::Function,
        Field::Tester,
        Field::Date,
        Field::Instrument,
        Field::Remark,
    ];

    /// Name in files, on the command line and in requests.
    pub fn key(self) -> &'static str {
        match self {
            Field::Continuity => "continuity",
            Field::InsulationVoltage => "insulation_voltage",
            Field::Insulation => "insulation",
            Field::LoopImpedance => "loop_impedance",
            Field::RcdCurrent => "rcd_current",
            Field::RcdTime => "rcd_time",
            Field::RcdTime5 => "rcd_time_5",
            Field::RcdVoltage => "rcd_voltage",
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
            Field::InsulationVoltage => tr_const!("Insulation test voltage (V)"),
            Field::Insulation => tr_const!("Insulation resistance (MΩ)"),
            Field::LoopImpedance => tr_const!("Loop impedance Zs (Ω)"),
            Field::RcdCurrent => tr_const!("RCD trip current (mA)"),
            Field::RcdTime => tr_const!("RCD trip time (ms)"),
            Field::RcdTime5 => tr_const!("RCD trip time at 5 × IΔn (ms)"),
            Field::RcdVoltage => tr_const!("Contact voltage UB (V)"),
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
        matches!(
            self,
            Field::Continuity | Field::InsulationVoltage | Field::Insulation | Field::LoopImpedance | Field::RcdCurrent | Field::RcdTime | Field::RcdTime5 | Field::RcdVoltage
        )
    }

    /// A test of the residual current device (only where one protects the circuit).
    pub fn is_rcd(self) -> bool {
        matches!(self, Field::RcdCurrent | Field::RcdTime | Field::RcdTime5 | Field::RcdVoltage)
    }

    pub fn get(self, r: &TestResult) -> &str {
        match self {
            Field::Continuity => &r.continuity,
            Field::InsulationVoltage => &r.insulation_voltage,
            Field::Insulation => &r.insulation,
            Field::LoopImpedance => &r.loop_impedance,
            Field::RcdCurrent => &r.rcd_current,
            Field::RcdTime => &r.rcd_time,
            Field::RcdTime5 => &r.rcd_time_5,
            Field::RcdVoltage => &r.rcd_voltage,
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
            Field::InsulationVoltage => &mut r.insulation_voltage,
            Field::Insulation => &mut r.insulation,
            Field::LoopImpedance => &mut r.loop_impedance,
            Field::RcdCurrent => &mut r.rcd_current,
            Field::RcdTime => &mut r.rcd_time,
            Field::RcdTime5 => &mut r.rcd_time_5,
            Field::RcdVoltage => &mut r.rcd_voltage,
            Field::Function => &mut r.function,
            Field::Tester => &mut r.tester,
            Field::Date => &mut r.date,
            Field::Instrument => &mut r.instrument,
            Field::Remark => &mut r.remark,
        }
    }

    /// The value as shown: a translated function test, the others as entered.
    pub fn display(self, r: &TestResult) -> String {
        if self == Field::Function {
            return answer_text(self.get(r));
        }
        self.get(r).to_string()
    }
}

/// What the report holds besides the circuits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeaderField {
    Client,
    Site,
    Installer,
    Reason,
    Inspector,
    Instruments,
    Ze,
    Ik,
    EarthElectrode,
    Bonding,
    NextInspection,
    SignedBy,
    SignedOn,
}

impl HeaderField {
    pub const ALL: [HeaderField; 13] = [
        HeaderField::Client,
        HeaderField::Site,
        HeaderField::Installer,
        HeaderField::Reason,
        HeaderField::Inspector,
        HeaderField::Instruments,
        HeaderField::Ze,
        HeaderField::Ik,
        HeaderField::EarthElectrode,
        HeaderField::Bonding,
        HeaderField::NextInspection,
        HeaderField::SignedBy,
        HeaderField::SignedOn,
    ];

    pub fn key(self) -> &'static str {
        match self {
            HeaderField::Client => "client",
            HeaderField::Site => "site",
            HeaderField::Installer => "installer",
            HeaderField::Reason => "reason",
            HeaderField::Inspector => "inspector",
            HeaderField::Instruments => "instruments",
            HeaderField::Ze => "ze",
            HeaderField::Ik => "ik",
            HeaderField::EarthElectrode => "earth_electrode",
            HeaderField::Bonding => "bonding",
            HeaderField::NextInspection => "next_inspection",
            HeaderField::SignedBy => "signed_by",
            HeaderField::SignedOn => "signed_on",
        }
    }

    pub fn title_key(self) -> &'static str {
        match self {
            HeaderField::Client => tr_const!("Client"),
            HeaderField::Site => tr_const!("Installation site"),
            HeaderField::Installer => tr_const!("Installer"),
            HeaderField::Reason => tr_const!("Reason for the test"),
            HeaderField::Inspector => tr_const!("Inspector"),
            HeaderField::Instruments => tr_const!("Test instruments (IEC 61557)"),
            HeaderField::Ze => tr_const!("External loop impedance Ze (Ω)"),
            HeaderField::Ik => tr_const!("Prospective fault current at the origin (kA)"),
            HeaderField::EarthElectrode => tr_const!("Earth electrode resistance RA (Ω)"),
            HeaderField::Bonding => tr_const!("Main equipotential bonding (Ω)"),
            HeaderField::NextInspection => tr_const!("Next inspection"),
            HeaderField::SignedBy => tr_const!("Signed by"),
            HeaderField::SignedOn => tr_const!("Signed on"),
        }
    }

    pub fn title(self) -> &'static str {
        tr(self.title_key())
    }

    pub fn is_measurement(self) -> bool {
        matches!(self, HeaderField::Ze | HeaderField::Ik | HeaderField::EarthElectrode | HeaderField::Bonding)
    }

    pub fn get(self, h: &Header) -> &str {
        match self {
            HeaderField::Client => &h.client,
            HeaderField::Site => &h.site,
            HeaderField::Installer => &h.installer,
            HeaderField::Reason => &h.reason,
            HeaderField::Inspector => &h.inspector,
            HeaderField::Instruments => &h.instruments,
            HeaderField::Ze => &h.ze,
            HeaderField::Ik => &h.ik,
            HeaderField::EarthElectrode => &h.earth_electrode,
            HeaderField::Bonding => &h.bonding,
            HeaderField::NextInspection => &h.next_inspection,
            HeaderField::SignedBy => &h.signed_by,
            HeaderField::SignedOn => &h.signed_on,
        }
    }

    fn slot(self, h: &mut Header) -> &mut String {
        match self {
            HeaderField::Client => &mut h.client,
            HeaderField::Site => &mut h.site,
            HeaderField::Installer => &mut h.installer,
            HeaderField::Reason => &mut h.reason,
            HeaderField::Inspector => &mut h.inspector,
            HeaderField::Instruments => &mut h.instruments,
            HeaderField::Ze => &mut h.ze,
            HeaderField::Ik => &mut h.ik,
            HeaderField::EarthElectrode => &mut h.earth_electrode,
            HeaderField::Bonding => &mut h.bonding,
            HeaderField::NextInspection => &mut h.next_inspection,
            HeaderField::SignedBy => &mut h.signed_by,
            HeaderField::SignedOn => &mut h.signed_on,
        }
    }
}

/// The items of the visual inspection (IEC 60364-6, 6.4.2): key and English text.
pub const CHECKS: [(&str, &str); 14] = [
    ("protection", tr_const!("Protection against electric shock: the measures and their components")),
    ("fire", tr_const!("Fire barriers and protection against thermal effects")),
    ("conductors", tr_const!("Conductors selected for current-carrying capacity and voltage drop")),
    ("devices", tr_const!("Protective and monitoring devices selected and set correctly")),
    ("isolation", tr_const!("Isolating and switching devices in the right places")),
    ("influences", tr_const!("Equipment and protective measures suit the external influences")),
    ("identification", tr_const!("Neutral and protective conductors identified")),
    ("documents", tr_const!("Diagrams, warning notices and similar information present")),
    ("labels", tr_const!("Circuits, protective devices, switches and terminals labelled")),
    ("connections", tr_const!("Connections of the conductors")),
    ("earthing", tr_const!("Protective, earthing and bonding conductors present and adequate")),
    ("access", tr_const!("Equipment accessible for operation, inspection and maintenance")),
    ("additional", tr_const!("Additional protection by RCDs of at most 30 mA where required")),
    ("surge", tr_const!("Surge protection where required")),
];

/// Stored answers of the function test and of the visual inspection.
pub const FUNCTION_OK: &str = "ok";
pub const FUNCTION_FAILED: &str = "failed";
pub const NOT_APPLICABLE: &str = "n/a";

/// An answer (function test, visual inspection) as shown.
pub fn answer_text(v: &str) -> String {
    match v {
        FUNCTION_OK => tr("OK").into(),
        FUNCTION_FAILED => tr("Not OK").into(),
        NOT_APPLICABLE => tr("Not applicable").into(),
        v => v.to_string(),
    }
}

/// An entered answer: OK, Not OK or (`na`) not applicable, in English or translated.
fn answer(value: &str, na: bool) -> Result<String, String> {
    let is = |words: &[&str]| words.iter().any(|w| value.eq_ignore_ascii_case(w));
    if is(&[FUNCTION_OK, "OK", "pass", "passed", "yes", tr("OK"), Verdict::Passed.title()]) {
        return Ok(FUNCTION_OK.into());
    }
    if is(&[FUNCTION_FAILED, "Not OK", "fail", "no", tr("Not OK"), Verdict::Failed.title()]) {
        return Ok(FUNCTION_FAILED.into());
    }
    if na && is(&[NOT_APPLICABLE, "na", "-", "Not applicable", tr("Not applicable")]) {
        return Ok(NOT_APPLICABLE.into());
    }
    if na {
        return Err(tr_fmt!("The answer is OK, Not OK or Not applicable, not {}", value));
    }
    Err(tr_fmt!("The function test is OK or Not OK, not {}", value))
}

/// The number of a measured value: decimal comma or point, a unit after it and `>` or
/// `<` before it (instruments show `> 500`) allowed.
pub fn number(text: &str) -> Option<f32> {
    let t = text.trim().trim_start_matches(['>', '<', '=']).trim_start();
    let num: String = t.chars().take_while(|c| c.is_ascii_digit() || matches!(c, '.' | ',')).collect();
    num.replace(',', ".").parse::<f32>().ok().filter(|v| v.is_finite() && *v >= 0.0)
}

/// The stored form of an entered value: measurements must be numbers.
fn text_value(measurement: bool, value: &str) -> Result<String, String> {
    let value = value.trim();
    if measurement && !value.is_empty() && number(value).is_none() {
        return Err(tr_fmt!("Not a number: {}", value));
    }
    Ok(value.replace("\\n", "\n"))
}

/// Changing the data of a signed report takes the signature away.
fn unsign(h: &mut Header) {
    if !h.signed_by.is_empty() || !h.signed_on.is_empty() {
        h.signed_by.clear();
        h.signed_on.clear();
        h.rev += 1;
    }
}

/// Sets a value of the results of the circuit `circuit` ("" clears it). Returns whether
/// something changed.
pub fn set(prj: &mut Project, circuit: &str, field: Field, value: &str) -> Result<bool, String> {
    let value = if field == Field::Function && !value.trim().is_empty() { answer(value.trim(), false)? } else { text_value(field.is_measurement(), value)? };
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
    unsign(&mut prj.inspection.header);
    Ok(true)
}

/// Sets a value of the report: a header field by its key or an item of the visual
/// inspection as `check.<key>` ("" clears it). Every change but the signature itself
/// takes the signature away. Returns whether something changed.
pub fn set_header(prj: &mut Project, key: &str, value: &str) -> Result<bool, String> {
    let key = key.trim();
    let h = &mut prj.inspection.header;
    if let Some(item) = key.strip_prefix("check.") {
        if !CHECKS.iter().any(|c| c.0 == item) {
            return Err(tr_fmt!("Unknown field: {}", key));
        }
        let value = if value.trim().is_empty() { String::new() } else { answer(value.trim(), true)? };
        if h.checks.get(item).map_or("", String::as_str) == value {
            return Ok(false);
        }
        if value.is_empty() {
            h.checks.remove(item);
        } else {
            h.checks.insert(item.into(), value);
        }
        h.rev += 1;
        unsign(h);
        return Ok(true);
    }
    let field = HeaderField::ALL.into_iter().find(|f| f.key().eq_ignore_ascii_case(key) || f.title_key().eq_ignore_ascii_case(key)).ok_or_else(|| tr_fmt!("Unknown field: {}", key))?;
    let value = text_value(field.is_measurement(), value)?;
    let slot = field.slot(h);
    if *slot == value {
        return Ok(false);
    }
    *slot = value;
    h.rev += 1;
    if !matches!(field, HeaderField::SignedBy | HeaderField::SignedOn) {
        unsign(h);
    }
    Ok(true)
}

/// Sets several values of the report ([`set_header`]); the signature last, so that it
/// stays when it is given with other changes. Returns whether something changed.
pub fn set_header_all<'a>(prj: &mut Project, values: impl IntoIterator<Item = (&'a str, &'a str)>) -> Result<bool, String> {
    let signature = |k: &str| [HeaderField::SignedBy, HeaderField::SignedOn].iter().any(|f| f.key().eq_ignore_ascii_case(k.trim()));
    let (sign, rest): (Vec<_>, Vec<_>) = values.into_iter().partition(|(k, _)| signature(k));
    let mut changed = false;
    for (k, v) in rest.into_iter().chain(sign) {
        changed |= set_header(prj, k, v)?;
    }
    Ok(changed)
}

/// Where a circuit or the whole report stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Nothing measured yet.
    #[default]
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
    /// Type (AC, A, F, B, with S for selective); `None`: not given.
    pub kind: Option<String>,
    /// Rated residual current, mA; `None`: not given.
    pub rated: Option<f32>,
}

impl Rcd {
    /// Selective (type S, time-delayed): an `S` in its type (`A S`, `AS`, `B+ S`).
    pub fn selective(&self) -> bool {
        self.kind.as_deref().is_some_and(|k| {
            k.to_ascii_uppercase().split(|c: char| !c.is_ascii_alphanumeric() && c != '+').any(|t| t == "S" || (t.len() > 1 && t.ends_with('S') && ["A", "AC", "F", "B", "B+"].contains(&&t[..t.len() - 1])))
        })
    }
}

/// The range a measured value must lie in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Limit {
    pub field: Field,
    pub min: Option<f32>,
    pub max: Option<f32>,
}

impl Limit {
    pub fn passes(&self, v: f32) -> bool {
        self.min.is_none_or(|m| v >= m) && self.max.is_none_or(|m| v <= m)
    }

    /// The range as written in the report: `≥ 1`, `≤ 2.57`, `15 – 30`.
    pub fn text(&self) -> String {
        match (self.min, self.max) {
            (Some(a), Some(b)) => format!("{} – {}", fmt(a), fmt(b)),
            (Some(a), None) => format!("≥ {}", fmt(a)),
            (None, Some(b)) => format!("≤ {}", fmt(b)),
            (None, None) => String::new(),
        }
    }
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
    /// The ranges of the measured values (a value without one is only recorded).
    pub limits: Vec<Limit>,
    pub result: TestResult,
    pub verdict: Verdict,
    /// Values out of the limits.
    pub failed: Vec<Field>,
}

impl Circuit {
    /// The fields of this circuit: the tests of the residual current device only
    /// where one protects it.
    pub fn fields(&self) -> Vec<Field> {
        Field::ALL.into_iter().filter(|f| self.rcd.is_some() || !f.is_rcd()).collect()
    }

    /// The measurements this circuit needs.
    pub fn required(&self) -> Vec<Field> {
        let mut out = vec![Field::Continuity, Field::InsulationVoltage, Field::Insulation, Field::LoopImpedance, Field::Function];
        if self.rcd.is_some() {
            out.push(Field::RcdTime);
        }
        out
    }

    pub fn limit(&self, field: Field) -> Option<&Limit> {
        self.limits.iter().find(|l| l.field == field)
    }

    /// The values out of the limits.
    fn evaluate(&self) -> Vec<Field> {
        let r = &self.result;
        let mut out: Vec<Field> = self.limits.iter().filter(|l| number(l.field.get(r)).is_some_and(|v| !l.passes(v))).map(|l| l.field).collect();
        if r.function == FUNCTION_FAILED {
            out.push(Field::Function);
        }
        out.sort_by_key(|f| Field::ALL.iter().position(|x| x == f));
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

/// Test voltage of the insulation measurement of the mains, V DC (IEC 60364-6, table
/// 6.1): 500 V up to a nominal voltage of 500 V, 1000 V above.
pub fn insulation_voltage(prj: &Project) -> f32 {
    if prj.electrical.supply.voltage <= 500.0 { 500.0 } else { 1000.0 }
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
            // Disconnection by the overcurrent device within the time of the standard
            // (Zs × Ia ≤ U0, also in TT systems: 411.5.4), or by the residual current
            // device: Zs × IΔn ≤ U0 (TN, IT), RA × IΔn ≤ UL in TT systems (411.5.3; Zs
            // includes RA, so it is checked against the same limit).
            let by_device = faults
                .iter()
                .find(|x| x.device == f.device)
                .filter(|_| characteristic.is_some())
                .map(|x| x.trip_current)
                .filter(|ia| ia.is_finite() && *ia > 0.0)
                .map(|ia| LOOP_FACTOR * u0 / ia);
            let by_rcd = rcd.as_ref().and_then(|d| d.rated).map(|i| if tt { CONTACT_VOLTAGE } else { u0 } / (i / 1000.0));
            let zs_max = match (by_device, by_rcd) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            };
            let limit = |field, (min, max): (Option<f32>, Option<f32>)| Limit { field, min, max };
            let mut limits = vec![limit(Field::InsulationVoltage, (Some(insulation_voltage(prj)), None)), limit(Field::Insulation, (Some(MIN_INSULATION), None))];
            if let Some(max) = zs_max {
                limits.push(limit(Field::LoopImpedance, (None, Some(max))));
            }
            if let Some(d) = &rcd {
                if let Some(i) = d.rated {
                    // Between half the rated residual current and the rated residual current.
                    limits.push(limit(Field::RcdCurrent, (Some(i / 2.0), Some(i))));
                }
                let s = d.selective();
                limits.push(limit(Field::RcdTime, if s { RCD_TIME_S } else { RCD_TIME }));
                limits.push(limit(Field::RcdTime5, if s { RCD_TIME_5_S } else { RCD_TIME_5 }));
                limits.push(limit(Field::RcdVoltage, (None, Some(CONTACT_VOLTAGE))));
            }
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
                limits,
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

/// An item of the visual inspection with its answer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckItem {
    pub key: String,
    /// English text (translated where shown).
    pub title: String,
    /// `ok`, `failed`, `n/a` or empty.
    pub answer: String,
}

/// The whole report: the data of the installation and of the report, the visual
/// inspection, the circuits and the verdict.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Report {
    /// Line-to-earth voltage of the supply, V.
    pub u0: f32,
    pub earthing: String,
    /// Test voltage of the insulation measurements, V DC.
    pub insulation_voltage: f32,
    /// TT system: highest resistance of the earth electrode that passes (UL / the
    /// largest rated residual current), Ω.
    pub earth_electrode_max: Option<f32>,
    pub header: Header,
    pub checks: Vec<CheckItem>,
    pub circuits: Vec<Circuit>,
    pub verdict: Verdict,
    /// What fails besides the circuits: `earth_electrode`, `check.<key>`.
    pub failed: Vec<String>,
}

impl Report {
    /// The report of the project (see [`circuits`] for `nl` and `conns`).
    pub fn of(prj: &Project, nl: &Netlist, conns: &[Connection]) -> Report {
        let supply = &prj.electrical.supply;
        let circuits = circuits(prj, nl, conns);
        let h = &prj.inspection.header;
        let tt = supply.earthing == Earthing::Tt;
        let largest = circuits.iter().filter_map(|c| c.rcd.as_ref()?.rated).reduce(f32::max);
        let earth_electrode_max = largest.filter(|_| tt).map(|i| CONTACT_VOLTAGE / (i / 1000.0));
        let checks: Vec<CheckItem> = CHECKS.iter().map(|(k, t)| CheckItem { key: k.to_string(), title: t.to_string(), answer: h.checks.get(*k).cloned().unwrap_or_default() }).collect();
        let mut failed = Vec::new();
        if let (Some(v), Some(max)) = (number(&h.earth_electrode), earth_electrode_max) {
            if v > max {
                failed.push(HeaderField::EarthElectrode.key().to_string());
            }
        }
        failed.extend(checks.iter().filter(|c| c.answer == FUNCTION_FAILED).map(|c| format!("check.{}", c.key)));
        let started = checks.iter().any(|c| !c.answer.is_empty()) || circuits.iter().any(|c| c.verdict != Verdict::NotTested) || HeaderField::ALL.iter().any(|f| f.is_measurement() && !f.get(h).is_empty());
        let complete = checks.iter().all(|c| !c.answer.is_empty())
            && circuits.iter().all(|c| c.verdict == Verdict::Passed)
            && !h.inspector.trim().is_empty()
            && (!tt || number(&h.earth_electrode).is_some());
        let verdict = if !failed.is_empty() || circuits.iter().any(|c| c.verdict == Verdict::Failed) {
            Verdict::Failed
        } else if !started {
            Verdict::NotTested
        } else if complete {
            Verdict::Passed
        } else {
            Verdict::Incomplete
        };
        Report { u0: supply.u0(), earthing: supply.earthing.title().into(), insulation_voltage: insulation_voltage(prj), earth_electrode_max, header: h.clone(), checks, circuits, verdict, failed }
    }
}

/// The circuits of the inspection and test report: one row per circuit, the missing
/// data marked.
pub fn circuit_table(circuits: &[Circuit]) -> Table {
    let mut cols: Vec<(&'static str, f32)> = vec![
        (tr_const!("Circuit"), 0.7),
        (tr_const!("Description"), 1.4),
        (tr_const!("Phases"), 0.6),
        (tr_const!("Cable"), 1.2),
        (tr_const!("Cross-section"), 0.7),
        (tr_const!("Protective device"), 0.8),
        (tr_const!("RCD"), 1.0),
        (tr_const!("Zs max (Ω)"), 0.6),
    ];
    cols.extend(Field::ALL.iter().take(9).map(|f| (f.title_key(), 0.6)));
    cols.push((tr_const!("Result"), 0.7));
    cols.extend(Field::ALL.iter().skip(9).map(|f| (f.title_key(), if *f == Field::Remark { 1.4 } else { 0.8 })));
    let mut t = Table::with_widths(&cols);
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
            or_missing(c.zs_max.map(|v| format!("{v:.2}"))),
        ];
        row.extend(Field::ALL.iter().take(9).map(|f| if f.is_rcd() && c.rcd.is_none() { "–".into() } else { f.display(r) }));
        row.push(c.verdict.title().into());
        row.extend(Field::ALL.iter().skip(9).map(|f| f.display(r).replace('\n', " ")));
        t.push(row, Some(c.target));
    }
    t
}

/// The data of the report, the visual inspection and the verdict: item, value and
/// the limit or a note.
pub fn summary_table(r: &Report) -> Table {
    let mut t = Table::with_widths(&[(tr_const!("Item"), 2.4), (tr_const!("Value"), 1.6), (tr_const!("Note"), 1.4)]);
    let h = &r.header;
    t.push(vec![tr("Supply system").into(), format!("U0 {} V, {}", fmt(r.u0.round()), r.earthing), String::new()], None);
    for f in HeaderField::ALL {
        let note = match f {
            HeaderField::EarthElectrode => r.earth_electrode_max.map(|m| format!("≤ {}", fmt(m))).unwrap_or_default(),
            _ => String::new(),
        };
        t.push(vec![f.title().into(), f.get(h).replace('\n', " "), note], None);
    }
    t.push(vec![Field::InsulationVoltage.title().into(), fmt(r.insulation_voltage), format!("≥ {} MΩ", fmt(MIN_INSULATION))], None);
    for c in &r.checks {
        let title = CHECKS.iter().find(|x| x.0 == c.key).map_or(c.title.as_str(), |x| tr(x.1));
        t.push(vec![title.into(), if c.answer.is_empty() { String::new() } else { answer_text(&c.answer) }, String::new()], None);
    }
    let passed = r.circuits.iter().filter(|c| c.verdict == Verdict::Passed).count();
    t.push(vec![tr("Circuits").into(), tr_fmt!("{} of {} passed", passed, r.circuits.len()), String::new()], None);
    t.push(vec![tr("Result").into(), r.verdict.title().into(), String::new()], None);
    t
}

#[cfg(test)]
mod tests;
