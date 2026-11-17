use super::*;

#[test]
fn png_round_trips_in_grey() {
    let mut img = Gray::new(5, 3, 200);
    img.set(1, 2, 7);
    let back = Gray::from_png(&img.to_png()).unwrap();
    assert_eq!(back, img);
    assert!(Gray::from_png(b"no png").is_err());
}

#[test]
fn colour_png_becomes_grey_on_white() {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, 2, 1);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().unwrap();
        // Red, opaque; black, fully transparent.
        w.write_image_data(&[255, 0, 0, 255, 0, 0, 0, 0]).unwrap();
    }
    let img = Gray::from_png(&out).unwrap();
    assert_eq!(img.px, vec![luma(255, 0, 0), 255]);
}

#[test]
fn shrinking_averages_blocks() {
    let img = Gray { w: 4, h: 2, px: vec![0, 100, 200, 200, 100, 200, 200, 200] };
    assert_eq!(img.shrunk(2).px, vec![100, 200]);
}

#[test]
fn median_removes_specks_and_keeps_edges() {
    let mut img = Gray::new(9, 9, 240);
    img.set(4, 4, 0);
    img.fill([0, 6], [9, 9], 100);
    let m = img.median();
    assert_eq!(m.at(4, 4), 240);
    assert_eq!(m.at(4, 7), 100);
    assert_eq!(m.at(4, 5), 240);
}

#[test]
fn paper_and_background() {
    let mut img = Gray::new(40, 20, 230);
    img.fill([0, 0], [25, 20], 250);
    img.fill([5, 5], [8, 8], 0);
    assert_eq!(img.paper(), 250);
    let bg = img.background(4);
    assert_eq!(bg.at(6, 6), 250);
    assert_eq!(bg.at(35, 10), 230);
    let ink = Mask::ink(&img, &bg, 10);
    assert!(ink.get(6, 6) && !ink.get(35, 10) && !ink.get(2, 2));
}

#[test]
fn opening_removes_thin_lines() {
    let mut m = Mask::new(30, 30);
    m.fill([5, 5], [25, 15]);
    m.fill([0, 20], [30, 22]);
    let o = m.opened(2);
    assert!(o.get(10, 10) && o.get(5, 5) && o.get(24, 14));
    assert!(!o.get(10, 21));
    assert_eq!(o.count(), 200);
    let mut gap = Mask::new(10, 5);
    gap.fill([0, 0], [4, 5]);
    gap.fill([5, 0], [10, 5]);
    assert!(gap.closed(1).get(4, 2));
}

#[test]
fn components_and_runs() {
    let mut m = Mask::new(10, 6);
    m.fill([0, 0], [3, 2]);
    m.fill([5, 3], [6, 6]);
    let (label, parts) = m.components();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], Component { area: 6, lo: [0, 0], hi: [3, 2] });
    assert_eq!(parts[1].span(), 3);
    assert!(parts[0].touches_border(10, 6) && parts[1].touches_border(10, 6));
    assert!(!parts[1].near_border(20, 20, 2));
    assert_eq!(label[0], 1);
    assert_eq!(label[5 * 10 + 5], 2);
    assert_eq!(m.runs(true)[0], 3);
    assert_eq!(m.runs(false)[3 * 10 + 5], 3);
    assert_eq!(m.runs(true)[9], 0);
}
