pub static STORE_CREATE: &'static str = "CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                ms INTEGER NOT NULL,
                draft TEXT NOT NULL,
                response TEXT NOT NULL
            );";

pub static RETRIEVE_SAVED_REQ: &'static str =
    "SELECT id, name, ms, draft, response FROM history ORDER BY ms DESC";

pub static INSERT_REQ: &'static str =
    "INSERT INTO history (id, name, ms, draft, response) VALUES (?1, ?2, ?3, ?4, ?5)";
