pub mod game_cover;
pub mod game_targets;
pub mod achievements;

use steam_api::game_fetch;
use jiff::Timestamp;
use steam_utils::{
    goals,
    last_played_converter_to_timestamp,
};
use simple_error::SimpleResult;
use preferences::{PreferencesMap, Preferences};
use local_dir::get_local_dir;
use std::fs::File;
use anyhow::Result;
use std::env;

/// A list of all available modules that are supported
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Module {
    STEAM(SteamCredentials),
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
    pub last_played: Timestamp,
}

const STEAM_ID_KEY: &str = "steam_id";

pub enum ModuleEnable {
    STEAM(SteamEnable)
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
    let mut settings: PreferencesMap<String> = PreferencesMap::new();
    match module {
        ModuleEnable::STEAM(steam_enable) => {
            settings.insert(STEAM_ID_KEY.into(), steam_enable.steam_id.into());
        }
    }

    let path = get_local_dir("settings");
    let mut writer = File::create(path)?;
    
    settings.save_to(&mut writer)?;
    Ok(())
}

pub fn get_modules() -> Result<Vec<Module>> {
    let path = get_local_dir("settings");
    let mut reader = File::open(path)?;
    let settings = PreferencesMap::<String>::load_from(&mut reader)?;

    let mut modules = vec![];
    if let Some(id) = settings.get(STEAM_ID_KEY) {
        let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
        let crendentials = SteamCredentials{key, steam_id: id.clone()};
        modules.push(Module::STEAM(crendentials));
    }
    Ok(modules)
}

pub async fn get_module_games(module: Module) -> Vec<Game> {
    match module {
        Module::STEAM(ref credentials) => {
            game_fetch::get_owned_games(&credentials.key, &credentials.steam_id).await.iter().map(|g| {
                Game {
                    identifier: GameIdentifier { module: module.clone(), id: g.appid },
                    name: g.name.clone(),
                    playtime_forever: Some(g.playtime_forever as u32),
                    last_played: last_played_converter_to_timestamp(g.last_played)
                }
            })
            .collect()
        }
    }
}

pub async fn sync_caches(modules: Vec<Module>) -> SimpleResult<()> {
    for m in modules {
        match m {
            Module::STEAM(credentials) => {
                goals::sync_caches(&credentials.key, &credentials.steam_id).await;
            }
        }
    }
    Ok(())
}