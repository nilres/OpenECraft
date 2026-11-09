use super::*;

#[derive(Default)]
struct Rec {
    rects: Vec<(P, P)>,
    texts: Vec<(P, String, f32, Anchor)>,
    styled: Vec<(P, String, TextStyle)>,
    polys: Vec<(Vec<P>, Ink, bool)>,
    urls: Vec<(P, P, String)>,
    lines: Vec<Ink>,
}

impl Draw for Rec {
    fn line(&mut self, _: P, _: P, _: f32, ink: Ink, _: bool) {
        self.lines.push(ink);
    }
    fn poly(&mut self, pts: &[P], _: bool, _: f32, ink: Ink, fill: bool) {
        self.polys.push((pts.to_vec(), ink, fill));
    }
    fn circle(&mut self, _: P, _: f32, _: f32, _: Ink, _: bool) {}
    fn text(&mut self, pos: P, text: &str, size: f32, h: Anchor, _: VAnchor, _: Ink) {
        self.texts.push((pos, text.into(), size, h));
    }
    fn styled_text(&mut self, pos: P, text: &str, style: &TextStyle, ink: Ink) {
        self.styled.push((pos, text.into(), *style));
        self.text(pos, text, style.size, style.h, style.v, ink);
    }
    fn rect(&mut self, a: P, b: P, _: f32, _: Ink, _: bool) {
        self.rects.push((a, b));
    }
    fn link_url(&mut self, min: P, max: P, url: &str) {
        self.urls.push((min, max, url.into()));
    }
}

#[test]
fn transformed_imported_labels_stay_upright_at_their_pin() {
    let label = Prim::Text { pos: p(2., 0.), text: "U1".into(), size: 2., anchor: Anchor::Left, angle: Some(0.), color: None };
    for (rot, mirror, anchor) in [(0, true, Anchor::Right), (2, false, Anchor::Right), (2, true, Anchor::Left)] {
        let s = SymbolInst { rot, mirror, ..Default::default() };
        let mut draw = Rec::default();
        draw_prims(&mut draw, std::slice::from_ref(&label), &s, Ink::Line);
        let (pos, _, style) = &draw.styled[0];
        assert_eq!(*pos, p(2., 0.).transform(rot, mirror));
        assert_eq!((style.angle, style.h), (0., anchor));
    }
}

#[test]
fn free_texts_are_drawn_with_style_background_and_link() {
    let mut prj = Project::default();
    prj.info.name = "Pump station".into();
    prj.pages[0].texts.push(TextItem {
        id: 1,
        pos: p(100.0, 100.0),
        text: "{project}\nline 2".into(),
        angle: 90.0,
        bold: true,
        background: true,
        frame: true,
        link: "https://example.org".into(),
        ..Default::default()
    });
    let an = Analysis::with_builtin(&prj);
    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, 0, &Opts::default());
    let mine: Vec<_> = d.styled.iter().filter(|(_, _, st)| st.angle == 90.0).collect();
    assert_eq!(mine.len(), 2);
    assert_eq!(mine[0].1, "Pump station", "placeholders are filled in");
    assert!(mine[0].2.bold && mine[0].2.v == VAnchor::Middle);
    assert!((mine[1].0.x - 104.2).abs() < 0.01 && (mine[1].0.y - 100.0).abs() < 0.01, "second line to the right: {:?}", mine[1].0);
    assert!(d.polys.iter().any(|(pts, ink, fill)| pts.len() == 4 && *ink == Ink::Paper && *fill), "background");
    assert!(d.polys.iter().any(|(pts, ink, fill)| pts.len() == 4 && *ink == Ink::Line && !*fill), "frame");
    assert_eq!(d.urls.len(), 1);
    assert_eq!(d.urls[0].2, "https://example.org");
    let b = text_bounds(&prj, 0, &prj.pages[0].texts[0]);
    assert!((b.min.x - 97.9).abs() < 0.01 && (b.max.x - 106.3).abs() < 0.01, "{b:?}");
    assert!((b.min.y - 77.68).abs() < 0.01 && (b.max.y - 100.0).abs() < 0.01, "{b:?}");
}

