use crate::math::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Aabb { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }

impl Aabb {
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.x < other.x + other.w && self.x + self.w > other.x &&
        self.y < other.y + other.h && self.y + self.h > other.y
    }
}

pub struct Body {
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: Vec2,
}

impl Body {
    pub fn update(&mut self, dt: f32) {self.pos = self.pos.add(self.vel.mul(dt)); }
    pub fn aabb(&self) -> Aabb { Aabb { x: self.pos.x, 
                                        y: self.pos.y, 
                                        w: self.size.x, 
                                        h: self.size.y 
                                      }
                                }
}
