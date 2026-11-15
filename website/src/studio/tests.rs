use super::*;

#[test]
fn the_page_calls_functions_the_module_has() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| std::fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("{path}: {e}")).replace("\r\n", "\n");
    let (script, module) = (read("site.js"), read("src/main.rs"));
    let called: std::collections::BTreeSet<&str> = script.split("api.").skip(1).filter_map(|rest| rest.split_once('(')).map(|(name, _)| name).filter(|n| n.chars().all(|c| c.is_alphanumeric() || c == '_')).collect();
    assert_eq!(called.into_iter().collect::<Vec<_>>(), ["call", "export_file", "start"]);
    // The worker starts the module and has it search the routing.
    let worker = read("worker.js");
    assert!(worker.contains("api.default({ module_or_path: job.wasm })") && worker.contains("api.search_routing(job.project, job.lang, job.prebuilt || \"\")"));
    assert!(script.contains("new Worker(\"worker.js\", { type: \"module\" })") && read("index.html").contains("rel=\"copy-file\" href=\"worker.js\""));
    for name in ["call", "export_file", "start", "search_routing"] {
        assert!(module.contains(&format!("#[wasm_bindgen]\n    pub fn {name}(")), "{name}");
    }
    // wasm-bindgen gives a function named like a JavaScript keyword another name.
    assert!(!module.contains("pub fn export("));
}

fn studio() -> Studio {
    let mut s = Studio::default();
    s.settle();
    s
}

fn ok(s: &mut Studio, method: &str, arg: &str) -> Value {
    let answer = s.call(method, arg);
    assert!(answer.get("error").is_none(), "{method} {arg}: {answer}");
    answer
}

#[test]
fn the_first_example_has_pages_devices_and_a_panel_layout() {
    let mut s = studio();
    let state = ok(&mut s, "state", "");
    assert_eq!(state["example"], FIRST_EXAMPLE);
    assert!(state["pages"].as_array().unwrap().len() > 3);
    assert!(state["devices"].as_u64().unwrap() > 5);
    assert!(!state["places"].as_array().unwrap().is_empty(), "the 3D view needs an enclosure");
    assert!(state["issues"].as_array().unwrap().iter().all(|i| i["severity"] == "info"), "the examples pass the checks");
    assert_eq!(state["loading"], false);
}

#[test]
fn every_example_opens() {
    let mut s = studio();
    let examples = ok(&mut s, "examples", "");
    assert_eq!(examples.as_array().unwrap().len(), sample::EXAMPLES.len());
    ok(&mut s, "open", "lamp_circuit");
    s.settle();
    let state = ok(&mut s, "state", "");
    assert_eq!((state["example"].as_str(), state["page"].as_u64()), (Some("lamp_circuit"), Some(s.session.page as u64)));
    assert!(s.call("open", "nothing").get("error").is_some());
}

#[test]
fn commands_run_as_on_the_command_line_and_can_be_undone() {
    let mut s = studio();
    let pages = s.run("list_pages").unwrap();
    assert!(pages.contains("\"name\""), "{pages}");
    let tag = s.session.index.sorted_tags().into_iter().find(|t| t.contains("-K")).cloned().expect("a contactor");
    let before = s.session.prj.clone();
    s.run(&format!("rename_device --old {tag} --new=-K77")).unwrap();
    s.settle();
    assert!(s.session.index.devices.keys().any(|t| t.ends_with("-K77")));
    assert_eq!(ok(&mut s, "state", "")["undo"], true);
    ok(&mut s, "undo", "");
    assert!(s.session.prj == before);
    // Errors are answers, not panics; a failed command changes nothing.
    assert!(s.run("no_such_command").is_err());
    assert!(s.run("rename_device --old").is_err());
    assert!(s.run("list_pages extra").is_err());
    assert!(s.run("").is_err());
    assert!(s.session.prj == before);
}

#[test]
fn a_change_shows_in_the_checks() {
    let mut s = studio();
    let page = s.session.page;
    let wires = s.session.prj.pages[page].wires.len();
    assert!(wires > 0);
    assert_eq!(ok(&mut s, "state", "")["issues"].as_array().unwrap().iter().filter(|i| i["severity"] != "info").count(), 0);
    s.session.prj.pages[page].wires.truncate(wires / 2);
    s.settle();
    let problems = |s: &mut Studio| ok(s, "state", "")["issues"].as_array().unwrap().iter().filter(|i| i["severity"] != "info").count();
    assert!(problems(&mut s) > 0, "open connections are reported");
    ok(&mut s, "issue", "0");
    ok(&mut s, "undo", "");
    s.settle();
    assert_eq!(problems(&mut s), 0);
}

