use super::*;
use openecraft_core::sample::demo_project;

/// Empty directory for the files of one test.
fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("openecraft-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn split_projects_can_be_opened_like_project_files() {
    for purpose in [Purpose::Project, Purpose::ProjectTemplate, Purpose::Compare, Purpose::Subproject] {
        assert!(purpose.filter().1.contains(&INDEX_EXT), "{purpose:?}");
    }
    assert!(is_split("plant/plant.oecp") && is_split("PLANT.OECP"));
    assert!(!is_split("plant.oec") && !is_split("plant.oecz"));
}

#[test]
fn projects_are_read_and_written_by_the_kind_of_their_file() {
    let dir = temp_dir("io-projects");
    let io = FileIo::default();
    let prj = demo_project();

    let file = dir.join("demo.oec").display().to_string();
    assert_eq!(write_project(&io, &file, &prj, &mut None), Ok(0));
    let bytes = std::fs::read(&file).unwrap();
    let (back, _, archive) = read_project(&file, &bytes).unwrap();
    assert!(back == prj && !archive);

    // A split project is read from the folder of its index, whatever was loaded.
    let index = dir.join("split").join("demo.oecp").display().to_string();
    let mut tracked = None;
    assert_eq!(write_project(&io, &index, &prj, &mut tracked), Ok(0));
    assert!(dir.join("split").join("pages").is_dir() && tracked.is_some());
    // A file another program changed meanwhile is kept and counted.
    let page = dir.join("split").join("pages").join(format!("{}.json", prj.pages[0].id));
    let mut theirs = prj.clone();
    theirs.pages[0].description = "Theirs".into();
    std::fs::write(&page, &openecraft_core::storage::split(&theirs).unwrap()[&format!("pages/{}.json", prj.pages[0].id)]).unwrap();
    let mut ours = prj.clone();
    ours.info.name = "Ours".into();
    assert_eq!(write_project(&io, &index, &ours, &mut tracked), Ok(1));
    let merged = folder::open(Path::new(&index)).unwrap().0;
    assert_eq!((merged.info.name.as_str(), merged.pages[0].description.as_str()), ("Ours", "Theirs"));
    let prj = merged;
    let (back, conv, archive) = read_project(&index, &std::fs::read(&index).unwrap()).unwrap();
    assert!(back == prj && !archive && !conv.converted());
    assert!(read_project(&dir.join("none.oecp").display().to_string(), b"").is_err());

    let zip = openecraft_core::archive::archive(&prj, &Default::default());
    let (back, _, archive) = read_project("demo.oecz", &zip).unwrap();
    assert!(back == prj && archive);
}

#[test]
fn the_worker_of_the_web_page_searches_with_the_module() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| std::fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("{path}: {e}")).replace("\r\n", "\n");
    // The page has the worker beside it, which starts the module of the page and has
    // it search the routing (`openecraft_ui::io::RoutingWorker` sends the jobs).
    assert!(read("index.html").contains("rel=\"copy-file\" href=\"worker.js\""));
    let worker = read("worker.js");
    assert!(worker.contains("api.default({ module_or_path: job.wasm })") && worker.contains("api.search_routing(job.project, job.lang, await built(job))"));
    let module = read("src/main.rs");
    assert!(module.contains("#[wasm_bindgen::prelude::wasm_bindgen]\npub fn search_routing(project: &str, lang: &str, prebuilt: &str)"));
    // In the worker the module only searches: without a window it starts no editor.
    assert!(module.contains("let Some(window) = web_sys::window() else { return };"));
    // The routings of the examples come with a release build of the page.
    assert!(read("Trunk.toml").contains("command_arguments = [\"run\", \"--release\", \"-p\", \"openecraft-core\", \"--example\", \"routings\"]"));
}
