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
        format!("output/{}-{}.csv", subsector.name(), time.as_secs()),
        csv,
    )
    .unwrap();
    fs::write(
        format!("output/{}-{}.svg", subsector.name(), time.as_secs()),
        svg,
    )
    .unwrap();

    // let read_in_csv = fs::read_to_string("output/Sonora-6252ea92.csv").unwrap();
    // let subsector = astrography::Subsector::from_csv(&read_in_csv).unwrap();
    // let csv = subsector.to_csv();
    // let svg = subsector.generate_svg();

    // let time = SystemTime::now()
    //     .duration_since(SystemTime::UNIX_EPOCH)
    //     .unwrap();

    // fs::write(
    //     format!("output/{}-{}.csv", subsector.name(), time.as_secs()),
    //     csv,
    // )
    // .unwrap();
    // fs::write(
    //     format!("output/{}-{}.svg", subsector.name(), time.as_secs()),
    //     svg,
    // )
    // .unwrap();
}
