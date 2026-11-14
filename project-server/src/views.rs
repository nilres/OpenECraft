//! What the web page shows of a project (read only): its view (pages, sections,
//! structures, devices: `openecraft_view`) from the device index and its pages as SVG,
//! drawn from the analysis of the project. Both are kept per project and revision:
//! the analysis is built on another thread when the first page is drawn, the pages
//! when they are asked for.

use openecraft_core::analysis::Analysis;
use openecraft_core::index::Index;
use openecraft_core::model::Project;
use openecraft_i18n::Lang;
use openecraft_view::ProjectView;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

/// Projects whose views are kept.
pub const KEPT: usize = 8;

pub struct Viewed {
    pub revision: u64,
    project: Project,
    index: Index,
    analysis: OnceLock<Analysis>,
    /// Pages drawn so far by id.
    pages: Mutex<HashMap<u64, Arc<String>>>,
}

impl Viewed {
    pub fn new(project: Project, revision: u64) -> Viewed {
        let index = Index::build(&project);
        Viewed { revision, project, index, analysis: OnceLock::new(), pages: Mutex::new(HashMap::new()) }
    }

    pub fn view(&self, lang: Lang) -> ProjectView {
        openecraft_view::project_view(&self.project, &self.index, lang)
    }

    /// The page `id` as SVG; `None` if the project has no such page. The first page
    /// analyses the project (takes a while for large projects).
    pub fn page_svg(&self, id: u64) -> Option<Arc<String>> {
        let i = self.project.pages.iter().position(|p| p.id == id)?;
        if let Some(svg) = self.pages.lock().unwrap_or_else(|e| e.into_inner()).get(&id) {
            return Some(svg.clone());
        }
        let analysis = self.analysis.get_or_init(|| Analysis::with_builtin(&self.project));
        let svg = Arc::new(openecraft_core::svg::page_svg_with(&self.project, analysis, i));
        Some(self.pages.lock().unwrap_or_else(|e| e.into_inner()).entry(id).or_insert(svg).clone())
    }
}

/// The viewed projects by name.
#[derive(Default)]
pub struct Views {
    kept: BTreeMap<String, (u64, Arc<Viewed>)>,
    /// Incremented on every use, to drop the one used longest ago.
    clock: u64,
}

impl Views {
    /// The kept view of `name` at `revision`.
    pub fn get(&mut self, name: &str, revision: u64) -> Option<Arc<Viewed>> {
        self.clock += 1;
        let clock = self.clock;
        let (used, v) = self.kept.get_mut(name).filter(|(_, v)| v.revision == revision)?;
        *used = clock;
        Some(v.clone())
    }

    /// Keeps a view, dropping the one used longest ago when there are more than [`KEPT`].
    pub fn put(&mut self, name: &str, view: Arc<Viewed>) {
        self.clock += 1;
        self.kept.insert(name.to_string(), (self.clock, view));
        while self.kept.len() > KEPT {
            let Some(oldest) = self.kept.iter().min_by_key(|(_, (used, _))| *used).map(|(n, _)| n.clone()) else { break };
            self.kept.remove(&oldest);
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.kept.remove(name);
    }
}

#[cfg(test)]
mod tests;
