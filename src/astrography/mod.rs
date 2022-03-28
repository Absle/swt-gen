use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use serde::{Deserialize, Serialize};

use super::dice;
use super::histogram::Histogram;
use super::markdown;

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

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct Point {
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
    #[allow(dead_code)]
    fn to_markdown(&self) {
        let mut doc = markdown::Document::new();
        doc.h2(&self.name);
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
            let world = Self::new(String::from("0101"));

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

    pub fn new(name: String) -> Self {
        let mut world = Self::empty();
        world.name = name;

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

    pub fn new() -> Self {
        let mut subsector = Self::empty();
        subsector.generate_worlds(0);
        subsector
    }

    fn generate_worlds(&mut self, world_abundance_dm: u16) {
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
                        let name = format!("{:0>2}{:0>2}", x, y);
                        self.map.insert(point, World::new(name));
                    }
                    _ => (),
                }
            }
        }
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
    pub fn to_markdown(&self) {
        let mut doc = markdown::Document::new();
        doc.h1(&self.name);
    }
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