#[test]
fn cover_rows_sit_centred_inside_their_box() {
    let mut prj = crate::sample::demo_project();
    prj.info.company = "Schaltanlagenbau Musterstadt GmbH & Co. KG, Werk Nord".into();
    let cover = prj.pages.iter().position(|pg| pg.kind == PageKind::Cover).unwrap();
    let mut d = Rec::default();
    draw_cover(&mut d, &prj, &crate::frame::of_page(&prj, cover));
    let (a, b) = *d.rects.last().unwrap();
    let rows: Vec<_> = d.texts.iter().filter(|t| t.0.y > a.y).collect();
    assert_eq!(rows.len(), 10);
    for (pos, text, size, h) in &rows {
        let w = text_width(text, *size);
        let (x0, x1) = match h {
            Anchor::Left => (pos.x, pos.x + w),
            Anchor::Right => (pos.x - w, pos.x),
            Anchor::Center => (pos.x - w / 2.0, pos.x + w / 2.0),
        };
        assert!(x0 > a.x && x1 < b.x, "{text} sticks out of the box");
        assert!(pos.y - size / 2.0 > a.y && pos.y + size / 2.0 < b.y, "{text} overlaps the box");
    }
    let first = rows.iter().map(|t| t.0.y).fold(f32::INFINITY, f32::min);
    let last = rows.iter().map(|t| t.0.y).fold(f32::NEG_INFINITY, f32::max);
    assert!(((first - a.y) - (b.y - last)).abs() < 0.01);
}

#[test]
fn graphical_terminal_and_cable_diagrams() {
    use crate::forms::ReportForm;
    use crate::model::ReportKind;
    let mut prj = crate::sample::demo_project();
    let data = prj.strips.entry("-X1".into()).or_default();
    data.jumpers.insert("1".into(), 1);
    data.jumpers.insert("2".into(), 1);
    for kind in [ReportKind::TerminalDiagram, ReportKind::CableDiagram] {
        prj.forms.reports.insert(kind.into(), ReportForm { graphic: true, ..Default::default() });
    }
    let at = prj.pages.len();
    crate::ops::add_page(&mut prj, PageKind::Report(ReportKind::TerminalDiagram.into()), at);
    crate::ops::add_page(&mut prj, PageKind::Report(ReportKind::CableDiagram.into()), at + 1);
    let an = Analysis::with_builtin(&prj);
    let table = crate::report_blocks::page_table(&prj, &an, at).unwrap();
    assert!(table.graphic);
    assert_eq!(table.per_page, crate::report_blocks::GRAPHIC_ROWS_PER_PAGE);

    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, at, &Opts::default());
    let texts: Vec<&str> = d.texts.iter().map(|t| t.1.as_str()).collect();
    assert!(texts.contains(&"-X1"), "strip heading");
    for label in ["1", "2", "3", "PE"] {
        assert!(texts.contains(&label), "terminal {label}");
    }
    assert!(texts.iter().any(|t| t.starts_with("Plug-in jumper")), "jumper listed below: {texts:?}");
    // A box per terminal (4) and a bar per end bracket / end plate (4), and the frame.
    assert!(d.rects.len() >= 8, "{} boxes", d.rects.len());
    assert!(!texts.contains(&"External (top)"), "no table header");

    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, at + 1, &Opts::default());
    let texts: Vec<&str> = d.texts.iter().map(|t| t.1.as_str()).collect();
    assert!(texts.iter().any(|t| t.starts_with("-W1")), "cable band heading: {texts:?}");

    // Without the switch the table is drawn.
    prj.forms.reports.clear();
    let an = Analysis::with_builtin(&prj);
    assert!(!crate::report_blocks::page_table(&prj, &an, at).unwrap().graphic);
    assert!(crate::report_blocks::has_graphic(ReportKind::CableDiagram) && !crate::report_blocks::has_graphic(ReportKind::DeviceList));
}

