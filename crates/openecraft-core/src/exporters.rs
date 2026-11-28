//! The built-in exports as registry entries ([`Exporter`]): drawings of the pages
//! (SVG, HTML, PDF, DXF, DWG), production and PLC lists, exchange files (AutomationML,
//! KBL, the harness interface), the panel layout and the harnesses in 3D (glTF, STEP,
//! IGES, IFC, JT, Parasolid XT), the exchange with KiCad and FreeCAD and the translation
//! dictionary. The front ends offer them next to the exports of plugins; the language
//! of the texts is the host's business (`translation::with_language`).

use crate::ext::{Export, Exporter, Registry};
use crate::model::Project;
use crate::params::{optional, param, Args, Kind, Param};
use crate::analysis::Analysis;
use openecraft_i18n::{tr, tr_const, tr_fmt};

const PAGE: Param = param("page", Kind::Str, tr_const!("Page name as shown by list_pages"));
const PAGE_OPT: Param = optional("page", Kind::Str, tr_const!("Page name as shown by list_pages (default: all pages)"));
const FILE: Param = optional("file", Kind::Str, tr_const!("File name written into the file header"));
const PAGES: Param = optional("pages", Kind::Str, tr_const!("Pages: names and ranges, e.g. 1-3, 5 (default: all pages)"));
const HARNESS: Param = param("harness", Kind::Str, tr_const!("Harness tag, e.g. -WH1"));
const HARNESS_OPT: Param = optional("harness", Kind::Str, tr_const!("Harness tag, e.g. -WH1 (default: the only harness of the project)"));
const HARNESSES: Param = optional("harness", Kind::Str, tr_const!("Harness tag, e.g. -WH1 (default: all harnesses)"));
const SCHEME: Param = optional("scheme", Kind::Str, tr_const!("Labeling scheme (default: the scheme selected in the project)"));
const ALL_SYMBOLS: Param = optional("all", Kind::Bool, tr_const!("All symbols of the library instead of those the project uses"));
const PLUGS: Param = optional("plug", Kind::Str, tr_const!("Plug tag, e.g. -X10 (default: all plugs)"));

fn marking_scheme(prj: &Project, a: &Args) -> Result<crate::marking::Scheme, String> {
    let name = a.str("scheme").filter(|s| !s.trim().is_empty());
    crate::marking::scheme(&prj.marking, name).ok_or_else(|| tr_fmt!("No labeling scheme named {}", name.unwrap_or_default()))
}

fn page_index(prj: &Project, a: &Args) -> Result<Option<usize>, String> {
    let Some(name) = a.str("page") else { return Ok(None) };
    prj.pages.iter().position(|pg| pg.name == name).map(Some).ok_or_else(|| tr_fmt!("No page named {}", name))
}

/// The page given, otherwise all pages.
fn pages(prj: &Project, a: &Args) -> Result<Vec<usize>, String> {
    Ok(match page_index(prj, a)? {
        Some(i) => vec![i],
        None => (0..prj.pages.len()).collect(),
    })
}

fn with_layout(prj: &Project) -> Result<(), String> {
    if prj.layout.enclosures.is_empty() {
        return Err(tr("The project has no panel layout").into());
    }
    Ok(())
}

/// The harness exports have nothing to write without harnesses (like `kbl`).
fn with_harnesses(prj: &Project) -> Result<(), String> {
    if prj.harnesses.is_empty() {
        return Err(tr("The project has no harnesses").into());
    }
    Ok(())
}

fn file_name<'a>(a: &'a Args, default: &'a str) -> &'a str {
    a.str("file").filter(|f| !f.trim().is_empty()).unwrap_or(default)
}

