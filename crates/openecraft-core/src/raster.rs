//! Grey images (8 bit, 0 black … 255 white) and pixel masks: read from and written as
//! PNG, smoothed, shrunk and filtered with the morphology the detection of walls in
//! floor plans ([`crate::wallscan`]) works with.

/// A grey image, row by row from the top left.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Gray {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u8>,
}

/// Brightness of a colour (ITU-R BT.601).
pub fn luma(r: u8, g: u8, b: u8) -> u8 {
    ((u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114 + 500) / 1000) as u8
}

/// Largest image read from a file: 100 megapixels.
pub const MAX_PIXELS: usize = 100_000_000;

impl Gray {
    pub fn new(w: usize, h: usize, fill: u8) -> Gray {
        Gray { w, h, px: vec![fill; w * h] }
    }

    pub fn at(&self, x: usize, y: usize) -> u8 {
        self.px[y * self.w + x]
    }

    pub fn set(&mut self, x: usize, y: usize, v: u8) {
        self.px[y * self.w + x] = v;
    }

    /// Fills the rectangle from `lo` to `hi` (pixels, `hi` excluded, clipped).
    pub fn fill(&mut self, lo: [usize; 2], hi: [usize; 2], v: u8) {
        for y in lo[1]..hi[1].min(self.h) {
            for x in lo[0]..hi[0].min(self.w) {
                self.set(x, y, v);
            }
        }
    }

    /// The image of a PNG file in grey: colours by their brightness, transparent parts
    /// on white.
    pub fn from_png(bytes: &[u8]) -> Result<Gray, String> {
        let bad = |e: &dyn std::fmt::Display| openecraft_i18n::tr_fmt!("Not a readable PNG image: {}", e);
        let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info().map_err(|e| bad(&e))?;
        let (w, h) = (reader.info().width as usize, reader.info().height as usize);
        if w * h > MAX_PIXELS {
            return Err(openecraft_i18n::tr_fmt!("The image is too large ({} × {} pixels)", w, h));
        }
        let size = reader.output_buffer_size().ok_or_else(|| bad(&"size"))?;
        let mut buffer = vec![0; size];
        let info = reader.next_frame(&mut buffer).map_err(|e| bad(&e))?;
        let channels = match info.color_type {
            png::ColorType::Grayscale => 1,
            png::ColorType::GrayscaleAlpha => 2,
            png::ColorType::Rgb => 3,
            png::ColorType::Rgba => 4,
            png::ColorType::Indexed => return Err(bad(&"palette")),
        };
        let over_white = |v: u8, a: u8| ((u32::from(v) * u32::from(a) + 255 * (255 - u32::from(a)) + 127) / 255) as u8;
        let px = buffer[..info.buffer_size()]
            .chunks_exact(channels)
            .map(|p| match channels {
                1 => p[0],
                2 => over_white(p[0], p[1]),
                3 => luma(p[0], p[1], p[2]),
                _ => over_white(luma(p[0], p[1], p[2]), p[3]),
            })
            .collect();
        Ok(Gray { w, h, px })
    }

