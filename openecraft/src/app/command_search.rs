//! The command search (Ctrl+Shift+P, the box in the quick access row): every command of
//! the editor and of the plugins by its title, tooltip, id and place in the ribbon;
//! arrows choose, Enter runs, Esc or a click outside closes. The entries of the fixed menus (Align left,
//! New cover sheet, …) are found as well; a menu command opens its menu in a small
//! window.

use super::*;
use crate::commands::{self, Hit};
use crate::ribbon;
use openecraft_i18n::tr_const;

#[derive(Default)]
pub(super) struct SearchUi {
    open: bool,
    query: String,
    selected: usize,
    focus: bool,
    /// The menu command whose entries are shown in a window.
    pub menu: Option<Cmd>,
}

impl SearchUi {
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.focus = true;
    }

    /// The search or a menu opened from it is shown.
    pub fn is_open(&self) -> bool {
        self.open || self.menu.is_some()
    }
}

/// An entry of the search: a command of the editor or of a plugin.
#[derive(Clone)]
enum Entry {
    Cmd(Cmd),
    Plugin(openecraft_api::Command),
    /// An entry of the Align menu.
    Align(openecraft_core::arrange::Align),
    /// An entry of the Distribute menu (horizontally).
    Distribute(bool),
    /// An entry of the New page menu.
    NewPage(PageKind),
}

impl Entry {
    /// The command whose menu holds the entry.
    fn cmd(&self) -> Option<Cmd> {
        match self {
            Entry::Cmd(c) => Some(*c),
            Entry::Plugin(_) => None,
            Entry::Align(_) => Some(Cmd::Align),
            Entry::Distribute(_) => Some(Cmd::Distribute),
            Entry::NewPage(_) => Some(Cmd::NewPage),
        }
    }
}

