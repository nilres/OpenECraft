//! STEP import (ISO 10303-21, AP203 / AP214 / AP242), IGES import (5.3) and Parasolid
//! import (XT text and binary transmit files): the solids and surface models of a
//! file, placed by its assembly structure and tessellated into a part mesh in
//! millimetres. Reading and tessellating the B-rep is done by `monstertruck` (STEP
//! and IGES decoding by its `cadmpeg` backend; Parasolid files are parsed by
//! `parasolid-core` and converted to the `cadmpeg` model).

mod iges;
mod import;
mod parasolid;

pub use iges::parse_iges;
pub use import::{parse_step, parse_step_without};
pub use parasolid::parse_parasolid;
