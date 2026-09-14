use rusqlite::{params, Connection, Result};
use db_lib::get_connection;
use std::sync::Once;

pub struct Achievement {
    pub name: String,
    pub status: Status,
}

#[derive(Eq, PartialEq)]
pub enum Status {
    Achieved,
    Unachieved,
    Target,
    Excluded,
}

struct AchievementShim {
    name: String,
    achieved: u8,
    target: Option<i8>
}

impl AchievementShim {
    fn to_pub(self) -> Achievement {
        Achievement { name: self.name, status: {
            if self.achieved == 1 {
                Status::Achieved
            }
            else if let Some(tar) = self.target {
                match tar {
                    1 => Status::Target,
                    -1 => Status::Excluded,
                    _ => unreachable!("Should never hit this value")
                }
            }
            else {
                Status::Unachieved
            }
        } }
    }
}

pub fn get_achievements() -> Result<Vec<Achievement>> {
    let conn: Connection = get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT name, achieved, target FROM mtga_achievements")?;
    let achieve_iter = stmt.query_map([], |row| {
        Ok(AchievementShim {
            name: row.get(0)?,
            achieved: row.get(1)?,
            target: row.get(2)?,
        })
    })?;

    let mut achievement_vec : Vec<Achievement> = Vec::new();
    for d in achieve_iter {
        achievement_vec.push(d?.to_pub());
    }
    Ok(achievement_vec)
}

pub fn get_goals() -> Result<Vec<Achievement>> {
    filter_by_target(TargetFilter::Goal)
}

pub fn get_excluded() -> Result<Vec<Achievement>> {
    filter_by_target(TargetFilter::Excluded)
}

enum TargetFilter {
    Goal,
    Excluded
}

fn filter_by_target(filter: TargetFilter) -> Result<Vec<Achievement>> {
    let conn: Connection = get_connection();
    create_table(&conn)?;

    let target_value = match filter {
        TargetFilter::Excluded => -1,
        TargetFilter::Goal => 1,
    };

    let mut stmt = conn.prepare("SELECT name, achieved, target FROM mtga_achievements WHERE target = ?1")?;
    let achieve_iter = stmt.query_map([target_value], |row| {
        Ok(AchievementShim {
            name: row.get(0)?,
            achieved: row.get(1)?,
            target: row.get(2)?,
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

pub fn save_goal(name: &str) -> Result<()> {
    let conn: Connection = get_connection();
    create_table(&conn)?;
    
    conn.execute(
        "UPDATE SET target = 1 WHERE name = ?1",
        params![name],
    )?;

    Ok(())
}

pub fn save_excluded(name: &str) -> Result<()> {
    let conn: Connection = get_connection();
    create_table(&conn)?;
    
    conn.execute(
        "UPDATE SET target = -1 WHERE name = ?1",
        params![name],
    )?;

    Ok(())
}

static TABLE_CREATE: Once = Once::new();

fn create_table(conn: &Connection) -> Result<()> {
    TABLE_CREATE.call_once(|| {
        if let Err(e) = conn.execute(
            "CREATE TABLE IF NOT EXISTS mtga_achievements (
                name TEXT PRIMARY KEY,
                achieved INTEGER NOT NULL,
                target INTEGER
            )",
            [], // No parameters needed
        ){
            panic!("Table failed to create {}", e);
        }
    });

    Ok(())
}