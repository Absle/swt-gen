use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::astrography::{
    AtmoRecord, CulturalDiffRecord, GovRecord, HydroRecord, LawRecord, PopRecord, StarportClass,
    StarportRecord, Table, TechLevelRecord, TempRecord, WorldTagRecord, TABLES,
};
use crate::dice;
use crate::histogram::Histogram;

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub(crate) struct Faction {
    pub(crate) name: String,
    pub(crate) code: u16,
    pub(crate) strength: String,
    pub(crate) government: GovRecord,
}

impl Faction {
    pub(crate) fn random() -> Faction {
        let faction_info = TABLES.faction_table.roll_normal_2d6(0);

        Faction {
            name: String::from("Unnamed"),
            code: faction_info.code,
            strength: faction_info.strength.clone(),
            government: TABLES.gov_table.roll_normal_2d6(0).clone(),
        }
    }
}

impl PartialEq for Faction {
    fn eq(&self, other: &Self) -> bool {
        // We ignore the `code` field because it's lost during serialization
        self.name == other.name
            && self.strength == other.strength
            && self.government == other.government
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum TravelCode {
    Safe,
    Amber,
    Red,
}

impl TravelCode {
    pub(crate) fn as_short_string(&self) -> String {
        match self {
            TravelCode::Safe => "-".to_string(),
            TravelCode::Amber => "A".to_string(),
            TravelCode::Red => "R".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) enum TradeCode {
    /// Agricultural
    Ag,
    /// Asteroid
    As,
    /// Barren
    Ba,
    /// Desert
    De,
    /// Fluid Oceans
    Fl,
    /// Garden
    Ga,
    /// High Population
    Hi,
    /// High Tech
    Ht,
    /// Ice-Capped
    Ic,
    /// Industrial
    In,
    /// Low Population
    Lo,
    /// Low Tech
    Lt,
    /// Non-Agricultural
    Na,
    /// Non-Industrial
    Ni,
    /// Poor
    Po,
    /// Rich
    Ri,
    /// Vacuum
    Va,
    /// Water World
    Wa,
}

impl TradeCode {
    fn to_long_str(&self) -> String {
        use TradeCode::*;
        match self {
            Ag => "Agricultural".to_string(),
            As => "Asteroid".to_string(),
            Ba => "Barren".to_string(),
            De => "Desert".to_string(),
            Fl => "Fluid Oceans".to_string(),
            Ga => "Garden".to_string(),
            Hi => "High Population".to_string(),
            Ht => "High Tech".to_string(),
            Ic => "Ice-Capped".to_string(),
            In => "Industrial".to_string(),
            Lo => "Low Population".to_string(),
            Lt => "Low Tech".to_string(),
            Na => "Non-Agricultural".to_string(),
            Ni => "Non-Industrial".to_string(),
            Po => "Poor".to_string(),
            Ri => "Rich".to_string(),
            Va => "Vacuum".to_string(),
            Wa => "Water World".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub(crate) struct World {
    pub(crate) name: String,
    pub(crate) has_gas_giant: bool,
    pub(crate) size: u16,
    pub(crate) diameter: u32,
    pub(crate) atmosphere: AtmoRecord,
    pub(crate) temperature: TempRecord,
    pub(crate) hydrographics: HydroRecord,
    pub(crate) population: PopRecord,
    pub(crate) unmodified_pop: u16,
    pub(crate) government: GovRecord,
    pub(crate) law_level: LawRecord,
    pub(crate) factions: Vec<Faction>,
    pub(crate) culture: CulturalDiffRecord,
    pub(crate) world_tags: [WorldTagRecord; Self::NUM_TAGS],
    pub(crate) starport: StarportRecord,
    pub(crate) tech_level: TechLevelRecord,
    pub(crate) has_naval_base: bool,
    pub(crate) has_scout_base: bool,
    pub(crate) has_research_base: bool,
    pub(crate) has_tas: bool,
    pub(crate) travel_code: TravelCode,
    pub(crate) trade_codes: BTreeSet<TradeCode>,
    pub(crate) notes: String,
}

impl World {
    pub(crate) const SIZE_MIN: u16 = 0;
    pub(crate) const SIZE_MAX: u16 = 10;

    pub(crate) const NUM_TAGS: usize = 2;

    /** Add a randomized faction and return its index. */
    pub(crate) fn add_faction(&mut self) -> usize {
        self.factions.push(Faction::random());
        self.factions.len() - 1
    }

    pub(crate) fn base_str(&self) -> String {
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
        let s = bases.join("");

        if !s.is_empty() {
            s
        } else {
            "-".to_string()
        }
    }

    pub(crate) fn empty() -> Self {
        World {
            name: String::from(""),
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
            tech_level: TABLES.tech_level_table[0].clone(),
            has_naval_base: false,
            has_scout_base: false,
            has_research_base: false,
            has_tas: false,
            travel_code: TravelCode::Safe,
            trade_codes: BTreeSet::new(),
            notes: String::new(),
        }
    }

    pub(crate) fn generate_atmosphere(&mut self) {
        let modifier = self.size as i32 - 7;
        self.atmosphere = TABLES.atmo_table.roll_normal_2d6(modifier).clone();
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
                naval_target = i32::MAX; // Impossible
                scout_target = 8;
                research_target = 10;
                tas_target = 10;
            }

            StarportClass::D => {
                naval_target = i32::MAX; // Impossible
                scout_target = 7;
                research_target = i32::MAX; // Impossible
                tas_target = i32::MAX; // Impossible
            }

            _ => {
                naval_target = i32::MAX; // Impossible
                scout_target = i32::MAX; // Impossible
                research_target = i32::MAX; // Impossible
                tas_target = i32::MAX; // Impossible
            }
        }

        self.has_naval_base = dice::roll_2d(6) >= naval_target;
        self.has_scout_base = dice::roll_2d(6) >= scout_target;
        self.has_research_base = dice::roll_2d(6) >= research_target;
        self.has_tas = dice::roll_2d(6) >= tas_target;
    }

    pub(crate) fn generate_berthing_cost(&mut self) {
        let index = self.starport.code as usize;
        self.starport.berthing_cost = dice::roll_1d(6) * TABLES.starport_table[index].berthing_cost;
    }

    pub(crate) fn generate_culture(&mut self) {
        self.culture = TABLES.culture_table.roll_uniform().clone();
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
            self.factions.push(Faction::random());
        }
    }

    fn generate_gas_giant(&mut self) {
        match dice::roll_2d(6) {
            0..=9 => self.has_gas_giant = true,
            _ => self.has_gas_giant = false,
        }
    }

    pub(crate) fn generate_government(&mut self) {
        if self.population.code == 0 {
            self.government = TABLES.gov_table[0].clone();
            return;
        }
        let modifier = self.unmodified_pop as i32 - 7;
        self.government = TABLES.gov_table.roll_normal_2d6(modifier).clone();
    }

    pub(crate) fn generate_hydrographics(&mut self) {
        if self.size <= 1 {
            self.hydrographics = TABLES.hydro_table[0].clone();
            return;
        }

        let atmo_modifier: i32 = match self.atmosphere.code {
            0 | 1 | 10 | 11 | 12 => -4,
            _ => 0,
        };

        let temp_modifier: i32 = if self.atmosphere.code != 13 {
            match self.temperature.code {
                10 | 11 => -2,
                12 => -6,
                _ => 0,
            }
        } else {
            0
        };

        let modifier = atmo_modifier + temp_modifier;
        self.hydrographics = TABLES.hydro_table.roll_normal_2d6(modifier).clone();
    }

    pub(crate) fn generate_law_level(&mut self) {
        if self.government.code == 0 {
            self.law_level = TABLES.law_table[0].clone();
            return;
        }
        let modifier = self.government.code as i32 - 7;
        self.law_level = TABLES.law_table.roll_normal_2d6(modifier).clone();
    }

    pub(crate) fn generate_population(&mut self) {
        // By default, the population roll is a straight 2d6-2; in my opinion
        // the population of a planet should be modified by the habitability of
        // its size, atmosphere, and hydrographic. Because of this, we borrow
        // the modifiers from the Cepheus Engine's world population modifiers.
        // See: https://www.orffenspace.com/cepheus-srd/worlds.html#world-population

        // We keep the unmodified roll to use as the modifier for the government rolls;
        // this is to avoid too high of an average law level in the subsector

        let size_mod: i32 = match self.size {
            0..=2 => -1,
            _ => 0,
        };

        let atmo_mod: i32 = match self.atmosphere.code {
            10..=u16::MAX => -2,
            6 => 3,
            5 | 8 => 1,
            _ => 0,
        };

        let hydro_mod: i32 = if self.hydrographics.code == 0 && self.atmosphere.code < 3 {
            -2
        } else {
            0
        };
        let modifier = size_mod + atmo_mod + hydro_mod - 2;

        let roll = dice::roll_2d(6);
        let modified_roll = roll + modifier;

        let lower = 0;
        let upper = (TABLES.pop_table.len() - 1) as i32;
        let index = modified_roll.clamp(lower, upper) as usize;

        self.unmodified_pop = roll.clamp(lower, upper) as u16;
        self.population = TABLES.pop_table[index].clone();
    }

    pub(crate) fn generate_size(&mut self) {
        self.size = (dice::roll_2d(6) - 2).clamp(Self::SIZE_MIN, Self::SIZE_MAX);

        let median: u32 = match self.size {
            0 => 700,
            _ => (1600 * self.size).into(),
        };
        let min = median - 200;
        let max = median + 200;
        self.diameter = dice::roll_range(min..=max);
    }

    pub(crate) fn generate_starport(&mut self) {
        let pop_mod: i32 = match self.population.code {
            8..=9 => 1,
            x if x >= 10 => 2,
            3..=4 => -1,
            x if x <= 2 => -2,
            _ => 0,
        };

        self.starport = TABLES.starport_table.roll_normal_2d6(pop_mod).clone();
        self.generate_berthing_cost();
    }

    pub(crate) fn generate_tech_level(&mut self) {
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

        let modifier = size_mod + atmo_mod + hydro_mod + pop_mod + gov_mod + starport_mod;
        self.tech_level = TABLES.tech_level_table.roll_normal_2d6(modifier).clone();
    }

    pub(crate) fn generate_temperature(&mut self) {
        let modifier: i32 = match self.atmosphere.code {
            0 | 1 => 0,
            2 | 3 => -2,
            4 | 5 | 14 => -1,
            6 | 7 => 0,
            8 | 9 => 1,
            10 | 13 | 15 => 2,
            11 | 12 => 6,
            _ => unreachable!("The atmosphere should always be in the range 0..=12"),
        };
        self.temperature = TABLES.temp_table.roll_normal_2d6(modifier).clone();
    }

    /** Mutate the world tag at `index` to a random one on the `world_tag_table`.

    Currently each world only has two world tags, so the only valid indices are `0` and `1`.

    # Returns
    - `Some(world_tag)` with the old, displaced world tag if `index` is valid, or
    - `None` otherwise
    */
    pub(crate) fn generate_world_tag(&mut self, index: usize) -> Option<WorldTagRecord> {
        match self.world_tags.get_mut(index) {
            Some(world_tag) => {
                let old_tag = world_tag.clone();
                *world_tag = TABLES.world_tag_table.roll_uniform().clone();
                Some(old_tag)
            }
            None => None,
        }
    }

    /** Regenerate all of the world's world tags. */
    fn generate_world_tags(&mut self) {
        for index in 0..self.world_tags.len() {
            self.generate_world_tag(index);
        }
    }

    pub(crate) fn gravity(&mut self) -> &str {
        match self.size {
            0 => "N/A",
            1 => "0.05 G",
            2 => "0.15 G",
            3 => "0.25 G",
            4 => "0.35 G",
            5 => "0.45 G",
            6 => "0.70 G",
            7 => "0.90 G",
            8 => "1.00 G",
            9 => "1.25 G",
            10 => "1.40 G",
            _ => unreachable!(),
        }
    }

    pub(crate) fn importance_extension(&self) -> String {
        let mut importance = 0;
        importance += match self.starport.class {
            StarportClass::A | StarportClass::B => 1,
            StarportClass::D | StarportClass::E | StarportClass::X => -1,
            _ => 0,
        };

        if self.tech_level.code >= 16 {
            importance += 1;
        }
        if self.tech_level.code >= 10 {
            importance += 1;
        }
        if self.tech_level.code <= 8 {
            importance -= 1;
        }

        const IMPORTANT_TRADE_CODES: [TradeCode; 4] =
            [TradeCode::Ag, TradeCode::Hi, TradeCode::In, TradeCode::Ri];
        for trade_code in IMPORTANT_TRADE_CODES {
            if self.trade_codes.contains(&trade_code) {
                importance += 1;
            }
        }

        if self.population.code <= 6 {
            importance -= 1;
        }

        if self.has_naval_base && self.has_scout_base {
            importance += 1;
        }

        format!("{{ {} }}", importance)
    }

    pub(crate) fn is_wet_world(&self) -> bool {
        self.hydrographics.code > 3
    }

    /** Attempts to mutate the `World` into a "player-safe" state.

    To do so, it defaults all of the fields that are likely to have spoilers to the zeroth index of
    their respective roll tables or completely blanks them where possible.
    These likely fields are:

    1. Factions
    2. Culture
    3. World Tags
    4. Notes

    This is intended to work alongside a player-safe version of the GUI that has the defaulted
    fields removed; this is more to prevent overly-clever players from mining the JSON for spoilers.
    */
    pub(crate) fn make_player_safe(&mut self) {
        self.factions.clear();
        self.culture = TABLES.culture_table[0].clone();
        for world_tag in self.world_tags.iter_mut() {
            *world_tag = TABLES.world_tag_table[0].clone();
        }
        self.notes = String::new();
    }

    /** Create a randomized `World` named `name` at `location`. */
    pub(crate) fn new(name: String) -> Self {
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

    pub(crate) fn profile_str(&self) -> String {
        format!(
            "{starport:?}{size:X}{atmo:X}{hydro:X}{pop:X}{gov:X}{law:X}-{tech:X}",
            starport = self.starport.class,
            size = self.size,
            atmo = self.atmosphere.code,
            hydro = self.hydrographics.code,
            pop = self.population.code,
            gov = self.government.code,
            law = self.law_level.code,
            tech = self.tech_level.code,
        )
    }

    /** Remove the [`Faction`] at `idx` and return the nearest valid index to `idx`.

    Does nothing and returns 0 if `idx` is out of bounds.
    */
    pub(crate) fn remove_faction(&mut self, idx: usize) -> usize {
        if idx >= self.factions.len() {
            return 0;
        }

        self.factions.remove(idx);
        if self.factions.is_empty() {
            0
        } else if idx >= self.factions.len() {
            self.factions.len() - 1
        } else {
            idx
        }
    }

    pub(crate) fn resolve_trade_codes(&mut self) {
        self.trade_codes.clear();

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
            && (5..=7).contains(&self.hydrographics.code)
        {
            self.trade_codes.insert(TradeCode::Ga);
        }

        // High population
        if self.population.code >= 9 {
            self.trade_codes.insert(TradeCode::Hi);
        }

        // High tech
        if self.tech_level.code >= 12 {
            self.trade_codes.insert(TradeCode::Ht);
        }

        // Ice-capped
        if (0..=1).contains(&self.atmosphere.code) && self.hydrographics.code >= 1 {
            self.trade_codes.insert(TradeCode::Ic);
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
        if self.tech_level.code <= 5 {
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

    pub(crate) fn resolve_travel_code(&mut self) {
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

    pub(crate) fn starport_tl_str(&self) -> String {
        format!("{:?}-{}", self.starport.class, self.tech_level.code)
    }

    pub(crate) fn trade_code_long_str(&self) -> String {
        self.trade_codes
            .iter()
            .map(|code| code.to_long_str())
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub(crate) fn trade_code_str(&self) -> String {
        let s = self
            .trade_codes
            .iter()
            .map(|code| format!("{:?}", code))
            .collect::<Vec<String>>()
            .join(" ");
        if !s.is_empty() {
            s
        } else {
            "-".to_string()
        }
    }

    pub(crate) fn travel_code_str(&self) -> String {
        format!("{:?}", self.travel_code)
    }
}

impl Default for World {
    fn default() -> Self {
        World::new("".to_string())
    }
}

impl PartialEq for World {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
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
            && self.notes == other.notes
    }
}

#[allow(dead_code)]
pub(crate) fn histograms(n: usize) {
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
    let mut tech_hist =
        Histogram::with_domain("Tech Level", 0..=(TABLES.tech_level_table.len() as u16 - 1));
    let mut trade_code_hist = Histogram::new("Trade Codes");

    for _ in 0..n {
        let world = World::new(String::from("0101"));

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
        tech_hist.inc(world.tech_level.code);

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

    // TODO: this, and other statistical analysis functions, should probably be moved into a
    // separate bin or something at some point
    #[allow(dead_code)]
    fn show_histograms() {
        histograms(100_000);
        // Purposefully fail get cargo test to show stdout and to make sure this doesn't get
        // commited as a test
        panic!();
    }
}
