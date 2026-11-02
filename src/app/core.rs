use eframe::wgpu::wgt::AccelerationStructureCopy::Clone;
use rusqlite::ffi::sqlite3_snapshot;
use uuid::Uuid;

use crate::app::service::AppService;

use crate::error::Error;
use crate::infra::store::HistoryStore;
use crate::models::rest::{RequestDraft, ResponseSnapshot, SavedRequest};
use crate::models::room::Room;
use crate::ui::core::show;

pub struct App {
    pub request_draft: RequestDraft,
    pub response_snapshot: Option<ResponseSnapshot>,
    pub history: Vec<SavedRequest>,
    pub status: String,
    pub service: AppService,
    pub store: Option<HistoryStore>,
    pub request_name: String,
    pub rooms: Vec<Room>,
    pub current_room: Uuid,
    pub selected_id: Option<Uuid>,
    // room_service (service layer + repo layer)
    // request_service (service lauer + repo layer)
}

impl App {
    pub fn new() -> Self {
        let loaded = HistoryStore::open().and_then(|store| {
            let rooms = store.list_rooms()?;
            let current_room = rooms
                .first()
                .map(|room| room.id)
                .ok_or_else(|| crate::error::Error::Database("rooms empty".into()))?;
            let history = store.list_requests(current_room)?;

            Ok((store, rooms, current_room, history))
        });
        let (store, rooms, current_room, history, status) = match loaded {
            Ok((store, rooms, current_room, history)) => {
                (Some(store), rooms, current_room, history, String::new())
            }
            Err(err) => (None, Vec::new(), Uuid::nil(), Vec::new(), err.to_string()),
        };
        Self {
            request_draft: RequestDraft::default(),
            request_name: String::new(),
            response_snapshot: None,
            history,
            status,
            service: AppService::new(),
            store,
            rooms,
            current_room,
            selected_id: None,
        }
    }

    fn insert_new(&mut self, draft: RequestDraft, snapshot: ResponseSnapshot) -> Result<(), Error> {
        let saved: SavedRequest = SavedRequest::from_exchange(self.current_room, draft, snapshot);

        let trimmed_name = self.request_name.trim();
        if !trimmed_name.is_empty() {
            let _ = saved.name == trimmed_name.to_string();
        }

        if let Some(store) = &self.store {
            store.insert_request(&saved)?;
        }
        self.request_name = saved.name.clone();
        self.selected_id = Some(saved.id);
        self.history.insert(0, saved);

        Ok(())
    }

    fn update_selected(
        &mut self,
        id: Uuid,
        draft: RequestDraft,
        snapshot: ResponseSnapshot,
    ) -> Result<(), Error> {
        let saved_in_history = self
            .history
            .iter_mut()
            .find(|saved| saved.id == id)
            .ok_or_else(|| Error::Database("Selected data not found".into()))?;

        let mut updated_saved = saved_in_history.clone();

        updated_saved.refresh(draft, snapshot);
        let trimmed_name = self.request_name.trim();
        if !trimmed_name.is_empty() {
            updated_saved.name = trimmed_name.to_string();
        }
        if let Some(store) = &self.store {
            store.update(&updated_saved)?;
        }
        *saved_in_history = updated_saved;

        Ok(())
    }

    // TODO вынести в отедльный модуль
    pub fn open_room(&mut self, id: Uuid) {
        self.current_room = id;
        self.selected_id = None;
        self.request_name.clear();
        self.request_draft = RequestDraft::default();
        self.response_snapshot = None;

        match self.store.as_ref().map(|store| store.list_requests(id)) {
            Some(Ok(history)) => self.history = history,
            Some(Err(err)) => {
                self.history.clear();
                self.status = err.to_string();
            }
            None => self.history.clear(),
        }
    }

    pub fn create_room(&mut self, name: String) {
        let room = Room::new(name);
        if let Some(store) = &self.store {
            if let Err(err) = store.insert_room(&room) {
                self.status = err.to_string();
                return;
            }
        }
        self.rooms.push(room.clone());
        self.open_room(room.id);
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(result) = self.service.poll(ui.ctx()) {
            match result {
                Ok((draft, snapshot)) => {
                    let update_result = match self.selected_id {
                        Some(id) => self.update_selected(id, draft, snapshot.clone()),
                        None => self.insert_new(draft, snapshot.clone()),
                    };
                    match update_result {
                        Ok(()) => {
                            self.response_snapshot = Some(snapshot);
                            self.status = "Data saved successfully".to_string();
                        }
                        Err(err) => {
                            self.status = format!("Error while saving data {}", err);
                        }
                    }
                }
                Err(err) => {
                    self.status = err.to_string();
                }
            }

            show(ui, self);
        }
    }
}