#[test]
fn reports_and_exports_come_from_the_plugins() {
    let mut s = studio();
    let reports = ok(&mut s, "reports", "");
    let reports = reports.as_array().unwrap();
    assert!(reports.len() > 5);
    let mut filled = 0;
    for r in reports {
        let table = ok(&mut s, "report", r["name"].as_str().unwrap());
        assert!(!table["headers"].as_array().unwrap().is_empty(), "{r}");
        filled += usize::from(table["total"].as_u64().unwrap() > 0);
    }
    assert!(filled > 5);
    let exporters = ok(&mut s, "exporters", "");
    let exporters = exporters.as_array().unwrap();
    assert!(exporters.iter().any(|e| e["extension"] == "pdf"), "{exporters:?}");
    for e in exporters {
        let (name, bytes) = s.export(e["name"].as_str().unwrap()).unwrap_or_else(|err| panic!("{e}: {err}"));
        assert!(name.starts_with(FIRST_EXAMPLE) && !bytes.is_empty(), "{e}");
    }
}

#[test]
fn the_exports_are_shown_in_groups() {
    let mut s = studio();
    let exporters = ok(&mut s, "exporters", "");
    let exporters = exporters.as_array().unwrap();
    let names: Vec<&str> = exporters.iter().map(|e| e["name"].as_str().unwrap()).collect();
    let other = tr(EXPORT_GROUPS.last().unwrap().0);
    let mut groups: Vec<&str> = Vec::new();
    for e in exporters {
        let group = e["group"].as_str().unwrap();
        assert_ne!(group, other, "{e} belongs to a group of EXPORT_GROUPS");
        if groups.last() != Some(&group) {
            assert!(!groups.contains(&group), "the exports of {group} are not together");
            groups.push(group);
        }
    }
    assert_eq!(groups, EXPORT_GROUPS[..EXPORT_GROUPS.len() - 1].iter().map(|g| tr(g.0)).collect::<Vec<_>>());
    // The table names no export twice and none that is not offered.
    let listed: Vec<&str> = EXPORT_GROUPS.iter().flat_map(|g| g.1.iter().copied()).collect();
    for name in &listed {
        assert!(names.contains(name), "{name} is no export of the page");
        assert_eq!(listed.iter().filter(|n| n == &name).count(), 1, "{name}");
    }
}

#[test]
fn the_examples_are_shown_without_their_numbers() {
    assert_eq!(plain_title("4 · Conveyor with PLC"), "Conveyor with PLC");
    assert_eq!(plain_title("16. Office building"), "Office building");
    assert_eq!(plain_title("3D models"), "D models", "a number is taken off; no title of an example starts like this");
    assert_eq!(plain_title("Conveyor 2"), "Conveyor 2");
    let mut s = studio();
    for lang in Lang::ALL {
        openecraft_i18n::with_lang(lang, || {
            let examples = ok(&mut s, "examples", "");
            for (e, example) in examples.as_array().unwrap().iter().zip(sample::EXAMPLES) {
                let title = e["title"].as_str().unwrap();
                assert!(!title.is_empty() && tr(example.title).ends_with(title), "{lang:?}: {title}");
                assert!(title.starts_with(|c: char| c.is_alphabetic()), "{lang:?}: {title}");
            }
            let title = ok(&mut s, "state", "")["title"].as_str().unwrap().to_string();
            assert!(title.starts_with(|c: char| c.is_alphabetic()), "{lang:?}: {title}");
        });
    }
}

#[test]
fn an_example_is_the_same_project_every_time_it_is_opened() {
    // The page keeps the routing of an example by the text of its project.
    let mut s = studio();
    ok(&mut s, "open", "cabinet_lineup");
    let first = ok(&mut s, "project", "")["project"].clone();
    ok(&mut s, "open", "conveyor");
    assert_ne!(ok(&mut s, "project", "")["project"], first);
    ok(&mut s, "open", "cabinet_lineup");
    assert_eq!(ok(&mut s, "project", "")["project"], first);
}

/// Asks for the state until the project is loaded.
fn loaded(s: &mut Studio) {
    while ok(s, "state", "")["loading"] == true {
        std::thread::yield_now();
    }
}

