use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::astrography::{Point, Subsector, World};

/** Representation of a `Subsector` that can be easily serialized to JSON.

Specifically, `serde_json` requires all maps use `String` keys, so to accomodate this we create this
representation using the result of `Point::to_string` as the key for `Subsector::map`.
*/
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct JsonableSubsector {
    name: String,
    map: BTreeMap<String, World>,
}

impl fmt::Display for JsonableSubsector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

impl From<&Subsector> for JsonableSubsector {
    fn from(subsector: &Subsector) -> Self {
        let mut map: BTreeMap<String, World> = BTreeMap::new();
        for (point, world) in subsector.map.iter() {
            map.insert(point.to_string(), world.clone());
        }

        Self {
            name: subsector.name.clone(),
            map,
        }
    }
}

impl TryFrom<JsonableSubsector> for Subsector {
    type Error = Box<dyn Error>;
    fn try_from(jsonable: JsonableSubsector) -> Result<Self, Self::Error> {
        let JsonableSubsector { name, map } = jsonable;
        let mut point_map: BTreeMap<Point, World> = BTreeMap::new();
        for (point_str, mut world) in map {
            let point = Point::try_from(&point_str[..])?;
            world.normalize_data();
            point_map.insert(point, world);
        }

        Ok(Self {
            name,
            map: point_map,
        })
    }
}
