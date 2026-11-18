pub static STORE_CREATE: &'static str = "
CREATE TABLE IF NOT EXISTS rooms (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL DEFAULT '',
    ms INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS history (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    ms INTEGER NOT NULL,
    draft TEXT NOT NULL,
    response TEXT NOT NULL
);";

pub static ROOMS_HAS_BASE_URL: &'static str =
    "SELECT COUNT(*) FROM pragma_table_info('rooms') WHERE name = 'base_url'";

pub static ROOMS_ADD_BASE_URL: &'static str =
    "ALTER TABLE rooms ADD COLUMN base_url TEXT NOT NULL DEFAULT ''";

pub static UPDATE_ROOM_BASE_URL: &'static str = "UPDATE rooms SET base_url = ?2 where id = ?1";

pub static HISTORY_HAS_ROOM: &'static str =
    "SELECT COUNT(*) FROM pragma_table_info('history') WHERE name = 'room_id'";

pub static HISTORY_ADD_ROOM: &'static str =
    "ALTER TABLE history ADD COLUMN room_id TEXT NOT NULL DEFAULT ''";

pub static FIRST_ROOM: &'static str = "SELECT id FROM rooms ORDER BY ms ASC LIMIT 1";

pub static INSERT_ROOM: &'static str =
    "INSERT INTO rooms (id, name, base_url, ms) VALUES (?1, ?2, ?3, ?4)";

pub static LIST_ROOMS: &'static str = "SELECT id, name, base_url, ms FROM rooms ORDER BY ms ASC";

pub static ATTACH_ORPHANS: &'static str = "UPDATE history SET room_id = ?1 WHERE room_id = ''";

pub static RETRIEVE_SAVED_REQ: &'static str = "SELECT id, name, ms, draft, response, room_id FROM history WHERE room_id = ?1 ORDER BY ms DESC";

pub static INSERT_REQ: &'static str =
    "INSERT INTO history (id, name, ms, draft, response, room_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

pub static UPDATE_REQ: &'static str =
    "UPDATE history SET name = ?2, ms = ?3, draft = ?4, response = ?5 WHERE id = ?1";

pub static DELETE_ROOM_REQUESTS: &'static str = "DELETE FROM history WHERE room_id = ?1";

pub static DELETE_ROOM: &'static str = "DELETE FROM rooms WHERE id = ?1";

pub static DELETE_REQUEST: &'static str = "DELETE FROM history WHERE id = ?1";
