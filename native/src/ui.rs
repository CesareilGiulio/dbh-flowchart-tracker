//! Interfaccia: stesso aspetto e stesso funzionamento della versione Electron
//! (renderer/index.html, style.css, app.js). Si ridisegna tutto a ogni evento e
//! durante il disegno si registrano le zone cliccabili.

use crate::data::{self, Chapter, Kind, Lang, Progress};
use crate::layout::{self, Layout, H as NH, W as NW};
use crate::paint::{Color, Painter, Weight};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const SIDE_W: f32 = 260.0;
const MAXCH: usize = 28;
const ZMIN: f64 = 0.3;
const ZMAX: f64 = 2.0;
const ZFIT: f64 = 0.85;
const ROW_H: f32 = 31.6;
const FOOT_H: f32 = 121.4;
const LIST_TOP: f32 = 130.4;
pub const SAVE_DELAY: Duration = Duration::from_millis(400);

pub struct Theme { bg: Color, panel: Color, ink: Color, dim: Color, line: Color, edge: Color, node: Color, on: Color, on_ink: Color }
const LIGHT: Theme = Theme { bg: Color::hex(0xf4f4f2), panel: Color::hex(0xfbfbfa), ink: Color::hex(0x1b1c1e), dim: Color::hex(0x85878b),
    line: Color::hex(0xd9d9d6), edge: Color::hex(0xb9bab6), node: Color::hex(0xffffff), on: Color::hex(0x1b1c1e), on_ink: Color::hex(0xf4f4f2) };
const DARK: Theme = Theme { bg: Color::hex(0x161718), panel: Color::hex(0x1c1d1f), ink: Color::hex(0xe7e7e4), dim: Color::hex(0x8a8c90),
    line: Color::hex(0x2f3033), edge: Color::hex(0x44464a), node: Color::hex(0x202124), on: Color::hex(0xe7e7e4), on_ink: Color::hex(0x161718) };

struct Txt { chapters_full: &'static str, all: &'static str, other: &'static str, export: &'static str, import: &'static str,
    hide_names: &'static str, show_names: &'static str, game_pct: &'static str, fit: &'static str, no_data: &'static str,
    hidden_name: &'static str, bad_backup: &'static str, path: &'static str, save_error: &'static str }
const IT: Txt = Txt { chapters_full: "capitoli completati", all: "Tutti", other: "Altro", export: "Esporta", import: "Importa",
    hide_names: "Nascondi nomi bloccati", show_names: "Mostra nomi bloccati", game_pct: "nel gioco", fit: "Adatta",
    no_data: "Diagramma non ancora disponibile.", hidden_name: "Bloccato", bad_backup: "Il file non è un backup valido di questo tracker.",
    path: "Percorso", save_error: "Impossibile salvare i progressi in" };
const EN: Txt = Txt { chapters_full: "chapters completed", all: "All", other: "Other", export: "Export", import: "Import",
    hide_names: "Hide locked names", show_names: "Show locked names", game_pct: "in game", fit: "Fit",
    no_data: "Flowchart not available yet.", hidden_name: "Locked", bad_backup: "This file is not a valid backup of this tracker.",
    path: "Path", save_error: "Could not save progress to" };

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action { Chapter(usize), Filter(Option<char>), Section(usize), Spoiler, Export, Import, Lang(Lang), Game, ZoomIn, ZoomOut, Fit }

#[derive(Clone, Copy)]
pub struct View { pub x: f64, pub y: f64, pub k: f64 }

pub struct App {
    pub data: Vec<Option<Chapter>>,
    pub prog: Progress,
    pub cur: usize,
    pub sec: usize,
    pub filter: Option<char>,
    pub view: View,
    pub list_scroll: f32,
    pub dark: bool,
    pub size: (f32, f32),
    pub mouse: Option<(f32, f32)>,
    pub hits: Vec<([f32; 4], Action)>,
    pub hover_node: Option<usize>,
    pub press_node: Option<usize>,
    pub drag: Option<(f32, f32, f64, f64, bool)>,
    pub game_focus: bool,
    pub game_text: String,
    pub save_at: Option<Instant>,
    pub need_fit: bool,
    pub message: Option<String>,
    layouts: HashMap<(usize, usize), Layout>,
    canvas: [f32; 4],
}

