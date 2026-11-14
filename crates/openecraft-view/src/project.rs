//! The views of a project: groups of pages and devices, places, `ProjectView`.

use openecraft_core::index::{DeviceEntry, Index, Loc};
use openecraft_core::model::{natural_cmp, PageKind, Project};
use openecraft_core::sections;
use openecraft_core::structure::{self, Aspect, Dt};
use openecraft_i18n::{tr_const, translate, Lang};
use serde::{Deserialize, Serialize};

/// The icon of a page kind in page lists.
pub fn page_icon(kind: PageKind) -> &'static str {
    match kind {
        PageKind::Schematic => "🗋",
        PageKind::Cover => "🏷",
        PageKind::Report(_) => "📊",
        PageKind::Mounting(_) => "🗄",
        PageKind::Overview => "🗺",
        PageKind::Graphic => "✏",
        PageKind::Document => "📎",
    }
}

/// The pages `shown` (indices) grouped by their structure (`=A1+CAB1`), in the order
/// the structures first occur; pages without structure have an empty one.
pub fn structure_groups(prj: &Project, shown: &[usize]) -> Vec<(String, Vec<usize>)> {
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for &i in shown {
        let st = structure::page_structure(&prj.pages[i]);
        match groups.iter_mut().find(|g| g.0 == st) {
            Some(g) => g.1.push(i),
            None => groups.push((st, vec![i])),
        }
    }
    groups
}

/// `=A1+CAB1  Pump station` (descriptions from the structure management).
pub fn structure_title(prj: &Project, st: &str) -> String {
    let dt = Dt::parse_structure(st);
    let s = &prj.structures;
    let desc: Vec<String> = Aspect::ALL.into_iter().filter_map(|a| s.entry(a, dt.get(a)).map(|e| e.description.clone())).filter(|d| !d.is_empty()).collect();
    if desc.is_empty() {
        st.to_string()
    } else {
        format!("{st}  {}", desc.join(", "))
    }
}

/// Device tags grouped by their structure (=function+location), the groups in natural
/// order; the tags keep their order.
pub fn device_groups<'a>(tags: impl IntoIterator<Item = &'a String>) -> Vec<(String, Vec<&'a String>)> {
    let mut groups: Vec<(String, Vec<&String>)> = Vec::new();
    for tag in tags {
        let st = Dt::parse(tag).structure();
        match groups.iter_mut().find(|g| g.0 == st) {
            Some(g) => g.1.push(tag),
            None => groups.push((st, vec![tag])),
        }
    }
    groups.sort_by(|a, b| natural_cmp(&a.0, &b.0));
    groups
}

/// The name of a device in a list grouped by structure: without the structure when
/// the list has groups.
pub fn device_title(tag: &str, structured: bool) -> String {
    if structured {
        Dt::parse(tag).name
    } else {
        tag.to_string()
    }
}

/// How a device is shown at a place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaceKind {
    /// Main function (coil, motor).
    Main,
    /// Auxiliary function (contacts).
    Aux,
    /// Terminals, PLC points and others without main / aux relation.
    Other,
}

impl PlaceKind {
    /// English title (empty for others), translated where it is shown.
    pub fn title(self) -> &'static str {
        match self {
            PlaceKind::Main => tr_const!("Main"),
            PlaceKind::Aux => tr_const!("Aux"),
            PlaceKind::Other => "",
        }
    }
}

/// A place a device is shown on: a symbol on a page.
#[derive(Clone, Debug)]
pub struct Place {
    pub kind: PlaceKind,
    pub loc: Loc,
    /// Cross reference of the place, e.g. `=A1/3.4`.
    pub reference: String,
    /// English name of the symbol, `?` if the project has no such symbol.
    pub symbol: &'static str,
    /// The label of the symbol or, without one, its pins (`13-14`).
    pub extra: String,
}

