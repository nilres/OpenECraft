use super::*;
use crate::remote::Answer;
use openecraft_core::collab::{Opened, Session, Snapshot};

fn listed(names: &[&str]) -> Vec<ProjectSummary> {
    names.iter().map(|n| ProjectSummary { name: n.to_string(), title: "Old title".into(), ..Default::default() }).collect()
}

fn properties(revision: u64, rights: &[ProjectRight]) -> ProjectProperties {
    let mut info = ProjectInfo { name: "Press line".into(), company: "ACME".into(), creator: "Anna".into(), date: "2026-11-02".into(), revision: "B".into(), ..Default::default() };
    info.properties.insert("Customer".into(), "Bolt Ltd".into());
    ProjectProperties { name: "Plant 1".into(), revision, info, locked_by: None, rights: rights.to_vec(), ..Default::default() }
}

const EDITOR: &[ProjectRight] = &[ProjectRight::View, ProjectRight::EditPages, ProjectRight::EditProject];

/// Takes `replies` as the answers of requests sent before.
fn answer(r: &mut Remote, replies: Vec<Reply>) -> Vec<Reply> {
    r.pending += replies.len();
    for reply in replies {
        r.inbox.lock().unwrap().push(Answer { generation: r.sign_in.generation, counted: true, endpoint: r.endpoint(), project: None, reply });
    }
    r.poll(&egui::Context::default())
}

fn remote(copy: bool) -> Remote {
    let mut r = Remote { user: "ben".into(), projects: listed(&["Plant 1"]), ..Default::default() };
    r.draft = Some(Draft::new("Plant 1", copy, &r.projects));
    r.draft.as_mut().unwrap().busy = true;
    r
}

#[test]
fn a_copy_is_a_draft_with_the_properties_of_the_source_and_a_free_name() {
    let projects = listed(&["Plant 1", "plant 1 COPY"]);
    let mut d = Draft::new("Plant 1", true, &projects);
    assert_eq!(d.name, "Plant 1 copy 2");
    // Nothing to confirm before the properties are loaded.
    assert!(!d.ready(&projects));
    d.take(properties(4, &[ProjectRight::View]));
    assert_eq!((d.revision, d.info.company.as_str(), d.info.properties["Customer"].as_str()), (4, "ACME", "Bolt Ltd"));
    // Viewing the source is enough; the properties of the copy are free to change.
    assert_eq!(d.blocked(), None);
    assert!(d.ready(&projects));
    d.info.name = "Press line 2".into();
    d.records = true;
    let req = d.request("ben");
    assert_eq!((req.name.as_str(), req.user.as_str(), req.revision, req.records), ("Plant 1 copy 2", "ben", 4, true));
    assert_eq!(req.info.unwrap().name, "Press line 2");

    d.name = "Plant 1".into();
    assert_eq!(d.name_error(&projects), Some(NameError::Taken("Plant 1".into())));
    d.name = "PLANT 1 copy".into();
    assert_eq!(d.name_error(&projects), Some(NameError::Taken("plant 1 COPY".into())));
    for bad in ["", " x", "a/b", "CON"] {
        d.name = bad.into();
        assert_eq!(d.name_error(&projects), Some(NameError::Invalid), "{bad:?}");
        assert!(!d.ready(&projects));
    }
}

#[test]
fn properties_are_changed_only_with_the_right_and_without_a_lock() {
    let projects = listed(&["Plant 1"]);
    let mut d = Draft::new("Plant 1", false, &projects);
    d.take(properties(4, &[ProjectRight::View, ProjectRight::Review]));
    assert_eq!(d.blocked(), Some(Blocked::Rights));
    d.info.company = "Other".into();
    assert!(!d.ready(&projects));

    d.take(ProjectProperties { locked_by: Some("anna".into()), ..properties(4, EDITOR) });
    assert_eq!(d.blocked(), Some(Blocked::Locked("anna".into())));
    d.info.company = "Other".into();
    assert!(!d.ready(&projects));

    d.take(properties(5, EDITOR));
    assert_eq!(d.blocked(), None);
    // Nothing to save until something is changed.
    assert!(!d.ready(&projects));
    d.info.company = "Other".into();
    assert!(d.edited() && d.ready(&projects));
}

#[test]
fn a_source_changed_meanwhile_keeps_the_entries() {
    let projects = listed(&["Plant 1"]);
    let mut copy = Draft::new("Plant 1", true, &projects);
    copy.take(properties(4, EDITOR));
    copy.info.name = "Mine".into();
    copy.busy = true;
    copy.creating = true;
    copy.refused(Refusal { changed: true, error: "project 'Plant 1' was changed meanwhile (revision 5)".into() });
    // Confirmed again, the copy takes whatever is saved then.
    assert!(copy.changed && !copy.busy && !copy.creating && copy.error.is_empty());
    assert_eq!((copy.revision, copy.info.name.as_str()), (0, "Mine"));
    assert!(copy.ready(&projects));
    assert_eq!(copy.request("ben").revision, 0);

    // Changed properties are never saved over newer ones: they are loaded again first.
    let mut edit = Draft::new("Plant 1", false, &projects);
    edit.take(properties(4, EDITOR));
    edit.info.name = "Mine".into();
    edit.refused(Refusal { changed: true, error: "changed".into() });
    assert!(edit.changed && !edit.ready(&projects));
    edit.take(properties(5, EDITOR));
    assert!(!edit.changed);
    assert_eq!(edit.info.name, "Press line");
}

