//! The bundled motor starter templates: power and control circuit of one
//! three-phase motor, sized by formulas from its rated power (rated current, setting
//! range of the motor protection switch, contactors, cross-section) and with report
//! pages that the parameter `REPORTS` leaves out.

use super::{TemplateDefinition, TemplateFormula, TemplateParameter};
use crate::model::*;
use crate::sample::{add_page, number_wires, Pg};
use openecraft_i18n::{tr, tr_const, tr_fmt};

/// Setting ranges of the motor protection switches of the built-in catalog: upper
/// limit, range, part.
const RANGES: [(&str, &str, &str); 10] = [
    ("1", "0.63 - 1 A", "GEN.MPS-0.63-1A"),
    ("1.6", "1 - 1.6 A", "GEN.MPS-1-1.6A"),
    ("2.5", "1.6 - 2.5 A", "GEN.MPS-1.6-2.5A"),
    ("4", "2.5 - 4 A", "GEN.MPS-2.5-4A"),
    ("6.3", "4 - 6.3 A", "GEN.MPS-4-6.3A"),
    ("10", "6.3 - 10 A", "GEN.MPS-6.3-10A"),
    ("16", "10 - 16 A", "GEN.MPS-10-16A"),
    ("20", "16 - 20 A", "GEN.MPS-16-20A"),
    ("25", "20 - 25 A", "GEN.MPS-20-25A"),
    ("32", "25 - 32 A", "GEN.MPS-25-32A"),
];

/// Contactors of the built-in catalog: rated current AC-3, part.
const CONTACTORS: [(&str, &str); 4] = [("9", "GEN.CT-9A-24DC"), ("18", "GEN.CT-18A-24DC"), ("25", "GEN.CT-25A-24DC"), ("32", "GEN.CT-32A-24DC")];

/// `step(x, limit, value, …, otherwise)` over a table.
fn step(x: &str, rows: &[(&str, String)], otherwise: &str) -> String {
    let mut s = format!("step({x}");
    for (limit, value) in rows {
        s += &format!(", {limit}, {value}");
    }
    s + &format!(", {otherwise})")
}

fn quoted(s: &str) -> String {
    format!("\"{s}\"")
}

/// The contactor for the current `x`.
fn contactor(x: &str) -> String {
    step(x, &CONTACTORS.map(|(i, p)| (i, quoted(p))), "\"\"")
}

fn parameter(name: &str, label: &str, unit: &str, default: &str, number: bool) -> TemplateParameter {
    TemplateParameter { name: name.into(), label: label.into(), unit: unit.into(), default: default.into(), choices: Vec::new(), number }
}

fn formula(name: &str, label: &str, unit: &str, formula: String) -> TemplateFormula {
    TemplateFormula { name: name.into(), label: label.into(), unit: unit.into(), formula }
}

