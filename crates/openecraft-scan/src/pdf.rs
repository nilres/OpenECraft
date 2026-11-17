//! The pages of PDF files as grey pictures, for floor plans: the images on a page
//! (scans, also those of office scanners split into a background picture and black and
//! white masks) at the resolution of the finest of them, and the lines and filled areas
//! of drawings at [`DRAWING_DPI`]. Text, shadings and clipping are not drawn; encrypted
//! files and JBIG2 or JPEG 2000 images are not read.
//!
//! The file is read without its cross-reference table: every `n g obj` in it (later
//! ones replacing earlier ones) and the objects in object streams.

use openecraft_core::raster::{luma, Gray, MAX_PIXELS};
use openecraft_i18n::{tr, tr_fmt};
use std::collections::{BTreeMap, HashMap};

use crate::Sheet;

/// Resolution of the pages without images, dots per inch.
pub const DRAWING_DPI: f32 = 200.0;
/// Finest resolution a page is drawn at, dots per inch.
pub const MAX_DPI: f32 = 600.0;

/// A PDF object.
#[derive(Clone, Debug, PartialEq)]
pub enum Obj {
    Null,
    Bool(bool),
    Num(f64),
    Str(Vec<u8>),
    Name(String),
    Array(Vec<Obj>),
    Dict(Dict),
    Ref(u32),
    /// Its dictionary and its data as stored (not decoded).
    Stream(Dict, Vec<u8>),
}

pub type Dict = BTreeMap<String, Obj>;

/// Filters (their names and parameters) left for the image codecs.
type Codecs = Vec<(String, Dict)>;

impl Obj {
    pub fn num(&self) -> Option<f64> {
        if let Obj::Num(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn name(&self) -> Option<&str> {
        if let Obj::Name(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn dict(&self) -> Option<&Dict> {
        match self {
            Obj::Dict(d) | Obj::Stream(d, _) => Some(d),
            _ => None,
        }
    }
}

/// The file starts like a PDF file.
pub fn is_pdf(bytes: &[u8]) -> bool {
    bytes.iter().take(1024).collect::<Vec<_>>().windows(5).any(|w| w.iter().map(|b| **b).eq(*b"%PDF-"))
}

// ------------------------------------------------------------------ objects

struct Lexer<'a> {
    s: &'a [u8],
    at: usize,
}

fn white(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0c' | b'\0')
}

fn delimiter(b: u8) -> bool {
    matches!(b, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%')
}

impl<'a> Lexer<'a> {
    fn new(s: &'a [u8], at: usize) -> Self {
        Lexer { s, at }
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.at).copied()
    }

    fn skip(&mut self) {
        while let Some(b) = self.peek() {
            if white(b) {
                self.at += 1;
            } else if b == b'%' {
                while self.peek().is_some_and(|b| b != b'\n' && b != b'\r') {
                    self.at += 1;
                }
            } else {
                break;
            }
        }
    }

    /// A keyword or number (a run of regular characters).
    fn word(&mut self) -> &'a [u8] {
        let start = self.at;
        while self.peek().is_some_and(|b| !white(b) && !delimiter(b)) {
            self.at += 1;
        }
        &self.s[start..self.at]
    }

    fn starts(&self, w: &[u8]) -> bool {
        self.s[self.at..].starts_with(w)
    }

    /// The next object; numbers followed by `g R` are references. `None` at the end
    /// or at a keyword (left in place as `Name` of the keyword when `keywords`).
    fn obj(&mut self, depth: usize) -> Option<Obj> {
        if depth > 64 {
            return None;
        }
        self.skip();
        let b = self.peek()?;
        match b {
            b'<' if self.starts(b"<<") => {
                self.at += 2;
                let mut d = Dict::new();
                loop {
                    self.skip();
                    if self.starts(b">>") {
                        self.at += 2;
                        break;
                    }
                    match self.obj(depth + 1)? {
                        Obj::Name(k) => {
                            let v = self.obj(depth + 1).unwrap_or(Obj::Null);
                            d.insert(k, v);
                        }
                        _ => return None,
                    }
                }
                Some(Obj::Dict(d))
            }
            b'<' => {
                self.at += 1;
                let mut hex = Vec::new();
                while let Some(b) = self.peek() {
                    self.at += 1;
                    if b == b'>' {
                        break;
                    }
                    if b.is_ascii_hexdigit() {
                        hex.push(b);
                    }
                }
                if hex.len() % 2 == 1 {
                    hex.push(b'0');
                }
                Some(Obj::Str(hex.chunks(2).map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap_or("00"), 16).unwrap_or(0)).collect()))
            }
            b'[' => {
                self.at += 1;
                let mut v = Vec::new();
                loop {
                    self.skip();
                    match self.peek() {
                        Some(b']') => {
                            self.at += 1;
                            break;
                        }
                        None => break,
                        _ => v.push(self.obj(depth + 1)?),
                    }
                }
                Some(Obj::Array(v))
            }
            b'(' => {
                self.at += 1;
                let mut out = Vec::new();
                let mut nest = 0;
                while let Some(b) = self.peek() {
                    self.at += 1;
                    match b {
                        b'\\' => {
                            if let Some(e) = self.peek() {
                                self.at += 1;
                                out.push(match e {
                                    b'n' => b'\n',
                                    b'r' => b'\r',
                                    b't' => b'\t',
                                    b'b' => 8,
                                    b'f' => 12,
                                    b'0'..=b'7' => {
                                        let mut v = u32::from(e - b'0');
                                        for _ in 0..2 {
                                            match self.peek() {
                                                Some(d @ b'0'..=b'7') => {
                                                    v = v * 8 + u32::from(d - b'0');
                                                    self.at += 1;
                                                }
                                                _ => break,
                                            }
                                        }
                                        v as u8
                                    }
                                    other => other,
                                });
                            }
                        }
                        b'(' => {
                            nest += 1;
                            out.push(b);
                        }
                        b')' if nest == 0 => break,
                        b')' => {
                            nest -= 1;
                            out.push(b);
                        }
                        _ => out.push(b),
                    }
                }
                Some(Obj::Str(out))
            }
            b'/' => {
                self.at += 1;
                let w = self.word();
                let mut name = Vec::new();
                let mut k = 0;
                while k < w.len() {
                    if w[k] == b'#' && k + 2 < w.len() {
                        if let Ok(v) = u8::from_str_radix(std::str::from_utf8(&w[k + 1..k + 3]).unwrap_or("x"), 16) {
                            name.push(v);
                            k += 3;
                            continue;
                        }
                    }
                    name.push(w[k]);
                    k += 1;
                }
                Some(Obj::Name(String::from_utf8_lossy(&name).into_owned()))
            }
            b'{' | b'}' | b')' | b'>' | b']' => {
                self.at += 1;
                Some(Obj::Null)
            }
            _ => {
                let start = self.at;
                let w = self.word();
                if w.is_empty() {
                    self.at += 1;
                    return Some(Obj::Null);
                }
                match w {
                    b"true" => return Some(Obj::Bool(true)),
                    b"false" => return Some(Obj::Bool(false)),
                    b"null" => return Some(Obj::Null),
                    _ => {}
                }
                let Some(v) = std::str::from_utf8(w).ok().and_then(|t| t.parse::<f64>().ok()) else {
                    // A keyword: left in place.
                    self.at = start;
                    return None;
                };
                // `n g R`
                if w.iter().all(u8::is_ascii_digit) {
                    let save = self.at;
                    self.skip();
                    let g = self.word();
                    if !g.is_empty() && g.iter().all(u8::is_ascii_digit) {
                        self.skip();
                        if self.peek() == Some(b'R') && self.s.get(self.at + 1).is_none_or(|b| white(*b) || delimiter(*b)) {
                            self.at += 1;
                            return Some(Obj::Ref(v as u32));
                        }
                    }
                    self.at = save;
                }
                Some(Obj::Num(v))
            }
        }
    }
}

