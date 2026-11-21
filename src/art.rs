// Procedural pixel art. Every sprite is baked at startup into one atlas texture.
use macroquad::prelude::*;
use std::collections::HashMap;

pub type C = [u8; 4];

pub const fn rgb(r: u8, g: u8, b: u8) -> C {
    [r, g, b, 255]
}
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> C {
    [r, g, b, a]
}
pub const fn shade(c: C, f: f32) -> C {
    [
        (c[0] as f32 * f).min(255.0) as u8,
        (c[1] as f32 * f).min(255.0) as u8,
        (c[2] as f32 * f).min(255.0) as u8,
        c[3],
    ]
}
pub fn col(c: C) -> Color {
    Color::new(c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, c[3] as f32 / 255.0)
}
pub fn mix(a: C, b: C, t: f32) -> C {
    [
        (a[0] as f32 * (1.0 - t) + b[0] as f32 * t) as u8,
        (a[1] as f32 * (1.0 - t) + b[1] as f32 * t) as u8,
        (a[2] as f32 * (1.0 - t) + b[2] as f32 * t) as u8,
        255,
    ]
}

// ---------- palette ----------
pub const INK: C = rgb(12, 15, 20);
pub const PANEL: C = rgb(23, 29, 38);
pub const PANEL2: C = rgb(31, 39, 50);
pub const PANEL3: C = rgb(43, 53, 66);
pub const LINE: C = rgb(62, 74, 90);
pub const TEXT: C = rgb(232, 228, 210);
pub const TEXT_DIM: C = rgb(158, 158, 140);
pub const AMBER: C = rgb(226, 168, 66);
pub const AMBER_D: C = rgb(140, 96, 30);
pub const RED: C = rgb(196, 64, 48);
pub const RED_D: C = rgb(108, 32, 28);
pub const GREEN: C = rgb(112, 176, 92);
pub const GREEN_D: C = rgb(56, 104, 52);
pub const BLUE: C = rgb(92, 142, 220);
pub const CYAN: C = rgb(96, 190, 200);
pub const UA_BLUE: C = rgb(48, 98, 190);
pub const UA_YELLOW: C = rgb(238, 202, 74);
pub const RU_WHITE: C = rgb(226, 226, 224);
pub const RU_BLUE: C = rgb(42, 66, 140);
pub const RU_RED: C = rgb(176, 48, 44);

pub const GRASS: [C; 5] = [
    rgb(46, 72, 42),
    rgb(58, 88, 48),
    rgb(72, 106, 56),
    rgb(90, 128, 66),
    rgb(112, 148, 80),
];
pub const DIRT: [C; 5] = [
    rgb(58, 45, 32),
    rgb(78, 60, 42),
    rgb(100, 78, 54),
    rgb(124, 99, 70),
    rgb(148, 121, 88),
];
pub const ROAD: [C; 4] = [rgb(56, 54, 49), rgb(72, 69, 62), rgb(90, 86, 76), rgb(108, 103, 90)];
pub const WATER: [C; 4] = [rgb(26, 52, 72), rgb(36, 70, 94), rgb(50, 92, 118), rgb(74, 122, 148)];
pub const ROOF_RUST: [C; 4] = [rgb(76, 40, 34), rgb(108, 54, 42), rgb(140, 72, 52), rgb(172, 96, 68)];
pub const ROOF_GRAY: [C; 4] = [rgb(44, 48, 54), rgb(62, 67, 75), rgb(82, 88, 97), rgb(106, 113, 122)];
pub const ROOF_GREEN: [C; 4] = [rgb(40, 58, 42), rgb(54, 76, 52), rgb(70, 96, 62), rgb(92, 120, 78)];
pub const WALL_PLASTER: [C; 3] = [rgb(150, 138, 116), rgb(186, 172, 144), rgb(214, 200, 168)];
pub const WALL_BRICK: [C; 3] = [rgb(104, 62, 48), rgb(136, 82, 60), rgb(164, 104, 76)];
pub const WALL_GREEN: [C; 3] = [rgb(56, 70, 50), rgb(78, 94, 64), rgb(100, 116, 80)];
pub const METAL: [C; 5] = [
    rgb(38, 42, 48),
    rgb(56, 62, 70),
    rgb(78, 86, 96),
    rgb(104, 114, 126),
    rgb(140, 150, 160),
];
pub const CONCRETE: [C; 4] = [rgb(92, 92, 88), rgb(118, 118, 112), rgb(144, 144, 136), rgb(170, 170, 160)];
pub const FIRE: [C; 4] = [rgb(132, 46, 20), rgb(210, 82, 26), rgb(244, 140, 40), rgb(255, 214, 110)];
pub const GOLD: [C; 3] = [rgb(150, 104, 32), rgb(210, 158, 58), rgb(246, 208, 110)];

// ---------- pixel canvas ----------
pub struct Pix {
    pub w: i32,
    pub h: i32,
    pub px: Vec<C>,
}

impl Pix {
    pub fn new(w: i32, h: i32) -> Self {
        Pix { w, h, px: vec![[0, 0, 0, 0]; (w * h) as usize] }
    }
    pub fn put(&mut self, x: i32, y: i32, c: C) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h && c[3] > 0 {
            self.px[(y * self.w + x) as usize] = c;
        }
    }
    pub fn get(&self, x: i32, y: i32) -> C {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize]
        } else {
            [0, 0, 0, 0]
        }
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: C) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.put(xx, yy, c);
            }
        }
    }
    pub fn rect_out(&mut self, x: i32, y: i32, w: i32, h: i32, c: C) {
        for xx in x..x + w {
            self.put(xx, y, c);
            self.put(xx, y + h - 1, c);
        }
        for yy in y..y + h {
            self.put(x, yy, c);
            self.put(x + w - 1, yy, c);
        }
    }
    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: C) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.put(x, y, c);
        }
    }
    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, c: C) {
        for y in y0.min(y1)..=y0.max(y1) {
            self.put(x, y, c);
        }
    }
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: C) {
        let (mut x0, mut y0) = (x0, y0);
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.put(x0, y0, c);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }
    pub fn tri(&mut self, a: (f32, f32), b: (f32, f32), c: (f32, f32), color: C) {
        let minx = a.0.min(b.0).min(c.0).floor().max(0.0) as i32;
        let maxx = a.0.max(b.0).max(c.0).ceil().min(self.w as f32 - 1.0) as i32;
        let miny = a.1.min(b.1).min(c.1).floor().max(0.0) as i32;
        let maxy = a.1.max(b.1).max(c.1).ceil().min(self.h as f32 - 1.0) as i32;
        let edge = |p: (f32, f32), q: (f32, f32), r: (f32, f32)| -> f32 {
            (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0)
        };
        for y in miny..=maxy {
            for x in minx..=maxx {
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                let w0 = edge(b, c, p);
                let w1 = edge(c, a, p);
                let w2 = edge(a, b, p);
                if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
                    self.put(x, y, color);
                }
            }
        }
    }
    pub fn quad(&mut self, a: (f32, f32), b: (f32, f32), c: (f32, f32), d: (f32, f32), color: C) {
        self.tri(a, b, c, color);
        self.tri(a, c, d, color);
    }
    pub fn iso_diamond(&mut self, cx: i32, cy: i32, hw: i32, hh: i32, color: C) {
        for y in -hh..=hh {
            let w = hw - (hw * y.abs()) / hh.max(1);
            for x in -w..=w {
                self.put(cx + x, cy + y, color);
            }
        }
    }
    pub fn disc(&mut self, cx: i32, cy: i32, r: i32, color: C) {
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r * r {
                    self.put(cx + x, cy + y, color);
                }
            }
        }
    }
    pub fn ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: C) {
        for y in -ry..=ry {
            for x in -rx..=rx {
                let fx = x as f32 / rx.max(1) as f32;
                let fy = y as f32 / ry.max(1) as f32;
                if fx * fx + fy * fy <= 1.0 {
                    self.put(cx + x, cy + y, color);
                }
            }
        }
    }
    /// random dots only inside the existing opaque silhouette
    pub fn noise_opaque(&mut self, c0: C, density: f32, seed: u32) {
        let mut r = Rng::new(seed);
        for i in 0..self.px.len() {
            if self.px[i][3] > 0 && r.f() < density {
                self.px[i] = c0;
            }
        }
    }
    pub fn noise_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c0: C, density: f32, seed: u32) {
        let mut r = Rng::new(seed);
        for yy in y..y + h {
            for xx in x..x + w {
                if r.f() < density {
                    self.put(xx, yy, c0);
                }
            }
        }
    }
    /// add a dark silhouette outline outside the shape
    pub fn outline(&mut self, color: C) {
        let src = self.px.clone();
        let (w, h) = (self.w, self.h);
        let at = move |x: i32, y: i32| -> C {
            if x >= 0 && y >= 0 && x < w && y < h {
                src[(y * w + x) as usize]
            } else {
                [0, 0, 0, 0]
            }
        };
        for y in 0..h {
            for x in 0..w {
                if at(x, y)[3] == 0 {
                    if at(x - 1, y)[3] > 0 || at(x + 1, y)[3] > 0 || at(x, y - 1)[3] > 0 || at(x, y + 1)[3] > 0 {
                        self.put(x, y, color);
                    }
                }
            }
        }
    }
    pub fn cast_shadow(&mut self, cx: i32, cy: i32, rx: i32, ry: i32) {
        for y in -ry..=ry {
            for x in -rx..=rx {
                let fx = x as f32 / rx.max(1) as f32;
                let fy = y as f32 / ry.max(1) as f32;
                if fx * fx + fy * fy <= 1.0 {
                    let dst = self.get(cx + x, cy + y);
                    if dst[3] == 0 {
                        let a = if fx * fx + fy * fy > 0.55 { 34 } else { 58 };
                        self.put(cx + x, cy + y, rgba(6, 8, 10, a));
                    }
                }
            }
        }
    }
}

