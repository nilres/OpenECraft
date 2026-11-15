//! The OpenECraft website. The page (`index.html`, `site.css`, `site.js`) presents the
//! project; its live demos are the editor's own code compiled to WebAssembly: the
//! page editor and the panel layout view draw into canvases of the page (`views`), and
//! the page asks the [`studio::Studio`] for pages, checks, reports, commands, exports
//! and translations.
//!
//! Built with trunk for the browser (`cd website && trunk serve`). The native build is
//! for working on the demos: it shows the canvases in a window (`schematic` or
//! `layout` as argument: only that one), `call <function> [argument]` prints an answer
//! of the studio as the page gets it, `export <exporter>` writes the file of an export,
//! `sheet` prints the still picture of the first screen (`sheet.svg`).

#[cfg(test)]
mod links;
#[cfg(test)]
mod seo;
mod studio;
mod texts;
mod views;

/// The page the demo opens with as SVG, in English: the still picture the page shows
/// until the module is loaded (`sheet.svg`; `… -- sheet > website/sheet.svg`).
#[cfg(not(target_arch = "wasm32"))]
fn first_sheet() -> String {
    openecraft_i18n::with_lang(openecraft_i18n::Lang::En, || {
        let mut studio = studio::Studio::default();
        studio.settle();
        let page = studio.call("state", "")["page"].as_u64().unwrap_or_default();
        studio.call("svg", &page.to_string()).as_str().unwrap_or_default().to_string()
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Both canvases side by side on one project, or the one named as argument.
    struct Window(Rc<RefCell<studio::Studio>>, Option<views::Kind>);

    impl eframe::App for Window {
        fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
            let gl = frame.gl().is_some();
            let studio = &mut self.0.borrow_mut();
            if let Some(kind) = self.1 {
                views::show(ui, kind, studio, gl);
                return;
            }
            let half = ui.available_width() / 2.0;
            egui::Panel::left("schematic").exact_size(half).show(ui, |ui| views::show(ui, views::Kind::Schematic, studio, gl));
            views::show(ui, views::Kind::Layout, studio, gl);
        }
    }

    env_logger::init();
    openecraft_i18n::init(None);
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["call", method, ref rest @ ..] => {
            let mut studio = studio::Studio::default();
            studio.settle();
            println!("{:#}", studio.call(method, rest.first().copied().unwrap_or_default()));
            return Ok(());
        }
        ["export", name] => {
            let mut studio = studio::Studio::default();
            studio.settle();
            match studio.export(name).and_then(|(file, bytes)| std::fs::write(&file, bytes).map(|()| file).map_err(|e| openecraft_i18n::io_error(&e))) {
                Ok(file) => println!("{file}"),
                Err(e) => eprintln!("{e}"),
            }
            return Ok(());
        }
        ["sheet"] => {
            print!("{}", first_sheet());
            return Ok(());
        }
        _ => {}
    }
    let options = eframe::NativeOptions { viewport: egui::ViewportBuilder::default().with_title("OpenECraft").with_inner_size([1440.0, 800.0]), ..Default::default() };
    eframe::run_native(
        "OpenECraft website",
        options,
        Box::new(move |cc| {
            views::setup(&cc.egui_ctx);
            let studio = Rc::new(RefCell::new(studio::Studio::default()));
            studio.borrow_mut().contexts.push(cc.egui_ctx.clone());
            Ok(Box::new(Window(studio, args.first().and_then(|a| views::Kind::from_name(a)))))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Info).ok();
    if let Some(lang) = web_sys::window().and_then(|w| w.navigator().language()).and_then(|code| openecraft_i18n::Lang::from_code(&code)) {
        openecraft_i18n::set_lang(lang);
    }
}

/// What the page calls: the module, which `site.js` loads once the page is shown.
#[cfg(target_arch = "wasm32")]
mod web {
    use crate::studio::Studio;
    use crate::views::{self, Kind};
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::prelude::*;

    thread_local! {
        static STUDIO: Rc<RefCell<Studio>> = Rc::new(RefCell::new(Studio::default()));
    }

    struct DemoApp {
        kind: Kind,
        studio: Rc<RefCell<Studio>>,
    }

    impl DemoApp {
        fn new(cc: &eframe::CreationContext<'_>, kind: Kind, studio: Rc<RefCell<Studio>>) -> DemoApp {
            views::setup(&cc.egui_ctx);
            studio.borrow_mut().contexts.push(cc.egui_ctx.clone());
            DemoApp { kind, studio }
        }
    }

    impl eframe::App for DemoApp {
        fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
            let gl = frame.gl().is_some();
            views::show(ui, self.kind, &mut self.studio.borrow_mut(), gl);
        }
    }

    /// A function of the studio by its name (`Studio::call`); the answer is JSON.
    /// A canvas that is drawing answers `{"busy": true}`: the page asks again.
    #[wasm_bindgen]
    pub fn call(method: &str, arg: &str) -> String {
        STUDIO.with(|s| match s.try_borrow_mut() {
            Ok(mut studio) => studio.call(method, arg).to_string(),
            Err(_) => "{\"busy\":true}".to_string(),
        })
    }

    /// The file of an exporter; its name is `<example>.<extension of the exporter>`.
    /// Not `export`: wasm-bindgen renames a function with the name of a JavaScript keyword.
    #[wasm_bindgen]
    pub fn export_file(name: &str) -> Result<Vec<u8>, JsValue> {
        STUDIO.with(|s| s.borrow().export(name)).map(|(_, bytes)| bytes).map_err(|e| JsValue::from_str(&e))
    }

    /// The 3D routing of a project in a language (`studio::search_routing`): called
    /// in the worker of the page, a second instance of this module.
    #[wasm_bindgen]
    pub fn search_routing(project: &str, lang: &str, prebuilt: &str) -> Result<String, JsValue> {
        crate::studio::search_routing(project, lang, prebuilt).map_err(|e| JsValue::from_str(&e))
    }

    /// Starts a view (`schematic`, `layout`) in the canvas with the id.
    #[wasm_bindgen]
    pub fn start(canvas_id: &str, view: &str) -> Result<(), JsValue> {
        use wasm_bindgen::JsCast;
        let kind = Kind::from_name(view).ok_or_else(|| JsValue::from_str("unknown view"))?;
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(canvas_id))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .ok_or_else(|| JsValue::from_str("canvas missing"))?;
        let studio = STUDIO.with(Rc::clone);
        wasm_bindgen_futures::spawn_local(async move {
            let started = eframe::WebRunner::new().start(canvas, eframe::WebOptions::default(), Box::new(move |cc| Ok(Box::new(DemoApp::new(cc, kind, studio))))).await;
            if let Err(e) = started {
                log::error!("The view failed to start: {e:?}");
            }
        });
        Ok(())
    }
}
