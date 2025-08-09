#[cfg(feature = "render-pixels")]
use pixels::{Pixels, SurfaceTexture};

#[cfg(feature = "render-pixels")]
use winit::{event_loop::EventLoop, window::WindowBuilder, dpi::LogicalSize};

use super::canvas::{Canvas, Color};

#[cfg(feature = "render-pixels")]
pub struct PixelsCanvas {
    pixels: Pixels,
    width: u32,
    height: u32,
}

#[cfg(feature = "render-pixels")]
impl PixelsCanvas {
    pub fn new(width: u32, 
               height: u32, 
               scale: u32, 
               title: &str) -> 
               anyhow::Result<(Self,
               winit::event_loop::EventLoop<()>,
               winit::window::Window)>
    {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new((width*scale) as f64, (height*scale) as f64))
            .with_resizable(false)
            .build(&event_loop)?;

        let surface = SurfaceTexture::new(width*scale, height*scale, &window);
        let pixels = Pixels::new(width, height, surface)?;
        Ok((Self { pixels, width, height }, event_loop, window))
    }
}

#[cfg(feature = "render-pixels")]
impl Canvas for PixelsCanvas {
    fn size(&self) -> (u32, u32) { (self.width, self.height) }

    fn clear(&mut self, color: Color) {
        for px in self.pixels.frame_mut().chunks_exact_mut(4) {
            px.copy_from_slice(&[color.0, color.1, color.2, color.3]);
        }
    }

    fn fill_rect (&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        let (W, H) = (self.width as i32, self.height as i32);
        let frame = self.pixels.frame_mut();
        for yy in y.max(0)..(y+h).min(H) {
            for xx in x.max(0)..(x+w).min(W) {
                let idx = ((yy as u32 * self.width + xx as u32) * 4) as usize;
                frame[idx..idx+4].copy_from_slice(&[color.0, color.1, color.2, color.3]);
            }
        }
    }

    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color) {
        // Bresenham
        let (mut x, mut y) = (x1, y1);
        let dx = (x2 - x1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let dy = -(y2 - y1).abs();
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.fill_rect(x, y, 1, 1, color);
            if x == x2 && y == y2 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x += sx; }
            if e2 <= dx { err += dx; y += sy; }
        }
    }

    fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        // Midpoint circle (outline)
        if radius <= 0 { return; }
        let mut x = 0;
        let mut y = radius;
        let mut d = 1 - radius;

        // helper to plot 8-way symmetry
        let mut plot = |px: i32, py: i32| { self.fill_rect(px, py, 1, 1, color); };

        // initial cardinal points
        plot(cx, cy + radius);
        plot(cx, cy - radius);
        plot(cx + radius, cy);
        plot(cx - radius, cy);

        while x < y {
            if d < 0 {
                d += 2 * x + 3;
            } else {
                d += 2 * (x - y) + 5;
                y -= 1;
            }
            x += 1;

            // 8 octants
            plot(cx + x, cy + y);
            plot(cx - x, cy + y);
            plot(cx + x, cy - y);
            plot(cx - x, cy - y);
            plot(cx + y, cy + x);
            plot(cx - y, cy + x);
            plot(cx + y, cy - x);
            plot(cx - y, cy - x);
        }
    }

    fn present(&mut self) -> Result<(), String> {
        self.pixels.render().map_err(|e| e.to_string())
    }
}
