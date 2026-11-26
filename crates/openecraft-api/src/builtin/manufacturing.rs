//! Panel manufacturing ([`openecraft_core::manufacturing`]): CNC machines and their
//! programs, drilling / milling / punching strategies, copper bars, the enclosure
//! configurator, the hand-off of jobs to the machines and the production feedback.

use crate::builtin::common::{int_in, num_in, NO_MAX};
use crate::params::{optional, param, Args, Kind};
use crate::{Registry, Table};
use crate::{Info, Plugin, Text};
use openecraft_core::manufacturing::{self as mf, bars, configurator, feedback, handoff, machines, post, strategy, BarRules, CopperBar, Machine, Method, Strategy};
use openecraft_core::model::Project;
use openecraft_core::production;
use openecraft_core::panel::{self, Face};
use openecraft_core::issue::{Issue, Severity};
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde_json::{json, Value};

const ENCLOSURE: &str = tr_const!("Enclosure (location, e.g. +CAB1)");
const FACE: &str = tr_const!("Face: plate, door, left, right, rear, roof or bottom");
const SHEET: &str = tr_const!("Face: plate, door, left, right, rear, roof or bottom; door_left and door_right for the leaves of a double door");
const MACHINE: &str = tr_const!("Machine id (default: the machine of the face)");
const DATE: &str = tr_const!("Date (YYYY-MM-DD)");

pub struct PanelManufacturing;

