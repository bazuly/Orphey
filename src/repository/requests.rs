use std::sync::{MutexGuard};

use uuid::Uuid;

use crate::error::Error;
use crate::models::request::SavedRequest;
use crate::repository::db::Db;
use crate::repository::sql;
use rusqlite::params;

pub struct RequestRepository {
    conn: Db,
}

impl RequestRepository {
    pub fn new(conn: Db) -> Self {
        Self { conn }
    }

    fn lock_conn(&self) -> Result<MutexGuard<'_, rusqlite::Connection>, Error> {
        self.conn
            .lock()
            .map_err(|_| Error::Database("database lock poisoned".into()))
    }

    pub fn list(&self, room_id: Uuid) -> Result<Vec<SavedRequest>, Error> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
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
        let conn = self.lock_conn()?;
        let draft =
            serde_json::to_string(&saved.draft).map_err(|err| Error::Database(err.to_string()))?;
        let response = serde_json::to_string(&saved.response)
            .map_err(|err| Error::Database(err.to_string()))?;

        conn.execute(
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
        let conn = self.lock_conn()?;
        let draft =
            serde_json::to_string(&saved.draft).map_err(|err| Error::Database(err.to_string()))?;
        let response = serde_json::to_string(&saved.response)
            .map_err(|err| Error::Database(err.to_string()))?;

        conn.execute(
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

    pub fn delete(&self, request_id: Uuid) -> Result<(), Error> {
        let conn = self.lock_conn()?;
        let id = request_id.to_string();
        conn.execute(sql::DELETE_REQUEST, params![id])
            .map(|_| ())
            .map_err(|err| Error::Database(err.to_string()))
    }
}