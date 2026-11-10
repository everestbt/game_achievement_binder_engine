use rusqlite::{params, Connection, Result};
use std::sync::Once;
use db_lib::get_connection;

struct ExcludedGame {
    app_id : i32
}

pub fn save_excluded_game(app_id: &i32) -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = get_connection();
    create_table(&conn)?;
    
    // Add in the excluded game
    conn.execute(
        "INSERT INTO excluded_steam_games (app_id) VALUES (?1)",
        params![app_id],
    )?;

    Ok(())
}

pub fn remove_excluded_game(app_id: &i32) -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = get_connection();
    create_table(&conn)?;
    
    // Add in the excluded game
    conn.execute(
        "DELETE FROM excluded_steam_games WHERE app_id = ?1",
        params![app_id],
    )?;

    Ok(())
}

pub fn is_game_excluded(app_id: &i32) -> Result<bool> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = get_connection();
    create_table(&conn)?;
    
    // Add in the achievement
    let mut stmt = conn.prepare("SELECT app_id FROM excluded_steam_games WHERE app_id = ?1")?;
    let mut iter = stmt.query_map([app_id], |row| {
        Ok(ExcludedGame {
            app_id: row.get(0)?,
        })
    })?;

    Ok(iter.next().is_some())
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