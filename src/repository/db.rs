use std::rc::Rc;

use rusqlite::{Connection, OptionalExtension, params};

use crate::error::Error;
use crate::models::room::Room;
use crate::repository::sql;

pub fn open() -> Result<Rc<Connection>, Error> {
    let path = dirs::data_dir()
        .ok_or_else(|| Error::Database("data dir not found".into()))?
        .join("Orphey");

    std::fs::create_dir_all(&path).map_err(|err| Error::Database(err.to_string()))?;

    let conn = Connection::open(path.join("history.db"))
        .map_err(|err| Error::Database(err.to_string()))?;

    conn.execute_batch(sql::STORE_CREATE)
        .map_err(|err| Error::Database(err.to_string()))?;

    migrate(&conn)?;
    Ok(Rc::new(conn))
}

fn migrate(conn: &Connection) -> Result<(), Error> {
    let has_room: i64 = conn
        .query_row(sql::HISTORY_HAS_ROOM, [], |row| row.get(0))
        .map_err(|err| Error::Database(err.to_string()))?;

    if has_room == 0 {
        conn.execute_batch(sql::HISTORY_ADD_ROOM)
            .map_err(|err| Error::Database(err.to_string()))?;
    }

    let first_room: Option<String> = conn
        .query_row(sql::FIRST_ROOM, [], |row| row.get(0))
        .optional()
        .map_err(|err| Error::Database(err.to_string()))?;

    // если нет комнат - создаем ее
    let room_id = match first_room {
        Some(id) => id,
        None => {
            let room = Room::new("General");
            conn.execute(
                sql::INSERT_ROOM,
                params![room.id.to_string(), room.name, room.ms as i64],
            )
            .map_err(|err| Error::Database(err.to_string()))?;
            room.id.to_string()
        }
    };

    conn.execute(sql::ATTACH_ORPHANS, params![room_id])
        .map(|_| ())
        .map_err(|err| Error::Database(err.to_string()))
}
