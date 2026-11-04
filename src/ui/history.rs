use crate::app::core::App;

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    egui::Panel::left("history")
        .default_size(280.0)
        .min_size(180.0)
        .max_size(480.0)
        .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                rooms(ui, app);
                ui.separator();
                requests(ui, app);
            });
        });
}

fn rooms(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Rooms");
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut app.room_draft)
                .desired_width(160.0)
                .hint_text("Room"),
        );
        if ui.button("Add room").clicked() {
            let name = app.room_draft.trim().to_string();
            if !name.is_empty() {
                app.room_draft.clear();
                app.create_room(name);
            }
        }
    });

    let mut switch_to = None;
    let mut delete_id = None;

    for room in &app.rooms {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(app.current_room == room.id, &room.name)
                .clicked()
            {
                switch_to = Some(room.id);
            }
            if ui.button("x").clicked() {
                delete_id = Some(room.id);
            }
        });
    }
    if let Some(id) = delete_id {
        app.delete_room(id);
    } else if let Some(id) = switch_to {
        app.open_room(id);
    }
}

fn requests(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Requests");
    if ui.button("New").clicked() {
        app.selected_id = None;
        app.request_name.clear();
        app.request_draft = crate::models::request::RequestDraft::default();
        app.response_snapshot = None;
    }

    let mut chosen = None;
    let mut delete_id = None;

    for saved in &app.history {
        ui.horizontal(|ui| {
            let selected = app.selected_id == Some(saved.id);
            if ui.selectable_label(selected, &saved.name).clicked() {
                chosen = Some(saved.clone());
            }
            if ui.button("x").clicked() {
                delete_id = Some(saved.id);
            }
        });
    }
    if let Some(id) = delete_id {
        app.delete_request(id);
    } else if let Some(saved) = chosen {
        app.selected_id = Some(saved.id);
        app.request_name = saved.name;
        app.request_draft = saved.draft;
        app.response_snapshot = Some(saved.response);
    }
}
