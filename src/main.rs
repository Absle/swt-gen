use traveller_generator::*;

fn main() {
    let subsector = astrography::Subsector::new(-1);
    let _csv = subsector.generate_csv();
    let _svg = subsector.generate_svg();

    println!("{}", _csv);
}
