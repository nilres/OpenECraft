//! Terminal strip commands: list strips, show a strip with its terminals, levels,
//! jumpers and accessories, change terminals (level, jumper), jumper kinds, spare
//! terminals and the order.

use crate::builtin::common::{int_in};
use crate::params::{optional, param, Args, Kind};
use crate::{Info, Plugin, Registry};
use super::{add_commands, info, Spec};
use openecraft_core::analysis::Analysis;
use openecraft_core::model::{JumperKind, Project};
use openecraft_core::strips::{self, ElementKind};
use openecraft_i18n::{tr_const, tr_fmt};
use serde_json::{json, Value};

pub struct Terminals;

impl Plugin for Terminals {
    fn info(&self) -> Info {
        info("terminals", tr_const!("Terminal strips"), tr_const!("Terminal order, levels, jumpers, spare terminals and accessories"))
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        add_commands(reg, COMMANDS)
    }
}

const STRIP: crate::params::Param = param("strip", Kind::Str, tr_const!("Tag of the terminal strip, e.g. -X1"));

pub static COMMANDS: &[Spec] = &[
    Spec {
        name: "create_strip",
        help: tr_const!("Creates an empty terminal strip with a new device tag"),
        params: &[STRIP],
        run: create_strip,
    },
    Spec {
        name: "add_terminal",
        help: tr_const!("Adds an unplaced spare terminal to a strip; it can be placed from the navigator"),
        params: &[STRIP],
        run: add_terminal,
    },
    Spec {
        name: "list_strips",
        help: tr_const!("Lists the terminal strips with their number of terminals and length on the rail"),
        params: &[],
        run: list_strips,
    },
    Spec {
        name: "get_strip",
        help: tr_const!("Shows a terminal strip: terminals in strip order with level, jumper and targets, the blocks on the rail and the jumpers"),
        params: &[STRIP],
        run: get_strip,
    },
    Spec {
        name: "set_terminal",
        help: tr_const!("Sets the level (multi-level terminals) or jumper number of a terminal"),
        params: &[
            STRIP,
            param("terminal", Kind::Str, tr_const!("Terminal designation")),
            optional("level", Kind::Int, tr_const!("Level of a multi-level terminal (1 = bottom); upper levels sit on the block of the terminal before")),
            optional("jumper", Kind::Int, tr_const!("Jumper number (terminals with the same number are bridged); 0 removes it")),
        ],
        run: set_terminal,
    },
    Spec {
        name: "set_jumper_kind",
        help: tr_const!("Sets how the terminals of a jumper are bridged"),
        params: &[STRIP, param("jumper", Kind::Int, tr_const!("Jumper number")), param("kind", Kind::Str, tr_const!("plug_in, saddle or wire"))],
        run: set_jumper_kind,
    },
    Spec {
        name: "set_strip",
        help: tr_const!("Sets the spare terminals of a strip and whether its accessories are added automatically"),
        params: &[
            STRIP,
            optional("spares", Kind::Int, tr_const!("Spare terminals at the end of the strip")),
            optional("automatic_accessories", Kind::Bool, tr_const!("End brackets and end plates from the tech data of the terminal parts")),
            optional("order", Kind::Str, tr_const!("Terminal designations in strip order, separated by commas")),
        ],
        run: set_strip,
    },
    Spec {
        name: "renumber_strip",
        help: tr_const!("Numbers the terminals of a strip 1 … n in strip order; non-numeric designations such as PE stay"),
        params: &[STRIP],
        run: renumber_strip,
    },
];

fn create_strip(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let tag = openecraft_core::ops::create_terminal_strip(prj, a.req("strip"))?;
    Ok(json!({ "strip": tag }))
}

fn add_terminal(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let strip = a.req("strip").trim();
    let label = strips::add_terminal(prj, strip)?;
    Ok(json!({ "strip": strip, "terminal": label, "spare": true }))
}

fn kind_code(k: JumperKind) -> &'static str {
    match k {
        JumperKind::PlugIn => "plug_in",
        JumperKind::Saddle => "saddle",
        JumperKind::Wire => "wire",
    }
}

fn existing(prj: &Project, strip: &str) -> Result<(), String> {
    if strips::strip_tags(prj).iter().any(|t| t == strip) {
        Ok(())
    } else {
        Err(tr_fmt!("No terminal strip {}", strip))
    }
}

fn list_strips(prj: &mut Project, _: &Args) -> Result<Value, String> {
    let list: Vec<Value> = strips::strip_tags(prj)
        .into_iter()
        .map(|tag| {
            let layout = strips::strip_layout(prj, &tag);
            let terminals = strips::terminals(prj, &tag).len();
            json!({ "strip": tag, "terminals": terminals, "blocks": layout.iter().filter(|e| matches!(e.kind, ElementKind::Terminal(_) | ElementKind::Spare(_))).count(), "length_mm": layout.iter().map(|e| e.width).sum::<f32>() })
        })
        .collect();
    Ok(Value::Array(list))
}

