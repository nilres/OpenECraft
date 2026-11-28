//! The picture of a link to a page of the site in chats, social media and search
//! results (1200 × 630, PNG): for the page of a symbol its drawing, for a category
//! its first symbols, for an example project its first schematic page, each on a card with the
//! icon, the name of the project and what the page is about. A page without one has
//! the picture of the site (`website/social.jpg`).
//!
//! A card is an SVG put together here and drawn by `resvg` with the font of the
//! editor, the same on every machine. Its texts are English, as on the picture of the
//! site: one picture serves the page in every language. [`of`] names the picture of a
//! page; the frame of the page names it to `website-site.sh`, which writes its address
//! into the head (`og:image`).

use crate::examples;
use crate::frame::{escape, File};
use crate::symbols::{self, slug};
use openecraft_core::model::{PageKind, Project};
use openecraft_core::sample::{Example, EXAMPLES};
use openecraft_core::symbols::{Category, SymbolDef};
use openecraft_i18n::Lang;
use resvg::{tiny_skia, usvg};
use std::sync::{Arc, OnceLock};

/// The size of a picture in pixels, the one link previews show whole.
pub const SIZE: (u32, u32) = (1200, 630);

/// The file of the picture of a category or an example in its folder.
pub const FILE: &str = "preview.png";

/// The folder of the pictures of the symbols below the folder of the symbol library.
pub const SYMBOLS: &str = "preview";

/// An area of a card: left, top, width and height in pixels.
type Area = (f32, f32, f32, f32);

/// The sheet of a card: where the drawing goes.
const SHEET: Area = (412.0, 48.0, 740.0, 534.0);

/// The symbols of a category on its card: columns and rows.
const GRID: (usize, usize) = (4, 2);

/// The icon of the project (`assets/icon.svg`, 256 units), without its outer element.
const ICON: &str = "<defs><linearGradient id=\"tile\" gradientUnits=\"userSpaceOnUse\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"256\"><stop offset=\"0\" stop-color=\"#2c72cc\"/><stop offset=\"1\" stop-color=\"#154586\"/></linearGradient></defs><rect x=\"7.68\" y=\"7.68\" width=\"240.64\" height=\"240.64\" rx=\"51.2\" fill=\"url(#tile)\"/><g stroke-width=\"19.2\" stroke-linecap=\"round\"><line x1=\"128\" y1=\"35.84\" x2=\"128\" y2=\"97.28\" stroke=\"#ffffff\"/><line x1=\"128\" y1=\"97.28\" x2=\"99.84\" y2=\"97.28\" stroke=\"#ffffff\"/><line x1=\"128\" y1=\"158.72\" x2=\"128\" y2=\"220.16\" stroke=\"#ffffff\"/><line x1=\"128\" y1=\"158.72\" x2=\"79.36\" y2=\"84.48\" stroke=\"#f5b82e\"/></g>";

/// The picture of the page in `folder` below the site (`symbols/no-contact`), as a
/// file below the site; none for a page that has the picture of the site.
pub fn of(folder: &str) -> Option<String> {
    let mut parts = folder.split('/').filter(|part| !part.is_empty());
    let (first, below, deeper) = (parts.next()?, parts.next()?, parts.next());
    match (first, deeper) {
        (symbols::FOLDER, None) if symbols::is_category(below) => Some(format!("{first}/{below}/{FILE}")),
        (symbols::FOLDER, None) => symbols::by_folder(below).map(symbol_file),
        (examples::FOLDER, None) => EXAMPLES.iter().any(|example| examples::slug(example) == below).then(|| format!("{first}/{below}/{FILE}")),
        _ => None,
    }
}

/// The file of the picture of a symbol below the site.
fn symbol_file(def: &SymbolDef) -> String {
    let drawing = symbols::drawing_file(def);
    let name = drawing.rsplit('/').next().unwrap_or_default().trim_end_matches(".svg");
    format!("{}/{SYMBOLS}/{name}.png", symbols::FOLDER)
}

/// `text` in lines of at most `width` characters, at most `lines` of them; what does
/// not fit ends in an ellipsis.
fn wrapped(text: &str, width: usize, lines: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match out.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => out.push(word.to_string()),
        }
    }
    if out.len() > lines {
        out.truncate(lines);
        if let Some(last) = out.last_mut() {
            let kept: String = last.chars().take(width.saturating_sub(1)).collect();
            *last = format!("{}…", kept.trim_end());
        }
    }
    out
}

