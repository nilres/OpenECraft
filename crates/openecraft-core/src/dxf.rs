//! DXF export (AutoCAD R12, millimetres): lines, polylines, circles and texts on
//! layers by kind (lines, wires, texts, frame, cross references …). Several pages
//! are placed side by side.

use crate::geom::P;
use crate::model::Project;
use crate::render::{draw_page, Draw, Ink, Opts, TextStyle, VAnchor};
use crate::analysis::Analysis;
use crate::symbols::Anchor;
use std::collections::HashMap;
use std::fmt::Write;

/// Layers: name and AutoCAD colour index.
const LAYERS: [(&str, u8); 7] = [
    ("OEC_LINES", 7),
    ("OEC_WIRES", 5),
    ("OEC_TEXT", 7),
    ("OEC_FRAME", 8),
    ("OEC_XREF", 4),
    ("OEC_ERROR", 1),
    ("OEC_FILL", 9),
];

fn layer(ink: Ink) -> &'static str {
    match ink {
        Ink::Wire | Ink::Highlight | Ink::Color(_) => "OEC_WIRES",
        Ink::Text => "OEC_TEXT",
        Ink::Frame | Ink::FrameText => "OEC_FRAME",
        Ink::CrossRef => "OEC_XREF",
        Ink::Error => "OEC_ERROR",
        Ink::Plate | Ink::Fill | Ink::Paper => "OEC_FILL",
        _ => "OEC_LINES",
    }
}

/// Text for DXF: non-ASCII characters as `\U+XXXX`; the code has four digits, so
/// characters beyond U+FFFF become `?`.
fn dxf_text(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if (' '..='~').contains(&c) {
            out.push(c);
        } else if (c as u32) <= 0xFFFF {
            let _ = write!(out, "\\U+{:04X}", c as u32);
        } else {
            out.push('?');
        }
    }
    out
}

pub struct DxfDraw {
    out: String,
    /// Horizontal offset of the current page.
    dx: f32,
    /// Paper height of the current page (DXF y runs upwards).
    h: f32,
    /// Layer of the project the following elements are on.
    user_layer: Option<String>,
    /// DXF layer of each project layer (unique, see [`user_layers`]).
    user_layers: HashMap<String, String>,
}

/// DXF layer of a project layer (the default layer has none of its own).
fn user_layer_name(name: &str) -> String {
    let clean: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c.to_ascii_uppercase() } else { '_' }).collect();
    format!("USER_{clean}")
}

/// DXF layers of the project layers, added to `layers`: names that become equal (DXF
/// layer names know neither blanks nor case) get a number.
fn user_layers(prj: &Project, layers: &mut Vec<(String, u8)>) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for name in crate::graphics::layer_names(prj).into_iter().filter(|n| !n.is_empty()) {
        let base = user_layer_name(&name);
        let mut dxf = base.clone();
        let mut k = 2;
        while layers.iter().any(|(n, _)| n.eq_ignore_ascii_case(&dxf)) {
            dxf = format!("{base}_{k}");
            k += 1;
        }
        layers.push((dxf.clone(), 7));
        out.insert(name, dxf);
    }
    out
}

impl DxfDraw {
    fn pt(&mut self, code: u16, q: P) {
        let _ = write!(self.out, "{}\n{:.3}\n{}\n{:.3}\n{}\n0.0\n", code, q.x + self.dx, code + 10, self.h - q.y, code + 20);
    }

    fn entity(&mut self, kind: &str, ink: Ink, dashed: bool) {
        let name = match &self.user_layer {
            Some(l) => self.user_layers.get(l).cloned().unwrap_or_else(|| user_layer_name(l)),
            None => layer(ink).to_string(),
        };
        let _ = write!(self.out, "0\n{kind}\n8\n{name}\n");
        if dashed {
            self.out.push_str("6\nDASHED\n");
        }
    }
}

impl Draw for DxfDraw {
    fn layer(&mut self, name: Option<&str>) {
        self.user_layer = name.filter(|n| !n.is_empty()).map(String::from);
    }

    fn line(&mut self, a: P, b: P, _width: f32, ink: Ink, dashed: bool) {
        self.entity("LINE", ink, dashed);
        self.pt(10, a);
        self.pt(11, b);
    }

    fn poly(&mut self, pts: &[P], closed: bool, _width: f32, ink: Ink, fill: bool) {
        if fill && (pts.len() == 3 || pts.len() == 4) {
            // SOLID corners are given in the order 1, 2, 4, 3.
            self.entity("SOLID", ink, false);
            let order = if pts.len() == 4 { [0, 1, 3, 2] } else { [0, 1, 2, 2] };
            for (i, k) in order.into_iter().enumerate() {
                self.pt(10 + i as u16, pts[k]);
            }
            return;
        }
        self.entity("POLYLINE", ink, false);
        let _ = write!(self.out, "66\n1\n70\n{}\n10\n0.0\n20\n0.0\n30\n0.0\n", if closed || fill { 1 } else { 0 });
        for q in pts {
            self.entity("VERTEX", ink, false);
            self.pt(10, *q);
        }
        self.entity("SEQEND", ink, false);
    }

