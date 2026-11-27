//! KBL harness data (VDA 4964, schema KBL 2.4 SR-1 of prostep ivip) for harness
//! manufacturing and the exchange with harness tools. One container per harness:
//! the parts (general wires and cables with their cores, connector housings with
//! slots and cavities, terminals, seals, cavity plugs, wire protections, fixings,
//! accessories, co-pack parts, assemblies, components), the harness with its
//! occurrences, the connections between contact points, the schematic connections
//! (potentials), wiring groups, the project options as modules, and the 3D topology
//! of [`crate::harness::topology`]: nodes, segments with centre curve, cross-section
//! area, protection areas and fixing assignments, the routing of every wire and the
//! dimensions with the default tolerances. Element order, ids and references follow
//! the schema (checked against it in the tests).

use crate::analysis::Analysis;
use crate::harness::topology;
use crate::harness::{self, Harness, HarnessWire};
use crate::model::{natural_cmp, Project};
use crate::parts::{FnKind, Part};
use openecraft_parts::vec3::{cross, norm, V3};
use std::collections::BTreeMap;
use std::fmt::Write;

pub const VERSION: &str = "2.4";
pub const NAMESPACE: &str = "http://www.prostep.org/Car_electric_container/KBL2.3/KBLSchema";
/// Default length tolerances: (lengths up to, mm; ± mm).
pub const TOLERANCES: [(f32, f32); 4] = [(500.0, 5.0), (1500.0, 10.0), (3000.0, 15.0), (100_000.0, 20.0)];
/// Language codes of the schema (ISO 639-1, capitalised).
const LANGUAGES: &str = "Aa Ab Af Am Ar As Ay Az Ba Be Bg Bh Bi Bn Bo Br Ca Co Cs Cy Da De Dz El En Eo Es Et Eu Fa Fi Fj Fo Fr Fy Ga Gd Gl Gn Gu Ha Hi He Hr Hu Hy Ia Id Ie Ik In Is It Iu Iw Ja Ji Jw Ka Kk Kl Km Kn Ko Ks Ku Ky La Ln Lo Lt Lv Mg Mi Mk Ml Mn Mo Mr Ms Mt My Na Ne Nl No Oc Om Or Pa Pl Ps Pt Qu Rm Rn Ro Ru Rw Sa Sd Sg Sh Si Sk Sl Sm Sn So Sq Sr Ss St Su Sv Sw Ta Te Tg Th Zi Tk Tl Tn To Tr Ts Tt Tw Ug Uk Ur Uz Vi Vo Wo Xh Yi Yo Za Zh Zu";
const MM: &str = "U_MM";
const MM2: &str = "U_MM2";
const GRAM: &str = "U_G";

/// An element of the document. Complex elements get an id when they are written
/// (every complex type of the schema has a required id).
struct X {
    name: &'static str,
    id: Option<String>,
    complex: bool,
    attrs: Vec<(&'static str, String)>,
    text: String,
    kids: Vec<X>,
}

fn c(name: &'static str) -> X {
    X { name, id: None, complex: true, attrs: Vec::new(), text: String::new(), kids: Vec::new() }
}

fn cid(name: &'static str, id: impl Into<String>) -> X {
    X { id: Some(id.into()), ..c(name) }
}

fn t(name: &'static str, text: impl Into<String>) -> X {
    X { name, id: None, complex: false, attrs: Vec::new(), text: text.into(), kids: Vec::new() }
}

impl X {
    fn k(mut self, x: X) -> X {
        self.kids.push(x);
        self
    }

    fn ks(mut self, xs: impl IntoIterator<Item = X>) -> X {
        self.kids.extend(xs);
        self
    }

    /// A text element, if the text is not empty.
    fn opt(self, name: &'static str, text: &str) -> X {
        if text.trim().is_empty() { self } else { self.k(t(name, text)) }
    }

    /// An id reference list, if it is not empty.
    fn refs(self, name: &'static str, ids: &[String]) -> X {
        if ids.is_empty() { self } else { self.k(t(name, ids.join(" "))) }
    }

    fn attr(mut self, key: &'static str, value: &str) -> X {
        self.attrs.push((key, value.into()));
        self
    }

    fn write(&self, depth: usize, counter: &mut usize, out: &mut String) {
        let _ = write!(out, "{:1$}<{2}", "", depth * 2, self.name);
        if self.complex {
            let id = self.id.clone().unwrap_or_else(|| {
                *counter += 1;
                format!("I{counter}")
            });
            let _ = write!(out, " id=\"{}\"", crate::xml::esc(&id));
        }
        for (k, v) in &self.attrs {
            let _ = write!(out, " {k}=\"{}\"", crate::xml::esc(v));
        }
        if self.kids.is_empty() {
            if self.complex {
                out.push_str("/>\n");
            } else {
                let _ = writeln!(out, ">{}</{}>", crate::xml::esc(&self.text), self.name);
            }
            return;
        }
        out.push_str(">\n");
        for kid in &self.kids {
            kid.write(depth + 1, counter, out);
        }
        let _ = writeln!(out, "{:1$}</{2}>", "", depth * 2, self.name);
    }
}

/// Number for the file: at most three decimals, no trailing zeros.
fn f(v: f32) -> String {
    let v = if v.abs() < 5e-4 { 0.0 } else { v };
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

fn num(name: &'static str, v: f32, unit: &str) -> X {
    c(name).k(t("Unit_component", unit)).k(t("Value_component", f(v)))
}

fn range(name: &'static str, min: f32, max: f32, unit: &str) -> X {
    c(name).k(t("Unit_component", unit)).k(t("Minimum", f(min))).k(t("Maximum", f(max)))
}

fn instruction(name: &'static str, kind: &str, value: &str) -> X {
    c(name).k(t("Instruction_type", kind)).k(t("Instruction_value", value))
}

fn wire_length(kind: &str, mm: f32) -> X {
    c("Length_information").k(t("Length_type", kind)).k(num("Length_value", mm, MM))
}

fn colour(name: &'static str, value: &str) -> X {
    c(name).k(t("Colour_type", "Base colour")).k(t("Colour_value", value))
}

fn tolerance(length: f32) -> f32 {
    TOLERANCES.iter().find(|(up_to, _)| length <= *up_to).map(|t| t.1).unwrap_or(TOLERANCES[TOLERANCES.len() - 1].1)
}

fn find_part<'a>(prj: &'a Project, number: &str) -> Option<&'a Part> {
    (!number.is_empty()).then(|| prj.parts.iter().chain(crate::parts::catalog()).find(|p| p.number == number)).flatten()
}

