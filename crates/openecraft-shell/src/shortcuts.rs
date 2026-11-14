//! Keyboard shortcuts the user can change (Settings › Keys and gestures): every command
//! of `crate::commands` and the few keys only the page editor reads (actions), their
//! default keys, the user's own keys (stored by the app), conflicts and the text shown
//! in the ribbon, the menus and the help. Bindings are named by the command id, so the
//! keys stored by earlier versions (`open`, `rotate`, `wire_tool`, …) keep working.
//! The app hands the shortcuts to the views of the panel layout with [`set`].

use crate::commands::{self, Cmd};
use egui::{InputState, Key, Modifiers};
use openecraft_i18n::tr_const;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Storage key of the user's shortcuts.
pub const STORAGE_KEY: &str = "shortcuts";

/// Keys that only the page editor reads while a tool is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    WireCorner,
    Autoroute,
    FinishPolyline,
    NudgeLeft,
    NudgeRight,
    NudgeUp,
    NudgeDown,
    NextRouting,
    FlyForward,
    FlyBack,
    FlyLeft,
    FlyRight,
    FlyUp,
    FlyDown,
    FlyCloser,
    FlyAway,
}

/// Where the keys of a binding act: conflicts are only found among the bindings of one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// The schematic page and the commands.
    Page,
    /// The panel layout, its plans and its 3D view.
    Layout,
}

impl Action {
    pub const ALL: [Action; 16] = [
        Action::WireCorner,
        Action::Autoroute,
        Action::FinishPolyline,
        Action::NudgeLeft,
        Action::NudgeRight,
        Action::NudgeUp,
        Action::NudgeDown,
        Action::NextRouting,
        Action::FlyForward,
        Action::FlyBack,
        Action::FlyLeft,
        Action::FlyRight,
        Action::FlyUp,
        Action::FlyDown,
        Action::FlyCloser,
        Action::FlyAway,
    ];

    /// Keys held to move in the 3D view, plus and minus of: ahead, to the right, up, closer.
    pub const FLY: [(Action, Action); 4] = [(Action::FlyForward, Action::FlyBack), (Action::FlyRight, Action::FlyLeft), (Action::FlyUp, Action::FlyDown), (Action::FlyCloser, Action::FlyAway)];

    pub fn scopes(self) -> &'static [Scope] {
        match self {
            Action::FinishPolyline => &[Scope::Page, Scope::Layout],
            Action::FlyForward | Action::FlyBack | Action::FlyLeft | Action::FlyRight | Action::FlyUp | Action::FlyDown | Action::FlyCloser | Action::FlyAway => &[Scope::Layout],
            _ => &[Scope::Page],
        }
    }

    /// Id in the stored settings.
    pub fn id(self) -> &'static str {
        match self {
            Action::WireCorner => "wire_corner",
            Action::Autoroute => "autoroute",
            Action::FinishPolyline => "finish_polyline",
            Action::NudgeLeft => "nudge_left",
            Action::NudgeRight => "nudge_right",
            Action::NudgeUp => "nudge_up",
            Action::NudgeDown => "nudge_down",
            Action::NextRouting => "next_routing",
            Action::FlyForward => "fly_forward",
            Action::FlyBack => "fly_back",
            Action::FlyLeft => "fly_left",
            Action::FlyRight => "fly_right",
            Action::FlyUp => "fly_up",
            Action::FlyDown => "fly_down",
            Action::FlyCloser => "fly_closer",
            Action::FlyAway => "fly_away",
        }
    }

    /// English title, translated where shown.
    pub fn title(self) -> &'static str {
        match self {
            Action::WireCorner => tr_const!("Toggle corner direction while drawing wires"),
            Action::Autoroute => tr_const!("Toggle routing around symbols (wire tool)"),
            Action::FinishPolyline => tr_const!("Finish a wire, polyline or polygon"),
            Action::NudgeLeft => tr_const!("Move selection left by one grid step"),
            Action::NudgeRight => tr_const!("Move selection right by one grid step"),
            Action::NudgeUp => tr_const!("Move selection up by one grid step"),
            Action::NudgeDown => tr_const!("Move selection down by one grid step"),
            Action::NextRouting => tr_const!("Next routing of the connection symbol to place"),
            Action::FlyForward => tr_const!("Move ahead in the 3D view (held)"),
            Action::FlyBack => tr_const!("Move back in the 3D view (held)"),
            Action::FlyLeft => tr_const!("Move left in the 3D view (held)"),
            Action::FlyRight => tr_const!("Move right in the 3D view (held)"),
            Action::FlyUp => tr_const!("Move up in the 3D view (held)"),
            Action::FlyDown => tr_const!("Move down in the 3D view (held)"),
            Action::FlyCloser => tr_const!("Zoom in in the 3D view (held)"),
            Action::FlyAway => tr_const!("Zoom out in the 3D view (held)"),
        }
    }

    pub fn defaults(self) -> Vec<Shortcut> {
        let plain = |key| Shortcut { command: false, shift: false, alt: false, key };
        match self {
            Action::WireCorner => vec![plain(Key::Space)],
            Action::Autoroute => vec![plain(Key::A)],
            Action::FinishPolyline => vec![plain(Key::Enter)],
            Action::NudgeLeft => vec![plain(Key::ArrowLeft)],
            Action::NudgeRight => vec![plain(Key::ArrowRight)],
            Action::NudgeUp => vec![plain(Key::ArrowUp)],
            Action::NudgeDown => vec![plain(Key::ArrowDown)],
            Action::NextRouting => vec![plain(Key::Tab)],
            Action::FlyForward => vec![plain(Key::W), plain(Key::ArrowUp)],
            Action::FlyBack => vec![plain(Key::S), plain(Key::ArrowDown)],
            Action::FlyLeft => vec![plain(Key::A), plain(Key::ArrowLeft)],
            Action::FlyRight => vec![plain(Key::D), plain(Key::ArrowRight)],
            Action::FlyUp => vec![plain(Key::E)],
            Action::FlyDown => vec![plain(Key::Q)],
            Action::FlyCloser => vec![plain(Key::Plus), plain(Key::Equals)],
            Action::FlyAway => vec![plain(Key::Minus)],
        }
    }
}

