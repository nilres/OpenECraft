//! Level 13: the distribution cabinet of an apartment building with an 88 kWp PV system
//! and transformer-rated metering, built like a Hager univers cabinet. The supply cable
//! lands on the busbars beside the surge arrester and the connection point zone (APZ);
//! the consumption metering (NH1 fuse switch disconnector, transformer module, voltage
//! taps, load break switch) feeds the distribution busbar with the heat pumps and the
//! general consumers; the coupling switch of the grid and system protection (a 4-pole
//! circuit breaker with motor operator, undervoltage release and auxiliary switch,
//! tripped by the voltage and frequency relay, the fire brigade switch at the entrance
//! and closed again after a delay) leads to the generation metering and the busbar of
//! the inverter feeders. The two meters of the distribution system operator sit in
//! meter cabinets of their own, wired over the test terminal blocks.
//!
//! The NH1 fuse switch disconnectors and the current transformers stand on univers N
//! component modules (accessories under the part), the release and the auxiliary switch
//! are built into the circuit breaker, the cabinet comes with its plinth.

use super::*;
use crate::model::panel::Enclosure;
use crate::panel::{self, Face, ItemKind};
use crate::structure::Aspect;

pub(super) fn project() -> Project {
    let mut prj = Project { pages: Vec::new(), ..Default::default() };
    prj.info = ProjectInfo {
        name: "Transformer-rated metering with PV".into(),
        description: "Distribution cabinet of an apartment building\nwith consumption and generation metering and NA protection".into(),
        company: "OpenECraft".into(),
        creator: "OpenECraft".into(),
        date: "2026-11-02".into(),
        revision: "A".into(),
        ..Default::default()
    };
    prj.apply_preset(crate::potentials::Preset::Iec60204);
    // A distribution room of a building, not a machine: 25 °C around the conductors.
    prj.electrical.conditions.ambient = 25.0;
    prj.potentials.push(crate::potentials::PotentialDef {
        name: "PEN".into(),
        kind: crate::potentials::PotentialKind::Pe,
        color: "GNYE".into(),
        cross_section: "120 mm²".into(),
        ..Default::default()
    });
    for (aspect, id, desc) in [
        (Aspect::Function, "GRID", "Supply and surge protection"),
        (Aspect::Function, "MS1", "Consumption metering"),
        (Aspect::Function, "DIST", "Distribution: heat pumps and general consumers"),
        (Aspect::Function, "NA", "Grid and system protection"),
        (Aspect::Function, "MS2", "Generation metering"),
        (Aspect::Function, "PV", "Inverter feeders"),
        (Aspect::Location, "HV", "Distribution cabinet"),
        (Aspect::Location, "ZS1", "Meter cabinet consumption"),
        (Aspect::Location, "ZS2", "Meter cabinet generation"),
        (Aspect::Location, "HAK", "House connection box"),
        (Aspect::Location, "ENT", "Building entrance"),
        (Aspect::Location, "TR", "Technical room"),
        (Aspect::Location, "UVA", "Sub-distribution general consumers"),
        (Aspect::Location, "INV", "Inverter room"),
    ] {
        prj.structures.define(aspect, id, desc);
    }
    let general = crate::sections::add(&mut prj, 0, "General");
    let supply = crate::sections::add(&mut prj, 0, "Supply and consumption metering");
    let dist = crate::sections::add(&mut prj, 0, "Distribution");
    let na = crate::sections::add(&mut prj, 0, "Grid and system protection");
    let generation = crate::sections::add(&mut prj, 0, "Generation");
    let layout = crate::sections::add(&mut prj, 0, "Panel layout");
    let reports = crate::sections::add(&mut prj, 0, "Reports");

    page(&mut prj, general, "1", "Cover sheet", PageKind::Cover, "", "");
    page(&mut prj, general, "2", "Table of contents", PageKind::Report(ReportKind::TableOfContents.into()), "", "");
    single_line(&mut prj, general);
    supply_page(&mut prj, supply);
    metering(&mut prj, supply, &CONSUMPTION);
    distribution(&mut prj, dist);
    coupling_switch(&mut prj, na);
    na_control(&mut prj, na);
    metering(&mut prj, generation, &GENERATION);
    inverter_feeders(&mut prj, generation);
    for (n, desc, kind) in [
        ("1", "Bill of materials", ReportKind::BillOfMaterials),
        ("2", "Device list", ReportKind::DeviceList),
        ("3", "Terminal diagram", ReportKind::TerminalDiagram),
        ("4", "Cable overview", ReportKind::CableOverview),
        ("5", "Panel legend", ReportKind::PanelLegend),
    ] {
        page(&mut prj, reports, n, desc, PageKind::Report(kind.into()), "", "");
    }

    let db = PartsDb { parts: crate::parts::bundled::default_parts() };
    assign_parts(&mut prj, &db, DEVICES);
    for (tag, part, count) in COUNTED {
        crate::parts::add_to_device(&mut prj, &db, tag, part, *count);
    }
    cables(&mut prj, &db, CABLES);
    enclosures(&mut prj, &db);
    let ids: Vec<(u64, String)> = prj.layout.enclosures.iter().map(|e| (e.id, e.name.clone())).collect();
    for (n, (id, name)) in ids.iter().enumerate() {
        page(&mut prj, layout, &(n + 1).to_string(), &format!("Mounting layout {name}"), PageKind::Mounting(*id), "", "");
    }
    crate::sections::sort_pages(&mut prj);
    crate::sizing::size_wires(&mut prj);
    number_wires(&mut prj);
    prj
}

