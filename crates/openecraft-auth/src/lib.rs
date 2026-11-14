//! Access control of the OpenECraft servers (parts server, project server): password
//! hashes (Argon2), random session tokens and the lists of session token hashes an
//! account keeps, sign-in with directories (Active Directory, LDAP) and the access
//! settings file (directories, roles, grants) with the administration of its directories
//! (`admin`) and the configuration files of the servers
//! (`server_config`), the throttling of failed sign-ins (`Throttle`) and the error
//! answers as JSON (`json_errors`). The rules of roles and grants are in
//! `openecraft_parts::access`, shared with the clients. Native only (the servers); the
//! clients just send tokens.

mod account;
pub mod admin;
mod config;
mod directory;
mod errors;
mod headers;
pub mod server_config;
pub mod throttle;

pub use account::*;
pub use config::{write_private, AccessConfig, AccessFile};
pub use directory::{common_name, sign_in, Directory, DirectoryKind, SignInError};
pub use errors::{error_text, json_errors};
pub use headers::{security_headers, ClientIp, DEFAULT_CSP};
pub use throttle::Throttle;
