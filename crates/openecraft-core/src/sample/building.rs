//! Level 11: building services. The supply cable comes into the house connection room
//! +HR through a building entry to the house connection box +HAK (an enclosure with a
//! hinged cover: the NH fuse base of the utility and the PEN terminal block inside),
//! on to the main distribution +MD with its busbar system, Hager NH fuse switch disconnectors of three
//! sizes and energy meters; the water main comes in through a second building entry
//! and passes the remote controlled valve. Cables and the water pipe lead through wall
//! penetrations into the technical room +TR: control cabinets with the PLC and the
//! pump starters, tanks, booster pumps and the pressure vessel, the generator junction
//! box of the PV system, Victron MultiPlus-II inverter / chargers and Inverter RS on
//! the wall, a Cerbo GX and Pylontech batteries in a 19-inch rack. The residual
//! current device of the backup outputs carries a Hager CZ001 signal contact (an
//! accessory beside it on the rail) that lights a lamp when it trips. The main
//! earthing busbar beside the house connection box joins the PE of the main
//! distribution (main bonding conductor) and the foundation earth electrode, whose
//! connection lug comes out of the wall below it.
//!
//! The manufacturer parts come from the catalogs a new parts server starts with
//! ([`crate::parts::bundled`]).

use super::*;
use crate::structure::Aspect;

mod air;
mod energy;
mod heat;
mod layout;
mod power;
mod water;

