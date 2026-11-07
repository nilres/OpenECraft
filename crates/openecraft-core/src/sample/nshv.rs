//! Level 14: the main low-voltage distribution (NSHV) of an industrial plant in two
//! bayed Hager univers cabinets. Two aluminium supply cables from two house connection
//! boxes land on the supply busbar with busbar terminals for Al conductors (a third is
//! prepared); the transformers of the distribution system operator's metering sit in
//! that busbar. Two NH fuse switch disconnector strips fitted with solid links isolate
//! it from the distribution busbar: one on each, the conductors between them the only
//! ones of that size that are no bars. A power analyser measures the connection point
//! for the zero export of the battery storage, its transformers in the distribution
//! busbar (185 mm system), which runs through both cabinets. Every feeder stands on an
//! NH strip: the measured ones with a current transformer clipped in behind each pole,
//! read by an energy meter on the Tree bus (DC charging station, two
//! sub-distributions, air conditioning), the heat pumps behind residual current
//! devices of type B. The PV inverters and the battery storage are behind two coupling
//! switches of the grid and system protection with motor operators, undervoltage
//! releases and auxiliary switches, fed from busbar terminals; the one of the
//! inverters sits on their busbar. Cable entry flanges in the roof and the bottom take
//! the large cables.

use super::metering::{cables, core, cross_section, feeder, gland_at, link, metering, on_rail, page, profile, push, rails, Metering, TAP};
use super::*;
use crate::panel::{self, Face, ItemKind};
use crate::structure::Aspect;

pub(super) fn project() -> Project {
    let mut prj = Project { pages: Vec::new(), ..Default::default() };
    prj.info = ProjectInfo {
        name: "Main LV distribution industrial plant".into(),
        description: "NSHV with transformer metering, measured feeders,\nNA protection, PV and battery storage".into(),
        company: "OpenECraft".into(),
        creator: "OpenECraft".into(),
        date: "2026-11-02".into(),
        revision: "A".into(),
        ..Default::default()
    };
    prj.apply_preset(crate::potentials::Preset::Iec60204);
    prj.electrical.conditions.ambient = 25.0;
    prj.potentials.push(crate::potentials::PotentialDef {
        name: "PEN".into(),
        kind: crate::potentials::PotentialKind::Pe,
        color: "GNYE".into(),
        cross_section: "240 mm²".into(),
        ..Default::default()
    });
    for (aspect, id, desc) in [
        (Aspect::Function, "GRID", "Supply and surge protection"),
        (Aspect::Function, "MS", "Transformer metering of the DSO"),
        (Aspect::Function, "EMS", "Energy monitoring and zero export"),
        (Aspect::Function, "DIST", "Distribution"),
        (Aspect::Function, "NA", "Grid and system protection"),
        (Aspect::Function, "PV", "Photovoltaics and battery storage"),
        (Aspect::Location, "HAK1", "House connection box 1"),
        (Aspect::Location, "HAK2", "House connection box 2"),
        (Aspect::Location, "HV1", "Main distribution, cabinet 1"),
        (Aspect::Location, "HV2", "Main distribution, cabinet 2"),
        (Aspect::Location, "CHG", "Public charging point"),
        (Aspect::Location, "UV1", "Sub-distribution workshop"),
        (Aspect::Location, "UV2", "Sub-distribution offices"),
        (Aspect::Location, "ROOF", "Roof"),
        (Aspect::Location, "TR", "Technical room"),
        (Aspect::Location, "INV", "Inverter room"),
    ] {
        prj.structures.define(aspect, id, desc);
    }
    let general = crate::sections::add(&mut prj, 0, "General");
    let supply = crate::sections::add(&mut prj, 0, "Supply and metering");
    let dist = crate::sections::add(&mut prj, 0, "Distribution");
    let na = crate::sections::add(&mut prj, 0, "Grid and system protection");
    let pv = crate::sections::add(&mut prj, 0, "Photovoltaics");
    let layout = crate::sections::add(&mut prj, 0, "Panel layout");
    let reports = crate::sections::add(&mut prj, 0, "Reports");

    page(&mut prj, general, "1", "Cover sheet", PageKind::Cover, "", "");
    page(&mut prj, general, "2", "Table of contents", PageKind::Report(ReportKind::TableOfContents.into()), "", "");
    single_line(&mut prj, general);
    supplies(&mut prj, supply);
    metering(&mut prj, supply, &METERING);
    zero_export(&mut prj, supply);
    heat_pumps(&mut prj, dist);
    for (n, f) in FEEDERS.iter().enumerate() {
        measured_feeder(&mut prj, dist, &(n + 2).to_string(), f);
    }
    coupling_switches(&mut prj, na);
    na_control(&mut prj, na);
    inverter_feeders(&mut prj, pv);
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
    ("=GRID+HAK1-F1", &["GEN.NH2-BASE-3P-400A"]),
    ("=GRID+HAK2-F1", &["GEN.NH2-BASE-3P-400A"]),
    ("=GRID+HV1-X01", &["HAG.KS30C"]),
    ("=GRID+HV1-X02", &["HAG.KS30C"]),
    ("=GRID+HV1-X1", &["HAG.KS30C"]),
    ("=GRID+HV1-F1", &["HAG.SPA811Z"]),
    ("=MS+HV1-F10", &["HAG.LVSG3CPX"]),
    ("=MS+HV1-T1", &["GEN.CT-SET-400-5A"]),
    ("=MS+HV1-F11", &["HAG.L73M"]),
    ("=MS+HV1-X1", &["GEN.TB-DISC-4"]),
    ("=MS+HV1-P1", &["GEN.METER-3PT-CT"]),
    ("=MS+HV1-Q10", &["HAG.LVSG3CPX"]),
    ("=EMS+HV2-T5", &["GEN.CT-SET-400-5A"]),
    ("=EMS+HV2-F5", &["HAG.MCN306"]),
    ("=EMS+HV2-P5", &["JAN.52.16.202"]),
    ("=DIST+HV2-Q25", &["HAG.LVSG00RPX"]),
    ("=DIST+HV2-Q26", &["HAG.LVSG00RPX"]),
    ("=DIST+HV2-F25", &["HAG.CDB640E"]),
    ("=DIST+HV2-F26", &["HAG.CDB640E"]),
    ("=DIST+HV2-X25", &["GEN.TB-35", "GEN.TB-N-35", "GEN.TB-PE-35"]),
    ("=DIST+HV2-X26", &["GEN.TB-35", "GEN.TB-N-35", "GEN.TB-PE-35"]),
    ("=DIST+TR-EQ25", &["GEN.HP-AW-12KW"]),
    ("=DIST+TR-EQ26", &["GEN.HP-AW-12KW"]),
    ("=DIST+HV2-Q21", &["HAG.LVSG1CPX", "HAG.LVZW150", "HAG.LVZVA-V"]),
    ("=DIST+HV2-Q22", &["HAG.LVSG1CPX", "HAG.LVZW150", "HAG.LVZVA-V"]),
    ("=DIST+HV2-Q23", &["HAG.LVSG1CPX", "HAG.LVZW150"]),
    ("=DIST+HV2-Q24", &["HAG.LVSG1CPX", "HAG.LVZW150"]),
    ("=DIST+HV2-F21", &["HAG.MCN306"]),
    ("=DIST+HV2-F22", &["HAG.MCN306"]),
    ("=DIST+HV2-F23", &["HAG.MCN306"]),
    ("=DIST+HV2-F24", &["HAG.MCN306"]),
    ("=DIST+HV2-X21", &["HAG.KS30C"]),
    ("=DIST+HV2-X22", &["GEN.TB-N-35", "GEN.TB-PE-35"]),
    ("=DIST+HV2-X23", &["GEN.TB-N-35", "GEN.TB-PE-35"]),
    ("=DIST+HV2-X24", &["GEN.TB-N-35", "GEN.TB-PE-35"]),
    ("=DIST+HV2-P21", &["LOX.100652"]),
    ("=DIST+HV2-P22", &["LOX.100652"]),
    ("=DIST+HV2-P23", &["LOX.100652"]),
    ("=DIST+HV2-P24", &["LOX.100652"]),
    ("=DIST+CHG-E21", &["GEN.EVSE-DC-100KW"]),
    ("=DIST+UV1-E22", &["GEN.SUBDIST-48M"]),
    ("=DIST+UV2-E23", &["GEN.SUBDIST-48M"]),
    ("=DIST+ROOF-EQ24", &["GEN.AC-OUT-25KW"]),
    ("=NA+HV1-X30", &["HAG.KS30C"]),
    ("=NA+HV1-X31", &["HAG.KS30C"]),
    ("=NA+HV1-Q30", &["HAG.HNT161GR", "HAG.HXT041H", "HAG.HXA011H", "HAG.HXA021H"]),
    ("=NA+HV1-Q31", &["HAG.HNT161GR", "HAG.HXT041H", "HAG.HXA011H", "HAG.HXA021H"]),
    ("=NA+HV1-F30", &["HAG.MCN306"]),
    ("=NA+HV1-K30", &["HAG.EU400"]),
    ("=NA+HV1-F33", &["HAG.MCN106"]),
    ("=NA+HV1-P30", &["GEN.LMP-DIN-RD-230"]),
    ("=NA+HV1-P31", &["GEN.LMP-DIN-GN-230"]),
    ("=PV+HV1-Q41", &["HAG.LVSG00RPX"]),
    ("=PV+HV1-Q42", &["HAG.LVSG00RPX"]),
    ("=PV+HV1-X41", &["GEN.TB-35", "GEN.TB-N-35", "GEN.TB-PE-35"]),
    ("=PV+HV1-X42", &["GEN.TB-35", "GEN.TB-N-35", "GEN.TB-PE-35"]),
    ("=PV+INV-TB41", &["GEN.INV-PV-40KW"]),
    ("=PV+INV-TB42", &["GEN.INV-PV-40KW"]),
    ("=PV+INV-TB43", &["GEN.INV-BAT-50KW"]),
];

