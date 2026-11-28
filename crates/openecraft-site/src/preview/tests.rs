use super::*;
use openecraft_core::symbols::library;
use std::collections::BTreeSet;

/// Width, height, bit depth and colour type of a PNG file.
fn header(png: &[u8]) -> (u32, u32, u8, u8) {
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n") && &png[12..16] == b"IHDR", "a PNG file");
    let number = |at: usize| u32::from_be_bytes([png[at], png[at + 1], png[at + 2], png[at + 3]]);
    (number(16), number(20), png[24], png[25])
}

#[test]
fn a_page_of_a_symbol_a_category_or_an_example_has_a_picture_of_its_own() {
    assert_eq!(of("symbols/no-contact").as_deref(), Some("symbols/preview/no.png"));
    assert_eq!(of("symbols/no-contact/").as_deref(), Some("symbols/preview/no.png"));
    assert_eq!(of("symbols/contacts").as_deref(), Some("symbols/contacts/preview.png"));
    assert_eq!(of("examples/lamp-circuit").as_deref(), Some("examples/lamp-circuit/preview.png"));
    // The other pages have the picture of the site.
    for folder in ["", "symbols", "examples", "examples/no-such-example", "symbols/no-such-symbol", "faq", "docs/tutorial", "compare/eplan-alternative", "news/v0-1-2"] {
        assert_eq!(of(folder), None, "{folder}");
    }
    // Every symbol has a file of its own, beside the drawings and not in the folder of a page.
    let files: BTreeSet<String> = library().iter().map(symbol_file).collect();
    assert_eq!(files.len(), library().len());
    for def in library() {
        assert_eq!(of(&format!("symbols/{}", symbols::symbol_slug(def))), Some(symbol_file(def)), "{}", def.key);
        assert_ne!(symbols::symbol_slug(def), SYMBOLS, "{}: the folder of the pictures", def.key);
    }
    assert!(!symbols::is_category(SYMBOLS));
    for example in &EXAMPLES {
        assert!(of(&format!("examples/{}", examples::slug(example))).is_some(), "{}", example.key);
    }
}

#[test]
fn a_long_title_is_set_in_lines() {
    assert_eq!(wrapped("NO contact", 15, 4), ["NO contact"]);
    assert_eq!(wrapped("Circuit breaker with auxiliary contact", 15, 4), ["Circuit breaker", "with auxiliary", "contact"]);
    assert_eq!(wrapped("one two three four five six", 7, 2), ["one two", "three…"]);
    assert!(wrapped("", 15, 4).is_empty());
}

#[test]
fn a_card_shows_the_drawing_with_what_the_page_is_about() {
    let def = symbols::by_folder("no-contact").unwrap();
    let card = symbol_card(def);
    assert!(card.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"630\" viewBox=\"0 0 1200 630\""));
    assert!(card.contains(">OpenECraft</text>") && card.contains(">NO contact</text>") && card.contains(">Electrical symbol · Contacts</text>"));
    // The drawing of the symbol library, fitted into the sheet.
    let drawing = symbols::drawing(def);
    let view = drawing.split("viewBox=\"").nth(1).and_then(|rest| rest.split('"').next()).unwrap();
    assert!(card.contains(&format!("<svg x=\"452.0\" y=\"88.0\" width=\"660.0\" height=\"454.0\" viewBox=\"{view}\" preserveAspectRatio=\"xMidYMid meet\">")));
    assert_eq!(card.matches("</svg>").count(), 2);
    // In English whatever the language of the build is at the moment: one picture
    // serves the pages of all languages.
    let german = openecraft_i18n::with_lang(Lang::De, || symbol_card(def));
    assert_eq!(german, card);
    // A category shows its first symbols, an example its sheet.
    let contacts = category_card(Category::Contacts);
    assert_eq!(contacts.matches("preserveAspectRatio").count(), symbols::of(Category::Contacts).len().min(GRID.0 * GRID.1));
    assert!(contacts.contains(">Contacts</text>") && contacts.contains(&format!(">{} electrical symbols</text>", symbols::of(Category::Contacts).len())));
    let lamp = EXAMPLES.iter().find(|example| example.key == "lamp_circuit").unwrap();
    let sheet = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"420mm\" height=\"297mm\" viewBox=\"0 0 420 297\"><rect width=\"420\" height=\"297\" fill=\"#fff\"/></svg>\n";
    let example = example_card(lamp, sheet);
    assert!(example.contains("viewBox=\"0 0 420 297\" preserveAspectRatio") && example.contains(">Example project, level 1</text>"));
    assert!(example.contains(">First circuit:</text>"), "the title without its number, in lines");
}

