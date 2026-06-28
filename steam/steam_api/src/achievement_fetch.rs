use serde::{Deserialize, Serialize};
use db::request_store;

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

pub async fn get_player_achievements(key : &str, steam_id : &str, app_id : &i32) -> Option<PlayerAchievements> {
    let get_player_achievements_request: String = "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/?".to_owned()
        + "&key=" + key + "&steamid=" + steam_id
        + "&appid=" + &app_id.to_string();

    if !request_store::increment().unwrap() {
        panic!("Hit request limit, wait until tomorrow");
    }
    let req: Result<reqwest::Response, reqwest::Error> = reqwest::Client::new()
        .get(get_player_achievements_request)
        .send()
        .await;

    if req.is_err() {
        panic!()
    }

    let response:Result<PlayerStatsResponse, reqwest::Error>  = req.unwrap().json().await;

    if response.is_err() {
        panic!()
    }

    let val = response.unwrap();
    // Success code can be true but no achievements present, typically for more modern games (Dota 2 is an example app_id 570)
    if val.playerstats.success && let Some(a) = val.playerstats.achievements && let Some(n) = val.playerstats.game_name {
        Option::Some(PlayerAchievements {
            achievements: a,
            game_name: n,
        })
    }
    else {
        Option::None
    }
}

pub async fn get_game_achievements(key : &str, app_id : &i32) -> Vec<GameAchievement> {
    let get_schema_for_game_request: String =
        "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/?key=".to_owned() + key + "&appid=" + &app_id.to_string();

    if !request_store::increment().unwrap() {
        panic!("Hit request limit, wait until tomorrow");
    }
    let req: Result<reqwest::Response, reqwest::Error> = reqwest::Client::new()
        .get(get_schema_for_game_request)
        .send()
        .await;

    if req.is_err() {
        panic!()
    }

    let response:Result<GameSchemaResponse, reqwest::Error>  = req.unwrap().json().await;

    if response.is_err() {
        panic!()
    }

    response.unwrap().game.available_game_stats.map(|s| s.achievements).unwrap_or(vec![])
}