pub struct Rng(pub u32);
impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng(seed.wrapping_mul(2654435761).wrapping_add(12345))
    }
    pub fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 & 0xffff) as f32 / 65535.0
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}

// ---------- atlas ----------
pub struct Atlas {
    pub tex: Texture2D,
    pub map: HashMap<&'static str, Rect>,
}

impl Atlas {
    fn pack(sprites: Vec<(&'static str, Pix)>) -> Atlas {
        let size = 1024u32;
        let mut img = Image::gen_image_color(size as u16, size as u16, Color::new(0., 0., 0., 0.));
        let mut map = HashMap::new();
        let mut px = 1u32;
        let mut py = 1u32;
        let mut row_h = 0u32;
        for (name, p) in sprites {
            let pw = p.w as u32;
            let ph = p.h as u32;
            if px + pw + 1 > size {
                px = 1;
                py += row_h + 1;
                row_h = 0;
            }
            for y in 0..ph {
                for x in 0..pw {
                    let c = p.px[(y as i32 * p.w + x as i32) as usize];
                    if c[3] > 0 {
                        img.set_pixel(px + x, py + y, col(c));
                    }
                }
            }
            map.insert(name, Rect::new(px as f32, py as f32, pw as f32, ph as f32));
            px += pw + 1;
            row_h = row_h.max(ph);
        }
        let tex = Texture2D::from_image(&img);
        tex.set_filter(FilterMode::Nearest);
        Atlas { tex, map }
    }
    pub fn rect(&self, name: &str) -> Rect {
        *self.map.get(name).unwrap_or(&Rect::new(0., 0., 1., 1.))
    }
    /// draw with top-left anchored at x,y
    pub fn draw(&self, name: &str, x: f32, y: f32, tint: Color) {
        let r = self.rect(name);
        draw_texture_ex(
            &self.tex,
            x.round(),
            y.round(),
            tint,
            DrawTextureParams { dest_size: Some(vec2(r.w, r.h)), source: Some(r), ..Default::default() },
        );
    }
    pub fn draw_ex(&self, name: &str, x: f32, y: f32, tint: Color, flip_x: bool) {
        let r = self.rect(name);
        draw_texture_ex(
            &self.tex,
            x.round(),
            y.round(),
            tint,
            DrawTextureParams { dest_size: Some(vec2(r.w, r.h)), source: Some(r), flip_x, ..Default::default() },
        );
    }
    /// anchored bottom-center, isometric style
    pub fn draw_at(&self, name: &str, cx: f32, by: f32, tint: Color) {
        let r = self.rect(name);
        self.draw(name, cx - r.w * 0.5, by - r.h, tint);
    }
    pub fn draw_at_scaled(&self, name: &str, cx: f32, by: f32, scale: f32, tint: Color) {
        let r = self.rect(name);
        draw_texture_ex(
            &self.tex,
            (cx - r.w * 0.5 * scale).round(),
            (by - r.h * scale).round(),
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(r.w, r.h) * scale),
                source: Some(r),
                ..Default::default()
            },
        );
    }
    pub fn size(&self, name: &str) -> Vec2 {
        let r = self.rect(name);
        vec2(r.w, r.h)
    }
}

// ---------- terrain ----------
fn tile_base() -> Pix {
    Pix::new(32, 16)
}

fn grass(variant: u32) -> Pix {
    let mut p = tile_base();
    p.iso_diamond(16, 8, 16, 8, GRASS[1]);
    let mut r = Rng::new(variant + 11);
    for _ in 0..120 {
        let x = r.range(1.0, 31.0);
        let y = r.range(1.0, 15.0);
        // keep inside diamond
        if (x - 16.0).abs() / 16.0 + (y - 8.0).abs() / 8.0 <= 1.0 {
            let c = if r.f() < 0.5 { GRASS[2] } else { GRASS[3] };
            p.put(x as i32, y as i32, c);
        }
    }
    for _ in 0..(3 + variant % 3) {
        let x = r.range(6.0, 26.0) as i32;
        let y = r.range(3.0, 13.0) as i32;
        if (x - 16).abs() as f32 / 16.0 + (y - 8).abs() as f32 / 8.0 <= 0.9 {
            p.vline(x, y, y + 2, GRASS[3]);
            p.put(x - 1, y + 1, GRASS[2]);
        }
    }
    if variant == 3 {
        for _ in 0..5 {
            let x = r.range(8.0, 24.0) as i32;
            let y = r.range(5.0, 12.0) as i32;
            p.put(x, y, rgb(224, 204, 96));
            p.put(x + 1, y + 1, GRASS[3]);
        }
    }
    p
}

fn field(kind: u32) -> Pix {
    let mut p = tile_base();
    p.iso_diamond(16, 8, 16, 8, DIRT[2]);
    let a = if kind == 0 { rgb(178, 142, 66) } else { rgb(94, 136, 66) };
    let b = if kind == 0 { rgb(146, 112, 52) } else { rgb(74, 112, 54) };
    for y in 0..16i32 {
        for x in 0..32i32 {
            if (x - 16).abs() as f32 / 16.0 + (y - 8).abs() as f32 / 8.0 <= 1.0 {
                let band = ((x + y * 2) / 3) % 2 == 0;
                p.put(x, y, if band { a } else { b });
            }
        }
    }
    let mut r = Rng::new(kind + 77);
    for _ in 0..30 {
        let x = r.range(2.0, 30.0) as i32;
        let y = r.range(2.0, 14.0) as i32;
        if (x - 16).abs() as f32 / 16.0 + (y - 8).abs() as f32 / 8.0 <= 0.95 {
            p.put(x, y, DIRT[2]);
        }
    }
    p
}

fn road_tile() -> Pix {
    let mut p = tile_base();
    p.iso_diamond(16, 8, 16, 8, ROAD[1]);
    let mut r = Rng::new(303);
    for _ in 0..70 {
        let x = r.range(3.0, 29.0) as i32;
        let y = r.range(2.0, 14.0) as i32;
        if (x - 16).abs() as f32 / 16.0 + (y - 8).abs() as f32 / 8.0 <= 1.0 {
            p.put(x, y, if r.f() < 0.5 { ROAD[2] } else { ROAD[0] });
        }
    }
    // wheel ruts following both iso axes
    for t in 3..28 {
        let y1 = 8 + (t as f32 * 0.45) as i32;
        let y2 = 8 - (t as f32 * 0.45) as i32;
        p.put(t, y1.min(15), ROAD[0]);
        p.put(t, y2.max(0), ROAD[0]);
    }
    p
}

fn dirt_tile() -> Pix {
    let mut p = tile_base();
    p.iso_diamond(16, 8, 16, 8, DIRT[2]);
    let mut r = Rng::new(411);
    for _ in 0..90 {
        let x = r.range(2.0, 30.0) as i32;
        let y = r.range(2.0, 14.0) as i32;
        if (x - 16).abs() as f32 / 16.0 + (y - 8).abs() as f32 / 8.0 <= 1.0 {
            p.put(x, y, if r.f() < 0.5 { DIRT[1] } else { DIRT[3] });
        }
    }
    p
}

fn water_tile(frame: u32) -> Pix {
    let mut p = tile_base();
    p.iso_diamond(16, 8, 16, 8, WATER[1]);
    let mut r = Rng::new(500 + frame);
    for _ in 0..44 {
        let x = r.range(3.0, 29.0) as i32;
        let y = r.range(3.0, 13.0) as i32;
        if (x - 16).abs() as f32 / 16.0 + (y - 8).abs() as f32 / 8.0 <= 0.9 {
            p.hline(x, x + (2 + frame as i32), y, WATER[3]);
        }
    }
    for _ in 0..20 {
        let x = r.range(3.0, 29.0) as i32;
        let y = r.range(3.0, 13.0) as i32;
        p.put(x, y, WATER[0]);
    }
    p
}

fn forest_floor() -> Pix {
    let mut p = grass(2);
    for _ in 0..40 {
        let mut r = Rng::new(900);
        let x = r.range(4.0, 28.0) as i32;
        let y = r.range(3.0, 13.0) as i32;
        p.put(x, y, rgb(58, 48, 32));
    }
    p
}