/// Every binding: the commands, then the actions of the page editor.
pub fn binding_ids() -> Vec<&'static str> {
    commands::ALL.iter().map(|d| d.id).chain(Action::ALL.iter().map(|a| a.id())).collect()
}

/// The bindings grouped for the shortcut editor (English title, ids): the commands of
/// each ribbon tab in ribbon order, each command in the first tab that holds it, then
/// the commands without a place in a tab and the keys of the page editor.
pub fn groups() -> Vec<(&'static str, Vec<&'static str>)> {
    let mut seen: Vec<Cmd> = Vec::new();
    let mut out: Vec<(&'static str, Vec<&'static str>)> = Vec::new();
    for t in crate::ribbon::TABS.iter() {
        let mut ids = Vec::new();
        for c in t.groups.iter().flat_map(|g| g.items.iter().map(|i| i.cmd())) {
            if !seen.contains(&c) {
                seen.push(c);
                ids.push(commands::def(c).id);
            }
        }
        if !ids.is_empty() {
            out.push((t.title, ids));
        }
    }
    let other: Vec<&'static str> = commands::ALL.iter().filter(|d| !seen.contains(&d.cmd)).map(|d| d.id).collect();
    out.push((tr_const!("File and further commands"), other));
    let (layout, page): (Vec<Action>, Vec<Action>) = Action::ALL.iter().partition(|a| a.scopes() == [Scope::Layout]);
    out.push((tr_const!("Keys of the page editor"), page.iter().map(|a| a.id()).collect()));
    out.push((tr_const!("Keys of the 3D view"), layout.iter().map(|a| a.id()).collect()));
    out
}

/// Where a key of a binding acts. Plain keys of commands only act on the page; the
/// keys the canvas reads itself (rotate, delete, fit) also in the panel layout.
pub fn scopes(id: &str, s: &Shortcut) -> &'static [Scope] {
    match commands::by_id(id) {
        Some(d) if !d.canvas_keys && s.page_only() => &[Scope::Page],
        Some(_) => &[Scope::Page, Scope::Layout],
        None => Action::ALL.into_iter().find(|a| a.id() == id).map_or(&[Scope::Page], Action::scopes),
    }
}

