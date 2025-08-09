use std::time::Instant;

use daniengine::prelude::*;
use daniengine::render::canvas::{Canvas, Color, CanvasFloatExt};
use daniengine::particles::{EmitterConfig, ParticleSystem};

#[cfg(feature = "render-pixels")]
use daniengine::render::pixels_impl::PixelsCanvas;

#[cfg(feature = "render-pixels")]
use winit::{
    event::{Event, WindowEvent, ElementState, VirtualKeyCode, KeyboardInput, MouseButton},
    event_loop::{ControlFlow, EventLoop},
};

#[cfg(feature = "render-pixels")]
fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Same init style as playground
    let (mut canvas, event_loop, window) =
        PixelsCanvas::new(320, 180, 3, "DaniEngine • Particles")?;

    // --- particle system ---
    let mut ps = ParticleSystem::new(10_000);
    ps.set_gravity(0.0, 500.0);

    // input state
    let mut mouse_pos = Vec2::new(160.0, 90.0);
    let mut mouse_down = false;
    let mut fountain = false;

    // timing
    let mut last = Instant::now();

    // a small helper to make a burst config
    let mut burst_cfg = || EmitterConfig {
        count: 64,
        speed_min: 80.0,
        speed_max: 220.0,
        spread_radians: std::f32::consts::FRAC_PI_2,       // 90°
        base_direction: -std::f32::consts::FRAC_PI_2,      // up
        life_min: 0.6,
        life_max: 1.2,
        size_min: 2.0,
        size_max: 4.0,
        color: Color(255, 160, 240, 255),                  // pink 💖
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                WindowEvent::CursorMoved { position, .. } => {
                    // Account for HiDPI scaling
                    let px = position.x as f32;
                    let py = position.y as f32;

                    // Physical window size
                    let ws = window.inner_size();
                    let ww = ws.width as f32;
                    let wh = ws.height as f32;

                    // Internal canvas size
                    let (cw, ch) = canvas.size();
                    let cw = cw as f32;
                    let ch = ch as f32;

                    // How much the canvas is scaled to fit in the window
                    let scale = (ww / cw).min(wh / ch);

                    // Letterbox offsets if aspect ratios differ
                    let ox = (ww - cw * scale) * 0.5;
                    let oy = (wh - ch * scale) * 0.5;

                    // Transform window coords to canvas coords
                    let cx = (px - ox) / scale;
                    let cy = (py - oy) / scale;

                    // Clamp inside the canvas
                    mouse_pos.x = cx.clamp(0.0, cw - 1.0);
                    mouse_pos.y = cy.clamp(0.0, ch - 1.0);
                }

                WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Left {
                        mouse_down = state == ElementState::Pressed;
                    }
                }

                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => {
                    let pressed = state == ElementState::Pressed;
                    match key {
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        VirtualKeyCode::F => fountain = pressed,
                        VirtualKeyCode::R => ps = ParticleSystem::new(10_000),
                        _ => {}
                    }
                }
                _ => {}
            },

            Event::MainEventsCleared => {
                // dt
                let now = Instant::now();
                let dt = (now - last).as_secs_f32();
                last = now;

                // emitters
                if mouse_down {
                    let mut cfg = burst_cfg();
                    // tiny x-based wiggle so it feels alive
                    cfg.base_direction = (-std::f32::consts::FRAC_PI_2) + 0.3 * ((mouse_pos.x / 50.0).sin());
                    ps.emit_burst([mouse_pos.x, mouse_pos.y], cfg);
                }

                if fountain {
                    let (w, h) = canvas.size();
                    let mut cfg = burst_cfg();
                    cfg.count = 24;
                    cfg.speed_min = 120.0;
                    cfg.speed_max = 240.0;
                    cfg.spread_radians = 0.35;
                    cfg.base_direction = -std::f32::consts::FRAC_PI_2;
                    ps.emit_burst([w as f32 * 0.5, h as f32 * 0.9], cfg);
                }

                // update + render
                ps.update(dt);

                canvas.clear(Color(12, 12, 16, 255));
                ps.draw(&mut canvas);
                // tiny mouse dot
                canvas.fill_rect_f32(mouse_pos.x - 2.0, mouse_pos.y - 2.0, 4.0, 4.0, Color(255, 255, 255, 160));

                if let Err(e) = canvas.present() {
                    eprintln!("present error: {e}");
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}

#[cfg(not(feature = "render-pixels"))]
fn main() {
    println!("Enable the `render-pixels` feature to run this example:
            \n  cargo run -p daniengine --example particles --features render-pixels");
}