fn oil_tile() -> Pix {
    let mut p = dirt_tile();
    p.ellipse(16, 8, 7, 3, rgb(24, 20, 18));
    p.ellipse(14, 7, 4, 2, rgb(44, 36, 28));
    p.put(13, 6, rgb(96, 120, 110));
    p.put(20, 9, rgb(96, 120, 110));
    p.line(4, 10, 8, 9, rgb(30, 24, 20));
    p.line(24, 5, 28, 6, rgb(30, 24, 20));
    p
}

fn rubble_tile() -> Pix {
    let mut p = tile_base();
    p.iso_diamond(16, 8, 16, 8, rgb(62, 56, 50));
    let mut r = Rng::new(1200);
    for _ in 0..60 {
        let x = r.range(3.0, 29.0) as i32;
        let y = r.range(3.0, 13.0) as i32;
        let c = if r.f() < 0.4 { rgb(96, 88, 78) } else { rgb(44, 40, 36) };
        p.rect(x, y, 2, 1, c);
    }
    p
}

// ---------- props ----------
fn tree_pine(seed: u32) -> Pix {
    let mut p = Pix::new(24, 34);
    p.cast_shadow(12, 30, 8, 3);
    p.rect(11, 22, 3, 8, rgb(66, 48, 32));
    p.rect(11, 22, 1, 8, rgb(88, 66, 44));
    let g = [rgb(32, 58, 38), rgb(44, 76, 46), rgb(58, 96, 56), rgb(76, 118, 66)];
    for i in 0..3 {
        let y = 6 + i * 5;
        let hw = 4 + i * 3;
        p.tri((12.0, y as f32 - 4.0), ((12 - hw) as f32, (y + 7) as f32), ((12 + hw) as f32, (y + 7) as f32), g[1]);
        p.tri((11.0, y as f32 - 4.0), ((12 - hw + 1) as f32, (y + 6) as f32), (11.0, (y + 6) as f32), g[2]);
        p.put(12, y + 6, g[3]);
    }
    p.noise_opaque(rgb(30, 52, 34), 0.12, seed);
    p.outline(rgba(10, 14, 10, 200));
    p
}

fn tree_oak(seed: u32) -> Pix {
    let mut p = Pix::new(26, 32);
    p.cast_shadow(13, 28, 9, 3);
    p.rect(12, 20, 3, 9, rgb(74, 54, 36));
    p.rect(12, 20, 1, 9, rgb(98, 74, 50));
    let g = [rgb(44, 70, 38), rgb(60, 90, 46), rgb(78, 112, 58), rgb(100, 138, 70)];
    p.disc(13, 14, 8, g[0]);
    p.disc(10, 12, 6, g[1]);
    p.disc(17, 11, 6, g[1]);
    p.disc(13, 9, 6, g[2]);
    p.disc(10, 10, 3, g[3]);
    p.noise_opaque(g[0], 0.18, seed + 3);
    p.outline(rgba(10, 14, 10, 200));
    p
}

fn bush() -> Pix {
    let mut p = Pix::new(16, 14);
    p.cast_shadow(8, 12, 6, 2);
    p.disc(6, 8, 4, GRASS[1]);
    p.disc(10, 8, 4, GRASS[1]);
    p.disc(8, 6, 4, GRASS[2]);
    p.disc(6, 6, 2, GRASS[3]);
    p.outline(rgba(10, 14, 10, 180));
    p
}

fn rock() -> Pix {
    let mut p = Pix::new(14, 12);
    p.cast_shadow(7, 10, 5, 2);
    p.quad((2., 10.), (5., 4.), (9., 3.), (12., 9.), rgb(96, 96, 90));
    p.quad((5., 4.), (9., 3.), (10., 6.), (6., 7.), rgb(124, 124, 116));
    p.outline(rgba(10, 12, 14, 190));
    p
}

fn sunflower() -> Pix {
    let mut p = Pix::new(12, 20);
    p.vline(6, 8, 19, GRASS[3]);
    p.put(5, 13, GRASS[3]);
    p.put(7, 15, GRASS[2]);
    p.disc(6, 5, 3, rgb(232, 188, 58));
    p.put(6, 5, rgb(96, 64, 30));
    p.put(5, 4, rgb(246, 214, 96));
    p.outline(rgba(10, 14, 10, 160));
    p
}

fn sandbags() -> Pix {
    let mut p = Pix::new(34, 18);
    p.cast_shadow(17, 15, 15, 3);
    for row in 0..2 {
        for i in 0..4 {
            let x = 3 + i * 7 + if row == 1 { 3 } else { 0 };
            let y = 9 - row * 4;
            p.ellipse(x, y, 4, 3, rgb(150, 134, 94));
            p.ellipse(x, y - 1, 3, 2, rgb(178, 160, 114));
            p.put(x - 2, y + 2, rgb(112, 98, 68));
        }
    }
    p.outline(rgba(20, 18, 12, 190));
    p
}

fn fence() -> Pix {
    let mut p = Pix::new(34, 16);
    p.vline(3, 4, 14, rgb(120, 100, 72));
    p.vline(16, 4, 14, rgb(120, 100, 72));
    p.vline(30, 4, 14, rgb(120, 100, 72));
    p.line(3, 6, 30, 6, rgb(92, 76, 54));
    p.line(3, 10, 30, 10, rgb(92, 76, 54));
    p.outline(rgba(20, 16, 10, 120));
    p
}

fn pole() -> Pix {
    let mut p = Pix::new(10, 30);
    p.cast_shadow(5, 27, 3, 1);
    p.vline(4, 4, 27, rgb(102, 82, 58));
    p.vline(5, 4, 27, rgb(78, 60, 42));
    p.hline(1, 8, 6, rgb(86, 68, 48));
    p.put(1, 5, rgb(140, 140, 130));
    p.put(8, 5, rgb(140, 140, 130));
    p.outline(rgba(12, 10, 8, 160));
    p
}

fn wreck() -> Pix {
    let mut p = Pix::new(26, 20);
    p.cast_shadow(13, 17, 11, 3);
    p.rect(4, 8, 18, 7, rgb(52, 46, 40));
    p.rect(7, 4, 10, 6, rgb(64, 58, 50));
    p.rect(8, 3, 3, 4, rgb(40, 36, 32));
    p.rect(14, 3, 3, 4, rgb(40, 36, 32));
    p.put(3, 14, rgb(28, 24, 22));
    p.put(22, 14, rgb(28, 24, 22));
    p.noise_opaque(rgb(124, 66, 38), 0.25, 55);
    p.outline(rgba(10, 8, 6, 200));
    p
}

fn flag(kind: u32) -> Pix {
    let mut p = Pix::new(20, 30);
    p.vline(4, 4, 29, rgb(110, 92, 68));
    p.put(4, 3, rgb(190, 180, 140));
    let (c1, c2, c3) = match kind {
        0 => (UA_BLUE, UA_YELLOW, UA_BLUE),
        1 => (RU_WHITE, RU_BLUE, RU_RED),
        _ => (GREEN_D, AMBER, RED),
    };
    for x in 5..17 {
        for y in 5..17 {
            let wave = ((x as f32 * 0.7 + y as f32 * 0.2).sin() * 0.9) as i32;
            let yy = y + wave;
            let c = if kind == 1 {
                if yy < 9 { c1 } else if yy < 13 { c2 } else { c3 }
            } else if yy < 11 {
                c1
            } else {
                c2
            };
            p.put(x, yy, c);
        }
    }
    p.outline(rgba(14, 14, 14, 190));
    p
}

// ---------- iso building helpers ----------
pub struct IsoBase {
    pub p: Pix,
    pub cx: i32,
    pub by: i32,
    pub hw: i32,
    pub hh: i32,
}

fn iso_start(w: i32, h: i32) -> IsoBase {
    let mut p = Pix::new(w, h);
    let cx = w / 2;
    let by = h - 8;
    p.cast_shadow(cx, by + 2, 16, 6);
    IsoBase { p, cx, by, hw: 14, hh: 7 }
}

