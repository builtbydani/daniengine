use crate::prelude::{Canvas, Color};
use crate::render::canvas::CanvasFloatExt;

#[derive(Clone, Copy)]
pub struct EmitterConfig {
    pub count: usize,
    pub speed_min: f32,
    pub speed_max: f32,
    pub spread_radians: f32,
    pub base_direction: f32,
    pub life_min: f32,
    pub life_max: f32,
    pub size_min: f32,
    pub size_max: f32,
    pub start_color: Color,
    pub end_color: Color,
}

#[derive(Clone, Copy)]
struct Particle {
    pos: [f32; 2],
    vel: [f32; 2],
    life: f32,
    life_total: f32,
    size: f32,
    start_color: Color,
    end_color: Color,
    alive: bool,
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
    gravity: [f32; 2],
    rng_state: u32,
}

impl ParticleSystem {
    pub fn new(capacity: usize) -> Self {
        Self {
            particles: (0..capacity)
                .map(|_| Particle {
                    pos: [0.0, 0.0],
                    vel: [0.0, 0.0],
                    life: 0.0,
                    life_total: 0.0,
                    size: 0.0,
                    start_color: Color(255, 255, 255, 255),
                    end_color: Color(0, 0, 255, 255),
                    alive: false,
                })
                .collect(),
            gravity: [0.0, 300.0],
            rng_state: 0x1234ABCD,
        }
    }

    pub fn set_gravity(&mut self, gx: f32, gy: f32) {
        self.gravity = [gx, gy];
    }

    pub fn apply_gravity_well(&mut self, center: [f32; 2], strength: f32, radius: f32, dt: f32) {
        let r2 = radius * radius;
        for p in &mut self.particles {
            if !p.alive { continue; }
            let dx = center[0] - p.pos[0];
            let dy = center[1] - p.pos[1];
            let d2 = dx*dx + dy*dy;
            if d2 > r2 || d2 == 0.0 { continue; }

            let falloff = 1.0 - (d2 / r2);

            let inv_d = 1.0 / d2.sqrt().max(1e-3);
            let nx = dx * inv_d;
            let ny = dy * inv_d;

            let a = strength * falloff;
            p.vel[0] += nx * a * dt;
            p.vel[1] += ny * a * dt;
        }
    }

    pub fn emit_burst(&mut self, pos: [f32; 2], config: EmitterConfig) {
        for _ in 0..config.count {
            if let Some(i) = self.alloc_slot_index() {
                // Generate randomness BEFORE mut-borrowing the particle slot.
                let dir = config.base_direction + self.rand_between(-config.spread_radians, config.spread_radians);
                let spd = self.rand_between(config.speed_min, config.speed_max);
                let life = self.rand_between(config.life_min, config.life_max);
                let size = self.rand_between(config.size_min, config.size_max);

                self.particles[i] = Particle {
                    pos,
                    vel: [dir.cos() * spd, dir.sin() * spd],
                    life,
                    life_total: life,
                    size,
                    start_color: config.start_color,
                    end_color: config.end_color,
                    alive: true,
                };
            } else {
                break;
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            if !p.alive { continue; }
            p.vel[0] += self.gravity[0] * dt;
            p.vel[1] += self.gravity[1] * dt;
            p.pos[0] += p.vel[0] * dt;
            p.pos[1] += p.vel[1] * dt;

            p.life -= dt;
            if p.life <= 0.0 {
                p.alive = false;
            }
        }
    }

    /// Accept *any* canvas that implements the `Canvas` trait.
    pub fn draw<C: Canvas>(&self, canvas: &mut C) {
        for p in &self.particles {
            if !p.alive { continue; }
            let t = (p.life / p.life_total).clamp(0.0, 1.0);
            let Color(sr, sg, sb, sa) = p.start_color;
            let Color(er, eg, eb, ea) = p.end_color;

            let r = sr as f32 + (er as f32 - sr as f32) * (1.0 - t);
            let g = sg as f32 + (eg as f32 - sg as f32) * (1.0 - t);
            let b = sb as f32 + (eb as f32 - sb as f32) * (1.0 - t);
            let a = sa as f32 + (ea as f32 - sa as f32) * (1.0 - t);

            let c = Color(r as u8, g as u8, b as u8, a as u8);
            canvas.fill_rect_f32(p.pos[0], p.pos[1], p.size, p.size, c);
        }
    }

    pub fn draw_additive<C: Canvas>(&self, canvas: &mut C) {
        for p in &self.particles {
            if !p.alive { continue; }
            let t = (p.life / p.life_total).clamp(0.0, 1.0);

            let Color(sr, sg, sb, sa) = p.start_color;
            let Color(er, eg, eb, ea) = p.end_color;
            
            let r = sr as f32 + (er as f32 - sr as f32) * (1.0 - t);
            let g = sg as f32 + (eg as f32 - sg as f32) * (1.0 - t);
            let b = sb as f32 + (eb as f32 - sb as f32) * (1.0 - t);
            let a = sa as f32 + (ea as f32 - sa as f32) * (1.0 - t);

            let boost = 1.5;
            let cr = (r * boost).min(255.0);
            let cg = (g * boost).min(255.0);
            let cb = (b * boost).min(255.0);

            let c = Color(cr as u8, cg as u8, cb as u8, a as u8);
            canvas.fill_rect_f32(p.pos[0], p.pos[1], p.size, p.size, c);

        }
    }

    // --- internals ---

    fn alloc_slot_index(&mut self) -> Option<usize> {
        // Simple linear scan for a free slot
        self.particles.iter().position(|p| !p.alive)
    }

    fn rand_u32(&mut self) -> u32 {
        self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.rng_state
    }

    fn rand_f32(&mut self) -> f32 {
        (self.rand_u32() as f32) / (u32::MAX as f32 + 1.0)
    }

    fn rand_between(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.rand_f32()
    }
}