/// The objects of a PDF file by number.
#[derive(Debug, Default)]
pub struct File {
    pub objects: HashMap<u32, Obj>,
}

impl File {
    pub fn parse(s: &[u8]) -> Result<File, String> {
        let mut objects = HashMap::new();
        let mut at = 0;
        while let Some(k) = find(s, b"obj", at) {
            at = k + 3;
            // `n g obj`, not `endobj`.
            let Some(num) = number_before(s, k) else { continue };
            let mut lx = Lexer::new(s, k + 3);
            let Some(mut o) = lx.obj(0) else { continue };
            lx.skip();
            if lx.starts(b"stream") {
                lx.at += 6;
                if lx.starts(b"\r\n") {
                    lx.at += 2;
                } else if lx.peek().is_some_and(|b| b == b'\n' || b == b'\r') {
                    lx.at += 1;
                }
                let start = lx.at;
                let d = o.dict().cloned().unwrap_or_default();
                let direct = d.get("Length").and_then(Obj::num).map(|v| v as usize).filter(|&n| {
                    let end = start + n;
                    end <= s.len() && {
                        let mut t = Lexer::new(s, end);
                        t.skip();
                        t.starts(b"endstream")
                    }
                });
                let end = match direct {
                    Some(n) => start + n,
                    None => {
                        let e = find(s, b"endstream", start).unwrap_or(s.len());
                        let mut e2 = e;
                        if e2 > start && s[e2 - 1] == b'\n' {
                            e2 -= 1;
                        }
                        if e2 > start && s[e2 - 1] == b'\r' {
                            e2 -= 1;
                        }
                        e2
                    }
                };
                o = Obj::Stream(d, s[start..end].to_vec());
                at = end;
            } else {
                at = lx.at;
            }
            objects.insert(num, o);
        }
        if objects.values().any(|o| o.dict().is_some_and(|d| d.contains_key("Encrypt"))) {
            return Err(tr("Encrypted PDF files cannot be read").to_string());
        }
        let mut file = File { objects };
        // Objects in object streams (those defined directly take precedence).
        let streams: Vec<Obj> = file.objects.values().filter(|o| matches!(o, Obj::Stream(d, _) if d.get("Type").and_then(Obj::name) == Some("ObjStm"))).cloned().collect();
        for st in streams {
            let Obj::Stream(d, _) = &st else { continue };
            let Ok(data) = file.decoded(&st) else { continue };
            let n = d.get("N").and_then(Obj::num).unwrap_or(0.0) as usize;
            let first = d.get("First").and_then(Obj::num).unwrap_or(0.0) as usize;
            let mut lx = Lexer::new(&data, 0);
            let mut heads = Vec::new();
            for _ in 0..n {
                let (Some(Obj::Num(num)), Some(Obj::Num(off))) = (lx.obj(0), lx.obj(0)) else { break };
                heads.push((num as u32, off as usize));
            }
            for (num, off) in heads {
                if file.objects.contains_key(&num) {
                    continue;
                }
                if let Some(o) = Lexer::new(&data, first + off).obj(0) {
                    file.objects.insert(num, o);
                }
            }
        }
        Ok(file)
    }

    /// The object, references followed.
    pub fn get<'a>(&'a self, o: &'a Obj) -> &'a Obj {
        let mut o = o;
        for _ in 0..32 {
            match o {
                Obj::Ref(n) => o = self.objects.get(n).unwrap_or(&Obj::Null),
                _ => return o,
            }
        }
        &Obj::Null
    }

    fn key<'a>(&'a self, d: &'a Dict, k: &str) -> &'a Obj {
        d.get(k).map_or(&Obj::Null, |o| self.get(o))
    }

    fn num(&self, d: &Dict, k: &str) -> Option<f64> {
        self.key(d, k).num()
    }

