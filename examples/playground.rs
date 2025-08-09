use std::time::{Duration, Instant};

use daniengine::prelude::*;
use daniengine::render::canvas::{Canvas, Color};
use daniengine::physics;

#[cfg(feature = "render-pixels")]
use daniengine::render::pixels_impl::PixelsCanvas;

#[cfg(feature = "render-pixels")]
use winit::{
    event::{Event, WindowEvent, ElementState, VirtualKeyCode, KeyboardInput}, 
    event_loop::{ControlFlow, EventLoop},
};

#[cfg(feature = "render-pixels")]
fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (mut canvas, event_loop, _window) =
        PixelsCanvas::new(320, 180, 3, "DaniEngine • Playground")?;

    let mut body = physics::Body { 
        pos: Vec2::new(40.0, 40.0),
        vel: Vec2::new(60.0, 45.0),
        size: Vec2::new(10.0, 10.0),
    };

    let mut input = Vec2::default();
    let speed = 120.0;

    let target = Duration::from_secs_f32(1.0 / 60.0);
    let mut acc = Duration::ZERO;
    let mut last = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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
                    let v = if pressed { 1.0 } else { 0.0 };
                    match key {
                        VirtualKeyCode::Left  => input.x = -v,
                        VirtualKeyCode::Right => input.x =  v,
                        VirtualKeyCode::Up    => input.y = -v,
                        VirtualKeyCode::Down  => input.y =  v,
                        _ => {}
                    }
                }
                _ => {}
            },

            Event::MainEventsCleared => {
                let now = Instant::now();
                let mut dt = now - last;
                last = now;
                if dt > Duration::from_millis(100) {
                    dt = Duration::from_millis(100); 
                }
                acc += dt;

                while acc >= target {
                    body.pos = body.pos.add(Vec2::new(input.x*speed, input.y*speed).mul(1.0/60.0));
                    body.update(1.0/60.0);

                    // simple bounce
                    let (w, h) = canvas.size();
                    let s = body.size.x;
                    let (w, h) = (w as f32, h as f32);

                    if body.pos.x <= 0.0 { body.pos.x = 0.0; body.vel.x = body.vel.x.abs(); }
                    if body.pos.x + s >= w { body.pos.x = w - s; body.vel.x = -body.vel.x.abs(); }
                    if body.pos.y <= 0.0 { body.pos.y = 0.0; body.vel.y = body.vel.y.abs(); }
                    if body.pos.y + s >= h { body.pos.y = h - s; body.vel.y = -body.vel.y.abs(); }

                    acc -= target;
                }

                canvas.clear(Color(12,12,16,255));
                canvas.fill_rect(
                    body.pos.x as i32, 
                    body.pos.y as i32, 
                    body.size.x as i32, 
                    body.size.y as i32, 
                    Color(255,179,218,255),
                );
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
            \n  cargo run -p daniengine --example playground --features render-pixels");
}

