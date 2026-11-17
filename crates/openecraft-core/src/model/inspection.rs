//! Data of `crate::inspection` that the project file holds (see `docs/architecture.md`).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The inspection and test of the installation (initial verification, IEC 60364-6 /
/// DIN VDE 0100-600): the data of the report, the visual inspection and the results
/// of each circuit. Shared projects merge the header and each circuit's results as
/// records, like the review: entering them needs no lock.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Inspection {
    #[serde(skip_serializing_if = "Header::is_empty")]
    pub header: Header,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<TestResult>,
    /// Protection by SELV, PELV or electrical separation, per source.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<SourceResult>,
}

/// The data of the report as a whole: who, where, why, what was measured at the
/// origin, the visual inspection and the signature. Values as entered.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Header {
    /// Number of edits: merging keeps the header with more.
    pub rev: u64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub client: String,
    /// Address or name of the installation.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub site: String,
    /// Company that built the installation.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub installer: String,
    /// New installation, extension, change, repair.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub reason: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub inspector: String,
    /// Test instruments used (IEC 61557: make, type, serial number).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub instruments: String,
    /// External earth fault loop impedance at the origin, Ω.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ze: String,
    /// Prospective fault current at the origin, kA.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ik: String,
    /// Resistance of the earth electrode (TT system), Ω.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub earth_electrode: String,
    /// Continuity of the main equipotential bonding, Ω.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub bonding: String,
    /// Resistance of insulating floors and walls (non-conducting locations), kΩ;
    /// empty: there are none.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub floor_insulation: String,
    /// IT system: function test of the insulation monitoring device (`ok`, `failed`, `n/a`).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub imd: String,
    /// IT system: current of a first fault to earth, A.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub first_fault: String,
    /// Date of the next inspection (`YYYY-MM-DD`).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub next_inspection: String,
    /// Who signed the report; any change of the data afterwards clears it.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub signed_by: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub signed_on: String,
    /// The visual inspection: `ok`, `failed` or `n/a` by item (`crate::inspection::CHECKS`).
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub checks: BTreeMap<String, String>,
    /// Items of the visual inspection added for this installation.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub own_checks: Vec<OwnCheck>,
    /// Limits that differ from the defaults (`crate::inspection::LIMITS`) by key.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub limits: BTreeMap<String, f32>,
}

/// An item of the visual inspection added for this installation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OwnCheck {
    pub title: String,
    /// `ok`, `failed`, `n/a` or empty.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub answer: String,
}

impl Header {
    pub fn is_empty(&self) -> bool {
        *self == Header::default()
    }
}

/// What was measured and found in one circuit. Values are kept as entered (decimal
/// comma or point); empty: not measured.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TestResult {
    /// Device tag of the protective device of the circuit.
    pub circuit: String,
    /// Number of edits: merging keeps the record with more.
    pub rev: u64,
    /// Continuity of the protective conductor, Ω.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub continuity: String,
    /// Test voltage of the insulation measurement, V DC.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub insulation_voltage: String,
    /// Insulation resistance between the line conductors (or all live conductors
    /// connected together) and the protective conductor, MΩ.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub insulation: String,
    /// Insulation resistance between the neutral and the protective conductor, MΩ.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub insulation_n_pe: String,
    /// Insulation resistance between the live conductors, MΩ.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub insulation_l_n: String,
    /// Earth fault loop impedance Zs, Ω.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub loop_impedance: String,
    /// Voltage drop measured at the end of the circuit, %.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub voltage_drop: String,
    /// Tripping current of the residual current device, mA.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rcd_current: String,
    /// Tripping time of the residual current device at its rated residual current, ms.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rcd_time: String,
    /// Tripping time at five times the rated residual current, ms.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rcd_time_5: String,
    /// Contact voltage at the rated residual current, V.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rcd_voltage: String,
    /// Polarity: `ok`, `failed` or empty.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub polarity: String,
    /// Phase sequence of multi-phase circuits: `ok`, `failed` or empty.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub phase_sequence: String,
    /// Functional test (switchgear, interlocks, controls): `ok`, `failed` or empty.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub function: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub remark: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tester: String,
    /// Date of the test (`YYYY-MM-DD`).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub date: String,
    /// Test instrument (make, type, serial number).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub instrument: String,
}

/// What was found for a power supply or transformer feeding circuits protected by
/// SELV, PELV or electrical separation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SourceResult {
    /// Device tag of the power supply or transformer.
    pub device: String,
    /// Number of edits: merging keeps the record with more.
    pub rev: u64,
    /// `selv`, `pelv`, `separation` or `n/a` (no protective measure of its own).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub measure: String,
    /// Test voltage of the insulation measurement, V DC.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub insulation_voltage: String,
    /// Insulation resistance of the separated circuits against other circuits and
    /// earth, MΩ.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub insulation: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub remark: String,
}
