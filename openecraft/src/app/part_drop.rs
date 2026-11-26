//! Parts dropped onto a schematic page from the parts management or the data portal:
//! a part that is not in the parts database yet is imported into it first (from the
//! example catalog at once, from the parts server when it has arrived), then its main
//! function is placed as a new device with the part assigned. Enclosures dropped into
//! the navigator of the panel layout that only the parts server has are imported the
//! same way, then made into an enclosure.

use super::*;
use crate::parts_ui::{drop_source, DropSource, PendingDrop};
use openecraft_core::geom::P;

impl App {
    /// A part was dropped onto page `page_i` at `at`.
    pub(super) fn drop_part(&mut self, ctx: &egui::Context, page_i: usize, number: String, at: P) {
        match drop_source(&self.session.prj, &self.parts_db, &number) {
            DropSource::Local => self.place_dropped(ctx, page_i, &number, at, false),
            DropSource::Catalog => {
                if let Some(p) = parts::catalog().iter().find(|p| p.number == number) {
                    self.parts_db.upsert(p.clone());
                    self.parts_db.sort();
                }
                self.place_dropped(ctx, page_i, &number, at, true);
            }
            DropSource::Server => {
                let Some(page) = self.session.prj.pages.get(page_i).map(|p| p.id) else { return };
                self.parts_ui.server.fetch(ctx, std::slice::from_ref(&number));
                self.notify(ctx, tr_fmt!("Importing {} from the parts server…", number));
                self.pending_drop = Some(PendingDrop { number, page, at });
            }
        }
    }

    /// An enclosure dropped into the navigator that is downloaded from the parts server
    /// first.
    pub(super) fn fetch_enclosure(&mut self, ctx: &egui::Context, drop: crate::panel_ui::PartDrop) {
        self.parts_ui.server.fetch(ctx, std::slice::from_ref(&drop.number));
        self.notify(ctx, tr_fmt!("Importing {} from the parts server…", drop.number));
        self.pending_enclosure = Some(drop);
    }

    /// Places a part downloaded for a drop once it is in the parts database (call
    /// every frame after the replies of the parts server).
    pub(super) fn finish_part_drop(&mut self, ctx: &egui::Context) {
        self.finish_enclosure_drop(ctx);
        let Some(pending) = self.pending_drop.clone() else { return };
        if self.parts_db.get(&pending.number).is_some() {
            self.pending_drop = None;
            match self.session.prj.pages.iter().position(|p| p.id == pending.page) {
                Some(page_i) => self.place_dropped(ctx, page_i, &pending.number, pending.at, true),
                None => self.notify_warning(ctx, tr_fmt!("{} was imported into the parts database; its page no longer exists", pending.number)),
            }
        } else if !self.parts_ui.server.busy() {
            self.pending_drop = None;
            self.notify_warning(ctx, tr_fmt!("{} is neither in the parts database nor on the parts server", pending.number));
        }
    }

    /// Makes an enclosure downloaded for a drop once it is in the parts database.
    fn finish_enclosure_drop(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending_enclosure.clone() else { return };
        if self.parts_db.get(&pending.number).is_some() {
            self.pending_enclosure = None;
            match self.panel_ui.drop_part(&mut self.session.prj, &self.parts_db, &pending.number, pending.room) {
                Ok(ei) => {
                    let name = self.session.prj.layout.enclosures[ei].name.clone();
                    self.notify_ok(ctx, tr_fmt!("{} imported into the parts database and added as enclosure {}", pending.number, name));
                }
                Err(e) => self.notify_warning(ctx, e),
            }
        } else if !self.parts_ui.server.busy() {
            self.pending_enclosure = None;
            self.notify_warning(ctx, tr_fmt!("{} is neither in the parts database nor on the parts server", pending.number));
        }
    }

    /// Places the main function of a local part as a new device with the part.
    fn place_dropped(&mut self, ctx: &egui::Context, page_i: usize, number: &str, at: P, imported: bool) {
        let Some(part) = parts::lookup(&self.session.prj, Some(&self.parts_db), number).cloned() else { return };
        let Some((symbol, pins)) = openecraft_core::symlib::ema::part_symbol(&mut self.session.prj, &part, None).or_else(|| openecraft_core::unplaced::part_symbol(&part)) else {
            self.notify_warning(ctx, tr_fmt!("{} has no function with a symbol; assign it to a device in its properties", number));
            return;
        };
        let out = self.editor.place_part_at(&mut self.session.prj, page_i, symbol, pins, number, at);
        let Some(id) = out.placed else { return };
        self.placed(page_i, id, number);
        let tag = self.session.prj.pages[page_i].symbols.iter().find(|s| s.id == id).map(|s| s.tag.clone()).unwrap_or_default();
        let msg = if imported { tr_fmt!("{} imported into the parts database and placed as {}", number, tag) } else { tr_fmt!("{} placed as {}", number, tag) };
        self.notify_ok(ctx, msg);
    }
}
