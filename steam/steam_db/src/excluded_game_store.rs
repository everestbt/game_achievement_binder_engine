use rusqlite::{params, Connection, Result};
use std::sync::Once;
use db_lib::get_connection;

pub fn save_excluded_game(app_id: &i32) -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = get_connection();
    create_table(&conn)?;
    
    // Add in the achievement
    conn.execute(
        "INSERT INTO excluded_steam_achievements (app_id) VALUES (?1)",
        params![app_id],
    )?;

    Ok(())
}

static TABLE_CREATE: Once = Once::new();

fn create_table(conn: &Connection) -> Result<()> {
    TABLE_CREATE.call_once(|| {
        if let Err(e) = conn.execute(
            "CREATE TABLE IF NOT EXISTS excluded_steam_games (
                app_id INTEGER PRIMARY KEY
            )",
            [], // No parameters needed
        ){
            panic!("Table failed to create {}", e);
        }
    });

    Ok(())
}