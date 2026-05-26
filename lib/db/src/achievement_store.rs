use rusqlite::{params, Connection, Result};

use db_lib::db_manager;

pub struct Achievement {
    pub id: i32,
    pub achievement_name: String,
    pub display_name: String,
    pub app_id: i32,
    pub description: Option<String>,
    pub last_played: i64, 
}

pub fn get_achievement(id: &i32) -> Result<Achievement> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT id, achievement_name, display_name, description, app_id, last_played FROM steam_achievements_v_2 WHERE id = ?1")?;
    let mut achieve_iter = stmt.query_map([id], |row| {
        Ok(Achievement {
            id: row.get(0)?,
            achievement_name: row.get(1)?,
            display_name: row.get(2)?,
            description: row.get(3)?,
            app_id: row.get(4)?,
            last_played: row.get(5)?,
        })
    })?;
    let val = achieve_iter.next();
    val.expect("Id not found")
}

pub fn get_achievements() -> Result<Vec<Achievement>> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT id, achievement_name, display_name, description, app_id, last_played FROM steam_achievements_v_2")?;
    let achieve_iter = stmt.query_map([], |row| {
        Ok(Achievement {
            id: row.get(0)?,
            achievement_name: row.get(1)?,
            display_name: row.get(2)?,
            description: row.get(3)?,
            app_id: row.get(4)?,
            last_played: row.get(5)?,
        })
    })?;

    let mut achievement_vec : Vec<Achievement> = Vec::new();
    for d in achieve_iter {
        achievement_vec.push(d.unwrap());
    }
    Ok(achievement_vec)
}

pub fn get_achievements_for_app(app_id: &i32) -> Result<Vec<Achievement>> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT id, achievement_name, display_name, description, app_id, last_played FROM steam_achievements_v_2 WHERE app_id = ?1")?;
    let achieve_iter = stmt.query_map([app_id], |row| {
        Ok(Achievement {
            id: row.get(0)?,
            achievement_name: row.get(1)?,
            display_name: row.get(2)?,
            description: row.get(3)?,
            app_id: row.get(4)?,
            last_played: row.get(5)?,
        })
    })?;

    let mut achievement_vec : Vec<Achievement> = Vec::new();
    for d in achieve_iter {
        achievement_vec.push(d.unwrap());
    }
    Ok(achievement_vec)
}

pub fn save_achievement(achievement_name: &String, display_name: &String, description: &Option<String>, app_id: &i32, last_played: &i64) -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;
    
    // Add in the achievement
    conn.execute(
        "INSERT INTO steam_achievements_v_2 (achievement_name, display_name, description, app_id, last_played) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![achievement_name, display_name, description, app_id, last_played],
    )?;

    Ok(())
}

pub fn update_last_played(id: &i32, last_played: &i64) -> Result<()> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;
    
    conn.execute(
        "UPDATE steam_achievements_v_2 SET last_played = ?1 WHERE id = ?2 LIMIT 1",
        params![last_played, id],
    )?;

    Ok(())
}

pub fn delete_achievement(id: &i32) -> Result<()> {
    let conn: Connection = db_manager::get_connection();
    
    conn.execute(
        "DELETE FROM steam_achievements_v_2 WHERE id = ?1",
        params![id],
    )?;

    Ok(())
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_achievements_v_2 (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            achievement_name TEXT NOT NULL,
            display_name TEXT NOT NULL,
            app_id INTEGER NOT NULL,
            description TEXT,
            last_played INTEGER NOT NULL
        )",
        [], // No parameters needed
    )?;

    Ok(())
}