mod art;
mod fx;
mod game;
mod ui;
mod world;

use art::Atlas;
use macroquad::prelude::*;

pub const VW: f32 = 640.0;
pub const VH: f32 = 360.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "СЕЛО ЗА 3 ДНЯ".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        high_dpi: false,
        sample_count: 0,
        ..Default::default()
    }
}

fn view_scale() -> (f32, f32, f32) {
    let sw = screen_width();
    let sh = screen_height();
    let mut scale = (sw / VW).min(sh / VH);
    if scale >= 1.0 {
        scale = scale.floor();
    }
    let ox = ((sw - VW * scale) * 0.5).floor();
    let oy = ((sh - VH * scale) * 0.5).floor();
    (scale, ox, oy)
}

pub(crate) struct Gfx {
    rt: RenderTarget,
    pub(crate) atlas: Atlas,
    pub(crate) font: Font,
}

impl Gfx {
    fn begin(&self) {
        set_camera(&Camera2D {
            render_target: Some(self.rt.clone()),
            zoom: vec2(2.0 / VW, -2.0 / VH),
            target: vec2(VW * 0.5, VH * 0.5),
            ..Default::default()
        });
    }

    fn end(&self) {
        set_default_camera();
        let (scale, ox, oy) = view_scale();
        clear_background(Color::new(0.015, 0.015, 0.02, 1.0));
        draw_texture_ex(
            &self.rt.texture,
            ox,
            oy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(VW * scale, VH * scale)),
                flip_y: true,
                ..Default::default()
            },
        );
    }

    pub(crate) fn mouse(&self) -> Vec2 {
        let (scale, ox, oy) = view_scale();
        let (mx, my) = mouse_position();
        vec2((mx - ox) / scale, (my - oy) / scale)
    }

    pub(crate) fn text(&self, s: &str, x: f32, y: f32, size: u16, color: Color) {
        draw_text_ex(
            s,
            x,
            y,
            TextParams { font: Some(&self.font), font_size: size, color, ..Default::default() },
        );
    }

    pub(crate) fn text_center(&self, s: &str, cx: f32, y: f32, size: u16, color: Color) {
        let m = measure_text(s, Some(&self.font), size, 1.0);
        self.text(s, cx - m.width * 0.5, y, size, color);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let atlas = art::build_atlas();
    let mut font = load_ttf_font_from_bytes(include_bytes!("../assets/PressStart2P.ttf")).expect("font");
    font.set_filter(FilterMode::Nearest);
    let rt = render_target(VW as u32, VH as u32);
    rt.texture.set_filter(FilterMode::Nearest);
    let gfx = Gfx { rt, atlas, font };

    let mut game = game::Game::new(game::Faction::Ua);
    if std::env::var("SELO_DEMO").is_ok() {
        let f = match std::env::var("SELO_FACTION").unwrap_or_default().as_str() {
            "ru" | "RU" => game::Faction::Ru,
            _ => game::Faction::Ua,
        };
        game.start(f);
    }
    let shot = std::env::var("SELO_SHOT").ok();
    let shot_t: f32 = std::env::var("SELO_SHOT_T").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    if std::env::var("SELO_FAST").is_ok() {
        game.speed = 3.0;
    }
    if let Some(t) = std::env::var("SELO_TAB").ok().and_then(|v| v.parse::<usize>().ok()) {
        game.tab = t.min(2);
    }
    if let Some(c) = std::env::var("SELO_CAT").ok().and_then(|v| v.parse::<usize>().ok()) {
        game.cat = c.min(4);
    }
    if std::env::var("SELO_OVERLAY").is_ok() {
        game.overlay = true;
    }
    if std::env::var("SELO_PIPES").is_ok() {
        game.debug_autoplay();
        if let Ok(v) = std::env::var("SELO_CAM") {
            let p: Vec<f32> = v.split(',').filter_map(|s| s.trim().parse().ok()).collect();
            if p.len() == 2 {
                game.cam = world::Map::iso(p[0] as i32, p[1] as i32);
            } else {
                game.cam = world::Map::iso(14, 30);
            }
        } else {
            game.cam = world::Map::iso(14, 30);
        }
    }
    if std::env::var("SELO_LOG").is_ok() {
        game.log_open = true;
    }
    if std::env::var("SELO_HELP").is_ok() {
        game.screen = game::Screen::Help;
    }
    if std::env::var("SELO_PAUSE").is_ok() {
        game.screen = game::Screen::Pause;
    }
    if std::env::var("SELO_END").is_ok() {
        game.win("СЕЛО ВЫСТОЯЛО ТРИ ДНЯ");
    }
    if let Ok(v) = std::env::var("SELO_SEL") {
        let p: Vec<i32> = v.split(',').filter_map(|s| s.trim().parse().ok()).collect();
        if p.len() == 2 {
            game.sel = game.building_at(p[0], p[1]);
        }
    }
    let mut frame = 0u32;
    let mut elapsed = 0.0f32;

    loop {
        let dt = get_frame_time().min(0.05);
        elapsed += dt;
        gfx.begin();
        clear_background(art::col(art::INK));
        ui::frame(&mut game, &gfx, dt);
        gfx.end();

        frame += 1;
        if is_key_pressed(KeyCode::F12) {
            let path = format!("screenshot_{}.png", frame);
            get_screen_data().export_png(&path);
        }
        if let Some(path) = &shot {
            if frame >= 4 && elapsed >= shot_t {
                get_screen_data().export_png(path);
                println!("saved {}", path);
                std::process::exit(0);
            }
        }
        next_frame().await;
    }
}
