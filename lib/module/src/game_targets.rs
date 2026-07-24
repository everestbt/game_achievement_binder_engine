use crate::Module;

use anyhow::Result;
use steam_db::game_target_store;

pub enum TargetStatus {
    Target,
    Complete,
}

pub fn get_game_target_status(module: &Module, game_id: &i32) -> Result<Option<TargetStatus>> {
    match module {
        &Module::STEAM(_, _) => {
            Ok(game_target_store::get_game_target(game_id)?.map(|t| steam_status_to_module_status(&t.complete)))
        },
        _ => todo!()
    }
}

pub struct GameTarget {
    pub module: Module,
    pub game_id: i32,
    pub status: TargetStatus,
}

pub fn get_game_targets(module: &Module) -> Result<Vec<GameTarget>> {
    match module {
        Module::STEAM(_, _) => {
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
        _ => todo!()
    }
}

fn steam_status_to_module_status(complete: &bool) -> TargetStatus {
    if *complete {
        TargetStatus::Complete
    }
    else {
        TargetStatus::Target
    }
}