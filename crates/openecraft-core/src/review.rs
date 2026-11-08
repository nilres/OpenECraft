//! Review of a project: comments with status and replies, redline / greenline
//! markup on pages, approvals of revisions and change requests.
//!
//! Every record has its own edit counter (`rev`), so the records of several users
//! of a shared project are merged without locks: the newer edit wins, deleted
//! records are kept as marks ([`merge`]).

use crate::geom::{dist_to_segment, p, P, R};
use crate::model::{natural_cmp, Project};
use crate::render::{Draw, Ink, VAnchor};
use crate::symbols::Anchor;
use openecraft_i18n::{tr, tr_fmt};
use openecraft_parts::{csv_escape, parse_csv};
use serde::{Deserialize, Serialize};
pub use crate::model::review::{Approval, ChangeRequest, Comment, Markup, MarkupKind, Pen, Priority, Reply, RequestStatus, Review, Status};

impl Review {
    pub fn is_empty(&self) -> bool {
        self.comments.is_empty() && self.markups.is_empty() && self.approvals.is_empty() && self.requests.is_empty()
    }

    pub fn comment(&self, id: u64) -> Option<&Comment> {
        self.comments.iter().find(|c| c.id == id && !c.deleted)
    }

    pub fn comment_mut(&mut self, id: u64) -> Option<&mut Comment> {
        self.comments.iter_mut().find(|c| c.id == id && !c.deleted)
    }

    pub fn request(&self, id: u64) -> Option<&ChangeRequest> {
        self.requests.iter().find(|c| c.id == id && !c.deleted)
    }

    /// Comments that are not deleted.
    pub fn live_comments(&self) -> impl Iterator<Item = &Comment> {
        self.comments.iter().filter(|c| !c.deleted)
    }

    pub fn live_markups(&self) -> impl Iterator<Item = &Markup> {
        self.markups.iter().filter(|m| !m.deleted)
    }

    pub fn live_requests(&self) -> impl Iterator<Item = &ChangeRequest> {
        self.requests.iter().filter(|r| !r.deleted)
    }

    /// Number of a comment as shown on the page (1-based, in creation order).
    pub fn comment_number(&self, id: u64) -> usize {
        self.live_comments().position(|c| c.id == id).map(|i| i + 1).unwrap_or(0)
    }
}

/// A record merged by id and edit counter.
pub trait Record: Clone {
    fn id(&self) -> u64;
    fn rev(&self) -> u64;
    /// Marks an edit; call after changing the record.
    fn bump(&mut self);
}

macro_rules! record {
    ($t:ty) => {
        impl Record for $t {
            fn id(&self) -> u64 {
                self.id
            }
            fn rev(&self) -> u64 {
                self.rev
            }
            fn bump(&mut self) {
                self.rev += 1;
            }
        }
    };
}

/// Records of both lists; of a record in both the one with more edits (on a tie
/// `theirs`). Order of `theirs`, then the new records of `mine`.
pub fn merge_records<T: Record>(theirs: &[T], mine: &[T]) -> Vec<T> {
    let mut out: Vec<T> = theirs
        .iter()
        .map(|t| match mine.iter().find(|m| m.id() == t.id()) {
            Some(m) if m.rev() > t.rev() => m.clone(),
            _ => t.clone(),
        })
        .collect();
    out.extend(mine.iter().filter(|m| !theirs.iter().any(|t| t.id() == m.id())).cloned());
    out
}

/// What a signed-in `user` may write into the review records of a shared project
/// (`mine`, compared with the stored `theirs`): new approvals, comments, replies and
/// markups are in their name, the authors of existing records stay, and approvals of
/// other users stay as they are.
pub fn sign(theirs: &Review, mine: &mut Review, user: &str) {
    let user = user.trim();
    for a in &mut mine.approvals {
        match theirs.approvals.iter().find(|t| t.id == a.id) {
            None => a.user = user.into(),
            Some(t) if a.rev > t.rev && !t.user.trim().eq_ignore_ascii_case(user) => *a = t.clone(),
            Some(t) => a.user = t.user.clone(),
        }
    }
    for c in &mut mine.comments {
        let old = theirs.comments.iter().find(|t| t.id == c.id);
        c.author = old.map_or_else(|| user.into(), |t| t.author.clone());
        // A reply keeps its author only where it is a stored reply unchanged (each
        // stored one once), so nobody writes under another name by inserting replies.
        let mut stored: Vec<&Reply> = old.map(|t| t.replies.iter().collect()).unwrap_or_default();
        for r in &mut c.replies {
            r.author = match stored.iter().position(|t| t.text == r.text && t.date == r.date) {
                Some(i) => stored.remove(i).author.clone(),
                None => user.into(),
            };
        }
    }
    for m in &mut mine.markups {
        m.author = theirs.markups.iter().find(|t| t.id == m.id).map_or_else(|| user.into(), |t| t.author.clone());
    }
}

