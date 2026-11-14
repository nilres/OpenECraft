use super::*;
use openecraft_i18n::translate;
use openecraft_parts::access::Right;

#[test]
fn texts_are_translated_with_the_rights_and_roles() {
    let de = web_texts(Lang::De);
    assert_eq!(de.lang, "de");
    assert_eq!(de.texts["Projects"], translate("Projects", Lang::De));
    assert_eq!(de.texts["Sign in"], translate("Sign in", Lang::De), "the texts of the shared scripts");
    assert_eq!(de.rights.len(), ProjectRight::ALL.len());
    assert_eq!(de.roles.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(), ProjectRight::ROLES);
    assert_eq!(de.roles[0].rights, ["view"]);
    assert_eq!(de.roles[3].title, translate("Project manager", Lang::De));
    assert!(web_texts(Lang::En).texts.iter().all(|(k, v)| k == v));
}

/// Every text the page shows through `t("…")` / `tf("…")` / `data-t` is in [`TEXTS`]
/// or in those of the shared scripts.
#[test]
fn the_page_uses_known_texts_only() {
    let known: Vec<&str> = TEXTS.iter().chain(openecraft_web::TEXTS).copied().collect();
    let missing = openecraft_web::unknown_texts(&[PAGE, SCRIPT, ADMIN_SCRIPT], &known);
    assert!(missing.is_empty(), "texts of the page missing in web.rs TEXTS: {missing:?}");
    // Titles and labels the scripts translate by name.
    for label in ["Sessions", "Open sessions", "Closed", "Expired", "Signed out", "User deleted", "New password", "Overview", "Users", "Access rights", "Directories", "Server", "Version", "Projects", "Defined roles", "Grants", "Without sign-in", "Description", "Company", "Created by", "Date", "Revision"] {
        assert!(known.contains(&label), "{label}");
    }
}
