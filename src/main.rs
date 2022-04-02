use traveller_generator::*;

fn main() {
    //astrography::World::histograms(10_000);

    let subsector = astrography::Subsector::new();
    let yaml = serde_yaml::to_string(&subsector).unwrap();
    //println!("{}", yaml);

    let de_subsector: astrography::Subsector = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(de_subsector, subsector);

    println!("{}", subsector.to_csv());

    // subsector.show();
    // de_subsector.show();

    // let maybe_yaml = fs::read_to_string("output/maybe.yaml").unwrap();
    // let maybe_subsector: astrography::Subsector = serde_yaml::from_str(&maybe_yaml).unwrap();
    // let mut writer = csv::Writer::from_path("output/maybe.csv").unwrap();
    // writer.serialize(maybe_subsector).unwrap();
    // writer.flush().unwrap();
}