#[test]
fn graphical_plug_diagram_and_plc_assignment() {
    use crate::forms::ReportForm;
    use crate::model::ReportKind;
    let mut prj = crate::sample::demo_project();
    for kind in [ReportKind::PlugDiagram, ReportKind::PlcAssignment] {
        prj.forms.reports.insert(kind.into(), ReportForm { graphic: true, ..Default::default() });
    }
    let at = prj.pages.len();
    crate::ops::add_page(&mut prj, PageKind::Report(ReportKind::PlugDiagram.into()), at);
    crate::ops::add_page(&mut prj, PageKind::Report(ReportKind::PlcAssignment.into()), at + 1);
    let an = Analysis::with_builtin(&prj);
    for (page, heading) in [(at, "Socket side"), (at + 1, "Inputs from")] {
        let table = crate::report_blocks::page_table(&prj, &an, page).unwrap();
        assert!(table.graphic);
        let mut d = Rec::default();
        draw_page(&mut d, &prj, &an.index, &an, page, &Opts::default());
        let texts: Vec<&str> = d.texts.iter().map(|t| t.1.as_str()).collect();
        assert!(texts.contains(&heading), "{texts:?}");
        if let Some(first) = table.rows.first() {
            assert!(texts.contains(&first[0].as_str()), "group heading {}", first[0]);
        }
    }
    let plc = crate::report_blocks::page_table(&prj, &an, at + 1).unwrap();
    assert!(!plc.rows.is_empty(), "the demo has PLC points");
    assert!(crate::report_blocks::has_graphic(ReportKind::PlugDiagram) && crate::report_blocks::has_graphic(ReportKind::PlcAssignment));
}

#[test]
fn mounting_pages_show_every_face_with_items() {
    let mut prj = crate::sample::demo_project();
    let pi = prj.pages.iter().position(|p| matches!(p.kind, PageKind::Mounting(_))).unwrap();
    let PageKind::Mounting(id) = prj.pages[pi].kind else { unreachable!() };
    let e = prj.layout.enclosures.iter_mut().find(|e| e.id == id).unwrap();
    let mut fan = e.items.iter().find(|i| i.face == crate::panel::Face::Plate).cloned().unwrap();
    fan.id = 987_654;
    fan.face = crate::panel::Face::Roof;
    fan.x = 10.0;
    fan.y = 10.0;
    e.items.push(fan);
    let an = Analysis::checked(&prj);
    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, pi, &Opts::default());
    let texts: Vec<&str> = d.texts.iter().map(|t| t.1.as_str()).collect();
    assert!(texts.contains(&"Mounting plate") && texts.contains(&"Roof"), "{texts:?}");
}

#[test]
fn a_row_of_terminals_shows_the_strip_tag_once() {
    let mut prj = Project::default();
    for (i, x) in [100.0, 112.0, 124.0, 200.0].into_iter().enumerate() {
        let label = (i + 1).to_string();
        prj.pages[0].symbols.push(SymbolInst { id: 10 + i as u64, sym: "term".into(), pos: p(x, 100.0), tag: "-X1".into(), label, ..Default::default() });
    }
    let an = Analysis::with_builtin(&prj);
    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, 0, &Opts::default());
    let tags: Vec<f32> = d.texts.iter().filter(|t| t.1 == "-X1").map(|t| t.0.x).collect();
    assert_eq!(tags.len(), 2, "first terminal of each row: {tags:?}");
    assert!(tags.iter().all(|x| *x < 100.0 || (*x > 180.0 && *x < 200.0)), "{tags:?}");
}

fn spd(id: u64, x: f32, tag: &str, function_text: &str) -> SymbolInst {
    SymbolInst { id, sym: "spd".into(), pos: p(x, 180.0), tag: tag.into(), function_text: function_text.into(), pins: vec!["L1".into(), "PE".into()], ..Default::default() }
}

