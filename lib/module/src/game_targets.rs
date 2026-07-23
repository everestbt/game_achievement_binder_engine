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
            Ok(game_target_store::get_game_target(game_id)?.map(|t| {
                if t.complete {
                    TargetStatus::Complete
                }
                else {
                    TargetStatus::Target
                }
            }))
        },
        _ => todo!()
    }
}