/// Parts used more than once by a device: fuse and solid links, transformers, terminals.
const COUNTED: &[(&str, &str, u32)] = &[
    ("=GRID+HAK1-F1", "GEN.NH2-GG-315A", 3),
    ("=GRID+HAK2-F1", "GEN.NH2-GG-315A", 3),
    ("=MS+HV1-F10", "GEN.NH3-SOLID-LINK", 3),
    ("=MS+HV1-Q10", "GEN.NH3-SOLID-LINK", 3),
    ("=MS+HV1-F11", "GEN.D01-GG-10A", 3),
    ("=DIST+HV2-Q25", "GEN.NH00-GG-50A", 3),
    ("=DIST+HV2-Q26", "GEN.NH00-GG-50A", 3),
    ("=DIST+HV2-Q21", "HAG.LVZW150", 2),
    ("=DIST+HV2-Q22", "HAG.LVZW150", 2),
    ("=DIST+HV2-Q23", "HAG.LVZW150", 2),
    ("=DIST+HV2-Q24", "HAG.LVZW150", 2),
    ("=DIST+HV2-Q21", "GEN.NH1-GG-160A", 3),
    ("=DIST+HV2-Q22", "GEN.NH1-GG-100A", 3),
    ("=DIST+HV2-Q23", "GEN.NH1-GG-63A", 3),
    ("=DIST+HV2-Q24", "GEN.NH1-GG-63A", 3),
    ("=PV+HV1-Q41", "GEN.NH00-GG-80A", 3),
    ("=PV+HV1-Q42", "GEN.NH00-GG-80A", 3),
];

const CABLES: &[(&str, &str, f32, &str)] = &[
    ("=GRID-W1", "GEN.CBL-NAYY-4X240", 25.0, "Supply from house connection box 1"),
    ("=GRID-W2", "GEN.CBL-NAYY-4X240", 25.0, "Supply from house connection box 2"),
    ("=DIST-W21", "GEN.CBL-NYY-5X95", 40.0, "DC charging station"),
    ("=DIST-W22", "GEN.CBL-NYY-5X35", 30.0, "Sub-distribution workshop"),
    ("=DIST-W23", "GEN.CBL-NYY-5X16", 30.0, "Sub-distribution offices"),
    ("=DIST-W24", "GEN.CBL-NYY-5X16", 35.0, "Air conditioning"),
    ("=DIST-W25", "GEN.CBL-NYY-5X16", 20.0, "Heat pump 1"),
    ("=DIST-W26", "GEN.CBL-NYY-5X16", 20.0, "Heat pump 2"),
    ("=PV-W41", "GEN.CBL-NYY-5X35", 30.0, "Inverter 1"),
    ("=PV-W42", "GEN.CBL-NYY-5X35", 30.0, "Inverter 2"),
    ("=PV-W43", "GEN.CBL-NYY-5X95", 25.0, "Battery inverter"),
];

