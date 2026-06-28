use rusqlite::{params, Connection, Result}; // For database operations and result handling

use db_lib::db_manager;

struct Id {
    id: String,
}

pub fn get_id() -> Result<String> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;
    
    let mut stmt = conn.prepare("SELECT steam_id FROM steam_id_store")?;
    let mut result = stmt.query_map([], |row| {
        Ok(Id {
            id: row.get(0)?
        })
    })?;

    let id: String = result.next()
        .expect("Failed to load the id, use --id at least once")
        .expect("Failed to load the id")
        .id;
    Ok(id)
}

pub fn save_id(id: &String) -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;
    
    // Clear the table
    conn.execute(
        "DELETE FROM steam_id_store",
        [], // No parameters needed
    )?;

    // Add in the id
    conn.execute(
        "INSERT INTO steam_id_store (steam_id) VALUES (?1)",
        params![id],
    )?;

    Ok(())
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_id_store (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            steam_id TEXT NOT NULL
        )",
        [], // No parameters needed
    )?;

    Ok(())
}