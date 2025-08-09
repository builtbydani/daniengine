#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2 { pub x: f32, pub y: f32 }
impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
    pub fn add(self, o: Self) -> Self { Self::new(self.x + o.x, self.y + o.y) }
    pub fn mul(self, s: f32) -> Self { Self::new(self.x * s, self.y * s) }
}