/// Parameters and sizing of a motor of `power` kW; `more` formulas follow.
fn definition(power: &str, more: Vec<TemplateFormula>) -> TemplateDefinition {
    let mut reports = parameter("REPORTS", tr_const!("Report pages"), "", "yes", false);
    reports.choices = vec!["yes".into(), "no".into()];
    let parameters = vec![
        parameter("MOTOR", tr_const!("Driven machine"), "", tr("Conveyor"), false),
        parameter("POWER_KW", tr_const!("Rated power"), "kW", power, true),
        parameter("VOLTAGE", tr_const!("Mains voltage"), "V", "400", true),
        parameter("COS_PHI", tr_const!("Power factor cos phi"), "", "0.85", true),
        parameter("EFFICIENCY", tr_const!("Efficiency"), "", "0.87", true),
        reports,
    ];
    let mut formulas = vec![
        formula("CURRENT", tr_const!("Rated current"), "A", "round(POWER_KW * 1000 / (sqrt(3) * VOLTAGE * COS_PHI * EFFICIENCY), 1)".into()),
        formula("PROTECTION", tr_const!("Setting range of the motor protection switch"), "", step("CURRENT", &RANGES.map(|(i, r, _)| (i, quoted(r))), "\"\"")),
        formula("PROTECTION_PART", tr_const!("Motor protection switch"), "", step("CURRENT", &RANGES.map(|(i, _, p)| (i, quoted(p))), "\"\"")),
        formula("PROTECTION_MAX", tr_const!("Upper limit of the setting range"), "A", step("CURRENT", &RANGES.map(|(i, _, _)| (i, i.to_string())), "ceil(CURRENT)")),
        // IEC 60204-1 table 6, B2: the wires behind the switch carry its upper limit.
        formula(
            "CROSS_SECTION",
            tr_const!("Cross-section of the motor circuit"),
            "mm²",
            step("PROTECTION_MAX", &[("13.5", "1.5".into()), ("18.3", "2.5".into()), ("25", "4".into()), ("32", "6".into()), ("44", "10".into()), ("60", "16".into()), ("77", "25".into())], "35"),
        ),
        formula("TERMINAL_PART", tr_const!("Terminal of the motor circuit"), "", "if(CROSS_SECTION <= 2.5, \"GEN.TB-2.5\", \"\")".into()),
        formula("PE_TERMINAL_PART", tr_const!("Protective earth terminal"), "", "if(CROSS_SECTION <= 2.5, \"GEN.TB-PE-2.5\", \"\")".into()),
    ];
    formulas.extend(more);
    TemplateDefinition { name: String::new(), description: String::new(), parameters, formulas }
}

/// Project with the potentials of IEC 60204-1 and a cover sheet.
fn project(description: &str) -> Project {
    let mut prj = Project { pages: Vec::new(), ..Default::default() };
    prj.info.name = "{MOTOR}".into();
    prj.info.description = description.into();
    prj.apply_preset(crate::potentials::Preset::Iec60204);
    add_page(&mut prj, "1", tr("Cover sheet"), PageKind::Cover);
    prj
}

/// The report pages, left out with `REPORTS=no`.
fn reports(prj: &mut Project) {
    let kinds = [ReportKind::TerminalDiagram, ReportKind::ConnectionList, ReportKind::DeviceList, ReportKind::BillOfMaterials];
    for k in kinds {
        let name = (prj.pages.len() + 1).to_string();
        let at = add_page(prj, &name, k.title(), PageKind::Report(k.into()));
        prj.pages[at].condition = "REPORTS=yes".into();
    }
}

/// Device data with the parts of each tag; the parts without a placeholder are copied
/// into the project from the built-in catalog.
fn devices(prj: &mut Project, devices: &[(&str, &[&str])]) {
    let db = crate::parts::PartsDb::default();
    for (tag, parts) in devices {
        for n in parts.iter().filter(|n| !n.contains('{')) {
            crate::parts::ensure_in_project(prj, &db, n);
        }
        let parts = parts.iter().map(|n| PartRef { number: n.to_string(), count: 1, total: false }).collect();
        prj.devices.insert(tag.to_string(), DeviceData { parts, ..Default::default() });
    }
}

/// Mains rails, the supply text and the 24 V DC supply of the control circuit at `x`.
fn supply(g: &mut Pg, x: f32) {
    for (y, key) in [(24.0, "pot_l1"), (32.0, "pot_l2"), (40.0, "pot_l3"), (48.0, "pot_n"), (56.0, "pot_pe")] {
        g.sym(key, 20.0, y, 3, "", "", "").w(24.0, y, 400.0, y);
    }
    g.sym("cb1", x, 64.0, 0, "-F1", "", "").w(x, 24.0, x, 60.0).w(x, 68.0, x, 76.0);
    g.sym("psu", x, 80.0, 0, "-T1", "", "24 V DC");
    g.w(x + 8.0, 48.0, x + 8.0, 76.0).w(x + 16.0, 56.0, x + 16.0, 76.0);
    g.path(&[(x, 96.0), (x, 112.0)]).sym("intr", x, 112.0, 0, "", "+24V", "");
    g.path(&[(x + 8.0, 96.0), (x + 8.0, 104.0), (x + 24.0, 104.0), (x + 24.0, 112.0)]).sym("intr", x + 24.0, 112.0, 0, "", "0V", "");
    g.text(x - 40.0, 140.0, &tr_fmt!("Supply {} V / 50 Hz, TN-S", "{VOLTAGE}"), 3.0);
    g.text(x - 40.0, 148.0, &tr_fmt!("Rated current {} A, motor protection switch {}", "{CURRENT}", "{PROTECTION}"), 2.5);
}

