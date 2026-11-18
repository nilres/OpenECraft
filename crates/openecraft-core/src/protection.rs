//! Protective devices: characteristic, rated current, breaking capacity and the
//! time-current curves (IEC 60898-1 circuit breakers, IEC 60947-4-1 motor protection,
//! IEC 60269 fuses) behind the disconnection times, short-circuit withstand and
//! selectivity. The curves are bands (shortest and longest tripping time) fitted to
//! the conventional currents and gates of the standards; device data from the parts
//! (`Characteristic: C`, `Breaking capacity: 10 kA`, `Instantaneous trip: 13 x In`,
//! `Let-through energy: 25000 A²s`) replaces the defaults. Curves of the manufacturer
//! replace the bands: `Trip curve min` / `Trip curve max` as multiples of the rated
//! current with times (`1.5: 600; 3: 20; 10: 0.01`), `Let-through energy` as currents
//! with energies (`1 kA: 8000; 6 kA: 25000`). Electronic trip units (IEC 60947-2,
//! `Characteristic: LSI`) are set with `Long-time delay` (s at 6 × Ir),
//! `Short-time pickup` (× Ir), `Short-time delay` (s) and `Instantaneous trip`.

use openecraft_i18n::tr;
use crate::electrical::value_with_unit;
use crate::model::Project;
use crate::parts::{FnKind, Part};
use serde::{Deserialize, Serialize};

/// The device does not trip.
pub const NO_TRIP: f32 = f32::INFINITY;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Characteristic {
    B,
    C,
    D,
    K,
    Z,
    /// Full-range fuse.
    Gg,
    /// Partial-range (motor) fuse.
    Am,
    /// Motor protection switch, trip class 10.
    Motor,
    /// Electronic trip unit with long-time, short-time and instantaneous protection.
    Electronic,
}

impl Characteristic {
    pub fn title(self) -> &'static str {
        match self {
            Characteristic::B => "B",
            Characteristic::C => "C",
            Characteristic::D => "D",
            Characteristic::K => "K",
            Characteristic::Z => "Z",
            Characteristic::Gg => "gG",
            Characteristic::Am => "aM",
            Characteristic::Motor => tr("Motor protection (class 10)"),
            Characteristic::Electronic => tr("Electronic (LSI)"),
        }
    }

    pub fn is_fuse(self) -> bool {
        matches!(self, Characteristic::Gg | Characteristic::Am)
    }

    fn parse(text: &str) -> Option<Characteristic> {
        Some(match text.trim() {
            "B" => Characteristic::B,
            "C" => Characteristic::C,
            "D" => Characteristic::D,
            "K" => Characteristic::K,
            "Z" => Characteristic::Z,
            t if t.eq_ignore_ascii_case("gG") || t.eq_ignore_ascii_case("gL") => Characteristic::Gg,
            t if t.eq_ignore_ascii_case("aM") => Characteristic::Am,
            t if ["LI", "LSI", "LSIG", "electronic"].iter().any(|x| t.eq_ignore_ascii_case(x)) => Characteristic::Electronic,
            _ => return None,
        })
    }

    /// Range of the instantaneous trip in multiples of the rated current.
    fn instantaneous(self) -> (f32, f32) {
        match self {
            Characteristic::B => (3.0, 5.0),
            Characteristic::C => (5.0, 10.0),
            Characteristic::D => (10.0, 20.0),
            Characteristic::K => (8.0, 14.0),
            Characteristic::Z => (2.0, 3.0),
            Characteristic::Motor => (11.0, 15.0),
            Characteristic::Electronic => (10.8, 13.2),
            Characteristic::Gg | Characteristic::Am => (f32::INFINITY, f32::INFINITY),
        }
    }
}

/// Settings of an electronic trip unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TripUnit {
    /// Long-time delay: tripping time at 6 × Ir, s.
    pub long_delay: f32,
    /// Short-time pickup in multiples of Ir (`None`: no short-time protection).
    pub short_pickup: Option<f32>,
    /// Short-time delay, s.
    pub short_delay: f32,
}