impl Plugin for PanelManufacturing {
    fn info(&self) -> Info {
        Info {
            id: "panel_manufacturing".into(),
            name: Text::Borrowed(tr_const!("Panel manufacturing")),
            version: env!("CARGO_PKG_VERSION").into(),
            description: Text::Borrowed(tr_const!("CNC programs for drilling, milling and punching machines, copper bar bending, enclosure configurator, hand-off to production and production feedback")),
        }
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        reg.check("panel_manufacturing", tr_const!("Panel manufacturing"), |prj, _| Ok(issues(prj)))
            .command("cnc_machines", tr_const!("Lists the CNC machines (built-in and of the project) with their tools and the machine of every face"), vec![], |prj, _| Ok(machines_json(prj)))
            .command(
                "set_cnc_machine",
                tr_const!("Adds or changes a machine of the project (a built-in machine of the same id is replaced) or removes it"),
                vec![
                    param("id", Kind::Str, tr_const!("Machine id (snake_case)")),
                    optional("name", Kind::Str, tr_const!("Name of the machine")),
                    optional("kind", Kind::Str, tr_const!("Kind: machining, punching or busbar")),
                    optional("dialect", Kind::Str, tr_const!("Program format: iso, heidenhain, punch or csv")),
                    optional("tools", Kind::Str, tr_const!("Tools, e.g. D4.2 T1; M5 T21; F10 T30; R22.5 T101; Q92x92 T120")),
                    optional("area_x", Kind::Num, tr_const!("Working area in x, mm")),
                    optional("area_y", Kind::Num, tr_const!("Working area in y, mm")),
                    optional("max_thickness", Kind::Num, tr_const!("Thickest sheet or bar, mm")),
                    optional("rapid", Kind::Num, tr_const!("Rapid traverse, mm/min")),
                    optional("feed", Kind::Num, tr_const!("Milling feed, mm/min")),
                    optional("hit_time", Kind::Num, tr_const!("Time per drilling, punching or bending operation, s")),
                    optional("tool_change", Kind::Num, tr_const!("Time per tool change, s")),
                    optional("time_factor", Kind::Num, tr_const!("Correction of the time estimate")),
                    optional("remove", Kind::Bool, tr_const!("Removes the machine from the project")),
                ],
                set_machine,
            )
            .command(
                "machining_strategy",
                tr_const!("Shows or changes the machining strategy and which machine makes which face"),
                vec![
                    optional("sheet_machine", Kind::Str, tr_const!("Machine of the sheet metal faces")),
                    optional("bar_machine", Kind::Str, tr_const!("Machine of the copper bars")),
                    optional("face", Kind::Str, FACE),
                    optional("machine", Kind::Str, tr_const!("Machine of that face (empty: the sheet machine)")),
                    optional("cutouts", Kind::Str, tr_const!("Openings: auto (punch of the size, else contour) or contour")),
                    optional("tap", Kind::Bool, tr_const!("Cut threads on the machine")),
                    optional("drill_max", Kind::Num, tr_const!("Larger round holes are milled, mm")),
                    optional("tolerance", Kind::Num, tr_const!("Largest difference between hole and tool, mm")),
                    optional("nibble_overlap", Kind::Num, tr_const!("Overlap of nibbling hits (0 … 0.9)")),
                    optional("optimize", Kind::Bool, tr_const!("Shortest path between the hits of a tool")),
                    optional("reset", Kind::Bool, tr_const!("Back to the default strategy first")),
                ],
                set_strategy,
            )
            .command(
                "machining_plan",
                tr_const!("Plans the machining of a face: operations (drilling, tapping, milling, punching, nibbling), tools, path, time and what the machine cannot make"),
                vec![param("enclosure", Kind::Str, ENCLOSURE), param("face", Kind::Str, SHEET), optional("machine", Kind::Str, MACHINE)],
                |prj, a| plan_json(prj, a, false),
            )
            .command(
                "cnc_program",
                tr_const!("Generates the CNC program of a face for its machine (ISO G-code, Heidenhain, punching code or CSV)"),
                vec![param("enclosure", Kind::Str, ENCLOSURE), param("face", Kind::Str, SHEET), optional("machine", Kind::Str, MACHINE)],
                |prj, a| plan_json(prj, a, true),
            )
            .command(
                "add_copper_bar",
                tr_const!("Adds a bent and drilled copper bar (or replaces the one of the same name in the enclosure)"),
                vec![
                    param("enclosure", Kind::Str, ENCLOSURE),
                    param("name", Kind::Str, tr_const!("Name of the bar, e.g. L1 -Q1")),
                    param("segments", Kind::Str, tr_const!("Outer lengths of the straight segments, mm, e.g. 200, 150, 300")),
                    optional("bends", Kind::Str, tr_const!("Bends between the segments: angle, e for edgewise, r<radius>, e.g. 90; -90 e; 45 r10")),
                    optional("width", Kind::Num, tr_const!("Bar width, mm (default 20)")),
                    optional("thickness", Kind::Num, tr_const!("Bar thickness, mm (default 5)")),
                    optional("count", Kind::Int, tr_const!("Number of equal bars")),
                    optional("holes", Kind::Str, tr_const!("Holes: segment:distance:diameter, e.g. 0:20:9; 2:30:11")),
                    optional("end_holes", Kind::Num, tr_const!("Diameter of a hole at both ends (0: none)")),
                ],
                add_bar,
            )
            .command("remove_copper_bar", tr_const!("Removes a copper bar"), vec![param("id", Kind::Int, tr_const!("Bar id"))], |prj, a| {
                let id = a.int("id").unwrap_or(0) as u64;
                let bars = &mut prj.layout.manufacturing.bars;
                let i = bars.iter().position(|b| b.id == id).ok_or_else(|| tr_fmt!("No copper bar {}", id))?;
                bars.remove(i);
                Ok(json!({ "removed": id }))
            })
            .command("copper_bars", tr_const!("Lists the copper bars developed flat: length, bends in their order, holes, problems, and the cutting from stock lengths"), vec![], |prj, _| Ok(bars_json(prj)))
            .command(
                "copper_bar_rules",
                tr_const!("Shows or changes the rules of copper bars; lengths in mm"),
                vec![
                    optional("k_factor", Kind::Num, tr_const!("Position of the neutral fibre (0 … 1, 0: DIN 6935)")),
                    optional("radius", Kind::Num, tr_const!("Inner radius as a multiple of the thickness")),
                    optional("min_radius", Kind::Num, tr_const!("Smallest inner radius as a multiple of the thickness")),
                    optional("min_flange", Kind::Num, tr_const!("Shortest straight length next to a bend")),
                    optional("springback", Kind::Num, tr_const!("Springback in percent of the angle")),
                    optional("stock_length", Kind::Num, tr_const!("Stock length of the bars")),
                    optional("kerf", Kind::Num, tr_const!("Saw cut")),
                    optional("end_distance", Kind::Num, tr_const!("End holes from the bar end as a multiple of the width")),
                    optional("reset", Kind::Bool, tr_const!("Back to the default rules first")),
                ],
                bar_rules,
            )
            .command(
                "copper_bar_program",
                tr_const!("Generates the program of the busbar machine: cuts, punched holes and bends"),
                vec![optional("enclosure", Kind::Str, ENCLOSURE), optional("machine", Kind::Str, tr_const!("Machine id (default: the busbar machine)"))],
                bar_program,
            )
            .command("export_enclosure_configuration", tr_const!("Configuration of an enclosure for the configurator of the enclosure maker: system, size, components, accessories and machining"), vec![param("enclosure", Kind::Str, ENCLOSURE)], |prj, a| {
                Ok(configurator::export(prj, enclosure(prj, a)?))
            })
            .command(
                "import_enclosure_configuration",
                tr_const!("Takes over an enclosure configured in the configurator of the enclosure maker (JSON): system, size, components, accessories and cutouts"),
                vec![param("text", Kind::Str, tr_const!("The configuration (JSON)")), optional("dry_run", Kind::Bool, tr_const!("Only shows what would change"))],
                import_configuration,
            )
            .command(
                "configure_enclosure",
                tr_const!("Finds the smallest standard size of an enclosure system that holds the layout and applies it"),
                vec![param("enclosure", Kind::Str, ENCLOSURE), optional("system", Kind::Str, tr_const!("Enclosure system (default: the enclosure's)")), optional("dry_run", Kind::Bool, tr_const!("Only shows what would change"))],
                configure,
            )
            .command(
                "manufacturing_handoff",
                tr_const!("Hands the programs of an enclosure off to production as a job: manifest and programs of all its faces and copper bars"),
                vec![param("enclosure", Kind::Str, ENCLOSURE), optional("date", Kind::Str, DATE), optional("dry_run", Kind::Bool, tr_const!("Only shows the package, no job is recorded"))],
                handoff_cmd,
            )
            .command(
                "production_feedback",
                tr_const!("Takes over feedback of the production (CSV or JSON): job, program, operation, item, status, deviations, time, message"),
                vec![param("text", Kind::Str, tr_const!("The feedback (CSV with header or JSON)"))],
                |prj, a| {
                    let n = feedback::import(prj, a.req("text"))?;
                    Ok(json!({ "imported": n, "jobs": status_json(prj), "problems": feedback::problems(prj) }))
                },
            )
            .command("production_status", tr_const!("State of the jobs handed off to production: progress, failures, rework, programs changed since the hand-off"), vec![], |prj, _| {
                Ok(json!({ "jobs": status_json(prj), "problems": feedback::problems(prj), "changed": changed_json(prj) }))
            })
            .command(
                "calibrate_machines",
                tr_const!("Calibrates the time estimate of the machines from the actual times of the production feedback"),
                vec![optional("apply", Kind::Bool, tr_const!("Stores the calibrated factors"))],
                |prj, a| {
                    let v = feedback::calibrate(prj, a.flag("apply"));
                    Ok(Value::Array(v.into_iter().map(|(id, old, new, n)| json!({ "machine": id, "factor": old, "calibrated": (new * 1000.0).round() / 1000.0, "programs": n })).collect()))
                },
            )
            .report("cnc_operations", tr_const!("CNC operations"), |prj, _| operations_report(prj))
            .report("copper_bar_cutting", tr_const!("Copper bar cutting list"), |prj, _| Ok(cutting_report(prj)))
            .report("production_status", tr_const!("Production status"), |prj, _| Ok(status_report(prj)))
            .exporter("manufacturing_package", tr_const!("Manufacturing package (CNC programs, JSON)"), "json", |prj| {
                serde_json::to_string_pretty(&handoff::package_all(prj)?).map_err(|e| e.to_string())
            })
            .exporter("copper_bar_program", tr_const!("Busbar machine program (CSV)"), "csv", |prj| {
                let m = machines::get(prj, &prj.layout.manufacturing.bar_machine)?;
                Ok(bars::program(prj, &m, None).0)
            });
        Ok(())
    }
}

