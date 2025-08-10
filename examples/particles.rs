use std::time::Instant;

use daniengine::prelude::*;
use daniengine::render::canvas::{Canvas, Color, CanvasFloatExt};
use daniengine::particles::{EmitterConfig, ParticleSystem};
use daniengine::physics;

use daniengine::input::{Input, Key, Mods, MouseButton};
use daniengine::ui::{Ui, Rect};

#[cfg(feature = "render-pixels")]
use daniengine::render::pixels_impl::PixelsCanvas;

#[cfg(feature = "render-pixels")]
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow},
};

#[cfg(feature = "render-pixels")]
fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Window + canvas
    let (mut canvas, event_loop, window) =
        PixelsCanvas::new(320, 180, 3, "DaniEngine • Particles")?;

    // App (wraps input/ui/state)
    let mut app = App::new(&mut canvas);

    let mut last = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::NewEvents(_) => {
                app.input.begin_frame();
            }

            Event::WindowEvent { event, .. } => {
                match &event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }

                    // Convert physical cursor coords -> canvas coords, write into Input
                    WindowEvent::CursorMoved { position, .. } => {
                        let px = position.x as f32;
                        let py = position.y as f32;

                        // Physical window size
                        let ws = window.inner_size();
                        let ww = ws.width as f32;
                        let wh = ws.height as f32;

                        // Internal canvas size
                        let (cw_i, ch_i) = canvas.size();
                        let cw = cw_i as f32;
                        let ch = ch_i as f32;

                        // Scale & letterbox offset
                        let scale = (ww / cw).min(wh / ch);
                        let ox = (ww - cw * scale) * 0.5;
                        let oy = (wh - ch * scale) * 0.5;

                        // Transform, clamp into canvas bounds
                        let cx = ((px - ox) / scale).clamp(0.0, cw - 1.0);
                        let cy = ((py - oy) / scale).clamp(0.0, ch - 1.0);

                        let new_pos = Vec2::new(cx, cy);
                        app.input.mouse_delta = Vec2::new(
                            new_pos.x - app.input.mouse_pos.x,
                            new_pos.y - app.input.mouse_pos.y,
                        );
                        app.input.mouse_pos = new_pos;
                    }

                    _ => {
                        // Feed everything else (clicks, wheel, keys) to Input
                        app.input.handle_window_event(&event);
                    }
                }

                // Optional: quit on Esc via action binding (handled during update too)
            }

            Event::MainEventsCleared => {
                // dt
                let now = Instant::now();
                let dt = (now - last).as_secs_f32();
                last = now;

                // Hit-test UI and consume click if a button was clicked this frame
                let (cw_i, ch_i) = canvas.size();
                let (cw, ch) = (cw_i as f32, ch_i as f32);
                let over_any_button = app.ui_button_rects(cw, ch)
                    .into_iter()
                    .any(|r| r.contains(app.input.mouse_pos));
                app.ui_click_consumed = over_any_button && 
                    app.input.mouse_pressed(MouseButton::Left);

                // Update
                let should_quit = app.update(dt);
                if should_quit {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                // Render
                canvas.clear(Color(12, 12, 16, 255));

                if let Err(e) = app.render(&mut canvas) {
                    eprintln!("render error: {e}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if let Err(e) = canvas.present() {
                    eprintln!("present error: {e}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            _ => {}
        }
    });
}

// ----------------- App -----------------

struct App {
    // simulation
    ps: ParticleSystem,
    additive: bool,

    // presets
    sparkle_cfg: EmitterConfig,
    burst_cfg: EmitterConfig,
    fire_cfg: EmitterConfig,
    active_cfg: EmitterConfig,

    // toggles
    fountain: bool,

    // gravity well
    well_active: bool,
    well_pos: Vec2,
    well_radius: f32,
    well_strength: f32,

    // bouncing square (like playground)
    body: physics::Body,

    // systems
    input: Input,
    ui: Ui,
    ui_click_consumed: bool,
}

impl App {
    fn new(_canvas: &mut PixelsCanvas) -> Self {
        let mut input = Input::new();

        // --- Actions (single place to bind keys) ---
        input.bind_action("quit",              Input::chord(Key::Escape, Mods::empty()));
        input.bind_action("toggle_fountain",   Input::chord(Key::F, Mods::empty()));
        input.bind_action("reset_particles",   Input::chord(Key::R, Mods::empty()));
        input.bind_action("toggle_additive",   Input::chord(Key::A, Mods::empty()));

        input.bind_action("preset_sparkles",   Input::chord(Key::Key1, Mods::empty()));
        input.bind_action("preset_burst",      Input::chord(Key::Key2, Mods::empty()));
        input.bind_action("preset_fire",       Input::chord(Key::Key3, Mods::empty()));

        input.bind_action("toggle_well",       Input::chord(Key::G, Mods::empty()));
        input.bind_action("move_well_to_mouse",Input::chord(Key::W, Mods::empty()));
        input.bind_action("well_radius_down",  Input::chord(Key::LBracket, Mods::empty()));
        input.bind_action("well_radius_up",    Input::chord(Key::RBracket, Mods::empty()));
        input.bind_action("well_strength_down",Input::chord(Key::Minus, Mods::empty()));
        input.bind_action("well_strength_up",  Input::chord(Key::Equals, Mods::empty()));

        // --- Presets (same as your original) ---
        let burst_cfg = EmitterConfig {
            count: 64,
            speed_min: 80.0,
            speed_max: 220.0,
            spread_radians: std::f32::consts::FRAC_PI_2,
            base_direction: -std::f32::consts::FRAC_PI_2,
            life_min: 0.6,
            life_max: 1.2,
            size_min: 2.0,
            size_max: 4.0,
            start_color: Color(255, 255, 0, 255),
            end_color: Color(0, 10, 255, 255),
        };

        let sparkle_cfg = EmitterConfig {
            count: 64,
            speed_min: 80.0,
            speed_max: 220.0,
            spread_radians: std::f32::consts::FRAC_PI_2,
            base_direction: -std::f32::consts::FRAC_PI_2,
            life_min: 0.6,
            life_max: 1.2,
            size_min: 2.0,
            size_max: 4.0,
            start_color: Color(255, 160, 240, 255),
            end_color: Color(180, 200, 255, 0),
        };

        let fire_cfg = EmitterConfig {
            count: 64,
            speed_min: 80.0,
            speed_max: 220.0,
            spread_radians: std::f32::consts::FRAC_PI_2,
            base_direction: -std::f32::consts::FRAC_PI_2,
            life_min: 0.6,
            life_max: 1.2,
            size_min: 2.0,
            size_max: 4.0,
            start_color: Color(255, 200, 40, 255),
            end_color: Color(80, 80, 80, 0),
        };

        let mut ps = ParticleSystem::new(10_000);
        ps.set_gravity(0.0, 500.0);

        Self {
            ps,
            additive: false,

            sparkle_cfg,
            burst_cfg,
            fire_cfg,
            active_cfg: sparkle_cfg, // default

            fountain: false,

            well_active: false,
            well_pos: Vec2::new(160.0, 90.0),
            well_radius: 70.0,
            well_strength: 1200.0,

            body: physics::Body {
                pos: Vec2::new(40.0, 40.0),
                vel: Vec2::new(60.0, 45.0),
                size: Vec2::new(18.0, 18.0),
            },

            input,
            ui: Ui::new(),
            ui_click_consumed: false,
        }
    }

    // Returns all interactive button rects for current canvas size.
    fn ui_button_rects(&self, canvas_w: f32, canvas_h: f32) -> Vec<daniengine::ui::Rect> {
        use daniengine::ui::Rect;

        // Keep these in sync with draw_ui()
        let m = 8.0;
        let gap = 6.0;
        let top_h = 18.0;
        let col_w = 110.0;
        let bh = 22.0;

        let x_left = m;
        let y_start = m + top_h + gap;

        // Left column buttons (4)
        let mut rects = vec![
            Rect { x: x_left, y: y_start + 0.0*(bh+gap), w: col_w, h: bh }, // Fountain
            Rect { x: x_left, y: y_start + 1.0*(bh+gap), w: col_w, h: bh }, // Well toggle
            Rect { x: x_left, y: y_start + 2.0*(bh+gap), w: col_w, h: bh }, // Blend
            Rect { x: x_left, y: y_start + 3.0*(bh+gap), w: col_w, h: bh }, // Clear
        ];

        // Right side rows
        let x2 = x_left + col_w + gap;
        let y2 = y_start;

        // Presets row (3 small buttons)
        let small_bw = 74.0;
        rects.push(Rect { x: x2 + 0.0*(small_bw+gap), y: y2, w: small_bw, h: bh }); // Sparkle
        rects.push(Rect { x: x2 + 1.0*(small_bw+gap), y: y2, w: small_bw, h: bh }); // Burst
        rects.push(Rect { x: x2 + 2.0*(small_bw+gap), y: y2, w: small_bw, h: bh }); // Fire

        // Well controls row
        let y3 = y2 + bh + gap;
        let sm = 32.0;
        let mut xg = x2;
        rects.push(Rect { x: xg, y: y3, w: sm, h: bh }); xg += sm + gap; // R-
        rects.push(Rect { x: xg, y: y3, w: sm, h: bh }); xg += sm + gap; // R+
        rects.push(Rect { x: xg, y: y3, w: sm, h: bh }); xg += sm + gap; // S-
        rects.push(Rect { x: xg, y: y3, w: sm, h: bh }); xg += sm + gap; // S+

        // "Well @ Mouse" stretches to right margin
        let rem_w = (canvas_w - m) - xg;
        if rem_w > 40.0 {
            rects.push(Rect { x: xg, y: y3, w: rem_w, h: bh });
        }

        rects
    }

    /// Returns true if the app wants to quit (Esc)
    fn update(&mut self, dt: f32) -> bool {
        // --- Global actions ---
        if self.input.action_just_pressed("quit", Mods::empty()) {
            return true;
        }
        if self.input.action_just_pressed("toggle_fountain", Mods::empty()) {
            self.fountain = !self.fountain;
        }
        if self.input.action_just_pressed("reset_particles", Mods::empty()) {
            self.ps = ParticleSystem::new(10_000);
            self.ps.set_gravity(0.0, 500.0);
        }
        if self.input.action_just_pressed("toggle_additive", Mods::empty()) {
            self.additive = !self.additive;
            println!("Additive mode: {}", self.additive);
        }

        // Preset switching
        if self.input.action_just_pressed("preset_sparkles", Mods::empty()) {
            self.active_cfg = self.sparkle_cfg;
            println!("Switched to sparkles preset");
        }
        if self.input.action_just_pressed("preset_burst", Mods::empty()) {
            self.active_cfg = self.burst_cfg;
            println!("Switched to burst preset");
        }
        if self.input.action_just_pressed("preset_fire", Mods::empty()) {
            self.active_cfg = self.fire_cfg;
            println!("Switched to fire preset");
        }

        // Gravity well control
        if self.input.action_just_pressed("toggle_well", Mods::empty()) {
            self.well_active = !self.well_active;
            println!("Gravity well: {}", if self.well_active { "ON" } else { "OFF" });
        }
        if self.input.action_just_pressed("move_well_to_mouse", Mods::empty()) {
            self.well_pos = self.input.mouse_pos.into();
            println!("Well moved to mouse");
        }
        if self.input.action_just_pressed("well_radius_down", Mods::empty()) {
            self.well_radius = (self.well_radius - 5.0).max(10.0);
            println!("Well radius: {:.1}", self.well_radius);
        }
        if self.input.action_just_pressed("well_radius_up", Mods::empty()) {
            self.well_radius += 5.0;
            println!("Well radius: {:.1}", self.well_radius);
        }
        if self.input.action_just_pressed("well_strength_down", Mods::empty()) {
            self.well_strength = (self.well_strength - 100.0).max(0.0);
            println!("Well strength: {:.0}", self.well_strength);
        }
        if self.input.action_just_pressed("well_strength_up", Mods::empty()) {
            self.well_strength += 100.0;
            println!("Well strength: {:.0}", self.well_strength);
        }

        // --- Emitters (mouse) ---
        let mouse_left_down = self.input.mouse_pressed(MouseButton::Left);
        let block_this_frame = self.ui_click_consumed;

        if mouse_left_down && !block_this_frame {
            let mut cfg = self.active_cfg;
            // tiny x-based wiggle so it feels alive
            cfg.base_direction = (-std::f32::consts::FRAC_PI_2) + 0.3 * 
                ((self.input.mouse_pos.x / 50.0).sin());

            // Reverse with Shift (either)
            let reverse = self.input.pressed(Key::LShift) || self.input.pressed(Key::RShift);
            if reverse {
                cfg.base_direction += std::f32::consts::PI;
            }

            self.ps.emit_burst([self.input.mouse_pos.x, self.input.mouse_pos.y], cfg);
        }

        if self.fountain {
            // Emit from bottom center
            let mut cfg = self.active_cfg;
            cfg.count = 24;
            cfg.speed_min = 120.0;
            cfg.speed_max = 240.0;
            cfg.spread_radians = 0.35;
            cfg.base_direction = -std::f32::consts::FRAC_PI_2;

            let reverse = self.input.pressed(Key::LShift) || self.input.pressed(Key::RShift);
            if reverse {
                cfg.base_direction += std::f32::consts::PI;
            }

            // We'll query canvas size in render; here just pick a reasonable 320x180 default
            // (the exact position isn't critical; visually updated in render loop)
            self.ps.emit_burst([160.0, 180.0 * 0.9], cfg);
        }

        if self.well_active {
            self.ps.apply_gravity_well(
                [self.well_pos.x, self.well_pos.y],
                self.well_strength,
                self.well_radius,
                dt,
            );
        }

        // --- Bouncing square (physics demo) ---
        self.body.pos = self.body.pos.add(self.body.vel.mul(dt));

        // We'll clamp in render after we know the current canvas size; but to keep it robust,
        // use the nominal 320x180 when dt spikes (it'll get corrected visually anyway).
        let (w, h) = (320.0, 180.0);
        let s = self.body.size.x;

        if self.body.pos.x <= 0.0 { self.body.pos.x = 0.0; self.body.vel.x =  self.body.vel.x.abs(); }
        if self.body.pos.x + s >= w { self.body.pos.x = w - s; self.body.vel.x = -self.body.vel.x.abs(); }
        if self.body.pos.y <= 0.0 { self.body.pos.y = 0.0; self.body.vel.y =  self.body.vel.y.abs(); }
        if self.body.pos.y + s >= h { self.body.pos.y = h - s; self.body.vel.y = -self.body.vel.y.abs(); }

        // --- Update particles ---
        self.ps.update(dt);

        self.ps.collide_rect(
            [self.body.pos.x, self.body.pos.y, self.body.size.x, self.body.size.y],
            0.6,
        );

        false
    }

    fn render(&mut self, canvas: &mut PixelsCanvas) -> anyhow::Result<()> {
        // Real canvas size for bounds-sensitive drawing
        let (w_i, h_i) = canvas.size();
        let (w, h) = (w_i as f32, h_i as f32);

        // Keep the physics box in-bounds with current size
        let s = self.body.size.x;
        if self.body.pos.x + s >= w { self.body.pos.x = w - s; self.body.vel.x = -self.body.vel.x.abs(); }
        if self.body.pos.y + s >= h { self.body.pos.y = h - s; self.body.vel.y = -self.body.vel.y.abs(); }

        // Gravity well visual
        if self.well_active {
            canvas.draw_circle_f32(
                self.well_pos.x,
                self.well_pos.y,
                self.well_radius,
                Color(120, 200, 255, 180)
            );
        }

        // Bouncing square
        canvas.fill_rect_f32(
            self.body.pos.x,
            self.body.pos.y,
            self.body.size.x,
            self.body.size.y,
            Color(120, 210, 255, 200),
        );

        // Particles
        if self.additive {
            self.ps.draw_additive(canvas);
        } else {
            self.ps.draw(canvas);
        }

        // Mouse dot
        canvas.fill_rect_f32(
            self.input.mouse_pos.x - 2.0,
            self.input.mouse_pos.y - 2.0,
            4.0, 4.0,
            Color(255, 255, 255, 160),
        );

        // --- UI overlay ---
        self.draw_ui(canvas);

        Ok(())
    }

    fn draw_ui(&mut self, canvas: &mut impl Canvas) {
        self.ui.begin();

        // Canvas size (logical px)
        let (cw_i, ch_i) = canvas.size();
        let (w, h) = (cw_i as f32, ch_i as f32);

        // ---- Layout constants (tweak these) ----
        let m = 8.0;        // outer margin
        let gap = 6.0;      // spacing between elements
        let top_h = 18.0;   // top bar height
        let col_w = 110.0;  // left column width
        let bh = 22.0;      // button height
        let bw = col_w;     // left buttons full width

        // ---- Top info bar ----
        self.ui.label(
            canvas,
            Rect { x: m, y: m, w: (w - 2.0 * m).max(0.0), h: top_h },
            "controls"
        );

        // ---- Left column (stacked toggles) ----
        let mut y = m + top_h + gap;
        let x = m;
        let mut vbutton = |label: &str| -> bool {
            let r = Rect { x, y, w: bw, h: bh };
            y += bh + gap;
            self.ui.button(&self.input, canvas, r, label)
        };

        if vbutton(if self.fountain { "Fountain: ON" } else { "Fountain: OFF" }) {
            self.fountain = !self.fountain;
        }
        if vbutton(if self.well_active { "Well: ON" } else { "Well: OFF" }) {
            self.well_active = !self.well_active;
        }
        if vbutton(if self.additive { "Blend: Add" } else { "Blend: Alpha" }) {
            self.additive = !self.additive;
        }
        if vbutton("Clear") {
            self.ps = ParticleSystem::new(10_000);
            self.ps.set_gravity(0.0, 500.0);
        }

        // ---- Right area (presets + well controls) ----
        let x2 = x + col_w + gap;
        let y2 = m + top_h + gap;

        // Presets row
        let small_bw = 74.0;
        let mut xrow = x2;
        let mut row_btn = |label: &str| -> bool {
            let r = Rect { x: xrow, y: y2, w: small_bw, h: bh };
            xrow += small_bw + gap;
            self.ui.button(&self.input, canvas, r, label)
        };

        if row_btn("Sparkle (1)") { self.active_cfg = self.sparkle_cfg; }
        if row_btn("Burst (2)")   { self.active_cfg = self.burst_cfg; }
        if row_btn("Fire (3)")    { self.active_cfg = self.fire_cfg; }

        // Gravity well controls row (below presets)
        let y3 = y2 + bh + gap;
        let mut xg = x2;
        let sm = 32.0; // small button width

        if self.ui.button(&self.input, canvas, Rect { x: xg, y: y3, w: sm, h: bh }, "R-") {
            self.well_radius = (self.well_radius - 5.0).max(10.0);
        }
        xg += sm + gap;

        if self.ui.button(&self.input, canvas, Rect { x: xg, y: y3, w: sm, h: bh }, "R+") {
            self.well_radius += 5.0;
        }
        xg += sm + gap;

        if self.ui.button(&self.input, canvas, Rect { x: xg, y: y3, w: sm, h: bh }, "S-") {
            self.well_strength = (self.well_strength - 100.0).max(0.0);
        }
        xg += sm + gap;

        if self.ui.button(&self.input, canvas, Rect { x: xg, y: y3, w: sm, h: bh }, "S+") {
            self.well_strength += 100.0;
        }
        xg += sm + gap;

        // Stretch the "Well @ Mouse" to the right edge but keep margins
        let rem_w = (w - m) - xg;
        if rem_w > 40.0 {
            if self.ui.button(&self.input, canvas, 
            Rect { x: xg, y: y3, w: rem_w, h: bh }, "Well @ Mouse") {
                self.well_pos = self.input.mouse_pos.into();
            }
        }
    }
}

#[cfg(not(feature = "render-pixels"))]
fn main() {
    println!("Enable the `render-pixels` feature to run this example:
            \n  cargo run -p daniengine --example particles --features render-pixels");
}

