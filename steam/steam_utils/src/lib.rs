use jiff::Timestamp;

pub mod goals;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct SteamAchievement {
    pub game_id: i32, 
    pub display_name: String, 
    pub achievement_name: String, 
    pub description: Option<String>, 
    pub last_played: Timestamp,
}

pub fn last_played_converter_to_timestamp(last_played: i64) -> Timestamp {
    Timestamp::from_second(last_played).expect("Failed to convert seconds")
}

pub fn last_played_converter_to_seconds(timestamp: Timestamp) -> i64 {
    timestamp.as_second()
}