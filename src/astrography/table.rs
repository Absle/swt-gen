use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::dice;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AtmoRecord {
    pub code: u16,
    pub composition: String,
}
pub type AtmoTable = Vec<AtmoRecord>;

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub struct TempRecord {
    pub code: u16,
    pub kind: String,
    pub description: String,
}

impl PartialEq for TempRecord {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.description == other.description
    }
}

pub type TempTable = Vec<TempRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HydroRecord {
    pub code: u16,
    pub description: String,
}
pub type HydroTable = Vec<HydroRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PopRecord {
    pub code: u16,
    pub inhabitants: String,
}
pub type PopTable = Vec<PopRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GovRecord {
    pub code: u16,
    pub kind: String,
    pub description: String,
    pub contraband: String,
}
pub type GovTable = Vec<GovRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactionRecord {
    pub code: u16,
    pub strength: String,
}
pub type FactionTable = Vec<FactionRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CulturalDiffRecord {
    pub code: u16,
    pub cultural_difference: String,
    pub description: String,
}
pub type CulturalDiffTable = Vec<CulturalDiffRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorldTagRecord {
    pub code: u16,
    pub tag: String,
    pub description: String,
}

impl WorldTagRecord {
    pub(crate) fn random() -> Self {
        let range = 0..TABLES.world_tag_table.len();
        let roll = dice::roll(range);
        TABLES.world_tag_table[roll].clone()
    }
}

pub type WorldTagTable = Vec<WorldTagRecord>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LawRecord {
    pub code: u16,
    pub banned_weapons: String,
    pub banned_armor: String,
}
pub type LawTable = Vec<LawRecord>;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum StarportClass {
    A,
    B,
    C,
    D,
    E,
    X,
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub struct StarportRecord {
    pub code: u16,
    pub class: StarportClass,
    pub berthing_cost: u32,
    pub fuel: String,
    pub facilities: String,
}

impl PartialEq for StarportRecord {
    fn eq(&self, other: &Self) -> bool {
        self.class == other.class
            && self.berthing_cost == other.berthing_cost
            && self.fuel == other.fuel
            && self.facilities == other.facilities
    }
}

pub type StarportTable = Vec<StarportRecord>;

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
pub fn test_table(file_path: &str) {
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
pub struct SubsectorTableCollection {
    pub atmo_table: AtmoTable,
    pub temp_table: TempTable,
    pub hydro_table: HydroTable,
    pub pop_table: PopTable,
    pub gov_table: GovTable,
    pub faction_table: FactionTable,
    pub culture_table: CulturalDiffTable,
    pub world_tag_table: WorldTagTable,
    pub law_table: LawTable,
    pub starport_table: StarportTable,
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
    pub static ref TABLES: SubsectorTableCollection = SubsectorTableCollection::new();
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