/// English title of a binding.
pub fn title(id: &str) -> &'static str {
    commands::by_id(id).map(|d| d.title).or_else(|| Action::ALL.into_iter().find(|a| a.id() == id).map(Action::title)).unwrap_or("")
}

/// The keys of a binding that cannot be changed (shown, not recorded).
pub fn fixed(id: &str) -> Option<&'static str> {
    commands::by_id(id).and_then(|d| commands::fixed_keys(d.cmd))
}

/// The default keys of a binding.
pub fn defaults(id: &str) -> Vec<Shortcut> {
    match commands::by_id(id) {
        Some(d) => d.default_keys(),
        None => Action::ALL.into_iter().find(|a| a.id() == id).map(Action::defaults).unwrap_or_default(),
    }
}

/// A key with modifiers; `command` is Ctrl (Cmd on macOS).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub command: bool,
    pub shift: bool,
    pub alt: bool,
    pub key: Key,
}

impl Shortcut {
    /// `Ctrl+Shift+Z`, `F`, `PageDown`.
    pub fn parse(s: &str) -> Option<Shortcut> {
        let mut sc = Shortcut { command: false, shift: false, alt: false, key: Key::A };
        let parts: Vec<&str> = s.split('+').map(str::trim).collect();
        let (key, mods) = parts.split_last()?;
        for m in mods {
            match m.to_ascii_lowercase().as_str() {
                "ctrl" | "cmd" | "command" => sc.command = true,
                "shift" => sc.shift = true,
                "alt" | "option" => sc.alt = true,
                _ => return None,
            }
        }
        sc.key = Key::from_name(key)?;
        Some(sc)
    }

    pub fn text(&self) -> String {
        let mut s = String::new();
        for (on, name) in [(self.command, "Ctrl+"), (self.shift, "Shift+"), (self.alt, "Alt+")] {
            if on {
                s.push_str(name);
            }
        }
        s.push_str(self.key.name());
        s
    }

    pub fn modifiers(&self) -> Modifiers {
        let mut m = Modifiers::NONE;
        if self.command {
            m |= Modifiers::COMMAND;
        }
        if self.shift {
            m |= Modifiers::SHIFT;
        }
        if self.alt {
            m |= Modifiers::ALT;
        }
        m
    }

    /// The modifiers held are exactly the ones of the shortcut.
    pub fn held(&self, m: Modifiers) -> bool {
        m.command == self.command && m.shift == self.shift && m.alt == self.alt
    }

    pub fn pressed(&self, i: &InputState) -> bool {
        i.key_pressed(self.key) && self.held(i.modifiers)
    }

    /// A key without Ctrl or Alt that the app only gives to the page (the others act
    /// in every pane).
    pub fn page_only(&self) -> bool {
        !self.command && !self.alt && !matches!(self.key, Key::F1 | Key::F2 | Key::F4 | Key::F6 | Key::Insert | Key::PageUp | Key::PageDown)
    }
}

fn ctx_id() -> egui::Id {
    egui::Id::unique("openecraft_shortcuts")
}

/// Hands the shortcuts to the views that do not belong to the page editor.
pub fn set(ctx: &egui::Context, s: &Shortcuts) {
    if ctx.data(|d| d.get_temp::<Arc<Shortcuts>>(ctx_id())).is_none_or(|old| *old != *s) {
        ctx.data_mut(|d| d.insert_temp(ctx_id(), Arc::new(s.clone())));
    }
}

/// The shortcuts handed over by [`set`], the defaults before.
pub fn get(ctx: &egui::Context) -> Arc<Shortcuts> {
    ctx.data(|d| d.get_temp(ctx_id())).unwrap_or_default()
}

/// The user's shortcuts: changed bindings with their keys (none = no shortcut), the
/// others keep their defaults.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shortcuts {
    custom: BTreeMap<String, Vec<Shortcut>>,
}

impl Shortcuts {
    pub fn keys(&self, id: &str) -> Vec<Shortcut> {
        self.custom.get(id).cloned().unwrap_or_else(|| defaults(id))
    }

    pub fn set(&mut self, id: &str, keys: Vec<Shortcut>) {
        if keys == defaults(id) {
            self.custom.remove(id);
        } else {
            self.custom.insert(id.to_string(), keys);
        }
    }

    pub fn reset(&mut self, id: &str) {
        self.custom.remove(id);
    }

