use rusqlite::{params, Connection, Result};

use db_lib::get_connection;

#[derive(Clone, Debug)]
pub struct GameCompletion {
    pub app_id: i32,
    pub achievements_completed: u32,
    pub last_played: i64,
    pub achievement_count: u32,
}

pub fn get_game_completion(app_id: &i32) -> Result<Option<GameCompletion>> {
    let conn: Connection = get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT app_id, achievements_completed, last_played, achievement_count FROM steam_game_completion WHERE app_id = ?1")?;
    let mut achieve_iter = stmt.query_map([app_id], |row| {
        Ok(GameCompletion {
            app_id: row.get(0)?,
            achievements_completed: row.get(1)?,
            last_played: row.get(2)?,
            achievement_count: row.get(3)?,
        })
    })?;
    if let Some(found) = achieve_iter.next() {
        if let Ok(result) = found {
            Ok(Some(result))
        }
        else {
            Err(found.unwrap_err())
        }
    }
    else {
        Ok(None)
    }
}

pub fn get_all_completions() -> Result<Vec<GameCompletion>> {
    let conn: Connection = get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT app_id, achievements_completed, last_played, achievement_count FROM steam_game_completion")?;
    let achieve_iter = stmt.query_map([], |row| {
        Ok(GameCompletion {
            app_id: row.get(0)?,
            achievements_completed: row.get(1)?,
            last_played: row.get(2)?,
            achievement_count: row.get(3)?,
        })
    })?;
    let mut vec : Vec<GameCompletion> = Vec::new();
    let mut error = None;
    for result in achieve_iter {
        match result {
            Ok(r) => vec.push(r),
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

// Returns all perfected games, which is where the achieved count equals the total count
// It does not return those with zero achievements as these can not be perfected
pub fn get_perfect_games() -> Result<Vec<GameCompletion>> {
    let conn: Connection = get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT app_id, achievements_completed, last_played, achievement_count FROM steam_game_completion WHERE achievements_completed = achievement_count AND achievement_count != 0")?;
    let achieve_iter = stmt.query_map([], |row| {
        Ok(GameCompletion {
            app_id: row.get(0)?,
            achievements_completed: row.get(1)?,
            last_played: row.get(2)?,
            achievement_count: row.get(3)?,
        })
    })?;

    let mut vec : Vec<GameCompletion> = Vec::new();
    for d in achieve_iter {
        vec.push(d.unwrap());
    }
    Ok(vec)
}

pub fn save_game_completion(app_id: &i32, achievements_completed: u32, last_played: i64, achievement_count: u32) -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = get_connection();
    create_table(&conn)?;
    
    // Add in the achievement
    conn.execute(
        "INSERT INTO steam_game_completion (app_id, achievements_completed, last_played, achievement_count) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(app_id) DO UPDATE SET achievements_completed=?5, last_played=?6, achievement_count=?7",
        params![app_id, achievements_completed, last_played, achievement_count, achievements_completed, last_played, achievement_count],
    )?;

    Ok(())
}

pub fn drop_table() -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = get_connection();

        conn.execute(
        "DROP TABLE IF EXISTS steam_game_completion",
        [], // No parameters needed
    )?;

    Ok(())
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_game_completion (
            app_id INTEGER PRIMARY KEY,
            achievements_completed INTEGER NOT NULL,
            last_played INTEGER NOT NULL,
            achievement_count INTEGER NOT NULL
        )",
        [], // No parameters needed
    )?;

    Ok(())
}