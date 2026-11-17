//! Inspection and test report of the installation (initial verification, IEC 60364-6 /
//! DIN VDE 0100-600): the circuits of the mains with what the schematic says about
//! them (description, cables, protective device, residual current device) and the
//! results measured on site, checked against the limits; the sources of SELV, PELV and
//! electrically separated circuits; the data of the report (client, site, inspector,
//! measurements at the origin, floors and walls, IT systems), the visual inspection
//! (6.4.2) and the signature. Data the project does not give is marked missing, never
//! assumed: a limit that would rest on assumed data is not given either.

use crate::electrical::Earthing;
use crate::model::*;
use crate::netlist::{Connection, Netlist};
use crate::parts::{kind_of_symbol, FnKind};
use crate::table::Table;
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub use crate::model::inspection::{Header, Inspection, SourceResult, TestResult};

/// Lowest insulation resistance of a circuit of the mains, MΩ (IEC 60364-6, table 6.1;
/// up to 500 V at 500 V DC, above at 1000 V DC).
pub const MIN_INSULATION: f32 = 1.0;
/// Conventional touch voltage limit UL, V (AC, IEC 60364-4-41): highest contact voltage
/// at the rated residual current; TT: RA × IΔn ≤ UL; IT: RA × Id ≤ UL.
pub const CONTACT_VOLTAGE: f32 = 50.0;
/// The measured loop impedance may reach this share of the calculated limit: the
/// conductors warm up in use (IEC 60364-6, annex C).
pub const LOOP_FACTOR: f32 = 2.0 / 3.0;
/// Tripping times of residual current devices, ms (IEC 61008-1 / IEC 61009-1): at the
/// rated residual current and at five times of it, general and selective (type S).
const RCD_TIME: (Option<f32>, Option<f32>) = (None, Some(300.0));
const RCD_TIME_5: (Option<f32>, Option<f32>) = (None, Some(40.0));
const RCD_TIME_S: (Option<f32>, Option<f32>) = (Some(130.0), Some(500.0));
const RCD_TIME_5_S: (Option<f32>, Option<f32>) = (Some(50.0), Some(150.0));
/// Highest voltages of SELV and PELV circuits, V (IEC 60364-4-41, 414.1.1: band I).
const SELV_AC: f32 = 50.0;
const SELV_DC: f32 = 120.0;

impl Inspection {
    pub fn is_empty(&self) -> bool {
        self.results.is_empty() && self.sources.is_empty() && self.header.is_empty()
    }

    /// The results of the circuit with the protective device `circuit`.
    pub fn result(&self, circuit: &str) -> Option<&TestResult> {
        self.results.iter().find(|r| r.circuit == circuit)
    }

    /// The results of the source `device`.
    pub fn source(&self, device: &str) -> Option<&SourceResult> {
        self.sources.iter().find(|r| r.device == device)
    }
}

/// Records of both lists by key; of a record in both the one with more edits (on a tie
/// `theirs`). Order of `theirs`, then the new records of `mine`.
fn merge_by<T: Clone>(theirs: &[T], mine: &[T], key: impl Fn(&T) -> &str, rev: impl Fn(&T) -> u64) -> Vec<T> {
    let mut out: Vec<T> = theirs
        .iter()
        .map(|t| match mine.iter().find(|m| key(m) == key(t)) {
            Some(m) if rev(m) > rev(t) => m.clone(),
            _ => t.clone(),
        })
        .collect();
    out.extend(mine.iter().filter(|m| !theirs.iter().any(|t| key(t) == key(m))).cloned());
    out
}

/// Both inspections merged: the header and the results of each circuit and source as
/// records ([`merge_by`]).
pub fn merge(theirs: &Inspection, mine: &Inspection) -> Inspection {
    Inspection {
        header: if mine.header.rev > theirs.header.rev { mine.header.clone() } else { theirs.header.clone() },
        results: merge_by(&theirs.results, &mine.results, |r| &r.circuit, |r| r.rev),
        sources: merge_by(&theirs.sources, &mine.sources, |r| &r.device, |r| r.rev),
    }
}

