//! Mouse gestures and modifier keys the user can change (Settings › Keys and gestures):
//! the buttons that pan and orbit the views, finish or cancel a drawing and open the
//! properties, and the modifiers that add to the selection, open links or turn the
//! wheel sideways. Like the shortcuts, the defaults are kept apart from the user's
//! changes (stored by the app). The app hands them to the views with [`set`]; what the
//! wheel and the fingers do is in `openecraft_ui::nav`, which takes the wheel modifiers
//! from here ([`Gestures::nav`]).

use egui::{Event, Modifiers, PointerButton, Response};
use openecraft_i18n::{tr, tr_const};
use openecraft_ui::nav::{self, Mod};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Storage key of the user's gestures and modifiers.
pub const STORAGE_KEY: &str = "gestures";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Button {
    Primary,
    Secondary,
    Middle,
}

impl Button {
    pub const ALL: [Button; 3] = [Button::Primary, Button::Secondary, Button::Middle];

    pub fn pointer(self) -> PointerButton {
        match self {
            Button::Primary => PointerButton::Primary,
            Button::Secondary => PointerButton::Secondary,
            Button::Middle => PointerButton::Middle,
        }
    }

    fn id(self) -> &'static str {
        match self {
            Button::Primary => "left",
            Button::Secondary => "right",
            Button::Middle => "middle",
        }
    }

    /// English name, translated where shown.
    pub fn title(self) -> &'static str {
        match self {
            Button::Primary => tr_const!("Left button"),
            Button::Secondary => tr_const!("Right button"),
            Button::Middle => tr_const!("Middle button"),
        }
    }
}

/// How the button of a mouse action is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Drag,
    Click,
    DoubleClick,
    /// Pressed while the left button moves something.
    Press,
}

impl Kind {
    pub fn title(self) -> &'static str {
        match self {
            Kind::Drag => tr_const!("Drag"),
            Kind::Click => tr_const!("Click"),
            Kind::DoubleClick => tr_const!("Double-click"),
            Kind::Press => tr_const!("Click while moving"),
        }
    }
}

/// A button with the modifier that must be held (none: any or none).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Binding {
    pub modifier: Option<Mod>,
    pub button: Button,
}

impl Binding {
    pub const fn new(modifier: Option<Mod>, button: Button) -> Binding {
        Binding { modifier, button }
    }

    /// `Shift+middle`, as stored.
    fn id(&self) -> String {
        match self.modifier {
            Some(m) => format!("{}+{}", m.name(), self.button.id()),
            None => self.button.id().to_string(),
        }
    }

    fn parse(s: &str) -> Option<Binding> {
        let (modifier, button) = match s.split_once('+') {
            Some((m, b)) => (Some(Mod::parse(m)?), b),
            None => (None, s),
        };
        let button = Button::ALL.into_iter().find(|b| b.id() == button.trim())?;
        Some(Binding { modifier, button })
    }

    /// `Shift+Middle button`, translated.
    pub fn text(&self) -> String {
        match self.modifier {
            Some(m) => format!("{}+{}", m.name(), tr(self.button.title())),
            None => tr(self.button.title()).to_string(),
        }
    }

    /// How well the held modifiers match: 2 the binding's modifier, 1 a binding without
    /// one, none when its modifier is not held.
    fn fit(&self, m: Modifiers) -> Option<u8> {
        match self.modifier {
            Some(k) => k.held(m).then_some(2),
            None => Some(1),
        }
    }
}

/// The views a gesture acts in: conflicts are only found within one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// Pages, panel layout, plans, symbol editor.
    Flat,
    /// The 3D view of the panel layout.
    Spatial,
}

/// The tools a gesture acts with: a select tool action and a drawing one never clash.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tools {
    All,
    Select,
    Drawing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MouseAction {
    PanView,
    Orbit3d,
    Pan3d,
    RotateWhileMoving,
    CancelTool,
    FinishByDoubleClick,
    OpenProperties,
    Focus3d,
}