pub(super) fn project() -> Project {
    let mut prj = Project { pages: Vec::new(), ..Default::default() };
    prj.info = ProjectInfo {
        name: "Building services".into(),
        description: "House connection, main distribution, water supply\nand PV system with battery storage".into(),
        company: "OpenECraft".into(),
        creator: "OpenECraft".into(),
        date: "2026-10-29".into(),
        revision: "A".into(),
        ..Default::default()
    };
    prj.apply_preset(crate::potentials::Preset::Iec60204);
    for (name, kind, voltage, color, cs) in [
        ("PEN", crate::potentials::PotentialKind::Pe, "", "GNYE", "16 mm²"),
        ("BAT+", crate::potentials::PotentialKind::DcPlus, "48 V DC", "RD", "70 mm²"),
        ("BAT-", crate::potentials::PotentialKind::DcMinus, "48 V DC", "BK", "70 mm²"),
    ] {
        prj.potentials.push(crate::potentials::PotentialDef { name: name.into(), kind, voltage: voltage.into(), color: color.into(), cross_section: cs.into(), ..Default::default() });
    }
    for (aspect, id, desc) in [
        (Aspect::Function, "GRID", "Supply and house connection"),
        (Aspect::Function, "MD", "Main distribution and metering"),
        (Aspect::Function, "CTL", "Building automation"),
        (Aspect::Function, "WAT", "Water supply"),
        (Aspect::Function, "AIR", "Compressed air"),
        (Aspect::Function, "HEAT", "Heating and hot water"),
        (Aspect::Function, "PV", "Photovoltaics"),
        (Aspect::Function, "ESS", "Battery storage and inverters"),
        (Aspect::Location, "KVS", "Cable distribution cabinet of the utility (street)"),
        (Aspect::Location, "HR", "House connection room"),
        (Aspect::Location, "HAK", "House connection box"),
        (Aspect::Location, "TR", "Technical room"),
        (Aspect::Location, "MD", "Main distribution"),
        (Aspect::Location, "CC1", "Control cabinet, automation"),
        (Aspect::Location, "CC2", "Control cabinet, pumps"),
        (Aspect::Location, "ACD", "AC distribution of the battery storage"),
        (Aspect::Location, "GAK", "Generator junction box"),
        (Aspect::Location, "RACK", "19-inch rack"),
        (Aspect::Location, "ROOF", "Roof (PV field)"),
    ] {
        prj.structures.define(aspect, id, desc);
    }
    // Sections in two levels: 2.1 / 2.2 below "Power supply", 5.1 / 5.2 below "Energy system".
    let general = crate::sections::add(&mut prj, 0, "General");
    let power = crate::sections::add(&mut prj, 0, "Power supply");
    let house = crate::sections::add(&mut prj, power, "House connection");
    let main = crate::sections::add(&mut prj, power, "Main distribution");
    let control = crate::sections::add(&mut prj, 0, "Building automation");
    let water = crate::sections::add(&mut prj, 0, "Water supply");
    let air = crate::sections::add(&mut prj, 0, "Compressed air");
    let heating = crate::sections::add(&mut prj, 0, "Heating and hot water");
    let energy = crate::sections::add(&mut prj, 0, "Energy system");
    let pv = crate::sections::add(&mut prj, energy, "Photovoltaics");
    let storage = crate::sections::add(&mut prj, energy, "Battery storage");
    let layout = crate::sections::add(&mut prj, 0, "Panel layout");
    let reports = crate::sections::add(&mut prj, 0, "Reports");

    sec_page(&mut prj, general, "1", "Cover sheet", PageKind::Cover, "", "");
    sec_page(&mut prj, general, "2", "Table of contents", PageKind::Report(ReportKind::TableOfContents.into()), "", "");
    power::single_line(&mut prj, general);
    power::house_connection(&mut prj, house);
    power::earthing(&mut prj, house);
    power::feeder_q3(&mut prj, main);
    power::control_supply(&mut prj, control);
    power::plc(&mut prj, control);
    power::pump_power(&mut prj, water);
    water::pid(&mut prj, water);
    air::power(&mut prj, air);
    air::control(&mut prj, air);
    air::pid(&mut prj, air);
    heat::power(&mut prj, heating);
    heat::control(&mut prj, heating);
    heat::measurements(&mut prj, heating);
    heat::pid(&mut prj, heating);
    energy::pv(&mut prj, pv);
    energy::dc(&mut prj, storage);
    energy::ac_input(&mut prj, storage);
    energy::backup(&mut prj, storage);
    energy::communication(&mut prj, storage);
    for (n, desc, kind) in [
        ("1", "Bill of materials", ReportKind::BillOfMaterials),
        ("2", "Device list", ReportKind::DeviceList),
        ("3", "Terminal diagram", ReportKind::TerminalDiagram),
        ("4", "Cable overview", ReportKind::CableOverview),
        ("5", "PLC assignment", ReportKind::PlcAssignment),
        ("6", "Plug diagram", ReportKind::PlugDiagram),
        ("7", "Panel legend", ReportKind::PanelLegend),
        ("8", "3D routing", ReportKind::RoutingList),
        ("9", "Hose list", ReportKind::HoseList),
        ("10", "19-inch rack elevation", ReportKind::RackElevation),
    ] {
        sec_page(&mut prj, reports, n, desc, PageKind::Report(kind.into()), "", "");
    }

    let db = server_parts();
    assign_parts(&mut prj, &db, DEVICES);
    for (tag, part, count) in COUNTED {
        crate::parts::add_to_device(&mut prj, &db, tag, part, *count);
    }
    for (card, start) in [("=CTL+CC1-A2", 0), ("=CTL+CC1-A3", 0), ("=CTL+CC1-A4", 64)] {
        prj.plc_cards.insert(card.into(), PlcCard { start_byte: start, rack: "0".into(), ..Default::default() });
    }
    for (card, slot) in [("=CTL+CC2-A11", "1"), ("=CTL+CC2-A12", "2"), ("=CTL+CC2-A13", "3"), ("=CTL+CC2-A14", "4")] {
        prj.plc_cards.insert(card.into(), PlcCard { rack: "1".into(), slot: slot.into(), description: "EtherCAT -A10".into(), ..Default::default() });
    }
    // The valve actuator turns the motor valve, the motors drive the pumps.
    for (fluid, electric) in [("=WAT+HR-QM2", "=WAT+HR-MA1"), ("=WAT+TR-GP1", "=WAT+TR-M1"), ("=WAT+TR-GP2", "=WAT+TR-M2"), ("=AIR+TR-GQ1", "=AIR+TR-M1")] {
        prj.fluid.links.insert(fluid.into(), vec![electric.into()]);
    }
    layout::cables(&mut prj, &db);
    layout::rooms(&mut prj, &db, layout);
    crate::sections::sort_pages(&mut prj);
    crate::sizing::size_wires(&mut prj);
    number_wires(&mut prj);
    super::bare_large_conductors(&mut prj);
    prj
}

/// The parts a new parts server starts with: the generic catalog and the bundled
/// manufacturer catalogs.
fn server_parts() -> PartsDb {
    PartsDb { parts: crate::parts::bundled::default_parts() }
}

