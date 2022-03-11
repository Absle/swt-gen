use lazy_static::lazy_static;
use serde::Deserialize;

#[allow(dead_code)]
pub fn test_table(file_path: &str) {
    //let mut reader = csv::Reader::from_path(file_path).unwrap();
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(file_path)
        .unwrap();
    
    for (index, result) in reader.records().enumerate() {
        let record = result.unwrap();
        println!("record[{}]: {:?}", index, record);
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AtmoRecord {
    pub code: isize,
    pub composition: String,
}

pub type AtmoTable = Vec<AtmoRecord>;
fn load_atmo_table(file_path: &str) -> AtmoTable {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for (index, result) in reader.deserialize().enumerate() {
        let record: AtmoRecord = result.unwrap();
        assert_eq!(record.code, index as isize);
        table.push(record);
    }
    table
}

#[derive(Clone, Debug, Deserialize)]
pub struct TempRecord {
    pub code: isize,
    pub kind: String,
    pub description: String,
}

pub type TempTable = Vec<TempRecord>;
fn load_temp_table(file_path: &str) -> TempTable {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for (index, result) in reader.deserialize().enumerate() {
        let record: TempRecord = result.unwrap();
        assert_eq!(record.code, index as isize);
        table.push(record);
    }
    table
}

#[derive(Clone, Debug, Deserialize)]
pub struct HydroRecord {
    pub code: isize,
    pub description: String,
}

pub type HydroTable = Vec<HydroRecord>;
fn load_hydro_table(file_path: &str) -> HydroTable {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for (index, result) in reader.deserialize().enumerate() {
        let record: HydroRecord = result.unwrap();
        assert_eq!(record.code, index as isize);
        table.push(record);
    }
    table
}

#[derive(Clone, Debug, Deserialize)]
pub struct PopRecord {
    pub code: isize,
    pub inhabitants: String,
}

pub type PopTable = Vec<PopRecord>;
fn load_pop_table(file_path: &str) -> PopTable {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for (index, result) in reader.deserialize().enumerate() {
        let record: PopRecord = result.unwrap();
        assert_eq!(record.code, index as isize);
        table.push(record);
    }
    table
}

#[derive(Clone, Debug, Deserialize)]
pub struct GovRecord {
    pub code: isize,
    pub kind: String,
    pub description: String,
    pub contraband: String,
}

pub type GovTable = Vec<GovRecord>;
fn load_gov_table(file_path: &str) -> GovTable {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for (index, result) in reader.deserialize().enumerate() {
        let record: GovRecord = result.unwrap();
        assert_eq!(record.code, index as isize);
        table.push(record);
    }
    table
}

#[derive(Clone, Debug, Deserialize)]
pub struct FactionRecord {
    pub code: isize,
    pub strength: String,
}

pub type FactionTable = Vec<FactionRecord>;
fn load_faction_table(file_path: &str) -> FactionTable {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for (index, result) in reader.deserialize().enumerate() {
        let record: FactionRecord = result.unwrap();
        assert_eq!(record.code, index as isize);
        table.push(record);
    }
    table
}

#[derive(Clone, Debug, Deserialize)]
pub struct CulturalDiffRecord {
    pub code: isize,
    pub cultural_difference: String,
    pub description: String,
}

pub type CulturalDiffTable = Vec<CulturalDiffRecord>;
fn load_cultural_table(file_path: &str) -> CulturalDiffTable {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for (index, result) in reader.deserialize().enumerate() {
        let record: CulturalDiffRecord = result.unwrap();
        assert_eq!(record.code, index as isize);
        table.push(record);
    }
    table
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorldTagRecord {
    pub code: isize,
    pub tag: String,
    pub description: String,
}

pub type WorldTagTable = Vec<WorldTagRecord>;
fn load_world_tag_table(file_path: &str) -> WorldTagTable {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for (index, result) in reader.deserialize().enumerate() {
        let record: WorldTagRecord = result.unwrap();
        assert_eq!(record.code, index as isize);
        table.push(record);
    }
    table
}

#[derive(Clone, Debug, Deserialize)]
pub struct LawRecord {
    pub code: isize,
    pub banned_weapons: String,
    pub banned_armor: String,
}

pub type LawTable = Vec<LawRecord>;
fn load_law_table(file_path: &str) -> LawTable {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for (index, result) in reader.deserialize().enumerate() {
        let record: LawRecord = result.unwrap();
        assert_eq!(record.code, index as isize);
        table.push(record);
    }
    table
}

#[derive(Clone, Debug, Deserialize)]
pub struct StarportRecord {
    pub code: isize,
    pub class: String,
    pub berthing_cost: String,
    pub fuel: String,
    pub facilities: String,
}

pub type StarportTable = Vec<StarportRecord>;
fn load_starport_table(file_path: &str) -> StarportTable {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for (index, result) in reader.deserialize().enumerate() {
        let record: StarportRecord = result.unwrap();
        assert_eq!(record.code, index as isize);
        table.push(record);
    }
    table
}

fn load_table<T>(file_path: &str) -> Vec<T>
    where T: for<'de> Deserialize<'de> {
    let mut table = Vec::new();
    let mut reader = csv::Reader::from_path(file_path).unwrap();
    for result in reader.deserialize() {
        let record: T = result.unwrap();
        table.push(record);
    }
    table
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
            atmo_table: load_atmo_table("resources/tables/atmospheres.csv"),
            temp_table: load_temp_table("resources/tables/temperatures.csv"),
            hydro_table: load_hydro_table("resources/tables/hydrographics.csv"),
            pop_table: load_pop_table("resources/tables/populations.csv"),
            gov_table: load_gov_table("resources/tables/governments.csv"),
            faction_table: load_faction_table("resources/tables/factions.csv"),
            culture_table: load_cultural_table("resources/tables/cultural_differences.csv"),
            world_tag_table: load_world_tag_table("resources/tables/world_tags.csv"),
            law_table: load_law_table("resources/tables/law_levels.csv"),
            starport_table: load_starport_table("resources/tables/starports.csv"),
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
