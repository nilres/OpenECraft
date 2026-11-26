//! Parts imported over parts already known (an EDZ file of the EPLAN Data Portal over a
//! part of a shipped catalog or of the database): the import is merged into the known
//! part instead of replacing it. A known part is the one of the same number, else the
//! one of the same order number from the same manufacturer (or the only one with that
//! order number); it keeps its number. The values the import brings win; what it lacks
//! stays, the classification (group, subgroup, catalog) stays, the technical data are
//! merged line by line (`Key: value`, the import's value for the same key), accessories
//! and the parts an accessory fits are united.

use crate::{Component, Part};
use serde_json::Value;

/// Fields of the known part that stay whatever the import says: OpenECraft's
/// classification, which its checks and views depend on.
const KEPT: &[&str] = &["number", "top_group", "group", "subgroup", "catalog"];

/// Imported parts merged with the parts already known.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Merged {
    pub parts: Vec<Part>,
    /// Imported numbers that became the number of a known part: (imported, known).
    pub renamed: Vec<(String, String)>,
    /// How many imported parts were merged into a known one.
    pub merged: usize,
}

impl Merged {
    /// The number a part of the import has now (for the files of the import).
    pub fn number_of<'a>(&'a self, imported: &'a str) -> &'a str {
        self.renamed.iter().find(|(from, _)| from == imported).map_or(imported, |(_, to)| to.as_str())
    }
}

/// Letters and digits in upper case: `2CPX 054002 R9999` and `ABB Striebel & John` as
/// they are compared.
fn normal(s: &str) -> String {
    s.chars().filter(char::is_ascii_alphanumeric).map(|c| c.to_ascii_uppercase()).collect()
}

/// The known part an imported one is: the one of the same number, else of the same
/// order number (at least 6 letters and digits) from a manufacturer of a similar name,
/// else the only part with that order number.
pub fn find_known<'a>(part: &Part, known: &[&'a Part]) -> Option<&'a Part> {
    if let Some(k) = known.iter().find(|k| k.number.eq_ignore_ascii_case(&part.number)) {
        return Some(k);
    }
    let order = normal(&part.order_number);
    if order.len() < 6 {
        return None;
    }
    let same: Vec<&'a Part> = known.iter().copied().filter(|k| normal(&k.order_number) == order).collect();
    let maker = normal(&part.manufacturer);
    let related = |k: &Part| {
        let other = normal(&k.manufacturer);
        !maker.is_empty() && !other.is_empty() && (other.contains(&maker) || maker.contains(&other))
    };
    if let Some(k) = same.iter().find(|k| related(k)) {
        return Some(k);
    }
    let mut numbers: Vec<&str> = same.iter().map(|k| k.number.as_str()).collect();
    numbers.sort_unstable();
    numbers.dedup();
    (numbers.len() == 1).then(|| same[0])
}

/// A value the import leaves open: missing, the default of a new part, empty, zero or
/// false.
fn is_empty(v: &Value, default: &Value) -> bool {
    v == default
        || match v {
            Value::Null => true,
            Value::Bool(b) => !b,
            Value::Number(n) => n.as_f64() == Some(0.0),
            Value::String(s) => s.trim().is_empty(),
            Value::Array(a) => a.is_empty(),
            Value::Object(o) => o.iter().all(|(k, x)| is_empty(x, default.get(k).unwrap_or(&Value::Null))),
        }
}

fn merge_values(base: &mut Value, new: &Value, default: &Value, top: bool) {
    let (Value::Object(base), Value::Object(new)) = (base, new) else { return };
    for (key, v) in new {
        let d = default.get(key).unwrap_or(&Value::Null);
        let kept = top && KEPT.contains(&key.as_str()) && base.get(key).is_some_and(|old| !is_empty(old, d));
        if kept || is_empty(v, d) {
            continue;
        }
        match base.get_mut(key) {
            Some(old) if old.is_object() && v.is_object() => merge_values(old, v, d, false),
            _ => {
                base.insert(key.clone(), v.clone());
            }
        }
    }
}

/// Technical data of a known part with those of an import: a line `Key: value` of the
/// import replaces the known one of the same key in its place, the other known lines
/// stay, the import's new lines follow.
pub fn merge_lines(known: &str, imported: &str) -> String {
    let key = |l: &str| l.split_once(':').map(|(k, _)| k.trim().to_lowercase()).filter(|k| !k.is_empty());
    let new: Vec<&str> = imported.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    let mut out: Vec<String> = Vec::new();
    for line in known.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let replaced = key(line).and_then(|k| new.iter().find(|n| key(n).as_ref() == Some(&k)));
        out.push(replaced.copied().unwrap_or(line).to_string());
    }
    for line in new {
        if !out.iter().any(|o| o == line || key(line).is_some_and(|k| key(o) == Some(k))) {
            out.push(line.to_string());
        }
    }
    out.join("\n")
}

fn union(known: &[String], imported: &[String]) -> Vec<String> {
    let mut out = known.to_vec();
    for n in imported {
        if !out.contains(n) {
            out.push(n.clone());
        }
    }
    out
}

/// Imported part `imported` merged into the known part `known` (see the module).
pub fn merge(known: &Part, imported: &Part) -> Part {
    let (Ok(mut base), Ok(new), Ok(default)) = (serde_json::to_value(known), serde_json::to_value(imported), serde_json::to_value(Part::default())) else {
        return imported.clone();
    };
    merge_values(&mut base, &new, &default, true);
    let mut out: Part = serde_json::from_value(base).unwrap_or_else(|_| imported.clone());
    out.tech_data = merge_lines(&known.tech_data, &imported.tech_data);
    out.accessories = union(&known.accessories, &imported.accessories);
    out.fits = union(&known.fits, &imported.fits);
    out
}

/// Imported parts merged with the parts already `known` (the first of a number wins):
/// each one that is a known part is merged into it and gets its number; the references
/// between the imported parts (accessories, fits, components, successor) follow.
pub fn merge_imported<'a>(imported: Vec<Part>, known: impl IntoIterator<Item = &'a Part>) -> Merged {
    let known: Vec<&Part> = known.into_iter().collect();
    let mut out = Merged::default();
    for p in imported {
        let part = match find_known(&p, &known) {
            Some(k) => {
                if k.number != p.number {
                    out.renamed.push((p.number.clone(), k.number.clone()));
                }
                out.merged += 1;
                merge(k, &p)
            }
            None => p,
        };
        match out.parts.iter_mut().find(|q| q.number == part.number) {
            Some(q) => *q = merge(q, &part),
            None => out.parts.push(part),
        }
    }
    if !out.renamed.is_empty() {
        let renamed = out.renamed.clone();
        let rename = |n: &mut String| {
            if let Some((_, to)) = renamed.iter().find(|(from, _)| from == n) {
                n.clone_from(to);
            }
        };
        for p in &mut out.parts {
            p.accessories.iter_mut().chain(p.fits.iter_mut()).chain(std::iter::once(&mut p.successor)).for_each(&rename);
            p.components.iter_mut().for_each(|c: &mut Component| rename(&mut c.number));
            for list in [&mut p.accessories, &mut p.fits] {
                let mut seen = std::collections::BTreeSet::new();
                list.retain(|n| seen.insert(n.clone()));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