/// Device tags and their parts.
const DEVICES: &[(&str, &[&str])] = &[
    ("=GRID+KVS-X1", &["GEN.TB-240"]),
    ("=GRID+HAK-F1", &["GEN.NH2-BASE-3P-400A"]),
    ("=GRID+HAK-X0", &["GEN.TB-PEN-240-V"]),
    ("=GRID+HR-X1", &["GEN.MET-BAR-10"]),
    ("=GRID+HR-E1", &["GEN.FE-LUG-V4A-10"]),
    ("=MD+MD-X0", &["GEN.BB-FEED-185"]),
    ("=MD+MD-X1", &["GEN.TB-PEN-150"]),
    ("=MD+MD-X2", &["GEN.TB-PE-35"]),
    ("=MD+MD-Q1", &["HAG.LT0057"]),
    ("=MD+MD-Q2", &["HAG.LT056"]),
    ("=MD+MD-Q3", &["HAG.LT253"]),
    ("=MD+MD-P1", &["HAG.ECR380D"]),
    ("=MD+MD-P2", &["HAG.ECR380D"]),
    ("=MD+MD-P3", &["HAG.ECR300C"]),
    ("=MD+MD-T11", &["GEN.CT-150-5A"]),
    ("=MD+MD-F10", &["DEH.952405"]),
    ("=MD+MD-F11", &["GEN.FUSE-10X38-3P-2A"]),
    ("=CTL+CC1-X0", &["GEN.TB-35"]),
    ("=CTL+CC1-X1", &["GEN.TB-2.5"]),
    ("=CTL+CC1-X3", &["GEN.TB-2.5"]),
    ("=CTL+CC1-Q1", &["HAG.SBN363"]),
    ("=CTL+CC1-F1", &["HAG.MCN116"]),
    ("=CTL+CC1-X2", &["GEN.SOCKET-DIN-16A"]),
    ("=CTL+CC1-F2", &["HAG.MCN110"]),
    ("=CTL+CC1-B2", &["GEN.TEMP-SW-60C"]),
    ("=CTL+CC1-E2", &["GEN.HEATER-100W"]),
    ("=CTL+CC1-T1", &["PXC.2904601"]),
    ("=CTL+CC1-A1", &["GEN.PLC-CPU"]),
    ("=CTL+CC1-A2", &["GEN.PLC-DI16"]),
    ("=CTL+CC1-A3", &["GEN.PLC-DO16"]),
    ("=CTL+CC1-A4", &["GEN.PLC-AI4"]),
    ("=CTL+CC1-P1", &["GEN.LMP-RD-24"]),
    ("=CTL+CC1-P2", &["GEN.HORN-24"]),
    ("=CTL+CC2-A10", &["BECK.EK1100"]),
    ("=CTL+CC2-A11", &["BECK.EL1008"]),
    ("=CTL+CC2-A12", &["BECK.EL2008"]),
    ("=WAT+CC2-X0", &["GEN.TB-35"]),
    ("=WAT+CC2-Q1", &["HAG.LT056"]),
    ("=WAT+CC2-Q11", &["GEN.MPS-4-6.3A", "GEN.MPS-AUX-1NO1NC"]),
    ("=WAT+CC2-Q12", &["GEN.MPS-4-6.3A", "GEN.MPS-AUX-1NO1NC"]),
    ("=WAT+CC2-K11", &["GEN.CT-9A-24DC"]),
    ("=WAT+CC2-K12", &["GEN.CT-9A-24DC"]),
    ("=WAT+CC2-X11", &["GEN.TB-2.5"]),
    ("=WAT+CC2-X12", &["GEN.TB-2.5"]),
    ("=WAT+TR-M1", &["GEN.MOT-2.2KW"]),
    ("=WAT+TR-M2", &["GEN.MOT-2.2KW"]),
    ("=WAT+HR-MA1", &["GEN.ACT-VLV-24V"]),
    ("=WAT+TR-BL1", &["GEN.LT-HYD-2M"]),
    ("=WAT+TR-BL2", &["GEN.FLOAT-SW-1NO"]),
    ("=WAT+TR-BP1", &["GEN.PT-10BAR-420"]),
    ("=WAT+HR-UM1", &["GEN.HOUSE-ENTRY-DN32"]),
    ("=WAT+HR-QM1", &["GEN.VLV-BALL-DN32"]),
    ("=WAT+HR-BF1", &["GEN.WM-DN32"]),
    ("=WAT+HR-RM1", &["GEN.CHECK-DN32"]),
    ("=WAT+HR-QM2", &["GEN.VLV-MOT-DN32"]),
    ("=WAT+TR-CM1", &["GEN.TANK-1000L"]),
    ("=WAT+TR-QM3", &["GEN.VLV-BALL-DN32"]),
    ("=WAT+TR-GP1", &["GEN.PUMP-BOOST-DN32"]),
    ("=WAT+TR-GP2", &["GEN.PUMP-BOOST-DN32"]),
    ("=WAT+TR-RM2", &["GEN.CHECK-DN32"]),
    ("=WAT+TR-RM3", &["GEN.CHECK-DN32"]),
    ("=WAT+TR-CM2", &["GEN.TANK-PRESS-100L"]),
    ("=WAT+TR-QM4", &["GEN.VLV-BALL-DN32"]),
    ("=WAT+TR-UM2", &["GEN.MANIFOLD-DN32-4"]),
    ("=WAT+TR-BL3", &["GEN.FLOAT-SW-1NO"]),
    ("=AIR+CC2-Q1", &["SIE.3RV2011-1JA10", "SIE.3RV2901-1E"]),
    ("=AIR+CC2-K1", &["GEN.CT-9A-24DC"]),
    ("=AIR+CC2-X1", &["GEN.TB-2.5"]),
    ("=AIR+TR-M1", &["GEN.MOT-4KW"]),
    ("=AIR+TR-BP1", &["GEN.PT-16BAR-420"]),
    ("=AIR+TR-RN1", &["GEN.INTAKE-FILTER-G1"]),
    ("=AIR+TR-GQ1", &["GEN.COMP-PISTON-450"]),
    ("=AIR+TR-CM1", &["GEN.RECEIVER-270L"]),
    ("=AIR+TR-HQ1", &["GEN.DRYER-REFR-600"]),
    ("=AIR+TR-HQ2", &["GEN.DRYER-MEMBRANE-G38"]),
    ("=AIR+TR-HQ3", &["GEN.LUB-G38"]),
    ("=AIR+TR-HQ4", &["GEN.LUB-G38"]),
    ("=AIR+TR-QM1", &["GEN.VLV-BALL-G12"]),
    ("=AIR+TR-QM2", &["GEN.VLV-BALL-G12"]),
    ("=AIR+TR-QM3", &["GEN.VLV-BALL-G12"]),
    ("=AIR+TR-QM4", &["GEN.VLV-BALL-G12"]),
    ("=AIR+TR-AZ1", &["GEN.FR-G38-WALL"]),
    ("=AIR+TR-AZ2", &["GEN.FR-G38-WALL"]),
    ("=AIR+TR-AZ3", &["GEN.FR-G38-WALL"]),
    ("=AIR+TR-AZ4", &["GEN.FR-G38-WALL"]),
    ("=AIR+TR-XM1", &["GEN.COUPLING-NW72"]),
    ("=AIR+TR-XM2", &["GEN.COUPLING-NW72"]),
    ("=AIR+TR-XM3", &["GEN.COUPLING-NW72"]),
    ("=AIR+TR-XM4", &["GEN.COUPLING-NW72"]),
    ("=HEAT+CC2-Q1", &["HAG.MCN316"]),
    ("=HEAT+CC2-F2", &["HAG.MCN106"]),
    ("=HEAT+CC2-K1", &["GEN.RLY-24DC-1NO"]),
    ("=HEAT+CC2-K2", &["GEN.RLY-24DC-1NO"]),
    ("=HEAT+CC2-K3", &["GEN.RLY-24DC-1NO"]),
    ("=HEAT+CC2-X1", &["GEN.TB-2.5"]),
    ("=HEAT+TR-EQ1", &["GEN.HP-AW-12KW"]),
    ("=HEAT+TR-CM1", &["GEN.TANK-BUFFER-500L"]),
    ("=HEAT+TR-GP1", &["GEN.PUMP-CIRC-DN25"]),
    ("=HEAT+TR-GP2", &["GEN.PUMP-CIRC-DN25"]),
    ("=HEAT+TR-QM1", &["GEN.VLV-BALL-DN32"]),
    ("=HEAT+TR-QM2", &["GEN.VLV-BALL-DN32"]),
    ("=HEAT+TR-RM1", &["GEN.CHECK-DN32"]),
    ("=HEAT+TR-RM2", &["GEN.CHECK-DN32"]),
    ("=HEAT+TR-UM1", &["GEN.MANIFOLD-DN32-4"]),
    ("=HEAT+TR-UM2", &["GEN.MANIFOLD-DN32-4"]),
    ("=HEAT+TR-BT1", &["GEN.TT-PT1000-420"]),
    ("=HEAT+TR-BT2", &["GEN.TT-PT1000-420"]),
    ("=HEAT+TR-BT3", &["GEN.TT-PT1000-420"]),
    ("=HEAT+TR-BT4", &["GEN.TT-OUT-420"]),
    ("=HEAT+TR-BP1", &["GEN.PT-10BAR-420"]),
    ("=CTL+CC2-A13", &["BECK.EL4004"]),
    ("=CTL+CC2-A14", &["BECK.EL3054"]),
    ("=PV+GAK-F1", &["GEN.FUSE-PV-2P-15A"]),
    ("=PV+GAK-F2", &["GEN.FUSE-PV-2P-15A"]),
    ("=PV+GAK-F3", &["DEH.952515"]),
    ("=PV+GAK-F4", &["DEH.952515"]),
    ("=PV+GAK-X1", &["GEN.TB-PV-6"]),
    ("=PV+TR-T1", &["VIC.PIN482601000"]),
    ("=PV+TR-T2", &["VIC.PIN482601000"]),
    ("=ESS+TR-T1", &["VIC.PMP482505010"]),
    ("=ESS+TR-T2", &["VIC.PMP482505010"]),
    ("=ESS+TR-T3", &["VIC.PMP482505010"]),
    ("=ESS+TR-K1", &["VIC.BPP900450100"]),
    ("=ESS+TR-P1", &["VIC.SHU050150050"]),
    ("=ESS+TR-F1", &["VIC.LYNX-DISTRIBUTOR-M10"]),
    ("=ESS+TR-F2", &["VIC.LYNX-DISTRIBUTOR-M10"]),
    ("=ESS+TR-F3", &["VIC.LYNX-DISTRIBUTOR-M10"]),
    ("=ESS+TR-F4", &["GEN.FUSE-5X20-2A"]),
    ("=ESS+RACK-G1", &["PYL.US5000"]),
    ("=ESS+RACK-G2", &["PYL.US5000"]),
    ("=ESS+RACK-G3", &["PYL.US5000"]),
    ("=ESS+RACK-G4", &["PYL.US5000"]),
    ("=ESS+RACK-K2", &["GEN.RK-SWITCH-24P"]),
    ("=ESS+ACD-X0", &["GEN.TB-35"]),
    ("=ESS+ACD-Q1", &["HAG.SBN363"]),
    ("=ESS+RACK-X3", &["GEN.RK-PDU-8"]),
    ("=ESS+ACD-X1", &["GEN.TB-2.5"]),
    ("=ESS+ACD-X2", &["GEN.TB-2.5"]),
    ("=ESS+ACD-F1", &["HAG.MCN150"]),
    ("=ESS+ACD-F2", &["HAG.MCN150"]),
    ("=ESS+ACD-F3", &["HAG.MCN150"]),
    ("=ESS+ACD-F10", &["HAG.CDA440D", "HAG.CZ001"]),
    ("=ESS+ACD-P10", &["GEN.LMP-DIN-RD-230"]),
    ("=ESS+ACD-F11", &["HAG.MCN116"]),
    ("=ESS+ACD-F12", &["HAG.MCN110"]),
    ("=ESS+ACD-F13", &["HAG.MCN110"]),
    ("=ESS+ACD-X11", &["GEN.SOCKET-DIN-16A"]),
    ("=ESS+ACD-F21", &["HAG.MCN116"]),
    ("=ESS+ACD-F22", &["HAG.MCN116"]),
    ("=ESS+ACD-X21", &["GEN.SOCKET-DIN-16A"]),
    ("=ESS+ACD-X22", &["GEN.SOCKET-DIN-16A"]),
];

