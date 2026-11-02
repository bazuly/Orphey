use crate::models::rest::SavedRequest;
use crate::models::room::Room;
use crate::{error::Error, infra};
use rusqlite::{Connection, params};
use uuid::Uuid;

pub struct HistoryStore {
    conn: Connection,
}

// TODO отедльный impl для работы с rooms и requests
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

        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn migrate(&self) -> Result<(), Error> {
        let has_room: i64 = self
            .conn
            .query_row(infra::sql::HISTORY_HAS_ROOM, [], |row| row.get(0))
            .map_err(|err| Error::Database(err.to_string()))?;

        if has_room == 0 {
            self.conn
                .execute_batch(infra::sql::HISTORY_ADD_ROOM)
                .map_err(|err| Error::Database(err.to_string()))?;
        }

        let count: i64 = self
            .conn
            .query_row(infra::sql::ROOM_COUNT, [], |row| row.get(0))
            .map_err(|err| Error::Database(err.to_string()))?;

        // если нет комнат - создаем ее
        let room_id = if count == 0 {
            let room = Room::new("General");
            self.insert_room(&room)?;
            room.id
        } else {
            self.list_rooms()?
                .into_iter()
                .next()
                .ok_or_else(|| Error::Database("rooms empty".into()))?
                .id
        };

        self.conn
            .execute(infra::sql::ATTACH_ORPHANS, params![room_id.to_string()])
            .map(|_| ())
            .map_err(|err| Error::Database(err.to_string()))
    }

    pub fn list_rooms(&self) -> Result<Vec<Room>, Error> {
        let mut stmt = self
            .conn
            .prepare(infra::sql::LIST_ROOMS)
            .map_err(|err| Error::Database(err.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)? as u64,
                ))
            })
            .map_err(|err| Error::Database(err.to_string()))?;

        rows.map(|row| {
            let (id, name, ms) = row.map_err(|err| Error::Database(err.to_string()))?;

            Ok(Room {
                id: uuid::Uuid::parse_str(&id).map_err(|err| Error::Database(err.to_string()))?,
                name,
                ms,
            })
        })
        .collect()
    }

    pub fn list_requests(&self, room_id: Uuid) -> Result<Vec<SavedRequest>, Error> {
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
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(|err| Error::Database(err.to_string()))?;

        rows.map(|row| {
            let (id, name, ms, draft, response, room_id) =
                row.map_err(|err| Error::Database(err.to_string()))?;

            Ok(SavedRequest {
                id: uuid::Uuid::parse_str(&id).map_err(|err| Error::Database(err.to_string()))?,
                room_id: uuid::Uuid::parse_str(&room_id)
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

    pub fn insert_request(&self, saved: &SavedRequest) -> Result<(), Error> {
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

    // TODO вот это тут не должно быть
    pub fn insert_room(&self, room: &Room) -> Result<(), Error> {
        self.conn
            .execute(
                infra::sql::INSERT_ROOM,
                params![room.id.to_string(), room.name, room.ms as i64],
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
                infra::sql::UPDATE_REQ,
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
