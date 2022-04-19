use std::fs;
use std::time::SystemTime;

use traveller_generator::*;

fn main() {
    let subsector = astrography::Subsector::new(0);
    let csv = subsector.to_csv();
    let svg = subsector.generate_svg();

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    fs::write(
        format!("output/{}-{:x}.csv", subsector.name(), time.as_secs()),
        csv,
    )
    .unwrap();
    fs::write(
        format!("output/{}-{:x}.svg", subsector.name(), time.as_secs()),
        svg,
    )
    .unwrap();
}
