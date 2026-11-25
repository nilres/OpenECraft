//! Model Context Protocol server: JSON-RPC 2.0, one message per line on stdin /
//! stdout. Offers the core tools of [`crate::tools`] for schematics and reaches the
//! others through `find_tools`, `describe_tool` and `call_tool` (all of them with
//! `OPENECRAFT_MCP_TOOLS=*`); the session (open project, undo history, parts server)
//! lives as long as the connection. Texts for the agent are English like the rest of
//! the protocol.

use crate::session::Session;
use crate::tools;
use openecraft_api::params::{optional, param, Kind, Param};
use openecraft_api::Effect;
use serde_json::{json, Map, Value};
use std::io::{BufRead, Write};
use std::path::{Component, Path, PathBuf};

pub const PROTOCOL_VERSION: &str = "2025-06-18";
const SUPPORTED: [&str; 3] = [PROTOCOL_VERSION, "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "OpenECraft is an electrical CAE (schematic) editor. Open or create a project first \
(open_project, new_project); the tools change it in memory until save_project writes it, undo / redo revert single \
tool calls. Pages are A3 (420 x 297 mm, y downwards, 10 columns, 6 rows); positions snap to a 4 mm grid. Use \
list_symbols for symbol keys and connection points, get_page for positions and item ids, connect to draw wires \
between connection points (TAG:PIN, e.g. -K1:A1), check to find errors and report / export for the results. Device \
tags follow IEC 81346 (-K1 contactor, -S1 switch, -X1 terminal strip). Cables: add_cable_definition draws a line \
across wires, which become cores of the cable it names (created when new); apply_cable_part takes type and cores \
from a cable part, set_cable sets length and description. Parts come from the project, the built-in catalog or the \
parts server (parts_search, parts_get; without a server they search the project and the built-in catalog).";

/// The tools offered by default (`OPENECRAFT_MCP_TOOLS` not set or `core`): what an
/// agent needs to draw, check and export schematics. The others stay reachable
/// through the meta tools.
pub const CORE: &[&str] = &[
    "new_project", "list_project_templates", "template_values", "open_project", "save_project", "convert_project", "list_examples", "project_info", "set_project_info", "undo", "redo",
    "list_pages", "get_page", "add_page", "set_page", "renumber_pages",
    "list_symbols", "place_symbol", "place_part", "update_symbol", "move_items", "delete_items",
    "add_wire", "connect", "route", "update_wire", "add_text", "list_potentials", "set_wire_colors", "set_wire_numbering", "renumber_wires",
    "list_devices", "assign_part", "parts_search", "parts_get",
    "list_cables", "add_cable", "set_cable", "rename_cable", "list_cable_parts", "add_cable_definition", "apply_cable_part",
    "table_get", "table_set", "check", "list_reports", "report", "list_exports", "export",
];

/// Tools of the MCP server itself that reach the tools not offered.
const META: [(&str, &str, &[Param], Effect); 3] = [
    (
        "find_tools",
        "Finds the tools beyond those offered (panel layout in 3D, PLC, terminals, harnesses, fluid power, buildings, \
parts server, imports, revisions and more): without a query the names of all tools, with one the tools whose name or \
description has every word, with their description",
        &[optional("query", Kind::Str, "Words in the name or description of the tool")],
        Effect::Read,
    ),
    ("describe_tool", "Shows the description and the parameters (JSON schema) of any tool", &[param("name", Kind::Str, "Name of the tool")], Effect::Read),
    (
        "call_tool",
        "Calls any tool by its name, also one not offered in the list of tools; the result is that of the tool",
        &[param("name", Kind::Str, "Name of the tool"), optional("arguments", Kind::Object, "Parameters of the tool")],
        Effect::Change,
    ),
];

/// Names of the tools to offer, from `OPENECRAFT_MCP_TOOLS` (comma separated; `parts_*`
/// for all tools starting with `parts_`, `core` for [`CORE`], `*` for all). Without it
/// the core tools: the list of all of them is too long for the context of an agent.
pub fn tools_from_env() -> Vec<String> {
    let names: Vec<String> = std::env::var("OPENECRAFT_MCP_TOOLS").unwrap_or_default().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    if names.is_empty() { vec!["core".into()] } else { names }
}

/// Folder the files of the MCP tools must be in, from `OPENECRAFT_MCP_ROOT` (`*`:
/// anywhere); default the working folder. An agent misled by text it reads cannot
/// read or overwrite files elsewhere (other projects, settings, plugin folders).
pub fn root_from_env() -> Option<PathBuf> {
    match std::env::var("OPENECRAFT_MCP_ROOT").unwrap_or_default().trim() {
        "*" => None,
        "" => std::env::current_dir().ok(),
        dir => Some(PathBuf::from(dir)),
    }
}

/// Parameters of the tools that name files or folders.
const PATH_PARAMS: [&str; 11] = ["path", "output", "folder", "database", "target", "template", "macros", "logo", "base", "ours", "theirs"];

/// Parameters of the tools that name several files, separated by `;`.
const PATH_LIST_PARAMS: [&str; 1] = ["files"];

/// Parameters that are ids of the vault or a format, not files (tool, parameter).
const NOT_PATHS: [(&str, &str); 3] = [("vault_browse", "folder"), ("vault_upload", "folder"), ("convert_project", "format")];

/// A path as people write it: without the `\\?\` prefix of a canonical Windows path.
fn shown(path: &Path) -> String {
    let text = path.display().to_string();
    match text.strip_prefix(r"\\?\") {
        Some(rest) => rest.strip_prefix(r"UNC\").map_or_else(|| rest.to_string(), |unc| format!(r"\\{unc}")),
        None => text,
    }
}

/// The values of the file and folder parameters of a call of the tool `tool`, each
/// with its parameter name.
fn path_values<'a>(tool: &str, args: &'a Map<String, Value>) -> Vec<(&'a str, &'a str)> {
    let named = args.iter().filter(|(name, _)| !NOT_PATHS.contains(&(tool, name.as_str())));
    named
        .filter_map(|(name, value)| Some((name.as_str(), value.as_str()?)))
        .flat_map(|(name, value)| {
            let list = PATH_LIST_PARAMS.contains(&name);
            let values: Vec<&str> = if list { value.split(';').collect() } else if PATH_PARAMS.contains(&name) { vec![value] } else { Vec::new() };
            values.into_iter().map(move |v| (name, v.trim()))
        })
        // Built-in templates are names, not files.
        .filter(|(name, path)| !path.is_empty() && (*name != "template" || path.contains(['/', '\\']) || [".oec", ".oecp", ".oecz"].iter().any(|e| path.to_lowercase().ends_with(e))))
        .collect()
}

/// Relative file and folder parameters of a call are relative to `root`, not to the
/// folder the client started the server in (Claude Desktop and others choose it).
fn anchor(root: &Path, tool: &str, args: &mut Map<String, Value>) {
    let relative: Vec<(String, String)> = path_values(tool, args).into_iter().filter(|(_, p)| Path::new(p).is_relative()).map(|(n, p)| (n.to_string(), p.to_string())).collect();
    for (name, path) in relative {
        let Some(Value::String(value)) = args.get_mut(&name) else { continue };
        let full = root.join(&path).display().to_string();
        *value = if PATH_LIST_PARAMS.contains(&name.as_str()) {
            value.split(';').map(|v| if v.trim() == path { full.clone() } else { v.trim().to_string() }).collect::<Vec<_>>().join("; ")
        } else {
            full
        };
    }
}

/// The absolute path without `.` and `..`, its existing part with symbolic links
/// resolved.
fn resolve(path: &Path) -> PathBuf {
    let full = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir().unwrap_or_default().join(path) };
    let mut out = PathBuf::new();
    for c in full.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            c => out.push(c),
        }
    }
    let mut missing = Vec::new();
    let mut existing = out.as_path();
    while !existing.exists() {
        let (Some(name), Some(parent)) = (existing.file_name(), existing.parent()) else { break };
        missing.push(name.to_owned());
        existing = parent;
    }
    let mut resolved = existing.canonicalize().unwrap_or_else(|_| existing.to_path_buf());
    resolved.extend(missing.iter().rev());
    resolved
}

