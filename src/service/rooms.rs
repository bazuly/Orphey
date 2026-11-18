use uuid::Uuid;

use crate::error::Error;
use crate::models::room::Room;
use crate::repository::rooms::RoomRepository;

pub struct RoomService {
    repo: Option<RoomRepository>,
}

impl RoomService {
    pub fn new(repo: Option<RoomRepository>) -> Self {
        Self { repo }
    }

    pub fn list(&self) -> Result<Vec<Room>, Error> {
        match &self.repo {
            Some(repo) => repo.list(),
            None => Ok(Vec::new()),
        }
    }

    pub fn create(&self, name: String, base_url: String) -> Result<Room, Error> {
        let mut room = Room::new(name);
        room.base_url = base_url.trim().to_string();

        if let Some(repo) = &self.repo {
            repo.insert(&room)?;
        }
        Ok(room)
    }

    pub fn delete(&self, id: Uuid) -> Result<(), Error> {
        match &self.repo {
            Some(repo) => repo.delete(id),
            None => Ok(()),
        }
    }

    pub fn set_base_url(&self, room_id: Uuid, base_url: String) -> Result<(), Error> {
        match &self.repo {
            Some(repo) => repo.update_base_url(room_id, &base_url),
            None => Ok(()),
        }
    }
}
