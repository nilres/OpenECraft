//! Part filter of the part trees: free text plus manufacturer, supplier and product
//! group, and the criterion the tree is grouped by.

use crate::{natural_cmp, Part, TopGroup};

/// Criterion of the first tree level.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Grouping {
    /// Top group › product group › subgroup.
    #[default]
    ProductGroup,
    Manufacturer,
    Supplier,
}

impl Grouping {
    pub const ALL: [Grouping; 3] = [Grouping::ProductGroup, Grouping::Manufacturer, Grouping::Supplier];

    pub fn title(self) -> &'static str {
        match self {
            Grouping::ProductGroup => "Product group",
            Grouping::Manufacturer => "Manufacturer",
            Grouping::Supplier => "Supplier",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PartFilter {
    /// Searched in number, type, order number, manufacturer, description and groups.
    pub text: String,
    pub manufacturer: Option<String>,
    pub supplier: Option<String>,
    pub top_group: Option<TopGroup>,
    pub group: Option<String>,
    /// Only parts that can be mounted directly on a room wall ([`Part::wall_mountable`]).
    pub wall_mount: bool,
    /// Only parts that stand on the floor of a room ([`Part::floor_standing`]).
    pub floor_mount: bool,
    pub grouping: Grouping,
}

impl PartFilter {
    pub fn matches(&self, p: &Part) -> bool {
        let eq = |want: &Option<String>, have: &str| want.as_deref().is_none_or(|w| w == have);
        p.matches_text(&self.text)
            && eq(&self.manufacturer, &p.manufacturer)
            && eq(&self.supplier, &p.supplier)
            && self.top_group.is_none_or(|t| t == p.top_group)
            && eq(&self.group, &p.group)
            && (!self.wall_mount || p.wall_mountable())
            && (!self.floor_mount || p.floor_standing())
    }

    /// True if any criterion restricts the parts (the grouping does not).
    pub fn is_active(&self) -> bool {
        !self.text.is_empty() || self.manufacturer.is_some() || self.supplier.is_some() || self.top_group.is_some() || self.group.is_some() || self.wall_mount || self.floor_mount
    }

    /// Clears all criteria, keeps the grouping.
    pub fn reset(&mut self) {
        *self = PartFilter { grouping: self.grouping, ..Default::default() };
    }
}

/// Where an enclosure part is mounted (filter of the new enclosure dialog).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EnclosureMounting {
    #[default]
    Any,
    Wall,
    /// Floor-standing: a baying system, a rack or a part made for the floor.
    Floor,
}

/// A range of an outer dimension, mm; 0 leaves its end open.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SizeRange {
    pub min: f32,
    pub max: f32,
}

impl SizeRange {
    pub fn contains(self, v: f32) -> bool {
        (self.min <= 0.0 || v >= self.min) && (self.max <= 0.0 || v <= self.max)
    }

    pub fn is_open(self) -> bool {
        self.min <= 0.0 && self.max <= 0.0
    }
}

/// How the enclosures found are sorted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EnclosureSort {
    /// Manufacturer, then part number.
    #[default]
    Number,
    Width,
    Height,
    Depth,
    /// Front area, the smallest first.
    Size,
}

impl EnclosureSort {
    pub const ALL: [EnclosureSort; 5] = [EnclosureSort::Number, EnclosureSort::Width, EnclosureSort::Height, EnclosureSort::Depth, EnclosureSort::Size];
}

/// Search of the enclosure parts offered for a new enclosure: free text, enclosure
/// system, manufacturer, mounting, ranges of the outer dimensions, and the field
/// widths and rows of meter and distribution cabinets (part data `Fields: 3`,
/// `Rows: 7`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnclosureFilter {
    /// Searched in number, type, order number, manufacturer, description and groups.
    pub text: String,
    /// Id of the enclosure system (part data `Enclosure system`), empty for all.
    pub system: String,
    /// Empty for all.
    pub manufacturer: String,
    pub mounting: EnclosureMounting,
    pub width: SizeRange,
    pub height: SizeRange,
    pub depth: SizeRange,
    pub fields: Option<u32>,
    pub rows: Option<u32>,
    pub sort: EnclosureSort,
}

/// An enclosure body with outer dimensions (product subgroup `Enclosure`, or EPLAN's
/// product group `Enclosure` as the makers' EPLAN data has it).
pub fn is_enclosure(p: &Part) -> bool {
    is_enclosure_body(p) && p.width > 0.0 && p.height > 0.0
}

/// An enclosure body by its product group, with or without its dimensions.
pub fn is_enclosure_body(p: &Part) -> bool {
    p.subgroup == "Enclosure" || (p.group == "Enclosure" && matches!(p.subgroup.as_str(), "" | "General") && !p.accessory)
}

