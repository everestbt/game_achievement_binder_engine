pub mod goals;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct SteamAchievement {
    pub game_id: i32, 
    pub display_name: String, 
    pub achievement_name: String, 
    pub description: Option<String>, 
    pub last_played: i64,
}