//! Post-processors: the program of a [`Plan`] in the dialect of the machine's
//! controller. Coordinates from the bottom left corner of the face, y up (like the NC
//! lists of [`crate::production`]); the programs carry no date, so that unchanged
//! machining gives the same program (checksums of the hand-off).

use super::machines::{Dialect, Machine};
use super::num;
use super::strategy::{OpKind, Plan};
use crate::geom::P;
use std::fmt::Write;

/// Pitch of metric coarse threads.
pub fn thread_pitch(thread: &str) -> f32 {
    match thread.trim_start_matches('M') {
        "3" => 0.5,
        "4" => 0.7,
        "5" => 0.8,
        "6" => 1.0,
        "8" => 1.25,
        "10" => 1.5,
        _ => 1.0,
    }
}

/// File name of a program: `CAB1_plate.nc`, `CAB1_door_left.nc`.
pub fn file_name(plan: &Plan, m: &Machine) -> String {
    format!("{}_{}.{}", super::file_stem(&plan.enclosure), plan.sheet, m.dialect.extension())
}

/// The program of a plan for its machine.
pub fn program(plan: &Plan, m: &Machine) -> String {
    match m.dialect {
        Dialect::Iso => iso(plan, m),
        Dialect::Heidenhain => heidenhain(plan, m),
        Dialect::Punch => punch(plan, m),
        Dialect::Csv => csv(plan, m),
    }
}

fn xy(plan: &Plan, q: P) -> (String, String) {
    (num(q.x), num(plan.height - q.y))
}

fn tool_label(m: &Machine, n: u32) -> String {
    m.tool_by_number(n).map(|t| t.name()).unwrap_or_default()
}

/// Spindle speed for a tool (cutting speed 25 m/min for drills and mills).
fn speed(m: &Machine, n: u32) -> u32 {
    let d = m.tool_by_number(n).map(|t| t.diameter).unwrap_or(5.0).max(1.0);
    ((25_000.0 / (std::f32::consts::PI * d)) / 10.0).round() as u32 * 10
}

fn iso(plan: &Plan, m: &Machine) -> String {
    let mut out = String::from("%\n");
    let _ = writeln!(out, "O0001 (OPENECRAFT {} {})", plan.enclosure, plan.sheet.to_uppercase());
    let _ = writeln!(out, "(SHEET {} X {} X {})", num(plan.width), num(plan.height), num(plan.thickness));
    out.push_str("G21 G90 G17 G40 G49 G80\nG0 Z50\n");
    let depth = num(-(plan.thickness + 1.0));
    let mut tool = None;
    let mut cycle = "";
    for o in plan.ops.iter().filter(|o| o.tool.is_some()) {
        let n = o.tool.unwrap();
        if tool != Some(n) {
            if !cycle.is_empty() {
                out.push_str("G80\n");
                cycle = "";
            }
            let _ = writeln!(out, "T{n} M6 ({})", tool_label(m, n));
            let _ = writeln!(out, "S{} M3", speed(m, n));
            tool = Some(n);
        }
        let (x, y) = xy(plan, o.pos);
        match &o.kind {
            OpKind::Drill { .. } => {
                if cycle != "G81" {
                    let _ = writeln!(out, "G81 X{x} Y{y} Z{depth} R2 F150");
                    cycle = "G81";
                } else {
                    let _ = writeln!(out, "X{x} Y{y}");
                }
            }
            OpKind::Tap { thread } => {
                if cycle != "G84" {
                    let _ = writeln!(out, "G84 X{x} Y{y} Z{depth} R4 F{}", num(thread_pitch(thread) * speed(m, n) as f32));
                    cycle = "G84";
                } else {
                    let _ = writeln!(out, "X{x} Y{y}");
                }
            }
            OpKind::MillCircle { diameter } => {
                if !cycle.is_empty() {
                    out.push_str("G80\n");
                    cycle = "";
                }
                let r = (diameter - m.tool_by_number(n).map(|t| t.diameter).unwrap_or(0.0)) / 2.0;
                let start = num(o.pos.x + r);
                let _ = writeln!(out, "G0 X{start} Y{y}\nG0 Z2\nG1 Z{depth} F{}\nG2 X{start} Y{y} I{} J0\nG0 Z50", num(m.feed / 2.0), num(-r));
            }
            OpKind::MillRect { w, h } => {
                if !cycle.is_empty() {
                    out.push_str("G80\n");
                    cycle = "";
                }
                let d = m.tool_by_number(n).map(|t| t.diameter).unwrap_or(0.0);
                let (hw, hh) = ((w - d) / 2.0, (h - d) / 2.0);
                let cy = plan.height - o.pos.y;
                let (x0, x1, y0, y1) = (num(o.pos.x - hw), num(o.pos.x + hw), num(cy - hh), num(cy + hh));
                let _ = writeln!(out, "G0 X{x0} Y{y0}\nG0 Z2\nG1 Z{depth} F{}\nG1 X{x1} F{}\nY{y1}\nX{x0}\nY{y0}\nG0 Z50", num(m.feed / 2.0), num(m.feed));
            }
            _ => {}
        }
    }
    if !cycle.is_empty() {
        out.push_str("G80\n");
    }
    manual_comments(&mut out, plan, |s| format!("({s})"));
    out.push_str("G0 Z50\nM30\n%\n");
    out
}

