//! Minimal input module for DaniEngine.
//! - Action mapping (strings -> keys)
//! - Axes (e.g., "move_x" from A/D)
//! - Edge detection for keys/mouse
//! - Mouse position/delta provided by caller (you can set it from your pixels transform)

use std::collections::{HashMap, HashSet};

pub use winit::event::VirtualKeyCode as Key;
pub use winit::event::MouseButton;

use winit::event::{ElementState, MouseScrollDelta, WindowEvent};

use crate::prelude::Vec2;

bitflags::bitflags! {
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Mods: u8 {
        const SHIFT = 0b0001;
        const CTRL  = 0b0010;
        const ALT   = 0b0100;
        const SUPER = 0b1000;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub key: Key,
    pub mods: Mods,
}

#[derive(Default)]
pub struct Input {
    // Keyboard state
    pressed_now: HashSet<Key>,
    pressed_prev: HashSet<Key>,

    // Mouse buttons
    mouse_pressed_now: HashSet<MouseButton>,
    mouse_pressed_prev: HashSet<MouseButton>,

    // Pointer + wheel (you set mouse_pos; we track delta)
    pub mouse_pos: Vec2,
    pub mouse_delta: Vec2,
    pub wheel_delta: f32,

    // Mapping
    actions: HashMap<&'static str, Vec<KeyChord>>,
    axes: HashMap<&'static str, Vec<(Key, f32)>>,
}

impl Input {
    pub fn new() -> Self { Self { ..Default::default() } }

    /// Call once per frame, before you handle new events.
    pub fn begin_frame(&mut self) {
        self.pressed_prev = self.pressed_now.clone();
        self.mouse_pressed_prev = self.mouse_pressed_now.clone();
        self.mouse_delta = Vec2::new(0.0, 0.0);
        self.wheel_delta = 0.0;
    }

    /// Feed winit window events (keyboard/mouse buttons + wheel).
    /// NOTE: We intentionally ignore `CursorMoved` here so the example can
    /// set mouse_pos in canvas coordinates after doing its pixels scaling transform.
    pub fn handle_window_event(&mut self, e: &WindowEvent) {
        match e {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    match input.state {
                        ElementState::Pressed => { self.pressed_now.insert(key); }
                        ElementState::Released => { self.pressed_now.remove(&key); }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => { self.mouse_pressed_now.insert(*button); }
                    ElementState::Released => { self.mouse_pressed_now.remove(button); }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.wheel_delta += match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p)   => p.y as f32 / 120.0,
                };
            }
            _ => {}
        }
    }

    // ---------- Queries ----------
    pub fn pressed(&self, key: Key) -> bool { self.pressed_now.contains(&key) }
    pub fn just_pressed(&self, key: Key) -> bool {
        self.pressed_now.contains(&key) && !self.pressed_prev.contains(&key)
    }
    pub fn just_released(&self, key: Key) -> bool {
        !self.pressed_now.contains(&key) && self.pressed_prev.contains(&key)
    }

    pub fn mouse_pressed(&self, b: MouseButton) -> bool { self.mouse_pressed_now.contains(&b) }
    pub fn mouse_clicked(&self, b: MouseButton) -> bool {
        self.mouse_pressed_prev.contains(&b) && !self.mouse_pressed_now.contains(&b)
    }
    pub fn mouse_just_pressed(&self, b: MouseButton) -> bool {
        self.mouse_pressed_now.contains(&b) && !self.mouse_pressed_prev.contains(&b)
    }

    // ---------- Actions ----------
    pub fn bind_action(&mut self, name: &'static str, chord: KeyChord) {
        self.actions.entry(name).or_default().push(chord);
    }

    pub fn action_pressed(&self, name: &str, mods: Mods) -> bool {
        if let Some(list) = self.actions.get(name) {
            list.iter().any(|c| self.pressed(c.key) && (c.mods.is_empty() || c.mods == mods))
        } else { false }
    }

    pub fn action_just_pressed(&self, name: &str, mods: Mods) -> bool {
        if let Some(list) = self.actions.get(name) {
            list.iter().any(|c| self.just_pressed(c.key) && (c.mods.is_empty() || c.mods == mods))
        } else { false }
    }

    // ---------- Axes ----------
    pub fn bind_axis(&mut self, name: &'static str, key: Key, value: f32) {
        self.axes.entry(name).or_default().push((key, value));
    }

    pub fn axis(&self, name: &str) -> f32 {
        self.axes.get(name).map(|pairs| {
            pairs.iter().map(|(k, v)| if self.pressed(*k) { *v } else { 0.0 }).sum()
        }).unwrap_or(0.0)
    }

    pub fn chord(key: Key, mods: Mods) -> KeyChord { KeyChord { key, mods } }
}