    pub fn reset_all(&mut self) {
        self.custom.clear();
    }

    pub fn is_default(&self, id: &str) -> bool {
        !self.custom.contains_key(id)
    }

    /// The keys of a binding for menus and the help (`Ctrl+Y / Ctrl+Shift+Z`).
    pub fn text(&self, id: &str) -> String {
        if let Some(fixed) = fixed(id) {
            return fixed.to_string();
        }
        self.keys(id).iter().map(Shortcut::text).collect::<Vec<_>>().join(" / ")
    }

    /// The keys of a command.
    pub fn cmd_text(&self, cmd: Cmd) -> String {
        self.text(commands::def(cmd).id)
    }

    pub fn pressed(&self, i: &InputState, id: &str) -> bool {
        self.keys(id).iter().any(|s| s.pressed(i))
    }

    pub fn cmd_pressed(&self, i: &InputState, cmd: Cmd) -> bool {
        self.pressed(i, commands::def(cmd).id)
    }

    pub fn action_pressed(&self, i: &InputState, a: Action) -> bool {
        self.pressed(i, a.id())
    }

    /// A key of the action is held down with the modifiers `m`: Ctrl and Alt must be the
    /// ones of the key, Shift is left to the caller (the 3D view moves faster).
    pub fn action_down(&self, i: &InputState, a: Action, m: Modifiers) -> bool {
        self.keys(a.id()).iter().any(|s| i.key_down(s.key) && (m.command || m.ctrl) == s.command && m.alt == s.alt)
    }

    /// Takes the presses of an action's plain Tab key out of the raw input, before
    /// egui moves the keyboard focus with them; returns how many there were.
    pub fn take_tab(&self, raw: &mut egui::RawInput, a: Action) -> usize {
        if !self.keys(a.id()).iter().any(|s| s.key == Key::Tab && !s.command && !s.shift && !s.alt) {
            return 0;
        }
        let tab = |e: &egui::Event| matches!(e, egui::Event::Key { key: Key::Tab, modifiers, .. } if modifiers.is_none());
        let presses = raw.events.iter().filter(|e| tab(e) && matches!(e, egui::Event::Key { pressed: true, .. })).count();
        raw.events.retain(|e| !tab(e));
        presses
    }

    /// Like [`Shortcuts::pressed`], and takes the key press so that widgets do not see it.
    pub fn consume(&self, i: &mut InputState, id: &str) -> bool {
        match self.keys(id).into_iter().find(|s| s.pressed(i)) {
            Some(s) => {
                i.consume_key(s.modifiers(), s.key);
                true
            }
            None => false,
        }
    }

    /// Shortcuts used by more than one binding where they act.
    pub fn conflicts(&self) -> Vec<(Shortcut, Vec<&'static str>)> {
        let mut out: Vec<(Shortcut, Vec<&'static str>)> = Vec::new();
        for scope in [Scope::Page, Scope::Layout] {
            let mut by_key: Vec<(Shortcut, Vec<&'static str>)> = Vec::new();
            for id in binding_ids() {
                for s in self.keys(id).into_iter().filter(|s| scopes(id, s).contains(&scope)) {
                    match by_key.iter_mut().find(|(k, _)| *k == s) {
                        Some((_, v)) => v.push(id),
                        None => by_key.push((s, vec![id])),
                    }
                }
            }
            for c in by_key.into_iter().filter(|(_, v)| v.len() > 1) {
                if !out.contains(&c) {
                    out.push(c);
                }
            }
        }
        out
    }

    /// For the app's storage: binding id → key texts.
    pub fn to_storage(&self) -> BTreeMap<String, Vec<String>> {
        self.custom.iter().map(|(id, v)| (id.clone(), v.iter().map(Shortcut::text).collect())).collect()
    }

    /// Unknown bindings and keys are left out.
    pub fn from_storage(stored: &BTreeMap<String, Vec<String>>) -> Shortcuts {
        let mut s = Shortcuts::default();
        let ids = binding_ids();
        for (id, keys) in stored {
            if ids.contains(&id.as_str()) {
                s.custom.insert(id.clone(), keys.iter().filter_map(|k| Shortcut::parse(k)).collect());
            }
        }
        s
    }
}

#[cfg(test)]
mod tests;
