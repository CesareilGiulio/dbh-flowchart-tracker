//! Disegno su un buffer in memoria (tiny-skia) e testo con fontdue.
//! Le coordinate sono in pixel logici; `scale` le porta ai pixel fisici dello schermo.
//!
//! Il buffer è quello della finestra (softbuffer, pixel 0x00RRGGBB in little endian, cioè
//! byte B,G,R,X): tiny-skia scrive R,G,B,A, quindi i colori vengono passati con rosso e blu
//! scambiati. Così non serve un secondo buffer da copiare a ogni fotogramma.

use fontdue::{Font, FontSettings, Metrics};
use std::collections::HashMap;
use tiny_skia::{FillRule, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

static REGULAR: &[u8] = include_bytes!("../assets/Inter-400Regular.ttf");
static SEMIBOLD: &[u8] = include_bytes!("../assets/Inter-600SemiBold.ttf");

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);
impl Color {
    pub const fn hex(v: u32) -> Self { Color((v >> 16) as u8, (v >> 8) as u8, v as u8, 255) }
    /// mescola `self` sopra `bg` con opacità a (come color-mix in CSS)
    pub fn over(self, bg: Color, a: f32) -> Color {
        let m = |x: u8, y: u8| (x as f32 * a + y as f32 * (1.0 - a)).round() as u8;
        Color(m(self.0, bg.0), m(self.1, bg.1), m(self.2, bg.2), 255)
    }
    fn paint(self) -> Paint<'static> {
        let mut p = Paint::default();
        p.set_color_rgba8(self.2, self.1, self.0, self.3); // rosso e blu scambiati, vedi sopra
        p.anti_alias = true;
        p
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weight { Regular, Semibold }

struct Glyph { m: Metrics, bmp: Vec<u8> }

pub struct Fonts {
    fonts: [Font; 2],
    cache: HashMap<(u8, char, u32), Glyph>,
}

impl Fonts {
    pub fn new() -> Self {
        let f = |b: &[u8]| Font::from_bytes(b, FontSettings::default()).expect("font");
        Fonts { fonts: [f(REGULAR), f(SEMIBOLD)], cache: HashMap::new() }
    }
    fn font(&self, w: Weight) -> &Font { &self.fonts[if w == Weight::Regular { 0 } else { 1 }] }
    /// larghezza del testo in pixel logici
    pub fn measure(&self, s: &str, size: f32, w: Weight, spacing: f32) -> f32 {
        let f = self.font(w);
        let mut x = 0.0;
        let mut prev: Option<char> = None;
        for c in s.chars() {
            if let Some(p) = prev { x += f.horizontal_kern(p, c, size).unwrap_or(0.0); }
            x += f.metrics(c, size).advance_width + spacing;
            prev = Some(c);
        }
        x
    }
    /// distanza dalla linea di base al centro delle maiuscole/minuscole, per centrare il testo
    pub fn center_offset(&self, size: f32, w: Weight) -> f32 {
        let lm = self.font(w).horizontal_line_metrics(size).unwrap();
        (lm.ascent + lm.descent) / 2.0
    }
    /// accorcia il testo con "…" finché sta in max_w
    pub fn ellipsize(&self, s: &str, size: f32, w: Weight, max_w: f32) -> String {
        if self.measure(s, size, w, 0.0) <= max_w { return s.to_string(); }
        let mut t: Vec<char> = s.chars().collect();
        while !t.is_empty() {
            t.pop();
            let cand: String = t.iter().collect::<String>().trim_end().to_string() + "…";
            if self.measure(&cand, size, w, 0.0) <= max_w { return cand; }
        }
        "…".into()
    }
    /// divide un testo in righe che stanno in max_w
    pub fn wrap(&self, s: &str, size: f32, w: Weight, max_w: f32) -> Vec<String> {
        let mut lines: Vec<String> = vec![];
        let mut cur = String::new();
        for word in s.split_whitespace() {
            let cand = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
            if !cur.is_empty() && self.measure(&cand, size, w, 0.0) > max_w { lines.push(std::mem::take(&mut cur)); cur = word.to_string(); } else { cur = cand; }
        }
        if !cur.is_empty() || lines.is_empty() { lines.push(cur); }
        lines
    }
}

pub struct Painter<'a> {
    pub px: PixmapMut<'a>,
    pub scale: f32,
    pub fonts: &'a mut Fonts,
}

