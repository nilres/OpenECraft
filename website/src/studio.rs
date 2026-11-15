//! The studio of the website: one open project with what the editor has for it (the
//! session with undo and analysis, the plugins, the page editor and the panel layout
//! view) and the functions the page calls by name ([`Studio::call`]: examples, pages,
//! checks, reports, commands, exports, languages). No UI code; `views` draws the
//! canvases, the page (`site.js`) everything else from the JSON answers.

use openecraft_api::{params, Plugins};
use openecraft_canvas::editor::{Editor, Tool};
use openecraft_canvas::panel_ui::{has_site, PanelUi, View};
use openecraft_core::analysis::{Analysis, Cache};
use openecraft_core::model::Project;
use openecraft_core::params::{cli_words, CliWord};
use openecraft_core::parts::PartsDb;
use openecraft_core::sample;
use openecraft_i18n::{tr, tr_const, tr_fmt, Lang};
use openecraft_session::session::Session;
use serde_json::{json, Map, Value};

/// The example shown first: schematics, PLC and a panel layout for the 3D view.
pub const FIRST_EXAMPLE: &str = "conveyor";

/// At most this many rows of a report are sent to the page.
pub const REPORT_ROWS: usize = 250;

/// Symbols the page offers for placing: name on the page, key of the built-in symbol.
pub const SYMBOLS: [(&str, &str); 6] = [("no", "no"), ("nc", "nc"), ("coil", "coil"), ("lamp", "lamp"), ("pb_no", "pb_no"), ("estop", "estop")];

/// The groups the page shows the exports in, each with the exporters it holds (the
/// last one takes the rest).
pub const EXPORT_GROUPS: [(&str, &[&str]); 8] = [
    (tr_const!("Documents"), &["html", "pdf", "dictionary", "device_tags"]),
    (tr_const!("Drawings"), &["dxf", "dwg"]),
    (tr_const!("3D models"), &["gltf", "step", "iges", "ifc", "jt", "parasolid"]),
    (tr_const!("PLC and automation"), &["plc_table", "plc_variables", "plc_tia", "plc_rockwell", "plc_step7", "plc_twincat", "plc_mitsubishi", "plc_schneider", "aml"]),
    (tr_const!("Wiring and harnesses"), &["wire_list", "pinouts", "kbl", "kbl_zip", "harness_json", "harness_processing", "harness_gltf", "harness_jt"]),
    (tr_const!("Manufacturing"), &["marking", "marking_xlsx", "manufacturing_package", "copper_bar_program"]),
    (tr_const!("Planning"), &["preplanning_csv", "preplanning_xlsx"]),
    (tr_const!("Other"), &[]),
];

pub struct Studio {
    pub session: Session,
    pub plugins: Plugins,
    pub editor: Editor,
    pub panel: PanelUi,
    pub parts: PartsDb,
    /// Key of the example that is open.
    pub example: &'static str,
    /// Last message of the page editor.
    pub message: String,
    /// The egui contexts of the canvases: repainted after a change from the page.
    pub contexts: Vec<egui::Context>,
    /// The page has a worker that searches the 3D routing ([`search_routing`]): a
    /// project that is opened waits for it instead of searching here, where the
    /// search would stop the page.
    pub worker: bool,
    /// Number of the search the worker is asked for; the answer to an earlier one
    /// is dropped.
    pub search: u64,
}

impl Default for Studio {
    fn default() -> Self {
        let example = sample::EXAMPLES.iter().find(|e| e.key == FIRST_EXAMPLE).unwrap_or(&sample::EXAMPLES[0]);
        let mut panel = PanelUi::default();
        panel.view = View::ThreeD;
        Studio {
            session: Session::new((example.build)(), None),
            plugins: Plugins::new(Vec::<String>::new()),
            editor: Editor::default(),
            panel,
            parts: PartsDb::default(),
            example: example.key,
            message: String::new(),
            contexts: Vec::new(),
            worker: false,
            search: 0,
        }
    }
}

