use crate::{
    GameIdentifier, 
    Module
};

use anyhow::Result;
use steam_db::game_target_store;

pub enum TargetStatus {
    Target,
    Complete,
}

pub fn get_game_target_status(game_identifier: &GameIdentifier) -> Result<Option<TargetStatus>> {
    match game_identifier.module {
        Module::STEAM(_) => {
            Ok(game_target_store::get_game_target(&game_identifier.id)?.map(|t| steam_status_to_module_status(&t.complete)))
        },
    }
}

pub struct GameTarget {
    pub module: Module,
    pub game_id: i32,
    pub status: TargetStatus,
}

pub fn get_game_targets(module: &Module) -> Result<Vec<GameTarget>> {
    match module {
        Module::STEAM(_) => {
            Ok(game_target_store::get_game_targets()?
                .iter().map(|t| {
                    GameTarget {
                        module: module.clone(),
                        game_id: t.app_id,
                        status: steam_status_to_module_status(&t.complete),
                    }
                })
                .collect())
        },
    }
}

pub fn save_game_target(game_identifier: &GameIdentifier, status: TargetStatus) -> Result<()> {
    match game_identifier.module {
        Module::STEAM(_) => {
            game_target_store::save_game_target(&game_identifier.id, match status {
                TargetStatus::Target => &false,
                TargetStatus::Complete => &true
            })?
        },
    }
    Ok(())
}

fn steam_status_to_module_status(complete: &bool) -> TargetStatus {
    if *complete {
        TargetStatus::Complete
    }
    else {
        TargetStatus::Target
    }
}