impl MouseAction {
    pub const ALL: [MouseAction; 8] = [
        MouseAction::PanView,
        MouseAction::CancelTool,
        MouseAction::FinishByDoubleClick,
        MouseAction::OpenProperties,
        MouseAction::RotateWhileMoving,
        MouseAction::Orbit3d,
        MouseAction::Pan3d,
        MouseAction::Focus3d,
    ];

    pub fn id(self) -> &'static str {
        match self {
            MouseAction::PanView => "pan_view",
            MouseAction::Orbit3d => "orbit_3d",
            MouseAction::Pan3d => "pan_3d",
            MouseAction::RotateWhileMoving => "rotate_while_moving",
            MouseAction::CancelTool => "cancel_tool",
            MouseAction::FinishByDoubleClick => "finish_double_click",
            MouseAction::OpenProperties => "open_properties",
            MouseAction::Focus3d => "focus_3d",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            MouseAction::PanView => tr_const!("Pan the page, the panel layout and the plans"),
            MouseAction::Orbit3d => tr_const!("Orbit the 3D view"),
            MouseAction::Pan3d => tr_const!("Pan the 3D view"),
            MouseAction::RotateWhileMoving => tr_const!("Rotate what is being moved"),
            MouseAction::CancelTool => tr_const!("Finish or cancel the drawing, then back to selecting"),
            MouseAction::FinishByDoubleClick => tr_const!("Finish a wire, polyline, polygon or wall"),
            MouseAction::OpenProperties => tr_const!("Open the selection: its properties, in the floor plan its face"),
            MouseAction::Focus3d => tr_const!("Turn the 3D view to the point"),
        }
    }

    pub fn kind(self) -> Kind {
        match self {
            MouseAction::PanView | MouseAction::Orbit3d | MouseAction::Pan3d => Kind::Drag,
            MouseAction::RotateWhileMoving => Kind::Press,
            MouseAction::CancelTool => Kind::Click,
            MouseAction::FinishByDoubleClick | MouseAction::OpenProperties | MouseAction::Focus3d => Kind::DoubleClick,
        }
    }

    pub fn view(self) -> View {
        match self {
            MouseAction::Orbit3d | MouseAction::Pan3d | MouseAction::Focus3d => View::Spatial,
            _ => View::Flat,
        }
    }

    /// The tools the action acts with.
    pub fn tools(self) -> Tools {
        match self {
            MouseAction::RotateWhileMoving | MouseAction::OpenProperties => Tools::Select,
            MouseAction::CancelTool | MouseAction::FinishByDoubleClick => Tools::Drawing,
            _ => Tools::All,
        }
    }

    pub fn defaults(self) -> Vec<Binding> {
        use Button::*;
        let plain = |b| Binding::new(None, b);
        match self {
            MouseAction::PanView => vec![plain(Middle), plain(Secondary)],
            MouseAction::Orbit3d => vec![plain(Primary)],
            MouseAction::Pan3d => vec![plain(Middle), plain(Secondary), Binding::new(Some(Mod::Shift), Primary)],
            MouseAction::RotateWhileMoving | MouseAction::CancelTool => vec![plain(Secondary)],
            MouseAction::FinishByDoubleClick | MouseAction::OpenProperties | MouseAction::Focus3d => vec![plain(Primary)],
        }
    }

    /// Whether the binding can do the action: the left button alone draws and selects
    /// in the flat views, so it neither pans nor cancels there, and it is the button
    /// that moves while another one rotates.
    pub fn accepts(self, b: &Binding) -> bool {
        let plain_left = b.button == Button::Primary && b.modifier.is_none();
        match self {
            MouseAction::PanView | MouseAction::CancelTool => !plain_left,
            MouseAction::RotateWhileMoving => b.button != Button::Primary,
            _ => true,
        }
    }
}

/// Functions that are done with a modifier key held.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Role {
    AddToSelection,
    OpenLink,
    SingleItem,
    ReverseKeepConnections,
    RotateBack,
    Faster3d,
    WheelSideways,
    WheelZoom,
}

