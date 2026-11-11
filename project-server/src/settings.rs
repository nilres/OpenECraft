//! Settings of the project server: the command line options (and their environment
//! variables) win over the configuration file (`--config`, TOML with the option names as
//! keys, see `project-server.example.toml`), the file over the defaults.

use clap::Parser;
use openecraft_auth::server_config;
use openecraft_core::collab::DEFAULT_PORT;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about = "Project server for OpenECraft: shared projects with page locks")]
pub struct Args {
    /// Configuration file (TOML) with the settings of the options below, e.g.
    /// `bind = "0.0.0.0:8091"`; options given on the command line win.
    #[arg(long, env = "OPENECRAFT_PROJECT_SERVER_CONFIG")]
    config: Option<PathBuf>,
    /// Address to listen on [default: 127.0.0.1:8091].
    #[arg(long)]
    bind: Option<SocketAddr>,
    /// Directory of the project files (created if missing) [default: projects].
    #[arg(long)]
    data: Option<PathBuf>,
    /// Access token required for every request.
    #[arg(long, env = "OPENECRAFT_PROJECT_TOKEN", hide_env_values = true)]
    token: Option<String>,
    /// Creates (or resets) an administrator account with the password from
    /// OPENECRAFT_PROJECT_ADMIN_PASSWORD.
    #[arg(long, env = "OPENECRAFT_PROJECT_ADMIN_USER")]
    admin_user: Option<String>,
    /// Access settings: directories (Active Directory, LDAP), roles and grants.
    /// Default: `access.json` in the data directory.
    #[arg(long)]
    access: Option<PathBuf>,
}

/// The configuration file: the options without `--config`.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct File {
    bind: Option<SocketAddr>,
    data: Option<PathBuf>,
    token: Option<String>,
    admin_user: Option<String>,
    access: Option<PathBuf>,
}

/// The settings the server runs with.
#[derive(Debug, PartialEq)]
pub struct Settings {
    pub bind: SocketAddr,
    pub data: PathBuf,
    /// Access token (not empty).
    pub token: Option<String>,
    /// Administrator to create (not empty).
    pub admin_user: Option<String>,
    pub access: PathBuf,
}

impl Settings {
    /// The settings of the command line and of its configuration file.
    pub fn new(args: Args) -> Result<Settings, String> {
        let file = match &args.config {
            Some(path) => server_config::load(path)?,
            None => File::default(),
        };
        Ok(Settings::merge(args, file))
    }

    fn merge(args: Args, file: File) -> Settings {
        let in_file = |p: PathBuf| match &args.config {
            Some(config) => server_config::relative_to(config, p),
            None => p,
        };
        let data = args.data.or(file.data.map(in_file)).unwrap_or_else(|| PathBuf::from("projects"));
        let access = args.access.or(file.access.map(in_file)).unwrap_or_else(|| data.join("access.json"));
        Settings {
            bind: args.bind.or(file.bind).unwrap_or(SocketAddr::from(([127, 0, 0, 1], DEFAULT_PORT))),
            token: args.token.or(file.token).filter(|t| !t.trim().is_empty()),
            admin_user: args.admin_user.or(file.admin_user).filter(|n| !n.trim().is_empty()),
            data,
            access,
        }
    }
}

#[cfg(test)]
mod tests;