/// The fields of the abstract part: number, company, version, abbreviation,
/// description (from the parts data), mass.
fn part_fields(x: X, prj: &Project, number: &str, fallback: &str) -> X {
    let p = find_part(prj, number);
    let description = p.map(|p| p.description.as_str()).filter(|d| !d.is_empty()).unwrap_or(fallback);
    let x = x
        .k(t("Part_number", number))
        .k(t("Company_name", p.map(|p| p.manufacturer.as_str()).unwrap_or("")))
        .k(t("Version", "1"))
        .k(t("Abbreviation", p.map(|p| p.type_number.as_str()).filter(|s| !s.is_empty()).unwrap_or(number)))
        .k(t("Description", description));
    match p.filter(|p| p.weight > 0.0) {
        Some(p) => x.k(num("Mass_information", p.weight * 1000.0, GRAM)),
        None => x,
    }
}

/// Parts of one kind, each once: (part number, id).
#[derive(Default)]
struct Catalog {
    prefix: &'static str,
    list: Vec<String>,
}

impl Catalog {
    fn new(prefix: &'static str) -> Catalog {
        Catalog { prefix, list: Vec::new() }
    }

    fn id(&mut self, number: &str) -> String {
        let i = match self.list.iter().position(|n| n == number) {
            Some(i) => i,
            None => {
                self.list.push(number.to_string());
                self.list.len() - 1
            }
        };
        format!("{}{}", self.prefix, i + 1)
    }
}

/// Cartesian points, each position once.
#[derive(Default)]
struct Points {
    list: Vec<V3>,
}

impl Points {
    fn id(&mut self, q: V3) -> String {
        let q = q.map(|v| (v * 1000.0).round() / 1000.0);
        let i = match self.list.iter().position(|p| *p == q) {
            Some(i) => i,
            None => {
                self.list.push(q);
                self.list.len() - 1
            }
        };
        format!("CP{}", i + 1)
    }
}

/// A placement at `at` with the direction `u` (and a perpendicular `v`).
fn placement(points: &mut Points, at: V3, u: V3) -> X {
    let up = if u[2].abs() > 0.9 { [0.0, 1.0, 0.0] } else { [0.0, 0.0, 1.0] };
    let v = norm(cross(up, u));
    c("Placement").ks(u.iter().map(|x| t("U", f(*x)))).ks(v.iter().map(|x| t("V", f(*x)))).k(t("Cartesian_point", points.id(at)))
}

/// The connection points of a device's parts: cavities of its housing.
fn part_pins(prj: &Project, device: &str) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    let plug = crate::plugs::plug_tags(prj).iter().any(|p| p == device);
    if plug {
        v.extend(crate::plugs::designations(prj, device));
    }
    for r in prj.devices.get(device).map(|d| d.parts.as_slice()).unwrap_or(&[]) {
        if let Some(p) = find_part(prj, &r.number) {
            for func in p.functions.iter().filter(|f| !plug || f.kind == FnKind::PlugPin) {
                v.extend(func.pin_list());
            }
        }
    }
    v
}

fn add_unique(v: &mut Vec<String>, s: &str) {
    if !v.iter().any(|x| x == s) {
        v.push(s.to_string());
    }
}

/// Year of the project date (first four digits in a row).
fn year(date: &str) -> String {
    let digits: Vec<char> = date.chars().collect();
    digits.windows(4).find(|w| w.iter().all(char::is_ascii_digit) && !matches!(w[0], '0')).map(|w| w.iter().collect()).unwrap_or_default()
}

/// Mass of the wires: copper and PVC insulation, g.
fn wire_mass(wires: &[HarnessWire]) -> f32 {
    wires
        .iter()
        .map(|w| {
            let cs = crate::production::cross_section_mm2(&w.cross_section).unwrap_or(1.0);
            let d = harness::outer_diameter(&w.cross_section);
            let insulation = (std::f32::consts::PI * d * d / 4.0 - cs).max(0.0);
            w.length * (cs * 0.008_96 + insulation * 0.001_4)
        })
        .sum()
}

/// File name of the KBL file of a harness.
pub fn file_name(tag: &str) -> String {
    format!("{}.kbl", tag.trim_start_matches(['-', '=', '+']).replace(['/', '\\', ' ', ':', '-', '=', '+'], "_"))
}

/// KBL files of all harnesses: (file name, content).
pub fn all(prj: &Project, an: &Analysis) -> Vec<(String, String)> {
    prj.harnesses.iter().map(|h| (file_name(&h.tag), container(prj, an, h))).collect()
}

/// All harnesses as a ZIP archive of KBL files.
pub fn zip(prj: &Project, an: &Analysis) -> Vec<u8> {
    let mut z = crate::zip::ZipWriter::new(&prj.info.date);
    for (name, text) in all(prj, an) {
        z.add(&name, text.as_bytes());
    }
    z.finish()
}

/// The KBL file of one harness; without a tag the only harness of the project.
pub fn export(prj: &Project, an: &Analysis, tag: Option<&str>) -> Result<String, String> {
    let h = match tag.map(str::trim).filter(|t| !t.is_empty()) {
        Some(tag) => prj.harnesses.iter().find(|h| h.tag == tag).ok_or_else(|| openecraft_i18n::tr_fmt!("No harness {}", tag))?,
        None => match prj.harnesses.as_slice() {
            [h] => h,
            [] => return Err(openecraft_i18n::tr("The project has no harnesses").into()),
            _ => return Err(openecraft_i18n::tr_fmt!("The project has several harnesses: {}", prj.harnesses.iter().map(|h| h.tag.as_str()).collect::<Vec<_>>().join(", "))),
        },
    };
    Ok(container(prj, an, h))
}

