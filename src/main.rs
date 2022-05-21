use traveller_generator::*;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1600.0, 900.0)),
        ..Default::default()
    };

    eframe::run_native(
        "svg example",
        options,
        Box::new(|_cc| Box::new(app::GeneratorApp::default())),
    );
}
