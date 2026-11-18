use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::error::Error;
use crate::models::request::{RequestDraft, ResponseSnapshot, SavedRequest};
use crate::models::room::Room;
use crate::repository::db;
use crate::repository::http;
use crate::repository::requests::RequestRepository;
use crate::repository::rooms::RoomRepository;
use crate::service::requests::RequestService;
use crate::service::rooms::RoomService;

pub struct Backend {
    pub rooms_svc: RoomService,
    pub requests_svc: RequestService,
    rt: Runtime,
}

impl Backend {
    pub fn open() -> Result<Self, Error> {
        let conn = db::open()?;
        let rooms_svc = RoomService::new(Some(RoomRepository::new(conn.clone())));
        let requests_svc = RequestService::new(Some(RequestRepository::new(conn)));

        let rt = Runtime::new().map_err(|err| Error::Network(err.to_string()))?;

        Ok(Self {
            rooms_svc,
            requests_svc,
            rt,
        })
    }

    pub fn list_rooms(&self) -> Result<Vec<Room>, Error> {
        self.rooms_svc.list()
    }

    pub fn list_requests(&self, room_id: Uuid) -> Result<Vec<SavedRequest>, Error> {
        self.requests_svc.list(room_id)
    }

    pub fn create_room(&self, name: String, base_url: String) -> Result<Room, Error> {
        self.rooms_svc.create(name, base_url)
    }

    pub fn delete_room(&self, id: Uuid) -> Result<(), Error> {
        self.rooms_svc.delete(id)
    }

    pub fn set_room_base_url(&self, room_id: Uuid, base_url: String) -> Result<(), Error> {
        self.rooms_svc.set_base_url(room_id, base_url)
    }

    pub fn delete_request(&self, id: Uuid) -> Result<(), Error> {
        self.requests_svc.delete(id)
    }

    pub fn send_http(
        &self,
        draft: RequestDraft,
        base_url: &str,
    ) -> Result<ResponseSnapshot, Error> {
        self.rt
            .block_on(http::send(draft, base_url))
    }

    pub fn save_exchange(
        &self,
        room_id: Uuid,
        selected_id: Option<Uuid>,
        name: &str,
        draft: RequestDraft,
        snapshot: ResponseSnapshot,
    ) -> Result<SavedRequest, Error> {
        let current = self.requests_svc.list(room_id)?;
        self.requests_svc
            .create(room_id, selected_id, name, draft, snapshot, &current)
    }
}