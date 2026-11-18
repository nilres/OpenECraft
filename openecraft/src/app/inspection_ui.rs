//! Inspection and test report (IEC 60364-6 / DIN VDE 0100-600) without the project
//! server: the data of the report with the visual inspection, the limits and the
//! signature, the sources of SELV, PELV and separated circuits and a section per
//! circuit with what the schematic gives (missing data marked) and the fields of the
//! results. The same report and rules as the web page of the project server
//! (`openecraft_core::inspection`); edits change the project (undo as usual).

use super::App;
use egui::collapsing_header::CollapsingState;
use egui::{Color32, RichText, TextEdit};
use openecraft_core::inspection::{self as insp, Field, HeaderField, Report, SourceField, Verdict};
use openecraft_core::model::ReportKind;
use openecraft_i18n::{tr, tr_fmt};
use openecraft_ui::icons::{self, Icon};
use std::collections::HashMap;

#[derive(Default)]
pub struct InspectionUi {
    /// Texts being typed by field (`<record>|<key>`), taken over when the field loses
    /// the focus.
    typing: HashMap<String, String>,
    filter: String,
    /// The last value that could not be taken over.
    error: Option<String>,
    /// An own item of the visual inspection is being added.
    adding: bool,
}

/// A change of the report, made after drawing.
enum Change {
    Header(String, String),
    Source(String, SourceField, String),
    Circuit(String, Field, String),
    Sign,
}

impl App {
    pub(super) fn inspection_pane(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let report = {
            let an = &self.session.analysis;
            Report::of(&self.session.prj, &an.netlist, &an.connections)
        };
        let mut changes = Vec::new();
        let (mut jump, mut excel, mut reports) = (None, false, false);
        let st = &mut self.inspection_ui;
        ui.horizontal_wrapped(|ui| {
            ui.strong(tr_fmt!("{} circuits", report.circuits.len()));
            for v in [Verdict::Failed, Verdict::Incomplete, Verdict::NotTested, Verdict::Passed] {
                let n = report.circuits.iter().filter(|c| c.verdict == v).count();
                if n > 0 {
                    ui.label(RichText::new(format!("{} {n}", tr(v.title()))).color(verdict_color(ui, v)));
                }
            }
            ui.weak(format!("U0 {} V · {}", report.u0.round(), report.earthing));
            ui.separator();
            ui.add(TextEdit::singleline(&mut st.filter).hint_text(tr("Filter circuits…")).desired_width(160.0));
            if icons::text_button(ui, Icon::Download, tr("Export Excel")).on_hover_text(tr("Download the report as an Excel workbook")).clicked() {
                excel = true;
            }
            if icons::text_button(ui, Icon::Report, tr("Reports")).on_hover_text(tr("The report as tables and report pages")).clicked() {
                reports = true;
            }
        });
        if let Some(e) = &st.error {
            ui.colored_label(crate::style::error(), e);
        }
        ui.separator();
        egui::ScrollArea::vertical().id_salt("inspection").auto_shrink([false, false]).show(ui, |ui| {
            report_card(ui, st, &report, &mut changes);
            if !report.sources.is_empty() {
                ui.add_space(6.0);
                ui.heading(tr("SELV, PELV and separation"));
                for s in &report.sources {
                    source_card(ui, st, s, &mut changes);
                }
                ui.add_space(6.0);
                ui.heading(tr("Circuits"));
            }
            let f = st.filter.trim().to_lowercase();
            let shown: Vec<&insp::Circuit> = report
                .circuits
                .iter()
                .filter(|c| f.is_empty() || c.circuit.to_lowercase().contains(&f) || c.description.as_deref().is_some_and(|d| d.to_lowercase().contains(&f)))
                .collect();
            if shown.is_empty() {
                ui.weak(if report.circuits.is_empty() { tr("The project has no circuits fed from the mains") } else { tr("No circuits found") });
            }
            for c in shown {
                circuit_card(ui, st, c, &mut changes, &mut jump);
            }
        });
        self.apply_inspection(changes, &report);
        if excel {
            let name: String = self.session.prj.info.name.chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' }).collect();
            let bytes = insp::workbook(&report);
            self.export_bytes(&ctx, format!("{}-inspection.xlsx", if name.is_empty() { "project" } else { &name }), ("Excel", "xlsx"), &bytes);
        }
        if reports {
            self.report_kind = ReportKind::InspectionSummary.into();
            self.show_reports = true;
        }
        if let Some((page, id)) = jump {
            self.goto(page, id);
        }
    }