fn heidenhain(plan: &Plan, m: &Machine) -> String {
    let mut lines: Vec<String> = Vec::new();
    let pgm = super::file_stem(&format!("{}_{}", plan.enclosure, plan.sheet)).to_uppercase();
    lines.push(format!("BEGIN PGM {pgm} MM"));
    lines.push(format!("BLK FORM 0.1 Z X+0 Y+0 Z-{}", num(plan.thickness)));
    lines.push(format!("BLK FORM 0.2 X+{} Y+{} Z+0", num(plan.width), num(plan.height)));
    let depth = num(plan.thickness + 1.0);
    let sign = |v: f32| if v < 0.0 { format!("-{}", num(-v)) } else { format!("+{}", num(v)) };
    let mut tool = None;
    for o in plan.ops.iter().filter(|o| o.tool.is_some()) {
        let n = o.tool.unwrap();
        if tool != Some(n) {
            lines.push(format!("TOOL CALL {n} Z S{} ; {}", speed(m, n), tool_label(m, n)));
            lines.push("L Z+50 R0 FMAX".into());
            match &o.kind {
                OpKind::Drill { .. } => lines.push(format!("CYCL DEF 200 DRILLING Q200=+2 Q201=-{depth} Q206=+150 Q202=+{depth} Q210=+0 Q203=+0 Q204=+50 Q211=+0")),
                OpKind::Tap { thread } => lines.push(format!("CYCL DEF 207 RIGID TAPPING Q200=+2 Q201=-{depth} Q239=+{} Q203=+0 Q204=+50", num(thread_pitch(thread)))),
                _ => {}
            }
            tool = Some(n);
        }
        let (x, y) = (o.pos.x, plan.height - o.pos.y);
        match &o.kind {
            OpKind::Drill { .. } | OpKind::Tap { .. } => lines.push(format!("L X{} Y{} R0 FMAX M99", sign(x), sign(y))),
            OpKind::MillCircle { diameter } => {
                let r = (diameter - m.tool_by_number(n).map(|t| t.diameter).unwrap_or(0.0)) / 2.0;
                lines.push(format!("L X{} Y{} R0 FMAX", sign(x + r), sign(y)));
                lines.push(format!("L Z-{depth} F{}", num(m.feed / 2.0)));
                lines.push(format!("CC X{} Y{}", sign(x), sign(y)));
                lines.push(format!("C X{} Y{} DR- F{}", sign(x + r), sign(y), num(m.feed)));
                lines.push("L Z+50 R0 FMAX".into());
            }
            OpKind::MillRect { w, h } => {
                let d = m.tool_by_number(n).map(|t| t.diameter).unwrap_or(0.0);
                let (hw, hh) = ((w - d) / 2.0, (h - d) / 2.0);
                lines.push(format!("L X{} Y{} R0 FMAX", sign(x - hw), sign(y - hh)));
                lines.push(format!("L Z-{depth} F{}", num(m.feed / 2.0)));
                lines.push(format!("L X{} F{}", sign(x + hw), num(m.feed)));
                lines.push(format!("L Y{}", sign(y + hh)));
                lines.push(format!("L X{}", sign(x - hw)));
                lines.push(format!("L Y{}", sign(y - hh)));
                lines.push("L Z+50 R0 FMAX".into());
            }
            _ => {}
        }
    }
    let mut notes = String::new();
    manual_comments(&mut notes, plan, |s| format!("; {s}"));
    lines.extend(notes.lines().map(String::from));
    lines.push("L Z+50 R0 FMAX M30".into());
    lines.push(format!("END PGM {pgm} MM"));
    lines.iter().enumerate().map(|(i, l)| format!("{i} {l}\n")).collect()
}