fn enclosure(prj: &Project, a: &Args) -> Result<usize, String> {
    let name = a.req("enclosure");
    mf::enclosure_index(prj, name).ok_or_else(|| tr_fmt!("No enclosure named {}", name))
}

fn r1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

/// Problems for the project check: work the machines cannot do, copper bars, programs
/// changed after their hand-off, and the production feedback.
fn issues(prj: &Project) -> Vec<Issue> {
    let warn = |message: String| Issue::plain(Severity::Warning, message, None);
    let mut v: Vec<Issue> = Vec::new();
    let m = &prj.layout.manufacturing;
    for enc in 0..prj.layout.enclosures.len() {
        match handoff::programs(prj, enc) {
            Ok(list) => v.extend(list.into_iter().flat_map(|p| p.problems).map(|message| Issue::plain(Severity::Info, message, None))),
            Err(e) => v.push(warn(e)),
        }
    }
    let bar_problems: Vec<String> = if m.bars.is_empty() { Vec::new() } else { bars::problems(prj) };
    v.extend(bar_problems.into_iter().map(warn));
    v.extend(handoff::changed(prj).into_iter().map(|(job, file)| warn(tr_fmt!("{} of job {} changed since the hand-off", file, job))));
    v.extend(feedback::problems(prj).into_iter().map(warn));
    v
}