#[test]
fn the_worker_of_the_page_searches_the_routing() {
    const EXAMPLE: &str = "cabinet_lineup";
    let mut s = studio();
    assert!(ok(&mut s, "state", "")["search"].is_null());
    let mut here = studio();
    ok(&mut here, "open", EXAMPLE);
    here.settle();
    assert!(!here.session.analysis.routing.items.is_empty());

    // With a worker the project waits for its routing instead of searching it.
    ok(&mut s, "worker", "on");
    ok(&mut s, "open", EXAMPLE);
    let state = ok(&mut s, "state", "");
    let search = state["search"].as_u64().unwrap();
    assert_eq!(state["loading"], true);
    assert!(s.call("report", "bom").get("error").is_some() && s.export("pdf").is_err(), "reports and exports wait for the analysis");
    let job = ok(&mut s, "project", "");
    assert_eq!((job["search"].as_u64(), job["lang"].as_str()), (Some(search), Some(openecraft_i18n::lang().code())));
    assert_eq!(job["example"], EXAMPLE, "the page has the routing of an unchanged example from its build");
    let routing = search_routing(job["project"].as_str().unwrap(), job["lang"].as_str().unwrap(), "").unwrap();
    // A routing from the build of the page is taken if it is of the project.
    let prebuilt = Cache::routing_for_any_language(&routing).unwrap();
    assert_eq!(search_routing(job["project"].as_str().unwrap(), job["lang"].as_str().unwrap(), &prebuilt).unwrap(), routing);
    let other = search_routing(&serde_json::to_string(&(sample::EXAMPLES.iter().find(|e| e.key == "conveyor").unwrap().build)()).unwrap(), "en", "").unwrap();
    assert_eq!(search_routing(job["project"].as_str().unwrap(), job["lang"].as_str().unwrap(), &other).unwrap(), routing);
    assert_eq!(search_routing(job["project"].as_str().unwrap(), job["lang"].as_str().unwrap(), "no routing").unwrap(), routing);
    assert_eq!(ok(&mut s, "routing", ""), Value::String(routing.clone()));
    // The answer to another search is dropped.
    ok(&mut s, "routed", &format!("{}\n{routing}", search + 7));
    assert_eq!(ok(&mut s, "state", "")["search"], search);
    ok(&mut s, "routed", &format!("{search}\n{routing}"));
    loaded(&mut s);
    assert!(ok(&mut s, "state", "")["search"].is_null());
    assert_eq!(s.session.analysis.routing, here.session.analysis.routing);
    assert!(s.export("pdf").is_ok());

    // A change of the language asks again; undo meanwhile asks for the project as it
    // is then, and a search that failed is made here.
    s.ask_worker();
    let asked = ok(&mut s, "state", "")["search"].as_u64().unwrap();
    assert!(asked > search);
    ok(&mut s, "undo", "");
    let again = ok(&mut s, "state", "")["search"].as_u64().unwrap();
    assert!(again > asked);
    ok(&mut s, "routed", &format!("{asked}\n{routing}"));
    assert_eq!(ok(&mut s, "state", "")["loading"], true);
    ok(&mut s, "routed", &format!("{again}\n"));
    loaded(&mut s);
    assert_eq!(s.session.analysis.routing, here.session.analysis.routing);
    assert!(search_routing("no project", "en", "").is_err() && search_routing(job["project"].as_str().unwrap(), "xx", "").is_err());

    // A console command does not wait for the worker.
    s.ask_worker();
    ok(&mut s, "run", "list_pages");
    assert_eq!(ok(&mut s, "state", "")["loading"], false);
    // A changed example is not the one of the build.
    ok(&mut s, "run", "set_project_languages --source en --targets de");
    assert!(ok(&mut s, "project", "")["example"].is_null());

    // Without the worker the project is analysed here again.
    ok(&mut s, "worker", "off");
    ok(&mut s, "open", EXAMPLE);
    assert!(ok(&mut s, "state", "")["search"].is_null());
    loaded(&mut s);
    assert_eq!(s.session.analysis.routing, here.session.analysis.routing);
}

#[test]
fn the_language_changes_generated_texts() {
    let mut s = studio();
    openecraft_i18n::with_lang(Lang::En, || {
        let english = ok(&mut s, "reports", "").to_string();
        let texts = ok(&mut s, "texts", "");
        assert_eq!(texts.as_object().unwrap().len(), crate::texts::TEXTS.len());
        assert!(s.call("language", "xx").get("error").is_some());
        let german = openecraft_i18n::with_lang(Lang::De, || ok(&mut s, "reports", "").to_string());
        assert_ne!(english, german);
    });
    assert_eq!(ok(&mut s, "languages", "").as_array().unwrap().len(), Lang::ALL.len());
}