    fn circle(&mut self, c: P, r: f32, _width: f32, ink: Ink, _fill: bool) {
        self.entity("CIRCLE", ink, false);
        self.pt(10, c);
        let _ = write!(self.out, "40\n{r:.3}\n");
    }

    fn text(&mut self, pos: P, text: &str, size: f32, h: Anchor, v: VAnchor, ink: Ink) {
        self.styled_text(pos, text, &TextStyle::plain(size, h, v), ink);
    }

    fn styled_text(&mut self, pos: P, text: &str, st: &TextStyle, ink: Ink) {
        if text.is_empty() {
            return;
        }
        self.entity("TEXT", ink, false);
        self.pt(10, pos);
        // Cap height of the SVG / screen text (font size 1.25 × size); `%%u` underlines.
        let shown = format!("{}{}", if st.underline { "%%u" } else { "" }, dxf_text(text));
        let _ = write!(self.out, "40\n{:.3}\n1\n{}\n", st.size * 0.9, shown);
        if st.width != 1.0 {
            let _ = write!(self.out, "41\n{:.3}\n", st.width);
        }
        if st.angle != 0.0 {
            let _ = write!(self.out, "50\n{:.3}\n", st.angle);
        }
        if st.italic {
            self.out.push_str("51\n15.0\n");
        }
        let h = match st.h {
            Anchor::Left => 0,
            Anchor::Center => 1,
            Anchor::Right => 2,
        };
        let v = match st.v {
            VAnchor::Top => 3,
            VAnchor::Middle => 2,
            VAnchor::Bottom => 1,
        };
        let _ = write!(self.out, "72\n{h}\n73\n{v}\n");
        self.pt(11, pos);
    }
}

/// DXF header with line types and the given layers (name, AutoCAD colour index).
pub fn header(out: &mut String, layers: &[(String, u8)]) {
    out.push_str("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1009\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n");
    out.push_str("0\nSECTION\n2\nTABLES\n");
    out.push_str("0\nTABLE\n2\nLTYPE\n70\n2\n");
    out.push_str("0\nLTYPE\n2\nCONTINUOUS\n70\n0\n3\nSolid line\n72\n65\n73\n0\n40\n0.0\n");
    out.push_str("0\nLTYPE\n2\nDASHED\n70\n0\n3\nDashed\n72\n65\n73\n2\n40\n2.5\n49\n1.5\n49\n-1.0\n");
    out.push_str("0\nENDTAB\n");
    let _ = write!(out, "0\nTABLE\n2\nLAYER\n70\n{}\n", layers.len());
    for (name, color) in layers {
        let _ = write!(out, "0\nLAYER\n2\n{name}\n70\n0\n62\n{color}\n6\nCONTINUOUS\n");
    }
    out.push_str("0\nENDTAB\n0\nENDSEC\n");
}

/// A free drawing `h` mm high as a DXF drawing (DXF y runs upwards): a symbol alone,
/// as [`crate::svg::drawing_svg`] draws it as a picture.
pub fn drawing_dxf(h: f32, f: impl FnOnce(&mut DxfDraw)) -> String {
    let layers: Vec<(String, u8)> = LAYERS.iter().map(|(n, c)| (n.to_string(), *c)).collect();
    let mut d = DxfDraw { out: String::new(), dx: 0.0, h, user_layer: None, user_layers: HashMap::new() };
    header(&mut d.out, &layers);
    d.out.push_str("0
SECTION
2
ENTITIES
");
    f(&mut d);
    d.out.push_str("0
ENDSEC
0
EOF
");
    d.out
}

/// Pages as one DXF drawing, side by side with 20 mm gap.
pub fn pages_dxf(prj: &Project, pages: &[usize]) -> String {
    pages_dxf_with(prj, &Analysis::with_builtin(prj), pages)
}

/// Pages as one DXF drawing, drawn with the host's analysis.
pub fn pages_dxf_with(prj: &Project, an: &Analysis, pages: &[usize]) -> String {
    let idx = &an.index;
    let mut layers: Vec<(String, u8)> = LAYERS.iter().map(|(n, c)| (n.to_string(), *c)).collect();
    let user_layers = user_layers(prj, &mut layers);
    let mut d = DxfDraw { out: String::new(), dx: 0.0, h: 0.0, user_layer: None, user_layers };
    header(&mut d.out, &layers);
    d.out.push_str("0\nSECTION\n2\nENTITIES\n");
    for &i in pages {
        let (w, h) = crate::frame::of_page(prj, i).size();
        d.h = h;
        draw_page(&mut d, prj, idx, an, i, &Opts::default());
        d.dx += w + 20.0;
    }
    d.out.push_str("0\nENDSEC\n0\nEOF\n");
    d.out
}

#[cfg(test)]
mod tests;