    /// Takes the changes over into the project; a new circuit gets who tests, the date
    /// and the test voltage of the standard with its first value.
    fn apply_inspection(&mut self, changes: Vec<Change>, report: &Report) {
        if changes.is_empty() {
            return;
        }
        let who = {
            let h = &self.session.prj.inspection.header;
            if h.inspector.trim().is_empty() { self.user_name() } else { h.inspector.clone() }
        };
        let prj = &mut self.session.prj;
        let mut result = Ok(());
        for change in changes {
            let done = match change {
                Change::Header(key, value) => insp::set_header(prj, &key, &value),
                Change::Source(device, field, value) => insp::set_source(prj, &device, field, &value),
                Change::Circuit(circuit, field, value) => {
                    let new = prj.inspection.result(&circuit).is_none();
                    let done = insp::set(prj, &circuit, field, &value);
                    if new && matches!(done, Ok(true)) {
                        let today = openecraft_ui::io::today();
                        let voltage = report.insulation_voltage.round().to_string();
                        for (f, v) in [(Field::Tester, who.as_str()), (Field::Date, today.as_str()), (Field::InsulationVoltage, voltage.as_str())] {
                            if f != field && f.get(prj.inspection.result(&circuit).unwrap_or(&Default::default())).is_empty() && !v.is_empty() {
                                let _ = insp::set(prj, &circuit, f, v);
                            }
                        }
                    }
                    done
                }
                Change::Sign if who.trim().is_empty() => Err(tr("Name the inspector in the report data first.").to_string()),
                Change::Sign => {
                    let today = openecraft_ui::io::today();
                    insp::set_header_all(prj, [("signed_by", who.as_str()), ("signed_on", today.as_str())])
                }
            };
            if let Err(e) = done {
                result = Err(e);
            }
        }
        self.inspection_ui.error = result.err();
    }
}

fn verdict_color(ui: &egui::Ui, v: Verdict) -> Color32 {
    match v {
        Verdict::Passed => crate::style::ok(),
        Verdict::Failed => crate::style::error(),
        Verdict::Incomplete => crate::style::warning(),
        Verdict::NotTested => ui.visuals().weak_text_color(),
    }
}

fn badge(ui: &mut egui::Ui, v: Verdict) {
    ui.label(RichText::new(tr(v.title())).strong().color(verdict_color(ui, v)));
}

/// A fact of the schematic; what it does not give is marked as missing.
fn fact(ui: &mut egui::Ui, title: &'static str, value: Option<String>) {
    ui.label(tr(title));
    match value {
        Some(v) => ui.label(v),
        None => ui.label(RichText::new(tr("missing")).color(crate::style::warning())),
    };
    ui.end_row();
}

/// A text field taking its text over when it loses the focus; `Some` with the new text.
fn entry(ui: &mut egui::Ui, typing: &mut HashMap<String, String>, key: String, value: &str, multiline: bool, hint: &str, color: Option<Color32>) -> Option<String> {
    let mut text = typing.get(&key).cloned().unwrap_or_else(|| value.to_string());
    let mut edit = if multiline { TextEdit::multiline(&mut text).desired_rows(2) } else { TextEdit::singleline(&mut text) };
    edit = edit.desired_width(f32::INFINITY).hint_text(hint);
    if let Some(c) = color {
        edit = edit.text_color(c);
    }
    let r = ui.add(edit);
    if r.lost_focus() {
        typing.remove(&key);
        return (text != value).then_some(text);
    }
    if r.has_focus() {
        typing.insert(key, text);
    }
    None
}

/// OK, Not OK and (`na`) Not applicable; a click on the chosen one clears it.
fn answer(ui: &mut egui::Ui, value: &str, na: bool) -> Option<String> {
    let mut out = None;
    let choices = [(insp::FUNCTION_OK, tr("OK")), (insp::FUNCTION_FAILED, tr("Not OK")), (insp::NOT_APPLICABLE, tr("Not applicable"))];
    ui.horizontal(|ui| {
        for (k, title) in choices.into_iter().take(if na { 3 } else { 2 }) {
            let on = value == k;
            let text = if on && k == insp::FUNCTION_FAILED { RichText::new(title).color(crate::style::error()) } else { RichText::new(title) };
            if ui.selectable_label(on, text).clicked() {
                out = Some(if on { String::new() } else { k.to_string() });
            }
        }
    });
    out
}

