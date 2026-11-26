//! DWG export and import (AutoCAD 2004 binary drawings, read: R12 … 2018) through the
//! DXF code: the drawing is converted with `acadrust`, so pages, NC data and imported
//! drawings look exactly like their DXF counterparts.

use crate::dxf_import::{self, DxfImport, DxfOptions};
use crate::model::Project;
use acadrust::types::DxfVersion;
use acadrust::{DwgReadOptions, DwgReader, DwgWriter, DxfReader, DxfWriter};
use openecraft_i18n::{tr, tr_fmt};
use std::io::Cursor;

/// Version of the written drawings: AutoCAD 2004, read by every current CAD program.
const VERSION: DxfVersion = DxfVersion::AC1018;

/// Whether the bytes are a DWG file (`AC10xx` / `AC1.x` version magic).
pub fn is_dwg(bytes: &[u8]) -> bool {
    bytes.len() > 6 && bytes.starts_with(b"AC1") && bytes[3..6].iter().all(u8::is_ascii_alphanumeric)
}

/// A DXF drawing as DWG.
pub fn from_dxf(dxf: &str) -> Result<Vec<u8>, String> {
    let mut doc = DxfReader::from_reader(Cursor::new(dxf.as_bytes().to_vec())).and_then(|r| r.read()).map_err(|e| tr_fmt!("DWG export failed: {}", e))?;
    doc.version = VERSION;
    DwgWriter::write_to_vec(&doc).map_err(|e| tr_fmt!("DWG export failed: {}", e))
}

/// A DWG drawing as ASCII DXF.
pub fn to_dxf(dwg: &[u8]) -> Result<String, String> {
    if !is_dwg(dwg) {
        return Err(tr("Not a DWG file").into());
    }
    // Damaged drawings: what the failsafe reader recovers, if anything.
    let doc = match DwgReader::from_stream(Cursor::new(dwg.to_vec())).read() {
        Ok(doc) => doc,
        Err(e) => match DwgReader::from_stream_with_options(Cursor::new(dwg.to_vec()), DwgReadOptions::failsafe()).read() {
            Ok(doc) if doc.entities().next().is_some() => doc,
            _ => return Err(tr_fmt!("Cannot read the DWG file: {}", e)),
        },
    };
    let dxf = DxfWriter::new(&doc).write_to_vec().map_err(|e| tr_fmt!("Cannot read the DWG file: {}", e))?;
    Ok(String::from_utf8_lossy(&dxf).replace("\r\n", "\n"))
}

/// Pages as one DWG drawing, side by side like [`crate::dxf::pages_dxf`].
pub fn pages_dwg(prj: &Project, pages: &[usize]) -> Result<Vec<u8>, String> {
    from_dxf(&crate::dxf::pages_dxf(prj, pages))
}

/// Pages as one DWG drawing, drawn with the host's analysis.
pub fn pages_dwg_with(prj: &Project, an: &crate::analysis::Analysis, pages: &[usize]) -> Result<Vec<u8>, String> {
    from_dxf(&crate::dxf::pages_dxf_with(prj, an, pages))
}

/// NC data of a sheet of an enclosure (face or leaf of the door) as DWG, like
/// [`crate::production::nc_dxf`].
pub fn nc_dwg(prj: &Project, enc: usize, sheet: impl Into<crate::panel::Sheet>) -> Result<Vec<u8>, String> {
    from_dxf(&crate::production::nc_dxf(prj, enc, sheet))
}

/// Inserts a DWG drawing as a new schematic page after `after` (see
/// [`dxf_import::import_page`]).
pub fn import_page(prj: &mut Project, dwg: &[u8], description: &str, after: usize, opts: DxfOptions) -> Result<DxfImport, String> {
    dxf_import::import_page(prj, &to_dxf(dwg)?, description, after, opts)
}

/// Imports a DXF or DWG drawing, told apart by the content.
pub fn import_drawing(prj: &mut Project, bytes: &[u8], description: &str, after: usize, opts: DxfOptions) -> Result<DxfImport, String> {
    if is_dwg(bytes) {
        import_page(prj, bytes, description, after, opts)
    } else {
        dxf_import::import_page(prj, &String::from_utf8_lossy(bytes), description, after, opts)
    }
}

#[cfg(test)]
mod tests;