fn machine_json(m: &Machine) -> Value {
    json!({
        "id": m.id,
        "name": m.name,
        "kind": m.kind.key(),
        "dialect": m.dialect.key(),
        "area": [m.area.0, m.area.1],
        "max_thickness": m.max_thickness,
        "tools": machines::tools_text(&m.tools),
        "rapid": m.rapid,
        "feed": m.feed,
        "hit_time": m.hit_time,
        "tool_change": m.tool_change,
        "time_factor": m.time_factor,
    })
}

fn machines_json(prj: &Project) -> Value {
    let m = &prj.layout.manufacturing;
    json!({
        "machines": machines::all(prj).iter().map(machine_json).collect::<Vec<_>>(),
        "sheet_machine": m.sheet_machine,
        "bar_machine": m.bar_machine,
        "faces": Face::ALL.iter().map(|f| (f.key().to_string(), Value::from(m.machine_of(*f)))).collect::<serde_json::Map<_, _>>(),
    })
}

fn set_machine(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let id = a.req("id").trim().to_string();
    if !crate::is_snake_case(&id) {
        return Err(tr_fmt!("Machine id {} is not snake_case", id));
    }
    if a.flag("remove") {
        let m = &prj.layout.manufacturing;
        if m.sheet_machine == id || m.bar_machine == id || m.face_machines.values().any(|x| *x == id) {
            return Err(tr_fmt!("Machine {} is in use", id));
        }
        let own = &mut prj.layout.manufacturing.machines;
        let i = own.iter().position(|m| m.id == id).ok_or_else(|| tr_fmt!("No machine {}", id))?;
        own.remove(i);
        return Ok(machines_json(prj));
    }
    let mut m = machines::find(prj, &id).unwrap_or(Machine { id: id.clone(), name: id.clone(), ..Default::default() });
    if let Some(v) = a.str("name") {
        m.name = v.to_string();
    }
    if let Some(v) = a.str("kind") {
        m.kind = machines::MachineKind::from_key(v).ok_or_else(|| tr_fmt!("Unknown machine kind {}", v))?;
    }
    if let Some(v) = a.str("dialect") {
        m.dialect = machines::Dialect::from_key(v).ok_or_else(|| tr_fmt!("Unknown program format {}", v))?;
    }
    if let Some(v) = a.str("tools") {
        m.tools = machines::parse_tools(v)?;
    }
    for (name, field) in [
        ("area_x", &mut m.area.0),
        ("area_y", &mut m.area.1),
        ("max_thickness", &mut m.max_thickness),
        ("rapid", &mut m.rapid),
        ("feed", &mut m.feed),
        ("hit_time", &mut m.hit_time),
        ("tool_change", &mut m.tool_change),
        ("time_factor", &mut m.time_factor),
    ] {
        if let Some(v) = num_in(a, name, 0.0, NO_MAX)? {
            *field = v;
        }
    }
    let own = &mut prj.layout.manufacturing.machines;
    match own.iter_mut().find(|x| x.id == id) {
        Some(x) => *x = m.clone(),
        None => own.push(m.clone()),
    }
    Ok(machine_json(&m))
}