/// Cross-section of the bars of the busbars (Cu 30 x 10 mm).
const BUSBAR: &str = "300 mm²";
/// Cross-section of the conductors between the two isolation strips.
const LINK: &str = "150 mm²";
/// Cross-section of the conductors of the coupling switches (160 A).
const COUPLING: &str = "70 mm²";

/// The transformer metering: the transformers in the supply busbar, an NH3 strip with
/// solid links on it after them (the conductors on to the strip on the distribution
/// busbar leave it at the bottom), the meter of the DSO on its meter changing plate in
/// the cabinet.
const METERING: Metering = Metering {
    function: "MS",
    location: "HV1",
    n: "2",
    desc: "Transformer metering",
    input: "HV",
    output: "ML",
    fuse: "",
    fuse_text: "",
    fuse_pins: ["1", "3", "5", "2", "4", "6"],
    ct: "-T1",
    ratio: "400/5 A",
    primary: LINK,
    taps: "-F11",
    meter: "-P1",
    meter_text: "Meter of the DSO on the meter changing plate",
    strip: "-X1",
    cable: "",
    cable_text: "",
    switch: Some("-F10"),
    switch_symbol: "fuse_swdisc3",
    switch_text: "Isolation of the supply busbar\nNH3 strip, solid links",
    switch_pins: ["1", "3", "5", "2", "4", "6"],
};

/// A feeder measured by the current transformers behind its NH strip.
struct Feeder {
    desc: &'static str,
    strip: &'static str,
    text: &'static str,
    taps: &'static str,
    meter: &'static str,
    cable: &'static str,
    target: &'static str,
}

const FEEDERS: [Feeder; 4] = [
    Feeder { desc: "DC charging station", strip: "-Q21", text: "DC charging station\nNH1 3 x 160 A", taps: "-F21", meter: "-P21", cable: "=DIST-W21", target: "+CHG-E21" },
    Feeder { desc: "Sub-distribution workshop", strip: "-Q22", text: "Sub-distribution workshop\nNH1 3 x 100 A", taps: "-F22", meter: "-P22", cable: "=DIST-W22", target: "+UV1-E22" },
    Feeder { desc: "Sub-distribution offices", strip: "-Q23", text: "Sub-distribution offices\nNH1 3 x 63 A", taps: "-F23", meter: "-P23", cable: "=DIST-W23", target: "+UV2-E23" },
    Feeder { desc: "Air conditioning", strip: "-Q24", text: "Air conditioning\nNH1 3 x 63 A", taps: "-F24", meter: "-P24", cable: "=DIST-W24", target: "+ROOF-EQ24" },
];

/// The interruption points the distribution busbar runs through, page by page: from
/// the zero export page over the heat pumps and the measured feeders to the coupling
/// switches.
const BUS: [&str; 6] = ["EX", "D1", "D2", "D3", "D4", "D5"];

/// Conductors L1 24 … PE 56 across the page between the interruption points `from` on
/// the left and `to` on the right.
fn bus(g: &mut Pg, from: &str, to: &str) {
    for (i, name) in ["L1", "L2", "L3", "N", "PE"].into_iter().enumerate() {
        let y = 24.0 + 8.0 * i as f32;
        link(g, 20.0, y, &format!("{from}-{name}"));
        g.w(20.0, y, 380.0, y);
        if i < 3 {
            cross_section(g, BUSBAR);
        }
        link(g, 380.0, y, &format!("{to}-{name}"));
    }
}

fn single_line(prj: &mut Project, section: u64) {
    let i = page(prj, section, "3", "Single-line diagram", PageKind::Overview, "", "");
    let mut g = Pg { prj, page: i };
    g.text(40.0, 20.0, "3/PEN AC 400 V 50 Hz; 2 x NAYY-J 4 x 240 from house connection boxes 1 and 2 (a third prepared)", 2.5);
    for x in [40.0, 80.0] {
        g.w(x, 26.0, x, 36.0).w(x, 36.0, 60.0, 36.0);
    }
    g.w(60.0, 36.0, 60.0, 44.0);
    g.sym("sl_meter", 60.0, 52.0, 0, "=MS+HV1-P1", "", "Transformer metering 400/5 A");
    g.w(60.0, 60.0, 60.0, 68.0);
    g.sym("sl_fuse_switch", 60.0, 76.0, 0, "=MS+HV1-F10", "", "Isolation, solid links");
    g.w(60.0, 84.0, 60.0, 92.0);
    g.sym("sl_fuse_switch", 60.0, 100.0, 0, "=MS+HV1-Q10", "", "Isolation, solid links");
    g.w(60.0, 108.0, 60.0, 116.0);
    g.sym("sl_meter", 60.0, 124.0, 0, "=EMS+HV2-P5", "", "Zero export UMG 604-PRO");
    g.w(60.0, 132.0, 60.0, 140.0).w(40.0, 140.0, 380.0, 140.0);
    g.text(40.0, 146.0, "Distribution busbar, 185 mm", 2.5);
    for (x, tag, text) in [
        (100.0, "=DIST+HV2-Q21", "DC charging"),
        (140.0, "=DIST+HV2-Q22", "UV workshop"),
        (180.0, "=DIST+HV2-Q23", "UV offices"),
        (220.0, "=DIST+HV2-Q24", "Air conditioning"),
        (260.0, "=DIST+HV2-Q25", "Heat pump 1"),
        (300.0, "=DIST+HV2-Q26", "Heat pump 2"),
    ] {
        g.w(x, 140.0, x, 152.0);
        g.sym("sl_fuse_switch", x, 160.0, 0, tag, "", text);
    }
    for (x, tag, text) in [(340.0, "=NA+HV1-Q30", "Coupling PV"), (380.0, "=NA+HV1-Q31", "Coupling battery")] {
        g.w(x, 140.0, x, 184.0);
        g.sym("sl_cb", x, 192.0, 0, tag, "", text);
    }
    g.w(340.0, 200.0, 340.0, 208.0).w(320.0, 208.0, 360.0, 208.0);
    for (x, tag) in [(320.0, "=PV+HV1-Q41"), (360.0, "=PV+HV1-Q42")] {
        g.w(x, 208.0, x, 216.0);
        g.sym("sl_fuse_switch", x, 224.0, 0, tag, "", "Inverter");
    }
}

