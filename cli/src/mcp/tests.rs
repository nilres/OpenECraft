use super::*;
use crate::testing::session;

fn request(id: u64, method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
}

fn call_tool(s: &mut Session, name: &str, args: Value) -> Value {
    handle(s, &request(9, "tools/call", json!({ "name": name, "arguments": args }))).unwrap()["result"].clone()
}

#[test]
fn initialize_negotiates_the_protocol_version() {
    let mut s = session();
    let r = handle(&mut s, &request(1, "initialize", json!({ "protocolVersion": "2025-03-26", "capabilities": {} }))).unwrap();
    assert_eq!(r["id"], 1);
    assert_eq!(r["result"]["protocolVersion"], "2025-03-26");
    assert!(r["result"]["capabilities"]["tools"].is_object());
    assert_eq!(r["result"]["serverInfo"]["name"], "openecraft");
    let r = handle(&mut s, &request(2, "initialize", json!({ "protocolVersion": "1999-01-01" }))).unwrap();
    assert_eq!(r["result"]["protocolVersion"], PROTOCOL_VERSION);
}

#[test]
fn lists_every_tool_with_its_schema() {
    let mut s = session();
    let r = handle(&mut s, &request(1, "tools/list", json!({}))).unwrap();
    let listed = r["result"]["tools"].as_array().unwrap();
    assert_eq!(listed.len(), tools::entries(&s).len());
    assert!(listed.len() > tools::all().count(), "commands of the built-in plugins");
    let connect = listed.iter().find(|t| t["name"] == "connect").unwrap();
    assert!(connect["description"].as_str().unwrap().contains("connection points"));
    assert_eq!(connect["inputSchema"]["type"], "object");
}

#[test]
fn tool_calls_return_text_and_tool_errors() {
    let mut s = session();
    let r = call_tool(&mut s, "new_project", json!({ "demo": true }));
    assert_eq!(r["isError"], false);
    let info: Value = serde_json::from_str(r["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(info["errors"], 0);

    let svg = call_tool(&mut s, "export", json!({ "format": "svg", "page": "1" }));
    assert!(svg["content"][0]["text"].as_str().unwrap().starts_with("<"));

    let r = call_tool(&mut s, "get_page", json!({ "page": "nope" }));
    assert_eq!(r["isError"], true);
    assert_eq!(r["content"][0]["text"], "No page named nope");
}

#[test]
fn protocol_errors() {
    let mut s = session();
    let r = handle(&mut s, &request(1, "resources/list", json!({}))).unwrap();
    assert_eq!(r["error"]["code"], -32601);
    let r = handle(&mut s, &request(2, "tools/call", json!({ "name": "fly" }))).unwrap();
    assert_eq!(r["error"]["code"], -32602);
    let r = handle(&mut s, &request(3, "tools/call", json!({ "name": "list_pages", "arguments": [] }))).unwrap();
    assert_eq!(r["error"]["code"], -32602);
    assert_eq!(handle(&mut s, &json!({ "jsonrpc": "2.0", "id": 4 })).unwrap()["error"]["code"], -32600);
}

#[test]
fn notifications_and_responses_get_no_answer() {
    let mut s = session();
    assert!(handle(&mut s, &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })).is_none());
    assert!(handle(&mut s, &json!({ "jsonrpc": "2.0", "id": 5, "result": {} })).is_none());
}

#[test]
fn serves_one_message_per_line() {
    let mut s = session();
    let input = [
        request(1, "initialize", json!({ "protocolVersion": PROTOCOL_VERSION })).to_string(),
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }).to_string(),
        String::new(),
        "{ not json".into(),
        request(2, "tools/call", json!({ "name": "new_project", "arguments": {} })).to_string(),
        request(3, "tools/call", json!({ "name": "list_pages" })).to_string(),
    ]
    .join("\n");
    let mut out = Vec::new();
    serve(&mut s, input.as_bytes(), &mut out).unwrap();
    let replies: Vec<Value> = String::from_utf8(out).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(replies.len(), 4);
    assert_eq!(replies[1]["error"]["code"], -32700);
    assert_eq!(replies[3]["id"], 3);
    assert_eq!(replies[3]["result"]["isError"], false, "the project stays open between calls");
}

