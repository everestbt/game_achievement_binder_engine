use api::{achievement_fetch::{self, GameAchievement}, game_fetch, game_fetch::Game};
use db::{
    achievement_store, 
    excluded_achievement_store, 
    game_completion_cache, 
    game_completion_cache::GameCompletion,
    game_target_store
};

use std::{collections::{HashMap, HashSet}};
use rand::prelude::*;

pub async fn get_random_achievement_for_game(key : &str, steam_id : &str, game: &Game) -> Option<GameAchievement> {
    // Get the achievements for a specific game
        let achievements = achievement_fetch::get_player_achievements(key, steam_id, &game.appid).await;
        if let Some(a) = achievements {
            // Get details of the achievements
            let achievements: Vec<achievement_fetch::GameAchievement> = achievement_fetch::get_game_achievements(key, &game.appid).await;

            // Load currently listed achievements
            let current_goals_for_app: Vec<achievement_store::Achievement> = achievement_store::get_achievements_for_app(&game.appid).expect("Failed to load current goals");

            // Load excluded achievement
            let excluded_achievement_for_app: Vec<excluded_achievement_store::ExcludedAchievement> = excluded_achievement_store::get_excluded_achievements_for_app(&game.appid).expect("Failed to load excluded achievements");

            // Randomly select achievement from game
            let filter_to_unachieved: Vec<achievement_fetch::PlayerAchievement> = a.achievements
                .iter()
                .filter(|a| a.achieved == 0) // Filter out achieved
                .filter(|a| !current_goals_for_app.iter().any(|x| x.achievement_name == a.apiname)) // Filter out already in goals
                .filter(|a| !excluded_achievement_for_app.iter().any(|x| x.achievement_name == a.apiname)) // Filter out any excluded achievements
                .cloned()
                .collect();

            // Check there is something still in it
            if filter_to_unachieved.is_empty() {
                None
            }
            else {
                let mut rng = rand::rng();
                let random_achievement = filter_to_unachieved.choose(&mut rng).unwrap();
                Some(achievements
                    .iter()
                    .find(|a| a.name == random_achievement.apiname).cloned().unwrap())
            }
        }
        else {
            None
        }
}

async fn sync_completed_achievements(key : &str, steam_id : &str) {
    let mut achievements: Vec<achievement_store::Achievement> = achievement_store::get_achievements().expect("Failed to load achievements");
    achievements.sort_by(|a, b| i32::cmp(&a.app_id,&b.app_id));
    let mut app_player_achievement_map: HashMap<i32, achievement_fetch::PlayerAchievements> = HashMap::new();
    let owned_games: HashMap<i32, game_fetch::Game> = game_fetch::get_owned_games(key, steam_id).await.iter().map(|n| (n.appid, n.clone())).collect();
    for a in achievements {
        // Get the game out of the map
        let game = owned_games.get(&a.app_id).unwrap();
        // Check if the last_played has changed
        if game.last_played != a.last_played {
            // Check if the app is already loaded (PlayerAchievements)
            let player_achievements = app_player_achievement_map.get(&a.app_id);
            let loaded_player: &achievement_fetch::PlayerAchievements = if let Some(a) = player_achievements {
                a
            }
            else {
                let player = achievement_fetch::get_player_achievements(key, steam_id, &a.app_id).await.expect("Somehow a game with no achievements has ended up with one?!?");
                app_player_achievement_map.insert(a.app_id, player);
                app_player_achievement_map.get(&a.app_id).unwrap()
            };
            // Remove any that are already completed
            if loaded_player.achievements.iter().find(|x| x.apiname==a.achievement_name).unwrap().achieved == 1 {
                achievement_store::delete_achievement(&a.id).expect("Failed to delete achievement");
            }
            // Update last_played to avoid checking again
            else {
                achievement_store::update_last_played(&a.id, &game.last_played).expect("Failed to update the last played")
            }
        }
    }
}

