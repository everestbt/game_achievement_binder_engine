use anyhow::Result;
use magic_the_gathering_arena_log_reader::{
    get_readable_achievements,
};
use magic_the_gathering_arena_db::achievement_store::{
    self, Achievement, save_achievement, Status
};
use jiff::Timestamp;

pub const GAME_NAME : &str = "Magic the Gathering Arena";
pub const ID : i32 = 0;

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct MTGAAchievement {
    pub name: String, 
    pub achieved: bool,
}

fn from_db_format(a: &Achievement) -> MTGAAchievement {
    MTGAAchievement { name: a.name.clone(), achieved: a.status == Status::Achieved }
}

pub fn sync_achievements() -> Result<()> {
    let achievements = get_readable_achievements();

    for a in achievements {
        save_achievement(&a.name, a.achieved)?;
    }
    
    Ok(())
}

pub fn get_achievements() -> Result<Vec<MTGAAchievement>> {
    Ok(achievement_store::get_achievements()?
        .iter()
        .map(|a| from_db_format(a))
        .collect())
}

pub fn get_goals() -> Result<Vec<MTGAAchievement>> {
    Ok(achievement_store::get_goals()?
        .iter()
        .map(|a| from_db_format(a))
        .collect())
}

pub fn save_goal(name: String) -> Result<()> {
    achievement_store::save_goal(&name)?;
    Ok(())
}

pub fn get_excluded_achievements() -> Result<Vec<MTGAAchievement>> {
    Ok(achievement_store::get_excluded()?
        .iter()
        .map(|a| from_db_format(a))
        .collect())
}

pub fn get_last_played_time() -> Option<Timestamp> {
    magic_the_gathering_arena_log_reader::get_last_played_time()
}