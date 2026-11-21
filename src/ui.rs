// HUD, panels, overlays and input. Screen space is the 640x360 virtual canvas.
use crate::art;
use crate::game::{Game, Screen};
use crate::world::{buildings_for, category_color, BKind, Map, CATS, MAP};
use crate::Gfx;
use macroquad::prelude::*;

const TOPH: f32 = 34.0;
const RIGHTW: f32 = 168.0;
const BARH: f32 = 86.0;
const PANEL_X: f32 = crate::VW - RIGHTW;

fn vp_rect() -> Rect {
    Rect::new(0.0, TOPH, PANEL_X, crate::VH - TOPH - BARH)
}

fn inside(r: Rect, m: Vec2) -> bool {
    m.x >= r.x && m.y >= r.y && m.x < r.x + r.w && m.y < r.y + r.h
}

fn panel(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, art::col(art::PANEL));
    draw_rectangle_lines(x + 0.5, y + 0.5, w - 1.0, h - 1.0, 1.0, art::col(art::LINE));
}

fn button(gfx: &Gfx, m: Vec2, r: Rect, label: &str, on: bool, enabled: bool, interactive: bool) -> bool {
    let hot = inside(r, m);
    let bg = if !enabled {
        art::col(art::PANEL)
    } else if hot {
        art::col(art::PANEL3)
    } else if on {
        art::col(art::PANEL3)
    } else {
        art::col(art::PANEL2)
    };
    draw_rectangle(r.x, r.y, r.w, r.h, bg);
    let border = if on { art::col(art::AMBER) } else { art::col(art::LINE) };
    draw_rectangle_lines(r.x + 0.5, r.y + 0.5, r.w - 1.0, r.h - 1.0, 1.0, border);
    let c = if !enabled {
        art::col(art::TEXT_DIM)
    } else if on {
        art::col(art::AMBER)
    } else if hot {
        WHITE
    } else {
        art::col(art::TEXT)
    };
    gfx.text_center(label, r.x + r.w * 0.5, r.y + r.h * 0.5 + 3.0, 8, c);
    interactive && enabled && hot && is_mouse_button_pressed(MouseButton::Left)
}

fn wrap(s: &str, max: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for w in s.split_whitespace() {
        let n = cur.chars().count();
        let wn = w.chars().count();
        if n > 0 && n + 1 + wn > max {
            out.push(cur.clone());
            cur.clear();
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(w);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn stat(gfx: &Gfx, x: f32, y: f32, label: &str, value: &str, vc: Color) {
    gfx.text(label, x, y, 8, art::col(art::TEXT_DIM));
    let w = measure_text(value, Some(&gfx.font), 8, 1.0).width;
    gfx.text(value, x + RIGHTW - 8.0 - w, y, 8, vc);
}

fn chip(
    gfx: &Gfx,
    x: f32,
    y: f32,
    w: f32,
    icon: &str,
    text: &str,
    danger: bool,
    label: &'static str,
    tips: &mut Vec<(Rect, &'static str)>,
) {
    draw_rectangle(x, y, w, 13.0, art::col(art::PANEL2));
    draw_rectangle_lines(x + 0.5, y + 0.5, w - 1.0, 12.0, 1.0, art::col(art::LINE));
    gfx.atlas.draw_at_scaled(icon, x + 7.0, y + 12.0, 0.6, WHITE);
    let c = if danger { art::col(art::RED) } else { art::col(art::TEXT) };
    gfx.text(text, x + 14.0, y + 10.0, 8, c);
    tips.push((Rect::new(x, y, w, 13.0), label));
}

pub fn frame(g: &mut Game, gfx: &Gfx, dt: f32) {
    if g.screen == Screen::Title {
        title(g, gfx);
        return;
    }
    let m = gfx.mouse();
    let vp = vp_rect();

    if is_key_pressed(KeyCode::F1) || is_key_pressed(KeyCode::H) {
        g.screen = if g.screen == Screen::Help { Screen::Play } else if g.screen == Screen::Play { Screen::Help } else { g.screen };
    }

    match g.screen {
        Screen::Help => {
            scene(g, gfx);
            hud(g, gfx, m, false);
            help_overlay(g, gfx, m);
            return;
        }
        Screen::Win | Screen::Lose => {
            g.update(dt);
            scene(g, gfx);
            hud(g, gfx, m, false);
            end_overlay(g, gfx, m);
            return;
        }
        Screen::Pause => {
            scene(g, gfx);
            hud(g, gfx, m, false);
            pause_overlay(g, gfx, m);
            return;
        }
        _ => {}
    }

    // ---- play input ----
    if is_key_pressed(KeyCode::Escape) {
        if g.build.is_some() || g.pipe_mode {
            g.build = None;
            g.pipe_mode = false;
        } else {
            g.screen = Screen::Pause;
        }
    }
    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::P) {
        g.screen = Screen::Pause;
    }
    if is_key_pressed(KeyCode::M) {
        g.overlay = !g.overlay;
    }
    if is_key_pressed(KeyCode::L) {
        g.log_open = !g.log_open;
    }
    if is_key_pressed(KeyCode::Tab) {
        g.tab = (g.tab + 1) % 3;
    }
    for i in 0..5 {
        if is_key_pressed(match i {
            0 => KeyCode::Key1,
            1 => KeyCode::Key2,
            2 => KeyCode::Key3,
            3 => KeyCode::Key4,
            _ => KeyCode::Key5,
        }) {
            g.cat = i;
        }
    }

    let mut pan = Vec2::ZERO;
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        pan.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        pan.x += 1.0;
    }
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        pan.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        pan.y += 1.0;
    }
    if pan.length_squared() > 0.0 {
        let n = pan.normalize();
        g.cam += n * dt * 230.0 / g.zoom;
    }
    if inside(vp, m) {
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            g.zoom = (g.zoom * (1.0 + wheel * 0.08)).clamp(0.8, 1.6);
        }
    }
    g.cam.x = g.cam.x.clamp(-600.0, 600.0);
    g.cam.y = g.cam.y.clamp(60.0, 580.0);

    if inside(vp, m) && !g.overlay && !g.log_open {
        let cx = vp.x + vp.w * 0.5;
        let cy = vp.y + vp.h * 0.5;
        let world = vec2(g.cam.x + (m.x - cx) / g.zoom, g.cam.y + (m.y - cy) / g.zoom);
        g.hover = Map::from_world(world);
    }

    if inside(vp, m) && !g.overlay && !g.log_open {
        if let Some(_k) = g.build {
            if is_mouse_button_pressed(MouseButton::Left) {
                g.place_at_hover();
                g.pipe_last = Some(g.hover);
            } else if is_mouse_button_down(MouseButton::Left) && g.build == Some(BKind::Pipe) {
                let fresh = g.map.get(g.hover.0, g.hover.1).map(|t| !t.pipe && t.building.is_none()).unwrap_or(false);
                if fresh && g.pipe_last != Some(g.hover) {
                    g.place_at_hover();
                    g.pipe_last = Some(g.hover);
                }
            }
        } else if is_mouse_button_pressed(MouseButton::Left) {
            g.sel = g.building_at(g.hover.0, g.hover.1);
        }
    }
    if !is_mouse_button_down(MouseButton::Left) {
        g.pipe_last = None;
    }
    if is_mouse_button_pressed(MouseButton::Right) {
        if g.build.is_some() || g.pipe_mode {
            g.build = None;
            g.pipe_mode = false;
        } else {
            g.sel = None;
        }
    }

    g.paused = g.log_open;
    if !g.log_open {
        g.update(dt);
    }
    if g.screen != Screen::Play {
        // update() may have ended the run.
        scene(g, gfx);
        hud(g, gfx, m, false);
        if g.screen == Screen::Win || g.screen == Screen::Lose {
            end_overlay(g, gfx, m);
        }
        return;
    }

    scene(g, gfx);
    hud(g, gfx, m, true);
    if g.overlay {
        strategic(g, gfx, m);
    }
    if g.log_open {
        log_overlay(g, gfx, m);
    }
    if g.screen == Screen::Play && g.tutorial < 26.0 {
        g.tutorial += dt;
    }
}

