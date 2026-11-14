//! Command line arguments: `openecraft-cli [options] <command> [--<parameter> <value>]…`.
//! The options may stand anywhere; everything else is a parameter of the command.

use openecraft_core::params::{cli_words, CliWord};
use openecraft_i18n::tr_fmt;

#[derive(Debug, Default, PartialEq)]
pub struct Options {
    pub project: Option<String>,
    pub server: Option<String>,
    pub token: Option<String>,
    pub lang: Option<String>,
    /// Folder of external plugins.
    pub plugins: Option<String>,
    /// All parameters as one JSON object (`--json '{…}'`).
    pub json: Option<String>,
    pub help: bool,
    /// `--version` / `-V`.
    pub version: bool,
}

#[derive(Debug, Default, PartialEq)]
pub struct Invocation {
    pub command: Option<String>,
    /// Further words after the command (`help <command>`).
    pub words: Vec<String>,
    pub options: Options,
    /// `--name value` pairs in their order; `None` for a flag without value.
    pub params: Vec<(String, Option<String>)>,
}

pub fn parse(argv: &[String]) -> Result<Invocation, String> {
    let mut inv = Invocation::default();
    for word in cli_words(argv, &["help", "version"], &["p"]) {
        let (name, value) = match word {
            CliWord::Plain(arg) if arg == "-h" => ("help".to_string(), None),
            CliWord::Plain(arg) if arg == "-V" => ("version".to_string(), None),
            CliWord::Plain(arg) => {
                if inv.command.is_none() {
                    inv.command = Some(arg);
                } else {
                    inv.words.push(arg);
                }
                continue;
            }
            CliWord::Pair(name, value) => (name, value),
        };
        let option = match name.as_str() {
            "help" => {
                inv.options.help = true;
                continue;
            }
            "version" => {
                inv.options.version = true;
                continue;
            }
            "p" | "project" => &mut inv.options.project,
            "server" => &mut inv.options.server,
            "token" => &mut inv.options.token,
            "lang" => &mut inv.options.lang,
            "plugins" => &mut inv.options.plugins,
            "json" => &mut inv.options.json,
            _ => {
                inv.params.push((name.replace('-', "_"), value));
                continue;
            }
        };
        // `-p` is short for `--project`.
        let long = if name == "p" { "project" } else { name.as_str() };
        *option = Some(value.ok_or_else(|| tr_fmt!("Missing value for --{}", long))?);
    }
    Ok(inv)
}

#[cfg(test)]
mod tests;