/// A drawing of the SVG export (`<svg … viewBox="0 0 w h">…</svg>`) as an element of
/// a card, fitted into the area `x`, `y`, `w`, `h` and centred there.
fn placed(drawing: &str, (x, y, w, h): Area) -> Option<String> {
    let head = drawing.split_once('>')?.0;
    let view = head.split_once("viewBox=\"")?.1.split('"').next()?;
    let inner = drawing.split_once('>')?.1.rsplit_once("</svg>")?.0;
    Some(format!("<svg x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" viewBox=\"{view}\" preserveAspectRatio=\"xMidYMid meet\">{inner}</svg>\n"))
}

/// A card: the `title` of what the page is about, a line `below` it and the
/// `drawings` on the sheet, each with the area it is fitted into.
fn card(title: &str, below: &str, drawings: &[(String, Area)]) -> String {
    let (width, height) = SIZE;
    let (x, y, w, h) = SHEET;
    let mut out = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" font-family=\"sans-serif\">\n<rect width=\"{width}\" height=\"{height}\" fill=\"#070b14\"/>\n");
    out.push_str(&format!("<g transform=\"translate(56 52) scale(0.28)\">{ICON}</g>\n<text x=\"140\" y=\"103\" font-size=\"38\" fill=\"#e8eef9\">OpenECraft</text>\n"));
    // A long title is set smaller, in more lines.
    let (size, lines) = match wrapped(title, 15, 9).len() {
        0..=4 => (40.0, wrapped(title, 15, 4)),
        _ => (30.0, wrapped(title, 20, 6)),
    };
    let mut line_y = 230.0;
    for line in &lines {
        out.push_str(&format!("<text x=\"56\" y=\"{line_y:.0}\" font-size=\"{size:.0}\" fill=\"#ffffff\">{}</text>\n", escape(line)));
        line_y += size * 1.25;
    }
    for line in wrapped(below, 28, 3) {
        out.push_str(&format!("<text x=\"56\" y=\"{:.0}\" font-size=\"22\" fill=\"#9aa9c4\">{}</text>\n", line_y + 6.0, escape(&line)));
        line_y += 30.0;
    }
    out.push_str("<text x=\"56\" y=\"572\" font-size=\"22\" fill=\"#3ad6ff\">Open source electrical CAE</text>\n");
    out.push_str(&format!("<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" rx=\"14\" fill=\"#ffffff\"/>\n"));
    for (drawing, area) in drawings {
        out.push_str(&placed(drawing, *area).unwrap_or_default());
    }
    out.push_str("</svg>\n");
    out
}

/// The card of a symbol: its drawing, as large as the sheet takes it.
pub fn symbol_card(def: &SymbolDef) -> String {
    let (x, y, w, h) = SHEET;
    card(def.name, &format!("Electrical symbol · {}", english(|| def.category.title().to_string())), &[(symbols::drawing(def), (x + 40.0, y + 40.0, w - 80.0, h - 80.0))])
}

/// The card of a category: its first symbols side by side.
pub fn category_card(category: Category) -> String {
    let (x, y, w, h) = SHEET;
    let all = symbols::of(category);
    let (columns, rows) = GRID;
    let (cell_w, cell_h) = ((w - 40.0) / columns as f32, (h - 40.0) / rows as f32);
    let drawings: Vec<(String, Area)> = all
        .iter()
        .take(columns * rows)
        .enumerate()
        .map(|(i, def)| {
            let (column, row) = ((i % columns) as f32, (i / columns) as f32);
            (symbols::drawing(def), (x + 20.0 + column * cell_w + 12.0, y + 20.0 + row * cell_h + 12.0, cell_w - 24.0, cell_h - 24.0))
        })
        .collect();
    card(&english(|| category.title().to_string()), &format!("{} electrical symbols", all.len()), &drawings)
}

/// The card of an example project: `sheet` is a page of it as the SVG export draws it.
pub fn example_card(example: &Example, sheet: &str) -> String {
    let (x, y, w, h) = SHEET;
    let title = english(|| examples::plain_title(openecraft_i18n::tr(example.title)).to_string());
    card(&title, &format!("Example project, level {}", example.level), &[(sheet.to_string(), (x + 8.0, y + 8.0, w - 16.0, h - 16.0))])
}

fn english<T>(text: impl FnOnce() -> T) -> T {
    openecraft_i18n::with_lang(Lang::En, text)
}