impl App {
    pub fn new() -> Self {
        let data = data::load_chapters();
        let mut prog = Progress::new(&data);
        if let Some(v) = data::read_json(&data::save_path()) { let _ = prog.apply_json(&data, &v); }
        let mut a = App { data, prog, cur: 0, sec: 0, filter: None, view: View { x: 0.0, y: 0.0, k: 1.0 }, list_scroll: 0.0, dark: false,
            size: (1440.0, 900.0), mouse: None, hits: vec![], hover_node: None, press_node: None, drag: None, game_focus: false,
            game_text: String::new(), save_at: None, need_fit: true, message: None, layouts: HashMap::new(), canvas: [0.0; 4] };
        a.sync_game_text();
        a
    }
    fn t(&self) -> &'static Txt { if self.prog.lang == Lang::It { &IT } else { &EN } }
    fn th(&self) -> &'static Theme { if self.dark { &DARK } else { &LIGHT } }
    fn chapter(&self) -> Option<&Chapter> { self.data[self.cur].as_ref() }
    fn label<'b>(&self, n: &'b data::Node) -> &'b str { if self.prog.lang == Lang::En { if n.en.is_empty() { &n.title } else { &n.en } } else if n.title.is_empty() { &n.en } else { &n.title } }
    fn other_label<'b>(&self, n: &'b data::Node) -> &'b str { let o = if self.prog.lang == Lang::En { &n.title } else { &n.en }; if o.is_empty() || o == self.label(n) { "" } else { o } }
    fn note<'b>(&self, n: &'b data::Node) -> &'b str { if self.prog.lang == Lang::En && !n.note_en.is_empty() { &n.note_en } else { &n.note } }
    fn unlocked(&self, si: usize, ni: usize) -> bool { self.prog.chapters[self.cur].unlocked.get(si).map_or(false, |s| s.contains(&ni)) }
    fn hidden(&self, ni: usize) -> bool { self.prog.spoiler && !self.unlocked(self.sec, ni) }
    pub fn schedule_save(&mut self) { self.save_at = Some(Instant::now() + SAVE_DELAY); }
    pub fn save_now(&mut self) {
        self.save_at = None;
        let path = data::save_path();
        if let Err(e) = data::write_json(&path, &self.prog.to_json(&self.data)) {
            self.message = Some(format!("{} {}: {e}", self.t().save_error, path.display()));
        }
    }
    fn sync_game_text(&mut self) {
        self.game_text = self.prog.chapters[self.cur].game.map_or(String::new(), |g| format!("{}", g.round() as i64));
    }
    fn layout(&mut self) -> Option<&Layout> {
        let ch = self.data[self.cur].as_ref()?;
        let s = ch.sections.get(self.sec)?;
        if s.nodes.is_empty() { return None; }
        let key = (self.cur, self.sec);
        if !self.layouts.contains_key(&key) { let l = layout::layout(&s.nodes); self.layouts.insert(key, l); }
        self.layouts.get(&key)
    }

    /* ---------- vista ---------- */
    pub fn fit(&mut self) {
        let [_, _, cw, ch] = self.canvas;
        let Some(b) = self.layout().and_then(|l| l.bounds.as_ref()).map(|b| (b.x0, b.x1, b.y0, b.y1)) else { return };
        let (cw, ch, m) = (cw as f64, ch as f64, 40.0);
        let (bw, bh) = (b.1 - b.0, b.3 - b.2);
        let k = ZFIT.max(1f64.min((cw - 2.0 * m) / bw).min((ch - 2.0 * m) / bh));
        let x = if bw * k <= cw - 2.0 * m { (cw - bw * k) / 2.0 - b.0 * k } else { m - b.0 * k };
        let y = if bh * k <= ch - 2.0 * m { (ch - bh * k) / 2.0 - b.2 * k } else { m - b.2 * k };
        self.view = View { x, y, k };
        self.need_fit = false;
    }
    pub fn zoom_at(&mut self, f: f64, mx: f64, my: f64) {
        let k = (self.view.k * f).clamp(ZMIN, ZMAX);
        self.view.x = mx - (mx - self.view.x) * k / self.view.k;
        self.view.y = my - (my - self.view.y) * k / self.view.k;
        self.view.k = k;
    }
    fn in_canvas(&self, (x, y): (f32, f32)) -> bool { let c = self.canvas; x >= c[0] && y >= c[1] && x < c[0] + c[2] && y < c[1] + c[3] }
    fn in_list(&self, (x, y): (f32, f32)) -> bool { x < SIDE_W && y >= LIST_TOP && y < self.size.1 - FOOT_H }
    /// nodo sotto il puntatore
    pub fn node_at(&mut self, p: (f32, f32)) -> Option<usize> {
        if !self.in_canvas(p) || self.hit_at(p).is_some() { return None; }
        let c = self.canvas;
        let (wx, wy) = ((p.0 - c[0]) as f64, (p.1 - c[1]) as f64);
        let v = self.view;
        let (x, y) = ((wx - v.x) / v.k, (wy - v.y) / v.k);
        let l = self.layout()?;
        l.pos.iter().position(|&(nx, ny)| (x - nx).abs() <= NW / 2.0 && (y - ny).abs() <= NH / 2.0)
    }
    pub fn hit_at(&self, (x, y): (f32, f32)) -> Option<Action> {
        self.hits.iter().rev().find(|(r, _)| x >= r[0] && y >= r[1] && x < r[0] + r[2] && y < r[1] + r[3]).map(|h| h.1)
    }

    /* ---------- azioni ---------- */
    pub fn toggle(&mut self, ni: usize) {
        let (cur, sec) = (self.cur, self.sec);
        let set = &mut self.prog.chapters[cur].unlocked[sec];
        if !set.remove(&ni) { set.insert(ni); }
        self.schedule_save();
    }
    fn go_chapter(&mut self, i: usize) { self.cur = i; self.sec = 0; self.game_focus = false; self.sync_game_text(); self.hover_node = None; self.need_fit = true; }
    pub fn act(&mut self, a: Action) {
        if a != Action::Game { self.commit_game(); }
        match a {
            Action::Chapter(i) => self.go_chapter(i),
            Action::Filter(f) => { self.filter = f; self.list_scroll = 0.0; }
            Action::Section(i) => { self.sec = i; self.hover_node = None; self.need_fit = true; }
            Action::Spoiler => { self.prog.spoiler = !self.prog.spoiler; self.schedule_save(); }
            Action::Lang(l) => { self.prog.lang = l; self.schedule_save(); }
            Action::Game => { self.game_focus = true; }
            Action::ZoomIn | Action::ZoomOut => {
                let f = if a == Action::ZoomIn { 1.2 } else { 1.0 / 1.2 };
                self.zoom_at(f, self.canvas[2] as f64 / 2.0, self.canvas[3] as f64 / 2.0);
            }
            Action::Fit => self.fit(),
            Action::Export => self.export(),
            Action::Import => self.import(),
        }
    }
    pub fn commit_game(&mut self) {
        if !self.game_focus { return; }
        self.game_focus = false;
        let v: Option<f64> = self.game_text.trim().parse::<f64>().ok().map(|g| g.clamp(0.0, 100.0));
        if self.prog.chapters[self.cur].game != v { self.prog.chapters[self.cur].game = v; self.schedule_save(); }
        self.sync_game_text();
    }
    pub fn game_key(&mut self, s: &str) {
        for c in s.chars() { if c.is_ascii_digit() && self.game_text.len() < 3 { self.game_text.push(c); } }
    }
    fn export(&mut self) {
        let v = self.prog.to_json(&self.data);
        if let Some(path) = crate::dialogs::save_file() {
            if let Err(e) = data::write_json(&path, &v) { self.message = Some(format!("{} {}: {e}", self.t().save_error, path.display())); }
        }
    }
    fn import(&mut self) {
        let Some(path) = crate::dialogs::open_file() else { return };
        let ok = data::read_json(&path).map(|v| {
            let mut p = Progress::new(&self.data);
            p.lang = self.prog.lang;
            p.apply_json(&self.data, &v).map(|_| p)
        });
        match ok {
            Some(Ok(p)) => { self.prog = p; self.sync_game_text(); self.save_now(); self.need_fit = true; }
            _ => crate::dialogs::alert(self.t().bad_backup),
        }
    }
    pub fn wheel(&mut self, dx: f32, dy: f32, ctrl: bool, shift: bool) {
        let Some(m) = self.mouse else { return };
        if self.in_list(m) {
            self.list_scroll = (self.list_scroll + dy).max(0.0);
        } else if self.in_canvas(m) {
            if ctrl {
                let c = self.canvas;
                self.zoom_at((-dy as f64 * 0.002).exp(), (m.0 - c[0]) as f64, (m.1 - c[1]) as f64);
            } else if shift && dx == 0.0 { self.view.x -= dy as f64; } else { self.view.x -= dx as f64; self.view.y -= dy as f64; }
        }
    }

    /* ---------- disegno ---------- */
    fn tab_groups(&self) -> (Vec<(Option<char>, Vec<usize>)>, Option<char>) {
        let Some(ch) = self.chapter() else { return (vec![], None) };
        let mut groups: Vec<(Option<char>, Vec<usize>)> = vec![];
        for (i, s) in ch.sections.iter().enumerate() {
            match groups.iter_mut().find(|g| g.0 == s.ch) { Some(g) => g.1.push(i), None => groups.push((s.ch, vec![i])) }
        }
        let ord = |c: Option<char>| match c { Some('K') => 0, Some('C') => 1, Some('M') => 2, _ => 3 };
        groups.sort_by_key(|g| ord(g.0));
        (groups, ch.sections.get(self.sec).and_then(|s| s.ch))
    }
    fn header_h(&self) -> f32 {
        let n = self.chapter().map_or(0, |c| c.sections.len());
        if n < 2 { return 94.6; }
        let (groups, active) = self.tab_groups();
        let sub = groups.iter().find(|g| g.0 == active).map_or(false, |g| g.1.len() > 1);
        if sub { 120.6 } else { 94.6 }
    }

    pub fn draw(&mut self, p: &mut Painter) {
        let th = self.th();
        let (w, h) = self.size;
        self.hits.clear();
        p.clear(th.bg);
        let hh = self.header_h();
        self.canvas = [SIDE_W, hh, (w - SIDE_W).max(1.0), (h - hh).max(1.0)];
        if self.need_fit { self.fit(); }
        self.draw_canvas(p);
        self.draw_header(p);
        self.draw_side(p);
        self.draw_overlays(p);
    }

    fn draw_canvas(&mut self, p: &mut Painter) {
        let th = self.th();
        let c = self.canvas;
        let has = self.layout().is_some();
        if !has {
            let t = self.t().no_data;
            let tw = p.fonts.measure(t, 14.0, Weight::Regular, 0.0);
            p.text_mid(c[0] + (c[2] - tw) / 2.0, c[1] + c[3] / 2.0, t, 14.0, Weight::Regular, th.dim);
            return;
        }
        let v = self.view;
        let (ox, oy) = (c[0] as f64 + v.x, c[1] as f64 + v.y);
        let sx = |x: f64| (ox + x * v.k) as f32;
        let sy = |y: f64| (oy + y * v.k) as f32;
        let k = v.k as f32;
        let (cur, sec) = (self.cur, self.sec);
        let key = (cur, sec);
        let l = self.layouts.get(&key).unwrap();
        let ch = self.data[cur].as_ref().unwrap();
        let nodes = &ch.sections[sec].nodes;
        let un = &self.prog.chapters[cur].unlocked[sec];
        let visible = |x0: f32, y0: f32, x1: f32, y1: f32| x1 >= c[0] && x0 <= c[0] + c[2] && y1 >= c[1] && y0 <= c[1] + c[3];
        // prima le linee spente, poi quelle accese
        for pass in 0..2 {
            for e in &l.edges {
                let lit = un.contains(&e.p) && un.contains(&e.c);
                if lit != (pass == 1) { continue; }
                let (col, wid) = if lit { (th.on, 1.4) } else { (th.edge, 1.0) };
                for line in &e.lines {
                    let pts: Vec<(f32, f32)> = line.iter().map(|&(x, y)| (sx(x), sy(y))).collect();
                    let (x0, x1) = pts.iter().fold((f32::MAX, f32::MIN), |a, p| (a.0.min(p.0), a.1.max(p.0)));
                    let (y0, y1) = pts.iter().fold((f32::MAX, f32::MIN), |a, p| (a.0.min(p.1), a.1.max(p.1)));
                    if visible(x0, y0, x1, y1) { p.polyline(&pts, wid * k, col); }
                }
                for &(x, y) in &e.dots { p.circle(sx(x), sy(y), 2.2 * k, col); }
            }
        }
        let t = self.t();
        for (i, n) in nodes.iter().enumerate() {
            let (nx, ny) = l.pos[i];
            let (x0, y0) = (sx(nx - NW / 2.0), sy(ny - NH / 2.0));
            let (bw, bh) = (NW as f32 * k, NH as f32 * k);
            if !visible(x0, y0, x0 + bw, y0 + bh) { continue; }
            let on = un.contains(&i);
            let hide = self.prog.spoiler && !on;
            let hov = self.hover_node == Some(i);
            let (fill, stroke) = if on { (th.on, th.on) } else { (th.node, if hov { th.ink } else { th.line }) };
            let rx = if n.kind == Kind::Checkpoint { bh / 2.0 } else { 2.0 * k };
            p.rrect(x0, y0, bw, bh, rx, Some(fill), Some((stroke, 1.0)));
            if n.kind == Kind::Ending {
                p.rrect(x0 + 3.0 * k, y0 + 3.0 * k, bw - 6.0 * k, bh - 6.0 * k, k, None, Some((if on { th.on_ink } else { th.line }, 1.0)));
            }
            let (size, wt, sp, upper) = match n.kind {
                Kind::Objective => (11.5, Weight::Semibold, 0.345, true),
                Kind::Ending => (12.5, Weight::Semibold, 0.0, false),
                _ => (12.5, Weight::Regular, 0.0, false),
            };
            let lab = if self.prog.lang == Lang::En { if n.en.is_empty() { &n.title } else { &n.en } } else if n.title.is_empty() { &n.en } else { &n.title };
            let lines: Vec<String> = if hide { vec![t.hidden_name.to_string()] } else { wrap(lab) }
                .into_iter().map(|s| if upper { s.to_uppercase() } else { s }).collect();
            let col = if on { th.on_ink } else if hide { th.dim } else { th.ink };
            let cx = sx(nx);
            for (li, s) in lines.iter().enumerate() {
                let by = sy(ny + (li as f64 - (lines.len() as f64 - 1.0) / 2.0) * 15.0 + 4.5);
                let tw = p.fonts.measure(s, size * k, wt, sp * k);
                p.text(cx - tw / 2.0, by, s, size * k, wt, col, sp * k);
            }
        }
    }

    fn draw_header(&mut self, p: &mut Painter) {
        let th = self.th();
        let t = self.t();
        let (w, _) = self.size;
        let hh = self.canvas[1];
        p.fill(SIDE_W, 0.0, w - SIDE_W, hh, th.bg);
        p.hline(SIDE_W, hh - 1.0, w - SIDE_W, th.line);
        let x0 = SIDE_W + 28.0;
        let (_, title, _) = data::CHAPTERS[self.cur];
        let right = w - 28.0;
        // a destra: percentuale e campo "nel gioco"
        let (_, _, calc, _) = self.prog.stats(&self.data, self.cur);
        p.text_right(right, 38.85, &format!("{calc}%"), 26.0, Weight::Semibold, th.ink);
        let lw = p.text_right(right, 69.7, t.game_pct, 12.5, Weight::Regular, th.dim);
        let (ix, iw, ih) = (right - lw - 5.0 - 52.0, 52.0, 23.0);
        let iy = 69.7 - ih / 2.0;
        let focus = self.game_focus;
        p.rrect(ix, iy, iw, ih, 3.0, Some(th.panel), Some((if focus { th.ink } else { th.line }, 1.0)));
        let gt = if focus || !self.game_text.is_empty() { self.game_text.clone() } else { "—".into() };
        let gw = p.text_right(ix + iw - 5.0, 69.7, &gt, 12.5, Weight::Regular, if focus || !self.game_text.is_empty() { th.ink } else { th.dim });
        if focus { let _ = gw; p.vline(ix + iw - 4.0, iy + 5.0, ih - 10.0, th.ink); }
        self.hits.push(([ix, iy, iw, ih], Action::Game));
        // titolo
        let tmax = right - lw - 90.0 - x0;
        let title = p.fonts.ellipsize(title, 22.0, Weight::Semibold, tmax);
        p.text_mid(x0, 36.0, &title, 22.0, Weight::Semibold, th.ink);
        // schede dei personaggi e dei percorsi
        let n = self.chapter().map_or(0, |c| c.sections.len());
        if n < 2 { return; }
        let (groups, active) = self.tab_groups();
        let mouse = self.mouse;
        let mut x = x0;
        if groups.len() > 1 {
            for g in &groups {
                let name = match g.0 { Some('C') => "Connor", Some('K') => "Kara", Some('M') => "Markus", _ => t.other };
                let tw = p.fonts.measure(name, 13.5, Weight::Regular, 0.0);
                let on = g.0 == active;
                let r = [x, 57.9, tw, 23.6];
                let hov = inside(mouse, r);
                p.text_mid(x, 69.7, name, 13.5, Weight::Regular, if on || hov { th.ink } else { th.dim });
                if on { p.hline(x, 80.5, tw, th.ink); }
                self.hits.push((r, Action::Section(g.1[0])));
                x += tw + 18.0;
            }
        }
        if let Some(g) = groups.iter().find(|g| g.0 == active) {
            if g.1.len() > 1 {
                let ch = self.chapter().unwrap();
                let mut x = x0;
                let items: Vec<(String, String, usize)> = g.1.iter().enumerate().map(|(j, &si)| {
                    let s = &ch.sections[si];
                    let u = self.prog.chapters[self.cur].unlocked[si].len();
                    (variant_label(&s.name).unwrap_or_else(|| format!("{} {}", t.path, j + 1)), format!("{u}/{}", s.nodes.len()), si)
                }).collect();
                for (name, cnt, si) in items {
                    let tw = p.fonts.measure(&name, 12.5, Weight::Regular, 0.0);
                    let cw = p.fonts.measure(&cnt, 10.8, Weight::Regular, 0.0);
                    let on = si == self.sec;
                    let r = [x, 85.5, tw + 6.0 + cw, 22.1];
                    let col = if on || inside(mouse, r) { th.ink } else { th.dim };
                    p.text_mid(x, 96.5, &name, 12.5, Weight::Regular, col);
                    p.text_mid(x + tw + 6.0, 96.9, &cnt, 10.8, Weight::Regular, col);
                    if on { p.hline(x, 107.0, tw + 6.0 + cw, th.ink); }
                    self.hits.push((r, Action::Section(si)));
                    x += tw + 6.0 + cw + 18.0;
                }
            }
        }
    }

    fn draw_side(&mut self, p: &mut Painter) {
        let th = self.th();
        let t = self.t();
        let (_, h) = self.size;
        let mouse = self.mouse;
        p.fill(0.0, 0.0, SIDE_W, h, th.panel);
        // elenco dei capitoli
        let list_h = h - FOOT_H - LIST_TOP;
        let items: Vec<usize> = (0..32).filter(|&i| self.filter.map_or(true, |f| data::CHAPTERS[i].2.contains(f))).collect();
        let content = 12.0 + items.len() as f32 * ROW_H;
        let max_scroll = (content - list_h).max(0.0);
        self.list_scroll = self.list_scroll.min(max_scroll);
        let top = LIST_TOP + 6.0 - self.list_scroll;
        for (j, &i) in items.iter().enumerate() {
            let y = top + j as f32 * ROW_H;
            if y + ROW_H < LIST_TOP || y > h - FOOT_H { continue; }
            let r = [0.0, y, SIDE_W, ROW_H];
            let sel = i == self.cur;
            let visible_hov = inside(mouse, r) && mouse.map_or(false, |m| m.1 >= LIST_TOP && m.1 < h - FOOT_H);
            if sel { p.fill(0.0, y, SIDE_W, ROW_H, th.ink.over(th.panel, 0.08)); } else if visible_hov { p.fill(0.0, y, SIDE_W, ROW_H, th.ink.over(th.panel, 0.05)); }
            let (n, title, _) = data::CHAPTERS[i];
            let cy = y + ROW_H / 2.0;
            let (_, _, _, shown) = self.prog.stats(&self.data, i);
            let has = self.data[i].is_some();
            p.text_mid(20.0, cy, &n.to_string(), 13.5, Weight::Regular, th.dim);
            let full = shown >= 100;
            let pw = p.text_right(SIDE_W - 20.0, cy, &format!("{shown}%"), 13.5, if full { Weight::Semibold } else { Weight::Regular }, if full { th.ink } else { th.dim });
            let tt = p.fonts.ellipsize(title, 13.5, Weight::Regular, SIDE_W - 20.0 - pw - 8.0 - 52.0);
            p.text_mid(52.0, cy, &tt, 13.5, Weight::Regular, if has { th.ink } else { th.dim });
            if y + ROW_H > LIST_TOP && y < h - FOOT_H {
                let r = [0.0, y.max(LIST_TOP), SIDE_W, (y + ROW_H).min(h - FOOT_H) - y.max(LIST_TOP)];
                self.hits.push((r, Action::Chapter(i)));
            }
        }
        if max_scroll > 0.0 {
            let bar_h = (list_h * list_h / content).max(24.0);
            let by = LIST_TOP + (list_h - bar_h) * (self.list_scroll / max_scroll);
            p.rrect(SIDE_W - 6.0, by + 2.0, 3.0, bar_h - 4.0, 1.5, Some(th.dim.over(th.panel, 0.45)), None);
        }
        // intestazione: capitoli completati e filtri
        p.fill(0.0, 0.0, SIDE_W, LIST_TOP, th.panel);
        let full = (0..32).filter(|&i| self.prog.stats(&self.data, i).3 >= 100).count();
        p.text_mid(20.0, 40.85, &format!("{full} / 32"), 26.0, Weight::Semibold, th.ink);
        p.text_mid(20.0, 69.0, t.chapters_full, 13.0, Weight::Regular, th.dim);
        let mut x = 20.0;
        for (f, name) in [(None, t.all), (Some('C'), "Connor"), (Some('K'), "Kara"), (Some('M'), "Markus")] {
            let tw = p.fonts.measure(name, 13.0, Weight::Regular, 0.0);
            let r = [x, 92.6, tw, 26.9];
            let on = self.filter == f;
            p.text_mid(x, 106.0, name, 13.0, Weight::Regular, if on || inside(mouse, r) { th.ink } else { th.dim });
            if on { p.hline(x, 118.5, tw, th.ink); }
            self.hits.push((r, Action::Filter(f)));
            x += tw + 16.0;
        }
        p.hline(0.0, LIST_TOP - 1.0, SIDE_W, th.line);
        // piede: collegamenti di testo e lingua
        let fy = h - FOOT_H;
        p.fill(0.0, fy, SIDE_W, FOOT_H, th.panel);
        p.hline(0.0, fy, SIDE_W, th.line);
        let spoil = if self.prog.spoiler { t.show_names } else { t.hide_names };
        let mut cy = fy + 12.0 + 9.4;
        for (s, a) in [(spoil, Action::Spoiler), (t.export, Action::Export), (t.import, Action::Import)] {
            let tw = p.fonts.measure(s, 13.0, Weight::Regular, 0.0);
            let r = [20.0, cy - 9.4, tw, 18.85];
            let hov = inside(mouse, r);
            p.text_mid(20.0, cy, s, 13.0, Weight::Regular, if hov { th.ink } else { th.dim });
            if hov { p.hline(20.0, cy + 7.0, tw, th.ink); }
            self.hits.push((r, a));
            cy += 22.85;
        }
        cy += 10.0 - 22.85 + 18.85;
        let mut x = 20.0;
        for (s, l) in [("Italiano", Lang::It), (" · ", Lang::It), ("English", Lang::En)] {
            let tw = p.fonts.measure(s, 13.0, Weight::Regular, 0.0);
            if s == " · " { p.text_mid(x, cy, s, 13.0, Weight::Regular, th.dim); x += tw; continue; }
            let r = [x, cy - 9.4, tw, 18.85];
            p.text_mid(x, cy, s, 13.0, Weight::Regular, if self.prog.lang == l || inside(mouse, r) { th.ink } else { th.dim });
            self.hits.push((r, Action::Lang(l)));
            x += tw;
        }
        p.vline(SIDE_W - 1.0, 0.0, h, th.line);
    }

    fn draw_overlays(&mut self, p: &mut Painter) {
        let th = self.th();
        let t = self.t();
        let c = self.canvas;
        let mouse = self.mouse;
        // pulsanti dello zoom in basso a destra
        let labels = [("−", Action::ZoomOut), (t.fit, Action::Fit), ("+", Action::ZoomIn)];
        let widths: Vec<f32> = labels.iter().map(|(s, _)| p.fonts.measure(s, 13.0, Weight::Regular, 0.0) + 24.0).collect();
        let tw: f32 = widths.iter().sum();
        let (bh, right, bottom) = (28.85, c[0] + c[2] - 20.0, c[1] + c[3] - 16.0);
        let (mut x, y) = (right - tw, bottom - bh);
        p.rrect(x, y, tw, bh, 4.0, Some(th.panel), None);
        for (i, (s, a)) in labels.iter().enumerate() {
            let r = [x, y, widths[i], bh];
            if inside(mouse, r) { p.fill(x + 1.0, y + 1.0, widths[i] - 2.0, bh - 2.0, th.ink.over(th.panel, 0.06)); }
            p.text_mid(x + 12.0, y + bh / 2.0, s, 13.0, Weight::Regular, th.ink);
            if i > 0 { p.vline(x, y, bh, th.line); }
            self.hits.push((r, *a));
            x += widths[i];
        }
        p.rrect(right - tw, y, tw, bh, 4.0, None, Some((th.line, 1.0)));
        // pannello informazioni del nodo sotto il puntatore
        let info: Option<(String, String)> = self.hover_node.and_then(|ni| {
            let n = self.chapter()?.sections.get(self.sec)?.nodes.get(ni)?;
            Some(if self.hidden(ni) { (t.hidden_name.to_string(), String::new()) } else {
                let o = self.other_label(n);
                (if o.is_empty() { self.label(n).to_string() } else { format!("{} · {}", self.label(n), o) }, self.note(n).to_string())
            })
        });
        let msg = self.message.clone();
        let boxes: Vec<(String, String)> = info.into_iter().chain(msg.map(|m| (m, String::new()))).collect();
        let mut bottom_y = c[1] + c[3] - 16.0;
        for (title, note) in boxes.iter().rev() {
            let max_w = (560.0f32).min(c[2] - 220.0 - 40.0).max(160.0) - 24.0;
            let tl = p.fonts.wrap(title, 13.0, Weight::Semibold, max_w);
            let nl = if note.is_empty() { vec![] } else { p.fonts.wrap(note, 13.0, Weight::Regular, max_w) };
            let lw = tl.iter().map(|s| p.fonts.measure(s, 13.0, Weight::Semibold, 0.0))
                .chain(nl.iter().map(|s| p.fonts.measure(s, 13.0, Weight::Regular, 0.0))).fold(0.0, f32::max);
            let lh = 18.85;
            let bh = 16.0 + lh * (tl.len() + nl.len()) as f32 + if nl.is_empty() { 0.0 } else { 2.0 };
            let (bx, by) = (c[0] + 20.0, bottom_y - bh);
            p.rrect(bx, by, lw + 24.0, bh, 4.0, Some(th.panel), Some((th.line, 1.0)));
            let mut cy = by + 8.0 + lh / 2.0;
            for s in &tl { p.text_mid(bx + 12.0, cy, s, 13.0, Weight::Semibold, th.ink); cy += lh; }
            cy += 2.0;
            for s in &nl { p.text_mid(bx + 12.0, cy, s, 13.0, Weight::Regular, th.dim); cy += lh; }
            bottom_y = by - 8.0;
        }
    }
}