impl Studio {
    /// Opens an example project of the gallery.
    pub fn open(&mut self, key: &str) -> Result<(), String> {
        let example = sample::EXAMPLES.iter().find(|e| e.key == key).ok_or_else(|| tr_fmt!("Unknown example: {}", key))?;
        self.session.replace((example.build)(), None, None);
        self.example = example.key;
        self.editor.selection.clear();
        self.editor.set_tool(Tool::Select);
        self.editor.fit_pending = true;
        let view = self.panel.view;
        self.panel = PanelUi::default();
        self.panel.view = view;
        self.message.clear();
        self.ask_worker();
        Ok(())
    }

    /// Has the worker of the page search the 3D routing of the project as it is now
    /// (`search` of [`Studio::state`], `project`, then `routed`); the project counts
    /// as loading until then. Without a worker the analysis searches it.
    fn ask_worker(&mut self) {
        if self.worker {
            self.search += 1;
            self.session.await_routing();
        }
    }

    /// What the worker gets for a search: its number, the language and the project,
    /// and the example the project is while nothing was changed in it (the page has
    /// the routings of the examples from its build, `routing/`).
    fn project(&self) -> Value {
        let example = (!self.session.history.can_undo()).then_some(self.example);
        json!({ "search": self.search, "lang": openecraft_i18n::lang().code(), "example": example, "project": serde_json::to_string(&self.session.prj).unwrap_or_default() })
    }

    /// The answer of the worker: the number of the search, then in a line of its own
    /// the routing it found ([`search_routing`]; none: it failed, the analysis
    /// searches here).
    pub fn routed(&mut self, answer: &str) {
        let (search, searched) = answer.split_once('\n').unwrap_or((answer, ""));
        if search.parse() == Ok(self.search) {
            self.session.routing_found((!searched.is_empty()).then_some(searched));
            self.tick();
        }
    }

    /// Commits finished changes and brings the analysis up to date (the canvases do
    /// the same every frame; the page calls it while none of them is drawn).
    pub fn tick(&mut self) {
        self.session.commit(!self.editor.busy() && !self.panel.busy());
        self.session.refresh(&self.plugins, false);
    }

    /// Waits for the analysis of a project just opened (tests, the first answer).
    pub fn settle(&mut self) {
        if self.session.awaits_routing() {
            self.search += 1;
            self.session.routing_found(None);
        }
        self.tick();
        while self.session.opening().is_some() {
            std::thread::yield_now();
            self.tick();
        }
    }

    pub fn repaint(&self) {
        for ctx in &self.contexts {
            ctx.request_repaint();
        }
    }

    /// Shows the page with `index` in the page editor.
    pub fn show_page(&mut self, index: usize) {
        if index < self.session.prj.pages.len() && index != self.session.page {
            self.session.page = index;
            self.editor.selection.clear();
            self.editor.fit_pending = true;
        }
    }

    /// The tool of the page editor by its name on the page.
    pub fn set_tool(&mut self, name: &str) -> Result<(), String> {
        let tool = match name {
            "select" => Tool::Select,
            "wire" => Tool::Wire,
            "text" => Tool::Text,
            "rect" => Tool::Rect,
            other => Tool::Place(SYMBOLS.iter().find(|(n, _)| *n == other).ok_or_else(|| tr_fmt!("Unknown tool: {}", other))?.1),
        };
        self.editor.set_tool(tool);
        Ok(())
    }

    /// Shows a place of the panel layout view: `site` for the whole plant, otherwise
    /// the index of an enclosure or room (`places` of [`Studio::state`]).
    pub fn show_place(&mut self, place: &str) -> Result<(), String> {
        let layout = &self.session.prj.layout;
        if place == "site" && has_site(layout) {
            self.panel.show_site();
            return Ok(());
        }
        let index = place.parse().ok().filter(|i| *i < layout.enclosures.len()).ok_or_else(|| tr_fmt!("Unknown enclosure: {}", place))?;
        self.panel.show_enclosure(&self.session.prj, index);
        Ok(())
    }

    /// The 3D view, or the drawing of the place shown: the mounting plate of an
    /// enclosure, the floor plan of a room, the site plan of the site.
    pub fn set_view(&mut self, three_d: bool) {
        let room = self.session.prj.layout.enclosures.get(self.panel.enclosure).is_some_and(|e| e.is_room());
        self.panel.view = match three_d {
            true => View::ThreeD,
            false if self.panel.site || room => View::Plan,
            false => View::Front,
        };
        self.panel.fit();
    }

