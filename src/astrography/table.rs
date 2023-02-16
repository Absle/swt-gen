use std::{fmt, ops::Deref};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::dice;

const ATMO_TABLE_CSV: &str = include_str!("../../resources/tables/atmospheres.csv");
const TEMP_TABLE_CSV: &str = include_str!("../../resources/tables/temperatures.csv");
const HYDRO_TABLE_CSV: &str = include_str!("../../resources/tables/hydrographics.csv");
const POP_TABLE_CSV: &str = include_str!("../../resources/tables/populations.csv");
const GOV_TABLE_CSV: &str = include_str!("../../resources/tables/governments.csv");
const FACTION_TABLE_CSV: &str = include_str!("../../resources/tables/factions.csv");
const CULTURE_TABLE_CSV: &str = include_str!("../../resources/tables/cultural_differences.csv");
const WORLD_TAG_TABLE_CSV: &str = include_str!("../../resources/tables/world_tags.csv");
const LAW_TABLE_CSV: &str = include_str!("../../resources/tables/law_levels.csv");
const STARPORT_TABLE_CSV: &str = include_str!("../../resources/tables/starports.csv");

/** Trait representing a record or row in a table. */
trait Record {
    /** Get the `code` of this `Record`; i.e. its index in the table.

    This *must* match the physical row index of the `Record` in the table.
    */
    fn code(&self) -> u16;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AtmoRecord {
    pub(crate) code: u16,
    pub(crate) composition: String,
}

impl Record for AtmoRecord {
    fn code(&self) -> u16 {
        self.code
    }
}
type AtmoTable = Vec<AtmoRecord>;

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub(crate) struct TempRecord {
    pub(crate) code: u16,
    pub(crate) kind: String,
    pub(crate) description: String,
}

impl PartialEq for TempRecord {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.description == other.description
    }
}

impl Record for TempRecord {
    fn code(&self) -> u16 {
        self.code
    }
}
type TempTable = Vec<TempRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct HydroRecord {
    pub(crate) code: u16,
    pub(crate) description: String,
}

impl Record for HydroRecord {
    fn code(&self) -> u16 {
        self.code
    }
}
type HydroTable = Vec<HydroRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct PopRecord {
    pub(crate) code: u16,
    pub(crate) inhabitants: String,
}

impl Record for PopRecord {
    fn code(&self) -> u16 {
        self.code
    }
}
type PopTable = Vec<PopRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct GovRecord {
    pub(crate) code: u16,
    pub(crate) kind: String,
    pub(crate) description: String,
    pub(crate) contraband: String,
}

impl GovRecord {
    /** Mutate `self` into `other`, but retain non-default `description` and `contraband` fields. */
    pub(crate) fn safe_mutate(&mut self, other: &Self) {
        let Self {
            code: new_code,
            kind: new_kind,
            description: new_desc,
            contraband: new_contra,
        } = other;

        if self.description == TABLES.gov_table[self.code as usize].description {
            self.description = new_desc.clone();
        }
        if self.contraband == TABLES.gov_table[self.code as usize].contraband {
            self.contraband = new_contra.clone();
        }

        self.code = *new_code;
        self.kind = new_kind.clone();
    }
}

impl Record for GovRecord {
    fn code(&self) -> u16 {
        self.code
    }
}
type GovTable = Vec<GovRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct FactionStrengthRecord {
    pub(crate) code: u16,
    pub(crate) strength: String,
}

impl Record for FactionStrengthRecord {
    fn code(&self) -> u16 {
        self.code
    }
}
type FactionTable = Vec<FactionStrengthRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct CulturalDiffRecord {
    pub(crate) code: u16,
    pub(crate) cultural_difference: String,
    pub(crate) description: String,
}

impl CulturalDiffRecord {
    /** Mutate `self` into `other`, but retain non-default `description` fields. */
    pub(crate) fn safe_mutate(&mut self, other: &Self) {
        let Self {
            code: new_code,
            cultural_difference: new_culture,
            description: new_desc,
        } = other;

        if self.description == TABLES.culture_table[self.code as usize].description {
            self.description = new_desc.clone();
        }

        self.code = *new_code;
        self.cultural_difference = new_culture.clone();
    }
}

impl Record for CulturalDiffRecord {
    fn code(&self) -> u16 {
        self.code
    }
}
type CulturalDiffTable = Vec<CulturalDiffRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WorldTagRecord {
    pub(crate) code: u16,
    pub(crate) tag: String,
    pub(crate) description: String,
}

impl WorldTagRecord {
    /** Mutate `self` into `other`, but retain non-default `description` fields. */
    pub(crate) fn safe_mutate(&mut self, other: &Self) {
        let Self {
            code: new_code,
            tag: new_tag,
            description: new_desc,
        } = other;

        if self.description == TABLES.world_tag_table[self.code as usize].description {
            self.description = new_desc.clone();
        }

        self.code = *new_code;
        self.tag = new_tag.clone();
    }
}

