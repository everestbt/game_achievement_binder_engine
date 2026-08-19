use rusqlite::{Connection};
use local_dir::get_local_dir;
use std::env;

static DATABASE_NAME: & str = "steam_randomiser_database.db";

pub fn get_connection() -> Connection {
    // Get a database prefix if set, used for testing
    let db_prefix = if let Ok(prefix) = env::var("GABE_DB_PREFIX") {
        Some(prefix)
    }
    else {
        None
    };
    
    let path = get_local_dir(&(db_prefix.unwrap_or("".to_string()) + DATABASE_NAME));
    let conn: Connection = Connection::open(path).expect("Failed to open a connection");
    conn
}