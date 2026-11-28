//! Exchange with KiCad, the PCB design suite, in its own file formats (S-expressions,
//! [`crate::sexpr`]):
//!
//! - [`net`]: the netlist (`.net`). The devices and potentials of the project are
//!   written as one, so that the PCB editor lays out a circuit drawn here; a netlist
//!   of KiCad is read with its components and nets.
//! - [`board`]: a board designed in KiCad becomes a device of the project: a device
//!   box with a connection point for every pin of its connectors, named with the nets
//!   of the board, and a part with the size of the board.
//! - [`sym`]: symbol libraries (`.kicad_sym`) in both directions: the symbols of the
//!   project for KiCad's schematic editor, KiCad's symbols as user symbols.

pub mod board;
pub mod net;
pub mod sym;

/// A file of KiCad starts with a list of this keyword (after white space).
pub fn is_kind(text: &str, keyword: &str) -> bool {
    text.trim_start_matches('\u{feff}').trim_start().strip_prefix('(').is_some_and(|t| t.trim_start().strip_prefix(keyword).is_some_and(|t| t.starts_with(|c: char| c.is_whitespace() || c == '(')))
}

/// Characters of a name KiCad keeps in file and symbol names; all others become `_`.
pub(crate) fn plain_name(s: &str) -> String {
    let t: String = s.trim().chars().map(|c| if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+' | '.') { c } else { '_' }).collect();
    if t.is_empty() { "_".into() } else { t }
}

#[cfg(test)]
mod tests;
