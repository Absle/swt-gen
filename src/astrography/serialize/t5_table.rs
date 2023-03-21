use std::collections::HashMap;
use std::fmt;

use crate::astrography::{Point, Subsector, World};

const UWP_REFERENCE: &str = r"# UWP Reference Diagram:
#
#     CA6A643-9
#     ||||||| `- Tech Level
#     |||||| `- Law Level
#     ||||| `- Government
#     |||| `- Population
#     ||| `- Hydrographics
#     || `- Atmosphere
#     | `- Size
#      `- Starport";

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Header {
    Hex,
    Name,
    UniversalWorldProfile,
    Remarks,
    Bases,
    Zone,
    Allegiance,
    ImportanceExtension,
    EconomicExtension,
    CulturalExtension,
    Nobility,
    PopModBeltsGasGiants,
    Worlds,
    Stellar,
}

impl Header {
    const ALL_VALUES: [Header; 14] = [
        Header::Hex,
        Header::Name,
        Header::UniversalWorldProfile,
        Header::Remarks,
        Header::Bases,
        Header::Zone,
        Header::Allegiance,
        Header::ImportanceExtension,
        Header::EconomicExtension,
        Header::CulturalExtension,
        Header::Nobility,
        Header::PopModBeltsGasGiants,
        Header::Worlds,
        Header::Stellar,
    ];
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Header::Hex => "Hex",
            Header::Name => "Name",
            Header::UniversalWorldProfile => "UWP",
            Header::Remarks => "Remarks",
            Header::Bases => "B",
            Header::Zone => "Z",
            Header::Allegiance => "A",
            Header::ImportanceExtension => "{Ix}",
            Header::EconomicExtension => "(Ex)",
            Header::CulturalExtension => "[Cx]",
            Header::Nobility => "N",
            Header::PopModBeltsGasGiants => "PBG",
            Header::Worlds => "W",
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

        let mut columns = HashMap::new();
        for header in Header::ALL_VALUES {
            let _ = match header {
                Header::Hex => columns.insert(header, point.to_string()),
                Header::Name => columns.insert(header, world.name.clone()),
                Header::UniversalWorldProfile => columns.insert(header, world.profile_str()),
                Header::Remarks => columns.insert(header, world.trade_code_str()),
                Header::Bases => columns.insert(header, world.base_str()),
                Header::Zone => columns.insert(header, world.travel_code.as_short_string()),
                Header::Allegiance => columns.insert(header, "Na".to_string()),
                Header::ImportanceExtension => columns.insert(header, world.importance_extension()),
                Header::EconomicExtension => columns.insert(header, "-".to_string()),
                Header::CulturalExtension => columns.insert(header, "-".to_string()),
                Header::Nobility => columns.insert(header, "-".to_string()),
                Header::PopModBeltsGasGiants => columns.insert(
                    header,
                    if world.has_gas_giant { "101" } else { "100" }.to_string(),
                ),
                Header::Worlds => columns.insert(header, "1".to_string()),
                Header::Stellar => columns.insert(header, "-".to_string()),
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
