use crate::models::rest::{Method, RequestDraft};

pub fn show(ui: &mut egui::Ui, app: &mut crate::app::core::App) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading("Request");

        // previous request template GET by default
        let previous = &app.request_draft.method.clone();

        method_and_url(ui, &mut app.request_draft);
        if app.request_draft.method != *previous {
            app.request_draft
                .clone()
                .apply_method_template(previous.clone());
        }
        headers(ui, &mut app.request_draft.headers);
        body(ui, &mut app.request_draft.body);
        send(ui, app);
    });
}

fn method_and_url(ui: &mut egui::Ui, draft: &mut RequestDraft) {
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("method")
            .selected_text(format!("{:?}", draft.method))
            .show_ui(ui, |ui| {
                for method in Method::ALL {
                    ui.selectable_value(&mut draft.method, method.clone(), format!("{method:?}"));
                }
            });
        ui.add(egui::TextEdit::singleline(&mut draft.url).desired_width(f32::INFINITY));
    });
}

fn headers(ui: &mut egui::Ui, headers: &mut Vec<(String, String)>) {
    ui.label("Headers");
    let mut remove_header = None;
    egui::Grid::new("headers").num_columns(3).show(ui, |ui| {
        for (index, (name, value)) in headers.iter_mut().enumerate() {
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
        headers.remove(index);
    }
    if ui.button("Add header").clicked() {
        headers.push((String::new(), String::new()));
    }
}

fn body(ui: &mut egui::Ui, body: &mut String) {
    ui.label("Body");
    ui.add(
        egui::TextEdit::multiline(body)
            .desired_width(f32::INFINITY)
            .desired_rows(12),
    );
}

fn send(ui: &mut egui::Ui, app: &mut crate::app::core::App) {
    if ui.button("Send").clicked() {
        let draft = app.request_draft.clone();
        if app.service.begin(draft, ui.ctx().clone()) {
            app.status = "In progress".to_string();
        }
    }
}
