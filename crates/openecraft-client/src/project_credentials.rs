//! Opt-in project-server sign-ins. Only the OS credential store persists session
//! tokens; account metadata contains no password or token.

use openecraft_i18n::tr_fmt;
use url::{Host, Url};

#[cfg(windows)]
mod native {
    use super::{CredentialError, Operation};
    use crate::credentials::{self, Operation as SourceOperation};

    fn error(error: credentials::CredentialError) -> CredentialError {
        let operation = match error.operation {
            SourceOperation::Read => Operation::Read,
            SourceOperation::Save => Operation::Save,
            SourceOperation::Forget => Operation::Forget,
        };
        CredentialError { operation, code: error.code }
    }

    pub fn load(target: &str) -> Result<Option<String>, CredentialError> {
        credentials::native::load(target).map_err(error)
    }
    pub fn save(target: &str, token: &str, user: &str) -> Result<bool, CredentialError> {
        credentials::native::save(target, token, user).map_err(error)
    }
    pub fn forget(target: &str) -> Result<bool, CredentialError> {
        credentials::native::forget(target).map_err(error)
    }
    pub fn forget_if(target: &str, expected: &str) -> Result<bool, CredentialError> {
        credentials::native::forget_if(target, expected).map_err(error)
    }
}

#[cfg(not(windows))]
mod native {
    use super::CredentialError;
    pub fn load(_: &str) -> Result<Option<String>, CredentialError> { Ok(None) }
    pub fn save(_: &str, _: &str, _: &str) -> Result<bool, CredentialError> { Ok(false) }
    pub fn forget(_: &str) -> Result<bool, CredentialError> { Ok(false) }
    pub fn forget_if(_: &str, _: &str) -> Result<bool, CredentialError> { Ok(false) }
}

/// A server and account binding, never a credential. Account names use ASCII
/// lowercase to match the project server's case-insensitive account lookup.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Account {
    server: String,
    user: String,
}

impl Account {
    /// HTTP and HTTPS retain separate credential bindings.
    /// Reject URL credentials, query strings and fragments rather than silently
    /// dropping them when constructing the credential-store key.
    pub fn new(server: &str, user: &str) -> Option<Self> {
        let server = server.trim();
        let user = user.trim();
        if user.is_empty() || user.chars().any(char::is_control)
            || server.chars().any(char::is_control) || server.contains('\\') {
            return None;
        }
        let (_, rest) = server.split_once("://")?;
        if rest.split(['/', '?', '#']).next()?.contains('@') { return None; }
        let parsed = Url::parse(server).ok()?;
        if !parsed.username().is_empty() || parsed.password().is_some()
            || parsed.query().is_some() || parsed.fragment().is_some() {
            return None;
        }
        let host = parsed.host()?;
        if !matches!(parsed.scheme(), "http" | "https") { return None; }
        let port = parsed.port_or_known_default()?;
        let host = match host {
            Host::Ipv6(address) => format!("[{address}]"),
            _ => host.to_string(),
        };
        let path = parsed.path().trim_end_matches('/');
        Some(Self { server: format!("{}://{host}:{port}{path}", parsed.scheme()), user: user.to_ascii_lowercase() })
    }

    pub fn server(&self) -> &str { &self.server }
    pub fn user(&self) -> &str { &self.user }

    fn target(&self) -> String {
        // Windows target names ignore case; hash the complete binding so that
        // distinct case-sensitive server paths cannot share a credential.
        let binding = serde_json::to_vec(&(self.server(), self.user())).expect("account binding");
        format!("OpenECraft/ProjectServer/Session/v1/{}", crate::update::sha256(&binding))
    }
}

/// Only an operation and a numeric system code leave the native backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CredentialError {
    pub operation: Operation,
    pub code: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation { Read, Save, Forget }

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self.operation {
            Operation::Read => tr_fmt!("Could not read the saved project-server sign-in (system error {})", self.code),
            Operation::Save => tr_fmt!("Could not save the project-server sign-in (system error {})", self.code),
            Operation::Forget => tr_fmt!("Could not remove the saved project-server sign-in (system error {})", self.code),
        };
        f.write_str(&message)
    }
}

impl std::error::Error for CredentialError {}

/// Tests can use an isolated store without reading the user's credentials.
pub trait CredentialStore {
    fn load(&self, account: &Account) -> Result<Option<String>, CredentialError>;
    fn save(&self, account: &Account, token: &str) -> Result<bool, CredentialError>;
    fn forget(&self, account: &Account) -> Result<bool, CredentialError>;
    fn forget_if(&self, account: &Account, expected: &str) -> Result<bool, CredentialError> {
        if self.load(account)?.as_deref() == Some(expected) { self.forget(account) } else { Ok(false) }
    }
}

/// Windows Credential Manager, current OS user on this computer. Other desktops
/// and the browser are session-only, with no plaintext persistence fallback.
pub struct SystemCredentialStore;

impl SystemCredentialStore {
    pub fn available_for(server: &str, user: &str) -> bool {
        cfg!(windows) && Account::new(server, user).is_some()
    }
}

impl CredentialStore for SystemCredentialStore {
    fn load(&self, account: &Account) -> Result<Option<String>, CredentialError> {
        native::load(&account.target()).map(|token| token.filter(|token| valid_token(token)))
    }

    fn save(&self, account: &Account, token: &str) -> Result<bool, CredentialError> {
        if !valid_token(token) { return Ok(false); }
        native::save(&account.target(), token, account.user())
    }

    fn forget(&self, account: &Account) -> Result<bool, CredentialError> {
        native::forget(&account.target())
    }

    fn forget_if(&self, account: &Account, expected: &str) -> Result<bool, CredentialError> {
        native::forget_if(&account.target(), expected)
    }
}

fn valid_token(token: &str) -> bool {
    !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_graphic())
}

#[cfg(test)]
mod tests;