fn header_field(ui: &mut egui::Ui, st: &mut InspectionUi, report: &Report, f: HeaderField, changes: &mut Vec<Change>) {
    let value = f.get(&report.header);
    let limit = report.limit(f.key());
    let bad = if f.is_answer() { value == insp::FUNCTION_FAILED } else { insp::number(value).zip(limit).is_some_and(|(v, l)| !l.passes(v)) };
    let title = RichText::new(f.title());
    ui.label(if bad { title.color(crate::style::error()) } else { title });
    let changed = if f.is_answer() {
        answer(ui, value, true)
    } else {
        let hint = match f {
            HeaderField::Reason => insp::REASONS.map(tr).join(", "),
            HeaderField::NextInspection => "YYYY-MM-DD".to_string(),
            _ => limit.map(insp::KeyLimit::text).unwrap_or_default(),
        };
        entry(ui, &mut st.typing, format!("|{}", f.key()), value, f == HeaderField::Instruments, &hint, bad.then(crate::style::error))
    };
    ui.end_row();
    if let Some(v) = changed {
        changes.push(Change::Header(f.key().into(), v));
    }
}

fn report_card(ui: &mut egui::Ui, st: &mut InspectionUi, report: &Report, changes: &mut Vec<Change>) {
    let h = &report.header;
    CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id("insp_report"), true)
        .show_header(ui, |ui| {
            ui.strong(tr("Report data and visual inspection"));
            ui.weak([h.client.as_str(), h.site.as_str()].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · "));
            if !h.signed_by.is_empty() {
                ui.label(RichText::new(tr("Signed")).color(crate::style::ok()));
            }
            badge(ui, report.verdict);
        })
        .body(|ui| {
            let text = |key: &str| -> String {
                if let Some(c) = key.strip_prefix("breaking.") {
                    return tr_fmt!("Breaking capacity of {} below the fault current at the origin", c);
                }
                if let Some(c) = report.checks.iter().find(|c| c.key == key) {
                    return if key.starts_with("own.") { c.title.clone() } else { translated(&c.title) };
                }
                HeaderField::ALL.into_iter().find(|f| f.key() == key).map_or_else(|| key.to_string(), |f| f.title().to_string())
            };
            egui::Grid::new("insp_facts").num_columns(2).show(ui, |ui| {
                fact(ui, "Supply system", Some(format!("U0 {} V · {}", report.u0.round(), report.earthing)));
                fact(ui, "Insulation test voltage (V)", Some(insp::limit_text(report.insulation_voltage)));
                let signed = if h.signed_by.is_empty() { tr("Not signed").to_string() } else { tr_fmt!("Signed by {} on {}", h.signed_by, h.signed_on) };
                fact(ui, "Signature", Some(signed));
                if !report.failed.is_empty() {
                    ui.label(tr("Not passed"));
                    ui.colored_label(crate::style::error(), report.failed.iter().map(|f| text(f)).collect::<Vec<_>>().join(", "));
                    ui.end_row();
                }
                if !report.notes.is_empty() {
                    ui.label(tr("To check"));
                    ui.colored_label(crate::style::warning(), report.notes.iter().map(|f| text(f)).collect::<Vec<_>>().join(", "));
                    ui.end_row();
                }
            });
            ui.add_space(4.0);
            egui::Grid::new("insp_header").num_columns(2).striped(true).show(ui, |ui| {
                for f in insp::REPORT_FIELDS {
                    header_field(ui, st, report, f, changes);
                }
            });
            ui.add_space(4.0);
            ui.strong(tr("Measured at the origin"));
            egui::Grid::new("insp_origin").num_columns(2).striped(true).show(ui, |ui| {
                for f in insp::ORIGIN_FIELDS {
                    header_field(ui, st, report, f, changes);
                }
            });
            if report.it {
                ui.add_space(4.0);
                ui.strong(tr("IT system"));
                egui::Grid::new("insp_it").num_columns(2).striped(true).show(ui, |ui| {
                    for f in insp::IT_FIELDS {
                        header_field(ui, st, report, f, changes);
                    }
                });
            }
            ui.add_space(4.0);
            ui.strong(tr("Visual inspection"));
            // Each item on a row of its own: the answer, then its text over the rest of
            // the width (long texts wrap there).
            let mut own = 0;
            for c in &report.checks {
                let bad = c.answer == insp::FUNCTION_FAILED;
                ui.horizontal(|ui| {
                    if let Some(v) = answer(ui, &c.answer, true) {
                        changes.push(Change::Header(c.key.clone(), v));
                    }
                    if c.key.starts_with("own.") {
                        own += 1;
                        let title = format!("{}.title", c.key);
                        if let Some(v) = entry(ui, &mut st.typing, format!("|{title}"), &c.title, false, tr("Own item of the visual inspection"), bad.then(crate::style::error)) {
                            changes.push(Change::Header(title, v));
                        }
                    } else {
                        let t = RichText::new(translated(&c.title));
                        ui.add(egui::Label::new(if bad { t.color(crate::style::error()) } else { t }).wrap());
                    }
                });
            }
            if st.adding {
                let key = format!("own.{}.title", own + 1);
                if let Some(v) = entry(ui, &mut st.typing, format!("|{key}"), "", false, tr("Own item of the visual inspection"), None) {
                    if !v.trim().is_empty() {
                        changes.push(Change::Header(key, v));
                    }
                    st.adding = false;
                }
            }
            if !st.adding && icons::text_button(ui, Icon::Plus, tr("Add an item")).clicked() {
                st.adding = true;
            }
            ui.add_space(4.0);
            ui.collapsing(tr("Limits"), |ui| {
                ui.weak(tr("Empty: the default. Set other values where the edition of the standard the installation is verified against differs."));
                egui::Grid::new("insp_limits").num_columns(3).striped(true).show(ui, |ui| {
                    for l in &report.settings {
                        ui.label(translated(&l.title));
                        let value = if (l.value - l.default).abs() > f32::EPSILON { insp::limit_text(l.value) } else { String::new() };
                        let key = format!("limit.{}", l.key);
                        ui.scope(|ui| {
                            ui.set_max_width(90.0);
                            if let Some(v) = entry(ui, &mut st.typing, format!("|{key}"), &value, false, &insp::limit_text(l.default), None) {
                                changes.push(Change::Header(key, v));
                            }
                        });
                        ui.weak(&l.source);
                        ui.end_row();
                    }
                });
            });
            ui.add_space(4.0);
            if icons::text_button(ui, Icon::Check, tr("Sign the report")).on_hover_text(tr("Changing the data or the results afterwards takes the signature away.")).clicked() {
                changes.push(Change::Sign);
            }
        });
}

