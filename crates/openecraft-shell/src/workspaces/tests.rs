use super::*;

#[test]
fn layouts_are_saved_by_name_and_replaced() {
    let mut list = Vec::new();
    assert!(save(&mut list, Layout { name: " PLC work ".into(), windows: vec!["plc".into()], ..Default::default() }));
    assert!(!save(&mut list, Layout { name: "  ".into(), ..Default::default() }));
    assert!(save(&mut list, Layout { name: "PLC work".into(), windows: vec!["plc".into(), "reports".into()], ..Default::default() }));
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].windows.len(), 2);
    remove(&mut list, "PLC work");
    assert!(list.is_empty());
}

#[test]
fn windows_of_earlier_workspaces_become_panes() {
    use crate::dock::ToolPane;
    let layout = Layout { windows: vec!["reports".into(), "check_settings".into(), "symbol_editor".into(), "gone".into()], ..Default::default() };
    assert_eq!(window_panes(&layout), vec![ToolPane::Reports, ToolPane::SymbolEditor]);
}

#[test]
fn workspace_keys_round_trip() {
    for w in Workspace::ALL {
        assert_eq!(Workspace::from_key(w.key()), Some(w));
    }
    assert_eq!(Workspace::from_key("cockpit"), None);
    assert!(Workspace::Mounting.is_workshop() && !Workspace::Panel.is_workshop());
}

#[test]
fn layouts_of_the_earlier_editor_are_read() {
    let old = r#"{"name":"Mine","workspace":"panel","left_tab":"devices","windows":["reports"]}"#;
    let l: Layout = serde_json::from_str(old).unwrap();
    assert_eq!(l.workspace, "panel");
    assert!(l.dock.is_empty() && l.ribbon.is_empty());
}
