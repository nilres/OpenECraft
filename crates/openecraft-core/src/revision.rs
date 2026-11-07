//! Revision management: a revision stores a snapshot of all pages; changes made
//! since the last released revision are listed and marked on the pages. Released
//! pages are locked against changes.
//!
//! Release workflow: with approvers defined ([`Workflow`]) a new revision starts as
//! a draft, is submitted for review, approved or rejected by the approvers
//! (`review::Approval`, recorded with user and date) and released once approved.
//! Releasing stores a baseline of the whole project.

use openecraft_i18n::{tr, tr_const, tr_fmt};
use crate::geom::{p, P};
use crate::model::{natural_cmp, CableDef, ConnDef, Dimension, ImageItem, Junction, Page, Project, Shape, SymbolInst, TextItem, Wire};
use std::sync::Arc;
pub use crate::model::revision::{Revision, Snapshot, State, Workflow};

impl PartialEq for Snapshot {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}

impl Workflow {
    pub fn is_default(&self) -> bool {
        *self == Workflow::default()
    }

    pub fn active(&self) -> bool {
        self.approvers.iter().any(|a| !a.trim().is_empty())
    }

    pub fn needed(&self) -> usize {
        let n = self.approvers.iter().filter(|a| !a.trim().is_empty()).count();
        if self.required == 0 {
            n
        } else {
            self.required.min(n)
        }
    }

    pub fn is_approver(&self, user: &str) -> bool {
        let user = user.trim();
        !user.is_empty() && self.approvers.iter().any(|a| a.trim().eq_ignore_ascii_case(user))
    }
}

/// Status of a revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Draft,
    InReview,
    Approved,
    Rejected,
    Released,
}

