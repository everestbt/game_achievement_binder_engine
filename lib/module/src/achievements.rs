use crate::{GameIdentifier, Module};

use std::collections::HashSet;
use steam_api::achievement_fetch;
use steam_db::{
    excluded_achievement_store,
    achievement_store,
};
use anyhow::Result;
use steam_utils::{
    goals,
    SteamAchievement,
    last_played_converter_to_timestamp,
    last_played_converter_to_seconds,
};
use magic_the_gathering_arena_utils::{
    self,
    MTGAAchievement,
};
use rand::prelude::*;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum ModuleGoal {
    STEAM(SteamAchievement),
    MTGA(MTGAAchievement)
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

pub async fn get_game_achievements(game_identifier: &GameIdentifier) -> Vec<GameAchievement> {
    match game_identifier.module.clone() {
        Module::STEAM(credentials) => {
            let achieved_set: HashSet<String> = achievement_fetch::get_player_achievements(&credentials.key, &credentials.steam_id, &game_identifier.id).await.expect("Failed to load").iter()
                    .filter(|a| a.achieved)
                    .map(|a| a.name.clone())
                    .collect();
            achievement_fetch::get_game_achievements(&credentials.key, &game_identifier.id).await.expect("Failed to load")
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
        Module::MTGA => {
            magic_the_gathering_arena_utils::get_achievements().expect("Failed to read MTGA achievements")
                .iter()
                .map(|g| GameAchievement {
                    id: g.name.clone(),
                    display_name: g.name.clone(),
                    description: None,
                    achieved: g.achieved,
                    achieved_icon_id: None,
                    unachieved_icon_id: None
                })
                .collect()
        }
    }
}

pub fn save_achievement_goal(achievement: ModuleGoal) -> Result<()> {
    match achievement {
        ModuleGoal::STEAM(achievement) => {
            achievement_store::save_achievement(&achievement.achievement_name, &achievement.display_name, &achievement.description, &achievement.game_id, &last_played_converter_to_seconds(achievement.last_played))?
        },
        ModuleGoal::MTGA(achievement) => {
            magic_the_gathering_arena_utils::save_goal(&achievement.name)?
        },
    }
    Ok(())
}

pub fn get_goals(module: &Module) -> Result<Vec<ModuleGoal>> {
    match module {
        Module::STEAM(_) => {
            Ok(achievement_store::get_achievements()?
                .iter()
                .map(|a| ModuleGoal::STEAM(SteamAchievement { 
                    achievement_name: a.achievement_name.clone(), 
                    display_name: a.display_name.clone(), 
                    description: a.description.clone(), 
                    game_id: a.app_id, 
                    last_played: last_played_converter_to_timestamp(a.last_played)
                }))
                .collect())
        },
        Module::MTGA => {
            Ok(magic_the_gathering_arena_utils::get_goals()?
                .iter()
                .map(|a| ModuleGoal::MTGA(a.clone()))
                .collect())
        }
    }
}

pub fn get_game_goals(game_identifier: &GameIdentifier) -> Result<Vec<ModuleGoal>> {
    match game_identifier.module {
        Module::STEAM(_) => {
            Ok(achievement_store::get_achievements_for_app(&game_identifier.id)?
                .iter()
                .map(|a| ModuleGoal::STEAM(SteamAchievement { 
                    achievement_name: a.achievement_name.clone(), 
                    display_name: a.display_name.clone(), 
                    description: a.description.clone(), 
                    game_id: a.app_id, 
                    last_played: last_played_converter_to_timestamp(a.last_played)
                }))
                .collect())
        },
        Module::MTGA => {
            Ok(magic_the_gathering_arena_utils::get_goals()?
                .iter()
                .map(|a| ModuleGoal::MTGA(a.clone()))
                .collect())
        }
    }
}

pub fn get_excluded_achievements(game_identifier: &GameIdentifier) -> Result<HashSet<String>> {
    match game_identifier.module {
        Module::STEAM(_) => {
            Ok(excluded_achievement_store::get_excluded_achievements_for_app(&game_identifier.id)?
                .iter()
                .map(|e| e.achievement_name.clone())
                .collect()
            )
        },
        Module::MTGA => {
            Ok(magic_the_gathering_arena_utils::get_excluded_achievements()?
                .iter()
                .map(|a| a.name.clone())
                .collect())
        }
    }
}

pub fn save_excluded_achievement(game_identifier: &GameIdentifier, achievement_name: &str) -> Result<()> {
    match game_identifier.module {
        Module::STEAM(_) => {
            excluded_achievement_store::save_excluded_achievement(achievement_name, &game_identifier.id)?
        },
        Module::MTGA => {
            magic_the_gathering_arena_utils::save_excluded_achievement(achievement_name)?
        },
    }
    Ok(())
}

pub async fn get_random_achievement_for_game(game_identifier: GameIdentifier) -> Option<GameAchievement> {
    match game_identifier.module.clone() {
        Module::STEAM(credentials) => {
            goals::get_random_achievement_for_game(&credentials.key, &credentials.steam_id, &game_identifier.id)
                .await.map(|g| GameAchievement { id: g.name, display_name: g.display_name, description: g.description, achieved: false, achieved_icon_id: Some(g.icon), unachieved_icon_id: Some(g.icongray) })
        },
        Module::MTGA => {
            let mut rng = rand::rng();
            magic_the_gathering_arena_utils::get_achievements().expect("Failed to load MTGA achievements")
                .iter()
                .filter(|a| !a.achieved)
                .choose(&mut rng)
                .map(|a| GameAchievement { 
                    id: a.name.clone(), 
                    display_name: a.name.clone(), 
                    description: None, 
                    achieved: a.achieved, 
                    achieved_icon_id: None, 
                    unachieved_icon_id: None,
                })
        }
    }
}