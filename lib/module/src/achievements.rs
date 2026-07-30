use crate::Module;

use std::collections::HashSet;
use steam_db::{
    excluded_achievement_store,
    achievement_store,
};
use anyhow::Result;
use steam_utils::SteamAchievement;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum ModuleGoal {
    STEAM(SteamAchievement) 
}

pub fn save_achievement_goal(achievement: ModuleGoal) -> Result<()> {
    match achievement {
        ModuleGoal::STEAM(achievement) => {
            achievement_store::save_achievement(&achievement.achievement_name, &achievement.display_name, &achievement.description, &achievement.game_id, &achievement.last_played)?
        }
    }
    Ok(())
}

pub fn get_goals(module: &Module) -> Result<Vec<ModuleGoal>> {
    match module {
        Module::STEAM(_, _) => {
            Ok(achievement_store::get_achievements()?
                .iter()
                .map(|a| ModuleGoal::STEAM(SteamAchievement { 
                    achievement_name: a.achievement_name.clone(), 
                    display_name: a.display_name.clone(), 
                    description: a.description.clone(), 
                    game_id: a.app_id, 
                    last_played: a.last_played 
                }))
                .collect())
        }
    }
}

pub fn get_game_goals(module: &Module, game_id: &i32) -> Result<Vec<ModuleGoal>> {
    match module {
        Module::STEAM(_, _) => {
            Ok(achievement_store::get_achievements_for_app(game_id)?
                .iter()
                .map(|a| ModuleGoal::STEAM(SteamAchievement { 
                    achievement_name: a.achievement_name.clone(), 
                    display_name: a.display_name.clone(), 
                    description: a.description.clone(), 
                    game_id: a.app_id, 
                    last_played: a.last_played 
                }))
                .collect())
        }
    }
}

pub fn get_excluded_achievements(module: &Module, game_id: &i32) -> Result<HashSet<String>> {
    match module {
        Module::STEAM(_, _) => {
            Ok(excluded_achievement_store::get_excluded_achievements_for_app(game_id)?
                .iter()
                .map(|e| e.achievement_name.clone())
                .collect()
            )
        }
    }
}

pub fn save_excluded_achievement(module: &Module, game_id: &i32, achievement_name: &str) -> Result<()> {
    match module {
        Module::STEAM(_, _) => {
            excluded_achievement_store::save_excluded_achievement(achievement_name, game_id)?
        }
    }
    Ok(())
}