/// Two aluminium supply cables on the busbars (busbar terminals for Al conductors),
/// the PEN split into N and PE, the surge arrester.
fn supplies(prj: &mut Project, section: u64) {
    let i = page(prj, section, "1", "Supplies and surge protection", PageKind::Schematic, "GRID", "HV1");
    let mut g = Pg { prj, page: i };
    for (k, (x0, hak, cable)) in [(60.0, "+HAK1-F1", "=GRID-W1"), (140.0, "+HAK2-F1", "=GRID-W2")].into_iter().enumerate() {
        for (j, (key, label)) in [("pot_l1", "L1"), ("pot_l2", "L2"), ("pot_l3", "L3"), ("pot", "PEN")].into_iter().enumerate() {
            g.sym(key, x0 + 8.0 * j as f32, 20.0, 0, "", label, "");
        }
        g.sym("fuse3", x0, 32.0, 0, hak, "", &format!("House connection box {}\nNH2 3 x 315 A gG", k + 1));
        for (j, c) in ["BN", "BK", "GY"].into_iter().enumerate() {
            let x = x0 + 8.0 * j as f32;
            g.w(x, 24.0, x, 28.0);
            core(&mut g, (x, 36.0), (x, 68.0), cable, c);
        }
        core(&mut g, (x0 + 24.0, 24.0), (x0 + 24.0, 68.0), cable, "GNYE");
        // The phases on the busbar terminals of the phase bars (a terminal on each bar
        // per cable, turned: the cable comes to it from below), the PEN on the PEN bar.
        for j in 0..3 {
            g.sym("term", x0 + 8.0 * j as f32, 72.0, 2, &format!("-X0{}", k + 1), &(j + 1).to_string(), "");
        }
        g.sym("term", x0 + 24.0, 72.0, 0, "-X1", &(k + 1).to_string(), "");
    }
    g.text(220.0, 52.0, "-W1, -W2 NAYY-J 4 x 240 SE, Al, on busbar terminals for aluminium", 2.5);
    g.text(220.0, 60.0, "A third supply from house connection box 3 is prepared", 2.5);
    for (x, y) in [(60.0, 120.0), (68.0, 128.0), (76.0, 136.0)] {
        g.path(&[(x, 76.0), (x, y), (380.0, y)]);
        cross_section(&mut g, BUSBAR);
    }
    g.path(&[(84.0, 76.0), (84.0, 144.0), (380.0, 144.0)]).path(&[(84.0, 144.0), (84.0, 152.0), (380.0, 152.0)]);
    for (x, y) in [(140.0, 120.0), (148.0, 128.0), (156.0, 136.0), (164.0, 144.0)] {
        g.w(x, 76.0, x, y);
    }
    for (y, name) in [(120.0, "L1"), (128.0, "L2"), (136.0, "L3"), (144.0, "N"), (152.0, "PE")] {
        link(&mut g, 380.0, y, &format!("HV-{name}"));
    }
    g.text(176.0, 112.0, "Supply busbar Cu 30 x 10 mm; PEN split into N and PE", 2.5);
    for (i, (x, y, l)) in [(200.0, 120.0, "L1"), (208.0, 128.0, "L2"), (216.0, 136.0, "L3"), (224.0, 144.0, "N")].into_iter().enumerate() {
        let text = if i == 0 { "Surge protection T1 + T2 + T3" } else { "" };
        g.sym("spd", x, 180.0, 0, "-F1", "", text).pins(&[l, "PE"]).w(x, y, x, 176.0);
        cross_section(&mut g, "16 mm²");
        g.w(x, 184.0, x, 192.0);
        cross_section(&mut g, "16 mm²");
    }
    g.w(200.0, 192.0, 232.0, 192.0);
    cross_section(&mut g, "16 mm²");
    g.w(232.0, 152.0, 232.0, 192.0);
    cross_section(&mut g, "16 mm²");
}

/// The strip that isolates the distribution busbar and the power analyser at the
/// connection point: its transformers in the busbar after the strip tell the battery
/// storage how much flows into the grid.
fn zero_export(prj: &mut Project, section: u64) {
    let i = page(prj, section, "3", "Zero export of the battery storage", PageKind::Schematic, "EMS", "HV2");
    let mut g = Pg { prj, page: i };
    rails(&mut g, "ML", [120.0, 128.0, 136.0, 0.0, 0.0], Some("EX"));
    // The conductors from the strip on the supply busbar come to its cable lugs at the
    // bottom: the outer one of the one to the outer one of the other, so that they do
    // not cross in the loop between the two.
    g.sym("fuse_swdisc3", 120.0, 72.0, 0, "=MS+HV1-Q10", "", "Isolation of the distribution busbar\nNH3 strip, solid links").pins(&["6", "4", "2", "5", "3", "1"]);
    for (x, y) in [(120.0, 24.0), (128.0, 32.0), (136.0, 40.0)] {
        g.w(x, y, x, 68.0);
        cross_section(&mut g, LINK);
    }
    g.w(120.0, 76.0, 120.0, 112.0);
    g.path(&[(128.0, 76.0), (128.0, 80.0), (152.0, 80.0), (152.0, 136.0)]);
    g.path(&[(136.0, 76.0), (136.0, 84.0), (184.0, 84.0), (184.0, 160.0)]);
    transformers(&mut g, "-T5", "400/5 A");
    for (x, y, to) in [(120.0, 96.0, 300.0), (152.0, 100.0, 308.0), (184.0, 104.0, 316.0)] {
        g.path(&[(x, y), (to, y), (to, 116.0)]);
        cross_section(&mut g, TAP);
    }
    g.sym("cb3", 300.0, 120.0, 0, "-F5", "", "Measuring voltages");
    for x in [300.0, 308.0, 316.0] {
        g.w(x, 124.0, x, 196.0);
    }
    g.w(324.0, 48.0, 324.0, 196.0);
    secondaries(&mut g, 196.0);
    g.sym("kwh3_ct", 252.0, 200.0, 0, "-P5", "", "Power analyser UMG 604-PRO");
    g.text(252.0, 222.0, "Ethernet to the battery inverter: no current back into the grid", 2.5);
    for (i, (x, y)) in [(120.0, 128.0), (152.0, 152.0), (184.0, 176.0)].into_iter().enumerate() {
        g.w(x, y, x, 236.0);
        link(&mut g, x, 236.0, &format!("EX-L{}", i + 1));
    }
}