#[test]
fn the_properties_come_from_the_server_not_from_the_list() {
    let mut r = remote(false);
    // An answer for another project, e.g. of a dialog closed before.
    assert!(answer(&mut r, vec![Reply::Properties("Other".into(), Ok(properties(9, EDITOR)))]).is_empty());
    assert!(!r.draft.as_ref().unwrap().loaded);
    assert!(answer(&mut r, vec![Reply::Properties("Plant 1".into(), Ok(properties(4, EDITOR)))]).is_empty());
    let d = r.draft.as_ref().unwrap();
    assert!(d.loaded && !d.busy);
    assert_eq!((d.info.name.as_str(), d.revision), ("Press line", 4));

    // Saved: the list shows the new title, the dialog is done.
    let mut saved = properties(5, EDITOR);
    saved.meta = ProjectMeta::created("anna", 7);
    saved.meta.changed("ben", 9);
    saved.info.name = "Press line 2".into();
    assert!(answer(&mut r, vec![Reply::PropertiesSaved("Plant 1".into(), Ok(saved))]).is_empty());
    assert!(r.draft.is_none());
    assert_eq!((r.projects[0].title.as_str(), r.projects[0].revision), ("Press line 2", 5));
    assert_eq!((r.projects[0].meta.changed_by.as_str(), r.projects[0].meta.changed_at), ("ben", 9));
    assert!(r.status.contains("revision 5"), "{}", r.status);
}

#[test]
fn a_failed_load_or_save_stays_in_the_dialog() {
    let mut r = remote(false);
    answer(&mut r, vec![Reply::Properties("Plant 1".into(), Err("connection failed".into()))]);
    let d = r.draft.as_ref().unwrap();
    assert!(!d.loaded && !d.busy);
    assert_eq!(d.error, "connection failed");

    answer(&mut r, vec![Reply::Properties("Plant 1".into(), Ok(properties(4, EDITOR)))]);
    r.draft.as_mut().unwrap().info.company = "Mine".into();
    r.draft.as_mut().unwrap().busy = true;
    let locked = Refusal { changed: false, error: "the project data is locked by anna".into() };
    assert!(answer(&mut r, vec![Reply::PropertiesSaved("Plant 1".into(), Err(locked))]).is_empty());
    let d = r.draft.as_ref().unwrap();
    assert_eq!((d.error.as_str(), d.info.company.as_str(), d.busy), ("the project data is locked by anna", "Mine", false));
    assert_eq!(r.projects[0].title, "Old title");
}

fn opened(name: &str) -> Box<Opened> {
    let mut snapshot = Snapshot { revision: 1, ..Default::default() };
    snapshot.project.info.name = name.into();
    Box::new(Opened { session: Session { user: "ben".into(), session: "s1".into() }, first_id: 100, snapshot, rights: vec![ProjectRight::View, ProjectRight::ManageAccess] })
}

#[test]
fn a_created_copy_is_opened_and_a_refused_one_leaves_the_draft() {
    let mut r = remote(true);
    answer(&mut r, vec![Reply::Properties("Plant 1".into(), Ok(properties(4, EDITOR)))]);
    let name = r.draft.as_ref().unwrap().name.clone();
    assert_eq!(name, "Plant 1 copy");

    // Creation failed: nothing opens, the entries stay to be corrected.
    r.draft.as_mut().unwrap().info.name = "Mine".into();
    r.draft.as_mut().unwrap().busy = true;
    r.draft.as_mut().unwrap().creating = true;
    let taken = Refusal { changed: false, error: "project 'Plant 1 copy' already exists".into() };
    assert!(answer(&mut r, vec![Reply::Copied(name.clone(), Err(taken))]).is_empty());
    let d = r.draft.as_ref().unwrap();
    assert_eq!((d.error.as_str(), d.info.name.as_str(), d.creating), ("project 'Plant 1 copy' already exists", "Mine", false));
    assert_eq!(r.projects.len(), 1);

    // Created: it goes on as an opened project.
    let replies = answer(&mut r, vec![Reply::Copied(name.clone(), Ok(opened("Mine")))]);
    assert!(matches!(&replies[..], [Reply::Opened(n, Ok(o))] if *n == name && o.session.session == "s1"));
    assert!(r.draft.is_none());
    assert_eq!(r.rights, vec![ProjectRight::View, ProjectRight::ManageAccess]);
    assert_eq!(r.projects.iter().map(|p| (p.name.as_str(), p.title.as_str())).collect::<Vec<_>>(), [("Plant 1", "Old title"), ("Plant 1 copy", "Mine")]);
    assert!(r.status.contains("Plant 1 copy") && r.status.contains("copy of Plant 1"), "{}", r.status);
    let listed = &r.projects[1];
    assert_eq!((listed.meta.created_by.as_str(), listed.copied_from.clone()), ("ben", Some(CopySource { name: "Plant 1".into(), revision: 4 })));
    // The message of the next project opened is the usual one.
    answer(&mut r, vec![Reply::Opened("Plant 1".into(), Ok(opened("Press line")))]);
    assert!(!r.status.contains("copy of"), "{}", r.status);
}

#[test]
fn signing_out_drops_the_draft() {
    let mut r = remote(true);
    r.selected = Some("Plant 1".into());
    r.change_user();
    assert!(r.draft.is_none() && r.selected.is_none());
}