/// Parts used more than once by a device: fuse links, modules, terminals.
const COUNTED: &[(&str, &str, u32)] = &[
    ("=GRID+KVS-X1", "GEN.TB-240", 3),
    ("=GRID+HAK-F1", "GEN.NH2-GG-250A", 3),
    ("=MD+MD-X0", "GEN.BB-FEED-185", 2),
    ("=MD+MD-X2", "GEN.TB-PE-35", 3),
    ("=MD+MD-X2", "GEN.TB-N-35", 1),
    ("=MD+MD-T11", "GEN.CT-150-5A", 2),
    ("=MD+MD-Q1", "GEN.NH000-GG-63A", 3),
    ("=MD+MD-Q2", "GEN.NH00-GG-80A", 3),
    ("=MD+MD-Q3", "GEN.NH2-GG-80A", 3),
    ("=CTL+CC1-X0", "GEN.TB-35", 2),
    ("=CTL+CC1-X0", "GEN.TB-N-35", 1),
    ("=CTL+CC1-X0", "GEN.TB-PE-35", 1),
    ("=CTL+CC1-X1", "GEN.TB-N-2.5", 1),
    ("=CTL+CC1-X3", "GEN.TB-2.5", 3),
    ("=CTL+CC1-X1", "GEN.TB-PE-2.5", 1),
    ("=WAT+CC2-X0", "GEN.TB-35", 2),
    ("=WAT+CC2-X0", "GEN.TB-N-35", 1),
    ("=WAT+CC2-X0", "GEN.TB-PE-35", 1),
    ("=WAT+CC2-Q1", "GEN.NH00-GG-50A", 3),
    ("=WAT+CC2-X11", "GEN.TB-2.5", 2),
    ("=WAT+CC2-X11", "GEN.TB-PE-2.5", 1),
    ("=WAT+CC2-X12", "GEN.TB-2.5", 2),
    ("=WAT+CC2-X12", "GEN.TB-PE-2.5", 1),
    ("=AIR+CC2-X1", "GEN.TB-2.5", 2),
    ("=AIR+CC2-X1", "GEN.TB-PE-2.5", 1),
    ("=HEAT+CC2-X1", "GEN.TB-2.5", 4),
    ("=HEAT+CC2-X1", "GEN.TB-N-2.5", 3),
    ("=HEAT+CC2-X1", "GEN.TB-PE-2.5", 3),
    ("=PV+ROOF-G1", "GEN.PV-430W", 9),
    ("=PV+ROOF-G2", "GEN.PV-430W", 9),
    ("=PV+GAK-X1", "GEN.TB-PV-6", 3),
    ("=ESS+ACD-X0", "GEN.TB-35", 2),
    ("=ESS+ACD-X0", "GEN.TB-N-35", 1),
    ("=ESS+ACD-X0", "GEN.TB-PE-35", 1),
    ("=ESS+ACD-X1", "GEN.TB-2.5", 1),
    ("=ESS+ACD-X1", "GEN.TB-N-2.5", 2),
    ("=ESS+ACD-X1", "GEN.TB-PE-2.5", 2),
    ("=ESS+ACD-X2", "GEN.TB-2.5", 1),
    ("=ESS+ACD-X2", "GEN.TB-N-2.5", 2),
    ("=ESS+ACD-X2", "GEN.TB-PE-2.5", 2),
];