/// Three current transformers of `tag` in the phases coming down at x 120 / 152 / 184.
fn transformers(g: &mut Pg, tag: &str, ratio: &str) {
    for (i, (x, y)) in [(120.0, 120.0), (152.0, 144.0), (184.0, 168.0)].into_iter().enumerate() {
        let n = i + 1;
        let pins = [format!("{n}P1"), format!("{n}P2"), format!("{n}S1"), format!("{n}S2")];
        g.sym("ct", x, y, 0, tag, "", if i == 0 { ratio } else { "" }).pins(&pins.each_ref().map(String::as_str));
    }
}

/// The secondaries of the transformers to the current inputs of a meter whose
/// connection points are at the height `to` (x 252 … 292).
fn secondaries(g: &mut Pg, to: f32) {
    for (i, (sx, sy)) in [(128.0, 120.0), (160.0, 144.0), (192.0, 168.0)].into_iter().enumerate() {
        let x1 = 252.0 + 16.0 * i as f32;
        g.path(&[(sx, sy - 4.0), (x1, sy - 4.0), (x1, to)]);
        g.path(&[(sx, sy + 4.0), (x1 + 8.0, sy + 4.0), (x1 + 8.0, to)]);
    }
}

/// Plug sockets of a consumer at the end of its cable: L1 L2 L3 N PE at the x given.
fn consumer(g: &mut Pg, xs: [f32; 5], from: f32, y: f32, cable: &str, target: &str) {
    for (i, (c, port)) in [("BN", "L1"), ("BK", "L2"), ("GY", "L3"), ("BU", "N"), ("GNYE", "PE")].into_iter().enumerate() {
        core(g, (xs[i], from), (xs[i], y - 4.0), cable, c);
        g.sym("plug_f", xs[i], y, 0, target, port, "");
    }
}

/// The heat pumps: NH00 strips, residual current devices of type B and the terminals
/// of their cables.
fn heat_pumps(prj: &mut Project, section: u64) {
    let i = page(prj, section, "1", "Heat pumps", PageKind::Schematic, "DIST", "HV2");
    let mut g = Pg { prj, page: i };
    bus(&mut g, BUS[0], BUS[1]);
    for (x, n) in [(80.0, "25"), (220.0, "26")] {
        let strip = format!("-X{n}");
        g.sym("fuse_swdisc3", x, 72.0, 0, &format!("-Q{n}"), "", "Heat pump\nNH00 3 x 50 A");
        for i in 0..3 {
            let xi = x + 8.0 * i as f32;
            g.w(xi, 24.0 + 8.0 * i as f32, xi, 68.0).w(xi, 76.0, xi, 100.0).w(xi, 108.0, xi, 136.0);
            g.sym("term", xi, 140.0, 0, &strip, &(i + 1).to_string(), "");
        }
        g.sym("rcd4", x, 104.0, 0, &format!("-F{n}"), "", "Type B, 30 mA");
        g.w(x + 24.0, 48.0, x + 24.0, 100.0).w(x + 24.0, 108.0, x + 24.0, 136.0);
        g.sym("term_n", x + 24.0, 140.0, 0, &strip, "N", "");
        g.sym("term_pe", x + 32.0, 140.0, 0, &strip, "PE", "").w(x + 32.0, 56.0, x + 32.0, 136.0);
        let xs = [x, x + 8.0, x + 16.0, x + 24.0, x + 32.0];
        consumer(&mut g, xs, 144.0, 200.0, &format!("=DIST-W{n}"), &format!("+TR-EQ{n}"));
    }
    g.text(80.0, 216.0, "Heat pumps with inverters: residual current devices of type B", 2.5);
}

/// A feeder on an NH1 strip, the current transformers behind its poles, their energy
/// meter (Tree bus to the Miniserver) with its measuring voltages, the cable to the
/// consumer.
fn measured_feeder(prj: &mut Project, section: u64, n: &str, f: &Feeder) {
    let i = page(prj, section, n, f.desc, PageKind::Schematic, "DIST", "HV2");
    let mut g = Pg { prj, page: i };
    let k: usize = n.parse().unwrap_or(2);
    bus(&mut g, BUS[k - 1], BUS[k]);
    // The transformers sit behind the poles of the strip, on its busbar contacts: the
    // phases pass them on the way into the fuses.
    g.w(120.0, 24.0, 120.0, 112.0);
    g.path(&[(128.0, 32.0), (128.0, 80.0), (152.0, 80.0), (152.0, 136.0)]);
    g.path(&[(136.0, 40.0), (136.0, 84.0), (184.0, 84.0), (184.0, 160.0)]);
    transformers(&mut g, f.strip, "150/5 A");
    for (a, b) in [
        ((120.0, 128.0), (120.0, 196.0)),
        ((152.0, 152.0), (152.0, 188.0)),
        ((152.0, 188.0), (128.0, 188.0)),
        ((128.0, 188.0), (128.0, 196.0)),
        ((184.0, 176.0), (184.0, 192.0)),
        ((184.0, 192.0), (136.0, 192.0)),
        ((136.0, 192.0), (136.0, 196.0)),
    ] {
        g.w(a.0, a.1, b.0, b.1);
        cross_section(&mut g, BUSBAR);
    }
    g.sym("fuse_swdisc3", 120.0, 200.0, 0, f.strip, "", f.text);
    // Measuring voltages tapped at the busbar contacts, short-circuit proof to the MCB.
    for (x, y, to) in [(120.0, 180.0, 300.0), (152.0, 184.0, 308.0), (184.0, 186.0, 316.0)] {
        g.path(&[(x, y), (to, y), (to, 196.0)]);
        cross_section(&mut g, TAP);
    }
    g.sym("cb3", 300.0, 200.0, 0, f.taps, "", "Measuring voltages");
    for x in [300.0, 308.0, 316.0] {
        g.w(x, 204.0, x, 224.0);
    }
    g.w(324.0, 48.0, 324.0, 224.0);
    secondaries(&mut g, 224.0);
    g.sym("kwh3_ct", 252.0, 228.0, 0, f.meter, "", "Energy meter, Tree bus");
    // The cable on the outgoing terminals of the strip, N and PE on terminals.
    let terminals = format!("-X{}", &f.strip[2..]);
    g.w(216.0, 48.0, 216.0, 196.0).w(232.0, 56.0, 232.0, 196.0);
    g.sym("term_n", 216.0, 200.0, 0, &terminals, "N", "").sym("term_pe", 232.0, 200.0, 0, &terminals, "PE", "");
    consumer(&mut g, [120.0, 128.0, 136.0, 216.0, 232.0], 204.0, 228.0, f.cable, f.target);
}

