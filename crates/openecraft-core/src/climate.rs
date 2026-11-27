//! Enclosure climate (IEC/TR 60890, DIN VDE 0660-507): power loss of the devices in
//! an enclosure, the effective surface by installation type, the temperature rise
//! and the fan, heat exchanger (sealed enclosures) or cooling unit needed to keep the
//! inside temperature, with the air flow corrected for the altitude; the heater keeps
//! the lowest inside temperature and the dew point of the ambient air. Power losses
//! come from the tech data of the parts (`Power loss: 7.5 W`). An open mounting
//! assembly has no walls and no inside air: its devices give off their heat to the
//! room air, so it needs neither cooling nor heating.

use openecraft_i18n::{tr, tr_fmt};
use crate::model::Project;
use crate::panel::{Enclosure, ItemKind};
pub use crate::model::climate::{Climate, Installation, Material};

impl Installation {
    pub const ALL: [Installation; 7] = [
        Installation::FreeStanding,
        Installation::WallMounted,
        Installation::EndFreeStanding,
        Installation::EndWallMounted,
        Installation::MiddleFreeStanding,
        Installation::MiddleWallMounted,
        Installation::MiddleWallMountedCovered,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Installation::FreeStanding => tr("Single, free-standing"),
            Installation::WallMounted => tr("Single, wall mounted"),
            Installation::EndFreeStanding => tr("End of a suite, free-standing"),
            Installation::EndWallMounted => tr("End of a suite, wall mounted"),
            Installation::MiddleFreeStanding => tr("Within a suite, free-standing"),
            Installation::MiddleWallMounted => tr("Within a suite, wall mounted"),
            Installation::MiddleWallMountedCovered => tr("Within a suite, wall mounted, roof covered"),
        }
    }

    /// Effective surface in m² (IEC/TR 60890) of an enclosure of width, height and
    /// depth in m.
    pub fn surface(self, w: f32, h: f32, d: f32) -> f32 {
        match self {
            Installation::FreeStanding => 1.8 * h * (w + d) + 1.4 * w * d,
            Installation::WallMounted => 1.4 * w * (h + d) + 1.8 * d * h,
            Installation::EndFreeStanding => 1.4 * d * (h + w) + 1.8 * w * h,
            Installation::EndWallMounted => 1.4 * h * (w + d) + 1.4 * w * d,
            Installation::MiddleFreeStanding => 1.8 * w * h + 1.4 * w * d + d * h,
            Installation::MiddleWallMounted => 1.4 * w * (h + d) + d * h,
            Installation::MiddleWallMountedCovered => 1.4 * w * h + 0.7 * w * d + d * h,
        }
    }
}

impl Material {
    pub const ALL: [Material; 4] = [Material::SheetSteel, Material::StainlessSteel, Material::Aluminium, Material::Plastic];

    pub fn title(self) -> &'static str {
        match self {
            Material::SheetSteel => tr("Sheet steel"),
            Material::StainlessSteel => tr("Stainless steel"),
            Material::Aluminium => tr("Aluminium"),
            Material::Plastic => tr("Plastic"),
        }
    }

    /// Heat transfer coefficient k in W/(m²·K).
    pub fn k(self) -> f32 {
        match self {
            Material::SheetSteel => 5.5,
            Material::StainlessSteel => 4.5,
            Material::Aluminium => 12.0,
            Material::Plastic => 3.5,
        }
    }
}

impl Default for Climate {
    fn default() -> Self {
        Climate {
            installation: Installation::default(),
            material: Material::default(),
            ambient_max: 25.0,
            ambient_min: 10.0,
            inside_max: 35.0,
            inside_min: 5.0,
            extra_loss: 0.0,
            altitude: 0.0,
            sealed: false,
            humidity: 0.0,
        }
    }
}

impl Climate {
    pub fn is_default(&self) -> bool {
        *self == Climate::default()
    }
}

/// Air flow factor of filter fans at sea level (m³·K/(W·h)).
const FAN_FACTOR: f32 = 3.1;
/// Fans and heat exchangers need at least this difference between inside and
/// ambient temperature (K).
const FAN_MIN_DIFFERENCE: f32 = 5.0;

/// Density of the air at an altitude relative to sea level (ISA troposphere).
pub fn air_density(altitude: f32) -> f32 {
    (1.0 - 2.25577e-5 * altitude.max(0.0)).max(0.1).powf(4.2559)
}

/// Dew point of air at a temperature (°C) and relative humidity (%), Magnus formula
/// (17.62, 243.12 °C).
pub fn dew_point(t: f32, humidity: f32) -> f32 {
    let (a, b) = (17.62, 243.12);
    let g = (humidity.clamp(1.0, 100.0) / 100.0).ln() + a * t / (b + t);
    b * g / (a - g)
}

