use crate::models::rest::Method;

pub fn show(ui: &mut egui::Ui, app: &mut crate::app::core::App) {
    egui::CentralPanel::default().show(ui, |ui| {
        let previous = app.request_draft.method.clone();

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

        if app.request_draft.method != previous {
            let previous_template = previous.body_template();
            let body = app.request_draft.body.trim();

            if body.is_empty() || app.request_draft.body == previous_template {
                app.request_draft.body =
                    app.request_draft.method.clone().body_template().to_string();
            }
        }

        ui.label("Headers");
        let mut remove_header = None;
        egui::Grid::new("headers").num_columns(3).show(ui, |ui| {
            for (index, (name, value)) in app.request_draft.headers.iter_mut().enumerate() {
                ui.add(
                    egui::TextEdit::singleline(name)
                        .desired_width(160.0)
                        .hint_text("Name"),
                );
                ui.add(
                    egui::TextEdit::singleline(value)
                        .desired_width(320.0)
                        .hint_text("Value"),
                );
                if ui.button("x").clicked() {
                    remove_header = Some(index);
                }
                ui.end_row();
            }
        });
        if let Some(index) = remove_header {
            app.request_draft.headers.remove(index);
        }
        if ui.button("Add header").clicked() {
            app.request_draft
                .headers
                .push((String::new(), String::new()));
        }

        ui.label("Body");
        ui.add(
            egui::TextEdit::multiline(&mut app.request_draft.body)
                .desired_width(f32::INFINITY)
                .desired_rows(12),
        );

        if ui.button("Send").clicked() {
            let draft = app.request_draft.clone();
            if app.service.begin(draft, ui.ctx().clone()) {
                app.status = "In progress".to_string();
            }
        }
    });
}
