//! The panes of the panel layout (layout spaces, view, properties) and of the wiring and
//! mounting assistants (checklist, workbench). They share `PanelUi`; what the panes ask
//! for (go to the schematic, NC export, mounting page) is handled once per frame.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LayoutView {
    Layout,
    ThreeD,
    MountingSurface,
}

impl LayoutView {
    const ALL: [Self; 3] = [Self::Layout, Self::ThreeD, Self::MountingSurface];

    fn title(self) -> &'static str {
        match self {
            Self::Layout => tr("Layout"),
            Self::ThreeD => tr("3D view"),
            Self::MountingSurface => tr("Mounting surface"),
        }
    }
}

pub(super) fn layout_view_combo(ui: &mut egui::Ui, current: Option<LayoutView>, has_layout: bool, has_panel: bool) -> Option<LayoutView> {
    let mut picked = None;
    egui::ComboBox::from_id_salt("layout_view").selected_text(current.map_or(tr("Floor plan"), LayoutView::title)).width(130.0).show_ui(ui, |ui| {
        for view in LayoutView::ALL {
            let enabled = if view == LayoutView::Layout { has_layout } else { has_panel };
            if ui.add_enabled(enabled, egui::Button::selectable(Some(view) == current, view.title())).clicked() && Some(view) != current {
                picked = Some(view);
            }
        }
    });
    picked
}

fn layout_page(prj: &Project, enclosure: u64, current: usize) -> Option<usize> {
    if prj.pages.get(current).is_some_and(|p| p.kind == PageKind::Mounting(enclosure)) {
        return Some(current);
    }
    prj.pages.iter().enumerate().filter(|(_, p)| p.kind == PageKind::Mounting(enclosure))
        .min_by(|(_, a), (_, b)| natural_cmp(&a.name, &b.name).then_with(|| a.id.cmp(&b.id))).map(|(i, _)| i)
}

impl App {
    /// Before the panes are drawn: the assistants mark what is done in the 3D view.
    pub(super) fn layout_frame(&mut self) {
        let workshop = self.pane_shown(Pane::WiringChecklist) || self.pane_shown(Pane::WiringWorkbench);
        if !workshop {
            self.panel_ui.done_wires.clear();
            self.panel_ui.done_items.clear();
        } else if self.mounting_ui.active {
            self.panel_ui.done_wires.clear();
            self.mounting_ui.sync(&self.session.prj, &mut self.panel_ui);
        } else {
            self.panel_ui.selection.clear();
            self.panel_ui.done_items.clear();
            self.wiring_ui.sync(&self.session.prj, &self.session.analysis, &mut self.panel_ui);
        }
    }

    pub(super) fn layout_spaces_pane(&mut self, ui: &mut egui::Ui) {
        self.panel_ui.navigator(ui, &mut self.session.prj, &self.parts_db);
    }

    pub(super) fn layout_properties_pane(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().id_salt("panel_props").auto_shrink([false, false]).show(ui, |ui| {
            if !self.roof_properties(ui) {
                self.panel_ui.properties(ui, &mut self.session.prj, &self.session.analysis, &self.parts_db);
            }
        });
    }

    pub(super) fn layout_view_pane(&mut self, ui: &mut egui::Ui) {
        let current = match self.panel_ui.view {
            crate::panel_ui::View::ThreeD => Some(LayoutView::ThreeD),
            crate::panel_ui::View::Front => Some(LayoutView::MountingSurface),
            crate::panel_ui::View::Plan => None,
        };
        let enclosure = self.session.prj.layout.enclosures.get(self.panel_ui.enclosure).map(|e| e.id);
        let has_layout = enclosure.and_then(|id| layout_page(&self.session.prj, id, self.session.page)).is_some();
        let mut picked = None;
        self.panel_ui.canvas(ui, &mut self.session.prj, &self.session.analysis, &self.parts_db, |ui| {
            picked = layout_view_combo(ui, current, has_layout, enclosure.is_some());
        });
        if let Some((view, enclosure)) = picked.zip(enclosure) {
            self.select_layout_view(ui.ctx(), enclosure, view);
        }
    }

    pub(super) fn layout_page_toolbar(&mut self, ui: &mut egui::Ui) {
        let Some(PageKind::Mounting(enclosure)) = self.session.prj.pages.get(self.session.page).map(|p| p.kind) else { return };
        let has_panel = self.session.prj.layout.enclosures.iter().any(|e| e.id == enclosure);
        let mut picked = None;
        ui.horizontal(|ui| { picked = layout_view_combo(ui, Some(LayoutView::Layout), true, has_panel); });
        ui.separator();
        if let Some(view) = picked {
            self.select_layout_view(ui.ctx(), enclosure, view);
        }
    }

