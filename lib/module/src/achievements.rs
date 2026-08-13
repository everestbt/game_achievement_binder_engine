use crate::{GameIdentifier, Module};

use std::collections::HashSet;
use steam_db::{
    excluded_achievement_store,
    achievement_store,
};
use anyhow::Result;
use steam_utils::{
    SteamAchievement,
    last_played_converter_to_timestamp,
    last_played_converter_to_seconds,
};

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum ModuleGoal {
    STEAM(SteamAchievement) 
}

pub fn save_achievement_goal(achievement: ModuleGoal) -> Result<()> {
    match achievement {
        ModuleGoal::STEAM(achievement) => {
            achievement_store::save_achievement(&achievement.achievement_name, &achievement.display_name, &achievement.description, &achievement.game_id, &last_played_converter_to_seconds(achievement.last_played))?
        }
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
        }
    }
}

pub fn save_excluded_achievement(game_identifier: &GameIdentifier, achievement_name: &str) -> Result<()> {
    match game_identifier.module {
        Module::STEAM(_) => {
            excluded_achievement_store::save_excluded_achievement(achievement_name, &game_identifier.id)?
        }
    }
    Ok(())
}