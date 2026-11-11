//! Models of devices of the bundled manufacturers' catalogs after their product photos:
//! housing colours and the parts that make them recognisable (corner caps, handles,
//! displays, windows, terminals, labels) where they are on the device. Positions are
//! fractions of the part's width (x), height (y, up) and depth (z, towards the front).

use super::*;
use crate::{ConnectionPoint, Exit};

const VICTRON_FRONT: [u8; 3] = [36, 108, 190];
const ORANGE: [u8; 3] = [232, 122, 40];
const HAGER_BLUE: [u8; 3] = [30, 112, 196];
const SILVER: [u8; 3] = [196, 200, 206];
const LIGHT: [u8; 3] = [226, 227, 228];
const DEHN_RED: [u8; 3] = [196, 34, 30];
const LCD: [u8; 3] = [150, 168, 150];
const GREEN: [u8; 3] = [64, 150, 72];
const AMBER: [u8; 3] = [236, 170, 40];
const SCREEN: [u8; 3] = [60, 124, 220];

/// A model drawn in fractions of the part's box; elements in front (reaching z 0.85)
/// are drawn in the front view as well. Terminals are drawn where the part's connection
/// points are.
pub(super) struct Frac {
    pub m: Mesh3d,
    s: [f32; 3],
    pub prims: Vec<Prim2d>,
    points: Vec<ConnectionPoint>,
}

impl Frac {
    pub(super) fn new(w: f32, h: f32, d: f32, points: Vec<ConnectionPoint>) -> Frac {
        Frac { m: Mesh3d::default(), s: [w, h, d], prims: Vec::new(), points }
    }

    fn at(&self, q: [f32; 3]) -> [f32; 3] {
        [q[0] * self.s[0], q[1] * self.s[1], q[2] * self.s[2]]
    }

    /// The connection points leaving in direction `exit`, as fractions (x, y up).
    fn points_to(&self, exit: Exit) -> Vec<(f32, f32)> {
        let [w, h, _] = self.s;
        self.points.iter().filter(|c| c.exit == exit).map(|c| (c.pos[0] / w, 1.0 - c.pos[1] / h)).collect()
    }

    /// Spring contacts in the front at the connection points leaving there: the wire
    /// entry and the release tab above it.
    fn springs(&mut self, z0: f32) {
        let [w, h, _] = self.s;
        let (dx, dy) = (2.2 / w, 1.0 / h);
        for (x, y) in self.points_to(Exit::Front) {
            self.b([x - dx, y - 2.4 * dy, z0], [x + dx, y + 0.6 * dy, 1.0], DARK);
            self.b([x - dx, y + 0.9 * dy, z0], [x + dx, y + 2.4 * dy, 0.99], ORANGE);
        }
    }

    /// The connection points leaving in direction `exit`.
    fn leaving(&self, exit: Exit) -> Vec<ConnectionPoint> {
        self.points.iter().filter(|c| c.exit == exit).cloned().collect()
    }

    /// A block across the part from `y0` to `y1` and `z0` to `z1` (fractions) with the
    /// wire entries of the connection points leaving up through its upper end (`up`) or
    /// down through its lower end, as [`crate::catalog::block_with_entries`].
    fn with_entries(&mut self, (y0, y1): (f32, f32), (z0, z1): (f32, f32), up: bool, c: [u8; 3]) {
        let [w, h, d] = self.s;
        let points = self.leaving(if up { Exit::Up } else { Exit::Down });
        crate::catalog::block_with_entries(&mut self.m, w, (y0 * h, y1 * h), (z0 * d, z1 * d), &points, 6.0, c);
    }

    /// A box from `lo` to `hi`.
    pub(super) fn b(&mut self, lo: [f32; 3], hi: [f32; 3], c: [u8; 3]) {
        self.m.add_painted_box(self.at(lo), self.at(hi), c);
        if hi[2] >= 0.85 && lo[2] > 0.3 {
            let [w, h, _] = self.s;
            self.prims.push(Prim2d::Rect { min: [lo[0] * w, (1.0 - hi[1]) * h], max: [hi[0] * w, (1.0 - lo[1]) * h], fill: false });
        }
    }

