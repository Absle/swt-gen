use std::collections::HashMap;
use std::fmt;

use crate::astrography::{Point, Subsector, World};

const UWP_REFERENCE: &str = r"# UWP Reference Diagram:
#
#      ,- Starport
#     |  ,- Atmosphere
#     | |  ,- Population
#     | | |  ,- Law Level
#     | | | |
#     CA6A643-9
#      | | |  |
#      | | |   `- Tech Level
#      | |  `- Government
#      |  `- Hydrographics
#       `- Size";

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Header {
    Name,
    Hex,
    UniversalWorldProfile,
    Bases,
    Remarks,
    Zone,
    PlanetsBeltsGasGiants,
    Allegiance,
    Stellar,
}

impl Header {
    const ALL_VALUES: [Header; 9] = [
        Header::Name,
        Header::Hex,
        Header::UniversalWorldProfile,
        Header::Bases,
        Header::Remarks,
        Header::Zone,
        Header::PlanetsBeltsGasGiants,
        Header::Allegiance,
        Header::Stellar,
    ];
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Header::Name => "Name",
            Header::Hex => "Hex",
            Header::UniversalWorldProfile => "UWP",
            Header::Bases => "Bases",
            Header::Remarks => "Remarks",
            Header::Zone => "Zone",
            Header::PlanetsBeltsGasGiants => "PBG",
            Header::Allegiance => "A",
            Header::Stellar => "Stellar",
        };
        write!(f, "{}", s)
    }
}

struct T5Record {
    columns: HashMap<Header, String>,
}

impl From<(&World, &Point)> for T5Record {
    fn from(value: (&World, &Point)) -> Self {
        let (world, point) = value;
        let pbg = if world.has_gas_giant { "101" } else { "100" };

        let mut columns = HashMap::new();
        for header in Header::ALL_VALUES {
            let _ = match header {
                Header::Name => columns.insert(header, world.name.clone()),
                Header::Hex => columns.insert(header, point.to_string()),
                Header::UniversalWorldProfile => columns.insert(header, world.profile_str()),
                Header::Bases => columns.insert(header, world.base_str()),
                Header::Remarks => columns.insert(header, world.trade_code_str()),
                Header::Zone => columns.insert(header, world.travel_code.as_short_string()),
                Header::PlanetsBeltsGasGiants => columns.insert(header, pbg.to_string()),
                Header::Allegiance => columns.insert(header, "Na".to_string()),
                Header::Stellar => columns.insert(header, String::new()),
            };
        }

        Self { columns }
    }
}

pub(crate) struct T5Table {
    rows: Vec<T5Record>,
}

impl fmt::Display for T5Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Find the minimum width of each column
        let mut widths = HashMap::new();
        for header in Header::ALL_VALUES {
            // Find the width of the longest value in either the headers or the values
            let width = header.to_string().len();
            let width = self.rows.iter().fold(width, |acc, row| {
                let len = row.columns[&header].len();
                if len > acc {
                    len
                } else {
                    acc
                }
            });

            widths.insert(header, width);
        }

        // Write out the header and separator rows
        let width = widths[&Header::ALL_VALUES[0]];
        let mut header_row = format!("{:width$}", Header::ALL_VALUES[0].to_string());
        let mut separator_row = format!("{:-<width$}", "");
        for header in &Header::ALL_VALUES[1..] {
            let width = widths[header];
            header_row += &format!("  {:width$}", header.to_string());
            separator_row += &format!("  {:-<width$}", "");
        }
        writeln!(f, "{}", header_row)?;
        writeln!(f, "{}", separator_row)?;

        for row in self.rows.iter() {
            let header = &Header::ALL_VALUES[0];
            let width = widths[header];
            let mut row_str = format!("{:width$}", row.columns[header]);
            for header in &Header::ALL_VALUES[1..] {
                let width = widths[header];
                row_str += &format!("  {:width$}", row.columns[header]);
            }
            let trimmed = row_str.trim();

            writeln!(f, "{}", trimmed)?;
        }

        write!(f, "\n{}", UWP_REFERENCE)
    }
}

impl From<&Subsector> for T5Table {
    fn from(value: &Subsector) -> Self {
        let mut rows = Vec::new();
        for (point, world) in value.map.iter() {
            rows.push(T5Record::from((world, point)));
        }

        Self { rows }
    }
}