/// Tripping times at multiples of the rated current, from the manufacturer.
pub type Curve = Vec<(f32, f32)>;

#[derive(Clone, Debug, PartialEq)]
pub struct Protection {
    pub characteristic: Characteristic,
    /// Part number of the device.
    pub part: String,
    /// Rated current (setting) in A.
    pub rated: f32,
    /// Rated (ultimate) short-circuit breaking capacity in kA.
    pub breaking: Option<f32>,
    /// Rated short-circuit making capacity (peak) in kA.
    pub making: Option<f32>,
    /// Instantaneous trip range in multiples of the rated current.
    pub instantaneous: (f32, f32),
    /// Let-through energy at the breaking capacity in A²s.
    pub let_through: Option<f32>,
    /// Let-through energy at prospective currents (A, A²s), from the manufacturer.
    pub energy_curve: Vec<(f32, f32)>,
    /// Pre-arcing energy of a fuse in A²s.
    pub pre_arcing: Option<f32>,
    /// Shortest and longest tripping times of the manufacturer (empty: the band of
    /// the standard).
    pub curve_min: Curve,
    pub curve_max: Curve,
    /// Settings of an electronic trip unit.
    pub trip_unit: Option<TripUnit>,
}

/// First number of a text ("25000 A²s" → 25000, "2.5e4" → 25000).
fn first_number(text: &str) -> Option<f32> {
    let start = text.find(|c: char| c.is_ascii_digit())?;
    let num: String = text[start..].chars().take_while(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | 'e' | 'E' | '-' | '+')).collect();
    num.replace(',', ".").trim_end_matches(['e', 'E', '-', '+']).parse().ok()
}

/// Pairs of numbers ("1.5: 600; 3: 20"; the first of a pair in A or kA when
/// `amps`), sorted by the first.
fn pairs(text: &str, amps: bool) -> Vec<(f32, f32)> {
    let mut out: Vec<(f32, f32)> = text
        .split(';')
        .filter_map(|p| {
            let (a, b) = p.split_once([':', '='])?;
            let a = if amps { value_with_unit(a, "kA").map(|v| v * 1000.0).or_else(|| first_number(a)) } else { first_number(a) }?;
            Some((a, first_number(b)?)).filter(|(a, b)| *a > 0.0 && *b > 0.0)
        })
        .collect();
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out
}

/// Value of a curve at `x`, interpolated log-log; `None` below the first point, the
/// last value beyond the last.
fn interpolate(curve: &[(f32, f32)], x: f32) -> Option<f32> {
    let first = curve.first()?;
    if x < first.0 {
        return None;
    }
    for w in curve.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if x <= x1 {
            let f = if x1 > x0 { (x / x0).ln() / (x1 / x0).ln() } else { 1.0 };
            return Some((y0.ln() + f * (y1 / y0).ln()).exp());
        }
    }
    curve.last().map(|p| p.1)
}

/// Characteristic letter in a type designation: `C16`, `B10`, `MCB-1P-C6`.
fn letter_in(text: &str) -> Option<Characteristic> {
    text.split(|c: char| !c.is_alphanumeric()).find_map(|tok| {
        let mut chars = tok.chars();
        let first = chars.next()?;
        let rest = chars.as_str();
        (!rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())).then(|| Characteristic::parse(&first.to_string())).flatten()
    })
}

/// The characteristic a part states (tech data, type of device, letter in its type
/// number), `None` where only the default C remains.
fn stated(part: &Part, kind: FnKind, electronic: bool) -> Option<Characteristic> {
    part.tech_value("Characteristic").or_else(|| part.tech_value("Utilisation category")).and_then(Characteristic::parse).or_else(|| match kind {
        FnKind::MotorProtection => Some(Characteristic::Motor),
        FnKind::Fuse | FnKind::Fuse3 if part.description.contains(" aM") => Some(Characteristic::Am),
        FnKind::Fuse | FnKind::Fuse3 => Some(Characteristic::Gg),
        _ if electronic => Some(Characteristic::Electronic),
        _ => letter_in(&part.type_number).or_else(|| letter_in(&part.number)).or_else(|| letter_in(&part.description)),
    })
}

