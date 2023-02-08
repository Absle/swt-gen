mod serialize;
mod table;
mod world;

pub(crate) use table::*;
pub(crate) use world::{Faction, TravelCode, World};

use std::{
    collections::BTreeMap,
    convert::TryFrom,
    error::Error,
    fmt, fs, io,
    ops::{Add, Sub},
    str,
};

use lazy_static::lazy_static;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::dice;

use serialize::{JsonableSubsector, T5Table};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Point {
    pub x: i32,
    pub y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}{:02}", self.x, self.y)
    }
}

impl TryFrom<&str> for Point {
    type Error = Box<dyn Error>;
    fn try_from(string: &str) -> Result<Self, Self::Error> {
        let string = string.trim();

        // Handle old and new prefix respectively
        let string = string.strip_prefix('\'').unwrap_or(string);
        let string = string.strip_prefix('_').unwrap_or(string);
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
        if chars.next().is_some() {
            return Err("World location string too long".into());
        }

        let x: i32 = x_str.parse()?;
        let y: i32 = y_str.parse()?;
        Ok(Self { x, y })
    }
}

#[derive(Debug)]
enum PolityColor {
    Turqoise,
    Yellow,
    Periwinkle,
    Red,
    Blue,
    Orange,
    Pear,
    Lavender,
    Grey,
    Violet,
    Pistachio,
    Gold,
}

impl PolityColor {
    const ALL_VALUES: [PolityColor; 12] = [
        Self::Turqoise,
        Self::Yellow,
        Self::Periwinkle,
        Self::Red,
        Self::Blue,
        Self::Orange,
        Self::Pear,
        Self::Lavender,
        Self::Grey,
        Self::Violet,
        Self::Pistachio,
        Self::Gold,
    ];

    fn class(&self) -> String {
        let lower = self.to_string().to_lowercase();
        format!("hex-color-{lower}")
    }
}

impl fmt::Display for PolityColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Turqoise => "Turqoise",
            Self::Yellow => "Yellow",
            Self::Periwinkle => "Periwinkle",
            Self::Red => "Red",
            Self::Blue => "Blue",
            Self::Orange => "Orange",
            Self::Pear => "Pear",
            Self::Lavender => "Lavender",
            Self::Grey => "Grey",
            Self::Violet => "Violet",
            Self::Pistachio => "Pistachio",
            Self::Gold => "Gold",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Translation {
    x: f64,
    y: f64,
}

