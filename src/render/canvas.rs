#[derive(Clone, Copy, Debug)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

pub trait Canvas {
    fn size(&self) -> (u32, u32);
    fn clear(&mut self, color: Color);
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color);
    fn present(&mut self) -> Result<(), String>;
}
