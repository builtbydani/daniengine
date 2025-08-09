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

    fn present(&mut self) -> Result<(), String> {
        self.pixels.render().map_err(|e| e.to_string())
    }
}