    /// The image as a grey PNG file.
    pub fn to_png(&self) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut out, self.w as u32, self.h as u32);
            enc.set_color(png::ColorType::Grayscale);
            enc.set_depth(png::BitDepth::Eight);
            enc.set_compression(png::Compression::High);
            if let Ok(mut writer) = enc.write_header() {
                let _ = writer.write_image_data(&self.px);
            }
        }
        out
    }

    /// The image `f` times smaller (the mean of every `f` × `f` block).
    pub fn shrunk(&self, f: usize) -> Gray {
        if f <= 1 {
            return self.clone();
        }
        let (w, h) = (self.w.div_ceil(f), self.h.div_ceil(f));
        let mut sum = vec![0u32; w * h];
        let mut n = vec![0u32; w * h];
        for y in 0..self.h {
            for x in 0..self.w {
                let k = (y / f) * w + x / f;
                sum[k] += u32::from(self.at(x, y));
                n[k] += 1;
            }
        }
        Gray { w, h, px: sum.iter().zip(&n).map(|(s, n)| ((s + n / 2) / n.max(&1)) as u8).collect() }
    }

    /// The image smoothed: every pixel the mean of the 3 × 3 pixels round it.
    pub fn blurred(&self) -> Gray {
        let mut out = self.clone();
        for y in 0..self.h {
            for x in 0..self.w {
                let (mut s, mut n) = (0u32, 0u32);
                for yy in y.saturating_sub(1)..(y + 2).min(self.h) {
                    for xx in x.saturating_sub(1)..(x + 2).min(self.w) {
                        s += u32::from(self.at(xx, yy));
                        n += 1;
                    }
                }
                out.set(x, y, ((s + n / 2) / n) as u8);
            }
        }
        out
    }

    /// The image without noise: every pixel the median of the 3 × 3 pixels round it
    /// (edges stay sharp, specks go).
    pub fn median(&self) -> Gray {
        let mut out = self.clone();
        let mut v = [0u8; 9];
        for y in 1..self.h.saturating_sub(1) {
            for x in 1..self.w.saturating_sub(1) {
                let mut n = 0;
                for yy in y - 1..=y + 1 {
                    for xx in x - 1..=x + 1 {
                        v[n] = self.at(xx, yy);
                        n += 1;
                    }
                }
                v.sort_unstable();
                out.set(x, y, v[4]);
            }
        }
        out
    }

    /// The brightness of the paper: the most frequent of the brighter half of the
    /// levels.
    pub fn paper(&self) -> u8 {
        let mut hist = [0u32; 256];
        for &v in &self.px {
            hist[usize::from(v)] += 1;
        }
        (128..256).max_by_key(|&v| (hist[v], v)).unwrap_or(255) as u8
    }

    /// The brightest level within `r` pixels (a square) of every pixel.
    pub fn local_max(&self, r: usize) -> Gray {
        let pass = |src: &Gray, along_x: bool| {
            let mut out = src.clone();
            for y in 0..src.h {
                for x in 0..src.w {
                    let (a, b) = if along_x { (x.saturating_sub(r), (x + r + 1).min(src.w)) } else { (y.saturating_sub(r), (y + r + 1).min(src.h)) };
                    let m = (a..b).map(|k| if along_x { src.at(k, y) } else { src.at(x, k) }).max().unwrap_or(0);
                    out.set(x, y, m);
                }
            }
            out
        };
        pass(&pass(self, true), false)
    }

    /// The brightness of the paper round every pixel: the brightest level within about
    /// `r` pixels (measured on the image shrunk, so blots and noise do not count).
    pub fn background(&self, r: usize) -> Gray {
        let f = (r / 8).clamp(1, 8);
        let small = self.shrunk(f).local_max(r.div_ceil(f));
        let mut out = Gray::new(self.w, self.h, 255);
        for y in 0..self.h {
            for x in 0..self.w {
                out.set(x, y, small.at((x / f).min(small.w - 1), (y / f).min(small.h - 1)));
            }
        }
        out
    }
}

/// Pixels that are on or off, row by row from the top left.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mask {
    pub w: usize,
    pub h: usize,
    pub on: Vec<bool>,
}

/// A connected part of a mask: its pixels and their extent (`hi` excluded).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Component {
    pub area: usize,
    pub lo: [usize; 2],
    pub hi: [usize; 2],
}

impl Component {
    /// The longer side of its extent.
    pub fn span(&self) -> usize {
        (self.hi[0] - self.lo[0]).max(self.hi[1] - self.lo[1])
    }

    pub fn touches_border(&self, w: usize, h: usize) -> bool {
        self.near_border(w, h, 0)
    }

    /// It comes within `margin` pixels of the border of a `w` × `h` image.
    pub fn near_border(&self, w: usize, h: usize, margin: usize) -> bool {
        self.lo[0] <= margin || self.lo[1] <= margin || self.hi[0] + margin >= w || self.hi[1] + margin >= h
    }
}

impl Mask {
    pub fn new(w: usize, h: usize) -> Mask {
        Mask { w, h, on: vec![false; w * h] }
    }

    /// The pixels of `img` darker than `threshold`.
    pub fn darker(img: &Gray, threshold: u8) -> Mask {
        Mask { w: img.w, h: img.h, on: img.px.iter().map(|&v| v < threshold).collect() }
    }

    /// The pixels of `img` more than `contrast` darker than the paper `background`
    /// round them ([`Gray::background`]).
    pub fn ink(img: &Gray, background: &Gray, contrast: u8) -> Mask {
        Mask { w: img.w, h: img.h, on: img.px.iter().zip(&background.px).map(|(&v, &b)| v < b.saturating_sub(contrast)).collect() }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        x < self.w && y < self.h && self.on[y * self.w + x]
    }

    /// Like [`Mask::get`] with signed coordinates: off outside the mask.
    pub fn at(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && self.get(x as usize, y as usize)
    }

    pub fn set(&mut self, x: usize, y: usize, v: bool) {
        if x < self.w && y < self.h {
            self.on[y * self.w + x] = v;
        }
    }