fn scene(g: &Game, gfx: &Gfx) {
    let vp = vp_rect();
    draw_rectangle(vp.x, vp.y, vp.w, vp.h, art::col(art::PANEL));
    g.draw_world(&gfx.atlas, vp);
    let origin = vec2(
        vp.x + vp.w * 0.5 - g.cam.x * g.zoom,
        vp.y + vp.h * 0.5 - g.cam.y * g.zoom,
    );
    g.fx.draw(&gfx.atlas, origin, g.zoom);
    g.weather_overlay(vp, &gfx.font);
    g.fx.draw_texts(&gfx.font, origin, g.zoom);
}

fn hud(g: &mut Game, gfx: &Gfx, m: Vec2, interactive: bool) {
    // ---------- top bar ----------
    draw_rectangle(0.0, 0.0, crate::VW, TOPH, art::col(art::PANEL));
    draw_rectangle(0.0, TOPH - 1.0, crate::VW, 1.0, art::col(art::LINE));
    let mut tips: Vec<(Rect, &'static str)> = Vec::new();

    chip(gfx, 2.0, 2.0, 64.0, "money", &fmt(g.res.money), false, "ДЕНЬГИ: НАЛОГИ, КОНВОИ, НАГРАДЫ", &mut tips);
    chip(gfx, 68.0, 2.0, 62.0, "fuel", &fmt(g.res.fuel), g.res.fuel < 8.0, "ТОПЛИВО: НПЗ, ТЕХНИКА, ШТУРМ", &mut tips);
    chip(gfx, 132.0, 2.0, 82.0, "power", &format!("{:.0}/{:.0}", g.power_cap, g.power_use), g.power_eff < 0.85, "ЭНЕРГИЯ: ВЫРАБОТКА / ПОТРЕБЛЕНИЕ", &mut tips);
    chip(gfx, 216.0, 2.0, 60.0, "food", &fmt(g.res.food), g.res.food < 8.0, "ЕДА: ФЕРМЫ КОРМЯТ СЕЛО", &mut tips);
    chip(gfx, 278.0, 2.0, 60.0, "water", &fmt(g.res.water), g.res.water < 8.0, "ВОДА: ВОДОНАПОРНАЯ БАШНЯ", &mut tips);

    // weather
    gfx.atlas.draw_at_scaled(g.weather.icon(), 348.0, 13.0, 0.6, WHITE);
    gfx.text(g.weather.name(), 356.0, 12.0, 8, art::col(art::TEXT_DIM));
    gfx.text(&g.time_str(), 428.0, 12.0, 8, art::col(art::TEXT));
    gfx.text_center(&format!("{:.0}ч", g.left_hours()), 415.0, 24.0, 8, art::col(art::TEXT_DIM));

    let btn_w = 26.0;
    for (i, label) in ["II", "1x", "2x", "3x"].iter().enumerate() {
        let r = Rect::new(524.0 + i as f32 * 28.0, 2.0, btn_w, 13.0);
        let on = if i == 0 { g.screen == Screen::Pause } else { !g.paused && g.speed == i as f32 };
        if button(gfx, m, r, label, on, true, interactive) {
            if i == 0 {
                g.screen = Screen::Pause;
            } else {
                g.speed = i as f32;
                g.paused = false;
            }
        }
    }

    chip(gfx, 2.0, 18.0, 58.0, "steel", &fmt(g.res.steel), g.res.steel < 4.0, "СТАЛЬ: ЗАВОДЫ, ТЕХНИКА, СТРОЙКА", &mut tips);
    chip(gfx, 62.0, 18.0, 58.0, "ammo", &fmt(g.res.ammo), g.res.ammo < 5.0, "БОЕПРИПАСЫ: ОБОРОНА И ШТУРМ", &mut tips);
    chip(gfx, 122.0, 18.0, 58.0, "dronei", &fmt(g.res.drones), g.res.drones < 1.0, "ДРОНЫ: РАЗВЕДКА, FPV, ГЛУБОКИЙ УДАР", &mut tips);
    chip(gfx, 182.0, 18.0, 66.0, "people", &format!("{:.0}/{:.0}", g.pop, g.pop_cap), g.pop < 30.0, "НАСЕЛЕНИЕ / ПРЕДЕЛ ДОМОВ", &mut tips);
    chip(gfx, 250.0, 18.0, 66.0, "morale", &fmt(g.morale), g.morale < 35.0, "ДУХ: ОБОРОНА, РОСТ, ШТУРМ", &mut tips);
    chip(gfx, 318.0, 18.0, 64.0, "shield", &fmt(g.res.manpower), false, "РЕЗЕРВ: МОБИЛИЗАЦИЯ ЧЕРЕЗ ТЦК", &mut tips);
    gfx.atlas.draw_at_scaled(g.faction.flag(), 386.0, 30.0, 0.55, WHITE);
    gfx.text(g.faction.name(), 406.0, 21.0, 8, g.faction.accent());

    // ---------- right panel ----------
    draw_rectangle(PANEL_X, TOPH, RIGHTW, crate::VH - TOPH, art::col(art::PANEL));
    draw_rectangle(PANEL_X, TOPH, 1.0, crate::VH - TOPH, art::col(art::LINE));
    let tabs = ["СЕЛО", "АРМИЯ", "ЦЕЛИ"];
    for (i, name) in tabs.iter().enumerate() {
        let r = Rect::new(PANEL_X + 2.0 + i as f32 * 55.0, TOPH + 2.0, 54.0, 14.0);
        if button(gfx, m, r, name, g.tab == i, true, interactive) {
            g.tab = i;
        }
    }
    draw_rectangle(PANEL_X + 2.0, TOPH + 17.0, RIGHTW - 4.0, 1.0, art::col(art::LINE));

    if g.tab == 0 {
        tab_village(g, gfx, m, interactive);
    } else if g.tab == 1 {
        tab_army(g, gfx, m, interactive);
    } else {
        tab_targets(g, gfx, m, interactive);
    }
    minimap(g, gfx, m, interactive);

    // ---------- build bar ----------
    draw_rectangle(0.0, crate::VH - BARH, crate::VW, BARH, art::col(art::PANEL));
    draw_rectangle(0.0, crate::VH - BARH, crate::VW, 1.0, art::col(art::LINE));
    for (i, (name, cat)) in CATS.iter().enumerate() {
        let r = Rect::new(i as f32 * 128.0 + 1.0, crate::VH - BARH + 2.0, 126.0, 12.0);
        let on = g.cat == i;
        if button(gfx, m, r, name, on, true, interactive) {
            g.cat = i;
        }
        if on {
            draw_rectangle(r.x, r.y + r.h - 1.0, r.w, 1.0, category_color(*cat));
        }
    }
    let list = buildings_for(CATS[g.cat].1);
    let mut hovered: Option<BKind> = None;
    for (i, k) in list.iter().enumerate() {
        let w = crate::VW / 6.0;
        let r = Rect::new(i as f32 * w + 2.0, crate::VH - BARH + 16.0, w - 4.0, 42.0);
        let d = k.def();
        let afford = g.can_afford(*k);
        let on = g.build == Some(*k);
        let hot = inside(r, m);
        draw_rectangle(r.x, r.y, r.w, r.h, art::col(if on { art::PANEL3 } else { art::PANEL2 }));
        draw_rectangle_lines(r.x + 0.5, r.y + 0.5, r.w - 1.0, r.h - 1.0, 1.0, art::col(if on { art::AMBER } else { art::LINE }));
        let tint = if afford {
            Color::new(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::new(0.62, 0.42, 0.4, 1.0)
        };
        if k == &BKind::Pipe {
            draw_line(r.x + 18.0, r.y + 24.0, r.x + r.w - 18.0, r.y + 24.0, 3.0, art::col(art::METAL[3]));
            draw_line(r.x + 18.0, r.y + 23.0, r.x + r.w - 18.0, r.y + 23.0, 1.0, art::col(art::AMBER));
        } else {
            gfx.atlas.draw_at_scaled(d.sprite, r.x + r.w * 0.5, r.y + 25.0, 0.55, tint);
        }
        gfx.text_center(d.name, r.x + r.w * 0.5, r.y + 31.0, 8, if afford { art::col(art::TEXT) } else { art::col(art::TEXT_DIM) });
        if interactive && hot && is_mouse_button_pressed(MouseButton::Left) {
            g.toggle_build(*k);
        }
        if hot {
            hovered = Some(*k);
        }
    }

    // tooltip above the bar
    if let Some(k) = hovered {
        let d = k.def();
        let w = 260.0;
        let h = 60.0;
        let x = ((m.x - w * 0.5).clamp(4.0, crate::VW - w - 4.0)).round();
        let y = crate::VH - BARH - h - 4.0;
        panel(x, y, w, h);
        gfx.text(d.full, x + 6.0, y + 13.0, 8, art::col(art::TEXT));
        gfx.text(&g.cost_str(k), x + 6.0, y + 25.0, 8, if g.can_afford(k) { art::col(art::AMBER) } else { art::col(art::RED) });
        for (j, line) in wrap(d.desc, 30).iter().take(2).enumerate() {
            gfx.text(line, x + 6.0, y + 39.0 + j as f32 * 10.0, 8, art::col(art::TEXT_DIM));
        }
    }

    // info strip
    let info = g.build.or_else(|| g.sel.map(|i| g.buildings[i].kind));
    let iy = crate::VH - 28.0;
    draw_rectangle(0.0, iy - 2.0, crate::VW, 30.0, art::col(art::PANEL2));
    draw_rectangle(0.0, iy - 2.0, crate::VW, 1.0, art::col(art::LINE));
    if let Some(k) = info {
        let d = k.def();
        gfx.text(&format!("{}  ·  {}", d.full, g.cost_str(k)), 6.0, iy + 9.0, 8, art::col(art::TEXT));
        gfx.text(d.desc, 6.0, iy + 19.0, 8, art::col(art::TEXT_DIM));
    } else if g.pipe_mode {
        gfx.text("ТЯНИТЕ ТРУБУ ЛКМ ОТ ВЫШКИ ДО НПЗ · ПКМ — ЗАКОНЧИТЬ", 6.0, iy + 14.0, 8, art::col(art::AMBER));
    } else {
        gfx.text("ВЫБЕРИТЕ ЗДАНИЕ · ЛКМ СТРОИТЬ · ПКМ ОТМЕНА · M КАРТА · L ЖУРНАЛ · H ПОМОЩЬ", 6.0, iy + 14.0, 8, art::col(art::TEXT_DIM));
    }

    // ---------- threat banner ----------
    if let Some(t) = &g.enemy.threat {
        let pulse = 0.55 + 0.45 * (g.frame_t * 7.0).sin().abs();
        let (label, col) = match t.kind {
            crate::game::ThreatKind::Arty => ("ОБСТРЕЛ", art::col(art::RED)),
            crate::game::ThreatKind::Drones => ("ДРОНЫ", art::col(art::AMBER)),
            crate::game::ThreatKind::Assault => ("ШТУРМ", art::col(art::RED)),
        };
        let text = if t.kind == crate::game::ThreatKind::Assault {
            if t.revealed {
                format!("{} · {} · {:.1}ч", label, Game::sector_name(t.sector), t.t.max(0.0))
            } else {
                format!("{} · НАПРАВЛЕНИЕ ? · {:.1}ч", label, t.t.max(0.0))
            }
        } else {
            format!("{} · {:.1}ч", label, t.t.max(0.0))
        };
        let w = 210.0;
        let x = (PANEL_X - w) * 0.5;
        draw_rectangle(x, TOPH + 6.0, w, 18.0, Color::new(0.12, 0.04, 0.05, 0.86));
        draw_rectangle_lines(x + 0.5, TOPH + 6.5, w - 1.0, 17.0, 1.0, Color::new(col.r, col.g, col.b, pulse));
        gfx.text_center(&text, x + w * 0.5, TOPH + 19.0, 8, Color::new(1.0, 0.85, 0.8, pulse));
    }

    // ---------- toasts ----------
    let mut ty = crate::VH - BARH - 6.0;
    if g.sel.is_some() {
        ty -= 58.0;
    }
    for t in g.toasts.iter().rev().take(3) {
        let mut lines = wrap(&t.text, 42);
        if lines.len() > 3 {
            lines.truncate(3);
            if let Some(last) = lines.last_mut() {
                last.push('…');
            }
        }
        let h = lines.len() as f32 * 10.0 + 5.0;
        ty -= h + 2.0;
        let x = 6.0;
        let w = 352.0;
        let a = (t.t / 1.2).clamp(0.0, 1.0);
        draw_rectangle(x, ty, w, h, Color::new(0.05, 0.05, 0.07, 0.82 * a));
        draw_rectangle_lines(x + 0.5, ty + 0.5, w - 1.0, h - 1.0, 1.0, Color::new(t.color.r, t.color.g, t.color.b, 0.5 * a));
        for (li, line) in lines.iter().enumerate() {
            gfx.text(line, x + 5.0, ty + 10.0 + li as f32 * 10.0, 8, Color::new(t.color.r, t.color.g, t.color.b, a));
        }
    }

    // ---------- selected building ----------
    if let Some(i) = g.sel {
        if let Some(b) = g.buildings.get(i) {
            if b.alive {
                let w = 204.0;
                let x = 6.0;
                let y = crate::VH - BARH - 54.0;
                panel(x, y, w, 48.0);
                let d = b.kind.def();
                gfx.text(d.name, x + 5.0, y + 12.0, 8, art::col(art::TEXT));
                let hp = (b.hp / b.max_hp).clamp(0.0, 1.0);
                gfx.text(&format!("ПРОЧНОСТЬ {:.0}%", hp * 100.0), x + 5.0, y + 23.0, 8, art::col(art::TEXT_DIM));
                draw_rectangle(x + 5.0, y + 26.0, w - 10.0, 3.0, art::col(art::INK));
                draw_rectangle(x + 5.0, y + 26.0, (w - 10.0) * hp, 3.0, if hp > 0.5 { art::col(art::GREEN) } else { art::col(art::RED) });
                let mut st = String::new();
                if b.kind.needs_power() {
                    st.push_str(if b.powered { "ПИТ. ЕСТЬ" } else { "ПИТ. НЕТ" });
                    st.push_str(" · ");
                }
                if b.kind == BKind::Derrick {
                    st.push_str(if b.connected { "ТРУБА ЕСТЬ" } else { "ТРУБЫ НЕТ" });
                }
                if b.kind == BKind::Refinery {
                    st.push_str(&format!("СЫРЬЁ {:.0}", g.res.crude));
                }
                gfx.text(&st, x + 5.0, y + 38.0, 8, art::col(art::AMBER));
                if button(gfx, m, Rect::new(x + w - 46.0, y + 4.0, 42.0, 13.0), "СНЕСТИ", false, true, interactive) {
                    let bi = i;
                    g.sel = None;
                    g.demolish(bi);
                }
            }
        }
    }

    if interactive {
        if let Some((_, label)) = tips.iter().find(|(r, _)| inside(*r, m)) {
            let w = measure_text(label, Some(&gfx.font), 8, 1.0).width + 8.0;
            let x = (m.x + 8.0).min(crate::VW - w - 4.0).max(4.0);
            let y = (m.y + 14.0).min(crate::VH - 20.0);
            panel(x, y, w, 15.0);
            gfx.text(label, x + 4.0, y + 11.0, 8, art::col(art::AMBER));
        }
    }
}

fn log_overlay(g: &mut Game, gfx: &Gfx, m: Vec2) {
    draw_rectangle(0.0, 0.0, crate::VW, crate::VH, Color::new(0.02, 0.03, 0.05, 0.72));
    let w = 420.0;
    let h = 206.0;
    let x = (PANEL_X - w) * 0.5;
    let y = 46.0;
    panel(x, y, w, h);
    gfx.text("ЖУРНАЛ СОБЫТИЙ", x + 10.0, y + 15.0, 8, art::col(art::AMBER));
    let mut ty = y + 34.0;
    for (s, c) in g.log_lines.iter().rev().take(6).rev() {
        let mut lines = wrap(s, 46);
        if lines.len() > 2 {
            lines.truncate(2);
            if let Some(last) = lines.last_mut() {
                last.push('…');
            }
        }
        for line in &lines {
            gfx.text(line, x + 10.0, ty, 8, *c);
            ty += 10.0;
        }
        ty += 2.0;
    }
    if button(gfx, m, Rect::new(x + w - 66.0, y + h - 20.0, 56.0, 14.0), "ЗАКРЫТЬ", false, true, true) {
        g.log_open = false;
    }
    gfx.text("L — ЗАКРЫТЬ", x + 10.0, y + h - 10.0, 8, art::col(art::TEXT_DIM));
}

fn tab_village(g: &mut Game, gfx: &Gfx, m: Vec2, interactive: bool) {
    let x = PANEL_X + 4.0;
    let mut y = TOPH + 32.0;
    stat(gfx, x, y, "НАСЕЛЕНИЕ", &format!("{:.0}/{:.0}", g.pop, g.pop_cap), art::col(art::TEXT));
    y += 11.0;
    stat(gfx, x, y, "ДУХ", &format!("{:.0}", g.morale), if g.morale < 35.0 { art::col(art::RED) } else { art::col(art::GREEN) });
    y += 11.0;
    stat(gfx, x, y, "ЦЕЛОСТНОСТЬ", &format!("{:.0}%", g.integrity), if g.integrity < 50.0 { art::col(art::RED) } else { art::col(art::TEXT) });
    y += 11.0;
    let food_rate = g.count(BKind::Farm) as f32 * 4.5 - g.pop * 0.05;
    stat(gfx, x, y, "ЕДА", &format!("{:+.1}/ч", food_rate), if food_rate < 0.0 { art::col(art::RED) } else { art::col(art::TEXT_DIM) });
    y += 11.0;
    let water_rate = g.count(BKind::Watertower) as f32 * 7.0 - g.count(BKind::Farm) as f32 * 0.5 - g.pop * 0.05;
    stat(gfx, x, y, "ВОДА", &format!("{:+.1}/ч", water_rate), if water_rate < 0.0 { art::col(art::RED) } else { art::col(art::TEXT_DIM) });
    y += 11.0;
    stat(gfx, x, y, "ЭНЕРГИЯ", &format!("{:.0}/{:.0}", g.power_cap, g.power_use), if g.power_eff < 0.85 { art::col(art::RED) } else { art::col(art::CYAN) });
    y += 11.0;
    stat(gfx, x, y, "НЕФТЬ/ТОПЛ", &format!("{:.0}/{:.0}", g.res.crude, g.res.fuel), art::col(art::AMBER));
    y += 11.0;
    stat(gfx, x, y, "ТРУБЫ", &format!("{}/{}", g.connected_derricks, g.derricks), if g.connected_derricks < g.derricks { art::col(art::RED) } else { art::col(art::GREEN) });

    y += 14.0;
    gfx.text("ТЦК", x, y, 8, art::col(art::TEXT_DIM));
    gfx.text(&format!("{:.0}%", g.tcc), x + 30.0, y, 8, art::col(art::TEXT));
    if button(gfx, m, Rect::new(x + 58.0, y - 8.0, 12.0, 12.0), "-", false, g.tcc > 0.0, interactive) {
        g.tcc = (g.tcc - 10.0).clamp(0.0, 100.0);
    }
    if button(gfx, m, Rect::new(x + 72.0, y - 8.0, 12.0, 12.0), "+", false, g.tcc < 100.0, interactive) {
        g.tcc = (g.tcc + 10.0).clamp(0.0, 100.0);
    }
    let (warn, wc) = if g.tcc >= 65.0 {
        ("БОЯТСЯ", art::col(art::RED))
    } else if g.tcc <= 20.0 {
        ("МАЛО", art::col(art::AMBER))
    } else {
        ("НОРМА", art::col(art::GREEN))
    };
    gfx.text(warn, x + 90.0, y, 8, wc);
    if g.tcc >= 65.0 {
        gfx.text("ЛЮДИ БЕГУТ ИЗ СЕЛА", x, y + 11.0, 8, art::col(art::RED));
    }
}

fn tab_army(g: &mut Game, gfx: &Gfx, m: Vec2, interactive: bool) {
    let x = PANEL_X + 4.0;
    let mut y = TOPH + 32.0;
    let focus = g.focus;
    for i in 0..3 {
        let r = Rect::new(x + i as f32 * 53.0, y - 10.0, 50.0, 14.0);
        let label = ["СЕВ", "ЦЕНТР", "ЮГ"][i];
        if button(gfx, m, r, label, focus == Some(i), true, interactive) {
            g.set_focus(i);
        }
    }
    y += 12.0;
    gfx.text(&format!("ПЕХ {:.0}  ТАНК {:.0}", g.army.inf, g.army.tanks), x, y, 8, art::col(art::TEXT));
    y += 10.0;
    gfx.text(&format!("БТР {:.0}  АРТ {:.0}", g.army.ifv, g.army.arty), x, y, 8, art::col(art::TEXT));
    y += 10.0;
    let sec = focus.unwrap_or(1);
    let def = g.army_defense(sec);
    gfx.text(&format!("ОБОРОНА {}: {:.0}", Game::sector_name(sec), def), x, y, 8, art::col(art::GREEN));
    y += 8.0;

    let bw = 78.0;
    let bh = 23.0;
    let items: [(&str, &str); 6] = [
        ("КОНТРБАТ", "15 БК"),
        ("ГЛ.УДАР", "3 ДР+10БК"),
        ("РАЗВЕДКА", "1 ДРОН"),
        ("FPV", "1 ДРОН"),
        ("РЕМОНТ", "60 ГРН"),
        ("ЗАКУП БК", "80 ГРН"),
    ];
    for (i, (name, cost)) in items.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;
        let r = Rect::new(x + col as f32 * (bw + 4.0), y + row as f32 * (bh + 3.0), bw, bh);
        let (cd, ok) = match i {
            0 => (g.ab.counter_cd, g.has(BKind::Radar) && g.res.ammo >= 15.0),
            1 => (g.ab.strike_cd, g.res.drones >= 3.0 && g.res.ammo >= 10.0),
            2 => (g.ab.recon_cd, g.res.drones >= 1.0),
            3 => (g.ab.fpv_cd, g.res.drones >= 1.0),
            4 => (0.0, g.res.money >= 60.0),
            _ => (0.0, g.res.money >= 80.0),
        };
        let bg = art::col(if ok { art::PANEL2 } else { art::PANEL });
        let hot = inside(r, m);
        draw_rectangle(r.x, r.y, r.w, r.h, if hot && ok { art::col(art::PANEL3) } else { bg });
        draw_rectangle_lines(r.x + 0.5, r.y + 0.5, r.w - 1.0, r.h - 1.0, 1.0, art::col(art::LINE));
        gfx.text_center(name, r.x + r.w * 0.5, r.y + 9.0, 8, if ok { art::col(art::TEXT) } else { art::col(art::TEXT_DIM) });
        gfx.text_center(cost, r.x + r.w * 0.5, r.y + 18.0, 8, art::col(art::TEXT_DIM));
        if cd > 0.0 {
            draw_rectangle(r.x, r.y + r.h - 2.0, r.w * (cd / 4.0).clamp(0.0, 1.0), 2.0, art::col(art::AMBER));
        }
        if interactive && ok && hot && is_mouse_button_pressed(MouseButton::Left) {
            let res = match i {
                0 => g.counter_battery(),
                1 => g.deep_strike(),
                2 => g.recon(),
                3 => g.fpv_sortie(),
                4 => g.repair_village(),
                _ => {
                    g.buy_ammo();
                    Ok(())
                }
            };
            if let Err(e) = res {
                g.log(&e, art::col(art::RED));
            }
        }
    }
    y += 3.0 * (bh + 3.0) + 2.0;
    let mut prod = String::new();
    if let Some(b) = g.buildings.iter().find(|b| b.alive && b.built() && b.kind == BKind::Barracks) {
        prod.push_str(&format!("КАЗАРМА {:.0}% ", b.progress() * 100.0));
    }
    if let Some(b) = g.buildings.iter().find(|b| b.alive && b.built() && b.kind == BKind::TankFactory) {
        prod.push_str(&format!("ТАНКИ {:.0}%", b.progress() * 100.0));
    }
    if !prod.is_empty() {
        gfx.text(&prod, x, y, 8, art::col(art::CYAN));
    }
}

fn tab_targets(g: &mut Game, gfx: &Gfx, m: Vec2, interactive: bool) {
    let x = PANEL_X + 4.0;
    let mut y = TOPH + 20.0;
    let can = g.army.inf >= 10.0 && g.army.tanks >= 2.0 && g.res.ammo >= 20.0 && g.res.fuel >= 20.0 && g.assault_cd <= 0.0;
    for i in 0..g.targets.len() {
        let t = &g.targets[i];
        let r = Rect::new(x, y, RIGHTW - 4.0, 38.0);
        draw_rectangle(r.x, r.y, r.w, r.h, art::col(art::PANEL2));
        draw_rectangle_lines(r.x + 0.5, r.y + 0.5, r.w - 1.0, r.h - 1.0, 1.0, art::col(if t.captured { art::GREEN } else { art::LINE }));
        gfx.text(t.name, x + 4.0, y + 10.0, 8, if t.captured { art::col(art::GREEN) } else { art::col(art::TEXT) });
        gfx.text(&format!("ГАРНИЗОН {:.0}", t.garrison), x + 4.0, y + 21.0, 8, art::col(art::TEXT_DIM));
        gfx.text(t.bonus, x + 4.0, y + 33.0, 8, art::col(art::AMBER));
        let label = if t.captured { "ВЗЯТО" } else if g.assault_cd > 0.0 { "ЖДИТЕ" } else { "ШТУРМ" };
        let br = Rect::new(x + 112.0, y + 2.0, 44.0, 14.0);
        if button(gfx, m, br, label, false, can && !t.captured, interactive) {
            if let Err(e) = g.assault_target(i) {
                g.log(&e, art::col(art::RED));
            }
        }
        y += 40.0;
    }
    y += 1.0;
    gfx.text("НУЖНО 10 ПЕХ 2 ТАНК", x, y + 3.0, 8, art::col(art::TEXT_DIM));
    gfx.text("20 БК И 20 ТОПЛ", x, y + 13.0, 8, art::col(art::TEXT_DIM));
    y += 24.0;
    if g.assault_cd > 0.0 {
        gfx.text(&format!("ОТРЯД ГОТОВ ЧЕРЕЗ {:.1}ч", g.assault_cd), x, y + 3.0, 8, art::col(art::AMBER));
    } else {
        gfx.text(&format!("ДО КОНЦА: {:.0}ч", g.left_hours()), x, y + 3.0, 8, art::col(art::AMBER));
    }
}

fn minimap(g: &mut Game, _gfx: &Gfx, m: Vec2, interactive: bool) {
    let mx = PANEL_X + 6.0;
    let my = 208.0;
    let tw = 3.7;
    let th = 1.7;
    draw_rectangle(mx - 2.0, my - 2.0, 40.0 * tw + 4.0, 40.0 * th + 4.0, art::col(art::PANEL2));
    for y in 0..MAP {
        for x in 0..MAP {
            let t = &g.map.tiles[(y * MAP + x) as usize];
            let c = t.terrain.mini_color();
            draw_rectangle(mx + x as f32 * tw, my + y as f32 * th, tw, th, Color::new(c.r * 0.78, c.g * 0.78, c.b * 0.78, 1.0));
        }
    }
    for b in &g.buildings {
        if !b.alive {
            continue;
        }
        let c = category_color(b.kind.def().cat);
        draw_rectangle(mx + b.x as f32 * tw, my + b.y as f32 * th, tw + 1.0, th + 1.0, Color::new(c.r * 0.9, c.g * 0.9, c.b * 0.9, 1.0));
    }
    let (tx, ty) = Map::from_world(g.cam);
    draw_rectangle_lines(mx + (tx as f32 - 3.0) * tw, my + (ty as f32 - 2.0) * th, 7.0 * tw, 5.0 * th, 1.0, Color::new(1.0, 1.0, 1.0, 0.55));

    if interactive && is_mouse_button_pressed(MouseButton::Left) {
        let rx = mx - 2.0;
        let ry = my - 2.0;
        let rw = 40.0 * tw + 4.0;
        let rh = 40.0 * th + 4.0;
        if m.x >= rx && m.y >= ry && m.x < rx + rw && m.y < ry + rh {
            let tx = ((m.x - mx) / tw).clamp(0.0, 39.0);
            let ty = ((m.y - my) / th).clamp(0.0, 39.0);
            g.cam = Map::iso(tx as i32, ty as i32);
        }
    }
}

fn strategic(g: &mut Game, gfx: &Gfx, m: Vec2) {
    draw_rectangle(0.0, TOPH, PANEL_X, crate::VH - TOPH - BARH, Color::new(0.02, 0.03, 0.05, 0.72));
    let w = 430.0;
    let h = 222.0;
    let x = (PANEL_X - w) * 0.5;
    let y = 46.0;
    panel(x, y, w, h);
    gfx.text_center("ОПЕРАТИВНАЯ КАРТА", x + w * 0.5, y + 18.0, 16, art::col(art::AMBER));
    gfx.text("СОБЕРИТЕ АРМИЮ И ШТУРМУЙТЕ СОСЕДНИЕ СЁЛА.", x + 12.0, y + 34.0, 8, art::col(art::TEXT_DIM));

    for i in 0..g.targets.len() {
        let t = &g.targets[i];
        let ry = y + 42.0 + i as f32 * 44.0;
        draw_rectangle(x + 10.0, ry, w - 20.0, 40.0, art::col(art::PANEL2));
        draw_rectangle_lines(x + 10.5, ry + 0.5, w - 21.0, 39.0, 1.0, art::col(if t.captured { art::GREEN } else { art::LINE }));
        gfx.text(t.name, x + 18.0, ry + 12.0, 8, if t.captured { art::col(art::GREEN) } else { art::col(art::TEXT) });
        gfx.text(&format!("{} · ГАРНИЗОН {:.0}", t.dir, t.garrison), x + 18.0, ry + 24.0, 8, art::col(art::TEXT_DIM));
        gfx.text(t.bonus, x + 18.0, ry + 34.0, 8, art::col(art::AMBER));
        let br = Rect::new(x + w - 96.0, ry + 11.0, 78.0, 18.0);
        let can = !t.captured && g.army.inf >= 10.0 && g.army.tanks >= 2.0 && g.res.ammo >= 20.0 && g.res.fuel >= 20.0 && g.assault_cd <= 0.0;
        let label = if t.captured { "ВЗЯТО" } else { "ШТУРМ" };
        if button(gfx, m, br, label, false, can, true) {
            if let Err(e) = g.assault_target(i) {
                g.log(&e, art::col(art::RED));
            }
        }
    }

    let ly = y + 182.0;
    gfx.text(&format!("СЛЕДУЮЩИЙ ШТУРМ: {}", g.next_wave_hour().map(|h| format!("{:.0}ч", h)).unwrap_or_else(|| "—".into())), x + 12.0, ly, 8, art::col(art::TEXT_DIM));
    gfx.text(&format!("ДУХ {:.0} · ОБОРОНА {:.0}", g.morale, g.army_defense(g.focus.unwrap_or(1))), x + 12.0, ly + 12.0, 8, art::col(art::TEXT_DIM));
    if button(gfx, m, Rect::new(x + w - 70.0, y + h - 24.0, 58.0, 16.0), "ЗАКРЫТЬ", false, true, true) {
        g.overlay = false;
    }
}

fn help_overlay(g: &mut Game, gfx: &Gfx, m: Vec2) {
    let w = 420.0;
    let h = 268.0;
    let x = (crate::VW - w) * 0.5;
    let y = 40.0;
    draw_rectangle(0.0, 0.0, crate::VW, crate::VH, Color::new(0.02, 0.02, 0.03, 0.72));
    panel(x, y, w, h);
    gfx.text_center("КАК ИГРАТЬ", x + w * 0.5, y + 18.0, 16, art::col(art::AMBER));
    let lines = [
        "WASD / СТРЕЛКИ — ДВИГАТЬ КАМЕРУ",
        "КОЛЕСО МЫШИ — ПРИБЛИЖЕНИЕ",
        "ЛКМ — ПОСТРОИТЬ ИЛИ ВЫБРАТЬ ЗДАНИЕ",
        "ПКМ / ESC — ОТМЕНИТЬ СТРОЙКУ",
        "1-5 — КАТЕГОРИИ СТРОЙКИ",
        "TAB — ПЕРЕКЛЮЧИТЬ ПАНЕЛЬ СПРАВА",
        "M — КАРТА СОСЕДНИХ СЁЛ",
        "L — ЖУРНАЛ СОБЫТИЙ",
        "H / F1 — ЭТА ПОДСКАЗКА",
        "ПРОБЕЛ / P — ПАУЗА",
        "F12 — СКРИНШОТ",
        "",
        "СОВЕТ: ТРУБА ОТ ВЫШКИ ДО НПЗ ДАЁТ ТОПЛИВО.",
        "СОВЕТ: КАЗАРМА + ПВО ДО ПЕРВОГО ШТУРМА.",
        "СОВЕТ: ТЦК ВЫШЕ 60% ЛОМАЕТ ДУХ СЕЛА.",
    ];
    for (i, line) in lines.iter().enumerate() {
        gfx.text(line, x + 18.0, y + 36.0 + i as f32 * 13.0, 8, art::col(if i >= 12 { art::AMBER } else { art::TEXT }));
    }
    if button(gfx, m, Rect::new(x + w * 0.5 - 50.0, y + h - 30.0, 100.0, 18.0), "ЗАКРЫТЬ", false, true, true) {
        g.screen = Screen::Play;
    }
}

fn pause_overlay(g: &mut Game, gfx: &Gfx, m: Vec2) {
    draw_rectangle(0.0, 0.0, crate::VW, crate::VH, Color::new(0.02, 0.02, 0.03, 0.55));
    let w = 240.0;
    let h = 116.0;
    let x = (crate::VW - w) * 0.5;
    let y = 110.0;
    panel(x, y, w, h);
    gfx.text_center("ПАУЗА", x + w * 0.5, y + 26.0, 16, art::col(art::AMBER));
    gfx.text_center(&g.time_str(), x + w * 0.5, y + 42.0, 8, art::col(art::TEXT_DIM));
    if button(gfx, m, Rect::new(x + 30.0, y + 54.0, 82.0, 18.0), "ПРОДОЛЖИТЬ", false, true, true) {
        g.screen = Screen::Play;
    }
    if button(gfx, m, Rect::new(x + 126.0, y + 54.0, 84.0, 18.0), "ПОМОЩЬ", false, true, true) {
        g.screen = Screen::Help;
    }
    if button(gfx, m, Rect::new(x + 30.0, y + 80.0, 180.0, 18.0), "В ГЛАВНОЕ МЕНЮ", false, true, true) {
        *g = Game::new(g.faction);
    }
}

fn end_overlay(g: &mut Game, gfx: &Gfx, m: Vec2) {
    let win = g.screen == Screen::Win;
    draw_rectangle(0.0, 0.0, crate::VW, crate::VH, Color::new(0.02, 0.02, 0.03, 0.7));
    let w = 360.0;
    let h = 236.0;
    let x = (crate::VW - w) * 0.5;
    let y = 60.0;
    panel(x, y, w, h);
    gfx.text_center(if win { "СЕЛО ВЫСТОЯЛО" } else { "СЕЛО ПАЛО" }, x + w * 0.5, y + 30.0, 24, if win { art::col(art::GREEN) } else { art::col(art::RED) });
    gfx.text_center(&g.end_reason, x + w * 0.5, y + 50.0, 8, art::col(art::TEXT));
    let lines = [
        format!("ЦЕЛОСТНОСТЬ: {:.0}%", g.integrity),
        format!("НАСЕЛЕНИЕ: {:.0}   ДУХ: {:.0}", g.pop, g.morale),
        format!("СБИТО ДРОНОВ: {}   ОБСТРЕЛОВ: {}", g.stats.drones_down, g.stats.shells),
        format!("ШТУРМОВ ОТБИТО: {}   СЁЛ ВЗЯТО: {}", g.stats.assaults, g.stats.captured),
        format!("ТОПЛИВА СДЕЛАНО: {:.0}", g.stats.fuel_made),
    ];
    for (i, line) in lines.iter().enumerate() {
        gfx.text_center(line, x + w * 0.5, y + 76.0 + i as f32 * 14.0, 8, art::col(art::TEXT_DIM));
    }
    if button(gfx, m, Rect::new(x + 40.0, y + h - 54.0, 120.0, 20.0), "ЗАНОВО", false, true, true) {
        g.start(g.faction);
    }
    if button(gfx, m, Rect::new(x + 200.0, y + h - 54.0, 120.0, 20.0), "В МЕНЮ", false, true, true) {
        *g = Game::new(g.faction);
    }
    gfx.text_center("F12 — СОХРАНИТЬ СКРИНШОТ", x + w * 0.5, y + h - 20.0, 8, art::col(art::TEXT_DIM));
}

fn title(g: &mut Game, gfx: &Gfx) {
    g.ambient = g.compute_ambient();
    g.draw_world(&gfx.atlas, Rect::new(0.0, 0.0, crate::VW, crate::VH));
    draw_rectangle(0.0, 0.0, crate::VW, crate::VH, Color::new(0.02, 0.03, 0.05, 0.62));
    let m = gfx.mouse();

    gfx.text_center("СЕЛО ЗА 3 ДНЯ", crate::VW * 0.5, 62.0, 24, art::col(art::AMBER));
    gfx.text_center("ОБОРОНА СЕЛА У ФРОНТА · ЭКОНОМИКА · ДРОНЫ · ШТУРМЫ", crate::VW * 0.5, 82.0, 8, art::col(art::TEXT));

    let cards = [Rect::new(48.0, 116.0, 258.0, 132.0), Rect::new(334.0, 116.0, 258.0, 132.0)];
    for i in 0..2 {
        let r = cards[i];
        let on = g.title_sel == i;
        draw_rectangle(r.x, r.y, r.w, r.h, art::col(if on { art::PANEL3 } else { art::PANEL }));
        draw_rectangle_lines(r.x + 0.5, r.y + 0.5, r.w - 1.0, r.h - 1.0, if on { 2.0 } else { 1.0 }, art::col(if on { art::AMBER } else { art::LINE }));
        if inside(r, m) && is_mouse_button_pressed(MouseButton::Left) {
            g.title_sel = i;
        }
        let (flag, name, col, sub) = if i == 0 {
            ("flag0", "УКРАИНА", art::col(art::UA_YELLOW), ["СИЛЬНЫЙ ДУХ СЕЛА", "ДРОНЫ И ПВО", "ТОЧНЫЕ УДАРЫ"])
        } else {
            ("flag1", "РОССИЯ", art::col(art::RU_RED), ["СЛАБАЯ АРМИЯ", "ЗАДЕРЖКИ ПОСТАВОК", "КОРРУПЦИЯ И ХАОС"])
        };
        gfx.atlas.draw_at_scaled(flag, r.x + 26.0, r.y + 38.0, 1.4, WHITE);
        gfx.text(name, r.x + 52.0, r.y + 26.0, 16, col);
        for (j, line) in sub.iter().enumerate() {
            gfx.text(line, r.x + 14.0, r.y + 62.0 + j as f32 * 15.0, 8, art::col(art::TEXT));
        }
        let diff = if i == 0 { "СЛОЖНОСТЬ: СРЕДНЯЯ" } else { "СЛОЖНОСТЬ: ВЫСОКАЯ" };
        gfx.text(diff, r.x + 14.0, r.y + r.h - 12.0, 8, art::col(if i == 0 { art::GREEN } else { art::RED }));
    }

    let start = Rect::new(crate::VW * 0.5 - 100.0, 268.0, 200.0, 24.0);
    if button(gfx, m, start, "НАЧАТЬ ОБОРОНУ", false, true, true) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
        g.start(if g.title_sel == 0 { crate::game::Faction::Ua } else { crate::game::Faction::Ru });
    }
    gfx.text_center("WASD — КАМЕРА · ЛКМ — СТРОЙКА · M — КАРТА · H — ПОМОЩЬ", crate::VW * 0.5, 318.0, 8, art::col(art::TEXT_DIM));
    gfx.text_center("ВЫМЫШЛЕННОЕ СЕЛО. ИГРОВАЯ СТРАТЕГИЯ, А НЕ РЕАЛЬНЫЕ СОБЫТИЯ.", crate::VW * 0.5, 342.0, 8, art::col(art::TEXT_DIM));
}

fn fmt(v: f32) -> String {
    if v >= 1000.0 {
        format!("{:.1}К", v / 1000.0)
    } else {
        format!("{:.0}", v)
    }
}