/// The tripping characteristic of the device `tag` as its parts state it; `None` where
/// the calculations assume C ([`of_device`]).
pub fn stated_characteristic(prj: &Project, tag: &str) -> Option<Characteristic> {
    crate::sizing::device_parts(prj, tag).into_iter().find_map(|p| {
        let f = p.functions.iter().find(|f| crate::sizing::PROTECTIVE.contains(&f.kind))?;
        let part = crate::sizing::fuse_link(prj, tag).unwrap_or(p);
        crate::sizing::parse_current(&part.current)?;
        stated(part, f.kind, part.tech_value("Short-time pickup").and_then(first_number).is_some())
    })
}

impl Protection {
    /// Protection data of a part with a protective function of this kind.
    pub fn of_part(part: &Part, kind: FnKind) -> Option<Protection> {
        let rated = crate::sizing::parse_current(&part.current)?;
        let short_pickup = part.tech_value("Short-time pickup").and_then(first_number);
        let characteristic = stated(part, kind, short_pickup.is_some()).unwrap_or(Characteristic::C);
        let ka = |keys: &[&str]| keys.iter().find_map(|k| part.tech_value(k)).and_then(|v| value_with_unit(v, "kA").or_else(|| first_number(v)));
        let breaking = ka(&["Breaking capacity", "Icu", "Icn"]);
        let making = ka(&["Making capacity", "Icm"]);
        let instantaneous = match part.tech_value("Instantaneous trip").and_then(first_number) {
            Some(m) if characteristic == Characteristic::Motor => (m * 0.8, m * 1.2),
            Some(m) if characteristic == Characteristic::Electronic => (m * 0.9, m * 1.1),
            Some(m) => (m, m),
            None => characteristic.instantaneous(),
        };
        let energy_text = part.tech_value("Let-through energy");
        let energy_curve = energy_text.map(|v| pairs(v, true)).unwrap_or_default();
        let let_through = energy_text.filter(|_| energy_curve.is_empty()).and_then(first_number);
        let pre_arcing = part.tech_value("Pre-arcing energy").and_then(first_number);
        let curve = |key: &str| part.tech_value(key).map(|v| pairs(v, false)).unwrap_or_default();
        let trip_unit = (characteristic == Characteristic::Electronic).then(|| TripUnit {
            long_delay: part.tech_value("Long-time delay").and_then(first_number).unwrap_or(10.0),
            short_pickup,
            short_delay: part.tech_value("Short-time delay").and_then(first_number).unwrap_or(0.1),
        });
        Some(Protection {
            characteristic,
            part: part.number.clone(),
            rated,
            breaking,
            making,
            instantaneous,
            let_through,
            energy_curve,
            pre_arcing,
            curve_min: curve("Trip curve min"),
            curve_max: curve("Trip curve max"),
            trip_unit,
        })
    }

    /// Shortest and longest tripping time at the current `i` in s ([`NO_TRIP`]: the
    /// device does not trip); the manufacturer's curves where given.
    pub fn trip_time(&self, i: f32) -> (f32, f32) {
        let m = i / self.rated.max(0.01);
        let (lo, hi) = self.band(m);
        let vendor = |c: &Curve, standard: f32| if c.is_empty() { standard } else { interpolate(c, m).unwrap_or(NO_TRIP) };
        (vendor(&self.curve_min, lo), vendor(&self.curve_max, hi))
    }