fn punch(plan: &Plan, m: &Machine) -> String {
    let mut out = String::from("%\n");
    let _ = writeln!(out, "(OPENECRAFT {} {})", plan.enclosure, plan.sheet.to_uppercase());
    let _ = writeln!(out, "G92 X{} Y{}", num(plan.width), num(plan.height));
    out.push_str("G90 G21\n");
    let mut tool = None;
    for o in plan.ops.iter().filter(|o| o.tool.is_some()) {
        let n = o.tool.unwrap();
        if tool != Some(n) {
            let _ = writeln!(out, "T{n} ({})", tool_label(m, n));
            tool = Some(n);
        }
        let hits: Vec<P> = match &o.kind {
            OpKind::Nibble { hits } => hits.clone(),
            OpKind::Punch { .. } | OpKind::Tap { .. } | OpKind::Drill { .. } => vec![o.pos],
            _ => Vec::new(),
        };
        let rotate = matches!(o.kind, OpKind::Punch { turned: true });
        for q in hits {
            let (x, y) = xy(plan, q);
            let _ = writeln!(out, "X{x} Y{y}{}", if rotate { " C90" } else { "" });
        }
    }
    manual_comments(&mut out, plan, |s| format!("({s})"));
    out.push_str("G70 X0 Y0\nM30\n%\n");
    out
}

/// Neutral operation list (one row per operation).
fn csv(plan: &Plan, m: &Machine) -> String {
    let mut out = String::from("Enclosure;Face;Machine;Seq;Item;Operation;Tool;X;Y;Diameter;Width;Height;Thread;Hits\n");
    for o in &plan.ops {
        let (x, y) = xy(plan, o.pos);
        let tool = o.tool.map(|n| format!("T{n} {}", tool_label(m, n))).unwrap_or_default();
        let (d, w, h, thread, hits) = match &o.kind {
            OpKind::Drill { diameter } | OpKind::MillCircle { diameter } => (num(*diameter), String::new(), String::new(), String::new(), 1),
            OpKind::Tap { thread } | OpKind::Stud { thread } => (String::new(), String::new(), String::new(), thread.clone(), 1),
            OpKind::MillRect { w, h } => (String::new(), num(*w), num(*h), String::new(), 1),
            OpKind::Nibble { hits } => (String::new(), String::new(), String::new(), String::new(), hits.len()),
            OpKind::Punch { .. } => (String::new(), String::new(), String::new(), String::new(), 1),
            OpKind::Manual { what } => (String::new(), String::new(), String::new(), what.clone(), 0),
        };
        let _ = writeln!(out, "{};{};{};{};{};{};{tool};{x};{y};{d};{w};{h};{thread};{hits}", plan.enclosure, plan.sheet, m.id, o.seq, o.item, o.kind.key());
    }
    out
}

/// Comments for the work that is not done on the machine (studs, manual work).
fn manual_comments(out: &mut String, plan: &Plan, fmt: impl Fn(&str) -> String) {
    for o in plan.ops.iter().filter(|o| o.tool.is_none()) {
        let (x, y) = xy(plan, o.pos);
        let what = match &o.kind {
            OpKind::Stud { thread } => format!("STUD {thread} X{x} Y{y}"),
            OpKind::Manual { what } => format!("MANUAL {} X{x} Y{y}", what.replace('Ø', "D")),
            _ => continue,
        };
        let _ = writeln!(out, "{}", fmt(&what));
    }
}

#[cfg(test)]
mod tests;