impl Record for WorldTagRecord {
    fn code(&self) -> u16 {
        self.code
    }
}
type WorldTagTable = Vec<WorldTagRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct LawRecord {
    pub(crate) code: u16,
    pub(crate) banned_weapons: String,
    pub(crate) banned_armor: String,
}

impl Record for LawRecord {
    fn code(&self) -> u16 {
        self.code
    }
}
type LawTable = Vec<LawRecord>;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) enum StarportClass {
    A,
    B,
    C,
    D,
    E,
    X,
}

impl fmt::Display for StarportClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::X => "X",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub(crate) struct StarportRecord {
    pub(crate) code: u16,
    pub(crate) class: StarportClass,
    pub(crate) berthing_cost: u32,
    pub(crate) fuel: String,
    pub(crate) facilities: String,
}

impl PartialEq for StarportRecord {
    fn eq(&self, other: &Self) -> bool {
        self.class == other.class
            && self.berthing_cost == other.berthing_cost
            && self.fuel == other.fuel
            && self.facilities == other.facilities
    }
}

impl Record for StarportRecord {
    fn code(&self) -> u16 {
        self.code
    }
}
type StarportTable = Vec<StarportRecord>;

pub(crate) trait Table<T> {
    /** Get a reference to an item within the `Table` using a uniform distribution. */
    fn roll_uniform(&self) -> &T;

    /** Get a reference to an item with the `Table` using a "2d6" normal distribution. */
    fn roll_normal_2d6(&self, modifier: i32) -> &T;
}

impl<T, U> Table<T> for U
where
    U: Deref<Target = [T]>,
{
    /** Get a reference to an item within the `Table` using a uniform distribution.

    # Panics
    Panics if the `Table` is empty.
    */
    fn roll_uniform(&self) -> &T {
        assert!(!self.is_empty(), "Cannot roll on an empty table");
        let range = 0..self.len();
        let index = dice::roll_range(range);
        &self[index]
    }

    /** Get a reference to an item with the `Table` using a "2d6" normal distribution.

    The value of `modifier` is added to the result of the 2d6 roll; however any modified rolls are
    clamped to be in-bounds for the `Table`.
    Because of this, double-peaks in the outcome of these rolls will tend to appear at the top or
    bottom of the table's domain when `modifier` is significantly greater than or less than zero,
    respectively.

    # Panics
    Panics if the `Table` is empty.
    */
    fn roll_normal_2d6(&self, modifier: i32) -> &T {
        assert!(!self.is_empty(), "Cannot roll on an empty table");
        let roll = dice::roll_2d(6);
        let modified_roll = roll + modifier;

        let low = 0;
        let high = (self.len() - 1) as i32;
        let index = (modified_roll).clamp(low, high) as usize;
        &self[index]
    }
}

fn load_table<T: for<'de> Deserialize<'de> + Record>(table_csv: &str) -> Vec<T> {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_reader(table_csv.as_bytes());
    for (index, result) in reader.deserialize().enumerate() {
        let record: T = result.unwrap();
        assert_eq!(
            record.code(),
            index as u16,
            "The code field in each row must match its zero-indexed position in the table"
        );
        table.push(record);
    }
    table
}

#[allow(dead_code)]
fn test_table(file_path: &str) {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(file_path)
        .unwrap();

    for (index, result) in reader.records().enumerate() {
        let record = result.unwrap();
        println!("record[{}]: {:?}", index, record);
    }
}

#[derive(Debug)]
pub(crate) struct SubsectorTableCollection {
    pub(crate) atmo_table: AtmoTable,
    pub(crate) temp_table: TempTable,
    pub(crate) hydro_table: HydroTable,
    pub(crate) pop_table: PopTable,
    pub(crate) gov_table: GovTable,
    pub(crate) faction_table: FactionTable,
    pub(crate) culture_table: CulturalDiffTable,
    pub(crate) world_tag_table: WorldTagTable,
    pub(crate) law_table: LawTable,
    pub(crate) starport_table: StarportTable,
}

impl SubsectorTableCollection {
    fn new() -> SubsectorTableCollection {
        SubsectorTableCollection {
            atmo_table: load_table(ATMO_TABLE_CSV),
            temp_table: load_table(TEMP_TABLE_CSV),
            hydro_table: load_table(HYDRO_TABLE_CSV),
            pop_table: load_table(POP_TABLE_CSV),
            gov_table: load_table(GOV_TABLE_CSV),
            faction_table: load_table(FACTION_TABLE_CSV),
            culture_table: load_table(CULTURE_TABLE_CSV),
            world_tag_table: load_table(WORLD_TAG_TABLE_CSV),
            law_table: load_table(LAW_TABLE_CSV),
            starport_table: load_table(STARPORT_TABLE_CSV),
        }
    }
}

lazy_static! {
    pub(crate) static ref TABLES: SubsectorTableCollection = SubsectorTableCollection::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_all_tables() {
        // No easy way to check the contents, just make sure they all load without panic
        SubsectorTableCollection::new();
    }
}
