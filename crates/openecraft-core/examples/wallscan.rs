//! Runs the wall detection on a floor plan image and draws what it found.
//!
//! `cargo run --release -p openecraft-core --example wallscan -- <plan.png> <mm per pixel> [out.png]`
//!
//! The picture written shows the plan faintly, the walls grey, the openings (doors red,
//! windows blue, a swing as a dot at the hinge on the side it opens to) and the outlines
//! of the rooms green; the rooms and openings are listed.

use openecraft_core::raster::Gray;
use openecraft_core::wallscan::{scan, GapKind, Settings};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(path), Some(scale)) = (args.get(1), args.get(2).and_then(|s| s.parse::<f32>().ok())) else {
        eprintln!("usage: wallscan <plan.png> <mm per pixel> [out.png]");
        std::process::exit(2);
    };
    let img = Gray::from_png(&std::fs::read(path).expect("read")).expect("png");
    let s = Settings::new(scale);
    let t = std::time::Instant::now();
    let found = scan(&img, &s);
    eprintln!("{} x {} px, {} ink pixels, {:.0} ms", img.w, img.h, found.ink.count(), t.elapsed().as_secs_f64() * 1000.0);
    for (k, r) in found.rooms.iter().enumerate() {
        let m2 = r.area * scale * scale / 1e6;
        println!("room {k}: {:.1} m², {} corners {:?}", m2, r.outline.len(), r.outline.iter().map(|q| [q[0].round(), q[1].round()]).collect::<Vec<_>>());
    }
    for g in &found.openings {
        let (len, thick) = g.size();
        println!("{:?} {:.0} mm wide in a wall of {:.0} mm at {:?}, sides {:?}, swing {:?}", g.kind, len * scale, thick * scale, g.lo.map(f32::round), g.sides, g.swing);
    }
    if let Ok(v) = std::env::var("BANDS_NEAR") {
        let v: Vec<usize> = v.split(',').filter_map(|x| x.parse().ok()).collect();
        for b in found.bands.iter().filter(|b| b.lo[0] <= v[2] && b.hi[0] >= v[0] && b.lo[1] <= v[3] && b.hi[1] >= v[1]) {
            println!("band {:?}", b);
        }
    }
    let out = args.get(3).cloned().unwrap_or_else(|| "wallscan.png".into());
    // RGB picture.
    let mut rgb: Vec<[u8; 3]> = img.px.iter().map(|&v| {
        let v = 255 - (255 - v) / 3;
        [v, v, v]
    }).collect();
    for (k, on) in found.walls.on.iter().enumerate() {
        if *on {
            rgb[k] = [150, 150, 150];
        }
    }
    let mut put = |x: f32, y: f32, c: [u8; 3]| {
        let (x, y) = (x.round() as isize, y.round() as isize);
        if x >= 0 && y >= 0 && (x as usize) < img.w && (y as usize) < img.h {
            rgb[y as usize * img.w + x as usize] = c;
        }
    };
    for b in &found.bands {
        for x in b.lo[0]..b.hi[0] {
            put(x as f32, b.lo[1] as f32, [230, 180, 0]);
            put(x as f32, b.hi[1] as f32 - 1.0, [230, 180, 0]);
        }
        for y in b.lo[1]..b.hi[1] {
            put(b.lo[0] as f32, y as f32, [230, 180, 0]);
            put(b.hi[0] as f32 - 1.0, y as f32, [230, 180, 0]);
        }
    }
    for g in &found.openings {
        let c = if g.kind == GapKind::Door { [220, 30, 30] } else { [30, 80, 230] };
        for y in g.lo[1] as usize..g.hi[1] as usize {
            for x in g.lo[0] as usize..g.hi[0] as usize {
                put(x as f32, y as f32, c);
            }
        }
        if let Some((side, lo)) = g.swing {
            let (axis, cross) = if g.along_x { (0, 1) } else { (1, 0) };
            let mut q = [0.0; 2];
            q[axis] = if lo { g.lo[axis] } else { g.hi[axis] };
            q[cross] = if side == 0 { g.lo[cross] - 6.0 } else { g.hi[cross] + 6.0 };
            for dy in -4..=4 {
                for dx in -4..=4 {
                    put(q[0] + dx as f32, q[1] + dy as f32, [240, 160, 0]);
                }
            }
        }
    }
    for r in &found.rooms {
        let n = r.outline.len();
        for k in 0..n {
            let (a, b) = (r.outline[k], r.outline[(k + 1) % n]);
            let steps = ((b[0] - a[0]).abs() + (b[1] - a[1]).abs()).max(1.0) as usize;
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                for d in -1..=1 {
                    put(a[0] + (b[0] - a[0]) * t + d as f32, a[1] + (b[1] - a[1]) * t + d as f32, [20, 170, 60]);
                }
            }
        }
    }
    let mut bytes = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut bytes, img.w as u32, img.h as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().expect("png");
        w.write_image_data(&rgb.concat()).expect("png");
    }
    std::fs::write(&out, bytes).expect("write");
    eprintln!("wrote {out}");
}
