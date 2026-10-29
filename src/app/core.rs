use crate::app::service::AppService;

use crate::models::rest::{RequestDraft, ResponseSnapshot, SavedRequest};
use crate::ui::core::show;

pub struct App {
    pub request_draft: RequestDraft,
    pub response_snapshot: Option<ResponseSnapshot>,
    pub history: Vec<SavedRequest>,
    pub status: String,
    pub service: AppService,
}

impl Default for App {
    fn default() -> Self {
        Self {
            request_draft: RequestDraft::default(),
            response_snapshot: None,
            history: Vec::new(),
            status: String::new(),
            service: AppService::new(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(result) = self.service.poll(ui.ctx()) {
            match result {
                Ok(snapshot) => {
                    self.response_snapshot = Some(snapshot);
                    self.status.clear();
                }
                Err(err) => self.status = err.to_string(),
            }
        }
        show(ui, self);
    }
}
