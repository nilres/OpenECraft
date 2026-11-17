//! Reading a picture or the pages of a PDF file.

use openecraft_core::panel::floorplan::{paper_of_dpi, DEFAULT_DPI};
use openecraft_core::raster::{Gray, MAX_PIXELS};
use openecraft_i18n::tr_fmt;

/// A sheet of a floor plan: its picture and the size of a pixel on the paper, mm.
#[derive(Clone, Debug, PartialEq)]
pub struct Sheet {
    pub img: Gray,
    pub paper: f32,
}

/// The sheets of the file `name`: the pages of a PDF file, or the one picture of any
/// other (pixels taken at 150 dpi).
pub fn read(name: &str, bytes: &[u8]) -> Result<Vec<Sheet>, String> {
    if crate::pdf::is_pdf(bytes) {
        return crate::pdf::pages(bytes);
    }
    let img = image::load_from_memory(bytes).map_err(|e| tr_fmt!("{} is not a picture or PDF file: {}", name, e))?;
    if img.width() as usize * img.height() as usize > MAX_PIXELS {
        return Err(tr_fmt!("The image is too large ({} × {} pixels)", img.width(), img.height()));
    }
    // Transparent parts on white.
    let rgba = img.to_rgba8();
    let px = rgba
        .pixels()
        .map(|p| {
            let v = u32::from(openecraft_core::raster::luma(p[0], p[1], p[2]));
            let a = u32::from(p[3]);
            ((v * a + 255 * (255 - a) + 127) / 255) as u8
        })
        .collect();
    Ok(vec![Sheet { img: Gray { w: rgba.width() as usize, h: rgba.height() as usize, px }, paper: paper_of_dpi(DEFAULT_DPI) }])
}

#[cfg(test)]
mod tests;