fn hip_house(base: &mut IsoBase, wh: i32, roof_h: i32, wall: [C; 3], roof: [C; 4], windows: u32, door: bool, seed: u32) -> (i32, i32) {
    let (cx, by, hw, hh) = (base.cx, base.by, base.hw, base.hh);
    let top_y = by - wh;
    // roof apex and corners
    let apex = (cx as f32, (top_y - hh - roof_h) as f32);
    let lc = ((cx - hw) as f32, top_y as f32);
    let rc = ((cx + hw) as f32, top_y as f32);
    let fc = (cx as f32, (top_y + hh) as f32);
    let bc = (cx as f32, (top_y - hh) as f32);
    // back roof
    base.p.tri(bc, lc, apex, shade(roof[1], 0.7));
    base.p.tri(bc, rc, apex, shade(roof[1], 0.55));
    let p = &mut base.p;
    // walls
    p.quad(
        ((cx - hw) as f32, by as f32),
        (cx as f32, (by + hh) as f32),
        (cx as f32, (by + hh - wh) as f32),
        ((cx - hw) as f32, top_y as f32),
        wall[1],
    );
    p.quad(
        (cx as f32, (by + hh) as f32),
        ((cx + hw) as f32, by as f32),
        ((cx + hw) as f32, top_y as f32),
        (cx as f32, (by + hh - wh) as f32),
        wall[0],
    );
    // wall texture
    p.noise_opaque(shade(wall[0], 0.94), 0.10, seed);
    // windows
    for i in 0..windows {
        let wx = cx - hw + 4 + (i as i32 % 3) * 6;
        let wy = by - 4 - (i as i32 / 3) * 6;
        let lit = ((i + seed) % 3) != 0;
        let wc = if lit { rgb(238, 194, 106) } else { rgb(44, 52, 58) };
        p.quad(
            (wx as f32, wy as f32),
            ((wx + 3) as f32, (wy + 1) as f32),
            ((wx + 3) as f32, (wy + 4) as f32),
            (wx as f32, (wy + 3) as f32),
            rgb(28, 30, 34),
        );
        p.quad(
            (wx as f32, wy as f32),
            ((wx + 2) as f32, (wy + 1) as f32),
            ((wx + 2) as f32, (wy + 3) as f32),
            (wx as f32, (wy + 2) as f32),
            wc,
        );
    }
    if door {
        p.quad(
            ((cx - 2) as f32, (by + hh - 2) as f32),
            ((cx + 2) as f32, (by + hh - 4) as f32),
            ((cx + 2) as f32, (by + hh - 9) as f32),
            ((cx - 2) as f32, (by + hh - 7) as f32),
            rgb(48, 36, 26),
        );
    }
    // front roof
    p.tri(lc, fc, apex, roof[2]);
    p.tri(fc, rc, apex, roof[1]);
    // ridge highlight
    p.line(lc.0 as i32, lc.1 as i32, apex.0 as i32, apex.1 as i32, roof[3]);
    p.line(fc.0 as i32, fc.1 as i32, apex.0 as i32, apex.1 as i32, roof[2]);
    // outline
    p.outline(rgba(12, 13, 16, 200));
    (cx, top_y)
}

fn flat_box(base: &mut IsoBase, wh: i32, wall: [C; 3], roof_top: C, parapet: bool, seed: u32) {
    let (cx, by, hw, hh) = (base.cx, base.by, base.hw, base.hh);
    let top_y = by - wh;
    let p = &mut base.p;
    p.quad(
        ((cx - hw) as f32, by as f32),
        (cx as f32, (by + hh) as f32),
        (cx as f32, (by + hh - wh) as f32),
        ((cx - hw) as f32, top_y as f32),
        wall[1],
    );
    p.quad(
        (cx as f32, (by + hh) as f32),
        ((cx + hw) as f32, by as f32),
        ((cx + hw) as f32, top_y as f32),
        (cx as f32, (by + hh - wh) as f32),
        wall[0],
    );
    p.iso_diamond(cx, top_y, hw, hh, roof_top);
    if parapet {
        p.line(cx - hw, top_y, cx, top_y - hh, shade(roof_top, 1.3));
        p.line(cx, top_y - hh, cx + hw, top_y, shade(roof_top, 1.15));
        p.line(cx - hw, top_y, cx, top_y + hh, shade(roof_top, 0.8));
        p.line(cx, top_y + hh, cx + hw, top_y, shade(roof_top, 0.7));
    }
    p.noise_opaque(shade(wall[0], 0.92), 0.08, seed);
    p.outline(rgba(12, 13, 16, 200));
}

fn cylinder(p: &mut Pix, cx: i32, base_y: i32, w: i32, h: i32, light: C, mid: C, dark: C) {
    let r = w / 2;
    p.rect(cx - r, base_y - h, w, h, mid);
    p.rect(cx - r, base_y - h, 2, h, dark);
    p.rect(cx + r - 2, base_y - h, 2, h, shade(mid, 0.8));
    p.rect(cx - r + 2, base_y - h, 2, h, light);
    p.ellipse(cx, base_y - h, r, 3, light);
    p.ellipse(cx, base_y, r, 2, shade(dark, 0.8));
    for y in (base_y - h + 5..base_y).step_by(6) {
        p.hline(cx - r, cx + r - 1, y, shade(mid, 0.85));
    }
}

fn smoke_stack(p: &mut Pix, x: i32, y: i32, h: i32, c: C) {
    p.rect(x, y - h, 4, h, c);
    p.rect(x, y - h, 1, h, shade(c, 1.2));
    p.hline(x - 1, x + 4, y - h, shade(c, 0.7));
    p.hline(x - 1, x + 4, y - h + 3, shade(c, 0.7));
}

