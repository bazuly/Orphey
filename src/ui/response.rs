pub fn show(ui: &mut egui::Ui, app: &crate::app::App) {
    egui::Panel::right("responses")
        .exact_size(420.0)
        .show(ui, |ui| {
            ui.heading("Response");
            if let Some(response) = &app.response_snapshot {
                ui.label(format!("{}   {} мс", response.status, response.elapsed_ms));
                ui.label(&response.body);
            }
        });
}
