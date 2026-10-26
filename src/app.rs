use crate::{
    models::rest::{RequestDraft, ResponseSnapshot, SavedRequest},
    ui::response::show,
};

pub struct App {
    pub request_draft: RequestDraft,
    pub response_snapshot: Option<ResponseSnapshot>,
    pub history: Vec<SavedRequest>,
    pub status: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            request_draft: RequestDraft::default(),
            response_snapshot: None,
            history: Vec::new(),
            status: String::new(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        show(ui, self);
    }
}