#[test]
fn pages_tools_and_views_follow_the_page() {
    let mut s = studio();
    ok(&mut s, "page", "1");
    assert_eq!(s.session.page, 1);
    assert!(s.call("page", "x").get("error").is_some());
    ok(&mut s, "page", "9999");
    assert_eq!(s.session.page, 1, "an unknown page changes nothing");
    for (name, _) in SYMBOLS {
        ok(&mut s, "tool", name);
        assert_eq!(ok(&mut s, "state", "")["tool"], name);
        assert!(matches!(&s.editor.tool, Tool::Place(k) if openecraft_core::symbols::builtin(k).is_some()), "{name}");
    }
    ok(&mut s, "tool", "wire");
    assert_eq!(s.editor.tool, Tool::Wire);
    assert!(s.call("tool", "hammer").get("error").is_some());
    ok(&mut s, "view", "front");
    assert_eq!(ok(&mut s, "state", "")["view"], "front");
    let svg = ok(&mut s, "svg", "0");
    assert!(svg.as_str().unwrap().starts_with("<svg"));
}

#[test]
fn the_layout_view_switches_between_enclosures_rooms_and_the_site() {
    let mut s = studio();
    ok(&mut s, "open", "plant_buildings");
    s.settle();
    let state = ok(&mut s, "state", "");
    let places = state["places"].as_array().unwrap().clone();
    let room = places.iter().position(|p| p["room"] == true).expect("a room");
    let cabinet = places.iter().position(|p| p["room"] == false).expect("an enclosure");
    assert_eq!(state["site"], true);

    ok(&mut s, "place", &room.to_string());
    let state = ok(&mut s, "state", "");
    assert_eq!((&state["place"], &state["place_kind"], &state["view"]), (&json!(room), &json!("room"), &json!("3d")));
    ok(&mut s, "view", "front");
    assert_eq!(s.panel.view, View::Plan, "the drawing of a room is its floor plan");

    ok(&mut s, "view", "3d");
    ok(&mut s, "place", "site");
    let state = ok(&mut s, "state", "");
    assert_eq!((&state["place"], &state["place_kind"], &state["view"]), (&json!("site"), &json!("site"), &json!("3d")));
    ok(&mut s, "view", "front");
    assert_eq!((s.panel.site, s.panel.view), (true, View::Plan), "the drawing of the site is the site plan");

    ok(&mut s, "place", &cabinet.to_string());
    let state = ok(&mut s, "state", "");
    assert_eq!((&state["place"], &state["place_kind"]), (&json!(cabinet), &json!("enclosure")));
    ok(&mut s, "view", "front");
    assert_eq!(s.panel.view, View::Front);

    assert!(s.call("place", "999").get("error").is_some());
    // An example without rooms has no site to show.
    ok(&mut s, "open", "conveyor");
    s.settle();
    assert_eq!(ok(&mut s, "state", "")["site"], false);
    assert!(s.call("place", "site").get("error").is_some());
}

#[test]
fn typed_lines_are_split_at_blanks_outside_quotes() {
    assert_eq!(split_line("  a  --b \"c d\" 'e \"f\"' g''h ").unwrap(), ["a", "--b", "c d", "e \"f\"", "gh"]);
    assert_eq!(split_line("--text \"\"").unwrap(), ["--text", ""]);
    assert!(split_line("a \"b").is_err());
    assert!(split_line("   ").unwrap().is_empty());
}

#[test]
fn the_still_picture_of_the_page_is_the_first_sheet_of_the_demo() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let picture = std::fs::read_to_string(dir.join("sheet.svg")).unwrap().replace("\r\n", "\n");
    let sheet = crate::first_sheet();
    assert!(sheet.starts_with("<svg ") && sheet.len() < 60_000, "{} bytes", sheet.len());
    // The export lists the junctions of the wires in no fixed order.
    let sorted = |svg: &str| {
        let mut lines: Vec<String> = svg.trim_end().lines().map(String::from).collect();
        lines.sort();
        lines
    };
    assert!(sorted(&picture) == sorted(&sheet), "website/sheet.svg is not the page the demo opens with: cargo run -p openecraft-website -- sheet > website/sheet.svg");
    let html = std::fs::read_to_string(dir.join("index.html")).unwrap();
    assert!(html.contains("<img src=\"sheet.svg\"") && html.contains("rel=\"copy-file\" href=\"sheet.svg\""));
}
