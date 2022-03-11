use std::fs;

mod dice;
mod table;

use table:: {
    AtmoRecord,
    TempRecord,
    HydroRecord,
    PopRecord,
    GovRecord,
    FactionRecord,
    CulturalDiffRecord,
    WorldTagRecord,
    LawRecord,
    StarportRecord,
    TABLES,
};

enum AtmoType {
    None,
    Trace,
    VeryThinTainted,
    VeryThin,
    ThinTainted,
    Thin,
    Standard,
    StandardTainted,
    Dense,
    DenseTainted,
    Exotic,
    Corrosive,
    Insidious,
    VeryDense,
    Low,
    Unusual,
}

enum TempType {
    Frozen,
    Cold,
    Temperate,
    Hot,
    Boiling,
}

struct Faction {
    name: String,
    code: isize,
    strength: String,
    government: GovRecord,
}

struct World {
    name: String,
    has_gas_giant: bool,
    size: isize,
    diameter: isize,
    gravity: f64,
    atmosphere: AtmoRecord,
    temperature: TempRecord,
    hydrographics: HydroRecord,
    population: PopRecord,
    unmodified_pop: isize,
    government: GovRecord,
    factions: Vec<Faction>,
    culture: CulturalDiffRecord,
    world_tags: [WorldTagRecord; 2],
    starport: StarportRecord,
}

impl World {
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
                TABLES.world_tag_table[0].clone()
            ],
            starport: TABLES.starport_table[0].clone(),
        }
    }
    
    pub fn new() -> Self {
        let mut world = Self::empty();

        // Generation *must* happen in this order, many fields depend on the value
        // of other fields when making their rolls
        world.generate_size();
        world.generate_atmosphere();
        world.generate_temperature();
        world.generate_hydrographics();
        world.generate_population();
        world.generate_government();
        world.generate_factions();
        world.generate_culture();
        
        world
    }

    fn generate_size(&mut self) {
        self.size = (dice::roll_2d(6) - 2).clamp(0, 10);

        let median = match self.size {
            0 => 700,
            _ => 1600 * self.size,
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
        let upper = TABLES.atmo_table.len() as isize - 1;
        let roll = dice::roll_2d(6) - 7 + self.size;
        let index = roll.clamp(lower, upper) as usize;
        self.atmosphere = TABLES.atmo_table[index].clone();
    }

    fn generate_temperature(&mut self) {
        let atmo_modifier = match self.atmosphere.code {
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
        let upper = TABLES.temp_table.len() as isize - 1;
        let roll = dice::roll_2d(6) + atmo_modifier;
        let index = roll.clamp(lower, upper) as usize;
        self.temperature = TABLES.temp_table[index].clone();
    }

    fn generate_hydrographics(&mut self) {
        let roll = if self.size > 1 {
            let atmo_modifier = match self.atmosphere.code {
                0 | 1 | 10 | 11 | 12 => -4,
                _ => 0,
            };

            let temp_modifier = if self.atmosphere.code != 13 {
                match self.temperature.code {
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
        let upper = TABLES.hydro_table.len() as isize - 1;
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
        
        let size_mod = match self.size {
            7..=9 => 1,
            _ => -1,
        };
        
        let atmo_mod = match self.atmosphere.code {
            5..=8 => 1,
            _ => -1,
        };
        
        let hydro_mod = match self.hydrographics.code {
            2..=8 => 1,
            _ => -1,
        };

        let lower = 0;
        let upper = TABLES.pop_table.len() as isize - 1;

        let roll = dice::roll_2d(6) - 2;
        let modified_roll = roll + size_mod + atmo_mod + hydro_mod;
        let index = modified_roll.clamp(lower, upper) as usize;

        self.unmodified_pop = roll.clamp(lower, upper);
        self.population = TABLES.pop_table[index].clone();
    }

    fn generate_government(&mut self) {
        let lower = 0;
        let upper = TABLES.gov_table.len() as isize - 1;
        let roll = dice::roll_2d(6) - 7 + self.unmodified_pop;
        let index = roll.clamp(lower, upper) as usize;
        self.government = TABLES.gov_table[index].clone();
    }

    fn generate_factions(&mut self) {
        let faction_count = dice::roll_1d(3) + match self.government.code {
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

    fn generate_culture(& mut self) {
        let range = 0..TABLES.culture_table.len();
        let roll = dice::roll(range);
        self.culture = TABLES.culture_table[roll].clone();
    }
}

type Hex = Option<World>;

type SubsectorMap = [[Hex; Subsector::ROWS]; Subsector::COLUMNS];

pub struct Subsector {
    name: String,
    map: SubsectorMap,
}

impl Subsector {
    const COLUMNS: usize = 8;
    const ROWS: usize = 10;

    fn empty() -> Self {
        // Need to make init values as consts, otherwise `World` needs `Copy` trait;
        // array is initialized by copying `None` when initializing with `map: [[None; 8]; 10]`
        const INIT_HEX: Hex = None;
        const INIT_COL: [Hex; Subsector::ROWS] = [INIT_HEX; Subsector::ROWS];
        Subsector {
            name: String::from(""),
            map: [INIT_COL; Subsector::COLUMNS],
        }
    }

    pub fn new() -> Self {
        let mut subsector = Self::empty();
        subsector.generate_worlds(0);

        subsector
    }

    fn generate_worlds(&mut self, world_abundance_dm: isize) {
        for column in &mut self.map {
            for hex in column {
                // Fifty-fifty with no modifiers
                let range = 4..=6;
                let roll = dice::roll_1d6() + world_abundance_dm;
                if range.contains(&roll) {
                    *hex = Some(World::new());
                } else {
                    *hex = None;
                }
            }
        }
    }

    pub fn show(&self) {
        let mut hex_grid = fs::read_to_string("resources/hex_grid.txt").unwrap();
        for i in 0..Subsector::COLUMNS {
            for j in 0..Subsector::ROWS {
                let column = i + 1;
                let row = j + 1;
                let marker = format!(".{}", 100 * column + row);

                if let Some(star_system) = &self.map[i][j] {
                    if star_system.has_gas_giant {
                        hex_grid = hex_grid.replace(&marker, "G   ")
                    } else {
                        hex_grid = hex_grid.replace(&marker, "*   ");
                    }
                } else {
                    hex_grid = hex_grid.replace(&marker, "    ");
                }
            }
        }

        println!("{}", hex_grid);
    }
}