/// The two coupling switches of the grid and system protection: PV inverters and
/// battery storage; the voltage and frequency relay at the grid side, the supply of
/// the protection circuit.
fn coupling_switches(prj: &mut Project, section: u64) {
    let i = page(prj, section, "1", "Coupling switches and NA protection relay", PageKind::Schematic, "NA", "HV1");
    let mut g = Pg { prj, page: i };
    for (i, (name, end)) in [("L1", 280.0), ("L2", 188.0), ("L3", 196.0), ("N", 288.0), ("PE", 132.0)].into_iter().enumerate() {
        let y = 24.0 + 8.0 * i as f32;
        link(&mut g, 20.0, y, &format!("{}-{name}", BUS[5]));
        g.w(20.0, y, end, y);
        if i < 3 {
            cross_section(&mut g, BUSBAR);
        }
    }
    // The phases of a coupling switch from busbar terminals on the distribution busbar
    // (`turned`: their conductors leave them upwards), its N from the N conductor.
    let fed = |g: &mut Pg, x0: f32, terminals: &str, turned: bool| {
        for i in 0..4 {
            let (x, y) = (x0 + 8.0 * i as f32, 24.0 + 8.0 * i as f32);
            if i == 3 {
                g.w(x, y, x, 76.0);
                cross_section(g, "16 mm²");
                continue;
            }
            g.w(x, y, x, 60.0);
            g.sym("term", x, 64.0, if turned { 2 } else { 0 }, terminals, &(i + 1).to_string(), "");
            g.w(x, 68.0, x, 76.0);
            cross_section(g, COUPLING);
        }
    };
    // Coupling switch of the PV inverters on their busbar, on to their feeders.
    fed(&mut g, 40.0, "-X30", true);
    g.sym("cb4", 40.0, 80.0, 0, "-Q30", "", "Coupling PV\n4P 160 A").pins(&["1", "3", "5", "N", "2", "4", "6", "N'"]);
    for (i, name) in ["L1", "L2", "L3", "N"].into_iter().enumerate() {
        let x = 40.0 + 8.0 * i as f32;
        g.w(x, 84.0, x, 236.0);
        if i == 3 {
            cross_section(&mut g, "16 mm²");
        }
        link(&mut g, x, 236.0, &format!("PV-{name}"));
    }
    g.w(72.0, 56.0, 72.0, 236.0);
    link(&mut g, 72.0, 236.0, "PV-PE");
    // Coupling switch of the battery storage, its cable to the battery inverter.
    fed(&mut g, 100.0, "-X31", false);
    g.sym("cb4", 100.0, 80.0, 0, "-Q31", "", "Coupling battery\n4P 160 A").pins(&["1", "3", "5", "N", "2", "4", "6", "N'"]);
    for (i, (c, port)) in [("BN", "L1"), ("BK", "L2"), ("GY", "L3"), ("BU", "N"), ("GNYE", "PE")].into_iter().enumerate() {
        let x = 100.0 + 8.0 * i as f32;
        // The PE from the PE rail, the others from the switch.
        core(&mut g, (x, if i == 4 { 56.0 } else { 84.0 }), (x, 196.0), "=PV-W43", c);
        g.sym("plug_f", x, 200.0, 0, "=PV+INV-TB43", port, "");
    }
    // Voltage and frequency relay at the grid side.
    g.sym("cb3", 180.0, 80.0, 0, "-F30", "", "Measuring");
    for (x, y) in [(180.0, 24.0), (188.0, 32.0), (196.0, 40.0)] {
        g.w(x, y, x, 76.0);
        cross_section(&mut g, TAP);
    }
    for x in [180.0, 188.0, 196.0] {
        g.w(x, 84.0, x, 116.0);
    }
    g.w(204.0, 48.0, 204.0, 116.0);
    g.sym("relay_grid_protection", 180.0, 120.0, 0, "-K30", "", "NA protection\nVDE-AR-N 4105");
    // Supply of the protection circuit, the releases and the motor operators.
    g.sym("cb1", 280.0, 80.0, 0, "-F33", "", "Protection circuit").w(280.0, 24.0, 280.0, 76.0);
    cross_section(&mut g, TAP);
    g.w(280.0, 84.0, 280.0, 236.0);
    g.w(288.0, 48.0, 288.0, 236.0);
    link(&mut g, 280.0, 236.0, "NA-L");
    link(&mut g, 288.0, 236.0, "NA-N");
}