fn strategy_json(prj: &Project) -> Value {
    let m = &prj.layout.manufacturing;
    let s = &m.strategy;
    json!({
        "sheet_machine": m.sheet_machine,
        "bar_machine": m.bar_machine,
        "face_machines": m.face_machines,
        "cutouts": s.cutouts.key(),
        "tap": s.tap,
        "drill_max": s.drill_max,
        "tolerance": s.tolerance,
        "nibble_overlap": s.nibble_overlap,
        "optimize": s.optimize,
    })
}

fn set_strategy(prj: &mut Project, a: &Args) -> Result<Value, String> {
    for key in ["sheet_machine", "bar_machine"] {
        if let Some(id) = a.str(key) {
            machines::get(prj, id)?;
        }
    }
    let face = a.str("face").map(|k| Face::from_key(k).ok_or_else(|| tr_fmt!("Unknown face {}", k))).transpose()?;
    if let Some(id) = a.str("machine").filter(|s| !s.is_empty()) {
        machines::get(prj, id)?;
    }
    let m = &mut prj.layout.manufacturing;
    if a.flag("reset") {
        m.strategy = Strategy::default();
    }
    if let Some(id) = a.str("sheet_machine") {
        m.sheet_machine = id.into();
    }
    if let Some(id) = a.str("bar_machine") {
        m.bar_machine = id.into();
    }
    if let Some(f) = face {
        match a.str("machine").filter(|s| !s.is_empty()) {
            Some(id) => {
                m.face_machines.insert(f.key().into(), id.into());
            }
            None => {
                m.face_machines.remove(f.key());
            }
        }
    }
    let s = &mut m.strategy;
    if let Some(v) = a.str("cutouts") {
        s.cutouts = Method::from_key(v).ok_or_else(|| tr_fmt!("Unknown method {}", v))?;
    }
    if a.value("tap").is_some() {
        s.tap = a.flag("tap");
    }
    if a.value("optimize").is_some() {
        s.optimize = a.flag("optimize");
    }
    if let Some(v) = num_in(a, "drill_max", 0.0, NO_MAX)? {
        s.drill_max = v;
    }
    if let Some(v) = num_in(a, "tolerance", 0.0, 5.0)? {
        s.tolerance = v;
    }
    if let Some(v) = num_in(a, "nibble_overlap", 0.0, 0.9)? {
        s.nibble_overlap = v;
    }
    Ok(strategy_json(prj))
}

