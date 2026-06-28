use serde::{Deserialize, Serialize};
use db::request_store;

// Owned Games Request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub appid: i32,
    pub name: String,
    pub playtime_forever: i32, // This is the number of minutes played
    #[serde(rename = "rtime_last_played")]
    pub last_played: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OwnedGames {
    game_count: i32,
    games: Vec<Game>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SteamOwnedGamesResponse {
    response: OwnedGames,
}

/// Fetch games for steamid
pub async fn get_owned_games(key : &str, steam_id : &str) -> Vec<Game> {

    let get_owned_games_request: String =
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?format=json&include_appinfo=true&include_played_free_games=true".to_owned() + "&key=" + key + "&steamid=" + steam_id;
    if !request_store::increment().unwrap() {
        panic!("Hit request limit, wait until tomorrow");
    }
    let req: Result<reqwest::Response, reqwest::Error> = reqwest::Client::new()
        .get(get_owned_games_request)
        .send()
        .await;

    if req.is_err() {
        panic!()
    }
    let response: Result<SteamOwnedGamesResponse, reqwest::Error> = req.unwrap()
        .json()
        .await;

    if response.is_err() {
        panic!()
    }

    response.unwrap().response.games
}