    /// The data of a stream with its filters applied, but for the image codecs (DCT,
    /// CCITT, JBIG2, JPX), which stay as they are: the data and those left.
    fn filtered(&self, o: &Obj) -> Result<(Vec<u8>, Codecs), String> {
        let Obj::Stream(d, raw) = o else { return Ok((Vec::new(), Vec::new())) };
        let names: Vec<String> = match self.key(d, "Filter") {
            Obj::Name(n) => vec![n.clone()],
            Obj::Array(v) => v.iter().filter_map(|f| self.get(f).name().map(str::to_string)).collect(),
            _ => Vec::new(),
        };
        let parms: Vec<Dict> = match self.key(d, "DecodeParms") {
            Obj::Dict(p) => vec![p.clone()],
            Obj::Array(v) => v.iter().map(|p| self.get(p).dict().cloned().unwrap_or_default()).collect(),
            _ => Vec::new(),
        };
        let mut data = raw.clone();
        let mut rest = Vec::new();
        for (k, f) in names.iter().enumerate() {
            let p = parms.get(k).cloned().unwrap_or_default();
            if !rest.is_empty() {
                rest.push((f.clone(), p));
                continue;
            }
            data = match f.as_str() {
                "FlateDecode" | "Fl" => predicted(&inflate(&data)?, &p, self),
                "ASCIIHexDecode" | "AHx" => ascii_hex(&data),
                "ASCII85Decode" | "A85" => ascii85(&data),
                "RunLengthDecode" | "RL" => run_length(&data),
                "LZWDecode" | "LZW" => predicted(&lzw(&data, self.num(&p, "EarlyChange").unwrap_or(1.0) != 0.0), &p, self),
                _ => {
                    rest.push((f.clone(), p));
                    data
                }
            };
        }
        Ok((data, rest))
    }

    /// The data of a stream fully decoded (no image codecs).
    pub fn decoded(&self, o: &Obj) -> Result<Vec<u8>, String> {
        let (data, rest) = self.filtered(o)?;
        if let Some((f, _)) = rest.first() {
            return Err(tr_fmt!("Unsupported PDF filter {}", f));
        }
        Ok(data)
    }

    /// The pages in their order with what they inherit from the page tree.
    fn pages(&self) -> Vec<PageRef> {
        let root = self.objects.values().find(|o| o.dict().is_some_and(|d| self.key(d, "Type").name() == Some("Catalog")));
        let mut out = Vec::new();
        if let Some(pages) = root.and_then(|r| r.dict()).map(|d| d.get("Pages").cloned().unwrap_or(Obj::Null)) {
            self.walk(&pages, &PageRef::default(), &mut out, 0);
        }
        if out.is_empty() {
            // No catalog: every page by its number.
            let mut nums: Vec<&u32> = self.objects.keys().filter(|n| self.objects[n].dict().is_some_and(|d| self.key(d, "Type").name() == Some("Page"))).collect();
            nums.sort();
            for n in nums {
                self.walk(&Obj::Ref(*n), &PageRef::default(), &mut out, 0);
            }
        }
        out
    }

    fn walk(&self, node: &Obj, inherit: &PageRef, out: &mut Vec<PageRef>, depth: usize) {
        let Some(d) = self.get(node).dict() else { return };
        if depth > 32 {
            return;
        }
        let mut here = inherit.clone();
        if let Some(r) = d.get("Resources") {
            here.resources = self.get(r).dict().cloned().unwrap_or_default();
        }
        if let Obj::Array(v) = self.key(d, "MediaBox") {
            let n: Vec<f64> = v.iter().filter_map(|x| self.get(x).num()).collect();
            if n.len() == 4 {
                here.media = [n[0].min(n[2]), n[1].min(n[3]), n[0].max(n[2]), n[1].max(n[3])];
            }
        }
        if let Some(r) = self.num(d, "Rotate") {
            here.rotate = (r as i64).rem_euclid(360) as u32;
        }
        match self.key(d, "Type").name() {
            Some("Pages") => {
                if let Obj::Array(kids) = self.key(d, "Kids") {
                    for k in kids {
                        self.walk(k, &here, out, depth + 1);
                    }
                }
            }
            _ => {
                here.contents = match d.get("Contents").map(|c| self.get(c)) {
                    Some(Obj::Array(v)) => v.clone(),
                    Some(c) => vec![c.clone()],
                    None => Vec::new(),
                };
                out.push(here);
            }
        }
    }
}

fn find(s: &[u8], w: &[u8], from: usize) -> Option<usize> {
    if from >= s.len() {
        return None;
    }
    s[from..].windows(w.len()).position(|x| x == w).map(|k| k + from)
}

/// The object number of `n g obj` ending before `k` (where `obj` starts).
fn number_before(s: &[u8], k: usize) -> Option<u32> {
    let mut i = k;
    let back_white = |i: &mut usize| {
        let start = *i;
        while *i > 0 && white(s[*i - 1]) {
            *i -= 1;
        }
        *i < start
    };
    let back_digits = |i: &mut usize| {
        let end = *i;
        while *i > 0 && s[*i - 1].is_ascii_digit() {
            *i -= 1;
        }
        (*i < end).then(|| std::str::from_utf8(&s[*i..end]).ok()?.parse::<u32>().ok()).flatten()
    };
    if !back_white(&mut i) {
        return None;
    }
    back_digits(&mut i)?;
    if !back_white(&mut i) {
        return None;
    }
    let n = back_digits(&mut i)?;
    (i == 0 || white(s[i - 1]) || delimiter(s[i - 1])).then_some(n)
}

#[derive(Clone, Debug)]
struct PageRef {
    resources: Dict,
    media: [f64; 4],
    rotate: u32,
    contents: Vec<Obj>,
}

impl Default for PageRef {
    fn default() -> Self {
        PageRef { resources: Dict::new(), media: [0.0, 0.0, 612.0, 792.0], rotate: 0, contents: Vec::new() }
    }
}

// ------------------------------------------------------------------ filters

fn inflate(data: &[u8]) -> Result<Vec<u8>, String> {
    miniz_oxide::inflate::decompress_to_vec_zlib(data)
        .or_else(|_| miniz_oxide::inflate::decompress_to_vec(data))
        .or_else(|e| {
            // A damaged end: what could be read.
            let part = e.output;
            if part.is_empty() {
                Err(tr("A compressed stream of the PDF file is damaged").to_string())
            } else {
                Ok(part)
            }
        })
}