    /// What the panel layout view shows: `site`, `room` or `enclosure`.
    fn place_kind(&self) -> &'static str {
        match self.session.prj.layout.enclosures.get(self.panel.enclosure) {
            _ if self.panel.site => "site",
            Some(e) if e.is_room() => "room",
            _ => "enclosure",
        }
    }

    fn tool_name(&self) -> &'static str {
        match &self.editor.tool {
            Tool::Wire => "wire",
            Tool::Text => "text",
            Tool::Rect => "rect",
            Tool::Place(key) => SYMBOLS.iter().find(|(_, k)| k == key).map_or("select", |(n, _)| n),
            _ => "select",
        }
    }

    /// What the page shows of the project: pages, check messages, counts and the
    /// state of the tools. The page asks for it regularly and redraws on a change.
    pub fn state(&self) -> Value {
        let s = &self.session;
        let view = openecraft_view::project_view(&s.prj, &s.index, openecraft_i18n::lang());
        let pages: Vec<Value> = view.pages.iter().map(|p| json!({ "name": p.full_name, "description": p.description, "icon": p.icon })).collect();
        let issues: Vec<Value> = s
            .analysis
            .issues
            .iter()
            .filter(|i| !i.accepted)
            .map(|i| json!({ "severity": i.severity.key(), "message": i.message(), "page": i.target.map(|t| t.0), "item": i.target.map(|t| t.1) }))
            .collect();
        let places: Vec<Value> = s.prj.layout.enclosures.iter().map(|e| json!({ "name": e.name, "room": e.is_room() })).collect();
        let title = sample::EXAMPLES.iter().find(|e| e.key == self.example).map_or("", |e| plain_title(tr(e.title)));
        json!({
            "example": self.example,
            "title": title,
            "page": s.page,
            "pages": pages,
            "issues": issues,
            "devices": s.index.devices.len(),
            "connections": s.analysis.connections.len(),
            "wires": s.analysis.wiring.len(),
            "undo": s.history.can_undo(),
            "redo": s.history.can_redo(),
            "loading": s.opening().is_some(),
            "search": s.awaits_routing().then_some(self.search),
            "tool": self.tool_name(),
            "view": if self.panel.view == View::ThreeD { "3d" } else { "front" },
            "show_wires": self.panel.show_wires,
            "places": places,
            "site": has_site(&s.prj.layout),
            "place": if self.panel.site { json!("site") } else { json!(self.panel.enclosure.min(s.prj.layout.enclosures.len().saturating_sub(1))) },
            "place_kind": self.place_kind(),
            "hint": self.editor.hint(),
            "message": self.message,
            "lang": openecraft_i18n::lang().code(),
        })
    }

    /// Jumps to the place of a check message (`issues` of [`Studio::state`]).
    pub fn goto_issue(&mut self, index: usize) {
        let Some((page, item)) = self.session.analysis.issues.iter().filter(|i| !i.accepted).nth(index).and_then(|i| i.target) else { return };
        self.show_page(page);
        self.editor.set_tool(Tool::Select);
        self.editor.selection = [item].into();
    }

    /// The table of a report of the plugins.
    pub fn report(&self, name: &str) -> Result<Value, String> {
        self.loaded()?;
        let report = self.plugins.report(name).ok_or_else(|| tr_fmt!("Unknown report: {}", name))?;
        let table = (report.run)(&self.session.prj, &self.session.analysis)?;
        let headers: Vec<&str> = table.headers.iter().map(params::text).collect();
        Ok(json!({
            "title": params::text(&report.title),
            "headers": headers,
            "rows": table.rows.iter().take(REPORT_ROWS).collect::<Vec<_>>(),
            "total": table.rows.len(),
        }))
    }

    /// Runs a line of the console: a command of the plugins with `--name value`
    /// parameters, as on the command line. Returns its answer as JSON text.
    pub fn run(&mut self, line: &str) -> Result<String, String> {
        let mut name = None;
        let mut pairs = Vec::new();
        for word in cli_words(&split_line(line)?, &[], &[]) {
            match word {
                CliWord::Plain(w) if name.is_none() => name = Some(w),
                CliWord::Plain(w) => return Err(tr_fmt!("Unexpected argument: {}", w)),
                CliWord::Pair(n, v) => pairs.push((n.replace('-', "_"), v)),
            }
        }
        let name = name.ok_or_else(|| tr("Type a command, for example list_pages").to_string())?;
        let command = self.plugins.command(&name).cloned().ok_or_else(|| tr_fmt!("Unknown command: {}", name))?;
        // Renamed parameters are taken by their old names as well, as on the command line.
        let pairs: Vec<_> = pairs.into_iter().map(|(k, v)| (openecraft_api::builtin::renamed_param(&name, &k).map_or(k.clone(), String::from), v)).collect();
        let args = params::from_cli(&command.params, None, &pairs)?;
        let answer = self.plugins.run(&command, &mut self.session.prj, &args)?;
        self.session.clamp_page();
        // The answers that follow (state, reports) are of the project as the command left it.
        self.settle();
        Ok(serde_json::to_string_pretty(&answer).unwrap_or_default())
    }

    /// The file of an exporter of the plugins: its name and content.
    pub fn export(&self, name: &str) -> Result<(String, Vec<u8>), String> {
        self.loaded()?;
        let exporter = self.plugins.exporter(name).ok_or_else(|| tr_fmt!("Unknown export: {}", name))?;
        let file = (exporter.run)(&self.session.prj, &self.session.analysis, &params::Args(&Map::new()))?;
        Ok((format!("{}.{}", self.example, exporter.extension), file.into_bytes()))
    }

    /// Reports and exports are of the analysis: not while it is built.
    fn loaded(&self) -> Result<(), String> {
        match self.session.opening() {
            Some(_) => Err(tr("Loading…").to_string()),
            None => Ok(()),
        }
    }

    /// The exports the page offers (those that need no parameters), in their groups.
    fn exporters(&self) -> Value {
        let group = |name: &str| EXPORT_GROUPS.iter().position(|g| g.1.contains(&name)).unwrap_or(EXPORT_GROUPS.len() - 1);
        let mut exporters: Vec<_> = self.plugins.exporters().filter(|e| e.params.iter().all(|p| !p.required)).collect();
        exporters.sort_by_key(|e| group(&e.name));
        exporters.iter().map(|e| json!({ "name": e.name, "title": params::text(&e.title), "extension": e.extension, "group": tr(EXPORT_GROUPS[group(&e.name)].0) })).collect()
    }

    /// Switches the language of the canvases and of everything generated.
    pub fn set_language(&mut self, code: &str) -> Result<(), String> {
        let lang = Lang::from_code(code).ok_or_else(|| tr_fmt!("Unknown language: {}", code))?;
        openecraft_i18n::set_lang(lang);
        for ctx in &self.contexts {
            openecraft_ui::fonts::follow_language(ctx);
        }
        // The 3D routing is searched again in the new language.
        self.ask_worker();
        self.tick();
        Ok(())
    }

    /// A function of the page by its name, with its argument. The answer is JSON;
    /// an error is `{"error": "…"}`.
    pub fn call(&mut self, method: &str, arg: &str) -> Value {
        let done = |r: Result<(), String>| r.map(|()| Value::Null);
        let result = match method {
            "state" => {
                self.tick();
                Ok(self.state())
            }
            "examples" => Ok(sample::EXAMPLES.iter().zip(with_layout()).map(|(e, layout)| json!({ "key": e.key, "level": e.level, "title": plain_title(tr(e.title)), "description": tr(e.description), "layout": layout })).collect()),
            "open" => done(self.open(arg)),
            "page" => done(arg.parse().map(|i| self.show_page(i)).map_err(|_| tr_fmt!("Unknown page: {}", arg))),
            "issue" => done(arg.parse().map(|i| self.goto_issue(i)).map_err(|_| tr_fmt!("Unknown page: {}", arg))),
            "undo" | "redo" => {
                match method {
                    "undo" => self.session.undo(),
                    _ => self.session.redo(),
                };
                self.session.clamp_page();
                // A search under way is of the project as it was.
                if self.session.awaits_routing() {
                    self.search += 1;
                }
                Ok(Value::Null)
            }
            "tool" => done(self.set_tool(arg)),
            "fit" => {
                self.editor.fit_pending = true;
                self.panel.fit();
                Ok(Value::Null)
            }
            "view" => {
                self.set_view(arg != "front");
                Ok(Value::Null)
            }
            "place" => done(self.show_place(arg)),
            "wires" => {
                self.panel.show_wires = !self.panel.show_wires;
                Ok(Value::Null)
            }
            "reports" => Ok(self.plugins.reports().map(|r| json!({ "name": r.name, "title": params::text(&r.title) })).collect()),
            "report" => self.report(arg),
            "commands" => Ok(self.plugins.commands().map(command_help).collect()),
            "run" => self.run(arg).map(Value::String),
            "exporters" => Ok(self.exporters()),
            "worker" => {
                self.worker = arg == "on";
                Ok(Value::Null)
            }
            "project" => Ok(self.project()),
            "routing" => search_routing(&serde_json::to_string(&self.session.prj).unwrap_or_default(), openecraft_i18n::lang().code(), "").map(Value::String),
            "routed" => {
                self.routed(arg);
                Ok(Value::Null)
            }
            "languages" => Ok(Lang::ALL.iter().map(|l| json!({ "code": l.code(), "name": l.native_name() })).collect()),
            "language" => done(self.set_language(arg)),
            "texts" => Ok(crate::texts::translations(openecraft_i18n::lang())),
            "svg" => arg
                .parse()
                .ok()
                .filter(|i| *i < self.session.prj.pages.len())
                .map(|i| Value::String(openecraft_core::svg::page_svg_with(&self.session.prj, &self.session.analysis, i)))
                .ok_or_else(|| tr_fmt!("Unknown page: {}", arg)),
            other => Err(tr_fmt!("Unknown command: {}", other)),
        };
        self.repaint();
        result.unwrap_or_else(|e| json!({ "error": e }))
    }
}