/// A wire end of the harness: wire index, device, pin.
struct WireEnd<'a> {
    wire: usize,
    device: &'a str,
    pin: &'a str,
}

/// The KBL container of one harness.
#[allow(clippy::too_many_lines)]
pub fn container(prj: &Project, an: &Analysis, h: &Harness) -> String {
    let ws = harness::wires(prj, an, &h.tag);
    let cons = harness::connectors(&ws);
    let topo = topology::build(prj, an, h, &ws);
    let mut points = Points::default();
    let mut wires_cat = Catalog::new("GW");
    let mut housings = Catalog::new("CH");
    let mut terminals = Catalog::new("GT");
    let mut seals = Catalog::new("CS");
    let mut plugs_cat = Catalog::new("CPL");
    let mut protections_cat = Catalog::new("WP");
    let mut fixings_cat = Catalog::new("FX");
    let mut extra_cat: BTreeMap<&'static str, Catalog> = BTreeMap::new();
    let ends: Vec<WireEnd> = ws
        .iter()
        .enumerate()
        .flat_map(|(i, w)| [WireEnd { wire: i, device: &w.from.0, pin: &w.from.1 }, WireEnd { wire: i, device: &w.to.0, pin: &w.to.1 }])
        .collect();

    // ------------------------------------------------------------ wires and cables
    let general_key = |w: &HarnessWire| format!("{} {}", w.cross_section, w.color).trim().to_string();
    let mut cables: Vec<&str> = Vec::new();
    for w in &ws {
        if w.cable.is_empty() {
            wires_cat.id(&general_key(w));
        } else if !cables.contains(&w.cable.as_str()) {
            cables.push(&w.cable);
        }
    }
    let cable_of = |tag: &str| prj.cables.iter().find(|c| c.tag == tag);
    let cable_part = |tag: &str| cable_of(tag).map(|c| if c.part.is_empty() { c.cable_type.clone() } else { c.part.clone() }).filter(|p| !p.is_empty()).unwrap_or_else(|| tag.to_string());
    let cable_ids: Vec<String> = cables.iter().map(|c| wires_cat.id(&cable_part(c))).collect();
    let core_ids = |cable: &str| -> Vec<String> { cable_of(cable).map(|c| c.core_list()).unwrap_or_default() };
    let bend = prj.layout.routing.bend_wire;
    let mut general_wires: Vec<X> = Vec::new();
    for (gi, number) in wires_cat.list.clone().iter().enumerate() {
        let id = format!("GW{}", gi + 1);
        let x = cid("General_wire", &id);
        if let Some(ci) = cable_ids.iter().position(|c| *c == id) {
            let tag = cables[ci];
            let cab = cable_of(tag);
            let cs = cab.map(|c| c.cross_section.clone()).unwrap_or_default();
            let mut x = part_fields(x, prj, number, &cab.map(|c| c.description.clone()).filter(|d| !d.is_empty()).unwrap_or_else(|| tag.to_string()))
                .k(t("Cable_designator", tag))
                .opt("Wire_type", cab.map(|c| c.cable_type.as_str()).unwrap_or(""));
            if let Some(cab) = cab {
                let d = crate::routing3d::cable_diameter(cab);
                x = x.k(num("Bend_radius", prj.layout.routing.bend_cable * d, MM)).k(num("Outside_diameter", d, MM));
            }
            let area = crate::production::cross_section_mm2(&cs).unwrap_or(0.0);
            let cores = core_ids(tag)
                .iter()
                .enumerate()
                .map(|(k, core)| {
                    cid("Core", format!("{id}_C{}", k + 1))
                        .k(t("Id", core.as_str()))
                        .k(num("Cross_section_area", area, MM2))
                        .k(num("Outside_diameter", harness::outer_diameter(&cs), MM))
                        .k(colour("Core_colour", core))
                })
                .collect::<Vec<_>>();
            general_wires.push(x.ks(cores).k(colour("Cover_colour", "")));
        } else {
            let w = ws.iter().find(|w| w.cable.is_empty() && general_key(w) == *number).cloned().unwrap_or_default();
            let d = harness::outer_diameter(&w.cross_section);
            let mut x = part_fields(x, prj, number, number).k(num("Bend_radius", bend * d, MM));
            if let Some(area) = crate::production::cross_section_mm2(&w.cross_section) {
                x = x.k(num("Cross_section_area", area, MM2));
            }
            general_wires.push(x.k(num("Outside_diameter", d, MM)).k(colour("Cover_colour", &w.color)));
        }
    }

    // ------------------------------------------------------------ connector housings
    let plug_tags = crate::plugs::plug_tags(prj);
    let housing_number = |dev: &str| prj.devices.get(dev).and_then(|d| d.parts.first()).map(|r| r.number.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| dev.to_string());
    let housing_of: Vec<String> = cons.iter().map(|con| housings.id(&housing_number(&con.device))).collect();
    // Cavities of a housing: the pins of its parts and every pin used at it.
    let mut cavities: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (con, hid) in cons.iter().zip(&housing_of) {
        let list = cavities.entry(hid.clone()).or_default();
        for p in part_pins(prj, &con.device) {
            add_unique(list, &p);
        }
        for (pin, _, _) in &con.pins {
            add_unique(list, pin);
        }
    }
    for list in cavities.values_mut() {
        list.sort_by(|a, b| natural_cmp(a, b));
    }
    let cavity_index = |hid: &str, pin: &str| cavities.get(hid).and_then(|l| l.iter().position(|p| p == pin)).unwrap_or(0);
    let housing_xml: Vec<X> = housings
        .list
        .iter()
        .enumerate()
        .map(|(i, number)| {
            let hid = format!("CH{}", i + 1);
            let dev = cons.iter().zip(&housing_of).find(|(_, h)| **h == hid).map(|(c, _)| c.device.as_str()).unwrap_or("");
            let plug = plug_tags.iter().any(|p| p == dev);
            let list = cavities.get(&hid).cloned().unwrap_or_default();
            let slot = cid("Slots", format!("{hid}_SL1"))
                .k(t("Id", "1"))
                .k(t("Number_of_cavities", list.len().to_string()))
                .ks(list.iter().enumerate().map(|(k, p)| cid("Cavities", format!("{hid}_CV{}", k + 1)).k(t("Cavity_number", p.as_str()))));
            part_fields(cid("Connector_housing", hid.clone()), prj, number, number).k(t("Housing_type", if plug { "connector" } else { "device terminals" })).k(slot)
        })
        .collect();

    // ------------------------------------------------------------ terminals, seals, plugs
    let end_parts: Vec<(String, String, f32)> = ends.iter().map(|e| harness::end_parts(prj, h, e.device, e.pin, &ws[e.wire].cross_section)).collect();
    let mut terminal_occ: Vec<X> = Vec::new();
    let mut seal_occ: Vec<X> = Vec::new();
    let mut plug_occ: Vec<X> = Vec::new();
    let mut end_assoc: Vec<Vec<String>> = vec![Vec::new(); ends.len()];
    for (i, (terminal, seal, _)) in end_parts.iter().enumerate() {
        if !terminal.is_empty() {
            let id = format!("TO{}", terminal_occ.len() + 1);
            terminal_occ.push(cid("Terminal_occurrence", &id).k(t("Id", format!("{}:{}", ends[i].device, ends[i].pin))).k(t("Part", terminals.id(terminal))));
            end_assoc[i].push(id);
        }
        if !seal.is_empty() {
            let id = format!("SO{}", seal_occ.len() + 1);
            seal_occ.push(cid("Cavity_seal_occurrence", &id).k(t("Id", format!("{}:{}", ends[i].device, ends[i].pin))).k(t("Part", seals.id(seal))));
            end_assoc[i].push(id);
        }
    }

    // ------------------------------------------------------------ connector occurrences
    let co_id = |dev: &str| cons.iter().position(|c| c.device == dev).map(|i| format!("CO{}", i + 1)).unwrap_or_default();
    let contact_id = |dev: &str, pin: &str| {
        let ci = cons.iter().position(|c| c.device == dev).unwrap_or(0);
        let pins = used_pins(&cons[ci]);
        format!("CO{}_CP{}", ci + 1, pins.iter().position(|p| p == pin).unwrap_or(0) + 1)
    };
    let cavity_occ_id = |dev: &str, pin: &str| {
        let ci = cons.iter().position(|c| c.device == dev).unwrap_or(0);
        format!("CO{}_CV{}", ci + 1, cavity_index(&housing_of[ci], pin) + 1)
    };
    let mut connector_occ: Vec<X> = Vec::new();
    for (ci, con) in cons.iter().enumerate() {
        let id = format!("CO{}", ci + 1);
        let hid = &housing_of[ci];
        let mut x = cid("Connector_occurrence", &id).k(t("Id", con.device.as_str())).opt("Description", prj.devices.get(&con.device).map(|d| d.description.as_str()).unwrap_or(""));
        if let Some(usage) = h.ends.iter().find(|e| e.device == con.device && e.pin.is_empty()).map(|e| e.usage.as_str()).filter(|u| harness::USAGES.contains(u)) {
            x = x.k(t("Usage", usage));
        }
        if let Some(n) = topo.node_of(&con.device) {
            let dir = topo.segments.iter().enumerate().find(|(_, s)| s.start == n || s.end == n).map(|(si, s)| {
                let (a, b) = topo.vectors(si);
                if s.start == n { a } else { b.map(|v| -v) }
            });
            x = x.k(placement(&mut points, topo.nodes[n].pos, dir.unwrap_or([1.0, 0.0, 0.0])));
        }
        x = x.k(t("Part", hid.as_str()));
        for (pi, pin) in used_pins(con).iter().enumerate() {
            let assoc: Vec<String> = ends.iter().enumerate().filter(|(_, e)| e.device == con.device && e.pin == pin).flat_map(|(i, _)| end_assoc[i].clone()).collect();
            x = x.k(cid("Contact_points", format!("{id}_CP{}", pi + 1)).k(t("Id", pin.as_str())).refs("Associated_parts", &assoc).k(t("Contacted_cavity", cavity_occ_id(&con.device, pin))));
        }
        let used = used_pins(con);
        let plug_part = h.ends.iter().find(|e| e.device == con.device && e.pin.is_empty()).map(|e| e.cavity_plug.clone()).unwrap_or_default();
        let mut slot = cid("Slots", format!("{id}_SL1")).k(t("Part", format!("{hid}_SL1")));
        for (k, cav) in cavities.get(hid).cloned().unwrap_or_default().iter().enumerate() {
            let mut cx = cid("Cavities", format!("{id}_CV{}", k + 1));
            if !plug_part.is_empty() && !used.contains(cav) {
                let pid = format!("PO{}", plug_occ.len() + 1);
                plug_occ.push(cid("Cavity_plug_occurrence", &pid).k(t("Id", format!("{}:{cav}", con.device))).k(t("Part", plugs_cat.id(&plug_part))));
                cx = cx.k(t("Associated_plug", pid));
            }
            slot = slot.k(cx.k(t("Part", format!("{hid}_CV{}", k + 1))));
        }
        connector_occ.push(x.k(slot));
    }

    // ------------------------------------------------------------ wire occurrences, connections
    let wire_occ_id = |i: usize| -> String {
        let w = &ws[i];
        if w.cable.is_empty() {
            format!("W{}", i + 1)
        } else {
            let ci = cables.iter().position(|c| *c == w.cable).unwrap_or(0);
            format!("SW{}_W{}", ci + 1, i + 1)
        }
    };
    let lengths = |i: usize| -> Vec<X> {
        let mut v = Vec::new();
        if let Some(l) = topo.wire_length(i).filter(|l| *l > 0.0) {
            v.push(wire_length("DMU", l));
        }
        if ws[i].length > 0.0 || v.is_empty() {
            v.push(wire_length(if ws[i].routed { "Production" } else { "Nominal" }, ws[i].length));
        }
        v
    };
    let mut wire_occ: Vec<X> = Vec::new();
    for (i, w) in ws.iter().enumerate().filter(|(_, w)| w.cable.is_empty()) {
        wire_occ.push(
            cid("General_wire_occurrence", wire_occ_id(i))
                .attr("xsi:type", "kbl:Wire_occurrence")
                .k(t("Part", wires_cat.id(&general_key(w))))
                .k(instruction("Installation_information", "Marking", &w.number))
                .ks(lengths(i))
                .k(t("Wire_number", w.number.as_str())),
        );
    }
    for (ci, tag) in cables.iter().enumerate() {
        let gid = &cable_ids[ci];
        let cores = core_ids(tag);
        let cab_length = cable_of(tag).map(|c| c.length * 1000.0).unwrap_or(0.0);
        let mut x = cid("General_wire_occurrence", format!("SW{}", ci + 1)).attr("xsi:type", "kbl:Special_wire_occurrence").k(t("Part", gid.as_str()));
        x = x.k(wire_length("Nominal", cab_length)).k(t("Special_wire_id", *tag));
        for (i, w) in ws.iter().enumerate().filter(|(_, w)| w.cable == *tag) {
            let k = cores.iter().position(|c| *c == w.core).unwrap_or(0);
            x = x.k(cid("Core_occurrence", wire_occ_id(i)).k(t("Wire_number", if w.number.is_empty() { w.core.as_str() } else { w.number.as_str() })).k(t("Part", format!("{gid}_C{}", k + 1))).ks(lengths(i)));
        }
        wire_occ.push(x);
    }
    // Schematic connections: the potentials at the harness ends.
    let mut nets: Vec<&str> = ws.iter().map(|w| w.net.as_str()).filter(|n| !n.is_empty()).collect();
    nets.sort_by(|a, b| natural_cmp(a, b));
    nets.dedup();
    let schematic: Vec<X> = nets
        .iter()
        .enumerate()
        .map(|(i, net)| {
            let mut cav: Vec<String> = Vec::new();
            for e in ends.iter().filter(|e| ws[e.wire].net == *net) {
                add_unique(&mut cav, &cavity_occ_id(e.device, e.pin));
            }
            let voltage = prj.potentials.iter().find(|p| p.name == *net).map(|p| p.voltage.as_str()).unwrap_or("");
            cid("Schematic_connection", format!("SC{}", i + 1)).k(t("Id", *net)).k(t("Signal_name", *net)).opt("Nominal_voltage", voltage).k(t("Cavities", cav.join(" ")))
        })
        .collect();
    let connections: Vec<X> = ws
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let voltage = prj.potentials.iter().find(|p| p.name == w.net).map(|p| p.voltage.as_str()).unwrap_or("");
            let mut x = cid("Connection", format!("CN{}", i + 1)).opt("Id", &w.number).opt("Signal_name", &w.net).opt("Nominal_voltage", voltage).k(t("External_references", "ER1"));
            if let Some(n) = nets.iter().position(|n| *n == w.net) {
                x = x.k(t("Realized_schematic_connection", format!("SC{}", n + 1)));
            }
            x = x.k(t("Wire", wire_occ_id(i)));
            for (k, e) in [&w.from, &w.to].into_iter().enumerate() {
                let strip = end_parts[2 * i + k].2;
                let mut ext = c("Extremities").k(t("Position_on_wire", if k == 0 { "0" } else { "1" })).k(t("Contact_point", contact_id(&e.0, &e.1)));
                if strip > 0.0 {
                    ext = ext.k(instruction("Processing_information", "Stripping length", &format!("{} mm", f(strip))));
                }
                x = x.k(ext);
            }
            x
        })
        .collect();
    // Wiring groups: wires sharing a contact point.
    let mut groups: Vec<X> = Vec::new();
    for con in &cons {
        for pin in used_pins(con) {
            let at: Vec<String> = ends.iter().filter(|e| e.device == con.device && e.pin == pin).map(|e| wire_occ_id(e.wire)).collect();
            if at.len() > 1 {
                groups.push(cid("Wiring_group", format!("WG{}", groups.len() + 1)).k(t("Id", format!("{}:{pin}", con.device))).k(t("Type", "common contact point")).k(t("Assigned_wire", at.join(" "))));
            }
        }
    }

    // ------------------------------------------------------------ protections, fixings, further parts
    let segs_of = |seg: &str| -> Vec<usize> { if seg.is_empty() { (0..topo.segments.len()).collect() } else { topo.segment_index(seg).into_iter().collect() } };
    let mut protection_occ: Vec<X> = Vec::new();
    let mut protection_areas: Vec<Vec<X>> = (0..topo.segments.len()).map(|_| Vec::new()).collect();
    for (i, p) in h.protections.iter().enumerate() {
        let id = format!("WPO{}", i + 1);
        let number = if p.part.is_empty() { p.kind.key().to_string() } else { p.part.clone() };
        let (a, b) = (p.start.clamp(0.0, 1.0).min(p.end.clamp(0.0, 1.0)), p.end.clamp(0.0, 1.0).max(p.start.clamp(0.0, 1.0)));
        let segs = segs_of(&p.segment);
        let length: f32 = segs.iter().map(|&s| topo.segments[s].length * (b - a)).sum();
        protection_occ.push(cid("Wire_protection_occurrence", &id).k(t("Id", p.id.as_str())).k(num("Protection_length", length, MM)).k(t("Part", protections_cat.id(&number))));
        for s in segs {
            let l = topo.segments[s].length;
            let mut area = c("Protection_area").k(t("Start_location", f(a))).k(num("Absolute_start_location", a * l, MM)).k(t("End_location", f(b))).k(num("Absolute_end_location", b * l, MM));
            if p.kind == harness::ProtectionKind::Tape {
                area = area.k(t("Taping_direction", "clockwise"));
            }
            protection_areas[s].push(area.k(t("Associated_protection", id.as_str())));
        }
    }
    let mut fixing_occ: Vec<X> = Vec::new();
    let mut fixing_assign: Vec<Vec<X>> = (0..topo.segments.len()).map(|_| Vec::new()).collect();
    for (i, fx) in h.fixings.iter().enumerate() {
        let id = format!("FXO{}", i + 1);
        let number = if fx.part.is_empty() { fx.kind.key().to_string() } else { fx.part.clone() };
        let at = fx.location.clamp(0.0, 1.0);
        let mut x = cid("Fixing_occurrence", &id).k(t("Id", fx.id.as_str()));
        if let Some(s) = topo.segment_index(&fx.segment) {
            let seg = &topo.segments[s];
            let q = topology::point_at(&seg.points, at);
            let dir = topo.vectors(s).0;
            x = x.k(placement(&mut points, q, dir));
            fixing_assign[s].push(c("Fixing_assignment").k(t("Location", f(at))).k(num("Absolute_location", at * seg.length, MM)).ks(dir.iter().map(|v| t("Orientation", f(*v)))).k(t("Fixing", id.as_str())));
        }
        fixing_occ.push(x.k(t("Part", fixings_cat.id(&number))));
    }
    let kinds: [(harness::PartKind, &'static str, &'static str, &'static str); 4] = [
        (harness::PartKind::Accessory, "ACC", "Accessory", "Accessory_occurrence"),
        (harness::PartKind::Assembly, "AP", "Assembly_part", "Assembly_part_occurrence"),
        (harness::PartKind::CoPack, "CPP", "Co_pack_part", "Co_pack_occurrence"),
        (harness::PartKind::Component, "CMP", "Component", "Component_occurrence"),
    ];
    let mut extra_occ: BTreeMap<&'static str, Vec<X>> = BTreeMap::new();
    let mut at_node: Vec<Vec<String>> = vec![Vec::new(); topo.nodes.len()];
    for (n, node) in topo.nodes.iter().enumerate() {
        at_node[n].extend(node.connectors.iter().map(|d| co_id(d)).filter(|id| !id.is_empty()));
    }
    for (kind, prefix, _, occ) in kinds {
        for p in h.parts.iter().filter(|p| p.kind == kind && !p.part.is_empty()) {
            let pid = extra_cat.entry(prefix).or_insert_with(|| Catalog::new(prefix)).id(&p.part);
            let node = topo.node_index(&p.node);
            for _ in 0..p.count.max(1) {
                let list = extra_occ.entry(occ).or_default();
                let id = format!("{prefix}O{}", list.len() + 1);
                let mut x = cid(occ, &id).k(t("Id", p.part.as_str())).opt("Description", &p.description);
                if let (Some(n), true) = (node, kind != harness::PartKind::CoPack && kind != harness::PartKind::Component) {
                    x = x.k(placement(&mut points, topo.nodes[n].pos, [1.0, 0.0, 0.0]));
                }
                if kind == harness::PartKind::Component {
                    if let Some(n) = node {
                        let mounted: Vec<String> = topo.nodes[n].connectors.iter().map(|d| co_id(d)).filter(|id| !id.is_empty()).collect();
                        x = x.refs("Mounting", &mounted);
                    }
                }
                if let (Some(n), harness::PartKind::Accessory) = (node, kind) {
                    at_node[n].push(id.clone());
                }
                list.push(x.k(t("Part", pid.as_str())));
            }
        }
    }

    // ------------------------------------------------------------ topology
    let node_xml: Vec<X> = topo
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let mut x = cid("Node", format!("ND{}", i + 1)).k(t("Id", n.id.as_str()));
            if n.bend_radius > 0.0 {
                x = x.k(num("Bend_radius", n.bend_radius, MM));
            }
            x.k(t("Cartesian_point", points.id(n.pos))).refs("Referenced_components", &at_node[i])
        })
        .collect();
    let mut segment_xml: Vec<X> = Vec::new();
    let mut dimensions: Vec<X> = Vec::new();
    for (i, s) in topo.segments.iter().enumerate() {
        let id = format!("SG{}", i + 1);
        let (a, b) = topo.vectors(i);
        let control: Vec<String> = s.points.iter().map(|q| points.id(*q)).collect();
        let mut x = cid("Segment", &id)
            .k(t("Id", s.id.as_str()))
            .ks(a.iter().map(|v| t("Start_vector", f(*v))))
            .ks(b.iter().map(|v| t("End_vector", f(*v))))
            .k(num("Virtual_length", s.length, MM))
            .k(num("Physical_length", s.physical, MM))
            .k(t("Form", "circular"))
            .k(t("End_node", format!("ND{}", s.end + 1)))
            .k(t("Start_node", format!("ND{}", s.start + 1)))
            .k(c("Center_curve").k(t("Degree", "1")).k(t("Control_points", control.join(" "))));
        if s.diameter > 0.0 {
            x = x.k(c("Cross_section_area_information").k(t("Value_determination", "calculated")).k(num("Area", std::f32::consts::PI * s.diameter * s.diameter / 4.0, MM2)));
        }
        x = x.ks(std::mem::take(&mut fixing_assign[i]));
        x = x.k(instruction("Processing_information", "Bundle diameter", &format!("{} mm", f(s.diameter))));
        segment_xml.push(x.ks(std::mem::take(&mut protection_areas[i])));
        dimensions.push(
            cid("Dimension_specification", format!("DS{}", i + 1))
                .k(t("Id", s.id.as_str()))
                .k(num("Dimension_value", s.length, MM))
                .k(t("Segments", id.as_str()))
                .k(t("Origin", format!("ND{}", s.start + 1)))
                .k(t("Target", format!("ND{}", s.end + 1)))
                .k(c("Tolerance_indication").k(num("Lower_limit", -tolerance(s.length), MM)).k(num("Upper_limit", tolerance(s.length), MM))),
        );
    }
    let mut routings: Vec<X> = Vec::new();
    let mut routed_cables: Vec<&str> = Vec::new();
    for (i, w) in ws.iter().enumerate() {
        let Some(route) = &topo.routes[i] else { continue };
        let target = if w.cable.is_empty() {
            wire_occ_id(i)
        } else if routed_cables.contains(&w.cable.as_str()) {
            continue;
        } else {
            routed_cables.push(&w.cable);
            format!("SW{}", cables.iter().position(|c| *c == w.cable).unwrap_or(0) + 1)
        };
        let segs: Vec<String> = route.iter().map(|s| format!("SG{}", s + 1)).collect();
        routings.push(cid("Routing", format!("RT{}", routings.len() + 1)).k(t("Routed_wire", target)).refs("Segments", &segs));
    }

    // ------------------------------------------------------------ the harness
    let number = if h.part.is_empty() { h.tag.clone() } else { h.part.clone() };
    let mut hx = cid("Harness", "H1")
        .k(t("Part_number", number.as_str()))
        .k(t("Company_name", prj.info.company.as_str()))
        .k(t("Version", prj.info.revision.as_str()))
        .k(t("Abbreviation", h.tag.as_str()))
        .k(t("Description", h.description.as_str()));
    for lang in &prj.languages.targets {
        let code: String = lang.chars().take(2).enumerate().map(|(i, ch)| if i == 0 { ch.to_ascii_uppercase() } else { ch.to_ascii_lowercase() }).collect();
        let text = prj.languages.translate(&h.description, lang);
        if LANGUAGES.split(' ').any(|l| l == code) && !h.description.is_empty() && text != h.description {
            hx = hx.k(c("Localized_description").k(t("Language_code", code)).k(t("Value", text)));
        }
    }
    // Wires and the housings of the plugs (the devices are not part of the harness).
    let housing_mass: f32 = cons
        .iter()
        .filter(|c| plug_tags.contains(&c.device))
        .filter_map(|c| prj.devices.get(&c.device)?.parts.first())
        .filter_map(|r| find_part(prj, &r.number))
        .map(|p| p.weight * 1000.0)
        .sum();
    let mass = wire_mass(&ws) + housing_mass;
    if mass > 0.0 {
        hx = hx.k(num("Mass_information", mass, GRAM));
    }
    hx = hx.k(t("External_references", "ER1"));
    for (i, r) in prj.revisions.iter().enumerate() {
        hx = hx.k(cid("Change", format!("CHG{}", i + 1)).k(t("Id", r.name.as_str())).opt("Description", &r.description).opt("Change_date", &r.date).k(t("Responsible_designer", r.author.as_str())).k(t("Designer_department", prj.info.company.as_str())));
    }
    hx = hx
        .opt("Project_number", &prj.info.name)
        .k(t("Car_classification_level_2", if prj.info.description.is_empty() { prj.info.name.as_str() } else { prj.info.description.as_str() }))
        .k(t("Model_year", year(&prj.info.date)))
        .k(t("Content", "harness complete set"))
        .ks(schematic)
        .ks(extra_occ.remove("Accessory_occurrence").unwrap_or_default())
        .ks(extra_occ.remove("Assembly_part_occurrence").unwrap_or_default())
        .ks(plug_occ)
        .ks(seal_occ)
        .ks(extra_occ.remove("Co_pack_occurrence").unwrap_or_default())
        .ks(extra_occ.remove("Component_occurrence").unwrap_or_default())
        .ks(connections)
        .ks(connector_occ)
        .ks(fixing_occ)
        .ks(wire_occ)
        .ks(terminal_occ)
        .ks(protection_occ)
        .ks(groups);
    let (modules, configuration) = modules(prj, an, h, &ws, &cons, &wire_occ_id);
    if let Some(configuration) = configuration {
        hx = hx.k(configuration);
    }
    hx = hx.ks(modules);

    // ------------------------------------------------------------ the container
    let extra_parts = |prefix: &'static str, element: &'static str, kind: &'static str, extra_cat: &BTreeMap<&'static str, Catalog>| -> Vec<X> {
        extra_cat
            .get(prefix)
            .map(|cat| {
                cat.list
                    .iter()
                    .enumerate()
                    .map(|(i, n)| {
                        let x = part_fields(cid(element, format!("{prefix}{}", i + 1)), prj, n, n);
                        if kind.is_empty() { x } else { x.k(t(kind, "harness")) }
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let simple_parts = |cat: &Catalog, element: &'static str, extra: &dyn Fn(&str) -> Vec<X>| -> Vec<X> {
        cat.list.iter().enumerate().map(|(i, n)| part_fields(cid(element, format!("{}{}", cat.prefix, i + 1)), prj, n, n).ks(extra(n))).collect()
    };
    let protection_kind = |n: &str| -> Vec<X> {
        let kind = h.protections.iter().find(|p| p.part == n || (p.part.is_empty() && p.kind.key() == n)).map(|p| p.kind.key()).unwrap_or("");
        vec![t("Protection_type", kind)]
    };
    let fixing_kind = |n: &str| -> Vec<X> {
        let kind = h.fixings.iter().find(|x| x.part == n || (x.part.is_empty() && x.kind.key() == n)).map(|x| x.kind.key()).unwrap_or("");
        vec![t("Fixing_type", kind)]
    };
    let terminal_kind = |n: &str| -> Vec<X> {
        match crate::production::FerruleCode::of_part(n) {
            Some((_, fe)) => vec![t("Terminal_type", "ferrule"), range("Cross_section_area", fe.cross_section, fe.cross_section, MM2)],
            None => vec![t("Terminal_type", "crimp contact")],
        }
    };
    let mut root = c("kbl:KBL_container").attr("xmlns:kbl", NAMESPACE).attr("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance").attr("version_id", VERSION);
    root.id = Some("KBL1".into());
    let created = c("Creation").k(t("Name", prj.info.creator.as_str())).k(t("Department", prj.info.company.as_str())).k(t("Date", prj.info.date.as_str())).k(t("Is_applied_to", "H1"));
    let approvals: Vec<X> = prj
        .revisions
        .iter()
        .filter(|r| r.state == crate::model::revision::State::Released)
        .map(|r| c("Approval").opt("Name", &r.author).opt("Department", &prj.info.company).k(t("Date", r.date.as_str())).k(t("Type_of_approval", format!("release {}", r.name))).k(t("Is_applied_to", "H1")))
        .collect();
    let defaults: Vec<X> = TOLERANCES
        .iter()
        .scan(0.0f32, |from, (up_to, tol)| {
            let x = c("Default_dimension_specification")
                .k(range("Dimension_value_range", *from, *up_to, MM))
                .k(t("Tolerance_type", "length"))
                .k(c("Tolerance_indication").k(num("Lower_limit", -tol, MM)).k(num("Upper_limit", *tol, MM)));
            *from = *up_to;
            Some(x)
        })
        .collect();
    let reference = cid("External_reference", "ER1")
        .k(t("Document_type", "schematic"))
        .k(t("Document_number", prj.info.name.as_str()))
        .k(t("Change_level", prj.info.revision.as_str()))
        .k(t("Data_format", "OpenECraft project"))
        .k(t("Creating_system", "OpenECraft"));
    let units = [(MM, "mm", None), (MM2, "mm2", Some("square")), (GRAM, "g", None)].map(|(id, name, dim)| {
        let x = cid("Unit", id).k(t("Unit_name", name)).k(t("Si_unit_name", if id == GRAM { "gram" } else { "metre" }));
        let x = if id == GRAM { x } else { x.k(t("Si_prefix", "milli")) };
        match dim {
            Some(d) => x.k(t("Si_dimension", d)),
            None => x,
        }
    });
    // Containers are written in the order of the schema; points last collected.
    let accessory = extra_parts("ACC", "Accessory", "Accessory_type", &extra_cat);
    let assembly = extra_parts("AP", "Assembly_part", "Part_type", &extra_cat);
    let copack = extra_parts("CPP", "Co_pack_part", "Part_type", &extra_cat);
    let component = extra_parts("CMP", "Component", "", &extra_cat);
    let plug_parts = simple_parts(&plugs_cat, "Cavity_plug", &|_| Vec::new());
    let seal_parts = simple_parts(&seals, "Cavity_seal", &|_| Vec::new());
    let fixing_parts = simple_parts(&fixings_cat, "Fixing", &fixing_kind);
    let terminal_parts = simple_parts(&terminals, "General_terminal", &terminal_kind);
    let protection_parts = simple_parts(&protections_cat, "Wire_protection", &protection_kind);
    let cps: Vec<X> = points.list.iter().enumerate().map(|(i, q)| cid("Cartesian_point", format!("CP{}", i + 1)).ks(q.iter().map(|v| t("Coordinates", f(*v))))).collect();
    root = root
        .ks(accessory)
        .ks(approvals)
        .ks(assembly)
        .ks(cps)
        .ks(plug_parts)
        .ks(seal_parts)
        .ks(copack)
        .ks(component)
        .ks(housing_xml)
        .k(created)
        .ks(defaults)
        .ks(dimensions)
        .k(reference)
        .ks(fixing_parts)
        .ks(terminal_parts)
        .ks(general_wires)
        .k(hx)
        .ks(node_xml)
        .ks(routings)
        .ks(segment_xml)
        .ks(units)
        .ks(protection_parts);
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- KBL (VDA 4964) harness data, exported by OpenECraft -->\n");
    root.write(0, &mut 0, &mut out);
    out
}

/// Pins of a connector with wires, each once, in pin order.
fn used_pins(con: &harness::Connector) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for (pin, _, _) in &con.pins {
        add_unique(&mut v, pin);
    }
    v
}

/// The project options that contain devices or wires of the harness, as modules of a
/// module family, and the harness configuration of the active options.
fn modules(prj: &Project, an: &Analysis, h: &Harness, ws: &[HarnessWire], cons: &[harness::Connector], wire_occ: &dyn Fn(usize) -> String) -> (Vec<X>, Option<X>) {
    let mut out = Vec::new();
    let mut active = Vec::new();
    let conns: Vec<&crate::netlist::Connection> = an.connections.iter().filter(|c| c.harness == h.tag).collect();
    let symbol = |p: crate::netlist::PinRef| prj.pages[p.page].symbols[p.sym].id;
    for (i, o) in prj.model_options.options.iter().enumerate() {
        let mut controlled: Vec<String> = Vec::new();
        for (ci, con) in cons.iter().enumerate() {
            let member = prj.pages.iter().flat_map(|pg| pg.symbols.iter()).any(|s| s.tag == con.device && o.members.contains(&s.id));
            if member {
                controlled.push(format!("CO{}", ci + 1));
            }
        }
        for (wi, w) in ws.iter().enumerate() {
            let hit = conns.iter().any(|c| c.number == w.number && (o.members.contains(&symbol(c.a)) || o.members.contains(&symbol(c.b))) && crate::harness::key(&crate::netlist::pin_designation(prj, c.a), &crate::netlist::pin_designation(prj, c.b)) == crate::harness::key(&joined(&w.from), &joined(&w.to)));
            if hit {
                controlled.push(wire_occ(wi));
            }
        }
        if controlled.is_empty() {
            continue;
        }
        let id = format!("MD{}", i + 1);
        if o.active {
            active.push(id.clone());
        }
        let logistic = if o.condition.is_empty() { o.name.clone() } else { o.condition.clone() };
        out.push(
            cid("Module", &id)
                .k(t("Part_number", format!("{}-{}", if h.part.is_empty() { &h.tag } else { &h.part }, o.name)))
                .k(t("Company_name", prj.info.company.as_str()))
                .k(t("Version", prj.info.revision.as_str()))
                .k(t("Abbreviation", o.name.as_str()))
                .k(t("Description", o.description.as_str()))
                .k(t("Car_classification_level_2", prj.info.name.as_str()))
                .k(t("Model_year", year(&prj.info.date)))
                .k(t("Content", "module"))
                .k(c("Module_configuration").k(t("Logistic_control_information", logistic)).k(t("Configuration_type", if o.condition.is_empty() { "module list" } else { "option code" })).refs("Controlled_components", &controlled)),
        );
    }
    if active.is_empty() {
        return (out, None);
    }
    let configuration = cid("Harness_configuration", "HC1")
        .k(t("Part_number", if h.part.is_empty() { h.tag.as_str() } else { h.part.as_str() }))
        .k(t("Company_name", prj.info.company.as_str()))
        .k(t("Version", prj.info.revision.as_str()))
        .k(t("Abbreviation", h.tag.as_str()))
        .k(t("Description", "active options"))
        .k(t("Car_classification_level_2", prj.info.name.as_str()))
        .k(t("Model_year", year(&prj.info.date)))
        .k(t("Modules", active.join(" ")));
    (out, Some(configuration))
}

fn joined(e: &(String, String)) -> String {
    if e.1.is_empty() { e.0.clone() } else { format!("{}:{}", e.0, e.1) }
}

#[cfg(test)]
mod tests;