fn inside(m: Option<(f32, f32)>, r: [f32; 4]) -> bool { m.map_or(false, |(x, y)| x >= r[0] && y >= r[1] && x < r[0] + r[2] && y < r[1] + r[3]) }

/// Stesse regole di app.js: righe di al massimo 28 caratteri, al massimo 2 righe
pub fn wrap(t: &str) -> Vec<String> {
    let mut lines = vec![String::new()];
    for w in t.split_whitespace() {
        let l = lines.last().unwrap().clone();
        let joined = format!("{l} {w}").trim().to_string();
        if joined.chars().count() > MAXCH && !l.is_empty() { lines.push(w.to_string()); } else { *lines.last_mut().unwrap() = joined; }
    }
    let cut = |s: &str| -> String { let mut v: String = s.chars().take(MAXCH - 1).collect(); v = v.trim().to_string(); v + "…" };
    if lines.len() > 2 { lines.truncate(2); lines[1] = cut(&lines[1]); }
    else if let Some(last) = lines.last_mut() { if last.chars().count() > MAXCH { *last = cut(last); } }
    lines
}

/// Nome breve di un percorso: "Battle for Detroit - Kara Leaving Detroit" → "Leaving Detroit"
pub fn variant_label(name: &str) -> Option<String> {
    let base = match name.find(" - ") { Some(i) => &name[i + 3..], None => name };
    let words: Vec<&str> = base.split_whitespace().filter(|w| {
        let lw = w.to_lowercase();
        let core = lw.trim_end_matches("'s").trim_end_matches("’s");
        !["kara", "connor", "markus"].contains(&core)
    }).collect();
    let s = words.join(" ");
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str())
}