/// What is entered per circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    Continuity,
    InsulationVoltage,
    Insulation,
    LoopImpedance,
    VoltageDrop,
    RcdCurrent,
    RcdTime,
    #[serde(rename = "rcd_time_5")]
    RcdTime5,
    RcdVoltage,
    Polarity,
    PhaseSequence,
    Function,
    Tester,
    Date,
    Instrument,
    Remark,
}

impl Field {
    pub const ALL: [Field; 16] = [
        Field::Continuity,
        Field::InsulationVoltage,
        Field::Insulation,
        Field::LoopImpedance,
        Field::VoltageDrop,
        Field::RcdCurrent,
        Field::RcdTime,
        Field::RcdTime5,
        Field::RcdVoltage,
        Field::Polarity,
        Field::PhaseSequence,
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
            Field::VoltageDrop => "voltage_drop",
            Field::RcdCurrent => "rcd_current",
            Field::RcdTime => "rcd_time",
            Field::RcdTime5 => "rcd_time_5",
            Field::RcdVoltage => "rcd_voltage",
            Field::Polarity => "polarity",
            Field::PhaseSequence => "phase_sequence",
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
            Field::VoltageDrop => tr_const!("Voltage drop (%)"),
            Field::RcdCurrent => tr_const!("RCD trip current (mA)"),
            Field::RcdTime => tr_const!("RCD trip time (ms)"),
            Field::RcdTime5 => tr_const!("RCD trip time at 5 × IΔn (ms)"),
            Field::RcdVoltage => tr_const!("Contact voltage UB (V)"),
            Field::Polarity => tr_const!("Polarity"),
            Field::PhaseSequence => tr_const!("Phase sequence"),
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
            Field::Continuity
                | Field::InsulationVoltage
                | Field::Insulation
                | Field::LoopImpedance
                | Field::VoltageDrop
                | Field::RcdCurrent
                | Field::RcdTime
                | Field::RcdTime5
                | Field::RcdVoltage
        )
    }

    /// A test answered with OK or Not OK.
    pub fn is_answer(self) -> bool {
        matches!(self, Field::Polarity | Field::PhaseSequence | Field::Function)
    }

    /// A test or measurement (not who, when, with what and the remark).
    pub fn is_test(self) -> bool {
        self.is_measurement() || self.is_answer()
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
            Field::VoltageDrop => &r.voltage_drop,
            Field::RcdCurrent => &r.rcd_current,
            Field::RcdTime => &r.rcd_time,
            Field::RcdTime5 => &r.rcd_time_5,
            Field::RcdVoltage => &r.rcd_voltage,
            Field::Polarity => &r.polarity,
            Field::PhaseSequence => &r.phase_sequence,
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
            Field::VoltageDrop => &mut r.voltage_drop,
            Field::RcdCurrent => &mut r.rcd_current,
            Field::RcdTime => &mut r.rcd_time,
            Field::RcdTime5 => &mut r.rcd_time_5,
            Field::RcdVoltage => &mut r.rcd_voltage,
            Field::Polarity => &mut r.polarity,
            Field::PhaseSequence => &mut r.phase_sequence,
            Field::Function => &mut r.function,
            Field::Tester => &mut r.tester,
            Field::Date => &mut r.date,
            Field::Instrument => &mut r.instrument,
            Field::Remark => &mut r.remark,
        }
    }

    /// The value as shown: translated answers, the others as entered.
    pub fn display(self, r: &TestResult) -> String {
        if self.is_answer() {
            return answer_text(self.get(r));
        }
        self.get(r).to_string()
    }
}

/// What the report holds besides the circuits and sources.
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
    FloorInsulation,
    Imd,
    FirstFault,
    NextInspection,
    SignedBy,
    SignedOn,
}

