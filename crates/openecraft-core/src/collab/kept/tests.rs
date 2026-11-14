use super::*;
use crate::collab::{save_plan, Lock, Opened, Session, Shared, Snapshot};
use crate::model::{Page, PageKind};
use std::collections::BTreeSet;

/// A project of two pages at revision 3 on the server.
fn server() -> Project {
    let mut prj = Project::default();
    for (name, d) in [("1", "Supply"), ("2", "Motors")] {
        let id = prj.new_id();
        prj.pages.push(Page::new(id, name, d, PageKind::Schematic));
    }
    prj
}

fn open(session: &str, project: &Project, first_id: u64) -> (Shared, Project) {
    let opened = Opened {
        session: Session { user: format!("user {session}"), session: session.into() },
        first_id,
        snapshot: Snapshot { revision: 3, project: project.clone(), ..Default::default() },
        rights: Vec::new(),
    };
    Shared::open("plant", opened)
}

/// What the project server does with a save based on `base` (see the save route).
fn save(stored: &Project, base: &Project, mine: &Project, locks: &mut Vec<Lock>, session: &str) -> (Project, Vec<Target>) {
    let plan = save_plan(&changes(base, mine), &changes(base, stored), locks, session, |_| true);
    locks.extend(plan.lock.iter().map(|t| Lock { target: *t, user: format!("user {session}"), session: session.into() }));
    let merged = merge(stored, mine, &Owned::of(locks, session));
    (merged, plan.ignored)
}

#[test]
fn a_partial_save_keeps_the_left_out_change() {
    let r = server();
    let (p1, p2) = (r.pages[0].id, r.pages[1].id);
    let (_, mut anna) = open("a", &r, 1_000_000);
    let (mut sh_b, mut ben) = open("b", &r, 2_000_000);
    let mut locks = Vec::new();

    // Anna saves page 1 and releases it.
    anna.pages[0].description = "Anna".into();
    let (stored, ignored) = save(&r, &r, &anna, &mut locks, "a");
    assert!(ignored.is_empty());
    locks.clear();

    // Ben, still on revision 3, changed page 1 and the free page 2.
    ben.pages[0].description = "Ben".into();
    ben.pages[1].description = "Ben 2".into();
    let (stored, ignored) = save(&stored, &r, &ben, &mut locks, "b");
    assert_eq!(ignored, vec![Target::Page(p1)]);
    assert_eq!(stored.pages[0].description, "Anna", "the foreign change is not overwritten");
    assert_eq!(stored.pages[1].description, "Ben 2");

    // Ben takes the answer: page 1 is Anna's now, his version is kept.
    let (lost, apply) = sh_b.update(Snapshot { revision: 5, project: stored.clone(), locks: locks.clone(), ignored: ignored.clone() });
    assert!(lost.is_empty());
    let after = apply(&ben);
    assert_eq!(after.pages[0].description, "Anna");
    assert_eq!(after.pages[1].description, "Ben 2");
    let kept = Kept::capture(&r, &ben, &after, KeptReason::NotSaved).expect("something was replaced");
    assert_eq!(kept.targets, vec![Target::Page(p1)]);
    assert_eq!(kept.project.pages[0].description, "Ben");
    assert_eq!(kept.differing(&after), vec![Target::Page(p1)]);

    // Taken over on purpose: only page 1 changes, as an edit that needs its lock.
    let restored = restore(&after, &kept.project, &kept.targets);
    assert_eq!(restored.pages[0].description, "Ben");
    assert_eq!(restored.pages[1].description, "Ben 2");
    assert_eq!(sh_b.unlocked_changes(&restored), vec![Target::Page(p1)]);
    assert!(kept.differing(&restored).is_empty());
    let _ = p2;
}

#[test]
fn nothing_is_kept_when_nothing_local_is_replaced() {
    let r = server();
    let (mut sh, mut prj) = open("b", &r, 2_000_000);
    let id = prj.pages[0].id;
    prj.pages[0].description = "mine".into();
    let locks = vec![Lock { target: Target::Page(id), user: "b".into(), session: "b".into() }];
    let mut theirs = r.clone();
    theirs.pages[1].description = "theirs".into();
    let (_, apply) = sh.update(Snapshot { revision: 4, project: theirs, locks, ..Default::default() });
    let after = apply(&prj);
    assert_eq!(after.pages[0].description, "mine");
    assert!(Kept::capture(&r, &prj, &after, KeptReason::NoLock).is_none());
}

