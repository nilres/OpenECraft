use super::*;
use openecraft_core::raster::luma;

#[test]
fn pictures_are_read_in_grey() {
    let img = image::RgbaImage::from_fn(3, 2, |x, _| if x == 0 { image::Rgba([0, 0, 0, 255]) } else if x == 1 { image::Rgba([255, 0, 0, 255]) } else { image::Rgba([0, 0, 0, 0]) });
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(img).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
    let sheets = read("plan.png", &png).unwrap();
    assert_eq!(sheets.len(), 1);
    let s = &sheets[0];
    assert_eq!((s.img.w, s.img.h), (3, 2));
    assert_eq!(s.img.px[..3], [0, luma(255, 0, 0), 255]);
    assert!((s.paper - 25.4 / 150.0).abs() < 1e-6);
}

#[test]
fn other_files_are_refused() {
    let e = read("notes.txt", b"just text").unwrap_err();
    assert!(e.contains("notes.txt"));
}
