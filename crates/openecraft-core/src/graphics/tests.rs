use super::*;

#[test]
fn circle_outline_and_bounds() {
    let s = Shape { kind: ShapeKind::Circle, a: p(10.0, 10.0), b: p(15.0, 10.0), ..Default::default() };
    let r = shape_bounds(&s);
    assert!((r.min.x - 5.0).abs() < 0.01 && (r.max.y - 15.0).abs() < 0.01);
    assert!(shape_hit(&s, p(10.0, 5.0), 0.2));
    assert!(!shape_hit(&s, p(10.0, 10.0), 0.2));
    let filled = Shape { fill: true, ..s };
    assert!(shape_hit(&filled, p(10.0, 10.0), 0.2));
}

#[test]
fn arc_runs_counter_clockwise() {
    let s = Shape { kind: ShapeKind::Arc, a: p(0.0, 0.0), b: p(10.0, 0.0), sweep: 90.0, ..Default::default() };
    let (pts, closed) = outline(&s);
    assert!(!closed);
    let last = *pts.last().unwrap();
    // y points down: a quarter turn counter-clockwise ends above the centre.
    assert!(last.dist(p(0.0, -10.0)) < 0.01, "{last:?}");
}

#[test]
fn polygon_hit_inside() {
    let s = Shape { kind: ShapeKind::Polygon, pts: vec![p(0.0, 0.0), p(10.0, 0.0), p(0.0, 10.0)], fill: true, ..Default::default() };
    assert!(shape_hit(&s, p(2.0, 2.0), 0.1));
    assert!(!shape_hit(&s, p(8.0, 8.0), 0.1));
}

#[test]
fn horizontal_dimension() {
    let d = Dimension { kind: DimKind::Horizontal, a: p(10.0, 50.0), b: p(40.0, 60.0), offset: 10.0, ..Default::default() };
    assert_eq!(dim_text(&d), "30");
    let g = dim_geom(&d);
    // Dimension line 10 mm above `a`, from x 10 to x 40.
    assert_eq!(g.line, (p(10.0, 40.0), p(40.0, 40.0)));
    // The second extension line runs vertically from `b`.
    assert_eq!(g.ext[1].0.x, 40.0);
    assert_eq!(g.ext[1].1.x, 40.0);
    assert!((dim_offset_through(&d, p(25.0, 30.0)) - 20.0).abs() < 1e-4);
}

#[test]
fn dimension_scale_unit_and_text() {
    let mut d = Dimension { a: p(0.0, 0.0), b: p(0.0, 25.0), scale: 10.0, decimals: 1, unit: "mm".into(), ..Default::default() };
    assert_eq!(dim_text(&d), "250.0 mm");
    d.text = "approx. 250".into();
    assert_eq!(dim_text(&d), "approx. 250");
}

#[test]
fn layers_hide_rename_remove() {
    let mut prj = Project::default();
    prj.layers.push(Layer { print: false, ..Layer::new("Notes") });
    prj.pages[0].texts.push(TextItem { id: 5, layer: "Notes".into(), ..Default::default() });
    assert!(shown(&prj, "Notes", false));
    assert!(!shown(&prj, "Notes", true));
    assert!(shown(&prj, "", true));
    assert_eq!(item_layer(&prj.pages[0], 5), Some("Notes"));
    rename_layer(&mut prj, "Notes", "Hints");
    assert_eq!(prj.pages[0].texts[0].layer, "Hints");
    assert_eq!(layer_count(&prj, "Hints"), 1);
    remove_layer(&mut prj, "Hints");
    assert!(prj.layers.is_empty());
    assert_eq!(prj.pages[0].texts[0].layer, "");
}

#[test]
fn layer_names_include_used_ones() {
    let mut prj = Project::default();
    prj.layers.push(Layer::new("A"));
    prj.pages[0].shapes.push(Shape { layer: "B".into(), ..Default::default() });
    assert_eq!(layer_names(&prj), vec!["A".to_string(), "B".to_string()]);
}

#[test]
fn image_pixel_size_from_headers() {
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    png.extend_from_slice(&200u32.to_be_bytes());
    png.extend_from_slice(&100u32.to_be_bytes());
    assert_eq!(image_pixels(&png), Some([200, 100]));
    let jpeg = [0xFF, 0xD8, 0xFF, 0xE0, 0, 4, 0, 0, 0xFF, 0xC0, 0, 11, 8, 0, 50, 0, 80, 3, 0, 0, 0];
    assert_eq!(image_pixels(&jpeg), Some([80, 50]));
    assert_eq!(image_pixels(b"GIF89a"), None);
}

#[test]
fn image_keeps_aspect_ratio() {
    let mut prj = Project::default();
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    png.extend_from_slice(&200u32.to_be_bytes());
    png.extend_from_slice(&100u32.to_be_bytes());
    let id = add_image(&mut prj, 0, p(10.0, 10.0), "logo.png", &png, 60.0, "");
    let img = prj.pages[0].images.iter().find(|i| i.id == id).unwrap();
    assert_eq!(img.b, p(70.0, 40.0));
    assert_eq!(img.file.bytes().unwrap(), png);
}

#[test]
fn layer_names_stay_unique_and_not_empty() {
    let names = vec!["Notes".to_string(), "Frame".to_string()];
    assert_eq!(layer_rename_problem(&names, "Notes", "Notes"), None);
    assert_eq!(layer_rename_problem(&names, "Notes", "Notes 2"), None);
    assert_eq!(layer_rename_problem(&names, "Notes", "Frame"), Some("Name already used"));
    assert_eq!(layer_rename_problem(&names, "Notes", " "), Some("A layer needs a name"));
}

#[test]
fn drawing_scales_are_written_as_ratios() {
    assert_eq!(scale_text(1.0), "1 : 1");
    assert_eq!(scale_text(50.0), "1 : 50");
    assert_eq!(scale_text(0.5), "2 : 1");
    assert_eq!(scale_text(12.5), "1 : 12.5");
    assert_eq!(scale_text(0.0), "1 : 1");
    let mut page = Page::default();
    assert_eq!(page_scale(&page), 1.0);
    page.scale = -3.0;
    assert_eq!(page_scale(&page), 1.0);
    page.scale = 100.0;
    assert_eq!(page_scale(&page), 100.0);
    // A page at 1 : 1 keeps no scale in the file.
    assert!(!serde_json::to_string(&Page::default()).unwrap().contains("scale"));
    let back: Page = serde_json::from_str(&serde_json::to_string(&page).unwrap()).unwrap();
    assert_eq!(back.scale, 100.0);
}

#[test]
fn shapes_have_a_size_and_a_length() {
    let rect = Shape { kind: ShapeKind::Rect, a: p(0.0, 0.0), b: p(30.0, 20.0), ..Default::default() };
    assert_eq!(shape_size(&rect), (p(30.0, 20.0), 100.0));
    let line = Shape { kind: ShapeKind::Line, a: p(0.0, 0.0), b: p(3.0, 4.0), ..Default::default() };
    assert_eq!(shape_size(&line).1, 5.0);
    let circle = Shape { kind: ShapeKind::Circle, a: p(0.0, 0.0), b: p(10.0, 0.0), ..Default::default() };
    assert!((shape_size(&circle).1 - 62.83).abs() < 0.01);
    let poly = Shape { kind: ShapeKind::Polyline, pts: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 5.0)], ..Default::default() };
    assert_eq!(shape_size(&poly), (p(10.0, 5.0), 15.0));
}
