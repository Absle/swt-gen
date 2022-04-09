use std::collections::BTreeSet;
use std::error::Error;

use serde::{Deserialize, Serialize};

use super::table::{
    AtmoRecord, CulturalDiffRecord, GovRecord, HydroRecord, LawRecord, PopRecord, StarportClass,
    StarportRecord, TempRecord, WorldTagRecord, TABLES,
};
use super::Point;
use crate::dice;
use crate::histogram::Histogram;

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub struct Faction {
    name: String,
    code: u16,
    strength: String,
    government: GovRecord,
}

impl PartialEq for Faction {
    fn eq(&self, other: &Self) -> bool {
        // We ignore the `code` field because it's lost during serialization
        self.name == other.name
            && self.strength == other.strength
            && self.government == other.government
    }
}

impl TryFrom<SimpleFaction> for Faction {
    type Error = Box<dyn Error>;
    fn try_from(simple: SimpleFaction) -> Result<Self, Self::Error> {
        let name = simple.name;

        // Because multiple rolls can yield the same factions,
        // we can not save this value
        let code = TABLES
            .faction_table
            .iter()
            .find(|fac| fac.strength == simple.strength)
            .ok_or(format!(
                "Could not parse faction strength '{}'",
                simple.strength
            ))?
            .code;

        let strength = simple.strength;

        let government = TABLES
            .gov_table
            .iter()
            .find(|gov| gov.kind == simple.government)
            .ok_or(format!(
                "Could not parse faction government '{}'",
                simple.government
            ))?
            .clone();

        Ok(Self {
            name,
            code,
            strength,
            government,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct SimpleFaction {
    name: String,
    strength: String,
    government: String,
}

impl SimpleFaction {
    fn empty() -> Self {
        Self {
            name: String::new(),
            strength: String::new(),
            government: String::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TravelCode {
    Safe,
    Amber,
    Red,
}

impl TryFrom<&str> for TravelCode {
    type Error = Box<dyn Error>;
    fn try_from(string: &str) -> Result<Self, Self::Error> {
        match &string[..] {
            "Safe" => Ok(TravelCode::Safe),
            "Amber" => Ok(TravelCode::Amber),
            "Red" => Ok(TravelCode::Red),
            _ => Err(format!("Could not parse travel code '{}'", string).into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum TradeCode {
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

impl TryFrom<&str> for TradeCode {
    type Error = Box<dyn Error>;
    fn try_from(string: &str) -> Result<Self, Self::Error> {
        match string {
            "Ag" => Ok(TradeCode::Ag),
            "As" => Ok(TradeCode::As),
            "Ba" => Ok(TradeCode::Ba),
            "De" => Ok(TradeCode::De),
            "Fl" => Ok(TradeCode::Fl),
            "Ga" => Ok(TradeCode::Ga),
            "Hi" => Ok(TradeCode::Hi),
            "Ht" => Ok(TradeCode::Ht),
            "Ie" => Ok(TradeCode::Ie),
            "In" => Ok(TradeCode::In),
            "Lo" => Ok(TradeCode::Lo),
            "Lt" => Ok(TradeCode::Lt),
            "Na" => Ok(TradeCode::Na),
            "Ni" => Ok(TradeCode::Ni),
            "Po" => Ok(TradeCode::Po),
            "Ri" => Ok(TradeCode::Ri),
            "Va" => Ok(TradeCode::Va),
            "Wa" => Ok(TradeCode::Wa),
            _ => Err(format!("Could not parse trade code '{}'", string).into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub struct World {
    pub name: String,
    pub location: Point,
    pub has_gas_giant: bool,
    pub size: u16,
    pub diameter: u32,
    pub atmosphere: AtmoRecord,
    pub temperature: TempRecord,
    pub hydrographics: HydroRecord,
    pub population: PopRecord,
    pub unmodified_pop: u16,
    pub government: GovRecord,
    pub law_level: LawRecord,
    pub factions: Vec<Faction>,
    pub culture: CulturalDiffRecord,
    pub world_tags: [WorldTagRecord; 2],
    pub starport: StarportRecord,
    pub tech_level: u16,
    pub has_naval_base: bool,
    pub has_scout_base: bool,
    pub has_research_base: bool,
    pub has_tas: bool,
    pub travel_code: TravelCode,
    pub trade_codes: BTreeSet<TradeCode>,
}

impl World {
    pub fn profile(&self) -> String {
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
                name: String::from("Unnamed"),
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
        self.tech_level = roll.clamp(0, 15) as u16;
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

impl PartialEq for World {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.location == other.location
            && self.has_gas_giant == other.has_gas_giant
            && self.size == other.size
            && self.diameter == other.diameter
            && self.atmosphere == other.atmosphere
            && self.temperature == other.temperature
            && self.hydrographics == other.hydrographics
            && self.population == other.population
            && self.government == other.government
            && self.law_level == other.law_level
            && self.factions == other.factions
            && self.culture == other.culture
            && self.world_tags == other.world_tags
            && self.starport == other.starport
            && self.tech_level == other.tech_level
            && self.has_naval_base == other.has_naval_base
            && self.has_scout_base == other.has_scout_base
            && self.has_research_base == other.has_research_base
            && self.has_tas == other.has_tas
            && self.travel_code == other.travel_code
            && self.trade_codes == other.trade_codes
    }
}

impl TryFrom<WorldRecord> for World {
    type Error = Box<dyn Error>;
    fn try_from(record: WorldRecord) -> Result<Self, Self::Error> {
        let profile = record.profile.split("-").collect::<Vec<_>>().join("");
        //dbg!(format!("profile: {}", profile));
        let mut chars = profile.chars();

        // Parsing profile string
        let starport_err = "Failed parse starport class";
        let maybe_starport: Result<StarportClass, Self::Error> =
            match chars.next().ok_or("Empty world profile")? {
                'A' => Ok(StarportClass::A),
                'B' => Ok(StarportClass::B),
                'C' => Ok(StarportClass::C),
                'D' => Ok(StarportClass::D),
                'E' => Ok(StarportClass::E),
                'X' => Ok(StarportClass::X),
                _ => Err(starport_err.into()),
            };
        let starport_class = maybe_starport?;
        let mut starport = TABLES
            .starport_table
            .iter()
            .find(|item| item.class == starport_class)
            .ok_or(starport_err)?
            .clone();
        starport.berthing_cost = record.berthing_cost;

        let mut size = 0;
        let mut atmo = 0;
        let mut hydro = 0;
        let mut pop = 0;
        let mut gov = 0;
        let mut law = 0;
        let mut tech = 0;
        for (c, field) in chars.zip([
            &mut size, &mut atmo, &mut hydro, &mut pop, &mut gov, &mut law, &mut tech,
        ]) {
            *field = match c {
                'A' => 10,
                'B' => 11,
                'C' => 12,
                'D' => 13,
                'E' => 14,
                'F' => 15,
                _ => c.to_string().parse()?,
            };
        }

        //dbg!(format!("tech = {}", tech));

        let temperature = TABLES
            .temp_table
            .iter()
            .find(|item| item.kind == record.temperature)
            .ok_or(format!(
                "Failed to parse temperature '{}'",
                record.temperature
            ))?
            .clone();

        let mut factions = Vec::new();
        for faction_record in [
            record.faction_1,
            record.faction_2,
            record.faction_3,
            record.faction_4,
        ] {
            if faction_record == SimpleFaction::empty() {
                break;
            }
            factions.push(Faction::try_from(faction_record)?);
        }

        let culture = TABLES
            .culture_table
            .iter()
            .find(|item| item.cultural_difference == record.culture)
            .ok_or(format!("Failed to parse culture '{}'", record.culture))?
            .clone();

        let mut world_tags = [
            WorldTagRecord {
                code: 0,
                tag: String::new(),
                description: String::new(),
            },
            WorldTagRecord {
                code: 0,
                tag: String::new(),
                description: String::new(),
            },
        ];
        for (i, tag) in [record.world_tag_1, record.world_tag_2].iter().enumerate() {
            world_tags[i] = TABLES
                .world_tag_table
                .iter()
                .find(|item| item.tag == *tag)
                .ok_or(format!("Failed to parse world tag '{}'", tag))?
                .clone();
        }

        let has_naval_base = record.bases.contains("N");
        let has_scout_base = record.bases.contains("S");
        let has_research_base = record.bases.contains("R");
        let has_tas = record.bases.contains("T");

        let mut trade_codes = BTreeSet::new();
        for code in record.trade_codes.split(" ") {
            if code == "" {
                continue;
            }
            trade_codes.insert(TradeCode::try_from(code)?);
        }

        Ok(Self {
            name: record.name,
            location: Point::try_from(&record.location[..])?,
            has_gas_giant: &record.gas_giant == "G",
            size: size as u16,
            diameter: record.diameter,
            atmosphere: TABLES.atmo_table[atmo].clone(),
            temperature: temperature,
            hydrographics: TABLES.hydro_table[hydro].clone(),
            population: TABLES.pop_table[pop].clone(),
            // The true value of this is lost, but it's not needed after generation
            unmodified_pop: pop as u16,
            government: TABLES.gov_table[gov].clone(),
            law_level: TABLES.law_table[law].clone(),
            factions: factions,
            culture: culture,
            world_tags: world_tags,
            starport: starport,
            tech_level: tech as u16,
            has_naval_base: has_naval_base,
            has_scout_base: has_scout_base,
            has_research_base: has_research_base,
            has_tas: has_tas,
            travel_code: TravelCode::try_from(&record.travel_code[..])?,
            trade_codes: trade_codes,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WorldRecord {
    // Summary
    pub name: String,
    location: String,
    profile: String,
    bases: String,
    trade_codes: String,
    travel_code: String,
    gas_giant: String,
    berthing_cost: u32,

    empty_column_1: String,

    // Societal
    soc_name: String,
    soc_location: String,
    government: String,
    contraband: String,
    culture: String,
    world_tag_1: String,
    world_tag_2: String,
    faction_1: SimpleFaction,
    faction_2: SimpleFaction,
    faction_3: SimpleFaction,
    faction_4: SimpleFaction,

    empty_column_2: String,

    // Physical details
    det_name: String,
    det_location: String,
    diameter: u32,
    atmosphere: String,
    temperature: String,
    hydrographics: String,
    population: String,
}

impl From<World> for WorldRecord {
    fn from(world: World) -> Self {
        // Summary
        let name = world.name.clone();
        let location = format!("'{}", world.location.to_string());
        let profile = world.profile();

        let mut bases = Vec::new();
        if world.has_naval_base {
            bases.push(String::from("N"));
        }
        if world.has_research_base {
            bases.push(String::from("R"));
        }
        if world.has_scout_base {
            bases.push(String::from("S"));
        }
        if world.has_tas {
            bases.push(String::from("T"));
        }
        let bases = bases.join(" ");

        let trade_codes = world
            .trade_codes
            .iter()
            .map(|code| format!("{:?}", code))
            .collect::<Vec<String>>()
            .join(" ");

        let travel_code = format!("{:?}", world.travel_code);
        let gas_giant = match world.has_gas_giant {
            true => String::from("G"),
            false => String::new(),
        };

        let berthing_cost = world.starport.berthing_cost;

        // Societal
        let soc_name = name.clone();
        let soc_location = location.clone();
        let government = world.government.kind;
        let contraband = world.government.contraband;
        let culture = world.culture.cultural_difference;
        let world_tag_1 = world.world_tags[0].tag.clone();
        let world_tag_2 = world.world_tags[1].tag.clone();

        let faction_1: SimpleFaction;
        let faction_2: SimpleFaction;
        let faction_3: SimpleFaction;
        let faction_4: SimpleFaction;

        match world.factions.get(0) {
            Some(faction) => {
                faction_1 = SimpleFaction {
                    name: faction.name.clone(),
                    strength: faction.strength.clone(),
                    government: faction.government.kind.clone(),
                };
            }
            None => faction_1 = SimpleFaction::empty(),
        }

        match world.factions.get(1) {
            Some(faction) => {
                faction_2 = SimpleFaction {
                    name: faction.name.clone(),
                    strength: faction.strength.clone(),
                    government: faction.government.kind.clone(),
                };
            }
            None => faction_2 = SimpleFaction::empty(),
        }

        match world.factions.get(2) {
            Some(faction) => {
                faction_3 = SimpleFaction {
                    name: faction.name.clone(),
                    strength: faction.strength.clone(),
                    government: faction.government.kind.clone(),
                };
            }
            None => faction_3 = SimpleFaction::empty(),
        }

        match world.factions.get(3) {
            Some(faction) => {
                faction_4 = SimpleFaction {
                    name: faction.name.clone(),
                    strength: faction.strength.clone(),
                    government: faction.government.kind.clone(),
                };
            }
            None => faction_4 = SimpleFaction::empty(),
        }

        // Physical details
        let det_name = name.clone();
        let det_location = location.clone();
        let diameter = world.diameter;
        let atmosphere = world.atmosphere.composition;
        let temperature = world.temperature.kind;
        let hydrographics = world.hydrographics.description;
        let population = world.population.inhabitants;

        let empty_column_1 = String::new();
        let empty_column_2 = String::new();

        WorldRecord {
            name,
            location,
            profile,
            bases,
            trade_codes,
            travel_code,
            gas_giant,
            berthing_cost,

            empty_column_1,

            soc_name,
            soc_location,
            government,
            contraband,
            culture,
            world_tag_1,
            world_tag_2,
            faction_1,
            faction_2,
            faction_3,
            faction_4,

            empty_column_2,

            det_name,
            det_location,
            diameter,
            atmosphere,
            temperature,
            hydrographics,
            population,
        }
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
    let mut law_hist = Histogram::with_domain("Law Level", 0..=(TABLES.law_table.len() as u16 - 1));
    let mut fac_strength_hist = Histogram::with_domain(
        "Faction Strength",
        0..=(TABLES.faction_table.len() as u16 - 1),
    );
    let mut fac_count_hist = Histogram::new("Faction Count");
    let mut starport_hist = Histogram::new("Starport");
    let mut tech_hist = Histogram::new("Tech Level");
    let mut trade_code_hist = Histogram::new("Trade Codes");

    for _ in 0..n {
        let world = World::new(String::from("0101"), Point { x: 0, y: 0 });

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_record_csv_serde() {
        let original = WorldRecord::from(World::new(String::from("Test"), Point { x: 0, y: 0 }));

        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(Vec::new());
        writer.serialize(original.clone()).unwrap();
        let data = String::from_utf8(writer.into_inner().unwrap()).unwrap();

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(data.as_bytes());
        let deserialized: WorldRecord = reader.deserialize().next().unwrap().unwrap();

        assert_eq!(deserialized, original);
    }
}