/// The motor wires (behind the motor protection switch) are black, of the sized cross-section.
fn power_wires(g: &mut Pg, x: &[f32], from_y: f32) {
    for w in g.prj.pages[g.page].wires.iter_mut().filter(|w| x.contains(&w.a.x) && w.a.y >= from_y && w.b.y >= from_y) {
        w.color = "BK".into();
        w.cross_section = "{CROSS_SECTION} mm²".into();
    }
}

/// Rails of the control circuit.
fn control_rails(g: &mut Pg) {
    g.sym("intr", 20.0, 24.0, 1, "", "+24V", "").w(20.0, 24.0, 400.0, 24.0);
    g.sym("intr", 20.0, 236.0, 1, "", "0V", "").w(20.0, 236.0, 400.0, 236.0);
}

/// Parts of the motor feeder that every starter has.
const COMMON: [(&str, &[&str]); 9] = [
    ("-Q1", &["{PROTECTION_PART}", "GEN.MPS-AUX-1NO1NC"]),
    ("-X1", &["{TERMINAL_PART}", "{PE_TERMINAL_PART}"]),
    ("-F1", &["GEN.MCB-1P-C6"]),
    ("-T1", &["GEN.PS-24DC-5A"]),
    ("-S0", &["GEN.ESTOP-1NC"]),
    ("-S1", &["GEN.PB-RD-1NC"]),
    ("-S2", &["GEN.PB-GN-1NO"]),
    ("-P1", &["GEN.LMP-GN-24"]),
    ("-P2", &["GEN.LMP-GN-24"]),
];

/// Direct on line: -Q1 → -K1 → -X1 → -M1; E-stop, stop, start with self-holding.
pub(super) fn direct_on_line() -> Project {
    let mut prj = project(tr("Motor starter direct on line"));
    prj.template = definition("7.5", vec![
        formula("CONTACTOR", tr_const!("Contactor at least (AC-3)"), "A", "max(9, ceil(CURRENT))".into()),
        formula("CONTACTOR_PART", tr_const!("Contactor"), "", contactor("CURRENT")),
    ]);

    let page = add_page(&mut prj, "2", &tr_fmt!("Power circuit {}", "{MOTOR}"), PageKind::Schematic);
    let mut g = Pg { prj: &mut prj, page };
    supply(&mut g, 180.0);
    g.sym("mps3", 60.0, 80.0, 0, "-Q1", "", tr("Motor protection"));
    g.w(60.0, 24.0, 60.0, 76.0).w(68.0, 32.0, 68.0, 76.0).w(76.0, 40.0, 76.0, 76.0);
    g.sym("main3", 60.0, 112.0, 0, "-K1", "", "");
    feeder(&mut g, 84.0, 108.0);
    terminals(&mut g, &[60.0, 68.0, 76.0], 116.0, 140.0);
    g.sym("motor3", 60.0, 172.0, 0, "-M1", "", "{MOTOR}\n{POWER_KW} kW");
    power_wires(&mut g, &[60.0, 68.0, 76.0], 84.0);

    let page = add_page(&mut prj, "3", &tr_fmt!("Control circuit {}", "{MOTOR}"), PageKind::Schematic);
    let mut g = Pg { prj: &mut prj, page };
    control_rails(&mut g);
    g.sym("estop", 40.0, 40.0, 0, "-S0", "", tr("Emergency stop")).w(40.0, 24.0, 40.0, 36.0);
    g.sym("ol_nc", 40.0, 60.0, 0, "-Q1", "", "").pins(&["21", "22"]).w(40.0, 44.0, 40.0, 56.0);
    g.sym("pb_nc", 40.0, 80.0, 0, "-S1", "", tr("Stop")).w(40.0, 64.0, 40.0, 76.0);
    g.sym("pb_no", 40.0, 100.0, 0, "-S2", "", tr("Start")).w(40.0, 84.0, 40.0, 96.0);
    g.sym("no", 64.0, 100.0, 0, "-K1", "", "").pins(&["13", "14"]);
    g.path(&[(40.0, 88.0), (64.0, 88.0), (64.0, 96.0)]).path(&[(64.0, 104.0), (64.0, 112.0), (40.0, 112.0)]).w(40.0, 104.0, 40.0, 112.0);
    g.sym("coil", 40.0, 140.0, 0, "-K1", "", "{MOTOR}").w(40.0, 112.0, 40.0, 136.0).w(40.0, 144.0, 40.0, 236.0);
    g.path(&[(40.0, 120.0), (100.0, 120.0), (100.0, 136.0)]);
    g.sym("lamp", 100.0, 140.0, 0, "-P1", "", tr("Motor running")).w(100.0, 144.0, 100.0, 236.0);

    reports(&mut prj);
    let mut parts: Vec<(&str, &[&str])> = COMMON.iter().filter(|(t, _)| *t != "-P2").copied().collect();
    parts.push(("-K1", &["{CONTACTOR_PART}"]));
    devices(&mut prj, &parts);
    number_wires(&mut prj);
    prj
}

