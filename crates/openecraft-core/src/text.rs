//! Free texts: the lines of a text (wrapped at its width), the anchor point of each
//! line with the text's rotation and alignment, the area a text covers, and path
//! function texts (the function text of the functions in the same column of a page
//! that have none of their own).

use crate::geom::{p, P, R};
use crate::model::{Project, SymbolInst, TextItem, VAnchor};
use crate::symbols::Anchor;

/// Line pitch in text heights.
pub const LINE: f32 = 1.4;

/// Estimated width of a text (the canvas, the exports and the hit tests share it).
pub fn est_width(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * 0.62
}

/// Glyph widths of Helvetica (1/1000 em) for ASCII 32..=126.
pub(crate) const HELVETICA: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556, 556, 556, 556,
    278, 278, 584, 584, 584, 556, 1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667,
    611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833,
    556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];

/// Conservative annotation width at the 1.25 rendering scale. Wide ASCII glyphs
/// use the PDF metrics; narrow glyphs retain the canvas estimate and other scripts
/// reserve one em per character.
pub fn annotation_width(text: &str, size: f32) -> f32 {
    text.chars().map(|c| match c {
        ' '..='~' => HELVETICA[c as usize - 32].max(620) as f32,
        _ => 1000.,
    }).sum::<f32>() * size * 1.25 / 1000.
}

/// How a text is drawn: height, anchors, rotation and style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub h: Anchor,
    pub v: VAnchor,
    /// Degrees, counter-clockwise.
    pub angle: f32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl TextStyle {
    pub fn plain(size: f32, h: Anchor, v: VAnchor) -> TextStyle {
        TextStyle { size, h, v, angle: 0.0, bold: false, italic: false, underline: false }
    }

    /// The style of a text item; its lines are anchored at their middle.
    pub fn of(t: &TextItem) -> TextStyle {
        TextStyle { size: t.size, h: t.align, v: VAnchor::Middle, angle: t.angle, bold: t.bold, italic: t.italic, underline: t.underline }
    }

    pub fn is_plain(&self) -> bool {
        self.angle == 0.0 && !self.bold && !self.italic && !self.underline
    }
}

/// Rotates a vector by `angle` degrees counter-clockwise (page coordinates, y down).
pub fn rotate(v: P, angle: f32) -> P {
    if angle == 0.0 {
        return v;
    }
    let (s, c) = angle.to_radians().sin_cos();
    p(v.x * c + v.y * s, v.y * c - v.x * s)
}

/// The lines of a text: explicit line breaks are kept; lines wider than `width` mm
/// (if > 0) are wrapped at blanks, a single word wider than that stays on its line.
pub fn wrap(text: &str, size: f32, width: f32) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        if width <= 0.0 || est_width(line, size) <= width {
            out.push(line.to_string());
            continue;
        }
        let mut cur = String::new();
        for word in line.split(' ') {
            let longer = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
            if cur.is_empty() || est_width(&longer, size) <= width {
                cur = longer;
            } else {
                out.push(std::mem::replace(&mut cur, word.to_string()));
            }
        }
        out.push(cur);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

/// Offset of the first line's anchor from `pos` along the text's "down" direction
/// (before rotation): the block hangs from `pos`, is centred on it or ends at it.
fn first_line(t: &TextItem, n: usize) -> f32 {
    let span = n.saturating_sub(1) as f32 * LINE * t.size;
    match t.valign {
        VAnchor::Top => 0.0,
        VAnchor::Middle => -span / 2.0,
        VAnchor::Bottom => -span,
    }
}

/// The lines of a text as shown (`shown`: its text with the placeholders filled in)
/// with the anchor point of each line.
pub fn lines(t: &TextItem, shown: &str) -> Vec<(P, String)> {
    let ls = wrap(shown, t.size, t.width);
    let y0 = first_line(t, ls.len());
    ls.into_iter().enumerate().map(|(i, l)| (t.pos + rotate(p(0.0, y0 + i as f32 * LINE * t.size), t.angle), l)).collect()
}

/// Corners of the area the text covers, rotated with the text, with a margin of
/// `pad` text heights (background and frame).
pub fn corners(t: &TextItem, shown: &str, pad: f32) -> [P; 4] {
    let ls = wrap(shown, t.size, t.width);
    let w = ls.iter().map(|l| est_width(l, t.size)).fold(0.0, f32::max).max(t.size * 0.62);
    let (x0, x1) = match t.align {
        Anchor::Left => (0.0, w),
        Anchor::Center => (-w / 2.0, w / 2.0),
        Anchor::Right => (-w, 0.0),
    };
    let y0 = first_line(t, ls.len());
    let y1 = y0 + (ls.len() - 1) as f32 * LINE * t.size;
    let (m, n) = (t.size * (0.7 + pad), t.size * pad);
    [p(x0 - n, y0 - m), p(x1 + n, y0 - m), p(x1 + n, y1 + m), p(x0 - n, y1 + m)].map(|q| t.pos + rotate(q, t.angle))
}

/// Axis-aligned area of a text (picking, selection frames).
pub fn bounds(t: &TextItem, shown: &str) -> R {
    let mut r = R::nothing();
    for q in corners(t, shown, 0.0) {
        r.extend(q);
    }
    r
}

/// The path function text of the column at `pos` on a page: the first path function
/// text in the same column of the plot frame.
pub fn path_text(prj: &Project, page_i: usize, pos: P) -> Option<&TextItem> {
    let page = prj.pages.get(page_i)?;
    if !page.texts.iter().any(|t| t.path_function) {
        return None;
    }
    let frame = crate::frame::of_page(prj, page_i);
    let col = frame.column_index(pos);
    page.texts.iter().filter(|t| t.path_function && !t.text.trim().is_empty()).find(|t| frame.column_index(t.pos) == col)
}

/// Function text of a placed function: its own, or the path function text of the
/// column it stands in.
pub fn function_text<'a>(prj: &'a Project, page_i: usize, s: &'a SymbolInst) -> &'a str {
    if !s.function_text.is_empty() {
        return &s.function_text;
    }
    path_text(prj, page_i, s.pos).map(|t| t.text.as_str()).unwrap_or("")
}

#[cfg(test)]
mod tests;
