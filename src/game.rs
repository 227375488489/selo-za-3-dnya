// Core simulation: economy, population, enemy pressure, combat, actions.
use crate::art::{self, Atlas, Rng};
use crate::fx::Fx;
use crate::world::{BKind, Map, Terrain, MAP};
use macroquad::prelude::*;
use std::collections::{HashSet, VecDeque};

/// Game hours per real second at 1x speed.
pub const HOUR: f32 = 0.1;
pub const TOTAL_HOURS: f32 = 72.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Faction {
    Ua,
    Ru,
}

impl Faction {
    pub fn name(self) -> &'static str {
        match self {
            Faction::Ua => "УКРАИНА",
            Faction::Ru => "РОССИЯ",
        }
    }
    pub fn sub(self) -> &'static str {
        match self {
            Faction::Ua => "СИЛЬНЫЙ ДУХ · ДРОНЫ · ПВО",
            Faction::Ru => "СЛАБАЯ АРМИЯ · ЗАДЕРЖКИ · КОРРУПЦИЯ",
        }
    }
    pub fn color(self) -> Color {
        art::col(match self {
            Faction::Ua => art::UA_BLUE,
            Faction::Ru => art::RU_RED,
        })
    }
    pub fn accent(self) -> Color {
        art::col(match self {
            Faction::Ua => art::UA_YELLOW,
            Faction::Ru => art::RU_WHITE,
        })
    }
    pub fn flag(self) -> &'static str {
        match self {
            Faction::Ua => "flag0",
            Faction::Ru => "flag1",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Title,
    Play,
    Pause,
    Help,
    Win,
    Lose,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThreatKind {
    Arty,
    Drones,
    Assault,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weather {
    Clear,
    Rain,
    Fog,
    Storm,
}

impl Weather {
    pub fn name(self) -> &'static str {
        match self {
            Weather::Clear => "ЯСНО",
            Weather::Rain => "ДОЖДЬ",
            Weather::Fog => "ТУМАН",
            Weather::Storm => "ГРОЗА",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Weather::Clear => "star",
            Weather::Rain => "water",
            Weather::Fog => "smoke0",
            Weather::Storm => "power",
        }
    }
    pub fn solar(self) -> f32 {
        match self {
            Weather::Clear => 1.0,
            Weather::Rain => 0.35,
            Weather::Fog => 0.55,
            Weather::Storm => 0.12,
        }
    }
    pub fn intercept(self) -> f32 {
        match self {
            Weather::Clear => 0.0,
            Weather::Rain => -0.06,
            Weather::Fog => 0.04,
            Weather::Storm => -0.12,
        }
    }
}

#[derive(Clone)]
pub struct Res {
    pub money: f32,
    pub fuel: f32,
    pub crude: f32,
    pub food: f32,
    pub water: f32,
    pub steel: f32,
    pub ammo: f32,
    pub drones: f32,
    pub manpower: f32,
}

impl Res {
    pub fn start(f: Faction) -> Self {
        let k = if f == Faction::Ua { 1.0 } else { 0.8 };
        Res {
            money: 650.0 * k,
            fuel: 30.0,
            crude: 0.0,
            food: 45.0,
            water: 45.0,
            steel: 34.0 * k,
            ammo: 30.0,
            drones: if f == Faction::Ua { 3.0 } else { 2.0 },
            manpower: 8.0,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Army {
    pub inf: f32,
    pub tanks: f32,
    pub ifv: f32,
    pub arty: f32,
}

impl Army {
    pub fn power(&self) -> f32 {
        self.inf * 1.0 + self.tanks * 4.5 + self.ifv * 2.2 + self.arty * 1.2
    }
}

pub struct Building {
    pub kind: BKind,
    pub x: i32,
    pub y: i32,
    pub hp: f32,
    pub max_hp: f32,
    pub build_t: f32,
    pub build_p: f32,
    pub powered: bool,
    pub covered: bool,
    pub connected: bool,
    pub active: bool,
    pub prod_t: f32,
    pub burning: f32,
    pub alive: bool,
    pub hit: f32,
}

impl Building {
    pub fn built(&self) -> bool {
        self.build_p >= 1.0
    }
    pub fn progress(&self) -> f32 {
        self.build_p.clamp(0.0, 1.0)
    }
}

pub struct Threat {
    pub kind: ThreatKind,
    pub t: f32,
    pub total: f32,
    pub sector: usize,
    pub power: f32,
    pub targets: Vec<(i32, i32)>,
    pub revealed: bool,
    pub count: i32,
}

pub struct Target {
    pub name: &'static str,
    pub name_ru: &'static str,
    pub garrison: f32,
    pub captured: bool,
    pub dir: &'static str,
    pub bonus: &'static str,
}

pub struct Toast {
    pub text: String,
    pub color: Color,
    pub t: f32,
    pub icon: &'static str,
}

pub struct DroneVis {
    pub x: f32,
    pub y: f32,
    pub sx: f32,
    pub sy: f32,
    pub tx: f32,
    pub ty: f32,
    pub p: f32,
    pub speed: f32,
    pub intercepted: bool,
    pub dead: bool,
    pub kind: u8,
    pub delay: f32,
}

#[derive(Default)]
pub struct Stats {
    pub shells: u32,
    pub drones_down: u32,
    pub drones_leak: u32,
    pub assaults: u32,
    pub killed: u32,
    pub lost: u32,
    pub captured: usize,
    pub built: u32,
    pub fuel_made: f32,
}

pub struct Enemy {
    pub next: f32,
    pub battery_alive: bool,
    pub battery_t: f32,
    pub wave: usize,
    pub wave_penalty: f32,
    pub pressure: f32,
    pub threat: Option<Threat>,
    pub intel: bool,
}

pub struct Abilities {
    pub counter_cd: f32,
    pub strike_cd: f32,
    pub recon_cd: f32,
    pub fpv_cd: f32,
}

pub struct Game {
    pub faction: Faction,
    pub screen: Screen,
    pub map: Map,
    pub buildings: Vec<Building>,
    pub res: Res,
    pub army: Army,
    pub stats: Stats,

    pub time: f32,
    pub speed: f32,
    pub paused: bool,
    pub weather: Weather,
    pub weather_t: f32,

    pub pop: f32,
    pub pop_cap: f32,
    pub morale: f32,
    pub tcc: f32,
    pub integrity: f32,
    pub power_cap: f32,
    pub power_use: f32,
    pub power_eff: f32,
    pub connected_derricks: usize,
    pub derricks: usize,
    pub pipe_net: Vec<bool>,

    pub enemy: Enemy,
    pub ab: Abilities,
    pub targets: Vec<Target>,
    pub focus: Option<usize>,
    pub redeploy: f32,
    pub assault_cd: f32,

    pub sel: Option<usize>,
    pub build: Option<BKind>,
    pub pipe_mode: bool,
    pub pipe_last: Option<(i32, i32)>,
    pub cat: usize,
    pub tab: usize,
    pub overlay: bool,
    pub log_open: bool,
    pub log_lines: Vec<(String, Color)>,
    pub toasts: Vec<Toast>,
    pub hints: [bool; 10],

    pub cam: Vec2,
    pub zoom: f32,
    pub hover: (i32, i32),
    pub fx: Fx,
    pub drones: Vec<DroneVis>,
    pub rng: Rng,
    pub convoy_t: f32,
    pub frame_t: f32,
    pub end_reason: String,
    pub title_sel: usize,
    pub end_t: f32,
    pub tutorial: f32,
    pub ambient: Color,
}

impl Game {
    pub fn new(faction: Faction) -> Self {
        let seed = if faction == Faction::Ua { 20261121 } else { 20261122 };
        let map = Map::generate(seed);
        let mut g = Game {
            faction,
            screen: Screen::Title,
            map,
            buildings: Vec::new(),
            res: Res::start(faction),
            army: Army { inf: 8.0, tanks: 2.0, ifv: 2.0, arty: 2.0 },
            stats: Stats::default(),
            time: 8.0,
            speed: 1.0,
            paused: false,
            weather: Weather::Clear,
            weather_t: 4.0,
            pop: 62.0,
            pop_cap: 80.0,
            morale: 68.0,
            tcc: 35.0,
            integrity: 100.0,
            power_cap: 0.0,
            power_use: 0.0,
            power_eff: 1.0,
            connected_derricks: 0,
            derricks: 0,
            pipe_net: vec![false; (MAP * MAP) as usize],
            enemy: Enemy { next: 11.0, battery_alive: true, battery_t: 0.0, wave: 0, wave_penalty: 1.0, pressure: 0.0, threat: None, intel: false },
            ab: Abilities { counter_cd: 0.0, strike_cd: 0.0, recon_cd: 0.0, fpv_cd: 0.0 },
            targets: vec![
                Target { name: "ЗАРЕЧНОЕ", name_ru: "Заречное", garrison: 12.0, captured: false, dir: "ВОСТОК", bonus: "+250 ГРН, +25 СТАЛИ" },
                Target { name: "НОВОСЁЛОВКА", name_ru: "Новосёловка", garrison: 18.0, captured: false, dir: "СЕВЕРО-ВОСТОК", bonus: "+15 ТОПЛ, +10 ДУХ" },
                Target { name: "КЛЕНОВОЕ", name_ru: "Кленовое", garrison: 26.0, captured: false, dir: "ЮГО-ВОСТОК", bonus: "+40 СТАЛИ, +20 ЛЮД" },
            ],
            focus: None,
            redeploy: 0.0,
            assault_cd: 0.0,
            sel: None,
            build: None,
            pipe_mode: false,
            pipe_last: None,
            cat: 0,
            tab: 0,
            overlay: false,
            log_open: false,
            log_lines: Vec::new(),
            toasts: Vec::new(),
            hints: [false; 10],
            cam: Map::iso(20, 20) + vec2(0.0, -20.0),
            zoom: 1.0,
            hover: (20, 20),
            fx: Fx::new(),
            drones: Vec::new(),
            rng: Rng::new(seed ^ 0x9e3779b9),
            convoy_t: 5.0,
            frame_t: 0.0,
            end_reason: String::new(),
            title_sel: 0,
            end_t: 0.0,
            tutorial: 0.0,
            ambient: WHITE,
        };
        g.found_village();
        g.log("Село Соколовка. До подхода врага — 3 дня.", art::col(art::TEXT));
        g
    }

    fn put(&mut self, kind: BKind, x: i32, y: i32) {
        if let Some(t) = self.map.get_mut(x, y) {
            t.terrain = if kind == BKind::Derrick { Terrain::Oil } else { Terrain::Dirt };
            t.deco = None;
            t.building = Some(self.buildings.len());
        }
        let d = kind.def();
        self.buildings.push(Building {
            kind,
            x,
            y,
            hp: d.hp,
            max_hp: d.hp,
            build_t: 0.0,
            build_p: 1.0,
            powered: true,
            covered: true,
            connected: false,
            active: true,
            prod_t: 0.0,
            burning: 0.0,
            alive: true,
            hit: 0.0,
        });
    }

    fn found_village(&mut self) {
        // Industrial pad in the south-west: clear scrub along the future pipeline.
        let (ax, ay) = (11.0f32, 33.0f32);
        let (bx, by) = (14.0f32, 28.0f32);
        let (vx, vy) = (bx - ax, by - ay);
        for y in 27..=35 {
            for x in 9..=17 {
                let px = x as f32;
                let py = y as f32;
                let t = (((px - ax) * vx + (py - ay) * vy) / (vx * vx + vy * vy)).clamp(0.0, 1.0);
                let dx = px - (ax + t * vx);
                let dy = py - (ay + t * vy);
                let near_path = dx * dx + dy * dy <= 5.5;
                let near_sub = (px - 17.0).abs() <= 1.0 && (py - 31.0).abs() <= 1.0;
                let near_yard = (px - 12.0).abs() <= 2.0 && (py - 31.0).abs() <= 2.0;
                if !(near_path || near_sub || near_yard) {
                    continue;
                }
                if let Some(t) = self.map.get_mut(x, y) {
                    if matches!(t.terrain, Terrain::Grass | Terrain::Field | Terrain::Forest) {
                        t.terrain = Terrain::Dirt;
                        t.deco = None;
                    }
                }
            }
        }
        // The core of the village.
        self.put(BKind::Hq, 23, 16);
        for (x, y) in [(15, 16), (16, 17), (21, 16), (24, 18), (17, 22), (22, 23)] {
            self.put(BKind::House, x, y);
        }
        self.put(BKind::Shop, 22, 21);
        self.put(BKind::Tcc, 18, 18);
        self.put(BKind::Farm, 15, 22);
        self.put(BKind::Watertower, 24, 26);
        self.put(BKind::Diesel, 22, 26);
        self.put(BKind::Solar, 26, 18);
        self.put(BKind::Derrick, 11, 33);
        self.put(BKind::Refinery, 14, 28);
        self.put(BKind::Substation, 17, 31);
        self.put(BKind::Aa, 25, 21);
        self.put(BKind::Trench, 28, 19);
        self.put(BKind::Trench, 28, 21);
        for (x, y, d) in [(10, 34, "rock"), (16, 30, "bush"), (15, 33, "wreck"), (10, 29, "bush"), (18, 32, "rock")] {
            if let Some(t) = self.map.get_mut(x, y) {
                if t.building.is_none() && !t.pipe {
                    t.deco = Some(d);
                }
            }
        }
        self.log("Штаб развёрнут. Наладьте нефть, энергию и оборону.", art::col(art::TEXT));
    }

    pub fn start(&mut self, faction: Faction) {
        *self = Game::new(faction);
        self.screen = Screen::Play;
        self.sel = None;
        self.start_hints();
    }

    fn start_hints(&mut self) {
        let (a, b) = if self.faction == Faction::Ua {
            ("Совет: постройте трубы от вышки к НПЗ.", "Совет: до первой атаки нужны казарма и ПВО.")
        } else {
            ("РФ: поставки с задержками, армия слабее.", "Совет: экономьте топливо и БК, стройте ПВО.")
        };
        self.log(a, art::col(art::AMBER));
        self.log(b, art::col(art::TEXT_DIM));
    }

    pub fn log(&mut self, s: &str, c: Color) {
        self.log_lines.push((s.to_string(), c));
        if self.log_lines.len() > 60 {
            self.log_lines.remove(0);
        }
        self.toasts.push(Toast { text: s.to_string(), color: c, t: 7.0, icon: "" });
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    fn hint(&mut self, i: usize, s: &str) {
        if !self.hints[i] {
            self.hints[i] = true;
            self.log(s, art::col(art::AMBER));
        }
    }

    pub fn day(&self) -> i32 {
        (self.time / 24.0).floor() as i32 + 1
    }
    pub fn clock(&self) -> f32 {
        self.time % 24.0
    }
    pub fn time_str(&self) -> String {
        let c = self.clock();
        format!("Д{} {:02}:{:02}", self.day(), c as i32, ((c.fract()) * 60.0) as i32)
    }
    pub fn left_hours(&self) -> f32 {
        (TOTAL_HOURS - self.time).max(0.0)
    }
    pub fn is_day(&self) -> bool {
        let c = self.clock();
        (6.5..19.5).contains(&c)
    }

    pub fn has(&self, k: BKind) -> bool {
        self.buildings.iter().any(|b| b.alive && b.kind == k && b.built())
    }
    pub fn count(&self, k: BKind) -> usize {
        self.buildings.iter().filter(|b| b.alive && b.kind == k).count()
    }
    pub fn building_at(&self, x: i32, y: i32) -> Option<usize> {
        self.map.get(x, y).and_then(|t| t.building).filter(|&i| self.buildings[i].alive)
    }
    pub fn hq(&self) -> Option<&Building> {
        self.buildings.iter().find(|b| b.alive && b.kind == BKind::Hq)
    }

    pub fn total_power_demand(&self) -> f32 {
        self.buildings.iter().filter(|b| b.alive && b.built() && b.kind.needs_power()).map(|b| b.kind.power_demand()).sum()
    }
    pub fn free_power(&self) -> f32 {
        (self.power_cap - self.power_use).max(0.0)
    }
    pub fn fort_power(&self) -> f32 {
        let mut f = 0.0;
        for b in &self.buildings {
            if !b.alive || !b.built() {
                continue;
            }
            f += match b.kind {
                BKind::Trench => 2.0,
                BKind::Bunker => 4.0,
                BKind::Mines => 3.0,
                BKind::Barracks => 2.0,
                BKind::Hq => 3.0,
                _ => 0.0,
            };
        }
        f
    }
    pub fn army_defense(&self, sector: usize) -> f32 {
        let mult = if self.redeploy > 0.0 {
            0.82
        } else if let Some(f) = self.focus {
            if f == sector {
                1.0
            } else {
                0.5
            }
        } else {
            0.72
        };
        let supply = if self.res.ammo > 0.0 { 1.0 } else { 0.55 };
        let morale_f = 0.7 + self.morale / 200.0;
        (self.army.power() * mult + self.fort_power()) * morale_f * supply
    }
    pub fn sector_name(i: usize) -> &'static str {
        ["СЕВЕР", "ЦЕНТР", "ЮГ"][i.min(2)]
    }

    pub fn can_afford(&self, k: BKind) -> bool {
        let d = k.def();
        self.res.money >= d.money && self.res.steel >= d.steel && self.res.fuel >= d.fuel
    }

    pub fn cost_str(&self, k: BKind) -> String {
        let d = k.def();
        let mut s = String::new();
        if d.money > 0.0 {
            s.push_str(&format!("{:.0}", d.money));
        }
        if d.steel > 0.0 {
            s.push_str(&format!(" +{}ст", d.steel));
        }
        if d.fuel > 0.0 {
            s.push_str(&format!(" +{}т", d.fuel));
        }
        s
    }

    pub fn try_place(&mut self, k: BKind, x: i32, y: i32) -> Result<(), String> {
        let d = k.def();
        if k == BKind::Hq {
            return Err("Штаб уже стоит".into());
        }
        if let Some(t) = self.map.get(x, y) {
            if t.building.is_some() && k != BKind::Pipe {
                return Err("Клетка занята".into());
            }
            if k == BKind::Derrick && t.terrain != Terrain::Oil {
                return Err("Вышку ставят только на нефтяную жилу".into());
            }
            if k != BKind::Derrick && t.terrain == Terrain::Oil {
                return Err("На жиле можно ставить только вышку".into());
            }
            if k == BKind::Pipe {
                if t.pipe {
                    return Err("Труба уже есть".into());
                }
                if !t.terrain.buildable() {
                    return Err("Здесь труба не пройдёт".into());
                }
                if t.building.is_some() {
                    return Err("Клетка занята".into());
                }
            } else {
                self.map.can_place(x, y)?;
            }
        } else {
            return Err("За границей карты".into());
        }
        if !self.can_afford(k) {
            return Err("Не хватает ресурсов".into());
        }
        self.res.money -= d.money;
        self.res.steel -= d.steel;
        self.res.fuel -= d.fuel;
        if k == BKind::Pipe {
            if let Some(t) = self.map.get_mut(x, y) {
                t.pipe = true;
                t.deco = None;
            }
            self.fx.dust(Map::iso(x, y).x, Map::iso(x, y).y);
            self.stats.built += 1;
            self.compute_pipes();
            if !self.hints[0] {
                self.hint(0, "Труба проложена. Доведите её до НПЗ.");
            }
            return Ok(());
        }
        // Clear scenery under the new building.
        if let Some(t) = self.map.get_mut(x, y) {
            t.deco = None;
            t.building = Some(self.buildings.len());
        }
        let bt = 0.6 + d.money / 400.0;
        self.buildings.push(Building {
            kind: k,
            x,
            y,
            hp: d.hp * 0.5,
            max_hp: d.hp,
            build_t: bt,
            build_p: 0.0,
            powered: false,
            covered: false,
            connected: false,
            active: true,
            prod_t: 0.0,
            burning: 0.0,
            alive: true,
            hit: 0.0,
        });
        self.stats.built += 1;
        let p = Map::iso(x, y);
        self.fx.dust(p.x, p.y);
        self.log(&format!("Стройка: {}", d.full), art::col(art::TEXT_DIM));
        Ok(())
    }

    pub fn demolish(&mut self, idx: usize) {
        if idx >= self.buildings.len() {
            return;
        }
        let b = &self.buildings[idx];
        if !b.alive {
            return;
        }
        let (x, y, kind) = (b.x, b.y, b.kind);
        if kind == BKind::Hq {
            return;
        }
        self.buildings[idx].alive = false;
        if let Some(t) = self.map.get_mut(x, y) {
            t.building = None;
            t.terrain = Terrain::Dirt;
        }
        let d = kind.def();
        self.res.money += d.money * 0.4;
        self.res.steel += d.steel * 0.4;
        self.log(&format!("Разобрано: {}", d.full), art::col(art::TEXT_DIM));
    }

    pub fn compute_pipes(&mut self) {
        let n = (MAP * MAP) as usize;
        let mut net = vec![false; n];
        let mut q = VecDeque::new();
        let refineries: HashSet<usize> = self
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.alive && b.kind == BKind::Refinery && b.built())
            .map(|(i, _)| i)
            .collect();
        // Seed from pipes adjacent to a refinery.
        for &ri in &refineries {
            let (bx, by) = (self.buildings[ri].x, self.buildings[ri].y);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (x, y) = (bx + dx, by + dy);
                if let Some(t) = self.map.get(x, y) {
                    if t.pipe {
                        if let Some(i) = Map::idx(x, y) {
                            if !net[i] {
                                net[i] = true;
                                q.push_back((x, y));
                            }
                        }
                    }
                }
            }
        }
        // Flood the connected pipe network.
        while let Some((x, y)) = q.pop_front() {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if let Some(t) = self.map.get(nx, ny) {
                    if t.pipe {
                        if let Some(i) = Map::idx(nx, ny) {
                            if !net[i] {
                                net[i] = true;
                                q.push_back((nx, ny));
                            }
                        }
                    }
                }
            }
        }
        let mut derricks = 0;
        let mut connected = 0;
        let derrick_ids: Vec<usize> = self
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.alive && b.kind == BKind::Derrick && b.built())
            .map(|(i, _)| i)
            .collect();
        for &di in &derrick_ids {
            let (bx, by) = (self.buildings[di].x, self.buildings[di].y);
            derricks += 1;
            let mut ok = false;
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (x, y) = (bx + dx, by + dy);
                if let Some(t) = self.map.get(x, y) {
                    if t.pipe {
                        if let Some(i) = Map::idx(x, y) {
                            if net[i] {
                                ok = true;
                            }
                        }
                    }
                    if let Some(bi) = t.building {
                        if refineries.contains(&bi) {
                            ok = true;
                        }
                    }
                }
            }
            self.buildings[di].connected = ok;
            if ok {
                connected += 1;
            }
        }
        self.pipe_net = net;
        // Mark refineries that actually touch the pipe network.
        let ref_list: Vec<usize> = refineries.iter().copied().collect();
        for ri in ref_list {
            let (bx, by) = (self.buildings[ri].x, self.buildings[ri].y);
            let mut ok = false;
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if let Some(t) = self.map.get(bx + dx, by + dy) {
                    if t.pipe {
                        if let Some(i) = Map::idx(bx + dx, by + dy) {
                            if self.pipe_net[i] {
                                ok = true;
                            }
                        }
                    }
                }
            }
            self.buildings[ri].connected = ok;
        }
        self.derricks = derricks;
        self.connected_derricks = connected;
    }

    fn refresh_power(&mut self, dt_h: f32) {
        let is_day = self.is_day();
        let solar_f = self.weather.solar();
        let mut buildings = std::mem::take(&mut self.buildings);
        let mut centers: Vec<(f32, f32)> = Vec::new();
        for b in &buildings {
            if !b.alive || !b.built() {
                continue;
            }
            if matches!(b.kind, BKind::Diesel | BKind::Tes | BKind::Solar | BKind::Substation) {
                centers.push((b.x as f32, b.y as f32));
            }
        }
        let covered = |x: i32, y: i32| -> bool {
            centers.iter().any(|&(cx, cy)| {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                dx * dx + dy * dy <= 49.0
            })
        };
        let mut cap = 0.0;
        let mut fuel = self.res.fuel;
        for b in &mut buildings {
            if !b.alive || !b.built() {
                continue;
            }
            b.covered = true;
            if matches!(b.kind, BKind::Diesel | BKind::Tes | BKind::Solar) {
                let mut out = match b.kind {
                    BKind::Diesel => 12.0,
                    BKind::Tes => 30.0,
                    BKind::Solar => {
                        if is_day {
                            10.0 * solar_f
                        } else {
                            0.0
                        }
                    }
                    _ => 0.0,
                };
                if b.kind != BKind::Solar && out > 0.0 {
                    let burn = if b.kind == BKind::Diesel { 0.6 } else { 1.6 } * dt_h;
                    if fuel >= burn {
                        fuel -= burn;
                        b.prod_t += out * dt_h;
                    } else if fuel > 0.0 {
                        let part = fuel / burn;
                        fuel = 0.0;
                        out *= part * 0.7;
                        b.prod_t += out * dt_h;
                    } else {
                        out = 0.0;
                    }
                }
                cap += out;
            }
            if b.kind.needs_power() {
                b.covered = covered(b.x, b.y);
            }
        }
        self.res.fuel = fuel.max(0.0);
        let mut use_p = 0.0;
        for b in &buildings {
            if b.alive && b.built() && b.kind.needs_power() && b.covered {
                use_p += b.kind.power_demand();
            }
        }
        let eff = if use_p > 0.01 { (cap / use_p).clamp(0.0, 1.0) } else { 1.0 };
        for b in &mut buildings {
            if b.alive && b.built() && b.kind.needs_power() {
                b.powered = b.covered && eff > 0.03;
            } else if b.alive {
                b.powered = true;
            }
        }
        self.buildings = buildings;
        self.power_cap = cap;
        self.power_use = use_p;
        self.power_eff = eff;
    }

    fn tick_econ(&mut self, dt_h: f32) {
        self.refresh_power(dt_h);
        let eff = self.power_eff;
        let mut crude_add = 0.0;
        let mut fuel_add = 0.0;
        let mut food_add = 0.0;
        let mut water_add = 0.0;
        let mut steel_add = 0.0;
        let mut money_add = 0.0;
        let mut pop_gain = 0.0;
        let mut prod_events: Vec<(usize, u8)> = Vec::new();

        let mut buildings = std::mem::take(&mut self.buildings);
        for (i, b) in buildings.iter_mut().enumerate() {
            if !b.alive {
                continue;
            }
            b.hit = (b.hit - dt_h * 4.0).max(0.0);
            if b.build_p < 1.0 {
                b.build_p = (b.build_p + dt_h / b.build_t.max(0.1)).min(1.0);
                if b.build_p >= 1.0 {
                    b.hp = b.max_hp;
                    prod_events.push((i, 255));
                }
                continue;
            }
            if b.burning > 0.0 {
                b.burning -= dt_h;
                b.hp -= 2.5 * dt_h;
            }
            if b.hp <= 0.0 {
                prod_events.push((i, 254));
                continue;
            }
            if !b.active || !b.powered {
                continue;
            }
            match b.kind {
                BKind::Derrick => {
                    if b.connected {
                        crude_add += 5.0 * dt_h;
                    }
                }
                BKind::Refinery => {
                    if b.connected && self.res.crude + crude_add > 0.05 {
                        let r = if b.powered { 1.0 } else { 0.35 };
                        let d = (6.0 * dt_h * r).min(self.res.crude + crude_add);
                        crude_add -= d;
                        fuel_add += d * 0.62;
                    }
                }
                BKind::Farm => {
                    if self.res.water > 0.1 {
                        food_add += 4.5 * dt_h * (0.35 + 0.65 * eff);
                        water_add -= 0.5 * dt_h;
                    }
                }
                BKind::Watertower => {
                    water_add += 7.0 * dt_h * (0.35 + 0.65 * eff);
                }
                BKind::Metalshop => {
                    let cost = 3.0 * dt_h;
                    if self.res.money > cost {
                        money_add -= cost;
                        steel_add += 1.2 * dt_h * (0.4 + 0.6 * eff);
                    }
                }
                BKind::Shop => {
                    money_add += (4.0 + self.pop * 0.035) * dt_h;
                }
                BKind::Barracks => {
                    b.prod_t += dt_h;
                    if b.prod_t >= 2.5 {
                        let can = self.res.manpower >= 2.0 && self.res.ammo >= 3.0 && self.res.money >= 15.0 && self.pop >= 4.0;
                        if can {
                            b.prod_t -= 2.5;
                            prod_events.push((i, 1));
                        } else {
                            b.prod_t = 2.5;
                        }
                    }
                }
                BKind::TankFactory => {
                    b.prod_t += dt_h;
                    if b.prod_t >= 5.0 {
                        let can = self.res.steel >= 12.0 && self.res.fuel >= 8.0 && self.res.money >= 90.0;
                        if can {
                            b.prod_t -= 5.0;
                            prod_events.push((i, 2));
                        } else {
                            b.prod_t = 5.0;
                        }
                    }
                }
                BKind::DroneHangar => {
                    b.prod_t += dt_h;
                    if b.prod_t >= 2.0 {
                        let can = self.res.steel >= 2.0 && self.res.money >= 40.0;
                        if can {
                            b.prod_t -= 2.0;
                            prod_events.push((i, 3));
                        } else {
                            b.prod_t = 2.0;
                        }
                    }
                }
                BKind::Repair => {
                    b.prod_t += dt_h;
                    if b.prod_t >= 1.0 {
                        b.prod_t -= 1.0;
                        prod_events.push((i, 4));
                    }
                }
                _ => {}
            }
        }
        self.buildings = buildings;

        self.res.crude = (self.res.crude + crude_add).clamp(0.0, 120.0);
        self.res.fuel = (self.res.fuel + fuel_add).clamp(0.0, 200.0);
        self.stats.fuel_made += fuel_add.max(0.0);
        self.res.food = (self.res.food + food_add).clamp(0.0, 300.0);
        self.res.water = (self.res.water + water_add).clamp(0.0, 300.0);
        self.res.steel = (self.res.steel + steel_add).clamp(0.0, 300.0);
        self.res.money = (self.res.money + money_add).clamp(0.0, 5000.0);

        for (i, ev) in prod_events {
            match ev {
                255 => {
                    let d = self.buildings[i].kind.def();
                    self.log(&format!("Построено: {}", d.full), art::col(art::GREEN));
                }
                254 => self.destroy_building(i),
                1 => {
                    self.army.inf += 1.0;
                    self.res.manpower -= 2.0;
                    self.res.ammo -= 3.0;
                    self.res.money -= 15.0;
                    self.pop -= 4.0;
                    self.log("Казарма: +1 отделение пехоты", art::col(art::GREEN));
                }
                2 => {
                    self.army.tanks += 1.0;
                    self.res.steel -= 12.0;
                    self.res.fuel -= 8.0;
                    self.res.money -= 90.0;
                    self.log("Танковый завод: готов танк", art::col(art::GREEN));
                }
                3 => {
                    self.res.drones += 2.0;
                    self.res.steel -= 2.0;
                    self.res.money -= 40.0;
                    self.log("Ангар: +2 FPV-дрона", art::col(art::GREEN));
                }
                4 => self.repair_tick(),
                _ => {}
            }
        }

        // People.
        let tax = self.pop * 0.34 * dt_h;
        self.res.money += tax;
        self.pop_cap = 20.0 + self.count(BKind::House) as f32 * 10.0 + self.stats.captured as f32 * 12.0;
        let food_need = self.pop * 0.05 * dt_h;
        let water_need = self.pop * 0.05 * dt_h;
        let food_ok = self.res.food >= food_need;
        let water_ok = self.res.water >= water_need;
        self.res.food = (self.res.food - food_need).max(0.0);
        self.res.water = (self.res.water - water_need).max(0.0);

        let mut target = 52.0;
        target += self.count(BKind::Church) as f32 * 5.0;
        target += self.count(BKind::School) as f32 * 4.0;
        target += self.count(BKind::Hospital) as f32 * 4.0;
        target += self.count(BKind::Shop) as f32 * 1.5;
        target += if self.has(BKind::Hq) { 4.0 } else { -25.0 };
        target -= self.tcc * 0.12;
        target += self.stats.captured as f32 * 3.0;
        if !food_ok {
            target -= 26.0;
        }
        if !water_ok {
            target -= 20.0;
        }
        if self.power_eff < 0.85 {
            target -= 8.0;
        }
        let target = target.clamp(0.0, 100.0);
        let rate = if target > self.morale { 1.6 } else { 2.6 };
        self.morale += (target - self.morale).clamp(-rate * dt_h, rate * dt_h);
        self.morale = self.morale.clamp(0.0, 100.0);

        if self.morale > 52.0 && food_ok && water_ok && self.pop < self.pop_cap {
            pop_gain += (0.35 + self.count(BKind::House) as f32 * 0.03) * dt_h * (self.morale / 90.0);
        }
        if self.morale < 35.0 {
            pop_gain -= 0.4 * dt_h;
        }
        if !food_ok || !water_ok {
            pop_gain -= 0.7 * dt_h;
        }
        self.pop = (self.pop + pop_gain).clamp(0.0, self.pop_cap);

        // Mobilisation.
        let mob = (0.15 + self.tcc / 100.0 * 1.5 * (self.pop / 120.0).clamp(0.25, 1.4) * (0.55 + self.morale / 150.0)) * dt_h;
        self.res.manpower = (self.res.manpower + mob).clamp(0.0, 200.0);
        self.pop -= self.tcc / 100.0 * 0.30 * dt_h;

        if self.has(BKind::Repair) && self.integrity < 100.0 {
            self.integrity = (self.integrity + 0.35 * dt_h).min(100.0);
        }

        self.ab.counter_cd = (self.ab.counter_cd - dt_h).max(0.0);
        self.ab.strike_cd = (self.ab.strike_cd - dt_h).max(0.0);
        self.ab.recon_cd = (self.ab.recon_cd - dt_h).max(0.0);
        self.ab.fpv_cd = (self.ab.fpv_cd - dt_h).max(0.0);
        self.redeploy = (self.redeploy - dt_h).max(0.0);
        self.assault_cd = (self.assault_cd - dt_h).max(0.0);
        self.enemy.battery_t = (self.enemy.battery_t - dt_h).max(0.0);
        if self.enemy.battery_t <= 0.0 {
            self.enemy.battery_alive = true;
        }

        // Convoy.
        self.convoy_t -= dt_h;
        if self.convoy_t <= 0.0 {
            self.convoy();
            self.convoy_t = 6.0;
        }

        // Weather.
        self.weather_t -= dt_h;
        if self.weather_t <= 0.0 {
            self.weather_t = 5.0 + self.rng.range(0.0, 4.0);
            self.weather = match (self.rng.f() * 4.0) as i32 {
                0 => Weather::Clear,
                1 => Weather::Rain,
                2 => Weather::Fog,
                _ => Weather::Storm,
            };
            self.log(&format!("Погода: {}", self.weather.name()), art::col(art::TEXT_DIM));
        }

        if self.pop < 1.0 {
            self.lose("Село опустело");
        }
        if self.morale <= 0.5 && self.time > 12.0 {
            self.lose("Дух села сломлен");
        }
        if let Some(b) = self.hq() {
            if b.hp <= 0.0 || !b.alive {
                self.lose("Штаб уничтожен");
            }
        } else if self.time > 1.0 {
            self.lose("Штаб уничтожен");
        }
    }

    fn convoy(&mut self) {
        let mut k = if self.faction == Faction::Ua { 1.0 } else { 0.78 };
        let mut delay = false;
        if self.faction == Faction::Ru && self.rng.f() < 0.35 {
            delay = true;
            k *= 0.5;
            self.convoy_t += 1.5;
            self.log("Конвой РФ задержан на складе. Половина груза.", art::col(art::RED));
        }
        self.res.money += 110.0 * k;
        self.res.fuel += 16.0 * k;
        self.res.food += 20.0 * k;
        self.res.water += 18.0 * k;
        self.res.steel += 10.0 * k;
        self.res.ammo += 20.0 * k;
        if self.rng.f() < 0.35 * k {
            self.res.drones += 1.0;
        }
        if !delay {
            self.log("Конвой доставил снабжение в село", art::col(art::GREEN));
        }
    }

    fn repair_tick(&mut self) {
        let mut worst: Option<usize> = None;
        let mut worst_frac = 1.0;
        for (i, b) in self.buildings.iter().enumerate() {
            if b.alive && b.built() && b.hp < b.max_hp {
                let f = b.hp / b.max_hp;
                if f < worst_frac {
                    worst_frac = f;
                    worst = Some(i);
                }
            }
        }
        if let Some(i) = worst {
            self.buildings[i].hp = (self.buildings[i].hp + 14.0).min(self.buildings[i].max_hp);
            self.buildings[i].burning = 0.0;
        }
    }

    pub fn damage_building(&mut self, idx: usize, amount: f32) {
        if idx >= self.buildings.len() {
            return;
        }
        if !self.buildings[idx].alive {
            return;
        }
        let r1 = self.rng.f();
        let r2 = self.rng.range(0.0, 1.5);
        let mut dead = false;
        {
            let b = &mut self.buildings[idx];
            b.hp -= amount;
            b.hit = 1.0;
            if b.hp < b.max_hp * 0.4 && b.burning <= 0.0 && r1 < 0.5 {
                b.burning = 1.0 + r2;
            }
            if b.hp <= 0.0 {
                dead = true;
            }
        }
        if dead {
            let (x, y) = (self.buildings[idx].x, self.buildings[idx].y);
            let p = Map::iso(x, y);
            self.destroy_building(idx);
            self.fx.boom(p.x, p.y, 1.4);
        }
    }

    fn destroy_building(&mut self, idx: usize) {
        if idx >= self.buildings.len() || !self.buildings[idx].alive {
            return;
        }
        let b = &mut self.buildings[idx];
        b.alive = false;
        b.hp = 0.0;
        let (x, y, kind) = (b.x, b.y, b.kind);
        if let Some(t) = self.map.get_mut(x, y) {
            t.building = None;
            t.terrain = Terrain::Rubble;
            t.deco = Some("rubble");
        }
        let p = Map::iso(x, y);
        self.fx.boom(p.x, p.y, 1.1);
        self.fx.smoke(p.x, p.y, 7);
        self.integrity = (self.integrity - 1.0).max(0.0);
        if kind == BKind::Hq {
            self.lose("Штаб уничтожен");
        } else {
            let d = kind.def();
            self.log(&format!("Разрушено: {}", d.full), art::col(art::RED));
        }
        if self.sel == Some(idx) {
            self.sel = None;
        }
        self.compute_pipes();
    }

    pub fn set_focus(&mut self, s: usize) {
        if self.focus != Some(s) {
            self.focus = Some(s);
            self.redeploy = 0.5;
            self.log(&format!("Переброска сил: {}", Self::sector_name(s)), art::col(art::AMBER));
        }
    }

    pub fn counter_battery(&mut self) -> Result<(), String> {
        if !self.has(BKind::Radar) {
            return Err("Нужен радар".into());
        }
        if self.res.ammo < 15.0 {
            return Err("Нужно 15 БК".into());
        }
        if self.ab.counter_cd > 0.0 {
            return Err("Расчёт перезаряжается".into());
        }
        self.res.ammo -= 15.0;
        self.ab.counter_cd = 3.0;
        self.enemy.battery_alive = false;
        self.enemy.battery_t = 8.0;
        self.morale = (self.morale + 2.0).min(100.0);
        self.stats.killed += 2;
        self.log("Контрбатарея: вражеская артиллерия подавлена", art::col(art::GREEN));
        Ok(())
    }

    pub fn deep_strike(&mut self) -> Result<(), String> {
        if self.res.drones < 3.0 || self.res.ammo < 10.0 {
            return Err("Нужно 3 дрона и 10 БК".into());
        }
        if self.ab.strike_cd > 0.0 {
            return Err("Дроны на перезарядке".into());
        }
        self.res.drones -= 3.0;
        self.res.ammo -= 10.0;
        self.ab.strike_cd = 4.0;
        self.enemy.wave_penalty *= 0.78;
        self.morale = (self.morale + 2.0).min(100.0);
        self.fx.flash = 0.4;
        self.fx.flash_col = art::col(art::AMBER);
        self.log("Глубокий удар: склад и техника врага горят", art::col(art::GREEN));
        Ok(())
    }

    pub fn recon(&mut self) -> Result<(), String> {
        if self.res.drones < 1.0 {
            return Err("Нужен дрон".into());
        }
        if self.ab.recon_cd > 0.0 {
            return Err("Разведка уже летала".into());
        }
        self.res.drones -= 1.0;
        self.ab.recon_cd = 2.0;
        if let Some(t) = &mut self.enemy.threat {
            t.revealed = true;
        } else {
            self.enemy.intel = true;
        }
        self.log("Разведка: направление удара вскрыто", art::col(art::CYAN));
        Ok(())
    }

    pub fn fpv_sortie(&mut self) -> Result<(), String> {
        if self.res.drones < 1.0 {
            return Err("Нужен дрон".into());
        }
        if self.ab.fpv_cd > 0.0 {
            return Err("FPV на перезарядке".into());
        }
        self.res.drones -= 1.0;
        self.ab.fpv_cd = 0.5;
        self.enemy.wave_penalty *= 0.94;
        self.stats.killed += 1;
        self.morale = (self.morale + 0.8).min(100.0);
        self.log("FPV-вылет: поражена единица техники врага", art::col(art::GREEN));
        Ok(())
    }

    pub fn repair_village(&mut self) -> Result<(), String> {
        if self.res.money < 60.0 {
            return Err("Нужно 60 грн".into());
        }
        self.res.money -= 60.0;
        self.integrity = (self.integrity + 8.0).min(100.0);
        for b in &mut self.buildings {
            if b.alive {
                b.hp = (b.hp + 12.0).min(b.max_hp);
            }
        }
        self.log("Ремонтные бригады восстановили укрепления", art::col(art::GREEN));
        Ok(())
    }

    pub fn assault_target(&mut self, i: usize) -> Result<(), String> {
        if i >= self.targets.len() || self.targets[i].captured {
            return Err("Село уже освобождено".into());
        }
        if self.assault_cd > 0.0 {
            return Err("Штурмовой отряд не готов".into());
        }
        if self.army.inf < 10.0 || self.army.tanks < 2.0 {
            return Err("Нужно 10 пехоты и 2 танка".into());
        }
        if self.res.ammo < 20.0 || self.res.fuel < 20.0 {
            return Err("Нужно 20 БК и 20 топлива".into());
        }
        let power = self.army.power() * 0.7;
        let gar = self.targets[i].garrison;
        self.army.inf -= 6.0;
        self.army.tanks -= 2.0;
        self.res.ammo -= 20.0;
        self.res.fuel -= 20.0;
        self.assault_cd = 4.0;
        let win = power * (0.9 + self.morale / 250.0) >= gar;
        if win {
            self.targets[i].captured = true;
            self.stats.captured += 1;
            self.army.inf -= 2.0;
            self.morale = (self.morale + 10.0).min(100.0);
            self.integrity = (self.integrity + 4.0).min(100.0);
            self.res.money += 200.0;
            self.res.steel += 15.0;
            if i == 1 {
                self.res.fuel += 15.0;
            }
            if i == 2 {
                self.res.steel += 25.0;
                self.pop += 10.0;
            }
            self.enemy.wave_penalty *= 0.88;
            self.log(&format!("{} освобождено! Гарнизон разбит.", self.targets[i].name_ru), art::col(art::GREEN));
            if self.targets.iter().all(|t| t.captured) {
                self.win("Все три села освобождены");
            }
        } else {
            self.army.inf -= 4.0;
            self.army.tanks -= 1.0;
            self.morale = (self.morale - 8.0).max(0.0);
            self.integrity = (self.integrity - 6.0).max(0.0);
            self.log(&format!("Штурм {} провалился: сильный гарнизон", self.targets[i].name_ru), art::col(art::RED));
        }
        Ok(())
    }

    pub fn buy_ammo(&mut self) {
        if self.res.money >= 80.0 {
            self.res.money -= 80.0;
            self.res.ammo += 12.0;
            self.log("Закуплено 12 БК", art::col(art::GREEN));
        } else {
            self.log("Нет денег на БК", art::col(art::RED));
        }
    }

    /// Dev helper: lay the initial pipeline from the derrick to the refinery.
    pub fn debug_autoplay(&mut self) {
        for (x, y) in [(12, 33), (12, 32), (13, 32), (13, 31), (13, 30), (14, 30), (14, 29)] {
            let _ = self.try_place(BKind::Pipe, x, y);
        }
    }

    // ---------- enemy ----------

    pub fn next_wave_hour(&self) -> Option<f32> {
        let times = [18.0, 40.0, 60.0];
        times.get(self.enemy.wave).copied()
    }

    fn tick_enemy(&mut self, dt_h: f32) {
        let mut fire = false;
        if let Some(t) = &mut self.enemy.threat {
            t.t -= dt_h;
            fire = t.t <= 0.0;
        }
        if fire {
            self.resolve_threat();
            return;
        }
        if self.enemy.threat.is_some() {
            return;
        }
        let times = [18.0, 40.0, 60.0];
        if self.enemy.wave < 3 && self.time >= times[self.enemy.wave] {
            self.spawn_assault();
            self.enemy.wave += 1;
        } else if self.time >= self.enemy.next {
            if self.enemy.battery_alive && self.rng.f() < 0.62 && self.time > 10.0 {
                self.spawn_arty();
            } else {
                self.spawn_drones();
            }
            self.enemy.next = self.time + 2.2 + self.rng.range(0.0, 2.2);
        }
    }

    fn pick_targets(&mut self, n: usize) -> Vec<(i32, i32)> {
        let mut v: Vec<(i32, i32)> = Vec::new();
        for _ in 0..n {
            let count = self.buildings.iter().filter(|b| b.alive && b.built()).count();
            if count == 0 {
                break;
            }
            let mut k = (self.rng.f() * count as f32) as usize;
            for b in &self.buildings {
                if !b.alive || !b.built() {
                    continue;
                }
                if k == 0 {
                    if !v.contains(&(b.x, b.y)) {
                        v.push((b.x, b.y));
                    }
                    break;
                }
                k -= 1;
            }
        }
        v
    }

    fn spawn_arty(&mut self) {
        let n = 2 + (self.time / 30.0) as usize;
        let targets = self.pick_targets(n.min(3));
        if targets.is_empty() {
            return;
        }
        let total = 0.9;
        self.enemy.threat = Some(Threat {
            kind: ThreatKind::Arty,
            t: total,
            total,
            sector: 1,
            power: 0.0,
            targets,
            revealed: true,
            count: n as i32,
        });
        self.log("ОБСТРЕЛ! Снаряды летят в село", art::col(art::RED));
        self.fx.flash = 0.15;
        self.fx.flash_col = art::col(art::RED);
    }

    fn spawn_drones(&mut self) {
        let n = 2 + (self.time / 24.0) as i32 + if self.faction == Faction::Ru { 1 } else { 0 };
        let targets = self.pick_targets(3);
        if targets.is_empty() {
            return;
        }
        let total = 1.4;
        self.enemy.threat = Some(Threat {
            kind: ThreatKind::Drones,
            t: total,
            total,
            sector: 1,
            power: 0.0,
            targets: targets.clone(),
            revealed: true,
            count: n,
        });
        self.log(&format!("Угроза: {} БПЛА «ШАХЕД» с востока", n), art::col(art::RED));
        // Visual raid.
        let mut aa = self.count(BKind::Aa) as f32;
        aa += self.count(BKind::Ew) as f32 * 0.6;
        let base = if self.faction == Faction::Ua { 0.55 } else { 0.42 };
        let chance = (base + aa * 0.10 + self.weather.intercept()).clamp(0.1, 0.95);
        for i in 0..n {
            let t = targets[i as usize % targets.len()];
            let tp = Map::iso(t.0, t.1);
            let sx = 640.0;
            let sy = tp.y - 260.0 + self.rng.range(-90.0, 90.0);
            self.drones.push(DroneVis {
                x: sx,
                y: sy,
                sx,
                sy,
                tx: tp.x,
                ty: tp.y - 18.0,
                p: 0.0,
                speed: 0.32 + self.rng.range(0.0, 0.08),
                intercepted: self.rng.f() < chance,
                dead: false,
                kind: 0,
                delay: i as f32 * 0.25,
            });
        }
    }

    fn spawn_assault(&mut self) {
        let sector = (self.rng.f() * 3.0) as usize;
        let base = [24.0, 40.0, 56.0][self.enemy.wave.min(2)];
        let mut power = base * (1.0 + self.enemy.pressure) * self.enemy.wave_penalty;
        if self.faction == Faction::Ru {
            power *= 1.18;
        }
        let revealed = self.has(BKind::Radar) || self.enemy.intel;
        self.enemy.intel = false;
        let total = 3.5;
        self.enemy.threat = Some(Threat {
            kind: ThreatKind::Assault,
            t: total,
            total,
            sector,
            power,
            targets: Vec::new(),
            revealed,
            count: 0,
        });
        self.log(
            "НАСТУПЛЕНИЕ! Враг пошёл на село. Укрепите сектор",
            art::col(art::RED),
        );
        self.fx.flash = 0.2;
        self.fx.flash_col = art::col(art::RED);
    }

    fn resolve_threat(&mut self) {
        let t = match self.enemy.threat.take() {
            Some(t) => t,
            None => return,
        };
        match t.kind {
            ThreatKind::Arty => {
                self.stats.shells += t.count as u32;
                let mut hits = 0;
                for (x, y) in &t.targets {
                    if self.rng.f() < if self.faction == Faction::Ru { 0.9 } else { 0.78 } {
                        hits += 1;
                        self.shell_impact(*x, *y);
                    } else {
                        let p = Map::iso(*x, *y);
                        self.fx.boom(p.x + 12.0, p.y + 4.0, 0.7);
                        if let Some(tile) = self.map.get_mut(*x, *y) {
                            tile.terrain = Terrain::Rubble;
                        }
                    }
                }
                self.integrity = (self.integrity - hits as f32 * 1.8).max(0.0);
                self.morale = (self.morale - hits as f32 * 1.6 - 1.0).max(0.0);
                self.log(
                    &format!("Обстрел: {} попаданий. Целостность {:.0}%", hits, self.integrity),
                    art::col(art::RED),
                );
            }
            ThreatKind::Drones => {
                let mut leaked = 0;
                for d in &self.drones {
                    if !d.intercepted {
                        leaked += 1;
                    }
                }
                let mut downed = t.count - leaked;
                if downed < 0 {
                    downed = 0;
                }
                self.drones.clear();
                self.stats.drones_down += downed as u32;
                self.stats.drones_leak += leaked as u32;
                for i in 0..leaked {
                    let tg = t.targets[i as usize % t.targets.len()];
                    let p = Map::iso(tg.0, tg.1);
                    self.fx.boom(p.x, p.y, 1.0);
                    if let Some(idx) = self.building_at(tg.0, tg.1) {
                        let hurt = if self.has(BKind::Hospital) { 22.0 } else { 32.0 };
                        self.damage_building(idx, hurt);
                    }
                    let loss = if self.has(BKind::Hospital) { 0.5 } else { 1.2 };
                    self.pop = (self.pop - loss).max(0.0);
                }
                self.morale = (self.morale - leaked as f32 * 3.0).max(0.0);
                self.integrity = (self.integrity - leaked as f32 * 1.2).max(0.0);
                if leaked > 0 {
                    self.log(&format!("БПЛА: сбито {}, прорвалось {}", downed, leaked), art::col(art::RED));
                } else {
                    self.log(&format!("ПВО СБИЛО ВСЕ ДРОНЫ: {}", downed), art::col(art::GREEN));
                }
            }
            ThreatKind::Assault => {
                let def = self.army_defense(t.sector);
                let attack = t.power * (0.9 + self.rng.range(0.0, 0.22));
                self.stats.assaults += 1;
                let sector = Self::sector_name(t.sector);
                if def >= attack {
                    let intensity = (attack / def).clamp(0.2, 1.0);
                    let inf_loss = (self.army.inf * 0.20 * intensity).ceil();
                    let tank_loss = (self.army.tanks * 0.08 * intensity).floor();
                    let ifv_loss = (self.army.ifv * 0.12 * intensity).floor();
                    self.army.inf = (self.army.inf - inf_loss).max(1.0);
                    self.army.tanks = (self.army.tanks - tank_loss).max(0.0);
                    self.army.ifv = (self.army.ifv - ifv_loss).max(0.0);
                    self.stats.lost += inf_loss as u32 + tank_loss as u32;
                    self.stats.killed += (attack * 0.8) as u32;
                    self.integrity = (self.integrity - 2.0).max(0.0);
                    self.morale = (self.morale + 6.0).min(100.0);
                    self.enemy.pressure += 0.03;
                    self.log(
                        &format!("{}: атака отбита. Потери: {:.0} пех., {:.0} тех.", sector, inf_loss, tank_loss + ifv_loss),
                        art::col(art::GREEN),
                    );
                    self.fx.text(-120.0, 120.0, "АТАКА ОТБИТА", art::col(art::GREEN));
                } else {
                    let dest = self.pick_targets(2);
                    for (x, y) in dest {
                        if let Some(idx) = self.building_at(x, y) {
                            self.damage_building(idx, 90.0);
                        }
                    }
                    self.army.inf = (self.army.inf * 0.55).max(1.0);
                    self.army.tanks = (self.army.tanks * 0.78).floor();
                    self.army.ifv = (self.army.ifv * 0.7).floor();
                    self.pop = (self.pop - 7.0).max(0.0);
                    self.integrity = (self.integrity - 18.0).max(0.0);
                    self.morale = (self.morale - 12.0).max(0.0);
                    self.stats.lost += 6;
                    self.enemy.pressure += 0.12;
                    self.log(
                        &format!("{}: позиции прорваны! Целостность {:.0}%", sector, self.integrity),
                        art::col(art::RED),
                    );
                    if self.integrity <= 0.0 {
                        self.lose("Оборона села прорвана");
                    }
                }
            }
        }
    }

    fn shell_impact(&mut self, x: i32, y: i32) {
        let p = Map::iso(x, y);
        self.fx.boom(p.x, p.y, 1.0 + self.rng.range(0.0, 0.5));
        self.fx.smoke(p.x, p.y - 6.0, 5);
        if let Some(idx) = self.building_at(x, y) {
            let dmg = if self.has(BKind::Bunker) { 34.0 } else { 48.0 } * (1.0 + self.rng.range(0.0, 0.5));
            self.damage_building(idx, dmg);
        } else if let Some(t) = self.map.get_mut(x, y) {
            t.terrain = Terrain::Rubble;
            t.deco = Some("rubble");
        }
        let loss = if self.has(BKind::Hospital) { 0.4 } else { 1.1 };
        self.pop = (self.pop - loss * (1.0 + self.rng.range(0.0, 0.7))).max(0.0);
        self.morale = (self.morale - 1.2).max(0.0);
    }

    pub fn win(&mut self, reason: &str) {
        if self.screen != Screen::Play {
            return;
        }
        self.end_reason = reason.to_string();
        self.screen = Screen::Win;
        self.end_t = 0.0;
        self.log("ПОБЕДА", art::col(art::GREEN));
    }
    pub fn lose(&mut self, reason: &str) {
        if self.screen != Screen::Play {
            return;
        }
        self.end_reason = reason.to_string();
        self.screen = Screen::Lose;
        self.end_t = 0.0;
        self.log("ПОРАЖЕНИЕ", art::col(art::RED));
    }

    // ---------- main update ----------

    pub fn update(&mut self, dt_real: f32) {
        self.fx.update(dt_real * if self.paused { 0.4 } else { self.speed.max(1.0) });
        self.frame_t += dt_real;
        for t in &mut self.toasts {
            t.t -= dt_real;
        }
        self.toasts.retain(|t| t.t > 0.0);
        if self.screen != Screen::Play {
            self.end_t += dt_real;
            self.ambient = self.compute_ambient();
            return;
        }
        let sp = if self.paused { 0.0 } else { self.speed };
        let dt_h = dt_real * sp * HOUR;
        self.update_drones(dt_real * sp);
        self.ambient = self.compute_ambient();
        if dt_h <= 0.0 {
            return;
        }
        self.time += dt_h;
        self.tick_econ(dt_h);
        self.tick_enemy(dt_h);
        self.compute_pipes();

        if self.time >= TOTAL_HOURS {
            self.win("Село выстояло 3 дня");
        }
        if self.time > 1.0 {
            if !self.has(BKind::Barracks) {
                self.hint(1, "Нет казармы: пехоту нечем пополнять.");
            }
            if self.power_eff < 0.8 {
                self.hint(2, "Не хватает энергии: постройте дизель или ТЭС.");
            }
            if self.tcc > 65.0 && !self.hints[3] {
                self.hint(3, "Высокая мобилизация злит людей. Следите за духом.");
            }
            if self.time > 6.0 && !self.hints[4] {
                self.hint(4, "Нажмите M — карта соседних сёл. Их можно штурмовать.");
            }
        }
        if self.connected_derricks > 0 && !self.hints[5] {
            self.hint(5, "Нефть пошла на НПЗ. Топливо теперь производится.");
        }
        self.check_hints();
    }

    fn check_hints(&mut self) {
        if self.time > 4.0 && self.focus.is_none() && !self.hints[6] {
            self.hint(6, "Сектор обороны не выбран: армия растянута по фронту.");
        }
    }

    fn update_drones(&mut self, dt: f32) {
        for d in &mut self.drones {
            if d.delay > 0.0 {
                d.delay -= dt;
                continue;
            }
            d.p = (d.p + dt * d.speed).min(1.0);
            let e = d.p * d.p * (3.0 - 2.0 * d.p);
            d.x = d.sx + (d.tx - d.sx) * e;
            d.y = d.sy + (d.ty - d.sy) * e - (e * std::f32::consts::PI).sin() * 34.0;
            if d.intercepted && !d.dead && d.p > 0.45 {
                d.dead = true;
                self.fx.boom(d.x, d.y, 0.5);
                self.fx.trace(d.x, d.y + 10.0);
            }
        }
        self.drones.retain(|d| d.p < 1.0);
    }

    pub fn compute_ambient(&self) -> Color {
        let h = self.clock();
        let mut c = if h < 5.0 {
            Color::new(0.52, 0.58, 0.80, 1.0)
        } else if h < 7.0 {
            let t = (h - 5.0) / 2.0;
            Color::new(0.52 + 0.46 * t, 0.58 + 0.36 * t, 0.80 + 0.14 * t, 1.0)
        } else if h < 17.5 {
            Color::new(1.0, 0.98, 0.94, 1.0)
        } else if h < 19.5 {
            let t = (h - 17.5) / 2.0;
            Color::new(1.0, 0.98 - 0.22 * t, 0.94 - 0.36 * t, 1.0)
        } else if h < 22.0 {
            let t = (h - 19.5) / 2.5;
            Color::new(1.0 - 0.48 * t, 0.76 - 0.18 * t, 0.58 + 0.22 * t, 1.0)
        } else {
            Color::new(0.52, 0.58, 0.80, 1.0)
        };
        if self.weather == Weather::Storm {
            c = Color::new(c.r * 0.72, c.g * 0.74, c.b * 0.84, 1.0);
        }
        c
    }

    pub fn tint(&self, base: Color) -> Color {
        Color::new(base.r * self.ambient.r, base.g * self.ambient.g, base.b * self.ambient.b, base.a)
    }

    pub fn can_shoot_drone(&self) -> bool {
        self.has(BKind::Aa) || self.has(BKind::Ew)
    }

    pub fn toggle_build(&mut self, k: BKind) {
        if self.build == Some(k) {
            self.build = None;
        } else {
            self.build = Some(k);
            self.pipe_mode = k == BKind::Pipe;
            self.sel = None;
        }
    }

    pub fn place_at_hover(&mut self) {
        if let Some(k) = self.build {
            let (x, y) = self.hover;
            match self.try_place(k, x, y) {
                Ok(_) => {
                    if k != BKind::Pipe {
                        self.build = None;
                        self.pipe_mode = false;
                    }
                }
                Err(e) => {
                    self.log(&e, art::col(art::RED));
                    self.fx.flash = 0.1;
                    self.fx.flash_col = art::col(art::RED);
                }
            }
        }
    }

    pub fn selected(&self) -> Option<&Building> {
        self.sel.and_then(|i| self.buildings.get(i)).filter(|b| b.alive)
    }

    pub fn unit_positions(&self) -> Vec<(f32, f32, &'static str, i32, i32)> {
        // Decorative garrison around the village for visual life.
        let mut v = Vec::new();
        let spots = [(20, 22), (20, 23), (21, 22), (19, 23), (23, 24), (17, 20)];
        for (i, &(x, y)) in spots.iter().enumerate() {
            if i as f32 >= self.army.inf.min(6.0) {
                break;
            }
            let p = Map::iso(x, y);
            v.push((p.x, p.y, if i % 2 == 0 { "soldier0" } else { "soldier1" }, x, y));
        }
        let tank_spots = [(27, 17), (27, 18)];
        for (i, &(x, y)) in tank_spots.iter().enumerate() {
            if i as f32 >= self.army.tanks.min(2.0) {
                break;
            }
            let p = Map::iso(x, y);
            v.push((p.x, p.y, "tank0", x, y));
        }
        v
    }

    pub fn draw_world(&self, atlas: &Atlas, vp: Rect) {
        let center = vec2(vp.x + vp.w * 0.5, vp.y + vp.h * 0.5);
        let cam = self.cam;
        let z = self.zoom;
        let to_screen = |wx: f32, wy: f32| vec2((wx - cam.x) * z + center.x, (wy - cam.y) * z + center.y);
        let amb = self.ambient;

        // Terrain.
        for y in 0..MAP {
            for x in 0..MAP {
                let p = Map::iso(x, y);
                let s = to_screen(p.x, p.y);
                if s.x < vp.x - 40.0 || s.x > vp.x + vp.w + 40.0 || s.y < vp.y - 30.0 || s.y > vp.y + vp.h + 60.0 {
                    continue;
                }
                let t = &self.map.tiles[(y * MAP + x) as usize];
                let name = t.terrain.sprite(t.variant);
                let tint = Color::new(amb.r, amb.g, amb.b, 1.0);
                if z == 1.0 {
                    atlas.draw(name, (s.x - 16.0).round(), (s.y - 8.0).round(), tint);
                } else {
                    draw_texture_ex(
                        &atlas.tex,
                        (s.x - 16.0 * z).round(),
                        (s.y - 8.0 * z).round(),
                        tint,
                        DrawTextureParams {
                            dest_size: Some(vec2(32.0 * z, 16.0 * z)),
                            source: Some(atlas.rect(name)),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        // Pipes: flat, under buildings.
        for y in 0..MAP {
            for x in 0..MAP {
                let t = &self.map.tiles[(y * MAP + x) as usize];
                if !t.pipe {
                    continue;
                }
                let p = Map::iso(x, y);
                let s = to_screen(p.x, p.y);
                let active = self.pipe_net[(y * MAP + x) as usize];
                let base = if active { Color::new(0.22, 0.20, 0.14, 1.0) } else { Color::new(0.18, 0.18, 0.18, 1.0) };
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (nx, ny) = (x + dx, y + dy);
                    if let Some(nt) = self.map.get(nx, ny) {
                        let connected_building = nt.building.map(|bi| {
                            let b = &self.buildings[bi];
                            b.alive && matches!(b.kind, BKind::Derrick | BKind::Refinery)
                        }).unwrap_or(false);
                        if nt.pipe || connected_building {
                            let q = Map::iso(nx, ny);
                            let e = to_screen(q.x, q.y);
                            draw_line(s.x, s.y, e.x, e.y, 4.0 * z, Color::new(base.r * amb.r, base.g * amb.g, base.b * amb.b, 1.0));
                            let hot = if active { Color::new(0.78, 0.56, 0.24, 1.0) } else { Color::new(0.34, 0.34, 0.32, 1.0) };
                            draw_line(s.x, s.y - 0.8 * z, e.x, e.y - 0.8 * z, 1.7 * z, hot);
                            draw_line(s.x, s.y - 1.6 * z, e.x, e.y - 1.6 * z, 0.7 * z, Color::new(1.0, 0.82, 0.5, 0.55 * if active { 1.0 } else { 0.3 }));
                        }
                    }
                }
                draw_rectangle(s.x - 2.0 * z, s.y - 2.0 * z, 4.0 * z, 4.0 * z, Color::new(0.32, 0.28, 0.2, 1.0));
                draw_rectangle(s.x - 1.0 * z, s.y - 1.0 * z, 2.0 * z, 2.0 * z, Color::new(0.8, 0.58, 0.27, 1.0));
            }
        }

        // Depth-sorted objects.
        struct D {
            key: i32,
            kind: u8,
            idx: usize,
            wx: f32,
            wy: f32,
            name: &'static str,
        }
        let mut list: Vec<D> = Vec::new();
        for (i, b) in self.buildings.iter().enumerate() {
            if !b.alive {
                continue;
            }
            let p = Map::iso(b.x, b.y);
            list.push(D { key: b.x + b.y, kind: 0, idx: i, wx: p.x, wy: p.y, name: "" });
        }
        for y in 0..MAP {
            for x in 0..MAP {
                if let Some(d) = self.map.tiles[(y * MAP + x) as usize].deco {
                    let p = Map::iso(x, y);
                    list.push(D { key: x + y + 1, kind: 1, idx: 0, wx: p.x, wy: p.y, name: d });
                }
            }
        }
        for (i, u) in self.unit_positions().iter().enumerate() {
            list.push(D { key: u.3 + u.4 + 2, kind: 2, idx: i, wx: u.0, wy: u.1, name: u.2 });
        }
        list.sort_by_key(|d| d.key);

        for d in list {
            let s = to_screen(d.wx, d.wy + 8.0);
            match d.kind {
                0 => {
                    let b = &self.buildings[d.idx];
                    let def = b.kind.def();
                    let mut name = def.sprite;
                    if b.kind == BKind::Derrick {
                        name = if (b.powered && b.connected) && ((self.frame_t * 1.6) as i32 % 2 == 0) { "derrick1" } else { "derrick0" };
                    }
                    if b.kind == BKind::House {
                        name = ["house0", "house1", "house2"][d.idx % 3];
                    }
                    if !b.built() {
                        let a = 0.55;
                        atlas.draw_at(name, s.x, s.y, Color::new(1.0, 1.0, 1.0, a));
                        let w = 18.0;
                        draw_rectangle(s.x - w * 0.5, s.y - 30.0, w, 3.0, Color::new(0.1, 0.1, 0.1, 0.8));
                        draw_rectangle(s.x - w * 0.5, s.y - 30.0, w * b.build_p, 3.0, art::col(art::AMBER));
                        continue;
                    }
                    let dmg = (b.hp / b.max_hp).clamp(0.0, 1.0);
                    let mut tint = Color::new(amb.r, amb.g, amb.b, 1.0);
                    if dmg < 0.65 {
                        tint = Color::new(tint.r * (0.55 + 0.45 * dmg), tint.g * (0.5 + 0.4 * dmg), tint.b * (0.5 + 0.35 * dmg), 1.0);
                    }
                    if b.hit > 0.3 {
                        tint = Color::new(1.0, 0.65, 0.55, 1.0);
                    }
                    atlas.draw_at(name, s.x, s.y, tint);
                    if b.burning > 0.0 {
                        // Flames are drawn by particles; add a glow.
                        draw_circle(s.x, s.y - 12.0, 5.0 * z, Color::new(1.0, 0.45, 0.1, 0.10));
                    }
                }
                1 => {
                    if d.name == "sunflower" || d.name == "bush" || d.name == "rock" {
                        let sc = if d.name == "rock" { 0.75 } else { 0.9 };
                        atlas.draw_at_scaled(d.name, s.x, s.y, sc * z, Color::new(amb.r, amb.g, amb.b, 1.0));
                    } else {
                        atlas.draw_at(d.name, s.x, s.y, Color::new(amb.r, amb.g, amb.b, 1.0));
                    }
                }
                _ => {
                    let u = self.unit_positions()[d.idx];
                    atlas.draw_at(u.2, s.x, s.y, Color::new(amb.r, amb.g, amb.b, 1.0));
                }
            }
        }

        // Enemy drones in the air.
        for d in &self.drones {
            if d.dead || d.p >= 1.0 {
                continue;
            }
            let s = to_screen(d.x, d.y);
            let name = if d.kind == 0 { "shahed" } else { "fpv" };
            let bob = (self.frame_t * 9.0 + d.sx).sin() * 1.5;
            atlas.draw_at(name, s.x, s.y + bob, Color::new(1.0, 1.0, 1.0, 1.0));
            draw_line(s.x, s.y + 6.0, s.x - 4.0, s.y + 16.0, 1.0, Color::new(0.8, 0.6, 0.3, 0.4));
        }

        // Threat markers.
        if let Some(t) = &self.enemy.threat {
            if t.kind == ThreatKind::Arty {
                let pulse = 0.5 + 0.5 * (self.frame_t * 8.0).sin();
                for (x, y) in &t.targets {
                    let p = Map::iso(*x, *y);
                    let s = to_screen(p.x, p.y);
                    let r = 7.0 + pulse * 3.0;
                    draw_line(s.x - r, s.y, s.x - r + 4.0, s.y, 1.0, Color::new(1.0, 0.25, 0.2, 0.9));
                    draw_line(s.x + r - 4.0, s.y, s.x + r, s.y, 1.0, Color::new(1.0, 0.25, 0.2, 0.9));
                    draw_line(s.x, s.y - r * 0.5, s.x, s.y - r * 0.5 + 4.0, 1.0, Color::new(1.0, 0.25, 0.2, 0.9));
                    draw_line(s.x, s.y + r * 0.5 - 4.0, s.x, s.y + r * 0.5, 1.0, Color::new(1.0, 0.25, 0.2, 0.9));
                }
            }
        }

        // Hover/build ghost.
        if self.screen == Screen::Play {
            let (hx, hy) = self.hover;
            if let Some(k) = self.build {
                if self.map.get(hx, hy).is_some() {
                    let p = Map::iso(hx, hy);
                    let s = to_screen(p.x, p.y);
                    let ok = if k == BKind::Pipe {
                        self.map.get(hx, hy).map(|t| !t.pipe && t.building.is_none() && t.terrain.buildable()).unwrap_or(false)
                    } else {
                        self.try_place_silent(k, hx, hy)
                    };
                    if k == BKind::Pipe {
                        let c = if ok { Color::new(0.5, 1.0, 0.6, 0.35) } else { Color::new(1.0, 0.35, 0.3, 0.35) };
                        draw_rectangle(s.x - 16.0 * z, s.y - 8.0 * z, 32.0 * z, 16.0 * z, c);
                    } else {
                        let tint = if ok { Color::new(0.65, 1.0, 0.7, 0.55) } else { Color::new(1.0, 0.35, 0.3, 0.55) };
                        atlas.draw_at(k.def().sprite, s.x, s.y + 8.0, tint);
                        if matches!(k, BKind::Substation | BKind::Diesel | BKind::Tes | BKind::Solar) {
                            let r = 7.0;
                            draw_ellipse(s.x, s.y, r * 16.0 * z, r * 8.0 * z, 0.0, Color::new(0.4, 0.8, 1.0, 0.07));
                        }
                    }
                    if ok_hover_marker(k) {
                        draw_rectangle_lines(s.x - 16.0 * z, s.y - 8.0 * z, 32.0 * z, 16.0 * z, 1.0, Color::new(1.0, 0.85, 0.4, 0.6));
                    }
                }
            } else if let Some(i) = self.sel {
                if let Some(b) = self.buildings.get(i) {
                    if b.alive {
                        let p = Map::iso(b.x, b.y);
                        let s = to_screen(p.x, p.y);
                        draw_rectangle_lines(s.x - 16.0 * z, s.y - 8.0 * z, 32.0 * z, 16.0 * z, 1.0, art::col(art::AMBER));
                    }
                }
            }
        }

        // Weather overlays handled in UI (screen space), not here.
    }

    pub fn try_place_silent(&self, k: BKind, x: i32, y: i32) -> bool {
        let d = k.def();
        if !self.can_afford(k) {
            return false;
        }
        if let Some(t) = self.map.get(x, y) {
            if k == BKind::Derrick {
                return t.terrain == Terrain::Oil && t.building.is_none();
            }
            if t.terrain == Terrain::Oil || !t.terrain.buildable() || t.building.is_some() || t.pipe {
                return false;
            }
            let _ = d;
            true
        } else {
            false
        }
    }

    pub fn weather_overlay(&self, vp: Rect, font: &Font) {
        let _ = font;
        match self.weather {
            Weather::Rain | Weather::Storm => {
                let n = if self.weather == Weather::Storm { 90 } else { 45 };
                for i in 0..n {
                    let t = self.frame_t * 260.0 + i as f32 * 37.0;
                    let x = vp.x + (i as f32 * 73.0 + t * 0.6) % vp.w;
                    let y = vp.y + (i as f32 * 131.0 + t) % vp.h;
                    draw_line(x, y, x - 3.0, y + 9.0, 1.0, Color::new(0.55, 0.65, 0.85, 0.25));
                }
                draw_rectangle(vp.x, vp.y, vp.w, vp.h, Color::new(0.08, 0.10, 0.16, 0.18));
            }
            Weather::Fog => {
                for i in 0..4 {
                    let y = vp.y + vp.h * (i as f32 / 4.0);
                    draw_rectangle(vp.x, y, vp.w, vp.h / 4.0, Color::new(0.62, 0.66, 0.66, 0.07));
                }
            }
            _ => {}
        }
    }
}

fn ok_hover_marker(k: BKind) -> bool {
    !matches!(k, BKind::Trench | BKind::Mines)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready(f: Faction) -> Game {
        let mut g = Game::new(f);
        g.start(f);
        g
    }

    #[test]
    fn pipe_chain_feeds_refinery() {
        let mut g = ready(Faction::Ua);
        let path = [(12, 33), (12, 32), (13, 32), (13, 31), (13, 30), (14, 30), (14, 29)];
        for (x, y) in path {
            g.try_place(BKind::Pipe, x, y).unwrap_or_else(|e| panic!("pipe {x},{y}: {e}"));
        }
        assert_eq!(g.derricks, 1, "one derrick");
        assert_eq!(g.connected_derricks, 1, "derrick linked to refinery by pipes");
        let fuel0 = g.res.fuel;
        for _ in 0..30 {
            g.tick_econ(0.1);
        }
        assert!(g.stats.fuel_made > 0.0, "refinery refined crude, fuel0={fuel0}");
    }

    #[test]
    fn village_survives_first_hours() {
        let mut g = ready(Faction::Ua);
        for _ in 0..120 {
            g.tick_econ(0.1);
        }
        assert!(g.pop > 40.0, "population collapsed: {}", g.pop);
        assert!(g.res.food > 0.0 && g.res.water > 0.0);
        assert!(g.power_cap > 0.0);
    }

    #[test]
    fn assault_can_capture_all_targets() {
        let mut g = ready(Faction::Ua);
        g.army.inf = 60.0;
        g.army.tanks = 10.0;
        g.army.ifv = 8.0;
        g.res.ammo = 120.0;
        g.res.fuel = 120.0;
        for i in 0..3 {
            g.assault_cd = 0.0;
            g.assault_target(i).unwrap();
            assert!(g.targets[i].captured, "target {i} captured");
        }
        assert_eq!(g.screen, Screen::Win);
    }
}
