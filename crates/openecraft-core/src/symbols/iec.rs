//! Extended IEC 60617 library: the variants of relays, contacts, switches, sensors,
//! protective devices, meters, transformers, power supplies and motors.
//!
//! Every symbol records the function kind it stands for (parts and function
//! templates), see [`kind`].

use super::*;
use openecraft_parts::FnKind;
use std::collections::HashMap;
use Anchor::*;
use Category as C;
use FnKind as K;

/// Blade of an NO / NC / changeover contact at the height of the linkage (y = 0).
const NO_BX: f32 = -0.9;
const NC_BX: f32 = 0.96;
const CO_BX: f32 = 1.0;

/// Operating devices (IEC 60617-02-13) at the end of the linkage.
#[derive(Clone, Copy, PartialEq)]
enum Op {
    Manual,
    Push,
    Pull,
    Turn,
    Key,
    PushKey,
    Mushroom,
    MushroomTurn,
    MushroomPull,
    MushroomKey,
    Foot,
    Cam,
    Roller,
    CamProfile,
    Pressure,
    Flow,
    GasFlow,
    Level,
    Thermal,
    Clock,
    Counter,
    CounterCam,
    CounterSwitch,
    Lock,
}

impl B {
    fn turn_at(&mut self, x: f32) -> &mut Self {
        self.l(x, -1.5, x, 1.5).l(x, -1.5, x + 1.0, -1.5).l(x, 1.5, x - 1.0, 1.5)
    }
    /// Pushing: a bracket open towards the contact.
    fn push_at(&mut self, x: f32) -> &mut Self {
        self.l(x - 1.0, -1.5, x - 1.0, 1.5).l(x - 1.0, -1.5, x, -1.5).l(x - 1.0, 1.5, x, 1.5).l(x, 0.0, x - 1.0, 0.0)
    }
    /// Pulling: a bracket open away from the contact.
    fn pull_at(&mut self, x: f32) -> &mut Self {
        self.l(x, -1.5, x, 1.5).l(x, -1.5, x - 1.0, -1.5).l(x, 1.5, x - 1.0, 1.5)
    }
    fn key_at(&mut self, x: f32) -> &mut Self {
        self.l(x, 0.0, x - 1.4, 0.0).l(x - 0.6, 0.0, x - 0.6, 1.0).circle(x - 2.4, 0.0, 1.0, false)
    }
    fn mushroom_at(&mut self, x: f32) -> &mut Self {
        self.l(x, -1.8, x, 1.8).arc(x, 0.0, 1.8, 90.0, 270.0)
    }
    /// Cam with its notch towards the contact, centre at `cx`.
    fn cam_at(&mut self, cx: f32) -> &mut Self {
        let (c, s) = (35f32.to_radians().cos() * 1.5, 35f32.to_radians().sin() * 1.5);
        self.arc(cx, 0.0, 1.5, 35.0, 325.0).l(cx, 0.0, cx + c, s).l(cx, 0.0, cx + c, -s)
    }
    fn op_box(&mut self, x: f32, t: &str) -> &mut Self {
        self.rect(x - 3.0, -1.5, x, 1.5, false);
        if !t.is_empty() {
            self.text(x - 1.5, 0.0, t, 1.6, Center);
        }
        self
    }
    fn counter_at(&mut self, x: f32, n: bool) -> &mut Self {
        self.rect(x - 3.0, -1.5, x, 1.5, false).circle(x - 1.5, 0.0, 0.7, false);
        if n {
            self.text(x - 1.5, -2.6, "n", 1.6, Center);
        }
        self
    }
    /// Thermal (bimetal) mark lying along the linkage.
    fn bimetal_at(&mut self, x: f32) -> &mut Self {
        self.poly(&[(x, 0.0), (x - 1.0, 0.0), (x - 1.0, -1.2), (x - 2.2, -1.2), (x - 2.2, 0.0), (x - 3.0, 0.0)], false, false)
    }
    fn clock_at(&mut self, cx: f32, cy: f32, r: f32) -> &mut Self {
        self.circle(cx, cy, r, false).l(cx, cy, cx, cy - r * 0.75).l(cx, cy, cx + r * 0.55, cy + r * 0.3)
    }
    /// Operating device linked to the contact blade at `bx`; `latch` = stays in its
    /// position (detent mark on the linkage), otherwise it returns automatically.
    fn op(&mut self, bx: f32, op: Op, latch: bool) -> &mut Self {
        let x = -5.0;
        self.dl(bx, 0.0, x, 0.0);
        if latch {
            self.poly(&[(-3.6, -1.0), (-3.0, 0.0), (-2.4, -1.0)], false, false);
        }
        match op {
            Op::Manual => self.l(x, -1.5, x, 1.5),
            Op::Push => self.push_at(x),
            Op::Pull => self.pull_at(x),
            Op::Turn => self.turn_at(x),
            Op::Key => self.key_at(x),
            Op::PushKey => self.push_at(x).key_at(x - 1.0),
            Op::Mushroom => self.mushroom_at(x),
            Op::MushroomTurn => self.mushroom_at(x).l(x - 1.8, 0.0, x - 2.6, 0.0).turn_at(x - 2.6),
            Op::MushroomPull => self.mushroom_at(x).l(x - 1.8, 0.0, x - 2.6, 0.0).pull_at(x - 2.6),
            Op::MushroomKey => self.mushroom_at(x).key_at(x - 1.8),
            Op::Foot => self.l(x, 0.0, x, 1.5).l(x, 1.5, x - 2.5, -1.0),
            Op::Cam => self.cam_at(x - 1.5),
            Op::Roller => self.circle(x - 1.5, 0.0, 1.5, false),
            Op::CamProfile => self.circle(x - 1.5, 0.0, 1.5, false).arc(x - 1.5, -1.5, 0.7, 180.0, 360.0),
            Op::Pressure => self.op_box(x, "p"),
            Op::Flow => self.op_box(x, "").poly(&[(x - 2.4, 1.0), (x - 2.4, -1.0), (x - 0.6, -1.0)], true, false),
            Op::GasFlow => self.op_box(x, "").poly(&[(x - 2.4, 1.0), (x - 2.4, -1.0), (x - 0.6, -1.0)], true, true),
            Op::Level => self.circle(x - 1.5, 0.0, 1.5, false).l(x - 4.0, 1.5, x + 0.5, 1.5),
            Op::Thermal => self.bimetal_at(x),
            Op::Clock => self.clock_at(x - 1.5, 0.0, 1.5),
            Op::Counter => self.counter_at(x, true),
            Op::CounterCam => self.cam_at(x - 1.5).l(x - 3.0, 0.0, x - 3.5, 0.0).counter_at(x - 3.5, true),
            Op::CounterSwitch => self.counter_at(x, false).l(x - 3.0, -1.5, x - 3.0, 1.5),
            Op::Lock => self.rect(x - 2.0, -0.8, x, 0.8, false).l(x - 2.0, 0.0, x - 3.0, 0.0).l(x - 3.0, -1.5, x - 3.0, 1.5),
        }
    }
    /// NO or NC contact at x = 0; returns where the blade crosses the linkage.
    fn contact(&mut self, nc: bool) -> f32 {
        if nc {
            self.nc(0.0);
            NC_BX
        } else {
            self.no(0.0);
            NO_BX
        }
    }
    /// Changeover contact; connection points: common, NC, NO (like `co`).
    fn changeover(&mut self) -> f32 {
        self.l(0.0, 4.0, 0.0, 2.0).l(0.0, 2.0, 2.3, -2.6).l(0.0, -4.0, 0.0, -2.0).l(4.0, -4.0, 4.0, -2.0).l(4.0, -2.0, 1.8, -2.0);
        CO_BX
    }
    fn pins2(&mut self, a: &'static str, b: &'static str) -> &mut Self {
        self.pin(0.0, -4.0, a).pin(0.0, 4.0, b)
    }
    /// Delay mark like [`B::delay`], also for both directions.
    fn delay_mark(&mut self, bx: f32, on: bool, off: bool) -> &mut Self {
        if on && off {
            self.l(bx, -0.5, -4.0, -0.5).l(bx, 0.5, -4.0, 0.5).arc(-4.0, 0.0, 1.5, 90.0, 270.0).arc(-7.0, 0.0, 1.5, 270.0, 450.0)
        } else {
            self.delay(bx, on)
        }
    }
    /// Relay coil (A1 / A2) with an empty side box for its mark.
    fn coil_box(&mut self, side: bool) -> &mut Self {
        self.l(0.0, -4.0, 0.0, -2.0).rect(-3.0, -2.0, 3.0, 2.0, false).l(0.0, 2.0, 0.0, 4.0);
        if side {
            self.rect(-6.0, -2.0, -3.0, 2.0, false);
        }
        self.pins2("A1", "A2")
    }
    /// Pole at x with pin numbers 2i+1 / 2i+2.
    fn pole_pins(&mut self, n: usize) -> &mut Self {
        const TOP: [&str; 4] = ["1", "3", "5", "7"];
        const BOTTOM: [&str; 4] = ["2", "4", "6", "8"];
        for (i, label) in TOP.into_iter().take(n).enumerate() {
            self.pin(i as f32 * 8.0, -4.0, label);
        }
        for (i, label) in BOTTOM.into_iter().take(n).enumerate() {
            self.pin(i as f32 * 8.0, 4.0, label);
        }
        self
    }
    /// Dashed linkage from the operating device at `x0` across n poles.
    fn gang(&mut self, n: usize, x0: f32, bx: f32) -> &mut Self {
        self.dl(x0, 0.0, (n - 1) as f32 * 8.0 + bx, 0.0)
    }
    /// Fuse inside the blade of a switch pole at x.
    fn fuse_blade(&mut self, x: f32) -> &mut Self {
        let (d, n) = (p(-2.0, -4.4) * (1.0 / p(-2.0, -4.4).len()), p(4.4, -2.0) * (1.0 / p(4.4, -2.0).len()));
        let m = p(x - 1.0, -0.2);
        let c = |a: f32, b: f32| {
            let q = m + d * a + n * b;
            (q.x, q.y)
        };
        self.l(x, -4.0, x, -2.0).l(x, 2.0, x, 4.0).l(x, 2.0, x - 2.0, -2.4);
        self.poly(&[c(1.6, 0.7), c(-1.6, 0.7), c(-1.6, -0.7), c(1.6, -0.7)], true, false)
    }
    /// Filled automatic tripping mark (IEC 60617-07-01-05) on the blade of a pole at x.
    fn trip_mark(&mut self, x: f32) -> &mut Self {
        self.rect(x - 1.5, -0.6, x - 0.5, 0.6, true)
    }
    /// Switch function (small circle) on the fixed contact of a pole at x.
    fn switch_circle(&mut self, x: f32) -> &mut Self {
        self.circle(x, -2.7, 0.7, false)
    }
    /// Release box with thermal (bimetal) and magnetic (I>) releases, right edge at x.
    fn release_tm(&mut self, x: f32) -> &mut Self {
        self.rect(x - 9.0, -1.5, x, 1.5, false).bimetal_at(x - 0.5).text(x - 6.5, 0.0, "I>", 1.6, Center)
    }
    /// Rotary drive of a motor circuit breaker above its releases: square with a cross.
    fn rotary_drive(&mut self) -> &mut Self {
        let (cx, cy) = (-9.5, -5.5);
        self.dl(cx, -1.5, cx, cy + 1.5).rect(cx - 1.5, cy - 1.5, cx + 1.5, cy + 1.5, false).l(cx, cy - 1.5, cx, cy + 1.5).l(cx - 1.5, cy, cx + 1.5, cy)
    }
    /// Winding along a vertical lead at x: n half circles from y0 downwards.
    fn vwinding(&mut self, x: f32, y0: f32, n: usize) -> &mut Self {
        for i in 0..n {
            self.arc(x, y0 + 1.0 + i as f32 * 2.0, 1.0, 270.0, 450.0);
        }
        self
    }
    /// Lead from the circle (c, r) towards the bend point q, then on to the pin at `to`.
    fn lead(&mut self, c: P, r: f32, q: P, to: P) -> &mut Self {
        let e = c + (q - c) * (r / (q - c).len());
        self.l(e.x, e.y, q.x, q.y).l(q.x, q.y, to.x, to.y)
    }
    /// Indicating instrument (IEC 60617-08-01): circle with the quantity.
    fn meter(&mut self, t: &str, sub: &str) -> &mut Self {
        self.l(0.0, -4.0, 0.0, -3.0).circle(0.0, 0.0, 3.0, false).l(0.0, 3.0, 0.0, 4.0);
        if t.is_empty() {
        } else if sub.is_empty() {
            let size = if t.chars().count() > 2 { 1.6 } else { 2.5 };
            self.text(0.0, 0.0, t, size, Center);
        } else {
            self.text(0.0, -0.8, t, 2.0, Center).text(0.0, 1.3, sub, 1.1, Center);
        }
        self.pins2("1", "2")
    }
    /// Integrating instrument (IEC 60617-08-04): rectangle with a band on top.
    fn int_meter(&mut self, t: &str, four: bool) -> &mut Self {
        self.rect(-4.0, -4.0, 4.0, 4.0, false).l(-4.0, -1.5, 4.0, -1.5).text(0.0, 1.0, t, 1.8, Center);
        self.l(0.0, -8.0, 0.0, -4.0).l(0.0, 4.0, 0.0, 8.0).pin(0.0, -8.0, "1");
        if four {
            self.l(-8.0, 0.0, -4.0, 0.0).l(4.0, 0.0, 8.0, 0.0).pin(8.0, 0.0, "2").pin(0.0, 8.0, "3").pin(-8.0, 0.0, "4")
        } else {
            self.pin(0.0, 8.0, "2")
        }
    }
    fn arrow_head(&mut self, tx: f32, ty: f32, dx: f32, dy: f32, s: f32) -> &mut Self {
        let (nx, ny) = (-dy, dx);
        self.poly(&[(tx, ty), (tx - dx * s + nx * s * 0.45, ty - dy * s + ny * s * 0.45), (tx - dx * s - nx * s * 0.45, ty - dy * s - ny * s * 0.45)], true, true)
    }
    /// Winding as a row of n half circles between x0 and x0 + 2n, bulging down (or up).
    fn winding(&mut self, x0: f32, y: f32, n: usize, down: bool) -> &mut Self {
        for i in 0..n {
            let cx = x0 + 1.0 + i as f32 * 2.0;
            if down {
                self.arc(cx, y, 1.0, 180.0, 0.0);
            } else {
                self.arc(cx, y, 1.0, 180.0, 360.0);
            }
        }
        self
    }
    /// Diode along direction d with its centre at m.
    fn diode_at(&mut self, m: P, d: P) -> &mut Self {
        let n = p(-d.y, d.x);
        let tip = m + d * 0.7;
        let base = m - d * 0.7;
        let (a, b) = (base + n * 0.8, base - n * 0.8);
        let (c, e) = (tip + n * 0.8, tip - n * 0.8);
        self.poly(&[(tip.x, tip.y), (a.x, a.y), (b.x, b.y)], true, false).l(c.x, c.y, e.x, e.y)
    }
    /// Converter box with the two quantities of its sides.
    fn converter(&mut self, w: f32, a: &str, b: &str) -> &mut Self {
        self.rect(-3.0, 0.0, w, 12.0, false).l(-3.0, 12.0, w, 0.0);
        self.text(1.5, 3.5, a, 3.0, Center).text(w - 4.5, 8.5, b, 3.0, Center)
    }
    /// Motor circle with its leads (like `motor3`), the leads at x = 0, 8, 16.
    fn motor3_body(&mut self, t1: &str, t2: &str, bottom: bool) -> &mut Self {
        let (cx, cy, r) = (8.0f32, 8.0f32, 6.0f32);
        for (x, y0, y1) in [(0.0, -4.0, 0.0), (16.0, -4.0, 0.0)] {
            let d = p(x, y1) - p(cx, cy);
            let e = p(cx, cy) + d * (r / d.len());
            self.l(x, y0, x, y1).l(x, y1, e.x, e.y);
        }
        self.l(8.0, -4.0, 8.0, cy - r);
        if bottom {
            for x in [0.0f32, 16.0] {
                let d = p(x, 16.0) - p(cx, cy);
                let e = p(cx, cy) + d * (r / d.len());
                self.l(x, 20.0, x, 16.0).l(x, 16.0, e.x, e.y);
            }
            self.l(8.0, 20.0, 8.0, cy + r);
        }
        self.circle(cx, cy, r, false).text(cx, cy - 1.2, t1, 3.5, Center).text(cx, cy + 2.4, t2, 2.2, Center)
    }
    /// Motor circle for 1-phase / DC motors (like `motor1`), the leads at x = 0, 8.
    fn motor1_body(&mut self, t1: &str, t2: &str) -> &mut Self {
        let (cx, cy, r) = (4.0f32, 8.0f32, 5.0f32);
        for x in [0.0f32, 8.0] {
            let d = p(x, 3.0) - p(cx, cy);
            let e = p(cx, cy) + d * (r / d.len());
            self.l(x, -4.0, x, 3.0).l(x, 3.0, e.x, e.y);
        }
        self.circle(cx, cy, r, false).text(cx, cy - 1.0, t1, 3.0, Center).text(cx, cy + 2.0, t2, 2.0, Center)
    }
    fn pe_lead(&mut self, x: f32, cx: f32, cy: f32, r: f32) -> &mut Self {
        self.l(x, -4.0, x, cy).l(x, cy, cx + r, cy).pin(x, -4.0, "PE")
    }
    /// Fan wheel below a motor circle.
    fn fan_at(&mut self, cx: f32, y: f32) -> &mut Self {
        self.rect(cx - 2.0, y, cx + 2.0, y + 4.0, false).l(cx, y + 2.0, cx - 1.4, y + 0.8).l(cx, y + 2.0, cx + 1.4, y + 3.2).circle(cx, y + 2.0, 0.4, true)
    }
    /// 3-phase transformer in circle form, windings marked with their connection.
    /// Returns the centres of the circles.
    fn trafo3_circles(&mut self, marks: &[Wnd]) -> Vec<P> {
        let n = marks.len();
        let centres: Vec<P> = (0..n).map(|i| p(8.0, if n == 3 { -6.0 + i as f32 * 6.0 } else { -3.5 + i as f32 * 7.0 })).collect();
        for (c, m) in centres.iter().zip(marks) {
            self.circle(c.x, c.y, 5.0, false);
            self.wnd_mark(c.x, c.y, *m);
        }
        let (first, last) = (centres[0], centres[n - 1]);
        for x in [0.0, 8.0, 16.0] {
            self.lead(first, 5.0, p(x, -12.0), p(x, -16.0)).lead(last, 5.0, p(x, 12.0), p(x, 16.0));
        }
        self.pin(0.0, -16.0, "1U").pin(8.0, -16.0, "1V").pin(16.0, -16.0, "1W");
        self.pin(0.0, 16.0, "2U").pin(8.0, 16.0, "2V").pin(16.0, 16.0, "2W");
        centres
    }
    fn wnd_mark(&mut self, cx: f32, cy: f32, m: Wnd) -> &mut Self {
        match m {
            Wnd::None => self,
            Wnd::Star => self.l(cx, cy, cx, cy - 1.6).l(cx, cy, cx - 1.4, cy + 0.8).l(cx, cy, cx + 1.4, cy + 0.8),
            Wnd::Delta => self.poly(&[(cx, cy - 1.4), (cx + 1.4, cy + 1.0), (cx - 1.4, cy + 1.0)], true, false),
            Wnd::Zigzag => self.poly(&[(cx, cy - 1.6), (cx, cy), (cx - 1.4, cy + 0.8), (cx - 0.4, cy + 1.6)], false, false).l(cx, cy, cx + 1.4, cy + 0.8),
        }
    }
}