/// Area of each placed symbol except `s`.
fn other_bodies(prj: &Project, s: &SymbolInst) -> Vec<R> {
    prj.pages[0].symbols.iter().filter(|o| o.id != s.id).map(|o| prj.symbol(&o.sym).unwrap().world_bounds(o.pos, o.rot, o.mirror)).collect()
}

#[test]
fn adjacent_functions_of_one_device_show_the_tag_once() {
    // Example 13: four surge protective devices 8 mm apart, all -F1.
    let mut prj = Project::default();
    for (i, x) in [140.0, 148.0, 156.0, 164.0].into_iter().enumerate() {
        prj.pages[0].symbols.push(spd(10 + i as u64, x, "-F1", if i == 0 { "Surge protection" } else { "" }));
    }
    let an = Analysis::with_builtin(&prj);
    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, 0, &Opts::default());
    let tags: Vec<f32> = d.texts.iter().filter(|t| t.1 == "-F1").map(|t| t.0.x).collect();
    assert_eq!(tags.len(), 1, "{tags:?}");
    assert!(tags[0] < 140.0, "{tags:?}");
    for s in &prj.pages[0].symbols {
        let def = prj.symbol(&s.sym).unwrap();
        let block = tag_block_bounds(&prj, 0, &def, s).unwrap();
        assert!(other_bodies(&prj, s).iter().all(|b| !b.overlaps(&block)), "{}: {block:?}", s.pos.x);
    }
    // Far enough apart, each shows its tag.
    prj.pages[0].symbols[1].pos.x = 160.0;
    prj.pages[0].symbols.truncate(2);
    let an = Analysis::with_builtin(&prj);
    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, 0, &Opts::default());
    assert_eq!(d.texts.iter().filter(|t| t.1 == "-F1").count(), 2);
}

#[test]
fn a_tag_block_moves_to_the_side_clear_of_neighbouring_symbols() {
    let mut prj = Project::default();
    prj.pages[0].symbols.push(spd(10, 100.0, "-F1", ""));
    prj.pages[0].symbols.push(spd(11, 108.0, "-F2", ""));
    let def = prj.symbol("spd").unwrap();
    let s = prj.pages[0].symbols[1].clone();
    let block = tag_block_bounds(&prj, 0, &def, &s).unwrap();
    assert!(block.min.x > 108.0, "right of -F2: {block:?}");
    assert!(other_bodies(&prj, &s).iter().all(|b| !b.overlaps(&block)));
    let an = Analysis::with_builtin(&prj);
    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, 0, &Opts::default());
    let f2 = d.texts.iter().find(|t| t.1 == "-F2").unwrap();
    assert!(f2.0.x > 108.0 && f2.3 == Anchor::Left, "{f2:?}");
    // The first keeps its usual place on the left.
    let first = tag_block_bounds(&prj, 0, &def, &prj.pages[0].symbols[0]).unwrap();
    assert!(first.max.x < 100.0, "{first:?}");
    // With symbols on both sides it goes above, unless a wire runs there too.
    prj.pages[0].symbols.push(spd(12, 116.0, "-F3", ""));
    let block = tag_block_bounds(&prj, 0, &def, &prj.pages[0].symbols[1]).unwrap();
    assert!(block.max.y < 176.0 && block.min.x < 108.0 && block.max.x > 108.0, "{block:?}");
    prj.pages[0].wires.push(Wire { id: 20, a: p(108.0, 160.0), b: p(108.0, 176.0), ..Default::default() });
    prj.pages[0].wires.push(Wire { id: 21, a: p(108.0, 184.0), b: p(108.0, 200.0), ..Default::default() });
    let block = tag_block_bounds(&prj, 0, &def, &prj.pages[0].symbols[1]).unwrap();
    assert!(block.max.x < 108.0, "stays on its usual side: {block:?}");
}

