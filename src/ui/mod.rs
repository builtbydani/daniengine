//! Minimal immediate-mode UI with tiny bitmap text.
//! - Rect hit-testing
//! - Buttons with centered labels
//! - Label panels with optional text
//! Uses a 3x5 uppercase bitmap font drawn via fill_rect_f32.

use crate::input::{Input, MouseButton};
use crate::render::canvas::{Canvas, Color, CanvasFloatExt};

#[derive(Clone, Copy)]
pub struct Rect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }

impl Rect {
    pub fn contains(&self, p: crate::prelude::Vec2) -> bool {
        p.x >= self.x && p.x <= self.x + self.w && p.y >= self.y && p.y <= self.y + self.h
    }
}

pub struct Ui {
    hot: Option<u64>,
    active: Option<u64>,
    next_id: u64,
}

impl Ui {
    pub fn new() -> Self { Self { hot: None, active: None, next_id: 1 } }
    pub fn begin(&mut self) { self.hot = None; self.next_id = 1; }
    fn make_id(&mut self) -> u64 { let id = self.next_id; self.next_id += 1; id }

    /// Translucent panel with optional text (uppercased).
    pub fn label(&mut self, canvas: &mut impl Canvas, r: Rect, text: &str) {
        panel(canvas, r);
        if !text.is_empty() {
            draw_text_centered(canvas, r, &text.to_uppercase(), Color(255,255,255,200));
        }
    }

    /// Returns true if clicked. Draws the label centered.
    pub fn button(&mut self, input: &Input, canvas: &mut impl Canvas, r: Rect, label: &str) -> bool {
        let id = self.make_id();
        let hovered = r.contains(input.mouse_pos);
        if hovered { self.hot = Some(id); }

        let pressed_now = input.mouse_pressed(MouseButton::Left);
        let just_released = input.mouse_clicked(MouseButton::Left);

        // Colors
        let base = if hovered { Color(235,235,235,210) } else { Color(220,220,220,180) };
        let border = if pressed_now && hovered { Color(255,100,200,255) } else { Color(80,80,80,220) };

        // Border (2px) then inner fill
        canvas.fill_rect_f32(r.x - 1.0, r.y - 1.0, r.w + 2.0, r.h + 2.0, border);
        canvas.fill_rect_f32(r.x, r.y, r.w, r.h, base);

        draw_text_centered(canvas, r, &label.to_uppercase(), Color(20,20,20,255));

        hovered && just_released
    }
}

/* ------------------------------ panels ------------------------------ */

fn panel(canvas: &mut impl Canvas, r: Rect) {
    canvas.fill_rect_f32(r.x, r.y, r.w, r.h, Color(0,0,0,140));
    // 1px border
    let b = Color(80,80,80,200);
    canvas.fill_rect_f32(r.x, r.y, r.w, 1.0, b);
    canvas.fill_rect_f32(r.x, r.y + r.h - 1.0, r.w, 1.0, b);
    canvas.fill_rect_f32(r.x, r.y, 1.0, r.h, b);
    canvas.fill_rect_f32(r.x + r.w - 1.0, r.y, 1.0, r.h, b);
}

/* ------------------------------ tiny font ------------------------------ */

// 3x5 glyphs encoded as 3-bit rows (LSB at left).
// Only the characters we need for the demo; unknown chars render as a box.
const GLYPH_W: usize = 3;
const GLYPH_H: usize = 5;