impl Role {
    pub const ALL: [Role; 8] = [Role::AddToSelection, Role::OpenLink, Role::SingleItem, Role::ReverseKeepConnections, Role::RotateBack, Role::Faster3d, Role::WheelSideways, Role::WheelZoom];

    pub fn id(self) -> &'static str {
        match self {
            Role::AddToSelection => "add_to_selection",
            Role::OpenLink => "open_link",
            Role::SingleItem => "single_item",
            Role::ReverseKeepConnections => "reverse_keep_connections",
            Role::RotateBack => "rotate_back",
            Role::Faster3d => "faster_3d",
            Role::WheelSideways => "wheel_sideways",
            Role::WheelZoom => "wheel_zoom",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Role::AddToSelection => tr_const!("Click or window: add to the selection or take out of it"),
            Role::OpenLink => tr_const!("Click a text: open its link"),
            Role::SingleItem => tr_const!("Click in the panel layout: a single item of an assembly"),
            Role::ReverseKeepConnections => tr_const!("Start of a move: Keep connections the other way round"),
            Role::RotateBack => tr_const!("Rotate what is being moved counter-clockwise"),
            Role::Faster3d => tr_const!("Move faster with the keys of the 3D view"),
            Role::WheelSideways => tr_const!("Scrolling up and down pans sideways"),
            Role::WheelZoom => tr_const!("Scrolling zooms, also where it pans"),
        }
    }

    pub fn default(self) -> Option<Mod> {
        match self {
            Role::AddToSelection | Role::RotateBack | Role::Faster3d | Role::WheelSideways => Some(Mod::Shift),
            Role::OpenLink | Role::WheelZoom => Some(Mod::Ctrl),
            Role::SingleItem | Role::ReverseKeepConnections => Some(Mod::Alt),
        }
    }

    /// When the modifier is read: roles read at the same moment need different
    /// modifiers. A click on the page, a click in the panel layout, the start of a move
    /// on the page, scrolling, rotating while moving, the keys of the 3D view.
    fn moments(self) -> &'static [u8] {
        match self {
            Role::AddToSelection => &[0, 1, 2],
            Role::OpenLink => &[0],
            Role::SingleItem => &[1],
            Role::ReverseKeepConnections => &[2],
            Role::WheelSideways | Role::WheelZoom => &[3],
            Role::RotateBack => &[4],
            Role::Faster3d => &[5],
        }
    }
}

/// English title of a mouse action or modifier role.
pub fn title(id: &str) -> &'static str {
    MouseAction::ALL.into_iter().find(|a| a.id() == id).map(MouseAction::title).or_else(|| Role::ALL.into_iter().find(|r| r.id() == id).map(Role::title)).unwrap_or("")
}

/// What two bindings share.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clash {
    Mouse(Kind, Binding),
    Modifier(Mod),
}

impl Clash {
    pub fn text(&self) -> String {
        match self {
            Clash::Mouse(kind, b) => format!("{} ({})", b.text(), tr(kind.title())),
            Clash::Modifier(m) => m.name().to_string(),
        }
    }
}

/// The user's gestures and modifiers: the changed ones, the others keep their defaults.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Gestures {
    mouse: BTreeMap<MouseAction, Vec<Binding>>,
    mods: BTreeMap<Role, Option<Mod>>,
}

impl Gestures {
    pub fn mouse(&self, a: MouseAction) -> Vec<Binding> {
        self.mouse.get(&a).cloned().unwrap_or_else(|| a.defaults())
    }

    /// Bindings the action does not accept are left out.
    pub fn set_mouse(&mut self, a: MouseAction, mut v: Vec<Binding>) {
        v.retain(|b| a.accepts(b));
        let mut seen = Vec::new();
        v.retain(|b| {
            let new = !seen.contains(b);
            seen.push(*b);
            new
        });
        if v == a.defaults() {
            self.mouse.remove(&a);
        } else {
            self.mouse.insert(a, v);
        }
    }

