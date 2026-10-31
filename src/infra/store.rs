use crate::models::rest::SavedRequest;
use crate::{error::Error, infra};
use rusqlite::{Connection, params};

pub struct HistoryStore {
    conn: Connection,
}

impl HistoryStore {
    pub fn open() -> Result<Self, Error> {
        let path = dirs::data_dir()
            .ok_or_else(|| Error::Database("data dir not found".into()))?
            .join("Orphey");

        std::fs::create_dir_all(&path).map_err(|err| Error::Database(err.to_string()))?;

        let conn = Connection::open(path.join("history.db"))
            .map_err(|err| Error::Database(err.to_string()))?;

        conn.execute_batch(infra::sql::STORE_CREATE)
            .map_err(|err| Error::Database(err.to_string()))?;

        Ok(Self { conn })
    }

    pub fn list(&self) -> Result<Vec<SavedRequest>, Error> {
        let mut stmt = self
            .conn
            .prepare(infra::sql::RETRIEVE_SAVED_REQ)
            .map_err(|err| Error::Database(err.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)? as u64,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|err| Error::Database(err.to_string()))?;

        rows.map(|row| {
            let (id, name, ms, draft, response) =
                row.map_err(|err| Error::Database(err.to_string()))?;

            Ok(SavedRequest {
                id: uuid::Uuid::parse_str(&id).map_err(|err| Error::Database(err.to_string()))?,
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
                infra::sql::INSERT_REQ,
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
