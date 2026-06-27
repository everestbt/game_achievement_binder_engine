use api::game_fetch;
use chrono::{DateTime, NaiveDate};

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