fn plan_json(prj: &mut Project, a: &Args, with_program: bool) -> Result<Value, String> {
    let enc = enclosure(prj, a)?;
    let key = a.req("face");
    let sheet = production::sheet_from_key(&prj.layout.enclosures[enc], key).ok_or_else(|| tr_fmt!("Unknown face {}", key))?;
    let id = a.str("machine").unwrap_or(prj.layout.manufacturing.machine_of(sheet.face)).to_string();
    let m = machines::get(prj, &id)?;
    let plan = strategy::plan(prj, enc, sheet, &m, &prj.layout.manufacturing.strategy);
    let mut counts = serde_json::Map::new();
    for o in &plan.ops {
        let n = counts.get(o.kind.key()).and_then(Value::as_u64).unwrap_or(0) + 1;
        counts.insert(o.kind.key().into(), n.into());
    }
    let mut tools: Vec<String> = Vec::new();
    for o in &plan.ops {
        if let Some(t) = o.tool.and_then(|n| m.tool_by_number(n)) {
            let name = format!("T{} {}", t.number, t.name());
            if !tools.contains(&name) {
                tools.push(name);
            }
        }
    }
    let mut v = json!({
        "enclosure": plan.enclosure,
        "face": plan.sheet,
        "machine": m.id,
        "sheet": [plan.width, plan.height, plan.thickness],
        "operations": plan.ops.len(),
        "by_kind": counts,
        "tools": tools,
        "tool_changes": plan.tool_changes,
        "travel_mm": r1(plan.travel),
        "milled_mm": r1(plan.milled),
        "time_s": plan.time.round(),
        "time": strategy::duration(plan.time),
        "problems": plan.problems,
    });
    if with_program {
        v["file"] = post::file_name(&plan, &m).into();
        v["content"] = post::program(&plan, &m).into();
    } else {
        v["steps"] = plan
            .ops
            .iter()
            .map(|o| json!({ "seq": o.seq, "item": o.item, "operation": o.kind.key(), "tool": o.tool, "x": r1(o.pos.x), "y": r1(plan.height - o.pos.y) }))
            .collect::<Vec<_>>()
            .into();
    }
    Ok(v)
}

fn add_bar(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let enc = enclosure(prj, a)?;
    let mut b = CopperBar {
        enclosure: prj.layout.enclosures[enc].name.clone(),
        name: a.req("name").trim().to_string(),
        width: num_in(a, "width", 1.0, NO_MAX)?.unwrap_or(20.0),
        thickness: num_in(a, "thickness", 0.5, NO_MAX)?.unwrap_or(5.0),
        count: int_in(a, "count", 1, u32::MAX.into())?.unwrap_or(1) as u32,
        segments: bars::parse_segments(a.req("segments"))?,
        bends: bars::parse_bends(a.str("bends").unwrap_or(""))?,
        holes: bars::parse_holes(a.str("holes").unwrap_or(""))?,
        end_holes: num_in(a, "end_holes", 0.0, NO_MAX)?.unwrap_or(0.0),
        ..Default::default()
    };
    if b.segments.is_empty() {
        return Err(tr("A copper bar needs at least one segment").to_string());
    }
    if b.bends.len() + 1 != b.segments.len() {
        return Err(tr_fmt!("{} segments need {} bends", b.segments.len(), b.segments.len() - 1));
    }
    let existing = prj.layout.manufacturing.bars.iter().position(|x| x.enclosure == b.enclosure && x.name == b.name);
    b.id = match existing {
        Some(i) => prj.layout.manufacturing.bars[i].id,
        None => prj.new_id(),
    };
    let out = bar_json(&b, &prj.layout.manufacturing.bar_rules);
    match existing {
        Some(i) => prj.layout.manufacturing.bars[i] = b,
        None => prj.layout.manufacturing.bars.push(b),
    }
    Ok(out)
}

fn bar_json(b: &CopperBar, rules: &BarRules) -> Value {
    let f = b.flat(rules);
    json!({
        "id": b.id,
        "enclosure": b.enclosure,
        "name": b.name,
        "profile": format!("{}x{}", mf::num(b.width), mf::num(b.thickness)),
        "count": b.count,
        "length": r1(f.length),
        "bends": f.sequence.iter().map(|&i| {
            let x = &f.bends[i];
            json!({ "bend": x.index + 1, "at": r1(x.at), "angle": x.angle, "set_angle": r1(x.set_angle), "radius": r1(x.radius), "edgewise": x.edgewise })
        }).collect::<Vec<_>>(),
        "holes": f.holes.iter().map(|(x, d)| json!({ "at": r1(*x), "diameter": d })).collect::<Vec<_>>(),
        "problems": f.problems,
    })
}