    pub fn add_mouse(&mut self, a: MouseAction, b: Binding) {
        let mut v = self.mouse(a);
        v.push(b);
        self.set_mouse(a, v);
    }

    pub fn modifier(&self, r: Role) -> Option<Mod> {
        self.mods.get(&r).copied().unwrap_or_else(|| r.default())
    }

    pub fn set_modifier(&mut self, r: Role, m: Option<Mod>) {
        if m == r.default() {
            self.mods.remove(&r);
        } else {
            self.mods.insert(r, m);
        }
    }

    pub fn mouse_is_default(&self, a: MouseAction) -> bool {
        !self.mouse.contains_key(&a)
    }

    pub fn modifier_is_default(&self, r: Role) -> bool {
        !self.mods.contains_key(&r)
    }

    pub fn reset_mouse(&mut self) {
        self.mouse.clear();
    }

    pub fn reset_modifiers(&mut self) {
        self.mods.clear();
    }

    /// The modifier of the role is held (never for a role switched off).
    pub fn held(&self, r: Role, m: Modifiers) -> bool {
        self.modifier(r).is_some_and(|k| k.held(m))
    }

    /// How well the held modifiers fit the best binding of `a` that `on` gives for its
    /// button.
    fn best(&self, a: MouseAction, m: Modifiers, on: impl Fn(PointerButton) -> bool) -> Option<u8> {
        self.mouse(a).iter().filter(|b| on(b.button.pointer())).filter_map(|b| b.fit(m)).max()
    }

    pub fn dragged(&self, a: MouseAction, resp: &Response, m: Modifiers) -> bool {
        self.best(a, m, |p| resp.dragged_by(p)).is_some()
    }

    /// Which of `actions` the drag of `resp` does: the one whose modifier is held before
    /// one without a modifier, the first of `actions` among equals.
    pub fn drag_of(&self, actions: &[MouseAction], resp: &Response, m: Modifiers) -> Option<MouseAction> {
        let mut out: Option<(u8, MouseAction)> = None;
        for &a in actions {
            if let Some(f) = self.best(a, m, |p| resp.dragged_by(p)) {
                if out.is_none_or(|(g, _)| f > g) {
                    out = Some((f, a));
                }
            }
        }
        out.map(|(_, a)| a)
    }

    pub fn clicked(&self, a: MouseAction, resp: &Response, m: Modifiers) -> bool {
        self.best(a, m, |p| resp.clicked_by(p)).is_some()
    }

    pub fn double_clicked(&self, a: MouseAction, resp: &Response, m: Modifiers) -> bool {
        self.best(a, m, |p| resp.double_clicked_by(p)).is_some()
    }

    /// The presses of the buttons of `a` among the events of a frame, each with the
    /// modifiers held.
    pub fn presses(&self, a: MouseAction, events: &[Event]) -> Vec<Modifiers> {
        let bindings = self.mouse(a);
        events
            .iter()
            .filter_map(|e| match e {
                Event::PointerButton { button, pressed: true, modifiers, .. } => bindings.iter().any(|b| b.button.pointer() == *button && b.fit(*modifiers).is_some()).then_some(*modifiers),
                _ => None,
            })
            .collect()
    }

    /// The navigation settings with the wheel modifiers of these.
    pub fn nav(&self, base: nav::Settings) -> nav::Settings {
        nav::Settings { sideways: self.modifier(Role::WheelSideways), zoom: self.modifier(Role::WheelZoom), ..base }
    }

