//! Multi-language support: the UI texts are written in English in the source and
//! translated at run time through [`tr`] / [`tr_args`]. English, German, French and
//! Spanish are mandatory (see `AGENTS.md`); every text passed to `tr` must have an
//! entry in the catalog (`catalog/<area>/required.tsv`), which `translate/tests.rs`
//! verifies. The further languages ([`Lang::ALL`]) have a catalog each
//! (`catalog/<area>/<code>.tsv`) and show texts it lacks in English. The areas are the
//! crate groups that show the texts (`scan`); `examples/tidy.rs` sorts the files and
//! moves every text to the file of its area.

mod catalog;
pub mod scan;
mod translate;

pub use catalog::AREAS;

pub use translate::*;
