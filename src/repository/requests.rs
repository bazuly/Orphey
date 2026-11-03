use std::rc::Rc;

use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::error::Error;
use crate::models::request::SavedRequest;
use crate::repository::sql;

pub struct RequestRepository {
    conn: Rc<Connection>,
}

impl RequestRepository {
    pub fn new(conn: Rc<Connection>) -> Self {
        Self { conn }
    }

    pub fn list(&self, room_id: Uuid) -> Result<Vec<SavedRequest>, Error> {
        let mut stmt = self
            .conn
            .prepare(sql::RETRIEVE_SAVED_REQ)
            .map_err(|err| Error::Database(err.to_string()))?;

        let rows = stmt
            .query_map(params![room_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)? as u64,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(|err| Error::Database(err.to_string()))?;

        rows.map(|row| {
            let (id, name, ms, draft, response, room_id) =
                row.map_err(|err| Error::Database(err.to_string()))?;

            Ok(SavedRequest {
                id: Uuid::parse_str(&id).map_err(|err| Error::Database(err.to_string()))?,
                room_id: Uuid::parse_str(&room_id)
                    .map_err(|err| Error::Database(err.to_string()))?,
                name,
                ms,
                draft: serde_json::from_str(&draft)
                    .map_err(|err| Error::Database(err.to_string()))?,
                response: serde_json::from_str(&response)
                    .map_err(|err| Error::Database(err.to_string()))?,
            })
        })
        .collect()
    }

    pub fn insert(&self, saved: &SavedRequest) -> Result<(), Error> {
        let draft =
            serde_json::to_string(&saved.draft).map_err(|err| Error::Database(err.to_string()))?;
        let response = serde_json::to_string(&saved.response)
            .map_err(|err| Error::Database(err.to_string()))?;

        self.conn
            .execute(
                sql::INSERT_REQ,
                params![
                    saved.id.to_string(),
                    saved.name,
                    saved.ms as i64,
                    draft,
                    response,
                    saved.room_id.to_string()
                ],
            )
            .map(|_| ())
            .map_err(|err| Error::Database(err.to_string()))
    }

    pub fn update(&self, saved: &SavedRequest) -> Result<(), Error> {
        let draft =
            serde_json::to_string(&saved.draft).map_err(|err| Error::Database(err.to_string()))?;
        let response = serde_json::to_string(&saved.response)
            .map_err(|err| Error::Database(err.to_string()))?;

        self.conn
            .execute(
                sql::UPDATE_REQ,
                params![
                    saved.id.to_string(),
                    saved.name,
                    saved.ms as i64,
                    draft,
                    response
                ],
            )
            .map(|_| ())
            .map_err(|err| Error::Database(err.to_string()))
    }
}
