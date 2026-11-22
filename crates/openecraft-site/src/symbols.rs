//! The symbol library as pages: one for every category of the built-in symbols, each
//! symbol with its drawing (the symbol itself, drawn by the SVG export), its name in
//! the language of the page, its key, identifying letter and connection points, an
//! overview that lists the categories, and a page for every symbol with its drawing
//! to download as SVG and DXF ([`symbol_page`], in the required languages).

use crate::frame::{attribute, escape, files, hero, shortened, File, Kind, Langs, Page, Place, DESCRIPTION};
use openecraft_core::geom::p;
use openecraft_core::model::SymbolInst;
use openecraft_core::render::{draw_symbol, Ink};
use openecraft_core::dxf::drawing_dxf;
use openecraft_core::svg::drawing_svg;
use openecraft_core::symbols::{library, Category, SymbolDef};
use openecraft_i18n::{decimal, tr, tr_fmt, Lang};
use std::collections::BTreeMap;
use std::sync::OnceLock;

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

/// Whether `folder` below [`FOLDER`] is the page of a category (the pages of the
/// symbols lie beside them).
pub fn is_category(folder: &str) -> bool {
    Category::ALL.into_iter().any(|category| slug(category) == folder)
}

/// The folder of a symbol below [`FOLDER`]: its English name in lower case with
/// dashes (`no-contact`), which people search for. A name taken by a category, by the
/// drawings or by another symbol gets the key of the symbol.
pub fn symbol_slug(def: &SymbolDef) -> &'static str {
    static SLUGS: OnceLock<BTreeMap<&'static str, String>> = OnceLock::new();
    let slugs = SLUGS.get_or_init(|| {
        let plain = |text: &str| -> String {
            let dashed: String = text.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect();
            dashed.split('-').filter(|part| !part.is_empty()).collect::<Vec<_>>().join("-")
        };
        let mut count: BTreeMap<String, usize> = BTreeMap::new();
        for def in library() {
            *count.entry(plain(def.name)).or_default() += 1;
        }
        library()
            .iter()
            .map(|def| {
                let name = plain(def.name);
                let taken = name.is_empty() || name == "art" || is_category(&name) || count[&name] > 1;
                (def.key, if taken { plain(&format!("{name}-{}", def.key)) } else { name })
            })
            .collect()
    });
    slugs.get(def.key).map_or("", String::as_str)
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

/// The file of the symbol as a DXF drawing below [`FOLDER`].
pub fn dxf_file(def: &SymbolDef) -> String {
    drawing_file(def).replace(".svg", ".dxf")
}