/// Undoes a PNG or TIFF predictor.
fn predicted(data: &[u8], p: &Dict, file: &File) -> Vec<u8> {
    let predictor = file.num(p, "Predictor").unwrap_or(1.0) as u32;
    if predictor < 2 {
        return data.to_vec();
    }
    let colors = file.num(p, "Colors").unwrap_or(1.0).max(1.0) as usize;
    let bpc = file.num(p, "BitsPerComponent").unwrap_or(8.0).max(1.0) as usize;
    let columns = file.num(p, "Columns").unwrap_or(1.0).max(1.0) as usize;
    let bpp = (colors * bpc).div_ceil(8).max(1);
    let row = (colors * bpc * columns).div_ceil(8);
    if predictor == 2 {
        let mut out = data.to_vec();
        if bpc == 8 {
            for line in out.chunks_mut(row) {
                for k in bpp..line.len() {
                    line[k] = line[k].wrapping_add(line[k - bpp]);
                }
            }
        }
        return out;
    }
    let mut out = Vec::with_capacity(data.len());
    let mut prev = vec![0u8; row];
    for chunk in data.chunks(row + 1) {
        let (kind, line) = (chunk[0], &chunk[1..]);
        let mut cur = vec![0u8; row];
        for k in 0..line.len().min(row) {
            let a = if k >= bpp { cur[k - bpp] } else { 0 };
            let b = prev[k];
            let c = if k >= bpp { prev[k - bpp] } else { 0 };
            cur[k] = match kind {
                1 => line[k].wrapping_add(a),
                2 => line[k].wrapping_add(b),
                3 => line[k].wrapping_add(((u16::from(a) + u16::from(b)) / 2) as u8),
                4 => {
                    let (pa, pb, pc) = ((i16::from(b) - i16::from(c)).abs(), (i16::from(a) - i16::from(c)).abs(), (i16::from(a) + i16::from(b) - 2 * i16::from(c)).abs());
                    line[k].wrapping_add(if pa <= pb && pa <= pc { a } else if pb <= pc { b } else { c })
                }
                _ => line[k],
            };
        }
        out.extend_from_slice(&cur[..line.len().min(row)]);
        prev = cur;
    }
    out
}

fn ascii_hex(data: &[u8]) -> Vec<u8> {
    let hex: Vec<u8> = data.iter().take_while(|b| **b != b'>').filter(|b| b.is_ascii_hexdigit()).copied().collect();
    hex.chunks(2).map(|c| u8::from_str_radix(std::str::from_utf8(&[c[0], *c.get(1).unwrap_or(&b'0')]).unwrap_or("00"), 16).unwrap_or(0)).collect()
}

fn ascii85(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut group = Vec::new();
    for &b in data {
        match b {
            b'~' => break,
            b'z' if group.is_empty() => out.extend_from_slice(&[0; 4]),
            b'!'..=b'u' => {
                group.push(b - b'!');
                if group.len() == 5 {
                    let v = group.iter().fold(0u64, |v, d| v * 85 + u64::from(*d)) as u32;
                    out.extend_from_slice(&v.to_be_bytes());
                    group.clear();
                }
            }
            _ => {}
        }
    }
    if group.len() > 1 {
        let n = group.len();
        group.resize(5, 84);
        let v = group.iter().fold(0u64, |v, d| v * 85 + u64::from(*d)) as u32;
        out.extend_from_slice(&v.to_be_bytes()[..n - 1]);
    }
    out
}

fn run_length(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut k = 0;
    while k < data.len() {
        let n = data[k];
        k += 1;
        match n {
            128 => break,
            0..=127 => {
                let end = (k + usize::from(n) + 1).min(data.len());
                out.extend_from_slice(&data[k..end]);
                k = end;
            }
            _ => {
                if let Some(&b) = data.get(k) {
                    out.extend(std::iter::repeat_n(b, 257 - usize::from(n)));
                }
                k += 1;
            }
        }
    }
    out
}

fn lzw(data: &[u8], early: bool) -> Vec<u8> {
    let mut out = Vec::new();
    let mut table: Vec<Vec<u8>> = Vec::new();
    let reset = |t: &mut Vec<Vec<u8>>| {
        t.clear();
        t.extend((0..=255u8).map(|b| vec![b]));
        t.push(Vec::new());
        t.push(Vec::new());
    };
    reset(&mut table);
    let (mut bits, mut acc, mut nbits) = (9u32, 0u32, 0u32);
    let mut prev: Option<Vec<u8>> = None;
    for &b in data {
        acc = (acc << 8) | u32::from(b);
        nbits += 8;
        while nbits >= bits {
            let code = ((acc >> (nbits - bits)) & ((1 << bits) - 1)) as usize;
            nbits -= bits;
            match code {
                256 => {
                    reset(&mut table);
                    bits = 9;
                    prev = None;
                    continue;
                }
                257 => return out,
                _ => {}
            }
            let entry = if code < table.len() {
                table[code].clone()
            } else if let Some(p) = &prev {
                let mut e = p.clone();
                e.push(p[0]);
                e
            } else {
                return out;
            };
            out.extend_from_slice(&entry);
            if let Some(p) = prev {
                let mut e = p;
                e.push(entry[0]);
                table.push(e);
            }
            let size = table.len() + usize::from(early);
            bits = if size >= 2048 { 12 } else if size >= 1024 { 11 } else if size >= 512 { 10 } else { 9 };
            prev = Some(entry);
        }
    }
    out
}

// ------------------------------------------------------------------ images

/// An image of a page: grey levels, or for a stencil mask whether a pixel is painted.
struct Image {
    w: usize,
    h: usize,
    grey: Vec<u8>,
    mask: Option<Vec<bool>>,
}

