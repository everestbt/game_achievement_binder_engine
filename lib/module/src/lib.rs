use std::collections::HashSet;

use steam_api::{
    game_fetch,
    achievement_fetch,
    game_cover_fetch,
};
use chrono::{DateTime, NaiveDate};
use steam_utils::goals;
use simple_error::{SimpleResult};
use bytes::Bytes;

/// A list of all available modules that are supported
#[derive(Debug, Clone)]
pub enum Module {
    STEAM(String, String), // key, Steam-id
    MTGA,
}

/// The generic interface for a game definition
#[derive(Debug, Clone)]
pub struct Game {
    pub module: Module,
    pub id: i32,
    pub name: String,
    pub playtime_forever: Option<u32>, // This is the number of minutes played
    pub last_played: NaiveDate,
}

pub async fn get_module_games(module: Module) -> Vec<Game> {
    match module {
        Module::STEAM(ref key, ref steam_id) => {
            game_fetch::get_owned_games(&key, &steam_id).await.iter().map(|g| {
                Game {
                    module: module.clone(),
                    id: g.appid,
                    name: g.name.clone(),
                    playtime_forever: Some(g.playtime_forever as u32),
                    last_played: DateTime::from_timestamp_secs(g.last_played).expect("Failed to provide a valid timestamp").date_naive()
                }
            })
            .collect()
        }
        Module::MTGA => todo!()
    }
}
/// Generic interface for achievements in games
#[derive(Debug, Clone)]
pub struct GameAchievement {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub achieved: bool,
    pub achieved_icon_id: Option<String>,
    pub unachieved_icon_id: Option<String>,
}

pub async fn get_random_achievement_for_game(module: Module, game_id: Option<i32>) -> Option<GameAchievement> {
    match module {
        Module::STEAM(key, steam_id) => {
            goals::get_random_achievement_for_game(&key, &steam_id, &game_id.expect("A game id must be provided for steam"))
                .await.map(|g| GameAchievement { id: g.name, display_name: g.display_name, description: g.description, achieved: false, achieved_icon_id: Some(g.icon), unachieved_icon_id: Some(g.icongray) })
        },
        Module::MTGA => todo!()
    }
}

pub async fn get_game_achievements(module: Module, game_id: Option<i32>) -> Vec<GameAchievement> {
    match module {
        Module::STEAM(key, steam_id) => {
            let app_id = &game_id.expect("A game id must be provided for steam");
            let achieved_set: HashSet<String> = if let Some(player) = achievement_fetch::get_player_achievements(&key, &steam_id, &app_id).await {
                player.achievements.iter()
                    .filter(|a| a.achieved == 1)
                    .map(|a| a.apiname.clone())
                    .collect()
            }
            else {
                HashSet::new()
            };  
            achievement_fetch::get_game_achievements(&key, &app_id).await
                .iter()
                .map(|g| GameAchievement { 
                    id: g.name.clone(), 
                    display_name: g.display_name.clone(), 
                    description: g.description.clone(), 
                    achieved: achieved_set.contains(&g.name), 
                    achieved_icon_id: Some(g.icon.clone()), 
                    unachieved_icon_id: Some(g.icongray.clone()) 
                })
                .collect()
            
        },
        Module::MTGA => todo!()
    }
}

pub enum GameCoverRequest {
    Steam(i32), // game_id
}

pub fn load_game_cover(request: GameCoverRequest) -> SimpleResult<Bytes> {
    match request {
        GameCoverRequest::Steam(id) => {
            game_cover_fetch::get_game_cover_blocking(&id)
        }
    }
}