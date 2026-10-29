pub fn show(ui: &mut egui::Ui, app: &crate::app::core::App) {
    egui::Panel::left("history")
        .exact_size(280.0)
        .show(ui, |ui| {
            ui.heading("History");
            for saved in &app.history {
                ui.label(&saved.name);
            }
        });
}