/// Decodes an image XObject.
fn image(file: &File, o: &Obj) -> Result<Image, String> {
    let Some(d) = o.dict() else { return Err(tr("Not an image").to_string()) };
    let w = file.num(d, "Width").unwrap_or(0.0).max(0.0) as usize;
    let h = file.num(d, "Height").unwrap_or(0.0).max(0.0) as usize;
    if w == 0 || h == 0 || w * h > MAX_PIXELS {
        return Err(tr("An image of the PDF file has no size or is too large").to_string());
    }
    let is_mask = matches!(file.key(d, "ImageMask"), Obj::Bool(true));
    let bpc = if is_mask { 1 } else { file.num(d, "BitsPerComponent").unwrap_or(8.0) as usize };
    let decode: Vec<f64> = match file.key(d, "Decode") {
        Obj::Array(v) => v.iter().filter_map(|x| file.get(x).num()).collect(),
        _ => Vec::new(),
    };
    let inverted = decode.len() >= 2 && decode[0] > decode[1];
    let (data, rest) = file.filtered(o)?;
    // One sample per pixel (0 black … 255 white), or grey levels straight from a codec.
    let samples = match rest.first().map(|(f, p)| (f.as_str(), p)) {
        Some(("DCTDecode" | "DCT", _)) => {
            let img = image::load_from_memory_with_format(&data, image::ImageFormat::Jpeg).map_err(|e| tr_fmt!("An image of the PDF file cannot be read: {}", e))?;
            let g = img.to_luma8();
            if g.width() as usize != w || g.height() as usize != h {
                return Err(tr("An image of the PDF file has no size or is too large").to_string());
            }
            return Ok(Image { w, h, grey: g.into_raw(), mask: None });
        }
        Some(("CCITTFaxDecode" | "CCF", p)) => ccitt(&data, p, file, w, h)?,
        Some((f, _)) => return Err(tr_fmt!("Unsupported PDF filter {}", f)),
        None => {
            let (space, n) = if is_mask { (Obj::Null, 1) } else { colour_space(file, file.key(d, "ColorSpace")) };
            let bits = unpack(&data, w, h, n, bpc);
            if !is_mask {
                return Ok(Image { w, h, grey: to_grey(file, &space, &bits, n, bpc, inverted), mask: None });
            }
            bits.iter().map(|&v| if v > 0 { 255 } else { 0 }).collect()
        }
    };
    if is_mask {
        // A sample 0 paints unless the decode array is inverted.
        let mask = samples.iter().map(|&v| (v == 0) != inverted).collect();
        return Ok(Image { w, h, grey: Vec::new(), mask: Some(mask) });
    }
    let grey = samples.iter().map(|&v| if inverted { 255 - v } else { v }).collect();
    Ok(Image { w, h, grey, mask: None })
}

/// The samples of a CCITT fax image: 0 black, 255 white (as `BlackIs1` false says).
fn ccitt(data: &[u8], p: &Dict, file: &File, w: usize, h: usize) -> Result<Vec<u8>, String> {
    let k = file.num(p, "K").unwrap_or(0.0);
    let columns = file.num(p, "Columns").unwrap_or(1728.0) as usize;
    let black_is_1 = matches!(file.key(p, "BlackIs1"), Obj::Bool(true));
    if columns > u16::MAX as usize || h > u16::MAX as usize {
        return Err(tr("An image of the PDF file has no size or is too large").to_string());
    }
    let mut rows: Vec<Vec<u8>> = Vec::with_capacity(h);
    let line = |t: &[u16]| -> Vec<u8> { fax::decoder::pels(t, columns as u16).map(|c| if c == fax::Color::Black { 0 } else { 255 }).collect() };
    let ok = if k < 0.0 {
        fax::decoder::decode_g4(data.iter().copied(), columns as u16, Some(h as u16), |t| rows.push(line(t)))
    } else {
        fax::decoder::decode_g3(data.iter().copied(), |t| rows.push(line(t)))
    };
    if ok.is_none() && rows.is_empty() {
        return Err(tr("A fax image of the PDF file cannot be read").to_string());
    }
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
        let row = rows.get(y);
        for x in 0..w {
            let v = row.and_then(|r| r.get(x)).copied().unwrap_or(255);
            out.push(if black_is_1 { 255 - v } else { v });
        }
    }
    Ok(out)
}

/// The colour space as itself and its number of components.
fn colour_space(file: &File, cs: &Obj) -> (Obj, usize) {
    let n = match cs {
        Obj::Name(n) => match n.as_str() {
            "DeviceRGB" | "RGB" | "CalRGB" | "Lab" => 3,
            "DeviceCMYK" | "CMYK" => 4,
            _ => 1,
        },
        Obj::Array(v) => match v.first().map(|x| file.get(x)).and_then(Obj::name) {
            Some("ICCBased") => v.get(1).map(|s| file.get(s)).and_then(Obj::dict).and_then(|d| file.num(d, "N")).unwrap_or(3.0) as usize,
            Some("CalRGB" | "Lab") => 3,
            Some("DeviceN") => match v.get(1).map(|x| file.get(x)) {
                Some(Obj::Array(c)) => c.len(),
                _ => 1,
            },
            _ => 1,
        },
        _ => 1,
    };
    (cs.clone(), n.max(1))
}

/// The samples of an image, unpacked to one value of 0 … 255 per component.
fn unpack(data: &[u8], w: usize, h: usize, n: usize, bpc: usize) -> Vec<u8> {
    let row_bytes = (w * n * bpc).div_ceil(8);
    let mut out = Vec::with_capacity(w * h * n);
    for y in 0..h {
        let row = data.get(y * row_bytes..((y + 1) * row_bytes).min(data.len())).unwrap_or(&[]);
        for k in 0..w * n {
            let v = match bpc {
                8 => u32::from(*row.get(k).unwrap_or(&255)),
                16 => u32::from(*row.get(2 * k).unwrap_or(&255)),
                1 | 2 | 4 => {
                    let bit = k * bpc;
                    let byte = u32::from(*row.get(bit / 8).unwrap_or(&255));
                    (byte >> (8 - bpc - bit % 8)) & ((1 << bpc) - 1)
                }
                _ => 0,
            };
            out.push(v as u8);
        }
    }
    out
}

