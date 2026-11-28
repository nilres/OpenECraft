//! OpenECraft – an open source electrical CAE editor (schematics, device tags,
//! cross references, reports) inspired by EPLAN Electric P8.
//!
//! Runs natively on Windows, Linux and macOS and in the browser (WebAssembly).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(not(test), warn(clippy::too_many_lines))]

mod app;
mod autosave;
mod shutdown;
#[cfg(test)]
mod conventions;
mod frame_stats;
#[cfg(test)]
mod glyphs;
mod insert_center;
mod io;
mod mounting_ui;
mod plugin_commands;
mod startup;
mod stored;
mod parts_ui;
mod remote;
mod vault;
mod server_save;
mod updater;
mod wiring_ui;

use openecraft_canvas::{editor, jobs, panel_ui};
use openecraft_session::session;
use openecraft_shell::{commands, dock, gestures, pane_windows, ribbon, shortcuts, style, workspaces};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("OpenECraft")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(icon()),
        ..Default::default()
    };
    // `--wiring` opens the wiring assistant (touch screens in the workshop), another
    // argument is a project file to open (file association of the installer).
    let wiring = std::env::args().any(|a| a == "--wiring");
    let file = std::env::args().skip(1).find(|a| !a.starts_with("--"));
    eframe::run_native("OpenECraft", options, Box::new(move |cc| Ok(Box::new(app::App::new(cc).start(wiring, file)))))
}

/// Window and taskbar icon (`assets/icon.png`, generated with `installer/icon.py`).
#[cfg(not(target_arch = "wasm32"))]
fn icon() -> egui::IconData {
    let img = image::load_from_memory(include_bytes!("../../assets/icon.png")).expect("bundled icon").into_rgba8();
    egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;
    eframe::WebLogger::init(log::LevelFilter::Info).ok();
    // The language chosen before (as `App` stores it), otherwise the one of the browser.
    let stored = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(openecraft_ui::language::STORAGE_KEY).ok().flatten())
        .and_then(|code| openecraft_i18n::Lang::from_code(code.trim_matches('"')));
    let browser = || web_sys::window().and_then(|w| w.navigator().language()).and_then(|code| openecraft_i18n::Lang::from_code(&code));
    if let Some(lang) = stored.or_else(browser) {
        openecraft_i18n::set_lang(lang);
    }
    // The worker of the page (`worker.js`) has no window: it only searches routings.
    let Some(window) = web_sys::window() else { return };
    // `…/#wiring` opens the wiring assistant (tablets in the workshop).
    let wiring = window.location().hash().is_ok_and(|h| h == "#wiring");
    wasm_bindgen_futures::spawn_local(async move {
        let document = window.document().expect("no document");
        let canvas = document
            .get_element_by_id("openecraft_canvas")
            .expect("canvas missing")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("not a canvas");
        // The texts of the language before anything is shown or generated in it.
        openecraft_ui::io::fetch_language(openecraft_i18n::chosen_lang()).await;
        let result = eframe::WebRunner::new()
            .start(canvas, eframe::WebOptions::default(), Box::new(move |cc| Ok(Box::new(app::App::new(cc).start(wiring, None)))))
            .await;
        if let Some(loading) = document.get_element_by_id("loading") {
            match result {
                Ok(()) => loading.remove(),
                Err(e) => {
                    loading.set_inner_html("<p>OpenECraft failed to start. See the browser console for details.</p>");
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

/// The 3D routing of a project in a language (`openecraft_session::session::search_routing`):
/// called in the worker of the page (`worker.js`), a second instance of this module.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn search_routing(project: &str, lang: &str, prebuilt: &str) -> Result<String, wasm_bindgen::JsValue> {
    openecraft_session::session::search_routing(project, lang, prebuilt).map_err(|e| wasm_bindgen::JsValue::from_str(&e))
}

/// Whether this instance has the texts of a language (`worker.js` fetches the
/// catalog of one it lacks: `i18n/<code>.tsv`).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn language_loaded(lang: &str) -> bool {
    openecraft_i18n::Lang::from_code(lang).is_none_or(openecraft_i18n::loaded)
}

/// Hands this instance the catalog of a language (`worker.js`).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn install_language(lang: &str, catalog: String) {
    if let Some(lang) = openecraft_i18n::Lang::from_code(lang) {
        openecraft_i18n::install(lang, catalog);
    }
}
