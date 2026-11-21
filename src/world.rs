// Map, terrain and building catalogue.
use crate::art::{self, Rng};
use macroquad::prelude::*;

pub const MAP: i32 = 40;
pub const TILE_W: f32 = 32.0;
pub const TILE_H: f32 = 16.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Terrain {
    Grass,
    Field,
    Road,
    Dirt,
    Water,
    Forest,
    Oil,
    Rubble,
}

impl Terrain {
    pub fn sprite(self, variant: u8) -> &'static str {
        match self {
            Terrain::Grass => ["grass0", "grass1", "grass2", "grass3"][variant as usize % 4],
            Terrain::Field => ["field0", "field1"][variant as usize % 2],
            Terrain::Road => "road",
            Terrain::Dirt => "dirt",
            Terrain::Water => ["water0", "water1"][variant as usize % 2],
            Terrain::Forest => "forest",
            Terrain::Oil => "oil",
            Terrain::Rubble => "rubble",
        }
    }
    pub fn buildable(self) -> bool {
        matches!(self, Terrain::Grass | Terrain::Field | Terrain::Road | Terrain::Dirt | Terrain::Rubble)
    }
    pub fn mini_color(self) -> Color {
        match self {
            Terrain::Grass => Color::new(0.22, 0.32, 0.18, 1.0),
            Terrain::Field => Color::new(0.36, 0.33, 0.16, 1.0),
            Terrain::Road => Color::new(0.31, 0.30, 0.27, 1.0),
            Terrain::Dirt => Color::new(0.33, 0.26, 0.19, 1.0),
            Terrain::Water => Color::new(0.10, 0.25, 0.35, 1.0),
            Terrain::Forest => Color::new(0.12, 0.24, 0.13, 1.0),
            Terrain::Oil => Color::new(0.12, 0.11, 0.10, 1.0),
            Terrain::Rubble => Color::new(0.30, 0.27, 0.22, 1.0),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub terrain: Terrain,
    pub variant: u8,
    pub building: Option<usize>,
    pub pipe: bool,
    pub deco: Option<&'static str>,
}

impl Tile {
    fn new(t: Terrain, variant: u8) -> Self {
        Tile { terrain: t, variant, building: None, pipe: false, deco: None }
    }
}

pub struct Map {
    pub tiles: Vec<Tile>,
    pub village: (i32, i32),
    pub oil_tiles: Vec<(i32, i32)>,
    pub main_road: Vec<(i32, i32)>,
}

impl Map {
    pub fn generate(seed: u32) -> Self {
        let mut rng = Rng::new(seed);
        let mut tiles = vec![Tile::new(Terrain::Grass, 0); (MAP * MAP) as usize];
        let idx = |x: i32, y: i32| (y * MAP + x) as usize;

        for y in 0..MAP {
            for x in 0..MAP {
                let v = (rng.f() * 4.0) as u8;
                tiles[idx(x, y)] = Tile::new(Terrain::Grass, v);
            }
        }

        // Fields around the village.
        for &(x0, y0, w, h) in &[(26, 3, 9, 7), (28, 12, 8, 6), (3, 12, 7, 5), (25, 31, 10, 6), (31, 24, 6, 5)] {
            for y in y0..(y0 + h).min(MAP) {
                for x in x0..(x0 + w).min(MAP) {
                    let v = (rng.f() * 2.0) as u8;
                    tiles[idx(x, y)] = Tile::new(Terrain::Field, v);
                }
            }
        }

        // Pond in the north-west.
        for y in 0..MAP {
            for x in 0..MAP {
                let dx = (x as f32 - 6.5) / 3.2;
                let dy = (y as f32 - 6.0) / 2.2;
                if dx * dx + dy * dy < 1.0 {
                    let v = (rng.f() * 2.0) as u8;
                    tiles[idx(x, y)] = Tile::new(Terrain::Water, v);
                }
            }
        }

        // Forest clusters, kept away from the village centre.
        for _ in 0..14 {
            let cx = rng.range(2.0, 38.0);
            let cy = rng.range(2.0, 38.0);
            let r = rng.range(1.6, 3.8);
            if (cx - 20.0).abs() < 9.0 && (cy - 20.0).abs() < 9.0 {
                continue;
            }
            for y in 0..MAP {
                for x in 0..MAP {
                    let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                    if d < r && rng.f() > 0.25 {
                        let t = &mut tiles[idx(x, y)];
                        if t.terrain == Terrain::Grass {
                            t.terrain = Terrain::Forest;
                            t.deco = Some(["tree0", "tree1", "tree2"][(rng.f() * 3.0) as usize]);
                        }
                    }
                }
            }
        }

        // Oil-bearing ground in the south-west.
        let mut oil_tiles = Vec::new();
        for &(x, y) in &[(11, 33), (9, 31), (13, 35), (7, 34), (15, 31)] {
            tiles[idx(x, y)] = Tile::new(Terrain::Oil, 0);
            oil_tiles.push((x, y));
        }

        // Main road west - east (the road to the front).
        let mut main_road = Vec::new();
        let pts: [(i32, i32); 8] = [(0, 18), (7, 19), (13, 19), (19, 20), (25, 20), (31, 21), (36, 22), (39, 23)];
        for w in pts.windows(2) {
            let (ax, ay) = w[0];
            let (bx, by) = w[1];
            let steps = (bx - ax).abs().max((by - ay).abs()).max(1);
            for s in 0..=steps {
                let x = ax + (bx - ax) * s / steps;
                let y = ay + (by - ay) * s / steps;
                for dy in 0..2 {
                    let yy = y + dy;
                    if x >= 0 && x < MAP && yy >= 0 && yy < MAP {
                        tiles[idx(x, yy)] = Tile::new(Terrain::Road, 0);
                        main_road.push((x, yy));
                    }
                }
            }
        }
        // Village streets.
        for y in 14..=27 {
            for x in 19..=20 {
                tiles[idx(x, y)] = Tile::new(Terrain::Road, 0);
            }
        }
        for x in 14..=26 {
            for y in 24..=25 {
                tiles[idx(x, y)] = Tile::new(Terrain::Road, 0);
            }
        }

        // Village yards: dirt patches between the streets.
        for y in 14..=27 {
            for x in 13..=29 {
                if rng.f() > 0.62 && tiles[idx(x, y)].terrain == Terrain::Grass {
                    tiles[idx(x, y)] = Tile::new(Terrain::Dirt, 0);
                }
            }
        }

        // Destroyed eastern edge (the front).
        for _ in 0..40 {
            let x = 34 + (rng.f() * 6.0) as i32;
            let y = (rng.f() * 40.0) as i32;
            let v = (rng.f() * 2.0) as u8;
            tiles[idx(x.min(39), y.min(39))] = Tile::new(Terrain::Rubble, v);
        }

        // Scenery.
        for y in 0..MAP {
            for x in 0..MAP {
                let i = idx(x, y);
                if tiles[i].deco.is_some() || tiles[i].terrain == Terrain::Water {
                    continue;
                }
                let r = rng.f();
                if tiles[i].terrain == Terrain::Field && r > 0.97 {
                    tiles[i].deco = Some("sunflower");
                } else if tiles[i].terrain == Terrain::Grass && r > 0.975 {
                    tiles[i].deco = Some(["bush", "bush", "rock"][(rng.f() * 3.0) as usize]);
                } else if tiles[i].terrain == Terrain::Rubble && r > 0.92 {
                    tiles[i].deco = Some("wreck");
                }
                let near_village = (13..=29).contains(&x) && (13..=27).contains(&y);
                if !near_village && r < 0.06 {
                    tiles[i].deco = Some(if r < 0.02 { "tree1" } else { "bush" });
                }
            }
        }
        // Fences and poles along the village.
        for x in 14..=28 {
            let i = idx(x, 13);
            tiles[i].deco = Some(if x % 3 == 0 { "pole" } else { "fence" });
        }
        for y in 15..=26 {
            if y % 4 == 0 {
                tiles[idx(30, y)].deco = Some("pole");
            }
        }

        Map { tiles, village: (20, 20), oil_tiles, main_road }
    }

    pub fn idx(x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= MAP || y >= MAP {
            None
        } else {
            Some((y * MAP + x) as usize)
        }
    }
    pub fn get(&self, x: i32, y: i32) -> Option<&Tile> {
        Self::idx(x, y).map(|i| &self.tiles[i])
    }
    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        Self::idx(x, y).map(move |i| &mut self.tiles[i])
    }
    pub fn terrain(&self, x: i32, y: i32) -> Terrain {
        self.get(x, y).map(|t| t.terrain).unwrap_or(Terrain::Grass)
    }
    /// Centre of a tile's floor diamond in world space.
    pub fn iso(x: i32, y: i32) -> Vec2 {
        vec2((x - y) as f32 * (TILE_W * 0.5), (x + y) as f32 * (TILE_H * 0.5) + TILE_H * 0.5)
    }
    /// Inverse iso projection (world -> tile index).
    pub fn from_world(p: Vec2) -> (i32, i32) {
        let a = p.x / (TILE_W * 0.5);
        let b = (p.y - TILE_H * 0.5) / (TILE_H * 0.5);
        let x = ((a + b) * 0.5).floor() as i32;
        let y = ((b - a) * 0.5).floor() as i32;
        (x, y)
    }
    pub fn can_place(&self, x: i32, y: i32) -> Result<(), &'static str> {
        let t = self.get(x, y).ok_or("За границей карты")?;
        if !t.terrain.buildable() {
            return Err(match t.terrain {
                Terrain::Water => "Нельзя строить на воде",
                Terrain::Forest => "Сначала вырубите лес",
                Terrain::Oil => "Нефтяная жила: ставьте вышку",
                _ => "Здесь нельзя строить",
            });
        }
        if t.building.is_some() {
            return Err("Клетка занята");
        }
        if t.pipe {
            return Err("Здесь проходит труба");
        }
        Ok(())
    }
}

