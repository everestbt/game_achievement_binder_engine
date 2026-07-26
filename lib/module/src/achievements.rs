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