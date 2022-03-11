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

#[derive(Clone, Debug, Deserialize)]
pub struct TempRecord {
    pub code: isize,
    pub kind: String,
    pub description: String,
}
pub type TempTable = Vec<TempRecord>;

#[derive(Clone, Debug, Deserialize)]
pub struct HydroRecord {
    pub code: isize,
    pub description: String,
}
pub type HydroTable = Vec<HydroRecord>;

#[derive(Clone, Debug, Deserialize)]
pub struct PopRecord {
    pub code: isize,
    pub inhabitants: String,
}
pub type PopTable = Vec<PopRecord>;

#[derive(Clone, Debug, Deserialize)]
pub struct GovRecord {
    pub code: isize,
    pub kind: String,
    pub description: String,
    pub contraband: String,
}
pub type GovTable = Vec<GovRecord>;

#[derive(Clone, Debug, Deserialize)]
pub struct FactionRecord {
    pub code: isize,
    pub strength: String,
}
pub type FactionTable = Vec<FactionRecord>;

#[derive(Clone, Debug, Deserialize)]
pub struct CulturalDiffRecord {
    pub code: isize,
    pub cultural_difference: String,
    pub description: String,
}
pub type CulturalDiffTable = Vec<CulturalDiffRecord>;

#[derive(Clone, Debug, Deserialize)]
pub struct WorldTagRecord {
    pub code: isize,
    pub tag: String,
    pub description: String,
}
pub type WorldTagTable = Vec<WorldTagRecord>;

#[derive(Clone, Debug, Deserialize)]
pub struct LawRecord {
    pub code: isize,
    pub banned_weapons: String,
    pub banned_armor: String,
}
pub type LawTable = Vec<LawRecord>;

#[derive(Clone, Debug, Deserialize)]
pub struct StarportRecord {
    pub code: isize,
    pub class: String,
    pub berthing_cost: String,
    pub fuel: String,
    pub facilities: String,
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