// ---------- buildings ----------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat {
    Life,
    Resource,
    Power,
    Army,
    Fort,
}

pub const CATS: [(&str, Cat); 5] = [
    ("БЫТ", Cat::Life),
    ("РЕСУРС", Cat::Resource),
    ("ЭНЕРГИЯ", Cat::Power),
    ("АРМИЯ", Cat::Army),
    ("ОБОРОНА", Cat::Fort),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BKind {
    Hq,
    House,
    Shop,
    School,
    Church,
    Hospital,
    Tcc,
    Farm,
    Watertower,
    Metalshop,
    Derrick,
    Refinery,
    Pipe,
    Diesel,
    Tes,
    Solar,
    Substation,
    Barracks,
    TankFactory,
    DroneHangar,
    Repair,
    Radar,
    Ew,
    Bunker,
    Aa,
    Trench,
    Mines,
}

pub const ALL_BUILDINGS: [BKind; 27] = [
    BKind::Hq,
    BKind::House,
    BKind::Shop,
    BKind::School,
    BKind::Church,
    BKind::Hospital,
    BKind::Tcc,
    BKind::Farm,
    BKind::Watertower,
    BKind::Metalshop,
    BKind::Derrick,
    BKind::Refinery,
    BKind::Pipe,
    BKind::Diesel,
    BKind::Tes,
    BKind::Solar,
    BKind::Substation,
    BKind::Barracks,
    BKind::TankFactory,
    BKind::DroneHangar,
    BKind::Repair,
    BKind::Radar,
    BKind::Ew,
    BKind::Bunker,
    BKind::Aa,
    BKind::Trench,
    BKind::Mines,
];

pub struct BDef {
    pub name: &'static str,
    pub full: &'static str,
    pub sprite: &'static str,
    pub cat: Cat,
    pub money: f32,
    pub steel: f32,
    pub fuel: f32,
    /// negative = consumption in kW, positive = generation in kW
    pub power: f32,
    pub hp: f32,
    pub desc: &'static str,
    pub hidden: bool,
}

impl BKind {
    pub fn def(self) -> &'static BDef {
        &DEFS[self as usize]
    }
    pub fn power_demand(self) -> f32 {
        (-self.def().power).max(0.0)
    }
    pub fn needs_power(self) -> bool {
        self.def().power < 0.0
    }
}

