//! Data of `crate::inspection` that the project file holds (see `docs/architecture.md`).

use serde::{Deserialize, Serialize};

/// The results of the inspection and test of the installation (initial verification,
/// IEC 60364-6 / DIN VDE 0100-600), one record per circuit. Shared projects merge
/// them record by record, like the review: entering results needs no lock.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Inspection {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<TestResult>,
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
    /// Insulation resistance, MΩ.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub insulation: String,
    /// Earth fault loop impedance Zs, Ω.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub loop_impedance: String,
    /// Tripping current of the residual current device, mA.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rcd_current: String,
    /// Tripping time of the residual current device at the rated residual current, ms.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rcd_time: String,
    /// Functional test (polarity, rotation, switching): `ok`, `failed` or empty.
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
