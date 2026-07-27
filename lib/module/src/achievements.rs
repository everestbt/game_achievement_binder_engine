use crate::Module;

use std::collections::HashSet;
use steam_db::excluded_achievement_store;
use anyhow::Result;

pub fn get_excluded_achievements(module: &Module, game_id: &i32) -> Result<HashSet<String>> {
    match module {
        Module::STEAM(_, _) => {
            Ok(excluded_achievement_store::get_excluded_achievements_for_app(game_id)?
                .iter()
                .map(|e| e.achievement_name.clone())
                .collect()
            )
        }
        _ => todo!()
    }
}

pub fn save_excluded_achievement(module: &Module, game_id: &i32, achievement_name: &str) -> Result<()> {
    match module {
        Module::STEAM(_, _) => {
            excluded_achievement_store::save_excluded_achievement(achievement_name, game_id)?
        },
        _ => todo!()
    }
    Ok(())
}