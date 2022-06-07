use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::dice;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AtmoRecord {
    pub(crate) code: u16,
    pub(crate) composition: String,
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

type TempTable = Vec<TempRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct HydroRecord {
    pub(crate) code: u16,
    pub(crate) description: String,
}
type HydroTable = Vec<HydroRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct PopRecord {
    pub(crate) code: u16,
    pub(crate) inhabitants: String,
}
type PopTable = Vec<PopRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct GovRecord {
    pub(crate) code: u16,
    pub(crate) kind: String,
    pub(crate) description: String,
    pub(crate) contraband: String,
}
type GovTable = Vec<GovRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct FactionRecord {
    pub(crate) code: u16,
    pub(crate) strength: String,
}
type FactionTable = Vec<FactionRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct CulturalDiffRecord {
    pub(crate) code: u16,
    pub(crate) cultural_difference: String,
    pub(crate) description: String,
}
type CulturalDiffTable = Vec<CulturalDiffRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WorldTagRecord {
    pub(crate) code: u16,
    pub(crate) tag: String,
    pub(crate) description: String,
}

impl WorldTagRecord {
    pub(crate) fn random() -> Self {
        let range = 0..TABLES.world_tag_table.len();
        let roll = dice::roll(range);
        TABLES.world_tag_table[roll].clone()
    }
}

type WorldTagTable = Vec<WorldTagRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct LawRecord {
    pub(crate) code: u16,
    pub(crate) banned_weapons: String,
    pub(crate) banned_armor: String,
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

impl ToString for StarportClass {
    fn to_string(&self) -> String {
        format!("{:?}", self)
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

type StarportTable = Vec<StarportRecord>;

fn load_table<T: for<'de> Deserialize<'de>>(file_path: &str) -> Vec<T> {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for result in reader.deserialize() {
        let record: T = result.unwrap();
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
            atmo_table: load_table("resources/tables/atmospheres.csv"),
            temp_table: load_table("resources/tables/temperatures.csv"),
            hydro_table: load_table("resources/tables/hydrographics.csv"),
            pop_table: load_table("resources/tables/populations.csv"),
            gov_table: load_table("resources/tables/governments.csv"),
            faction_table: load_table("resources/tables/factions.csv"),
            culture_table: load_table("resources/tables/cultural_differences.csv"),
            world_tag_table: load_table("resources/tables/world_tags.csv"),
            law_table: load_table("resources/tables/law_levels.csv"),
            starport_table: load_table("resources/tables/starports.csv"),
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