/// The relay holds the undervoltage releases of both coupling switches; the motor
/// operators close them again in turn, the lamps show the trip and both closed.
fn na_control(prj: &mut Project, section: u64) {
    let i = page(prj, section, "2", "Releases and motor operators", PageKind::Schematic, "NA", "HV1");
    let mut g = Pg { prj, page: i };
    link(&mut g, 20.0, 24.0, "NA-L");
    g.w(20.0, 24.0, 60.0, 24.0).w(60.0, 24.0, 60.0, 56.0);
    // Turned over: common at the top, NO below it, NC below left.
    g.sym("co", 60.0, 60.0, 2, "-K30", "", "Grid and system OK");
    g.path(&[(56.0, 64.0), (56.0, 72.0), (44.0, 72.0), (44.0, 156.0)]);
    g.sym("lamp", 44.0, 160.0, 0, "-P30", "", "NA protection tripped");
    g.w(44.0, 164.0, 44.0, 236.0);
    g.w(60.0, 64.0, 60.0, 96.0).w(60.0, 96.0, 210.0, 96.0);
    for (x, tag) in [(100.0, "-Q30"), (150.0, "-Q31")] {
        g.w(x, 96.0, x, 156.0);
        g.sym("relay_undervoltage", x, 160.0, 0, tag, "", "Undervoltage release").pins(&["D1", "D2"]);
        g.w(x, 164.0, x, 236.0);
    }
    g.w(210.0, 96.0, 210.0, 104.0);
    g.sym("co", 210.0, 108.0, 2, "-Q30", "", "Coupling PV");
    g.path(&[(206.0, 112.0), (206.0, 124.0), (190.0, 124.0), (190.0, 156.0)]);
    g.sym("coil_motor_operator", 190.0, 160.0, 0, "-Q30", "", "Motor operator");
    g.w(190.0, 164.0, 190.0, 236.0);
    g.path(&[(210.0, 112.0), (210.0, 130.0), (290.0, 130.0), (290.0, 136.0)]);
    g.sym("co", 290.0, 140.0, 2, "-Q31", "", "Coupling battery");
    g.path(&[(286.0, 144.0), (286.0, 150.0), (270.0, 150.0), (270.0, 156.0)]);
    g.sym("coil_motor_operator", 270.0, 160.0, 0, "-Q31", "", "Motor operator");
    g.w(270.0, 164.0, 270.0, 236.0);
    g.path(&[(290.0, 144.0), (290.0, 148.0), (330.0, 148.0), (330.0, 156.0)]);
    g.sym("lamp", 330.0, 160.0, 0, "-P31", "", "Coupling switches closed");
    g.w(330.0, 164.0, 330.0, 236.0);
    link(&mut g, 20.0, 236.0, "NA-N");
    g.w(20.0, 236.0, 330.0, 236.0);
}

/// The feeders of the PV inverters behind the coupling switch.
fn inverter_feeders(prj: &mut Project, section: u64) {
    let i = page(prj, section, "1", "Inverter feeders", PageKind::Schematic, "PV", "HV1");
    let mut g = Pg { prj, page: i };
    for (i, (name, end)) in [("L1", 216.0), ("L2", 224.0), ("L3", 232.0), ("N", 240.0), ("PE", 248.0)].into_iter().enumerate() {
        let y = 24.0 + 8.0 * i as f32;
        link(&mut g, 20.0, y, &format!("PV-{name}"));
        g.w(20.0, y, end, y);
        if i < 3 {
            cross_section(&mut g, BUSBAR);
        }
    }
    feeder(&mut g, 80.0, "-Q41", "Inverter 1\nNH00 3 x 80 A", "-X41", "=PV-W41", "+INV-TB41", "Inverter 1");
    feeder(&mut g, 216.0, "-Q42", "Inverter 2\nNH00 3 x 80 A", "-X42", "=PV-W42", "+INV-TB42", "Inverter 2");
}

