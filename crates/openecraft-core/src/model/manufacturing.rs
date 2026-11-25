//! Data of `crate::manufacturing` that the project file holds (see `docs/architecture.md`).

use super::panel::Face;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::geom::P;

/// Manufacturing data of the panel layout ([`crate::panel::Layout::manufacturing`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Manufacturing {
    /// Machines of the project; they replace built-in machines of the same id.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub machines: Vec<Machine>,
    /// Machine for the sheet metal faces (plate, door, walls).
    pub sheet_machine: String,
    /// Machine for faces that are not made on the sheet machine, by face key.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub face_machines: BTreeMap<String, String>,
    /// Machine for copper bars.
    pub bar_machine: String,
    pub strategy: Strategy,
    /// Bent and drilled copper bars.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bars: Vec<CopperBar>,
    pub bar_rules: BarRules,
    /// Jobs handed off to production.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub jobs: Vec<Job>,
    /// Feedback of the production, oldest first.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub feedback: Vec<Feedback>,
}

/// A manufacturing job handed off to production.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Job {
    /// `CAB1-1`, `CAB1-2`, …
    pub id: String,
    pub enclosure: String,
    pub date: String,
    /// Project revision at the hand-off.
    pub revision: String,
    pub programs: Vec<ProgramRef>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProgramRef {
    pub name: String,
    pub machine: String,
    /// Face key, `bars` for the copper bars.
    pub face: String,
    pub operations: usize,
    pub time: f32,
    /// Time factor of the machine in the estimate.
    pub factor: f32,
    pub checksum: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CopperBar {
    pub id: u64,
    /// Enclosure the bar is mounted in (name).
    pub enclosure: String,
    pub name: String,
    pub width: f32,
    pub thickness: f32,
    /// Number of equal bars (e.g. one per phase).
    pub count: u32,
    /// Outer lengths of the straight segments (up to the virtual sharps), mm.
    pub segments: Vec<f32>,
    /// One bend between two segments.
    pub bends: Vec<Bend>,
    pub holes: Vec<BarHole>,
    /// Diameter of a hole at both ends (0: none).
    pub end_holes: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BarHole {
    /// Straight segment of the bar (from 0).
    pub segment: usize,
    /// Distance from the start of the segment (outer dimension), mm.
    pub at: f32,
    pub diameter: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Bend {
    /// Bend angle in degrees (deviation from straight); negative bends the other way.
    pub angle: f32,
    /// Bent about the bar's thickness (on edge) instead of flat.
    pub edgewise: bool,
    /// Inner radius in mm, 0: from the rules.
    pub radius: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Strategy {
    pub cutouts: Method,
    /// Threads are cut on the machine; otherwise they are only drilled.
    pub tap: bool,
    /// Round holes larger than this are milled although a drill exists, mm.
    pub drill_max: f32,
    /// Largest difference between a hole and its drill or punch, mm.
    pub tolerance: f32,
    /// Overlap of neighbouring nibbling hits (0 … 0.9 of the tool size).
    pub nibble_overlap: f32,
    /// Improves the path between the hits of a tool (2-opt) after the nearest neighbour.
    pub optimize: bool,
}

/// How openings without a matching single tool are made.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    /// A punch of the opening's size if there is one, else the contour (milled on
    /// machining centres, nibbled on punching machines).
    #[default]
    Auto,
    /// Always the contour, also when a punch of the size exists.
    Contour,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Feedback {
    pub job: String,
    /// Program file; empty for the whole job.
    pub program: String,
    /// Operation (sequence number); 0 for the whole program.
    pub seq: usize,
    /// Device tag or item the feedback is about.
    pub item: String,
    pub status: Status,
    /// Measured deviation of the position, mm.
    pub dx: f32,
    pub dy: f32,
    /// Actual machine time, s (0: unknown).
    pub time: f32,
    pub message: String,
    pub date: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Done,
    Started,
    Failed,
    /// Made, but has to be reworked.
    Rework,
    Skipped,
}

/// A program of a job.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    /// File name.
    pub name: String,
    pub machine: String,
    /// The face; None for the copper bars.
    pub face: Option<Face>,
    /// The leaf of a door of several leaves.
    pub leaf: Option<usize>,
    /// Key of the sheet: the face's, `door_left`, `bars`, ….
    pub sheet: String,
    pub operations: usize,
    /// Estimated machine time, s.
    pub time: f32,
    /// Time factor of the machine in the estimate.
    pub factor: f32,
    pub problems: Vec<String>,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Operation {
    /// Sequence number in the program, from 1.
    pub seq: usize,
    pub item: u64,
    /// Tool number, None for studs and manual work.
    pub tool: Option<u32>,
    /// Face coordinates (origin top left, y down).
    pub pos: P,
    pub kind: OpKind,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tool {
    /// Tool number (pocket / station).
    pub number: u32,
    pub kind: ToolKind,
    /// Diameter, width of rectangular punches, nominal diameter of taps.
    pub diameter: f32,
    /// Height of rectangular punches.
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    #[default]
    Drill,
    Tap,
    /// End mill: round contours and rectangular cutouts.
    Mill,
    PunchRound,
    /// Rectangular or square punch (width × height).
    PunchRect,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OpKind {
    Drill { diameter: f32 },
    Tap { thread: String },
    /// One hit of a punch (round or rectangular; `turned`: by 90°).
    Punch { turned: bool },
    /// Hits along the contour of an opening, in order.
    Nibble { hits: Vec<P> },
    /// Circular milling of a round opening (centre at the operation's position).
    MillCircle { diameter: f32 },
    /// Contour milling of a rectangular opening.
    MillRect { w: f32, h: f32 },
    /// Weld stud: set by the stud welder, not on the machine.
    Stud { thread: String },
    /// The machine cannot make it.
    Manual { what: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BarRules {
    /// Position of the neutral fibre (0 … 1 of the thickness); 0: DIN 6935.
    pub k_factor: f32,
    /// Inner radius of bends without one, as a multiple of the thickness.
    pub radius: f32,
    /// Smallest inner radius, multiple of the thickness.
    pub min_radius: f32,
    /// Shortest straight length next to a bend, mm.
    pub min_flange: f32,
    /// Springback of copper, percent of the angle.
    pub springback: f32,
    /// Stock length of the bars, mm.
    pub stock_length: f32,
    /// Saw cut, mm.
    pub kerf: f32,
    /// Centre of the end holes from the bar end, multiple of the width (0.5 … 2).
    pub end_distance: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Machine {
    /// Unique, snake_case.
    pub id: String,
    pub name: String,
    pub kind: MachineKind,
    pub dialect: Dialect,
    /// Working area (x × y) in mm; the length of bars on busbar machines.
    pub area: (f32, f32),
    /// Thickest sheet or bar in mm.
    pub max_thickness: f32,
    pub tools: Vec<Tool>,
    /// Rapid traverse, mm/min.
    pub rapid: f32,
    /// Milling feed, mm/min.
    pub feed: f32,
    /// Time of one drilling, tapping, punching or bending operation, s.
    pub hit_time: f32,
    /// Time of a tool change, s.
    pub tool_change: f32,
    /// Correction of the time estimate, calibrated from the production feedback.
    pub time_factor: f32,
}

/// Program format of the machine's controller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dialect {
    /// ISO G-code (DIN 66025) with drilling and tapping cycles.
    #[default]
    Iso,
    /// Heidenhain conversational (Klartext) with cycles 200 and 207.
    Heidenhain,
    /// ISO punching code: one hit per block.
    Punch,
    /// Neutral operation list (CSV), also for busbar machines.
    Csv,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MachineKind {
    /// Drilling and milling centre for sheets.
    #[default]
    Machining,
    /// Punching machine (turret or single punch) for sheets.
    Punching,
    /// Cutting, punching and bending of copper bars.
    Busbar,
}