    /// Band of the standard at `m` times the rated current.
    fn band(&self, m: f32) -> (f32, f32) {
        let curve = |t: f32, m0: f32, m: f32, cap: f32| if m > m0 { (t * ((2.55 - m0) / (m - m0)).powi(2)).min(cap) } else { NO_TRIP };
        let (lo, hi) = self.instantaneous;
        match self.characteristic {
            Characteristic::Gg => (
                if m > 1.25 { (10.0 * (2.06 / m).powf(4.87)).min(3600.0) } else { NO_TRIP },
                if m > 1.6 { (5.0 * (4.06 / m).powf(4.66)).min(3600.0) } else { NO_TRIP },
            ),
            Characteristic::Am => (
                if m > 4.0 { (10.0 * (4.0 / m).powf(4.87)).min(3600.0) } else { NO_TRIP },
                if m > 6.3 { (5.0 * (8.0 / m).powf(4.66)).min(3600.0) } else { NO_TRIP },
            ),
            Characteristic::Motor => {
                let thermal = |t: f32, m0: f32| if m > m0 { (t * ((7.2 - m0) / (m - m0)).powi(2)).min(7200.0) } else { NO_TRIP };
                (if m >= lo { 0.0 } else { thermal(4.0, 1.05) }, if m >= hi { 0.1 } else { thermal(10.0, 1.2) })
            }
            // Long time I²t = constant (±20 %, pickup 1.05 … 1.2 Ir), short time at a
            // definite delay (pickup ±10 %, clearing within 0.1 s more), instantaneous.
            Characteristic::Electronic => {
                let u = self.trip_unit.unwrap_or(TripUnit { long_delay: 10.0, short_pickup: None, short_delay: 0.1 });
                let long = |f: f32, pickup: f32| if m > pickup { (f * u.long_delay * 36.0 / (m * m)).min(7200.0) } else { NO_TRIP };
                let short = |f: f32, t: f32| u.short_pickup.filter(|p| m >= p * f).map_or(NO_TRIP, |_| t);
                let min = long(0.8, 1.05).min(short(0.9, u.short_delay)).min(if m >= lo { 0.0 } else { NO_TRIP });
                let max = long(1.2, 1.2).min(short(1.1, u.short_delay + 0.1)).min(if m >= hi { 0.05 } else { NO_TRIP });
                (min, max)
            }
            // 2.55 In: 1 … 60 s by IEC 60898-1; the lower band of real devices is about 5 s.
            _ => (if m >= lo { 0.0 } else { curve(5.0, 1.13, m, 3600.0) }, if m >= hi { 0.1 } else { curve(60.0, 1.45, m, 3600.0) }),
        }
    }

    /// Smallest current that trips the device within `t` seconds at the latest.
    pub fn current_for(&self, t: f32) -> f32 {
        let (mut a, mut b) = (self.rated, self.rated * 1000.0);
        if self.trip_time(b).1 > t {
            return f32::INFINITY;
        }
        for _ in 0..60 {
            let m = (a * b).sqrt();
            if self.trip_time(m).1 <= t {
                b = m;
            } else {
                a = m;
            }
        }
        b
    }

    /// Energy let through at the current `i` in A²s; `None` in the instantaneous range
    /// of a circuit breaker without data from the manufacturer.
    pub fn energy(&self, i: f32) -> Option<f32> {
        // Within the currents of the manufacturer's curve.
        if let Some(e) = interpolate(&self.energy_curve, i).filter(|_| self.energy_curve.last().is_some_and(|l| i <= l.0)) {
            return Some(e);
        }
        let t = self.trip_time(i).1;
        if t == NO_TRIP {
            return None;
        }
        if self.characteristic.is_fuse() {
            return Some(i * i * t.max(0.004));
        }
        if t >= 0.1 && i < self.instantaneous.1 * self.rated {
            Some(i * i * t)
        } else {
            self.let_through
        }
    }
}

/// Protection data of a device: its part with a protective function; none for a fuse
/// switch disconnector fitted with solid links (an isolator).
pub fn of_device(prj: &Project, tag: &str) -> Option<Protection> {
    if crate::sizing::isolates_only(prj, tag) {
        return None;
    }
    crate::sizing::device_parts(prj, tag).into_iter().find_map(|p| {
        let f = p.functions.iter().find(|f| crate::sizing::PROTECTIVE.contains(&f.kind))?;
        // A fuse base or fuse switch disconnector protects with its fuse links.
        let link = crate::sizing::fuse_link(prj, tag);
        Protection::of_part(link.unwrap_or(p), f.kind)
    })
}

#[cfg(test)]
mod tests;