impl HeaderField {
    pub const ALL: [HeaderField; 16] = [
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
        HeaderField::FloorInsulation,
        HeaderField::Imd,
        HeaderField::FirstFault,
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
            HeaderField::FloorInsulation => "floor_insulation",
            HeaderField::Imd => "imd",
            HeaderField::FirstFault => "first_fault",
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
            HeaderField::FloorInsulation => tr_const!("Insulating floors and walls (kΩ)"),
            HeaderField::Imd => tr_const!("Insulation monitoring device"),
            HeaderField::FirstFault => tr_const!("First fault current Id (A)"),
            HeaderField::NextInspection => tr_const!("Next inspection"),
            HeaderField::SignedBy => tr_const!("Signed by"),
            HeaderField::SignedOn => tr_const!("Signed on"),
        }
    }

    pub fn title(self) -> &'static str {
        tr(self.title_key())
    }

    pub fn is_measurement(self) -> bool {
        matches!(self, HeaderField::Ze | HeaderField::Ik | HeaderField::EarthElectrode | HeaderField::Bonding | HeaderField::FloorInsulation | HeaderField::FirstFault)
    }

    /// Answered with OK, Not OK or Not applicable.
    pub fn is_answer(self) -> bool {
        self == HeaderField::Imd
    }

    /// Only for IT systems.
    pub fn is_it(self) -> bool {
        matches!(self, HeaderField::Imd | HeaderField::FirstFault)
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
            HeaderField::FloorInsulation => &h.floor_insulation,
            HeaderField::Imd => &h.imd,
            HeaderField::FirstFault => &h.first_fault,
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
            HeaderField::FloorInsulation => &mut h.floor_insulation,
            HeaderField::Imd => &mut h.imd,
            HeaderField::FirstFault => &mut h.first_fault,
            HeaderField::NextInspection => &mut h.next_inspection,
            HeaderField::SignedBy => &mut h.signed_by,
            HeaderField::SignedOn => &mut h.signed_on,
        }
    }

    /// The value as shown: a translated answer, the others as entered.
    pub fn display(self, h: &Header) -> String {
        if self.is_answer() {
            return answer_text(self.get(h));
        }
        self.get(h).replace('\n', " ")
    }
}

/// What is entered per source of SELV, PELV or separated circuits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceField {
    Measure,
    InsulationVoltage,
    Insulation,
    Remark,
}

impl SourceField {
    pub const ALL: [SourceField; 4] = [SourceField::Measure, SourceField::InsulationVoltage, SourceField::Insulation, SourceField::Remark];

    pub fn key(self) -> &'static str {
        match self {
            SourceField::Measure => "measure",
            SourceField::InsulationVoltage => "insulation_voltage",
            SourceField::Insulation => "insulation",
            SourceField::Remark => "remark",
        }
    }

    pub fn title_key(self) -> &'static str {
        match self {
            SourceField::Measure => tr_const!("Protective measure"),
            SourceField::InsulationVoltage => tr_const!("Insulation test voltage (V)"),
            SourceField::Insulation => tr_const!("Insulation resistance (MΩ)"),
            SourceField::Remark => tr_const!("Remark"),
        }
    }

    pub fn title(self) -> &'static str {
        tr(self.title_key())
    }

    pub fn from_key(name: &str) -> Option<SourceField> {
        let name = name.trim();
        SourceField::ALL.into_iter().find(|f| f.key().eq_ignore_ascii_case(name) || f.title_key().eq_ignore_ascii_case(name))
    }

    pub fn is_measurement(self) -> bool {
        matches!(self, SourceField::InsulationVoltage | SourceField::Insulation)
    }

    pub fn get(self, r: &SourceResult) -> &str {
        match self {
            SourceField::Measure => &r.measure,
            SourceField::InsulationVoltage => &r.insulation_voltage,
            SourceField::Insulation => &r.insulation,
            SourceField::Remark => &r.remark,
        }
    }

    fn slot(self, r: &mut SourceResult) -> &mut String {
        match self {
            SourceField::Measure => &mut r.measure,
            SourceField::InsulationVoltage => &mut r.insulation_voltage,
            SourceField::Insulation => &mut r.insulation,
            SourceField::Remark => &mut r.remark,
        }
    }

    pub fn display(self, r: &SourceResult) -> String {
        if self == SourceField::Measure {
            return measure_text(&r.measure);
        }
        self.get(r).replace('\n', " ")
    }
}

/// The protective measures of a source: stored key and English title.
pub const MEASURES: [(&str, &str); 4] =
    [("selv", tr_const!("SELV")), ("pelv", tr_const!("PELV")), ("separation", tr_const!("Electrical separation")), (NOT_APPLICABLE, tr_const!("No protective measure of its own"))];

