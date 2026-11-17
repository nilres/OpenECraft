use super::*;

/// An object: its dictionary (or value) and, for a stream, its data.
type Object = (String, Option<Vec<u8>>);

/// A PDF file of the objects (numbered from 1; streams as dictionary and data).
fn pdf(objects: &[Object]) -> Vec<u8> {
    pdf_at(&objects.iter().enumerate().map(|(k, o)| (k as u32 + 1, o.clone())).collect::<Vec<_>>())
}

/// A PDF file of the numbered objects.
fn pdf_at(objects: &[(u32, Object)]) -> Vec<u8> {
    let mut out = b"%PDF-1.7\n%\xe2\xe3\n".to_vec();
    for (n, (d, data)) in objects {
        out.extend_from_slice(format!("{} 0 obj\n{}", n, d).as_bytes());
        if let Some(data) = data {
            out.extend_from_slice(b"\nstream\n");
            out.extend_from_slice(data);
            out.extend_from_slice(b"\nendstream");
        }
        out.extend_from_slice(b"\nendobj\n");
    }
    out.extend_from_slice(b"trailer\n<< /Root 1 0 R >>\n%%EOF\n");
    out
}

fn obj(d: &str) -> Object {
    (d.to_string(), None)
}

fn stream(d: &str, data: &[u8]) -> Object {
    (format!("<< {d} /Length {} >>", data.len()), Some(data.to_vec()))
}

fn zlib(data: &[u8]) -> Vec<u8> {
    miniz_oxide::deflate::compress_to_vec_zlib(data, 6)
}

/// A page of 100 × 50 points: a grey image of 20 × 10 pixels over all of it (white with
/// a black square at its top left), a filled rectangle at the bottom right.
fn scan_file() -> Vec<u8> {
    let mut px = vec![255u8; 200];
    for y in 0..4 {
        for x in 0..4 {
            px[y * 20 + x] = 0;
        }
    }
    let content = b"q 100 0 0 50 0 0 cm /Im1 Do Q\n0.5 g 80 0 20 10 re f\n";
    pdf(&[
        obj("<< /Type /Catalog /Pages 2 0 R >>"),
        obj("<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 100 50] >>"),
        obj("<< /Type /Page /Parent 2 0 R /Resources << /XObject << /Im1 5 0 R >> >> /Contents 4 0 R >>"),
        stream("/Filter /FlateDecode", &zlib(content)),
        stream("/Type /XObject /Subtype /Image /Width 20 /Height 10 /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode", &zlib(&px)),
    ])
}

#[test]
fn a_scanned_page_becomes_a_picture() {
    let pages = pages(&scan_file()).unwrap();
    assert_eq!(pages.len(), 1);
    let p = &pages[0];
    // 20 pixels over 100 points: 14.4 dpi.
    assert_eq!((p.img.w, p.img.h), (20, 10));
    assert!((p.paper - 25.4 / 14.4).abs() < 1e-3);
    assert_eq!(p.img.at(1, 1), 0);
    assert_eq!(p.img.at(10, 3), 255);
    // The rectangle over the image (80 … 100 × 0 … 10 points: the bottom right).
    assert_eq!(p.img.at(18, 9), 128);
    assert_eq!(p.img.at(18, 5), 255);
}

#[test]
fn masks_of_office_scanners_are_painted() {
    // A white background and a fax coded mask (black where it paints) in grey 0.25.
    let (w, h) = (16u16, 8u16);
    let mut enc = fax::encoder::Encoder::new(fax::VecWriter::new());
    for y in 0..h {
        let line = (0..w).map(move |x| if (4..12).contains(&x) && (2..6).contains(&y) { fax::Color::Black } else { fax::Color::White });
        enc.encode_line(line, w).unwrap();
    }
    let g4 = enc.finish().unwrap().finish();
    let content = b"q 160 0 0 80 0 0 cm /Bg Do Q 0.25 g q 160 0 0 80 0 0 cm /M Do Q";
    let file = pdf(&[
        obj("<< /Type /Catalog /Pages 2 0 R >>"),
        obj("<< /Type /Pages /Kids [3 0 R] /Count 1 >>"),
        obj("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 160 80] /Rotate 90 /Resources << /XObject << /Bg 5 0 R /M 6 0 R >> >> /Contents [4 0 R] >>"),
        stream("", content),
        stream("/Subtype /Image /Width 16 /Height 8 /ColorSpace /DeviceRGB /BitsPerComponent 8", &[250; 16 * 8 * 3]),
        stream(&format!("/Subtype /Image /Width {w} /Height {h} /ImageMask true /Filter /CCITTFaxDecode /DecodeParms << /K -1 /Columns {w} /Rows {h} >>"), &g4),
    ]);
    let p = &pages(&file).unwrap()[0];
    // Turned a quarter clockwise: 8 × 16.
    assert_eq!((p.img.w, p.img.h), (8, 16));
    let unturned = turned(&p.img, 3);
    assert_eq!(unturned.at(6, 3), 64);
    assert_eq!(unturned.at(1, 1), luma(250, 250, 250));
}

#[test]
fn drawings_are_drawn_at_their_resolution() {
    // A frame of thick lines and a hatched (filled) wall, no images.
    let content = b"2 w 0 G 10 10 m 62 10 l 62 30 l S 0 g 20 20 10 5 re f";
    let file = pdf(&[
        obj("<< /Type /Catalog /Pages 2 0 R >>"),
        obj("<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 72 36] >>"),
        obj("<< /Type /Page /Parent 2 0 R /Contents 4 0 R >>"),
        stream("", content),
    ]);
    let p = &pages(&file).unwrap()[0];
    let k = DRAWING_DPI / 72.0;
    assert_eq!((p.img.w, p.img.h), ((72.0 * k).round() as usize, (36.0 * k).round() as usize));
    let at = |x: f32, y: f32| p.img.at((x * k) as usize, ((36.0 - y) * k) as usize);
    assert_eq!(at(30.0, 10.0), 0);
    assert_eq!(at(62.0, 20.0), 0);
    assert_eq!(at(25.0, 22.0), 0);
    assert_eq!(at(40.0, 20.0), 255);
}

