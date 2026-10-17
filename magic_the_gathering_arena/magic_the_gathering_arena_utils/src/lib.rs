use anyhow::Result;
use magic_the_gathering_arena_log_reader::{
    get_readable_achievements,
};
use magic_the_gathering_arena_db::achievement_store::{
    self, Achievement, save_achievement, Status
};
use jiff::Timestamp;
use local_dir::get_local_dir;
use preferences::{
    PreferencesMap, 
    Preferences,
};
use std::{
    collections::HashMap, 
    fs::{
        self, 
        File, 
        OpenOptions,
    }
};


pub const GAME_NAME : &str = "Magic the Gathering Arena";
pub const GAME_COVER_URL : &str = "https://cdn2.steamgriddb.com/thumb/96f03b40f10104906b823065df07e8c6.jpg";

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
        .map(from_db_format)
        .collect())
}

pub fn get_goals() -> Result<Vec<MTGAAchievement>> {
    Ok(achievement_store::get_goals()?
        .iter()
        .map(from_db_format)
        .collect())
}

pub fn save_goal(name: &str) -> Result<()> {
    achievement_store::save_goal(name)?;
    Ok(())
}

pub fn get_excluded_achievements() -> Result<Vec<MTGAAchievement>> {
    Ok(achievement_store::get_excluded()?
        .iter()
        .map(from_db_format)
        .collect())
}

pub fn save_excluded_achievement(name: &str) -> Result<()> {
    achievement_store::save_excluded(name)?;
    Ok(())
}

pub fn get_last_played_time() -> Option<Timestamp> {
    magic_the_gathering_arena_log_reader::get_last_played_time()
}

const SETTINGS_FILE: &str = "mtga_settings";

const URL_KEY: &str = "game_cover_url";

const TARGET_KEY: &str = "target_state";
const TARGET_STATE: &str = "TARGET";
const COMPLETE_STATE: &str = "COMPLETE";

fn read_local_settings() -> Result<PreferencesMap::<String>> {
    let path = get_local_dir(SETTINGS_FILE);
    if fs::exists(&path)? {
        let mut reader = OpenOptions::new()
            .read(true) // only needs read ability
            .open(path)?;
        Ok(PreferencesMap::<String>::load_from(&mut reader)?)
    }
    else {
        Ok(HashMap::new())
    }
}

fn write_local_settings(settings: HashMap<String, String>) -> Result<()> {
    let path = get_local_dir(SETTINGS_FILE);
    let mut writer = File::create(path)?;
    
    settings.save_to(&mut writer)?;
    Ok(())
}

pub fn set_game_cover_url(url: &str) -> Result<()> {
    let mut settings = read_local_settings()?;
    settings.insert(URL_KEY.into(), url.to_string());
    
    write_local_settings(settings)?;
    Ok(())
}

pub fn get_game_cover_url() -> Result<Option<String>> {
    let settings = read_local_settings()?;
    Ok(settings.get(URL_KEY).cloned())
}

pub fn set_as_target() -> Result<()> {
    let mut settings = read_local_settings()?;
    settings.insert(TARGET_KEY.into(), TARGET_STATE.to_string());

    write_local_settings(settings)?;
    Ok(())
}

pub fn set_as_complete() -> Result<()> {
    let mut settings = read_local_settings()?;
    settings.insert(TARGET_KEY.into(), COMPLETE_STATE.to_string());

    write_local_settings(settings)?;
    Ok(())
}

pub enum TargetState {
    TARGET,
    COMPLETE,
}

pub fn get_target_state() -> Result<Option<TargetState>> {
    let settings = read_local_settings()?;
    if let Some(target) = settings.get(TARGET_KEY) {
        match target.as_str() {
            TARGET_STATE => Ok(Some(TargetState::TARGET)),
            COMPLETE_STATE => Ok(Some(TargetState::COMPLETE)),
            _ => unreachable!("File has become corrupted")
        }
    }
    else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env;

    #[test]
    fn test_load_when_no_file_present() {
        // Use test directory incase a local database is present
        temp_env::with_var("GABE_DIR_SUFFIX", Some("test"), || {
            // First make sure the file doesn't exist
            let path = get_local_dir(SETTINGS_FILE);
            if fs::exists(&path).unwrap() {
                fs::remove_file(path).expect("Failed to delete ");
            }
            
            let settings = read_local_settings();
            assert!(settings.is_ok());
            assert!(settings.unwrap().is_empty());
        });
    }

    #[test]
    fn test_read_target_state_when_no_file() {
        // Use test directory incase a local database is present
        temp_env::with_var("GABE_DIR_SUFFIX", Some("test"), || {
            // First make sure the file doesn't exist
            let path = get_local_dir(SETTINGS_FILE);
            if fs::exists(&path).unwrap() {
                fs::remove_file(path).expect("Failed to delete ");
            }
            
            let state = get_target_state();
            assert!(state.is_ok());
            assert!(state.unwrap().is_none());
        });
    }
    
    #[test]
    fn test_read_game_cover_when_no_file() {
        // Use test directory incase a local database is present
        temp_env::with_var("GABE_DIR_SUFFIX", Some("test"), || {
            // First make sure the file doesn't exist
            let path = get_local_dir(SETTINGS_FILE);
            if fs::exists(&path).unwrap() {
                fs::remove_file(path).expect("Failed to delete ");
            }
            
            let state = get_game_cover_url();
            assert!(state.is_ok());
            assert!(state.unwrap().is_none());
        });
    }

    #[test]
    fn test_read_game_cover_when_target_set_but_no_cover() {
        // Use test directory incase a local database is present
        temp_env::with_var("GABE_DIR_SUFFIX", Some("test"), || {
            // First make sure the file doesn't exist
            let path = get_local_dir(SETTINGS_FILE);
            if fs::exists(&path).unwrap() {
                fs::remove_file(path).expect("Failed to delete ");
            }
            // Set the game as a target
            assert!(set_as_target().is_ok());

            let state = get_game_cover_url();
            assert!(state.is_ok());
            assert!(state.unwrap().is_none());
        });
    }
}