fn bars_json(prj: &Project) -> Value {
    let m = &prj.layout.manufacturing;
    let (stock, too_long) = bars::cutting(&bars::pieces(prj), &m.bar_rules);
    json!({
        "bars": m.bars.iter().map(|b| bar_json(b, &m.bar_rules)).collect::<Vec<_>>(),
        "stock_bars": stock.iter().map(|s| json!({
            "profile": format!("{}x{}", mf::num(s.width), mf::num(s.thickness)),
            "pieces": s.pieces.iter().map(|p| json!({ "bar": p.label, "length": r1(p.length) })).collect::<Vec<_>>(),
            "rest": r1(s.rest),
        })).collect::<Vec<_>>(),
        "too_long": too_long.iter().map(|p| p.label.clone()).collect::<Vec<_>>(),
    })
}

fn bar_rules(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let r = &mut prj.layout.manufacturing.bar_rules;
    if a.flag("reset") {
        *r = BarRules::default();
    }
    for (name, field) in [
        ("k_factor", &mut r.k_factor),
        ("radius", &mut r.radius),
        ("min_radius", &mut r.min_radius),
        ("min_flange", &mut r.min_flange),
        ("springback", &mut r.springback),
        ("stock_length", &mut r.stock_length),
        ("kerf", &mut r.kerf),
        ("end_distance", &mut r.end_distance),
    ] {
        if let Some(v) = num_in(a, name, 0.0, NO_MAX)? {
            *field = v;
        }
    }
    r.k_factor = r.k_factor.min(1.0);
    Ok(json!({
        "k_factor": r.k_factor, "radius": r.radius, "min_radius": r.min_radius, "min_flange": r.min_flange,
        "springback": r.springback, "stock_length": r.stock_length, "kerf": r.kerf, "end_distance": r.end_distance,
    }))
}

fn bar_program(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let id = a.str("machine").unwrap_or(&prj.layout.manufacturing.bar_machine).to_string();
    let m = machines::get(prj, &id)?;
    let enc = a.str("enclosure").filter(|s| !s.is_empty());
    if let Some(name) = enc {
        mf::enclosure_index(prj, name).ok_or_else(|| tr_fmt!("No enclosure named {}", name))?;
    }
    let (content, problems) = bars::program(prj, &m, enc);
    let file = format!("{}_bars.{}", mf::file_stem(enc.unwrap_or(&prj.info.name)), m.dialect.extension());
    Ok(json!({ "machine": m.id, "file": file, "problems": problems, "content": content }))
}

fn import_configuration(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let dry = a.flag("dry_run");
    let mut work = prj.clone();
    let r = configurator::import(&mut work, a.req("text"))?;
    if !dry {
        *prj = work;
    }
    Ok(json!({ "enclosure": r.enclosure, "created": r.created, "accessories": r.accessories, "cutouts": r.cutouts, "skipped": r.skipped, "dry_run": dry }))
}

fn configure(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let enc = enclosure(prj, a)?;
    let e = &prj.layout.enclosures[enc];
    let system = a.str("system").filter(|s| !s.is_empty()).unwrap_or(&e.build.system).to_string();
    if system.is_empty() {
        return Err(tr_fmt!("{} has no enclosure system; give one", e.name));
    }
    if panel::SYSTEMS.iter().all(|s| s.id() != system) {
        return Err(tr_fmt!("Unknown enclosure system: {}", system));
    }
    let need = configurator::needed(e);
    let size = configurator::suggest(e, &system).ok_or_else(|| tr_fmt!("No standard size of {} holds the layout of {}", system, e.name))?;
    let before = [e.width, e.height, e.depth];
    if !a.flag("dry_run") {
        let e = &mut prj.layout.enclosures[enc];
        panel::apply_system(e, &system, Some(size));
        panel::apply_anchors(e);
    }
    let e = &prj.layout.enclosures[enc];
    Ok(json!({ "enclosure": e.name, "system": system, "needed": [r1(need.0), r1(need.1), r1(need.2)], "before": before, "size": [size.0, size.1, size.2], "part": e.part, "dry_run": a.flag("dry_run") }))
}

