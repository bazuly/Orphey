mod app;
mod error;
mod models;
mod ui;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "oscar",
        native_options,
        Box::new(|_cc| Ok(Box::new(app::App::default()))),
    )
}