    /// Gestures and modifiers used by more than one function at the same time.
    pub fn conflicts(&self) -> Vec<(Clash, Vec<&'static str>)> {
        let mut out: Vec<(Clash, Vec<&'static str>)> = Vec::new();
        let mut add = |c: Clash, id: &'static str| match out.iter_mut().find(|(k, ids)| *k == c && ids.iter().all(|o| at_once(o, id))) {
            Some((_, ids)) => ids.push(id),
            None => out.push((c, vec![id])),
        };
        for a in MouseAction::ALL {
            for b in self.mouse(a) {
                add(Clash::Mouse(a.kind(), b), a.id());
            }
        }
        for r in Role::ALL {
            if let Some(m) = self.modifier(r) {
                add(Clash::Modifier(m), r.id());
            }
        }
        out.retain(|(_, ids)| ids.len() > 1);
        out
    }

    /// For the app's storage: id → bindings (a modifier switched off: none).
    pub fn to_storage(&self) -> BTreeMap<String, Vec<String>> {
        let mouse = self.mouse.iter().map(|(a, v)| (a.id().to_string(), v.iter().map(Binding::id).collect()));
        let mods = self.mods.iter().map(|(r, m)| (r.id().to_string(), m.iter().map(|m| m.name().to_string()).collect()));
        mouse.chain(mods).collect()
    }

    /// Unknown functions and bindings are left out.
    pub fn from_storage(stored: &BTreeMap<String, Vec<String>>) -> Gestures {
        let mut g = Gestures::default();
        for (id, v) in stored {
            if let Some(a) = MouseAction::ALL.into_iter().find(|a| a.id() == id) {
                g.set_mouse(a, v.iter().filter_map(|b| Binding::parse(b)).collect());
            } else if let Some(r) = Role::ALL.into_iter().find(|r| r.id() == id) {
                if let Some(m) = v.iter().map(|m| Mod::parse(m)).collect::<Option<Vec<Mod>>>() {
                    g.set_modifier(r, m.first().copied());
                }
            }
        }
        g
    }
}

/// Two functions can be wanted with one gesture: mouse actions of the same view and kind
/// that act with the same tools, or modifier roles read at the same moment.
fn at_once(a: &str, b: &str) -> bool {
    let mouse = |id: &str| MouseAction::ALL.into_iter().find(|a| a.id() == id);
    let role = |id: &str| Role::ALL.into_iter().find(|r| r.id() == id);
    match (mouse(a), mouse(b), role(a), role(b)) {
        (Some(x), Some(y), ..) => x.view() == y.view() && x.kind() == y.kind() && (x.tools() == Tools::All || y.tools() == Tools::All || x.tools() == y.tools()),
        (.., Some(x), Some(y)) => x.moments().iter().any(|m| y.moments().contains(m)),
        _ => false,
    }
}

/// The binding of `kind` done on `resp` (a recording pad) in this frame. A press while
/// moving takes no modifier: the modifier there turns the other way round.
pub fn recorded(kind: Kind, resp: &Response, events: &[Event], m: Modifiers) -> Option<Binding> {
    let button = Button::ALL.into_iter().find(|b| {
        let p = b.pointer();
        match kind {
            Kind::Drag => resp.drag_started_by(p),
            Kind::Click => resp.clicked_by(p),
            Kind::DoubleClick => resp.double_clicked_by(p),
            Kind::Press => resp.hovered() && events.iter().any(|e| matches!(e, Event::PointerButton { button, pressed: true, .. } if *button == p)),
        }
    })?;
    let modifier = if kind == Kind::Press { None } else { Mod::of(m) };
    Some(Binding { modifier, button })
}

fn ctx_id() -> egui::Id {
    egui::Id::unique("openecraft_gestures")
}

/// Hands the gestures to the views (every frame).
pub fn set(ctx: &egui::Context, g: &Gestures) {
    if ctx.data(|d| d.get_temp::<Arc<Gestures>>(ctx_id())).is_none_or(|old| *old != *g) {
        ctx.data_mut(|d| d.insert_temp(ctx_id(), Arc::new(g.clone())));
    }
}

/// The gestures handed over by [`set`], the defaults before.
pub fn get(ctx: &egui::Context) -> Arc<Gestures> {
    ctx.data(|d| d.get_temp(ctx_id())).unwrap_or_default()
}

#[cfg(test)]
mod tests;