/// The page kinds of the New page menu besides the schematic page (a command).
fn new_page_entries() -> [(PageKind, &'static str); 3] {
    [(PageKind::Cover, tr_const!("New cover sheet")), (PageKind::Overview, tr_const!("New overview page")), (PageKind::Graphic, tr_const!("New graphics page"))]
}

impl App {
    /// The entries with the texts they are found by (the title first).
    fn search_items(&self) -> Vec<(Entry, Vec<String>)> {
        let mut out: Vec<(Entry, Vec<String>)> = commands::ALL
            .iter()
            .map(|d| {
                let place = self.cmd_place(d.cmd);
                (Entry::Cmd(d.cmd), vec![tr(d.title).to_string(), tr(d.tip).to_string(), d.id.replace('_', " "), place, d.title.to_string()])
            })
            .collect();
        let sub = |cmd: Cmd, title: String| vec![title, tr(commands::def(cmd).title).to_string(), self.cmd_place(cmd)];
        for a in openecraft_core::arrange::Align::ALL {
            out.push((Entry::Align(a), sub(Cmd::Align, tr_fmt!("Align: {}", super::arrange_ui::align_title(a)))));
        }
        out.push((Entry::Distribute(true), sub(Cmd::Distribute, tr("Distribute horizontally").to_string())));
        out.push((Entry::Distribute(false), sub(Cmd::Distribute, tr("Distribute vertically").to_string())));
        for (kind, title) in new_page_entries() {
            out.push((Entry::NewPage(kind), sub(Cmd::NewPage, tr(title).to_string())));
        }
        for (plugin, cmds) in self.plugin_groups() {
            for c in cmds {
                let texts = vec![super::ribbon_ui::humanize(&c.name), text(&c.help).to_string(), plugin.clone(), c.name.clone()];
                out.push((Entry::Plugin(c), texts));
            }
        }
        out
    }

    /// Where a command is in the ribbon: `Insert › Graphics`.
    fn cmd_place(&self, cmd: Cmd) -> String {
        for t in ribbon::TABS {
            for g in t.groups {
                if g.items.iter().any(|i| i.cmd() == cmd) {
                    return format!("{} › {}", tr(t.title), tr(g.title));
                }
            }
        }
        if commands::def(cmd).pane.is_some() {
            return format!("{} › {}", tr("View"), tr("Panes"));
        }
        tr("File").to_string()
    }

    pub(super) fn command_search_window(&mut self, ctx: &egui::Context) {
        if !self.search.open {
            return;
        }
        let items = self.search_items();
        let hits: Vec<Hit<Entry>> = commands::rank(&self.search.query, &items);
        let hits: Vec<Hit<Entry>> = hits.into_iter().take(40).collect();
        // Taken before the text field sees them.
        let (up, down, enter, esc) = ctx.input_mut(|i| {
            let none = egui::Modifiers::NONE;
            (i.consume_key(none, Key::ArrowUp), i.consume_key(none, Key::ArrowDown), i.consume_key(none, Key::Enter), i.consume_key(none, Key::Escape))
        });
        if down {
            self.search.selected = (self.search.selected + 1).min(hits.len().saturating_sub(1));
        }
        if up {
            self.search.selected = self.search.selected.saturating_sub(1);
        }
        let mut run: Option<Entry> = None;
        let mut open = true;
        // The click that opened the search does not close it again.
        let just_opened = self.search.focus;
        let width = (ctx.content_rect().width() * 0.5).clamp(360.0, 640.0);
        let shown = egui::Window::new(tr("Find a command"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 80.0])
            .default_width(width)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    icons::show(ui, Icon::Find, 18.0);
                    let r = ui.add(egui::TextEdit::singleline(&mut self.search.query).hint_text(tr("Find a command…")).desired_width(f32::INFINITY));
                    if self.search.focus {
                        r.request_focus();
                        self.search.focus = false;
                    }
                    if r.changed() {
                        self.search.selected = 0;
                    }
                });
                ui.separator();
                ui.set_width(width);
                egui::ScrollArea::vertical().max_height(360.0).auto_shrink([false, true]).show(ui, |ui| {
                    for (i, h) in hits.iter().enumerate() {
                        let (icon, title, place, keys, enabled) = match &h.item {
                            Entry::Cmd(c) => {
                                let d = commands::def(*c);
                                (d.icon, tr(d.title).to_string(), self.cmd_place(*c), self.editor.shortcuts.cmd_text(*c), self.cmd_enabled(*c))
                            }
                            Entry::Plugin(c) => (Icon::Plugin, super::ribbon_ui::humanize(&c.name), tr("Plugins").to_string(), String::new(), true),
                            sub => {
                                let c = sub.cmd().unwrap_or(Cmd::Align);
                                let title = match sub {
                                    Entry::Align(a) => tr_fmt!("Align: {}", super::arrange_ui::align_title(*a)),
                                    Entry::Distribute(true) => tr("Distribute horizontally").to_string(),
                                    Entry::Distribute(false) => tr("Distribute vertically").to_string(),
                                    Entry::NewPage(k) => new_page_entries().iter().find(|(kind, _)| kind == k).map(|(_, t)| tr(t).to_string()).unwrap_or_default(),
                                    Entry::Cmd(_) | Entry::Plugin(_) => String::new(),
                                };
                                (commands::def(c).icon, title, self.cmd_place(c), String::new(), self.cmd_enabled(c))
                            }
                        };
                        let selected = i == self.search.selected;
                        let r = ui.add_enabled_ui(enabled, |ui| {
                            ui.horizontal(|ui| {
                                icons::show(ui, icon, 16.0);
                                let r = ui.selectable_label(selected, &title);
                                ui.weak(place);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.monospace(keys);
                                });
                                r
                            })
                            .inner
                        });
                        if selected {
                            r.inner.scroll_to_me(None);
                        }
                        if r.inner.clicked() {
                            run = Some(h.item.clone());
                        }
                    }
                    if hits.is_empty() {
                        ui.weak(tr("(no entries)"));
                    }
                });
            });
        // No title bar and no close button: a click outside closes the search.
        let outside = ctx.input(|i| i.pointer.any_pressed().then(|| i.pointer.press_origin()).flatten())
            .is_some_and(|at| shown.as_ref().is_some_and(|r| !r.response.rect.contains(at)));
        if outside && !just_opened {
            open = false;
        }
        if enter {
            run = hits.get(self.search.selected).map(|h| h.item.clone()).filter(|e| e.cmd().is_none_or(|c| self.cmd_enabled(c)));
        }
        if esc || !open {
            self.search.open = false;
        }
        if let Some(e) = run {
            self.search.open = false;
            let page = self.session.page;
            match e {
                Entry::Cmd(c) => self.run(ctx, c),
                Entry::Plugin(c) => self.start_plugin_command(c),
                Entry::Align(a) => {
                    openecraft_core::arrange::align(&mut self.session.prj, page, &self.editor.selection, a);
                }
                Entry::Distribute(horizontal) => {
                    openecraft_core::arrange::distribute(&mut self.session.prj, page, &self.editor.selection, horizontal);
                }
                Entry::NewPage(kind) => self.add_page(kind),
            }
        }
    }

    /// The entries of a menu command run from the search, a key or a key tip.
    pub(super) fn command_menu_window(&mut self, ctx: &egui::Context) {
        let Some(cmd) = self.search.menu else { return };
        let r = egui::Modal::new(egui::Id::unique("command_menu")).show(ctx, |ui| {
            ui.set_width(280.0);
            ui.horizontal(|ui| {
                icons::show(ui, commands::def(cmd).icon, 18.0);
                ui.strong(tr(commands::def(cmd).title));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icons::button(ui, Icon::Close, tr("Close")).clicked() {
                        ui.close();
                    }
                });
            });
            ui.separator();
            self.cmd_menu(ui, cmd);
        });
        if r.should_close() {
            self.search.menu = None;
        }
    }
}