/// Device tags and their parts: the main part first, then its accessories.
const DEVICES: &[(&str, &[&str])] = &[
    ("=GRID+HAK-F1", &["GEN.NH2-BASE-3P-400A"]),
    ("=GRID+HV-X0", &["GEN.BB-FEED-185"]),
    ("=GRID+HV-X1", &["GEN.TB-PEN-150"]),
    ("=GRID+HV-F1", &["HAG.SPA811Z"]),
    ("=GRID+HV-F2", &["HAG.NBN116"]),
    ("=GRID+HV-X2", &["GEN.SOCKET-DIN-16A"]),
    ("=MS1+HV-F10", &["HAG.LT153", "HAG.UE31E1B"]),
    ("=MS1+HV-T1", &["GEN.CT-SET-250-5A", "HAG.UE31SS400"]),
    ("=MS1+HV-F11", &["HAG.L73M"]),
    ("=MS1+HV-Q10", &["HAG.HA354"]),
    ("=MS1+ZS1-X1", &["GEN.TB-DISC-4"]),
    ("=MS1+ZS1-P1", &["GEN.METER-3PT-CT"]),
    ("=DIST+HV-Q21", &["HAG.LT056"]),
    ("=DIST+HV-Q22", &["HAG.LT056"]),
    ("=DIST+HV-X21", &["GEN.TB-35", "GEN.TB-N-35", "GEN.TB-PE-35"]),
    ("=DIST+HV-X22", &["GEN.TB-35", "GEN.TB-N-35", "GEN.TB-PE-35"]),
    ("=DIST+TR-EQ1", &["GEN.HP-AW-12KW"]),
    ("=DIST+UVA-E1", &["GEN.SUBDIST-48M"]),
    ("=NA+HV-Q30", &["HAG.HNT161GR", "HAG.HXT041H", "HAG.HXA011H", "HAG.HXA021H"]),
    ("=NA+HV-F31", &["HAG.MCN306"]),
    ("=NA+HV-K31", &["HAG.EU400"]),
    ("=NA+HV-F32", &["HAG.MCN106"]),
    ("=NA+HV-T31", &["HAG.HTG911H"]),
    ("=NA+HV-F33", &["HAG.MCN106"]),
    ("=NA+HV-K32", &["HAG.EZM100"]),
    ("=NA+HV-P30", &["GEN.LMP-DIN-RD-230"]),
    ("=NA+HV-P31", &["GEN.LMP-DIN-GN-230"]),
    ("=NA+HV-P32", &["GEN.LMP-RD-24"]),
    ("=NA+HV-X30", &["PXC.3209510"]),
    ("=NA+ENT-S1", &["GEN.FIRE-SW-1NC1NO"]),
    ("=MS2+HV-F40", &["HAG.LT153", "HAG.UE31E1B"]),
    ("=MS2+HV-T2", &["GEN.CT-SET-250-5A", "HAG.UE31SS400"]),
    ("=MS2+HV-F41", &["HAG.L73M"]),
    ("=MS2+ZS2-X1", &["GEN.TB-DISC-4"]),
    ("=MS2+ZS2-P2", &["GEN.METER-3PT-CT"]),
    ("=PV+HV-Q41", &["HAG.LT056"]),
    ("=PV+HV-Q42", &["HAG.LT056"]),
    ("=PV+HV-X41", &["GEN.TB-35", "GEN.TB-N-35", "GEN.TB-PE-35"]),
    ("=PV+HV-X42", &["GEN.TB-35", "GEN.TB-N-35", "GEN.TB-PE-35"]),
    ("=PV+INV-TB1", &["GEN.INV-PV-40KW"]),
    ("=PV+INV-TB2", &["GEN.INV-PV-40KW"]),
];

/// Parts used more than once by a device: fuse links (terminal strips take theirs
/// from their layout).
const COUNTED: &[(&str, &str, u32)] = &[
    ("=GRID+HAK-F1", "GEN.NH2-GG-250A", 3),
    ("=MS1+HV-F10", "GEN.NH1-GG-125A", 3),
    ("=MS1+HV-F11", "GEN.D01-GG-10A", 3),
    ("=DIST+HV-Q21", "GEN.NH00-GG-50A", 3),
    ("=DIST+HV-Q22", "GEN.NH00-GG-80A", 3),
    ("=MS2+HV-F40", "GEN.NH1-GG-125A", 3),
    ("=MS2+HV-F41", "GEN.D01-GG-10A", 3),
    ("=PV+HV-Q41", "GEN.NH00-GG-80A", 3),
    ("=PV+HV-Q42", "GEN.NH00-GG-80A", 3),
];

/// Cables: tag, part, length in m, what it connects.
const CABLES: &[(&str, &str, f32, &str)] = &[
    ("=GRID-W1", "GEN.CBL-NYY-4X120", 15.0, "Supply from the house connection box"),
    ("=MS1-W2", "GEN.CBL-NYY-12X2.5", 3.0, "Meter cabinet consumption"),
    ("=MS2-W3", "GEN.CBL-NYY-12X2.5", 3.0, "Meter cabinet generation"),
    ("=NA-W30", "GEN.CBL-NYM-3X2.5", 20.0, "Fire brigade switch"),
    ("=DIST-W21", "GEN.CBL-NYY-5X16", 25.0, "Heat pump"),
    ("=DIST-W22", "GEN.CBL-NYY-5X35", 10.0, "Sub-distribution general consumers"),
    ("=PV-W41", "GEN.CBL-NYY-5X35", 30.0, "Inverter 1"),
    ("=PV-W42", "GEN.CBL-NYY-5X35", 30.0, "Inverter 2"),
];

/// Cross-section of the bars of the busbars (Cu 30 x 10 mm).
const BUSBAR: &str = "300 mm²";
/// Cross-section of the conductors between the devices behind the 125 A fuses.
pub(super) const LINK: &str = "50 mm²";
/// Cross-section of the short-circuit proof taps to the small devices.
pub(super) const TAP: &str = "2.5 mm²";
/// Connection points of a three-pole switch fed at its lower connection points (the
/// even ones, at the top of the symbol): power flows upwards through it.
pub(super) const TURNED: [&str; 6] = ["2", "4", "6", "1", "3", "5"];

/// A page of a section with its structure; the name starts with the section number.
pub(super) fn page(prj: &mut Project, section: u64, n: &str, desc: &str, kind: PageKind, function: &str, location: &str) -> usize {
    let name = format!("{}.{n}", crate::sections::full_number(prj, section));
    let i = add_page(prj, &name, desc, kind);
    let page = &mut prj.pages[i];
    page.section = section;
    page.function = function.into();
    page.location = location.into();
    i
}

/// Wire carrying core `core` of cable `cable`; a plain wire without a cable.
pub(super) fn core(g: &mut Pg, (x1, y1): (f32, f32), (x2, y2): (f32, f32), cable: &str, core: &str) {
    g.w(x1, y1, x2, y2);
    if let Some(w) = g.prj.pages[g.page].wires.last_mut().filter(|_| !cable.is_empty()) {
        w.cable = cable.into();
        w.core = core.into();
    }
}

/// The wire drawn last has the cross-section `cs`.
pub(super) fn cross_section(g: &mut Pg, cs: &str) {
    if let Some(w) = g.prj.pages[g.page].wires.last_mut() {
        w.cross_section = cs.into();
    }
}

/// Interruption point (page connector) at a wire end.
pub(super) fn link(g: &mut Pg, x: f32, y: f32, name: &str) {
    g.sym("intr", x, y, 0, "", name, "");
}