impl Place {
    /// `/3.4 Aux NO contact 13-14`, the texts translated by `tr`.
    pub fn text(&self, tr: impl Fn(&'static str) -> &'static str) -> String {
        let kind = match self.kind {
            PlaceKind::Other => "",
            k => tr(k.title()),
        };
        format!("{} {kind} {} {}", self.reference, tr(self.symbol), self.extra).trim().to_string()
    }
}

/// The places of a device: main functions, then auxiliary ones, then the others.
pub fn places(prj: &Project, dev: &DeviceEntry) -> Vec<Place> {
    let mut out = Vec::new();
    for (kind, locs) in [(PlaceKind::Main, &dev.mains), (PlaceKind::Aux, &dev.auxs), (PlaceKind::Other, &dev.others)] {
        for loc in locs {
            let Some(inst) = prj.pages.get(loc.page).and_then(|p| p.symbols.iter().find(|s| s.id == loc.sym_id)) else { continue };
            let symbol = prj.symbol(&inst.sym).map(|d| d.name).unwrap_or("?");
            let extra = if inst.label.is_empty() { inst.pins.join("-") } else { inst.label.clone() };
            out.push(Place { kind, loc: *loc, reference: loc.reference(prj), symbol, extra });
        }
    }
    out
}

/// A project as the web page of the project server shows it (read only).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectView {
    pub title: String,
    pub description: String,
    pub company: String,
    pub creator: String,
    pub date: String,
    /// Revision of the project info (not the server's revision).
    pub revision: String,
    /// All pages in project order.
    pub pages: Vec<PageView>,
    /// The sections in tree order.
    pub sections: Vec<SectionView>,
    /// The pages grouped by structure.
    pub structures: Vec<StructureView>,
    /// The devices grouped by structure.
    pub devices: Vec<DeviceGroupView>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PageView {
    pub id: u64,
    /// Name within its section or structure (`3`).
    pub name: String,
    /// Name with structure (`=A1+CAB1/3`).
    pub full_name: String,
    pub description: String,
    pub icon: String,
    /// Section of the page; 0: the top level.
    pub section: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SectionView {
    pub id: u64,
    /// 0: the top level.
    pub parent: u64,
    /// `2.3  Main supply`.
    pub title: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StructureView {
    /// `=A1+CAB1`; empty: pages without structure.
    pub structure: String,
    /// The structure with its descriptions.
    pub title: String,
    /// Ids of the pages.
    pub pages: Vec<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DeviceGroupView {
    /// Empty: devices without structure.
    pub structure: String,
    pub devices: Vec<DeviceView>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DeviceView {
    pub tag: String,
    /// The tag without the structure of its group.
    pub title: String,
    pub places: Vec<PlaceView>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaceView {
    /// Id of the page.
    pub page: u64,
    /// `/3.4 Aux NO contact 13-14` in the language of the view.
    pub text: String,
}

/// The view of a project with the texts in `lang`.
pub fn project_view(prj: &Project, idx: &Index, lang: Lang) -> ProjectView {
    let pages = prj
        .pages
        .iter()
        .enumerate()
        .map(|(i, pg)| PageView {
            id: pg.id,
            name: pg.name.clone(),
            full_name: structure::page_full_name(pg),
            description: pg.description.clone(),
            icon: page_icon(pg.kind).into(),
            section: sections::of_page(prj, i),
        })
        .collect();
    let section_list = sections::tree(prj)
        .into_iter()
        .map(|(id, _)| SectionView { id, parent: sections::get(prj, id).map_or(0, |s| s.parent), title: sections::title(prj, id) })
        .collect();
    let all: Vec<usize> = (0..prj.pages.len()).collect();
    let structures = structure_groups(prj, &all)
        .into_iter()
        .map(|(st, pages)| StructureView { title: structure_title(prj, &st), structure: st, pages: pages.into_iter().map(|i| prj.pages[i].id).collect() })
        .collect();
    let groups = device_groups(idx.sorted_tags());
    let structured = groups.iter().any(|g| !g.0.is_empty());
    let devices = groups
        .into_iter()
        .map(|(st, tags)| DeviceGroupView {
            structure: st,
            devices: tags
                .into_iter()
                .map(|tag| DeviceView {
                    tag: tag.clone(),
                    title: device_title(tag, structured),
                    places: places(prj, &idx.devices[tag]).iter().map(|p| PlaceView { page: prj.pages[p.loc.page].id, text: p.text(|t| translate(t, lang)) }).collect(),
                })
                .collect(),
        })
        .collect();
    let info = &prj.info;
    ProjectView {
        title: info.name.clone(),
        description: info.description.clone(),
        company: info.company.clone(),
        creator: info.creator.clone(),
        date: info.date.clone(),
        revision: info.revision.clone(),
        pages,
        sections: section_list,
        structures,
        devices,
    }
}

#[cfg(test)]
mod tests;
