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
    let mut shift_down = false;
    let mut fountain = false;
    let mut well_active = false;
    let mut well_pos = Vec2::new(160.0, 90.0);
    let mut well_radius: f32 = 70.0;
    let mut well_strength: f32 = 1200.0;

    let mut last = Instant::now();

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
        start_color: Color(255, 255, 0, 255), // yellow
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
        start_color: Color(255, 160, 240, 255), // pink
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
        start_color: Color(255, 200, 40, 255), // orange
        end_color: Color(80, 80, 80, 0),       // smoke fade
    };

    let mut active_cfg = sparkle_cfg;

    let mut additive = false;

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
                        VirtualKeyCode::A => {
                            if pressed {
                                additive = !additive;
                                println!("Additive mode: {}", additive);
                            }
                        }
                        VirtualKeyCode::Key1 => {
                            if pressed {
                                active_cfg = sparkle_cfg;
                                println!("Switched to sparkles preset");
                            }
                        }
                        VirtualKeyCode::Key2 => {
                            if pressed {
                                active_cfg = burst_cfg;
                                println!("Switched to burst preset");
                            }
                        }
                        VirtualKeyCode::Key3 => {
                            if pressed {
                                active_cfg = fire_cfg;
                                println!("Switched to fire preset");
                            }
                        }
                        VirtualKeyCode::LShift | VirtualKeyCode::RShift => {
                            shift_down = pressed;
                        }
                        VirtualKeyCode::G => {
                            if pressed {
                                well_active = !well_active;
                                println!("Gravity well: {}", 
                                    if well_active { "ON" } else { "OFF" })
                            }
                        }
                        VirtualKeyCode::W => {
                            if pressed {
                                well_pos = mouse_pos;
                                println!("Well moved to mouse");
                            }
                        }
                        VirtualKeyCode::LBracket => {
                            if pressed {
                                well_radius = (well_radius - 5.0).max(10.0);
                                println!("Well radius: {:.1}", well_radius);
                            }
                        }
                        VirtualKeyCode::RBracket => {
                            if pressed {
                                well_radius += 5.0; 
                                println!("Well radius: {:.1}", well_radius);    
                            }
                        }
                        VirtualKeyCode::Minus => {
                            if pressed {
                                well_strength = (well_strength - 100.0).max(0.0);
                                println!("Well strength: {:.0}", well_strength);
                            }
                        }
                        VirtualKeyCode::Equals => {
                            if pressed {
                                well_strength += 100.0;
                                println!("Well strength: {:.0}", well_strength);
                            }
                        }
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
                    let mut cfg = active_cfg;
                    // tiny x-based wiggle so it feels alive
                    cfg.base_direction = (-std::f32::consts::FRAC_PI_2)
                        + 0.3 * ((mouse_pos.x / 50.0).sin());
                    
                    if shift_down {
                        cfg.base_direction += std::f32::consts::PI;
                    }

                    ps.emit_burst([mouse_pos.x, mouse_pos.y], cfg);
                }

                if fountain {
                    let (w, h) = canvas.size();
                    let mut cfg = active_cfg;
                    cfg.count = 24;
                    cfg.speed_min = 120.0;
                    cfg.speed_max = 240.0;
                    cfg.spread_radians = 0.35;
                    cfg.base_direction = -std::f32::consts::FRAC_PI_2;

                    if shift_down {
                        cfg.base_direction += std::f32::consts::PI;
                    }

                    ps.emit_burst([w as f32 * 0.5, h as f32 * 0.9], cfg);
                }

                if well_active {
                    ps.apply_gravity_well(
                        [well_pos.x, well_pos.y], 
                        well_strength, well_radius, 
                        dt);
                }

                // update + render
                ps.update(dt);

                canvas.clear(Color(12, 12, 16, 255));

                if well_active {
                    canvas.draw_circle_f32(
                        well_pos.x,
                        well_pos.y,
                        well_radius,
                        Color(120, 200, 255, 180)
                    );
                }

                if additive {
                    ps.draw_additive(&mut canvas);
                } else {
                    ps.draw(&mut canvas);
                }

                // tiny mouse dot
                canvas.fill_rect_f32(
                    mouse_pos.x - 2.0, 
                    mouse_pos.y - 2.0, 4.0, 4.0, 
                    Color(255, 255, 255, 160));

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