/// A page of a section with its structure; the name starts with the section number.
fn sec_page(prj: &mut Project, section: u64, n: &str, desc: &str, kind: PageKind, function: &str, location: &str) -> usize {
    let name = format!("{}.{n}", crate::sections::full_number(prj, section));
    let i = add_page(prj, &name, desc, kind);
    let page = &mut prj.pages[i];
    page.section = section;
    page.function = function.into();
    page.location = location.into();
    i
}

impl Pg<'_> {
    /// Wire carrying core `core` of cable `cable`.
    fn core(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, cable: &str, core: &str) -> &mut Self {
        self.w(x1, y1, x2, y2);
        if let Some(w) = self.prj.pages[self.page].wires.last_mut() {
            w.cable = cable.into();
            w.core = core.into();
        }
        self
    }

    /// Interruption point (page connector) at a wire end.
    fn link(&mut self, x: f32, y: f32, name: &str) -> &mut Self {
        self.sym("intr", x, y, 0, "", name, "")
    }

    /// Rails +24V at the top and 0V at the bottom of a control page.
    fn dc_rails(&mut self) -> &mut Self {
        self.sym("intr", 20.0, 24.0, 1, "", "+24V", "").w(20.0, 24.0, 400.0, 24.0);
        self.sym("intr", 20.0, 236.0, 1, "", "0V", "").w(20.0, 236.0, 400.0, 236.0)
    }

    /// A socket of a device for a plug (RJ45, CAN) at `(x, y)` (its connection point
    /// 4 mm above).
    fn socket(&mut self, x: f32, y: f32, tag: &str, port: &str) -> &mut Self {
        self.sym("plug_f", x, y, 0, tag, port, "")
    }
}