    /// A cylinder along z (out of the front) at `x`, `y` from `z0` to `z1`, its radius a
    /// fraction of the smaller of width and height.
    fn knob(&mut self, x: f32, y: f32, z0: f32, z1: f32, r: f32, c: [u8; 3]) {
        let rr = r * self.s[0].min(self.s[1]);
        self.m.add_cylinder(self.at([x, y, z0]), 2, rr, (z1 - z0) * self.s[2], ROUND, c);
        if z1 >= 0.97 {
            let [w, h, _] = self.s;
            self.prims.push(Prim2d::Circle { c: [x * w, (1.0 - y) * h], r: rr, fill: false });
        }
    }

    /// The stepped profile of a DIN rail device: base, shoulders with the terminals
    /// and the front in the middle (`front` up to z 0.985, details reach z 1).
    fn din(&mut self, c: [u8; 3]) {
        let [w, h, d] = self.s;
        crate::catalog::base_with_entries(&mut self.m, [w, h, 0.6 * d], &self.points, c);
        self.b([0.0, 0.14, 0.6], [1.0, 0.86, 0.8], c);
        self.b([0.02, 0.27, 0.8], [0.98, 0.73, 0.985], c);
        // Screw terminals on the steps above and below the shoulders, one per
        // connection point of the row.
        for (exit, y) in [(Exit::Up, 0.93), (Exit::Down, 0.07)] {
            let xs = self.points_to(exit);
            let r = (0.3 / xs.len().max(1) as f32).min(0.12);
            for (x, _) in xs {
                self.b([x - r, y - 0.035, 0.6], [x + r, y + 0.035, 0.62], DARK);
            }
        }
    }
}

/// What a device of the bundled catalogs is modelled as.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Brand {
    MultiPlus,
    InverterRs,
    VictronBox,
    Cerbo,
    Shunt,
    Lynx,
    Pylontech,
    HagerNh,
    HagerStrip,
    HagerMeter,
    HagerDin { rcd: bool, contactor: bool },
    Dehn(usize),
    BeckhoffIo,
    BeckhoffCoupler,
    PhoenixPsu,
    SiemensMps,
    SiemensContactor,
}

pub(super) fn brand_of(p: &Part) -> Option<Brand> {
    let sub = p.subgroup.as_str();
    Some(match p.manufacturer.as_str() {
        "Victron Energy" => match sub {
            "Inverter / charger" => Brand::MultiPlus,
            "Inverter" => Brand::InverterRs,
            "System monitoring" => Brand::Cerbo,
            // The BMV in the front panel is no shunt but a display.
            "Battery monitor" if p.mounting == "Front panel" => return None,
            "Battery monitor" => Brand::Shunt,
            "Fuse" => Brand::Lynx,
            _ => Brand::VictronBox,
        },
        "Pylontech" if sub == "Battery" => Brand::Pylontech,
        "Hager" => match sub {
            // Strips for 185 mm busbar systems and the narrow NH000 disconnector (its
            // poles one above the other): tall and narrow.
            "Fuse switch disconnector" if p.height > 3.0 * p.width => Brand::HagerStrip,
            "Fuse switch disconnector" => Brand::HagerNh,
            "Energy meter" => Brand::HagerMeter,
            "Residual current device" => Brand::HagerDin { rcd: true, contactor: false },
            "Power contactor" => Brand::HagerDin { rcd: false, contactor: true },
            _ => Brand::HagerDin { rcd: false, contactor: false },
        },
        "DEHN" if sub == "Surge protection" => Brand::Dehn(p.functions.iter().filter(|f| f.kind == crate::FnKind::SurgeProtection).count().max(1)),
        // Power feed terminals (EL9410) are listed as bus couplers but look like terminals.
        "Beckhoff" if sub == "Bus coupler" && p.width > 20.0 => Brand::BeckhoffCoupler,
        "Beckhoff" if sub == "Bus coupler" => Brand::BeckhoffIo,
        "Beckhoff" if sub.ends_with("module") => Brand::BeckhoffIo,
        "Phoenix Contact" if sub == "Power supply" => Brand::PhoenixPsu,
        "Siemens" if sub == "Motor protection switch" => Brand::SiemensMps,
        "Siemens" if sub == "Power contactor" => Brand::SiemensContactor,
        _ => return None,
    })
}