fn handoff_cmd(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let enc = enclosure(prj, a)?;
    if a.flag("dry_run") {
        let list = handoff::programs(prj, enc)?;
        return Ok(handoff::package(prj, None, &list));
    }
    let (job, list) = handoff::release(prj, enc, a.str("date").unwrap_or(""))?;
    Ok(handoff::package(prj, Some(&job), &list))
}

fn status_json(prj: &Project) -> Vec<Value> {
    feedback::status(prj)
        .into_iter()
        .map(|s| json!({ "job": s.job, "enclosure": s.enclosure, "state": s.title(), "programs": s.programs, "done": s.done, "failed": s.failed, "rework": s.rework, "progress": r1(s.progress()) }))
        .collect()
}

fn changed_json(prj: &Project) -> Vec<Value> {
    handoff::changed(prj).into_iter().map(|(job, file)| json!({ "job": job, "program": file })).collect()
}

fn operations_report(prj: &Project) -> Result<Table, String> {
    let mut t = Table::new([tr_const!("Enclosure"), tr_const!("Face"), tr_const!("Machine"), tr_const!("Operations"), tr_const!("Tools"), tr_const!("Time"), tr_const!("Problems")]);
    for enc in 0..prj.layout.enclosures.len() {
        for p in handoff::programs(prj, enc)? {
            let face = p.face.map(|face| panel::Sheet { face, leaf: p.leaf }.title(&prj.layout.enclosures[enc])).unwrap_or_else(|| tr("Copper bars").into());
            let tools = p.content.lines().filter(|l| l.starts_with('T') || l.contains("TOOL CALL")).count();
            t.push(vec![prj.layout.enclosures[enc].name.clone(), face.to_string(), p.machine, p.operations.to_string(), tools.to_string(), strategy::duration(p.time), p.problems.join("; ")], None);
        }
    }
    Ok(t)
}

fn cutting_report(prj: &Project) -> Table {
    let m = &prj.layout.manufacturing;
    let mut t = Table::new([tr_const!("Profile"), tr_const!("Stock bar"), tr_const!("Pieces"), tr_const!("Used (mm)"), tr_const!("Rest (mm)")]);
    let (stock, too_long) = bars::cutting(&bars::pieces(prj), &m.bar_rules);
    for (i, s) in stock.iter().enumerate() {
        let used: f32 = s.pieces.iter().map(|p| p.length).sum();
        let list = s.pieces.iter().map(|p| format!("{} ({})", p.label, mf::num(r1(p.length)))).collect::<Vec<_>>().join(", ");
        t.push(vec![format!("{}x{}", mf::num(s.width), mf::num(s.thickness)), (i + 1).to_string(), list, mf::num(r1(used)), mf::num(r1(s.rest))], None);
    }
    for p in too_long {
        t.push(vec![format!("{}x{}", mf::num(p.width), mf::num(p.thickness)), tr("too long").to_string(), p.label, mf::num(r1(p.length)), String::new()], None);
    }
    t
}

fn status_report(prj: &Project) -> Table {
    let mut t = Table::new([tr_const!("Job"), tr_const!("Enclosure"), tr_const!("State"), tr_const!("Programs"), tr_const!("Done"), tr_const!("Failed"), tr_const!("Rework"), tr_const!("Progress (%)")]);
    for s in feedback::status(prj) {
        t.push(vec![s.job.clone(), s.enclosure.clone(), s.title().to_string(), s.programs.to_string(), s.done.to_string(), s.failed.to_string(), s.rework.to_string(), mf::num(r1(s.progress()))], None);
    }
    t
}

#[cfg(test)]
mod tests;
