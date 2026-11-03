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

    pub fn insert(&self, room: &Room) -> Result<(), Error> {
        self.conn
            .execute(
                sql::INSERT_ROOM,
                params![room.id.to_string(), room.name, room.ms as i64],
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
