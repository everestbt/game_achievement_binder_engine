pub mod game_cover;
pub mod game_targets;
pub mod achievements;

use steam_api::game_fetch;
use jiff::Timestamp;
use steam_utils::{
    goals,
    last_played_converter_to_timestamp,
};
use simple_error::{
    SimpleResult,
    SimpleError,
};
use preferences::{PreferencesMap, Preferences};
use local_dir::get_local_dir;
use std::fs::File;
use anyhow::Result;
use std::env;
use std::collections::HashMap;

/// A list of all available modules that are supported
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Module {
    STEAM(SteamCredentials),
    MTGA,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SteamCredentials {
    key: String,
    steam_id: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GameIdentifier {
    pub module: Module,
    pub id: i32,
}

/// The generic interface for a game definition
#[derive(Debug, Clone)]
pub struct Game {
    pub identifier: GameIdentifier,
    pub name: String,
    pub playtime_forever: Option<u32>, // This is the number of minutes played
    pub last_played: Option<Timestamp>,
}

const STEAM_ID_KEY: &str = "steam_id";
const MTGA_KEY: &str = "MTGA";

pub enum ModuleEnable {
    STEAM(SteamEnable),
    MTGA,
}

pub struct SteamEnable {
    steam_id : String,
}

impl SteamEnable {
    pub fn new(steam_id: String) -> Self {
        SteamEnable { steam_id }
    }
}

pub fn enable_module(module: ModuleEnable) -> Result<()> {
    let mut settings = read_module_settings()?;
    match module {
        ModuleEnable::STEAM(steam_enable) => {
            settings.insert(STEAM_ID_KEY.into(), steam_enable.steam_id);
        },
        ModuleEnable::MTGA => {
            settings.insert(MTGA_KEY.into(), "Enable".to_string());
        }
    }

    let path = get_local_dir("settings");
    let mut writer = File::create(path)?;
    
    settings.save_to(&mut writer)?;
    Ok(())
}

pub fn get_modules() -> Result<Vec<Module>> {
    let settings = read_module_settings()?;

    let mut modules = vec![];
    if let Some(id) = settings.get(STEAM_ID_KEY) {
        let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
        let crendentials = SteamCredentials{key, steam_id: id.clone()};
        modules.push(Module::STEAM(crendentials));
    }
    if let Some(_) = settings.get(MTGA_KEY) {
        modules.push(Module::MTGA);
    }
    Ok(modules)
}

fn read_module_settings() -> Result<PreferencesMap::<String>> {
    let path = get_local_dir("settings");
    let mut reader = File::open(path)?;
    Ok(PreferencesMap::<String>::load_from(&mut reader)?)
}

pub async fn get_module_games(module: Module) -> Vec<Game> {
    match module {
        Module::STEAM(ref credentials) => {
            game_fetch::get_owned_games(&credentials.key, &credentials.steam_id).await.iter().map(|g| {
                Game {
                    identifier: GameIdentifier { module: module.clone(), id: g.appid },
                    name: g.name.clone(),
                    playtime_forever: Some(g.playtime_forever as u32),
                    last_played: Some(last_played_converter_to_timestamp(g.last_played))
                }
            })
            .collect()
        },
        Module::MTGA => {
            vec![Game{
                name : magic_the_gathering_arena_utils::GAME_NAME.to_string(),
                identifier : GameIdentifier { module: module.clone(), id: magic_the_gathering_arena_utils::ID },
                playtime_forever : None,
                last_played: magic_the_gathering_arena_utils::get_last_played_time(),
            }]
        }
    }
}

pub async fn sync_caches(modules: Vec<Module>) -> SimpleResult<()> {
    let mut err = None;
    for m in modules {
        match m {
            Module::STEAM(credentials) => {
                goals::sync_caches(&credentials.key, &credentials.steam_id).await;
            },
            Module::MTGA => {
                if magic_the_gathering_arena_utils::sync_achievements().is_err() {
                    err = Some(SimpleError::new("Failed to sync MTGA achievements"))
                }
            }
        }
    }
    if let Some(error) = err {
        Err(error)
    }
    else {
        Ok(())
    }
}

pub struct GameCompletionStatus {
    pub complete: bool,
    pub perfect: bool,
    pub progress: AchievementProgress,
}

#[derive(Default)]
pub struct AchievementProgress {
    pub total: u32,
    pub unlocked: u32,
    pub excluded: u32,
}

impl AchievementProgress {
    pub fn get_progress(&self) -> i8 {
        (100.0 * (((self.unlocked + self.excluded) as f32) / (self.total as f32))) as i8
    }
}

pub fn get_modules_progress(modules: &Vec<Module>) -> Result<HashMap<GameIdentifier, GameCompletionStatus>> {
    let mut map = HashMap::new();
    for m in modules {
        match m {
            Module::STEAM(_) => {
                let completetion = goals::get_game_completion();
                let progress = goals::get_game_progress();
                for k in completetion.keys() {
                    let complete_val = completetion.get(&k);
                    let progress_val = progress.get(&k);
                    map.insert(GameIdentifier { module: m.clone(), id: *k }, GameCompletionStatus { 
                        complete: complete_val.map(|c| c.complete).unwrap_or(false), 
                        perfect: complete_val.map(|c| c.perfect).unwrap_or(false), 
                        progress: {
                            if let Some(p) = progress_val {
                                AchievementProgress { total: p.total, unlocked: p.unlocked, excluded: p.excluded }
                            }
                            else {
                                AchievementProgress::default()
                            }
                        } });
                }
                for k in progress.keys() {
                    if !map.contains_key(&GameIdentifier { module: m.clone(), id: *k }) {
                        let complete_val = completetion.get(&k);
                        let progress_val = progress.get(&k);
                        map.insert(GameIdentifier { module: m.clone(), id: *k }, GameCompletionStatus { 
                            complete: complete_val.map(|c| c.complete).unwrap_or(false), 
                            perfect: complete_val.map(|c| c.perfect).unwrap_or(false), 
                            progress: {
                                if let Some(p) = progress_val {
                                    AchievementProgress { total: p.total, unlocked: p.unlocked, excluded: p.excluded }
                                }
                                else {
                                    AchievementProgress::default()
                                }
                            } });
                    }
                }
            },
            Module::MTGA => {
                let achievements = magic_the_gathering_arena_utils::get_achievements()?;
                let complete = achievements.iter().find(|a| !a.achieved).is_none();
                let achieved = achievements.iter().filter(|a| a.achieved).count();
                let excluded = magic_the_gathering_arena_utils::get_excluded_achievements()?.len();
                map.insert(
                    GameIdentifier { module: m.clone(), id: magic_the_gathering_arena_utils::ID }, 
                    GameCompletionStatus { 
                        complete: complete, 
                        perfect: complete, 
                        progress: AchievementProgress { 
                            total: achievements.len() as u32, 
                            unlocked: achieved as u32, 
                            excluded: excluded as u32,
                        } 
                    }
                    );
            }
        }
    }
    Ok(map)
}