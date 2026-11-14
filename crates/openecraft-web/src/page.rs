//! The shared scripts, styles and texts of the servers' web pages and their routes.

use axum::http::header;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use openecraft_i18n::{tr_const, translate, Lang};
use openecraft_parts::access::Right;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const COMMON_SCRIPT: &str = include_str!("web/common.js");
pub const ACCESS_SCRIPT: &str = include_str!("web/access.js");
pub const STYLE: &str = include_str!("web/web.css");
/// The OpenECraft logo (the application icon).
pub const LOGO: &str = include_str!("../../../assets/icon.svg");

/// The pages run their own scripts only; data never becomes code.
pub const PAGE_CSP: &str =
    "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

/// Texts of the shared scripts (`{}`: placeholders filled in by the page).
pub const TEXTS: &[&str] = &[
    tr_const!("Language"),
    tr_const!("Signed in as {}"),
    tr_const!("Sign out"),
    tr_const!("Access token"),
    tr_const!("Sign in"),
    tr_const!("User account"),
    tr_const!("Sign in with a user account"),
    tr_const!("User name"),
    tr_const!("Password"),
    tr_const!("Repeat password"),
    tr_const!("Register"),
    tr_const!("Cancel"),
    tr_const!("Create an account on this server"),
    tr_const!("Sign in with the access token of the server"),
    tr_const!("Wrong access token"),
    tr_const!("A user name must not be empty or contain blanks"),
    tr_const!("The passwords do not match"),
    tr_const!("The password needs at least {} characters"),
    tr_const!("Your account was created. An administrator has to approve it before you can sign in."),
    tr_const!("This user name is taken"),
    tr_const!("The registration is closed"),
    tr_const!("Too many registrations from this address, try again later"),
    tr_const!("This user name cannot be used"),
    tr_const!("Your account awaits the approval of an administrator"),
    tr_const!("Wrong user name or password"),
    tr_const!("Too many failed sign-ins, try again later"),
    tr_const!("Loading…"),
    tr_const!("Without sign-in"),
    tr_const!("Roles"),
    tr_const!("Role"),
    tr_const!("Description"),
    tr_const!("Built-in"),
    tr_const!("Name"),
    tr_const!("Remove"),
    tr_const!("New role"),
    tr_const!("Grants"),
    tr_const!("Type"),
    tr_const!("User or group"),
    tr_const!("Deny"),
    tr_const!("User"),
    tr_const!("Group"),
    tr_const!("directory:group"),
    tr_const!("No grants: users have the rights of their role"),
    tr_const!("Add grant"),
    tr_const!("Save"),
    tr_const!("Discard changes"),
    tr_const!("Access settings saved"),
    tr_const!("The access settings would leave nobody who manages users"),
    tr_const!("A grant needs a user or a group"),
    tr_const!("Users sign in with the account of a directory (Active Directory, LDAP) when the server has no local account of their name. Grants refer to the groups of a directory as directory:group."),
    tr_const!("Directories"),
    tr_const!("New directory"),
    tr_const!("Address"),
    tr_const!("Search base"),
    tr_const!("Edit"),
    tr_const!("Delete"),
    tr_const!("Only local accounts"),
    tr_const!("Delete directory {}? Its users can no longer sign in."),
    tr_const!("Edit directory {}"),
    tr_const!("Short name without blanks; qualifies the groups (id:group)"),
    tr_const!("ldap:// or ldaps://, with the port if it is not the usual one"),
    tr_const!("Looks the users up; empty: users bind with their own name (below)"),
    tr_const!("unchanged if empty"),
    tr_const!("Better kept in an environment variable of the server"),
    tr_const!("Environment variable with the password of the service account"),
    tr_const!("Bind name of a user without service account; {user} is replaced"),
    tr_const!("Empty: the usual filter of the type"),
    tr_const!("Empty: the memberOf attribute of the user"),
    tr_const!("Empty: the search base"),
    tr_const!("separated by commas"),
    tr_const!("Only their members sign in; empty: all users"),
    tr_const!("Test"),
    tr_const!("Without a user: connects and binds with the service account. With a user: signs them in and shows their groups."),
    tr_const!("A directory id must not be empty or contain blanks or ':'"),
    tr_const!("The address must start with ldap:// or ldaps://"),
    tr_const!("The test succeeded"),
    tr_const!("The test failed"),
    tr_const!("Directory {} exists already. Change it?"),
    tr_const!("Saved directory {}"),
    tr_const!("Identifier"),
    tr_const!("Encryption"),
    tr_const!("Service account"),
    tr_const!("Password variable"),
    tr_const!("User bind"),
    tr_const!("User filter"),
    tr_const!("Name attribute"),
    tr_const!("Groups"),
    tr_const!("Group filter"),
    tr_const!("Group search base"),
    tr_const!("Allowed groups"),
    tr_const!("Timeout (s)"),
    tr_const!("StartTLS (for ldap://)"),
    tr_const!("Accept any certificate (test installations only)"),
    tr_const!("Also groups of groups (Active Directory)"),
];

