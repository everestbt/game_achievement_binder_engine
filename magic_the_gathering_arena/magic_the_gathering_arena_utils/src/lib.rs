use anyhow::Result;
use magic_the_gathering_arena_log_reader::get_readable_achievements;
use magic_the_gathering_arena_db::achievement_store::save_achievement;

pub const GAME_NAME : &str = "Magic the Gathering Arena";
pub const ID : i32 = 0;

pub fn sync_achievements() -> Result<()> {
    let achievements = get_readable_achievements();

    for a in achievements {
        save_achievement(&a.name, a.achieved)?;
    }
    
    Ok(())
}