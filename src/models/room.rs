use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::utils::now_ms;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub ms: u64,
}

impl Room {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            ms: now_ms(),
        }
    }
}