#[test]
fn the_offered_tools_can_be_limited() {
    let mut s = session();
    s.mcp_tools = vec!["new_project".into(), "parts_*".into()];
    let r = handle(&mut s, &request(1, "tools/list", json!({}))).unwrap();
    let names: Vec<&str> = r["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"new_project") && names.contains(&"parts_search"));
    assert!(names.iter().all(|n| *n == "new_project" || n.starts_with("parts_")), "{names:?}");
    let r = handle(&mut s, &request(2, "tools/call", json!({ "name": "check", "arguments": {} }))).unwrap();
    assert_eq!(r["error"]["code"], -32602);
}

#[test]
fn files_stay_in_the_root_folder() {
    let root = std::env::temp_dir().join(format!("oec-mcp-root-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut s = session();
    s.mcp_root = Some(root.clone());
    // Built-in templates are names, not files.
    assert_eq!(call_tool(&mut s, "new_project", json!({ "template": "Machine" }))["isError"], false);
    let inside = root.join("plan.oec");
    let r = call_tool(&mut s, "save_project", json!({ "path": inside.display().to_string() }));
    assert_eq!(r["isError"], false, "{r}");
    assert!(inside.exists());
    for outside in [std::env::temp_dir().join("oec-elsewhere.oec"), root.join("..").join("oec-escaped.oec")] {
        let r = call_tool(&mut s, "save_project", json!({ "path": outside.display().to_string() }));
        assert_eq!(r["isError"], true, "{}", outside.display());
        assert!(r["content"][0]["text"].as_str().unwrap().contains("OPENECRAFT_MCP_ROOT"));
        assert!(!outside.exists());
    }
    let r = call_tool(&mut s, "new_project", json!({ "template": std::env::temp_dir().join("t.oec").display().to_string() }));
    assert_eq!(r["isError"], true);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn listed_project_files_and_macro_libraries_stay_in_the_root_folder() {
    let root = crate::testing::temp_dir("mcp-lists");
    let elsewhere = crate::testing::temp_dir("mcp-lists-elsewhere");
    let mut s = session();
    s.mcp_root = Some(root.clone());
    let outside = elsewhere.join("other.oec");
    std::fs::write(&outside, openecraft_core::sample::demo_project().to_json()).unwrap();
    let before = std::fs::read(&outside).unwrap();
    let inside = root.join("plan.oec").display().to_string();
    let files = format!("{inside}; {}", outside.display());
    let r = call_tool(&mut s, "batch_project_properties", json!({ "files": files, "set": { "company": "Mallory" } }));
    assert_eq!(r["isError"], true, "{r}");
    assert!(r["content"][0]["text"].as_str().unwrap().contains("OPENECRAFT_MCP_ROOT"));
    assert_eq!(std::fs::read(&outside).unwrap(), before, "the file outside is not changed");
    call_tool(&mut s, "new_project", json!({}));
    let r = call_tool(&mut s, "generate_pages", json!({ "macros": outside.display().to_string() }));
    assert!(r["content"][0]["text"].as_str().unwrap().contains("OPENECRAFT_MCP_ROOT"), "{r}");
    // Folder ids of the vault are no files.
    assert!(confine(&root, "vault_browse", json!({ "folder": "17" }).as_object().unwrap()).is_ok());
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&elsewhere);
}

#[test]
fn every_file_parameter_is_confined() {
    // Parameters whose help speaks of files but that hold content, ids or names.
    const NO_FILES: [&str; 8] = ["csv", "text", "file", "id", "version", "kind", "name", "part"];
    let s = session();
    for tool in tools::entries(&s) {
        for p in tool.params().iter().filter(|p| p.kind == openecraft_api::Kind::Str) {
            let help = p.help.to_lowercase();
            let name = p.name.as_ref();
            let files = help.split(|c: char| !c.is_alphanumeric()).any(|w| matches!(w, "file" | "files" | "folder"));
            if !files || NO_FILES.contains(&name) || NOT_PATHS.contains(&(tool.name(), name)) {
                continue;
            }
            assert!(PATH_PARAMS.contains(&name) || PATH_LIST_PARAMS.contains(&name), "{} --{name} ({}) names a file the MCP server does not confine", tool.name(), p.help);
        }
    }
}
