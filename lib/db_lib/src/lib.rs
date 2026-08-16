use rusqlite::{Connection};
use local_dir::get_local_dir;

static DATABASE_NAME: & str = "steam_randomiser_database.db";

pub fn get_connection() -> Connection {
    let path = get_local_dir(DATABASE_NAME);
    let conn: Connection = Connection::open(path).expect("Failed to open a connection");
    conn
}