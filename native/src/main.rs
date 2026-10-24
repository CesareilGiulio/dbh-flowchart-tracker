//! Flowchart Tracker per Detroit: Become Human, versione nativa (senza browser).
#![cfg_attr(windows, windows_subsystem = "windows")]

mod data;
mod dialogs;
mod layout;
mod paint;
mod ui;

use paint::{Fonts, Painter};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;
use tiny_skia::PixmapMut;
use ui::App;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{CursorIcon, Icon, Theme, Window, WindowId};

type Surface = softbuffer::Surface<Rc<Window>, Rc<Window>>;

struct Runner {
    app: App,
    fonts: Fonts,
    win: Option<Rc<Window>>,
    surface: Option<Surface>,
    mods: ModifiersState,
    scale: f64,
    hover_key: (Option<usize>, Option<ui::Action>),
    /// FT_DEBUG: stampa eventi e tempi di disegno (per i test)
    debug: bool,
}

impl Runner {
    fn redraw(&self) { if let Some(w) = &self.win { w.request_redraw(); } }
    fn render(&mut self) {
        let (Some(win), Some(surface)) = (self.win.as_ref(), self.surface.as_mut()) else { return };
        let size = win.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return };
        if surface.resize(w, h).is_err() { return; }
        let Ok(mut buf) = surface.buffer_mut() else { return };
        // il buffer della finestra viene usato direttamente come superficie di disegno (vedi paint.rs)
        let bytes = unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len() * 4) };
        let Some(px) = PixmapMut::from_bytes(bytes, w.get(), h.get()) else { return };
        let scale = self.scale as f32;
        self.app.size = (w.get() as f32 / scale, h.get() as f32 / scale);
        let mut p = Painter { px, scale, fonts: &mut self.fonts };
        let t0 = Instant::now();
        self.app.draw(&mut p);
        if self.debug { eprintln!("disegno {:.1} ms", t0.elapsed().as_secs_f64() * 1000.0); }
        let _ = buf.present();
    }
    fn update_hover(&mut self) {
        let Some(m) = self.app.mouse else { return };
        let hit = self.app.hit_at(m);
        let node = if self.app.drag.is_some() { None } else { self.app.node_at(m) };
        self.app.hover_node = node;
        let key = (node, hit);
        if key != self.hover_key { self.hover_key = key; self.redraw(); }
        if let Some(w) = &self.win {
            w.set_cursor(match (hit, node, self.app.drag.is_some()) {
                (Some(ui::Action::Game), _, _) => CursorIcon::Text,
                (Some(_), _, _) | (_, Some(_), _) => CursorIcon::Pointer,
                (_, _, true) => CursorIcon::Grabbing,
                _ => CursorIcon::Default,
            });
        }
    }
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.win.is_some() { return; }
        // finestra di 1440×900, ridotta se lo schermo è più piccolo
        let (mut w, mut h) = (1440.0, 900.0);
        if let Some(m) = el.primary_monitor() {
            let s = m.size().to_logical::<f64>(m.scale_factor());
            w = f64::min(w, s.width * 0.92);
            h = f64::min(h, s.height * 0.88);
        }
        let icon = Icon::from_rgba(include_bytes!("../assets/icon64.rgba").to_vec(), 64, 64).ok();
        let attrs = Window::default_attributes().with_title("Flowchart Tracker").with_inner_size(LogicalSize::new(w, h))
            .with_min_inner_size(LogicalSize::new(900.0, 600.0)).with_window_icon(icon);
        let win = Rc::new(el.create_window(attrs).expect("finestra"));
        self.scale = win.scale_factor();
        self.app.dark = win.theme() == Some(Theme::Dark) || std::env::var_os("FT_DARK").is_some();
        let ctx = softbuffer::Context::new(win.clone()).expect("softbuffer");
        self.surface = Some(softbuffer::Surface::new(&ctx, win.clone()).expect("superficie"));
        self.win = Some(win);
        self.redraw();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.debug && matches!(event, WindowEvent::KeyboardInput { .. } | WindowEvent::MouseWheel { .. }) { eprintln!("{event:?}"); }
        match event {
            WindowEvent::CloseRequested => {
                self.app.commit_game();
                if self.app.save_at.is_some() { self.app.save_now(); }
                el.exit();
            }
            WindowEvent::RedrawRequested => self.render(),
            WindowEvent::Resized(_) => { self.app.need_fit = true; self.redraw(); }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => { self.scale = scale_factor; self.redraw(); }
            WindowEvent::ThemeChanged(t) => { self.app.dark = t == Theme::Dark; self.redraw(); }
            WindowEvent::ModifiersChanged(m) => self.mods = m.state(),
            WindowEvent::CursorLeft { .. } => { self.app.mouse = None; self.app.hover_node = None; self.hover_key = (None, None); self.redraw(); }
            WindowEvent::CursorMoved { position, .. } => {
                let m = ((position.x / self.scale) as f32, (position.y / self.scale) as f32);
                self.app.mouse = Some(m);
                if let Some((x, y, vx, vy, _)) = self.app.drag {
                    self.app.view.x = vx + (m.0 - x) as f64;
                    self.app.view.y = vy + (m.1 - y) as f64;
                    self.app.drag = Some((x, y, vx, vy, true));
                    self.redraw();
                }
                self.update_hover();
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                let Some(m) = self.app.mouse else { return };
                match state {
                    ElementState::Pressed => {
                        self.app.message = None;
                        if let Some(a) = self.app.hit_at(m) { self.app.act(a); }
                        else if let Some(n) = self.app.node_at(m) { self.app.commit_game(); self.app.press_node = Some(n); }
                        else {
                            self.app.commit_game();
                            let c = self.app.view;
                            if m.0 >= 260.0 { self.app.drag = Some((m.0, m.1, c.x, c.y, false)); }
                        }
                    }
                    ElementState::Released => {
                        if let Some(n) = self.app.press_node.take() { if self.app.node_at(m) == Some(n) { self.app.toggle(n); } }
                        self.app.drag = None;
                    }
                }
                self.update_hover();
                self.redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // stesse unità di un browser: circa 100 pixel per scatto della rotellina
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (-x * 100.0, -y * 100.0),
                    MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => ((-x / self.scale) as f32, (-y / self.scale) as f32),
                };
                self.app.wheel(dx, dy, self.mods.control_key(), self.mods.shift_key());
                self.update_hover();
                self.redraw();
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed && self.app.game_focus => {
                match &event.logical_key {
                    Key::Named(NamedKey::Backspace) => { self.app.game_text.pop(); }
                    Key::Named(NamedKey::Delete) => self.app.game_text.clear(),
                    Key::Named(NamedKey::Enter | NamedKey::Escape | NamedKey::Tab) => self.app.commit_game(),
                    Key::Character(s) => self.app.game_key(s),
                    _ => {}
                }
                self.redraw();
            }
            WindowEvent::Focused(false) => { self.app.commit_game(); self.redraw(); }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        // salvataggio raggruppato: parte 400 ms dopo l'ultima modifica
        match self.app.save_at {
            Some(t) if Instant::now() >= t => { self.app.save_now(); el.set_control_flow(ControlFlow::Wait); self.redraw(); }
            Some(t) => el.set_control_flow(ControlFlow::WaitUntil(t)),
            None => el.set_control_flow(ControlFlow::Wait),
        }
    }
}

/// Per i test: stampa le posizioni calcolate per tutti i percorsi, da confrontare con renderer/layout.js
fn dump_layout() {
    let data = data::load_chapters();
    let mut out = serde_json::Map::new();
    for ch in data.iter().flatten() {
        for (si, s) in ch.sections.iter().enumerate() {
            let l = layout::layout(&s.nodes);
            let pos: Vec<[f64; 2]> = l.pos.iter().map(|p| [p.0, p.1]).collect();
            let lines: Vec<&Vec<Vec<(f64, f64)>>> = l.edges.iter().map(|e| &e.lines).collect();
            out.insert(format!("{}/{}", ch.n, si), serde_json::json!({ "pos": pos, "lines": lines, "crossings": l.crossings }));
        }
    }
    println!("{}", serde_json::Value::Object(out));
}

fn main() {
    if std::env::args().any(|a| a == "--dump-layout") { dump_layout(); return; }
    let el = EventLoop::new().expect("event loop");
    let mut r = Runner { app: App::new(), fonts: Fonts::new(), win: None, surface: None, mods: ModifiersState::empty(), scale: 1.0, hover_key: (None, None), debug: std::env::var_os("FT_DEBUG").is_some() };
    let _ = el.run_app(&mut r);
}