/// A page with the policy of [`PAGE_CSP`].
pub fn page(html: &'static str) -> impl IntoResponse {
    ([(header::CONTENT_SECURITY_POLICY, PAGE_CSP)], Html(html))
}

/// A script of a page.
pub fn script(js: &'static str) -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/javascript; charset=utf-8")], js)
}

/// `/common.js`, `/access.js`, `/web.css` and `/logo.svg`.
pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/common.js", get(|| async { script(COMMON_SCRIPT) }))
        .route("/access.js", get(|| async { script(ACCESS_SCRIPT) }))
        .route("/web.css", get(|| async { ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], STYLE) }))
        .route("/logo.svg", get(|| async { ([(header::CONTENT_TYPE, "image/svg+xml"), (header::CACHE_CONTROL, "max-age=86400")], LOGO) }))
}

/// The query of `GET /api/v1/texts?lang=de`.
#[derive(Debug, Default, Deserialize)]
pub struct TextQuery {
    pub lang: Option<String>,
}

impl TextQuery {
    /// The language asked for, English if unknown.
    pub fn lang(&self) -> Lang {
        self.lang.as_deref().and_then(Lang::from_code).unwrap_or(Lang::En)
    }
}

/// Translations for a page: `texts` by English text, `keys` by `kind:Key` of the
/// enumerations the page shows (`lifecycle:Active`, `role:Reader`, …).
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WebTexts {
    pub lang: String,
    /// All languages: code and native name.
    pub languages: Vec<(String, String)>,
    pub texts: BTreeMap<String, String>,
    pub keys: BTreeMap<String, String>,
    /// The rights of the server in display order, for the administration.
    pub rights: Vec<RightInfo>,
    /// The built-in roles with their rights, lowest first.
    pub roles: Vec<RoleInfo>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RightInfo {
    /// Name as in JSON (`edit_parts`).
    pub name: String,
    pub title: String,
    /// Only given on the whole server (`*`).
    pub server_wide: bool,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RoleInfo {
    /// Name in grants (`reader`).
    pub name: String,
    pub title: String,
    pub rights: Vec<String>,
}

impl WebTexts {
    /// The page's `texts` and those of the shared scripts in `lang`, the rights of `R`
    /// and its built-in roles with their English titles (`role_title` by name).
    pub fn new<R: Right>(lang: Lang, texts: impl IntoIterator<Item = &'static str>, role_title: impl Fn(&str) -> &'static str) -> WebTexts {
        let texts = TEXTS.iter().copied().chain(texts).map(|t| (t.to_string(), translate(t, lang).to_string())).collect();
        let languages = Lang::ALL.iter().map(|l| (l.code().to_string(), l.native_name().to_string())).collect();
        let rights = R::ALL.iter().map(|r| RightInfo { name: r.name().into(), title: translate(r.title(), lang).into(), server_wide: r.server_wide() }).collect();
        let roles = R::ROLES
            .iter()
            .map(|name| RoleInfo {
                name: name.to_string(),
                title: translate(role_title(name), lang).into(),
                rights: R::role(name).unwrap_or_default().iter().map(|r| r.name().to_string()).collect(),
            })
            .collect();
        WebTexts { lang: lang.code().into(), languages, texts, keys: BTreeMap::new(), rights, roles }
    }

    /// Adds the translated `title` of an enumeration value as `kind:key`.
    pub fn key(&mut self, kind: &str, key: impl std::fmt::Display, title: &'static str) {
        let lang = Lang::from_code(&self.lang).unwrap_or(Lang::En);
        self.keys.insert(format!("{kind}:{key}"), translate(title, lang).to_string());
    }
}

/// The texts a page shows through `t("…")`, `tf("…")`, `tp("…")`, `data-t="…"` and
/// `data-tp="…"` in `sources` that are not in `known` (for the tests of the pages).
pub fn unknown_texts(sources: &[&str], known: &[&str]) -> Vec<String> {
    let mut missing = Vec::new();
    for source in sources {
        for marker in ["t(\"", "tf(\"", "tp(\"", "data-t=\"", "data-tp=\""] {
            let mut rest = *source;
            while let Some(i) = rest.find(marker) {
                let before = rest[..i].chars().last();
                rest = &rest[i + marker.len()..];
                if before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '-') && !marker.starts_with("data") {
                    continue;
                }
                let end = rest.find('"').unwrap_or(rest.len());
                let text = &rest[..end];
                if !known.contains(&text) && !missing.iter().any(|m| m == text) {
                    missing.push(text.to_string());
                }
            }
        }
    }
    missing
}

#[cfg(test)]
mod tests;