#[test]
fn a_card_is_drawn_as_a_picture_of_the_size_of_a_link_preview() {
    let def = symbols::by_folder("no-contact").unwrap();
    let picture = png(&symbol_card(def)).expect("a picture");
    assert_eq!(header(&picture), (1200, 630, 8, 3), "1200 × 630, a table of colours without transparency");
    assert!(picture.len() < 20_000, "{} bytes", picture.len());
    // The card is drawn: dark around the sheet, the white sheet, and the texts of the
    // card, which need the font to be there.
    let mut decoder = png::Decoder::new(std::io::Cursor::new(&picture));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    reader.next_frame(&mut pixels).unwrap();
    let pixel = |x: usize, y: usize| -> [u8; 3] { pixels[(y * 1200 + x) * 3..][..3].try_into().unwrap() };
    assert_eq!(pixel(5, 5), [0x07, 0x0b, 0x14]);
    assert_eq!(pixel(430, 70), [0xff, 0xff, 0xff]);
    let text = (140..330).flat_map(|x| (70..110).map(move |y| (x, y))).filter(|(x, y)| pixel(*x, *y)[0] > 0x80).count();
    assert!(text > 300, "the name of the project is written: {text} light pixels");
    let drawn = (452..1112).flat_map(|x| (88..542).map(move |y| (x, y))).filter(|(x, y)| pixel(*x, *y)[0] < 0x40).count();
    assert!(drawn > 1000, "the symbol is drawn: {drawn} dark pixels");
    assert!(png("<svg").is_err());
    // To look at it: OPENECRAFT_PREVIEW=<folder> writes the pictures of this test.
    if let Ok(folder) = std::env::var("OPENECRAFT_PREVIEW") {
        for (name, card) in [("symbol.png", symbol_card(def)), ("category.png", category_card(Category::Contacts))] {
            std::fs::write(std::path::Path::new(&folder).join(name), png(&card).unwrap()).unwrap();
        }
    }
}

#[test]
fn a_picture_has_a_table_of_its_colours() {
    // Few colours keep theirs, in a fixed order.
    let few = [[9, 9, 9], [0, 0, 0], [9, 9, 9], [255, 255, 255]];
    assert_eq!(indexed(&few), (vec![[0, 0, 0], [9, 9, 9], [255, 255, 255]], vec![1, 0, 1, 2]));
    // More than the table takes: colours that are nearly alike share a place, with
    // the colour most pixels have, so a large area keeps its colour exactly.
    let mut many: Vec<[u8; 3]> = (0..=255u8).flat_map(|grey| [[grey, grey, grey], [grey, 0, 0], [0, grey, 0]]).collect();
    many.extend(std::iter::repeat_n([7, 11, 20], 5000));
    many.extend(std::iter::repeat_n([255, 255, 255], 5000));
    let (table, places) = indexed(&many);
    assert!(table.len() <= 256 && table.len() > 64, "{} colours", table.len());
    assert_eq!(places.len(), many.len());
    assert!(table.contains(&[7, 11, 20]) && table.contains(&[255, 255, 255]));
    assert_eq!(table[places[many.len() - 1] as usize], [255, 255, 255]);
    assert_eq!(table[places[many.len() - 5001] as usize], [7, 11, 20]);
    for (pixel, place) in many.iter().zip(&places) {
        let shown = table[*place as usize];
        assert!((0..3).all(|i| pixel[i].abs_diff(shown[i]) < 8), "{pixel:?} is shown as {shown:?}");
    }
    assert_eq!(indexed(&many), (table, places), "the same picture is the same file");
}

#[test]
fn an_example_shows_its_first_schematic_page() {
    let lamp = EXAMPLES.iter().find(|example| example.key == "lamp_circuit").unwrap();
    let project = (lamp.build)();
    // Not the cover sheet the project starts with.
    assert!(matches!(project.pages[0].kind, PageKind::Cover) && sheet_of(&project) > 0);
    assert!(matches!(project.pages[sheet_of(&project)].kind, PageKind::Schematic));
    assert_eq!(sheet_of(&Project::default()), 0);
    let file = example(lamp).expect("a picture");
    assert_eq!(file.path, "examples/lamp-circuit/preview.png");
    assert_eq!(Some(file.path.clone()), of("examples/lamp-circuit"));
    assert_eq!(header(&file.bytes), (1200, 630, 8, 3));
    assert!(file.bytes.len() < 60_000, "{} bytes", file.bytes.len());
    if let Ok(folder) = std::env::var("OPENECRAFT_PREVIEW") {
        std::fs::write(std::path::Path::new(&folder).join("example.png"), &file.bytes).unwrap();
    }
}