pub fn merge(theirs: &Review, mine: &Review) -> Review {
    Review {
        comments: merge_records(&theirs.comments, &mine.comments),
        markups: merge_records(&theirs.markups, &mine.markups),
        approvals: merge_records(&theirs.approvals, &mine.approvals),
        requests: merge_records(&theirs.requests, &mine.requests),
    }
}

// ------------------------------------------------------------------ comments

impl Status {
    pub const ALL: [Status; 4] = [Status::Open, Status::InProgress, Status::Resolved, Status::Rejected];

    pub fn title(self) -> &'static str {
        match self {
            Status::Open => tr("open"),
            Status::InProgress => tr("in progress"),
            Status::Resolved => tr("resolved"),
            Status::Rejected => tr("rejected"),
        }
    }

    pub fn is_done(self) -> bool {
        matches!(self, Status::Resolved | Status::Rejected)
    }
}

record!(Comment);

/// Adds a comment; returns its id.
pub fn add_comment(prj: &mut Project, page: u64, pos: P, author: &str, date: &str, text: &str) -> u64 {
    let id = prj.new_id();
    let c = Comment { id, rev: 1, page, pos, author: author.into(), date: date.into(), text: text.into(), ..Default::default() };
    prj.review.comments.push(c);
    id
}

// ------------------------------------------------------------------ markup

impl Pen {
    pub fn title(self) -> &'static str {
        match self {
            Pen::Red => tr("Redline"),
            Pen::Green => tr("Greenline"),
        }
    }

    pub fn ink(self) -> Ink {
        match self {
            Pen::Red => Ink::Color([220, 30, 30]),
            Pen::Green => Ink::Color([20, 150, 60]),
        }
    }
}

impl MarkupKind {
    pub const ALL: [MarkupKind; 4] = [MarkupKind::Line, MarkupKind::Rect, MarkupKind::Cloud, MarkupKind::Text];

    pub fn title(self) -> &'static str {
        match self {
            MarkupKind::Line => tr("Line"),
            MarkupKind::Rect => tr("Rectangle"),
            MarkupKind::Cloud => tr("Cloud"),
            MarkupKind::Text => tr("Text"),
        }
    }
}

record!(Markup);

impl Markup {
    pub fn bounds(&self) -> R {
        if self.kind == MarkupKind::Text {
            let w = self.text.lines().map(|l| l.chars().count()).max().unwrap_or(1) as f32 * TEXT_SIZE * 0.62;
            let h = self.text.lines().count().max(1) as f32 * TEXT_SIZE * 1.4;
            return R::from_points(self.a + p(0.0, -TEXT_SIZE * 0.7), self.a + p(w, h - TEXT_SIZE * 0.7));
        }
        R::from_points(self.a, self.b)
    }

    /// Distance of `q` to the markup (0 inside texts).
    fn distance(&self, q: P) -> f32 {
        match self.kind {
            MarkupKind::Line => dist_to_segment(q, self.a, self.b),
            MarkupKind::Text => {
                if self.bounds().contains(q) {
                    0.0
                } else {
                    f32::MAX
                }
            }
            MarkupKind::Rect | MarkupKind::Cloud => {
                let r = self.bounds();
                let c = [p(r.min.x, r.min.y), p(r.max.x, r.min.y), p(r.max.x, r.max.y), p(r.min.x, r.max.y)];
                (0..4).map(|i| dist_to_segment(q, c[i], c[(i + 1) % 4])).fold(f32::MAX, f32::min)
            }
        }
    }
}

