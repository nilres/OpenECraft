//! The Help pane (F1): the quick guide, the tutorial (`docs/tutorial.md`, rendered),
//! the keyboard shortcuts and the version. F1 or the pane menu › Help opens the
//! tutorial chapter of the pane in use.

use super::*;
use crate::shortcuts;

const TUTORIAL: &str = include_str!("../../../docs/tutorial.md");
const PRODUCT: &str = "OpenECraft";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Section {
    Guide,
    /// Chapter of the tutorial (index of its `##` heading; 0 is the introduction).
    Tutorial(usize),
    Shortcuts,
    About,
}

pub(super) struct HelpUi {
    section: Section,
    scroll_top: bool,
}

impl Default for HelpUi {
    fn default() -> Self {
        HelpUi { section: Section::Guide, scroll_top: false }
    }
}

/// The chapters of the tutorial: heading and lines.
fn chapters() -> Vec<(String, Vec<&'static str>)> {
    let mut out: Vec<(String, Vec<&'static str>)> = vec![(String::new(), Vec::new())];
    for line in TUTORIAL.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            out.push((h.trim().to_string(), Vec::new()));
        } else if let Some(h) = line.strip_prefix("# ") {
            out[0].0 = h.trim().to_string();
        } else if let Some(last) = out.last_mut() {
            last.1.push(line);
        }
    }
    out
}

/// The tutorial chapter of a pane (by the number in its heading).
fn chapter_of(pane: Pane) -> usize {
    use ToolPane as T;
    match pane {
        Pane::Navigator(LeftTab::Pages) => 2,
        Pane::Page | Pane::Library | Pane::Navigator(LeftTab::Devices | LeftTab::Unplaced) | Pane::Properties => 3,
        Pane::Navigator(LeftTab::Connections | LeftTab::Cables) | Pane::Tool(T::Potentials | T::Cables) => 4,
        Pane::Tool(T::Parts | T::DataPortal) => 5,
        Pane::Navigator(LeftTab::Strips) | Pane::Tool(T::TerminalStrips | T::Plugs) => 6,
        Pane::Navigator(LeftTab::Plc) | Pane::Tool(T::PlcOverview | T::PlcHardware) => 7,
        Pane::Navigator(LeftTab::Check) => 8,
        Pane::Tool(T::Reports | T::ReportBlocks | T::Tables | T::Forms) => 9,
        Pane::LayoutSpaces | Pane::LayoutView | Pane::LayoutProperties | Pane::WiringChecklist | Pane::WiringWorkbench => 10,
        Pane::Navigator(LeftTab::Review) | Pane::Tool(T::Revisions | T::Compare | T::ChangeRequests) => 12,
        Pane::Tool(T::Plugins) => 13,
        _ => 1,
    }
}

/// The chapter F1 opens: the one of the pane in use (`active`, else the document
/// `doc`); in the Help pane itself F1 keeps what it shows (`None`).
pub(super) fn key_help(active: Option<Pane>, doc: Pane) -> Option<usize> {
    match active {
        Some(Pane::Tool(ToolPane::Help)) => None,
        other => Some(chapter_of(other.unwrap_or(doc))),
    }
}

impl App {
    /// Shows the version (Help › About).
    pub(super) fn help_about(&mut self) {
        self.help.section = Section::About;
        self.help.scroll_top = true;
        self.show_pane(Pane::Tool(ToolPane::Help));
    }

    /// Opens the help at the chapter of a pane.
    pub(super) fn help_for(&mut self, pane: Pane) {
        self.help_chapter(chapter_of(pane));
    }

    /// Opens a chapter of the tutorial in the Help pane.
    pub(super) fn help_chapter(&mut self, chapter: usize) {
        self.help.section = Section::Tutorial(chapter);
        self.help.scroll_top = true;
        self.show_pane(Pane::Tool(ToolPane::Help));
    }