/// Conductors at the top of a page (L1 24 … PE 56), coming from the interruption
/// points `from` on the left; each rail runs to the x of its last tap, N and PE on to
/// the interruption points `to` on the right if given.
pub(super) fn rails(g: &mut Pg, from: &str, ends: [f32; 5], to: Option<&str>) {
    for (i, (name, end)) in ["L1", "L2", "L3", "N", "PE"].into_iter().zip(ends).enumerate() {
        let y = 24.0 + 8.0 * i as f32;
        link(g, 20.0, y, &format!("{from}-{name}"));
        match to.filter(|_| i >= 3) {
            Some(to) => {
                g.w(20.0, y, 380.0, y);
                link(g, 380.0, y, &format!("{to}-{name}"));
            }
            None => {
                g.w(20.0, y, end, y);
            }
        }
        if i < 3 {
            cross_section(g, BUSBAR);
        }
    }
}

/// Overview: supply, consumption metering, distribution busbar with its feeders,
/// coupling switch, generation metering and the busbar of the inverters.
fn single_line(prj: &mut Project, section: u64) {
    let i = page(prj, section, "3", "Single-line diagram", PageKind::Overview, "", "");
    let mut g = Pg { prj, page: i };
    g.text(48.0, 22.0, "3/PEN AC 400 V 50 Hz, TN-C-S; -W1 NYY-J 4 x 120 from the house connection box", 2.5);
    g.w(60.0, 28.0, 60.0, 40.0);
    g.sym("sl_fuse_switch", 60.0, 48.0, 0, "=MS1+HV-F10", "", "NH1 3 x 125 A");
    g.w(60.0, 56.0, 60.0, 64.0);
    g.sym("sl_meter", 60.0, 72.0, 0, "=MS1+ZS1-P1", "", "Consumption, two directions");
    g.w(60.0, 80.0, 60.0, 88.0);
    g.sym("sl_disc", 60.0, 96.0, 0, "=MS1+HV-Q10", "", "250 A");
    g.w(60.0, 104.0, 60.0, 116.0).w(40.0, 116.0, 300.0, 116.0);
    g.text(310.0, 117.0, "Distribution busbar", 2.5);
    for (x, tag, text) in [(100.0, "=DIST+HV-Q21", "Heat pumps"), (140.0, "=DIST+HV-Q22", "General consumers")] {
        g.w(x, 116.0, x, 124.0);
        g.sym("sl_fuse_switch", x, 132.0, 0, tag, "", text);
    }
    g.w(220.0, 116.0, 220.0, 124.0);
    g.sym("sl_cb", 220.0, 132.0, 0, "=NA+HV-Q30", "", "Coupling switch 4P 160 A");
    g.w(220.0, 140.0, 220.0, 148.0);
    g.sym("sl_meter", 220.0, 156.0, 0, "=MS2+ZS2-P2", "", "Generation");
    g.w(220.0, 164.0, 220.0, 172.0);
    g.sym("sl_fuse_switch", 220.0, 180.0, 0, "=MS2+HV-F40", "", "NH1 3 x 125 A");
    g.w(220.0, 188.0, 220.0, 196.0).w(200.0, 196.0, 320.0, 196.0);
    g.text(330.0, 197.0, "Busbar of the inverters, PV 88 kWp", 2.5);
    for (x, tag) in [(240.0, "=PV+HV-Q41"), (280.0, "=PV+HV-Q42")] {
        g.w(x, 196.0, x, 204.0);
        g.sym("sl_fuse_switch", x, 212.0, 0, tag, "", "Inverter");
    }
    g.text(240.0, 140.0, "Grid and system protection =NA", 2.5);
}

/// The supply cable on the busbars, the PEN split into N and PE, the surge arrester
/// tapped straight from the bars, the supply of the connection point zone.
fn supply_page(prj: &mut Project, section: u64) {
    let i = page(prj, section, "1", "Supply, surge protection and connection point zone", PageKind::Schematic, "GRID", "HV");
    let mut g = Pg { prj, page: i };
    for (x, key, label) in [(60.0, "pot_l1", "L1"), (68.0, "pot_l2", "L2"), (76.0, "pot_l3", "L3"), (84.0, "pot", "PEN")] {
        g.sym(key, x, 20.0, 0, "", label, "");
    }
    // The 250 A pre-fuse in the house connection box, below the 315 A the surge
    // arrester may be backed up with.
    g.sym("fuse3", 60.0, 32.0, 0, "+HAK-F1", "", "House connection box\nNH2 3 x 250 A gG");
    for (x, c) in [(60.0, "BN"), (68.0, "BK"), (76.0, "GY")] {
        g.w(x, 24.0, x, 28.0);
        core(&mut g, (x, 36.0), (x, 68.0), "=GRID-W1", c);
    }
    core(&mut g, (84.0, 24.0), (84.0, 68.0), "=GRID-W1", "GNYE");
    g.text(96.0, 52.0, "-W1 NYY-J 4 x 120 SM from the house connection box, in from below", 2.5);
    // The busbar terminals turned: the cable comes to them from below.
    for (i, x) in [60.0, 68.0, 76.0].into_iter().enumerate() {
        g.sym("term", x, 72.0, 2, "-X0", &format!("L{}", i + 1), "");
    }
    g.sym("term_pe", 84.0, 72.0, 0, "-X1", "PEN", "");
    for (x, y) in [(60.0, 120.0), (68.0, 128.0), (76.0, 136.0)] {
        g.path(&[(x, 76.0), (x, y), (380.0, y)]);
        cross_section(&mut g, BUSBAR);
    }
    g.path(&[(84.0, 76.0), (84.0, 144.0), (380.0, 144.0)]).path(&[(84.0, 144.0), (84.0, 152.0), (380.0, 152.0)]);
    for (y, name) in [(120.0, "L1"), (128.0, "L2"), (136.0, "L3"), (144.0, "N"), (152.0, "PE")] {
        link(&mut g, 380.0, y, &format!("HV-{name}"));
    }
    g.text(96.0, 112.0, "Busbars Cu 30 x 10 mm; PEN split into N and PE", 2.5);
    // Combined arrester type 1 + 2 + 3: backup fuse up to 315 A, so straight from the
    // bars over short-circuit proof conductors.
    for (i, (x, y, l)) in [(140.0, 120.0, "L1"), (148.0, 128.0, "L2"), (156.0, 136.0, "L3"), (164.0, 144.0, "N")].into_iter().enumerate() {
        let text = if i == 0 { "Surge protection T1 + T2 + T3" } else { "" };
        g.sym("spd", x, 180.0, 0, "-F1", "", text).pins(&[l, "PE"]).w(x, y, x, 176.0);
        cross_section(&mut g, "16 mm²");
        g.w(x, 184.0, x, 192.0);
        cross_section(&mut g, "16 mm²");
    }
    g.w(140.0, 192.0, 172.0, 192.0);
    cross_section(&mut g, "16 mm²");
    g.w(172.0, 152.0, 172.0, 192.0);
    cross_section(&mut g, "16 mm²");
    // Connection point zone (APZ) and room for additional equipment (RfZ), sealed.
    g.sym("cb1", 220.0, 180.0, 0, "-F2", "", "APZ / RfZ").w(220.0, 120.0, 220.0, 176.0);
    cross_section(&mut g, TAP);
    g.w(220.0, 184.0, 220.0, 196.0);
    g.sym("socket", 220.0, 200.0, 0, "-X2", "", "Supply of the APZ");
    g.w(228.0, 144.0, 228.0, 196.0).w(236.0, 152.0, 236.0, 196.0);
    g.text(248.0, 216.0, "Connection point zone and room for additional equipment: sealed area before the meter", 2.5);
}

