use crate::ui::history;
use crate::ui::requests;
use crate::ui::response;

pub fn show(ui: &mut egui::Ui, app: &mut crate::app::App) {
    egui::Panel::bottom("status").show(ui, |ui| {
        ui.label(&app.status);
    });
    history::show(ui, app);
    response::show(ui, app);
    requests::show(ui, app);
}