/// The file and folder parameters of a call of the tool `tool` lie in `root`.
fn confine(root: &Path, tool: &str, args: &Map<String, Value>) -> Result<(), String> {
    let root = resolve(root);
    for (_, path) in path_values(tool, args) {
        if !resolve(Path::new(path)).starts_with(&root) {
            return Err(format!("{} is outside {}, the folder this MCP server may use (OPENECRAFT_MCP_ROOT)", shown(Path::new(path)), shown(&root)));
        }
    }
    Ok(())
}

/// True if the MCP server offers the tool `name`.
fn offered(session: &Session, name: &str) -> bool {
    session.mcp_tools.is_empty()
        || session.mcp_tools.iter().any(|p| match p.strip_suffix('*') {
            Some(prefix) => name.starts_with(prefix),
            None if p == "core" => CORE.contains(&name),
            None => p == name,
        })
}

/// True if the MCP server offers every tool, so that it needs no meta tools.
fn offers_all(session: &Session) -> bool {
    session.mcp_tools.is_empty() || session.mcp_tools.iter().any(|p| p == "*")
}

pub fn serve(session: &mut Session, input: impl BufRead, mut output: impl Write) -> Result<(), String> {
    for line in input.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(msg) => handle(session, &msg),
            Err(e) => Some(error(Value::Null, -32700, &format!("parse error: {e}"))),
        };
        if let Some(reply) = reply {
            writeln!(output, "{reply}").and_then(|_| output.flush()).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Answer to one message; `None` for notifications and responses.
pub fn handle(session: &mut Session, msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned();
    let Some(method) = msg.get("method").and_then(Value::as_str) else {
        // The server sends no requests, so responses of the client need no answer.
        let response = msg.get("result").is_some() || msg.get("error").is_some();
        return (!response).then(|| error(id.unwrap_or(Value::Null), -32600, "invalid request"));
    };
    let id = id?;
    let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
    let result = match method {
        "initialize" => Ok(initialize(session, &params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": list(session) })),
        "tools/call" => call(session, &params, false),
        _ => Err((-32601, format!("method not found: {method}"))),
    };
    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err((code, message)) => error(id, code, &message),
    })
}

fn initialize(session: &Session, params: &Value) -> Value {
    let asked = params.get("protocolVersion").and_then(Value::as_str).unwrap_or_default();
    let version = SUPPORTED.into_iter().find(|v| *v == asked).unwrap_or(PROTOCOL_VERSION);
    let mut instructions = INSTRUCTIONS.to_string();
    if let Some(root) = &session.mcp_root {
        instructions += &format!(" Files: relative paths are in {}; paths outside this folder are refused.", shown(&resolve(root)));
    }
    if !offers_all(session) {
        let all = tools::entries(session).len();
        instructions += &format!(" The list offers the tools for schematics; find_tools, describe_tool and call_tool reach all {all} tools.");
    }
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": "openecraft", "title": "OpenECraft", "version": env!("CARGO_PKG_VERSION") },
        "instructions": instructions,
    })
}