const TEXT_SIZE: f32 = 3.0;
const MARKER_R: f32 = 2.6;

/// Adds a markup (with a new id); returns the id.
pub fn add_markup(prj: &mut Project, m: Markup) -> u64 {
    let id = prj.new_id();
    prj.review.markups.push(Markup { id, rev: 1, ..m });
    id
}

/// Something of the review layer under the pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Comment(u64),
    Markup(u64),
}

/// The comment marker or markup at `q` on page `page` (comments first).
pub fn hit(prj: &Project, page: u64, q: P, tolerance: f32) -> Option<Hit> {
    let r = &prj.review;
    if let Some(c) = r.live_comments().filter(|c| c.page == page).find(|c| c.pos.dist(q) <= MARKER_R + tolerance) {
        return Some(Hit::Comment(c.id));
    }
    r.live_markups()
        .filter(|m| m.page == page)
        .map(|m| (m.distance(q), m.id))
        .filter(|(d, _)| *d <= tolerance)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id)| Hit::Markup(id))
}

/// Draws markups and comment markers of a page (the review layer).
pub fn draw(d: &mut dyn Draw, prj: &Project, page_i: usize, selected: Option<Hit>) {
    let Some(page) = prj.pages.get(page_i) else { return };
    let review = &prj.review;
    for m in review.live_markups().filter(|m| m.page == page.id) {
        let ink = if selected == Some(Hit::Markup(m.id)) {
            Ink::Selected
        } else if m.done {
            Ink::Ghost
        } else {
            m.pen.ink()
        };
        let w = 0.5;
        match m.kind {
            MarkupKind::Line => d.line(m.a, m.b, w, ink, false),
            MarkupKind::Rect => d.rect(m.bounds().min, m.bounds().max, w, ink, false),
            MarkupKind::Cloud => cloud(d, m.bounds(), w, ink),
            MarkupKind::Text => {
                for (i, line) in m.text.lines().enumerate() {
                    d.text(m.a + p(0.0, i as f32 * TEXT_SIZE * 1.4), line, TEXT_SIZE, Anchor::Left, VAnchor::Middle, ink);
                }
            }
        }
    }
    for c in review.live_comments().filter(|c| c.page == page.id) {
        let ink = if selected == Some(Hit::Comment(c.id)) {
            Ink::Selected
        } else if c.status.is_done() {
            Ink::Ghost
        } else {
            Ink::Color([230, 140, 0])
        };
        d.circle(c.pos, MARKER_R, 0.35, ink, false);
        d.line(c.pos + p(-1.2, MARKER_R - 0.4), c.pos + p(-2.4, MARKER_R + 1.4), 0.35, ink, false);
        d.text(c.pos, &review.comment_number(c.id).to_string(), 2.2, Anchor::Center, VAnchor::Middle, ink);
    }
}

/// Revision cloud: arcs along the rectangle, bulging outwards.
fn cloud(d: &mut dyn Draw, r: R, w: f32, ink: Ink) {
    let corners = [p(r.min.x, r.min.y), p(r.max.x, r.min.y), p(r.max.x, r.max.y), p(r.min.x, r.max.y)];
    let center = r.center();
    for i in 0..4 {
        let (a, b) = (corners[i], corners[(i + 1) % 4]);
        let len = a.dist(b);
        let n = (len / 6.0).ceil().max(1.0) as usize;
        for k in 0..n {
            let s = a + (b - a) * (k as f32 / n as f32);
            let e = a + (b - a) * ((k + 1) as f32 / n as f32);
            let m = (s + e) * 0.5;
            let out = m - center;
            let out = out * (1.0 / out.len().max(1e-3));
            let radius = s.dist(e) / 2.0;
            let pts: Vec<P> = (0..=8)
                .map(|j| {
                    let t = j as f32 / 8.0;
                    let along = s + (e - s) * t;
                    let bulge = (std::f32::consts::PI * t).sin() * radius * 0.8;
                    along + out * bulge
                })
                .collect();
            d.poly(&pts, false, w, ink, false);
        }
    }
}

// ------------------------------------------------------------------ approvals

record!(Approval);

