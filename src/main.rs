use traveller_generator::*;

fn main() {
    let subsector = astrography::Subsector::new(-1);
    let yaml = serde_yaml::to_string(&subsector).unwrap();

    let de_subsector: astrography::Subsector = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(de_subsector, subsector);

    let csv = subsector.to_csv();

    println!("{}", subsector.generate_svg());
}