/// Connection of a transformer winding.
#[derive(Clone, Copy)]
enum Wnd {
    None,
    Star,
    Delta,
    Zigzag,
}

/// Draws the mark that tells a variant from its base symbol.
type Mark = fn(&mut B);
/// Key, name and mark of a variant.
type Variant = (&'static str, &'static str, Mark);

/// Collects the symbols and the function kind of each.
struct Lib<'a> {
    v: &'a mut Vec<SymbolDef>,
    b: B,
    kinds: HashMap<&'static str, FnKind>,
}

impl Lib<'_> {
    fn push(&mut self, d: Def, kind: Option<FnKind>) {
        if let Some(k) = kind {
            self.kinds.insert(d.key, k);
        }
        let s = d.build(&mut self.b);
        self.v.push(s);
    }
}

static KINDS: OnceLock<HashMap<&'static str, FnKind>> = OnceLock::new();

/// Function kind of a symbol of this library.
pub fn kind(key: &str) -> Option<FnKind> {
    library();
    KINDS.get().and_then(|k| k.get(key).copied())
}

type SwitchRow = (&'static str, &'static str, bool, Op, bool, &'static str, FnKind);

/// Switches and sensor contacts: key, name, NC, operating device, latching, letter, kind.
const SWITCHES: &[SwitchRow] = &[
    ("man_no", "Manual switch NO", false, Op::Manual, false, "S", K::SwitchNo),
    ("man_nc", "Manual switch NC", true, Op::Manual, false, "S", K::SwitchNc),
    ("man_no_m", "Manual switch NO, latching", false, Op::Manual, true, "S", K::SwitchNo),
    ("man_nc_m", "Manual switch NC, latching", true, Op::Manual, true, "S", K::SwitchNc),
    ("pb_no_m", "Push button NO, latching", false, Op::Push, true, "S", K::PushbuttonNo),
    ("pb_nc_m", "Push button NC, latching", true, Op::Push, true, "S", K::PushbuttonNc),
    ("pull_no", "Pull button NO", false, Op::Pull, false, "S", K::PushbuttonNo),
    ("pull_nc", "Pull button NC", true, Op::Pull, false, "S", K::PushbuttonNc),
    ("pull_no_m", "Pull button NO, latching", false, Op::Pull, true, "S", K::PushbuttonNo),
    ("pull_nc_m", "Pull button NC, latching", true, Op::Pull, true, "S", K::PushbuttonNc),
    ("key_nc", "Key switch NC", true, Op::Key, false, "S", K::SwitchNc),
    ("key_no_m", "Key switch NO, latching", false, Op::Key, true, "S", K::SwitchNo),
    ("key_nc_m", "Key switch NC, latching", true, Op::Key, true, "S", K::SwitchNc),
    ("pb_key_no_m", "Push button with key NO, latching", false, Op::PushKey, true, "S", K::PushbuttonNo),
    ("pb_key_nc_m", "Push button with key NC, latching", true, Op::PushKey, true, "S", K::PushbuttonNc),
    ("estop_no", "Emergency stop NO", false, Op::Mushroom, false, "S", K::PushbuttonNo),
    ("estop_turn_no", "Emergency stop NO, turn to reset", false, Op::MushroomTurn, true, "S", K::PushbuttonNo),
    ("estop_turn_nc", "Emergency stop NC, turn to reset", true, Op::MushroomTurn, true, "S", K::PushbuttonNc),
    ("estop_pull_no", "Emergency stop NO, pull to reset", false, Op::MushroomPull, true, "S", K::PushbuttonNo),
    ("estop_pull_nc", "Emergency stop NC, pull to reset", true, Op::MushroomPull, true, "S", K::PushbuttonNc),
    ("estop_key_no", "Emergency stop NO, key reset", false, Op::MushroomKey, true, "S", K::PushbuttonNo),
    ("estop_key_nc", "Emergency stop NC, key reset", true, Op::MushroomKey, true, "S", K::PushbuttonNc),
    ("sel_no_m", "Selector switch NO, latching", false, Op::Turn, true, "S", K::SwitchNo),
    ("sel_nc_m", "Selector switch NC, latching", true, Op::Turn, true, "S", K::SwitchNc),
    ("foot_no", "Foot switch NO", false, Op::Foot, false, "S", K::SwitchNo),
    ("foot_nc", "Foot switch NC", true, Op::Foot, false, "S", K::SwitchNc),
    ("counter_sw_no", "Counter switch NO", false, Op::CounterSwitch, false, "S", K::SwitchNo),
    ("counter_sw_nc", "Counter switch NC", true, Op::CounterSwitch, false, "S", K::SwitchNc),
    ("thermo_no", "Bimetal switch NO", false, Op::Thermal, false, "B", K::Sensor),
    ("thermo_nc", "Bimetal switch NC", true, Op::Thermal, false, "B", K::SensorNc),
    ("pressure_nc", "Pressure switch NC", true, Op::Pressure, false, "B", K::SensorNc),
    ("flow_no", "Flow switch NO", false, Op::Flow, false, "B", K::Sensor),
    ("flow_nc", "Flow switch NC", true, Op::Flow, false, "B", K::SensorNc),
    ("gasflow_no", "Gas flow switch NO", false, Op::GasFlow, false, "B", K::Sensor),
    ("gasflow_nc", "Gas flow switch NC", true, Op::GasFlow, false, "B", K::SensorNc),
    ("level_nc", "Float switch NC", true, Op::Level, false, "B", K::SensorNc),
    ("cam_no", "Cam switch NO", false, Op::Cam, false, "B", K::Sensor),
    ("cam_nc", "Cam switch NC", true, Op::Cam, false, "B", K::SensorNc),
    ("roller_no", "Roller switch NO", false, Op::Roller, false, "B", K::Sensor),
    ("roller_nc", "Roller switch NC", true, Op::Roller, false, "B", K::SensorNc),
    ("camprof_no", "Cam profile switch NO", false, Op::CamProfile, false, "B", K::Sensor),
    ("camprof_nc", "Cam profile switch NC", true, Op::CamProfile, false, "B", K::SensorNc),
];

/// Three-position selector switches: key, name, NC, return of the left / middle /
/// right position (true = latching), positions in which the contact is actuated.
type Selector3 = (&'static str, &'static str, bool, [bool; 3], [bool; 3]);
const SELECTORS3: &[Selector3] = &[
    ("sel3_no_sr_off", "Selector switch 3 positions NO, spring return to middle, middle off", false, [false, true, false], [true, false, true]),
    ("sel3_nc_sr_off", "Selector switch 3 positions NC, spring return to middle, middle off", true, [false, true, false], [true, false, true]),
    ("sel3_no_ret_off", "Selector switch 3 positions NO, return to off, latching", false, [false, true, true], [false, false, true]),
    ("sel3_nc_ret_off", "Selector switch 3 positions NC, return to off, latching", true, [false, true, true], [false, false, true]),
    ("sel3_no_m_off", "Selector switch 3 positions NO, latching, middle off", false, [true, true, true], [true, false, true]),
    ("sel3_nc_m_off", "Selector switch 3 positions NC, latching, middle off", true, [true, true, true], [true, false, true]),
    ("sel3_no_sr_mid", "Selector switch 3 positions NO, spring return, middle latching", false, [false, true, false], [false, true, false]),
    ("sel3_nc_sr_mid", "Selector switch 3 positions NC, spring return, middle latching", true, [false, true, false], [false, true, false]),
    ("sel3_no_sr_mm", "Selector switch 3 positions NO, spring return, two positions latching", false, [false, true, true], [false, true, true]),
    ("sel3_nc_sr_mm", "Selector switch 3 positions NC, spring return, two positions latching", true, [false, true, true], [false, true, true]),
    ("sel3_no_m", "Selector switch 3 positions NO, latching", false, [true, true, true], [false, false, true]),
    ("sel3_nc_m", "Selector switch 3 positions NC, latching", true, [true, true, true], [false, false, true]),
];

/// Position diagram above the operating device: one stroke per position, a dot
/// where the contact is actuated.
fn positions(b: &mut B, actuated: &[bool]) {
    let n = actuated.len();
    for (i, on) in actuated.iter().enumerate() {
        let a = if n == 2 { [-110.0f32, -70.0][i] } else { [-135.0f32, -90.0, -45.0][i] }.to_radians();
        let (c, s) = (a.cos(), a.sin());
        b.l(-6.0, -2.5, -6.0 + c * 2.2, -2.5 + s * 2.2);
        if *on {
            b.circle(-6.0 + c * 2.6, -2.5 + s * 2.6, 0.35, true);
        }
    }
}

pub(super) fn add(v: &mut Vec<SymbolDef>) {
    let mut lib = Lib { v, b: B::new(), kinds: HashMap::new() };
    coils(&mut lib);
    contacts(&mut lib);
    switches(&mut lib);
    sensors(&mut lib);
    protection(&mut lib);
    signals_and_motors(&mut lib);
    measuring(&mut lib);
    power(&mut lib);
    let _ = KINDS.set(lib.kinds);
}

fn coils(lib: &mut Lib) {
    let marks: [Variant; 13] = [
        ("coil_ac", "Relay coil, AC", |b| {
            b.text(-4.5, 0.0, "~", 2.2, Center);
        }),
        ("coil_fast", "Relay coil, high-speed", |b| {
            b.l(-6.0, 0.0, -3.0, 0.0);
        }),
        ("coil_latched", "Relay coil, mechanically latched", |b| {
            b.poly(&[(-3.5, -1.2), (-5.5, 0.0), (-3.5, 1.2)], true, false);
        }),
        ("coil_latching", "Relay coil, latching", |b| {
            b.poly(&[(-5.5, 1.0), (-4.5, 1.0), (-4.5, -1.0), (-3.5, -1.0)], false, false);
        }),
        ("coil_led", "Relay coil with LED", |b| {
            let d = 0.7;
            b.circle(-4.5, 0.0, 1.0, false).l(-4.5 - d, -d, -4.5 + d, d).l(-4.5 - d, d, -4.5 + d, -d);
        }),
        ("coil_resonance", "Relay coil, resonant", |b| {
            b.l(-5.2, -1.4, -5.2, 0.4).l(-3.8, -1.4, -3.8, 0.4).arc(-4.5, 0.4, 0.7, 0.0, 180.0).l(-4.5, 1.1, -4.5, 2.0);
        }),
        ("coil_thermal", "Relay coil, thermal", |b| {
            b.poly(&[(-5.0, -2.0), (-5.0, -0.8), (-4.0, -0.8), (-4.0, 0.8), (-5.0, 0.8), (-5.0, 2.0)], false, false);
        }),
        ("coil_indicator", "Relay coil with indicator", |b| {
            b.circle(-4.5, 0.0, 1.0, false).l(-4.5, -1.0, -4.5, 1.0);
        }),
        ("coil_flashing", "Relay coil, flashing", |b| {
            b.poly(&[(-5.7, 0.8), (-5.1, 0.8), (-5.1, -0.8), (-3.9, -0.8), (-3.9, 0.8), (-3.3, 0.8)], false, false);
        }),
        ("coil_suppressor", "Relay coil with suppressor diode", |b| {
            b.diode_at(p(-4.5, 0.0), p(0.0, -1.0)).l(-4.5, -2.0, -4.5, 2.0);
        }),
        ("coil_remanence", "Relay coil, remanent", |b| {
            b.l(-6.0, 2.0, -3.0, -2.0);
        }),
        ("coil_polarized", "Relay coil, polarized", |b| {
            b.rect(-6.0, -2.0, -4.5, 2.0, true);
        }),
        ("coil_ac_resistant", "Relay coil, AC resistant", |b| {
            b.rect(-6.0, -2.0, -3.0, -1.2, true).rect(-6.0, 1.2, -3.0, 2.0, true);
        }),
    ];
    for (key, name, mark) in marks {
        lib.b.coil_box(true);
        mark(&mut lib.b);
        lib.push(def(key, name, C::Coils, "K", MAIN_MIRROR), Some(K::Coil));
    }

    lib.b.coil_box(true).l(-6.0, -2.0, -3.0, 2.0).l(-6.0, 2.0, -3.0, -2.0).rect(-6.0, -2.0, -4.5, 2.0, true);
    lib.push(def("coil_on_off", "Timer coil, on- and off-delay", C::Coils, "K", MAIN_MIRROR), Some(K::Coil));

    lib.b.coil_box(true).clock_at(-4.5, 0.0, 1.1);
    lib.push(def("coil_clock", "Time switch (electronic)", C::Coils, "K", MAIN_MIRROR), Some(K::Coil));

    for (key, name, t) in [("relay_overvoltage", "Overvoltage relay", "U>"), ("relay_undervoltage", "Undervoltage relay", "U<")] {
        lib.b.coil_box(false).rect(-7.0, -2.0, -3.0, 2.0, false).text(-5.0, 0.0, t, 1.5, Center);
        lib.push(def(key, name, C::Coils, "K", MAIN_MIRROR), Some(K::Coil));
    }
    // Motor operator switching a circuit breaker on (and off), drawn as its drive.
    lib.b.coil_box(false).circle(0.0, 0.0, 1.6, false).text(0.0, 0.0, "M", 1.6, Center);
    lib.push(def("coil_motor_operator", "Motor operator of a circuit breaker", C::Coils, "Q", MAIN_MIRROR), Some(K::Coil));
    // Grid and system protection (VDE-AR-N 4105): voltage and frequency relay measuring
    // the three phases and N; its contacts trip the coupling switch.
    lib.b.rect(-3.0, 0.0, 27.0, 12.0, false).text(12.0, 3.8, "U< U>", 2.0, Center).text(12.0, 8.4, "f< f>", 2.0, Center);
    for (i, label) in ["L1", "L2", "L3", "N"].into_iter().enumerate() {
        let x = i as f32 * 8.0;
        lib.b.l(x, -4.0, x, 0.0).pin(x, -4.0, label);
    }
    lib.push(def("relay_grid_protection", "Grid and system protection relay (voltage and frequency)", C::Measuring, "K", MAIN), Some(K::Generic));

    // Counting devices (IEC 60617-08-07): drive with the counter mechanism.
    lib.b.coil_box(false).rect(3.0, -2.0, 6.0, 2.0, false).circle(4.5, 0.0, 0.8, false);
    lib.push(def("counter_pulse", "Pulse counter", C::Coils, "K", MAIN_MIRROR), Some(K::Coil));

    lib.b.coil_box(false).rect(3.0, -2.0, 6.0, 2.0, false).circle(4.5, 0.0, 0.8, false);
    lib.b.l(6.0, 0.0, 7.5, 0.0).l(7.5, -1.2, 7.5, 1.2).text(4.5, -3.2, "0", 1.6, Center);
    lib.push(def("counter_preset", "Pulse counter, manual reset", C::Coils, "K", MAIN_MIRROR), Some(K::Coil));

    lib.b.coil_box(false).rect(3.0, -2.0, 6.0, 2.0, false).circle(4.5, 0.0, 0.8, false);
    lib.b.l(8.0, -4.0, 8.0, -2.0).rect(6.0, -2.0, 10.0, 2.0, false).l(8.0, 2.0, 8.0, 4.0).text(8.0, 0.0, "R", 1.6, Center);
    lib.b.pin(8.0, -4.0, "R1").pin(8.0, 4.0, "R2");
    lib.push(def("counter_reset", "Pulse counter, electrical reset", C::Coils, "K", MAIN_MIRROR), Some(K::Coil));

    // Preset counters: supply, count and reset inputs, output contacts per stage.
    for (key, name, stages) in [("counter_1stage", "Preset counter, 1 stage", 1), ("counter_2stage", "Preset counter, 2 stages", 2)] {
        const OUT: [&str; 6] = ["15", "16", "18", "25", "26", "28"];
        let w = 16.0 + stages as f32 * 24.0;
        lib.b.rect(-4.0, -4.0, w - 4.0, 4.0, false).text(8.0, 0.0, "n", 2.5, Center);
        for (i, label) in ["A1", "B1", "R"].into_iter().enumerate() {
            let x = i as f32 * 8.0;
            lib.b.l(x, -8.0, x, -4.0).pin(x, -8.0, label);
        }
        lib.b.l(0.0, 4.0, 0.0, 8.0).pin(0.0, 8.0, "A2");
        for (i, label) in OUT.iter().take(stages * 3).enumerate() {
            let x = 16.0 + i as f32 * 8.0;
            lib.b.l(x, 4.0, x, 8.0).pin(x, 8.0, label);
        }
        lib.push(def(key, name, C::Coils, "K", MAIN), None);
    }

    // Electromagnetic actuators.
    lib.b.l(0.0, -4.0, 0.0, -2.0).rect(-3.0, -2.0, 3.0, 2.0, false).l(0.0, 2.0, 0.0, 4.0);
    lib.b.l(-3.0, 0.0, -4.5, 0.0).l(-4.5, -1.5, -4.5, 1.5).l(-5.5, -1.5, -5.5, 1.5).l(-5.5, 0.0, -7.0, 0.0);
    lib.b.pins2("1", "2");
    lib.push(def("clutch", "Clutch (electromagnetic)", C::Loads, "Y", MAIN), Some(K::Brake));

    lib.b.l(0.0, -4.0, 0.0, -2.0).rect(-3.0, -2.0, 3.0, 2.0, false).l(0.0, 2.0, 0.0, 4.0);
    lib.b.l(-3.0, 0.0, -5.0, 0.0).poly(&[(-5.0, -1.0), (-6.0, 0.0), (-5.0, 1.0)], true, false);
    lib.b.pins2("1", "2");
    lib.push(def("solenoid", "Solenoid actuator", C::Loads, "Y", MAIN), Some(K::Valve));
}

fn contacts(lib: &mut Lib) {
    let aux = |k: ContactKind| Func::Aux(k);
    // Early / late operating contacts (IEC 60617-07-04): a stroke on the blade tip.
    for (key, name, nc, early, kind) in [
        ("no_early", "NO contact, early closing", false, true, K::No),
        ("no_late", "NO contact, late closing", false, false, K::No),
        ("nc_early", "NC contact, early opening", true, true, K::Nc),
        ("nc_late", "NC contact, late opening", true, false, K::Nc),
    ] {
        lib.b.contact(nc);
        let (tx, ty) = if nc { (2.3, -2.6) } else { (-2.0, -2.4) };
        let dy = if early { 1.2 } else { -1.2 };
        lib.b.l(tx, ty, tx - 1.0, ty + dy);
        let (a, b, ck) = if nc { ("21", "22", ContactKind::Nc) } else { ("13", "14", ContactKind::No) };
        lib.b.pins2(a, b);
        lib.push(def(key, name, C::Contacts, "K", aux(ck)).auto(), Some(kind));
    }
    // Passing contacts (IEC 60617-07-04-05): close briefly when actuated or released.
    for (key, name, dirs) in [
        ("no_pass", "Passing NO contact, on actuation", [true, false]),
        ("no_pass_rel", "Passing NO contact, on release", [false, true]),
        ("no_pass_both", "Passing NO contact, both directions", [true, true]),
    ] {
        lib.b.no(0.0);
        if dirs[0] {
            lib.b.l(1.2, -2.6, 1.2, -0.4).arrow_head(1.2, -0.2, 0.0, 1.0, 0.9);
        }
        if dirs[1] {
            lib.b.l(2.4, -0.2, 2.4, -2.4).arrow_head(2.4, -2.6, 0.0, -1.0, 0.9);
        }
        lib.b.pins2("13", "14");
        lib.push(def(key, name, C::Contacts, "K", aux(ContactKind::No)).auto(), Some(K::No));
    }
    // Changeover contacts.
    lib.b.changeover();
    lib.b.arc(2.6, -2.0, 1.2, 100.0, 260.0);
    lib.b.pin(0.0, 4.0, "11").pin(4.0, -4.0, "12").pin(0.0, -4.0, "14");
    lib.push(def("co_mbb", "Changeover contact, make before break", C::Contacts, "K", aux(ContactKind::Changeover)).auto(), Some(K::Changeover));

    lib.b.changeover();
    lib.b.l(-1.2, -2.6, -1.2, -0.4).arrow_head(-1.2, -0.2, 0.0, 1.0, 0.9);
    lib.b.pin(0.0, 4.0, "11").pin(4.0, -4.0, "12").pin(0.0, -4.0, "14");
    lib.push(def("co_pass", "Changeover contact, passing make", C::Contacts, "K", aux(ContactKind::Changeover)).auto(), Some(K::Changeover));

    for (key, name, on, off) in [
        ("co_on", "Changeover contact, on-delay", true, false),
        ("co_off", "Changeover contact, off-delay", false, true),
        ("co_on_off", "Changeover contact, on- and off-delay", true, true),
    ] {
        let bx = lib.b.changeover();
        lib.b.delay_mark(bx, on, off);
        lib.b.pin(0.0, 4.0, "15").pin(4.0, -4.0, "16").pin(0.0, -4.0, "18");
        lib.push(def(key, name, C::Contacts, "K", aux(ContactKind::Changeover)).auto(), Some(K::Changeover));
    }
    lib.b.no(0.0).delay_mark(NO_BX, true, true).pins2("17", "18");
    lib.push(def("no_on_off", "NO contact, on- and off-delay", C::Contacts, "K", aux(ContactKind::NoOnDelay)).auto(), Some(K::NoOnDelay));
    lib.b.nc(0.0).delay_mark(1.0, true, true).pins2("15", "16");
    lib.push(def("nc_on_off", "NC contact, on- and off-delay", C::Contacts, "K", aux(ContactKind::NcOnDelay)).auto(), Some(K::NcOnDelay));

    // Contacts of a time switch and of a counting device.
    for (key, name, nc, op) in [
        ("clock_no", "Time switch contact NO", false, Op::Clock),
        ("clock_nc", "Time switch contact NC", true, Op::Clock),
        ("counter_no", "Counter contact NO", false, Op::Counter),
        ("counter_nc", "Counter contact NC", true, Op::Counter),
        ("counter_cam_no", "Counter contact NO, cam operated", false, Op::CounterCam),
        ("counter_cam_nc", "Counter contact NC, cam operated", true, Op::CounterCam),
    ] {
        let bx = lib.b.contact(nc);
        lib.b.op(bx, op, false);
        let (a, b, ck, kind) = if nc { ("21", "22", ContactKind::Nc, K::Nc) } else { ("13", "14", ContactKind::No, K::No) };
        lib.b.pins2(a, b);
        lib.push(def(key, name, C::Contacts, "K", aux(ck)).auto(), Some(kind));
    }

    // Main contacts: NC, with automatic tripping, semiconductor contactor.
    for x in [0.0, 8.0, 16.0] {
        lib.b.nc(x).arc(x + 0.8, -1.3, 0.7, 180.0, 360.0);
    }
    lib.b.dl(0.96, 0.0, 16.96, 0.0).pole_pins(3);
    lib.push(def("main3_nc", "Main contacts 3-pole NC", C::Contacts, "K", aux(ContactKind::MainNo)), Some(K::MainContacts3));

    for x in [0.0, 8.0, 16.0] {
        lib.b.no(x).arc(x, -1.3, 0.7, 180.0, 360.0).trip_mark(x);
    }
    lib.b.dl(-0.5, 0.0, 15.1, 0.0).pole_pins(3);
    lib.push(def("main3_trip", "Main contacts 3-pole, automatic tripping", C::Contacts, "K", aux(ContactKind::MainNo)), Some(K::MainContacts3));

    for x in [0.0, 8.0, 16.0] {
        lib.b.no(x).arc(x, -1.3, 0.7, 180.0, 360.0).l(x - 2.0, 1.0, x + 1.5, 1.0);
    }
    lib.b.dl(-0.9, 0.0, 15.1, 0.0).pole_pins(3);
    lib.push(def("main3_semi", "Semiconductor contactor 3-pole", C::Contacts, "K", aux(ContactKind::MainNo)), Some(K::MainContacts3));
}

fn switches(lib: &mut Lib) {
    for &(key, name, nc, op, latch, letter, kind) in SWITCHES {
        let bx = lib.b.contact(nc);
        lib.b.op(bx, op, latch);
        if nc {
            lib.b.pins2("11", "12");
        } else {
            lib.b.pins2("13", "14");
        }
        lib.push(def(key, name, C::Switches, letter, MAIN), Some(kind));
    }
    // Selector switches with position diagram: A = actuated right, B = actuated left.
    for (key, name, nc, actuated, latch) in [
        ("sel_no_pa", "Selector switch NO, position diagram A", false, [false, true], false),
        ("sel_no_pb", "Selector switch NO, position diagram B", false, [true, false], false),
        ("sel_nc_pa", "Selector switch NC, position diagram A", true, [false, true], false),
        ("sel_nc_pb", "Selector switch NC, position diagram B", true, [true, false], false),
        ("sel_no_m_pd", "Selector switch NO, latching, with position diagram", false, [false, true], true),
        ("sel_nc_m_pd", "Selector switch NC, latching, with position diagram", true, [false, true], true),
    ] {
        let bx = lib.b.contact(nc);
        lib.b.op(bx, Op::Turn, latch);
        positions(&mut lib.b, &actuated);
        let (a, b, kind) = if nc { ("11", "12", K::SwitchNc) } else { ("13", "14", K::SwitchNo) };
        lib.b.pins2(a, b);
        lib.push(def(key, name, C::Switches, "S", MAIN), Some(kind));
    }
    // Three positions: per position a detent (latching) or a return mark.
    for &(key, name, nc, latch, actuated) in SELECTORS3 {
        let bx = lib.b.contact(nc);
        lib.b.dl(bx, 0.0, -11.0, 0.0).turn_at(-11.0);
        for (i, l) in latch.iter().enumerate() {
            let x = -3.0 - i as f32 * 2.6;
            if *l {
                lib.b.poly(&[(x - 0.6, -1.0), (x, 0.0), (x + 0.6, -1.0)], false, false);
            } else {
                lib.b.poly(&[(x - 0.5, -0.9), (x + 0.5, -0.45), (x - 0.5, 0.0)], true, false);
            }
        }
        for (i, on) in actuated.iter().enumerate() {
            let a = [-135.0f32, -90.0, -45.0][i].to_radians();
            let (c, s) = (a.cos(), a.sin());
            lib.b.l(-12.0, -2.5, -12.0 + c * 2.2, -2.5 + s * 2.2);
            if *on {
                lib.b.circle(-12.0 + c * 2.6, -2.5 + s * 2.6, 0.35, true);
            }
        }
        let (a, b, kind) = if nc { ("11", "12", K::SwitchNc) } else { ("13", "14", K::SwitchNo) };
        lib.b.pins2(a, b);
        lib.push(def(key, name, C::Switches, "S", MAIN), Some(kind));
    }
    // Changeover switches operated by cam, roller or cam profile.
    for (key, name, op) in [
        ("co_cam", "Changeover switch, cam operated", Op::Cam),
        ("co_roller", "Changeover switch, roller operated", Op::Roller),
        ("co_camprof", "Changeover switch, cam profile operated", Op::CamProfile),
    ] {
        let bx = lib.b.changeover();
        lib.b.op(bx, op, false);
        lib.b.pin(0.0, 4.0, "11").pin(4.0, -4.0, "12").pin(0.0, -4.0, "14");
        lib.push(def(key, name, C::Switches, "B", MAIN), Some(K::Sensor));
    }
    // A manually operated changeover switch: the two-way switch of an installation.
    let bx = lib.b.changeover();
    lib.b.op(bx, Op::Manual, false);
    lib.b.pin(0.0, 4.0, "11").pin(4.0, -4.0, "12").pin(0.0, -4.0, "14");
    lib.push(def("co_man", "Changeover switch, manually operated", C::Switches, "S", MAIN), Some(K::Changeover));
    // A manually operated intermediate switch: the two blades crossed.
    lib.b.l(0.0, -4.0, 0.0, -2.0).l(4.0, -4.0, 4.0, -2.0).l(0.0, 4.0, 0.0, 2.0).l(4.0, 4.0, 4.0, 2.0);
    lib.b.l(0.0, -2.0, 4.0, 2.0).l(4.0, -2.0, 0.0, 2.0);
    lib.b.op(2.0, Op::Manual, false);
    lib.b.pin(0.0, -4.0, "1").pin(4.0, -4.0, "2").pin(0.0, 4.0, "3").pin(4.0, 4.0, "4");
    lib.push(def("co_inter", "Intermediate switch, manually operated", C::Switches, "S", MAIN), Some(K::Changeover));

    // Disconnectors (IEC 60617-07-13-06), switch-disconnectors (07-13-08) and main
    // switches, 1 to 3 poles.
    let names = [
        ["Disconnector 1-pole", "Disconnector 2-pole", "Disconnector 3-pole"],
        ["Disconnector 1-pole, lockable", "Disconnector 2-pole, lockable", "Disconnector 3-pole, lockable"],
        ["Switch-disconnector 1-pole", "Switch-disconnector 2-pole", "Switch-disconnector 3-pole"],
        ["Switch-disconnector 1-pole, automatic tripping", "Switch-disconnector 2-pole, automatic tripping", "Switch-disconnector 3-pole, automatic tripping"],
        ["Main switch 1-pole", "Main switch 2-pole", ""],
    ];
    let keys = [
        ["isol1", "isol2", "isol3"],
        ["isol1_lock", "isol2_lock", "isol3_lock"],
        ["swdisc1", "swdisc2", "swdisc3"],
        ["swdisc1_trip", "swdisc2_trip", "swdisc3_trip"],
        ["main_sw1", "main_sw2", ""],
    ];
    for (variant, (names, keys)) in names.iter().zip(keys).enumerate() {
        for n in 1..=3 {
            if keys[n - 1].is_empty() {
                continue;
            }
            for i in 0..n {
                let x = i as f32 * 8.0;
                lib.b.no(x).disconnector_bar(x);
                if (2..=3).contains(&variant) {
                    lib.b.switch_circle(x);
                }
                if variant == 3 {
                    lib.b.trip_mark(x);
                }
            }
            match variant {
                1 => {
                    lib.b.op(NO_BX, Op::Lock, false);
                }
                4 => {
                    lib.b.dl(-5.0, 0.0, NO_BX, 0.0).l(-5.0, -1.5, -5.0, 1.5).l(-5.0, -1.5, -6.5, -1.5);
                }
                _ => {}
            }
            if n > 1 {
                lib.b.gang(n, if matches!(variant, 1 | 4) { -5.0 } else { NO_BX }, NO_BX);
            }
            lib.b.pole_pins(n);
            let kind = if n == 3 { K::Disconnector3 } else { K::SwitchNo };
            lib.push(def(keys[n - 1], names[n - 1], C::Switches, "Q", MAIN), Some(kind));
        }
    }
}

fn sensors(lib: &mut Lib) {
    // Electronic sensors: body with the sensing diamond and the kind of detection,
    // supply BN / BU, switching output BK (and WH for antivalent outputs).
    fn body(b: &mut B, outputs: usize, mark: fn(&mut B)) {
        b.rect(-3.0, -5.0, 8.0, 5.0, false);
        b.poly(&[(1.0, -3.8), (2.6, -2.2), (1.0, -0.6), (-0.6, -2.2)], true, false);
        mark(b);
        b.l(0.0, -8.0, 0.0, -5.0).l(0.0, 5.0, 0.0, 8.0).pin(0.0, -8.0, "BN");
        // Switching output: NO contact lying on the output lead.
        b.l(8.0, 0.0, 12.0, 0.0).l(5.0, 0.0, 6.0, 0.0).l(6.0, 0.0, 7.4, -1.2).l(7.4, 0.0, 8.0, 0.0);
        b.pin(12.0, 0.0, "BK");
        if outputs == 2 {
            b.l(8.0, 4.0, 12.0, 4.0).pin(12.0, 4.0, "WH");
        }
        b.pin(0.0, 8.0, "BU");
    }
    // Key, name, "2" for 2-wire sensors, number of outputs, detection mark.
    let marks: [(&str, &str, &str, usize, Mark); 13] = [
        ("prox_ind3", "Inductive sensor, 3-wire", "", 1, |b| {
            b.winding(-1.0, 2.0, 2, false);
        }),
        ("prox_cap3", "Capacitive sensor, 3-wire", "", 1, |b| {
            b.l(-1.0, 1.4, 3.0, 1.4).l(-1.0, 2.6, 3.0, 2.6);
        }),
        ("prox_mag", "Magnetic sensor", "", 1, |b| {
            b.rect(0.2, 1.0, 1.8, 3.4, true);
        }),
        ("touch3", "Touch sensor, 3-wire", "", 1, |b| {
            b.circle(1.0, 2.2, 1.0, false).circle(1.0, 2.2, 0.3, true);
        }),
        ("photo_retro3", "Photoelectric sensor, retro-reflective, 3-wire", "", 1, |b| {
            b.l(-1.0, 1.4, 3.0, 1.4).arrow_head(3.4, 1.4, 1.0, 0.0, 0.8).l(3.0, 3.0, -0.6, 3.0).arrow_head(-1.0, 3.0, -1.0, 0.0, 0.8).l(4.4, 0.8, 4.4, 3.6);
        }),
        ("photo_retro4", "Photoelectric sensor, retro-reflective, 4-wire", "", 2, |b| {
            b.l(-1.0, 1.4, 3.0, 1.4).arrow_head(3.4, 1.4, 1.0, 0.0, 0.8).l(3.0, 3.0, -0.6, 3.0).arrow_head(-1.0, 3.0, -1.0, 0.0, 0.8).l(4.4, 0.8, 4.4, 3.6);
        }),
        ("photo_diffuse3", "Photoelectric sensor, diffuse, 3-wire", "", 1, |b| {
            b.l(-1.0, 1.4, 3.0, 1.0).arrow_head(3.4, 1.0, 1.0, -0.1, 0.8).l(3.0, 2.6, -0.6, 3.2).arrow_head(-1.0, 3.3, -1.0, 0.15, 0.8);
        }),
        ("photo_diffuse4", "Photoelectric sensor, diffuse, 4-wire", "", 2, |b| {
            b.l(-1.0, 1.4, 3.0, 1.0).arrow_head(3.4, 1.0, 1.0, -0.1, 0.8).l(3.0, 2.6, -0.6, 3.2).arrow_head(-1.0, 3.3, -1.0, 0.15, 0.8);
        }),
        ("photo_rx3", "Through-beam receiver, 3-wire", "", 1, |b| {
            b.l(4.0, 2.2, -0.6, 2.2).arrow_head(-1.0, 2.2, -1.0, 0.0, 0.8);
        }),
        ("photo_rx4", "Through-beam receiver, 4-wire", "", 2, |b| {
            b.l(4.0, 2.2, -0.6, 2.2).arrow_head(-1.0, 2.2, -1.0, 0.0, 0.8);
        }),
        ("prox_ind2", "Inductive sensor, 2-wire", "2", 0, |b| {
            b.winding(-1.0, 2.0, 2, false);
        }),
        ("prox_cap2", "Capacitive sensor, 2-wire", "2", 0, |b| {
            b.l(-1.0, 1.4, 3.0, 1.4).l(-1.0, 2.6, 3.0, 2.6);
        }),
        ("touch2", "Touch sensor, 2-wire", "2", 0, |b| {
            b.circle(1.0, 2.2, 1.0, false).circle(1.0, 2.2, 0.3, true);
        }),
    ];
    for (key, name, two_wire, outputs, mark) in marks {
        if two_wire.is_empty() {
            body(&mut lib.b, outputs, mark);
        } else {
            // 2-wire: the output switches the supply current (BN / BU only).
            let b = &mut lib.b;
            b.rect(-3.0, -5.0, 8.0, 5.0, false);
            b.poly(&[(1.0, -3.8), (2.6, -2.2), (1.0, -0.6), (-0.6, -2.2)], true, false);
            mark(b);
            b.l(0.0, -8.0, 0.0, -5.0).l(0.0, 5.0, 0.0, 8.0).l(5.0, -2.0, 6.0, -2.0).l(6.0, -2.0, 7.4, -3.2);
            b.pin(0.0, -8.0, "BN").pin(0.0, 8.0, "BU");
        }
        lib.push(def(key, name, C::Sensors, "B", MAIN), Some(K::Sensor));
    }
    // Transmitter: supply only, sends the beam.
    lib.b.rect(-3.0, -5.0, 8.0, 5.0, false).poly(&[(1.0, -3.8), (2.6, -2.2), (1.0, -0.6), (-0.6, -2.2)], true, false);
    lib.b.l(-1.0, 2.2, 3.6, 2.2).arrow_head(4.0, 2.2, 1.0, 0.0, 0.8);
    lib.b.l(0.0, -8.0, 0.0, -5.0).l(0.0, 5.0, 0.0, 8.0).pin(0.0, -8.0, "BN").pin(0.0, 8.0, "BU");
    lib.push(def("photo_tx", "Through-beam transmitter", C::Sensors, "B", MAIN), Some(K::Sensor));

    // Through-beam sensor: transmitter and receiver in one symbol.
    for (key, name, outputs) in [("photo_thru3", "Through-beam sensor, 3-wire", 1), ("photo_thru4", "Through-beam sensor, 4-wire", 2)] {
        let b = &mut lib.b;
        b.rect(-3.0, -5.0, 4.0, 5.0, false).poly(&[(0.5, -3.8), (2.1, -2.2), (0.5, -0.6), (-1.1, -2.2)], true, false);
        b.l(-1.5, 2.2, 2.6, 2.2).arrow_head(3.0, 2.2, 1.0, 0.0, 0.8);
        b.l(0.0, -8.0, 0.0, -5.0).l(0.0, 5.0, 0.0, 8.0).pin(0.0, -8.0, "BN1").pin(0.0, 8.0, "BU1");
        b.rect(5.0, -5.0, 16.0, 5.0, false).poly(&[(9.0, -3.8), (10.6, -2.2), (9.0, -0.6), (7.4, -2.2)], true, false);
        b.l(12.0, 2.2, 7.4, 2.2).arrow_head(7.0, 2.2, -1.0, 0.0, 0.8);
        b.l(8.0, -8.0, 8.0, -5.0).l(8.0, 5.0, 8.0, 8.0).pin(8.0, -8.0, "BN2").pin(8.0, 8.0, "BU2");
        b.l(16.0, 0.0, 20.0, 0.0).l(13.0, 0.0, 14.0, 0.0).l(14.0, 0.0, 15.4, -1.2).pin(20.0, 0.0, "BK");
        if outputs == 2 {
            b.l(16.0, 4.0, 20.0, 4.0).pin(20.0, 4.0, "WH");
        }
        lib.push(def(key, name, C::Sensors, "B", MAIN), Some(K::Sensor));
    }

    // Temperature controller: sensor input, supply and output contact.
    let b = &mut lib.b;
    b.rect(-4.0, -4.0, 20.0, 4.0, false).text(8.0, 0.0, "θ", 2.5, Center);
    for (i, label) in ["L", "N", "T1", "T2"].into_iter().enumerate() {
        let x = i as f32 * 4.0 + if i >= 2 { 4.0 } else { 0.0 };
        b.l(x, -8.0, x, -4.0).pin(x, -8.0, label);
    }
    for (i, label) in ["11", "12", "14"].into_iter().enumerate() {
        let x = i as f32 * 8.0;
        b.l(x, 4.0, x, 8.0).pin(x, 8.0, label);
    }
    lib.push(def("temp_controller", "Temperature controller", C::Sensors, "K", MAIN), None);
}

fn protection(lib: &mut Lib) {
    let n_names = |s: [&'static str; 3], n: usize| s[n - 1];
    // Circuit breakers with thermal and magnetic release (IEC 60617-07-13-05).
    for n in 1..=3 {
        for i in 0..n {
            lib.b.no(i as f32 * 8.0).breaker_x(i as f32 * 8.0);
        }
        lib.b.gang(n, -5.0, NO_BX).release_tm(-5.0).pole_pins(n);
        let key = n_names(["cb1_tm", "cb2_tm", "cb3_tm"], n);
        let name = n_names(["Circuit breaker 1-pole, thermal-magnetic", "Circuit breaker 2-pole, thermal-magnetic", "Circuit breaker 3-pole, thermal-magnetic"], n);
        let kind = [K::Breaker1, K::Breaker2, K::Breaker3][n - 1];
        lib.push(def(key, name, C::Protection, if n == 3 { "Q" } else { "F" }, MAIN), Some(kind));
    }
    // Motor circuit breakers: rotary drive, thermal and magnetic releases.
    for n in 1..=2 {
        for i in 0..n {
            lib.b.no(i as f32 * 8.0).breaker_x(i as f32 * 8.0);
        }
        lib.b.gang(n, -5.0, NO_BX).release_tm(-5.0).rotary_drive().pole_pins(n);
        let key = ["mcb1", "mcb2"][n - 1];
        let name = ["Motor protection switch 1-pole", "Motor protection switch 2-pole"][n - 1];
        lib.push(def(key, name, C::Protection, "Q", MAIN), Some(K::MotorProtection));
    }
    // Residual current devices with 1 to 3 poles.
    for n in 1..=3 {
        for i in 0..n {
            lib.b.no(i as f32 * 8.0);
        }
        lib.b.rect(-1.5, 2.6, (n - 1) as f32 * 8.0 + 1.5, 3.4, false);
        lib.b.gang(n, -5.0, NO_BX).rect(-11.0, -1.5, -5.0, 1.5, false).text(-8.0, 0.0, "IΔ", 1.6, Center).pole_pins(n);
        let key = n_names(["rcd1", "rcd2", "rcd3"], n);
        let name = n_names(["Residual current device 1-pole", "Residual current device 2-pole", "Residual current device 3-pole"], n);
        lib.push(def(key, name, C::Protection, "F", MAIN), Some(K::ResidualCurrent));
    }
    // Fuses: plain 2-pole, low voltage high breaking capacity (NH) 1 to 3 poles.
    for x in [0.0, 8.0] {
        lib.b.l(x, -4.0, x, 4.0).rect(x - 1.2, -2.6, x + 1.2, 2.6, false);
    }
    lib.b.pole_pins(2);
    lib.push(def("fuse2", "Fuse 2-pole", C::Protection, "F", MAIN), Some(K::Fuse));
    for n in 1..=3 {
        for i in 0..n {
            let x = i as f32 * 8.0;
            lib.b.l(x, -4.0, x, 4.0).rect(x - 1.2, -2.6, x + 1.2, 2.6, false).l(x - 1.2, -1.8, x + 1.2, -1.8).l(x - 1.2, 1.8, x + 1.2, 1.8);
        }
        lib.b.pole_pins(n);
        let key = n_names(["fuse_hbc1", "fuse_hbc2", "fuse_hbc3"], n);
        let name = n_names(["Fuse, LV HBC, 1-pole", "Fuse, LV HBC, 2-pole", "Fuse, LV HBC, 3-pole"], n);
        lib.push(def(key, name, C::Protection, "F", MAIN), Some(if n == 3 { K::Fuse3 } else { K::Fuse }));
    }
    // Fuse switches (IEC 60617-07-21): the fuse is the moving contact.
    // Keys and names by number of poles; disconnector bar, switch circle, breaker.
    type FuseSwitch = ([&'static str; 3], [&'static str; 3], bool, bool, bool);
    let variants: [FuseSwitch; 4] = [
        (["fuse_sw1", "fuse_sw2", "fuse_sw3"], ["Fuse switch 1-pole", "Fuse switch 2-pole", "Fuse switch 3-pole"], false, false, false),
        (["fuse_disc1", "fuse_disc2", "fuse_disc3"], ["Fuse disconnector 1-pole", "Fuse disconnector 2-pole", "Fuse disconnector 3-pole"], true, false, false),
        (
            ["fuse_swdisc1", "fuse_swdisc2", "fuse_swdisc3"],
            ["Fuse switch-disconnector 1-pole", "Fuse switch-disconnector 2-pole", "Fuse switch-disconnector 3-pole"],
            true,
            true,
            false,
        ),
        (
            ["fuse_mcb1", "fuse_mcb2", "fuse_mcb3"],
            [
                "Fuse disconnector with motor circuit breaker 1-pole",
                "Fuse disconnector with motor circuit breaker 2-pole",
                "Fuse disconnector with motor circuit breaker 3-pole",
            ],
            true,
            false,
            true,
        ),
    ];
    for (keys, names, bar, circle, breaker) in variants {
        for n in 1..=3 {
            for i in 0..n {
                let x = i as f32 * 8.0;
                lib.b.fuse_blade(x);
                if bar {
                    lib.b.disconnector_bar(x);
                }
                if circle {
                    lib.b.switch_circle(x);
                }
            }
            if breaker {
                lib.b.gang(n, -5.0, -1.6).release_tm(-5.0).rotary_drive();
            } else {
                lib.b.gang(n, -5.0, -1.6).l(-5.0, -1.5, -5.0, 1.5);
            }
            lib.b.pole_pins(n);
            let kind = if breaker {
                K::MotorProtection
            } else if n == 3 {
                K::Fuse3
            } else {
                K::Fuse
            };
            lib.push(def(keys[n - 1], names[n - 1], C::Protection, if breaker { "Q" } else { "F" }, MAIN), Some(kind));
        }
    }
    // Overload relays: thermal 1 / 2 poles (like `ol3`), thermal and magnetic 1 to 3.
    for n in 1..=2 {
        let w = (n - 1) as f32 * 8.0;
        lib.b.rect(-2.0, -2.5, w + 2.0, 2.5, false);
        for i in 0..n {
            let x = i as f32 * 8.0;
            lib.b.l(x, -4.0, x, -2.5).l(x, 2.5, x, 4.0);
            lib.b.poly(&[(x, -2.5), (x, -1.0), (x + 1.2, -1.0), (x + 1.2, 1.0), (x, 1.0), (x, 2.5)], false, false);
        }
        lib.b.pole_pins(n);
        let key = ["ol1", "ol2"][n - 1];
        let name = ["Thermal overload relay 1-pole", "Thermal overload relay 2-pole"][n - 1];
        lib.push(def(key, name, C::Protection, "F", MAIN_MIRROR), Some(K::Overload));
    }
    for n in 1..=3 {
        let w = (n - 1) as f32 * 8.0;
        lib.b.rect(-3.0, -3.0, w + 3.0, 3.0, false).l(-3.0, 0.0, w + 3.0, 0.0);
        for i in 0..n {
            let x = i as f32 * 8.0;
            lib.b.l(x, -4.0, x, -3.0).l(x, 3.0, x, 4.0);
            lib.b.poly(&[(x - 1.0, -1.0), (x - 0.2, -1.0), (x - 0.2, -2.2), (x + 0.8, -2.2), (x + 0.8, -1.0), (x + 1.6, -1.0)], false, false);
            lib.b.text(x, 1.5, "I>", 1.5, Center);
        }
        lib.b.pole_pins(n);
        let key = n_names(["olm1", "olm2", "olm3"], n);
        let name = n_names(["Overload relay 1-pole, thermal and magnetic", "Overload relay 2-pole, thermal and magnetic", "Overload relay 3-pole, thermal and magnetic"], n);
        lib.push(def(key, name, C::Protection, "F", MAIN_MIRROR), Some(K::Overload));
    }
}

fn signals_and_motors(lib: &mut Lib) {
    let d = 2.5 * std::f32::consts::FRAC_1_SQRT_2;
    let lamp = |b: &mut B, cx: f32| {
        b.circle(cx, 0.0, 2.5, false).l(cx - d, -d, cx + d, d).l(cx - d, d, cx + d, -d);
    };
    lamp(&mut lib.b, 0.0);
    lib.b.l(0.0, -4.0, 0.0, -2.5).l(0.0, 2.5, 0.0, 4.0);
    lib.b.poly(&[(3.2, 0.6), (3.8, 0.6), (3.8, -0.6), (4.6, -0.6), (4.6, 0.6), (5.2, 0.6)], false, false);
    lib.b.pins2("X1", "X2");
    lib.push(def("lamp_flash", "Indicator lamp, flashing", C::Loads, "P", MAIN), Some(K::Signal));

    lamp(&mut lib.b, 3.0);
    lib.b.circle(0.0, 0.0, 2.5, false).l(0.0, -4.0, 0.0, -2.5).l(0.0, 2.5, 0.0, 4.0);
    lib.b.pins2("X1", "X2");
    lib.push(def("lamp_trafo", "Indicator lamp with transformer", C::Loads, "P", MAIN), Some(K::Signal));

    lamp(&mut lib.b, 0.0);
    lib.b.l(0.0, -4.0, 0.0, -2.5).l(0.0, 2.5, 0.0, 4.0).arc(0.0, 0.0, 3.6, 200.0, 330.0).arrow_head(3.4, -1.6, 0.5, 0.9, 0.9);
    lib.b.pins2("X1", "X2");
    lib.push(def("beacon", "Beacon, rotating", C::Loads, "P", MAIN), Some(K::Signal));

    lib.b.l(0.0, -4.0, 0.0, -2.0).l(0.0, 2.0, 0.0, 4.0).rect(0.0, -2.0, 1.5, 2.0, false).poly(&[(1.5, -2.0), (4.0, -3.5), (4.0, 3.5), (1.5, 2.0)], false, false);
    lib.b.pins2("1", "2");
    lib.push(def("siren", "Siren", C::Loads, "P", MAIN), Some(K::Signal));

    lib.b.l(0.0, -4.0, 0.0, 4.0).l(0.0, -2.5, 1.0, -2.5).l(0.0, 2.5, 1.0, 2.5).arc(1.0, 0.0, 2.5, 270.0, 450.0);
    lib.b.pins2("1", "2");
    lib.push(def("buzzer", "Buzzer", C::Loads, "P", MAIN), Some(K::Signal));

    // DC motors.
    lib.b.motor1_body("M", "=");
    // Shunt field winding next to the armature.
    lib.b.l(16.0, -4.0, 16.0, 14.0).l(20.0, -4.0, 20.0, 14.0).winding(16.0, 14.0, 2, true);
    lib.b.pin(0.0, -4.0, "A1").pin(8.0, -4.0, "A2").pin(16.0, -4.0, "E1").pin(20.0, -4.0, "E2");
    lib.push(def("motor_dc_shunt", "Motor DC, shunt-wound", C::Loads, "M", MAIN), Some(K::MotorDc));

    lib.b.motor3_body("M", "EC", false).pin(0.0, -4.0, "U").pin(8.0, -4.0, "V").pin(16.0, -4.0, "W");
    lib.push(def("motor_ec", "Motor, electronically commutated (brushless DC)", C::Loads, "M", MAIN), Some(K::MotorDc));

    // 1-phase AC motors.
    lib.b.motor1_body("M", "1~");
    lib.b.pin(0.0, -4.0, "1").pin(8.0, -4.0, "2");
    lib.push(def("motor1_2", "Motor 1-phase, 2 terminals", C::Loads, "M", MAIN), Some(K::Motor1));

    lib.b.motor1_body("M", "1~").vwinding(0.0, -2.0, 2);
    lib.b.pin(0.0, -4.0, "1").pin(8.0, -4.0, "2");
    lib.push(def("motor1_series", "Motor 1-phase, series-wound (universal)", C::Loads, "M", MAIN), Some(K::Motor1));

    lib.b.motor1_body("M", "1~").l(16.0, -4.0, 16.0, 4.0).l(16.0, 4.0, 8.53, 5.89).l(24.0, -4.0, 24.0, 12.0).l(24.0, 12.0, 8.53, 10.11);
    lib.b.pin(0.0, -4.0, "1").pin(8.0, -4.0, "2").pin(16.0, -4.0, "3").pin(24.0, -4.0, "4");
    lib.push(def("motor1_aux", "Motor 1-phase with auxiliary winding", C::Loads, "M", MAIN), Some(K::Motor1));

    lib.b.motor1_body("M", "1~").fan_at(4.0, 14.0);
    lib.b.pin(0.0, -4.0, "1").pin(8.0, -4.0, "2");
    lib.push(def("fan1", "Fan 1-phase", C::Loads, "M", MAIN), Some(K::Motor1));

    // 3-phase AC motors.
    lib.b.motor3_body("M", "3~", true).pe_lead(24.0, 8.0, 8.0, 6.0);
    lib.b.pin(0.0, -4.0, "U1").pin(8.0, -4.0, "V1").pin(16.0, -4.0, "W1").pin(0.0, 20.0, "U2").pin(8.0, 20.0, "V2").pin(16.0, 20.0, "W2");
    lib.push(def("motor3_6", "Motor 3-phase, 6 terminals", C::Loads, "M", MAIN), Some(K::Motor3));

    lib.b.motor3_body("M", "3~", false).vwinding(0.0, -3.0, 1).vwinding(8.0, -3.0, 1).vwinding(16.0, -3.0, 1).pe_lead(24.0, 8.0, 8.0, 6.0);
    lib.b.pin(0.0, -4.0, "U").pin(8.0, -4.0, "V").pin(16.0, -4.0, "W");
    lib.push(def("motor3_series", "Motor 3-phase, series-wound", C::Loads, "M", MAIN), Some(K::Motor3));

    lib.b.motor3_body("M", "3~", true).circle(8.0, 8.0, 4.8, false).pe_lead(24.0, 8.0, 8.0, 6.0);
    lib.b.pin(0.0, -4.0, "U").pin(8.0, -4.0, "V").pin(16.0, -4.0, "W").pin(0.0, 20.0, "K").pin(8.0, 20.0, "L").pin(16.0, 20.0, "M");
    lib.push(def("motor3_slip", "Motor 3-phase, slip-ring rotor", C::Loads, "M", MAIN), Some(K::Motor3));

    lib.b.motor3_body("M", "3~", false).l(1.0, 15.0, 15.0, 15.0).arrow_head(16.0, 15.0, 1.0, 0.0, 1.0).arrow_head(0.0, 15.0, -1.0, 0.0, 1.0).pe_lead(24.0, 8.0, 8.0, 6.0);
    lib.b.pin(0.0, -4.0, "U").pin(8.0, -4.0, "V").pin(16.0, -4.0, "W");
    lib.push(def("motor3_linear", "Linear motor 3-phase", C::Loads, "M", MAIN), Some(K::Motor3));

    lib.b.motor3_body("M", "3~", false).fan_at(8.0, 14.0).pe_lead(24.0, 8.0, 8.0, 6.0);
    lib.b.pin(0.0, -4.0, "U").pin(8.0, -4.0, "V").pin(16.0, -4.0, "W");
    lib.push(def("fan3", "Fan 3-phase", C::Loads, "M", MAIN), Some(K::Motor3));

    lib.b.motor3_body("M", "3~", true).fan_at(-6.0, 6.0).pe_lead(24.0, 8.0, 8.0, 6.0);
    lib.b.pin(0.0, -4.0, "U1").pin(8.0, -4.0, "V1").pin(16.0, -4.0, "W1").pin(0.0, 20.0, "U2").pin(8.0, 20.0, "V2").pin(16.0, 20.0, "W2");
    lib.push(def("fan3_6", "Fan 3-phase, 6 terminals", C::Loads, "M", MAIN), Some(K::Motor3));
}

fn measuring(lib: &mut Lib) {
    let indicating: [(&str, &str, &str, &str); 12] = [
        ("ammeter_reactive", "Ammeter, reactive current", "A", "I sin φ"),
        ("varmeter", "Varmeter", "var", ""),
        ("freqmeter", "Frequency meter", "Hz", ""),
        ("voltmeter_diff", "Voltmeter, differential", "V", "ΔU"),
        ("salinity_meter", "Salinity meter", "NaCl", ""),
        ("tachometer", "Tachometer", "n", ""),
        ("pf_meter", "Power factor meter", "cos φ", ""),
        ("phase_meter", "Phase meter", "φ", ""),
        ("wavemeter", "Wavemeter", "λ", ""),
        ("thermometer", "Thermometer / pyrometer", "θ", ""),
        ("wattmeter", "Wattmeter", "W", ""),
        ("ohmmeter", "Ohmmeter", "Ω", ""),
    ];
    for (key, name, t, sub) in indicating {
        lib.b.meter(t, sub);
        lib.push(def(key, name, C::Measuring, "P", MAIN), Some(K::Meter));
    }
    lib.b.meter("", "").l(0.0, 1.8, 0.0, -0.6).arrow_head(0.0, -1.8, 0.0, -1.0, 1.2);
    lib.push(def("galvanometer", "Galvanometer", C::Measuring, "P", MAIN), Some(K::Meter));

    // Recording instruments (IEC 60617-08-03): square.
    lib.b.rect(-4.0, -4.0, 4.0, 4.0, false).text(0.0, 0.0, "W", 3.0, Center);
    lib.b.l(0.0, -8.0, 0.0, -4.0).l(0.0, 4.0, 0.0, 8.0).l(-8.0, 0.0, -4.0, 0.0).l(4.0, 0.0, 8.0, 0.0);
    lib.b.pin(0.0, -8.0, "1").pin(8.0, 0.0, "2").pin(0.0, 8.0, "3").pin(-8.0, 0.0, "4");
    lib.push(def("wattmeter_rec", "Wattmeter, recording", C::Measuring, "P", MAIN), Some(K::Meter));

    lib.b.rect(-4.0, -4.0, 12.0, 4.0, false).l(4.0, -4.0, 4.0, 4.0).text(0.0, 0.0, "W", 3.0, Center).text(8.0, 0.0, "var", 2.0, Center);
    lib.b.l(0.0, -8.0, 0.0, -4.0).l(8.0, -8.0, 8.0, -4.0).l(12.0, 0.0, 16.0, 0.0).l(8.0, 4.0, 8.0, 8.0).l(0.0, 4.0, 0.0, 8.0).l(-8.0, 0.0, -4.0, 0.0);
    lib.b.pin(0.0, -8.0, "1").pin(8.0, -8.0, "2").pin(16.0, 0.0, "3").pin(8.0, 8.0, "4").pin(0.0, 8.0, "5").pin(-8.0, 0.0, "6");
    lib.push(def("watt_var_rec", "Wattmeter and varmeter, recording", C::Measuring, "P", MAIN), Some(K::Meter));

    // Integrating instruments (energy meters, IEC 60617-08-04).
    for (key, name, t) in [("hour_meter", "Hour meter", "h"), ("ah_meter", "Ampere-hour meter", "Ah"), ("wh_meter", "Watt-hour meter", "Wh"), ("varh_meter", "Var-hour meter", "varh")] {
        lib.b.int_meter(t, false);
        lib.push(def(key, name, C::Measuring, "P", MAIN), Some(K::Meter));
    }
    let variants: [Variant; 11] = [
        ("wh_uni", "Watt-hour meter, one direction", |b| {
            b.l(-2.0, -2.75, 1.6, -2.75).arrow_head(2.4, -2.75, 1.0, 0.0, 0.9);
        }),
        ("wh_from", "Watt-hour meter, energy from the busbar", |b| {
            b.l(-2.0, -3.6, -2.0, -1.9).l(-2.0, -2.75, 1.6, -2.75).arrow_head(2.4, -2.75, 1.0, 0.0, 0.9);
        }),
        ("wh_to", "Watt-hour meter, energy to the busbar", |b| {
            b.l(-2.0, -3.6, -2.0, -1.9).l(-1.2, -2.75, 2.4, -2.75).arrow_head(-2.0, -2.75, -1.0, 0.0, 0.9);
        }),
        ("wh_from_to", "Watt-hour meter, energy from and to the busbar", |b| {
            b.l(-2.6, -3.6, -2.6, -1.9).l(-1.8, -2.75, 1.6, -2.75).arrow_head(2.4, -2.75, 1.0, 0.0, 0.9).arrow_head(-2.6, -2.75, -1.0, 0.0, 0.9);
        }),
        ("wh_multi", "Watt-hour meter, multi-rate", |b| {
            b.l(-4.0, -2.75, 4.0, -2.75);
        }),
        ("wh_excess", "Watt-hour meter, excess energy", |b| {
            b.text(0.0, 2.9, "P>", 1.3, Center);
        }),
        ("wh_tx", "Watt-hour meter with transmitter", |b| {
            b.l(4.0, -2.75, 6.6, -2.75).arrow_head(7.4, -2.75, 1.0, 0.0, 0.9);
        }),
        ("wh_slave", "Watt-hour meter, slave (repeater)", |b| {
            b.l(-7.0, -2.75, -4.8, -2.75).arrow_head(-4.0, -2.75, 1.0, 0.0, 0.9);
        }),
        ("wh_slave_print", "Watt-hour meter, slave with printer", |b| {
            b.l(-7.0, -2.75, -4.8, -2.75).arrow_head(-4.0, -2.75, 1.0, 0.0, 0.9).l(-3.0, 3.0, 3.0, 3.0);
        }),
        ("wh_maxdem", "Watt-hour meter with maximum demand indicator", |b| {
            b.text(0.0, 2.9, "Pmax", 1.3, Center);
        }),
        ("wh_maxrec", "Watt-hour meter with maximum demand recorder", |b| {
            b.text(0.0, 2.9, "Pmax", 1.3, Center).l(-3.0, 3.8, 3.0, 3.8);
        }),
    ];
    for (key, name, mark) in variants {
        lib.b.int_meter("Wh", true);
        mark(&mut lib.b);
        lib.push(def(key, name, C::Measuring, "P", MAIN), Some(K::Meter));
    }

    // Current transformers with two secondaries: two cores, or one core.
    for (key, name, cores) in [("ct_2core", "Current transformer, 2 cores", 2), ("ct_2wind", "Current transformer, 2 secondary windings", 1)] {
        let b = &mut lib.b;
        b.l(0.0, -16.0, 0.0, 16.0);
        // Secondaries: core centre y, radius, S1 / S2 heights.
        let secondaries: [(f32, f32, f32, f32); 2] = if cores == 2 {
            b.circle(0.0, -5.0, 2.5, false).circle(0.0, 5.0, 2.5, false);
            [(-5.0, 2.5, -12.0, -4.0), (5.0, 2.5, 4.0, 12.0)]
        } else {
            b.circle(0.0, 0.0, 3.0, false);
            [(-1.0, 2.0, -12.0, -4.0), (1.0, 2.0, 4.0, 12.0)]
        };
        for (cy, r, y1, y2) in secondaries {
            let c = p(0.0, cy);
            b.lead(c, r, p(4.0, y1), p(8.0, y1)).lead(c, r, p(4.0, y2), p(8.0, y2));
        }
        b.pin(0.0, -16.0, "P1").pin(0.0, 16.0, "P2");
        b.pin(8.0, -12.0, "1S1").pin(8.0, -4.0, "1S2").pin(8.0, 4.0, "2S1").pin(8.0, 12.0, "2S2");
        lib.push(def(key, name, C::Measuring, "T", MAIN), Some(K::CurrentTransformer));
    }
}

fn power(lib: &mut Lib) {
    // Converters (like `psu`): input above, output below.
    lib.b.converter(19.0, "=", "~");
    lib.b.l(0.0, -4.0, 0.0, 0.0).l(8.0, -4.0, 8.0, 0.0).l(0.0, 12.0, 0.0, 16.0).l(8.0, 12.0, 8.0, 16.0).l(16.0, 12.0, 16.0, 16.0);
    lib.b.pin(0.0, -4.0, "L+").pin(8.0, -4.0, "M").pin(0.0, 16.0, "L").pin(8.0, 16.0, "N").pin(16.0, 16.0, "PE");
    lib.push(def("psu_dcac", "Inverter DC/AC", C::Power, "T", MAIN), Some(K::PowerSupply));

    lib.b.converter(11.0, "=", "=");
    lib.b.l(0.0, -4.0, 0.0, 0.0).l(8.0, -4.0, 8.0, 0.0).l(0.0, 12.0, 0.0, 16.0).l(8.0, 12.0, 8.0, 16.0);
    lib.b.pin(0.0, -4.0, "L+").pin(8.0, -4.0, "M").pin(0.0, 16.0, "L+'").pin(8.0, 16.0, "M'");
    lib.push(def("psu_dcdc", "DC/DC converter", C::Power, "T", MAIN), Some(K::PowerSupply));

    lib.b.converter(19.0, "~", "=").l(0.8, 10.8, 4.2, 10.8).arrow_head(0.0, 10.8, -1.0, 0.0, 0.9).arrow_head(5.0, 10.8, 1.0, 0.0, 0.9);
    lib.b.l(0.0, -4.0, 0.0, 0.0).l(8.0, -4.0, 8.0, 0.0).l(16.0, -4.0, 16.0, 0.0).l(0.0, 12.0, 0.0, 16.0).l(8.0, 12.0, 8.0, 16.0);
    lib.b.pin(0.0, -4.0, "L").pin(8.0, -4.0, "N").pin(16.0, -4.0, "PE").pin(0.0, 16.0, "L+").pin(8.0, 16.0, "M");
    lib.push(def("psu_bidir", "Rectifier / inverter AC/DC, bidirectional", C::Power, "T", MAIN), Some(K::PowerSupply));

    // Bridge rectifiers: diamond with four diodes, and the box forms B, C and D.
    let (top, right, bottom, left) = (p(0.0, -6.0), p(6.0, 0.0), p(0.0, 6.0), p(-6.0, 0.0));
    lib.b.poly(&[(0.0, -6.0), (6.0, 0.0), (0.0, 6.0), (-6.0, 0.0)], true, false);
    for (a, c) in [(top, right), (bottom, right), (left, top), (left, bottom)] {
        let d = (c - a) * (1.0 / (c - a).len());
        lib.b.diode_at((a + c) * 0.5, d);
    }
    lib.b.l(0.0, -8.0, 0.0, -6.0).l(0.0, 6.0, 0.0, 8.0).l(6.0, 0.0, 8.0, 0.0).l(-8.0, 0.0, -6.0, 0.0);
    lib.b.pin(0.0, -8.0, "~1").pin(0.0, 8.0, "~2").pin(8.0, 0.0, "+").pin(-8.0, 0.0, "-");
    lib.push(def("rect_bridge", "Bridge rectifier", C::Power, "T", MAIN), Some(K::PowerSupply));

    lib.b.rect(-4.0, -4.0, 12.0, 4.0, false).diode_at(p(4.0, 0.0), p(0.0, 1.0)).l(4.0, -2.5, 4.0, 2.5);
    lib.b.l(0.0, -8.0, 0.0, -4.0).l(8.0, -8.0, 8.0, -4.0).l(0.0, 4.0, 0.0, 8.0).l(8.0, 4.0, 8.0, 8.0);
    lib.b.text(-2.5, -2.6, "~", 1.6, Center).text(10.5, -2.6, "~", 1.6, Center).text(-2.5, 2.6, "+", 1.6, Center).text(10.5, 2.6, "-", 1.6, Center);
    lib.b.pin(0.0, -8.0, "~1").pin(8.0, -8.0, "~2").pin(0.0, 8.0, "+").pin(8.0, 8.0, "-");
    lib.push(def("rect_b", "Bridge rectifier, form B", C::Power, "T", MAIN), Some(K::PowerSupply));

    lib.b.rect(-4.0, -4.0, 4.0, 4.0, false).diode_at(p(0.0, 0.0), p(1.0, 0.0)).l(-2.5, 0.0, 2.5, 0.0);
    lib.b.l(0.0, -8.0, 0.0, -4.0).l(0.0, 4.0, 0.0, 8.0).l(4.0, 0.0, 8.0, 0.0).l(-8.0, 0.0, -4.0, 0.0);
    lib.b.pin(0.0, -8.0, "~1").pin(0.0, 8.0, "~2").pin(8.0, 0.0, "+").pin(-8.0, 0.0, "-");
    lib.push(def("rect_c", "Bridge rectifier, form C", C::Power, "T", MAIN), Some(K::PowerSupply));

    lib.b.poly(&[(0.0, -6.0), (6.0, 0.0), (0.0, 6.0), (-6.0, 0.0)], true, false).diode_at(p(0.0, 0.0), p(1.0, 0.0)).l(-2.5, 0.0, 2.5, 0.0);
    lib.b.l(0.0, -8.0, 0.0, -6.0).l(0.0, 6.0, 0.0, 8.0).l(6.0, 0.0, 8.0, 0.0).l(-8.0, 0.0, -6.0, 0.0);
    lib.b.pin(0.0, -8.0, "~1").pin(0.0, 8.0, "~2").pin(8.0, 0.0, "+").pin(-8.0, 0.0, "-");
    lib.push(def("rect_d", "Bridge rectifier, form D", C::Power, "T", MAIN), Some(K::PowerSupply));

    // 1-phase transformers in winding form: primary above, secondary below.
    let unit = |b: &mut B, x: f32, taps_p: &[f32], taps_s: &[f32]| {
        b.l(x, -8.0, x, -1.5).l(x + 8.0, -8.0, x + 8.0, -1.5).winding(x, -1.5, 4, true);
        b.l(x, 8.0, x, 1.5).l(x + 8.0, 8.0, x + 8.0, 1.5).winding(x, 1.5, 4, false);
        for t in taps_p {
            b.l(x + t, -8.0, x + t, -0.5);
        }
        for t in taps_s {
            b.l(x + t, 8.0, x + t, 0.5);
        }
    };
    let pins1 = |b: &mut B| {
        b.pin(0.0, -8.0, "1L").pin(8.0, -8.0, "1N").pin(0.0, 8.0, "2L").pin(8.0, 8.0, "2N");
    };
    unit(&mut lib.b, 0.0, &[], &[]);
    lib.b.l(-1.0, 3.0, 8.4, -2.4).arrow_head(9.2, -2.9, 0.87, -0.5, 1.0);
    pins1(&mut lib.b);
    lib.push(def("trafo1_var", "Transformer 1-phase, variable coupling", C::Power, "T", MAIN), Some(K::Transformer));

    unit(&mut lib.b, 0.0, &[], &[]);
    lib.b.dl(-1.0, 0.0, 12.0, 0.0);
    pins1(&mut lib.b);
    lib.b.pin(12.0, 0.0, "E");
    lib.push(def("trafo1_screen", "Transformer 1-phase with screen", C::Power, "T", MAIN), Some(K::Transformer));

    unit(&mut lib.b, 0.0, &[], &[4.0]);
    pins1(&mut lib.b);
    lib.b.pin(4.0, 8.0, "2M");
    lib.push(def("trafo1_ct", "Transformer 1-phase, centre tap", C::Power, "T", MAIN), Some(K::Transformer));

    unit(&mut lib.b, 0.0, &[4.0], &[4.0]);
    pins1(&mut lib.b);
    lib.b.pin(4.0, -8.0, "1T").pin(4.0, 8.0, "2T");
    lib.push(def("trafo1_taps", "Transformer 1-phase, multiple taps", C::Power, "T", MAIN), Some(K::Transformer));

    unit(&mut lib.b, 0.0, &[], &[]);
    lib.b.l(12.0, 8.0, 12.0, 1.5).l(20.0, 8.0, 20.0, 1.5).winding(12.0, 1.5, 4, false).l(-1.0, 0.0, 21.0, 0.0);
    pins1(&mut lib.b);
    lib.b.pin(12.0, 8.0, "3L").pin(20.0, 8.0, "3N");
    lib.push(def("trafo_3w", "Transformer, 3 windings", C::Power, "T", MAIN), Some(K::Transformer));

    // Autotransformers: one winding with a tap.
    for (key, name, adjustable) in [("autotrafo1", "Autotransformer 1-phase", false), ("autotrafo1_var", "Autotransformer 1-phase, adjustable", true)] {
        let b = &mut lib.b;
        b.l(0.0, -8.0, 0.0, -1.5).l(16.0, -8.0, 16.0, 8.0).winding(0.0, -1.5, 8, true);
        if adjustable {
            b.l(8.0, 8.0, 8.0, 1.8).arrow_head(8.0, 0.6, 0.0, -1.0, 1.2);
        } else {
            b.l(8.0, 8.0, 8.0, -0.5);
        }
        b.pin(0.0, -8.0, "1L").pin(16.0, -8.0, "N").pin(8.0, 8.0, "2L").pin(16.0, 8.0, "N'");
        lib.push(def(key, name, C::Power, "T", MAIN), Some(K::Transformer));
    }

    // 3-phase transformers in circle form with the connection of each winding.
    let three: [(&str, &str, &[Wnd]); 8] = [
        ("trafo3", "Transformer 3-phase", &[Wnd::None, Wnd::None]),
        ("trafo3_dd", "Transformer 3-phase, delta / delta", &[Wnd::Delta, Wnd::Delta]),
        ("trafo3_yy", "Transformer 3-phase, star / star", &[Wnd::Star, Wnd::Star]),
        ("trafo3_dy", "Transformer 3-phase, delta / star", &[Wnd::Delta, Wnd::Star]),
        ("trafo3_yd", "Transformer 3-phase, star / delta", &[Wnd::Star, Wnd::Delta]),
        ("trafo3_yz", "Transformer 3-phase, star / zigzag", &[Wnd::Star, Wnd::Zigzag]),
        ("trafo3_yyd", "Transformer 3-phase, star / star / delta", &[Wnd::Star, Wnd::Delta, Wnd::Star]),
        ("trafo3_taps", "Transformer 3-phase, multiple taps", &[Wnd::None, Wnd::None]),
    ];
    for (key, name, marks) in three {
        let c = lib.b.trafo3_circles(marks);
        if marks.len() == 3 {
            for (y, label) in [(0.0, "3U"), (4.0, "3V"), (8.0, "3W")] {
                lib.b.lead(c[1], 5.0, p(20.0, y), p(24.0, y)).pin(24.0, y, label);
            }
        }
        if key == "trafo3_taps" {
            for (y, label) in [(-8.0, "1T1"), (-4.0, "1T2")] {
                lib.b.lead(c[0], 5.0, p(20.0, y), p(24.0, y)).pin(24.0, y, label);
            }
        }
        lib.push(def(key, name, C::Power, "T", MAIN), Some(K::Transformer));
    }

    // 3-phase autotransformers: one tapped winding per phase, star point (or open).
    for (key, name, star) in [("autotrafo3", "Autotransformer 3-phase", false), ("autotrafo3_star", "Autotransformer 3-phase, star connection", true)] {
        const IN: [&str; 3] = ["1U", "1V", "1W"];
        const OUT: [&str; 3] = ["2U", "2V", "2W"];
        const N: [&str; 3] = ["1U2", "1V2", "1W2"];
        for i in 0..3 {
            let x = i as f32 * 24.0;
            let b = &mut lib.b;
            b.l(x, -8.0, x, -1.5).winding(x, -1.5, 6, true).l(x + 12.0, -1.5, x + 12.0, 4.0);
            b.l(x + 4.0, -1.5, x + 4.0, 8.0);
            b.pin(x, -8.0, IN[i]).pin(x + 4.0, 8.0, OUT[i]);
            if !star {
                b.l(x + 12.0, 4.0, x + 12.0, 8.0).pin(x + 12.0, 8.0, N[i]);
            }
        }
        if star {
            lib.b.l(12.0, 4.0, 60.0, 4.0).l(60.0, 4.0, 60.0, 8.0).pin(60.0, 8.0, "N");
        }
        lib.push(def(key, name, C::Power, "T", MAIN), Some(K::Transformer));
    }

    // Bank of three 1-phase transformers.
    const BANK: [[&str; 4]; 3] = [["1U1", "1U2", "2U1", "2U2"], ["1V1", "1V2", "2V1", "2V2"], ["1W1", "1W2", "2W1", "2W2"]];
    for (i, labels) in BANK.iter().enumerate() {
        let x = i as f32 * 16.0;
        unit(&mut lib.b, x, &[], &[]);
        lib.b.pin(x, -8.0, labels[0]).pin(x + 8.0, -8.0, labels[1]).pin(x, 8.0, labels[2]).pin(x + 8.0, 8.0, labels[3]);
    }
    lib.push(def("trafo3_bank", "Transformer bank of three 1-phase transformers", C::Power, "T", MAIN), Some(K::Transformer));
}

#[cfg(test)]
mod tests;