    fn select_layout_view(&mut self, ctx: &egui::Context, enclosure: u64, view: LayoutView) {
        if view == LayoutView::Layout {
            if let Some(page) = layout_page(&self.session.prj, enclosure, self.session.page) {
                self.goto(page, 0);
            }
            return;
        }
        let Some(index) = self.session.prj.layout.enclosures.iter().position(|e| e.id == enclosure) else { return };
        if self.panel_ui.enclosure != index {
            self.panel_ui.selection.clear();
            self.panel_ui.fit();
        }
        self.panel_ui.enclosure = index;
        self.panel_ui.site = false;
        self.show_pane(Pane::LayoutView);
        match view {
            LayoutView::ThreeD if self.panel_ui.view != crate::panel_ui::View::ThreeD => self.run(ctx, Cmd::Layout3d),
            LayoutView::MountingSurface => self.run(ctx, Cmd::LayoutFront),
            _ => {}
        }
    }

    pub(super) fn wiring_checklist_pane(&mut self, ui: &mut egui::Ui) {
        self.mounting_ui.mode_switch(ui);
        if self.mounting_ui.active {
            self.mounting_ui.checklist(ui, &mut self.session.prj, &mut self.panel_ui);
        } else {
            self.wiring_ui.checklist(ui, &mut self.session.prj, &self.session.analysis, &mut self.panel_ui);
        }
    }

    pub(super) fn wiring_workbench_pane(&mut self, ui: &mut egui::Ui) {
        if self.mounting_ui.active {
            self.mounting_ui.workbench(ui, &mut self.session.prj, &self.session.analysis, &mut self.panel_ui);
        } else {
            self.wiring_ui.workbench(ui, &mut self.session.prj, &self.session.analysis, &mut self.panel_ui);
        }
    }

    /// After the panes: the requests of the panel layout and the assistants.
    pub(super) fn layout_requests(&mut self, ctx: &egui::Context, idx: &Index) {
        self.panel_ui.dialogs(ctx, &mut self.session.prj, &self.parts_db);
        self.pv_properties(ctx);
        self.earthing_properties(ctx);
        if std::mem::take(&mut self.mounting_ui.export) {
            let content = openecraft_core::mounting::csv(&self.session.prj, self.mounting_ui.enclosure);
            self.export(ctx, "mounting.csv".into(), ("CSV", "csv"), content);
        }
        if std::mem::take(&mut self.wiring_ui.export) {
            let (prj, an) = (&self.session.prj, &self.session.analysis);
            let content = openecraft_core::wiring::csv(prj, &an.ends, &an.wiring, self.wiring_ui.order);
            self.export(ctx, "wiring.csv".into(), ("CSV", "csv"), content);
        }
        if let Some(tag) = self.panel_ui.goto_schematic.take() {
            let target = idx.devices.get(&tag).and_then(|d| d.mains.first().or(d.others.first()).or(d.auxs.first()).copied());
            if let Some(loc) = target {
                self.set_workspace(Workspace::Schematic);
                self.goto(loc.page, loc.sym_id);
            }
        }
        if let Some((ei, face, format)) = self.panel_ui.export.take() {
            if let Some(e) = self.session.prj.layout.enclosures.get(ei) {
                let base: String = e.name.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
                let what = face.key();
                match format {
                    NcFormat::Dxf => {
                        let content = openecraft_core::production::nc_dxf(&self.session.prj, ei, face);
                        self.export(ctx, format!("{what}{base}.dxf"), (tr("DXF drawing"), "dxf"), content);
                    }
                    NcFormat::Dwg => {
                        let content = openecraft_core::dwg::nc_dwg(&self.session.prj, ei, face);
                        self.export_dwg(ctx, format!("{what}{base}.dwg"), content);
                    }
                    NcFormat::Csv => {
                        let content = openecraft_core::production::nc_csv(&self.session.prj, ei, face);
                        self.export(ctx, format!("{what}{base}_drilling.csv"), ("CSV", "csv"), content);
                    }
                }
            }
        }
        if std::mem::take(&mut self.panel_ui.open_parts) {
            self.run(ctx, crate::commands::Cmd::PartsManagement);
        }
        if std::mem::take(&mut self.panel_ui.open_properties) {
            self.run(ctx, crate::commands::Cmd::LayoutProperties);
        }
        if let Some(id) = self.panel_ui.add_page.take() {
            self.add_page(PageKind::Mounting(id));
            self.set_workspace(Workspace::Schematic);
        }
    }
}

#[cfg(test)]
mod tests;
