pub fn show(ui: &mut egui::Ui, app: &mut crate::app::core::App) {
    egui::Panel::right("responses")
        .default_size(420.0)
        .min_size(240.0)
        .max_size(720.0)
        .show(ui, |ui| {
            ui.heading("Response");
            if let Some(response) = &mut app.response_snapshot {
                ui.label(format!("{}   {} мс", response.status, response.elapsed_ms));
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut response.body)
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .interactive(false),
                    );
                });
            }
        });
}
