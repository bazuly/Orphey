use crate::app::service::AppService;

use crate::infra::store::{self, HistoryStore};
use crate::models::rest::{RequestDraft, ResponseSnapshot, SavedRequest};
use crate::ui::core::show;

pub struct App {
    pub request_draft: RequestDraft,
    pub response_snapshot: Option<ResponseSnapshot>,
    pub history: Vec<SavedRequest>,
    pub status: String,
    pub service: AppService,
    pub store: Option<HistoryStore>,
}

impl App {
    pub fn new() -> Self {
        let (store, history, status) = match HistoryStore::open() {
            Ok(store) => match store.list() {
                Ok(history) => (Some(store), history, String::new()),
                Err(err) => (Some(store), Vec::new(), err.to_string()),
            },
            Err(_) => (None, Vec::new(), String::new()),
        };

        Self {
            request_draft: RequestDraft::default(),
            response_snapshot: None,
            history: history,
            status: status,
            service: AppService::new(),
            store: store,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(result) = self.service.poll(ui.ctx()) {
            match result {
                Ok((draft, snapshot)) => {
                    let saved = SavedRequest::from_exchange(draft, snapshot.clone());
                    if let Some(store) = &self.store {
                        if let Err(err) = store.insert(&saved) {
                            self.status = err.to_string();
                        }
                    }
                    self.history.insert(0, saved);
                    self.response_snapshot = Some(snapshot);
                    self.status.clear();
                }
                Err(err) => {
                    self.status = err.to_string();
                }
            }
        }
        show(ui, self);
    }
}