/// The three wires from the switch above (`from`) to the contactor (`to`).
fn feeder(g: &mut Pg, from: f32, to: f32) {
    for x in [60.0, 68.0, 76.0] {
        g.w(x, from, x, to);
    }
}

/// Terminals -X1 at `y` with the wires from `from` above and to the motor below
/// (the protective earth beside them, from the PE rail).
fn terminals(g: &mut Pg, x: &[f32], from: f32, y: f32) {
    for (i, x) in x.iter().enumerate() {
        g.sym("term", *x, y, 0, "-X1", &(i + 1).to_string(), "");
        g.w(*x, from, *x, y - 4.0);
    }
    g.sym("term_pe", 84.0, y, 0, "-X1", "PE", "").w(84.0, 56.0, 84.0, y - 4.0);
    for x in [60.0, 68.0, 76.0, 84.0] {
        g.w(x, y + 4.0, x, y + 28.0);
    }
}

/// Reversing: -Q1 → -K1 (right) or -K2 (left, L1 and L3 swapped) → -X1 → -M1; each
/// direction with its start button, self-holding and the NC contact of the other
/// contactor.
pub(super) fn reversing() -> Project {
    let mut prj = project(tr("Motor starter reversing"));
    prj.template = definition("4", vec![
        formula("CONTACTOR", tr_const!("Contactor at least (AC-3)"), "A", "max(9, ceil(CURRENT))".into()),
        formula("CONTACTOR_PART", tr_const!("Contactor"), "", contactor("CURRENT")),
    ]);

    let page = add_page(&mut prj, "2", &tr_fmt!("Power circuit {}", "{MOTOR}"), PageKind::Schematic);
    let mut g = Pg { prj: &mut prj, page };
    supply(&mut g, 260.0);
    g.sym("mps3", 60.0, 80.0, 0, "-Q1", "", tr("Motor protection"));
    g.w(60.0, 24.0, 60.0, 76.0).w(68.0, 32.0, 68.0, 76.0).w(76.0, 40.0, 76.0, 76.0);
    g.sym("main3", 60.0, 112.0, 0, "-K1", "", tr("Clockwise"));
    feeder(&mut g, 84.0, 108.0);
    g.path(&[(60.0, 92.0), (140.0, 92.0), (140.0, 108.0)]);
    g.path(&[(68.0, 96.0), (148.0, 96.0), (148.0, 108.0)]);
    g.path(&[(76.0, 100.0), (156.0, 100.0), (156.0, 108.0)]);
    g.sym("main3", 140.0, 112.0, 0, "-K2", "", tr("Anticlockwise"));
    // -K2 swaps L1 and L3.
    g.path(&[(140.0, 116.0), (140.0, 128.0), (76.0, 128.0)]);
    g.path(&[(148.0, 116.0), (148.0, 124.0), (68.0, 124.0)]);
    g.path(&[(156.0, 116.0), (156.0, 120.0), (60.0, 120.0)]);
    terminals(&mut g, &[60.0, 68.0, 76.0], 116.0, 140.0);
    g.sym("motor3", 60.0, 172.0, 0, "-M1", "", "{MOTOR}\n{POWER_KW} kW");
    power_wires(&mut g, &[60.0, 68.0, 76.0, 140.0, 148.0, 156.0], 84.0);
    for w in g.prj.pages[page].wires.iter_mut().filter(|w| w.a.y == w.b.y && (92.0..=128.0).contains(&w.a.y) && w.a.x.max(w.b.x) <= 156.0) {
        w.color = "BK".into();
        w.cross_section = "{CROSS_SECTION} mm²".into();
    }

    let page = add_page(&mut prj, "3", &tr_fmt!("Control circuit {}", "{MOTOR}"), PageKind::Schematic);
    let mut g = Pg { prj: &mut prj, page };
    control_rails(&mut g);
    g.sym("estop", 40.0, 40.0, 0, "-S0", "", tr("Emergency stop")).w(40.0, 24.0, 40.0, 36.0);
    g.sym("ol_nc", 40.0, 60.0, 0, "-Q1", "", "").pins(&["21", "22"]).w(40.0, 44.0, 40.0, 56.0);
    g.sym("pb_nc", 40.0, 80.0, 0, "-S1", "", tr("Stop")).w(40.0, 64.0, 40.0, 76.0);
    g.w(40.0, 84.0, 40.0, 92.0).w(40.0, 92.0, 120.0, 92.0);
    for (x, start, own, other, text, lamp) in [(40.0, "-S2", "-K1", "-K2", tr("Clockwise"), "-P1"), (120.0, "-S3", "-K2", "-K1", tr("Anticlockwise"), "-P2")] {
        g.w(x, 92.0, x, 100.0);
        g.sym("pb_no", x, 104.0, 0, start, "", text);
        g.sym("no", x + 24.0, 104.0, 0, own, "", "").pins(&["13", "14"]);
        g.path(&[(x, 96.0), (x + 24.0, 96.0), (x + 24.0, 100.0)]).path(&[(x + 24.0, 108.0), (x + 24.0, 116.0), (x, 116.0)]);
        g.w(x, 108.0, x, 116.0).w(x, 116.0, x, 128.0);
        g.sym("nc", x, 132.0, 0, other, "", "").pins(&["61", "62"]);
        g.sym("coil", x, 160.0, 0, own, "", text).w(x, 136.0, x, 156.0).w(x, 164.0, x, 236.0);
        g.path(&[(x, 144.0), (x + 40.0, 144.0), (x + 40.0, 156.0)]);
        g.sym("lamp", x + 40.0, 160.0, 0, lamp, "", text).w(x + 40.0, 164.0, x + 40.0, 236.0);
    }

    reports(&mut prj);
    let mut parts: Vec<(&str, &[&str])> = COMMON.to_vec();
    parts.push(("-S3", &["GEN.PB-GN-1NO"]));
    parts.push(("-K1", &["{CONTACTOR_PART}", "GEN.AUX-1NO1NC"]));
    parts.push(("-K2", &["{CONTACTOR_PART}", "GEN.AUX-1NO1NC"]));
    devices(&mut prj, &parts);
    number_wires(&mut prj);
    prj
}