/// Grey levels of unpacked samples in a colour space.
fn to_grey(file: &File, space: &Obj, s: &[u8], n: usize, bpc: usize, inverted: bool) -> Vec<u8> {
    let scale = |v: u8| -> u8 {
        let max = (1u32 << bpc.min(8)) - 1;
        if bpc >= 8 {
            v
        } else {
            ((u32::from(v) * 255) / max.max(1)) as u8
        }
    };
    if let Obj::Array(v) = space {
        if v.first().map(|x| file.get(x)).and_then(Obj::name) == Some("Indexed") {
            let (base, bn) = colour_space(file, v.get(1).map(|x| file.get(x)).unwrap_or(&Obj::Null));
            let hival = v.get(2).map(|x| file.get(x)).and_then(Obj::num).unwrap_or(255.0) as usize;
            let lookup = match v.get(3).map(|x| file.get(x)) {
                Some(Obj::Str(b)) => b.clone(),
                Some(st @ Obj::Stream(..)) => file.decoded(st).unwrap_or_default(),
                _ => Vec::new(),
            };
            let table: Vec<u8> = (0..=hival).map(|i| colour(&base, lookup.get(i * bn..(i + 1) * bn).unwrap_or(&[255, 255, 255, 0][..bn.min(4)]), bn)).collect();
            return s.iter().map(|&i| *table.get(usize::from(i)).unwrap_or(&255)).collect();
        }
    }
    s.chunks(n)
        .map(|c| {
            let c: Vec<u8> = c.iter().map(|&v| scale(v)).collect();
            let g = colour(space, &c, n);
            if inverted {
                255 - g
            } else {
                g
            }
        })
        .collect()
}

/// The grey level of a colour of `n` components.
fn colour(_space: &Obj, c: &[u8], n: usize) -> u8 {
    match (n, c) {
        (3, [r, g, b, ..]) => luma(*r, *g, *b),
        (4, [cy, m, y, k, ..]) => {
            let r = 255 - (u32::from(*cy) + u32::from(*k)).min(255);
            let g = 255 - (u32::from(*m) + u32::from(*k)).min(255);
            let b = 255 - (u32::from(*y) + u32::from(*k)).min(255);
            luma(r as u8, g as u8, b as u8)
        }
        (_, [v, ..]) => *v,
        _ => 255,
    }
}

// ------------------------------------------------------------------ pages

/// A transformation: `[a b c d e f]` maps (x, y) to (a x + c y + e, b x + d y + f).
type Matrix = [f64; 6];

fn mul(m: &Matrix, n: &Matrix) -> Matrix {
    // m then n.
    [m[0] * n[0] + m[1] * n[2], m[0] * n[1] + m[1] * n[3], m[2] * n[0] + m[3] * n[2], m[2] * n[1] + m[3] * n[3], m[4] * n[0] + m[5] * n[2] + n[4], m[4] * n[1] + m[5] * n[3] + n[5]]
}

fn apply(m: &Matrix, x: f64, y: f64) -> (f64, f64) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

fn invert(m: &Matrix) -> Option<Matrix> {
    let det = m[0] * m[3] - m[1] * m[2];
    if det.abs() < 1e-12 {
        return None;
    }
    let (a, b, c, d) = (m[3] / det, -m[1] / det, -m[2] / det, m[0] / det);
    Some([a, b, c, d, -(m[4] * a + m[5] * c), -(m[4] * b + m[5] * d)])
}

/// What a content stream draws, in page coordinates (points).
enum Mark {
    Image { m: Matrix, obj: Obj, fill: u8 },
    Fill { paths: Vec<Vec<(f64, f64)>>, even_odd: bool, grey: u8 },
}

#[derive(Clone)]
struct GState {
    m: Matrix,
    fill: u8,
    stroke: u8,
    width: f64,
}

