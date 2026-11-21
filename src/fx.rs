// Lightweight particle and floating-text system.
use crate::art::{self, Atlas};
use macroquad::prelude::*;

#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max: f32,
    pub kind: u8, // 0 smoke, 1 fire, 2 spark, 3 debris, 4 dust
    pub seed: u32,
    pub size: f32,
}

#[derive(Clone)]
pub struct FloatText {
    pub x: f32,
    pub y: f32,
    pub life: f32,
    pub color: Color,
    pub text: String,
    pub size: u16,
}

pub struct Fx {
    pub p: Vec<Particle>,
    pub texts: Vec<FloatText>,
    pub shake: f32,
    pub flash: f32,
    pub flash_col: Color,
    pub t: f32,
}

impl Fx {
    pub fn new() -> Self {
        Fx {
            p: Vec::new(),
            texts: Vec::new(),
            shake: 0.0,
            flash: 0.0,
            flash_col: Color::new(1.0, 0.8, 0.4, 1.0),
            t: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.t += dt;
        self.shake *= (1.0 - dt * 6.0).max(0.0);
        if self.shake < 0.05 {
            self.shake = 0.0;
        }
        self.flash = (self.flash - dt * 2.4).max(0.0);
        let mut i = 0;
        while i < self.p.len() {
            let p = &mut self.p[i];
            p.life -= dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            match p.kind {
                0 => {
                    p.vy -= 5.0 * dt;
                    p.vx *= 1.0 - dt * 0.6;
                }
                1 => {
                    p.vy -= 9.0 * dt;
                    p.vx *= 1.0 - dt * 1.2;
                }
                2 => p.vy += 90.0 * dt,
                3 => {
                    p.vy += 120.0 * dt;
                    p.vx *= 1.0 - dt * 0.4;
                }
                _ => {
                    p.vx *= 1.0 - dt * 1.5;
                    p.vy *= 1.0 - dt * 1.5;
                }
            }
            if p.life <= 0.0 {
                self.p.swap_remove(i);
            } else {
                i += 1;
            }
        }
        let mut i = 0;
        while i < self.texts.len() {
            self.texts[i].life -= dt;
            self.texts[i].y -= 9.0 * dt;
            if self.texts[i].life <= 0.0 {
                self.texts.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn boom(&mut self, x: f32, y: f32, scale: f32) {
        let n = (7.0 * scale) as usize + 4;
        for k in 0..n {
            let a = k as f32 / n as f32 * std::f32::consts::TAU;
            let sp = 22.0 + (k % 5) as f32 * 9.0;
            self.p.push(Particle {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp * 0.5 - 16.0,
                life: 0.35 + (k % 4) as f32 * 0.12,
                max: 0.8,
                kind: if k % 3 == 0 { 3 } else { 2 },
                seed: k as u32,
                size: 1.0 + (k % 3) as f32,
            });
        }
        for k in 0..(4.0 * scale) as usize + 2 {
            self.p.push(Particle {
                x: x + (k as f32 - 2.0) * 3.0,
                y: y - k as f32 * 2.0,
                vx: (k as f32 - 2.0) * 3.0,
                vy: -10.0 - k as f32 * 2.0,
                life: 0.7 + k as f32 * 0.16,
                max: 1.4,
                kind: if k % 2 == 0 { 1 } else { 0 },
                seed: k as u32,
                size: 1.0 + k as f32 * 0.18,
            });
        }
        self.shake = (self.shake + 3.0 * scale).min(8.0);
        self.flash = (self.flash + 0.25 * scale).min(0.8);
        self.flash_col = Color::new(1.0, 0.72, 0.34, 1.0);
    }

    pub fn smoke(&mut self, x: f32, y: f32, n: usize) {
        for k in 0..n {
            self.p.push(Particle {
                x: x + (k as f32 - n as f32 * 0.5) * 2.0,
                y: y - 4.0,
                vx: (k as f32 - n as f32 * 0.5) * 1.5,
                vy: -7.0 - (k % 3) as f32 * 2.0,
                life: 1.2 + k as f32 * 0.25,
                max: 2.4,
                kind: if k % 4 == 0 { 1 } else { 0 },
                seed: k as u32,
                size: 0.8 + k as f32 * 0.1,
            });
        }
    }

    pub fn dust(&mut self, x: f32, y: f32) {
        for k in 0..5 {
            self.p.push(Particle {
                x,
                y,
                vx: (k as f32 - 2.0) * 8.0,
                vy: -2.0 - (k % 2) as f32 * 3.0,
                life: 0.5,
                max: 0.5,
                kind: 4,
                seed: k,
                size: 1.0,
            });
        }
    }

    pub fn trace(&mut self, x: f32, y: f32) {
        self.p.push(Particle { x, y, vx: 0.0, vy: -18.0, life: 0.25, max: 0.25, kind: 2, seed: 9, size: 1.0 });
    }

    pub fn text(&mut self, x: f32, y: f32, s: &str, color: Color) {
        self.texts.push(FloatText { x, y, life: 1.6, color, text: s.to_string(), size: 8 });
        if self.texts.len() > 24 {
            self.texts.remove(0);
        }
    }

    pub fn draw(&self, atlas: &Atlas, origin: Vec2, zoom: f32) {
        for p in &self.p {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            let sx = p.x * zoom + origin.x;
            let sy = p.y * zoom + origin.y;
            match p.kind {
                0 => {
                    let name = ["smoke0", "smoke1", "smoke2"][p.seed as usize % 3];
                    let s = 0.6 + (1.0 - a) * 0.9;
                    atlas.draw_at_scaled(name, sx, sy, s * p.size * zoom, Color::new(0.82, 0.84, 0.86, a * 0.55));
                }
                1 => {
                    let name = if p.seed % 2 == 0 { "fire0" } else { "fire1" };
                    let s = 0.5 + a * 0.8;
                    atlas.draw_at_scaled(name, sx, sy, s * p.size * zoom, Color::new(1.0, 0.95, 0.8, a));
                }
                2 => draw_rectangle(sx, sy, 1.0, 1.0, Color::new(1.0, 0.85, 0.45, a)),
                3 => draw_rectangle(sx, sy, p.size * zoom, p.size * zoom, Color::new(0.25, 0.22, 0.18, a)),
                _ => draw_rectangle(
                    sx - 2.0 * zoom,
                    sy - 1.0 * zoom,
                    4.0 * p.size * zoom,
                    2.0 * zoom,
                    Color::new(0.52, 0.48, 0.38, a * 0.4),
                ),
            }
        }
        let _ = art::TEXT;
    }

    pub fn draw_texts(&self, font: &Font, origin: Vec2, zoom: f32) {
        for t in &self.texts {
            let a = (t.life / 1.6).clamp(0.0, 1.0);
            let c = Color::new(t.color.r, t.color.g, t.color.b, a);
            let x = t.x * zoom + origin.x;
            let y = t.y * zoom + origin.y;
            let m = measure_text(&t.text, Some(font), t.size, 1.0);
            draw_text_ex(
                &t.text,
                x - m.width * 0.5 + 1.0,
                y + 1.0,
                TextParams { font: Some(font), font_size: t.size, color: Color::new(0.0, 0.0, 0.0, a * 0.7), ..Default::default() },
            );
            draw_text_ex(
                &t.text,
                x - m.width * 0.5,
                y,
                TextParams { font: Some(font), font_size: t.size, color: c, ..Default::default() },
            );
        }
    }
}