    pub(super) fn help_pane(&mut self, ui: &mut egui::Ui) {
        let chapters = chapters();
        egui::Panel::left("help_sections").resizable(true).default_size(220.0).show(ui, |ui| {
            egui::ScrollArea::vertical().id_salt("help_sections").auto_shrink([false, false]).show(ui, |ui| {
                for (s, title) in [(Section::Guide, tr("Quick guide")), (Section::Shortcuts, tr("Keyboard shortcuts")), (Section::About, tr("About OpenECraft"))] {
                    if ui.selectable_label(self.help.section == s, title).clicked() {
                        self.help.section = s;
                        self.help.scroll_top = true;
                    }
                }
                ui.separator();
                ui.label(RichText::new(tr("Tutorial")).strong());
                for (i, (title, _)) in chapters.iter().enumerate() {
                    let title = if i == 0 { tr("Introduction").to_string() } else { title.clone() };
                    if ui.selectable_label(self.help.section == Section::Tutorial(i), title).clicked() {
                        self.help.section = Section::Tutorial(i);
                        self.help.scroll_top = true;
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            let mut area = egui::ScrollArea::vertical().id_salt("help_text").auto_shrink([false, false]);
            if std::mem::take(&mut self.help.scroll_top) {
                area = area.vertical_scroll_offset(0.0);
            }
            area.show(ui, |ui| {
                ui.set_max_width(760.0);
                match self.help.section {
                    Section::Guide => self.quick_guide(ui),
                    Section::Tutorial(i) => {
                        if let Some((title, lines)) = chapters.get(i) {
                            ui.heading(title);
                            markdown(ui, lines);
                        }
                    }
                    Section::Shortcuts => self.shortcut_list(ui),
                    Section::About => {
                        ui.heading(PRODUCT);
                        ui.label(tr_fmt!("Version {}", env!("CARGO_PKG_VERSION")));
                        ui.label(tr("An open electrical CAE editor in the spirit of EPLAN Electric P8."));
                        ui.hyperlink("https://github.com/nilres/OpenECraft");
                    }
                }
            });
        });
    }

    /// The first hour in eight steps, each with a link to its tutorial chapter.
    fn quick_guide(&mut self, ui: &mut egui::Ui) {
        ui.label(tr("The ribbon: File opens, saves, imports, exports and prints; the tabs hold the commands by topic. Alt shows the key tips, Ctrl+Shift+P finds any command, F6 moves between the panes, F1 opens the help of the pane in use."));
        ui.add_space(6.0);
        ui.label(RichText::new(tr("The first hour")).strong());
        let sc = &self.editor.shortcuts;
        let place = tr_fmt!("3. Insert › Insert center ({}): search a symbol and click into the page to place it. Select a coil, then place contacts: they are assigned to it and numbered.", sc.cmd_text(Cmd::InsertCenter));
        let wires = tr_fmt!("4. Draw wires with the wire tool ({}); they end on connection points. Potentials and interruption points name the nets across pages.", sc.cmd_text(Cmd::WireTool));
        let steps: [(String, usize); 8] = [
            (tr("1. File › New creates a project (or open the demo from the start page).").to_string(), 1),
            (tr("2. Home › Page adds pages; the page properties give them a structure (=A1 +CAB1).").to_string(), 2),
            (place, 3),
            (wires, 4),
            (tr("5. Assign parts in the properties (Parts tab) or with Devices › Part selection.").to_string(), 5),
            (tr("6. Project data › Check: the Messages pane lists what is missing; a click shows the place.").to_string(), 8),
            (tr("7. Project data › Reports: device list, bill of materials, terminals and more, inserted as report pages.").to_string(), 9),
            (tr("8. File › Export and Print: PDF, DXF / DWG, SVG and the production data.").to_string(), 11),
        ];
        let chapters = chapters();
        let mut open = None;
        for (text, chapter) in &steps {
            ui.label(text);
            if let Some((title, _)) = chapters.get(*chapter) {
                if ui.link(tr_fmt!("Tutorial: {}", title)).clicked() {
                    open = Some(*chapter);
                }
            }
            ui.add_space(4.0);
        }
        ui.add_space(4.0);
        ui.weak(tr("The tutorial goes on with terminals, PLC, the panel layout, revisions and automation."));
        if let Some(chapter) = open {
            self.help_chapter(chapter);
        }
    }

    fn shortcut_list(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("help_keys").striped(true).show(ui, |ui| {
            let sc = &self.editor.shortcuts;
            let configured: Vec<(String, &str)> = shortcuts::binding_ids().into_iter().map(|id| (sc.text(id), tr(shortcuts::title(id)))).filter(|(k, _)| !k.is_empty()).collect();
            // The mouse gestures and modifiers as set in Keys and gestures.
            let g = &self.editor.gestures;
            let mouse = crate::gestures::MouseAction::ALL.into_iter().filter_map(|a| {
                let keys: Vec<String> = g.mouse(a).iter().map(crate::gestures::Binding::text).collect();
                (!keys.is_empty()).then(|| (format!("{} ({})", keys.join(" / "), tr(a.kind().title())), tr(a.title())))
            });
            let mods = crate::gestures::Role::ALL.into_iter().filter_map(|r| g.modifier(r).map(|m| (m.name().to_string(), tr(r.title()))));
            let fixed: Vec<(String, &str)> = [(tr("Esc"), tr("Cancel / select tool / clear selection")), ("Alt / F10", tr("Key tips of the ribbon"))]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
            for (k, v) in configured.into_iter().chain(mouse).chain(mods).chain(fixed) {
                ui.label(RichText::new(k).monospace());
                ui.label(v);
                ui.end_row();
            }
        });
        ui.add_space(8.0);
        if ui.button(tr("Keyboard shortcuts…")).clicked() {
            self.settings.open_at(settings_ui::Node::Shortcuts);
        }
    }
}

/// Renders the Markdown of the tutorial: headings, lists, paragraphs, code blocks,
/// `code` and *emphasis*.
fn markdown(ui: &mut egui::Ui, lines: &[&str]) {
    let mut para = String::new();
    let mut code = false;
    let flush = |ui: &mut egui::Ui, para: &mut String| {
        if !para.trim().is_empty() {
            ui.label(inline(ui, para.trim()));
            ui.add_space(4.0);
        }
        para.clear();
    };
    for line in lines {
        let t = line.trim_end();
        if t.trim_start().starts_with("```") {
            flush(ui, &mut para);
            code = !code;
            continue;
        }
        if code {
            ui.monospace(t);
            continue;
        }
        if let Some(h) = t.strip_prefix("### ") {
            flush(ui, &mut para);
            ui.label(RichText::new(h).strong());
        } else if t.trim_start().starts_with("- ") || starts_numbered(t) {
            flush(ui, &mut para);
            para.push_str(t.trim_start());
        } else if t.is_empty() {
            flush(ui, &mut para);
        } else {
            if !para.is_empty() {
                para.push(' ');
            }
            para.push_str(t.trim());
        }
    }
    flush(ui, &mut para);
}

fn starts_numbered(t: &str) -> bool {
    let t = t.trim_start();
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    digits > 0 && t[digits..].starts_with(". ")
}

/// A line with `code` (monospace) and *emphasis* / **strong** (italics, strong).
fn inline(ui: &egui::Ui, s: &str) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let body = egui::TextStyle::Body.resolve(ui.style());
    let mono = egui::TextStyle::Monospace.resolve(ui.style());
    let color = ui.visuals().text_color();
    let strong = ui.visuals().strong_text_color();
    let text = s.strip_prefix("- ").map(|r| format!("•  {r}")).unwrap_or_else(|| s.to_string());
    let mut code = false;
    for (i, part) in text.split('`').enumerate() {
        if i > 0 {
            code = !code;
        }
        if code {
            job.append(part, 0.0, egui::TextFormat::simple(mono.clone(), color));
            continue;
        }
        for (j, seg) in part.split('*').enumerate() {
            let fmt = if j % 2 == 1 { egui::TextFormat { italics: true, color: strong, ..egui::TextFormat::simple(body.clone(), strong) } } else { egui::TextFormat::simple(body.clone(), color) };
            job.append(seg, 0.0, fmt);
        }
    }
    job.wrap.max_width = ui.available_width();
    job
}

#[cfg(test)]
mod tests;