/// Draws the model of `brand` into `f`.
pub(super) fn draw(f: &mut Frac, brand: Brand) {
    match brand {
        Brand::MultiPlus => {
            // Blue housing on black side rails, black corner caps over the top, black
            // connection compartment at the bottom, orange band, LED panel.
            f.b([0.0, 0.0, 0.0], [1.0, 0.08, 1.0], BLACK);
            f.b([0.025, 0.08, 0.0], [0.975, 0.96, 0.9], VICTRON);
            f.b([0.0, 0.08, 0.0], [0.025, 0.96, 0.6], BLACK);
            f.b([0.975, 0.08, 0.0], [1.0, 0.96, 0.6], BLACK);
            f.b([0.03, 0.1, 0.9], [0.97, 0.95, 0.96], VICTRON_FRONT);
            for x in [0.06, 0.8] {
                f.b([x, 0.92, 0.0], [x + 0.14, 1.0, 1.0], BLACK);
            }
            f.b([0.03, 0.52, 0.96], [0.97, 0.555, 0.975], ORANGE);
            f.b([0.07, 0.85, 0.96], [0.32, 0.875, 0.975], WHITE);
            f.b([0.08, 0.32, 0.96], [0.32, 0.45, 0.98], DISPLAY);
            for (k, c) in [LED, LED, AMBER, [220, 60, 50]].into_iter().enumerate() {
                let y = 0.34 + k as f32 * 0.027;
                f.b([0.11, y, 0.98], [0.13, y + 0.015, 1.0], c);
            }
            f.b([0.55, 0.36, 0.96], [0.84, 0.48, 0.97], [205, 215, 230]);
        }
        Brand::InverterRs => {
            // Square blue housing, black corner caps and bottom, display at the top,
            // orange band under it.
            f.b([0.0, 0.0, 0.0], [1.0, 0.1, 1.0], BLACK);
            f.b([0.01, 0.1, 0.0], [0.99, 0.88, 0.9], VICTRON);
            f.b([0.1, 0.88, 0.0], [0.9, 0.96, 0.9], VICTRON);
            f.b([0.03, 0.12, 0.9], [0.97, 0.87, 0.96], VICTRON_FRONT);
            f.b([0.1, 0.87, 0.9], [0.9, 0.95, 0.96], VICTRON_FRONT);
            for x in [0.0, 0.9] {
                f.b([x, 0.88, 0.0], [x + 0.1, 1.0, 1.0], BLACK);
            }
            f.b([0.38, 0.77, 0.96], [0.62, 0.9, 0.98], BLACK);
            f.b([0.41, 0.79, 0.98], [0.59, 0.88, 1.0], SCREEN);
            f.b([0.03, 0.66, 0.96], [0.97, 0.69, 0.975], ORANGE);
            f.b([0.06, 0.9, 0.96], [0.26, 0.925, 0.975], WHITE);
            f.b([0.06, 0.52, 0.96], [0.42, 0.62, 0.97], [205, 215, 230]);
        }
        Brand::VictronBox => {
            // Blue housing with the black terminal cover at the bottom.
            f.b([0.0, 0.3, 0.0], [1.0, 1.0, 0.92], VICTRON);
            f.b([0.0, 0.0, 0.0], [1.0, 0.3, 1.0], BLACK);
            f.b([0.04, 0.32, 0.92], [0.96, 0.96, 0.97], VICTRON_FRONT);
            f.b([0.08, 0.85, 0.97], [0.4, 0.88, 0.985], WHITE);
            f.b([0.08, 0.5, 0.97], [0.5, 0.6, 0.98], [205, 215, 230]);
        }
        Brand::Cerbo => {
            // Blue box between black ends, the row of ports along the top, LEDs.
            f.b([0.06, 0.0, 0.0], [0.94, 1.0, 0.85], VICTRON);
            for x in [0.0, 0.94] {
                f.b([x, 0.0, 0.0], [x + 0.06, 1.0, 1.0], BLACK);
            }
            for k in 0..7 {
                let x = 0.1 + k as f32 * 0.115;
                f.b([x, 0.7, 0.85], [x + 0.09, 0.95, 1.0], if k < 2 { WHITE } else { BLACK });
            }
            f.b([0.1, 0.12, 0.85], [0.4, 0.2, 0.9], WHITE);
            for k in 0..3 {
                let x = 0.6 + k as f32 * 0.08;
                f.b([x, 0.3, 0.85], [x + 0.04, 0.38, 0.9], LED);
            }
        }
        Brand::Shunt => {
            // The shunt on its black base between its two bolts, the blue housing over
            // it.
            f.b([0.0, 0.0, 0.0], [1.0, 1.0, 0.3], [56, 58, 62]);
            f.b([0.22, 0.05, 0.3], [0.78, 0.95, 0.97], VICTRON);
            f.b([0.3, 0.3, 0.97], [0.7, 0.4, 1.0], WHITE);
            for x in [0.1, 0.9] {
                f.knob(x, 0.5, 0.3, 0.75, 0.22, BRASS);
                f.m.add_frustum(f.at([x, 0.5, 0.75]), 2, 0.3 * f.s[0].min(f.s[1]), 0.3 * f.s[0].min(f.s[1]), 0.15 * f.s[2], 6, STEEL);
            }
        }
        Brand::Lynx => {
            // Blue base with copper bus bar ends, four fuses under the cover, LEDs.
            f.b([0.0, 0.3, 0.1], [1.0, 0.7, 0.4], [200, 120, 64]);
            f.b([0.05, 0.0, 0.0], [0.95, 1.0, 0.5], VICTRON);
            f.b([0.07, 0.08, 0.5], [0.93, 0.92, 0.9], [60, 110, 170]);
            for k in 0..4 {
                let x = 0.12 + k as f32 * 0.2;
                f.b([x, 0.25, 0.9], [x + 0.16, 0.75, 0.97], [210, 40, 40]);
                f.b([x + 0.05, 0.8, 0.9], [x + 0.11, 0.86, 1.0], LED);
            }
        }
        Brand::Pylontech => {
            // 19-inch module: black front with the bow handles, power switch, the ports
            // CAN, RS485, Link 0 and Link 1, LEDs and the battery terminals (two -, two
            // +); the catalog's connection points are on these ports and terminals.
            f.b([0.03, 0.0, 0.0], [0.97, 1.0, 0.92], BLACK);
            f.b([0.0, 0.0, 0.92], [1.0, 1.0, 0.95], [34, 36, 40]);
            for x in [0.035, 0.945] {
                f.b([x, 0.12, 0.97], [x + 0.02, 0.88, 1.0], SILVER);
                for y in [0.12, 0.83] {
                    f.b([x, y, 0.95], [x + 0.02, y + 0.05, 0.97], SILVER);
                }
            }
            f.b([0.09, 0.7, 0.95], [0.115, 0.8, 0.97], RED);
            for k in 0..4 {
                let x = 0.13 + k as f32 * 0.036;
                f.b([x, 0.68, 0.95], [x + 0.028, 0.8, 0.97], WHITE);
            }
            f.knob(0.08, 0.4, 0.95, 0.97, 0.06, SILVER);
            f.b([0.12, 0.35, 0.95], [0.15, 0.47, 0.975], DARK);
            for k in 0..6 {
                let x = 0.2 + k as f32 * 0.013;
                f.b([x, 0.41, 0.95], [x + 0.007, 0.44, 0.96], LED);
            }
            for (x, c) in [(0.48, BLACK), (0.56, BLACK), (0.7, ORANGE), (0.78, ORANGE)] {
                f.b([x, 0.28, 0.95], [x + 0.06, 0.5, 0.99], c);
                f.knob(x + 0.03, 0.39, 0.99, 1.0, 0.04, [70, 70, 70]);
            }
            f.b([0.5, 0.72, 0.95], [0.6, 0.78, 0.955], WHITE);
            f.b([0.65, 0.72, 0.95], [0.82, 0.78, 0.955], WHITE);
        }
        Brand::HagerNh => {
            // Light grey: cover with its grip and three windows on the fuse links,
            // terminal covers top and bottom (the clamps entered from their end faces,
            // below the screws), blue label.
            let base = [200, 202, 204];
            f.b([0.0, 0.0, 0.0], [1.0, 1.0, 0.45], base);
            f.with_entries((0.86, 1.0), (0.45, 0.7), true, LIGHT);
            f.with_entries((0.0, 0.14), (0.45, 0.7), false, LIGHT);
            f.b([0.03, 0.14, 0.45], [0.97, 0.86, 0.88], [220, 222, 224]);
            f.b([0.05, 0.78, 0.88], [0.95, 0.88, 1.0], [208, 210, 212]);
            f.b([0.72, 0.72, 0.88], [0.92, 0.76, 0.9], HAGER_BLUE);
            for k in 0..3 {
                let x = (k as f32 + 0.5) / 3.0;
                f.b([x - 0.12, 0.26, 0.88], [x + 0.12, 0.7, 0.9], [38, 44, 54]);
                f.b([x - 0.05, 0.34, 0.9], [x + 0.05, 0.62, 0.91], [214, 210, 196]);
                for y in [0.07, 0.93] {
                    f.knob(x, y, 0.7, 0.72, 0.05, DARK);
                }
            }
        }
        Brand::HagerStrip => {
            // Grey strip on the bars: the three fuse carriers one above the other in the
            // black handle frame, the connection compartment at the bottom (white cover,
            // blue label, the screws of the clamps of the outgoing cables entered from
            // below), a cap on top.
            let base = [200, 202, 204];
            f.b([0.0, 0.12, 0.0], [1.0, 0.97, 0.55], base);
            f.with_entries((0.0, 0.12), (0.0, 0.8), false, LIGHT);
            f.with_entries((0.97, 1.0), (0.0, 0.8), true, base);
            f.b([0.15, 0.03, 0.8], [0.85, 0.05, 0.81], HAGER_BLUE);
            for (x, _) in f.points_to(Exit::Down) {
                f.knob(x, 0.08, 0.8, 0.82, 0.1, DARK);
            }
            f.b([0.4, 0.13, 0.55], [0.6, 0.96, 0.99], BLACK);
            for k in 0..3 {
                let y = 0.14 + k as f32 * 0.275;
                f.b([0.06, y, 0.55], [0.94, y + 0.25, 0.9], [220, 222, 224]);
                f.b([0.12, y + 0.02, 0.9], [0.88, y + 0.23, 0.97], BLACK);
                f.b([0.25, y + 0.07, 0.97], [0.75, y + 0.18, 0.98], [214, 210, 196]);
            }
        }
        Brand::HagerMeter => {
            // Light grey DIN module: display, buttons, QR code, blue label.
            f.din(LIGHT);
            f.b([0.08, 0.5, 0.985], [0.55, 0.68, 1.0], LCD);
            for k in 0..3 {
                let x = 0.1 + k as f32 * 0.13;
                f.b([x, 0.33, 0.985], [x + 0.09, 0.39, 1.0], DARK);
            }
            f.b([0.68, 0.52, 0.985], [0.86, 0.67, 0.995], BLACK);
            f.b([0.74, 0.69, 0.985], [0.96, 0.72, 0.995], HAGER_BLUE);
        }
        Brand::HagerDin { rcd, contactor } => {
            // White DIN module: toggle (or the front cap of a contactor), label window,
            // test button of a residual current device.
            f.din([238, 238, 234]);
            if contactor {
                f.b([0.1, 0.36, 0.985], [0.9, 0.64, 0.995], [214, 214, 210]);
                f.b([0.4, 0.44, 0.995], [0.6, 0.56, 1.0], DARK);
            } else {
                f.b([0.3, 0.42, 0.985], [0.7, 0.56, 1.0], DARK);
                f.b([0.15, 0.6, 0.985], [0.85, 0.66, 0.995], HAGER_BLUE);
            }
            if rcd {
                f.b([0.72, 0.3, 0.985], [0.9, 0.37, 1.0], [96, 124, 170]);
            }
        }
        Brand::Dehn(n) => {
            // Red base with the plug-in protection modules: white label, green status
            // window each.
            f.din(DEHN_RED);
            for k in 0..n {
                let (x0, x1) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
                f.b([x0 + 0.01, 0.18, 0.6], [x1 - 0.01, 0.82, 0.985], [214, 42, 36]);
                f.b([x0 + 0.04, 0.34, 0.985], [x1 - 0.04, 0.64, 1.0], WHITE);
                f.b([x0 + 0.06, 0.24, 0.985], [x1 - 0.06, 0.3, 1.0], [80, 184, 92]);
            }
        }
        Brand::BeckhoffIo => {
            // Slim light grey terminal: LEDs at the top, the spring contacts with their
            // release tabs, the orange release tab at the bottom.
            f.b([0.0, 0.0, 0.0], [1.0, 1.0, 0.95], LIGHT);
            f.b([0.1, 0.82, 0.95], [0.9, 0.95, 0.98], [206, 206, 200]);
            for y in [0.86, 0.9] {
                f.b([0.3, y, 0.98], [0.7, y + 0.025, 1.0], LED);
            }
            if f.points_to(Exit::Front).is_empty() {
                for k in 0..4 {
                    let y = 0.22 + k as f32 * 0.14;
                    f.b([0.2, y, 0.95], [0.8, y + 0.08, 1.0], if k % 2 == 0 { ORANGE } else { [150, 152, 150] });
                }
            }
            f.springs(0.95);
            f.b([0.2, 0.02, 0.95], [0.8, 0.12, 1.0], ORANGE);
        }
        Brand::BeckhoffCoupler => {
            // Grey coupler: two network ports, LEDs, the power contacts on the right.
            f.b([0.0, 0.0, 0.0], [1.0, 1.0, 0.92], LIGHT);
            f.b([0.05, 0.2, 0.92], [0.13, 0.55, 0.96], [200, 40, 40]);
            for y in [0.58, 0.72] {
                f.b([0.22, y, 0.92], [0.55, y + 0.11, 1.0], BLACK);
            }
            f.b([0.22, 0.88, 0.92], [0.55, 0.95, 0.98], LED);
            if f.points_to(Exit::Front).is_empty() {
                for (k, c) in [ORANGE, [60, 110, 200], [200, 50, 50], [150, 152, 150]].into_iter().enumerate() {
                    let y = 0.2 + k as f32 * 0.14;
                    f.b([0.65, y, 0.92], [0.9, y + 0.08, 1.0], c);
                }
            }
            f.springs(0.92);
        }
        Brand::PhoenixPsu => {
            // Silver ribbed housing, light grey front with the green pluggable terminal
            // blocks, status bar with LEDs, adjusting potentiometer.
            f.b([0.0, 0.0, 0.0], [1.0, 0.95, 0.9], SILVER);
            for k in 0..6 {
                let z = 0.08 + k as f32 * 0.13;
                f.b([0.0, 0.95, z], [1.0, 1.0, z + 0.06], [172, 176, 182]);
            }
            f.b([0.04, 0.06, 0.9], [0.96, 0.94, 0.96], [214, 216, 218]);
            for (y0, y1) in [(0.76, 0.9), (0.1, 0.24)] {
                f.b([0.1, y0, 0.96], [0.9, y1, 1.0], GREEN);
            }
            f.b([0.15, 0.42, 0.96], [0.45, 0.66, 0.98], DARK);
            for k in 0..3 {
                let y = 0.46 + k as f32 * 0.06;
                f.b([0.2, y, 0.98], [0.26, y + 0.03, 1.0], LED);
            }
            f.knob(0.7, 0.54, 0.96, 1.0, 0.07, [60, 64, 70]);
        }
        Brand::SiemensMps => {
            // Grey body with the rotary handle in its round recess and the current
            // setting dial above it.
            f.din([200, 202, 204]);
            f.knob(0.5, 0.48, 0.985, 0.99, 0.34, [150, 152, 156]);
            f.knob(0.5, 0.48, 0.99, 1.0, 0.24, [60, 62, 66]);
            f.b([0.44, 0.38, 0.99], [0.56, 0.6, 1.0], [40, 42, 46]);
            f.knob(0.5, 0.78, 0.8, 0.82, 0.1, [70, 120, 190]);
        }
        Brand::SiemensContactor => {
            // Grey body, dark front cap with the switch position indicator.
            f.din([200, 202, 204]);
            f.b([0.08, 0.3, 0.985], [0.92, 0.7, 0.995], [110, 114, 120]);
            f.b([0.4, 0.46, 0.995], [0.6, 0.56, 1.0], [230, 230, 226]);
        }
    }
}