fn measure_text(v: &str) -> String {
    MEASURES.iter().find(|m| m.0 == v).map_or_else(|| v.to_string(), |m| tr(m.1).to_string())
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

/// Stored answers of the tests and of the visual inspection.
pub const FUNCTION_OK: &str = "ok";
pub const FUNCTION_FAILED: &str = "failed";
pub const NOT_APPLICABLE: &str = "n/a";

/// An answer (tests, visual inspection) as shown.
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
    let value = if field.is_answer() && !value.trim().is_empty() { answer(value.trim(), false)? } else { text_value(field.is_measurement(), value)? };
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

/// Sets a value of the source `device` ("" clears it). Returns whether something changed.
pub fn set_source(prj: &mut Project, device: &str, field: SourceField, value: &str) -> Result<bool, String> {
    let value = if field == SourceField::Measure && !value.trim().is_empty() {
        let v = value.trim();
        let m = MEASURES.iter().find(|m| m.0.eq_ignore_ascii_case(v) || m.1.eq_ignore_ascii_case(v) || tr(m.1).eq_ignore_ascii_case(v));
        let m = m.or_else(|| ["na", "-", "none"].contains(&v.to_ascii_lowercase().as_str()).then_some(&MEASURES[3]));
        m.map(|m| m.0.to_string()).ok_or_else(|| tr_fmt!("The protective measure is SELV, PELV, electrical separation or none, not {}", v))?
    } else {
        text_value(field.is_measurement(), value)?
    };
    let sources = &mut prj.inspection.sources;
    let i = match sources.iter().position(|r| r.device == device) {
        Some(i) => i,
        None if value.is_empty() => return Ok(false),
        None => {
            sources.push(SourceResult { device: device.to_string(), ..Default::default() });
            sources.len() - 1
        }
    };
    let slot = field.slot(&mut sources[i]);
    if *slot == value {
        return Ok(false);
    }
    *slot = value;
    sources[i].rev += 1;
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
    let value = if field.is_answer() && !value.trim().is_empty() { answer(value.trim(), true)? } else { text_value(field.is_measurement(), value)? };
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

/// Where a circuit, a source or the whole report stands.
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

/// A range by the key of a value of the report or of a source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyLimit {
    pub key: String,
    pub min: Option<f32>,
    pub max: Option<f32>,
}

/// Whether `v` lies in the range, and the range as written in the report.
fn passes(min: Option<f32>, max: Option<f32>, v: f32) -> bool {
    min.is_none_or(|m| v >= m) && max.is_none_or(|m| v <= m)
}

fn range_text(min: Option<f32>, max: Option<f32>) -> String {
    match (min, max) {
        (Some(a), Some(b)) => format!("{} – {}", fmt(a), fmt(b)),
        (Some(a), None) => format!("≥ {}", fmt(a)),
        (None, Some(b)) => format!("≤ {}", fmt(b)),
        (None, None) => String::new(),
    }
}

impl Limit {
    pub fn passes(&self, v: f32) -> bool {
        passes(self.min, self.max, v)
    }

    /// The range as written in the report: `≥ 1`, `≤ 2.57`, `15 – 30`.
    pub fn text(&self) -> String {
        range_text(self.min, self.max)
    }
}

impl KeyLimit {
    fn new(key: &str, min: Option<f32>, max: Option<f32>) -> KeyLimit {
        KeyLimit { key: key.into(), min, max }
    }

    pub fn passes(&self, v: f32) -> bool {
        passes(self.min, self.max, v)
    }

    pub fn text(&self) -> String {
        range_text(self.min, self.max)
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
    /// A circuit of three phases: its phase sequence is tested.
    pub fn multi_phase(&self) -> bool {
        self.phases.len() >= 3
    }

    /// The fields of this circuit: the tests of the residual current device only
    /// where one protects it, the phase sequence only with three phases.
    pub fn fields(&self) -> Vec<Field> {
        Field::ALL.into_iter().filter(|f| (self.rcd.is_some() || !f.is_rcd()) && (self.multi_phase() || *f != Field::PhaseSequence)).collect()
    }

    /// The tests this circuit needs.
    pub fn required(&self) -> Vec<Field> {
        let mut out = vec![Field::Continuity, Field::InsulationVoltage, Field::Insulation, Field::LoopImpedance, Field::Polarity, Field::Function];
        if self.multi_phase() {
            out.push(Field::PhaseSequence);
        }
        if self.rcd.is_some() {
            out.push(Field::RcdTime);
        }
        out
    }

    pub fn limit(&self, field: Field) -> Option<&Limit> {
        self.limits.iter().find(|l| l.field == field)
    }

    /// The values out of the limits and the failed tests.
    fn evaluate(&self) -> Vec<Field> {
        let r = &self.result;
        let mut out: Vec<Field> = self.limits.iter().filter(|l| number(l.field.get(r)).is_some_and(|v| !l.passes(v))).map(|l| l.field).collect();
        out.extend(Field::ALL.into_iter().filter(|f| f.is_answer() && f.get(r) == FUNCTION_FAILED));
        out.sort_by_key(|f| Field::ALL.iter().position(|x| x == f));
        out
    }

    fn verdict(&self) -> Verdict {
        let r = &self.result;
        let given = |f: Field| if f.is_measurement() { number(f.get(r)).is_some() } else { !f.get(r).is_empty() };
        if !self.failed.is_empty() {
            Verdict::Failed
        } else if !Field::ALL.iter().any(|f| f.is_test() && given(*f)) {
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

    /// A cell of the report: "–" for a test the circuit does not have.
    pub fn cell(&self, f: Field) -> String {
        if (f.is_rcd() && self.rcd.is_none()) || (f == Field::PhaseSequence && !self.multi_phase()) {
            "–".into()
        } else {
            f.display(&self.result).replace('\n', " ")
        }
    }
}

/// A power supply or transformer that may feed SELV, PELV or separated circuits.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub device: String,
    /// Page index and id of its symbol.
    pub target: (usize, u64),
    /// Power supply (DC output) or transformer.
    pub transformer: bool,
    pub description: Option<String>,
    /// Output voltage, V; `None`: not given.
    pub voltage: Option<f32>,
    /// The ranges by protective measure (`selv`, `pelv`, `separation`).
    pub limits: BTreeMap<String, Vec<KeyLimit>>,
    pub result: SourceResult,
    pub verdict: Verdict,
    /// What fails: the field keys, `measure` for a voltage too high for SELV or PELV.
    pub failed: Vec<String>,
}

impl Source {
    /// The highest voltage of SELV and PELV circuits from this source (AC 50 V, DC 120 V).
    pub fn selv_limit(&self) -> f32 {
        if self.transformer { SELV_AC } else { SELV_DC }
    }

    fn evaluate(&mut self) {
        let r = &self.result;
        let mut failed = Vec::new();
        if matches!(r.measure.as_str(), "selv" | "pelv") && self.voltage.is_some_and(|v| v > self.selv_limit()) {
            failed.push(SourceField::Measure.key().to_string());
        }
        for l in self.limits.get(&r.measure).into_iter().flatten() {
            let f = SourceField::from_key(&l.key).unwrap_or(SourceField::Remark);
            if number(f.get(r)).is_some_and(|v| !l.passes(v)) {
                failed.push(l.key.clone());
            }
        }
        let given = |f: SourceField| if f.is_measurement() { number(f.get(r)).is_some() } else { !f.get(r).is_empty() };
        self.verdict = if !failed.is_empty() {
            Verdict::Failed
        } else if r.measure == NOT_APPLICABLE {
            Verdict::Passed
        } else if !SourceField::ALL.iter().any(|f| *f != SourceField::Remark && given(*f)) {
            Verdict::NotTested
        } else if [SourceField::Measure, SourceField::InsulationVoltage, SourceField::Insulation].into_iter().all(given) {
            Verdict::Passed
        } else {
            Verdict::Incomplete
        };
        self.failed = failed;
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

/// What a device is for: its description, else a function text.
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
    // IT: the second fault in another phase (411.6.4): U / 2 without, U0 / 2 with a
    // distributed neutral.
    let it_voltage = (supply.earthing == Earthing::It).then(|| if supply.neutral { u0 } else { supply.voltage } / 2.0);
    let rules = crate::rules::limits(prj);
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
            // (Zs × Ia ≤ U0, also in TT systems: 411.5.4; IT: the second fault), or by
            // the residual current device: Zs × IΔn ≤ U0 (TN, IT), RA × IΔn ≤ UL in TT
            // systems (411.5.3; Zs includes RA, so it is checked against the same limit).
            let by_device = faults
                .iter()
                .find(|x| x.device == f.device)
                .filter(|_| characteristic.is_some())
                .map(|x| x.trip_current)
                .filter(|ia| ia.is_finite() && *ia > 0.0)
                .map(|ia| LOOP_FACTOR * it_voltage.unwrap_or(u0) / ia);
            let by_rcd = rcd.as_ref().and_then(|d| d.rated).map(|i| if tt { CONTACT_VOLTAGE } else { u0 } / (i / 1000.0));
            let zs_max = match (by_device, by_rcd) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            };
            let limit = |field, (min, max): (Option<f32>, Option<f32>)| Limit { field, min, max };
            let mut limits = vec![
                limit(Field::InsulationVoltage, (Some(insulation_voltage(prj)), None)),
                limit(Field::Insulation, (Some(MIN_INSULATION), None)),
                limit(Field::VoltageDrop, (None, Some(rules.voltage_drop))),
            ];
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
            limits.sort_by_key(|l| Field::ALL.iter().position(|x| *x == l.field));
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

/// The output voltage of a power supply or transformer: the part's `Voltage` after
/// the `/` (`230 V / 24 V DC`), else a `Output voltage` of its tech data.
fn output_voltage(prj: &Project, tag: &str) -> Option<f32> {
    crate::sizing::device_parts(prj, tag).iter().find_map(|p| {
        let after = p.voltage.split_once('/').map(|(_, b)| b.trim());
        after.and_then(crate::sizing::parse_voltage).or_else(|| p.tech_value("Output voltage").and_then(crate::sizing::parse_voltage))
    })
}

/// The power supplies and transformers of the schematic that may feed SELV, PELV or
/// separated circuits, in schematic order, with the results of `prj.inspection`.
pub fn sources(prj: &Project) -> Vec<Source> {
    let mut out: Vec<Source> = Vec::new();
    for (pi, pg) in prj.pages.iter().enumerate().filter(|(_, p)| p.kind == PageKind::Schematic) {
        for s in &pg.symbols {
            let kind = kind_of_symbol(&s.sym);
            if !matches!(kind, Some(FnKind::PowerSupply | FnKind::Transformer)) || s.tag.is_empty() || out.iter().any(|o| o.device == s.tag) {
                continue;
            }
            let k = |key: &str, min: f32| KeyLimit::new(key, Some(min), None);
            // Table 6.1: SELV and PELV at 250 V, at least 0.5 MΩ; separation as the mains.
            let selv = vec![k(SourceField::InsulationVoltage.key(), 250.0), k(SourceField::Insulation.key(), 0.5)];
            let separation = vec![k(SourceField::InsulationVoltage.key(), insulation_voltage(prj)), k(SourceField::Insulation.key(), MIN_INSULATION)];
            let limits = BTreeMap::from([("selv".to_string(), selv.clone()), ("pelv".to_string(), selv), ("separation".to_string(), separation)]);
            let mut src = Source {
                device: s.tag.clone(),
                target: (pi, s.id),
                transformer: kind == Some(FnKind::Transformer),
                description: description(prj, &s.tag),
                voltage: output_voltage(prj, &s.tag),
                limits,
                result: prj.inspection.source(&s.tag).cloned().unwrap_or_else(|| SourceResult { device: s.tag.clone(), ..Default::default() }),
                verdict: Verdict::NotTested,
                failed: Vec::new(),
            };
            src.evaluate();
            out.push(src);
        }
    }
    out
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
/// inspection, the sources, the circuits and the verdict.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Report {
    /// Line-to-earth voltage of the supply, V.
    pub u0: f32,
    pub earthing: String,
    /// IT system: the insulation monitoring device and the first fault are tested.
    pub it: bool,
    /// Test voltage of the insulation measurements, V DC.
    pub insulation_voltage: f32,
    /// The ranges of the values of the report by key: the earth electrode (TT: UL / the
    /// largest rated residual current), insulating floors and walls, the first fault
    /// current (IT: UL / RA).
    pub limits: Vec<KeyLimit>,
    pub header: Header,
    pub checks: Vec<CheckItem>,
    pub sources: Vec<Source>,
    pub circuits: Vec<Circuit>,
    pub verdict: Verdict,
    /// What fails besides the circuits and sources: header keys, `check.<key>`.
    pub failed: Vec<String>,
}

impl Report {
    /// The report of the project (see [`circuits`] for `nl` and `conns`).
    pub fn of(prj: &Project, nl: &Netlist, conns: &[Connection]) -> Report {
        let supply = &prj.electrical.supply;
        let circuits = circuits(prj, nl, conns);
        let sources = sources(prj);
        let h = &prj.inspection.header;
        let tt = supply.earthing == Earthing::Tt;
        let it = supply.earthing == Earthing::It;
        let mut limits = Vec::new();
        let largest = circuits.iter().filter_map(|c| c.rcd.as_ref()?.rated).reduce(f32::max);
        if let Some(i) = largest.filter(|_| tt) {
            limits.push(KeyLimit::new(HeaderField::EarthElectrode.key(), None, Some(CONTACT_VOLTAGE / (i / 1000.0))));
        }
        // Non-conducting locations (IEC 60364-4-41, C.1): 50 kΩ up to 500 V, 100 kΩ above.
        limits.push(KeyLimit::new(HeaderField::FloorInsulation.key(), Some(if supply.voltage <= 500.0 { 50.0 } else { 100.0 }), None));
        if let Some(ra) = number(&h.earth_electrode).filter(|ra| it && *ra > 0.0) {
            // IT, first fault: RA × Id ≤ UL (411.6.2).
            limits.push(KeyLimit::new(HeaderField::FirstFault.key(), None, Some(CONTACT_VOLTAGE / ra)));
        }
        let checks: Vec<CheckItem> = CHECKS.iter().map(|(k, t)| CheckItem { key: k.to_string(), title: t.to_string(), answer: h.checks.get(*k).cloned().unwrap_or_default() }).collect();
        let mut failed: Vec<String> = limits
            .iter()
            .filter(|l| HeaderField::ALL.iter().find(|f| f.key() == l.key).and_then(|f| number(f.get(h))).is_some_and(|v| !l.passes(v)))
            .map(|l| l.key.clone())
            .collect();
        if it && h.imd == FUNCTION_FAILED {
            failed.push(HeaderField::Imd.key().to_string());
        }
        failed.extend(checks.iter().filter(|c| c.answer == FUNCTION_FAILED).map(|c| format!("check.{}", c.key)));
        let started = checks.iter().any(|c| !c.answer.is_empty())
            || circuits.iter().any(|c| c.verdict != Verdict::NotTested)
            || sources.iter().any(|s| s.verdict != Verdict::NotTested)
            || HeaderField::ALL.iter().any(|f| f.is_measurement() && !f.get(h).is_empty());
        let complete = checks.iter().all(|c| !c.answer.is_empty())
            && circuits.iter().all(|c| c.verdict == Verdict::Passed)
            && sources.iter().all(|s| s.verdict == Verdict::Passed)
            && !h.inspector.trim().is_empty()
            && (!tt || number(&h.earth_electrode).is_some())
            && (!it || !h.imd.is_empty());
        let verdict = if !failed.is_empty() || circuits.iter().any(|c| c.verdict == Verdict::Failed) || sources.iter().any(|s| s.verdict == Verdict::Failed) {
            Verdict::Failed
        } else if !started {
            Verdict::NotTested
        } else if complete {
            Verdict::Passed
        } else {
            Verdict::Incomplete
        };
        Report {
            u0: supply.u0(),
            earthing: supply.earthing.title().into(),
            it,
            insulation_voltage: insulation_voltage(prj),
            limits,
            header: h.clone(),
            checks,
            sources,
            circuits,
            verdict,
            failed,
        }
    }

    /// The range of a value of the report.
    pub fn limit(&self, key: &str) -> Option<&KeyLimit> {
        self.limits.iter().find(|l| l.key == key)
    }

    /// The fields of the report header that apply (the IT tests only in IT systems).
    pub fn header_fields(&self) -> Vec<HeaderField> {
        HeaderField::ALL.into_iter().filter(|f| self.it || !f.is_it()).collect()
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
    cols.extend(Field::ALL.iter().filter(|f| f.is_test()).map(|f| (f.title_key(), 0.6)));
    cols.push((tr_const!("Result"), 0.7));
    cols.extend(Field::ALL.iter().filter(|f| !f.is_test()).map(|f| (f.title_key(), if *f == Field::Remark { 1.4 } else { 0.8 })));
    let mut t = Table::with_widths(&cols);
    for c in circuits {
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
        row.extend(Field::ALL.iter().filter(|f| f.is_test()).map(|f| c.cell(*f)));
        row.push(c.verdict.title().into());
        row.extend(Field::ALL.iter().filter(|f| !f.is_test()).map(|f| c.cell(*f)));
        t.push(row, Some(c.target));
    }
    t
}

/// The sources of SELV, PELV and separated circuits: one row per power supply or
/// transformer.
pub fn source_table(sources: &[Source]) -> Table {
    let mut t = Table::with_widths(&[
        (tr_const!("Source"), 0.7),
        (tr_const!("Description"), 1.4),
        (tr_const!("Output voltage (V)"), 0.7),
        (SourceField::Measure.title_key(), 1.0),
        (SourceField::InsulationVoltage.title_key(), 0.7),
        (SourceField::Insulation.title_key(), 0.7),
        (tr_const!("Result"), 0.7),
        (SourceField::Remark.title_key(), 1.4),
    ]);
    for s in sources {
        let r = &s.result;
        let voltage = s.voltage.map(|v| format!("{} {}", fmt(v), if s.transformer { "AC" } else { "DC" }));
        let row = vec![
            s.device.clone(),
            or_missing(s.description.clone()),
            or_missing(voltage),
            SourceField::Measure.display(r),
            SourceField::InsulationVoltage.display(r),
            SourceField::Insulation.display(r),
            s.verdict.title().into(),
            SourceField::Remark.display(r),
        ];
        t.push(row, Some(s.target));
    }
    t
}

/// The data of the report, the visual inspection, the sources and the verdict: item,
/// value and the limit or a note.
pub fn summary_table(r: &Report) -> Table {
    let mut t = Table::with_widths(&[(tr_const!("Item"), 2.4), (tr_const!("Value"), 1.6), (tr_const!("Note"), 1.4)]);
    let h = &r.header;
    t.push(vec![tr("Supply system").into(), format!("U0 {} V, {}", fmt(r.u0.round()), r.earthing), String::new()], None);
    for f in r.header_fields() {
        t.push(vec![f.title().into(), f.display(h), r.limit(f.key()).map(KeyLimit::text).unwrap_or_default()], None);
    }
    t.push(vec![Field::InsulationVoltage.title().into(), fmt(r.insulation_voltage), format!("≥ {} MΩ", fmt(MIN_INSULATION))], None);
    for c in &r.checks {
        let title = CHECKS.iter().find(|x| x.0 == c.key).map_or(c.title.as_str(), |x| tr(x.1));
        t.push(vec![title.into(), if c.answer.is_empty() { String::new() } else { answer_text(&c.answer) }, String::new()], None);
    }
    for s in &r.sources {
        t.push(vec![tr_fmt!("Source {}", s.device), SourceField::Measure.display(&s.result), s.verdict.title().into()], None);
    }
    let passed = r.circuits.iter().filter(|c| c.verdict == Verdict::Passed).count();
    t.push(vec![tr("Circuits").into(), tr_fmt!("{} of {} passed", passed, r.circuits.len()), String::new()], None);
    t.push(vec![tr("Result").into(), r.verdict.title().into(), String::new()], None);
    t
}

#[cfg(test)]
mod tests;