/// The outer size a description names with the order of its values (`WHD: 800x2000x600
/// mm`, `BHT`, `H x W x D`, …): width, height and depth in mm. Values without such a
/// label are left alone, makers write them in different orders.
pub fn size_from_text(text: &str) -> Option<(f32, f32, f32)> {
    // The label without spaces, dots and x, and where its values go (0 width, 1 height,
    // 2 depth): English, German, French, Spanish, Italian and Polish.
    const LABELS: [(&str, [usize; 3]); 10] = [
        ("WHD", [0, 1, 2]),
        ("BHT", [0, 1, 2]),
        ("LHP", [0, 1, 2]),
        ("ANALPR", [0, 1, 2]),
        ("LAP", [0, 1, 2]),
        ("SWG", [0, 1, 2]),
        ("HWD", [1, 0, 2]),
        ("HBT", [1, 0, 2]),
        ("WDH", [0, 2, 1]),
        ("BTH", [0, 2, 1]),
    ];
    let chars: Vec<char> = text.chars().collect();
    let digit = |i: usize| chars.get(i).is_some_and(char::is_ascii_digit);
    let number = |i: &mut usize| -> Option<f32> {
        let start = *i;
        while digit(*i) || (*i > start && matches!(chars.get(*i), Some('.' | ',')) && digit(*i + 1)) {
            *i += 1;
        }
        (*i > start).then(|| chars[start..*i].iter().collect::<String>().replace(',', ".").parse().ok()).flatten()
    };
    let spaces = |i: &mut usize| {
        while chars.get(*i) == Some(&' ') {
            *i += 1;
        }
    };
    let mut i = 0;
    while i < chars.len() {
        if !digit(i) || (i > 0 && (digit(i - 1) || matches!(chars[i - 1], '.' | ','))) {
            i += 1;
            continue;
        }
        let (start, mut j, mut values) = (i, i, Vec::new());
        while let Some(v) = number(&mut j) {
            values.push(v);
            if values.len() == 3 {
                break;
            }
            spaces(&mut j);
            if !matches!(chars.get(j), Some('x' | 'X' | '\u{d7}')) {
                break;
            }
            j += 1;
            spaces(&mut j);
        }
        if values.len() == 3 && values.iter().all(|v| *v > 0.0) {
            let before: Vec<char> = chars[..start].iter().rev().take_while(|c| c.is_alphabetic() || matches!(c, ' ' | '.' | ':' | '/' | '(' | ')' | '-' | '\u{d7}')).copied().collect();
            let label: String = before.iter().rev().filter(|c| c.is_alphabetic() && !matches!(c, 'x' | 'X')).collect::<String>().to_uppercase();
            if let Some((_, order)) = LABELS.iter().find(|(l, _)| label.ends_with(l)) {
                let mut size = [0.0; 3];
                for (v, &d) in values.iter().zip(order) {
                    size[d] = *v;
                }
                return Some((size[0], size[1], size[2]));
            }
        }
        i = j.max(i + 1);
    }
    None
}

/// What the panel layout needs of an enclosure body ([`is_enclosure_body`]) taken from
/// its maker's texts where its data lacks it (the EPLAN Data Portal names no
/// dimensions): the outer size, the enclosure system (`Enclosure system`) and that it has
/// no mounting plate (`Mounting plate: no`).
pub fn complete_enclosure(p: &mut Part, texts: &[&str]) {
    if !is_enclosure_body(p) {
        return;
    }
    if p.width <= 0.0 || p.height <= 0.0 {
        if let Some((w, h, d)) = texts.iter().find_map(|t| size_from_text(t)) {
            (p.width, p.height) = (w, h);
            if p.depth <= 0.0 {
                p.depth = d;
            }
        }
    }
    let system = if p.tech_value("Enclosure system").is_none() { texts.iter().find_map(|t| crate::mechanics::system_from_text(t)) } else { None };
    let no_plate = p.tech_value("Mounting plate").is_none() && texts.iter().any(|t| t.to_lowercase().contains("without mounting plate"));
    let lines = system.map(|s| format!("Enclosure system: {s}")).into_iter().chain(no_plate.then(|| "Mounting plate: no".to_string()));
    for line in lines {
        if !p.tech_data.is_empty() {
            p.tech_data.push('\n');
        }
        p.tech_data.push_str(&line);
    }
}

