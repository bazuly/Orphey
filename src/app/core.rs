use uuid::Uuid;

use crate::models::request::{RequestDraft, ResponseSnapshot, SavedRequest};
use crate::models::room::Room;
use crate::repository::db;
use crate::repository::requests::RequestRepository;
use crate::repository::rooms::RoomRepository;
use crate::service::http::AppService;
use crate::service::requests::RequestService;
use crate::service::rooms::RoomService;
use crate::ui::core::show;

pub struct App {
    pub request_draft: RequestDraft,
    pub response_snapshot: Option<ResponseSnapshot>,
    pub history: Vec<SavedRequest>,
    pub status: String,
    pub http: AppService,
    pub request_name: String,
    pub room_draft: String,
    pub rooms: Vec<Room>,
    pub current_room: Uuid,
    pub selected_id: Option<Uuid>,
    pub rooms_svc: RoomService,
    pub requests_svc: RequestService,
}

impl App {
    pub fn new() -> Self {
        let (conn, mut status) = match db::open() {
            Ok(conn) => (Some(conn), String::new()),
            Err(err) => (None, err.to_string()),
        };

        let rooms_svc =
            RoomService::new(conn.as_ref().map(|conn| RoomRepository::new(conn.clone())));
        let requests_svc = RequestService::new(conn.map(|conn| RequestRepository::new(conn)));

        let rooms = match rooms_svc.list() {
            Ok(rooms) => rooms,
            Err(err) => {
                status = err.to_string();
                Vec::new()
            }
        };
        let current_room = rooms.first().map(|room| room.id).unwrap_or_else(Uuid::nil);
        let history = match requests_svc.list(current_room) {
            Ok(history) => history,
            Err(err) => {
                if status.is_empty() {
                    status = err.to_string();
                }
                Vec::new()
            }
        };

        Self {
            request_draft: RequestDraft::default(),
            request_name: String::new(),
            room_draft: String::new(),
            response_snapshot: None,
            history,
            status,
            http: AppService::new(),
            rooms_svc,
            requests_svc,
            rooms,
            current_room,
            selected_id: None,
        }
    }

    pub fn open_room(&mut self, id: Uuid) {
        self.current_room = id;
        self.selected_id = None;
        self.request_name.clear();
        self.request_draft = RequestDraft::default();
        self.response_snapshot = None;

        match self.requests_svc.list(id) {
            Ok(history) => self.history = history,
            Err(err) => {
                self.history.clear();
                self.status = err.to_string();
            }
        }
    }

    pub fn create_room(&mut self, name: String) {
        match self.rooms_svc.create(name) {
            Ok(room) => {
                self.rooms.push(room.clone());
                self.open_room(room.id);
            }
            Err(err) => self.status = err.to_string(),
        }
    }

    pub fn delete_room(&mut self, id: Uuid) {
        if self.rooms.len() <= 1 {
            self.status = "Keep at least one room alive".to_string();
            return;
        }

        if let Err(err) = self.rooms_svc.delete(id) {
            self.status = err.to_string();
            return;
        }

        self.rooms.retain(|room| room.id != id);

        if self.current_room == id {
            if let Some(next) = self.rooms.first() {
                self.open_room(next.id);
            }
        }
    }

    pub fn delete_request(&mut self, id: Uuid) {
        if let Err(err) = self.requests_svc.delete(id) {
            self.status = err.to_string();
            return;
        }
        self.history.retain(|saved| saved.id != id);
        if self.selected_id == Some(id) {
            self.selected_id = None;
            self.request_name.clear();
            self.request_draft = RequestDraft::default();
            self.response_snapshot = None;
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(result) = self.http.poll(ui.ctx()) {
            match result {
                Ok((draft, snapshot)) => {
                    let selected = self.selected_id;
                    let saved = self.requests_svc.create(
                        self.current_room,
                        selected,
                        &self.request_name,
                        draft,
                        snapshot.clone(),
                        &self.history,
                    );
                    match saved {
                        Ok(saved) => {
                            if selected.is_some() {
                                if let Some(slot) =
                                    self.history.iter_mut().find(|item| item.id == saved.id)
                                {
                                    *slot = saved.clone();
                                }
                            } else {
                                self.history.insert(0, saved.clone());
                            }
                            self.selected_id = Some(saved.id);
                            self.request_name = saved.name;
                            self.response_snapshot = Some(snapshot);
                            self.status = "Data saved successfully".to_string();
                        }
                        Err(err) => {
                            self.status = format!("Error while saving data {err}");
                        }
                    }
                }
                Err(err) => {
                    self.status = err.to_string();
                }
            }
        }
        show(ui, self);
    }
}
