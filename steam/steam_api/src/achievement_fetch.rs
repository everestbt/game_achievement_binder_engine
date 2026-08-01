use serde::{Deserialize, Serialize};
use steam_db::request_store;
use anyhow::Result;
use reqwest::Client;

// Player Achievements Request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerAchievement {
    pub apiname: String,
    pub achieved: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerAchievements {
    pub achievements: Vec<PlayerAchievement>,
    #[serde(rename = "gameName")]
    pub game_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerAchievementsInternal {
    achievements: Option<Vec<PlayerAchievement>>,
    #[serde(rename = "gameName")]
    game_name: Option<String>,
    success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerStatsResponse {
    playerstats: PlayerAchievementsInternal,
}

pub async fn get_player_achievements(key : &str, steam_id : &str, app_id : &i32) -> Result<Option<PlayerAchievements>> {
    let get_player_achievements_request: String = "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/?".to_owned()
        + "&key=" + key + "&steamid=" + steam_id
        + "&appid=" + &app_id.to_string();

    if !request_store::increment().unwrap() {
        panic!("Hit request limit, wait until tomorrow");
    }
    let response: PlayerStatsResponse = Client::new()
        .get(get_player_achievements_request)
        .send()
        .await?
        .json()
        .await?;
    // Success code can be true but no achievements present, typically for more modern games (Dota 2 is an example app_id 570)
    if response.playerstats.success && let Some(a) = response.playerstats.achievements && let Some(n) = response.playerstats.game_name {
        Ok(Some(PlayerAchievements {
            achievements: a,
            game_name: n,
        }))
    }
    else {
        Ok(None)
    }
}

// Game Schema request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameAchievement {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: Option<String>,
    pub icon: String,
    pub icongray: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameStats {
    achievements: Vec<GameAchievement>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AvailableGameStats {
    #[serde(rename = "availableGameStats")]
    available_game_stats: Option<GameStats>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameSchemaResponse {
    game: AvailableGameStats,
}

pub async fn get_game_achievements(key : &str, app_id : &i32) -> Result<Vec<GameAchievement>> {
    let get_schema_for_game_request: String =
        "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/?key=".to_owned() + key + "&appid=" + &app_id.to_string();

    if !request_store::increment().unwrap() {
        panic!("Hit request limit, wait until tomorrow");
    }
    let response: GameSchemaResponse = Client::new()
        .get(get_schema_for_game_request)
        .send()
        .await?
        .json()
        .await?;

    Ok(response.game.available_game_stats.map(|s| s.achievements).unwrap_or(vec![]))
}