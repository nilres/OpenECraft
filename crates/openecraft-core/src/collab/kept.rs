//! Local changes of a shared project that the editor replaced with the server's
//! version (a save left them out, another user changed them first, a lock was refused
//! or lost). A copy of the local project is kept, so that the changes can be compared,
//! exported or taken over again on purpose; nothing of them is lost silently.

use super::{changes, merge, page, Owned, Target};
use crate::model::Project;
use serde::{Deserialize, Serialize};

/// Copies kept at most (the oldest are dropped).
pub const MAX_KEPT: usize = 20;

/// Why local changes were replaced.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeptReason {
    /// A save left them out: locked or changed by another user, or not allowed.
    #[default]
    NotSaved,
    /// Another user changed them before this session got the lock.
    Changed,
    /// The lock was refused or could not be requested.
    Refused,
    /// The session no longer holds the lock when the server's version arrived.
    NoLock,
}

/// Local version of `targets` of a shared project, replaced by the server's version
/// of `revision`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Kept {
    /// Address of the project server.
    pub server: String,
    /// Name of the project on the server.
    pub name: String,
    pub user: String,
    pub revision: u64,
    pub reason: KeptReason,
    pub targets: Vec<Target>,
    /// When it happened (shown as is).
    pub time: String,
    /// The local project before the replacement.
    pub project: Project,
}

impl Kept {
    /// The copy of the local changes (`before`, edited from the server's `base`) that
    /// `after` replaced; none if no local change was lost. A changed page that `after`
    /// no longer has (another user removed it) is lost too.
    pub fn capture(base: &Project, before: &Project, after: &Project, reason: KeptReason) -> Option<Kept> {
        let differ = changes(after, before);
        let removed = |t: &Target| matches!(t, Target::Page(id) if page(after, *id).is_none());
        let targets: Vec<Target> = changes(base, before).into_iter().filter(|t| differ.contains(t) || removed(t)).collect();
        (!targets.is_empty()).then(|| Kept { reason, targets, project: before.clone(), ..Default::default() })
    }

    /// Belongs to the project `name` of the server `server`.
    pub fn of(&self, server: &str, name: &str) -> bool {
        self.server.trim_end_matches('/') == server.trim_end_matches('/') && super::name_key(&self.name) == super::name_key(name)
    }

    /// Targets whose kept version differs from `current` (the others need nothing).
    pub fn differing(&self, current: &Project) -> Vec<Target> {
        let differ = changes(current, &self.project);
        self.targets.iter().copied().filter(|t| differ.contains(t)).collect()
    }
}

/// Adds `kept` to `list` (newest last): a copy equal to the newest one of its project
/// is not added twice; at most [`MAX_KEPT`] stay. Returns whether `list` changed.
pub fn keep(list: &mut Vec<Kept>, kept: Kept) -> bool {
    let newest = list.iter().rev().find(|k| k.of(&kept.server, &kept.name));
    if newest.is_some_and(|k| k.targets == kept.targets && k.project == kept.project) {
        return false;
    }
    list.push(kept);
    if list.len() > MAX_KEPT {
        let drop = list.len() - MAX_KEPT;
        list.drain(..drop);
    }
    true
}

/// `current` with the kept version of `targets` taken over, as a change of the user:
/// it needs the locks like any other edit. Pages that only `current` has stay (pages of
/// others added meanwhile), the review and the newest object id are those of `current`.
pub fn restore(current: &Project, kept: &Project, targets: &[Target]) -> Project {
    let mut owned = Owned::default();
    for t in targets {
        owned.add(*t);
    }
    let mut out = merge(current, kept, &owned);
    if owned.project {
        for p in &current.pages {
            if page(&out, p.id).is_none() {
                out.pages.push(p.clone());
            }
        }
    }
    out.review = current.review.clone();
    out.next_id = current.next_id.max(kept.next_id);
    out
}

#[cfg(test)]
mod tests;
