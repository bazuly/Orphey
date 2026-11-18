use std::rc::Rc;

use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::error::Error;
use crate::models::room::Room;
use crate::repository::sql;

pub struct RoomRepository {
    conn: Rc<Connection>,
}

impl RoomRepository {
    pub fn new(conn: Rc<Connection>) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> Result<Vec<Room>, Error> {
        let mut stmt = self
            .conn
            .prepare(sql::LIST_ROOMS)
            .map_err(|err| Error::Database(err.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)? as u64,
                ))
            })
            .map_err(|err| Error::Database(err.to_string()))?;

        rows.map(|row| {
            let (id, name, base_url, ms) = row.map_err(|err| Error::Database(err.to_string()))?;

            Ok(Room {
                id: uuid::Uuid::parse_str(&id).map_err(|err| Error::Database(err.to_string()))?,
                name,
                base_url,
                ms,
            })
        })
        .collect()
    }

    pub fn insert(&self, room: &Room) -> Result<(), Error> {
        self.conn
            .execute(
                sql::INSERT_ROOM,
                params![
                    room.id.to_string(),
                    room.name,
                    room.base_url,
                    room.ms as i64
                ],
            )
            .map(|_| ())
            .map_err(|err| Error::Database(err.to_string()))
    }

    pub fn update_base_url(&self, room_id: Uuid, base_url: &str) -> Result<(), Error> {
        self.conn
            .execute(
                sql::UPDATE_ROOM_BASE_URL,
                params![room_id.to_string(), base_url],
            )
            .map(|_| ())
            .map_err(|err| Error::Database(err.to_string()))
    }

    pub fn delete(&self, room_id: Uuid) -> Result<(), Error> {
        let id = room_id.to_string();
        self.conn
            .execute(sql::DELETE_ROOM_REQUESTS, params![id])
            .map(|_| ())
            .map_err(|err| Error::Database(err.to_string()))?;

        self.conn
            .execute(sql::DELETE_ROOM, params![id])
            .map(|_| ())
            .map_err(|err| Error::Database(err.to_string()))
    }
}
