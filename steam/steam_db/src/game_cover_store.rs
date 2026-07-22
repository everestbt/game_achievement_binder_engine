use rusqlite::{params, Connection, Result};

use db_lib::db_manager;

pub struct GameCover {
    pub app_id: i32,
    pub url: String,
}

pub fn get_game_cover(app_id: &i32) -> Result<Option<GameCover>> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT app_id, url FROM steam_custom_covers WHERE app_id = ?1")?;
    let mut iter = stmt.query_map([app_id], |row| {
        Ok(GameCover {
            app_id: row.get(0)?,
            url: row.get(1)?,
        })
    })?;

    if let Some(result) = iter.next() {
        match result {
            Ok(target) => Ok(Some(target)),
            Err(error) => Err(error)
        }
    }
    else {
        Ok(None)
    }
}

pub fn save_game_cover(url: &str, app_id: &i32) -> Result<()> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;
    
    // Add in the achievement
    conn.execute(
        "INSERT INTO steam_custom_covers (app_id, url) VALUES (?1, ?2)",
        params![app_id, url],
    )?;

    Ok(())
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_custom_covers (
            app_id INTEGER PRIMARY KEY,
            url TEXT NOT NULL
        )",
        [], // No parameters needed
    )?;

    Ok(())
}