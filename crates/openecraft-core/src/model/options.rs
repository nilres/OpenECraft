//! Data of `crate::options` that the project file holds (see `docs/architecture.md`).

use super::*;

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ModelOptions {
    pub options: Vec<ModelOption>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelOption {
    pub name: String,
    pub description: String,
    /// Configurator condition (`OPTION=value & …`); empty: switched by hand.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub condition: String,
    pub active: bool,
    /// Ids of the pages and page items that belong to the option.
    pub members: BTreeSet<u64>,
    /// Content of the inactive option, in the order it was taken out.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parked: Vec<Parked>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parked {
    /// Page the item was on; pages: the page before it (0 = the first page).
    pub page: u64,
    /// Position in its list.
    pub index: usize,
    pub item: Item,
    /// Pages: the order of all pages (also parked ones) when it was taken out, so that
    /// it goes back between the right pages whatever came back meanwhile. Empty in
    /// files of older versions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<u64>,
}

/// Content of an inactive option.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Page(Box<Page>),
    Symbol(Box<SymbolInst>),
    Wire(Box<Wire>),
    Text(TextItem),
    Shape(Shape),
    Dimension(Dimension),
    Image(ImageItem),
    ConnDef(ConnDef),
    CableDef(CableDef),
    Junction(Junction),
}
