use rusqlite::{params, Connection, Result};
use db_lib::get_connection;
use std::sync::Once;

pub struct Achievement {
    pub name: String,
    pub achieved: bool,
}

struct AchievementShim {
    name: String,
    achieved: u8,
}

impl AchievementShim {
    fn to_pub(self) -> Achievement {
        Achievement { name: self.name, achieved: self.achieved == 1 }
    }
}

pub fn get_achievements() -> Result<Vec<Achievement>> {
    let conn: Connection = get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT name, achieved FROM mtga_achievements")?;
    let achieve_iter = stmt.query_map([], |row| {
        Ok(AchievementShim {
            name: row.get(0)?,
            achieved: row.get(1)?,
        })
    })?;

    let mut achievement_vec : Vec<Achievement> = Vec::new();
    for d in achieve_iter {
        achievement_vec.push(d?.to_pub());
    }
    Ok(achievement_vec)
}

pub fn save_achievement(name: &str, achieved: bool) -> Result<()> {
    let conn: Connection = get_connection();
    create_table(&conn)?;

    let achieved_int = if achieved {
        1
    }
    else {
        0
    };
    
    conn.execute(
        "INSERT INTO mtga_achievements (name, achieved) VALUES (?1, ?2) ON CONFLICT(name) DO UPDATE SET achieved=?3",
        params![name, achieved_int, achieved_int],
    )?;

    Ok(())
}

static TABLE_CREATE: Once = Once::new();

fn create_table(conn: &Connection) -> Result<()> {
    TABLE_CREATE.call_once(|| {
        if let Err(e) = conn.execute(
            "CREATE TABLE IF NOT EXISTS mtga_achievements (
                name TEXT PRIMARY KEY,
                achieved INTEGER NOT NULL
            )",
            [], // No parameters needed
        ){
            panic!("Table failed to create {}", e);
        }
    });

    Ok(())
}