    /// Switches on the rectangle from `lo` to `hi` (`hi` excluded, clipped).
    pub fn fill(&mut self, lo: [usize; 2], hi: [usize; 2]) {
        for y in lo[1]..hi[1].min(self.h) {
            for x in lo[0]..hi[0].min(self.w) {
                self.on[y * self.w + x] = true;
            }
        }
    }

    pub fn count(&self) -> usize {
        self.on.iter().filter(|v| **v).count()
    }

    pub fn inverted(&self) -> Mask {
        Mask { w: self.w, h: self.h, on: self.on.iter().map(|v| !v).collect() }
    }

    /// One pass along x (`along_x`) or y over a window of `2 r + 1` pixels: a pixel stays
    /// on when all pixels of the window are on (`all`) or when any is.
    fn pass(&self, r: usize, along_x: bool, all: bool) -> Mask {
        let (n, lines) = if along_x { (self.w, self.h) } else { (self.h, self.w) };
        let idx = |line: usize, k: usize| if along_x { line * self.w + k } else { k * self.w + line };
        let mut out = Mask::new(self.w, self.h);
        let win = 2 * r + 1;
        for line in 0..lines {
            // Pixels on in the window round k (outside the mask counts as off).
            let mut count = (0..r.min(n)).filter(|&k| self.on[idx(line, k)]).count();
            for k in 0..n {
                if k + r < n && self.on[idx(line, k + r)] {
                    count += 1;
                }
                if k > r && self.on[idx(line, k - r - 1)] {
                    count -= 1;
                }
                out.on[idx(line, k)] = if all { count == win } else { count > 0 };
            }
        }
        out
    }

    /// Erosion by a square of `2 r + 1` pixels: on where the whole square round the
    /// pixel is on.
    pub fn eroded(&self, r: usize) -> Mask {
        if r == 0 {
            return self.clone();
        }
        self.pass(r, true, true).pass(r, false, true)
    }

    /// Dilation by a square of `2 r + 1` pixels: on where any pixel of the square is.
    pub fn dilated(&self, r: usize) -> Mask {
        if r == 0 {
            return self.clone();
        }
        self.pass(r, true, false).pass(r, false, false)
    }

    /// Opening: removes what is thinner than `2 r + 1` pixels.
    pub fn opened(&self, r: usize) -> Mask {
        self.eroded(r).dilated(r)
    }

    /// Closing: fills gaps and holes narrower than `2 r + 1` pixels.
    pub fn closed(&self, r: usize) -> Mask {
        self.dilated(r).eroded(r)
    }

    /// The 4-connected parts of the pixels that are on: a label per pixel (0 off, `k + 1`
    /// for part `k`) and the parts.
    pub fn components(&self) -> (Vec<u32>, Vec<Component>) {
        let mut label = vec![0u32; self.w * self.h];
        let mut parts = Vec::new();
        let mut stack = Vec::new();
        for start in 0..self.on.len() {
            if !self.on[start] || label[start] != 0 {
                continue;
            }
            let id = parts.len() as u32 + 1;
            let mut c = Component { area: 0, lo: [usize::MAX; 2], hi: [0; 2] };
            label[start] = id;
            stack.push(start);
            while let Some(i) = stack.pop() {
                let (x, y) = (i % self.w, i / self.w);
                c.area += 1;
                c.lo = [c.lo[0].min(x), c.lo[1].min(y)];
                c.hi = [c.hi[0].max(x + 1), c.hi[1].max(y + 1)];
                let mut visit = |j: usize| {
                    if self.on[j] && label[j] == 0 {
                        label[j] = id;
                        stack.push(j);
                    }
                };
                if x > 0 {
                    visit(i - 1);
                }
                if x + 1 < self.w {
                    visit(i + 1);
                }
                if y > 0 {
                    visit(i - self.w);
                }
                if y + 1 < self.h {
                    visit(i + self.w);
                }
            }
            parts.push(c);
        }
        (label, parts)
    }

    /// For every pixel that is on the length of its run of pixels that are on along x
    /// (`along_x`) or y; 0 for those that are off.
    pub fn runs(&self, along_x: bool) -> Vec<u32> {
        let (n, lines) = if along_x { (self.w, self.h) } else { (self.h, self.w) };
        let idx = |line: usize, k: usize| if along_x { line * self.w + k } else { k * self.w + line };
        let mut out = vec![0u32; self.w * self.h];
        for line in 0..lines {
            let mut k = 0;
            while k < n {
                if !self.on[idx(line, k)] {
                    k += 1;
                    continue;
                }
                let start = k;
                while k < n && self.on[idx(line, k)] {
                    k += 1;
                }
                for j in start..k {
                    out[idx(line, j)] = (k - start) as u32;
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests;
