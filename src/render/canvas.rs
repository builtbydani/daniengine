#[derive(Clone, Copy, Debug)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

pub trait Canvas {
    fn size(&self) -> (u32, u32);
    fn clear(&mut self, color: Color);
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color);
    fn draw_circle(&mut self, x: i32, y: i32, radius: i32, color: Color);
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color);
    fn present(&mut self) -> Result<(), String>;
}

pub trait CanvasFloatExt: Canvas {
    fn fill_rect_f32(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let xi = x.round() as i32;
        let yi = y.round() as i32;
        let wi = w.max(1.0).round() as i32;
        let hi = h.max(1.0).round() as i32;
        self.fill_rect(xi, yi, wi, hi, color);
    }

    fn draw_circle_f32(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        let x1 = x.round() as i32;
        let y1 = y.round() as i32;
        let r1 = radius.max(1.0).round() as i32;
        self.draw_circle(x1, y1, r1, color);
    }

    fn draw_line_f32(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color) {
        let x1i = x1.round() as i32;
        let y1i = y1.round() as i32;
        let x2i = x2.round() as i32;
        let y2i = y2.round() as i32;
        self.draw_line(x1i, y1i, x2i, y2i, color);
    }
}

impl<T: Canvas> CanvasFloatExt for T {}