fn glyph_rows(c: char) -> [u8; GLYPH_H] {
    use Row as R;
    match c {
        ' ' => [0,0,0,0,0],
        '-' => [0,0b111,0,0,0],
        '+' => [0b010,0b010,0b111,0b010,0b010],
        ':' => [0,0b010,0,0b010,0],
        '(' => [0b001,0b010,0b010,0b010,0b001],
        ')' => [0b100,0b010,0b010,0b010,0b100],
        '@' => [0b111,0b101,0b111,0b100,0b111],

        // digits 0-9
        '0' => [0b111,0b101,0b101,0b101,0b111],
        '1' => [0b010,0b110,0b010,0b010,0b111],
        '2' => [0b111,0b001,0b111,0b100,0b111],
        '3' => [0b111,0b001,0b111,0b001,0b111],
        '4' => [0b101,0b101,0b111,0b001,0b001],
        '5' => [0b111,0b100,0b111,0b001,0b111],
        '6' => [0b111,0b100,0b111,0b101,0b111],
        '7' => [0b111,0b001,0b010,0b100,0b100],
        '8' => [0b111,0b101,0b111,0b101,0b111],
        '9' => [0b111,0b101,0b111,0b001,0b111],

        // A-Z (subset is enough for our labels)
        'A' => [0b010,0b101,0b111,0b101,0b101],
        'B' => [0b110,0b101,0b110,0b101,0b110],
        'C' => [0b011,0b100,0b100,0b100,0b011],
        'D' => [0b110,0b101,0b101,0b101,0b110],
        'E' => [0b111,0b100,0b110,0b100,0b111],
        'F' => [0b111,0b100,0b110,0b100,0b100],
        'G' => [0b011,0b100,0b101,0b101,0b011],
        'H' => [0b101,0b101,0b111,0b101,0b101],
        'I' => [0b111,0b010,0b010,0b010,0b111],
        'K' => [0b101,0b110,0b100,0b110,0b101],
        'L' => [0b100,0b100,0b100,0b100,0b111],
        'M' => [0b101,0b111,0b111,0b101,0b101],
        'N' => [0b101,0b111,0b111,0b111,0b101],
        'O' => [0b111,0b101,0b101,0b101,0b111],
        'P' => [0b110,0b101,0b110,0b100,0b100],
        'R' => [0b110,0b101,0b110,0b110,0b101],
        'S' => [0b011,0b100,0b011,0b001,0b110],
        'T' => [0b111,0b010,0b010,0b010,0b010],
        'U' => [0b101,0b101,0b101,0b101,0b111],
        'V' => [0b101,0b101,0b101,0b101,0b010],
        'W' => [0b101,0b101,0b111,0b111,0b101],
        'Y' => [0b101,0b101,0b010,0b010,0b010],
        _ => [0b111,0b101,0b101,0b101,0b111], // unknown -> box
    }
}

// Draw a string with a given pixel scale.
fn draw_text(canvas: &mut impl Canvas, x: f32, y: f32, text: &str, color: Color, scale: f32) {
    let mut cx = x;
    for ch in text.chars() {
        let rows = glyph_rows(ch);
        for (ry, bits) in rows.iter().enumerate() {
            // NEW (MSB-left → correct orientation)
            for cxbit in 0..GLYPH_W {
                let mask = 1 << (GLYPH_W - 1 - cxbit);
                if (bits & mask) != 0 {
                    canvas.fill_rect_f32(
                        cx + (cxbit as f32) * scale,
                        y  + (ry as f32) * scale,
                        scale, scale, color
                    );
                }
            }
        }
        // 1px spacing between glyphs
        cx += (GLYPH_W as f32 + 1.0) * scale;
    }
}

// Compute scaled width of a string in pixels.
fn measure_text_px(text: &str, scale: f32) -> f32 {
    let n = text.chars().count() as f32;
    ((GLYPH_W as f32 + 1.0) * n - 1.0) * scale
}

// Center text inside a rect. Scale to fit if needed.
fn draw_text_centered(canvas: &mut impl Canvas, r: Rect, text: &str, color: Color) {
    if text.is_empty() { return; }
    // Choose the largest integer-ish scale that fits height & width.
    let max_h_scale = (r.h - 4.0).max(4.0) / (GLYPH_H as f32);
    // Start with height-limited scale; clamp by width if needed.
    let mut scale = max_h_scale.floor().max(1.0);
    let mut w_px = measure_text_px(text, scale);
    if w_px > r.w - 6.0 {
        scale = ((r.w - 6.0) / ((GLYPH_W as f32 + 1.0) * text.chars().count() as f32 - 1.0)).floor().max(1.0);
        w_px = measure_text_px(text, scale);
    }
    let x = r.x + (r.w - w_px) * 0.5;
    let y = r.y + (r.h - (GLYPH_H as f32 * scale)) * 0.5;
    draw_text(canvas, x, y, text, color, scale);
}

// Simple alias to make row defs readable if you tweak glyphs.
#[allow(dead_code)]
enum Row { _0=0, _1=1, _2=2, _3=3, _4=4, _5=5, _6=6, _7=7 }

