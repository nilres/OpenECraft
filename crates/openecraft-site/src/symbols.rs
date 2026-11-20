//! The symbol library as pages: one for every category of the built-in symbols, each
//! symbol with its drawing (the symbol itself, drawn by the SVG export), its name in
//! the language of the page, its key, identifying letter and connection points, and
//! an overview that lists the categories.

use crate::frame::{attribute, escape, files, hero, shortened, File, Kind, Langs, Page, Place, DESCRIPTION};
use openecraft_core::geom::p;
use openecraft_core::model::SymbolInst;
use openecraft_core::render::{draw_symbol, Ink};
use openecraft_core::svg::drawing_svg;
use openecraft_core::symbols::{library, Category, SymbolDef};
use openecraft_i18n::{tr, tr_fmt, Lang};

/// The folder of the pages below the site.
pub const FOLDER: &str = "symbols";

/// White around a symbol in its drawing, in mm.
const MARGIN: f32 = 4.0;

/// The folder of a category below [`FOLDER`].
pub fn slug(category: Category) -> &'static str {
    match category {
        Category::Coils => "coils-relays",
        Category::Contacts => "contacts",
        Category::Switches => "switches",
        Category::Protection => "protective-devices",
        Category::Loads => "motors-loads",
        Category::Terminals => "terminals",
        Category::Plugs => "plugs",
        Category::Plc => "plc",
        Category::Potentials => "potentials-connectors",
        Category::Passive => "passive-components",
        Category::Sensors => "sensors",
        Category::Measuring => "measuring-instruments",
        Category::Power => "transformers-power-supplies",
        Category::BlackBox => "black-boxes",
        Category::SingleLine => "single-line",
        Category::Nfpa => "nfpa-jic",
        Category::Fluid => "fluid-power-iso-1219",
        Category::Process => "p-and-id-iso-10628",
        Category::Installation => "installation-plan-iec-60617-11",
        Category::Gb => "gb-t-4728",
        Category::Gost => "gost-eskd",
        Category::Jis => "jis-c-0617",
        Category::User => "user",
    }
}

/// The symbols of a category, in the order of the library.
pub fn of(category: Category) -> Vec<&'static SymbolDef> {
    library().iter().filter(|def| def.category == category).collect()
}

/// The categories that have symbols.
pub fn categories() -> Vec<Category> {
    Category::ALL.into_iter().filter(|c| !of(*c).is_empty()).collect()
}

/// The file of the drawing of a symbol below [`FOLDER`].
pub fn drawing_file(def: &SymbolDef) -> String {
    let name: String = def.key.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect();
    format!("art/{name}.svg")
}

/// The symbol alone on white, as the SVG export draws it.
pub fn drawing(def: &SymbolDef) -> String {
    let (min, max) = (def.bounds.min, def.bounds.max);
    let (w, h) = ((max.x - min.x).max(1.0) + 2.0 * MARGIN, (max.y - min.y).max(1.0) + 2.0 * MARGIN);
    let symbol = SymbolInst { sym: def.key.to_string(), pos: p(MARGIN - min.x, MARGIN - min.y), pins: def.pins.iter().map(|pin| pin.label.to_string()).collect(), ..SymbolInst::default() };
    drawing_svg(w, h, |d| draw_symbol(d, def, &symbol, Ink::Line))
}

fn card(def: &SymbolDef, place: Place) -> String {
    let name = tr(def.name);
    let mut facts = format!("<dt>{}</dt><dd><code>{}</code></dd>", escape(tr("Key")), escape(def.key));
    if !def.letter.is_empty() {
        facts.push_str(&format!("<dt>{}</dt><dd>{}</dd>", escape(tr("Identifying letter")), escape(def.letter)));
    }
    if !def.pins.is_empty() {
        facts.push_str(&format!("<dt>{}</dt><dd>{}</dd>", escape(tr("Connection points")), def.pins.len()));
    }
    format!(
        "        <article class=\"card symbol\" id=\"{key}\"><div class=\"symbol-art\"><img loading=\"lazy\" src=\"{src}\" alt=\"{alt}\" /></div><h3>{name}</h3><dl>{facts}</dl></article>\n",
        key = attribute(def.key),
        src = place.file(&format!("{FOLDER}/{}", drawing_file(def))),
        alt = attribute(&tr_fmt!("Symbol: {}", name)),
        name = escape(name),
    )
}

