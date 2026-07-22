use crate::Module;

use anyhow::Result;
use steam_db::game_cover_store;

pub fn save_game_cover(module: Module, game_id: &i32, cover_url: &str) -> Result<()> {
    match module {
        Module::STEAM(_, _) => game_cover_store::save_game_cover(cover_url, game_id)?,
        _ => todo!()
    }
    Ok(())
}

pub fn get_game_cover_url(module: Module, game_id: &i32) -> Result<Option<String>> {
    match module {
        Module::STEAM(_, _) => Ok(game_cover_store::get_game_cover(game_id)?.map(|c| c.url)),
        _ => todo!()
    }
    
}