impl Translation {
    fn try_from_transform_str(transform: &str) -> Result<Self, Box<dyn Error>> {
        let translate_args: Vec<&str> = transform
            .strip_prefix("translate(")
            .ok_or(format!("Incorrect prefix in transform '{transform}'"))?
            .strip_suffix(')')
            .ok_or(format!("Unclosed tranform '{transform}'"))?
            .split(',')
            .collect();

        let x: f64 = translate_args
            .first()
            .ok_or(format!("Could not find x value in '{transform}'"))?
            .parse()?;

        let y: f64 = translate_args.get(1).unwrap_or(&"0.0").parse()?;

        Ok(Translation { x, y })
    }
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

impl Default for Translation {
    fn default() -> Self {
        Translation { x: 0.0, y: 0.0 }
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum WorldAbundance {
    Rift,
    Sparse,
    Nominal,
    Dense,
    Abundant,
}

impl WorldAbundance {
    pub const WORLD_ABUNDANCE_VALUES: [WorldAbundance; 5] = [
        Self::Rift,
        Self::Sparse,
        Self::Nominal,
        Self::Dense,
        Self::Abundant,
    ];
}

impl From<WorldAbundance> for i16 {
    fn from(world_abundance: WorldAbundance) -> Self {
        match world_abundance {
            WorldAbundance::Rift => -2,
            WorldAbundance::Sparse => -1,
            WorldAbundance::Nominal => 0,
            WorldAbundance::Dense => 1,
            WorldAbundance::Abundant => 2,
        }
    }
}

impl fmt::Display for WorldAbundance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Rift => "Rift",
            Self::Sparse => "Sparse",
            Self::Nominal => "Nominal",
            Self::Dense => "Dense",
            Self::Abundant => "Abundant",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Subsector {
    name: String,
    map: BTreeMap<Point, World>,
}

const TEMPLATE_SVG: &str = include_str!("../resources/subsector_grid_template.svg");

lazy_static! {
    static ref CENTER_MARKERS: BTreeMap<Point, Translation> = center_markers();
    static ref GAS_GIANT_TRANS: Translation = map_legend_translation("GasGiantCircle");
    static ref DRY_WORLD_TRANS: Translation = map_legend_translation("DryWorldSymbol");
    static ref WET_WORLD_TRANS: Translation = map_legend_translation("WetWorldSymbol");
}

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

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn new(world_abundance_dm: i16) -> Self {
        let mut subsector = Self::empty();
        let mut names = random_names(Subsector::COLUMNS * Subsector::ROWS + 1).into_iter();
        subsector.name = names.next().unwrap();

        for x in 1..=Subsector::COLUMNS {
            for y in 1..=Subsector::ROWS {
                // Fifty-fifty chance with no modifiers
                let roll = dice::roll_1d(6) + world_abundance_dm;
                if roll >= 4 {
                    let point = Point {
                        x: x as i32,
                        y: y as i32,
                    };

                    let name = names.next().unwrap();
                    let world = World::new(name);
                    subsector
                        .insert_world(&point, world)
                        .expect("All new subsector world's should be valid");
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
                    x: x as i32,
                    y: y as i32,
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

    pub fn to_json(&self) -> String {
        JsonableSubsector::from(self).to_string()
    }

    pub fn try_from_json(json: &str) -> Result<Self, Box<dyn Error>> {
        let jsonable: JsonableSubsector = serde_json::from_str(json)?;
        let subsector = Self::try_from(jsonable)?;
        Ok(subsector)
    }

    pub fn to_sec_table(&self) -> String {
        T5Table::from(self).to_string()
    }

    pub fn generate_svg(&self, colored: bool) -> String {
        let mut reader = quick_xml::Reader::from_str(TEMPLATE_SVG);
        // TODO: indented SVG writing would be better but for some reason it causes the UWP and hex
        // strings to be misaligned
        // let mut writer = quick_xml::Writer::new_with_indent(io::Cursor::new(Vec::new()), b' ', 2);
        let mut writer = quick_xml::Writer::new(io::Cursor::new(Vec::new()));
        loop {
            match reader.read_event() {
                Err(e) => unreachable!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Eof) => break,
                Ok(Event::Comment(_)) => (),

                Ok(Event::Start(element)) => {
                    writer.write_event(Event::Start(element)).unwrap();
                }

                Ok(Event::End(element)) => {
                    if element.name().as_ref() == b"svg" {
                        let mut layer = BytesStart::new("g");
                        layer.extend_attributes(vec![
                            ("inkscape:groupmode", "layer"),
                            ("id", "layer6"),
                            ("inkscape:label", "Generated"),
                        ]);
                        writer.write_event(Event::Start(layer)).unwrap();

                        for (point, world) in &self.map {
                            let point_str = point.to_string();
                            let marker_translation = CENTER_MARKERS
                                .get(point)
                                .expect("Found a point with no center marker");

                            // Place gas giant symbol
                            if world.has_gas_giant {
                                let offset = Translation { x: 0.0, y: -6.0 };
                                let trans = &(marker_translation - &GAS_GIANT_TRANS) + &offset;

                                writer
                                    .create_element("use")
                                    .with_attributes(vec![
                                        ("href", "#GasGiantSymbol"),
                                        (
                                            "id",
                                            &format!("{:02}{:02}GasGiantSymbol", point.x, point.y),
                                        ),
                                        (
                                            "transform",
                                            &format!("translate({},{})", trans.x, trans.y),
                                        ),
                                    ])
                                    .write_empty()
                                    .unwrap();
                            }

                            // Place world name
                            writer
                                .create_element("text")
                                .with_attributes(vec![
                                    ("xml:space", "preserve"),
                                    ("class", "text-world-name"),
                                    ("x", &marker_translation.x.to_string()),
                                    ("y", &marker_translation.y.to_string()),
                                    ("id", &format!("{}NameText", point_str)),
                                ])
                                .write_inner_content(|writer| {
                                    writer
                                        .create_element("tspan")
                                        .with_attributes(vec![
                                            ("sodipodi:role", "line"),
                                            ("id", &format!("{}NameTspan", point_str)),
                                        ])
                                        .write_text_content(BytesText::new(&world.name))
                                        .unwrap();
                                    Ok(())
                                })
                                .unwrap();

                            // Place dry/world symbol
                            let (symbol_id, world_trans) = match world.hydrographics.code {
                                h if h <= 3 => ("DryWorldSymbol", *DRY_WORLD_TRANS),
                                _ => ("WetWorldSymbol", *WET_WORLD_TRANS),
                            };
                            let offset = Translation { x: -5.0, y: 4.0 };
                            let trans = &(marker_translation - &world_trans) + &offset;
                            writer
                                .create_element("use")
                                .with_attributes(vec![
                                    ("href", &format!("#{}", symbol_id)[..]),
                                    ("id", &format!("{}{}", point_str, symbol_id)),
                                    ("transform", &format!("translate({},{})", trans.x, trans.y)),
                                ])
                                .write_empty()
                                .unwrap();

                            // Add `StarportClass-TL` text to hex
                            let offset = Translation { x: 5.0, y: 5.0 };
                            let trans = marker_translation + &offset;
                            writer
                                .create_element("text")
                                .with_attributes(vec![
                                    ("xml:space", "preserve"),
                                    ("class", "text-starport-tl"),
                                    ("x", &trans.x.to_string()),
                                    ("y", &trans.y.to_string()),
                                    ("id", &format!("{}StarportTlText", point_str)),
                                ])
                                .write_inner_content(|writer| {
                                    let starport_tl =
                                        format!("{:?}-{}", world.starport.class, world.tech_level);
                                    writer
                                        .create_element("tspan")
                                        .with_attributes(vec![
                                            ("sodipodi:role", "line"),
                                            ("id", &format!("{}StarportTlTspan", point_str)),
                                        ])
                                        .write_text_content(BytesText::new(&starport_tl))
                                        .unwrap();
                                    Ok(())
                                })
                                .unwrap();

                            // Place world profile code
                            let offset = Translation { x: 0.0, y: 10.0 };
                            let trans = marker_translation + &offset;
                            writer
                                .create_element("text")
                                .with_attributes(vec![
                                    ("xml:space", "preserve"),
                                    ("class", "text-world-profile"),
                                    ("x", &format!("{}", trans.x)),
                                    ("y", &format!("{}", trans.y)),
                                    ("id", &format!("{}WorldProfileText", point_str)),
                                ])
                                .write_inner_content(|writer| {
                                    writer
                                        .create_element("tspan")
                                        .with_attributes(vec![
                                            ("sodipodi:role", "line"),
                                            ("id", &format!("{}WorldProfileTspan", point_str)),
                                            ("x", &format!("{}", trans.x)),
                                            ("y", &format!("{}", trans.y)),
                                        ])
                                        .write_text_content(BytesText::new(&world.profile_str()))
                                        .unwrap();
                                    Ok(())
                                })
                                .unwrap();
                        }
                        // End of layer
                        writer.write_event(Event::End(BytesEnd::new("g"))).unwrap();
                    }
                    // Close svg section
                    writer.write_event(Event::End(element)).unwrap();
                }

                Ok(Event::Empty(element)) => {
                    if !colored {
                        writer.write_event(Event::Empty(element)).unwrap();
                        continue;
                    }

                    let element = if let Ok(Some(id_attr)) = element.try_get_attribute("id") {
                        let id = str::from_utf8(&id_attr.value).unwrap();
                        if let Some(point_str) = id.strip_prefix("HexPath-") {
                            let point =
                                Point::try_from(point_str).expect("Failed to parse HexPath point");
                            let x = point.x as usize;
                            let y = point.y as usize;
                            let point_index =
                                ((x - 1) * Subsector::ROWS + y - 1) % PolityColor::ALL_VALUES.len();
                            let class = PolityColor::ALL_VALUES[point_index].class();

                            let mut hex = BytesStart::new("path");
                            hex.extend_attributes(element.attributes().map(|attr| {
                                let attr = attr.unwrap();
                                if attr.key.as_ref() == b"class" {
                                    ("class", &class[..]).into()
                                } else {
                                    attr
                                }
                            }));

                            hex
                        } else {
                            element
                        }
                    } else {
                        element
                    };
                    writer.write_event(Event::Empty(element)).unwrap();
                }

                Ok(Event::Text(text)) => {
                    let t: &[u8] = text.as_ref();
                    if t == b"Subsector Name" {
                        let map_title = format!("{} Subsector", self.name());
                        let subsector_name = BytesText::new(&map_title);
                        writer.write_event(Event::Text(subsector_name)).unwrap();
                    } else {
                        writer.write_event(Event::Text(text)).unwrap();
                    }
                }

                Ok(Event::Decl(element)) => {
                    writer.write_event(Event::Decl(element)).unwrap();
                }

                _ => panic!("Unexpected element in template svg"),
            }
        }

        str::from_utf8(&writer.into_inner().into_inner())
            .expect("Invalid UTF-8 while generating svg")
            .to_string()
    }

    #[cfg(test)]
    pub fn get_map(&mut self) -> &BTreeMap<Point, World> {
        &self.map
    }

    /** Returns a reference to the `World` at `point` or `None` if there isn't one. */
    pub fn get_world(&self, point: &Point) -> Option<&World> {
        self.map.get(point)
    }

    pub(crate) fn point_is_inbounds(point: &Point) -> bool {
        point.x > 0
            && point.x as usize <= Self::COLUMNS
            && point.y > 0
            && point.y as usize <= Self::ROWS
    }

    /** Inserts `world` at `point`, replacing any other [`World`] that was there previously.

    # Returns
    - `Ok(Some(world))` with the `World` that was already at `point` if there was one,
    - `Ok(None)` if the was inserted into an empty location,
    - `Err(msg)` if `point` was out of bounds and the insertion failed
    */
    pub fn insert_world(&mut self, point: &Point, world: World) -> Result<Option<World>, String> {
        if Self::point_is_inbounds(point) {
            Ok(self.map.insert(*point, world))
        } else {
            Err("Can not insert a world at an out of bounds point".to_string())
        }
    }

    /** Inserts a random [`World`] at `point`, replacing any [`World`] there.

    # Returns
    - `Ok(Some(World))` containing the displaced world if there was one,
    - `Ok(None)` if the world was inserted into an empty location,
    - `Err(msg)` if `point` was out of bounds and the insertion failed
    */
    pub fn insert_random_world(&mut self, point: &Point) -> Result<Option<World>, String> {
        let mut names = random_names(Subsector::COLUMNS * Subsector::ROWS + 1).into_iter();
        let name = names.next().unwrap();
        self.insert_world(point, World::new(name))
    }

    /** Remove any [`World`] at `point` and return it if there was one.

    # Returns
    - `Ok(Some(World))` containing the removed world if there was one,
    - `Ok(None)` if there was no world to remove,
    - `Err(msg)` if `point` is out of bounds and the removal failed
    */
    pub fn remove_world(&mut self, point: &Point) -> Result<Option<World>, String> {
        if Self::point_is_inbounds(point) {
            Ok(self.map.remove(point))
        } else {
            Err("Can not remove a world from an out of bounds point".to_string())
        }
    }

    /** Move any [`World`] at `source` to `destination`, replacing any [`World`] there.

    # Returns
    - `Ok(Some(World))` containing the displaced world that at `destination` if the world moved
    successfully
    - `Ok(None)` if the world moved successfully to an empty location, or
    - `Err(msg)` if the world could not be moved for one of the following reasons:
        - `source` was out of bounds
        - `destination` was out of bounds
        - There was no world to move at `source`
    */
    pub fn move_world(
        &mut self,
        source: &Point,
        destination: &Point,
    ) -> Result<Option<World>, String> {
        if let Some(world) = self.remove_world(source)? {
            match self.insert_world(destination, world.clone()) {
                Err(msg) => {
                    self.insert_world(source, world)
                        .expect("World should insert back into same location with no problems");
                    Err(msg)
                }
                ok => ok,
            }
        } else {
            Err(format!("No world to move at {}", source))
        }
    }

    /** Attempts to produce a "player-safe" copy of the `Subsector`.

    To do so, for each `World` it defaults all of the fields that are likely to have spoilers to the
    zeroth index of their respective roll tables or completely blanks them where possible.
    These likely fields are:

    1. Factions
    2. Culture
    3. World Tags
    4. Notes

    This is intended to work alongside a player-safe version of the GUI that has the defaulted
    fields removed; this is more to prevent overly-clever players from mining the JSON for spoilers.
    */
    pub fn copy_player_safe(&self) -> Self {
        let mut player_safe_subsector = self.clone();
        player_safe_subsector.make_player_safe();
        player_safe_subsector
    }

    /** Attempts to mutate the `Subsector` to a "player-safe" state.

    To do so, for each `World` it defaults all of the fields that are likely to have spoilers to the
    zeroth index of their respective roll tables or completely blanks them where possible.
    These likely fields are:

    1. Factions
    2. Culture
    3. World Tags
    4. Notes

    This is intended to work alongside a player-safe version of the GUI that has the defaulted
    fields removed; this is more to prevent overly-clever players from mining the JSON for spoilers.
    */
    pub fn make_player_safe(&mut self) {
        for (_point, world) in self.map.iter_mut() {
            world.make_player_safe();
        }
    }
}

impl Default for Subsector {
    fn default() -> Self {
        Subsector::new(0)
    }
}

fn center_markers() -> BTreeMap<Point, Translation> {
    let mut reader = quick_xml::Reader::from_str(TEMPLATE_SVG);
    let mut column_translations: [Translation; Subsector::COLUMNS] =
        [Translation::default(); Subsector::COLUMNS];
    let mut circle_translations: BTreeMap<Point, Translation> = BTreeMap::new();
    loop {
        match reader.read_event() {
            Err(e) => unreachable!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,

            Ok(Event::Start(element)) => {
                let attributes: BTreeMap<_, _> = element
                    .attributes()
                    .map(|a| {
                        let attribute = a.unwrap();
                        (
                            str::from_utf8(attribute.key.as_ref()).unwrap().to_string(),
                            str::from_utf8(attribute.value.as_ref())
                                .unwrap()
                                .to_string(),
                        )
                    })
                    .collect();

                if let Some(id) = attributes.get("id") {
                    if let Some(column_num) = id.strip_prefix("CenterMarkerColumn-") {
                        // If the element is a center marker column, get the column offset
                        let column_num: usize = column_num
                            .parse()
                            .unwrap_or_else(|_| panic!("Unparsable column number in {id}"));
                        assert!(
                            (1..=Subsector::COLUMNS).contains(&column_num),
                            "Out of bounds column number while parsing {id}"
                        );

                        let column_idx = column_num - 1;
                        assert_eq!(
                            column_translations[column_idx],
                            Translation::default(),
                            "Found double definition of CenterMarkerColumn {id}"
                        );

                        if let Some(transform) = attributes.get("transform") {
                            column_translations[column_idx] =
                                Translation::try_from_transform_str(transform).unwrap();
                        }
                    }
                }
            }

            Ok(Event::Empty(element)) => {
                let attributes: BTreeMap<_, _> = element
                    .attributes()
                    .map(|a| {
                        let attribute = a.unwrap();
                        (
                            str::from_utf8(attribute.key.as_ref()).unwrap().to_string(),
                            str::from_utf8(attribute.value.as_ref())
                                .unwrap()
                                .to_string(),
                        )
                    })
                    .collect();

                if let Some(id) = attributes.get("id") {
                    if let Some(point_str) = id.strip_prefix("CenterMark-") {
                        // If the element is a center mark circle itself, get the center coordinates
                        let point = Point::try_from(point_str).unwrap();
                        assert!(
                            circle_translations.get(&point).is_none(),
                            "Found double definition of CenterMark {id}"
                        );
                        assert!(
                            Subsector::point_is_inbounds(&point),
                            "Found out-of-bounds CenterMark {id}"
                        );

                        let x: f64 = attributes
                            .get("cx")
                            .unwrap_or_else(|| panic!("Could not find cx attr while parsing {id}"))
                            .parse()
                            .unwrap_or_else(|_| panic!("Unparsable cx attr in {id}"));
                        let y: f64 = attributes
                            .get("cy")
                            .unwrap_or_else(|| panic!("Could not find cy attr while parsing {id}"))
                            .parse()
                            .unwrap_or_else(|_| panic!("Unparsable cy attr in {id}"));

                        circle_translations.insert(point, Translation { x, y });
                    }
                }
            }
            _ => (),
        }
    }

    let mut center_marks = BTreeMap::new();
    for x in 1..=Subsector::COLUMNS {
        let column_idx = x - 1;
        let column_translation = column_translations[column_idx];
        for y in 1..=Subsector::ROWS {
            let point = Point {
                x: x as i32,
                y: y as i32,
            };

            let center_mark = circle_translations
                .get(&point)
                .expect("Not all expected center marks were parsed")
                + &column_translation;
            center_marks.insert(point, center_mark);
        }
    }
    center_marks
}

fn map_legend_translation(id: &str) -> Translation {
    let mut reader = quick_xml::Reader::from_str(TEMPLATE_SVG);
    loop {
        match reader.read_event() {
            Err(e) => unreachable!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => unreachable!("Failed to find {id} before readching EOF"),

            Ok(Event::Start(element)) => {
                let attributes: BTreeMap<_, _> = element
                    .attributes()
                    .map(|a| {
                        let attribute = a.unwrap();
                        (
                            str::from_utf8(attribute.key.as_ref()).unwrap().to_string(),
                            str::from_utf8(attribute.value.as_ref())
                                .unwrap()
                                .to_string(),
                        )
                    })
                    .collect();

                if let Some(found_id) = attributes.get("id") {
                    if id == found_id {
                        let x = attributes
                            .get("cx")
                            .unwrap_or_else(|| panic!("Fail to find cx attr translating {id}"))
                            .parse()
                            .unwrap_or_else(|_| panic!("Fail to parse cx value translating {id}"));
                        let y = attributes
                            .get("cy")
                            .unwrap_or_else(|| panic!("Fail to find cy attrib translating {id}"))
                            .parse()
                            .unwrap_or_else(|_| panic!("Fail to parse cy value translating {id}"));
                        return Translation { x, y };
                    }
                }
            }

            Ok(Event::Empty(element)) => {
                let attributes: BTreeMap<_, _> = element
                    .attributes()
                    .map(|a| {
                        let attribute = a.unwrap();
                        (
                            str::from_utf8(attribute.key.as_ref()).unwrap().to_string(),
                            str::from_utf8(attribute.value.as_ref())
                                .unwrap()
                                .to_string(),
                        )
                    })
                    .collect();

                if let Some(found_id) = attributes.get("id") {
                    if id == found_id {
                        let x = attributes
                            .get("cx")
                            .unwrap_or_else(|| panic!("Fail to find cx attr translating {id}"))
                            .parse()
                            .unwrap_or_else(|_| panic!("Fail to parse cx value translating {id}"));
                        let y = attributes
                            .get("cy")
                            .unwrap_or_else(|| panic!("Fail to find cy attr translating {id}"))
                            .parse()
                            .unwrap_or_else(|_| panic!("Fail to parse cy value translating {id}"));
                        return Translation { x, y };
                    }
                }
            }
            _ => (),
        }
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
        const ATTEMPTS: usize = 1000;
        for _ in 0..ATTEMPTS {
            Subsector::default();
        }
    }

    #[test]
    fn subsector_json_serde() {
        const ATTEMPTS: usize = 100;
        for _ in 0..ATTEMPTS {
            let subsector = Subsector::default();
            let json = subsector.to_json();
            let deserialized = Subsector::try_from_json(&json[..]).unwrap();
            assert_eq!(deserialized, subsector);
        }
    }

    #[test]
    fn subsector_svg() {
        const ATTEMPTS: usize = 100;
        for _ in 0..ATTEMPTS {
            let subsector = Subsector::default();
            let _svg = subsector.generate_svg(false);
        }
    }
}