#[test]
fn a_tag_block_placed_by_hand_stays_where_it_is() {
    let mut prj = Project::default();
    prj.pages[0].symbols.push(spd(10, 100.0, "-F1", ""));
    let mut s = spd(11, 108.0, "-F1", "Second");
    s.tag_offset = Some(p(-9.0, -1.0));
    prj.pages[0].symbols.push(s.clone());
    let def = prj.symbol("spd").unwrap();
    let block = tag_block_bounds(&prj, 0, &def, &s).unwrap();
    assert_eq!(block.min, p(99.0, 179.0));
    let an = Analysis::with_builtin(&prj);
    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, 0, &Opts::default());
    assert_eq!(d.texts.iter().filter(|t| t.1 == "-F1").count(), 2, "the tag placed by hand is shown");
}

#[test]
fn the_contact_image_of_a_turned_coil_stays_clear_of_its_pins() {
    let mut prj = Project::default();
    let sym = |id: u64, key: &str, pos: P, rot: u8, pins: &[&str]| SymbolInst {
        id,
        sym: key.into(),
        pos,
        rot,
        tag: "-K1".into(),
        pins: pins.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    };
    prj.pages[0].symbols.push(sym(1, "coil", p(200.0, 100.0), 1, &["A1", "A2"]));
    prj.pages[0].symbols.push(sym(2, "no", p(100.0, 60.0), 0, &["13", "14"]));
    let an = Analysis::with_builtin(&prj);
    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, 0, &Opts::default());
    let def = prj.symbol("coil").unwrap();
    let b = def.world_bounds(p(200.0, 100.0), 1, false);
    // The rows of the contact image ("13", "14") lie below the coil, nothing of it
    // beside the coil where A1 and A2 are.
    let near = |t: &&(P, String, f32, Anchor)| (t.0.x - b.center().x).abs() < 20.0;
    let image: Vec<_> = d.texts.iter().filter(near).filter(|t| t.1 == "13" || t.1 == "14").collect();
    assert_eq!(image.len(), 2, "{:?}", d.texts);
    assert!(image.iter().all(|t| t.0.y > b.max.y), "{image:?} {b:?}");
}

#[test]
fn black_wires_are_drawn_black_not_in_the_wire_ink() {
    let mut prj = Project { wire_colors: true, ..Default::default() };
    let black = crate::potentials::color_rgb("BK").unwrap();
    let red = crate::potentials::color_rgb("RD").unwrap();
    prj.pages[0].wires.push(Wire { id: 1, a: p(100.0, 100.0), b: p(200.0, 100.0), color: "BK".into(), ..Default::default() });
    prj.pages[0].wires.push(Wire { id: 2, a: p(100.0, 150.0), b: p(200.0, 150.0), color: "RD".into(), ..Default::default() });
    let an = Analysis::with_builtin(&prj);
    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, 0, &Opts::default());
    assert!(d.lines.contains(&Ink::Color(black)), "{:?}", d.lines);
    assert!(d.lines.contains(&Ink::Color(red)));
    assert!(!d.lines.contains(&Ink::Wire), "no wire falls back to the wire ink");
}

#[test]
fn only_web_links_are_exported() {
    let mut prj = Project::default();
    for (id, link) in [(1, "javascript:alert(1)"), (2, "file:///C:/Windows/System32/calc.exe"), (3, "mailto:anna@example.org")] {
        prj.pages[0].texts.push(TextItem { id, pos: p(100.0, 100.0 + id as f32 * 10.0), text: "x".into(), link: link.into(), ..Default::default() });
    }
    let an = Analysis::with_builtin(&prj);
    let mut d = Rec::default();
    draw_page(&mut d, &prj, &an.index, &an, 0, &Opts::default());
    let urls: Vec<&str> = d.urls.iter().map(|u| u.2.as_str()).collect();
    assert_eq!(urls, ["mailto:anna@example.org"]);
}