fn text(f: impl Fn(&Project, &Analysis, &Args) -> Result<String, String> + Send + Sync + 'static) -> impl Fn(&Project, &Analysis, &Args) -> Result<Export, String> + Send + Sync + 'static {
    move |prj, an, a| f(prj, an, a).map(Export::Text)
}

/// The built-in exports in the order of File › Export.
pub fn builtin() -> Vec<Exporter> {
    let mut reg = Registry::default();
    reg.exporter_with("svg", tr_const!("SVG image (one page)"), "svg", false, vec![PAGE], text(|prj, an, a| {
        let i = page_index(prj, a)?.ok_or_else(|| tr("This format needs a page").to_string())?;
        Ok(crate::svg::page_svg_with(prj, an, i))
    }))
    .exporter_with("html", tr_const!("HTML document for printing"), "html", false, vec![PAGES], text(|prj, an, a| {
        Ok(crate::svg::print_document_pages(prj, an, &crate::page_select::parse(prj, a.str("pages").unwrap_or_default())?))
    }))
    .exporter_with("pdf", tr_const!("PDF document (bookmarks, linked cross references)"), "pdf", false, vec![PAGES], text(|prj, an, a| {
        Ok(crate::pdf::document_pages(prj, an, &crate::page_select::parse(prj, a.str("pages").unwrap_or_default())?))
    }))
    .exporter_with("dxf", tr_const!("DXF drawing"), "dxf", false, vec![PAGE_OPT], text(|prj, an, a| Ok(crate::dxf::pages_dxf_with(prj, an, &pages(prj, a)?))))
    .exporter_with("dwg", tr_const!("DWG drawing"), "dwg", true, vec![PAGE_OPT], |prj, an, a| crate::dwg::pages_dwg_with(prj, an, &pages(prj, a)?).map(Export::Binary))
    .exporter_with("wire_list", tr_const!("Production wire list (CSV)"), "csv", false, vec![], text(|prj, an, _| Ok(crate::reports::production_wire_list(prj, an).to_csv_bom())))
    .exporter_with("plc_table", tr_const!("PLC symbol table (CSV)"), "csv", false, vec![], text(|prj, an, _| Ok(crate::plc::symbol_table(prj, &an.netlist))))
    .exporter_with("plc_variables", tr_const!("PLC global variables (IEC 61131-3)"), "st", false, vec![], text(|prj, an, _| Ok(crate::plc::global_variables(prj, &an.netlist))))
    .exporter_with("plc_tia", tr_const!("PLC tag table for Siemens TIA Portal (Excel)"), "xlsx", true, vec![], |prj, an, _| {
        Ok(Export::Binary(crate::xlsx::write(&[("PLC Tags".into(), crate::plc::tia_tags(prj, &an.netlist))])))
    })
    .exporter_with("plc_rockwell", tr_const!("PLC tags for Rockwell Studio 5000 (CSV)"), "csv", false, vec![], text(|prj, an, _| Ok(crate::plc::rockwell_tags(prj, &an.netlist))))
    .exporter_with("plc_step7", tr_const!("PLC symbol table for Siemens STEP 7 classic (SDF)"), "sdf", false, vec![], text(|prj, an, _| Ok(crate::plc::step7_sdf(prj, &an.netlist))))
    .exporter_with("plc_twincat", tr_const!("PLC global variable list for Beckhoff TwinCAT 3 (TcGVL)"), "TcGVL", false, vec![], text(|prj, an, _| Ok(crate::plc::twincat_gvl(prj, &an.netlist))))
    .exporter_with("plc_mitsubishi", tr_const!("PLC device comments for Mitsubishi GX Works (CSV)"), "csv", false, vec![], text(|prj, an, _| Ok(crate::plc::mitsubishi_comments(prj, &an.netlist))))
    .exporter_with("plc_schneider", tr_const!("PLC variables for Schneider Control Expert (XSY)"), "xsy", false, vec![], text(|prj, an, _| Ok(crate::plc::schneider_xsy(prj, &an.netlist))))
    .exporter_with("aml", tr_const!("PLC hardware (AutomationML)"), "aml", false, vec![], text(|prj, an, _| Ok(crate::aml::export(prj, &an.netlist))))
    .exporter_with("kbl", tr_const!("Harness (KBL, VDA 4964)"), "kbl", false, vec![HARNESS_OPT], text(|prj, an, a| crate::kbl::export(prj, an, a.str("harness"))))
    .exporter_with("kbl_zip", tr_const!("All harnesses as KBL files (ZIP)"), "zip", true, vec![], |prj, an, _| {
        with_harnesses(prj)?;
        Ok(Export::Binary(crate::kbl::zip(prj, an)))
    })
    .exporter_with("harness_json", tr_const!("Harness interface for machines and tools (JSON)"), "json", false, vec![HARNESSES], text(|prj, an, a| {
        with_harnesses(prj)?;
        crate::harness_export::document(prj, an, a.str("harness")).map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
    }))
    .exporter_with("harness_processing", tr_const!("Wire processing list for cutting and crimping machines (CSV)"), "csv", false, vec![HARNESSES], text(|prj, an, a| {
        with_harnesses(prj)?;
        crate::harness_export::processing_list(prj, an, a.str("harness"))
    }))
    .exporter_with("harness_gltf", tr_const!("Harnesses in 3D (glTF)"), "gltf", false, vec![HARNESSES], text(|prj, an, a| {
        with_harnesses(prj)?;
        crate::harness_export::gltf(prj, an, a.str("harness"))
    }))
    .exporter_with("harness_jt", tr_const!("Harnesses in 3D (JT)"), "jt", true, vec![HARNESSES], |prj, an, a| {
        with_harnesses(prj)?;
        crate::harness_export::jt(prj, an, a.str("harness")).map(Export::Binary)
    })
    .exporter_with("nailboard", tr_const!("Harness nailboard drawing to scale (SVG)"), "svg", false, vec![HARNESS], text(|prj, an, a| {
        let tag = a.req("harness");
        crate::nailboard::layout(prj, an, tag).map(|nb| crate::nailboard::svg(&nb)).ok_or_else(|| tr_fmt!("Harness {} has no wires", tag))
    }))
    .exporter_with("pinouts", tr_const!("Connector pin-outs: mating faces and pin tables (SVG)"), "svg", false, vec![PLUGS], text(|prj, an, a| {
        let plugs: Vec<String> = a.str("plug").filter(|p| !p.trim().is_empty()).map(|p| vec![p.to_string()]).unwrap_or_default();
        if let Some(p) = plugs.first().filter(|p| !crate::plugs::plug_tags(prj).contains(p)) {
            return Err(tr_fmt!("No plug {}", p));
        }
        Ok(crate::pinout::svg(prj, &an.netlist, &an.open, &plugs))
    }))
    .exporter_with("marking", tr_const!("Marking (CSV)"), "csv", false, vec![SCHEME], text(|prj, an, a| {
        let scheme = marking_scheme(prj, a)?;
        Ok(crate::marking::text(&crate::marking::markers(prj, &an.connections), &scheme))
    }))
    .exporter_with("marking_xlsx", tr_const!("Marking (Excel)"), "xlsx", true, vec![SCHEME], |prj, an, a| {
        let scheme = marking_scheme(prj, a)?;
        Ok(Export::Binary(crate::marking::xlsx(&crate::marking::markers(prj, &an.connections), &scheme)))
    })
    .exporter_with("gltf", tr_const!("Panel layout in 3D (glTF)"), "gltf", false, vec![], text(|prj, _, _| {
        with_layout(prj)?;
        Ok(crate::export3d::gltf(prj))
    }))
    .exporter_with("step", tr_const!("Panel layout in 3D (STEP)"), "stp", false, vec![FILE], text(|prj, _, a| {
        with_layout(prj)?;
        Ok(crate::export3d::step(prj, file_name(a, "panel.stp")))
    }))
    .exporter_with("iges", tr_const!("Panel layout in 3D (IGES)"), "igs", false, vec![FILE], text(|prj, _, a| {
        with_layout(prj)?;
        Ok(crate::export3d::iges(prj, file_name(a, "panel.igs")))
    }))
    .exporter_with("ifc", tr_const!("Panel layout in 3D (IFC)"), "ifc", false, vec![FILE], text(|prj, _, a| {
        with_layout(prj)?;
        Ok(crate::export3d::ifc(prj, file_name(a, "panel.ifc")))
    }))
    .exporter_with("jt", tr_const!("Panel layout in 3D (JT)"), "jt", true, vec![], |prj, _, _| {
        with_layout(prj)?;
        Ok(Export::Binary(crate::export3d::jt(prj)))
    })
    .exporter_with("parasolid", tr_const!("Panel layout in 3D (Parasolid XT)"), "x_t", false, vec![], text(|prj, _, _| {
        with_layout(prj)?;
        Ok(crate::export3d::parasolid(prj))
    }))
    .exporter_with("kicad_netlist", tr_const!("KiCad netlist of the devices and potentials (.net)"), "net", false, vec![], text(|prj, an, _| Ok(crate::kicad::net::export(prj, an))))
    .exporter_with("kicad_symbols", tr_const!("KiCad symbol library (.kicad_sym)"), "kicad_sym", false, vec![ALL_SYMBOLS], text(|prj, _, a| Ok(crate::kicad::sym::export_project(prj, a.flag("all")))))
    .exporter_with("freecad", tr_const!("Panel layout in 3D for FreeCAD (macro)"), "FCMacro", false, vec![], text(|prj, _, _| {
        with_layout(prj)?;
        Ok(crate::freecad::macro_text(prj))
    }))
    .exporter_with("dictionary", tr_const!("Translation dictionary (CSV)"), "csv", false, vec![], text(|prj, _, _| Ok(crate::translation::to_csv(prj))));
    reg.exporters
}

#[cfg(test)]
mod tests;