fn source_card(ui: &mut egui::Ui, st: &mut InspectionUi, s: &insp::Source, changes: &mut Vec<Change>) {
    CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id(("insp_source", &s.device)), false)
        .show_header(ui, |ui| {
            ui.strong(&s.device);
            match &s.description {
                Some(d) => ui.weak(d),
                None => ui.colored_label(crate::style::warning(), tr("missing")),
            };
            badge(ui, s.verdict);
        })
        .body(|ui| {
            egui::Grid::new(("insp_source_facts", &s.device)).num_columns(2).show(ui, |ui| {
                fact(ui, "Kind", Some(if s.transformer { tr("Transformer") } else { tr("Power supply unit") }.to_string()));
                fact(ui, "Output voltage (V)", s.voltage.map(|v| format!("{} V {}", insp::limit_text(v), if s.transformer { "AC" } else { "DC" })));
                if s.failed.iter().any(|f| f == SourceField::Measure.key()) {
                    ui.label(tr("Not passed"));
                    ui.colored_label(crate::style::error(), tr_fmt!("Above {} V: no SELV or PELV", insp::limit_text(s.selv_max)));
                    ui.end_row();
                }
            });
            egui::Grid::new(("insp_source", &s.device)).num_columns(2).striped(true).show(ui, |ui| {
                for f in SourceField::ALL {
                    let value = f.get(&s.result);
                    let limit = s.limits.get(&s.result.measure).and_then(|l| l.iter().find(|l| l.key == f.key()));
                    let bad = s.failed.iter().any(|k| k == f.key());
                    let title = RichText::new(f.title());
                    ui.label(if bad { title.color(crate::style::error()) } else { title });
                    let changed = if f == SourceField::Measure {
                        let mut m = value.to_string();
                        let shown = insp::MEASURES.iter().find(|x| x.0 == m).map_or("–", |x| tr(x.1));
                        egui::ComboBox::from_id_salt(("insp_measure", &s.device)).selected_text(shown).show_ui(ui, |ui| {
                            ui.selectable_value(&mut m, String::new(), "–");
                            for (k, t) in insp::MEASURES {
                                ui.selectable_value(&mut m, k.to_string(), tr(t));
                            }
                        });
                        (m != value).then_some(m)
                    } else {
                        let hint = limit.map(insp::KeyLimit::text).unwrap_or_default();
                        entry(ui, &mut st.typing, format!("source:{}|{}", s.device, f.key()), value, f == SourceField::Remark, &hint, bad.then(crate::style::error))
                    };
                    ui.end_row();
                    if let Some(v) = changed {
                        changes.push(Change::Source(s.device.clone(), f, v));
                    }
                }
            });
        });
}