/// The font of the cards and of the texts in their drawings: the one of the editor,
/// and no font of the machine that builds the site.
fn fonts() -> Arc<usvg::fontdb::Database> {
    static FONTS: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    FONTS
        .get_or_init(|| {
            let mut db = usvg::fontdb::Database::new();
            db.load_font_data(epaint_default_fonts::UBUNTU_LIGHT.to_vec());
            let family = db.faces().next().and_then(|face| face.families.first()).map(|(name, _)| name.clone());
            if let Some(family) = family {
                db.set_sans_serif_family(family.clone());
                db.set_serif_family(family.clone());
                db.set_monospace_family(family);
            }
            Arc::new(db)
        })
        .clone()
}

/// The colours of a picture as a table of at most 256 and, for every pixel, its place
/// in the table: half the size of a file with three bytes for every pixel. A card
/// has a few hundred colours, most of them between two others at the edge of a line;
/// colours that differ in their lowest bits only share a place, with the colour most
/// pixels of them have, so the large areas keep theirs exactly.
fn indexed(pixels: &[[u8; 3]]) -> (Vec<[u8; 3]>, Vec<u8>) {
    use std::collections::HashMap;
    let mut counts: HashMap<[u8; 3], usize> = HashMap::new();
    for pixel in pixels {
        *counts.entry(*pixel).or_default() += 1;
    }
    let mut bits = 0;
    loop {
        let group = |colour: &[u8; 3]| colour.map(|channel| channel >> bits);
        // The colour of a group: the one most pixels have (the darker one of two alike).
        let mut groups: HashMap<[u8; 3], ([u8; 3], usize)> = HashMap::new();
        for (colour, count) in &counts {
            let best = groups.entry(group(colour)).or_insert((*colour, 0));
            if *count > best.1 || (*count == best.1 && *colour < best.0) {
                *best = (*colour, *count);
            }
        }
        if groups.len() > 256 && bits < 7 {
            bits += 1;
            continue;
        }
        // In a fixed order: the same picture is the same file.
        let mut table: Vec<[u8; 3]> = groups.values().map(|(colour, _)| *colour).collect();
        table.sort_unstable();
        table.truncate(256);
        let place: HashMap<[u8; 3], u8> = groups.iter().map(|(key, (colour, _))| (*key, table.binary_search(colour).unwrap_or(0) as u8)).collect();
        return (table, pixels.iter().map(|pixel| place[&group(pixel)]).collect());
    }
}

/// A card as a PNG file of [`SIZE`], without transparency.
pub fn png(card: &str) -> Result<Vec<u8>, String> {
    let fonts = fonts();
    let family = fonts.family_name(&usvg::fontdb::Family::SansSerif).to_string();
    let options = usvg::Options { fontdb: fonts, font_family: family, ..Default::default() };
    let tree = usvg::Tree::from_str(card, &options).map_err(|e| e.to_string())?;
    let (width, height) = SIZE;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or("no picture of this size")?;
    resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut());
    // The card covers the picture: the colours are those of the pixels.
    let pixels: Vec<[u8; 3]> = pixmap.data().as_chunks::<4>().0.iter().map(|pixel| [pixel[0], pixel[1], pixel[2]]).collect();
    let (table, places) = indexed(&pixels);
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_palette(table.concat());
    encoder.set_compression(png::Compression::High);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(&places).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())?;
    Ok(out)
}

/// The pictures of the symbol library: one for every symbol and every category.
pub fn symbols() -> Result<Vec<File>, String> {
    let mut out = Vec::new();
    for category in symbols::categories() {
        out.push(File { path: format!("{}/{}/{FILE}", symbols::FOLDER, slug(category)), bytes: png(&category_card(category))? });
        for def in symbols::of(category) {
            out.push(File { path: symbol_file(def), bytes: png(&symbol_card(def))? });
        }
    }
    Ok(out)
}

/// The page of a project its card shows: the first schematic page, as a cover sheet
/// says little of what the project is.
pub fn sheet_of(project: &Project) -> usize {
    project.pages.iter().position(|page| matches!(page.kind, PageKind::Schematic)).unwrap_or(0)
}

/// The picture of an example project, with its page as the SVG export draws it.
pub fn example(example: &Example) -> Result<File, String> {
    let sheet = english(|| {
        let project = (example.build)();
        openecraft_core::svg::page_svg(&project, sheet_of(&project))
    });
    Ok(File { path: format!("{}/{}/{FILE}", examples::FOLDER, examples::slug(example)), bytes: png(&example_card(example, &sheet))? })
}

#[cfg(test)]
mod tests;