/// The mounting plate of an enclosure part from the size of a model of the plate (as
/// the `CAD_MPL_…` files of Rittal's CAD models): its width along x, its height the
/// larger of the other two extents (the plate standing or lying). Without a `Mounting
/// origin` the enclosure centres it. `None` for a model larger than the enclosure.
pub fn set_mounting_plate(p: &mut Part, model: [f32; 3]) -> Option<[f32; 2]> {
    let round = |v: f32| (v * 10.0).round() / 10.0;
    let size = [round(model[0]), round(model[1].max(model[2]))];
    let positive = size.iter().all(|v| v.is_finite() && *v > 0.0);
    if !positive || (p.width > 0.0 && size[0] > p.width) || (p.height > 0.0 && size[1] > p.height) {
        return None;
    }
    p.set_tech_value("Mounting size", Some(&format!("{} x {}", size[0], size[1])));
    p.set_tech_value("Mounting plate", None);
    Some(size)
}

/// A whole number of the part data (`Rows: 7`), for the fields and rows of a cabinet.
pub fn count_value(p: &Part, key: &str) -> Option<u32> {
    p.tech_value(key)?.split_whitespace().next()?.parse().ok()
}

/// The enclosure stands on the floor: its part says so, or its system is a baying
/// system or a rack.
pub fn enclosure_on_floor(p: &Part) -> bool {
    let system = p.tech_value("Enclosure system").map(str::trim);
    p.floor_standing() || crate::mechanics::SYSTEMS.iter().any(|s| Some(s.id) == system && (s.bayable || s.id == crate::mechanics::RACK_SYSTEM))
}

impl EnclosureFilter {
    pub fn matches(&self, p: &Part) -> bool {
        let system = p.tech_value("Enclosure system").map(str::trim).unwrap_or("");
        let count = |want: Option<u32>, key: &str| want.is_none_or(|n| count_value(p, key) == Some(n));
        is_enclosure(p)
            && p.matches_text(&self.text)
            && (self.system.is_empty() || self.system == system)
            && (self.manufacturer.is_empty() || self.manufacturer == p.manufacturer)
            && match self.mounting {
                EnclosureMounting::Any => true,
                EnclosureMounting::Wall => !enclosure_on_floor(p),
                EnclosureMounting::Floor => enclosure_on_floor(p),
            }
            && self.width.contains(p.width)
            && self.height.contains(p.height)
            && (self.depth.is_open() || (p.depth > 0.0 && self.depth.contains(p.depth)))
            && count(self.fields, "Fields")
            && count(self.rows, "Rows")
    }

    /// True if any criterion restricts the enclosures (the order does not).
    pub fn is_active(&self) -> bool {
        *self != EnclosureFilter { sort: self.sort, ..Default::default() }
    }

    /// Clears all criteria, keeps the order.
    pub fn reset(&mut self) {
        *self = EnclosureFilter { sort: self.sort, ..Default::default() };
    }

    /// The enclosures of `parts` that match, each part number once (the first one
    /// wins), in the order chosen.
    pub fn apply<'a>(&self, parts: impl IntoIterator<Item = &'a Part>) -> Vec<&'a Part> {
        let mut found: Vec<&Part> = Vec::new();
        for p in parts {
            if self.matches(p) && !found.iter().any(|x| x.number == p.number) {
                found.push(p);
            }
        }
        let by_number = |a: &&Part, b: &&Part| a.manufacturer.cmp(&b.manufacturer).then(natural_cmp(&a.number, &b.number));
        let by = |f: fn(&Part) -> f32| move |a: &&Part, b: &&Part| f(a).total_cmp(&f(b)).then(by_number(a, b));
        match self.sort {
            EnclosureSort::Number => found.sort_by(by_number),
            EnclosureSort::Width => found.sort_by(by(|p| p.width)),
            EnclosureSort::Height => found.sort_by(by(|p| p.height)),
            EnclosureSort::Depth => found.sort_by(by(|p| p.depth)),
            EnclosureSort::Size => found.sort_by(by(|p| p.width * p.height)),
        }
        found
    }
}

/// The distinct numbers of a part data key (`Fields`, `Rows`) among the parts,
/// ascending.
pub fn distinct_counts<'a>(parts: impl IntoIterator<Item = &'a Part>, key: &str) -> Vec<u32> {
    let mut v: Vec<u32> = parts.into_iter().filter_map(|p| count_value(p, key)).collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// Distinct non-empty values of a part field, naturally sorted.
pub fn distinct_values<'a>(parts: impl IntoIterator<Item = &'a Part>, field: impl Fn(&Part) -> &str) -> Vec<String> {
    let mut values: Vec<String> = parts.into_iter().map(&field).filter(|v| !v.is_empty()).map(String::from).collect();
    values.sort_by(|a, b| natural_cmp(a, b));
    values.dedup();
    values
}

#[cfg(test)]
mod tests;
