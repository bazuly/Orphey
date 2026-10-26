use crate::models::rest::Method;

pub fn show(ui: &mut egui::Ui, app: &mut crate::app::App) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading("Request");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("method")
                .selected_text(format!("{:?}", app.request_draft.method))
                .show_ui(ui, |ui| {
                    for method in [
                        Method::GET,
                        Method::POST,
                        Method::PUT,
                        Method::PATCH,
                        Method::DELETE,
                    ] {
                        ui.selectable_value(
                            &mut app.request_draft.method,
                            method.clone(),
                            format!("{method:?}"),
                        );
                    }
                });
            ui.add(
                egui::TextEdit::singleline(&mut app.request_draft.url).desired_width(f32::INFINITY),
            );
        });
        ui.label("Body");
        ui.add(
            egui::TextEdit::multiline(&mut app.request_draft.body)
                .desired_width(f32::INFINITY)
                .desired_rows(12),
        );
        if ui.button("Send").clicked() {
            app.status = "In progress".to_string();
        }
    });
}