/// The symbol alone as a DXF drawing in millimetres, as the DXF export draws it.
pub fn dxf(def: &SymbolDef) -> String {
    let (min, max) = (def.bounds.min, def.bounds.max);
    let symbol = SymbolInst { sym: def.key.to_string(), pos: p(MARGIN - min.x, MARGIN - min.y), pins: def.pins.iter().map(|pin| pin.label.to_string()).collect(), ..SymbolInst::default() };
    drawing_dxf((max.y - min.y).max(1.0) + 2.0 * MARGIN, |d| draw_symbol(d, def, &symbol, Ink::Line))
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
        "        <article class=\"card symbol\" id=\"{key}\"><a class=\"symbol-art\" href=\"{page}\"><img loading=\"lazy\" src=\"{src}\" alt=\"{alt}\" /></a><h3><a href=\"{page}\">{name}</a></h3><dl>{facts}</dl></article>\n",
        key = attribute(def.key),
        page = place.to(&format!("{FOLDER}/{}/", symbol_slug(def))),
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

/// The page of a symbol in `lang`: its drawing to look at and to download, what the
/// library knows about it and the other symbols of its category.
pub fn symbol_page(def: &SymbolDef, lang: Lang) -> Page {
    let folder = format!("{FOLDER}/{}", symbol_slug(def));
    let place = Place::of(&folder, lang);
    let (name, category) = (tr(def.name), def.category.title());
    // Two symbols of one name in this language are told apart in the title.
    let same: Vec<&SymbolDef> = library().iter().filter(|other| tr(other.name) == name).collect();
    let label = match same.len() {
        1 => name.to_string(),
        _ if same.iter().filter(|other| other.category == def.category).count() == 1 => format!("{name} ({category})"),
        _ => format!("{name} ({})", def.key),
    };
    let (svg, dxf) = (place.file(&format!("{FOLDER}/{}", drawing_file(def))), place.file(&format!("{FOLDER}/{}", dxf_file(def))));
    let lead = tr_fmt!("The symbol {} of the category {} from the symbol library of OpenECraft: its drawing, free to download as SVG and DXF, and its data.", name, category);
    let mut main = format!(
        "<section class=\"hero page-hero\">\n    <div class=\"hero-text\">\n        <p class=\"eyebrow\"><a href=\"{}\">{}</a> › <a href=\"{}\">{}</a></p>\n        <h1>{}</h1>\n        <p class=\"lead\">{}</p>\n        <div class=\"actions\"><a class=\"button primary\" href=\"{svg}\" download>{}</a><a class=\"button\" href=\"{dxf}\" download>{}</a><a class=\"button ghost\" href=\"{}\">{}</a></div>\n    </div>\n</section>\n",
        place.to(&format!("{FOLDER}/")),
        escape(tr("Symbol library")),
        place.to(&format!("{FOLDER}/{}/", slug(def.category))),
        escape(category),
        escape(name),
        escape(&lead),
        escape(tr("Download as SVG")),
        escape(tr("Download as DXF")),
        place.to("#studio"),
        escape(tr("Try the live demo")),
    );
    let mut facts = format!("<dt>{}</dt><dd><code>{}</code></dd>", escape(tr("Key")), escape(def.key));
    if !def.letter.is_empty() {
        facts.push_str(&format!("<dt>{}</dt><dd>{}</dd>", escape(tr("Identifying letter")), escape(def.letter)));
    }
    if !def.pins.is_empty() {
        let labels: Vec<&str> = def.pins.iter().map(|pin| pin.label).filter(|label| !label.is_empty()).collect();
        let named = if labels.is_empty() { String::new() } else { format!(" ({})", escape(&labels.join(", "))) };
        facts.push_str(&format!("<dt>{}</dt><dd>{}{named}</dd>", escape(tr("Connection points")), def.pins.len()));
    }
    let (w, h) = (def.bounds.max.x - def.bounds.min.x, def.bounds.max.y - def.bounds.min.y);
    facts.push_str(&format!("<dt>{}</dt><dd>{} × {} mm</dd>", escape(tr("Size")), decimal(w as f64, 1), decimal(h as f64, 1)));
    facts.push_str(&format!("<dt>{}</dt><dd><a href=\"{}\">{}</a></dd>", escape(tr("Category")), place.to(&format!("{FOLDER}/{}/", slug(def.category))), escape(category)));
    main.push_str(&format!(
        "\n<section class=\"band\">\n    <h2>{}</h2>\n    <div class=\"symbol-sheet\">\n        <div class=\"symbol-art\"><img src=\"{svg}\" alt=\"{}\" /></div>\n        <dl>{facts}</dl>\n    </div>\n    <p class=\"text\">{}</p>\n    <p class=\"fine\">{}</p>\n</section>\n",
        escape(tr("Drawing and data")),
        attribute(&tr_fmt!("Symbol: {}", name)),
        escape(tr("In OpenECraft the symbol is placed from the Insert center and wired on the page; with a part it becomes a device of the reports and the checks.")),
        escape(tr("The drawing is in millimetres, as on a schematic page. The symbols of OpenECraft are free to use, also in commercial documents.")),
    ));
    let others: Vec<&SymbolDef> = of(def.category).into_iter().filter(|other| other.key != def.key).collect();
    if !others.is_empty() {
        main.push_str(&format!("\n<section class=\"band\">\n    <h2>{}</h2>\n    <p class=\"symbol-links\">\n", escape(&tr_fmt!("More symbols of the category {}", category))));
        for other in others {
            main.push_str(&format!("        <a href=\"{}\">{}</a>\n", place.to(&format!("{FOLDER}/{}/", symbol_slug(other))), escape(tr(other.name))));
        }
        main.push_str("    </p>\n</section>\n");
    }
    Page {
        title: crate::frame::title(tr_fmt!("{} – electrical symbol, SVG and DXF | OpenECraft", label), &label),
        description: shortened(&tr_fmt!("{}: electrical symbol of the category {} with its drawing, identifying letter and connection points. Free to download as SVG and DXF.", name, category), DESCRIPTION.end - 1),
        folder,
        main,
        kind: Kind::Plain,
        langs: Langs::Required,
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

/// The pages of the library in every language, with the drawings of the symbols and
/// the page of every symbol.
pub fn site() -> Vec<File> {
    let mut out = files(Langs::All, overview);
    for category in categories() {
        out.extend(files(Langs::All, |lang| category_page(category, lang)));
        for def in of(category) {
            out.push(File::text(format!("{FOLDER}/{}", drawing_file(def)), drawing(def)));
            out.push(File::text(format!("{FOLDER}/{}", dxf_file(def)), dxf(def)));
            out.extend(files(Langs::Required, |lang| symbol_page(def, lang)));
        }
    }
    out
}

#[cfg(test)]
mod tests;
