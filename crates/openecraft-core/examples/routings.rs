//! Writes the 3D routing of the example projects into a folder
//! ([`Cache::routing_searched`]): the website serves them beside its page, so that an
//! example is there at once instead of after a search in the browser
//! (`docs/website.md`).
//!
//! An example whose routing is the same in the languages of [`PROBES`] has no
//! translated text in it and gets one file for every language, `<example>.json`; the
//! others get `<example>.<language>.json` for each language. `index.json` lists the
//! files.
//!
//! `cargo run --release -p openecraft-core --example routings -- <folder>`; as a hook
//! of trunk (no argument) the folder is `routing` in the page being built, and a
//! debug build is left without.

use openecraft_core::analysis::{Analysis, Cache};
use openecraft_core::model::Project;
use openecraft_core::sample::EXAMPLES;
use openecraft_i18n::Lang;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// The languages a routing is searched in to see whether it has translated text.
const PROBES: [Lang; 3] = [Lang::En, Lang::De, Lang::Zh];

fn main() {
    let dir = match (std::env::args().nth(1), std::env::var("TRUNK_STAGING_DIR")) {
        (Some(dir), _) => PathBuf::from(dir),
        (None, Ok(_)) if std::env::var("TRUNK_PROFILE").is_ok_and(|p| p != "release") => {
            println!("no routings for a debug build");
            return;
        }
        (None, Ok(staging)) => Path::new(&staging).join("routing"),
        (None, Err(_)) => fail("usage: routings <folder>"),
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        fail(&format!("{}: {e}", dir.display()));
    }
    // An example without a panel layout has no routing.
    let projects: Vec<(&str, Project)> = EXAMPLES.iter().map(|e| (e.key, (e.build)())).filter(|(_, p)| !p.layout.enclosures.is_empty()).collect();
    let probes: Vec<(usize, Lang)> = (0..projects.len()).flat_map(|i| PROBES.map(|l| (i, l))).collect();
    let probed = search(&projects, &probes);
    let mut files: Vec<(String, String)> = Vec::new();
    let mut rest: Vec<(usize, Lang)> = Vec::new();
    for (i, (key, _)) in projects.iter().enumerate() {
        let of: Vec<&(String, String)> = probes.iter().zip(&probed).filter(|(p, _)| p.0 == i).map(|(_, r)| r).collect();
        if of.iter().all(|r| r.1 == of[0].1) {
            files.push((format!("{key}.json"), of[0].1.clone()));
        } else {
            files.extend(probes.iter().zip(&probed).filter(|(p, _)| p.0 == i).map(|(p, r)| (format!("{key}.{}.json", p.1.code()), r.0.clone())));
            rest.extend(Lang::ALL.into_iter().filter(|l| !PROBES.contains(l)).map(|l| (i, l)));
        }
    }
    let searched = search(&projects, &rest);
    files.extend(rest.iter().zip(searched).map(|(job, r)| (format!("{}.{}.json", projects[job.0].0, job.1.code()), r.0)));
    files.sort();
    let names: Vec<&str> = files.iter().map(|f| f.0.as_str()).collect();
    files.push(("index.json".to_string(), serde_json::to_string(&names).unwrap_or_default()));
    for (name, text) in &files {
        if let Err(e) = std::fs::write(dir.join(name), text) {
            fail(&format!("{}: {e}", dir.join(name).display()));
        }
    }
    println!("{} routings in {}", files.len() - 1, dir.display());
}

/// The routing of each job (a project in a language) as it is searched, and for any
/// language; the jobs run side by side.
fn search(projects: &[(&str, Project)], jobs: &[(usize, Lang)]) -> Vec<(String, String)> {
    let next = AtomicUsize::new(0);
    let found: Vec<Mutex<(String, String)>> = jobs.iter().map(|_| Mutex::default()).collect();
    std::thread::scope(|scope| {
        for _ in 0..std::thread::available_parallelism().map_or(1, |n| n.get()).min(jobs.len()) {
            scope.spawn(|| loop {
                let at = next.fetch_add(1, Ordering::Relaxed);
                let Some(&(i, lang)) = jobs.get(at) else { break };
                let (key, prj) = &projects[i];
                let mut cache = Cache::default();
                openecraft_i18n::with_lang(lang, || Analysis::build_with(prj, &mut cache));
                let routing = cache.routing_searched().unwrap_or_else(|| fail(&format!("{key}: no routing")));
                let any = Cache::routing_for_any_language(&routing).unwrap_or_else(|e| fail(&format!("{key}: {e}")));
                *found[at].lock().unwrap() = (routing, any);
            });
        }
    });
    found.into_iter().map(|f| f.into_inner().unwrap()).collect()
}

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1)
}
