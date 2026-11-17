//! Floor plans on paper for OpenECraft: a picture file (PNG, JPEG, GIF, WebP, TIFF,
//! BMP) or the pages of a PDF file read as grey images
//! ([`openecraft_core::raster::Gray`]) with the size of their pixels on the paper, to
//! put under a level of a building ([`openecraft_core::panel::floorplan`]). The PDF
//! reader ([`pdf`]) draws what floor plans are made of: the pictures of scans and the
//! lines and areas of drawings (no text). Kept out of `openecraft-core` because of the
//! picture decoders.

pub mod pdf;
mod picture;

pub use picture::{read, Sheet};