fn circuit_card(ui: &mut egui::Ui, st: &mut InspectionUi, c: &insp::Circuit, changes: &mut Vec<Change>, jump: &mut Option<(usize, u64)>) {
    CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id(("insp_circuit", &c.circuit)), false)
        .show_header(ui, |ui| {
            ui.strong(&c.circuit);
            match &c.description {
                Some(d) => ui.weak(d),
                None => ui.colored_label(crate::style::warning(), tr("missing")),
            };
            badge(ui, c.verdict);
        })
        .body(|ui| {
            egui::Grid::new(("insp_circuit_facts", &c.circuit)).num_columns(2).show(ui, |ui| {
                fact(ui, "Phases", Some(c.phases.join(" ")));
                fact(ui, "Cable", Some(if c.cables.is_empty() { "–".to_string() } else { c.cables.join(", ") }));
                fact(ui, "Cross-section", c.cross_section.clone());
                ui.label(tr("Protective device"));
                ui.horizontal(|ui| {
                    let missing = |ui: &mut egui::Ui| ui.colored_label(crate::style::warning(), tr("missing"));
                    match &c.characteristic {
                        Some(k) => ui.label(k),
                        None => missing(ui),
                    };
                    match c.rated {
                        Some(a) => ui.label(format!("{} A", insp::limit_text(a))),
                        None => missing(ui),
                    };
                });
                ui.end_row();
                ui.label(tr("RCD"));
                ui.label(c.rcd_text());
                ui.end_row();
                fact(ui, "Zs max (Ω)", c.zs_max.map(insp::limit_text));
            });
            if icons::text_button(ui, Icon::Find, tr("Show in the schematic")).clicked() {
                *jump = Some(c.target);
            }
            egui::Grid::new(("insp_circuit", &c.circuit)).num_columns(2).striped(true).show(ui, |ui| {
                for f in c.fields() {
                    let value = f.get(&c.result);
                    let color = if c.failed.contains(&f) {
                        Some(crate::style::error())
                    } else if c.notes.contains(&f) {
                        Some(crate::style::warning())
                    } else {
                        None
                    };
                    let title = RichText::new(f.title());
                    ui.label(match color {
                        Some(k) => title.color(k),
                        None => title,
                    });
                    let changed = if f.is_answer() {
                        answer(ui, value, false)
                    } else {
                        let hint = match f {
                            Field::Continuity => c.continuity_calculated.map(|r| format!("≈ {}", insp::limit_text(r))).unwrap_or_default(),
                            Field::Date => "YYYY-MM-DD".to_string(),
                            _ => c.limit(f).map(insp::Limit::text).unwrap_or_default(),
                        };
                        entry(ui, &mut st.typing, format!("{}|{}", c.circuit, f.key()), value, f == Field::Remark, &hint, color)
                    };
                    ui.end_row();
                    if let Some(v) = changed {
                        changes.push(Change::Circuit(c.circuit.clone(), f, v));
                    }
                }
            });
        });
}

/// A text of the report given in English (the items of the visual inspection, the
/// limits), in the language of the editor.
fn translated(en: &str) -> String {
    openecraft_i18n::translation(en, openecraft_i18n::lang()).map_or_else(|| en.to_string(), str::to_string)
}
