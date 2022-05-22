#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::vec2;

use traveller_generator::*;

fn main() {
    let options = eframe::NativeOptions {
        min_window_size: Some(vec2(1600.0, 900.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Subsector Generator",
        options,
        Box::new(|_cc| Box::new(app::GeneratorApp::default())),
    );
}