/// The 3D routing of a project (as JSON) in a language, for the analysis of the same
/// project elsewhere ([`Studio::routed`]). The worker of the page runs it.
/// `prebuilt`: a routing of the project from the build of the page (none: empty); it
/// is taken instead of a search if it is of the wiring of the project, which is
/// seen here and not where the page is drawn.
pub fn search_routing(project: &str, lang: &str, prebuilt: &str) -> Result<String, String> {
    let prj: Project = serde_json::from_str(project).map_err(|e| e.to_string())?;
    let lang = Lang::from_code(lang).ok_or_else(|| tr_fmt!("Unknown language: {}", lang))?;
    let mut cache = Cache::default();
    if !prebuilt.is_empty() {
        // One that cannot be read is searched.
        let _ = cache.adopt_routing(prebuilt);
    }
    openecraft_i18n::with_lang(lang, || Analysis::build_with(&prj, &mut cache));
    cache.routing_searched().ok_or_else(|| tr("Loading…").to_string())
}

/// The title of an example without its number in the gallery of the editor
/// ("4 · Conveyor …"): the page shows the examples as chips, not as a numbered list.
pub fn plain_title(title: &str) -> &str {
    let rest = title.trim_start_matches(|c: char| c.is_ascii_digit());
    match rest.len() < title.len() {
        true => rest.trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '·' | '.' | ':' | '-' | '–')),
        false => title,
    }
}

/// Per example: it has an enclosure or a room, something for the panel layout view.
fn with_layout() -> &'static [bool] {
    static LAYOUT: std::sync::OnceLock<Vec<bool>> = std::sync::OnceLock::new();
    LAYOUT.get_or_init(|| sample::EXAMPLES.iter().map(|e| !(e.build)().layout.enclosures.is_empty()).collect())
}

/// A command for the help of the console: name, help text and parameters.
fn command_help(c: &openecraft_api::Command) -> Value {
    let params: Vec<Value> = c.params.iter().map(|p| json!({ "name": p.name, "hint": p.kind.hint(), "required": p.required, "help": params::text(&p.help) })).collect();
    json!({ "name": c.name, "help": params::text(&c.help), "params": params })
}

/// The words of a typed line: separated by blanks, a part in single or double quotes
/// is taken as it is.
pub fn split_line(line: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quote: Option<char> = None;
    for c in line.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => word.push(c),
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                started = true;
            }
            None if c.is_whitespace() => {
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            None => {
                word.push(c);
                started = true;
            }
        }
    }
    if quote.is_some() {
        return Err(tr("A quote is not closed").to_string());
    }
    if started {
        words.push(word);
    }
    Ok(words)
}

#[cfg(test)]
mod tests;
