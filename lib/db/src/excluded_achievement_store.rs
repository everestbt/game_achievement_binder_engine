use rusqlite::{params, Connection, Result};

use db_lib::db_manager;

pub struct ExcludedAchievement {
    pub id: i32,
    pub achievement_name: String,
    pub app_id: i32,
}

pub fn get_excluded_achievements() -> Result<Vec<ExcludedAchievement>> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT id, achievement_name, app_id FROM excluded_steam_achievements")?;
    let achieve_iter = stmt.query_map([], |row| {
        Ok(ExcludedAchievement {
            id: row.get(0)?,
            achievement_name: row.get(1)?,
            app_id: row.get(2)?,
        })
    })?;

    let mut achievement_vec : Vec<ExcludedAchievement> = Vec::new();
    for d in achieve_iter {
        achievement_vec.push(d.unwrap());
    }
    Ok(achievement_vec)
}

pub fn get_excluded_achievements_for_app(app_id: &i32) -> Result<Vec<ExcludedAchievement>> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT id, achievement_name, app_id FROM excluded_steam_achievements WHERE app_id = ?1")?;
    let achieve_iter = stmt.query_map([app_id], |row| {
        Ok(ExcludedAchievement {
            id: row.get(0)?,
            achievement_name: row.get(1)?,
            app_id: row.get(2)?,
        })
    })?;

    let mut achievement_vec : Vec<ExcludedAchievement> = Vec::new();
    for d in achieve_iter {
        achievement_vec.push(d.unwrap());
    }
    Ok(achievement_vec)
}

pub fn save_excluded_achievement(achievement_name: &String, app_id: &i32) -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;
    
    // Add in the achievement
    conn.execute(
        "INSERT INTO excluded_steam_achievements (achievement_name, app_id) VALUES (?1, ?2)",
        params![achievement_name, app_id],
    )?;

    Ok(())
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS excluded_steam_achievements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            achievement_name TEXT NOT NULL,
            app_id INTEGER NOT NULL
        )",
        [], // No parameters needed
    )?;

    Ok(())
}