/// The marks of a content stream with resources `res` (forms followed).
fn marks(file: &File, content: &[u8], res: &Dict, m0: Matrix, out: &mut Vec<Mark>, depth: usize) {
    if depth > 8 {
        return;
    }
    let xobjects = res.get("XObject").map(|x| file.get(x)).and_then(Obj::dict).cloned().unwrap_or_default();
    let mut lx = Lexer::new(content, 0);
    let mut stack: Vec<Obj> = Vec::new();
    let mut gs = GState { m: m0, fill: 0, stroke: 0, width: 1.0 };
    let mut saved: Vec<GState> = Vec::new();
    let mut path: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut cur: Vec<(f64, f64)> = Vec::new();
    let mut steps = 0usize;
    loop {
        steps += 1;
        if steps > 50_000_000 {
            break;
        }
        if let Some(o) = lx.obj(0) {
            stack.push(o);
            continue;
        }
        lx.skip();
        let op = lx.word().to_vec();
        if op.is_empty() {
            if lx.peek().is_none() {
                break;
            }
            lx.at += 1;
            continue;
        }
        let nums: Vec<f64> = stack.iter().filter_map(Obj::num).collect();
        let grey = |v: &[f64]| -> u8 {
            let c: Vec<u8> = v.iter().map(|x| (x.clamp(0.0, 1.0) * 255.0).round() as u8).collect();
            colour(&Obj::Null, &c, c.len())
        };
        let to_page = |gs: &GState, x: f64, y: f64| apply(&gs.m, x, y);
        match op.as_slice() {
            b"q" => saved.push(gs.clone()),
            b"Q" => {
                if let Some(g) = saved.pop() {
                    gs = g;
                }
            }
            b"cm" if nums.len() >= 6 => {
                let n = [nums[0], nums[1], nums[2], nums[3], nums[4], nums[5]];
                gs.m = mul(&n, &gs.m);
            }
            b"w" if !nums.is_empty() => gs.width = nums[0],
            b"g" | b"rg" | b"k" | b"sc" | b"scn" if !nums.is_empty() => gs.fill = grey(&nums),
            b"G" | b"RG" | b"K" | b"SC" | b"SCN" if !nums.is_empty() => gs.stroke = grey(&nums),
            b"m" if nums.len() >= 2 => {
                if cur.len() > 1 {
                    path.push(std::mem::take(&mut cur));
                }
                cur = vec![to_page(&gs, nums[0], nums[1])];
            }
            b"l" if nums.len() >= 2 => cur.push(to_page(&gs, nums[0], nums[1])),
            b"c" | b"v" | b"y" if nums.len() >= 4 => {
                let start = cur.last().copied().unwrap_or((0.0, 0.0));
                let pts: Vec<(f64, f64)> = nums.chunks(2).map(|p| to_page(&gs, p[0], p[1])).collect();
                let (p1, p2, p3) = match (op.as_slice(), pts.len()) {
                    (b"c", 3) => (pts[0], pts[1], pts[2]),
                    (b"v", 2) => (start, pts[0], pts[1]),
                    (b"y", 2) => (pts[0], pts[1], pts[1]),
                    _ => (start, start, *pts.last().unwrap_or(&start)),
                };
                for i in 1..=8 {
                    let t = f64::from(i) / 8.0;
                    let u = 1.0 - t;
                    let x = u * u * u * start.0 + 3.0 * u * u * t * p1.0 + 3.0 * u * t * t * p2.0 + t * t * t * p3.0;
                    let y = u * u * u * start.1 + 3.0 * u * u * t * p1.1 + 3.0 * u * t * t * p2.1 + t * t * t * p3.1;
                    cur.push((x, y));
                }
            }
            b"re" if nums.len() >= 4 => {
                if cur.len() > 1 {
                    path.push(std::mem::take(&mut cur));
                }
                let (x, y, w, h) = (nums[0], nums[1], nums[2], nums[3]);
                path.push(vec![to_page(&gs, x, y), to_page(&gs, x + w, y), to_page(&gs, x + w, y + h), to_page(&gs, x, y + h), to_page(&gs, x, y)]);
                cur.clear();
            }
            b"h" => {
                if let Some(&f) = cur.first() {
                    cur.push(f);
                }
            }
            b"S" | b"s" | b"f" | b"F" | b"f*" | b"B" | b"B*" | b"b" | b"b*" | b"n" => {
                if op == b"s" || op == b"b" || op == b"b*" {
                    if let Some(&f) = cur.first() {
                        cur.push(f);
                    }
                }
                if cur.len() > 1 {
                    path.push(std::mem::take(&mut cur));
                }
                cur.clear();
                let fill = matches!(op.as_slice(), b"f" | b"F" | b"f*" | b"B" | b"B*" | b"b" | b"b*");
                let stroke = matches!(op.as_slice(), b"S" | b"s" | b"B" | b"B*" | b"b" | b"b*");
                if fill && gs.fill < 250 {
                    let closed: Vec<Vec<(f64, f64)>> = path.iter().filter(|p| p.len() > 2).cloned().collect();
                    out.push(Mark::Fill { paths: closed, even_odd: op.ends_with(b"*"), grey: gs.fill });
                }
                if stroke && gs.stroke < 250 {
                    // Each segment as a thin quadrilateral of the line's width (at least a
                    // hairline).
                    let scale = (gs.m[0] * gs.m[3] - gs.m[1] * gs.m[2]).abs().sqrt();
                    let half = (gs.width * scale / 2.0).max(0.0);
                    let mut quads = Vec::new();
                    for p in &path {
                        for s in p.windows(2) {
                            let ((x0, y0), (x1, y1)) = (s[0], s[1]);
                            let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
                            if len < 1e-9 {
                                continue;
                            }
                            let (nx, ny) = (-(y1 - y0) / len * half, (x1 - x0) / len * half);
                            quads.push(vec![(x0 + nx, y0 + ny), (x1 + nx, y1 + ny), (x1 - nx, y1 - ny), (x0 - nx, y0 - ny)]);
                        }
                    }
                    out.push(Mark::Fill { paths: quads, even_odd: false, grey: gs.stroke });
                }
                path.clear();
            }
            b"Do" => {
                if let Some(Obj::Name(n)) = stack.last() {
                    let o = xobjects.get(n).map(|x| file.get(x)).cloned().unwrap_or(Obj::Null);
                    if let Some(d) = o.dict() {
                        match file.key(d, "Subtype").name() {
                            Some("Image") => out.push(Mark::Image { m: gs.m, obj: o.clone(), fill: gs.fill }),
                            Some("Form") => {
                                let fm = match file.key(d, "Matrix") {
                                    Obj::Array(v) => {
                                        let n: Vec<f64> = v.iter().filter_map(|x| file.get(x).num()).collect();
                                        if n.len() == 6 {
                                            [n[0], n[1], n[2], n[3], n[4], n[5]]
                                        } else {
                                            [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
                                        }
                                    }
                                    _ => [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                                };
                                let fres = d.get("Resources").map(|r| file.get(r)).and_then(Obj::dict).cloned().unwrap_or_else(|| res.clone());
                                if let Ok(data) = file.decoded(&o) {
                                    marks(file, &data, &fres, mul(&fm, &gs.m), out, depth + 1);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            b"BI" => {
                // Inline images are skipped.
                if let Some(k) = find(content, b"EI", lx.at) {
                    lx.at = k + 2;
                }
            }
            _ => {}
        }
        stack.clear();
    }
}

/// Fills polygons (pixel coordinates) on a picture.
fn fill(img: &mut Gray, paths: &[Vec<(f64, f64)>], even_odd: bool, grey: u8) {
    let mut edges: Vec<(f64, f64, f64, f64)> = Vec::new();
    let (mut y0, mut y1) = (f64::MAX, f64::MIN);
    for p in paths {
        let n = p.len();
        for k in 0..n {
            let (a, b) = (p[k], p[(k + 1) % n]);
            if (a.1 - b.1).abs() < 1e-12 {
                continue;
            }
            edges.push((a.0, a.1, b.0, b.1));
            y0 = y0.min(a.1.min(b.1));
            y1 = y1.max(a.1.max(b.1));
        }
    }
    if edges.is_empty() {
        return;
    }
    let (ya, yb) = (y0.floor().max(0.0) as usize, (y1.ceil().max(0.0) as usize).min(img.h));
    let mut xs: Vec<(f64, i32)> = Vec::new();
    for y in ya..yb {
        let sy = y as f64 + 0.5;
        xs.clear();
        for &(x0, y0, x1, y1) in &edges {
            if (y0 <= sy) != (y1 <= sy) {
                xs.push((x0 + (sy - y0) / (y1 - y0) * (x1 - x0), if y1 > y0 { 1 } else { -1 }));
            }
        }
        xs.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut wind = 0;
        for k in 0..xs.len() {
            let before = wind;
            wind = if even_odd { (wind + 1) % 2 } else { wind + xs[k].1 };
            let inside = if even_odd { before == 1 } else { before != 0 };
            if inside && k > 0 {
                let (a, b) = (xs[k - 1].0, xs[k].0);
                let (xa, xb) = ((a + 0.5).floor().max(0.0) as usize, ((b + 0.5).floor().max(0.0) as usize).min(img.w));
                for x in xa..xb {
                    let v = img.at(x, y).min(grey);
                    img.set(x, y, v);
                }
                // A hairline narrower than a pixel still shows.
                if xb <= xa && xa < img.w {
                    let v = img.at(xa, y).min(grey);
                    img.set(xa, y, v);
                }
            }
        }
    }
}

/// The pages of a PDF file drawn as grey pictures.
pub fn pages(bytes: &[u8]) -> Result<Vec<Sheet>, String> {
    let file = File::parse(bytes)?;
    let refs = file.pages();
    if refs.is_empty() {
        return Err(tr("The PDF file has no pages").to_string());
    }
    let mut out = Vec::new();
    let mut errors = Vec::new();
    for p in refs {
        let mut content = Vec::new();
        for c in &p.contents {
            match file.decoded(file.get(c)) {
                Ok(d) => {
                    content.extend_from_slice(&d);
                    content.push(b'\n');
                }
                Err(e) => errors.push(e),
            }
        }
        let [x0, y0, x1, y1] = p.media;
        // Page points to pixels of the picture: y down from the top.
        let mut all = Vec::new();
        marks(&file, &content, &p.resources, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0], &mut all, 0);
        // The image covering most of the page sets the resolution.
        let mut dpi: f32 = 0.0;
        let mut largest = 0.0;
        let mut images = Vec::new();
        for mk in &all {
            if let Mark::Image { m, obj, .. } = mk {
                match image(&file, obj) {
                    Ok(img) => {
                        let (sx, sy) = ((m[0].powi(2) + m[1].powi(2)).sqrt(), (m[2].powi(2) + m[3].powi(2)).sqrt());
                        if sx > 1e-6 && sy > 1e-6 && sx * sy > largest {
                            largest = sx * sy;
                            dpi = (72.0 * img.w as f64 / sx).max(72.0 * img.h as f64 / sy) as f32;
                        }
                        images.push(Some(img));
                    }
                    Err(e) => {
                        errors.push(e);
                        images.push(None);
                    }
                }
            }
        }
        if dpi <= 0.0 {
            dpi = DRAWING_DPI;
        }
        let (w_pt, h_pt) = (x1 - x0, y1 - y0);
        let mut dpi = dpi.min(MAX_DPI);
        while (w_pt * h_pt) as f32 * (dpi / 72.0).powi(2) > MAX_PIXELS as f32 && dpi > 10.0 {
            dpi *= 0.8;
        }
        let k = f64::from(dpi) / 72.0;
        let (w, h) = (((w_pt * k).round() as usize).max(1), ((h_pt * k).round() as usize).max(1));
        let mut page = Gray::new(w, h, 255);
        let to_px = |(x, y): (f64, f64)| ((x - x0) * k, (y1 - y) * k);
        let mut drawn = false;
        let mut next_image = images.into_iter();
        for mk in &all {
            match mk {
                Mark::Image { m, fill, .. } => {
                    let Some(Some(img)) = next_image.next() else { continue };
                    let Some(inv) = invert(m) else { continue };
                    // The image's unit square on the page.
                    let corners = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].map(|(u, v)| to_px(apply(m, u, v)));
                    let (xa, xb) = (corners.iter().map(|c| c.0).fold(f64::MAX, f64::min), corners.iter().map(|c| c.0).fold(f64::MIN, f64::max));
                    let (ya, yb) = (corners.iter().map(|c| c.1).fold(f64::MAX, f64::min), corners.iter().map(|c| c.1).fold(f64::MIN, f64::max));
                    for py in (ya.floor().max(0.0) as usize)..(yb.ceil().max(0.0) as usize).min(h) {
                        for px in (xa.floor().max(0.0) as usize)..(xb.ceil().max(0.0) as usize).min(w) {
                            // The pixel of the image at a place in this one.
                            let sample = |dx: f64, dy: f64| {
                                let (x, y) = (px as f64 + dx, py as f64 + dy);
                                let (u, v) = apply(&inv, x / k + x0, y1 - y / k);
                                ((0.0..1.0).contains(&u) && (0.0..1.0).contains(&v)).then(|| {
                                    let (i, j) = ((u * img.w as f64) as usize, ((1.0 - v) * img.h as f64) as usize);
                                    j.min(img.h - 1) * img.w + i.min(img.w - 1)
                                })
                            };
                            match &img.mask {
                                // Painted where any of four places is (finer masks keep
                                // their thin lines).
                                Some(mask) => {
                                    if [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)].iter().any(|&(dx, dy)| sample(dx, dy).is_some_and(|at| mask[at])) {
                                        page.set(px, py, *fill);
                                    }
                                }
                                None => {
                                    if let Some(at) = sample(0.5, 0.5) {
                                        page.set(px, py, img.grey[at]);
                                    }
                                }
                            }
                        }
                    }
                    drawn = true;
                }
                Mark::Fill { paths, even_odd, grey } => {
                    let pix: Vec<Vec<(f64, f64)>> = paths.iter().map(|p| p.iter().map(|&q| to_px(q)).collect()).collect();
                    fill(&mut page, &pix, *even_odd, *grey);
                    drawn = true;
                }
            }
        }
        if !drawn {
            continue;
        }
        let page = match p.rotate {
            90 => turned(&page, 1),
            180 => turned(&page, 2),
            270 => turned(&page, 3),
            _ => page,
        };
        out.push(Sheet { img: page, paper: 25.4 / dpi });
    }
    if out.is_empty() {
        return Err(errors.into_iter().next().unwrap_or_else(|| tr("The PDF file has nothing to show").to_string()));
    }
    Ok(out)
}

/// The picture turned clockwise by `quarters` quarter turns (as a page's /Rotate).
pub fn turned(img: &Gray, quarters: u32) -> Gray {
    let mut out = img.clone();
    for _ in 0..quarters % 4 {
        let src = out;
        let mut t = Gray::new(src.h, src.w, 255);
        for y in 0..src.h {
            for x in 0..src.w {
                t.set(src.h - 1 - y, x, src.at(x, y));
            }
        }
        out = t;
    }
    out
}

#[cfg(test)]
mod tests;
