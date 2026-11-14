//! Administration of the directories of the access settings, shared by the servers
//! (`/api/v1/directories`): a directory as administrators see it (without the password
//! of its service account), the directory of a change (an empty password keeps the
//! stored one) and the test of a directory's settings.

use crate::directory::{Directory, SignInError};
use openecraft_parts::api::DirectoryTest;
use serde::{Deserialize, Serialize};

/// A directory as administrators see it: without the password of its service account.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DirectoryEntry {
    #[serde(flatten)]
    pub directory: Directory,
    /// The password of the service account is set (in the file or its variable).
    pub password_set: bool,
}

impl DirectoryEntry {
    pub fn of(d: &Directory) -> DirectoryEntry {
        DirectoryEntry { password_set: d.has_bind_password(), directory: Directory { bind_password: String::new(), ..d.clone() } }
    }
}

/// The directory of a request to the address of `id`, checked against it; an empty
/// password is the one stored with the directory in `stored`.
pub fn directory_of(stored: &[Directory], id: &str, mut d: Directory) -> Result<Directory, String> {
    if d.id.trim().is_empty() {
        d.id = id.to_string();
    }
    if !d.id.trim().eq_ignore_ascii_case(id.trim()) {
        return Err("directory id in URL and body differ".into());
    }
    d.id = d.id.trim().to_string();
    if d.bind_password.is_empty() {
        if let Some(old) = stored.iter().find(|o| o.id.trim().eq_ignore_ascii_case(&d.id)) {
            d.bind_password = old.bind_password.clone();
        }
    }
    Ok(d)
}

/// The directories with `d` added or replacing the one of its id.
pub fn with_directory(list: &[Directory], d: Directory) -> Vec<Directory> {
    let mut list = list.to_vec();
    match list.iter_mut().find(|o| o.id.trim().eq_ignore_ascii_case(&d.id)) {
        Some(old) => *old = d,
        None => list.push(d),
    }
    list
}

/// The directories without the one of `id`; `None` if there is none.
pub fn without_directory(list: &[Directory], id: &str) -> Option<Vec<Directory>> {
    let rest: Vec<Directory> = list.iter().filter(|d| !d.id.trim().eq_ignore_ascii_case(id.trim())).cloned().collect();
    (rest.len() < list.len()).then_some(rest)
}

/// Settings of a directory to test and, optionally, a user to sign in with.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct DirectoryTestRequest {
    pub directory: Directory,
    pub name: String,
    pub password: String,
}

/// Tests the settings of a directory (saved or not): connects and binds with the
/// service account or, with a user, signs them in and reads their groups.
pub async fn test_directory(d: &Directory, name: &str, password: &str) -> DirectoryTest {
    let result = if name.trim().is_empty() {
        d.test_connection().await.map(|message| (message, None))
    } else {
        d.authenticate(name, password).await.map(|i| (format!("signed in as {} with {} groups", i.name, i.groups.len()), Some(i)))
    };
    match result {
        Ok((message, identity)) => DirectoryTest { ok: true, message, identity },
        Err(e) => {
            let message = match e {
                SignInError::Unknown => "the directory does not know the user".into(),
                SignInError::Refused => "wrong password, or the user may not sign in".into(),
                SignInError::Unavailable(e) => e,
            };
            DirectoryTest { ok: false, message, identity: None }
        }
    }
}

#[cfg(test)]
mod tests;
