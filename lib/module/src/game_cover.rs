use crate::{
    Module,
    GameIdentifier,
};

use anyhow::Result;
use steam_db::game_cover_store;
use bytes::Bytes;
use steam_api::game_cover_fetch;

pub fn load_game_cover(game_identifier: &GameIdentifier) -> Result<Bytes> {
    match game_identifier.module {
        Module::STEAM(_) => {
            game_cover_fetch::get_game_cover_blocking(&game_identifier.id)
        }
    }
}

pub fn save_game_cover(game_identifier: &GameIdentifier, cover_url: &str) -> Result<()> {
    match game_identifier.module {
        Module::STEAM(_) => game_cover_store::save_game_cover(cover_url, &game_identifier.id)?,
    }
    Ok(())
}

pub fn get_game_cover_url(game_identifier: &GameIdentifier) -> Result<Option<String>> {
    match game_identifier.module {
        Module::STEAM(_) => Ok(game_cover_store::get_game_cover(&game_identifier.id)?.map(|c| c.url)),
    }
    
}