// ------------------------------------------------------------------ change requests

impl Priority {
    pub const ALL: [Priority; 3] = [Priority::Low, Priority::Normal, Priority::High];

    pub fn title(self) -> &'static str {
        match self {
            Priority::Low => tr("low"),
            Priority::Normal => tr("normal"),
            Priority::High => tr("high"),
        }
    }

    /// Name in exchange files.
    pub fn key(self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Normal => "normal",
            Priority::High => "high",
        }
    }
}

impl RequestStatus {
    pub const ALL: [RequestStatus; 6] =
        [RequestStatus::New, RequestStatus::Accepted, RequestStatus::InWork, RequestStatus::Implemented, RequestStatus::Rejected, RequestStatus::Closed];

    pub fn title(self) -> &'static str {
        match self {
            RequestStatus::New => tr("new"),
            RequestStatus::Accepted => tr("accepted"),
            RequestStatus::InWork => tr("in work"),
            RequestStatus::Implemented => tr("implemented"),
            RequestStatus::Rejected => tr("rejected"),
            RequestStatus::Closed => tr("closed"),
        }
    }

    /// Name in exchange files.
    pub fn key(self) -> &'static str {
        match self {
            RequestStatus::New => "new",
            RequestStatus::Accepted => "accepted",
            RequestStatus::InWork => "in_work",
            RequestStatus::Implemented => "implemented",
            RequestStatus::Rejected => "rejected",
            RequestStatus::Closed => "closed",
        }
    }

    fn from_key(s: &str) -> Option<RequestStatus> {
        let s = s.trim().to_lowercase().replace([' ', '-'], "_");
        RequestStatus::ALL.into_iter().find(|r| r.key() == s)
    }

    pub fn is_done(self) -> bool {
        matches!(self, RequestStatus::Rejected | RequestStatus::Closed)
    }
}

record!(ChangeRequest);

/// Next free number "CR-001", "CR-002", …
pub fn next_request_number(prj: &Project) -> String {
    let max = prj
        .review
        .live_requests()
        .filter_map(|r| r.number.strip_prefix("CR-").and_then(|n| n.parse::<u32>().ok()))
        .max()
        .unwrap_or(0);
    format!("CR-{:03}", max + 1)
}

pub fn add_request(prj: &mut Project, title: &str, requester: &str, date: &str) -> u64 {
    let id = prj.new_id();
    let number = next_request_number(prj);
    let r = ChangeRequest { id, rev: 1, number, title: title.into(), requester: requester.into(), date: date.into(), ..Default::default() };
    prj.review.requests.push(r);
    id
}

/// Column names of the CSV exchange format (not translated).
pub const REQUEST_FIELDS: [&str; 11] =
    ["Number", "Title", "Description", "Requester", "Origin", "Date", "Priority", "Status", "Pages", "Devices", "Revision"];

/// Change request as exchanged with other systems: pages by name instead of id.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExchangeRequest {
    pub number: String,
    pub title: String,
    pub description: String,
    pub requester: String,
    pub origin: String,
    pub date: String,
    pub priority: Priority,
    pub status: RequestStatus,
    pub pages: Vec<String>,
    pub devices: Vec<String>,
    pub revision: String,
}

fn page_name(prj: &Project, id: u64) -> Option<String> {
    prj.pages.iter().find(|p| p.id == id).map(crate::structure::page_full_name)
}

fn page_id(prj: &Project, name: &str) -> Option<u64> {
    let name = name.trim();
    prj.pages.iter().find(|p| crate::structure::page_full_name(p) == name || p.name == name).map(|p| p.id)
}

fn to_exchange(prj: &Project, r: &ChangeRequest) -> ExchangeRequest {
    ExchangeRequest {
        number: r.number.clone(),
        title: r.title.clone(),
        description: r.description.clone(),
        requester: r.requester.clone(),
        origin: r.origin.clone(),
        date: r.date.clone(),
        priority: r.priority,
        status: r.status,
        pages: r.pages.iter().filter_map(|id| page_name(prj, *id)).collect(),
        devices: r.devices.clone(),
        revision: r.revision.clone(),
    }
}

