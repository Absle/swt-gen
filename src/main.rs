// Hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(clippy::todo)]

use egui::vec2;

use swt_gen::GeneratorApp;

fn main() {
    let options = eframe::NativeOptions {
        min_window_size: Some(vec2(1760.0, 990.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Subsector Generator",
        options,
        Box::new(|_cc| Box::<GeneratorApp>::default()),
    );
}
