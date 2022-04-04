use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::ops::{Add, Sub};

use rand::Rng;
use roxmltree as xml;
use serde::{Deserialize, Serialize};

use super::dice;
use super::histogram::Histogram;

mod table;

use table::{
    AtmoRecord, CulturalDiffRecord, GovRecord, HydroRecord, LawRecord, PopRecord, StarportClass,
    StarportRecord, TempRecord, WorldTagRecord, TABLES,
};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
enum TravelCode {
    Safe,
    Amber,
    Red,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
enum TradeCode {
    Ag,
    As,
    Ba,
    De,
    Fl,
    Ga,
    Hi,
    Ht,
    Ie,
    In,
    Lo,
    Lt,
    Na,
    Ni,
    Po,
    Ri,
    Va,
    Wa,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Point {
    x: u16,
    y: u16,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Faction {
    name: String,
    code: u16,
    strength: String,
    government: GovRecord,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct World {
    name: String,
    location: Point,
    has_gas_giant: bool,
    size: u16,
    diameter: u32,
    gravity: f64,
    atmosphere: AtmoRecord,
    temperature: TempRecord,
    hydrographics: HydroRecord,
    population: PopRecord,
    unmodified_pop: u16,
    government: GovRecord,
    law_level: LawRecord,
    factions: Vec<Faction>,
    culture: CulturalDiffRecord,
    world_tags: [WorldTagRecord; 2],
    starport: StarportRecord,
    tech_level: u16,
    has_naval_base: bool,
    has_scout_base: bool,
    has_research_base: bool,
    has_tas: bool,
    travel_code: TravelCode,
    trade_codes: BTreeSet<TradeCode>,
}

impl World {
    fn profile_string(&self) -> String {
        format!(
            "{starport:?}{size:X}{atmo:X}{hydro:X}{pop:X}{gov:X}{law:X}-{tech:X}",
            starport = self.starport.class,
            size = self.size,
            atmo = self.atmosphere.code,
            hydro = self.hydrographics.code,
            pop = self.population.code,
            gov = self.government.code,
            law = self.law_level.code,
            tech = self.tech_level
        )
    }

    fn summary_csv(&self, location: &str) -> String {
        let profile = self.profile_string();

        let mut bases = Vec::new();
        if self.has_naval_base {
            bases.push(String::from("N"));
        }
        if self.has_research_base {
            bases.push(String::from("R"));
        }
        if self.has_scout_base {
            bases.push(String::from("S"));
        }
        if self.has_tas {
            bases.push(String::from("T"));
        }
        let bases = bases.join(" ");

        let trade_codes = self
            .trade_codes
            .iter()
            .map(|code| format!("{:?}", code))
            .collect::<Vec<String>>()
            .join(" ");

        // Row format: name,location,profile,bases,trade codes,travel code,gas giant
        //     Profile format: starport,size,atmo,hydro,pop,gov,law,tech
        format!(
            "{name},{location},{profile},{bases},{trade_codes},{travel_code:?},{gas_giant}",
            name = self.name,
            location = location,
            profile = profile,
            bases = bases,
            trade_codes = trade_codes,
            travel_code = self.travel_code,
            gas_giant = match self.has_gas_giant {
                true => "G",
                false => "",
            }
        )
    }

    fn societal_csv(&self, location: &str) -> String {
        format!(
            "{name},{location},{culture},{world_tag_1},{world_tag_2}",
            name = self.name,
            location = location,
            culture = self.culture.cultural_difference,
            world_tag_1 = self.world_tags[0].tag,
            world_tag_2 = self.world_tags[1].tag
        )
    }

    pub fn new(name: String, location: Point) -> Self {
        let mut world = Self::empty();
        world.name = name;
        world.location = location;

        // Generation *must* happen in this order, many fields depend on the value
        // of other fields when making their rolls
        world.generate_gas_giant();
        world.generate_size();
        world.generate_atmosphere();
        world.generate_temperature();
        world.generate_hydrographics();
        world.generate_population();
        world.generate_government();
        world.generate_law_level();
        world.generate_factions();
        world.generate_culture();
        world.generate_world_tags();
        world.generate_starport();
        world.generate_tech_level();
        world.generate_bases();
        world.resolve_travel_code();
        world.resolve_trade_codes();

        world
    }

    fn empty() -> Self {
        World {
            name: String::from(""),
            location: Point { x: 0, y: 0 },
            has_gas_giant: false,
            size: 0,
            diameter: 0,
            gravity: 0.0,
            atmosphere: TABLES.atmo_table[0].clone(),
            temperature: TABLES.temp_table[0].clone(),
            hydrographics: TABLES.hydro_table[0].clone(),
            population: TABLES.pop_table[0].clone(),
            unmodified_pop: 0,
            government: TABLES.gov_table[0].clone(),
            factions: Vec::new(),
            culture: TABLES.culture_table[0].clone(),
            world_tags: [
                TABLES.world_tag_table[0].clone(),
                TABLES.world_tag_table[0].clone(),
            ],
            law_level: TABLES.law_table[0].clone(),
            starport: TABLES.starport_table[0].clone(),
            tech_level: 0,
            has_naval_base: false,
            has_scout_base: false,
            has_research_base: false,
            has_tas: false,
            travel_code: TravelCode::Safe,
            trade_codes: BTreeSet::new(),
        }
    }

    fn generate_gas_giant(&mut self) {
        match dice::roll_2d(6) {
            0..=9 => self.has_gas_giant = true,
            _ => self.has_gas_giant = false,
        }
    }

    fn generate_size(&mut self) {
        self.size = (dice::roll_2d(6) - 2).clamp(0, 10);

        let median: u32 = match self.size {
            0 => 700,
            _ => (1600 * self.size).into(),
        };
        let min = median - 200;
        let max = median + 200;
        self.diameter = dice::roll(min..=max);

        self.gravity = match self.size {
            0 => 0.0,
            1 => 0.05,
            2 => 0.15,
            3 => 0.25,
            4 => 0.35,
            5 => 0.45,
            6 => 0.7,
            7 => 0.9,
            8 => 1.0,
            9 => 1.25,
            _ => 1.4,
        };
    }

    fn generate_atmosphere(&mut self) {
        let lower = 0;
        let upper = (TABLES.atmo_table.len() - 1) as i32;
        let roll: i32 = dice::roll_2d(6) - 7 + self.size as i32;
        let index = roll.clamp(lower, upper) as usize;
        self.atmosphere = TABLES.atmo_table[index].clone();
    }

    fn generate_temperature(&mut self) {
        let atmo_modifier: i32 = match self.atmosphere.code {
            0 | 1 => 0,
            2 | 3 => -2,
            4 | 5 | 14 => -1,
            6 | 7 => 0,
            8 | 9 => 1,
            10 | 13 | 15 => 2,
            11 | 12 => 6,
            // Should *never* happen
            _ => 0,
        };

        let lower = 0;
        let upper = (TABLES.temp_table.len() - 1) as i32;
        let roll = dice::roll_2d(6) + atmo_modifier;
        let index = roll.clamp(lower, upper) as usize;
        self.temperature = TABLES.temp_table[index].clone();
    }

    fn generate_hydrographics(&mut self) {
        let roll = if self.size > 1 {
            let atmo_modifier: i32 = match self.atmosphere.code {
                0 | 1 | 10 | 11 | 12 => -4,

                // These two are not strictly following the rulebook, but any hydro
                // above a 5 is impossible without them
                6..=7 => 2,
                8..=9 => 4,

                _ => 0,
            };

            let temp_modifier: i32 = if self.atmosphere.code != 13 {
                match self.temperature.code {
                    5..=9 => 2, // Not following the rulebook for same reason as above
                    10 | 11 => -2,
                    12 => -6,
                    _ => 0,
                }
            } else {
                0
            };

            dice::roll_2d(6) - 7 + atmo_modifier + temp_modifier
        } else {
            0
        };

        let lower = 0;
        let upper = (TABLES.hydro_table.len() - 1) as i32;
        let index = roll.clamp(lower, upper) as usize;
        self.hydrographics = TABLES.hydro_table[index].clone();
    }

    fn generate_population(&mut self) {
        // By default, the population roll is a straight 2d6-2; in my opinion
        // the population of a planet should be modified by the habitability of
        // its size, atmosphere, and hydrographic. See this reddit post for the
        // inspiration:
        // https://www.reddit.com/r/traveller/comments/2xoqyy/mgt_new_to_traveller_question_about_worlds/cp29vt1/

        // We keep the unmodified roll to use as the modifier for the government rolls;
        // this is to avoid too high of an average law level in the subsector

        let size_mod: i32 = match self.size {
            7..=9 => 1,
            _ => -1,
        };

        let atmo_mod: i32 = match self.atmosphere.code {
            5..=8 => 1,
            _ => -1,
        };

        let hydro_mod: i32 = match self.hydrographics.code {
            2..=8 => 1,
            _ => -1,
        };

        let lower = 0;
        let upper = (TABLES.pop_table.len() - 1) as i32;

        let roll = dice::roll_2d(6) - 2;
        let modified_roll = roll + size_mod + atmo_mod + hydro_mod;
        let index = modified_roll.clamp(lower, upper) as usize;

        self.unmodified_pop = roll.clamp(lower, upper) as u16;
        self.population = TABLES.pop_table[index].clone();
    }

    fn generate_government(&mut self) {
        if self.population.code == 0 {
            self.government = TABLES.gov_table[0].clone();
            return;
        }

        let lower = 0;
        let upper = (TABLES.gov_table.len() - 1) as i32;
        let roll: i32 = dice::roll_2d(6) - 7 + self.unmodified_pop as i32;
        let index = roll.clamp(lower, upper) as usize;
        self.government = TABLES.gov_table[index].clone();
    }

    fn generate_law_level(&mut self) {
        if self.population.code == 0 {
            self.law_level = TABLES.law_table[0].clone();
            return;
        }

        let lower = 0;
        let upper = (TABLES.law_table.len() - 1) as i32;
        let roll: i32 = dice::roll_2d(6) - 7 + self.government.code as i32;
        let index = roll.clamp(lower, upper) as usize;
        self.law_level = TABLES.law_table[index].clone();
    }

    fn generate_factions(&mut self) {
        if self.population.code == 0 {
            return;
        }

        let faction_count = dice::roll_1d(3)
            + match self.government.code {
                0 | 7 => 1,
                x if x >= 10 => -1,
                _ => 0,
            };

        for _ in 0..faction_count {
            let gov_roll = dice::roll_2d(6);
            let faction_roll = dice::roll_2d(6);

            self.factions.push(Faction {
                name: String::from(""),
                code: TABLES.faction_table[faction_roll].code,
                strength: TABLES.faction_table[faction_roll].strength.clone(),
                government: TABLES.gov_table[gov_roll].clone(),
            });
        }
    }

    fn generate_culture(&mut self) {
        let range = 0..TABLES.culture_table.len();
        let roll = dice::roll(range);
        self.culture = TABLES.culture_table[roll].clone();
    }

    fn generate_world_tags(&mut self) {
        for tag in self.world_tags.iter_mut() {
            let range = 0..TABLES.world_tag_table.len();
            let roll = dice::roll(range);
            *tag = TABLES.world_tag_table[roll].clone();
        }
    }

    fn generate_starport(&mut self) {
        let pop_mod: i32 = match self.population.code {
            8..=9 => 1,
            x if x >= 10 => 2,
            3..=4 => -1,
            x if x <= 2 => -2,
            _ => 0,
        };

        let lower = 0;
        let upper = (TABLES.starport_table.len() - 1) as i32;
        let roll = dice::roll_2d(6) + pop_mod;
        let index = roll.clamp(lower, upper) as usize;
        self.starport = TABLES.starport_table[index].clone();

        self.starport.berthing_cost *= dice::roll_1d(6);
    }

    fn generate_tech_level(&mut self) {
        let size_mod = match self.size {
            0..=1 => 2,
            2..=4 => 1,
            _ => 0,
        };

        let atmo_mod = match self.atmosphere.code {
            0..=3 => 1,
            10..=15 => 1,
            _ => 0,
        };

        let hydro_mod = match self.hydrographics.code {
            0 => 1,
            9 => 1,
            10 => 2,
            _ => 0,
        };

        let pop_mod = match self.population.code {
            1..=5 => 1,
            8 => 1,
            9 => 2,
            10 => 4,
            _ => 0,
        };

        let gov_mod = match self.government.code {
            0 => 1,
            5 => 1,
            7 => 2,
            13..=14 => -2,
            _ => 0,
        };

        let starport_mod = match self.starport.class {
            StarportClass::A => 6,
            StarportClass::B => 4,
            StarportClass::C => 2,
            StarportClass::X => -4,
            _ => 0,
        };

        let roll: i16 =
            dice::roll_1d(6) + size_mod + atmo_mod + hydro_mod + pop_mod + gov_mod + starport_mod;
        self.tech_level = roll.clamp(0, 30) as u16;
    }

    fn generate_bases(&mut self) {
        let naval_target;
        let scout_target;
        let research_target;
        let tas_target;
        match self.starport.class {
            StarportClass::A => {
                naval_target = 8;
                scout_target = 10;
                research_target = 8;
                tas_target = 0; // Guaranteed
            }

            StarportClass::B => {
                naval_target = 8;
                scout_target = 8;
                research_target = 10;
                tas_target = 0; // Guaranteed
            }

            StarportClass::C => {
                naval_target = 100; // Impossible
                scout_target = 8;
                research_target = 10;
                tas_target = 10;
            }

            StarportClass::D => {
                naval_target = 100; // Impossible
                scout_target = 7;
                research_target = 100; // Impossible
                tas_target = 100; // Impossible
            }

            _ => {
                naval_target = 100; // Impossible
                scout_target = 100; // Impossible
                research_target = 100; // Impossible
                tas_target = 100; // Impossible
            }
        }

        self.has_naval_base = dice::roll_2d(6) >= naval_target;
        self.has_scout_base = dice::roll_2d(6) >= scout_target;
        self.has_research_base = dice::roll_2d(6) >= research_target;
        self.has_tas = dice::roll_2d(6) >= tas_target;
    }

    fn resolve_travel_code(&mut self) {
        self.travel_code = TravelCode::Safe;

        match self.atmosphere.code {
            x if x >= 10 => self.travel_code = TravelCode::Amber,
            _ => (),
        }

        match self.government.code {
            0 | 7 | 10 => self.travel_code = TravelCode::Amber,
            _ => (),
        }

        match self.law_level.code {
            0 => self.travel_code = TravelCode::Amber,
            x if x >= 9 => self.travel_code = TravelCode::Amber,
            _ => (),
        }
    }

    fn resolve_trade_codes(&mut self) {
        // Agricultural
        if (4..=9).contains(&self.atmosphere.code)
            && (4..=8).contains(&self.hydrographics.code)
            && (5..=7).contains(&self.population.code)
        {
            self.trade_codes.insert(TradeCode::Ag);
        }

        // Asteroid
        if self.size == 0 && self.atmosphere.code == 0 && self.hydrographics.code == 0 {
            self.trade_codes.insert(TradeCode::As);
        }

        // Barren
        if self.population.code == 0 && self.government.code == 0 && self.law_level.code == 0 {
            self.trade_codes.insert(TradeCode::Ba);
        }

        // Desert
        if self.atmosphere.code >= 2 && self.hydrographics.code == 0 {
            self.trade_codes.insert(TradeCode::De);
        }

        // Fluid (non-water) oceans
        if self.atmosphere.code >= 10 && self.hydrographics.code >= 1 {
            self.trade_codes.insert(TradeCode::Fl);
        }

        // Garden
        if (6..=8).contains(&self.size)
            && [5, 6, 8].contains(&self.atmosphere.code)
            && (5..=7).contains(&self.population.code)
        {
            self.trade_codes.insert(TradeCode::Ga);
        }

        // High population
        if self.population.code >= 9 {
            self.trade_codes.insert(TradeCode::Hi);
        }

        // High tech
        if self.tech_level >= 12 {
            self.trade_codes.insert(TradeCode::Ht);
        }

        // Ice-capped
        if (0..=1).contains(&self.atmosphere.code) && self.hydrographics.code >= 1 {
            self.trade_codes.insert(TradeCode::Ie);
        }

        // Industrial
        if ((0..=2).contains(&self.atmosphere.code) || [4, 7, 9].contains(&self.atmosphere.code))
            && self.population.code >= 9
        {
            self.trade_codes.insert(TradeCode::In);
        }

        // Low population
        if self.population.code <= 3 {
            self.trade_codes.insert(TradeCode::Lo);
        }

        // Low tech
        if self.tech_level <= 5 {
            self.trade_codes.insert(TradeCode::Lt);
        }

        // Non-agricultural
        if (0..=3).contains(&self.atmosphere.code)
            && (0..=3).contains(&self.hydrographics.code)
            && self.population.code >= 6
        {
            self.trade_codes.insert(TradeCode::Na);
        }

        // Non-industrial
        if self.population.code <= 6 {
            self.trade_codes.insert(TradeCode::Ni);
        }

        // Poor
        if (2..=5).contains(&self.atmosphere.code) && self.hydrographics.code <= 3 {
            self.trade_codes.insert(TradeCode::Po);
        }

        // Rich
        if [6, 8].contains(&self.atmosphere.code)
            && (6..=8).contains(&self.population.code)
            && (4..=9).contains(&self.government.code)
        {
            self.trade_codes.insert(TradeCode::Ri);
        }

        // Vacuum
        if self.atmosphere.code == 0 {
            self.trade_codes.insert(TradeCode::Va);
        }

        // Water world
        if self.hydrographics.code >= 10 {
            self.trade_codes.insert(TradeCode::Wa);
        }
    }

    #[allow(dead_code)]
    pub fn histograms(n: usize) {
        let mut gas_giant_hist = Histogram::with_domain("Gas Giant", [false, true]);
        let mut size_hist = Histogram::with_domain("Size", 0..=10);
        let mut atmo_hist =
            Histogram::with_domain("Atmosphere", 0..=(TABLES.atmo_table.len() as u16 - 1));
        let mut temp_hist =
            Histogram::with_domain("Temperature", 0..=(TABLES.temp_table.len() as u16 - 1));
        let mut hydro_hist =
            Histogram::with_domain("Hydrographics", 0..=(TABLES.hydro_table.len() as u16 - 1));
        let mut pop_hist =
            Histogram::with_domain("Population", 0..=(TABLES.pop_table.len() as u16 - 1));
        let mut gov_hist =
            Histogram::with_domain("Government", 0..=(TABLES.gov_table.len() as u16 - 1));
        let mut law_hist =
            Histogram::with_domain("Law Level", 0..=(TABLES.law_table.len() as u16 - 1));
        let mut fac_strength_hist = Histogram::with_domain(
            "Faction Strength",
            0..=(TABLES.faction_table.len() as u16 - 1),
        );
        let mut fac_count_hist = Histogram::new("Faction Count");
        let mut starport_hist = Histogram::new("Starport");
        let mut tech_hist = Histogram::new("Tech Level");
        let mut trade_code_hist = Histogram::new("Trade Codes");

        for _ in 0..n {
            let world = Self::new(String::from("0101"), Point { x: 0, y: 0 });

            gas_giant_hist.inc(world.has_gas_giant);
            size_hist.inc(world.size);
            atmo_hist.inc(world.atmosphere.code);
            temp_hist.inc(world.temperature.code);
            hydro_hist.inc(world.hydrographics.code);
            pop_hist.inc(world.population.code);
            gov_hist.inc(world.government.code);
            law_hist.inc(world.law_level.code);

            for faction in &world.factions {
                fac_strength_hist.inc(faction.code);
            }
            fac_count_hist.inc(world.factions.len());

            starport_hist.inc(world.starport.class);
            tech_hist.inc(world.tech_level);

            for trade_code in world.trade_codes {
                trade_code_hist.inc(trade_code);
            }
        }

        gas_giant_hist.show_percent(n / 50);
        size_hist.show_percent(n / 200);
        atmo_hist.show_percent(n / 200);
        temp_hist.show_percent(n / 200);
        hydro_hist.show_percent(n / 200);
        pop_hist.show_percent(n / 200);
        gov_hist.show_percent(n / 200);
        law_hist.show_percent(n / 200);
        fac_strength_hist.show_percent(n / 200);
        fac_count_hist.show_percent(n / 200);
        starport_hist.show_percent(n / 200);
        tech_hist.show_percent(n / 200);
        trade_code_hist.show(n / 100); // Percent doesn't work well for this one
    }
}

#[derive(Debug)]
struct Translation {
    x: f64,
    y: f64,
}

impl Add for &Translation {
    type Output = Translation;
    fn add(self, other: Self) -> Translation {
        Translation {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for &Translation {
    type Output = Translation;
    fn sub(self, other: Self) -> Translation {
        Translation {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Subsector {
    name: String,
    map: BTreeMap<Point, World>,
}

impl Subsector {
    const COLUMNS: usize = 8;
    const ROWS: usize = 10;

    fn empty() -> Self {
        Subsector {
            name: String::from("Subsector"),
            map: BTreeMap::new(),
        }
    }

    pub fn new(world_abundance_dm: i16) -> Self {
        let mut subsector = Self::empty();
        let mut names = random_names(Subsector::COLUMNS * Subsector::ROWS + 1).into_iter();
        subsector.name = names.next().unwrap();

        for x in 1..=Subsector::COLUMNS {
            for y in 1..=Subsector::ROWS {
                // Fifty-fifty chance with no modifiers
                let roll = dice::roll_1d(6) + world_abundance_dm;
                match roll {
                    4..=6 => {
                        let point = Point {
                            x: x as u16,
                            y: y as u16,
                        };
                        // let name = format!("{:0>2}{:0>2}", x, y);
                        let name = names.next().unwrap();
                        let world = World::new(name, point.clone());
                        subsector.map.insert(point, world);
                    }
                    _ => (),
                }
            }
        }
        subsector
    }

    #[allow(dead_code)]
    pub fn show(&self) {
        let mut hex_grid = fs::read_to_string("resources/hex_grid.txt").unwrap();
        for x in 1..=Subsector::COLUMNS {
            for y in 1..=Subsector::ROWS {
                let marker = format!(".{}", 100 * x + y);
                let point = Point {
                    x: x as u16,
                    y: y as u16,
                };

                if let Some(world) = &self.map.get(&point) {
                    if world.has_gas_giant {
                        hex_grid = hex_grid.replace(&marker, "G   ")
                    } else {
                        hex_grid = hex_grid.replace(&marker, "*   ");
                    }
                } else {
                    hex_grid = hex_grid.replace(&marker, "    ");
                }
            }
        }

        println!("{}\n", hex_grid);
    }

    #[allow(dead_code)]
    pub fn to_csv(&self) -> String {
        let mut summary_table = vec![String::from(
            "Name,Location,Profile,Bases,Trade Codes,Travel Code,Gas Giant",
        )];
        let mut societal_table = vec![String::from(
            "Name,Location,Culture,World Tag 1,World Tag 2",
        )];
        for (point, world) in &self.map {
            let location = format!("{:0>2}{:0>2}", point.x, point.y);
            summary_table.push(world.summary_csv(&location));
            societal_table.push(world.societal_csv(&location));
        }

        let table_separator = "-- >8 --";
        let mut table = summary_table;
        table.push(String::from(table_separator));
        table.append(&mut societal_table);
        table.join("\n")
    }

    #[allow(dead_code)]
    pub fn generate_svg(&self) -> String {
        let template_svg = fs::read_to_string("resources/traveller_sector_grid.svg").unwrap();
        let doc = xml::Document::parse(&template_svg).unwrap();

        // Parse through svg document to find coordinates of center markers
        let mut marker_coordinates: BTreeMap<Point, Translation> = BTreeMap::new();
        for (x, column) in doc
            .descendants()
            .find(|node| node.attribute("id") == Some("CenterMarkers"))
            .unwrap()
            .descendants()
            .filter(|node| {
                node.is_element()
                    && node.tag_name().name() == "g"
                    && node.attribute("id") != Some("CenterMarkers")
            })
            .enumerate()
        {
            let column_x: f64;
            let column_y: f64;
            if let Some(tranform) = column.attribute("transform") {
                let translate_args: Vec<&str> = tranform
                    .strip_prefix("translate(")
                    .unwrap()
                    .strip_suffix(")")
                    .unwrap()
                    .split(',')
                    .collect();

                column_x = match translate_args.get(0) {
                    Some(arg) => arg.parse().unwrap(),
                    None => 0.0,
                };

                column_y = match translate_args.get(1) {
                    Some(arg) => arg.parse().unwrap(),
                    None => 0.0,
                };
            } else {
                column_x = 0.0;
                column_y = 0.0;
            }

            let column_translation = Translation {
                x: column_x,
                y: column_y,
            };

            for (y, circle) in column
                .descendants()
                .filter(|node| node.tag_name().name() == "circle")
                .enumerate()
            {
                let circle_translation = Translation {
                    x: circle.attribute("cx").unwrap().parse().unwrap(),
                    y: circle.attribute("cy").unwrap().parse().unwrap(),
                };

                let point = Point {
                    x: x as u16 + 1,
                    y: y as u16 + 1,
                };

                marker_coordinates.insert(point, &column_translation + &circle_translation);
            }
        }

        // Find translations of all symbols in the map legend
        let gas_giant = doc
            .descendants()
            .find(|node| node.attribute("id") == Some("GasGiantSymbol"))
            .unwrap()
            .descendants()
            .find(|node| node.tag_name().name() == "circle")
            .unwrap();
        let gas_giant_trans = Translation {
            x: gas_giant.attribute("cx").unwrap().parse().unwrap(),
            y: gas_giant.attribute("cy").unwrap().parse().unwrap(),
        };

        let dry_world = doc
            .descendants()
            .find(|node| node.attribute("id") == Some("DryWorldSymbol"))
            .unwrap();
        let dry_world_trans = Translation {
            x: dry_world.attribute("cx").unwrap().parse().unwrap(),
            y: dry_world.attribute("cy").unwrap().parse().unwrap(),
        };

        let wet_world = doc
            .descendants()
            .find(|node| node.attribute("id") == Some("WetWorldSymbol"))
            .unwrap();
        let wet_world_trans = Translation {
            x: wet_world.attribute("cx").unwrap().parse().unwrap(),
            y: wet_world.attribute("cy").unwrap().parse().unwrap(),
        };

        let mut output_buffer: Vec<String> =
            template_svg.lines().map(|s| String::from(s)).collect();
        let close_svg = output_buffer.pop().unwrap();

        // Adding a "layer" called "Generated" to contain all the generated symbols
        output_buffer.push(String::from(
            "<g inkscape:groupmode=\"layer\" id=\"layer6\" inkscape:label=\"Generated\">",
        ));

        for (point, world) in &self.map {
            // Add gas giant symbol
            if world.has_gas_giant {
                let offset = Translation { x: 0.0, y: -6.0 };
                let translation = &(&marker_coordinates[point] - &gas_giant_trans) + &offset;
                output_buffer.push(format!(
                    "<use \
                    x=\"0\" \
                    y=\"0\" \
                    href=\"#{symbol}\" \
                    id=\"{id}\" \
                    width=\"100%\" \
                    height=\"100%\" \
                    transform=\"translate({translate_x},{translate_y})\"/>",
                    symbol = "GasGiantSymbol",
                    id = format!("{:02}{:02}GasGiantSymbol", point.x, point.y),
                    translate_x = translation.x,
                    translate_y = translation.y
                ));
            }

            // Add world name in center of hex
            output_buffer.push(format!(
                "<text \
                xml:space=\"preserve\" \
                style=\"font-style:normal;font-variant:normal;font-weight:normal;font-stretch:condensed;font-size:3.52777px;line-height:0;font-family:Arial;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
                x=\"{translate_x}\" \
                y=\"{translate_y}\" \
                id=\"{point_str}NameText\">\
                <tspan \
                sodipodi:role=\"line\" \
                id=\"{point_str}NameTspan\" \
                style=\"font-style:normal;font-variant:normal;font-weight:normal;font-stretch:condensed;font-family:Arial;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
                x=\"{translate_x}\" \
                y=\"{translate_y}\">{name}</tspan></text>",
                translate_x = marker_coordinates[point].x,
                translate_y = marker_coordinates[point].y,
                point_str = format!("{:02}{:02}", point.x, point.y),
                name = world.name
            ));

            // Decide whether to use dry or wet world symbol
            let (world_symbol, world_trans) = match world.hydrographics.code {
                h if h <= 3 => ("DryWorldSymbol", &dry_world_trans),
                _ => ("WetWorldSymbol", &wet_world_trans),
            };

            // Add dry/wet world symbol below and to the left of center
            let offset = Translation { x: -5.0, y: 4.0 };
            let translation = &(&marker_coordinates[point] - world_trans) + &offset;
            output_buffer.push(format!(
                "<use \
                x=\"0\" \
                y=\"0\" \
                href=\"#{symbol}\" \
                id=\"{id}\" \
                width=\"100%\" \
                height=\"100%\" \
                transform=\"translate({translate_x},{translate_y})\"/>",
                symbol = world_symbol,
                id = format!("{:02}{:02}{}", point.x, point.y, world_symbol),
                translate_x = translation.x,
                translate_y = translation.y
            ));

            // Add `StarportClass-TL` text to hex
            let offset = Translation { x: 5.0, y: 5.0 };
            let translation = &marker_coordinates[point] + &offset;
            output_buffer.push(format!(
                "<text \
                xml:space=\"preserve\" \
                style=\"font-style:italic;font-variant:normal;font-weight:normal;font-stretch:condensed;font-size:3.52777px;line-height:0;font-family:Arial;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
                x=\"{translate_x}\" \
                y=\"{translate_y}\" \
                id=\"{point_str}StarportTlText\">\
                <tspan \
                sodipodi:role=\"line\" \
                id=\"{point_str}StarportTlTspan\" \
                style=\"font-style:italic;font-variant:normal;font-weight:normal;font-stretch:condensed;font-family:Arial;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
                x=\"{translate_x}\" \
                y=\"{translate_y}\">{starport:?}-{tech_level}</tspan></text>",
                translate_x = translation.x,
                translate_y = translation.y,
                point_str = format!("{:02}{:02}", point.x, point.y),
                starport = world.starport.class,
                tech_level = world.tech_level
            ));

            // Add world profile code at bottom of hex
            let offset = Translation { x: 0.0, y: 10.0 };
            let translation = &marker_coordinates[point] + &offset;
            output_buffer.push(format!(
                "<text \
                xml:space=\"preserve\" \
                style=\"font-style:italic;font-variant:normal;font-weight:normal;font-stretch:condensed;font-size:2.8px;line-height:0;font-family:Arial;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
                x=\"{translate_x}\" \
                y=\"{translate_y}\" \
                id=\"{point_str}StarportTlText\">\
                <tspan \
                sodipodi:role=\"line\" \
                id=\"{point_str}StarportTlTspan\" \
                style=\"font-style:italic;font-variant:normal;font-weight:normal;font-stretch:condensed;font-family:Arial;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
                x=\"{translate_x}\" \
                y=\"{translate_y}\">{profile}</tspan></text>",
                translate_x = translation.x,
                translate_y = translation.y,
                point_str = format!("{:02}{:02}", point.x, point.y),
                profile = world.profile_string()
            ));
        }

        // Closing layer and svg document
        output_buffer.push(String::from("</g>"));
        output_buffer.push(close_svg);

        output_buffer.join("\n")
    }
}

pub fn random_names(count: usize) -> Vec<String> {
    let vowels = vec![
        vec![
            "b", "c", "d", "f", "g", "h", "i", "j", "k", "l", "m", "n", "p", "q", "r", "s", "t",
            "v", "w", "x", "y", "z",
        ],
        vec!["a", "e", "o", "u"],
        vec![
            "br", "cr", "dr", "fr", "gr", "pr", "str", "tr", "bl", "cl", "fl", "gl", "pl", "sl",
            "sc", "sk", "sm", "sn", "sp", "st", "sw", "ch", "sh", "th", "wh",
        ],
        vec![
            "ae", "ai", "ao", "au", "a", "ay", "ea", "ei", "eo", "eu", "e", "ey", "ua", "ue", "ui",
            "uo", "u", "uy", "ia", "ie", "iu", "io", "iy", "oa", "oe", "ou", "oi", "o", "oy",
        ],
        vec![
            "turn", "ter", "nus", "rus", "tania", "hiri", "hines", "gawa", "nides", "carro",
            "rilia", "stea", "lia", "lea", "ria", "nov", "phus", "mia", "nerth", "wei", "ruta",
            "tov", "zuno", "vis", "lara", "nia", "liv", "tera", "gantu", "yama", "tune", "ter",
            "nus", "cury", "bos", "pra", "thea", "nope", "tis", "clite",
        ],
        vec![
            "una", "ion", "iea", "iri", "illes", "ides", "agua", "olla", "inda", "eshan", "oria",
            "ilia", "erth", "arth", "orth", "oth", "illon", "ichi", "ov", "arvis", "ara", "ars",
            "yke", "yria", "onoe", "ippe", "osie", "one", "ore", "ade", "adus", "urn", "ypso",
            "ora", "iuq", "orix", "apus", "ion", "eon", "eron", "ao", "omia",
        ],
    ];

    let matrix = vec![
        vec![1, 1, 2, 2, 5, 5],
        vec![2, 2, 3, 3, 6, 6],
        vec![3, 3, 4, 4, 5, 5],
        vec![4, 4, 3, 3, 6, 6],
        vec![3, 3, 4, 4, 2, 2, 5, 5],
        vec![2, 2, 1, 1, 3, 3, 6, 6],
        vec![3, 3, 4, 4, 2, 2, 5, 5],
        vec![4, 4, 3, 3, 1, 1, 6, 6],
        vec![3, 3, 4, 4, 1, 1, 4, 4, 5, 5],
        vec![4, 4, 1, 1, 4, 4, 3, 3, 6, 6],
    ];

    let mut ret: Vec<String> = Vec::new();

    let mut rng = rand::thread_rng();
    for c in 0..count {
        let mut name = String::from("");
        let component = &matrix[c % matrix.len()];
        let length = component.len() / 2;

        for i in 0..length {
            let idx = component[2 * i + 1] - 1;
            let idx = rng.gen_range(0..vowels[idx].len());
            name.push_str(vowels[component[i * 2] - 1][idx]);
        }

        // Capitalize name
        let mut c = name.chars();
        let name = match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        };

        ret.push(name);
    }

    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsector_creation() {
        const ATTEMPTS: usize = 1000;
        for _ in 0..ATTEMPTS {
            Subsector::new();
        }
    }

    #[test]
    fn subsector_serde() {
        const ATTEMPTS: usize = 100;
        for _ in 0..ATTEMPTS {
            let subsector = Subsector::new();
            let yaml = serde_yaml::to_string(&subsector).unwrap();

            let de_subsector: Subsector = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(de_subsector, subsector);
            assert_eq!(subsector, de_subsector);
        }
    }
}