/// A tool as the list of tools has it. The title and the annotations tell the client
/// what to show and when to ask its user: `readOnlyHint` for a tool that changes
/// nothing, `destructiveHint` for one that may change or remove what is there,
/// `openWorldHint` for one that reaches a server or the program of a plugin.
fn tool(name: &str, help: &str, schema: Value, effect: Effect, open_world: bool) -> Value {
    let title = openecraft_api::title(name);
    json!({
        "name": name,
        "title": title,
        "description": help,
        "inputSchema": schema,
        "annotations": { "title": title, "readOnlyHint": effect == Effect::Read, "destructiveHint": effect == Effect::Change, "openWorldHint": open_world },
    })
}

fn describe(session: &Session, entry: &tools::Entry) -> Value {
    tool(entry.name(), entry.help(), tools::schema(entry.clone()), entry.effect(), entry.open_world(session))
}

/// The offered tools, followed by the meta tools when not all of them are offered.
fn list(session: &Session) -> Vec<Value> {
    let mut list: Vec<Value> = tools::entries(session).iter().filter(|t| offered(session, t.name())).map(|t| describe(session, t)).collect();
    if !offers_all(session) {
        // call_tool reaches every tool, also those that ask a server.
        list.extend(META.iter().map(|(name, help, params, effect)| tool(name, help, openecraft_api::params::schema(params), *effect, *name == "call_tool")));
    }
    list
}

