use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::Error;
use crate::repository::sql;

pub type Db = Arc<Mutex<Connection>>;

pub fn open() -> Result<Db, Error> {
    let path = dirs::data_dir()
        .ok_or_else(|| Error::Database("data dir not found".into()))?
        .join("Orphey");

    std::fs::create_dir_all(&path).map_err(|err| Error::Database(err.to_string()))?;

    let conn = Connection::open(path.join("history.db"))
        .map_err(|err| Error::Database(err.to_string()))?;

    conn.execute_batch(sql::STORE_CREATE)
        .map_err(|err| Error::Database(err.to_string()))?;

    migrate(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

fn migrate(conn: &Connection) -> Result<(), Error> {
    let has_room: i64 = conn
        .query_row(sql::HISTORY_HAS_ROOM, [], |row| row.get(0))
        .map_err(|err| Error::Database(err.to_string()))?;

    let has_base_url: i64 = conn
        .query_row(sql::ROOMS_HAS_BASE_URL, [], |row| row.get(0))
        .map_err(|err| Error::Database(err.to_string()))?;

    if has_room == 0 {
        conn.execute_batch(sql::HISTORY_ADD_ROOM)
            .map_err(|err| Error::Database(err.to_string()))?;
    }

    if has_base_url == 0 {
        conn.execute_batch(sql::ROOMS_ADD_BASE_URL)
            .map_err(|err| Error::Database(err.to_string()))?;
    }

    let first_room: Option<String> = conn
        .query_row(sql::FIRST_ROOM, [], |row| row.get(0))
        .optional()
        .map_err(|err| Error::Database(err.to_string()))?;

    let Some(room_id) = first_room else {
        return Ok(());
    };

    conn.execute(sql::ATTACH_ORPHANS, params![room_id])
        .map(|_| ())
        .map_err(|err| Error::Database(err.to_string()))
}