// ---------- buildings ----------
fn house(kind: u32) -> Pix {
    let (wall, roof, wh, hw_scale) = match kind {
        0 => (WALL_PLASTER, ROOF_RUST, 14, 0),
        1 => (WALL_BRICK, ROOF_GRAY, 16, 1),
        _ => (WALL_PLASTER, ROOF_GREEN, 12, 0),
    };
    let mut b = iso_start(44, 48);
    b.cx += hw_scale;
    hip_house(&mut b, wh, 11, wall, roof, 4, true, kind * 7 + 3);
    let (cx, top_y) = (b.cx, b.by - wh);
    // chimney
    b.p.rect(cx + 5, top_y - 17, 4, 9, WALL_BRICK[0]);
    b.p.rect(cx + 5, top_y - 17, 1, 9, WALL_BRICK[1]);
    b.p.hline(cx + 4, cx + 9, top_y - 17, WALL_BRICK[0]);
    b.p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn hq() -> Pix {
    let mut b = iso_start(50, 56);
    b.hw = 15;
    let wall = [rgb(126, 118, 104), rgb(166, 156, 136), rgb(200, 190, 166)];
    let (cx, top_y) = hip_house(&mut b, 24, 8, wall, ROOF_GRAY, 6, true, 21);
    // second row of windows
    for i in 0..4 {
        let wx = cx - 10 + i * 5;
        let wy = top_y + 4;
        b.p.rect(wx, wy, 2, 2, rgb(240, 200, 110));
    }
    // flagpole + flag
    b.p.vline(cx + 12, top_y - 26, top_y, rgb(170, 160, 130));
    b.p.rect(cx + 13, top_y - 26, 7, 5, UA_BLUE);
    b.p.rect(cx + 13, top_y - 21, 7, 4, UA_YELLOW);
    b.p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn shop() -> Pix {
    let mut b = iso_start(46, 48);
    b.hw = 13;
    let (cx, top_y) = hip_house(&mut b, 13, 8, WALL_PLASTER, ROOF_RUST, 2, true, 33);
    // awning stripes
    for i in 0..6 {
        let x = cx - 12 + i * 4;
        b.p.quad(
            (x as f32, (top_y + 8) as f32),
            ((x + 3) as f32, (top_y + 9) as f32),
            ((x + 3) as f32, (top_y + 12) as f32),
            (x as f32, (top_y + 11) as f32),
            if i % 2 == 0 { rgb(196, 86, 62) } else { rgb(224, 210, 180) },
        );
    }
    // sign
    b.p.rect(cx - 7, top_y + 3, 9, 2, rgb(60, 52, 40));
    b.p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn school() -> Pix {
    let mut b = iso_start(50, 54);
    b.hw = 15;
    let wall = [rgb(150, 140, 120), rgb(190, 180, 156), rgb(220, 210, 184)];
    let (cx, top_y) = hip_house(&mut b, 20, 7, wall, ROOF_RUST, 6, true, 44);
    // flag
    b.p.vline(cx + 12, top_y - 22, top_y, rgb(170, 160, 130));
    b.p.rect(cx + 13, top_y - 22, 6, 4, UA_BLUE);
    b.p.rect(cx + 13, top_y - 18, 6, 3, UA_YELLOW);
    b.p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn church() -> Pix {
    let mut b = iso_start(44, 52);
    b.hw = 12;
    let wall = [rgb(168, 160, 142), rgb(206, 198, 176), rgb(232, 224, 202)];
    let (cx, top_y) = hip_house(&mut b, 16, 6, wall, ROOF_GRAY, 2, true, 55);
    // golden dome
    b.p.disc(cx, top_y - 18, 6, GOLD[1]);
    b.p.disc(cx - 2, top_y - 20, 3, GOLD[2]);
    b.p.disc(cx, top_y - 18, 4, GOLD[2]);
    b.p.vline(cx, top_y - 30, top_y - 22, GOLD[2]);
    b.p.hline(cx - 2, cx + 2, top_y - 27, GOLD[2]);
    b.p.vline(cx, top_y - 29, top_y - 24, GOLD[2]);
    b.p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn hospital() -> Pix {
    let mut b = iso_start(48, 50);
    let wall = [rgb(158, 164, 158), rgb(196, 202, 194), rgb(224, 230, 222)];
    let (cx, top_y) = hip_house(&mut b, 18, 7, wall, ROOF_GRAY, 4, true, 66);
    // red cross on roof
    b.p.rect(cx - 5, top_y - 12, 11, 3, RED);
    b.p.rect(cx - 2, top_y - 15, 3, 9, RED);
    b.p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn tcc() -> Pix {
    let mut b = iso_start(48, 52);
    let (cx, _top_y) = hip_house(&mut b, 16, 7, WALL_BRICK, ROOF_GRAY, 4, true, 77);
    // bus in front
    b.p.rect(cx - 17, b.by - 1, 12, 6, rgb(84, 96, 72));
    b.p.rect(cx - 16, b.by - 4, 10, 3, rgb(104, 118, 88));
    b.p.rect(cx - 15, b.by - 4, 3, 2, rgb(60, 72, 90));
    b.p.rect(cx - 10, b.by - 4, 3, 2, rgb(60, 72, 90));
    b.p.put(cx - 16, b.by + 6, rgb(30, 28, 26));
    b.p.put(cx - 7, b.by + 6, rgb(30, 28, 26));
    b.p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn barracks() -> Pix {
    let mut b = iso_start(50, 48);
    b.hw = 15;
    let (cx, top_y) = hip_house(&mut b, 12, 6, WALL_GREEN, ROOF_GREEN, 3, true, 88);
    // sandbags
    for i in 0..4 {
        b.p.ellipse(cx - 12 + i * 6, b.by + 3, 4, 2, rgb(160, 146, 104));
    }
    // flag
    b.p.vline(cx + 13, top_y - 18, top_y, rgb(150, 140, 110));
    b.p.rect(cx + 14, top_y - 18, 6, 4, UA_BLUE);
    b.p.rect(cx + 14, top_y - 14, 6, 3, UA_YELLOW);
    b.p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn derrick(frame: u32) -> Pix {
    let mut b = iso_start(46, 56);
    let p = &mut b.p;
    // base pad
    p.iso_diamond(b.cx, b.by, 12, 5, DIRT[2]);
    // lattice tower
    let bx = b.cx - 6;
    let ty = b.by - 40;
    for i in 0..4 {
        let y0 = b.by - i * 9;
        let y1 = b.by - (i + 1) * 9;
        let ins = i * 2;
        p.line(bx + 2 + ins, y0, bx + 10 - ins, y0, METAL[2]);
        p.line(bx + 2 + ins, y0, bx + 4 + ins, y1, METAL[2]);
        p.line(bx + 10 - ins, y0, bx + 8 - ins, y1, METAL[1]);
        p.line(bx + 4 + ins, y1, bx + 8 - ins, y1, METAL[2]);
    }
    p.hline(bx, bx + 12, ty, METAL[3]);
    // pumpjack arm
    let pivot = (b.cx, b.by - 14);
    let ang: f32 = if frame == 0 { -0.6 } else { 0.35 };
    let dx = (ang.cos() * 15.0) as i32;
    let dy = (ang.sin() * 8.0) as i32;
    p.line(pivot.0, pivot.1, pivot.0 + dx, pivot.1 + dy, METAL[3]);
    p.line(pivot.0, pivot.1 - 1, pivot.0 + dx, pivot.1 + dy - 1, METAL[4]);
    p.disc(pivot.0 + dx, pivot.1 + dy, 3, METAL[2]);
    p.disc(pivot.0 + dx, pivot.1 + dy, 1, rgb(210, 190, 120));
    p.disc(pivot.0, pivot.1, 3, METAL[1]);
    // wellhead
    p.rect(b.cx - 2, b.by - 8, 5, 8, METAL[1]);
    p.rect(b.cx - 2, b.by - 8, 1, 8, METAL[3]);
    p.rect(b.cx - 5, b.by - 3, 11, 3, METAL[0]);
    // small oil tank
    cylinder(p, b.cx + 12, b.by + 1, 11, 8, METAL[3], METAL[2], METAL[1]);
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn refinery() -> Pix {
    let mut b = iso_start(52, 60);
    b.hw = 15;
    flat_box(&mut b, 8, [rgb(70, 66, 60), rgb(92, 88, 80), rgb(116, 112, 102)], CONCRETE[1], false, 99);
    let p = &mut b.p;
    let cx = b.cx;
    let by = b.by;
    // tanks
    cylinder(p, cx - 12, by + 2, 16, 20, METAL[3], METAL[2], METAL[1]);
    cylinder(p, cx + 8, by + 5, 13, 16, METAL[3], METAL[2], METAL[1]);
    // fractioning column
    p.rect(cx - 2, by - 34, 7, 30, METAL[2]);
    p.rect(cx - 2, by - 34, 2, 30, METAL[4]);
    p.rect(cx + 3, by - 34, 1, 30, METAL[0]);
    for i in 0..4 {
        p.hline(cx - 3, cx + 6, by - 30 + i * 7, METAL[1]);
    }
    p.hline(cx - 3, cx + 6, by - 34, METAL[3]);
    // pipes
    p.line(cx - 2, by - 8, cx - 12, by - 5, METAL[1]);
    p.line(cx + 5, by - 8, cx + 8, by - 4, METAL[1]);
    // flare stack
    p.rect(cx + 18, by - 30, 3, 30, METAL[1]);
    p.rect(cx + 18, by - 30, 1, 30, METAL[3]);
    p.disc(cx + 19, by - 33, 3, FIRE[2]);
    p.disc(cx + 19, by - 34, 2, FIRE[3]);
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn diesel() -> Pix {
    let mut b = iso_start(46, 46);
    flat_box(&mut b, 10, [rgb(66, 72, 62), rgb(88, 96, 82), rgb(112, 120, 104)], METAL[2], false, 111);
    let p = &mut b.p;
    // vent
    p.rect(b.cx + 6, b.by - 26, 3, 14, METAL[1]);
    p.rect(b.cx + 6, b.by - 26, 1, 14, METAL[3]);
    p.hline(b.cx + 5, b.cx + 10, b.by - 26, METAL[0]);
    // fuel tank
    cylinder(p, b.cx - 12, b.by + 2, 10, 8, METAL[3], METAL[2], METAL[1]);
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn tes() -> Pix {
    let mut b = iso_start(52, 58);
    flat_box(&mut b, 12, [rgb(74, 70, 66), rgb(98, 94, 88), rgb(124, 120, 112)], CONCRETE[1], false, 121);
    let p = &mut b.p;
    let cx = b.cx;
    let by = b.by;
    // two chimneys
    for i in 0..2 {
        let x = cx - 9 + i * 15;
        p.rect(x, by - 40, 6, 32, CONCRETE[1]);
        p.rect(x, by - 40, 2, 32, CONCRETE[2]);
        p.rect(x + 5, by - 40, 1, 32, CONCRETE[0]);
        p.hline(x - 1, x + 6, by - 40, CONCRETE[0]);
    }
    // turbine hall windows
    for i in 0..4 {
        p.rect(cx - 4 + i * 4, by - 9, 2, 3, rgb(238, 196, 110));
    }
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn solar() -> Pix {
    let mut b = iso_start(46, 42);
    let p = &mut b.p;
    p.iso_diamond(b.cx, b.by, 14, 6, DIRT[2]);
    for row in 0..2 {
        for i in 0..2 {
            let x = b.cx - 13 + i * 14;
            let y = b.by - 6 + row * 7;
            p.quad(
                (x as f32, y as f32),
                ((x + 10) as f32, (y - 4) as f32),
                ((x + 14) as f32, (y - 1) as f32),
                ((x + 4) as f32, (y + 3) as f32),
                rgb(38, 56, 96),
            );
            p.quad(
                (x as f32, y as f32),
                ((x + 4) as f32, (y + 3) as f32),
                ((x + 4) as f32, (y + 5) as f32),
                (x as f32, (y + 2) as f32),
                rgb(52, 72, 118),
            );
            p.line(x, y, x + 4, y + 3, rgb(86, 116, 170));
        }
    }
    p.outline(rgba(12, 13, 16, 190));
    b.p
}

fn substation() -> Pix {
    let mut b = iso_start(40, 44);
    let p = &mut b.p;
    p.iso_diamond(b.cx, b.by, 13, 6, DIRT[2]);
    // transformer
    p.rect(b.cx - 8, b.by - 12, 14, 10, METAL[2]);
    p.rect(b.cx - 8, b.by - 12, 2, 10, METAL[3]);
    p.rect(b.cx - 8, b.by - 12, 14, 2, METAL[1]);
    for i in 0..3 {
        p.vline(b.cx - 5 + i * 4, b.by - 17, b.by - 12, CONCRETE[3]);
    }
    p.hline(b.cx - 5, b.cx + 3, b.by - 17, METAL[1]);
    // pole
    p.vline(b.cx + 12, b.by - 26, b.by, rgb(102, 82, 58));
    p.hline(b.cx + 7, b.cx + 17, b.by - 24, rgb(86, 68, 48));
    p.outline(rgba(12, 13, 16, 190));
    b.p
}

fn farm() -> Pix {
    let mut b = iso_start(50, 48);
    b.hw = 14;
    let (cx, top_y) = hip_house(&mut b, 14, 10, WALL_BRICK, ROOF_RUST, 2, true, 133);
    // silo
    cylinder(&mut b.p, cx + 13, b.by + 1, 10, 18, METAL[3], METAL[2], METAL[1]);
    b.p.tri(
        ((cx + 8) as f32, (b.by - 17) as f32),
        ((cx + 18) as f32, (b.by - 17) as f32),
        ((cx + 13) as f32, (b.by - 23) as f32),
        METAL[2],
    );
    let _ = top_y;
    b.p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn water_tower() -> Pix {
    let mut b = iso_start(40, 48);
    let p = &mut b.p;
    let cx = b.cx;
    let by = b.by;
    p.line(cx - 7, by, cx - 3, by - 22, METAL[1]);
    p.line(cx + 7, by, cx + 3, by - 22, METAL[1]);
    p.line(cx - 1, by, cx - 2, by - 22, METAL[0]);
    p.line(cx - 7, by - 10, cx + 7, by - 10, METAL[0]);
    p.line(cx - 7, by, cx + 7, by - 12, METAL[0]);
    cylinder(p, cx, by - 20, 16, 11, METAL[4], METAL[3], METAL[1]);
    p.rect(cx - 3, by - 10, 6, 6, METAL[1]);
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn metal_shop() -> Pix {
    let mut b = iso_start(46, 46);
    flat_box(&mut b, 10, [rgb(68, 62, 54), rgb(92, 84, 72), rgb(118, 108, 92)], ROOF_GRAY[1], false, 144);
    let p = &mut b.p;
    smoke_stack(p, b.cx + 8, b.by - 8, 22, METAL[1]);
    // scrap pile
    p.quad(
        ((b.cx - 17) as f32, (b.by + 3) as f32),
        ((b.cx - 9) as f32, (b.by + 7) as f32),
        ((b.cx - 5) as f32, (b.by + 2) as f32),
        ((b.cx - 12) as f32, (b.by - 2) as f32),
        METAL[2],
    );
    p.put(b.cx - 9, b.by + 2, METAL[4]);
    p.put(b.cx - 11, b.by + 3, METAL[1]);
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn tank_factory() -> Pix {
    let mut b = iso_start(56, 56);
    b.hw = 17;
    flat_box(&mut b, 16, [rgb(70, 74, 70), rgb(94, 100, 94), rgb(120, 126, 118)], METAL[2], true, 155);
    let p = &mut b.p;
    let cx = b.cx;
    let by = b.by;
    // hangar door
    p.rect(cx - 7, by - 9, 14, 9, rgb(26, 30, 30));
    for i in 0..7 {
        p.vline(cx - 7 + i * 2, by - 9, by, rgb(44, 48, 46));
    }
    p.hline(cx - 8, cx + 7, by - 10, METAL[0]);
    // tank silhouette in door
    p.rect(cx - 4, by - 5, 7, 4, rgb(48, 58, 42));
    p.rect(cx - 1, by - 7, 4, 3, rgb(40, 50, 36));
    p.hline(cx + 2, cx + 7, by - 6, rgb(40, 46, 38));
    // chimney
    smoke_stack(p, cx + 12, by - 18, 26, METAL[1]);
    // roof vents
    p.rect(cx - 4, by - 22, 3, 4, METAL[1]);
    p.rect(cx + 2, by - 21, 3, 4, METAL[1]);
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn drone_hangar() -> Pix {
    let mut b = iso_start(48, 46);
    let p = &mut b.p;
    // quonset hut
    p.ellipse(b.cx, b.by - 6, 16, 10, METAL[2]);
    p.rect(b.cx - 16, b.by - 8, 32, 8, METAL[1]);
    for i in 0..5 {
        p.line(b.cx - 16 + i * 6, b.by - 16, b.cx - 16 + i * 6, b.by - 2, METAL[1]);
    }
    p.ellipse(b.cx, b.by - 13, 16, 5, METAL[3]);
    p.rect(b.cx - 5, b.by - 8, 10, 8, rgb(30, 34, 36));
    // antenna
    p.vline(b.cx + 13, b.by - 28, b.by - 12, METAL[1]);
    p.line(b.cx + 13, b.by - 28, b.cx + 18, b.by - 32, METAL[3]);
    p.disc(b.cx + 18, b.by - 32, 2, RED);
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn repair() -> Pix {
    let mut b = iso_start(48, 48);
    flat_box(&mut b, 9, [rgb(72, 68, 60), rgb(96, 92, 82), rgb(122, 118, 106)], ROOF_GRAY[1], false, 166);
    let p = &mut b.p;
    let cx = b.cx;
    let by = b.by;
    p.rect(cx - 7, by - 8, 14, 8, rgb(30, 32, 32));
    // crane
    p.vline(cx + 12, by - 30, by, AMBER_D);
    p.hline(cx + 2, cx + 16, by - 30, AMBER_D);
    p.vline(cx + 2, by - 30, by - 22, METAL[1]);
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn bunker() -> Pix {
    let mut b = iso_start(44, 40);
    b.hw = 12;
    flat_box(&mut b, 8, [rgb(78, 78, 70), rgb(102, 102, 92), rgb(128, 128, 116)], CONCRETE[0], false, 177);
    let p = &mut b.p;
    // firing slit
    p.rect(b.cx - 6, b.by - 5, 12, 2, rgb(18, 20, 18));
    // sandbags
    for i in 0..4 {
        p.ellipse(b.cx - 12 + i * 7, b.by + 3, 4, 2, rgb(150, 136, 96));
    }
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn aa() -> Pix {
    let mut b = iso_start(42, 46);
    let p = &mut b.p;
    // truck base
    p.rect(b.cx - 13, b.by - 6, 26, 6, WALL_GREEN[0]);
    p.rect(b.cx - 13, b.by - 8, 9, 5, WALL_GREEN[1]);
    p.rect(b.cx - 11, b.by - 8, 3, 2, rgb(50, 70, 84));
    for i in 0..3 {
        p.disc(b.cx - 10 + i * 9, b.by, 2, rgb(26, 28, 26));
    }
    // launcher
    p.rect(b.cx - 4, b.by - 17, 12, 8, METAL[1]);
    for i in 0..2 {
        for j in 0..2 {
            p.line(b.cx - 2 + i * 5, b.by - 15 + j * 3, b.cx + 9 + i * 5, b.by - 22 + j * 3, METAL[3]);
            p.put(b.cx + 9 + i * 5, b.by - 22 + j * 3, RED);
        }
    }
    // radar
    p.vline(b.cx + 9, b.by - 20, b.by - 9, METAL[2]);
    p.disc(b.cx + 9, b.by - 21, 3, METAL[3]);
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn ew() -> Pix {
    let mut b = iso_start(42, 46);
    let p = &mut b.p;
    p.rect(b.cx - 13, b.by - 7, 26, 7, WALL_GREEN[0]);
    p.rect(b.cx - 12, b.by - 11, 8, 6, WALL_GREEN[1]);
    for i in 0..3 {
        p.disc(b.cx - 10 + i * 9, b.by, 2, rgb(26, 28, 26));
    }
    // antenna array
    p.vline(b.cx + 3, b.by - 30, b.by - 7, METAL[1]);
    for i in 0..4 {
        p.hline(b.cx - 2, b.cx + 8, b.by - 26 + i * 5, METAL[3]);
    }
    p.disc(b.cx + 3, b.by - 32, 2, RED);
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn radar() -> Pix {
    let mut b = iso_start(42, 46);
    let p = &mut b.p;
    p.rect(b.cx - 9, b.by - 8, 18, 8, CONCRETE[1]);
    p.vline(b.cx, b.by - 26, b.by - 8, METAL[1]);
    // dish
    p.ellipse(b.cx + 2, b.by - 28, 9, 5, METAL[3]);
    p.ellipse(b.cx + 2, b.by - 28, 6, 3, METAL[1]);
    p.line(b.cx + 2, b.by - 28, b.cx + 10, b.by - 32, METAL[4]);
    p.disc(b.cx + 10, b.by - 32, 2, RED);
    p.outline(rgba(12, 13, 16, 200));
    b.p
}

fn trench() -> Pix {
    let mut p = Pix::new(34, 16);
    p.cast_shadow(17, 12, 14, 3);
    p.quad((3., 10.), (14., 14.), (30., 7.), (19., 4.), rgb(52, 44, 32));
    for i in 0..5 {
        p.ellipse(6 + i * 5, 9 - (i % 2), 4, 2, rgb(134, 120, 86));
        p.ellipse(6 + i * 5, 8 - (i % 2), 3, 1, rgb(160, 146, 108));
    }
    p.outline(rgba(14, 12, 8, 160));
    p
}

fn mines() -> Pix {
    let mut p = Pix::new(32, 16);
    p.cast_shadow(16, 12, 12, 3);
    for i in 0..3 {
        let x = 5 + i * 9;
        let y = 9 - (i % 2) * 2;
        p.vline(x, y - 5, y, rgb(120, 100, 70));
        p.tri((x as f32, (y - 8) as f32), ((x - 3) as f32, (y - 4) as f32), ((x + 3) as f32, (y - 4) as f32), RED);
        p.put(x, y - 6, rgb(240, 230, 210));
    }
    p
}

// ---------- units ----------
fn soldier(variant: u32) -> Pix {
    let mut p = Pix::new(24, 20);
    p.cast_shadow(12, 17, 8, 2);
    let uni = if variant == 0 { [rgb(74, 84, 54), rgb(94, 106, 68), rgb(56, 64, 42)] } else { [rgb(96, 92, 60), rgb(118, 112, 74), rgb(72, 68, 46)] };
    for i in 0..3 {
        let x = 5 + i * 6;
        let y = 11 - (i % 2) * 2;
        p.rect(x, y, 3, 6, uni[0]);
        p.rect(x, y - 3, 3, 3, rgb(58, 56, 46));
        p.put(x + 1, y - 4, rgb(84, 82, 66));
        p.put(x + 2, y + 1, uni[1]);
        p.line(x + 3, y + 1, x + 7, y - 1, rgb(26, 28, 26));
        p.put(x, y + 5, rgb(28, 26, 22));
        p.put(x + 2, y + 5, rgb(28, 26, 22));
    }
    p.outline(rgba(10, 12, 10, 210));
    p
}

fn tank(kind: u32) -> Pix {
    let mut p = Pix::new(26, 22);
    p.cast_shadow(13, 18, 11, 3);
    let (hull, tur, dark) = if kind == 0 {
        (rgb(78, 88, 54), rgb(92, 104, 62), rgb(46, 54, 34))
    } else {
        (rgb(60, 66, 50), rgb(74, 82, 58), rgb(38, 42, 32))
    };
    // tracks
    p.rect(3, 12, 20, 5, rgb(38, 40, 34));
    for i in 0..5 {
        p.put(4 + i * 4, 14, rgb(20, 22, 18));
    }
    // hull
    p.quad((4., 13.), (22., 10.), (24., 5.), (6., 8.), hull);
    p.quad((6., 8.), (24., 5.), (23., 3.), (7., 6.), rgb(110, 116, 74));
    // turret
    p.quad((9., 6.), (19., 4.), (18., 0.), (10., 2.), tur);
    p.rect(11, 1, 4, 2, tur);
    // barrel
    p.line(16, 3, 25, 0, dark);
    p.line(16, 4, 25, 1, rgb(52, 56, 40));
    // marking
    if kind == 0 {
        p.rect(7, 10, 4, 1, UA_BLUE);
        p.rect(7, 11, 4, 1, UA_YELLOW);
    } else {
        p.rect(7, 10, 4, 3, RU_WHITE);
        p.rect(8, 10, 2, 3, RU_RED);
    }
    p.outline(rgba(10, 12, 10, 220));
    p
}

fn ifv() -> Pix {
    let mut p = Pix::new(24, 20);
    p.cast_shadow(12, 16, 10, 3);
    p.rect(4, 11, 17, 4, rgb(34, 36, 30));
    for i in 0..4 {
        p.disc(5 + i * 5, 14, 2, rgb(22, 24, 20));
    }
    p.quad((4., 12.), (21., 9.), (22., 5.), (5., 8.), rgb(76, 84, 56));
    p.quad((5., 8.), (22., 5.), (21., 3.), (6., 6.), rgb(98, 106, 68));
    p.rect(9, 3, 6, 3, rgb(58, 64, 44));
    p.line(14, 4, 23, 2, rgb(40, 44, 32));
    p.rect(6, 9, 3, 1, UA_BLUE);
    p.outline(rgba(10, 12, 10, 220));
    p
}

fn mlrs() -> Pix {
    let mut p = Pix::new(26, 22);
    p.cast_shadow(13, 18, 11, 3);
    p.rect(4, 12, 18, 5, rgb(56, 60, 46));
    for i in 0..3 {
        p.disc(6 + i * 7, 16, 2, rgb(24, 26, 22));
    }
    p.rect(5, 9, 8, 5, rgb(70, 78, 54));
    p.rect(6, 10, 2, 2, rgb(50, 66, 80));
    // rocket pack
    for i in 0..3 {
        p.line(11, 10 + i, 23, 5 + i, rgb(52, 56, 42));
    }
    for i in 0..3 {
        p.put(23, 5 + i, rgb(30, 32, 28));
    }
    p.outline(rgba(10, 12, 10, 220));
    p
}

fn truck() -> Pix {
    let mut p = Pix::new(24, 18);
    p.cast_shadow(12, 15, 10, 2);
    p.rect(5, 9, 11, 5, rgb(76, 82, 58));
    p.rect(3, 8, 7, 6, rgb(88, 96, 66));
    p.rect(4, 9, 3, 2, rgb(50, 66, 80));
    for i in 0..3 {
        p.disc(6 + i * 6, 15, 2, rgb(24, 26, 22));
    }
    p.rect(8, 7, 8, 2, rgb(108, 116, 74));
    p.outline(rgba(10, 12, 10, 200));
    p
}

fn drone_quad(frame: u32) -> Pix {
    let mut p = Pix::new(20, 12);
    p.cast_shadow(10, 10, 6, 2);
    p.rect(8, 4, 5, 3, rgb(46, 52, 54));
    p.put(8, 6, RED);
    let rot = frame % 2;
    let off = if rot == 0 { 2 } else { 1 };
    p.line(3, 5, 8, 5, rgb(80, 90, 96));
    p.line(12, 5, 17, 5, rgb(80, 90, 96));
    p.line(5, 7, 8, 9, rgb(64, 72, 76));
    p.line(15, 7, 12, 9, rgb(64, 72, 76));
    p.ellipse(2, 5, 3, 1, rgb(150, 160, 160));
    p.ellipse(18, 5, 3, 1, rgb(150, 160, 160));
    p.put(10 + off, 3, rgb(220, 220, 210));
    p.outline(rgba(10, 12, 10, 170));
    p
}

fn fpv() -> Pix {
    let mut p = Pix::new(16, 12);
    p.cast_shadow(8, 10, 5, 1);
    p.rect(6, 4, 4, 4, rgb(52, 56, 54));
    p.line(2, 5, 6, 5, rgb(90, 96, 92));
    p.line(10, 5, 14, 5, rgb(90, 96, 92));
    p.line(2, 6, 6, 6, rgb(70, 76, 72));
    p.line(10, 6, 14, 6, rgb(70, 76, 72));
    p.put(7, 7, RED);
    p.outline(rgba(10, 12, 10, 190));
    p
}

fn shahed() -> Pix {
    let mut p = Pix::new(22, 12);
    p.cast_shadow(11, 10, 7, 2);
    p.quad((2., 6.), (12., 4.), (20., 5.), (12., 8.), rgb(72, 76, 64));
    p.quad((3., 6.), (12., 5.), (12., 7.), (4., 8.), rgb(56, 60, 50));
    p.rect(9, 2, 3, 3, rgb(60, 64, 54));
    p.put(13, 5, RED);
    p.outline(rgba(10, 12, 10, 190));
    p
}

fn lancet() -> Pix {
    let mut p = Pix::new(18, 10);
    p.cast_shadow(9, 8, 5, 1);
    p.quad((2., 5.), (9., 3.), (16., 5.), (9., 7.), rgb(70, 74, 62));
    p.rect(7, 3, 3, 2, rgb(52, 56, 46));
    p.rect(4, 4, 2, 1, rgb(40, 44, 36));
    p.rect(12, 4, 2, 1, rgb(40, 44, 36));
    p.outline(rgba(10, 12, 10, 180));
    p
}

fn arty() -> Pix {
    let mut p = Pix::new(26, 20);
    p.cast_shadow(13, 16, 11, 3);
    p.rect(5, 12, 14, 4, rgb(60, 64, 48));
    p.rect(4, 14, 18, 2, rgb(34, 36, 30));
    for i in 0..2 {
        p.disc(8 + i * 8, 17, 3, rgb(26, 28, 24));
    }
    p.rect(7, 8, 9, 5, rgb(76, 82, 56));
    p.line(12, 10, 25, 2, rgb(52, 56, 42));
    p.line(12, 11, 25, 3, rgb(40, 44, 34));
    p.outline(rgba(10, 12, 10, 200));
    p
}

// ---------- fx ----------
fn boom(frame: u32) -> Pix {
    let mut p = Pix::new(48, 48);
    let r = 6 + frame as i32 * 5;
    p.disc(24, 24, r, FIRE[1]);
    p.disc(24, 24, (r - 3).max(1), FIRE[2]);
    p.disc(24, 24, (r - 7).max(1), FIRE[3]);
    let mut rng = Rng::new(700 + frame);
    for _ in 0..14 {
        let a = rng.range(0., 6.28);
        let d = rng.range(r as f32 * 0.5, r as f32 * 1.2);
        let x = 24 + (a.cos() * d) as i32;
        let y = 24 + (a.sin() * d) as i32;
        let c = if rng.f() < 0.5 { FIRE[3] } else { FIRE[2] };
        p.rect(x, y, 2, 2, c);
    }
    p
}

fn smoke_puff(frame: u32) -> Pix {
    let mut p = Pix::new(20, 20);
    let col = [rgb(64, 64, 62), rgb(86, 86, 82), rgb(108, 108, 102)];
    let mut rng = Rng::new(800 + frame);
    for _ in 0..5 {
        p.disc(10 + rng.range(-3., 3.) as i32, 12 + rng.range(-3., 2.) as i32, 3 + rng.range(0., 2.) as i32, col[frame as usize % 3]);
    }
    p
}

fn fire_puff(frame: u32) -> Pix {
    let mut p = Pix::new(14, 18);
    p.disc(7, 12, 4, FIRE[1]);
    p.disc(7, 9, 3, FIRE[2]);
    p.tri((4., 10.), (10., 10.), (7., if frame == 0 { 1. } else { 4. }), FIRE[3]);
    p
}

fn spark() -> Pix {
    let mut p = Pix::new(6, 6);
    p.put(3, 2, FIRE[3]);
    p.put(3, 3, FIRE[2]);
    p.put(2, 3, FIRE[3]);
    p.put(4, 3, FIRE[3]);
    p
}

fn marker() -> Pix {
    let mut p = Pix::new(18, 10);
    p.ellipse(9, 5, 8, 4, rgba(214, 68, 48, 60));
    p.ellipse(9, 5, 6, 3, rgba(214, 68, 48, 90));
    p.ellipse(9, 5, 3, 2, rgba(244, 110, 72, 140));
    p
}

// ---------- icons ----------
fn icon(kind: &str) -> Pix {
    let mut p = Pix::new(14, 14);
    match kind {
        "fuel" => {
            p.rect(3, 4, 8, 9, rgb(64, 58, 48));
            p.rect(4, 3, 6, 2, rgb(64, 58, 48));
            p.rect(5, 2, 2, 2, rgb(90, 82, 66));
            p.rect(5, 6, 5, 5, rgb(214, 158, 52));
            p.rect(5, 6, 2, 2, rgb(244, 202, 96));
            p.hline(6, 9, 10, rgb(140, 98, 34));
        }
        "power" => {
            p.tri((8., 1.), (3., 8.), (7., 8.), rgb(240, 208, 84));
            p.tri((7., 5.), (11., 5.), (5., 13.), rgb(240, 208, 84));
            p.put(5, 7, rgb(255, 240, 160));
        }
        "food" => {
            p.ellipse(7, 8, 5, 4, rgb(194, 148, 74));
            p.ellipse(7, 6, 4, 3, rgb(222, 180, 100));
            p.line(4, 5, 9, 10, rgb(150, 108, 50));
            p.put(5, 5, rgb(240, 210, 140));
        }
        "water" => {
            p.tri((7., 1.), (2., 8.), (12., 8.), rgb(84, 152, 214));
            p.disc(7, 9, 4, rgb(84, 152, 214));
            p.disc(6, 8, 2, rgb(140, 200, 244));
            p.put(5, 8, rgb(210, 240, 255));
        }
        "money" => {
            p.disc(7, 7, 6, rgb(196, 148, 48));
            p.disc(7, 7, 4, rgb(232, 190, 82));
            p.rect(6, 4, 2, 7, rgb(140, 100, 32));
            p.hline(5, 8, 7, rgb(140, 100, 32));
        }
        "steel" => {
            p.disc(7, 7, 6, rgb(118, 126, 136));
            p.disc(7, 7, 3, rgb(60, 66, 74));
            for i in 0..4 {
                let a = i as f32 * 1.57;
                p.put(7 + (a.cos() * 5.) as i32, 7 + (a.sin() * 5.) as i32, rgb(158, 168, 178));
            }
        }
        "people" => {
            p.disc(5, 5, 2, rgb(206, 184, 148));
            p.rect(3, 7, 4, 5, rgb(92, 122, 148));
            p.disc(10, 6, 2, rgb(206, 184, 148));
            p.rect(8, 8, 4, 4, rgb(78, 106, 132));
        }
        "ammo" => {
            for i in 0..3 {
                let x = 3 + i * 3;
                p.rect(x, 5, 2, 6, rgb(194, 162, 90));
                p.tri((x as f32, 5.), ((x + 2) as f32, 5.), ((x + 1) as f32, 2.), rgb(150, 120, 66));
            }
        }
        "morale" => {
            p.disc(5, 5, 3, rgb(200, 72, 66));
            p.disc(9, 5, 3, rgb(200, 72, 66));
            p.tri((2., 6.), (12., 6.), (7., 13.), rgb(200, 72, 66));
            p.disc(5, 4, 1, rgb(240, 128, 118));
        }
        "shield" => {
            p.quad((7., 1.), (12., 3.), (11., 9.), (7., 13.), rgb(96, 132, 168));
            p.quad((7., 1.), (2., 3.), (3., 9.), (7., 13.), rgb(124, 160, 192));
            p.line(7, 3, 7, 11, rgb(180, 204, 224));
        }
        "home" => {
            p.tri((7., 1.), (1., 7.), (13., 7.), rgb(168, 96, 64));
            p.rect(3, 7, 8, 6, rgb(196, 178, 146));
            p.rect(6, 9, 3, 4, rgb(80, 56, 36));
        }
        "dronei" => {
            p.rect(6, 5, 3, 4, rgb(64, 70, 72));
            p.hline(2, 5, 4, rgb(130, 140, 144));
            p.hline(9, 12, 4, rgb(130, 140, 144));
            p.ellipse(2, 4, 2, 1, rgb(190, 198, 200));
            p.ellipse(12, 4, 2, 1, rgb(190, 198, 200));
            p.put(7, 9, RED);
        }
        "clock" => {
            p.disc(7, 7, 6, rgb(196, 190, 170));
            p.disc(7, 7, 5, rgb(60, 62, 62));
            p.vline(7, 4, 8, rgb(220, 216, 196));
            p.line(7, 7, 10, 8, rgb(220, 216, 196));
        }
        "star" => {
            p.line(7, 1, 7, 13, rgb(240, 206, 96));
            p.line(2, 5, 12, 5, rgb(240, 206, 96));
            p.line(3, 12, 11, 3, rgb(214, 176, 70));
            p.line(3, 3, 11, 12, rgb(214, 176, 70));
        }
        _ => {}
    }
    p.outline(rgba(8, 10, 12, 160));
    p
}

// ---------- building assembly ----------
pub fn build_atlas() -> Atlas {
    let mut s: Vec<(&'static str, Pix)> = Vec::new();
    s.push(("grass0", grass(0)));
    s.push(("grass1", grass(1)));
    s.push(("grass2", grass(2)));
    s.push(("grass3", grass(3)));
    s.push(("field0", field(0)));
    s.push(("field1", field(1)));
    s.push(("road", road_tile()));
    s.push(("dirt", dirt_tile()));
    s.push(("water0", water_tile(0)));
    s.push(("water1", water_tile(1)));
    s.push(("forest", forest_floor()));
    s.push(("oil", oil_tile()));
    s.push(("rubble", rubble_tile()));
    s.push(("tree0", tree_pine(1)));
    s.push(("tree1", tree_oak(2)));
    s.push(("tree2", tree_pine(3)));
    s.push(("bush", bush()));
    s.push(("rock", rock()));
    s.push(("sunflower", sunflower()));
    s.push(("sandbags", sandbags()));
    s.push(("fence", fence()));
    s.push(("pole", pole()));
    s.push(("wreck", wreck()));
    s.push(("flag0", flag(0)));
    s.push(("flag1", flag(1)));
    s.push(("flag2", flag(2)));
    s.push(("house0", house(0)));
    s.push(("house1", house(1)));
    s.push(("house2", house(2)));
    s.push(("hq", hq()));
    s.push(("shop", shop()));
    s.push(("school", school()));
    s.push(("church", church()));
    s.push(("hospital", hospital()));
    s.push(("tcc", tcc()));
    s.push(("barracks", barracks()));
    s.push(("derrick0", derrick(0)));
    s.push(("derrick1", derrick(1)));
    s.push(("refinery", refinery()));
    s.push(("diesel", diesel()));
    s.push(("tes", tes()));
    s.push(("solar", solar()));
    s.push(("substation", substation()));
    s.push(("farm", farm()));
    s.push(("watertower", water_tower()));
    s.push(("metalshop", metal_shop()));
    s.push(("tankfactory", tank_factory()));
    s.push(("dronehangar", drone_hangar()));
    s.push(("repair", repair()));
    s.push(("bunker", bunker()));
    s.push(("aa", aa()));
    s.push(("ew", ew()));
    s.push(("radar", radar()));
    s.push(("trench", trench()));
    s.push(("mines", mines()));
    s.push(("soldier0", soldier(0)));
    s.push(("soldier1", soldier(1)));
    s.push(("tank0", tank(0)));
    s.push(("tank1", tank(1)));
    s.push(("ifv", ifv()));
    s.push(("mlrs", mlrs()));
    s.push(("truck", truck()));
    s.push(("drone0", drone_quad(0)));
    s.push(("drone1", drone_quad(1)));
    s.push(("fpv", fpv()));
    s.push(("shahed", shahed()));
    s.push(("lancet", lancet()));
    s.push(("arty", arty()));
    s.push(("boom0", boom(0)));
    s.push(("boom1", boom(1)));
    s.push(("boom2", boom(2)));
    s.push(("boom3", boom(3)));
    s.push(("smoke0", smoke_puff(0)));
    s.push(("smoke1", smoke_puff(1)));
    s.push(("smoke2", smoke_puff(2)));
    s.push(("fire0", fire_puff(0)));
    s.push(("fire1", fire_puff(1)));
    s.push(("spark", spark()));
    s.push(("marker", marker()));
    for k in [
        "fuel", "power", "food", "water", "money", "steel", "people", "ammo", "morale", "shield", "home", "dronei", "clock", "star",
    ] {
        s.push((k, icon(k)));
    }
    Atlas::pack(s)
}