fn actions(place: Place) -> String {
    format!(
        "<a class=\"button primary\" href=\"{}\">{}</a><a class=\"button\" data-link=\"editor\" href=\"#\">{}</a>",
        place.to("#studio"),
        escape(tr("Try the live demo")),
        escape(tr("Open the web editor"))
    )
}

/// The page of a category in `lang`.
pub fn category_page(category: Category, lang: Lang) -> Page {
    let folder = format!("{FOLDER}/{}", slug(category));
    let place = Place::of(&folder, lang);
    let symbols = of(category);
    let title = category.title();
    let names: Vec<&str> = symbols.iter().map(|def| tr(def.name)).collect();
    let lead = tr_fmt!("{} symbols of the category {} with their drawings, identifying letters and connection points. They come with OpenECraft, free to use in your schematics.", symbols.len(), title);
    let mut main = hero(tr("Symbol library"), title, &lead, &actions(place));
    main.push_str(&format!("\n<section class=\"band\">\n    <h2>{}</h2>\n    <p class=\"sub\">{}</p>\n    <div class=\"cards symbols\">\n", escape(&tr_fmt!("{} symbols", symbols.len())), escape(&names.join(" · "))));
    for def in &symbols {
        main.push_str(&card(def, place));
    }
    main.push_str("    </div>\n</section>\n\n");
    main.push_str(&others(Some(category), place));
    Page {
        title: crate::frame::title(tr_fmt!("{} – schematic symbols | OpenECraft", title), title),
        description: shortened(&tr_fmt!("{} electrical symbols of the category {}, drawn by OpenECraft, the free open source electrical CAE: {}", symbols.len(), title, names.join(", ")), DESCRIPTION.end - 1),
        folder,
        main,
        kind: Kind::Plain,
        langs: Langs::All,
    }
}

/// The categories as cards, without `current`.
fn others(current: Option<Category>, place: Place) -> String {
    let heading = if current.is_some() { tr("More symbols") } else { tr("Categories") };
    let mut out = format!("<section class=\"band\">\n    <h2>{}</h2>\n    <div class=\"cards\">\n", escape(heading));
    for category in categories().into_iter().filter(|c| Some(*c) != current) {
        let symbols = of(category);
        let sample: Vec<&str> = symbols.iter().take(4).map(|def| tr(def.name)).collect();
        out.push_str(&format!(
            "        <article class=\"card\"><h3>{}</h3><p>{}</p><a class=\"button\" href=\"{}\">{}</a></article>\n",
            escape(category.title()),
            escape(&format!("{} …", sample.join(" · "))),
            place.to(&format!("{FOLDER}/{}/", slug(category))),
            escape(&tr_fmt!("{} symbols", symbols.len()))
        ));
    }
    out.push_str("    </div>\n</section>\n");
    out
}

/// The overview of the library in `lang`.
pub fn overview(lang: Lang) -> Page {
    let place = Place::of(FOLDER, lang);
    let count = library().len();
    let lead = tr_fmt!(
        "The {} symbols that come with OpenECraft: IEC 60617 for schematics, NFPA, GB, GOST and JIS sets, fluid power after ISO 1219, P&ID and installation plans. Pick a category to see every symbol.",
        count
    );
    let mut main = hero(tr("Symbol library"), tr("Electrical symbols for schematics"), &lead, &actions(place));
    main.push('\n');
    main.push_str(&others(None, place));
    Page {
        folder: FOLDER.to_string(),
        title: tr("Electrical symbols for schematics – OpenECraft").to_string(),
        description: tr("The symbol library of OpenECraft, the free open source electrical CAE: IEC 60617, NFPA, GB, GOST, JIS, fluid power, P&ID and installation plan symbols.").to_string(),
        main,
        kind: Kind::Plain,
        langs: Langs::All,
    }
}

/// The pages of the library in every language, with the drawings of the symbols.
pub fn site() -> Vec<File> {
    let mut out = files(Langs::All, overview);
    for category in categories() {
        out.extend(files(Langs::All, |lang| category_page(category, lang)));
        out.extend(of(category).into_iter().map(|def| File::text(format!("{FOLDER}/{}", drawing_file(def)), drawing(def))));
    }
    out
}

#[cfg(test)]
mod tests;
