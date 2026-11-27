//! Level 17: the office building of level 16 with installation plan pages: its ground
//! level fitted onto the page with the devices' plan symbols turned towards their walls,
//! the room identifier and the description of the location (`+OF1` *Office*); the attic
//! at a fixed scale of 1:50 with larger room names, smaller symbols and the legend at the
//! bottom.

use super::*;
use crate::model::{LegendAt, PageKind};

pub(super) fn project() -> Project {
    let mut prj = super::house::project();
    prj.info.name = "Office building with installation plans".into();
    prj.info.description = "The office building of the plant\nwith installation plans of its levels".into();
    let b3 = prj.layout.buildings.iter().position(|b| b.name == "+B3").expect("plans");
    let levels: Vec<u64> = prj.layout.buildings[b3].levels.iter().map(|l| l.id).collect();
    let page = |prj: &mut Project, level: u64, name: &str, description: &str| {
        let i = crate::ops::add_page(prj, PageKind::InstallationPlan(level), prj.pages.len());
        prj.pages[i].name = name.into();
        prj.pages[i].description = description.into();
        i
    };
    // The ground level fitted onto the page, every option at its default.
    page(&mut prj, levels[0], "IP1", "Installation plan, ground level");
    // The attic at 1:50: larger room names, smaller symbols, the legend at the bottom.
    let i = page(&mut prj, levels[1], "IP2", "Installation plan, attic");
    let pg = &mut prj.pages[i];
    pg.scale = 50.0;
    pg.plan.legend_at = LegendAt::Bottom;
    pg.plan.room_size = 14.0;
    pg.plan.symbol_scale = 80.0;
    prj
}
