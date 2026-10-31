use crate::{
    models::rest::{Method, RequestDraft},
    ui::consts,
};

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

    for (index, (name, value)) in headers.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(name)
                    .desired_width(180.0)
                    .hint_text("Name"),
            );
            let value_width = (ui.available_width() - 36.0).max(80.0);
            ui.add(
                egui::TextEdit::singleline(value)
                    .desired_width(value_width)
                    .hint_text("Value"),
            );
            if ui.button("x").clicked() {
                remove_header = Some(index);
            }
        });
    }

    if let Some(index) = remove_header {
        headers.remove(index);
    }

    ui.horizontal(|ui| {
        if ui.button("Add headers").clicked() {
            headers.push((String::new(), String::new()));
        }
        select_headers(ui, headers);
    });
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

fn select_headers(ui: &mut egui::Ui, headers: &mut Vec<(String, String)>) {
    let mut picked = None;
    egui::ComboBox::from_id_salt("header_template")
        .selected_text("Header template")
        .show_ui(ui, |ui| {
            for (name, value) in consts::HEADER_TEMPLATES {
                if ui.selectable_label(false, *name).clicked() {
                    picked = Some((*name, *value));
                }
            }
        });

    if let Some((name, value)) = picked {
        let exists = headers
            .iter()
            .any(|(current, _)| current.eq_ignore_ascii_case(name));
        if !exists {
            headers.push((name.to_string(), value.to_string()));
        }
    }
}
