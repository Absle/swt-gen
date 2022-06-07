pub(crate) mod table;
pub(crate) mod world;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::ops::{Add, Sub};

use rand::Rng;
use roxmltree as xml;
use serde::{Deserialize, Serialize};

use super::dice;
use world::{World, WorldRecord};

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Point {
    pub x: u16,
    pub y: u16,
}

impl Default for Point {
    fn default() -> Self {
        Point { x: 0, y: 0 }
    }
}

impl ToString for Point {
    fn to_string(&self) -> String {
        format!("{:02}{:02}", self.x, self.y)
    }
}

impl TryFrom<&str> for Point {
    type Error = Box<dyn Error>;
    fn try_from(string: &str) -> Result<Self, Self::Error> {
        let string = string.trim();

        // Handle old and new prefix respectively
        let string = string.strip_prefix("'").unwrap_or(&string);
        let string = string.strip_prefix("_").unwrap_or(&string);
        let string = string.trim();

        let mut chars = string.chars();

        let mut x_str = String::new();
        let mut y_str = String::new();
        for string in [&mut x_str, &mut y_str] {
            for _ in 0..2 {
                let c = chars.next().ok_or("World location string too short")?;
                string.push(c);
            }
        }

        // After removing prefixes and process four characters, there should be nothing left
        if !chars.next().is_none() {
            return Err("World location string too long".into());
        }

        let x: u16 = x_str.parse()?;
        let y: u16 = y_str.parse()?;
        Ok(Self { x, y })
    }
}

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

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Subsector {
    pub(crate) name: String,
    pub(crate) map: BTreeMap<Point, World>,
}

#[allow(dead_code)]
const CSV_HEADERS: &str = "Subsector,Name,Location,Profile,Bases,Trade Codes,Travel Code,Gas Giant,Berthing Cost,,,,Government,Contraband,Culture,World Tag 1,World Tag 2,,,,Faction 1,Strength 1,Government 1,Faction 2,Strength 2,Government 2,Faction 3,Strength 3,Government 3,Faction 4,Strength 4,Government 4,,,,Diameter (km),Atmosphere,Temperature,Hydrographics,Population,Notes";

impl Subsector {
    pub const COLUMNS: usize = 8;
    pub const ROWS: usize = 10;

    pub(crate) fn empty() -> Self {
        Subsector {
            name: String::from("Subsector"),
            map: BTreeMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name[..]
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
        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(Vec::new());

        for (_, world) in &self.map {
            let mut record = WorldRecord::from(world.clone());
            record.subsector_name = self.name.clone();
            writer.serialize(record).unwrap();
        }

        let table = String::from_utf8(writer.into_inner().unwrap()).unwrap();

        [String::from(CSV_HEADERS), table].join("\n")
    }

    #[allow(dead_code)]
    pub fn from_csv(csv: &str) -> Result<Self, Box<dyn Error>> {
        let mut rows = csv.lines();

        match rows.next().ok_or("Ran out of rows while parsing header")? {
            CSV_HEADERS => (),
            _ => return Err("Could not find column headers".into()),
        }

        let world_table = rows.collect::<Vec<_>>().join("\n");
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(world_table.as_bytes());

        let mut name = String::new();
        let mut map = BTreeMap::new();
        for result in reader.deserialize() {
            let world_record: WorldRecord = result?;

            if name.is_empty() {
                name = world_record.subsector_name.clone();
            }

            let world_name = String::from(world_record.name());
            let maybe_world = World::try_from(world_record);
            if let Some(err) = maybe_world.as_ref().err() {
                return Err(format!("Error while parsing world '{}': {}", world_name, err).into());
            }
            let world = maybe_world.unwrap();
            map.insert(world.location.clone(), world);
        }

        Ok(Self { name, map })
    }

    pub fn generate_svg(&self) -> String {
        let template_svg = fs::read_to_string("resources/traveller_subsector_grid.svg").unwrap();
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
                style=\"font-style:normal;font-variant:normal;font-weight:normal;font-stretch:condensed;font-size:3.52777px;line-height:0;font-family:sans-serif;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
                x=\"{translate_x}\" \
                y=\"{translate_y}\" \
                id=\"{point_str}NameText\">\
                <tspan \
                sodipodi:role=\"line\" \
                id=\"{point_str}NameTspan\" \
                style=\"font-style:normal;font-variant:normal;font-weight:normal;font-stretch:condensed;font-family:sans-serif;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
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
                style=\"font-style:italic;font-variant:normal;font-weight:normal;font-stretch:condensed;font-size:3.52777px;line-height:0;font-family:sans-serif;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
                x=\"{translate_x}\" \
                y=\"{translate_y}\" \
                id=\"{point_str}StarportTlText\">\
                <tspan \
                sodipodi:role=\"line\" \
                id=\"{point_str}StarportTlTspan\" \
                style=\"font-style:italic;font-variant:normal;font-weight:normal;font-stretch:condensed;font-family:sans-serif;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
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
                style=\"font-style:italic;font-variant:normal;font-weight:normal;font-stretch:condensed;font-size:2.8px;line-height:0;font-family:sans-serif;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
                x=\"{translate_x}\" \
                y=\"{translate_y}\" \
                id=\"{point_str}WorldProfileText\">\
                <tspan \
                sodipodi:role=\"line\" \
                id=\"{point_str}WorldProfileTspan\" \
                style=\"font-style:italic;font-variant:normal;font-weight:normal;font-stretch:condensed;font-family:sans-serif;-inkscape-font-specification:'Arial Italic Condensed';text-align:center;text-anchor:middle;stroke-width:0.264583\" \
                x=\"{translate_x}\" \
                y=\"{translate_y}\">{profile}</tspan></text>",
                translate_x = translation.x,
                translate_y = translation.y,
                point_str = format!("{:02}{:02}", point.x, point.y),
                profile = world.profile()
            ));
        }

        // Closing layer and svg document
        output_buffer.push(String::from("</g>"));
        output_buffer.push(close_svg);

        // Place name of subsector as title
        for i in 0..output_buffer.len() {
            if output_buffer[i].contains("Subsector Name") {
                output_buffer[i] =
                    output_buffer[i].replace("Subsector Name", &format!("{} Subsector", self.name));
                // As a sanity check, make sure we only do this once
                break;
            }
        }

        output_buffer.join("\n")
    }
}

impl Default for Subsector {
    fn default() -> Self {
        Subsector::new(0)
    }
}

fn random_names(count: usize) -> Vec<String> {
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
        const ATTEMPTS: usize = 100;
        for _ in 0..ATTEMPTS {
            Subsector::new(0);
        }
    }

    #[test]
    fn subsector_yaml_serde() {
        let subsector = Subsector::new(0);
        let yaml = serde_yaml::to_string(&subsector).unwrap();

        let de_subsector: Subsector = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(de_subsector, subsector);
    }

    #[test]
    fn subsector_csv_serde() {
        const ATTEMPTS: usize = 100;
        for _ in 0..ATTEMPTS {
            let subsector = Subsector::new(0);
            let csv = subsector.to_csv();
            let deserialized = Subsector::from_csv(&csv[..]);
            assert_eq!(deserialized.unwrap(), subsector);
        }
    }

    #[test]
    fn subsector_svg() {
        const ATTEMPTS: usize = 10;
        for _ in 0..ATTEMPTS {
            let subsector = Subsector::new(0);
            let _svg = subsector.generate_svg();
        }
    }
}
