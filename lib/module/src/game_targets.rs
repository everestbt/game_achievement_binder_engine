use crate::{
    GameIdentifier, Module
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
            Ok(game_target_store::get_game_target(&game_identifier.id.expect("Steam must always have an ID"))?.map(|t| steam_status_to_module_status(&t.complete)))
        },
        Module::MTGA => Ok(magic_the_gathering_arena_utils::get_target_state()?.map(|t| mtga_status_to_module_status(t))),
    }
}

pub struct GameTarget {
    pub game_identifier: GameIdentifier,
    pub status: TargetStatus,
}

pub fn get_game_targets(module: &Module) -> Result<Vec<GameTarget>> {
    match module {
        Module::STEAM(_) => {
            Ok(game_target_store::get_game_targets()?
                .iter().map(|t| {
                    GameTarget {
                        game_identifier: GameIdentifier { module: module.clone(), id: Some(t.app_id) },
                        status: steam_status_to_module_status(&t.complete),
                    }
                })
                .collect())
        },
        Module::MTGA => {
            let game_identifier = GameIdentifier{ module: module.clone(), id: None};
            if let Some(status) = get_game_target_status(&game_identifier)? {
                Ok(vec![GameTarget {game_identifier, status}])
            }
            else {
                Ok(vec![])
            }
        },
    }
}

pub fn save_game_target(game_identifier: &GameIdentifier, status: TargetStatus) -> Result<()> {
    match game_identifier.module {
        Module::STEAM(_) => {
            game_target_store::save_game_target(&game_identifier.id.expect("Steam must always have an id"), match status {
                TargetStatus::Target => &false,
                TargetStatus::Complete => &true
            })?
        },
        Module::MTGA => {
            match status {
                TargetStatus::Target => magic_the_gathering_arena_utils::set_as_target()?,
                TargetStatus::Complete => magic_the_gathering_arena_utils::set_as_complete()?,
            }
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

fn mtga_status_to_module_status(target_state: magic_the_gathering_arena_utils::TargetState) -> TargetStatus {
    match target_state {
        magic_the_gathering_arena_utils::TargetState::COMPLETE => TargetStatus::Complete,
        magic_the_gathering_arena_utils::TargetState::TARGET => TargetStatus::Target,
    }
}