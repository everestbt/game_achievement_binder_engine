use rusqlite::{params, Connection, Result};

use db_lib::db_manager;

pub struct GameTarget {
    pub app_id: i32,
    pub complete: bool,
}

pub fn get_game_targets() -> Result<Vec<GameTarget>> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT app_id, complete FROM game_targets")?;
    let iter = stmt.query_map([], |row| {
        Ok(GameTarget {
            app_id: row.get(0)?,
            complete: row.get(1)?,
        })
    })?;

    let mut vec : Vec<GameTarget> = Vec::new();
    let mut error = None;
    for result in iter {
        match result {
            Ok(target) => vec.push(target),
            Err(e) =>  {
                error = Some(e);
                break
            },
        }
    }
    if let Some(e) = error {
        Err(e)
    }
    else {
        Ok(vec)
    }
}

pub fn get_game_target(app_id: &i32) -> Result<Option<GameTarget>> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT app_id, complete FROM game_targets WHERE app_id=?1 LIMIT 1")?;
    let mut iter = stmt.query_map([app_id], |row| {
        Ok(GameTarget {
            app_id: row.get(0)?,
            complete: row.get(1)?,
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

pub fn save_game_target(app_id: &i32, complete: &bool) -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;
    
    // Add in the achievement
    conn.execute(
        "INSERT INTO game_targets (app_id, complete) VALUES (?1, ?2) ON CONFLICT(app_id) DO UPDATE SET complete=?3",
        params![app_id, complete, complete],
    )?;

    Ok(())
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS game_targets (
            app_id INTEGER PRIMARY KEY,
            complete BOOL NOT NULL
        )",
        [], // No parameters needed
    )?;

    Ok(())
}