/// Star-delta: -Q0 → -Q1 → mains contactor -K1 to U1 V1 W1, delta contactor -K3 and
/// star contactor -K2 at W2 U2 V2; the timer -K4 switches from star to delta.
pub(super) fn star_delta() -> Project {
    let mut prj = project(tr("Motor starter star-delta"));
    prj.template = definition("11", vec![
        formula("PHASE_CURRENT", tr_const!("Current of the mains and delta contactor"), "A", "round(CURRENT / sqrt(3), 1)".into()),
        formula("STAR_CURRENT", tr_const!("Current of the star contactor"), "A", "round(CURRENT / 3, 1)".into()),
        formula("CONTACTOR", tr_const!("Contactor at least (AC-3)"), "A", "max(9, ceil(PHASE_CURRENT))".into()),
        formula("CONTACTOR_PART", tr_const!("Mains and delta contactor"), "", contactor("PHASE_CURRENT")),
        formula("STAR_PART", tr_const!("Star contactor"), "", contactor("STAR_CURRENT")),
    ]);

    let page = add_page(&mut prj, "2", &tr_fmt!("Power circuit {}", "{MOTOR}"), PageKind::Schematic);
    let mut g = Pg { prj: &mut prj, page };
    supply(&mut g, 300.0);
    g.sym("disc3", 60.0, 72.0, 0, "-Q0", "", tr("Main switch"));
    g.w(60.0, 24.0, 60.0, 68.0).w(68.0, 32.0, 68.0, 68.0).w(76.0, 40.0, 76.0, 68.0);
    g.sym("mps3", 60.0, 100.0, 0, "-Q1", "", tr("Motor protection"));
    for x in [60.0, 68.0, 76.0] {
        g.w(x, 76.0, x, 96.0);
    }
    g.sym("main3", 60.0, 136.0, 0, "-K1", "", tr("Mains"));
    for x in [60.0, 68.0, 76.0] {
        g.w(x, 104.0, x, 132.0).w(x, 140.0, x, 160.0);
    }
    g.path(&[(60.0, 112.0), (140.0, 112.0), (140.0, 132.0)]);
    g.path(&[(68.0, 116.0), (148.0, 116.0), (148.0, 132.0)]);
    g.path(&[(76.0, 120.0), (156.0, 120.0), (156.0, 132.0)]);
    g.sym("main3", 140.0, 136.0, 0, "-K3", "", tr("Delta"));
    for x in [140.0, 148.0, 156.0] {
        g.w(x, 140.0, x, 160.0);
    }
    g.path(&[(140.0, 148.0), (220.0, 148.0), (220.0, 180.0)]);
    g.path(&[(148.0, 152.0), (228.0, 152.0), (228.0, 180.0)]);
    g.path(&[(156.0, 156.0), (236.0, 156.0), (236.0, 180.0)]);
    g.sym("main3", 220.0, 184.0, 0, "-K2", "", tr("Star"));
    g.path(&[(220.0, 188.0), (220.0, 196.0), (236.0, 196.0)]).w(228.0, 188.0, 228.0, 196.0).w(236.0, 188.0, 236.0, 196.0);
    for (i, x) in [60.0, 68.0, 76.0, 140.0, 148.0, 156.0].into_iter().enumerate() {
        g.sym("term", x, 164.0, 0, "-X1", &(i + 1).to_string(), "");
    }
    g.sym("term_pe", 84.0, 164.0, 0, "-X1", "PE", "").w(84.0, 56.0, 84.0, 160.0);
    for x in [60.0, 68.0, 76.0, 84.0] {
        g.w(x, 168.0, x, 192.0);
    }
    g.sym("motor3_6", 60.0, 196.0, 0, "-M1", "", "{MOTOR}\n{POWER_KW} kW");
    g.path(&[(140.0, 168.0), (140.0, 228.0), (76.0, 228.0), (76.0, 216.0)]);
    g.path(&[(148.0, 168.0), (148.0, 232.0), (60.0, 232.0), (60.0, 216.0)]);
    g.path(&[(156.0, 168.0), (156.0, 224.0), (68.0, 224.0), (68.0, 216.0)]);
    for w in g.prj.pages[page].wires.iter_mut().filter(|w| w.a.y >= 104.0 && w.a.x < 260.0 && w.a.x != 84.0) {
        w.color = "BK".into();
        w.cross_section = "{CROSS_SECTION} mm²".into();
    }

    let page = add_page(&mut prj, "3", &tr_fmt!("Control circuit {}", "{MOTOR}"), PageKind::Schematic);
    let mut g = Pg { prj: &mut prj, page };
    control_rails(&mut g);
    g.sym("estop", 40.0, 36.0, 0, "-S0", "", tr("Emergency stop")).w(40.0, 24.0, 40.0, 32.0);
    g.sym("ol_nc", 40.0, 52.0, 0, "-Q1", "", "").pins(&["21", "22"]).w(40.0, 40.0, 40.0, 48.0);
    g.sym("pb_nc", 40.0, 64.0, 0, "-S1", "", tr("Stop")).w(40.0, 56.0, 40.0, 60.0);
    g.sym("pb_no", 40.0, 88.0, 0, "-S2", "", tr("Start")).w(40.0, 68.0, 40.0, 84.0);
    g.sym("no", 64.0, 88.0, 0, "-K1", "", "").pins(&["13", "14"]);
    g.path(&[(40.0, 76.0), (64.0, 76.0), (64.0, 84.0)]).w(64.0, 92.0, 64.0, 100.0);
    g.w(40.0, 92.0, 40.0, 100.0);
    g.w(40.0, 100.0, 100.0, 100.0).w(100.0, 100.0, 140.0, 100.0).w(140.0, 100.0, 200.0, 100.0);
    g.sym("coil", 40.0, 200.0, 0, "-K1", "", tr("Mains")).w(40.0, 100.0, 40.0, 196.0).w(40.0, 204.0, 40.0, 236.0);
    g.sym("coil_on", 100.0, 200.0, 0, "-K4", "", tr("Star time")).w(100.0, 100.0, 100.0, 196.0).w(100.0, 204.0, 100.0, 236.0);
    g.sym("nc_on", 140.0, 128.0, 0, "-K4", "", "").pins(&["25", "26"]).w(140.0, 100.0, 140.0, 124.0);
    g.sym("nc", 140.0, 160.0, 0, "-K3", "", "").pins(&["61", "62"]).w(140.0, 132.0, 140.0, 156.0);
    g.sym("coil", 140.0, 200.0, 0, "-K2", "", tr("Star")).w(140.0, 164.0, 140.0, 196.0).w(140.0, 204.0, 140.0, 236.0);
    g.sym("no_on", 200.0, 128.0, 0, "-K4", "", "").pins(&["17", "18"]).w(200.0, 100.0, 200.0, 124.0);
    g.sym("nc", 200.0, 160.0, 0, "-K2", "", "").pins(&["61", "62"]).w(200.0, 132.0, 200.0, 156.0);
    g.sym("coil", 200.0, 200.0, 0, "-K3", "", tr("Delta")).w(200.0, 164.0, 200.0, 196.0).w(200.0, 204.0, 200.0, 236.0);
    g.sym("no", 270.0, 128.0, 0, "-K2", "", "").pins(&["13", "14"]).w(270.0, 24.0, 270.0, 124.0);
    g.sym("lamp", 270.0, 200.0, 0, "-P1", "", tr("Star")).w(270.0, 132.0, 270.0, 196.0).w(270.0, 204.0, 270.0, 236.0);
    g.sym("no", 330.0, 128.0, 0, "-K3", "", "").pins(&["13", "14"]).w(330.0, 24.0, 330.0, 124.0);
    g.sym("lamp", 330.0, 200.0, 0, "-P2", "", tr("Delta")).w(330.0, 132.0, 330.0, 196.0).w(330.0, 204.0, 330.0, 236.0);

    reports(&mut prj);
    let mut parts: Vec<(&str, &[&str])> = COMMON.to_vec();
    parts.push(("-Q0", &["GEN.MS-3P-40A"]));
    parts.push(("-K1", &["{CONTACTOR_PART}"]));
    parts.push(("-K2", &["{STAR_PART}", "GEN.AUX-1NO1NC"]));
    parts.push(("-K3", &["{CONTACTOR_PART}", "GEN.AUX-1NO1NC"]));
    parts.push(("-K4", &["GEN.TMR-24DC-OND-2"]));
    devices(&mut prj, &parts);
    number_wires(&mut prj);
    prj
}
