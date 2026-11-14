use super::*;

fn directory(id: &str, password: &str) -> Directory {
    Directory { id: id.into(), url: "ldaps://dc1.corp.example".into(), base_dn: "DC=corp".into(), bind_password: password.into(), ..Default::default() }
}

#[test]
fn entries_hide_the_password() {
    let e = DirectoryEntry::of(&directory("corp", "secret"));
    assert!(e.password_set && e.directory.bind_password.is_empty());
    let json = serde_json::to_value(&e).unwrap();
    assert_eq!(json["id"], "corp");
    assert_eq!(json["password_set"], true);
    assert!(!DirectoryEntry::of(&directory("corp", "")).password_set);
}

#[test]
fn an_empty_password_keeps_the_stored_one() {
    let stored = [directory("corp", "secret")];
    let d = directory_of(&stored, "CORP", directory("", "")).unwrap();
    assert_eq!((d.id.as_str(), d.bind_password.as_str()), ("CORP", "secret"));
    assert_eq!(directory_of(&stored, "corp", directory("corp", "new")).unwrap().bind_password, "new");
    assert!(directory_of(&stored, "corp", directory("other", "")).is_err());
}

#[test]
fn directories_are_added_replaced_and_removed_by_id() {
    let list = with_directory(&[directory("corp", "a")], directory("lab", "b"));
    assert_eq!(list.len(), 2);
    let list = with_directory(&list, directory("Corp", "c"));
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].bind_password, "c");
    assert_eq!(without_directory(&list, " LAB ").unwrap().len(), 1);
    assert!(without_directory(&list, "none").is_none());
}