impl<'a> Painter<'a> {
    pub fn clear(&mut self, c: Color) { self.px.fill(tiny_skia::Color::from_rgba8(c.2, c.1, c.0, 255)); }
    /// rettangolo pieno, allineato ai pixel fisici (bordi netti)
    pub fn fill(&mut self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        let s = self.scale;
        let (x0, y0, x1, y1) = ((x * s).round(), (y * s).round(), ((x + w) * s).round(), ((y + h) * s).round());
        if let Some(r) = Rect::from_ltrb(x0, y0, x1, y1) { self.px.fill_rect(r, &c.paint(), Transform::identity(), None); }
    }
    /// linea orizzontale o verticale sottile e netta (bordi dei pannelli)
    pub fn hline(&mut self, x: f32, y: f32, w: f32, c: Color) {
        let t = self.scale.round().max(1.0) / self.scale;
        self.fill(x, y, w, t, c);
    }
    pub fn vline(&mut self, x: f32, y: f32, h: f32, c: Color) {
        let t = self.scale.round().max(1.0) / self.scale;
        self.fill(x, y, t, h, c);
    }
    fn rounded(&self, x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
        let s = self.scale;
        let (x, y, w, h, r) = (x * s, y * s, w * s, h * s, (r * s).min(w * s / 2.0).min(h * s / 2.0));
        if r <= 0.01 { return Some(PathBuilder::from_rect(Rect::from_xywh(x, y, w, h)?)); }
        let k = 0.552_284_8 * r;
        let mut pb = PathBuilder::new();
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
        pb.line_to(x, y + r);
        pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
        pb.close();
        pb.finish()
    }
    /// rettangolo arrotondato con riempimento e bordo (il bordo resta dentro la forma)
    pub fn rrect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, fill: Option<Color>, stroke: Option<(Color, f32)>) {
        if let Some(c) = fill {
            if let Some(p) = self.rounded(x, y, w, h, r) { self.px.fill_path(&p, &c.paint(), FillRule::Winding, Transform::identity(), None); }
        }
        if let Some((c, sw)) = stroke {
            let i = sw / 2.0;
            if let Some(p) = self.rounded(x + i, y + i, w - sw, h - sw, (r - i).max(0.0)) {
                let st = Stroke { width: sw * self.scale, ..Stroke::default() };
                self.px.stroke_path(&p, &c.paint(), &st, Transform::identity(), None);
            }
        }
    }
    /// spezzata ortogonale; i punti vengono allineati ai pixel perché le linee restino nitide
    pub fn polyline(&mut self, pts: &[(f32, f32)], width: f32, c: Color) {
        if pts.len() < 2 { return; }
        let s = self.scale;
        let wpx = (width * s).max(1.0);
        let snap = |v: f32| { let p = (v * s).round(); if (wpx.round() as i32) % 2 == 1 { p + 0.5 } else { p } };
        let mut pb = PathBuilder::new();
        pb.move_to(snap(pts[0].0), snap(pts[0].1));
        for p in &pts[1..] { pb.line_to(snap(p.0), snap(p.1)); }
        if let Some(path) = pb.finish() {
            let st = Stroke { width: wpx, ..Stroke::default() };
            let mut paint = c.paint();
            paint.anti_alias = false; // linee solo orizzontali e verticali: più nette senza
            self.px.stroke_path(&path, &paint, &st, Transform::identity(), None);
        }
    }
    pub fn circle(&mut self, x: f32, y: f32, r: f32, c: Color) {
        if let Some(p) = PathBuilder::from_circle(x * self.scale, y * self.scale, r * self.scale) {
            self.px.fill_path(&p, &c.paint(), FillRule::Winding, Transform::identity(), None);
        }
    }

    /// Testo con la linea di base a y. Restituisce la larghezza in pixel logici.
    pub fn text(&mut self, x: f32, y: f32, s: &str, size: f32, w: Weight, c: Color, spacing: f32) -> f32 {
        let sc = self.scale;
        let px_size = size * sc;
        if px_size < 3.0 { return self.fonts.measure(s, size, w, spacing); }
        let q = (px_size * 4.0).round() as u32; // dimensione arrotondata a 1/4 di pixel per la cache
        let px_size = q as f32 / 4.0;
        let wi = if w == Weight::Regular { 0u8 } else { 1 };
        if self.fonts.cache.len() > 6000 { self.fonts.cache.clear(); }
        let (pw, ph) = (self.px.width() as i32, self.px.height() as i32);
        let base = (y * sc).round() as i32;
        let mut pen = x * sc;
        let mut prev: Option<char> = None;
        for ch in s.chars() {
            let font = &self.fonts.fonts[wi as usize];
            if let Some(p) = prev { pen += font.horizontal_kern(p, ch, px_size).unwrap_or(0.0); }
            prev = Some(ch);
            let key = (wi, ch, q);
            if !self.fonts.cache.contains_key(&key) {
                let (m, bmp) = font.rasterize(ch, px_size);
                self.fonts.cache.insert(key, Glyph { m, bmp });
            }
            let g = &self.fonts.cache[&key];
            let gx = (pen + g.m.xmin as f32).round() as i32;
            let gy = base - g.m.ymin - g.m.height as i32;
            let data = self.px.data_mut();
            for row in 0..g.m.height as i32 {
                let py = gy + row;
                if py < 0 || py >= ph { continue; }
                for col in 0..g.m.width as i32 {
                    let pxx = gx + col;
                    if pxx < 0 || pxx >= pw { continue; }
                    let cov = g.bmp[(row * g.m.width as i32 + col) as usize] as u32;
                    if cov == 0 { continue; }
                    let a = cov * c.3 as u32 / 255;
                    let i = ((py * pw + pxx) * 4) as usize;
                    // byte nell'ordine B,G,R,X (vedi sopra)
                    for (k, comp) in [(0, c.2), (1, c.1), (2, c.0)] {
                        let d = data[i + k] as u32;
                        data[i + k] = ((comp as u32 * a + d * (255 - a) + 127) / 255) as u8;
                    }
                    data[i + 3] = 255;
                }
            }
            pen += g.m.advance_width + spacing * sc;
        }
        pen / sc - x
    }
    /// testo centrato verticalmente su cy
    pub fn text_mid(&mut self, x: f32, cy: f32, s: &str, size: f32, w: Weight, c: Color) -> f32 {
        let off = self.fonts.center_offset(size, w);
        self.text(x, cy + off, s, size, w, c, 0.0)
    }
    pub fn text_right(&mut self, right: f32, cy: f32, s: &str, size: f32, w: Weight, c: Color) -> f32 {
        let tw = self.fonts.measure(s, size, w, 0.0);
        self.text_mid(right - tw, cy, s, size, w, c);
        tw
    }
}
