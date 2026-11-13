use super::*;

#[test]
fn annotation_bounds_include_render_scale_and_wide_glyphs() {
    let size=1.8;
    let expected=(778.+722.+667.+667.)*size*1.25/1000.;
    assert!((annotation_width("GNYE",size)-expected).abs()<0.001);
    assert!(annotation_width("WWWW",size)>annotation_width("IIII",size));
    assert!(annotation_width("WWWW",size)>4.*size*1.25*0.94);
    assert!(annotation_width("GNYE",size)>est_width("GNYE",size)*1.4);
}

fn close(a: P, b: P) -> bool {
    (a.x - b.x).abs() < 0.01 && (a.y - b.y).abs() < 0.01
}

#[test]
fn rotation_is_counter_clockwise_on_the_page() {
    assert!(close(rotate(p(1.0, 0.0), 90.0), p(0.0, -1.0)), "right turns into up");
    assert!(close(rotate(p(0.0, 1.0), 90.0), p(1.0, 0.0)), "down turns into right");
    assert!(close(rotate(p(1.0, 0.0), 180.0), p(-1.0, 0.0)));
    assert_eq!(rotate(p(3.0, 4.0), 0.0), p(3.0, 4.0));
}

#[test]
fn wrapping_keeps_line_breaks_and_long_words() {
    assert_eq!(wrap("a\nb", 3.0, 0.0), ["a", "b"]);
    assert_eq!(wrap("", 3.0, 0.0), [""]);
    // 0.62 × 3 mm = 1.86 mm per character: 20 mm hold 10 characters.
    assert_eq!(wrap("one two three four", 3.0, 20.0), ["one two", "three four"]);
    assert_eq!(wrap("extraordinarily long", 3.0, 20.0), ["extraordinarily", "long"]);
    assert_eq!(wrap("short\nlonger line here", 3.0, 20.0), ["short", "longer", "line here"]);
}

#[test]
fn lines_follow_the_anchor_and_the_rotation() {
    let t = TextItem { pos: p(100.0, 50.0), text: "a\nb".into(), size: 3.0, ..Default::default() };
    let ls = lines(&t, &t.text);
    assert_eq!(ls.len(), 2);
    assert_eq!(ls[0].0, p(100.0, 50.0), "the first line sits at the position (as before)");
    assert!(close(ls[1].0, p(100.0, 50.0 + 4.2)));

    let middle = TextItem { valign: VAnchor::Middle, ..t.clone() };
    let ls = lines(&middle, &middle.text);
    assert!(close(ls[0].0, p(100.0, 47.9)) && close(ls[1].0, p(100.0, 52.1)));

    let bottom = TextItem { valign: VAnchor::Bottom, ..t.clone() };
    assert!(close(lines(&bottom, &bottom.text)[1].0, p(100.0, 50.0)));

    let up = TextItem { angle: 90.0, ..t.clone() };
    let ls = lines(&up, &up.text);
    assert!(close(ls[1].0, p(104.2, 50.0)), "the second line of a text reading upwards is to its right: {:?}", ls[1].0);
}

#[test]
fn bounds_cover_the_rotated_text() {
    let t = TextItem { pos: p(100.0, 50.0), text: "abcd".into(), size: 3.0, ..Default::default() };
    let b = bounds(&t, &t.text);
    assert!(close(b.min, p(100.0, 47.9)) && close(b.max, p(107.44, 52.1)), "{b:?}");
    let old = crate::ops::text_bounds(&t);
    assert!(close(old.min, b.min) && close(old.max, b.max), "the hit area of plain texts did not change");

    let up = TextItem { angle: 90.0, ..t.clone() };
    let b = bounds(&up, &up.text);
    assert!(close(b.min, p(97.9, 42.56)) && close(b.max, p(102.1, 50.0)), "{b:?}");

    let centred = TextItem { align: Anchor::Center, ..t.clone() };
    let b = bounds(&centred, &centred.text);
    assert!(close(b.min, p(96.28, 47.9)) && close(b.max, p(103.72, 52.1)), "{b:?}");

    let c = corners(&t, &t.text, 0.25);
    assert!(close(c[0], p(99.25, 47.15)) && close(c[2], p(108.19, 52.85)), "{c:?}");
}

#[test]
fn style_of_a_text_item() {
    let t = TextItem { bold: true, angle: 270.0, size: 2.5, align: Anchor::Right, ..Default::default() };
    let st = TextStyle::of(&t);
    assert_eq!((st.size, st.h, st.v, st.angle, st.bold, st.italic), (2.5, Anchor::Right, VAnchor::Middle, 270.0, true, false));
    assert!(!st.is_plain());
    assert!(TextStyle::plain(3.0, Anchor::Left, VAnchor::Top).is_plain());
}

#[test]
fn path_function_text_applies_to_the_functions_of_its_column() {
    let mut prj = Project::default();
    let frame = crate::frame::of_page(&prj, 0);
    let (c0, c1) = (frame.area().min.x + 2.0, frame.area().min.x + frame.column_width() + 2.0);
    let page = &mut prj.pages[0];
    page.texts.push(TextItem { id: 10, pos: p(c0, 40.0), text: "Pump 1".into(), path_function: true, ..Default::default() });
    page.texts.push(TextItem { id: 11, pos: p(c1, 40.0), text: "ignored".into(), ..Default::default() });
    page.symbols.push(SymbolInst { id: 1, sym: "coil".into(), pos: p(c0, 100.0), tag: "-K1".into(), ..Default::default() });
    page.symbols.push(SymbolInst { id: 2, sym: "coil".into(), pos: p(c0, 160.0), tag: "-K2".into(), function_text: "own".into(), ..Default::default() });
    page.symbols.push(SymbolInst { id: 3, sym: "coil".into(), pos: p(c1, 100.0), tag: "-K3".into(), ..Default::default() });
    let s = |id: u64| prj.pages[0].symbols.iter().find(|s| s.id == id).unwrap();
    assert_eq!(function_text(&prj, 0, s(1)), "Pump 1");
    assert_eq!(function_text(&prj, 0, s(2)), "own", "an own function text wins");
    assert_eq!(function_text(&prj, 0, s(3)), "", "a plain text in the next column is no path function text");
    assert_eq!(path_text(&prj, 0, p(c0, 200.0)).map(|t| t.id), Some(10));
    assert!(path_text(&prj, 1, p(c0, 200.0)).is_none(), "no such page");

    // The reports take it over.
    let an = crate::analysis::Analysis::with_builtin(&prj);
    let table = an.report(&prj, crate::model::ReportKind::DeviceList).unwrap();
    let col = table.header_texts().iter().position(|h| *h == "Function text").unwrap();
    let row = |tag: &str| table.rows.iter().find(|r| r[0] == tag).map(|r| r[col].clone()).unwrap();
    assert_eq!(row("-K1"), "Pump 1");
    assert_eq!(row("-K2"), "own");
}
