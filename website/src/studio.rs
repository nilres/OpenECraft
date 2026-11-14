//! The studio of the website: one open project with what the editor has for it (the
//! session with undo and analysis, the plugins, the page editor and the panel layout
//! view) and the functions the page calls by name ([`Studio::call`]: examples, pages,
//! checks, reports, commands, exports, languages). No UI code; `views` draws the
//! canvases, the page (`site.js`) everything else from the JSON answers.

use openecraft_api::{params, Plugins};
use openecraft_canvas::editor::{Editor, Tool};
use openecraft_canvas::panel_ui::{PanelUi, View};
use openecraft_core::params::{cli_words, CliWord};
use openecraft_core::parts::PartsDb;
use openecraft_core::sample;
use openecraft_i18n::{tr, tr_fmt, Lang};
use openecraft_session::session::Session;
use serde_json::{json, Map, Value};

/// The example shown first: schematics, PLC and a panel layout for the 3D view.
pub const FIRST_EXAMPLE: &str = "conveyor";

/// At most this many rows of a report are sent to the page.
pub const REPORT_ROWS: usize = 250;

/// Symbols the page offers for placing: name on the page, key of the built-in symbol.
pub const SYMBOLS: [(&str, &str); 6] = [("no", "no"), ("nc", "nc"), ("coil", "coil"), ("lamp", "lamp"), ("pb_no", "pb_no"), ("estop", "estop")];

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
        Ok(())
    }

    /// Commits finished changes and brings the analysis up to date (the canvases do
    /// the same every frame; the page calls it while none of them is drawn).
    pub fn tick(&mut self) {
        self.session.commit(!self.editor.busy() && !self.panel.busy());
        self.session.refresh(&self.plugins, false);
    }

    /// Waits for the analysis of a project just opened (tests, the first answer).
    pub fn settle(&mut self) {
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
        let title = sample::EXAMPLES.iter().find(|e| e.key == self.example).map_or("", |e| tr(e.title));
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
            "tool": self.tool_name(),
            "view": if self.panel.view == View::ThreeD { "3d" } else { "front" },
            "door": self.panel.door_open,
            "show_wires": self.panel.show_wires,
            "enclosures": s.prj.layout.enclosures.len(),
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
        let exporter = self.plugins.exporter(name).ok_or_else(|| tr_fmt!("Unknown export: {}", name))?;
        let file = (exporter.run)(&self.session.prj, &self.session.analysis, &params::Args(&Map::new()))?;
        Ok((format!("{}.{}", self.example, exporter.extension), file.into_bytes()))
    }

    /// Switches the language of the canvases and of everything generated.
    pub fn set_language(&mut self, code: &str) -> Result<(), String> {
        let lang = Lang::from_code(code).ok_or_else(|| tr_fmt!("Unknown language: {}", code))?;
        openecraft_i18n::set_lang(lang);
        for ctx in &self.contexts {
            openecraft_ui::fonts::follow_language(ctx);
        }
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
            "examples" => Ok(sample::EXAMPLES.iter().zip(with_layout()).map(|(e, layout)| json!({ "key": e.key, "level": e.level, "title": tr(e.title), "description": tr(e.description), "layout": layout })).collect()),
            "open" => done(self.open(arg)),
            "page" => done(arg.parse().map(|i| self.show_page(i)).map_err(|_| tr_fmt!("Unknown page: {}", arg))),
            "issue" => done(arg.parse().map(|i| self.goto_issue(i)).map_err(|_| tr_fmt!("Unknown page: {}", arg))),
            "undo" => {
                self.session.undo();
                self.session.clamp_page();
                Ok(Value::Null)
            }
            "redo" => {
                self.session.redo();
                self.session.clamp_page();
                Ok(Value::Null)
            }
            "tool" => done(self.set_tool(arg)),
            "fit" => {
                self.editor.fit_pending = true;
                self.panel.fit();
                Ok(Value::Null)
            }
            "view" => {
                self.panel.view = if arg == "front" { View::Front } else { View::ThreeD };
                Ok(Value::Null)
            }
            "door" => {
                self.panel.door_open = !self.panel.door_open;
                Ok(Value::Null)
            }
            "wires" => {
                self.panel.show_wires = !self.panel.show_wires;
                Ok(Value::Null)
            }
            "reports" => Ok(self.plugins.reports().map(|r| json!({ "name": r.name, "title": params::text(&r.title) })).collect()),
            "report" => self.report(arg),
            "commands" => Ok(self.plugins.commands().map(command_help).collect()),
            "run" => self.run(arg).map(Value::String),
            "exporters" => Ok(self.plugins.exporters().filter(|e| e.params.iter().all(|p| !p.required)).map(|e| json!({ "name": e.name, "title": params::text(&e.title), "extension": e.extension })).collect()),
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