/// A transformer-rated metering: the devices in the cabinet and the meter with its test
/// terminal block in the meter cabinet.
pub(super) struct Metering {
    pub function: &'static str,
    /// Location of the page (the cabinet).
    pub location: &'static str,
    pub n: &'static str,
    pub desc: &'static str,
    /// Interruption points of the conductors coming in on the left.
    pub input: &'static str,
    /// Interruption points of N and PE on the right and of L1 … L3 at the bottom.
    pub output: &'static str,
    /// Fuse switch disconnector before the transformers; empty: none (the transformers
    /// sit in the busbar).
    pub fuse: &'static str,
    pub fuse_text: &'static str,
    /// Its connection points: those at the top of the symbol first.
    pub fuse_pins: [&'static str; 6],
    pub ct: &'static str,
    /// Ratio of the transformers.
    pub ratio: &'static str,
    /// Cross-section of the conductors through the transformers and on from the switch
    /// after them.
    pub primary: &'static str,
    pub taps: &'static str,
    pub meter: &'static str,
    pub meter_text: &'static str,
    pub strip: &'static str,
    pub cable: &'static str,
    pub cable_text: &'static str,
    /// Load break switch after the transformers (system side isolation).
    pub switch: Option<&'static str>,
    /// Symbol of the switch: a load break switch, or a fuse switch disconnector with
    /// solid links.
    pub switch_symbol: &'static str,
    /// Text of the switch.
    pub switch_text: &'static str,
    /// Its connection points: those at the top of the symbol first.
    pub switch_pins: [&'static str; 6],
}

const CONSUMPTION: Metering = Metering {
    function: "MS1",
    location: "HV",
    n: "2",
    desc: "Consumption metering",
    input: "HV",
    output: "DB",
    fuse: "-F10",
    fuse_text: "Isolation before the meter\nNH1 3 x 125 A gG",
    fuse_pins: TURNED,
    ct: "-T1",
    ratio: "250/5 A",
    primary: LINK,
    taps: "-F11",
    meter: "+ZS1-P1",
    meter_text: "Meter of the DSO, two directions",
    strip: "+ZS1-X1",
    cable: "=MS1-W2",
    cable_text: "-W2 NYY-J 12 x 2.5 to the meter cabinet +ZS1",
    switch: Some("-Q10"),
    switch_symbol: "disc3",
    switch_text: "Isolation after the meter\n3 x 250 A",
    switch_pins: TURNED,
};

const GENERATION: Metering = Metering {
    function: "MS2",
    location: "HV",
    n: "1",
    desc: "Generation metering",
    // From the busbar of the inverters up to the coupling switch.
    input: "PVB",
    output: "PV",
    fuse: "-F40",
    fuse_text: "Isolation before the meter\nNH1 3 x 125 A gG",
    fuse_pins: TURNED,
    ct: "-T2",
    ratio: "250/5 A",
    primary: LINK,
    taps: "-F41",
    meter: "+ZS2-P2",
    meter_text: "Generation meter of the DSO",
    strip: "+ZS2-X1",
    cable: "=MS2-W3",
    cable_text: "-W3 NYY-J 12 x 2.5 to the meter cabinet +ZS2",
    switch: None,
    switch_symbol: "disc3",
    switch_text: "",
    switch_pins: TURNED,
};

pub(super) fn metering(prj: &mut Project, section: u64, m: &Metering) {
    let i = page(prj, section, m.n, m.desc, PageKind::Schematic, m.function, m.location);
    let mut g = Pg { prj, page: i };
    rails(&mut g, m.input, [120.0, 128.0, 136.0, 0.0, 0.0], Some(m.output));
    // The phases spread to the transformers: from a fuse switch disconnector before
    // them, or straight from the bars the transformers sit in.
    let fused = !m.fuse.is_empty();
    if fused {
        g.sym("fuse_swdisc3", 120.0, 72.0, 0, m.fuse, "", m.fuse_text).pins(&m.fuse_pins);
        g.w(120.0, 24.0, 120.0, 68.0).w(128.0, 32.0, 128.0, 68.0).w(136.0, 40.0, 136.0, 68.0);
    }
    let from = |rail: f32| if fused { 76.0 } else { rail };
    g.w(120.0, from(24.0), 120.0, 112.0);
    g.path(&[(128.0, from(32.0)), (128.0, 80.0), (152.0, 80.0), (152.0, 136.0)]);
    g.path(&[(136.0, from(40.0)), (136.0, 84.0), (184.0, 84.0), (184.0, 160.0)]);
    for (i, (x, y)) in [(120.0, 120.0), (152.0, 144.0), (184.0, 168.0)].into_iter().enumerate() {
        let n = i + 1;
        let pins = [format!("{n}P1"), format!("{n}P2"), format!("{n}S1"), format!("{n}S2")];
        g.sym("ct", x, y, 0, m.ct, "", if i == 0 { m.ratio } else { "" }).pins(&pins.each_ref().map(String::as_str));
    }
    // Voltage paths through the NEOZED fuses, before the transformers.
    for (x, y, to) in [(120.0, 96.0, 300.0), (152.0, 100.0, 308.0), (184.0, 104.0, 316.0)] {
        g.path(&[(x, y), (to, y), (to, 116.0)]);
        cross_section(&mut g, TAP);
    }
    g.sym("fuse_swdisc3", 300.0, 120.0, 0, m.taps, "", "Voltage paths\nD01 3 x 10 A");
    // Cable to the meter cabinet: secondaries, voltages and N on the test terminals.
    let mut n = 0;
    let mut next = || {
        n += 1;
        n.to_string()
    };
    for (i, (sx, sy)) in [(128.0, 120.0), (160.0, 144.0), (192.0, 168.0)].into_iter().enumerate() {
        let x1 = 252.0 + 16.0 * i as f32;
        g.w(sx, sy - 4.0, x1, sy - 4.0);
        core(&mut g, (x1, sy - 4.0), (x1, 180.0), m.cable, &next());
        g.w(sx, sy + 4.0, x1 + 8.0, sy + 4.0);
        core(&mut g, (x1 + 8.0, sy + 4.0), (x1 + 8.0, 180.0), m.cable, &next());
    }
    for x in [300.0, 308.0, 316.0] {
        core(&mut g, (x, 124.0), (x, 180.0), m.cable, &next());
    }
    core(&mut g, (324.0, 48.0), (324.0, 180.0), m.cable, &next());
    if !m.cable_text.is_empty() {
        g.text(332.0, 150.0, m.cable_text, 2.5);
    }
    for i in 0..10 {
        let x = 252.0 + 8.0 * i as f32;
        g.sym("term_disc", x, 184.0, 0, m.strip, &(i + 1).to_string(), if i == 0 { "Test terminal block" } else { "" });
        g.w(x, 188.0, x, 200.0);
    }
    g.sym("kwh3_ct", 252.0, 204.0, 0, m.meter, "", m.meter_text);
    // On to the busbar after the metering.
    let ends = [(120.0, 128.0), (152.0, 152.0), (184.0, 176.0)];
    match m.switch {
        Some(q) => {
            g.w(120.0, 128.0, 120.0, 196.0);
            g.path(&[(152.0, 152.0), (152.0, 188.0), (128.0, 188.0), (128.0, 196.0)]);
            g.path(&[(184.0, 176.0), (184.0, 192.0), (136.0, 192.0), (136.0, 196.0)]);
            g.sym(m.switch_symbol, 120.0, 200.0, 0, q, "", m.switch_text).pins(&m.switch_pins);
            // Spread apart so that the names of the interruption points stay readable.
            g.w(120.0, 204.0, 120.0, 236.0);
            cross_section(&mut g, m.primary);
            g.path(&[(128.0, 204.0), (128.0, 212.0), (160.0, 212.0), (160.0, 236.0)]);
            cross_section(&mut g, m.primary);
            g.path(&[(136.0, 204.0), (136.0, 208.0), (200.0, 208.0), (200.0, 236.0)]);
            cross_section(&mut g, m.primary);
            for (i, x) in [120.0, 160.0, 200.0].into_iter().enumerate() {
                link(&mut g, x, 236.0, &format!("{}-L{}", m.output, i + 1));
            }
        }
        None => {
            for (i, (x, y)) in ends.into_iter().enumerate() {
                g.w(x, y, x, 236.0);
                cross_section(&mut g, m.primary);
                link(&mut g, x, 236.0, &format!("{}-L{}", m.output, i + 1));
            }
        }
    }
    // The conductors through the transformers.
    {
        let inside = |v: f32, lo: f32, hi: f32| v >= lo && v <= hi;
        for w in g.prj.pages[g.page].wires.iter_mut() {
            if [w.a, w.b].iter().all(|q| inside(q.x, 120.0, 184.0) && inside(q.y, 76.0, 196.0)) {
                w.cross_section = m.primary.into();
            }
        }
    }
}

/// A feeder of a busbar: NH00 fuse switch disconnector, the terminals and the cable
/// the installer connects the consumer `target` with on site.
#[allow(clippy::too_many_arguments)]
pub(super) fn feeder(g: &mut Pg, x: f32, q: &str, text: &str, strip: &str, cable: &str, target: &str, note: &str) {
    g.sym("fuse_swdisc3", x, 100.0, 0, q, "", text);
    for i in 0..3 {
        let xi = x + 8.0 * i as f32;
        g.w(xi, 24.0 + 8.0 * i as f32, xi, 96.0).w(xi, 104.0, xi, 136.0);
        g.sym("term", xi, 140.0, 0, strip, &(i + 1).to_string(), "");
    }
    g.sym("term_n", x + 24.0, 140.0, 0, strip, "N", "").w(x + 24.0, 48.0, x + 24.0, 136.0);
    g.sym("term_pe", x + 32.0, 140.0, 0, strip, "PE", "").w(x + 32.0, 56.0, x + 32.0, 136.0);
    for (i, (c, port)) in [("BN", "L1"), ("BK", "L2"), ("GY", "L3"), ("BU", "N"), ("GNYE", "PE")].into_iter().enumerate() {
        let xi = x + 8.0 * i as f32;
        core(g, (xi, 144.0), (xi, 196.0), cable, c);
        g.sym("plug_f", xi, 200.0, 0, target, port, "");
    }
    g.text(x, 216.0, note, 2.5);
}

/// The distribution busbar after the consumption metering: heat pumps and general
/// consumers (lift, staircase lighting), on to the coupling switch.
fn distribution(prj: &mut Project, section: u64) {
    let i = page(prj, section, "1", "Heat pumps and general consumers", PageKind::Schematic, "DIST", "HV");
    let mut g = Pg { prj, page: i };
    for (i, name) in ["L1", "L2", "L3", "N", "PE"].into_iter().enumerate() {
        let y = 24.0 + 8.0 * i as f32;
        link(&mut g, 20.0, y, &format!("DB-{name}"));
        g.w(20.0, y, 380.0, y);
        if i < 3 {
            cross_section(&mut g, BUSBAR);
        }
        link(&mut g, 380.0, y, &format!("CS-{name}"));
    }
    feeder(&mut g, 80.0, "-Q21", "Heat pumps\nNH00 3 x 50 A", "-X21", "=DIST-W21", "+TR-EQ1", "Heat pump; further feeders retrofitted on site");
    feeder(&mut g, 200.0, "-Q22", "General consumers\nNH00 3 x 80 A", "-X22", "=DIST-W22", "+UVA-E1", "Sub-distribution: lift, staircase lighting");
    g.text(260.0, 80.0, "On to the coupling switch of the grid and system protection", 2.5);
}

/// The coupling switch between the distribution and the PV busbar, the voltage and
/// frequency relay measuring at the grid side, the buffered 24 V supply and the supply
/// of the motor operator.
fn coupling_switch(prj: &mut Project, section: u64) {
    let i = page(prj, section, "1", "Coupling switch and NA protection relay", PageKind::Schematic, "NA", "HV");
    let mut g = Pg { prj, page: i };
    for (i, (name, end)) in [("L1", 280.0), ("L2", 148.0), ("L3", 156.0), ("N", 288.0), ("PE", 236.0)].into_iter().enumerate() {
        let y = 24.0 + 8.0 * i as f32;
        link(&mut g, 20.0, y, &format!("CS-{name}"));
        g.w(20.0, y, end, y);
        if i < 3 {
            cross_section(&mut g, BUSBAR);
        }
    }
    g.sym("cb4", 60.0, 80.0, 0, "-Q30", "", "Coupling switch\n4P 160 A").pins(&["1", "3", "5", "N", "2", "4", "6", "N'"]);
    for (i, (x, y)) in [(60.0, 24.0), (68.0, 32.0), (76.0, 40.0), (84.0, 48.0)].into_iter().enumerate() {
        let cs = if i < 3 { LINK } else { "16 mm²" };
        g.w(x, y, x, 76.0);
        cross_section(&mut g, cs);
        g.w(x, 84.0, x, 236.0);
        cross_section(&mut g, cs);
        link(&mut g, x, 236.0, &format!("PV-{}", ["L1", "L2", "L3", "N"][i]));
    }
    g.w(92.0, 56.0, 92.0, 236.0);
    link(&mut g, 92.0, 236.0, "PV-PE");
    g.text(100.0, 200.0, "To the generation metering", 2.5);
    // Voltage and frequency relay at the grid side of the coupling switch.
    g.sym("cb3", 140.0, 80.0, 0, "-F31", "", "Measuring");
    for (x, y) in [(140.0, 24.0), (148.0, 32.0), (156.0, 40.0)] {
        g.w(x, y, x, 76.0);
        cross_section(&mut g, TAP);
    }
    for x in [140.0, 148.0, 156.0] {
        g.w(x, 84.0, x, 116.0);
    }
    g.w(164.0, 48.0, 164.0, 116.0);
    g.sym("relay_grid_protection", 140.0, 120.0, 0, "-K31", "", "NA protection\nVDE-AR-N 4105");
    // Buffered 24 V supply of the protection circuit.
    g.sym("cb1", 220.0, 80.0, 0, "-F32", "", "24 V supply").w(220.0, 24.0, 220.0, 76.0);
    cross_section(&mut g, TAP);
    g.w(220.0, 84.0, 220.0, 116.0);
    g.w(228.0, 48.0, 228.0, 116.0).w(236.0, 56.0, 236.0, 116.0);
    g.sym("psu", 220.0, 120.0, 0, "-T31", "", "24 V DC buffered");
    g.w(220.0, 136.0, 220.0, 236.0).w(228.0, 136.0, 228.0, 236.0);
    link(&mut g, 220.0, 236.0, "+24V");
    link(&mut g, 228.0, 236.0, "0V");
    // Supply of the motor operator and the signal lamp.
    g.sym("cb1", 280.0, 80.0, 0, "-F33", "", "Motor operator").w(280.0, 24.0, 280.0, 76.0);
    cross_section(&mut g, TAP);
    g.w(280.0, 84.0, 280.0, 236.0);
    g.w(288.0, 48.0, 288.0, 236.0);
    link(&mut g, 280.0, 236.0, "NA-L");
    link(&mut g, 288.0, 236.0, "NA-N");
}

/// The protection circuit: the fire brigade switch at the building entrance and the
/// relay hold the undervoltage release of the coupling switch and start the response
/// delay; after it the motor operator closes the open switch again. The lamps show
/// the trip, the locked and the closed switch. The changeovers are turned over: their
/// common terminal is fed from above.
fn na_control(prj: &mut Project, section: u64) {
    let i = page(prj, section, "2", "Release, motor operator and fire brigade switch", PageKind::Schematic, "NA", "HV");
    let mut g = Pg { prj, page: i };
    g.sym("intr", 20.0, 24.0, 1, "", "+24V", "").w(20.0, 24.0, 100.0, 24.0);
    g.sym("intr", 20.0, 236.0, 1, "", "0V", "").w(20.0, 236.0, 160.0, 236.0);
    g.w(100.0, 24.0, 100.0, 36.0);
    g.sym("term", 100.0, 40.0, 0, "-X30", "1", "");
    core(&mut g, (100.0, 44.0), (100.0, 56.0), "=NA-W30", "BN");
    g.sym("sel_nc", 100.0, 60.0, 0, "+ENT-S1", "", "Fire brigade switch\nat the building entrance");
    core(&mut g, (100.0, 64.0), (100.0, 76.0), "=NA-W30", "BU");
    g.sym("term", 100.0, 80.0, 0, "-X30", "2", "");
    g.w(100.0, 84.0, 100.0, 96.0);
    // Turned over: common 11 at the top, NO 14 below it, NC 12 below left.
    g.sym("co", 100.0, 100.0, 2, "-K31", "", "Grid and system OK");
    g.w(100.0, 104.0, 100.0, 140.0).w(100.0, 140.0, 100.0, 156.0);
    g.sym("coil_on", 100.0, 160.0, 0, "-K32", "", "Response delay");
    g.w(100.0, 164.0, 100.0, 236.0);
    g.path(&[(100.0, 140.0), (160.0, 140.0), (160.0, 156.0)]);
    g.sym("relay_undervoltage", 160.0, 160.0, 0, "-Q30", "", "Undervoltage release").pins(&["D1", "D2"]);
    g.w(160.0, 164.0, 160.0, 236.0);
    g.path(&[(96.0, 104.0), (96.0, 112.0), (44.0, 112.0), (44.0, 156.0)]);
    g.sym("lamp", 44.0, 160.0, 0, "-P32", "", "NA protection tripped");
    g.w(44.0, 164.0, 44.0, 236.0);
    // 230 V: after the delay the motor operator closes the open coupling switch.
    link(&mut g, 260.0, 24.0, "NA-L");
    g.w(260.0, 24.0, 280.0, 24.0).w(280.0, 24.0, 280.0, 56.0);
    g.sym("co", 280.0, 60.0, 2, "-K32", "", "").pins(&["15", "16", "18"]);
    g.path(&[(276.0, 64.0), (276.0, 72.0), (232.0, 72.0), (232.0, 156.0)]);
    g.sym("lamp", 232.0, 160.0, 0, "-P30", "", "Coupling switch locked");
    g.w(232.0, 164.0, 232.0, 236.0);
    g.w(280.0, 64.0, 280.0, 96.0);
    g.sym("co", 280.0, 100.0, 2, "-Q30", "", "Coupling switch");
    g.path(&[(276.0, 104.0), (276.0, 120.0), (340.0, 120.0), (340.0, 156.0)]);
    g.sym("coil_motor_operator", 340.0, 160.0, 0, "-Q30", "", "Motor operator");
    g.w(340.0, 164.0, 340.0, 236.0);
    g.w(280.0, 104.0, 280.0, 156.0);
    g.sym("lamp", 280.0, 160.0, 0, "-P31", "", "Coupling switch closed");
    g.w(280.0, 164.0, 280.0, 236.0);
    link(&mut g, 220.0, 236.0, "NA-N");
    g.w(220.0, 236.0, 340.0, 236.0);
}

/// The busbar of the inverters after the generation metering: two feeders the PV
/// installer connects the inverters to.
fn inverter_feeders(prj: &mut Project, section: u64) {
    let i = page(prj, section, "2", "Inverter feeders", PageKind::Schematic, "PV", "HV");
    let mut g = Pg { prj, page: i };
    for (i, (name, end)) in [("L1", 296.0), ("L2", 304.0), ("L3", 312.0), ("N", 320.0), ("PE", 328.0)].into_iter().enumerate() {
        let y = 24.0 + 8.0 * i as f32;
        link(&mut g, 20.0, y, &format!("PVB-{name}"));
        g.w(20.0, y, end, y);
        if i < 3 {
            cross_section(&mut g, BUSBAR);
        }
    }
    feeder(&mut g, 80.0, "-Q41", "Inverter 1\nNH00 3 x 80 A", "-X41", "=PV-W41", "+INV-TB1", "Inverter 1, connected by the PV installer");
    feeder(&mut g, 296.0, "-Q42", "Inverter 2\nNH00 3 x 80 A", "-X42", "=PV-W42", "+INV-TB2", "Inverter 2");
    g.text(140.0, 80.0, "Room for a smart meter of a later battery storage", 2.5);
}

/// Cables: tag, part, length in m, what it connects.
pub(super) fn cables(prj: &mut Project, db: &PartsDb, list: &[(&str, &str, f32, &str)]) {
    for (tag, number, length, desc) in list {
        let part = db.get(number).unwrap_or_else(|| panic!("cable part {number}")).clone();
        crate::parts::ensure_in_project(prj, db, number);
        let id = prj.new_id();
        prj.cables.push(Cable {
            id,
            tag: tag.to_string(),
            cable_type: part.type_number.clone(),
            cores: part.tech_value("Cores").unwrap_or_default().into(),
            cross_section: part.tech_value("Cross-section").unwrap_or_default().into(),
            length: *length,
            part: part.number.clone(),
            description: desc.to_string(),
            ..Default::default()
        });
    }
}

pub(super) fn profile(prj: &mut Project, enc: usize, kind: ItemKind, part: &str, a: (f32, f32), b: (f32, f32)) -> u64 {
    let item = panel::new_profile(prj, kind, part, p(a.0, a.1), p(b.0, b.1));
    let id = item.id;
    prj.layout.enclosures[enc].items.push(item);
    id
}

/// Devices side by side from `x` on the rail at height `rail_y`.
pub(super) fn on_rail(prj: &mut Project, enc: usize, rail_y: f32, x: f32, tags: &[&str]) {
    let mut x = x;
    for tag in tags {
        let (w, h) = panel::device_size(prj, tag).map(|d| (d.w, d.h)).unwrap_or((40.0, 40.0));
        panel::place_device(prj, enc, tag, p(x, rail_y - h / 2.0));
        x += panel::snap(w + 10.0);
    }
}

/// A cable gland in the roof or the bottom (`face`) in line with a device, its middle
/// in the routing cell of the device's connection points that face it (of its front
/// where it has none): the cables go straight up or down.
pub(super) fn gland_at(prj: &mut Project, enc: usize, part: &str, tag: &str, face: Face) {
    let e = &prj.layout.enclosures[enc];
    let Some(it) = e.items.iter().find(|i| i.tag == tag) else { return };
    let towards = if face == Face::Roof { crate::parts::Exit::Up } else { crate::parts::Exit::Down };
    let points: Vec<crate::parts::vec3::V3> = panel::device_points(prj, it).into_iter().filter(|q| q.exit == towards).map(|q| q.pos).collect();
    let at = match points.len() {
        0 => [it.x + it.w / 2.0, it.y + it.h / 2.0, it.z + it.d],
        n => [0, 1, 2].map(|k| points.iter().map(|q| q[k]).sum::<f32>() / n as f32),
    };
    let c = panel::face_point(e, it.face, at);
    let w = prj.parts.iter().find(|q| q.number == part).map_or(30.0, |q| q.width);
    let cell = prj.layout.routing.grid.max(5.0);
    let mid = |v: f32| ((v / cell).floor() + 0.5) * cell;
    // The bottom is measured from the front.
    let across = if face == Face::Bottom { e.depth - mid(c[2]) } else { mid(c[2]) };
    panel::place_accessory(prj, enc, part, face, p(mid(c[0]) - w / 2.0, across - w / 2.0));
}

/// A cable gland in the bottom with its middle `x` mm from the left and `z` mm in
/// front of the rear wall.
pub(super) fn bottom_gland(prj: &mut Project, enc: usize, part: &str, x: f32, z: f32) {
    let w = prj.parts.iter().find(|q| q.number == part).map_or(30.0, |q| q.width);
    let depth = prj.layout.enclosures[enc].depth;
    panel::place_accessory(prj, enc, part, Face::Bottom, p(x - w / 2.0, depth - z - w / 2.0));
}

pub(super) fn push(prj: &mut Project, e: Enclosure) -> usize {
    prj.layout.enclosures.push(e);
    prj.layout.enclosures.len() - 1
}

/// The univers cabinet +HV in five fields of 250 mm: connection point zone and surge
/// arrester, consumption metering, NA protection and distribution, coupling switch and
/// generation metering, inverter feeders; busbars at the bottom. The meter cabinets
/// +ZS1 and +ZS2 with their test terminal blocks.
fn enclosures(prj: &mut Project, db: &PartsDb) {
    for part in ["HAG.FR25S2", "HAG.FZ831", "HAG.UD21APZ", "HAG.FP62ZG2", panel::RAIL_PART, "GEN.BB-3X30X10", "GEN.BB-3X30X10-60", "GEN.PE-BAR-10", "GEN.GLAND-M63", "GEN.GLAND-M40", "GEN.GLAND-M32", "GEN.GLAND-M20", "PXC.3022218", "PXC.3209552"] {
        crate::parts::ensure_in_project(prj, db, part);
    }
    let e = panel::new_enclosure(prj, "HAG.FR25S2", "+HV");
    let hv = push(prj, e);
    let rail = |prj: &mut Project, x0: f32, x1: f32, y: f32| profile(prj, hv, ItemKind::Rail, panel::RAIL_PART, (x0, y), (x1, y));
    // Field 1: connection point zone, surge arrester, PEN terminal, main busbar.
    panel::place_accessory(prj, hv, "HAG.UD21APZ", Face::Plate, p(0.0, 0.0));
    rail(prj, 10.0, 240.0, 420.0);
    on_rail(prj, hv, 420.0, 15.0, &["=GRID+HV-F1", "=GRID+HV-F2", "=GRID+HV-X2"]);
    rail(prj, 10.0, 240.0, 1330.0);
    on_rail(prj, hv, 1330.0, 15.0, &["=GRID+HV-X1"]);
    // The main busbar, a feed-in terminal on each of its bars; high enough for the fuse
    // switch disconnector on it to be operated.
    let main = profile(prj, hv, ItemKind::Busbar, "GEN.BB-3X30X10-60", (20.0, 1500.0), (480.0, 1500.0));
    panel::busbars::place_on_bars(prj, hv, "=GRID+HV-X0", main, 30.0);
    panel::place_accessory(prj, hv, "GEN.PE-BAR-10", Face::Plate, p(20.0, 1765.0));
    // Field 2: consumption metering, power flowing up from the busbar: the fuse switch
    // disconnector on the bars, the transformers above it, the switch at the top.
    panel::place_device(prj, hv, "=MS1+HV-Q10", p(300.0, 150.0));
    rail(prj, 260.0, 490.0, 445.0);
    on_rail(prj, hv, 445.0, 330.0, &["=MS1+HV-F11"]);
    panel::place_device(prj, hv, "=MS1+HV-T1", p(250.0, 675.0));
    panel::place_device(prj, hv, "=MS1+HV-F10", p(250.0, 1175.0));
    // Field 3: NA protection, distribution busbar with its feeders.
    rail(prj, 510.0, 740.0, 420.0);
    on_rail(prj, hv, 420.0, 515.0, &["=NA+HV-K31", "=NA+HV-T31", "=NA+HV-K32"]);
    // The rows far enough apart for the wires between them to cross each other.
    rail(prj, 510.0, 740.0, 620.0);
    on_rail(prj, hv, 620.0, 515.0, &["=NA+HV-F31", "=NA+HV-F32", "=NA+HV-F33", "=NA+HV-P30", "=NA+HV-P31"]);
    panel::place_device_on(prj, hv, "=NA+HV-P32", p(600.0, 900.0), Face::Door);
    rail(prj, 510.0, 740.0, 820.0);
    on_rail(prj, hv, 820.0, 690.0, &["=NA+HV-X30"]);
    // The terminals far enough below the feeders for the bends of their conductors.
    rail(prj, 510.0, 740.0, 1700.0);
    on_rail(prj, hv, 1700.0, 515.0, &["=DIST+HV-X21", "=DIST+HV-X22"]);
    profile(prj, hv, ItemKind::Busbar, "GEN.BB-3X30X10", (510.0, 1420.0), (740.0, 1420.0));
    for (tag, x) in [("=DIST+HV-Q21", 515.0), ("=DIST+HV-Q22", 630.0)] {
        panel::place_device(prj, hv, tag, p(x, 1315.0));
    }
    // Field 4: generation metering, power flowing up from the busbar of the inverters
    // to the coupling switch at the top.
    panel::place_device(prj, hv, "=NA+HV-Q30", p(805.0, 150.0));
    rail(prj, 760.0, 990.0, 445.0);
    on_rail(prj, hv, 445.0, 830.0, &["=MS2+HV-F41"]);
    panel::place_device(prj, hv, "=MS2+HV-T2", p(750.0, 675.0));
    // Field 5: busbar of the inverters with their feeders; it reaches into field 4 for
    // the fuse switch disconnector of the generation metering.
    rail(prj, 1010.0, 1240.0, 1700.0);
    on_rail(prj, hv, 1700.0, 1015.0, &["=PV+HV-X41", "=PV+HV-X42"]);
    profile(prj, hv, ItemKind::Busbar, "GEN.BB-3X30X10", (760.0, 1420.0), (1240.0, 1420.0));
    panel::place_device(prj, hv, "=MS2+HV-F40", p(750.0, 1175.0));
    for (tag, x) in [("=PV+HV-Q41", 1015.0), ("=PV+HV-Q42", 1130.0)] {
        panel::place_device(prj, hv, tag, p(x, 1315.0));
    }
    // Lifting eyes in the roof for the crane, the cables in from below.
    for (x, z) in [(30.0, 117.0), (1230.0, 117.0)] {
        panel::place_accessory(prj, hv, "HAG.FZ831", Face::Roof, p(x, z));
    }
    // The supply cable in line with its busbar terminals, the meter cables beside the
    // fuse switch disconnectors.
    gland_at(prj, hv, "GEN.GLAND-M63", "=GRID+HV-X0", Face::Bottom);
    bottom_gland(prj, hv, "GEN.GLAND-M32", 237.5, 137.5);
    bottom_gland(prj, hv, "GEN.GLAND-M32", 737.5, 212.5);
    // The cable of the fire brigade switch leaves through the roof, those of the
    // consumers through the bottom below their terminals.
    gland_at(prj, hv, "GEN.GLAND-M20", "=NA+HV-X30", Face::Roof);
    for (tag, part) in [("=DIST+HV-X21", "GEN.GLAND-M32"), ("=DIST+HV-X22", "GEN.GLAND-M40"), ("=PV+HV-X41", "GEN.GLAND-M40"), ("=PV+HV-X42", "GEN.GLAND-M40")] {
        gland_at(prj, hv, part, tag, Face::Bottom);
    }
    // The meter cabinets beside it, each with its test terminal block.
    for (name, strip) in [("+ZS1", "=MS1+ZS1-X1"), ("+ZS2", "=MS2+ZS2-X1")] {
        let e = panel::new_enclosure(prj, "HAG.FP62ZG2", name);
        let zs = push(prj, e);
        profile(prj, zs, ItemKind::Rail, panel::RAIL_PART, (20.0, 700.0), (480.0, 700.0));
        on_rail(prj, zs, 700.0, 200.0, &[strip]);
        panel::place_accessory(prj, zs, "GEN.GLAND-M32", Face::Bottom, p(230.0, 80.0));
    }
}