fn get_strip(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let strip = a.req("strip").trim();
    existing(prj, strip)?;
    let an = Analysis::build(prj);
    let terminals: Vec<Value> = strips::rows(prj, strip, &an.netlist, &an.open)
        .into_iter()
        .map(|r| {
            json!({
                "terminal": r.label, "level": r.level, "jumper": r.jumper, "spare": r.terminal.is_none(), "external": r.external,
                "internal": r.internal, "potential": r.potential, "location": r.location, "function_text": r.function_text
            })
        })
        .collect();
    let rail: Vec<Value> = strips::strip_layout(prj, strip)
        .into_iter()
        .map(|e| {
            let kind = match &e.kind {
                ElementKind::Terminal(_) => "terminal",
                ElementKind::Spare(_) => "spare",
                ElementKind::EndBracket => "end_bracket",
                ElementKind::EndPlate => "end_plate",
            };
            json!({ "kind": kind, "terminals": e.designations(), "part": e.part, "width_mm": e.width })
        })
        .collect();
    let jumpers: Vec<Value> = strips::strip_jumpers(prj, strip)
        .into_iter()
        .map(|j| {
            json!({
                "number": j.number, "kind": kind_code(j.kind), "level": j.level, "part": j.part, "length": j.length_text(), "span": j.span,
                "broken": j.broken, "problem": j.problem_text(strip)
            })
        })
        .collect();
    let problems = strips::level_problems(prj, strip);
    Ok(json!({ "strip": strip, "terminals": terminals, "rail": rail, "jumpers": jumpers, "level_problems": problems }))
}

fn set_terminal(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let strip = a.req("strip").trim().to_string();
    let terminal = a.req("terminal").trim().to_string();
    if !strips::terminals(prj, &strip).iter().any(|t| t.label == terminal) {
        return Err(tr_fmt!("No terminal {} on strip {}", terminal, strip));
    }
    let (level, jumper) = (int_in(a, "level", 1, 9)?, int_in(a, "jumper", 0, u32::MAX.into())?);
    let data = prj.strips.entry(strip.clone()).or_default();
    if let Some(l) = level {
        if l <= 1 {
            data.levels.remove(&terminal);
        } else {
            data.levels.insert(terminal.clone(), l as u8);
        }
    }
    if let Some(j) = jumper {
        if j <= 0 {
            data.jumpers.remove(&terminal);
        } else {
            data.jumpers.insert(terminal.clone(), j as u32);
        }
    }
    get_strip(prj, a)
}

fn set_jumper_kind(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let strip = a.req("strip").trim().to_string();
    existing(prj, &strip)?;
    let number = int_in(a, "jumper", 0, u32::MAX.into())?.unwrap_or_default() as u32;
    let kind = match a.req("kind").trim().to_ascii_lowercase().as_str() {
        "plug_in" | "plugin" => JumperKind::PlugIn,
        "saddle" => JumperKind::Saddle,
        "wire" => JumperKind::Wire,
        other => return Err(tr_fmt!("Unknown jumper kind: {}", other)),
    };
    let data = prj.strips.entry(strip).or_default();
    if kind == JumperKind::PlugIn {
        data.jumper_kinds.remove(&number);
    } else {
        data.jumper_kinds.insert(number, kind);
    }
    get_strip(prj, a)
}

fn set_strip(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let strip = a.req("strip").trim().to_string();
    existing(prj, &strip)?;
    let labels: Vec<String> = strips::terminals(prj, &strip).into_iter().map(|t| t.label).collect();
    let data = prj.strips.entry(strip).or_default();
    if let Some(n) = int_in(a, "spares", 0, 1000)? {
        data.spares = n as u32;
    }
    if let Some(auto) = a.value("automatic_accessories").and_then(Value::as_bool) {
        data.manual_accessories = !auto;
    }
    if let Some(order) = a.str("order") {
        let order: Vec<String> = order.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        if let Some(unknown) = order.iter().find(|o| !labels.contains(o)) {
            return Err(tr_fmt!("No terminal {} on strip {}", unknown, a.req("strip")));
        }
        data.order = order;
    }
    get_strip(prj, a)
}

fn renumber_strip(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let strip = a.req("strip").trim().to_string();
    existing(prj, &strip)?;
    let n = strips::renumber(prj, &strip);
    let mut out = get_strip(prj, a)?;
    out["renumbered"] = json!(n);
    Ok(out)
}

#[cfg(test)]
mod tests;
