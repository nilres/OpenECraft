use super::*;
use openecraft_parts::portal::PartsRight;

#[test]
fn the_shared_scripts_use_known_texts_only() {
    let missing = unknown_texts(&[COMMON_SCRIPT, ACCESS_SCRIPT], TEXTS);
    assert!(missing.is_empty(), "texts of the shared scripts missing in TEXTS: {missing:?}");
    // Labels the scripts translate by name.
    for label in ["Identifier", "Name", "Type", "Address", "Encryption", "Service account", "Password", "Search base", "Groups", "Test", "Timeout (s)"] {
        assert!(TEXTS.contains(&label), "{label}");
    }
}

#[test]
fn unknown_texts_are_found_once() {
    let page = r#"<b data-t="Known">x</b> t("New") tf("New") tp("Other {}") at("not a text")"#;
    assert_eq!(unknown_texts(&[page], &["Known"]), ["New", "Other {}"]);
}

#[test]
fn texts_hold_the_page_the_rights_and_the_roles() {
    let title = |name: &str| if name == "admin" { "Administrator" } else { "Reader" };
    let mut de = WebTexts::new::<PartsRight>(Lang::De, ["Favorites"], title);
    de.key("role", "Admin", "Administrator");
    assert_eq!(de.lang, "de");
    assert_eq!(de.texts["Favorites"], translate("Favorites", Lang::De));
    assert_eq!(de.texts["Sign in"], translate("Sign in", Lang::De));
    assert_eq!(de.keys["role:Admin"], translate("Administrator", Lang::De));
    assert_eq!(de.languages.len(), Lang::ALL.len());
    assert_eq!(de.rights.len(), PartsRight::ALL.len());
    assert_eq!(de.roles.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(), PartsRight::ROLES);
    assert_eq!(de.roles[0].rights, ["read"]);
    assert_eq!(de.roles.last().unwrap().title, translate("Administrator", Lang::De));
}
