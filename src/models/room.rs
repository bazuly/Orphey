use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::now_ms;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub base_url: String,
    pub ms: u64,
}

impl Room {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            base_url: String::new(),
            ms: now_ms(),
        }
    }
}
