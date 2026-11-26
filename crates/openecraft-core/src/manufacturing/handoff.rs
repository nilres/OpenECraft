//! Hand-off to the manufacturing machines: the programs of an enclosure (each face on
//! its machine, the copper bars on the busbar machine) as a job with a manifest. A
//! released job keeps the checksums of its programs, so that changes of the layout
//! after the hand-off are found.

use super::{bars, checksum, machines, post, strategy};
use crate::model::Project;
use openecraft_i18n::tr_fmt;
use serde_json::{json, Value};
pub use crate::model::manufacturing::{Job, Program, ProgramRef};

/// The programs of an enclosure: one per sheet with machining (a face, each leaf of a
/// door of several leaves), and one for its copper bars and busbar systems; none for a
/// room (rooms are not machined).
pub fn programs(prj: &Project, enc: usize) -> Result<Vec<Program>, String> {
    let mf = &prj.layout.manufacturing;
    let e = &prj.layout.enclosures[enc];
    let mut v = Vec::new();
    if e.is_room() {
        return Ok(v);
    }
    for sheet in crate::production::sheets(e) {
        if crate::production::sheet_holes(e, &prj.parts, sheet).is_empty() && crate::production::sheet_openings(e, &prj.parts, sheet).is_empty() {
            continue;
        }
        let m = machines::get(prj, mf.machine_of(sheet.face))?;
        let plan = strategy::plan(prj, enc, sheet, &m, &mf.strategy);
        let content = post::program(&plan, &m);
        v.push(Program { name: post::file_name(&plan, &m), machine: m.id.clone(), face: Some(sheet.face), leaf: sheet.leaf, sheet: plan.sheet.clone(), operations: plan.ops.len(), time: plan.time, factor: m.time_factor, problems: plan.problems, content });
    }
    let has_bars = bars::pieces(prj).iter().any(|p| p.enclosure == e.name);
    if has_bars {
        let m = machines::get(prj, &mf.bar_machine)?;
        let (content, problems) = bars::program(prj, &m, Some(&e.name));
        let operations = content.lines().count().saturating_sub(1);
        let time = operations as f32 * m.hit_time * m.time_factor.max(0.01);
        let name = format!("{}_bars.{}", super::file_stem(&e.name), m.dialect.extension());
        v.push(Program { name, machine: m.id.clone(), face: None, leaf: None, sheet: "bars".into(), operations, time, factor: m.time_factor, problems, content });
    }
    Ok(v)
}

fn reference(p: &Program) -> ProgramRef {
    ProgramRef { name: p.name.clone(), machine: p.machine.clone(), face: p.sheet.clone(), operations: p.operations, time: p.time, factor: p.factor, checksum: checksum(&p.content) }
}

/// Releases the programs of an enclosure as a new job.
pub fn release(prj: &mut Project, enc: usize, date: &str) -> Result<(Job, Vec<Program>), String> {
    let list = programs(prj, enc)?;
    let e = &prj.layout.enclosures[enc];
    if list.is_empty() {
        return Err(tr_fmt!("{} has nothing to manufacture", e.name));
    }
    let n = prj.layout.manufacturing.jobs.iter().filter(|j| j.enclosure == e.name).count() + 1;
    let job = Job { id: format!("{}-{n}", super::file_stem(&e.name)), enclosure: e.name.clone(), date: date.into(), revision: prj.info.revision.clone(), programs: list.iter().map(reference).collect() };
    prj.layout.manufacturing.jobs.push(job.clone());
    Ok((job, list))
}

/// The package handed to the machines: manifest and programs.
pub fn package(prj: &Project, job: Option<&Job>, list: &[Program]) -> Value {
    let mut v = json!({
        "format": "openecraft-manufacturing/1",
        "project": prj.info.name,
        "revision": prj.info.revision,
        "programs": list.iter().map(|p| {
            let r = reference(p);
            json!({ "file": p.name, "machine": p.machine, "face": r.face, "operations": p.operations, "time_s": p.time.round(), "checksum": r.checksum, "problems": p.problems, "content": p.content })
        }).collect::<Vec<_>>(),
    });
    // A package handed off as a job names it; the plain export has no job.
    if let Some(j) = job {
        v["job"] = j.id.as_str().into();
        v["date"] = j.date.as_str().into();
    }
    v
}

/// Package of all enclosures (not released as a job); rooms are not machined.
pub fn package_all(prj: &Project) -> Result<Value, String> {
    let mut list = Vec::new();
    for enc in (0..prj.layout.enclosures.len()).filter(|&i| !prj.layout.enclosures[i].is_room()) {
        list.extend(programs(prj, enc)?);
    }
    Ok(package(prj, None, &list))
}

/// Latest job of each enclosure whose programs differ from the current layout: (job,
/// program file).
pub fn changed(prj: &Project) -> Vec<(String, String)> {
    let jobs = &prj.layout.manufacturing.jobs;
    let mut v = Vec::new();
    for (i, job) in jobs.iter().enumerate() {
        if jobs[i + 1..].iter().any(|j| j.enclosure == job.enclosure) {
            continue;
        }
        let Some(enc) = super::enclosure_index(prj, &job.enclosure) else { continue };
        let now: Vec<ProgramRef> = programs(prj, enc).map(|l| l.iter().map(reference).collect()).unwrap_or_default();
        for r in &job.programs {
            if !now.iter().any(|n| n.name == r.name && n.checksum == r.checksum) {
                v.push((job.id.clone(), r.name.clone()));
            }
        }
        for n in now.iter().filter(|n| !job.programs.iter().any(|r| r.name == n.name)) {
            v.push((job.id.clone(), n.name.clone()));
        }
    }
    v
}

#[cfg(test)]
mod tests;
