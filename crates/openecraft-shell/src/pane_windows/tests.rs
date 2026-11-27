use super::*;
use crate::dock::{default_tree_for, LeftTab, ToolPane};
use crate::workspaces::Workspace;

/// Every pane is in one place: the main tree or one window.
fn assert_once(main: &Tree<Pane>, w: &PaneWindows) {
    let all: Vec<Pane> = dock::panes(main).into_iter().chain(w.panes()).collect();
    for p in &all {
        assert_eq!(all.iter().filter(|q| *q == p).count(), 1, "{p:?} twice");
    }
}

#[test]
fn a_pane_opens_in_a_window_of_its_own_and_leaves_the_main_window() {
    let mut main = default_tree_for(Workspace::Panel);
    let mut w = PaneWindows::default();
    let id = w.open(&mut main, Pane::LayoutView).unwrap();
    assert_eq!(id, 2, "the main window is window 1");
    assert_eq!(w.find(Pane::LayoutView), Some(id));
    assert!(dock::find(&main, Pane::LayoutView).is_none());
    assert_once(&main, &w);
    // Again: it has its window already.
    assert_eq!(w.open(&mut main, Pane::LayoutView), Some(id));
    assert_eq!(w.list.len(), 1);
    // A closed pane opens in a window too.
    let id3 = w.open(&mut main, Pane::Tool(ToolPane::Reports)).unwrap();
    assert_eq!(id3, 3);
    assert_once(&main, &w);
}

#[test]
fn the_page_and_the_last_document_stay_in_the_main_window() {
    let mut main = default_tree_for(Workspace::Schematic);
    let mut w = PaneWindows::default();
    assert_eq!(w.open(&mut main, Pane::Page), None);
    assert!(dock::find(&main, Pane::Page).is_some());
    let mut wiring = default_tree_for(Workspace::Wiring);
    assert_eq!(w.open(&mut wiring, Pane::WiringWorkbench), None, "the only document of the workspace");
    assert!(w.open(&mut wiring, Pane::WiringChecklist).is_some());
    assert_eq!(w.find(Pane::WiringWorkbench), None);
}

#[test]
fn panes_move_between_windows_and_back() {
    let mut main = default_tree_for(Workspace::Panel);
    let mut w = PaneWindows::default();
    let a = w.open(&mut main, Pane::LayoutView).unwrap();
    let b = w.open(&mut main, Pane::LayoutProperties).unwrap();
    assert!(w.move_to(&mut main, Pane::LayoutProperties, a));
    assert_eq!(w.find(Pane::LayoutProperties), Some(a));
    assert!(w.get(b).is_none(), "the window left empty closed");
    assert!(w.move_to(&mut main, Pane::Navigator(LeftTab::Devices), a), "from the main window into a window");
    assert!(!w.move_to(&mut main, Pane::Page, a));
    assert!(!w.move_to(&mut main, Pane::LayoutView, 99), "no such window");
    assert_once(&main, &w);

    w.back(&mut main, Pane::LayoutView);
    assert!(dock::find(&main, Pane::LayoutView).is_some());
    assert_eq!(w.find(Pane::LayoutView), None);
    assert_once(&main, &w);
}

#[test]
fn closing_a_window_puts_its_panes_back() {
    let mut main = default_tree_for(Workspace::Panel);
    let mut w = PaneWindows::default();
    let a = w.open(&mut main, Pane::LayoutView).unwrap();
    w.move_to(&mut main, Pane::LayoutProperties, a);
    w.close(&mut main, a);
    assert!(w.is_empty());
    for p in [Pane::LayoutView, Pane::LayoutProperties] {
        assert!(dock::find(&main, p).is_some(), "{p:?}");
    }
    assert!(dock::visible(&main).contains(&Pane::LayoutView), "shown as it was in the window");
    assert_once(&main, &w);

    w.open(&mut main, Pane::LayoutView);
    w.open(&mut main, Pane::LayoutProperties);
    w.close_all(&mut main);
    assert!(w.is_empty());
    assert_once(&main, &w);
}

#[test]
fn closing_the_last_pane_of_a_window_closes_it() {
    let mut main = default_tree_for(Workspace::Panel);
    let mut w = PaneWindows::default();
    w.open(&mut main, Pane::LayoutView);
    w.remove(Pane::LayoutView);
    assert!(w.is_empty());
    assert!(dock::find(&main, Pane::LayoutView).is_none(), "closed, not docked");
}

#[test]
fn saved_windows_are_read_and_tidied() {
    let mut main = default_tree_for(Workspace::Panel);
    let mut w = PaneWindows::default();
    let a = w.open(&mut main, Pane::LayoutView).unwrap();
    w.get_mut(a).unwrap().rect = Some([1920.0, 0.0, 1600.0, 900.0]);
    w.get_mut(a).unwrap().maximized = true;
    let read = PaneWindows::from_json(&w.to_json(), &main);
    assert_eq!(read, w);

    // A pane docked in the main window and the page are dropped from the windows.
    let mut broken = w.clone();
    dock::show(&mut broken.get_mut(a).unwrap().tree, Pane::Page);
    dock::show(&mut broken.get_mut(a).unwrap().tree, Pane::LayoutSpaces);
    broken.list.push(broken.list[0].clone());
    let tidy = PaneWindows::from_json(&broken.to_json(), &main);
    assert_eq!(tidy.list.len(), 1);
    assert_eq!(tidy.panes(), vec![Pane::LayoutView]);
    let other = default_tree_for(Workspace::Panel);
    assert!(PaneWindows::from_json(&w.to_json(), &other).is_empty(), "the view is docked in that tree");
    assert!(PaneWindows::from_json("not json", &main).is_empty());
}