/// The meta tools: other tools found, described or called.
fn meta(session: &mut Session, name: &str, args: &Map<String, Value>) -> Result<Value, String> {
    let text = |key: &str| args.get(key).and_then(Value::as_str).unwrap_or_default().trim().to_string();
    match name {
        "find_tools" => {
            let words: Vec<String> = text("query").split_whitespace().map(|w| w.to_lowercase().replace('_', " ")).collect();
            let entries = tools::entries(session);
            if words.is_empty() {
                let names: Vec<&str> = entries.iter().map(|t| t.name()).collect();
                return Ok(json!({ "total": names.len(), "tools": names }));
            }
            let hits: Vec<Value> = entries
                .iter()
                .filter(|t| {
                    let hay = format!("{} {}", t.name(), t.help()).to_lowercase().replace('_', " ");
                    words.iter().all(|w| hay.contains(w.as_str()))
                })
                .map(|t| json!({ "name": t.name(), "description": t.help(), "offered": offered(session, t.name()) }))
                .collect();
            Ok(json!({ "total": hits.len(), "tools": hits }))
        }
        "describe_tool" => {
            let tool = text("name");
            tools::lookup(session, &tool).map(|t| describe(session, &t)).ok_or_else(|| format!("unknown tool: {tool}"))
        }
        _ => {
            let params = json!({ "name": text("name"), "arguments": args.get("arguments").cloned().unwrap_or(Value::Null) });
            call(session, &params, true).map_err(|(_, message)| message)
        }
    }
}

/// A call of a tool; `any`: also of one not offered (`call_tool`).
fn call(session: &mut Session, params: &Value, any: bool) -> Result<Value, (i64, String)> {
    let name = params.get("name").and_then(Value::as_str).ok_or((-32602, "missing tool name".to_string()))?;
    let mut args = match params.get("arguments") {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(map)) => map.clone(),
        Some(_) => return Err((-32602, "arguments must be an object".into())),
    };
    if !any && !offers_all(session) && META.iter().any(|(m, ..)| *m == name) {
        return Ok(match meta(session, name, &args) {
            // call_tool answers with the result of the tool it called.
            Ok(result) if name == "call_tool" => result,
            Ok(value) => json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&value).unwrap_or_default() }], "isError": false }),
            Err(e) => json!({ "content": [{ "type": "text", "text": e }], "isError": true }),
        });
    }
    let tool = tools::lookup(session, name).filter(|_| any || offered(session, name)).ok_or_else(|| (-32602, format!("unknown tool: {name}")))?;
    if let Some(root) = session.mcp_root.clone() {
        anchor(&root, name, &mut args);
    }
    // Tool errors are results the agent can react to, not protocol errors.
    let allowed = session.mcp_root.as_deref().map_or(Ok(()), |root| confine(root, name, &args));
    let (text, is_error) = match allowed.and_then(|_| tools::call(session, tool, &args)) {
        Ok(Value::String(text)) => (text, false),
        Ok(mut value) => {
            tools::tidy(&mut value);
            (serde_json::to_string_pretty(&value).unwrap_or_default(), false)
        }
        Err(e) => (e, true),
    };
    Ok(json!({ "content": [{ "type": "text", "text": text }], "isError": is_error }))
}

#[cfg(test)]
mod tests;