impl Status {
    pub fn title(self) -> &'static str {
        match self {
            Status::Draft => tr("draft"),
            Status::InReview => tr("in review"),
            Status::Approved => tr("approved"),
            Status::Rejected => tr("rejected"),
            Status::Released => tr("released"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangeKind {
    Added,
    Removed,
    Modified,
}

impl ChangeKind {
    pub fn title(self) -> &'static str {
        match self {
            ChangeKind::Added => tr("added"),
            ChangeKind::Removed => tr("removed"),
            ChangeKind::Modified => tr("changed"),
        }
    }
}

/// One changed item of a page.
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub kind: ChangeKind,
    /// "Symbol", "Wire", "Text", "Graphic", "Dimension", "Image", "Connection
    /// definition point", "Cable definition" or "Page".
    pub what: &'static str,
    pub id: u64,
    pub label: String,
    /// Where to put the revision mark.
    pub pos: P,
    /// Changed properties of a changed item.
    pub details: Vec<Detail>,
}

/// A changed property: its name (English, translated where shown), old and new value.
#[derive(Clone, Debug, PartialEq)]
pub struct Detail {
    pub field: &'static str,
    pub old: String,
    pub new: String,
}

impl Change {
    /// The changed properties as text: `Device tag: -K1 › -K2; Position`.
    pub fn details_text(&self) -> String {
        self.details
            .iter()
            .map(|d| if d.old.is_empty() && d.new.is_empty() { tr(d.field).to_string() } else { format!("{}: {} › {}", tr(d.field), shown(&d.old), shown(&d.new)) })
            .collect::<Vec<_>>()
            .join("; ")
    }
}

fn shown(s: &str) -> &str {
    if s.is_empty() { "–" } else { s }
}

/// Details of the fields that differ: (name, old, new).
fn changed(fields: &[(&'static str, String, String)]) -> Vec<Detail> {
    fields.iter().filter(|(_, a, b)| a != b).map(|(f, a, b)| Detail { field: f, old: a.clone(), new: b.clone() }).collect()
}

fn xy(q: P) -> String {
    format!("{:.0} / {:.0}", q.x, q.y)
}

fn symbol_details(o: &SymbolInst, n: &SymbolInst) -> Vec<Detail> {
    let mut d = changed(&[
        (tr_const!("Symbol"), o.sym.clone(), n.sym.clone()),
        (tr_const!("Device tag"), o.tag.clone(), n.tag.clone()),
        (tr_const!("Designation"), o.label.clone(), n.label.clone()),
        (tr_const!("Function text"), o.function_text.clone(), n.function_text.clone()),
        (tr_const!("Connection points"), o.pins.join(" "), n.pins.join(" ")),
        (tr_const!("Position"), xy(o.pos), xy(n.pos)),
        (tr_const!("Rotation"), format!("{}°", o.rot as u32 * 90), format!("{}°", n.rot as u32 * 90)),
        (tr_const!("Mirrored"), o.mirror.to_string(), n.mirror.to_string()),
        (tr_const!("Function definition"), o.function_def.clone(), n.function_def.clone()),
    ]);
    let keys: std::collections::BTreeSet<&String> = o.values.keys().chain(n.values.keys()).collect();
    for k in keys {
        let (a, b) = (o.values.get(k).cloned().unwrap_or_default(), n.values.get(k).cloned().unwrap_or_default());
        if a != b {
            d.push(Detail { field: tr_const!("Property"), old: format!("{k} = {}", shown(&a)), new: format!("{k} = {}", shown(&b)) });
        }
    }
    if d.is_empty() {
        // Placement of the tag or the shown properties.
        d.push(Detail { field: tr_const!("Display"), old: String::new(), new: String::new() });
    }
    d
}

fn wire_details(o: &Wire, n: &Wire) -> Vec<Detail> {
    let d = changed(&[
        (tr_const!("Wire number"), o.number.clone(), n.number.clone()),
        (tr_const!("Colour"), o.color.clone(), n.color.clone()),
        (tr_const!("Cross-section"), o.cross_section.clone(), n.cross_section.clone()),
        (tr_const!("Cable"), o.cable.clone(), n.cable.clone()),
        (tr_const!("Core"), o.core.clone(), n.core.clone()),
        (tr_const!("Position"), format!("{} – {}", xy(o.a), xy(o.b)), format!("{} – {}", xy(n.a), xy(n.b))),
    ]);
    if d.is_empty() { vec![Detail { field: tr_const!("Other properties"), old: String::new(), new: String::new() }] } else { d }
}

fn text_details(o: &TextItem, n: &TextItem) -> Vec<Detail> {
    let d = changed(&[
        (tr_const!("Text"), o.text.clone(), n.text.clone()),
        (tr_const!("Position"), xy(o.pos), xy(n.pos)),
        (tr_const!("Size"), format!("{}", o.size), format!("{}", n.size)),
        (tr_const!("Angle"), format!("{}°", o.angle), format!("{}°", n.angle)),
    ]);
    if d.is_empty() { vec![Detail { field: tr_const!("Other properties"), old: String::new(), new: String::new() }] } else { d }
}

/// Changes of one page (by page id; `page` is the current index if it still exists).
#[derive(Clone, Debug, PartialEq)]
pub struct PageChanges {
    pub page_id: u64,
    pub page: Option<usize>,
    pub name: String,
    pub changes: Vec<Change>,
}

/// How an item is found, named and compared.
struct Kind<'a, T> {
    what: &'static str,
    id: &'a dyn Fn(&T) -> u64,
    describe: &'a dyn Fn(&T) -> (String, P),
    details: &'a dyn Fn(&T, &T) -> Vec<Detail>,
}

fn diff<T: PartialEq>(old: &[T], new: &[T], k: &Kind<T>, out: &mut Vec<Change>) {
    let (id, what) = (k.id, k.what);
    for n in new {
        match old.iter().find(|o| id(o) == id(n)) {
            None => {
                let (label, pos) = (k.describe)(n);
                out.push(Change { kind: ChangeKind::Added, what, id: id(n), label, pos, details: Vec::new() });
            }
            Some(o) if o != n => {
                let (label, pos) = (k.describe)(n);
                out.push(Change { kind: ChangeKind::Modified, what, id: id(n), label, pos, details: (k.details)(o, n) });
            }
            Some(_) => {}
        }
    }
    for o in old.iter().filter(|o| !new.iter().any(|n| id(n) == id(o))) {
        let (label, pos) = (k.describe)(o);
        out.push(Change { kind: ChangeKind::Removed, what, id: id(o), label, pos, details: Vec::new() });
    }
}

fn no_details<T>(_: &T, _: &T) -> Vec<Detail> {
    Vec::new()
}

fn mid(a: P, b: P) -> P {
    p((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}

/// Item level differences between two page lists (matched by page and item ids).
pub fn compare(old: &[Page], new: &[Page]) -> Vec<PageChanges> {
    let mut out = Vec::new();
    for (i, n) in new.iter().enumerate() {
        let mut changes = Vec::new();
        match old.iter().find(|o| o.id == n.id) {
            None => changes.push(Change { kind: ChangeKind::Added, what: "Page", id: n.id, label: n.description.clone(), pos: p(0.0, 0.0), details: Vec::new() }),
            Some(o) => {
                let mut details = changed(&[
                    (tr_const!("Page name"), o.name.clone(), n.name.clone()),
                    (tr_const!("Description"), o.description.clone(), n.description.clone()),
                    (tr_const!("Structure"), crate::structure::page_structure(o), crate::structure::page_structure(n)),
                    (tr_const!("Page type"), o.kind.label(), n.kind.label()),
                    (tr_const!("Title block"), o.title_block.clone(), n.title_block.clone()),
                    (tr_const!("Plot frame"), o.frame.clone(), n.frame.clone()),
                ]);
                // Page data without a value to show: named only.
                for (field, differs) in [
                    (tr_const!("Section"), o.section != n.section),
                    (tr_const!("Document"), o.document != n.document),
                    (tr_const!("Page properties"), o.values != n.values),
                    (tr_const!("Macro"), o.macro_boxes != n.macro_boxes),
                    (tr_const!("Report"), o.report != n.report),
                ] {
                    if differs {
                        details.push(Detail { field, old: String::new(), new: String::new() });
                    }
                }
                if !details.is_empty() {
                    changes.push(Change { kind: ChangeKind::Modified, what: "Page", id: n.id, label: tr("page properties").into(), pos: p(0.0, 0.0), details });
                }
                let sym = Kind { what: "Symbol", id: &|s: &SymbolInst| s.id, describe: &|s: &SymbolInst| (format!("{} {}", s.tag, s.label).trim().to_string(), s.pos), details: &symbol_details };
                diff(&o.symbols, &n.symbols, &sym, &mut changes);
                let wire = Kind { what: "Wire", id: &|w: &Wire| w.id, describe: &|w: &Wire| (w.number.clone(), mid(w.a, w.b)), details: &wire_details };
                diff(&o.wires, &n.wires, &wire, &mut changes);
                let text = Kind { what: "Text", id: &|t: &TextItem| t.id, describe: &|t: &TextItem| (t.text.lines().next().unwrap_or("").to_string(), t.pos), details: &text_details };
                diff(&o.texts, &n.texts, &text, &mut changes);
                let shape = Kind { what: "Graphic", id: &|s: &Shape| s.id, describe: &|s: &Shape| (String::new(), mid(s.a, s.b)), details: &no_details };
                diff(&o.shapes, &n.shapes, &shape, &mut changes);
                let dim = Kind { what: "Dimension", id: &|d: &Dimension| d.id, describe: &|d: &Dimension| (d.text.clone(), mid(d.a, d.b)), details: &no_details };
                diff(&o.dimensions, &n.dimensions, &dim, &mut changes);
                let image = Kind { what: "Image", id: &|i: &ImageItem| i.id, describe: &|i: &ImageItem| (i.file.name.clone(), mid(i.a, i.b)), details: &no_details };
                diff(&o.images, &n.images, &image, &mut changes);
                let conn = Kind { what: "Connection definition point", id: &|c: &ConnDef| c.id, describe: &|c: &ConnDef| (String::new(), c.pos), details: &no_details };
                diff(&o.conn_defs, &n.conn_defs, &conn, &mut changes);
                let cable = Kind { what: "Cable definition", id: &|c: &CableDef| c.id, describe: &|c: &CableDef| (c.cable.clone(), mid(c.a, c.b)), details: &no_details };
                diff(&o.cable_defs, &n.cable_defs, &cable, &mut changes);
                let tee = Kind { what: "T-junction", id: &|j: &Junction| j.id, describe: &|j: &Junction| (String::new(), j.pos), details: &no_details };
                diff(&o.junctions, &n.junctions, &tee, &mut changes);
            }
        }
        if !changes.is_empty() {
            out.push(PageChanges { page_id: n.id, page: Some(i), name: n.name.clone(), changes });
        }
    }
    for o in old.iter().filter(|o| !new.iter().any(|n| n.id == o.id)) {
        let removed = Change { kind: ChangeKind::Removed, what: "Page", id: o.id, label: o.description.clone(), pos: p(0.0, 0.0), details: Vec::new() };
        out.push(PageChanges { page_id: o.id, page: None, name: o.name.clone(), changes: vec![removed] });
    }
    out
}

/// Changes since the last released revision (empty without one).
pub fn pending(prj: &Project) -> Vec<PageChanges> {
    match last_released(prj) {
        Some(r) => compare(&r.pages.0, &prj.pages),
        None => Vec::new(),
    }
}

pub fn last_released(prj: &Project) -> Option<&Revision> {
    prj.revisions.iter().rev().find(|r| r.state == State::Released)
}

/// The revision being prepared (draft or in review), if any.
pub fn open_revision(prj: &Project) -> Option<usize> {
    prj.revisions.iter().rposition(|r| r.state != State::Released)
}

/// Name of the revision being prepared, or the suggested name of the next one:
/// A → B, Z → AA, 01 → 02.
pub fn next_name(prj: &Project) -> String {
    if let Some(i) = open_revision(prj) {
        return prj.revisions[i].name.clone();
    }
    let Some(last) = prj.revisions.last().map(|r| r.name.trim().to_string()) else {
        return "A".into();
    };
    if let Ok(n) = last.parse::<u32>() {
        return format!("{:0width$}", n + 1, width = last.len());
    }
    let mut chars: Vec<char> = last.chars().collect();
    match chars.last_mut() {
        Some(c) if c.is_ascii_uppercase() && *c != 'Z' => *c = (*c as u8 + 1) as char,
        Some(c) if *c == 'Z' => chars.push('A'),
        _ => chars.push('A'),
    }
    chars.into_iter().collect()
}

/// Creates a revision from the current pages. Without approval workflow it is
/// released at once (the project revision becomes its name, a baseline is stored;
/// with `lock` all pages are locked), otherwise it starts as a draft.
pub fn create(prj: &mut Project, name: &str, date: &str, author: &str, description: &str, lock: bool) {
    let pages = Snapshot(Arc::new(prj.pages.clone()));
    let state = if prj.workflow.active() { State::Draft } else { State::Released };
    prj.revisions.push(Revision { name: name.into(), date: date.into(), author: author.into(), description: description.into(), pages, state, round: 0 });
    if state == State::Released {
        finish_release(prj, prj.revisions.len() - 1, lock);
    }
}

fn finish_release(prj: &mut Project, i: usize, lock: bool) {
    if lock {
        for pg in &mut prj.pages {
            pg.locked = true;
        }
    }
    let r = &mut prj.revisions[i];
    r.state = State::Released;
    r.pages = Snapshot(Arc::new(prj.pages.clone()));
    let (name, date, author, description) = (r.name.clone(), r.date.clone(), r.author.clone(), r.description.clone());
    prj.info.revision = name.clone();
    crate::compare::add_baseline(prj, &tr_fmt!("Revision {}", name), &date, &author, &description);
}

/// Decisions of the current review round of revision `i`, the latest per user.
pub fn decisions(prj: &Project, i: usize) -> Vec<&crate::review::Approval> {
    let Some(r) = prj.revisions.get(i) else { return Vec::new() };
    let mut out: Vec<&crate::review::Approval> = Vec::new();
    for a in prj.review.approvals.iter().filter(|a| !a.deleted && a.revision == r.name && a.round == r.round) {
        match out.iter_mut().find(|o| o.user.eq_ignore_ascii_case(&a.user)) {
            Some(o) => *o = a,
            None => out.push(a),
        }
    }
    out
}

pub fn status(prj: &Project, i: usize) -> Status {
    let Some(r) = prj.revisions.get(i) else { return Status::Draft };
    match r.state {
        State::Released => Status::Released,
        State::Draft => Status::Draft,
        State::InReview => {
            let d = decisions(prj, i);
            if d.iter().any(|a| !a.approved) {
                Status::Rejected
            } else if d.iter().filter(|a| a.approved && prj.workflow.is_approver(&a.user)).count() >= prj.workflow.needed() {
                Status::Approved
            } else {
                Status::InReview
            }
        }
    }
}

/// Submits a draft (or a revision in review again, e.g. after a rejection) for
/// review with the current pages; earlier approvals become void.
pub fn submit(prj: &mut Project, i: usize) -> Result<(), String> {
    match prj.revisions.get(i).map(|r| r.state) {
        Some(State::Draft | State::InReview) => {}
        _ => return Err(tr("Only revisions that are not released can be submitted").into()),
    }
    let pages = Snapshot(Arc::new(prj.pages.clone()));
    let r = &mut prj.revisions[i];
    r.pages = pages;
    r.round += 1;
    r.state = State::InReview;
    Ok(())
}

/// Takes a revision in review back to draft.
pub fn withdraw(prj: &mut Project, i: usize) {
    if let Some(r) = prj.revisions.get_mut(i).filter(|r| r.state == State::InReview) {
        r.state = State::Draft;
    }
}

/// Removes a revision that is not released.
pub fn discard(prj: &mut Project, i: usize) {
    if prj.revisions.get(i).is_some_and(|r| r.state != State::Released) {
        prj.revisions.remove(i);
    }
}

/// Records the decision of `user` on revision `i` in review.
pub fn decide(prj: &mut Project, i: usize, user: &str, date: &str, approved: bool, note: &str) -> Result<(), String> {
    use crate::review::Record;
    let Some(r) = prj.revisions.get(i) else { return Err(tr("Unknown revision").into()) };
    if r.state != State::InReview {
        return Err(tr("The revision is not in review").into());
    }
    if !prj.workflow.is_approver(user) {
        return Err(tr_fmt!("{} is not an approver of this project", user));
    }
    let (name, round) = (r.name.clone(), r.round);
    let existing = prj.review.approvals.iter_mut().find(|a| !a.deleted && a.revision == name && a.round == round && a.user.eq_ignore_ascii_case(user));
    match existing {
        Some(a) => {
            a.approved = approved;
            a.date = date.into();
            a.note = note.into();
            a.bump();
        }
        None => {
            let id = prj.new_id();
            prj.review.approvals.push(crate::review::Approval {
                id,
                rev: 1,
                deleted: false,
                revision: name,
                round,
                user: user.trim().into(),
                date: date.into(),
                approved,
                note: note.into(),
            });
        }
    }
    Ok(())
}

/// Releases an approved revision; the pages must still be the approved ones.
pub fn release(prj: &mut Project, i: usize, lock: bool) -> Result<(), String> {
    match status(prj, i) {
        Status::Approved => {}
        Status::Draft if !prj.workflow.active() => {}
        Status::Released => return Err(tr("The revision is already released").into()),
        _ => return Err(tr("The revision is not approved").into()),
    }
    // Any change of the pages counts (only releasing locks them).
    let unlocked = |pages: &[Page]| pages.iter().map(|p| Page { locked: false, ..p.clone() }).collect::<Vec<_>>();
    if prj.revisions[i].state == State::InReview && unlocked(&prj.revisions[i].pages.0) != unlocked(&prj.pages) {
        return Err(tr("The pages were changed after the submission; submit the revision again").into());
    }
    finish_release(prj, i, lock);
    Ok(())
}

/// Report row: page, kind, item, description and jump target.
pub type OverviewRow = (String, String, String, String, Option<(usize, u64)>);

/// Report rows of the changes since the last revision.
pub fn overview(prj: &Project) -> Vec<OverviewRow> {
    let mut rows = Vec::new();
    let mut pages = pending(prj);
    pages.sort_by(|a, b| a.page.unwrap_or(usize::MAX).cmp(&b.page.unwrap_or(usize::MAX)).then_with(|| natural_cmp(&a.name, &b.name)));
    for pc in pages {
        for c in &pc.changes {
            let target = pc.page.map(|pi| (pi, if c.kind == ChangeKind::Removed || c.what == "Page" { 0 } else { c.id }));
            let label = match (c.label.is_empty(), c.details.is_empty()) {
                (_, true) => c.label.clone(),
                (true, false) => c.details_text(),
                (false, false) => format!("{} ({})", c.label, c.details_text()),
            };
            rows.push((pc.name.clone(), c.kind.title().to_string(), tr(c.what).to_string(), label, target));
        }
    }
    rows
}

#[cfg(test)]
mod tests;
