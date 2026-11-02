use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::utils::now_ms;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

impl Method {
    pub const ALL: [Self; 5] = [Self::GET, Self::POST, Self::PUT, Self::PATCH, Self::DELETE];

    pub fn body_template(self) -> &'static str {
        match self {
            Method::GET => "{\n  \"id\": 1\n}",
            Method::POST => "{\n  \"name\": \"Ada\",\n  \"email\": \"ada@example.com\"\n}",
            Method::PUT => {
                "{\n  \"id\": 1,\n  \"name\": \"Ada\",\n  \"email\": \"ada@example.com\"\n}"
            }
            Method::PATCH => "{\n  \"name\": \"Ada Lovelace\"\n}",
            Method::DELETE => "{\n  \"id\": 1\n}",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestDraft {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl RequestDraft {
    pub fn apply_method_template(mut self, previous: Method) {
        let previous_template = previous.body_template();
        if self.body.trim().is_empty() || self.body == previous_template {
            self.body = self.method.body_template().to_string();
        }
    }
}

impl Default for RequestDraft {
    fn default() -> Self {
        Self {
            method: Method::GET,
            url: String::new(),
            headers: vec![],
            body: Method::GET.body_template().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSnapshot {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub elapsed_ms: u64,
}

// для истории запросов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRequest {
    pub id: Uuid,
    pub room_id: Uuid,
    pub name: String,
    pub draft: RequestDraft,
    pub response: ResponseSnapshot,
    pub ms: u64,
}

impl SavedRequest {
    pub fn from_exchange(room_id: Uuid, draft: RequestDraft, response: ResponseSnapshot) -> Self {
        let name = format!("{:?} {}", draft.method, draft.url);

        Self {
            id: Uuid::new_v4(),
            room_id,
            name,
            draft,
            response,
            ms: now_ms(),
        }
    }

    pub fn refresh(&mut self, draft: RequestDraft, response: ResponseSnapshot) {
        self.draft = draft;
        self.response = response;
        self.ms = now_ms();
    }
}
