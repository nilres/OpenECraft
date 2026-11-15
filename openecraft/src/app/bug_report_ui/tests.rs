use super::*;
use crate::style::Level;

fn facts() -> ProjectFacts {
    ProjectFacts { pages: 12, page: 6, page_kind: "Schematic".into(), devices: 40, connections: 95, errors: 1, warnings: 2, notes: 3, unsaved: true, shared: false }
}

#[test]
fn details_give_counts_of_the_project_and_the_shell_but_no_contents() {
    let shell = [("Workspace", "Schematic".to_string()), ("Tool", "Wire".to_string())];
    let text = details(&facts(), &shell, &["acme_export 1.2".into()], &[]);
    assert!(text.starts_with("- Workspace: Schematic\n- Tool: Wire\n- External plugins: acme_export 1.2\n"));
    assert!(text.contains("- Pages: 12 (current: 7 of 12, Schematic)\n- Devices: 40\n- Connections: 95\n"));
    assert!(text.contains("- Check: 1 errors, 2 warnings, 3 notes\n- Unsaved changes: yes\n- Shared on a project server: no\n"));
    assert!(text.ends_with("### Messages\nnone\n"));
    assert!(details(&facts(), &[], &[], &[]).contains("- External plugins: none\n"));
}

#[test]
fn details_list_the_last_messages_on_one_line_each_and_bounded() {
    let mut messages: Vec<(Level, String)> = (0..10).map(|i| (Level::Info, format!("message {i}"))).collect();
    messages.push((Level::Error, format!("Saving failed\n{}", "x".repeat(400))));
    let text = details(&facts(), &[], &[], &messages);
    assert!(text.contains("### Messages (last 8 of 11)\n- Info: message 3\n"));
    assert!(!text.contains("message 2\n"));
    let last = text.lines().last().unwrap();
    assert!(last.starts_with("- Error: Saving failed x"), "{last}");
    assert!(last.ends_with('…') && last.chars().count() < 320, "{last}");
}