#[test]
fn objects_in_object_streams_and_references() {
    // The page tree inside an object stream.
    let (a, b) = ("<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 100 50] >>", "<< /Type /Page /Parent 2 0 R /Resources << /XObject << /Im1 5 0 R >> >> /Contents 4 0 R >>");
    let head = format!("2 0 3 {} ", a.len() + 1);
    let inner = format!("{head}{a} {b}");
    let (inner, first) = (inner.as_bytes(), head.len());
    let mut objects = vec![(1, obj("<< /Type /Catalog /Pages 2 0 R >>"))];
    let base = scan_file();
    let file = File::parse(&base).unwrap();
    objects.push((6, stream(&format!("/Type /ObjStm /N 2 /First {first} /Filter /FlateDecode"), &zlib(inner))));
    // Objects 4 and 5 as in the scan.
    for n in [4, 5] {
        let Obj::Stream(d, data) = &file.objects[&n] else { panic!() };
        let dict: Vec<String> = d.iter().filter(|(k, _)| *k != "Length").map(|(k, v)| format!("/{k} {}", show(v))).collect();
        objects.push((n, stream(&dict.join(" "), data)));
    }
    let bytes = pdf_at(&objects);
    let parsed = File::parse(&bytes).unwrap();
    assert!(parsed.objects[&2].dict().is_some_and(|d| d.contains_key("Kids")));
    assert!(parsed.objects[&3].dict().is_some_and(|d| d.contains_key("Parent")));
    assert_eq!(pages(&bytes).unwrap()[0].img.at(1, 1), 0);
}

fn show(o: &Obj) -> String {
    match o {
        Obj::Num(v) => format!("{v}"),
        Obj::Name(n) => format!("/{n}"),
        Obj::Ref(r) => format!("{r} 0 R"),
        _ => "null".into(),
    }
}

#[test]
fn objects_are_read() {
    let mut lx = Lexer::new(b"<< /A#20b (x\\(y\\)\\101) /R 12 0 R /N -1.5 /H <414> /L [true null 3] >>", 0);
    let o = lx.obj(0).unwrap();
    let d = o.dict().unwrap();
    assert_eq!(d["A b"], Obj::Str(b"x(y)A".to_vec()));
    assert_eq!(d["R"], Obj::Ref(12));
    assert_eq!(d["N"], Obj::Num(-1.5));
    assert_eq!(d["H"], Obj::Str(vec![0x41, 0x40]));
    assert_eq!(d["L"], Obj::Array(vec![Obj::Bool(true), Obj::Null, Obj::Num(3.0)]));
    assert_eq!(number_before(b"12 0 obj", 5), Some(12));
    assert_eq!(number_before(b"endobj", 3), None);
}

#[test]
fn filters_decode() {
    assert_eq!(ascii_hex(b"48 65 6c6C6f>"), b"Hello");
    assert_eq!(ascii85(b"87cURD]i,\"Ebo80~>"), b"Hello World!");
    assert_eq!(run_length(&[2, b'a', b'b', b'c', 254, b'x', 128]), b"abcxxx");
    // The example of the PDF reference.
    assert_eq!(lzw(&[0x80, 0x0B, 0x60, 0x50, 0x22, 0x0C, 0x0C, 0x85, 0x01], true), [45, 45, 45, 45, 45, 65, 45, 45, 45, 66]);
    // PNG predictor "up" on rows of two bytes.
    let file = File::default();
    let mut p = Dict::new();
    p.insert("Predictor".into(), Obj::Num(12.0));
    p.insert("Columns".into(), Obj::Num(2.0));
    assert_eq!(predicted(&[2, 1, 2, 2, 1, 1], &p, &file), [1, 2, 2, 3]);
}

#[test]
fn what_cannot_be_read() {
    assert!(!is_pdf(b"\x89PNG"));
    assert!(is_pdf(&scan_file()));
    assert!(pages(b"%PDF-1.4\n1 0 obj << /Type /Catalog >> endobj").is_err());
    let enc = pdf(&[obj("<< /Type /Catalog /Pages 2 0 R >>"), obj("<< /Encrypt 3 0 R >>")]);
    assert!(File::parse(&enc).is_err());
    let jbig = pdf(&[
        obj("<< /Type /Catalog /Pages 2 0 R >>"),
        obj("<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 10 10] >>"),
        obj("<< /Type /Page /Parent 2 0 R /Resources << /XObject << /I 5 0 R >> >> /Contents 4 0 R >>"),
        stream("", b"q 10 0 0 10 0 0 cm /I Do Q"),
        stream("/Subtype /Image /Width 4 /Height 4 /ImageMask true /Filter /JBIG2Decode", b"xx"),
    ]);
    assert!(pages(&jbig).unwrap_err().contains("JBIG2"));
}

/// The scanned plans of an issue (not in the repository): `OPENECRAFT_SCAN_PDF` names
/// the file.
#[test]
fn scanned_plans_from_a_file() {
    let Ok(path) = std::env::var("OPENECRAFT_SCAN_PDF") else { return };
    let sheets = pages(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(sheets.len(), 3);
    for s in &sheets {
        assert!(s.img.w > s.img.h, "landscape: {} × {}", s.img.w, s.img.h);
        assert!((s.paper - 25.4 / 150.0).abs() < 0.01);
    }
}
