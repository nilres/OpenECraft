//! KiCad netlists (`.net`, the format of *File › Export › Netlist* and of
//! `kicad-cli sch export netlist`): components with their value, footprint and fields,
//! and nets with the pins they connect.

use crate::analysis::Analysis;
use crate::model::{natural_cmp, Project};
use crate::sexpr::{self, quote, Sexpr};
use crate::symbols::Func;
use openecraft_i18n::tr;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

/// Name of the field that carries the part number in both directions.
pub const PART_FIELD: &str = "Part number";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Component {
    /// Reference designator (`J1`); for a netlist written here the device tag.
    pub reference: String,
    pub value: String,
    pub footprint: String,
    pub description: String,
    /// Symbol library of the component (`Connector`).
    pub library: String,
    /// Fields and properties besides the fixed ones, by name.
    pub fields: Vec<(String, String)>,
}

impl Component {
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }
}

/// A pin on a net.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Node {
    pub reference: String,
    pub pin: String,
    /// Name of the pin in its symbol (`VCC`), empty when it has none.
    pub function: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Net {
    pub name: String,
    pub nodes: Vec<Node>,
}

impl Net {
    /// KiCad names the net of a pin that connects nothing after the pin.
    pub fn unconnected(&self) -> bool {
        self.name.starts_with("unconnected-") || self.nodes.len() < 2 && self.name.starts_with("Net-(")
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Netlist {
    /// The schematic file the netlist was made from.
    pub source: String,
    /// Title, company and revision of the title block of the root sheet.
    pub title: String,
    pub company: String,
    pub revision: String,
    pub components: Vec<Component>,
    pub nets: Vec<Net>,
}

impl Netlist {
    /// Name of the design: the title, otherwise the name of the schematic file.
    pub fn name(&self) -> String {
        if !self.title.trim().is_empty() {
            return self.title.trim().to_string();
        }
        let file = self.source.rsplit(['/', '\\']).next().unwrap_or_default();
        file.rsplit_once('.').map_or(file, |(stem, _)| stem).trim().to_string()
    }
}

/// Reads a KiCad netlist.
pub fn read(text: &str) -> Result<Netlist, String> {
    let root = sexpr::parse(text)?;
    if root.name() != "export" {
        return Err(tr("Not a KiCad netlist (in KiCad: File › Export › Netlist, format KiCad)").into());
    }
    let mut out = Netlist::default();
    if let Some(design) = root.child("design") {
        out.source = design.value("source").unwrap_or_default().to_string();
        // The root sheet is the first one.
        if let Some(block) = design.child("sheet").and_then(|s| s.child("title_block")) {
            out.title = block.value("title").unwrap_or_default().to_string();
            out.company = block.value("company").unwrap_or_default().to_string();
            out.revision = block.value("rev").unwrap_or_default().to_string();
        }
    }
    for comp in root.child("components").into_iter().flat_map(|c| c.children("comp")) {
        let mut c = Component {
            reference: comp.value("ref").unwrap_or_default().to_string(),
            value: comp.value("value").unwrap_or_default().to_string(),
            footprint: comp.value("footprint").unwrap_or_default().to_string(),
            description: comp.value("description").unwrap_or_default().to_string(),
            ..Default::default()
        };
        // The description of the library symbol says what kind of component it is, not
        // what this one is for: it is not taken as the component's.
        c.library = comp.child("libsource").and_then(|lib| lib.value("lib")).unwrap_or_default().to_string();
        // `(fields (field (name "MPN") "123"))` and `(property (name "MPN") (value "123"))`.
        for f in comp.child("fields").into_iter().flat_map(|f| f.children("field")) {
            let value = f.args().iter().filter(|a| a.name().is_empty()).find_map(Sexpr::text).unwrap_or_default();
            c.fields.push((f.value("name").unwrap_or_default().to_string(), value.to_string()));
        }
        for p in comp.children("property") {
            let name = p.value("name").unwrap_or_default();
            if !name.is_empty() && c.field(name).is_none() {
                c.fields.push((name.to_string(), p.value("value").unwrap_or_default().to_string()));
            }
        }
        if !c.reference.is_empty() {
            out.components.push(c);
        }
    }
    for net in root.child("nets").into_iter().flat_map(|n| n.children("net")) {
        let nodes = net
            .children("node")
            .map(|n| Node {
                reference: n.value("ref").unwrap_or_default().to_string(),
                pin: n.value("pin").unwrap_or_default().to_string(),
                function: n.value("pinfunction").unwrap_or_default().to_string(),
            })
            .filter(|n| !n.reference.is_empty())
            .collect();
        out.nets.push(Net { name: net.value("name").unwrap_or_default().to_string(), nodes });
    }
    Ok(out)
}

/// The device tag and connection point designation a pin of the netlist stands for;
/// `None` for what is no connection of a device (potentials, interruption points).
fn node(prj: &Project, pin: crate::netlist::PinRef) -> Option<(String, String)> {
    let s = prj.pages.get(pin.page)?.symbols.get(pin.sym)?;
    let def = prj.symbol(&s.sym)?;
    if !def.func.has_tag() || s.tag.is_empty() || matches!(def.func, Func::Overview) {
        return None;
    }
    let own = || s.pins.get(pin.pin).filter(|d| !d.trim().is_empty()).cloned().unwrap_or_else(|| (pin.pin + 1).to_string());
    let designation = match def.func {
        Func::Terminal | Func::Plug(_) => s.label.clone(),
        Func::Plc if def.plc_channel.is_none() && def.plc_channels.is_empty() => s.label.clone(),
        _ => own(),
    };
    Some((s.tag.clone(), if designation.trim().is_empty() { own() } else { designation }))
}

/// A UUID that stays the same for the same name, so that KiCad recognises the
/// components of an earlier netlist.
fn stable_uuid(name: &str) -> String {
    let hash = |seed: u64| name.bytes().fold(seed, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3));
    let (a, b) = (hash(0xcbf2_9ce4_8422_2325), hash(0x8422_2325_cbf2_9ce4));
    format!("{:08x}-{:04x}-4{:03x}-8{:03x}-{:012x}", a >> 32, (a >> 16) & 0xffff, a & 0xfff, b >> 52, b & 0xffff_ffff_ffff)
}

/// The project as a KiCad netlist: every device with electrical connections as a
/// component (reference: its tag, value: the type number of its part), every
/// potential as a net of connection points (`-K1` pin `A1`). Footprints are assigned
/// in KiCad; the part number is carried in the field [`PART_FIELD`].
pub fn export(prj: &Project, an: &Analysis) -> String {
    // Nets first: only devices that connect something are components.
    let mut nets: Vec<(String, Vec<(String, String)>)> = Vec::new();
    let mut names = BTreeSet::new();
    for (i, net) in an.netlist.nets.iter().enumerate().filter(|(_, n)| !n.fluid) {
        let mut nodes: Vec<(String, String)> = net.pins.iter().filter_map(|&p| node(prj, p)).collect();
        nodes.sort_by(|a, b| natural_cmp(&a.0, &b.0).then_with(|| natural_cmp(&a.1, &b.1)));
        nodes.dedup();
        if nodes.is_empty() {
            continue;
        }
        let mut name = if net.name.trim().is_empty() { format!("N{}", i + 1) } else { net.name.trim().to_string() };
        while !names.insert(name.clone()) {
            name.push('_');
        }
        nets.push((name, nodes));
    }
    let tags: BTreeSet<&str> = nets.iter().flat_map(|(_, nodes)| nodes.iter().map(|n| n.0.as_str())).collect();
    let mut tags: Vec<&str> = tags.into_iter().collect();
    tags.sort_by(|a, b| natural_cmp(a, b));
    // The symbol of a device: its main function, otherwise any of its functions.
    let mut symbols: BTreeMap<&str, (&str, bool)> = BTreeMap::new();
    for s in prj.pages.iter().flat_map(|pg| &pg.symbols).filter(|s| !s.tag.is_empty()) {
        let Some(def) = prj.symbol(&s.sym) else { continue };
        let main = matches!(def.func, Func::Main { .. });
        let e = symbols.entry(s.tag.as_str()).or_insert((def.key, main));
        if main && !e.1 {
            *e = (def.key, true);
        }
    }
    let mut out = String::from("(export (version \"E\")\n");
    let _ = writeln!(out, "  (design\n    (source {})\n    (date {})\n    (tool {}))", quote(&prj.info.name), quote(&prj.info.date), quote(concat!("OpenECraft ", env!("CARGO_PKG_VERSION"))));
    out.push_str("  (components");
    for tag in &tags {
        let device = prj.devices.get(*tag);
        let number = device.and_then(|d| d.parts.first()).map(|r| r.number.as_str()).unwrap_or_default();
        let part = prj.parts.iter().chain(crate::parts::catalog()).find(|p| !number.is_empty() && p.number == number);
        let description = device.map(|d| d.description.trim()).unwrap_or_default();
        let of_part = part.map(|p| p.description.trim()).unwrap_or_default();
        let symbol = symbols.get(tag).map(|s| s.0).unwrap_or_default();
        let value = [part.map(|p| p.type_number.trim()).unwrap_or_default(), number, description, of_part, symbol].into_iter().find(|v| !v.is_empty()).unwrap_or_default();
        let _ = write!(out, "\n    (comp (ref {})\n      (value {})\n      (footprint \"\")\n      (description {})", quote(tag), quote(value), quote(description));
        let fields: Vec<(&str, &str)> = [(PART_FIELD, number), ("Manufacturer", part.map(|p| p.manufacturer.as_str()).unwrap_or_default()), ("Order number", part.map(|p| p.order_number.as_str()).unwrap_or_default())]
            .into_iter()
            .filter(|(_, v)| !v.trim().is_empty())
            .collect();
        if !fields.is_empty() {
            out.push_str("\n      (fields");
            for (name, v) in &fields {
                let _ = write!(out, "\n        (field (name {}) {})", quote(name), quote(v));
            }
            out.push(')');
        }
        let _ = write!(out, "\n      (libsource (lib \"OpenECraft\") (part {}) (description {}))", quote(&super::plain_name(symbol)), quote(of_part));
        for (name, v) in &fields {
            let _ = write!(out, "\n      (property (name {}) (value {}))", quote(name), quote(v));
        }
        let _ = write!(out, "\n      (sheetpath (names \"/\") (tstamps \"/\"))\n      (tstamps {}))", quote(&stable_uuid(tag)));
    }
    out.push_str(")\n  (nets");
    for (code, (name, nodes)) in nets.iter().enumerate() {
        let _ = write!(out, "\n    (net (code {}) (name {})", quote(&(code + 1).to_string()), quote(name));
        for (tag, pin) in nodes {
            let _ = write!(out, "\n      (node (ref {}) (pin {}) (pintype \"passive\"))", quote(tag), quote(pin));
        }
        out.push(')');
    }
    out.push_str("))\n");
    out
}