/// What keeps the enclosure below its highest inside temperature.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cooling {
    /// Natural convection through the walls is enough.
    None,
    /// Filter fan with this air flow (m³/h).
    Fan(f32),
    /// Air/air heat exchanger with this specific output (W/K).
    HeatExchanger(f32),
    /// Cooling unit with this useful cooling output (W).
    CoolingUnit(f32),
}

/// Result of the climate calculation of an enclosure.
#[derive(Clone, Debug, PartialEq)]
pub struct ClimateResult {
    /// Effective surface (m²).
    pub surface: f32,
    /// Power loss of the devices plus the extra loss (W).
    pub loss: f32,
    /// Devices with parts but without power loss data.
    pub without_data: Vec<String>,
    /// Inside temperature without cooling at the highest ambient temperature (°C).
    pub inside: f32,
    pub cooling: Cooling,
    /// Heater output to keep the lowest inside temperature and the dew point, devices
    /// switched off (W).
    pub heating: f32,
    /// Dew point of the ambient air (°C), when the humidity is given.
    pub dew_point: Option<f32>,
    /// Open mounting assembly: no enclosure, the inside is the room air.
    pub open: bool,
}

/// Power loss of a part from its tech data (`Power loss: 7.5 W`).
pub fn power_loss(part: &crate::parts::Part) -> Option<f32> {
    let v = part.data_value("Power loss")?;
    let num: String = v.chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',').collect();
    num.replace(',', ".").parse().ok()
}

/// Power loss of the devices placed in an enclosure and the devices that have parts
/// without power loss data.
pub fn enclosure_loss(prj: &Project, e: &Enclosure) -> (f32, Vec<String>) {
    let mut loss = 0.0;
    let mut without = Vec::new();
    for it in e.items.iter().filter(|i| i.kind == ItemKind::Device) {
        match crate::panel::thermal::device_loss(prj, &it.tag) {
            Some(l) => loss += l,
            None if prj.devices.get(&it.tag).is_some_and(|d| !d.parts.is_empty()) && !without.contains(&it.tag) => without.push(it.tag.clone()),
            None => {}
        }
    }
    (loss, without)
}

/// Climate calculation of an enclosure.
pub fn calculate(prj: &Project, e: &Enclosure) -> ClimateResult {
    let c = &e.climate;
    let (devices, without_data) = enclosure_loss(prj, e);
    let loss = devices + c.extra_loss.max(0.0);
    let dew_point = (c.humidity > 0.0).then(|| dew_point(c.ambient_max, c.humidity));
    if e.is_assembly() {
        return ClimateResult { surface: 0.0, loss, without_data, inside: c.ambient_max, cooling: Cooling::None, heating: 0.0, dew_point, open: true };
    }
    let surface = c.installation.surface(e.width / 1000.0, e.height / 1000.0, e.depth / 1000.0).max(0.01);
    let ka = c.material.k() * surface;
    let inside = c.ambient_max + loss / ka;
    // Heat that the walls cannot give off at the highest inside temperature.
    let excess = loss - ka * (c.inside_max - c.ambient_max);
    let difference = c.inside_max - c.ambient_max;
    let cooling = if excess <= 0.0 {
        Cooling::None
    } else if difference >= FAN_MIN_DIFFERENCE && c.sealed {
        Cooling::HeatExchanger(excess / difference)
    } else if difference >= FAN_MIN_DIFFERENCE {
        Cooling::Fan(FAN_FACTOR / air_density(c.altitude) * excess / difference)
    } else {
        Cooling::CoolingUnit(excess)
    };
    // Moist air must not condense when the devices are off at the lowest ambient
    // temperature: the heater keeps the inside above its dew point.
    let lowest = dew_point.map_or(c.inside_min, |d| d.max(c.inside_min));
    let heating = (ka * (lowest - c.ambient_min)).max(0.0);
    ClimateResult { surface, loss, without_data, inside, cooling, heating, dew_point, open: false }
}

impl Cooling {
    /// "–", "filter fan 120 m³/h", "heat exchanger 25 W/K", "cooling unit 450 W".
    pub fn text(self) -> String {
        match self {
            Cooling::None => "–".into(),
            Cooling::Fan(v) => tr_fmt!("filter fan {} m³/h", format!("{v:.0}")),
            Cooling::HeatExchanger(q) => tr_fmt!("heat exchanger {} W/K", format!("{q:.1}")),
            Cooling::CoolingUnit(q) => tr_fmt!("cooling unit {} W", format!("{q:.0}")),
        }
    }
}

#[cfg(test)]
mod tests;