pub fn requests_json(prj: &Project) -> String {
    let list: Vec<ExchangeRequest> = prj.review.live_requests().map(|r| to_exchange(prj, r)).collect();
    serde_json::to_string_pretty(&list).unwrap_or_default()
}

pub fn requests_csv(prj: &Project) -> String {
    let mut out = REQUEST_FIELDS.join(";");
    out.push('\n');
    for r in prj.review.live_requests() {
        let e = to_exchange(prj, r);
        let cells = [
            e.number,
            e.title,
            e.description,
            e.requester,
            e.origin,
            e.date,
            e.priority.key().to_string(),
            e.status.key().to_string(),
            e.pages.join(", "),
            e.devices.join(", "),
            e.revision,
        ];
        out.push_str(&cells.iter().map(|c| csv_escape(c)).collect::<Vec<_>>().join(";"));
        out.push('\n');
    }
    out
}

fn parse_requests(text: &str) -> Result<Vec<ExchangeRequest>, String> {
    let trimmed = text.trim_start_matches('\u{feff}').trim_start();
    if trimmed.starts_with('[') {
        return serde_json::from_str(trimmed).map_err(|e| tr_fmt!("Not a change request file: {}", e));
    }
    let rows = parse_csv(trimmed);
    let Some((header, rows)) = rows.split_first() else { return Ok(Vec::new()) };
    let col = |name: &str| header.iter().position(|h| h.trim().eq_ignore_ascii_case(name));
    let number = col("Number").ok_or_else(|| tr("Not a change request file: the column \"Number\" is missing").to_string())?;
    let cols: Vec<Option<usize>> = REQUEST_FIELDS.iter().map(|f| col(f)).collect();
    let list = |s: &str| s.split([',', '|']).map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).collect::<Vec<_>>();
    Ok(rows
        .iter()
        .filter(|r| r.get(number).is_some_and(|n| !n.trim().is_empty()))
        .map(|r| {
            let cell = |i: usize| cols[i].and_then(|c| r.get(c)).map(|s| s.trim().to_string()).unwrap_or_default();
            ExchangeRequest {
                number: cell(0),
                title: cell(1),
                description: cell(2),
                requester: cell(3),
                origin: cell(4),
                date: cell(5),
                priority: Priority::ALL.into_iter().find(|p| p.key() == cell(6).to_lowercase()).unwrap_or_default(),
                status: RequestStatus::from_key(&cell(7)).unwrap_or_default(),
                pages: list(&cell(8)),
                devices: list(&cell(9)),
                revision: cell(10),
            }
        })
        .collect())
}

/// Reads change requests from CSV or JSON. Requests with a known number are
/// updated, others added. Returns (added, updated).
pub fn import_requests(prj: &mut Project, text: &str) -> Result<(usize, usize), String> {
    let list = parse_requests(text)?;
    let (mut added, mut updated) = (0, 0);
    for e in list {
        let pages: Vec<u64> = e.pages.iter().filter_map(|n| page_id(prj, n)).collect();
        let existing = prj.review.requests.iter().position(|r| !r.deleted && r.number == e.number);
        let id = match existing {
            Some(i) => prj.review.requests[i].id,
            None => prj.new_id(),
        };
        let mut r = ChangeRequest {
            id,
            rev: 0,
            deleted: false,
            number: e.number,
            title: e.title,
            description: e.description,
            requester: e.requester,
            origin: e.origin,
            date: e.date,
            priority: e.priority,
            status: e.status,
            pages,
            devices: e.devices,
            revision: e.revision,
        };
        match existing {
            Some(i) => {
                let old = &mut prj.review.requests[i];
                r.rev = old.rev;
                if *old != r {
                    r.bump();
                    *old = r;
                    updated += 1;
                }
            }
            None => {
                r.rev = 1;
                prj.review.requests.push(r);
                added += 1;
            }
        }
    }
    Ok((added, updated))
}

/// Change requests sorted by number.
pub fn sorted_requests(prj: &Project) -> Vec<&ChangeRequest> {
    let mut v: Vec<&ChangeRequest> = prj.review.live_requests().collect();
    v.sort_by(|a, b| natural_cmp(&a.number, &b.number));
    v
}

#[cfg(test)]
mod tests;