static DEFS: [BDef; 27] = [
    BDef { name: "Штаб", full: "Штаб обороны", sprite: "hq", cat: Cat::Life, money: 0.0, steel: 0.0, fuel: 0.0, power: -2.0, hp: 900.0, desc: "Сердце села. Потеря штаба — поражение.", hidden: true },
    BDef { name: "Дом", full: "Жилой дом", sprite: "house0", cat: Cat::Life, money: 70.0, steel: 3.0, fuel: 0.0, power: 0.0, hp: 100.0, desc: "+10 к вместимости села. Люди переезжают при высоком духе.", hidden: false },
    BDef { name: "Магазин", full: "Магазин", sprite: "shop", cat: Cat::Life, money: 110.0, steel: 5.0, fuel: 0.0, power: -1.0, hp: 120.0, desc: "Налоги и продажа топлива. +доход от населения.", hidden: false },
    BDef { name: "Школа", full: "Школа", sprite: "school", cat: Cat::Life, money: 160.0, steel: 8.0, fuel: 0.0, power: -2.0, hp: 160.0, desc: "+3 к духу. Дети учатся даже под обстрелами.", hidden: false },
    BDef { name: "Церковь", full: "Церковь", sprite: "church", cat: Cat::Life, money: 90.0, steel: 3.0, fuel: 0.0, power: 0.0, hp: 140.0, desc: "+4 к духу. Не требует электричества.", hidden: false },
    BDef { name: "Больница", full: "Больница", sprite: "hospital", cat: Cat::Life, money: 220.0, steel: 14.0, fuel: 0.0, power: -3.0, hp: 200.0, desc: "Меньше потерь от обстрелов, +3 к духу.", hidden: false },
    BDef { name: "ТЦК", full: "ТЦК", sprite: "tcc", cat: Cat::Life, money: 130.0, steel: 5.0, fuel: 0.0, power: -1.0, hp: 150.0, desc: "Мобилизация. Больше людей в армии — ниже дух села.", hidden: false },
    BDef { name: "Ферма", full: "Ферма", sprite: "farm", cat: Cat::Resource, money: 120.0, steel: 5.0, fuel: 0.0, power: -2.0, hp: 140.0, desc: "Производит еду. Нужна вода и электричество.", hidden: false },
    BDef { name: "Водонап.", full: "Водонапорная башня", sprite: "watertower", cat: Cat::Resource, money: 140.0, steel: 7.0, fuel: 0.0, power: -3.0, hp: 140.0, desc: "Качает воду для села и производства.", hidden: false },
    BDef { name: "Металлоцех", full: "Металлоцех", sprite: "metalshop", cat: Cat::Resource, money: 210.0, steel: 16.0, fuel: 0.0, power: -4.0, hp: 180.0, desc: "Выпускает сталь: 1,2/ч. Нужны деньги и энергия.", hidden: false },
    BDef { name: "Вышка", full: "Нефтяная вышка", sprite: "derrick0", cat: Cat::Resource, money: 170.0, steel: 14.0, fuel: 0.0, power: -2.0, hp: 140.0, desc: "Качает нефть. Только на жиле. Нужна труба до НПЗ.", hidden: false },
    BDef { name: "НПЗ", full: "Нефтеперерабатывающий завод", sprite: "refinery", cat: Cat::Resource, money: 420.0, steel: 30.0, fuel: 0.0, power: -6.0, hp: 320.0, desc: "Нефть → топливо. Без энергии работает вполсилы.", hidden: false },
    BDef { name: "Труба", full: "Труба", sprite: "", cat: Cat::Resource, money: 12.0, steel: 2.0, fuel: 0.0, power: 0.0, hp: 40.0, desc: "Тянет нефть. Строится по клеткам, как дорога.", hidden: false },
    BDef { name: "Дизель", full: "Дизель-генератор", sprite: "diesel", cat: Cat::Power, money: 150.0, steel: 8.0, fuel: 0.0, power: 12.0, hp: 120.0, desc: "12 кВт. Сжигает 0,6 топлива в час.", hidden: false },
    BDef { name: "ТЭС", full: "Тепловая электростанция", sprite: "tes", cat: Cat::Power, money: 360.0, steel: 24.0, fuel: 0.0, power: 30.0, hp: 300.0, desc: "30 кВт. Сжигает 1,6 топлива в час.", hidden: false },
    BDef { name: "Солнце", full: "Солнечная станция", sprite: "solar", cat: Cat::Power, money: 220.0, steel: 10.0, fuel: 0.0, power: 10.0, hp: 100.0, desc: "10 кВт днём. Ночью и в дождь почти бесполезна.", hidden: false },
    BDef { name: "Подстан.", full: "Подстанция", sprite: "substation", cat: Cat::Power, money: 110.0, steel: 6.0, fuel: 0.0, power: 0.0, hp: 120.0, desc: "Питает здания в радиусе 7 клеток.", hidden: false },
    BDef { name: "Казарма", full: "Казарма", sprite: "barracks", cat: Cat::Army, money: 210.0, steel: 12.0, fuel: 0.0, power: -2.0, hp: 260.0, desc: "Готовит пехоту. Требует людей и БК.", hidden: false },
    BDef { name: "Танк.завод", full: "Танковый завод", sprite: "tankfactory", cat: Cat::Army, money: 480.0, steel: 40.0, fuel: 0.0, power: -8.0, hp: 380.0, desc: "Собирает танки из стали, топлива и денег.", hidden: false },
    BDef { name: "Дроны", full: "Ангар дронов", sprite: "dronehangar", cat: Cat::Army, money: 250.0, steel: 16.0, fuel: 0.0, power: -4.0, hp: 200.0, desc: "Собирает FPV-дроны. Основа разведки и ударов.", hidden: false },
    BDef { name: "Ремонт", full: "Ремонтная база", sprite: "repair", cat: Cat::Army, money: 170.0, steel: 10.0, fuel: 0.0, power: -2.0, hp: 160.0, desc: "Постепенно чинит здания и укрепления.", hidden: false },
    BDef { name: "Радар", full: "Радар", sprite: "radar", cat: Cat::Army, money: 180.0, steel: 8.0, fuel: 0.0, power: -4.0, hp: 140.0, desc: "Раскрывает сектор атаки. Нужен для контрбатареи.", hidden: false },
    BDef { name: "РЭБ", full: "РЭБ", sprite: "ew", cat: Cat::Army, money: 230.0, steel: 12.0, fuel: 0.0, power: -4.0, hp: 180.0, desc: "Глушит дроны: +12% к перехвату.", hidden: false },
    BDef { name: "Бункер", full: "Бункер", sprite: "bunker", cat: Cat::Fort, money: 150.0, steel: 14.0, fuel: 0.0, power: 0.0, hp: 300.0, desc: "+4 к обороне. Укрывает людей от обстрела.", hidden: false },
    BDef { name: "ПВО", full: "ПВО", sprite: "aa", cat: Cat::Fort, money: 190.0, steel: 10.0, fuel: 0.0, power: -3.0, hp: 160.0, desc: "Сбивает дроны и ракеты. +10% к перехвату.", hidden: false },
    BDef { name: "Окоп", full: "Окоп", sprite: "trench", cat: Cat::Fort, money: 35.0, steel: 0.0, fuel: 0.0, power: 0.0, hp: 120.0, desc: "+2 к обороне сектора.", hidden: false },
    BDef { name: "Мины", full: "Минное поле", sprite: "mines", cat: Cat::Fort, money: 45.0, steel: 1.0, fuel: 0.0, power: 0.0, hp: 60.0, desc: "+3 к обороне. Взрывается при штурме.", hidden: false },
];

pub fn buildings_for(cat: Cat) -> Vec<BKind> {
    ALL_BUILDINGS.iter().copied().filter(|b| b.def().cat == cat && !b.def().hidden).collect()
}

pub fn category_color(cat: Cat) -> Color {
    art::col(match cat {
        Cat::Life => art::AMBER,
        Cat::Resource => art::GREEN,
        Cat::Power => art::CYAN,
        Cat::Army => art::BLUE,
        Cat::Fort => art::RED,
    })
}
