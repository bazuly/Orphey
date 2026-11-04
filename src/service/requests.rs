use uuid::Uuid;

use crate::error::Error;
use crate::models::request::{RequestDraft, ResponseSnapshot, SavedRequest};
use crate::repository::requests::RequestRepository;
use crate::utils::now_ms;

pub struct RequestService {
    repo: Option<RequestRepository>,
}

impl RequestService {
    pub fn new(repo: Option<RequestRepository>) -> Self {
        Self { repo }
    }

    pub fn list(&self, room_id: Uuid) -> Result<Vec<SavedRequest>, Error> {
        match &self.repo {
            Some(repo) => repo.list(room_id),
            None => Ok(Vec::new()),
        }
    }

    pub fn create(
        &self,
        room_id: Uuid,
        selected_id: Option<Uuid>,
        name: &str,
        draft: RequestDraft,
        snapshot: ResponseSnapshot,
        current: &[SavedRequest],
    ) -> Result<SavedRequest, Error> {
        let mut saved = match selected_id {
            Some(id) => {
                let mut saved = current
                    .iter()
                    .find(|saved| saved.id == id)
                    .cloned()
                    .ok_or_else(|| Error::Database("Selected data not found".into()))?;
                refresh(&mut saved, draft, snapshot);
                saved
            }
            None => from_exchange(room_id, draft, snapshot),
        };

        let trimmed = name.trim();
        if !trimmed.is_empty() {
            saved.name = trimmed.to_string();
        }

        if let Some(repo) = &self.repo {
            match selected_id {
                Some(_) => repo.update(&saved)?,
                None => repo.insert(&saved)?,
            }
        }

        Ok(saved)
    }

    pub fn delete(&self, request_id: Uuid) -> Result<(), Error> {
        match &self.repo {
            Some(repo) => repo.delete(request_id),
            None => Ok(()),
        }
    }
}

fn from_exchange(room_id: Uuid, draft: RequestDraft, response: ResponseSnapshot) -> SavedRequest {
    SavedRequest {
        id: Uuid::new_v4(),
        room_id,
        name: format!("{:?} {}", draft.method, draft.url),
        draft,
        response,
        ms: now_ms(),
    }
}

fn refresh(saved: &mut SavedRequest, draft: RequestDraft, response: ResponseSnapshot) {
    saved.draft = draft;
    saved.response = response;
    saved.ms = now_ms();
}