async fn refresh_game_achievement_cache(key : &str, steam_id : &str) {
    let games = game_fetch::get_owned_games(key, steam_id).await;
    // Get cached completed games
    let cache_load = game_completion_cache::get_all_completions().expect("Failed to load completed games");
    let completed_games_cache: HashMap<i32, &GameCompletion> = cache_load
        .iter()
        .map(|n| (n.app_id, n))
        .collect();
    for game in games {
        // Check if cached and not played since
        let cache_check = completed_games_cache.get(&game.appid);
        if cache_check.is_some_and(|c| c.last_played == game.last_played) {
            continue;
        }
        // Get the achievements completed for that game
        // When not present, the game has no achievements, include with zeroes
        let player_achievements = achievement_fetch::get_player_achievements(key, steam_id, &game.appid).await;
        if let Some(achievements) = player_achievements.map(|p| p.achievements) {
            game_completion_cache::save_game_completion(
                &game.appid, 
                achievements.iter().filter(|a| a.achieved==1).count() as u32, 
                game.last_played, 
                achievements.len() as u32).expect("Failed to save game completion");
        }
        else {
            game_completion_cache::save_game_completion(
                &game.appid, 
                0, 
                game.last_played, 
                0).expect("Failed to save game completion");
        }
    }
}

// Check for any excluded achievements that are now completed and remove them
async fn sync_excluded_achievements(key : &str, steam_id : &str) {
    let mut game_map = HashMap::new(); 
    for e in excluded_achievement_store::get_excluded_achievements().expect("Failed to read excluded achievements") {
        let pa = if let Some(loaded) = game_map.get(&e.app_id) {
            loaded
        }
        else {
            let player_achievements = achievement_fetch::get_player_achievements(key, steam_id, &e.app_id).await.expect("Game should have achievements if exclusions exist");
            game_map.insert(e.app_id, player_achievements);
            game_map.get(&e.app_id).unwrap()
        };
        if pa.achievements.iter().find(|a| a.apiname == e.achievement_name && a.achieved == 1).is_some() {
            excluded_achievement_store::delete_excluded_achievement(&e.id).expect("Failed to delete excluded achievement")
        }
    }
}

pub async fn sync_caches(key : &str, steam_id : &str) {
    refresh_game_achievement_cache(key, steam_id).await;
    sync_completed_achievements(key, steam_id).await;
    sync_excluded_achievements(key, steam_id).await;
}

pub struct GameCompletionStatus {
    pub complete: bool,
    pub perfect: bool,
}

// Returns all completed and perfected games, if not returned then the game is not complete
pub fn get_game_completion() -> HashMap<i32, GameCompletionStatus> {
    let mut completion_map = HashMap::new();
    // Get all set targets
    let mut active_targets = HashSet::new();
    for t in game_target_store::get_game_targets().expect("Failed to read targets") {
        if t.complete {
            completion_map.insert(t.app_id, GameCompletionStatus { complete: true, perfect: false });
        }
        else {
            active_targets.insert(t.app_id);
        }
    }
    // Find any perfect games
    for p in game_completion_cache::get_perfect_games().expect("Failed to read completion cache") {
        if let Some(present) = completion_map.get_mut(&p.app_id) {
            present.perfect = true;
        }
        else {
            completion_map.insert(p.app_id, GameCompletionStatus { complete: !active_targets.contains(&p.app_id), perfect: true });
        }
    }
    // Check for any excluded achievements and whether this sets any games as completed
    let excluded_count = get_excluded_count();
    for e in excluded_count {
        let game_status = game_completion_cache::get_game_completion(&e.0).expect("Failed to read game completion");
        if let Some(progress) = game_status && progress.achievements_completed + e.1 == progress.achievement_count {
            if let Some(present) = completion_map.get_mut(&progress.app_id) {
                present.complete = true;
            }
            else {
                completion_map.insert(progress.app_id, GameCompletionStatus { complete: true, perfect: false });
            }
        }
    }

    completion_map
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

// Returns progress through games with achievements
pub fn get_game_progress() -> HashMap<i32, AchievementProgress> {
    let mut progress_map = HashMap::new();
    let excluded_count = get_excluded_count();
    for p in game_completion_cache::get_all_completions().expect("Failed to read completion cache") {
        if p.achievements_completed != 0 || excluded_count.contains_key(&p.app_id) {
            progress_map.insert(p.app_id, AchievementProgress { total: p.achievement_count, unlocked: p.achievements_completed, excluded: *excluded_count.get(&p.app_id).unwrap_or(&0) });
        }
        else if p.achievement_count > 0 {
            progress_map.insert(p.app_id, AchievementProgress { total: p.achievement_count, unlocked: 0, excluded: 0 });
        }
    }
    progress_map
}

fn get_excluded_count() -> HashMap<i32, u32> {
    let mut excluded_count: HashMap<i32, u32> = HashMap::new();
    for e in excluded_achievement_store::get_excluded_achievements().expect("Failed to read excluded achievements") {
        *excluded_count.entry(e.app_id).or_default() += 1;
    }
    excluded_count
}