use traveller_subsector_generator::*;

fn main() {
    //astrography::World::histograms(10_000);

    let subsector = astrography::Subsector::new();
    //subsector.show();
    let yaml = serde_yaml::to_string(&subsector).unwrap();
    //println!("{}", yaml);

    let de_subsector: astrography::Subsector = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(de_subsector, subsector);

    subsector.show();
    de_subsector.show();
}