#[test]
fn a_lost_lock_keeps_the_change() {
    let r = server();
    let (mut sh, mut prj) = open("b", &r, 2_000_000);
    let id = prj.pages[0].id;
    sh.locks = vec![Lock { target: Target::Page(id), user: "b".into(), session: "b".into() }];
    prj.pages[0].description = "mine".into();
    // The session expired: the server's answer has no lock of it any more.
    let (_, apply) = sh.update(Snapshot { revision: 4, project: r.clone(), ..Default::default() });
    let after = apply(&prj);
    assert_eq!(after.pages[0].description, r.pages[0].description);
    let kept = Kept::capture(&r, &prj, &after, KeptReason::NoLock).unwrap();
    assert_eq!(kept.targets, vec![Target::Page(id)]);
}

#[test]
fn requested_locks_keep_the_change_until_answered() {
    let r = server();
    let (p1, p2) = (r.pages[0].id, r.pages[1].id);
    let (mut sh, mut prj) = open("b", &r, 2_000_000);
    prj.pages[0].description = "mine 1".into();
    prj.pages[1].description = "mine 2".into();
    sh.requested.extend([Target::Page(p1), Target::Page(p2)]);
    // Another user changed page 2 meanwhile, page 1 stays as it was.
    let mut theirs = r.clone();
    theirs.pages[1].description = "theirs 2".into();
    let (_, apply) = sh.update(Snapshot { revision: 4, project: theirs, ..Default::default() });
    let after = apply(&prj);
    assert_eq!(after.pages[0].description, "mine 1");
    assert_eq!(after.pages[1].description, "theirs 2");
    assert_eq!(Kept::capture(&r, &prj, &after, KeptReason::Changed).unwrap().targets, vec![Target::Page(p2)]);
}

#[test]
fn restoring_project_data_keeps_pages_of_others() {
    let r = server();
    let (_, mut mine) = open("b", &r, 2_000_000);
    mine.info.name = "mine".into();
    let new = mine.new_id();
    mine.pages.push(Page::new(new, "9", "my new page", PageKind::Schematic));
    let mut current = r.clone();
    current.next_id = 3_000_000;
    let other = current.new_id();
    current.pages.push(Page::new(other, "8", "their new page", PageKind::Schematic));
    current.review.comments.push(Default::default());
    let out = restore(&current, &mine, &[Target::Project]);
    assert_eq!(out.info.name, "mine");
    let ids: BTreeSet<u64> = out.pages.iter().map(|p| p.id).collect();
    assert!(ids.contains(&new) && ids.contains(&other));
    assert_eq!(out.review, current.review);
    assert_eq!(out.next_id, current.next_id);
}

#[test]
fn copies_are_listed_once_and_limited() {
    let r = server();
    let mut prj = r.clone();
    prj.pages[0].description = "x".into();
    let kept = |name: &str, prj: &Project| Kept { server: "http://h:8091/".into(), name: name.into(), ..Kept::capture(&r, prj, &r, KeptReason::NotSaved).unwrap() };
    let mut list = Vec::new();
    assert!(keep(&mut list, kept("plant", &prj)));
    assert!(!keep(&mut list, kept("plant", &prj)), "the same copy again");
    assert!(keep(&mut list, kept("other", &prj)));
    assert!(list[0].of("http://h:8091", "PLANT"));
    for i in 0..MAX_KEPT {
        prj.pages[0].description = format!("v{i}");
        keep(&mut list, kept("plant", &prj));
    }
    assert_eq!(list.len(), MAX_KEPT);
    assert_eq!(list.last().unwrap().project.pages[0].description, format!("v{}", MAX_KEPT - 1));
}

#[test]
fn copies_survive_storing() {
    let r = server();
    let mut prj = r.clone();
    prj.pages[1].description = "kept".into();
    let kept = Kept { server: "http://h".into(), name: "plant".into(), revision: 7, time: "2026-11-14 10:00".into(), ..Kept::capture(&r, &prj, &r, KeptReason::Refused).unwrap() };
    let text = serde_json::to_string(&vec![kept.clone()]).unwrap();
    let back: Vec<Kept> = serde_json::from_str(&text).unwrap();
    assert_eq!(back, vec![kept]);
}

#[test]
fn a_change_of_a_page_another_user_removed_is_kept() {
    let base = server();
    let last = base.pages.len() - 1;
    let mut mine = base.clone();
    mine.pages[last].description = "mine".into();
    let mut theirs = base.clone();
    theirs.pages.remove(last);
    let after = merge(&theirs, &mine, &Owned::default());
    assert_eq!(after.pages.len(), last);
    let kept = Kept::capture(&base, &mine, &after, KeptReason::Changed).unwrap();
    assert_eq!(kept.targets, vec![Target::Page(mine.pages[last].id)]);
    // A page removed locally is no lost change.
    let mut removed = base.clone();
    removed.pages.remove(last);
    assert!(Kept::capture(&base, &removed, &removed, KeptReason::Changed).is_none());
}
