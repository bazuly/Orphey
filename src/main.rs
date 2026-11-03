mod app;
mod error;
mod models;
mod repository;
mod service;
mod ui;
mod utils;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Orphey",
        native_options,
        Box::new(|_cc| Ok(Box::new(app::core::App::new()))),
    )
}