/// Two bayed univers cabinets of four fields, open to each other. +HV1: the supply
/// busbar with the terminals of the supply cables, the transformers of the metering
/// and the isolation strip; the busbar of the PV inverters with their coupling switch
/// on it; the beginning of the distribution busbar with its isolation strip and the
/// terminals of the coupling switches. +HV2: the distribution busbar on from +HV1 with
/// the transformers of the power analyser, the measured feeders and the heat pumps.
fn enclosures(prj: &mut Project, db: &PartsDb) {
    for part in [
        "HAG.FR24H2",
        "HAG.FZ831",
        "HAG.FZ402",
        panel::RAIL_PART,
        "GEN.BB-3X30X10-185",
        "GEN.BB-1X30X10",
        "GEN.GLAND-M32",
        "GEN.GLAND-M40",
        "GEN.GLAND-M63",
    ] {
        crate::parts::ensure_in_project(prj, db, part);
    }
    let e = panel::new_enclosure(prj, "HAG.FR24H2", "+HV1");
    let hv1 = push(prj, e);
    let e = panel::new_enclosure(prj, "HAG.FR24H2", "+HV2");
    let hv2 = push(prj, e);
    panel::bay_right_of(prj, hv2, hv1);
    // Bayed without the side walls between them: the busbar and the conductors run
    // through.
    prj.layout.enclosures[hv1].build.right_open = true;
    prj.layout.enclosures[hv2].build.left_open = true;
    let rail = |prj: &mut Project, enc: usize, x0: f32, x1: f32, y: f32| profile(prj, enc, ItemKind::Rail, panel::RAIL_PART, (x0, y), (x1, y));
    let bars = |prj: &mut Project, enc: usize, x0: f32, x1: f32| profile(prj, enc, ItemKind::Busbar, "GEN.BB-3X30X10-185", (x0, BARS), (x1, BARS));
    let strip = |prj: &mut Project, enc: usize, tag: &str, x: f32| {
        let h = panel::device_size(prj, tag).map_or(745.0, |d| d.h);
        panel::place_device(prj, enc, tag, p(x, BARS - h / 2.0));
    };
    // +HV1, the supply busbar: the terminals of the supply cables (room for a third),
    // the transformers of the metering in the bars, the isolation strip.
    // (The two cables come up through one flange: their terminals as far left and
    // right of it.)
    let supply = bars(prj, hv1, 20.0, 470.0);
    panel::busbars::place_on_bars(prj, hv1, "=GRID+HV1-X01", supply, 25.0);
    panel::busbars::place_on_bars(prj, hv1, "=GRID+HV1-X02", supply, 145.0);
    in_bars(prj, hv1, "=MS+HV1-T1", 235.0);
    strip(prj, hv1, "=MS+HV1-F10", 365.0);
    // The PEN bar below the phases, the PEN of both supply cables on it beside their
    // way up to the phase terminals.
    profile(prj, hv1, ItemKind::Busbar, "GEN.BB-1X30X10", (20.0, 1100.0), (345.0, 1100.0));
    panel::place_device(prj, hv1, "=GRID+HV1-X1", p(260.0, 1070.0));
    // The busbar of the PV inverters behind their coupling switch, which sits on it.
    bars(prj, hv1, 475.0, 755.0);
    let h = panel::device_size(prj, "=NA+HV1-Q30").map_or(165.0, |d| d.h);
    panel::place_device(prj, hv1, "=NA+HV1-Q30", p(480.0, BARS - h / 2.0));
    strip(prj, hv1, "=PV+HV1-Q41", 635.0);
    strip(prj, hv1, "=PV+HV1-Q42", 700.0);
    // The distribution busbar begins: its isolation strip, the terminals the coupling
    // switches are fed from.
    let dist = bars(prj, hv1, 775.0, 990.0);
    strip(prj, hv1, "=MS+HV1-Q10", 780.0);
    panel::busbars::place_on_bars(prj, hv1, "=NA+HV1-X30", dist, 120.0);
    panel::busbars::place_on_bars(prj, hv1, "=NA+HV1-X31", dist, 170.0);
    // Above the busbars the surge arrester, the metering and the protection circuit;
    // below them the coupling switch of the battery storage and the terminals of the
    // inverters.
    rail(prj, hv1, 20.0, 980.0, 300.0);
    // A cell and more apart: every wire comes up to its terminal beside the ways out of
    // the device next to it.
    let mut x = 30.0;
    for tag in ["=GRID+HV1-F1", "=MS+HV1-F11", "=MS+HV1-X1", "=NA+HV1-K30", "=NA+HV1-F30", "=NA+HV1-F33", "=NA+HV1-P30", "=NA+HV1-P31"] {
        let (w, h) = panel::device_size(prj, tag).map_or((40.0, 40.0), |d| (d.w, d.h));
        panel::place_device(prj, hv1, tag, p(x, 300.0 - h / 2.0));
        x += panel::snap(w + 35.0);
    }
    panel::place_device(prj, hv1, "=NA+HV1-Q31", p(835.0, 1300.0));
    rail(prj, hv1, 580.0, 820.0, 1600.0);
    on_rail(prj, hv1, 1600.0, 590.0, &["=PV+HV1-X41", "=PV+HV1-X42"]);
    // +HV2, the distribution busbar on from +HV1: the transformers of the power
    // analyser in the bars, the measured feeders, the heat pumps.
    bars(prj, hv2, 10.0, 980.0);
    in_bars(prj, hv2, "=EMS+HV2-T5", 20.0);
    for (tag, x) in [("=DIST+HV2-Q21", 210.0), ("=DIST+HV2-Q22", 325.0), ("=DIST+HV2-Q23", 440.0), ("=DIST+HV2-Q24", 555.0), ("=DIST+HV2-Q25", 670.0), ("=DIST+HV2-Q26", 735.0)] {
        strip(prj, hv2, tag, x);
    }
    // Meters and their voltage MCBs at the top (room above them for the leads of the
    // transformers), N and PE of the measured feeders below them, clear of the cables'
    // way down.
    rail(prj, hv2, 20.0, 980.0, 205.0);
    // Each meter with its MCB above its strip, the power analyser above its transformers.
    on_rail(prj, hv2, 205.0, 30.0, &["=EMS+HV2-P5", "=EMS+HV2-F5", "=DIST+HV2-P21", "=DIST+HV2-F21", "=DIST+HV2-P22", "=DIST+HV2-F22", "=DIST+HV2-P23", "=DIST+HV2-F23", "=DIST+HV2-P24", "=DIST+HV2-F24"]);
    rail(prj, hv2, 20.0, 980.0, 335.0);
    on_rail(prj, hv2, 335.0, 340.0, &["=DIST+HV2-X21", "=DIST+HV2-X22", "=DIST+HV2-X23", "=DIST+HV2-X24"]);
    // The residual current devices and the terminals of the heat pumps below their
    // strips.
    rail(prj, hv2, 645.0, 875.0, 1380.0);
    on_rail(prj, hv2, 1380.0, 665.0, &["=DIST+HV2-F25", "=DIST+HV2-F26"]);
    rail(prj, hv2, 625.0, 875.0, 1650.0);
    on_rail(prj, hv2, 1650.0, 635.0, &["=DIST+HV2-X25", "=DIST+HV2-X26"]);
    // Cable entries: flanges with membranes for the large cables (the one of the supply
    // cables midway below their terminals), glands for the others.
    panel::place_accessory(prj, hv1, "HAG.FZ402", Face::Bottom, p(30.0, 150.0));
    for (tag, part) in [("=PV+HV1-X41", "GEN.GLAND-M40"), ("=PV+HV1-X42", "GEN.GLAND-M40"), ("=NA+HV1-Q31", "GEN.GLAND-M63")] {
        gland_at(prj, hv1, part, tag, Face::Bottom);
    }
    for (tag, part) in [
        ("=DIST+HV2-Q21", "GEN.GLAND-M63"),
        ("=DIST+HV2-Q22", "GEN.GLAND-M63"),
        ("=DIST+HV2-Q23", "GEN.GLAND-M32"),
        ("=DIST+HV2-Q24", "GEN.GLAND-M32"),
        ("=DIST+HV2-X25", "GEN.GLAND-M32"),
        ("=DIST+HV2-X26", "GEN.GLAND-M32"),
    ] {
        gland_at(prj, hv2, part, tag, Face::Bottom);
    }
    // Entries prepared in the roof, the lifting eyes for the crane.
    for x in [300.0, 650.0] {
        panel::place_accessory(prj, hv2, "HAG.FZ402", Face::Roof, p(x, 150.0));
    }
    for (enc, x) in [(hv1, 30.0), (hv2, 980.0)] {
        panel::place_accessory(prj, enc, "HAG.FZ831", Face::Roof, p(x, 180.0));
    }
}

/// Height of the middle of the busbar systems on the mounting plates.
const BARS: f32 = 800.0;

/// Places the current transformer set `tag` in the bars of the busbar system at
/// [`BARS`], `x` mm from the left of the plate: turned by a quarter, the bars pass
/// through its transformers.
fn in_bars(prj: &mut Project, enc: usize, tag: &str, x: f32) {
    let Some((w, h)) = panel::device_size(prj, tag).map(|d| (d.w, d.h)) else { return };
    let Some(id) = panel::place_device(prj, enc, tag, p(x, BARS - w / 2.0)) else { return };
    if let Some(it) = prj.layout.enclosures[enc].items.iter_mut().find(|i| i.id == id) {
        (it.w, it.h) = (h, w);
    }
}
