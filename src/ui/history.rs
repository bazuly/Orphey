pub fn show(ui: &mut egui::Ui, app: &mut crate::app::core::App) {
    egui::Panel::left("history")
        .exact_size(280.0)
        .show(ui, |ui| {
            ui.heading("History");
            let mut chosen = None;

            for (index, saved) in app.history.iter().enumerate() {
                if ui.button(&saved.name).clicked() {
                    chosen = Some(index);
                }
            }
            if let Some(index) = chosen {
                app.request_draft = app.history[index].draft.clone();
            }
        });
}
